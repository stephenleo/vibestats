//! Codex harness: parses `~/.codex/sessions/**/*.jsonl` rollout files.

use crate::harnesses::{DailyActivity, Harness};
use serde::Deserialize;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

pub struct Codex;

impl Harness for Codex {
    fn id(&self) -> &'static str {
        "codex"
    }

    fn display_name(&self) -> &'static str {
        "Codex"
    }

    fn is_installed(&self) -> bool {
        codex_sessions_dir().is_some_and(|p| p.is_dir())
    }

    fn parse_date_range(&self, start: &str, end: &str) -> HashMap<String, DailyActivity> {
        parse_date_range(start, end)
    }
}

// ── Private serde types — moved verbatim from src/codex_parser.rs ───────────

#[derive(Debug, Default, Clone, Copy, Deserialize)]
struct CodexTokenUsage {
    #[serde(default)]
    input_tokens: u64,
    #[serde(default)]
    cached_input_tokens: u64,
    #[serde(default)]
    output_tokens: u64,
}

#[derive(Debug, Default, Deserialize)]
struct CodexTokenInfo {
    #[serde(default)]
    last_token_usage: Option<CodexTokenUsage>,
}

#[derive(Debug, Default, Deserialize)]
struct CodexPayload {
    #[serde(rename = "type", default)]
    payload_type: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    current_date: Option<String>,
    #[serde(default)]
    info: Option<CodexTokenInfo>,
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct CodexEntry {
    #[serde(default)]
    timestamp: Option<String>,
    #[serde(rename = "type", default)]
    entry_type: Option<String>,
    #[serde(default)]
    payload: Option<CodexPayload>,
}

#[derive(Debug, Default)]
struct CodexSessionDay {
    activity: DailyActivity,
    first_ts: Option<i64>,
    last_ts: Option<i64>,
}

// ── Private helpers — bodies moved verbatim from src/codex_parser.rs ────────

fn codex_sessions_dir() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(|h| PathBuf::from(h).join(".codex").join("sessions"))
}

fn collect_jsonl_files(dir: &Path, acc: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
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

fn date_from_session_path(path: &Path) -> Option<String> {
    let parts: Vec<String> = path
        .components()
        .filter_map(|c| c.as_os_str().to_str().map(String::from))
        .collect();

    let sessions_idx = parts.iter().position(|p| p == "sessions")?;
    let year = parts.get(sessions_idx + 1)?;
    let month = parts.get(sessions_idx + 2)?;
    let day = parts.get(sessions_idx + 3)?;

    if year.len() == 4 && month.len() == 2 && day.len() == 2 {
        Some(format!("{year}-{month}-{day}"))
    } else {
        None
    }
}

fn parse_u64(s: &str) -> Option<u64> {
    s.parse::<u64>().ok()
}

fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = year - i64::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let month_prime = month + if month > 2 { -3 } else { 9 };
    let doy = (153 * month_prime + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn unix_seconds_from_utc_timestamp(ts: &str) -> Option<i64> {
    if ts.len() < 20 || !ts.ends_with('Z') {
        return None;
    }

    let year = parse_u64(ts.get(0..4)?)? as i64;
    let month = parse_u64(ts.get(5..7)?)? as i64;
    let day = parse_u64(ts.get(8..10)?)? as i64;
    let hour = parse_u64(ts.get(11..13)?)? as i64;
    let minute = parse_u64(ts.get(14..16)?)? as i64;
    let second = parse_u64(ts.get(17..19)?)? as i64;

    if !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || hour > 23
        || minute > 59
        || second > 60
    {
        return None;
    }

    Some(days_from_civil(year, month, day) * 86_400 + hour * 3_600 + minute * 60 + second)
}

fn parse_file(path: &Path, start: &str, end: &str, result: &mut HashMap<String, DailyActivity>) {
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return,
    };

    let path_date = date_from_session_path(path);
    let mut active_date = path_date.clone();
    let mut current_model: Option<String> = None;
    let mut session_days: HashMap<String, CodexSessionDay> = HashMap::new();

    for line in BufReader::new(file).lines().map_while(Result::ok) {
        let entry: CodexEntry = match serde_json::from_str(&line) {
            Ok(e) => e,
            Err(_) => continue,
        };

        let Some(payload) = &entry.payload else {
            continue;
        };

        if let Some(date) = payload
            .current_date
            .as_ref()
            .filter(|d| d.len() == 10 && !d.trim().is_empty())
        {
            active_date = Some(date.clone());
        }
        if let Some(model) = payload.model.as_ref().filter(|m| !m.trim().is_empty()) {
            current_model = Some(model.clone());
        }

        let entry_date = active_date
            .clone()
            .or_else(|| {
                entry
                    .timestamp
                    .as_ref()
                    .and_then(|ts| ts.get(..10).map(String::from))
            })
            .or_else(|| path_date.clone());
        let Some(date) = entry_date else {
            continue;
        };

        let day = session_days.entry(date).or_default();
        if let Some(ts) = entry
            .timestamp
            .as_deref()
            .and_then(unix_seconds_from_utc_timestamp)
        {
            day.first_ts = Some(day.first_ts.map_or(ts, |prev| prev.min(ts)));
            day.last_ts = Some(day.last_ts.map_or(ts, |prev| prev.max(ts)));
        }

        match (entry.entry_type.as_deref(), payload.payload_type.as_deref()) {
            (Some("event_msg"), Some("user_message")) => {
                day.activity.message_count += 1;
            }
            (Some("response_item"), Some("function_call")) if payload.name.is_some() => {
                day.activity.tool_uses += 1;
            }
            (Some("event_msg"), Some("token_count")) => {
                let Some(usage) = payload.info.as_ref().and_then(|info| info.last_token_usage)
                else {
                    continue;
                };

                let cached = usage.cached_input_tokens.min(usage.input_tokens);
                day.activity.input_tokens += usage.input_tokens - cached;
                day.activity.cache_read_tokens += cached;
                day.activity.output_tokens += usage.output_tokens;

                if let Some(model) = &current_model {
                    *day.activity.models.entry(model.clone()).or_insert(0) += usage.output_tokens;
                }
            }
            _ => {}
        }
    }

    for (date, mut session_day) in session_days {
        if date.as_str() < start || date.as_str() > end {
            continue;
        }

        let session_minutes = match (session_day.first_ts, session_day.last_ts) {
            (Some(first), Some(last)) if last > first => ((last - first) / 60) as u32,
            _ => 0,
        };

        let activity = result.entry(date).or_default();
        activity.sessions += 1;
        activity.active_minutes += session_minutes;
        activity.input_tokens += session_day.activity.input_tokens;
        activity.output_tokens += session_day.activity.output_tokens;
        activity.cache_read_tokens += session_day.activity.cache_read_tokens;
        activity.message_count += session_day.activity.message_count;
        activity.tool_uses += session_day.activity.tool_uses;
        for (model, count) in std::mem::take(&mut session_day.activity.models) {
            *activity.models.entry(model).or_insert(0) += count;
        }
        if session_minutes > activity.longest_session_minutes {
            activity.longest_session_minutes = session_minutes;
        }
    }
}

/// Read Codex rollout files and aggregate per-day session activity.
///
/// Codex rollouts expose per-turn `last_token_usage` counters. We group those
/// counters by the local `current_date` emitted in each turn context, matching
/// the day boundaries used by Codex usage tools.
fn parse_date_range(start: &str, end: &str) -> HashMap<String, DailyActivity> {
    let mut result: HashMap<String, DailyActivity> = HashMap::new();

    let sessions_dir = match codex_sessions_dir() {
        Some(p) => p,
        None => return result,
    };

    let mut jsonl_files = Vec::new();
    collect_jsonl_files(&sessions_dir, &mut jsonl_files);

    for path in &jsonl_files {
        parse_file(path, start, end, &mut result);
    }

    result
}

// ── Tests — moved verbatim from src/codex_parser.rs ─────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp_jsonl(lines: &[&str]) -> PathBuf {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!(
            "vibestats_codex_test_{}_{}_{}.jsonl",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            n
        ));
        let mut f = std::fs::File::create(&path).unwrap();
        for line in lines {
            writeln!(f, "{line}").unwrap();
        }
        path
    }

    #[test]
    fn date_from_session_path_uses_local_codex_path_date() {
        let path =
            Path::new("/Users/me/.codex/sessions/2026/05/03/rollout-2026-05-03T14-48-20-id.jsonl");
        assert_eq!(date_from_session_path(path), Some("2026-05-03".to_string()));
    }

    #[test]
    fn utc_timestamp_parser_handles_fractional_seconds() {
        assert_eq!(
            unix_seconds_from_utc_timestamp("2026-05-03T06:48:55.745Z"),
            Some(1_777_790_935)
        );
    }

    #[test]
    fn parse_file_accumulates_last_token_usage() {
        let lines = &[
            r#"{"timestamp":"2026-05-03T06:48:55.000Z","type":"turn_context","payload":{"model":"gpt-5.5"}}"#,
            r#"{"timestamp":"2026-05-03T06:49:04.000Z","type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":100,"cached_input_tokens":40,"output_tokens":10,"total_tokens":110}}}}"#,
            r#"{"timestamp":"2026-05-03T06:49:05.000Z","type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":150,"cached_input_tokens":60,"output_tokens":15,"total_tokens":165}}}}"#,
        ];
        let path = write_temp_jsonl(lines);
        let mut result = HashMap::new();
        parse_file(&path, "2026-05-03", "2026-05-03", &mut result);
        let _ = std::fs::remove_file(path);

        let day = result.get("2026-05-03").expect("date present");
        assert_eq!(day.sessions, 1);
        assert_eq!(day.input_tokens, 150);
        assert_eq!(day.cache_read_tokens, 100);
        assert_eq!(day.output_tokens, 25);
        assert_eq!(day.models.get("gpt-5.5").copied(), Some(25));
    }

    #[test]
    fn repeated_last_token_usage_matches_codex_usage_reports() {
        let usage = r#"{"input_tokens":150,"cached_input_tokens":60,"output_tokens":15,"total_tokens":165}"#;
        let lines = &[
            r#"{"timestamp":"2026-05-03T06:48:55.000Z","type":"turn_context","payload":{"model":"gpt-5.5"}}"#,
            &format!(
                r#"{{"timestamp":"2026-05-03T06:49:04.000Z","type":"event_msg","payload":{{"type":"token_count","info":{{"last_token_usage":{usage}}}}}}}"#
            ),
            &format!(
                r#"{{"timestamp":"2026-05-03T06:49:05.000Z","type":"event_msg","payload":{{"type":"token_count","info":{{"last_token_usage":{usage}}}}}}}"#
            ),
        ];
        let path = write_temp_jsonl(lines);
        let mut result = HashMap::new();
        parse_file(&path, "2026-05-03", "2026-05-03", &mut result);
        let _ = std::fs::remove_file(path);

        let day = result.get("2026-05-03").expect("date present");
        assert_eq!(day.input_tokens, 180);
        assert_eq!(day.cache_read_tokens, 120);
        assert_eq!(day.output_tokens, 30);
    }

    #[test]
    fn model_switch_attributes_output_to_current_model() {
        let lines = &[
            r#"{"timestamp":"2026-05-03T06:48:55.000Z","type":"turn_context","payload":{"model":"gpt-5.4"}}"#,
            r#"{"timestamp":"2026-05-03T06:49:04.000Z","type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":100,"cached_input_tokens":40,"output_tokens":10,"total_tokens":110}}}}"#,
            r#"{"timestamp":"2026-05-03T06:50:00.000Z","type":"turn_context","payload":{"model":"gpt-5.5"}}"#,
            r#"{"timestamp":"2026-05-03T06:50:04.000Z","type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":150,"cached_input_tokens":60,"output_tokens":25,"total_tokens":175}}}}"#,
        ];
        let path = write_temp_jsonl(lines);
        let mut result = HashMap::new();
        parse_file(&path, "2026-05-03", "2026-05-03", &mut result);
        let _ = std::fs::remove_file(path);

        let day = result.get("2026-05-03").expect("date present");
        assert_eq!(day.models.get("gpt-5.4").copied(), Some(10));
        assert_eq!(day.models.get("gpt-5.5").copied(), Some(25));
        assert_eq!(day.output_tokens, 35);
    }

    #[test]
    fn current_date_groups_usage_when_session_crosses_midnight() {
        let lines = &[
            r#"{"timestamp":"2026-04-11T15:59:00.000Z","type":"turn_context","payload":{"current_date":"2026-04-11","model":"gpt-5.4"}}"#,
            r#"{"timestamp":"2026-04-11T15:59:10.000Z","type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":100,"cached_input_tokens":40,"output_tokens":10}}}}"#,
            r#"{"timestamp":"2026-04-11T16:00:00.000Z","type":"turn_context","payload":{"current_date":"2026-04-12","model":"gpt-5.4"}}"#,
            r#"{"timestamp":"2026-04-11T16:00:10.000Z","type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":200,"cached_input_tokens":80,"output_tokens":20}}}}"#,
        ];
        let path = write_temp_jsonl(lines);
        let mut result = HashMap::new();
        parse_file(&path, "2026-04-11", "2026-04-12", &mut result);
        let _ = std::fs::remove_file(path);

        let apr11 = result.get("2026-04-11").expect("Apr 11 present");
        let apr12 = result.get("2026-04-12").expect("Apr 12 present");
        assert_eq!(apr11.input_tokens, 60);
        assert_eq!(apr11.cache_read_tokens, 40);
        assert_eq!(apr11.output_tokens, 10);
        assert_eq!(apr12.input_tokens, 120);
        assert_eq!(apr12.cache_read_tokens, 80);
        assert_eq!(apr12.output_tokens, 20);
    }

    #[test]
    fn counts_messages_tools_and_active_minutes() {
        let lines = &[
            r#"{"timestamp":"2026-05-03T06:48:00.000Z","type":"event_msg","payload":{"type":"user_message"}}"#,
            r#"{"timestamp":"2026-05-03T06:49:00.000Z","type":"response_item","payload":{"type":"function_call","name":"exec_command"}}"#,
            r#"{"timestamp":"2026-05-03T06:51:30.000Z","type":"event_msg","payload":{"type":"user_message"}}"#,
        ];
        let path = write_temp_jsonl(lines);
        let mut result = HashMap::new();
        parse_file(&path, "2026-05-03", "2026-05-03", &mut result);
        let _ = std::fs::remove_file(path);

        let day = result.get("2026-05-03").expect("date present");
        assert_eq!(day.sessions, 1);
        assert_eq!(day.active_minutes, 3);
        assert_eq!(day.message_count, 2);
        assert_eq!(day.tool_uses, 1);
    }
}
