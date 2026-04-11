//! Logger module for vibestats.
//!
//! Appends structured log entries to `~/.config/vibestats/vibestats.log`.
//! Rotates the file at 1 MB (renames to `vibestats.log.1`).
//!
//! # Silent failure contract
//!
//! All IO errors are silently discarded. This module **never** writes to
//! stdout or stderr, and **never** panics. It is designed to be called from
//! Claude Code hook hot-paths where any terminal output would break the hook
//! protocol (NFR10, NFR11).

// The public API is intentionally not called from main.rs yet (other modules
// will use `use crate::logger` once implemented). Suppress dead-code lints so
// that `cargo clippy -- -D warnings` passes during this intermediate state.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

// ─── Path helpers ─────────────────────────────────────────────────────────────

fn log_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home)
        .join(".config")
        .join("vibestats")
        .join("vibestats.log")
}

// ─── Timestamp (stdlib only, no chrono) ───────────────────────────────────────

/// Convert Unix epoch seconds to `(year, month, day, hour, minute, second)`.
///
/// Uses a straightforward day/month/year extraction without external crates.
fn epoch_to_datetime(secs: u64) -> (u64, u64, u64, u64, u64, u64) {
    let s = secs % 60;
    let total_min = secs / 60;
    let min = total_min % 60;
    let total_hours = total_min / 60;
    let h = total_hours % 24;
    let mut days = total_hours / 24; // days since 1970-01-01

    // Determine the year by iterating through years from 1970.
    let mut year: u64 = 1970;
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }

    // Determine the month.
    let leap = is_leap_year(year);
    let month_days: [u64; 12] = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut month: u64 = 1;
    for &md in &month_days {
        if days < md {
            break;
        }
        days -= md;
        month += 1;
    }

    let day = days + 1; // 1-indexed
    (year, month, day, h, min, s)
}

fn is_leap_year(year: u64) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

fn utc_timestamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (y, mo, d, h, min, s) = epoch_to_datetime(secs);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y, mo, d, h, min, s
    )
}

// ─── Rotation ─────────────────────────────────────────────────────────────────

const MAX_LOG_BYTES: u64 = 1_048_576; // 1 MB

fn rotate_if_needed(log_path: &Path) {
    if let Ok(meta) = std::fs::metadata(log_path) {
        if meta.len() >= MAX_LOG_BYTES {
            // Build the path for `vibestats.log.1` alongside the log file.
            // We cannot use `.with_extension("log.1")` because `Path::with_extension`
            // replaces only the last component, turning `vibestats.log` into
            // `vibestats.log.1` correctly when the current extension is `log`.
            // However, to be explicit and unambiguous we build the path manually.
            let rotated = match log_path.parent() {
                Some(parent) => parent.join("vibestats.log.1"),
                None => return,
            };
            let _ = std::fs::rename(log_path, rotated); // ignore errors
        }
    }
}

// ─── Core write helper (accepts path — enables testability) ───────────────────

fn write_log_entry(log_path: &Path, level: &str, message: &str) -> std::io::Result<()> {
    use std::io::Write;

    // Ensure the parent directory exists.
    if let Some(parent) = log_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    rotate_if_needed(log_path);

    let line = format!("{} {} {}\n", utc_timestamp(), level, message);

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;

    file.write_all(line.as_bytes())?;
    Ok(())
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Write a structured log entry to `~/.config/vibestats/vibestats.log`.
///
/// All IO errors are silently discarded. Nothing is written to stdout or
/// stderr under any circumstances.
pub fn log(level: &str, message: &str) {
    let _ = write_log_entry(&log_path(), level, message);
}

/// Convenience wrapper: log at INFO level.
pub fn info(message: &str) {
    log("INFO", message);
}

/// Convenience wrapper: log at ERROR level.
pub fn error(message: &str) {
    log("ERROR", message);
}

/// Convenience wrapper: log at WARN level.
pub fn warn(message: &str) {
    log("WARN", message);
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    /// Helper: write a log entry to a caller-supplied path (for test isolation).
    fn write_entry_to(path: &Path, level: &str, msg: &str) {
        let _ = write_log_entry(path, level, msg);
    }

    fn unique_test_dir(name: &str) -> PathBuf {
        // Use process ID + test name to avoid cross-test collisions.
        std::env::temp_dir()
            .join(format!("vibestats_test_{}_{}", name, std::process::id()))
    }

    // ── AC #1: log entry format ───────────────────────────────────────────────

    /// Parse a log line and verify it matches `YYYY-MM-DDTHH:MM:SSZ LEVEL message`.
    fn assert_log_line_format(line: &str, expected_level: &str, expected_msg: &str) {
        // Minimum length: "1970-01-01T00:00:00Z " = 21 chars + level + " " + msg
        assert!(line.len() >= 21, "line too short: {line:?}");

        // Timestamp portion: first 20 chars
        let ts = &line[..20];
        let rest = &line[20..]; // should start with " LEVEL message"

        // Verify timestamp structure: YYYY-MM-DDTHH:MM:SSZ
        assert!(ts.ends_with('Z'), "timestamp must end with Z: {ts}");
        let chars: Vec<char> = ts.chars().collect();
        assert_eq!(chars[4], '-', "YYYY-MM sep missing");
        assert_eq!(chars[7], '-', "MM-DD sep missing");
        assert_eq!(chars[10], 'T', "date-time T sep missing");
        assert_eq!(chars[13], ':', "HH:MM sep missing");
        assert_eq!(chars[16], ':', "MM:SS sep missing");
        // All other chars must be digits
        for (i, &c) in chars.iter().enumerate() {
            if ![4, 7, 10, 13, 16, 19].contains(&i) {
                assert!(c.is_ascii_digit(), "expected digit at pos {i} in ts, got {c:?}");
            }
        }

        // Rest should be " LEVEL message"
        let expected_rest = format!(" {} {}", expected_level, expected_msg);
        assert_eq!(rest, expected_rest, "level/message portion mismatch");
    }

    #[test]
    fn test_log_format_matches_spec() {
        let dir = unique_test_dir("format");
        let _ = fs::create_dir_all(&dir);
        let log_file = dir.join("test.log");

        write_entry_to(&log_file, "INFO", "hello world");

        let content = fs::read_to_string(&log_file).expect("log file should exist");
        let line = content.lines().next().expect("should have one line");

        assert_log_line_format(line, "INFO", "hello world");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_log_format_ends_with_newline() {
        let dir = unique_test_dir("newline");
        let _ = fs::create_dir_all(&dir);
        let log_file = dir.join("test.log");

        write_entry_to(&log_file, "ERROR", "something went wrong");

        let content = fs::read_to_string(&log_file).expect("log file should exist");
        assert!(content.ends_with('\n'), "log entry must end with newline");

        let _ = fs::remove_dir_all(&dir);
    }

    // ── AC #1 + AC #2: append semantics ──────────────────────────────────────

    #[test]
    fn test_multiple_entries_are_appended() {
        let dir = unique_test_dir("append");
        let _ = fs::create_dir_all(&dir);
        let log_file = dir.join("test.log");

        write_entry_to(&log_file, "INFO", "first");
        write_entry_to(&log_file, "INFO", "second");
        write_entry_to(&log_file, "INFO", "third");

        let content = fs::read_to_string(&log_file).expect("log file should exist");
        let lines: Vec<&str> = content.lines().collect();

        assert_eq!(lines.len(), 3, "should have 3 log entries");
        assert!(lines[0].ends_with("first"), "first entry missing");
        assert!(lines[1].ends_with("second"), "second entry missing");
        assert!(lines[2].ends_with("third"), "third entry missing");

        let _ = fs::remove_dir_all(&dir);
    }

    // ── AC #2: rotation at exactly 1 MB ──────────────────────────────────────

    #[test]
    fn test_rotation_at_1mb_threshold() {
        let dir = unique_test_dir("rotate");
        let _ = fs::create_dir_all(&dir);
        let log_file = dir.join("vibestats.log");
        let rotated_file = dir.join("vibestats.log.1");

        // Fill the file to exactly MAX_LOG_BYTES.
        {
            let mut f = fs::OpenOptions::new()
                .create(true)
                .write(true)
                .open(&log_file)
                .expect("should create log file");
            let filler = vec![b'x'; MAX_LOG_BYTES as usize];
            f.write_all(&filler).expect("should write filler");
        }

        // Next write should trigger rotation.
        write_entry_to(&log_file, "INFO", "after rotation");

        assert!(
            rotated_file.exists(),
            "vibestats.log.1 should exist after rotation"
        );
        assert!(
            log_file.exists(),
            "vibestats.log should be recreated after rotation"
        );

        let new_content = fs::read_to_string(&log_file).expect("new log should exist");
        assert!(
            new_content.contains("after rotation"),
            "new log should contain the post-rotation entry"
        );

        // The rotated file should NOT contain "after rotation"
        let rotated_content = fs::read_to_string(&rotated_file).expect("rotated log should exist");
        assert!(
            !rotated_content.contains("after rotation"),
            "rotated log should not contain the new entry"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_no_rotation_below_1mb() {
        let dir = unique_test_dir("no_rotate");
        let _ = fs::create_dir_all(&dir);
        let log_file = dir.join("vibestats.log");
        let rotated_file = dir.join("vibestats.log.1");

        // File is below threshold.
        {
            let mut f = fs::OpenOptions::new()
                .create(true)
                .write(true)
                .open(&log_file)
                .expect("should create log file");
            let filler = vec![b'x'; (MAX_LOG_BYTES - 1) as usize];
            f.write_all(&filler).expect("should write filler");
        }

        write_entry_to(&log_file, "INFO", "no rotation");

        assert!(
            !rotated_file.exists(),
            "vibestats.log.1 should NOT exist when below threshold"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    // ── epoch_to_datetime ─────────────────────────────────────────────────────

    #[test]
    fn test_epoch_to_datetime_known_values() {
        // Unix epoch itself: 1970-01-01T00:00:00Z → 0
        let (y, mo, d, h, min, s) = epoch_to_datetime(0);
        assert_eq!((y, mo, d, h, min, s), (1970, 1, 1, 0, 0, 0));

        // 2000-01-01T00:00:00Z → 946684800
        let (y, mo, d, h, min, s) = epoch_to_datetime(946_684_800);
        assert_eq!((y, mo, d, h, min, s), (2000, 1, 1, 0, 0, 0));

        // 2025-04-10T00:00:00Z → 1744243200
        let (y, mo, d, h, min, s) = epoch_to_datetime(1_744_243_200);
        assert_eq!((y, mo, d, h, min, s), (2025, 4, 10, 0, 0, 0));

        // 2026-01-01T00:00:00Z → 1767225600
        let (y, mo, d, h, min, s) = epoch_to_datetime(1_767_225_600);
        assert_eq!((y, mo, d, h, min, s), (2026, 1, 1, 0, 0, 0));

        // A time with hours/minutes/seconds: 2026-04-11T03:07:00Z
        // Days from 1970-01-01 to 2026-04-11:
        //   2026-01-01 = 1767225600 (verified above)
        //   + 100 days (Jan=31, Feb=28, Mar=31, Apr 1-11 = 10 → offset 100) * 86400
        //   + 3*3600 + 7*60
        // = 1767225600 + 8640000 + 10800 + 420 = 1775876820
        let (y, mo, d, h, min, s) = epoch_to_datetime(1_775_876_820);
        assert_eq!((y, mo, d, h, min, s), (2026, 4, 11, 3, 7, 0));
    }

    #[test]
    fn test_utc_timestamp_format() {
        let ts = utc_timestamp();
        // Should be exactly "YYYY-MM-DDTHH:MM:SSZ" = 20 chars
        assert_eq!(ts.len(), 20, "timestamp should be 20 characters: {ts}");
        assert!(ts.ends_with('Z'), "timestamp must end with Z");
        assert_eq!(&ts[4..5], "-", "separator at pos 4 should be -");
        assert_eq!(&ts[7..8], "-", "separator at pos 7 should be -");
        assert_eq!(&ts[10..11], "T", "T separator at pos 10");
        assert_eq!(&ts[13..14], ":", "colon at pos 13");
        assert_eq!(&ts[16..17], ":", "colon at pos 16");
    }

    // ── Silent failure: log path non-existent parent ──────────────────────────

    #[test]
    fn test_log_to_non_existent_parent_creates_dir() {
        let dir = unique_test_dir("mkdir");
        // Do NOT pre-create the dir — the module should create it.
        let log_file = dir.join("sub").join("vibestats.log");

        // Should not panic, and should create the file.
        let _ = write_log_entry(&log_file, "INFO", "dir created");

        assert!(log_file.exists(), "log file should be created even if parent was absent");

        let _ = fs::remove_dir_all(&dir);
    }

}
