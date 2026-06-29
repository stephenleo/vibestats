//! GitHub contributions fetch + persistence for vibestats.
//!
//! Fetches the authenticated user's daily contribution counts — INCLUDING
//! private contributions, because `viewer` returns your own private counts with
//! any valid token — via the GitHub GraphQL API, and snapshots them into
//! `github/contributions.json` in the private vibestats-data repo.
//!
//! # Why snapshot
//! GitHub retroactively REMOVES private contributions from your graph when you
//! lose access to an org/repo. The max-merge here makes every per-day, per-type
//! count monotonic: a later fetch can only ever RAISE a stored count, so org
//! departure can never erase history already captured (the core of issue #117).
//!
//! # Privacy (NFR8/NFR9)
//! The GraphQL queries request ONLY counts and dates (`contributionCount`,
//! `commitCount`, `occurredAt`, `date`). No repository name, title, or any other
//! identifying field is ever requested or stored. `query_has_no_repo_fields`
//! guards this at test time.
//!
//! # Data
//! `total` is GitHub's authoritative daily total from the contribution calendar
//! (exact, correct timezone, no repo cap). The four typed fields (`commits`,
//! `prs`, `issues`, `reviews`) come from the typed contribution streams and power
//! the dashboard filters. `total` is NOT computed as the sum of the four — the
//! calendar is authoritative; the typed sum can differ slightly (see ceilings).
//!
//! # Ceilings (ponytail: documented, upgrade path noted)
//! - Commits per day come from `commitContributionsByRepository(maxRepositories:
//!   100)` with `contributions(first: 100)` — a user with >100 repos in one year,
//!   or one repo with commits on >100 distinct days, may undercount the *commits
//!   filter*. The `total` is unaffected (it comes from the calendar). Upgrade:
//!   paginate the per-repo `contributions` connection if it ever matters.
//! - Day bucketing for issues/PRs/reviews uses the UTC date of `occurredAt`;
//!   GitHub's official graph uses the profile timezone, so a few near-midnight
//!   events may land on an adjacent day. `total` (calendar) matches github.com.

use crate::checkpoint::Checkpoint;
use crate::github_api::GithubApi;
use crate::logger;
use crate::sync::sha256_hex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

/// Checkpoint key for the contributions snapshot hash. The suffix is not a
/// 10-char date, so `Checkpoint::get_last_sync_date` ignores it (checkpoint.rs).
const CHECKPOINT_KEY: &str = "github:contributions";
/// Path to the snapshot file in the private vibestats-data repo.
const CONTRIB_PATH: &str = "github/contributions.json";
/// Stop backfilling further years when the GraphQL point budget drops below this.
/// GitHub's GraphQL limit is 5000 points/hour; one year costs a few hundred at
/// most, so this margin lets the in-progress year finish without a 429 storm.
const RATE_LIMIT_SAFETY: i64 = 200;

/// Per-day contribution counts, split by type. See module docs for `total` vs
/// the typed fields.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
struct DayCounts {
    #[serde(default)]
    total: u32,
    #[serde(default)]
    commits: u32,
    #[serde(default)]
    prs: u32,
    #[serde(default)]
    issues: u32,
    #[serde(default)]
    reviews: u32,
}

/// On-disk snapshot. `days` is the durable data; `backfilled_to_year` is the
/// oldest year already pulled by `--backfill` (the resume marker); `last_updated`
/// is informational only and excluded from the idempotency hash.
#[derive(Debug, Default, Serialize, Deserialize)]
struct StoredContributions {
    #[serde(default)]
    last_updated: String,
    #[serde(default)]
    backfilled_to_year: Option<i32>,
    #[serde(default)]
    days: BTreeMap<String, DayCounts>,
}

// ─── Query builders (privacy: counts/dates only — no repository fields) ────────

/// Returns the `(from, to)` ISO-8601 window covering an entire calendar year.
fn year_window(year: i32) -> (String, String) {
    (
        format!("{year}-01-01T00:00:00Z"),
        format!("{year}-12-31T23:59:59Z"),
    )
}

/// Build the GraphQL query for one year's contribution calendar (`total`) and
/// per-repository commit counts. Requests no repository-identifying fields.
fn calendar_commits_query(year: i32) -> String {
    let (from, to) = year_window(year);
    format!(
        "query {{ rateLimit {{ remaining resetAt }} viewer {{ \
         contributionsCollection(from: \"{from}\", to: \"{to}\") {{ \
         contributionCalendar {{ weeks {{ contributionDays {{ date contributionCount }} }} }} \
         commitContributionsByRepository(maxRepositories: 100) {{ \
         contributions(first: 100) {{ nodes {{ occurredAt commitCount }} }} }} \
         }} }} }}"
    )
}

/// Build the GraphQL query for one paginated typed contribution stream
/// (issues / PRs / reviews). `after` is the literal `null` or a quoted cursor.
fn flat_query(year: i32, field: &str, after: &str) -> String {
    let (from, to) = year_window(year);
    format!(
        "query {{ viewer {{ contributionsCollection(from: \"{from}\", to: \"{to}\") {{ \
         {field}(first: 100, after: {after}) {{ \
         nodes {{ occurredAt }} pageInfo {{ hasNextPage endCursor }} }} \
         }} }} }}"
    )
}

// ─── Response parsing (pure, network-free, unit-tested) ────────────────────────

/// Extract the `YYYY-MM-DD` date prefix from an ISO-8601 timestamp.
fn day_of(ts: &str) -> Option<String> {
    let d = ts.get(0..10)?;
    if d.len() == 10 {
        Some(d.to_string())
    } else {
        None
    }
}

/// Fold the contribution calendar's authoritative daily totals into `out`.
fn apply_calendar(cc: &Value, out: &mut HashMap<String, DayCounts>) {
    let Some(weeks) = cc["contributionCalendar"]["weeks"].as_array() else {
        return;
    };
    for week in weeks {
        let Some(days) = week["contributionDays"].as_array() else {
            continue;
        };
        for d in days {
            if let (Some(date), Some(count)) = (d["date"].as_str(), d["contributionCount"].as_u64())
            {
                out.entry(date.to_string()).or_default().total = count as u32;
            }
        }
    }
}

/// Sum per-repository commit counts into per-day totals in `out` (repo names are
/// never read — only `occurredAt` + `commitCount`).
fn apply_commits(cc: &Value, out: &mut HashMap<String, DayCounts>) {
    let Some(repos) = cc["commitContributionsByRepository"].as_array() else {
        return;
    };
    for repo in repos {
        let Some(nodes) = repo["contributions"]["nodes"].as_array() else {
            continue;
        };
        for n in nodes {
            if let (Some(ts), Some(count)) = (n["occurredAt"].as_str(), n["commitCount"].as_u64()) {
                if let Some(date) = day_of(ts) {
                    out.entry(date).or_default().commits += count as u32;
                }
            }
        }
    }
}

/// Selects which typed counter a flat contribution stream increments.
type FieldSelector = fn(&mut DayCounts) -> &mut u32;

/// Count one event per node into the field selected by `sel`, bucketed by day.
fn apply_flat(conn: &Value, sel: FieldSelector, out: &mut HashMap<String, DayCounts>) {
    let Some(nodes) = conn["nodes"].as_array() else {
        return;
    };
    for n in nodes {
        if let Some(ts) = n["occurredAt"].as_str() {
            if let Some(date) = day_of(ts) {
                *sel(out.entry(date).or_default()) += 1;
            }
        }
    }
}

/// Per-field max-merge: a fetched count can only ever RAISE a stored count, never
/// lower it. All-zero fetched days are skipped to keep the file lean. Existing
/// stored days are never removed — this is what makes org-departure non-destructive.
fn max_merge(stored: &mut BTreeMap<String, DayCounts>, fetched: HashMap<String, DayCounts>) {
    for (date, fc) in fetched {
        if fc == DayCounts::default() {
            continue;
        }
        let e = stored.entry(date).or_default();
        e.total = e.total.max(fc.total);
        e.commits = e.commits.max(fc.commits);
        e.prs = e.prs.max(fc.prs);
        e.issues = e.issues.max(fc.issues);
        e.reviews = e.reviews.max(fc.reviews);
    }
}

// ─── Fetch orchestration ───────────────────────────────────────────────────────

/// Fetch a single year's counts into `out`. Returns the GraphQL points remaining
/// (for backfill rate-limit pacing). Errors propagate so the caller silent-fails.
fn fetch_year(
    api: &GithubApi,
    year: i32,
    out: &mut HashMap<String, DayCounts>,
) -> Result<i64, String> {
    let resp = api
        .graphql_query(&calendar_commits_query(year))
        .map_err(|e| e.to_string())?;
    let cc = &resp["data"]["viewer"]["contributionsCollection"];
    if cc.is_null() {
        return Err(format!(
            "null contributionsCollection (token scope?): {}",
            resp["errors"]
        ));
    }
    apply_calendar(cc, out);
    apply_commits(cc, out);
    let remaining = resp["data"]["rateLimit"]["remaining"]
        .as_i64()
        .unwrap_or(i64::MAX);

    let streams: [(&str, FieldSelector); 3] = [
        ("issueContributions", |d| &mut d.issues),
        ("pullRequestContributions", |d| &mut d.prs),
        ("pullRequestReviewContributions", |d| &mut d.reviews),
    ];
    for (field, sel) in streams {
        let mut after = String::from("null");
        loop {
            let resp = api
                .graphql_query(&flat_query(year, field, &after))
                .map_err(|e| e.to_string())?;
            let conn = &resp["data"]["viewer"]["contributionsCollection"][field];
            if conn.is_null() {
                return Err(format!("null {field} (token scope?): {}", resp["errors"]));
            }
            apply_flat(conn, sel, out);
            if conn["pageInfo"]["hasNextPage"].as_bool() == Some(true) {
                match conn["pageInfo"]["endCursor"].as_str() {
                    Some(cursor) => after = format!("\"{cursor}\""),
                    None => break,
                }
            } else {
                break;
            }
        }
    }
    Ok(remaining)
}

/// Sync GitHub contributions into the private snapshot. Non-fatal: any failure is
/// logged and swallowed (silent-failure contract — never disturbs the hook).
///
/// `start_date`/`end_date` are `YYYY-MM-DD`. When `backfill` is false, only the
/// year(s) spanned by that range are refreshed (normally just the current year).
/// When true, walks from the account-creation year up to the current year,
/// newest-first, resuming from `backfilled_to_year` and pausing if the GraphQL
/// budget runs low.
pub fn sync(
    api: &GithubApi,
    checkpoint: &mut Checkpoint,
    start_date: &str,
    end_date: &str,
    backfill: bool,
) {
    if let Err(e) = sync_inner(api, checkpoint, start_date, end_date, backfill) {
        logger::error(&format!(
            "github_contributions: sync failed (non-fatal): {e}"
        ));
    }
}

fn sync_inner(
    api: &GithubApi,
    checkpoint: &mut Checkpoint,
    start_date: &str,
    end_date: &str,
    backfill: bool,
) -> Result<(), String> {
    let parse_year = |d: &str| d.get(0..4).and_then(|s| s.parse::<i32>().ok());
    let current_year = parse_year(end_date).ok_or("bad end_date")?;

    let mut stored: StoredContributions = match api
        .get_file_content(CONTRIB_PATH)
        .map_err(|e| e.to_string())?
    {
        Some(body) => serde_json::from_str(&body).unwrap_or_default(),
        None => StoredContributions::default(),
    };

    let mut fetched: HashMap<String, DayCounts> = HashMap::new();

    if backfill {
        let resp = api
            .graphql_query("query { viewer { createdAt } }")
            .map_err(|e| e.to_string())?;
        let created = resp["data"]["viewer"]["createdAt"]
            .as_str()
            .and_then(parse_year)
            .ok_or("could not read viewer.createdAt")?;
        // Resume newest-first below the oldest year already backfilled.
        let start = stored
            .backfilled_to_year
            .map(|y| y - 1)
            .unwrap_or(current_year);
        for year in (created..=start).rev() {
            match fetch_year(api, year, &mut fetched) {
                Ok(remaining) => {
                    stored.backfilled_to_year = Some(year);
                    if remaining < RATE_LIMIT_SAFETY {
                        logger::error(&format!(
                            "github_contributions: GraphQL budget low ({remaining} pts); paused \
                             backfill at {year} — re-run `vibestats sync --backfill` after reset to continue"
                        ));
                        break;
                    }
                }
                Err(e) => {
                    // Keep years already fetched; backfilled_to_year reflects the
                    // last success, so a re-run resumes from there.
                    logger::error(&format!(
                        "github_contributions: backfill stopped at {year}: {e}"
                    ));
                    break;
                }
            }
        }
    } else {
        // Normal sync: refresh the year(s) spanned by the range (usually just one).
        let start_year = parse_year(start_date).unwrap_or(current_year);
        for year in start_year..=current_year {
            fetch_year(api, year, &mut fetched)?;
        }
    }

    max_merge(&mut stored.days, fetched);

    // Idempotency: hash the durable state (data + resume marker), not last_updated.
    let hashable = serde_json::json!({ "days": &stored.days, "backfilled_to_year": stored.backfilled_to_year });
    let hash = sha256_hex(hashable.to_string().as_bytes());
    if checkpoint.hash_matches(CHECKPOINT_KEY, &hash) {
        return Ok(());
    }

    stored.last_updated = end_date.to_string();
    let payload = serde_json::to_string_pretty(&stored).map_err(|e| e.to_string())?;
    api.put_file(CONTRIB_PATH, &payload)
        .map_err(|e| e.to_string())?;
    checkpoint.update_hash(CHECKPOINT_KEY, &hash);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dc(total: u32, commits: u32, prs: u32, issues: u32, reviews: u32) -> DayCounts {
        DayCounts {
            total,
            commits,
            prs,
            issues,
            reviews,
        }
    }

    #[test]
    fn max_merge_never_lowers_stored() {
        // Org-departure case: a re-fetch returns fewer than stored — keep stored.
        let mut stored = BTreeMap::new();
        stored.insert("2025-10-14".to_string(), dc(23, 20, 1, 1, 1));
        let mut fetched = HashMap::new();
        fetched.insert("2025-10-14".to_string(), dc(0, 0, 0, 0, 0));
        max_merge(&mut stored, fetched);
        assert_eq!(stored["2025-10-14"], dc(23, 20, 1, 1, 1));
    }

    #[test]
    fn max_merge_raises_and_adds() {
        let mut stored = BTreeMap::new();
        stored.insert("2026-01-01".to_string(), dc(5, 5, 0, 0, 0));
        let mut fetched = HashMap::new();
        fetched.insert("2026-01-01".to_string(), dc(9, 5, 2, 0, 2)); // higher total + new types
        fetched.insert("2026-01-02".to_string(), dc(3, 1, 1, 1, 0)); // brand-new day
        max_merge(&mut stored, fetched);
        assert_eq!(stored["2026-01-01"], dc(9, 5, 2, 0, 2));
        assert_eq!(stored["2026-01-02"], dc(3, 1, 1, 1, 0));
    }

    #[test]
    fn max_merge_skips_all_zero_days() {
        let mut stored = BTreeMap::new();
        let mut fetched = HashMap::new();
        fetched.insert("2026-01-03".to_string(), dc(0, 0, 0, 0, 0));
        max_merge(&mut stored, fetched);
        assert!(stored.is_empty(), "all-zero days must not be stored");
    }

    #[test]
    fn apply_calendar_reads_total() {
        let cc = serde_json::json!({
            "contributionCalendar": { "weeks": [
                { "contributionDays": [
                    { "date": "2026-06-28", "contributionCount": 12 },
                    { "date": "2026-06-29", "contributionCount": 0 }
                ] }
            ] }
        });
        let mut out = HashMap::new();
        apply_calendar(&cc, &mut out);
        assert_eq!(out["2026-06-28"].total, 12);
        assert_eq!(out["2026-06-29"].total, 0);
    }

    #[test]
    fn apply_commits_sums_across_repos_per_day() {
        let cc = serde_json::json!({
            "commitContributionsByRepository": [
                { "contributions": { "nodes": [
                    { "occurredAt": "2026-06-28T10:00:00Z", "commitCount": 3 }
                ] } },
                { "contributions": { "nodes": [
                    { "occurredAt": "2026-06-28T15:00:00Z", "commitCount": 2 },
                    { "occurredAt": "2026-06-27T09:00:00Z", "commitCount": 4 }
                ] } }
            ]
        });
        let mut out = HashMap::new();
        apply_commits(&cc, &mut out);
        assert_eq!(
            out["2026-06-28"].commits, 5,
            "commits summed across two repos"
        );
        assert_eq!(out["2026-06-27"].commits, 4);
    }

    #[test]
    fn apply_flat_counts_one_per_event() {
        let conn = serde_json::json!({ "nodes": [
            { "occurredAt": "2026-06-28T10:00:00Z" },
            { "occurredAt": "2026-06-28T12:00:00Z" },
            { "occurredAt": "2026-06-27T08:00:00Z" }
        ] });
        let mut out = HashMap::new();
        apply_flat(&conn, |d| &mut d.prs, &mut out);
        assert_eq!(out["2026-06-28"].prs, 2);
        assert_eq!(out["2026-06-27"].prs, 1);
    }

    #[test]
    fn query_has_no_repo_fields() {
        // Privacy guard (NFR8/NFR9): no query may request a repository name or
        // any identifying field — only counts and dates.
        let forbidden = [
            "repository",
            "nameWithOwner",
            "name",
            "title",
            "url",
            "login",
        ];
        let queries = [
            calendar_commits_query(2026),
            flat_query(2026, "issueContributions", "null"),
            flat_query(2026, "pullRequestReviewContributions", "\"cursor\""),
        ];
        for q in &queries {
            for word in &forbidden {
                assert!(
                    !q.contains(word),
                    "query must not request '{word}' (privacy): {q}"
                );
            }
        }
    }

    #[test]
    fn flat_query_paginates_with_cursor() {
        assert!(flat_query(2026, "issueContributions", "null").contains("after: null"));
        assert!(flat_query(2026, "issueContributions", "\"abc\"").contains("after: \"abc\""));
    }
}
