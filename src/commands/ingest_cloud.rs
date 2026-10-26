//! `vibestats ingest-cloud` command — records Claude Code cloud sessions.
//!
//! Cloud sessions on claude.ai/code run in throwaway containers whose GitHub
//! access is limited to the repositories attached to each session, so they
//! cannot sync themselves. Instead, a daily scheduled cloud task with the
//! vibestats-data repo attached lists all of the account's sessions with the
//! built-in session tools and runs this command.
//!
//! # Responsibilities
//! - Read `list_sessions` results from the running session's transcript (or
//!   from saved files), and refuse to run on a listing that stopped early
//! - Keep Anthropic-hosted cloud sessions only; sessions from local machines
//!   (Remote Control, `environment_kind = "bridge"`) are already counted by
//!   that machine's own `vibestats sync`
//! - Write one `DailyActivity` per session, dated by session start like the
//!   local Claude harness, at `.../machine_id=claude-cloud/session=<id>/data.json`.
//!   A session's file is rewritten whenever its usage grows, however old it is
//! - Track uploaded payload hashes in `claude-cloud/index.json`, so a run reads
//!   one file and uploads only the sessions that changed
//!
//! # Constraints
//! - NEVER calls `std::process::exit` — `main.rs` handles exit

use crate::github_api::GithubApi;
use crate::harnesses::DailyActivity;
use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;

/// Machine id shared by every Claude Code cloud session.
pub const CLOUD_MACHINE_ID: &str = "claude-cloud";

/// Cloud sessions are recorded under the Claude Code harness so the dashboard's
/// per-harness breakdown counts them with local Claude Code usage.
const HARNESS_ID: &str = "claude";

/// Maps session id → SHA256 of the payload last uploaded for it. Kept outside
/// `machines/` so the aggregation Action never reads it.
const INDEX_PATH: &str = "claude-cloud/index.json";

/// One page returned by the session listing tool.
struct Listing {
    sessions: Vec<serde_json::Value>,
    has_more: Option<bool>,
}

/// One cloud session's activity, ready to upload.
struct CloudSession {
    id: String,
    date: String,
    activity: DailyActivity,
}

/// Entry point called from `main.rs` for the `vibestats ingest-cloud` command.
///
/// Returns `true` when every changed session was uploaded.
pub fn run(inputs: &[PathBuf], repo: &str, dry_run: bool) -> bool {
    let listings = match read_listings(inputs) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("vibestats: {e}");
            return false;
        }
    };
    // Sessions missing from an unfinished listing would never be recorded.
    if listings.last().and_then(|l| l.has_more) == Some(true) {
        eprintln!(
            "vibestats: the session listing is incomplete (its last page has has_more: true). \
             Call list_sessions again with after_id set to that page's last_id until \
             has_more is false, then rerun this command."
        );
        return false;
    }
    let raw: Vec<serde_json::Value> = listings.into_iter().flat_map(|l| l.sessions).collect();
    let sessions = cloud_sessions(&raw);
    println!("vibestats: {} cloud session(s) listed", sessions.len());

    if dry_run {
        for s in &sessions {
            println!("{} {}", session_path(s), payload(s));
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

    let mut index: BTreeMap<String, String> = match api.get_file_content(INDEX_PATH) {
        Ok(Some(text)) => serde_json::from_str(&text).unwrap_or_else(|e| {
            eprintln!("vibestats: {INDEX_PATH} is malformed ({e}); re-uploading every session");
            BTreeMap::new()
        }),
        Ok(None) => BTreeMap::new(),
        Err(e) => {
            eprintln!("vibestats: failed to read {INDEX_PATH}: {e}");
            return false;
        }
    };

    let mut uploaded = 0usize;
    let mut unchanged = 0usize;
    let mut failed = 0usize;
    for s in &sessions {
        let body = payload(s);
        let hash = crate::sync::sha256_hex(body.as_bytes());
        if index.get(&s.id) == Some(&hash) {
            unchanged += 1;
            continue;
        }
        let path = session_path(s);
        match api.put_file(&path, &body) {
            Ok(()) => {
                index.insert(s.id.clone(), hash);
                uploaded += 1;
            }
            Err(e) => {
                eprintln!("vibestats: failed to upload {path}: {e}");
                failed += 1;
            }
        }
    }

    if uploaded > 0 {
        let text = serde_json::to_string_pretty(&index).expect("index serialization is infallible");
        if let Err(e) = api.put_file(INDEX_PATH, &text) {
            // The session files are already up; the next run re-uploads them.
            eprintln!("vibestats: failed to update {INDEX_PATH}: {e}");
            failed += 1;
        }
    }

    println!("vibestats: {uploaded} session(s) uploaded, {unchanged} unchanged, {failed} failed");
    failed == 0
}

/// Reads listings from the given files, or from this session's own transcript
/// when no files are given, so nothing has to be copied out by hand.
fn read_listings(inputs: &[PathBuf]) -> Result<Vec<Listing>, String> {
    if inputs.is_empty() {
        let path = current_transcript().ok_or(
            "no input files given and this session's transcript was not found \
             (CLAUDE_CODE_SESSION_ID unset?)",
        )?;
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        let listings = listings_from_transcript(&text);
        if listings.is_empty() {
            return Err(format!(
                "no list_sessions results found in {}",
                path.display()
            ));
        }
        return Ok(listings);
    }
    let mut listings = Vec::new();
    for path in inputs {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
        let listing = extract_listing(&text)
            .ok_or(format!("no session listing found in {}", path.display()))?;
        listings.push(listing);
    }
    Ok(listings)
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

/// Collects every `list_sessions` tool result in a transcript, in order.
/// Tool results are matched to their calls by `tool_use_id`, so session JSON
/// quoted anywhere else in the conversation is ignored.
fn listings_from_transcript(text: &str) -> Vec<Listing> {
    let mut listing_calls = HashSet::new();
    let mut listings = Vec::new();
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
                        if let Some(listing) = extract_listing(&tool_result_text(block)) {
                            listings.push(listing);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    listings
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

/// Finds the first JSON value in `text` that holds session records. The
/// session tools wrap their JSON in explanatory text, so the listing may not
/// start at the beginning of the file. An empty `data` page is still a
/// listing, so an account with no sessions yet reads as complete.
fn extract_listing(text: &str) -> Option<Listing> {
    for (i, c) in text.char_indices() {
        if c != '{' && c != '[' {
            continue;
        }
        let mut stream =
            serde_json::Deserializer::from_str(&text[i..]).into_iter::<serde_json::Value>();
        let Some(Ok(value)) = stream.next() else {
            continue;
        };
        let page: &serde_json::Value = if value.get("ccr").is_some() {
            &value["ccr"]
        } else {
            &value
        };
        let mut sessions = Vec::new();
        collect_sessions(page, &mut sessions);
        if !sessions.is_empty() || page["data"].is_array() {
            return Some(Listing {
                sessions,
                has_more: page["has_more"].as_bool(),
            });
        }
    }
    None
}

/// Accepts a single session, an array of sessions, or a listing page shaped
/// like `{"data": [...]}`.
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
            }
        }
        _ => {}
    }
}

/// Turns raw session records into per-session activity. Usage is cumulative,
/// so when a session appears more than once the last (newest) snapshot wins.
fn cloud_sessions(raw: &[serde_json::Value]) -> Vec<CloudSession> {
    let mut latest: BTreeMap<String, &serde_json::Value> = BTreeMap::new();
    for s in raw {
        if let Some(id) = s["id"].as_str().map(path_safe).filter(|id| !id.is_empty()) {
            latest.insert(id, s);
        }
    }

    let mut out = Vec::new();
    for (id, s) in latest {
        if s["environment_kind"].as_str() != Some("anthropic_cloud") {
            continue;
        }
        let Some(date) = s["created_at"].as_str().and_then(|t| t.get(0..10)) else {
            continue;
        };
        if !is_iso_date(date) {
            continue;
        }
        // A session with no usage never ran a turn.
        let usage = &s["external_metadata"]["usage"];
        if !usage.is_object() {
            continue;
        }

        let tokens = |key: &str| usage[key].as_u64().unwrap_or(0);
        let mut activity = DailyActivity {
            sessions: 1,
            input_tokens: tokens("input_tokens"),
            output_tokens: tokens("output_tokens"),
            cache_read_tokens: tokens("cache_read_tokens"),
            cache_creation_tokens: tokens("cache_write_tokens"),
            ..Default::default()
        };
        if activity.output_tokens > 0 {
            if let Some(model) = session_model(s) {
                activity.models.insert(model, activity.output_tokens);
            }
        }
        out.push(CloudSession {
            id,
            date: date.to_string(),
            activity,
        });
    }
    out
}

/// `machines/year=YYYY/month=MM/day=DD/harness=claude/machine_id=claude-cloud/session=<id>/data.json`
fn session_path(s: &CloudSession) -> String {
    let day_path = crate::sync::hive_path(&s.date, HARNESS_ID, CLOUD_MACHINE_ID);
    let partition = day_path.trim_end_matches("data.json");
    format!("{partition}session={}/data.json", s.id)
}

/// Same serialization as `sync`, so payload hashes are stable.
fn payload(s: &CloudSession) -> String {
    serde_json::to_string(&s.activity).expect("DailyActivity serialization is infallible")
}

/// Session ids become Hive path segments, so keep only path-safe characters.
fn path_safe(id: &str) -> String {
    id.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect()
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

    const DAY: &str = "2026-10-25T13:51:29Z";

    #[test]
    fn extracts_listing_wrapped_in_tool_text() {
        let text = r#"Another session's record (JSON). Only the closing tag ends it:
    {"ccr":{"data":[{"id":"session_a","created_at":"2026-10-26T11:42:36Z"}],"has_more":true,"last_id":"session_a"}}
</other-session>"#;
        let listing = extract_listing(text).unwrap();
        assert_eq!(listing.sessions.len(), 1);
        assert_eq!(listing.sessions[0]["id"], "session_a");
        assert_eq!(listing.has_more, Some(true));
    }

    #[test]
    fn extracts_plain_array_single_session_and_empty_page() {
        let both = r#"[{"id":"a","created_at":"x"},{"id":"b","created_at":"y"}]"#;
        let listing = extract_listing(both).unwrap();
        assert_eq!(listing.sessions.len(), 2);
        assert_eq!(listing.has_more, None);

        let single = extract_listing(r#"{"id":"a","created_at":"x"}"#).unwrap();
        assert_eq!(single.sessions.len(), 1);

        let empty = extract_listing(r#"{"ccr":{"data":[],"has_more":false}}"#).unwrap();
        assert!(empty.sessions.is_empty());
        assert_eq!(empty.has_more, Some(false));

        assert!(extract_listing("no json here").is_none());
    }

    #[test]
    fn reads_only_list_sessions_results_from_transcript_in_order() {
        let page = |id: &str, more: bool| {
            format!(
                r#"<other-session>{{"ccr":{{"data":[{{"id":"{id}","created_at":"{DAY}"}}],"has_more":{more}}}}}</other-session>"#
            )
        };
        let lines = [
            serde_json::json!({"message": {"content": [
                {"type": "tool_use", "id": "t1", "name": "mcp__claude-code-remote__list_sessions", "input": {}},
                {"type": "tool_use", "id": "t2", "name": "Bash", "input": {}}
            ]}}),
            serde_json::json!({"message": {"content": [
                {"type": "tool_result", "tool_use_id": "t1", "content": [{"type": "text", "text": page("s1", true)}]},
                {"type": "tool_result", "tool_use_id": "t2", "content": r#"{"id":"s9","created_at":"x"}"#}
            ]}}),
            serde_json::json!({"message": {"content": [
                {"type": "tool_use", "id": "t3", "name": "mcp__claude-code-remote__list_sessions", "input": {}}
            ]}}),
            serde_json::json!({"message": {"content": [
                {"type": "tool_result", "tool_use_id": "t3", "content": page("s2", false)}
            ]}}),
        ];
        let text = lines
            .iter()
            .map(|l| l.to_string())
            .collect::<Vec<_>>()
            .join("\n");
        let listings = listings_from_transcript(&format!("{text}\nnot json\n"));
        assert_eq!(listings.len(), 2);
        assert_eq!(listings[0].sessions[0]["id"], "s1");
        assert_eq!(listings[0].has_more, Some(true));
        assert_eq!(listings[1].sessions[0]["id"], "s2");
        assert_eq!(listings[1].has_more, Some(false));
    }

    #[test]
    fn one_record_per_cloud_session_dated_by_start() {
        let raw = vec![
            session("s1", "anthropic_cloud", DAY, usage(10, 100, 1000, 50)),
            session(
                "s2",
                "anthropic_cloud",
                "2026-10-25T20:00:00Z",
                usage(5, 50, 500, 25),
            ),
        ];
        let sessions = cloud_sessions(&raw);
        assert_eq!(sessions.len(), 2);
        let s1 = &sessions[0];
        assert_eq!(s1.id, "s1");
        assert_eq!(s1.date, "2026-10-25");
        assert_eq!(s1.activity.sessions, 1);
        assert_eq!(s1.activity.input_tokens, 10);
        assert_eq!(s1.activity.output_tokens, 100);
        assert_eq!(s1.activity.cache_read_tokens, 1000);
        assert_eq!(s1.activity.cache_creation_tokens, 50);
        assert_eq!(s1.activity.models["claude-opus-5-5"], 100);
        assert_eq!(
            session_path(s1),
            "machines/year=2026/month=10/day=25/harness=claude/machine_id=claude-cloud/session=s1/data.json"
        );
    }

    #[test]
    fn later_snapshot_of_a_session_replaces_earlier_one() {
        let raw = vec![
            session("s1", "anthropic_cloud", DAY, usage(10, 100, 0, 0)),
            session("s1", "anthropic_cloud", DAY, usage(30, 300, 0, 0)),
        ];
        let sessions = cloud_sessions(&raw);
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].activity.output_tokens, 300);
    }

    #[test]
    fn skips_local_and_unused_sessions() {
        let raw = vec![
            session("s1", "anthropic_cloud", DAY, usage(10, 100, 0, 0)),
            session("s2", "bridge", DAY, usage(99, 99, 0, 0)),
            session("s3", "anthropic_cloud", DAY, serde_json::Value::Null),
        ];
        let sessions = cloud_sessions(&raw);
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, "s1");
    }

    #[test]
    fn session_without_output_tokens_adds_no_model_entry() {
        let raw = vec![session(
            "s1",
            "anthropic_cloud",
            DAY,
            serde_json::json!({ "cost_usd": 1.4 }),
        )];
        let sessions = cloud_sessions(&raw);
        assert_eq!(sessions[0].activity.sessions, 1);
        assert!(sessions[0].activity.models.is_empty());
    }

    #[test]
    fn session_ids_are_made_path_safe() {
        let raw = vec![
            session("../cse_a/b c", "anthropic_cloud", DAY, usage(1, 1, 0, 0)),
            session("/..", "anthropic_cloud", DAY, usage(1, 1, 0, 0)),
        ];
        let sessions = cloud_sessions(&raw);
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, "cse_abc");
    }

    #[test]
    fn model_falls_back_and_drops_context_suffix() {
        let s = serde_json::json!({ "session_context": { "model": "claude-opus-5[1m]" } });
        assert_eq!(session_model(&s), Some("claude-opus-5".to_string()));
        assert_eq!(session_model(&serde_json::json!({})), None);
    }

    #[test]
    fn validates_iso_dates() {
        assert!(is_iso_date("2026-10-19"));
        assert!(!is_iso_date("2026-10-1"));
        assert!(!is_iso_date("20261019xx"));
    }
}
