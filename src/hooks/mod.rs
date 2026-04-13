// session_start_hook() arrives in Story 3.3

pub mod session_start;

// Hook configuration reference for ~/.claude/settings.json (written by installer, Story 6.4):
// {
//   "hooks": {
//     "Stop": [{ "hooks": [{ "type": "command", "command": "vibestats sync", "async": true }] }]
//   }
// }
// async: true ensures Claude Code does NOT wait for vibestats to finish (NFR10 hook non-interference).

#[cfg(test)]
mod tests {
    use crate::checkpoint::Checkpoint;

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

    /// Regression test for the "throttle-save clobbers sync-save" bug.
    ///
    /// `sync::run` loads and saves its OWN checkpoint inside the stop hook sequence,
    /// writing `date_hashes` and potentially flipping `auth_error`. If the caller
    /// held an in-memory copy from BEFORE `sync::run` and then saved that stale copy
    /// (with only `throttle_timestamp` updated), it would overwrite the sync-persisted
    /// fields — breaking idempotency (NFR12) and silently discarding the auth-error flag.
    /// This test simulates the sequence and asserts all fields survive.
    #[test]
    fn throttle_save_does_not_clobber_sync_persisted_fields() {
        let path = temp_path("no_clobber");

        // Step 1 — simulate `sync::run` persisting its state.
        let mut sync_cp = Checkpoint::default();
        sync_cp.update_hash("2026-04-10", "deadbeef");
        sync_cp.set_auth_error();
        sync_cp.save(&path).unwrap();

        // Step 2 — emulate post-sync throttle save: re-load fresh, stamp throttle, save.
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
