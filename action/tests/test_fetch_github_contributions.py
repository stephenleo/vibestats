"""test_fetch_github_contributions.py — Unit tests for fetch_github_contributions.py.

GH Issue: #117. Covers the pure parsers, the monotonic max-merge, the privacy
guarantee (no repo fields in any query), and the sync orchestration (current-year
refresh + progressive newest-first backfill with rate-limit pacing + resume),
all without touching the network via an injected fake GraphQL callable.
"""

import datetime
import importlib.util
import json
import pathlib
import re
import tempfile
import unittest


def _import_module():
    """Dynamically import fetch_github_contributions from action/."""
    path = pathlib.Path(__file__).parent.parent / "fetch_github_contributions.py"
    spec = importlib.util.spec_from_file_location("fetch_github_contributions", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


M = _import_module()
CURRENT_YEAR = datetime.datetime.now(datetime.timezone.utc).year


class FakeGitHub:
    """A scripted GraphQL responder keyed by year and query type — no network."""

    def __init__(self, calendars=None, commits=None, flats=None,
                 created_year=CURRENT_YEAR, remaining=5000):
        self.calendars = calendars or {}        # {year: {date: total}}
        self.commits = commits or {}            # {year: {date: commitCount}}
        self.flats = flats or {}                # {(year, field): [occurredAt, ...]}
        self.created_year = created_year
        self.remaining = remaining

    def __call__(self, query):
        if "createdAt" in query:
            return {"data": {"viewer": {"createdAt": f"{self.created_year}-01-01T00:00:00Z"}}}
        year = int(re.search(r'from: "(\d{4})', query).group(1))
        if "contributionCalendar" in query:
            weeks = [{"contributionDays": [
                {"date": d, "contributionCount": c}
                for d, c in self.calendars.get(year, {}).items()
            ]}]
            repos = [{"contributions": {"nodes": [
                {"occurredAt": f"{d}T10:00:00Z", "commitCount": c}
                for d, c in self.commits.get(year, {}).items()
            ]}}]
            return {"data": {
                "rateLimit": {"remaining": self.remaining},
                "viewer": {"contributionsCollection": {
                    "contributionCalendar": {"weeks": weeks},
                    "commitContributionsByRepository": repos,
                }},
            }}
        field = next(f for f in ("issueContributions", "pullRequestContributions",
                                 "pullRequestReviewContributions") if f in query)
        nodes = [{"occurredAt": ts} for ts in self.flats.get((year, field), [])]
        return {"data": {"viewer": {"contributionsCollection": {
            field: {"nodes": nodes, "pageInfo": {"hasNextPage": False, "endCursor": None}},
        }}}}


class TestMaxMerge(unittest.TestCase):
    def test_never_lowers_stored(self):
        """Org-departure case: a re-fetch returns fewer than stored — keep stored."""
        stored = {"2025-10-14": {"total": 23, "commits": 20, "prs": 1, "issues": 1, "reviews": 1}}
        M.max_merge(stored, {"2025-10-14": M._empty_day()})
        self.assertEqual(stored["2025-10-14"]["total"], 23)
        self.assertEqual(stored["2025-10-14"]["commits"], 20)

    def test_raises_and_adds(self):
        stored = {"2026-01-01": {"total": 5, "commits": 5, "prs": 0, "issues": 0, "reviews": 0}}
        fetched = {
            "2026-01-01": {"total": 9, "commits": 5, "prs": 2, "issues": 0, "reviews": 2},
            "2026-01-02": {"total": 3, "commits": 1, "prs": 1, "issues": 1, "reviews": 0},
        }
        M.max_merge(stored, fetched)
        self.assertEqual(stored["2026-01-01"], {"total": 9, "commits": 5, "prs": 2, "issues": 0, "reviews": 2})
        self.assertEqual(stored["2026-01-02"]["total"], 3)

    def test_skips_all_zero_days(self):
        stored = {}
        M.max_merge(stored, {"2026-01-03": M._empty_day()})
        self.assertEqual(stored, {})


class TestParsers(unittest.TestCase):
    def test_apply_calendar_reads_total(self):
        cc = {"contributionCalendar": {"weeks": [{"contributionDays": [
            {"date": "2026-06-28", "contributionCount": 12},
        ]}]}}
        out = {}
        M.apply_calendar(cc, out)
        self.assertEqual(out["2026-06-28"]["total"], 12)

    def test_apply_commits_sums_across_repos(self):
        cc = {"commitContributionsByRepository": [
            {"contributions": {"nodes": [{"occurredAt": "2026-06-28T10:00:00Z", "commitCount": 3}]}},
            {"contributions": {"nodes": [{"occurredAt": "2026-06-28T15:00:00Z", "commitCount": 2}]}},
        ]}
        out = {}
        M.apply_commits(cc, out)
        self.assertEqual(out["2026-06-28"]["commits"], 5)

    def test_apply_flat_counts_one_per_event(self):
        conn = {"nodes": [{"occurredAt": "2026-06-28T10:00:00Z"},
                          {"occurredAt": "2026-06-28T12:00:00Z"},
                          {"occurredAt": "2026-06-27T09:00:00Z"}]}
        out = {}
        M.apply_flat(conn, "prs", out)
        self.assertEqual(out["2026-06-28"]["prs"], 2)
        self.assertEqual(out["2026-06-27"]["prs"], 1)


class TestPrivacy(unittest.TestCase):
    def test_no_repo_fields_in_queries(self):
        """Privacy guard: no query may request a repo name or identifying field."""
        forbidden = ["repository", "nameWithOwner", "name", "title", "url", "login"]
        queries = [
            M.calendar_commits_query(2026),
            M.flat_query(2026, "issueContributions", "null"),
            M.flat_query(2026, "pullRequestReviewContributions", '"cursor"'),
        ]
        for q in queries:
            for word in forbidden:
                self.assertNotIn(word, q, f"query must not request '{word}': {q}")


class TestSync(unittest.TestCase):
    def _run(self, fake, seed=None):
        with tempfile.TemporaryDirectory() as tmp:
            root = pathlib.Path(tmp)
            if seed is not None:
                (root / "github").mkdir(parents=True)
                (root / "github" / "contributions.json").write_text(json.dumps(seed))
            changed = M.sync(fake, str(root))
            path = root / "github" / "contributions.json"
            data = json.loads(path.read_text()) if path.exists() else None
            return changed, data

    def test_writes_current_year(self):
        fake = FakeGitHub(
            calendars={CURRENT_YEAR: {f"{CURRENT_YEAR}-06-28": 12}},
            commits={CURRENT_YEAR: {f"{CURRENT_YEAR}-06-28": 5}},
            flats={(CURRENT_YEAR, "pullRequestContributions"): [f"{CURRENT_YEAR}-06-28T09:00:00Z"]},
            created_year=CURRENT_YEAR,
        )
        changed, data = self._run(fake)
        self.assertTrue(changed)
        day = data["days"][f"{CURRENT_YEAR}-06-28"]
        self.assertEqual(day["total"], 12)
        self.assertEqual(day["commits"], 5)
        self.assertEqual(day["prs"], 1)

    def test_backfills_down_to_created_year(self):
        cy = CURRENT_YEAR
        fake = FakeGitHub(
            calendars={cy: {f"{cy}-01-01": 1}, cy - 1: {f"{cy - 1}-06-01": 2}, cy - 2: {f"{cy - 2}-03-01": 3}},
            created_year=cy - 2,
        )
        _, data = self._run(fake)
        self.assertEqual(data["backfilled_to_year"], cy - 2)
        self.assertIn(f"{cy - 2}-03-01", data["days"])

    def test_pauses_on_low_budget(self):
        cy = CURRENT_YEAR
        fake = FakeGitHub(
            calendars={cy: {f"{cy}-01-01": 1}, cy - 1: {f"{cy - 1}-01-01": 1}, cy - 2: {f"{cy - 2}-01-01": 1}},
            created_year=cy - 2,
            remaining=100,  # below RATE_LIMIT_SAFETY (200)
        )
        _, data = self._run(fake)
        # Only the first backfill year (cy-1) completes before the budget pause.
        self.assertEqual(data["backfilled_to_year"], cy - 1)

    def test_resumes_from_marker(self):
        cy = CURRENT_YEAR
        fake = FakeGitHub(
            calendars={cy: {}, cy - 1: {}, cy - 2: {f"{cy - 2}-01-01": 1}},
            created_year=cy - 2,
        )
        seed = {"last_updated": "", "backfilled_to_year": cy - 1, "days": {}}
        _, data = self._run(fake, seed=seed)
        # Resumes below the marker and finishes at the creation year.
        self.assertEqual(data["backfilled_to_year"], cy - 2)

    def test_unchanged_run_is_noop(self):
        fake = FakeGitHub(calendars={CURRENT_YEAR: {f"{CURRENT_YEAR}-06-28": 12}}, created_year=CURRENT_YEAR)
        with tempfile.TemporaryDirectory() as tmp:
            root = str(tmp)
            self.assertTrue(M.sync(fake, root))      # first run writes
            self.assertFalse(M.sync(fake, root))     # second run: identical data → no write


if __name__ == "__main__":
    unittest.main()
