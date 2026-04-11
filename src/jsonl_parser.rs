#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, BufReader};

/// Per-day aggregated session activity.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DailyActivity {
    pub sessions: u32,
    pub active_minutes: u32,
}

/// Internal struct for deserializing individual JSONL lines.
/// All fields are optional and default to `None` so that unknown
/// or missing fields are silently tolerated (NFR14).
#[derive(Debug, Default, Deserialize)]
struct ClaudeEntry {
    /// Entry type: "assistant", "user", "system", "attachment", etc.
    #[serde(rename = "type", default)]
    entry_type: Option<String>,

    /// Entry subtype — "turn_duration" for the session summary entry.
    #[serde(default)]
    subtype: Option<String>,

    /// ISO 8601 UTC timestamp with milliseconds: "2026-04-01T15:03:39.992Z"
    #[serde(default)]
    timestamp: Option<String>,

    /// Session duration in milliseconds — only present on type=system, subtype=turn_duration.
    /// JSON field name is "durationMs" (camelCase).
    #[serde(rename = "durationMs", default)]
    duration_ms: Option<u64>,
}

/// Return the path to `~/.claude/projects` using `HOME` env var.
/// Returns `None` if `HOME` is not set.
fn claude_projects_dir() -> Option<std::path::PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(|h| std::path::PathBuf::from(h).join(".claude").join("projects"))
}

/// Recursively collect all `*.jsonl` files under `dir` into `acc`.
/// Unreadable directories are skipped silently (NFR10).
fn collect_jsonl_files(dir: &std::path::Path, acc: &mut Vec<std::path::PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return, // unreadable directory — skip silently
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_jsonl_files(&path, acc);
        } else if path.extension().and_then(|e| e.to_str()) == Some("jsonl") {
            acc.push(path);
        }
    }
}

/// Parse a single JSONL file (one session) and accumulate its activity
/// into `result` if its date falls within `[start, end]` (inclusive).
///
/// - Unreadable files are skipped silently.
/// - Malformed JSON lines are skipped silently (NFR14).
/// - One file = one session (+1 to `sessions`).
fn parse_file(
    path: &std::path::Path,
    start: &str,
    end: &str,
    result: &mut HashMap<String, DailyActivity>,
) {
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return, // unreadable file — skip silently
    };

    let mut session_date: Option<String> = None;
    let mut duration_ms: u64 = 0;

    for line in BufReader::new(file).lines().map_while(Result::ok) {
        let entry: ClaudeEntry = match serde_json::from_str(&line) {
            Ok(e) => e,
            Err(_) => continue, // malformed line — skip silently (NFR14)
        };

        // Capture session date from the first timestamp we encounter.
        if session_date.is_none() {
            if let Some(ts) = &entry.timestamp {
                if let Some(date) = ts.get(..10) {
                    session_date = Some(date.to_string());
                }
            }
        }

        // Capture duration from the turn_duration system entry.
        if entry.entry_type.as_deref() == Some("system")
            && entry.subtype.as_deref() == Some("turn_duration")
        {
            if let Some(ms) = entry.duration_ms {
                duration_ms = ms;
            }
        }
    }

    // Count this file as one session for its date (if in range).
    if let Some(date) = session_date {
        if date.as_str() >= start && date.as_str() <= end {
            let activity = result.entry(date).or_default();
            activity.sessions += 1;
            activity.active_minutes += (duration_ms / 60_000) as u32;
        }
    }
}

/// Walk `~/.claude/projects/**/*.jsonl` and aggregate per-day session activity
/// for dates in `[start, end]` inclusive (YYYY-MM-DD strings).
///
/// Returns an empty map if the directory is missing or unreadable.
pub fn parse_date_range(start: &str, end: &str) -> HashMap<String, DailyActivity> {
    let mut result: HashMap<String, DailyActivity> = HashMap::new();

    let projects_dir = match claude_projects_dir() {
        Some(p) => p,
        None => return result,
    };

    let mut jsonl_files: Vec<std::path::PathBuf> = Vec::new();
    collect_jsonl_files(&projects_dir, &mut jsonl_files);

    for path in &jsonl_files {
        parse_file(path, start, end, &mut result);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Write JSONL content to a unique temp file and return the path.
    /// Caller is responsible for cleanup.
    fn write_temp_jsonl(lines: &[&str]) -> std::path::PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let name = format!(
            "vibestats_test_{}_{}_{}.jsonl",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
            n
        );
        let path = std::env::temp_dir().join(name);
        let mut f = std::fs::File::create(&path).unwrap();
        for line in lines {
            writeln!(f, "{}", line).unwrap();
        }
        path
    }

    // Minimal valid JSONL session: one assistant entry with timestamp,
    // one system/turn_duration entry with durationMs.
    const SAMPLE_VALID: &[&str] = &[
        r#"{"type":"assistant","timestamp":"2026-04-10T14:00:00.000Z","uuid":"aaa","sessionId":"s1"}"#,
        r#"{"type":"system","subtype":"turn_duration","durationMs":3600000,"timestamp":"2026-04-10T15:00:00.000Z","uuid":"bbb","sessionId":"s1"}"#,
    ];

    #[test]
    fn valid_file_within_range_accumulates() {
        let path = write_temp_jsonl(SAMPLE_VALID);
        let mut result = HashMap::new();
        parse_file(&path, "2026-04-10", "2026-04-10", &mut result);
        let _ = std::fs::remove_file(&path);
        let day = result.get("2026-04-10").expect("date must be present");
        assert_eq!(day.sessions, 1);
        assert_eq!(day.active_minutes, 60); // 3_600_000 ms / 60_000 = 60
    }

    #[test]
    fn file_outside_range_not_included() {
        let path = write_temp_jsonl(SAMPLE_VALID);
        let mut result = HashMap::new();
        parse_file(&path, "2026-04-11", "2026-04-11", &mut result);
        let _ = std::fs::remove_file(&path);
        assert!(result.is_empty());
    }

    #[test]
    fn unknown_fields_silently_ignored() {
        let lines = &[
            r#"{"type":"assistant","timestamp":"2026-04-10T14:00:00.000Z","unknownField":"xyz","uuid":"x"}"#,
            r#"{"type":"system","subtype":"turn_duration","durationMs":60000,"timestamp":"2026-04-10T14:01:00.000Z","extraKey":42}"#,
        ];
        let path = write_temp_jsonl(lines);
        let mut result = HashMap::new();
        parse_file(&path, "2026-04-10", "2026-04-10", &mut result);
        let _ = std::fs::remove_file(&path);
        let day = result.get("2026-04-10").expect("date must be present");
        assert_eq!(day.sessions, 1);
        assert_eq!(day.active_minutes, 1); // 60_000 ms / 60_000 = 1
    }

    #[test]
    fn malformed_lines_skipped_file_still_counted() {
        let lines = &[
            "NOT VALID JSON AT ALL {{{",
            r#"{"type":"assistant","timestamp":"2026-04-10T14:00:00.000Z","uuid":"x"}"#,
        ];
        let path = write_temp_jsonl(lines);
        let mut result = HashMap::new();
        parse_file(&path, "2026-04-10", "2026-04-10", &mut result);
        let _ = std::fs::remove_file(&path);
        let day = result
            .get("2026-04-10")
            .expect("date found from valid line");
        assert_eq!(day.sessions, 1);
        assert_eq!(day.active_minutes, 0); // no turn_duration entry
    }

    #[test]
    fn empty_file_returns_no_entry() {
        let path = write_temp_jsonl(&[]);
        let mut result = HashMap::new();
        parse_file(&path, "2026-04-10", "2026-04-10", &mut result);
        let _ = std::fs::remove_file(&path);
        assert!(result.is_empty());
    }

    #[test]
    fn two_files_same_date_accumulate() {
        let path1 = write_temp_jsonl(SAMPLE_VALID);
        let path2 = write_temp_jsonl(SAMPLE_VALID);
        let mut result = HashMap::new();
        parse_file(&path1, "2026-04-10", "2026-04-10", &mut result);
        parse_file(&path2, "2026-04-10", "2026-04-10", &mut result);
        let _ = std::fs::remove_file(&path1);
        let _ = std::fs::remove_file(&path2);
        let day = result.get("2026-04-10").expect("date present");
        assert_eq!(day.sessions, 2);
        assert_eq!(day.active_minutes, 120); // 60 + 60
    }

    #[test]
    fn file_without_turn_duration_active_minutes_zero() {
        let lines = &[
            r#"{"type":"assistant","timestamp":"2026-04-10T14:00:00.000Z","uuid":"x"}"#,
            r#"{"type":"user","timestamp":"2026-04-10T14:01:00.000Z","uuid":"y"}"#,
        ];
        let path = write_temp_jsonl(lines);
        let mut result = HashMap::new();
        parse_file(&path, "2026-04-10", "2026-04-10", &mut result);
        let _ = std::fs::remove_file(&path);
        let day = result.get("2026-04-10").expect("date present");
        assert_eq!(day.sessions, 1);
        assert_eq!(day.active_minutes, 0);
    }
}
