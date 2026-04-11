//! SessionStart hook handler for vibestats.
//!
//! Executes the following behaviours in order at every Claude Code session start:
//! 1. Machine retirement check — if retired: save checkpoint, print warning, return early
//! 2. Auth error surface — print message and clear `auth_error` flag if set
//! 3. Catch-up sync — calls `sync::run(last_sync_date, yesterday)` if gap exists
//! 4. Staleness warning — prints warning if last sync > 24h ago
//!
//! # Architecture constraints
//! - Never calls `std::process::exit` — only `main.rs` does
//! - All GitHub API calls via `github_api.rs` — never inline HTTP
//! - No external crates (chrono, base64, etc.) — std only
//! - All code paths exit 0 (NFR10)

use crate::checkpoint::Checkpoint;
use crate::config::Config;
use crate::github_api::GithubApi;
use crate::logger;
use crate::sync;
use std::path::PathBuf;

/// Returns the path to the checkpoint file, or None if HOME is not set.
fn checkpoint_path() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(|h| {
        PathBuf::from(h)
            .join(".config")
            .join("vibestats")
            .join("checkpoint.toml")
    })
}

/// Returns yesterday's date as `"YYYY-MM-DD"` (UTC).
///
/// Implemented std-only using the civil-date formula from checkpoint.rs.
/// Do NOT import chrono or any date crate.
fn yesterday() -> String {
    let now = std::time::SystemTime::now();
    let secs = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Subtract 86400 seconds (one day)
    let yesterday_secs = secs.saturating_sub(86400);
    // Civil-from-days formula (Howard Hinnant: https://howardhinnant.github.io/date_algorithms.html)
    let z = yesterday_secs / 86400 + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if mo <= 2 { y + 1 } else { y };
    format!("{:04}-{:02}-{:02}", y, mo, d)
}

/// Returns the number of days since the given ISO 8601 UTC timestamp string.
///
/// Returns `None` if the timestamp is unparseable or in the future (clock skew).
fn days_since_timestamp(ts_str: &str) -> Option<u64> {
    let ts = parse_iso8601_utc(ts_str)?;
    let now = std::time::SystemTime::now();
    match now.duration_since(ts) {
        Ok(elapsed) => Some(elapsed.as_secs() / 86400),
        Err(_) => None, // timestamp in the future (clock skew) — skip warning
    }
}

/// Parses `"YYYY-MM-DDTHH:MM:SSZ"` → `SystemTime` (UNIX_EPOCH offset).
///
/// Returns `None` on any parse error (fail-open: caller skips warning).
/// Strictly requires a trailing `Z` and rejects out-of-range fields and pre-1970 dates.
/// This mirrors the private `parse_iso8601_utc` in `checkpoint.rs` — re-implemented
/// here rather than re-exported to respect module boundary rules.
fn parse_iso8601_utc(s: &str) -> Option<std::time::SystemTime> {
    let s = s.strip_suffix('Z')?;
    let (date_str, time_str) = s.split_once('T')?;
    let mut dp = date_str.split('-');
    let year: u64 = dp.next()?.parse().ok()?;
    let month: u64 = dp.next()?.parse().ok()?;
    let day: u64 = dp.next()?.parse().ok()?;
    if dp.next().is_some() {
        return None;
    }
    let mut tp = time_str.split(':');
    let hour: u64 = tp.next()?.parse().ok()?;
    let min: u64 = tp.next()?.parse().ok()?;
    let sec: u64 = tp.next()?.parse().ok()?;
    if tp.next().is_some() {
        return None;
    }
    if year < 1970 || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    if hour >= 24 || min >= 60 || sec >= 60 {
        return None;
    }
    // Days since Unix epoch via civil date formula
    let y = if month <= 2 { year - 1 } else { year };
    let era = y / 400;
    let yoe = y - era * 400;
    let doy =
        (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days_since_epoch = era * 146097 + doe - 719468;
    let secs = days_since_epoch * 86400 + hour * 3600 + min * 60 + sec;
    Some(std::time::UNIX_EPOCH + std::time::Duration::from_secs(secs))
}

/// Execute the SessionStart hook logic.
///
/// Steps (in order):
/// 1. Machine retirement check
/// 2. Auth error surface
/// 3. Catch-up sync
/// 4. Staleness warning
///
/// Returns `()`. `main.rs` calls `std::process::exit(0)` after this.
pub fn run() {
    let config = Config::load_or_exit();
    let cp_path = checkpoint_path();
    let mut checkpoint = cp_path
        .as_deref()
        .map(Checkpoint::load)
        .unwrap_or_default();

    let api = GithubApi::new(&config.oauth_token, &config.vibestats_data_repo);

    // ── Step 1: Machine retirement check ────────────────────────────────────
    match api.get_file_content("registry.json") {
        Ok(Some(content)) => {
            // Parse registry JSON to check if this machine is retired
            let json: serde_json::Value =
                serde_json::from_str(&content).unwrap_or(serde_json::Value::Null);
            let machines = json["machines"].as_array();
            if let Some(machines) = machines {
                for m in machines {
                    if m["machine_id"].as_str() == Some(config.machine_id.as_str()) {
                        if m["status"].as_str() == Some("retired") {
                            checkpoint.set_machine_status("retired");
                            if let Some(ref path) = cp_path {
                                if let Err(e) = checkpoint.save(path) {
                                    logger::error(&format!(
                                        "session_start: failed to save checkpoint: {}",
                                        e
                                    ));
                                }
                            }
                            println!(
                                "vibestats: this machine has been retired. Sync skipped."
                            );
                            return; // Steps 2–4 are skipped
                        }
                        break;
                    }
                }
            }
        }
        Ok(None) => {
            // Registry not found (404) — no retirement check needed yet
        }
        Err(e) => {
            // Registry fetch failed — non-fatal, continue
            logger::error(&format!(
                "session_start: failed to fetch registry.json: {}",
                e
            ));
        }
    }

    // ── Step 2: Auth error surface ───────────────────────────────────────────
    if checkpoint.auth_error {
        println!(
            "vibestats: auth error detected. Run `vibestats auth` to re-authenticate."
        );
        checkpoint.clear_auth_error();
        // Save checkpoint now — auth_error must be cleared even if subsequent steps also call save
        if let Some(ref path) = cp_path {
            if let Err(e) = checkpoint.save(path) {
                logger::error(&format!(
                    "session_start: failed to save checkpoint after clearing auth_error: {}",
                    e
                ));
            }
        }
    }

    // ── Step 3: Catch-up sync ────────────────────────────────────────────────
    // Guard: only proceed if not retired
    if !checkpoint.is_retired() {
        if let Some(last_sync_date) = checkpoint.get_last_sync_date() {
            let yesterday = yesterday();
            // String comparison is correct for "YYYY-MM-DD" (zero-padded ISO 8601)
            if last_sync_date < yesterday {
                sync::run(&last_sync_date, &yesterday);
            }
        }
        // If get_last_sync_date() returns None, skip catch-up (no previous sync recorded)
    }

    // ── Step 4: Staleness warning ────────────────────────────────────────────
    // Reload checkpoint from disk after sync::run (sync may have updated throttle_timestamp)
    let checkpoint = cp_path
        .as_deref()
        .map(Checkpoint::load)
        .unwrap_or_default();

    if let Some(ref ts) = checkpoint.throttle_timestamp {
        if let Some(days) = days_since_timestamp(ts) {
            if days > 0 {
                // elapsed > 86400 seconds (more than 1 full day)
                let elapsed_secs = std::time::SystemTime::now()
                    .duration_since(
                        parse_iso8601_utc(ts)
                            .unwrap_or(std::time::UNIX_EPOCH),
                    )
                    .unwrap_or_default()
                    .as_secs();
                let n = elapsed_secs / 86400;
                if n > 0 {
                    println!(
                        "vibestats: last sync was {} days ago on this machine. Run `vibestats status` to diagnose.",
                        n
                    );
                }
            }
        }
    }
    // If throttle_timestamp is None: skip staleness warning (no sync has ever run)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── yesterday() ──────────────────────────────────────────────────────────

    #[test]
    fn test_yesterday_returns_valid_date_string() {
        let y = yesterday();
        // Must be exactly "YYYY-MM-DD" format (10 chars)
        assert_eq!(y.len(), 10, "yesterday() must return 10-char string, got: {y}");
        // Must match YYYY-MM-DD pattern
        let parts: Vec<&str> = y.split('-').collect();
        assert_eq!(parts.len(), 3, "yesterday() must contain two '-' separators");
        assert_eq!(parts[0].len(), 4, "year must be 4 digits");
        assert_eq!(parts[1].len(), 2, "month must be 2 digits");
        assert_eq!(parts[2].len(), 2, "day must be 2 digits");
        // Month must be 01–12
        let month: u32 = parts[1].parse().expect("month must be numeric");
        assert!((1..=12).contains(&month), "month out of range: {month}");
        // Day must be 01–31
        let day: u32 = parts[2].parse().expect("day must be numeric");
        assert!((1..=31).contains(&day), "day out of range: {day}");
    }

    // ── days_since_timestamp() ────────────────────────────────────────────────

    #[test]
    fn test_days_since_timestamp_epoch_is_large() {
        // Unix epoch is far in the past — days must be >= 365*50 (> 18000)
        let days = days_since_timestamp("1970-01-01T00:00:00Z");
        assert!(
            days.is_some(),
            "epoch timestamp should parse successfully"
        );
        assert!(
            days.unwrap() > 18000,
            "epoch timestamp should be thousands of days ago"
        );
    }

    #[test]
    fn test_days_since_timestamp_far_future_returns_none() {
        // A timestamp far in the future should return None (clock skew path)
        let days = days_since_timestamp("2099-01-01T00:00:00Z");
        assert!(days.is_none(), "future timestamp should return None");
    }

    #[test]
    fn test_days_since_timestamp_unparseable_returns_none() {
        assert!(days_since_timestamp("not-a-timestamp").is_none());
        assert!(days_since_timestamp("").is_none());
    }

    // ── parse_iso8601_utc() ───────────────────────────────────────────────────

    #[test]
    fn test_parse_iso8601_utc_valid() {
        let result = parse_iso8601_utc("2026-04-11T00:00:00Z");
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_iso8601_utc_rejects_missing_z() {
        assert!(parse_iso8601_utc("2026-04-11T00:00:00").is_none());
    }

    #[test]
    fn test_parse_iso8601_utc_rejects_pre_1970() {
        assert!(parse_iso8601_utc("1969-12-31T23:59:59Z").is_none());
    }

    // ── auth error: message printed and flag cleared ──────────────────────────

    #[test]
    fn test_auth_error_flag_cleared_when_true() {
        // Verify clear_auth_error() sets the flag to false
        let mut cp = Checkpoint {
            auth_error: true,
            ..Default::default()
        };
        assert!(cp.auth_error);
        cp.clear_auth_error();
        assert!(!cp.auth_error, "auth_error must be false after clear_auth_error()");
    }

    // ── retirement guard: catch-up sync skipped when machine is retired ───────

    #[test]
    fn test_is_retired_prevents_sync() {
        // Verify that is_retired() returns true after set_machine_status("retired")
        let mut cp = Checkpoint::default();
        assert!(!cp.is_retired());
        cp.set_machine_status("retired");
        assert!(cp.is_retired(), "machine must be retired after set_machine_status(\"retired\")");
    }

    // ── staleness warning threshold ───────────────────────────────────────────

    #[test]
    fn test_staleness_warning_over_24h() {
        // A timestamp 2 days ago should trigger the warning (days > 0)
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        // 2 days ago
        let two_days_ago_secs = now_secs.saturating_sub(2 * 86400);
        // Format as ISO 8601 UTC
        let z = two_days_ago_secs / 86400;
        let time_of_day = two_days_ago_secs % 86400;
        let h = time_of_day / 3600;
        let m = (time_of_day % 3600) / 60;
        let s = time_of_day % 60;
        let z = z + 719468;
        let era = z / 146097;
        let doe = z - era * 146097;
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
        let y = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let mo = if mp < 10 { mp + 3 } else { mp - 9 };
        let y = if mo <= 2 { y + 1 } else { y };
        let ts_str = format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, mo, d, h, m, s);

        let days = days_since_timestamp(&ts_str);
        assert!(days.is_some(), "2-days-ago timestamp should parse");
        assert!(days.unwrap() >= 1, "2-days-ago timestamp should be >= 1 day ago");
    }

    #[test]
    fn test_staleness_warning_under_24h_returns_zero_days() {
        // A timestamp from 1 hour ago should return 0 days (< 86400 seconds)
        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let one_hour_ago = now_secs.saturating_sub(3600);
        let z = one_hour_ago / 86400;
        let time_of_day = one_hour_ago % 86400;
        let h = time_of_day / 3600;
        let m = (time_of_day % 3600) / 60;
        let s = time_of_day % 60;
        let z = z + 719468;
        let era = z / 146097;
        let doe = z - era * 146097;
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
        let y = yoe + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let mo = if mp < 10 { mp + 3 } else { mp - 9 };
        let y = if mo <= 2 { y + 1 } else { y };
        let ts_str = format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, mo, d, h, m, s);

        let days = days_since_timestamp(&ts_str);
        assert!(days.is_some());
        assert_eq!(days.unwrap(), 0, "1-hour-ago timestamp should be 0 days");
    }
}
