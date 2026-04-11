#![allow(dead_code)]
// session_start_hook() arrives in Story 3.3

// Hook configuration reference for ~/.claude/settings.json (written by installer, Story 6.4):
// {
//   "hooks": {
//     "Stop": [{ "hooks": [{ "type": "command", "command": "vibestats sync", "async": true }] }]
//   }
// }
// async: true ensures Claude Code does NOT wait for vibestats to finish (NFR10 hook non-interference).

use crate::checkpoint::Checkpoint;
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

/// Returns the current UTC date as "YYYY-MM-DD".
/// Implemented std-only — no chrono or time crate.
fn today_utc() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let z = secs / 86400 + 719468;
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

/// Handles the Stop hook event: checks throttle, calls sync if not throttled,
/// then updates the throttle timestamp and saves the checkpoint.
///
/// NEVER calls std::process::exit — exit is always delegated to main.rs.
///
/// Error handling:
/// - HOME not set: treat as no checkpoint — call sync::run (fail-open), skip checkpoint save
/// - checkpoint.save fails: log via logger::error, continue
/// - sync::run (any error): handled internally by sync.rs — always returns ()
/// - Throttle active: return immediately — main.rs exits 0
pub fn stop_hook() {
    let path = checkpoint_path();

    // Load checkpoint once up-front solely to evaluate the throttle window.
    // We deliberately discard this copy after the throttle check because
    // `sync::run` loads and persists its OWN checkpoint internally
    // (updating `date_hashes` and possibly `auth_error`). Re-using a stale
    // in-memory copy across `sync::run` would silently overwrite those
    // updates and break idempotency (NFR12) plus the auth-error flag.
    let throttle_checkpoint = path
        .as_deref()
        .map(Checkpoint::load)
        .unwrap_or_default();

    if throttle_checkpoint.should_throttle() {
        return; // throttle active — caller (main.rs) exits 0
    }
    drop(throttle_checkpoint);

    let today = today_utc();
    sync::run(&today, &today);

    // Re-load checkpoint AFTER `sync::run` has persisted its own updates so
    // that stamping the throttle timestamp does not clobber date_hashes or
    // auth_error just written to disk. `sync::run` always returns `()`.
    let mut checkpoint = path
        .as_deref()
        .map(Checkpoint::load)
        .unwrap_or_default();
    checkpoint.update_throttle_timestamp();
    if let Some(p) = path.as_deref() {
        if let Err(e) = checkpoint.save(p) {
            logger::error(&format!("stop_hook: failed to save checkpoint: {e}"));
        }
    }
    // Return to main.rs which calls std::process::exit(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn today_utc_format() {
        let date = today_utc();
        // Must be exactly 10 characters: YYYY-MM-DD
        assert_eq!(date.len(), 10, "date must be 10 chars: got {date}");
        // Verify digit-dash pattern: DDDD-DD-DD
        let chars: Vec<char> = date.chars().collect();
        assert!(chars[0].is_ascii_digit());
        assert!(chars[1].is_ascii_digit());
        assert!(chars[2].is_ascii_digit());
        assert!(chars[3].is_ascii_digit());
        assert_eq!(chars[4], '-');
        assert!(chars[5].is_ascii_digit());
        assert!(chars[6].is_ascii_digit());
        assert_eq!(chars[7], '-');
        assert!(chars[8].is_ascii_digit());
        assert!(chars[9].is_ascii_digit());
    }

    #[test]
    fn today_utc_zero_padded() {
        // Ensure month and day are always zero-padded (2 digits).
        // We can't control what today is, but we can validate format constraints.
        let date = today_utc();
        let parts: Vec<&str> = date.split('-').collect();
        assert_eq!(parts.len(), 3, "must have 3 parts: got {date}");
        assert_eq!(parts[0].len(), 4, "year must be 4 chars");
        assert_eq!(parts[1].len(), 2, "month must be 2 chars (zero-padded)");
        assert_eq!(parts[2].len(), 2, "day must be 2 chars (zero-padded)");
        // month must be 01..12
        let month: u32 = parts[1].parse().expect("month must be numeric");
        assert!((1..=12).contains(&month), "month out of range: {month}");
        // day must be 01..31
        let day: u32 = parts[2].parse().expect("day must be numeric");
        assert!((1..=31).contains(&day), "day out of range: {day}");
    }

    #[test]
    fn throttle_branch_checkpoint_with_recent_timestamp() {
        // A Checkpoint updated just now should report should_throttle() == true.
        let mut cp = Checkpoint::default();
        cp.update_throttle_timestamp();
        assert!(
            cp.should_throttle(),
            "should_throttle must be true immediately after update"
        );
    }

    #[test]
    fn no_throttle_branch_checkpoint_with_old_timestamp() {
        // A Checkpoint with a stale timestamp should report should_throttle() == false.
        let cp = Checkpoint {
            throttle_timestamp: Some("2020-01-01T00:00:00Z".to_string()),
            ..Default::default()
        };
        assert!(
            !cp.should_throttle(),
            "should_throttle must be false for old timestamp"
        );
    }

    #[test]
    fn no_throttle_branch_checkpoint_absent() {
        // A default Checkpoint (no throttle_timestamp) must not throttle.
        let cp = Checkpoint::default();
        assert!(
            !cp.should_throttle(),
            "should_throttle must be false when no throttle_timestamp is set"
        );
    }

    /// Produce a unique temp path per call (mirrors checkpoint.rs::tests::temp_path).
    /// Prevents collisions between parallel test runs and concurrent repo checkouts.
    fn temp_path(name: &str) -> std::path::PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let pid = std::process::id();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("vibestats_hooks_{name}_{pid}_{nanos}_{seq}.toml"))
    }

    /// Regression test for the "throttle-save clobbers sync-save" bug.
    ///
    /// `sync::run` loads and saves its OWN checkpoint inside `stop_hook`,
    /// writing `date_hashes` and potentially flipping `auth_error`. If
    /// `stop_hook` held an in-memory copy from BEFORE `sync::run` and then
    /// saved that stale copy (with only `throttle_timestamp` updated), it
    /// would overwrite the sync-persisted fields — breaking idempotency
    /// (NFR12) and silently discarding the auth-error flag. This test
    /// simulates the sequence and asserts all fields survive.
    #[test]
    fn throttle_save_does_not_clobber_sync_persisted_fields() {
        let path = temp_path("no_clobber");

        // Step 1 — simulate `sync::run` persisting its state.
        let mut sync_cp = Checkpoint::default();
        sync_cp.update_hash("2026-04-10", "deadbeef");
        sync_cp.set_auth_error();
        sync_cp.save(&path).unwrap();

        // Step 2 — emulate `stop_hook`'s post-sync save: re-load fresh
        // (this is the fix), stamp throttle, save.
        let mut throttle_cp = Checkpoint::load(&path);
        throttle_cp.update_throttle_timestamp();
        throttle_cp.save(&path).unwrap();

        // Step 3 — final state on disk must contain BOTH the sync-written
        // fields AND the new throttle timestamp.
        let final_cp = Checkpoint::load(&path);
        assert!(
            final_cp.hash_matches("2026-04-10", "deadbeef"),
            "date_hashes written by sync::run must survive throttle save"
        );
        assert!(
            final_cp.auth_error,
            "auth_error written by sync::run must survive throttle save"
        );
        assert!(
            final_cp.throttle_timestamp.is_some(),
            "stop_hook must have stamped throttle_timestamp"
        );

        let _ = std::fs::remove_file(&path); // cleanup
    }
}
