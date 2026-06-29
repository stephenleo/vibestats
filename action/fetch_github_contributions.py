"""fetch_github_contributions.py — Snapshot GitHub contribution counts to vibestats-data.

Runs daily in the GitHub Action (issue #117). Fetches the authenticated user's
daily contribution counts — INCLUDING private contributions, since GraphQL
`viewer.contributionsCollection` returns your own private counts — and merges
them into ``github/contributions.json`` with a monotonic max-merge so a stored
count can only ever rise. That makes the snapshot survive GitHub's retroactive
removal of private contributions when you lose access to an org.

Running here (not on each machine) means contributions refresh daily regardless
of AI-tool usage, and the GitHub API is called once per day instead of redundantly
from every machine.

Privacy: every query requests only counts and dates — never repository names,
titles, or any other identifying field. Stdlib only (urllib) — no third-party
packages, consistent with the rest of ``action/``.
"""

import datetime
import json
import os
import pathlib
import sys
import urllib.request

# Endpoint, snapshot location (relative to the vibestats-data checkout root),
# and the GraphQL point budget below which backfill pauses for the day.
GRAPHQL_URL = "https://api.github.com/graphql"
CONTRIB_PATH = "github/contributions.json"
RATE_LIMIT_SAFETY = 200
TIMEOUT_SECONDS = 30

# Typed contribution streams: (GraphQL field, snapshot key). The contribution
# calendar gives the authoritative daily `total`; these give the per-type split.
STREAMS = [
    ("issueContributions", "issues"),
    ("pullRequestContributions", "prs"),
    ("pullRequestReviewContributions", "reviews"),
]
COUNT_KEYS = ["total", "commits", "prs", "issues", "reviews"]


# ─── GraphQL transport ─────────────────────────────────────────────────────────

def _graphql(token: str, query: str) -> dict:
    """POST a GraphQL query and return the parsed JSON response.

    Args:
        token (str): GitHub token authorized to read the user's contributions.
        query (str): GraphQL query string.

    Returns:
        dict: The full parsed response, including any `data` and `errors` keys.
    """
    body = json.dumps({"query": query}).encode("utf-8")
    req = urllib.request.Request(
        GRAPHQL_URL,
        data=body,
        method="POST",
        headers={
            "Authorization": f"Bearer {token}",
            "User-Agent": "vibestats",
            "Content-Type": "application/json",
            "Accept": "application/vnd.github+json",
        },
    )
    with urllib.request.urlopen(req, timeout=TIMEOUT_SECONDS) as resp:  # noqa: S310 (fixed https URL)
        return json.loads(resp.read().decode("utf-8"))


# ─── Query builders (privacy: counts/dates only — no repository fields) ─────────

def _year_window(year: int) -> tuple[str, str]:
    """Return the (from, to) ISO-8601 window covering an entire calendar year.

    Args:
        year (int): Four-digit year.

    Returns:
        tuple[str, str]: (from, to) ISO-8601 timestamps.
    """
    return f"{year}-01-01T00:00:00Z", f"{year}-12-31T23:59:59Z"


def calendar_commits_query(year: int) -> str:
    """Build the query for one year's contribution calendar and commit counts.

    Args:
        year (int): Four-digit year.

    Returns:
        str: GraphQL query requesting only dates and counts (no repo fields).
    """
    frm, to = _year_window(year)
    return (
        "query { rateLimit { remaining resetAt } viewer { "
        f'contributionsCollection(from: "{frm}", to: "{to}") {{ '
        "contributionCalendar { weeks { contributionDays { date contributionCount } } } "
        "commitContributionsByRepository(maxRepositories: 100) { "
        "contributions(first: 100) { nodes { occurredAt commitCount } } } "
        "} } }"
    )


def flat_query(year: int, field: str, after: str) -> str:
    """Build the query for one paginated typed contribution stream.

    Args:
        year (int): Four-digit year.
        field (str): The contributionsCollection connection (e.g. issueContributions).
        after (str): The literal `null` or a quoted GraphQL cursor.

    Returns:
        str: GraphQL query requesting only `occurredAt` per node (no repo fields).
    """
    frm, to = _year_window(year)
    return (
        "query { viewer { "
        f'contributionsCollection(from: "{frm}", to: "{to}") {{ '
        f"{field}(first: 100, after: {after}) {{ "
        "nodes { occurredAt } pageInfo { hasNextPage endCursor } } "
        "} } }"
    )


# ─── Response parsing (pure, network-free, unit-tested) ─────────────────────────

def _empty_day() -> dict:
    """Return a zeroed per-day count dict.

    Returns:
        dict: {total, commits, prs, issues, reviews} all 0.
    """
    return {k: 0 for k in COUNT_KEYS}


def _day_of(ts: str) -> str | None:
    """Extract the YYYY-MM-DD date prefix from an ISO-8601 timestamp.

    Args:
        ts (str): ISO-8601 timestamp.

    Returns:
        str | None: The date prefix, or None if malformed.
    """
    if isinstance(ts, str) and len(ts) >= 10:
        return ts[:10]
    return None


def apply_calendar(cc: dict, out: dict) -> None:
    """Fold the calendar's authoritative daily totals into `out`.

    Args:
        cc (dict): The contributionsCollection object.
        out (dict): Accumulator mapping date -> count dict (mutated in place).

    Returns:
        None.
    """
    weeks = ((cc.get("contributionCalendar") or {}).get("weeks")) or []
    for week in weeks:
        for day in week.get("contributionDays") or []:
            date = day.get("date")
            count = day.get("contributionCount")
            if isinstance(date, str) and isinstance(count, int):
                out.setdefault(date, _empty_day())["total"] = count


def apply_commits(cc: dict, out: dict) -> None:
    """Sum per-repository commit counts into per-day totals in `out`.

    Repo names are never read — only `occurredAt` + `commitCount`.

    Args:
        cc (dict): The contributionsCollection object.
        out (dict): Accumulator mapping date -> count dict (mutated in place).

    Returns:
        None.
    """
    for repo in cc.get("commitContributionsByRepository") or []:
        for node in (repo.get("contributions") or {}).get("nodes") or []:
            date = _day_of(node.get("occurredAt"))
            count = node.get("commitCount")
            if date is not None and isinstance(count, int):
                out.setdefault(date, _empty_day())["commits"] += count


def apply_flat(conn: dict, key: str, out: dict) -> None:
    """Count one event per node into `out[date][key]`, bucketed by UTC day.

    Args:
        conn (dict): A typed contribution connection (nodes + pageInfo).
        key (str): The count dict key to increment (e.g. "prs").
        out (dict): Accumulator mapping date -> count dict (mutated in place).

    Returns:
        None.
    """
    for node in conn.get("nodes") or []:
        date = _day_of(node.get("occurredAt"))
        if date is not None:
            out.setdefault(date, _empty_day())[key] += 1


def max_merge(stored: dict, fetched: dict) -> None:
    """Merge `fetched` into `stored` so each per-type count can only ever rise.

    All-zero fetched days are skipped (keeps the file lean); existing stored days
    are never removed — this is what makes org-departure non-destructive.

    Args:
        stored (dict): The persistent days map (mutated in place).
        fetched (dict): Freshly fetched days map.

    Returns:
        None.
    """
    for date, fc in fetched.items():
        if all(fc.get(k, 0) == 0 for k in COUNT_KEYS):
            continue
        cur = stored.setdefault(date, _empty_day())
        for k in COUNT_KEYS:
            cur[k] = max(cur.get(k, 0), fc.get(k, 0))


# ─── Fetch orchestration ────────────────────────────────────────────────────────

def fetch_year(graphql_fn, year: int, out: dict) -> int:
    """Fetch one year's counts into `out`; return the GraphQL points remaining.

    Args:
        graphql_fn (Callable[[str], dict]): Executes a query and returns the response.
        year (int): Four-digit year.
        out (dict): Accumulator mapping date -> count dict (mutated in place).

    Returns:
        int: GraphQL points remaining (for backfill pacing).

    Raises:
        RuntimeError: If the response lacks a viewer/contributionsCollection
            (e.g. insufficient token scope), so the caller can skip without writing.
    """
    resp = graphql_fn(calendar_commits_query(year))
    data = resp.get("data") or {}
    cc = (data.get("viewer") or {}).get("contributionsCollection")
    if cc is None:
        raise RuntimeError(f"null contributionsCollection (token scope?): {resp.get('errors')}")
    apply_calendar(cc, out)
    apply_commits(cc, out)
    remaining = (data.get("rateLimit") or {}).get("remaining")
    if not isinstance(remaining, int):
        remaining = 10 ** 9

    for field, key in STREAMS:
        after = "null"
        while True:
            resp = graphql_fn(flat_query(year, field, after))
            conn = ((resp.get("data") or {}).get("viewer") or {}).get("contributionsCollection")
            conn = (conn or {}).get(field)
            if conn is None:
                raise RuntimeError(f"null {field} (token scope?): {resp.get('errors')}")
            apply_flat(conn, key, out)
            page = conn.get("pageInfo") or {}
            if page.get("hasNextPage") and page.get("endCursor"):
                after = '"%s"' % page["endCursor"]
            else:
                break
    return remaining


def _created_year(graphql_fn) -> int | None:
    """Return the authenticated user's account-creation year, or None on failure.

    Args:
        graphql_fn (Callable[[str], dict]): Executes a query and returns the response.

    Returns:
        int | None: The account-creation year, or None if it can't be read.
    """
    resp = graphql_fn("query { viewer { createdAt } }")
    created = ((resp.get("data") or {}).get("viewer") or {}).get("createdAt")
    if isinstance(created, str) and len(created) >= 4 and created[:4].isdigit():
        return int(created[:4])
    return None


def sync(graphql_fn, root: str) -> bool:
    """Refresh the snapshot: current year + progressive newest-first backfill.

    Always refreshes the current year, then walks older years down to the
    account-creation year, resuming from `backfilled_to_year` and pausing when
    the GraphQL point budget runs low. Writes the snapshot only when the data
    actually changed (so unchanged days produce no commit).

    Args:
        graphql_fn (Callable[[str], dict]): Executes a query and returns the response.
        root (str): The vibestats-data checkout root.

    Returns:
        bool: True if the snapshot file was written, False if unchanged.
    """
    today = datetime.datetime.now(datetime.timezone.utc).date()
    current_year = today.year
    path = pathlib.Path(root) / CONTRIB_PATH
    snapshot = _load_snapshot(path)
    before = (json.dumps(snapshot["days"], sort_keys=True), snapshot.get("backfilled_to_year"))

    fetched: dict = {}
    # Always refresh the current year. A failure here propagates so we never
    # overwrite an existing snapshot with nothing.
    fetch_year(graphql_fn, current_year, fetched)

    # Progressive backfill, newest-first, paced against the GraphQL budget.
    created_year = None
    try:
        created_year = _created_year(graphql_fn)
    except Exception as exc:  # noqa: BLE001 (backfill is best-effort)
        print(f"fetch_github_contributions: could not read createdAt, skipping backfill: {exc}",
              file=sys.stderr)
    if created_year is not None:
        done_to = snapshot.get("backfilled_to_year")
        start = (done_to - 1) if isinstance(done_to, int) else (current_year - 1)
        for year in range(start, created_year - 1, -1):
            try:
                remaining = fetch_year(graphql_fn, year, fetched)
            except Exception as exc:  # noqa: BLE001
                print(f"fetch_github_contributions: backfill stopped at {year}: {exc}",
                      file=sys.stderr)
                break
            snapshot["backfilled_to_year"] = year
            if remaining < RATE_LIMIT_SAFETY:
                print(f"fetch_github_contributions: GraphQL budget low ({remaining}); "
                      f"paused backfill at {year} — next daily run continues", file=sys.stderr)
                break

    max_merge(snapshot["days"], fetched)

    after = (json.dumps(snapshot["days"], sort_keys=True), snapshot.get("backfilled_to_year"))
    if after == before:
        return False
    snapshot["last_updated"] = today.isoformat()
    _write_snapshot(path, snapshot)
    return True


def _load_snapshot(path: pathlib.Path) -> dict:
    """Load the snapshot, returning a default skeleton if absent or malformed.

    Args:
        path (pathlib.Path): Path to github/contributions.json.

    Returns:
        dict: {last_updated, backfilled_to_year, days} with `days` a dict.
    """
    if path.exists():
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
            if isinstance(data, dict):
                if not isinstance(data.get("days"), dict):
                    data["days"] = {}
                return data
        except (OSError, json.JSONDecodeError):
            pass
    return {"last_updated": "", "backfilled_to_year": None, "days": {}}


def _write_snapshot(path: pathlib.Path, snapshot: dict) -> None:
    """Write the snapshot deterministically (sorted keys) for stable diffs.

    Args:
        path (pathlib.Path): Path to github/contributions.json.
        snapshot (dict): The snapshot to serialize.

    Returns:
        None.
    """
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(snapshot, indent=2, sort_keys=True), encoding="utf-8")


def main() -> None:
    """Entry point: fetch + snapshot if a token is configured, else no-op.

    Non-fatal by contract — never fails the daily Action over this optional
    feature; any error is logged and swallowed.

    Returns:
        None.
    """
    token = os.environ.get("VIBESTATS_GH_TOKEN", "").strip()
    if not token:
        print("fetch_github_contributions: no VIBESTATS_GH_TOKEN — skipping (feature disabled)")
        return
    try:
        changed = sync(lambda query: _graphql(token, query), ".")
        print(f"fetch_github_contributions: snapshot {'updated' if changed else 'unchanged'}")
    except Exception as exc:  # noqa: BLE001 (optional feature must never break the Action)
        print(f"fetch_github_contributions: skipped ({exc})", file=sys.stderr)


if __name__ == "__main__":
    main()
