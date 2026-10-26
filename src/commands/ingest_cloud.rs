//! `vibestats ingest-cloud` command — records Claude Code cloud sessions.
//!
//! Cloud sessions on claude.ai/code run in throwaway containers whose GitHub
//! access is limited to the repositories attached to each session, so they
//! cannot sync themselves. Instead, a daily scheduled cloud task with the
//! vibestats-data repo attached lists the account's sessions with the
//! built-in session tools, saves that JSON, and runs this command.
//!
//! # Responsibilities
//! - Parse saved session listings (tolerant of the tool's wrapper text)
//! - Keep Anthropic-hosted cloud sessions only; sessions from local machines
//!   (Remote Control, `environment_kind = "bridge"`) are already counted by
//!   that machine's own `vibestats sync`
//! - Sum each day's sessions into one `DailyActivity`, dated by session start
//!   like the local Claude harness, under machine id `claude-cloud`
//! - Upload changed days via `github_api`, skipping days whose remote content
//!   is already identical
//!
//! # Constraints
//! - NEVER calls `std::process::exit` — `main.rs` handles exit
//! - Only days on or after `since` are written. The listing must cover every
//!   session started since then, because each day file is replaced whole.

use crate::github_api::GithubApi;
use crate::harnesses::DailyActivity;
use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;

/// Machine id shared by every Claude Code cloud session.
pub const CLOUD_MACHINE_ID: &str = "claude-cloud";

/// Cloud sessions are recorded under the Claude Code harness so the dashboard's
/// per-harness breakdown counts them with local Claude Code usage.
const HARNESS_ID: &str = "claude";

/// Entry point called from `main.rs` for the `vibestats ingest-cloud` command.
///
/// Returns `true` when every changed day was uploaded.
pub fn run(inputs: &[PathBuf], repo: &str, since: &str, dry_run: bool) -> bool {
    if !is_iso_date(since) {
        eprintln!("vibestats: --since must be a date in YYYY-MM-DD format");
        return false;
    }

    let mut sessions = Vec::new();
    if inputs.is_empty() {
        // No files given: read the listings this session fetched from its
        // own transcript, so nothing has to be copied out by hand.
        let Some(path) = current_transcript() else {
            eprintln!(
                "vibestats: no input files given and this session's transcript was not found \
                 (CLAUDE_CODE_SESSION_ID unset?)"
            );
            return false;
        };
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("vibestats: failed to read {}: {e}", path.display());
                return false;
            }
        };
        sessions = sessions_from_transcript(&text);
        if sessions.is_empty() {
            eprintln!(
                "vibestats: no list_sessions results found in {}",
                path.display()
            );
            return false;
        }
    }
    for path in inputs {
        let text = match std::fs::read_to_string(path) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("vibestats: failed to read {}: {e}", path.display());
                return false;
            }
        };
        let found = extract_sessions(&text);
        if found.is_empty() {
            eprintln!("vibestats: no session listing found in {}", path.display());
            return false;
        }
        sessions.extend(found);
    }

    let (days, counted) = aggregate(&sessions, since);
    println!(
        "vibestats: {} cloud session(s) since {since} across {} day(s)",
        counted,
        days.len()
    );

    if dry_run {
        for (date, activity) in &days {
            let payload =
                serde_json::to_string(activity).expect("DailyActivity serialization is infallible");
            println!("{date} {payload}");
        }
        return true;
    }

    let token = match github_token() {
        Some(t) => t,
        None => {
            eprintln!("vibestats: set GITHUB_TOKEN or GH_TOKEN to upload cloud sessions");
            return false;
        }
    };
    let api = GithubApi::new(&token, repo);

    let mut uploaded = 0usize;
    let mut unchanged = 0usize;
    let mut failed = 0usize;
    for (date, activity) in &days {
        // Same serialization as `sync`, so identical days compare equal.
        let payload =
            serde_json::to_string(activity).expect("DailyActivity serialization is infallible");
        let path = crate::sync::hive_path(date, HARNESS_ID, CLOUD_MACHINE_ID);
        match api.get_file_content(&path) {
            Ok(Some(existing)) if existing == payload => {
                unchanged += 1;
                continue;
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("vibestats: failed to read {path}: {e}");
                failed += 1;
                continue;
            }
        }
        match api.put_file(&path, &payload) {
            Ok(()) => uploaded += 1,
            Err(e) => {
                eprintln!("vibestats: failed to upload {path}: {e}");
                failed += 1;
            }
        }
    }

    println!("vibestats: {uploaded} day(s) uploaded, {unchanged} unchanged, {failed} failed");
    failed == 0
}

/// Inside a cloud session the GitHub proxy swaps the `proxy-injected`
/// placeholder in these variables for the session's real credentials.
fn github_token() -> Option<String> {
    ["GITHUB_TOKEN", "GH_TOKEN"]
        .iter()
        .filter_map(|name| std::env::var(name).ok())
        .find(|t| !t.trim().is_empty())
}

/// The current Claude Code session's transcript:
/// `~/.claude/projects/<project>/<CLAUDE_CODE_SESSION_ID>.jsonl`.
fn current_transcript() -> Option<PathBuf> {
    let session_id = std::env::var("CLAUDE_CODE_SESSION_ID").ok()?;
    let home = std::env::var("HOME").ok()?;
    let projects = PathBuf::from(home).join(".claude").join("projects");
    let file_name = format!("{session_id}.jsonl");
    std::fs::read_dir(projects)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path().join(&file_name))
        .find(|path| path.is_file())
}

/// Collects sessions from every `list_sessions` tool result in a transcript.
/// Tool results are matched to their calls by `tool_use_id`, so session JSON
/// quoted anywhere else in the conversation is ignored.
fn sessions_from_transcript(text: &str) -> Vec<serde_json::Value> {
    let mut listing_calls = HashSet::new();
    let mut sessions = Vec::new();
    for line in text.lines() {
        let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        let Some(blocks) = entry["message"]["content"].as_array() else {
            continue;
        };
        for block in blocks {
            match block["type"].as_str() {
                Some("tool_use") => {
                    let is_listing = block["name"]
                        .as_str()
                        .is_some_and(|n| n.ends_with("__list_sessions"));
                    if let (true, Some(id)) = (is_listing, block["id"].as_str()) {
                        listing_calls.insert(id.to_string());
                    }
                }
                Some("tool_result") => {
                    let is_listing = block["tool_use_id"]
                        .as_str()
                        .is_some_and(|id| listing_calls.contains(id));
                    if is_listing {
                        sessions.extend(extract_sessions(&tool_result_text(block)));
                    }
                }
                _ => {}
            }
        }
    }
    sessions
}

/// A tool result's content is either a string or a list of text blocks.
fn tool_result_text(block: &serde_json::Value) -> String {
    match &block["content"] {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(parts) => parts
            .iter()
            .filter_map(|p| p["text"].as_str())
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

fn is_iso_date(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b.iter()
            .enumerate()
            .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
}

/// Finds the first JSON value in `text` that holds session records. The
/// session tools wrap their JSON in explanatory text, so the listing may not
/// start at the beginning of the file.
fn extract_sessions(text: &str) -> Vec<serde_json::Value> {
    for (i, c) in text.char_indices() {
        if c != '{' && c != '[' {
            continue;
        }
        let mut stream = serde_json::Deserializer::from_str(&text[i..]).into_iter();
        if let Some(Ok(value)) = stream.next() {
            let mut sessions = Vec::new();
            collect_sessions(&value, &mut sessions);
            if !sessions.is_empty() {
                return sessions;
            }
        }
    }
    Vec::new()
}

/// Accepts a single session, an array of sessions, or a listing page shaped
/// like `{"data": [...]}`, optionally nested under `"ccr"`.
fn collect_sessions(value: &serde_json::Value, out: &mut Vec<serde_json::Value>) {
    match value {
        serde_json::Value::Array(items) => {
            for item in items {
                collect_sessions(item, out);
            }
        }
        serde_json::Value::Object(map) => {
            if map.contains_key("id") && map.contains_key("created_at") {
                out.push(value.clone());
            } else if let Some(data) = map.get("data") {
                collect_sessions(data, out);
            } else if let Some(ccr) = map.get("ccr") {
                collect_sessions(ccr, out);
            }
        }
        _ => {}
    }
}

/// Sums cloud sessions started on or after `since` into per-day activity.
/// Returns the per-day map and the number of sessions counted.
fn aggregate(
    sessions: &[serde_json::Value],
    since: &str,
) -> (BTreeMap<String, DailyActivity>, usize) {
    // Usage is cumulative per session, so when a session appears in several
    // listings the last one (the most recent snapshot) wins.
    let mut latest: BTreeMap<&str, &serde_json::Value> = BTreeMap::new();
    for s in sessions {
        if let Some(id) = s["id"].as_str() {
            latest.insert(id, s);
        }
    }

    let mut days: BTreeMap<String, DailyActivity> = BTreeMap::new();
    let mut counted = 0usize;
    for s in latest.values() {
        if s["environment_kind"].as_str() != Some("anthropic_cloud") {
            continue;
        }
        let Some(date) = s["created_at"].as_str().and_then(|t| t.get(0..10)) else {
            continue;
        };
        if !is_iso_date(date) || date < since {
            continue;
        }
        // A session with no usage never ran a turn.
        let usage = &s["external_metadata"]["usage"];
        if !usage.is_object() {
            continue;
        }

        let tokens = |key: &str| usage[key].as_u64().unwrap_or(0);
        let output = tokens("output_tokens");
        let day = days.entry(date.to_string()).or_default();
        day.sessions += 1;
        day.input_tokens += tokens("input_tokens");
        day.output_tokens += output;
        day.cache_read_tokens += tokens("cache_read_tokens");
        day.cache_creation_tokens += tokens("cache_write_tokens");
        if output > 0 {
            if let Some(model) = session_model(s) {
                *day.models.entry(model).or_insert(0) += output;
            }
        }
        counted += 1;
    }
    (days, counted)
}

/// The model the session last ran on, without a context-window suffix such as
/// `[1m]`, so it matches the model names in local transcripts.
fn session_model(s: &serde_json::Value) -> Option<String> {
    let model = s["external_metadata"]["last_served_model"]
        .as_str()
        .or_else(|| s["session_context"]["model"].as_str())
        .or_else(|| s["configured_model"].as_str())?;
    let base = model.split('[').next().unwrap_or(model).trim();
    if base.is_empty() {
        None
    } else {
        Some(base.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(id: &str, kind: &str, created: &str, usage: serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "created_at": created,
            "environment_kind": kind,
            "configured_model": "claude-opus-5-5",
            "external_metadata": { "usage": usage, "last_served_model": "claude-opus-5-5[1m]" }
        })
    }

    fn usage(input: u64, output: u64, read: u64, write: u64) -> serde_json::Value {
        serde_json::json!({
            "input_tokens": input,
            "output_tokens": output,
            "cache_read_tokens": read,
            "cache_write_tokens": write,
            "cost_usd": 1.5
        })
    }

    #[test]
    fn extracts_listing_wrapped_in_tool_text() {
        let text = r#"Another session's record (JSON). Only the closing tag ends it:
    {"ccr":{"data":[{"id":"session_a","created_at":"2026-10-26T11:42:36Z"}],"has_more":false}}
</other-session>"#;
        let sessions = extract_sessions(text);
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0]["id"], "session_a");
    }

    #[test]
    fn extracts_plain_array_and_single_session() {
        assert_eq!(
            extract_sessions(r#"[{"id":"a","created_at":"x"},{"id":"b","created_at":"y"}]"#).len(),
            2
        );
        assert_eq!(extract_sessions(r#"{"id":"a","created_at":"x"}"#).len(), 1);
        assert!(extract_sessions("no json here").is_empty());
    }

    #[test]
    fn sums_cloud_sessions_per_start_day() {
        let sessions = vec![
            session(
                "s1",
                "anthropic_cloud",
                "2026-10-25T13:51:29Z",
                usage(10, 100, 1000, 50),
            ),
            session(
                "s2",
                "anthropic_cloud",
                "2026-10-25T20:00:00Z",
                usage(5, 50, 500, 25),
            ),
            session(
                "s3",
                "anthropic_cloud",
                "2026-10-26T11:42:36Z",
                usage(1, 2, 3, 4),
            ),
        ];
        let (days, counted) = aggregate(&sessions, "2026-10-01");
        assert_eq!(counted, 3);
        let day = &days["2026-10-25"];
        assert_eq!(day.sessions, 2);
        assert_eq!(day.input_tokens, 15);
        assert_eq!(day.output_tokens, 150);
        assert_eq!(day.cache_read_tokens, 1500);
        assert_eq!(day.cache_creation_tokens, 75);
        assert_eq!(day.models["claude-opus-5-5"], 150);
        assert_eq!(days["2026-10-26"].sessions, 1);
    }

    #[test]
    fn later_snapshot_of_a_session_replaces_earlier_one() {
        let sessions = vec![
            session(
                "s1",
                "anthropic_cloud",
                "2026-10-25T13:51:29Z",
                usage(10, 100, 0, 0),
            ),
            session(
                "s1",
                "anthropic_cloud",
                "2026-10-25T13:51:29Z",
                usage(30, 300, 0, 0),
            ),
        ];
        let (days, counted) = aggregate(&sessions, "2026-10-01");
        assert_eq!(counted, 1);
        assert_eq!(days["2026-10-25"].sessions, 1);
        assert_eq!(days["2026-10-25"].output_tokens, 300);
        assert_eq!(days["2026-10-25"].models["claude-opus-5-5"], 300);
    }

    #[test]
    fn session_without_output_tokens_adds_no_model_entry() {
        let sessions = vec![session(
            "s1",
            "anthropic_cloud",
            "2026-10-25T13:51:29Z",
            serde_json::json!({ "cost_usd": 1.4 }),
        )];
        let (days, _) = aggregate(&sessions, "2026-10-01");
        assert_eq!(days["2026-10-25"].sessions, 1);
        assert!(days["2026-10-25"].models.is_empty());
    }

    #[test]
    fn skips_local_sessions_unused_sessions_and_duplicates() {
        let sessions = vec![
            session(
                "s1",
                "anthropic_cloud",
                "2026-10-25T13:51:29Z",
                usage(10, 100, 0, 0),
            ),
            session(
                "s1",
                "anthropic_cloud",
                "2026-10-25T13:51:29Z",
                usage(10, 100, 0, 0),
            ),
            session("s2", "bridge", "2026-10-25T13:51:29Z", usage(99, 99, 0, 0)),
            session(
                "s3",
                "anthropic_cloud",
                "2026-10-25T13:51:29Z",
                serde_json::Value::Null,
            ),
        ];
        let (days, counted) = aggregate(&sessions, "2026-10-01");
        assert_eq!(counted, 1);
        assert_eq!(days["2026-10-25"].sessions, 1);
        assert_eq!(days["2026-10-25"].output_tokens, 100);
    }

    #[test]
    fn ignores_sessions_started_before_since() {
        let sessions = vec![
            session(
                "old",
                "anthropic_cloud",
                "2026-10-18T23:59:59Z",
                usage(1, 1, 0, 0),
            ),
            session(
                "new",
                "anthropic_cloud",
                "2026-10-19T00:00:00Z",
                usage(1, 1, 0, 0),
            ),
        ];
        let (days, counted) = aggregate(&sessions, "2026-10-19");
        assert_eq!(counted, 1);
        assert!(!days.contains_key("2026-10-18"));
        assert!(days.contains_key("2026-10-19"));
    }

    #[test]
    fn model_falls_back_and_drops_context_suffix() {
        let s = serde_json::json!({ "session_context": { "model": "claude-opus-5[1m]" } });
        assert_eq!(session_model(&s), Some("claude-opus-5".to_string()));
        assert_eq!(session_model(&serde_json::json!({})), None);
    }

    #[test]
    fn reads_only_list_sessions_results_from_transcript() {
        let listing = r#"<other-session>{"ccr":{"data":[{"id":"s1","created_at":"2026-10-25T00:00:00Z"}]}}</other-session>"#;
        let lines = [
            serde_json::json!({"message": {"content": [
                {"type": "tool_use", "id": "t1", "name": "mcp__claude-code-remote__list_sessions", "input": {}},
                {"type": "tool_use", "id": "t2", "name": "Bash", "input": {}}
            ]}}),
            serde_json::json!({"message": {"content": [
                {"type": "tool_result", "tool_use_id": "t1", "content": [{"type": "text", "text": listing}]},
                {"type": "tool_result", "tool_use_id": "t2", "content": r#"{"id":"s9","created_at":"x"}"#}
            ]}}),
        ];
        let text = lines
            .iter()
            .map(|l| l.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        let sessions = sessions_from_transcript(&format!("{text}\nnot json\n"));
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0]["id"], "s1");
    }

    #[test]
    fn validates_since_date_format() {
        assert!(is_iso_date("2026-10-19"));
        assert!(!is_iso_date("2026-10-1"));
        assert!(!is_iso_date("20261019xx"));
    }
}
