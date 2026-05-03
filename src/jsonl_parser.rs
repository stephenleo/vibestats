use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::io::{BufRead, BufReader};

/// Per-day aggregated session activity.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DailyActivity {
    pub sessions: u32,
    pub active_minutes: u32,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_creation_tokens: u64,
    /// Maps model name → total output tokens attributed to that model on this day.
    /// BTreeMap ensures deterministic serialization order (alphabetical keys).
    pub models: BTreeMap<String, u64>,
    pub longest_session_minutes: u32,
    pub message_count: u32,
    pub tool_uses: u32,
}

/// Token usage from an assistant message's `usage` object.
#[derive(Debug, Default, Deserialize)]
struct MessageUsage {
    #[serde(default)]
    input_tokens: Option<u64>,
    #[serde(default)]
    output_tokens: Option<u64>,
    #[serde(rename = "cache_read_input_tokens", default)]
    cache_read_tokens: Option<u64>,
    #[serde(rename = "cache_creation_input_tokens", default)]
    cache_creation_tokens: Option<u64>,
}

/// A single content block inside an assistant message.
#[derive(Debug, Default, Deserialize)]
struct ContentBlock {
    #[serde(rename = "type", default)]
    block_type: Option<String>,
}

/// The `message` field on an assistant JSONL entry.
#[derive(Debug, Default, Deserialize)]
struct AssistantMessage {
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    usage: Option<MessageUsage>,
    #[serde(default)]
    content: Option<Vec<ContentBlock>>,
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

    /// Assistant message object — only present on type=assistant entries.
    #[serde(default)]
    message: Option<AssistantMessage>,

    /// Total message count for the session — present on type=system, subtype=turn_duration.
    /// JSON field name is "messageCount" (camelCase).
    #[serde(rename = "messageCount", default)]
    message_count: Option<u32>,
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
    let mut input_tokens: u64 = 0;
    let mut output_tokens: u64 = 0;
    let mut cache_read_tokens: u64 = 0;
    let mut cache_creation_tokens: u64 = 0;
    let mut session_models: BTreeMap<String, u64> = BTreeMap::new();
    let mut message_count: u32 = 0;
    let mut tool_uses: u32 = 0;

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

        // Accumulate token usage and model stats from assistant entries.
        if entry.entry_type.as_deref() == Some("assistant") {
            if let Some(msg) = &entry.message {
                if let Some(usage) = &msg.usage {
                    input_tokens += usage.input_tokens.unwrap_or(0);
                    let out = usage.output_tokens.unwrap_or(0);
                    output_tokens += out;
                    cache_read_tokens += usage.cache_read_tokens.unwrap_or(0);
                    cache_creation_tokens += usage.cache_creation_tokens.unwrap_or(0);

                    // Tally output tokens per model.
                    if let Some(model) = &msg.model {
                        *session_models.entry(model.clone()).or_insert(0) += out;
                    }
                }

                // Count tool_use content blocks.
                if let Some(blocks) = &msg.content {
                    for block in blocks {
                        if block.block_type.as_deref() == Some("tool_use") {
                            tool_uses += 1;
                        }
                    }
                }
            }
        }

        // Capture duration and message count from the turn_duration system entry.
        if entry.entry_type.as_deref() == Some("system")
            && entry.subtype.as_deref() == Some("turn_duration")
        {
            if let Some(ms) = entry.duration_ms {
                duration_ms = ms;
            }
            if let Some(mc) = entry.message_count {
                message_count = mc;
            }
        }
    }

    // Count this file as one session for its date (if in range).
    if let Some(date) = session_date {
        if date.as_str() >= start && date.as_str() <= end {
            let activity = result.entry(date).or_default();
            activity.sessions += 1;
            let session_minutes = (duration_ms / 60_000) as u32;
            activity.active_minutes += session_minutes;
            activity.input_tokens += input_tokens;
            activity.output_tokens += output_tokens;
            activity.cache_read_tokens += cache_read_tokens;
            activity.cache_creation_tokens += cache_creation_tokens;
            activity.message_count += message_count;
            activity.tool_uses += tool_uses;
            for (model, count) in session_models {
                *activity.models.entry(model).or_insert(0) += count;
            }
            // Track longest single session on this day.
            if session_minutes > activity.longest_session_minutes {
                activity.longest_session_minutes = session_minutes;
            }
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

    #[test]
    fn assistant_entry_with_usage_accumulates_tokens() {
        let lines = &[
            r#"{"type":"assistant","timestamp":"2026-04-10T14:00:00.000Z","uuid":"a","message":{"model":"claude-sonnet-4-5","usage":{"input_tokens":100,"output_tokens":50,"cache_read_input_tokens":20,"cache_creation_input_tokens":10},"content":[{"type":"text"},{"type":"tool_use"}]}}"#,
            r#"{"type":"system","subtype":"turn_duration","durationMs":120000,"messageCount":4,"timestamp":"2026-04-10T14:02:00.000Z","uuid":"b"}"#,
        ];
        let path = write_temp_jsonl(lines);
        let mut result = HashMap::new();
        parse_file(&path, "2026-04-10", "2026-04-10", &mut result);
        let _ = std::fs::remove_file(&path);
        let day = result.get("2026-04-10").expect("date present");
        assert_eq!(day.sessions, 1);
        assert_eq!(day.active_minutes, 2); // 120_000 ms / 60_000 = 2
        assert_eq!(day.input_tokens, 100);
        assert_eq!(day.output_tokens, 50);
        assert_eq!(day.cache_read_tokens, 20);
        assert_eq!(day.cache_creation_tokens, 10);
        assert_eq!(day.message_count, 4);
        assert_eq!(day.tool_uses, 1);
        assert_eq!(
            day.models.get("claude-sonnet-4-5").copied().unwrap_or(0),
            50
        );
    }

    #[test]
    fn old_format_entry_no_message_field_still_counted() {
        // Old JSONL entries without a `message` field must still count sessions/minutes.
        let lines = &[
            r#"{"type":"assistant","timestamp":"2026-04-10T14:00:00.000Z","uuid":"x","sessionId":"s1"}"#,
            r#"{"type":"system","subtype":"turn_duration","durationMs":60000,"timestamp":"2026-04-10T14:01:00.000Z","uuid":"y"}"#,
        ];
        let path = write_temp_jsonl(lines);
        let mut result = HashMap::new();
        parse_file(&path, "2026-04-10", "2026-04-10", &mut result);
        let _ = std::fs::remove_file(&path);
        let day = result.get("2026-04-10").expect("date present");
        assert_eq!(day.sessions, 1);
        assert_eq!(day.active_minutes, 1);
        assert_eq!(day.input_tokens, 0);
        assert_eq!(day.output_tokens, 0);
        assert!(day.models.is_empty());
        assert_eq!(day.tool_uses, 0);
    }

    #[test]
    fn longest_session_minutes_tracks_max_across_sessions() {
        // Two files on the same day: 60 min and 120 min. longest_session_minutes = 120.
        let long_session = &[
            r#"{"type":"assistant","timestamp":"2026-04-10T10:00:00.000Z","uuid":"a"}"#,
            r#"{"type":"system","subtype":"turn_duration","durationMs":7200000,"timestamp":"2026-04-10T12:00:00.000Z","uuid":"b"}"#,
        ];
        let short_session = &[
            r#"{"type":"assistant","timestamp":"2026-04-10T13:00:00.000Z","uuid":"c"}"#,
            r#"{"type":"system","subtype":"turn_duration","durationMs":3600000,"timestamp":"2026-04-10T14:00:00.000Z","uuid":"d"}"#,
        ];
        let p1 = write_temp_jsonl(long_session);
        let p2 = write_temp_jsonl(short_session);
        let mut result = HashMap::new();
        parse_file(&p1, "2026-04-10", "2026-04-10", &mut result);
        parse_file(&p2, "2026-04-10", "2026-04-10", &mut result);
        let _ = std::fs::remove_file(&p1);
        let _ = std::fs::remove_file(&p2);
        let day = result.get("2026-04-10").expect("date present");
        assert_eq!(day.sessions, 2);
        assert_eq!(day.active_minutes, 180); // 120 + 60
        assert_eq!(day.longest_session_minutes, 120); // max, not sum
    }

    #[test]
    fn models_accumulated_across_multiple_assistant_entries() {
        // Two assistant entries with different models in one session.
        let lines = &[
            r#"{"type":"assistant","timestamp":"2026-04-10T14:00:00.000Z","uuid":"a","message":{"model":"claude-sonnet-4-5","usage":{"output_tokens":30},"content":[]}}"#,
            r#"{"type":"assistant","timestamp":"2026-04-10T14:01:00.000Z","uuid":"b","message":{"model":"claude-opus-4","usage":{"output_tokens":70},"content":[]}}"#,
            r#"{"type":"system","subtype":"turn_duration","durationMs":60000,"uuid":"c","timestamp":"2026-04-10T14:02:00.000Z"}"#,
        ];
        let path = write_temp_jsonl(lines);
        let mut result = HashMap::new();
        parse_file(&path, "2026-04-10", "2026-04-10", &mut result);
        let _ = std::fs::remove_file(&path);
        let day = result.get("2026-04-10").expect("date present");
        assert_eq!(
            day.models.get("claude-sonnet-4-5").copied().unwrap_or(0),
            30
        );
        assert_eq!(day.models.get("claude-opus-4").copied().unwrap_or(0), 70);
        assert_eq!(day.output_tokens, 100);
    }
}
