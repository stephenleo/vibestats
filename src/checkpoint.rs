#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Checkpoint {
    /// ISO 8601 UTC string; absent means no sync has run yet
    pub throttle_timestamp: Option<String>,
    /// "active" | "retired" | "purged"; defaults to "active"
    #[serde(default = "default_machine_status")]
    pub machine_status: String,
    /// true when last GitHub API call returned 401
    #[serde(default)]
    pub auth_error: bool,
    /// Per-date SHA256 content hashes; key = "YYYY-MM-DD"
    #[serde(default)]
    pub date_hashes: HashMap<String, String>,
}

fn default_machine_status() -> String {
    "active".to_string()
}

impl Default for Checkpoint {
    fn default() -> Self {
        Self {
            throttle_timestamp: None,
            machine_status: default_machine_status(),
            auth_error: false,
            date_hashes: HashMap::new(),
        }
    }
}

/// Parses "YYYY-MM-DDTHH:MM:SSZ" → SystemTime (UNIX_EPOCH offset).
/// Returns None on any parse error (fail-open: caller treats as no timestamp).
/// Strictly requires a trailing 'Z' and rejects out-of-range fields and pre-1970 dates.
fn parse_iso8601_utc(s: &str) -> Option<std::time::SystemTime> {
    // Require explicit trailing Z — we never want to misinterpret a naive local
    // timestamp as UTC. The schema in docs/schemas.md requires the Z suffix.
    let s = s.strip_suffix('Z')?;
    let (date_str, time_str) = s.split_once('T')?;
    let mut dp = date_str.split('-');
    let year: u64 = dp.next()?.parse().ok()?;
    let month: u64 = dp.next()?.parse().ok()?;
    let day: u64 = dp.next()?.parse().ok()?;
    if dp.next().is_some() {
        return None; // extra '-' segments
    }
    let mut tp = time_str.split(':');
    let hour: u64 = tp.next()?.parse().ok()?;
    let min: u64 = tp.next()?.parse().ok()?;
    let sec: u64 = tp.next()?.parse().ok()?;
    if tp.next().is_some() {
        return None; // extra ':' segments (e.g. fractional or offset)
    }

    // Validate ranges before the civil-date math. The u64 arithmetic below
    // relies on the subtraction `- 719468` not underflowing, which requires
    // year >= 1970 (actually >= March 1970, but 1970 is a safe lower bound).
    if year < 1970 || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    if hour >= 24 || min >= 60 || sec >= 60 {
        return None;
    }

    // Days since Unix epoch (1970-01-01) via the civil date formula.
    // Source: https://howardhinnant.github.io/date_algorithms.html  (days_from_civil)
    let y = if month <= 2 { year - 1 } else { year };
    let era = y / 400;
    let yoe = y - era * 400; // [0, 399]
    let doy = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + day - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
                                                     // days since 1970-01-01; safe: year >= 1970 ⇒ era*146097 + doe >= 719468.
    let days_since_epoch = era * 146097 + doe - 719468;

    let secs = days_since_epoch * 86400 + hour * 3600 + min * 60 + sec;
    Some(std::time::UNIX_EPOCH + std::time::Duration::from_secs(secs))
}

/// Formats a SystemTime as "YYYY-MM-DDTHH:MM:SSZ".
fn format_iso8601_utc(t: std::time::SystemTime) -> String {
    let secs = t
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Reverse civil date formula (days_from_civil inverse: civil_from_days)
    // Source: https://howardhinnant.github.io/date_algorithms.html
    let z = secs / 86400;
    let time_of_day = secs % 86400;
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

    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, mo, d, h, m, s)
}

impl Checkpoint {
    /// Load checkpoint from file; returns default if file is missing or unreadable.
    /// Never panics or exits non-zero on any error (NFR10).
    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Save checkpoint to file, creating parent directories as needed.
    /// Writes atomically: serializes to a sibling `.tmp` file, then renames
    /// over the target path. This prevents a crash mid-write from producing a
    /// truncated/corrupt checkpoint file (which would lose throttle state and
    /// risk NFR2/NFR12 violations on the next run).
    /// Returns Result — callers decide whether to log or silently ignore.
    pub fn save(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string(self)?;
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        // Build a sibling temp path: "<path>.tmp". Using a sibling (same dir)
        // guarantees the rename is atomic on POSIX (same filesystem).
        let mut tmp_os = path.as_os_str().to_os_string();
        tmp_os.push(".tmp");
        let tmp_path = std::path::PathBuf::from(tmp_os);
        std::fs::write(&tmp_path, content)?;
        std::fs::rename(&tmp_path, path)?;
        Ok(())
    }

    /// Returns true if throttle_timestamp is within 5 minutes of now.
    /// Returns false if throttle_timestamp is absent or unparseable (fail-safe: allow sync).
    pub fn should_throttle(&self) -> bool {
        let ts_str = match &self.throttle_timestamp {
            Some(ts) => ts,
            None => return false, // no timestamp → allow sync
        };
        let ts = match parse_iso8601_utc(ts_str) {
            Some(t) => t,
            None => return false, // unparseable → fail-safe: allow sync
        };
        let now = std::time::SystemTime::now();
        match now.duration_since(ts) {
            Ok(elapsed) => elapsed.as_secs() < 300, // 5 min = 300 s
            Err(_) => false,                        // clock skew (ts in future) → allow sync
        }
    }

    /// Sets throttle_timestamp to current UTC time as ISO 8601 string.
    pub fn update_throttle_timestamp(&mut self) {
        self.throttle_timestamp = Some(format_iso8601_utc(std::time::SystemTime::now()));
    }

    /// Returns true if stored hash for date equals provided hash.
    pub fn hash_matches(&self, date: &str, hash: &str) -> bool {
        self.date_hashes
            .get(date)
            .map(|stored| stored == hash)
            .unwrap_or(false)
    }

    /// Upserts entry in date_hashes for the given date.
    pub fn update_hash(&mut self, date: &str, hash: &str) {
        self.date_hashes.insert(date.to_string(), hash.to_string());
    }

    /// Sets auth_error to true.
    pub fn set_auth_error(&mut self) {
        self.auth_error = true;
    }

    /// Sets auth_error to false.
    pub fn clear_auth_error(&mut self) {
        self.auth_error = false;
    }

    /// Returns true when machine_status == "retired".
    pub fn is_retired(&self) -> bool {
        self.machine_status == "retired"
    }

    /// Sets machine_status to the given value.
    pub fn set_machine_status(&mut self, status: &str) {
        self.machine_status = status.to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Produce a unique temp path per call: combines process id, test name,
    /// and a monotonic nanosecond counter. This prevents collisions between
    /// parallel test runs (cargo test is multi-threaded by default) and
    /// between concurrent repo checkouts on the same machine.
    fn temp_path(name: &str) -> PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let pid = std::process::id();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("vibestats_{name}_{pid}_{nanos}_{seq}.toml"))
    }

    #[test]
    fn load_missing_file_returns_default() {
        let path = temp_path("missing");
        // Ensure file really does not exist (temp_path is unique, but be defensive).
        let _ = std::fs::remove_file(&path);
        let cp = Checkpoint::load(&path);
        assert!(!cp.auth_error);
        assert_eq!(cp.machine_status, "active");
        assert!(cp.date_hashes.is_empty());
    }

    #[test]
    fn save_load_roundtrip() {
        let path = temp_path("roundtrip");
        let mut cp = Checkpoint::default();
        cp.set_auth_error();
        cp.update_hash("2026-04-10", "abc123");
        cp.save(&path).unwrap();
        let loaded = Checkpoint::load(&path);
        assert!(loaded.auth_error);
        assert!(loaded.hash_matches("2026-04-10", "abc123"));
        let _ = std::fs::remove_file(&path); // cleanup
    }

    #[test]
    fn should_throttle_recent_timestamp() {
        let mut cp = Checkpoint::default();
        cp.update_throttle_timestamp(); // sets to now
        assert!(cp.should_throttle()); // just set — must be throttled
    }

    #[test]
    fn should_throttle_old_timestamp_returns_false() {
        // Use struct-update syntax to avoid clippy::field_reassign_with_default.
        let cp = Checkpoint {
            throttle_timestamp: Some("2020-01-01T00:00:00Z".to_string()),
            ..Default::default()
        };
        assert!(!cp.should_throttle());
    }

    #[test]
    fn hash_matches_correct_and_incorrect() {
        let mut cp = Checkpoint::default();
        cp.update_hash("2026-04-10", "deadbeef");
        assert!(cp.hash_matches("2026-04-10", "deadbeef"));
        assert!(!cp.hash_matches("2026-04-10", "wronghash"));
        assert!(!cp.hash_matches("2026-04-11", "deadbeef")); // different date
    }

    #[test]
    fn auth_error_roundtrip() {
        let mut cp = Checkpoint::default();
        assert!(!cp.auth_error);
        cp.set_auth_error();
        assert!(cp.auth_error);
        cp.clear_auth_error();
        assert!(!cp.auth_error);
    }

    #[test]
    fn is_retired_variants() {
        let mut cp = Checkpoint::default();
        assert!(!cp.is_retired()); // default is "active"
        cp.set_machine_status("retired");
        assert!(cp.is_retired());
        cp.set_machine_status("purged");
        assert!(!cp.is_retired()); // purged is not the same as retired
        cp.set_machine_status("active");
        assert!(!cp.is_retired());
    }

    #[test]
    fn parse_iso8601_rejects_missing_z() {
        // Naive-looking timestamps without Z must be rejected so we never
        // misinterpret a local-time string as UTC.
        assert!(parse_iso8601_utc("2026-04-10T14:23:00").is_none());
    }

    #[test]
    fn parse_iso8601_rejects_out_of_range() {
        assert!(parse_iso8601_utc("2026-13-01T00:00:00Z").is_none()); // month 13
        assert!(parse_iso8601_utc("2026-00-01T00:00:00Z").is_none()); // month 0
        assert!(parse_iso8601_utc("2026-04-32T00:00:00Z").is_none()); // day 32
        assert!(parse_iso8601_utc("2026-04-00T00:00:00Z").is_none()); // day 0
        assert!(parse_iso8601_utc("2026-04-10T24:00:00Z").is_none()); // hour 24
        assert!(parse_iso8601_utc("2026-04-10T00:60:00Z").is_none()); // min 60
        assert!(parse_iso8601_utc("2026-04-10T00:00:60Z").is_none()); // sec 60
    }

    #[test]
    fn parse_iso8601_rejects_pre_1970() {
        // Must reject pre-epoch dates to prevent u64 underflow in the
        // civil-date arithmetic.
        assert!(parse_iso8601_utc("1969-12-31T23:59:59Z").is_none());
        assert!(parse_iso8601_utc("0001-01-01T00:00:00Z").is_none());
    }

    #[test]
    fn format_parse_iso8601_roundtrip() {
        // Formatting a SystemTime and parsing the result must yield the same
        // second-granularity instant. Guards against drift between the
        // civil-date and civil-from-days formulas.
        let now = std::time::SystemTime::now();
        let formatted = format_iso8601_utc(now);
        let parsed = parse_iso8601_utc(&formatted).expect("format output must parse");
        let now_secs = now.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
        let parsed_secs = parsed
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert_eq!(now_secs, parsed_secs);
    }

    #[test]
    fn save_is_atomic_no_tmp_left_behind() {
        // After a successful save, no sibling .tmp file should remain.
        let path = temp_path("atomic");
        let cp = Checkpoint::default();
        cp.save(&path).unwrap();
        let mut tmp_os = path.as_os_str().to_os_string();
        tmp_os.push(".tmp");
        let tmp_path = std::path::PathBuf::from(tmp_os);
        assert!(!tmp_path.exists(), "temp file must be renamed away");
        assert!(path.exists(), "target file must exist");
        let _ = std::fs::remove_file(&path);
    }
}
