# Story 2.4: Implement JSONL Parser

Status: review

<!-- GH Issue: #16 | Epic: #2 | PR must include: Closes #16 -->

## Story

As the vibestats binary,
I want a JSONL parser that walks `~/.claude/projects/**/*.jsonl` and extracts per-day session activity,
So that usage data is derived from the authoritative local source.

## Acceptance Criteria

1. **Given** JSONL files exist under `~/.claude/projects/` **When** `jsonl_parser::parse_date_range(start, end)` is called **Then** it returns a `HashMap<String, DailyActivity>` where keys are `"YYYY-MM-DD"` and values are `{ sessions, active_minutes }` aggregated across all matching files for the requested date range (FR14)

2. **Given** a JSONL file contains fields not in the known schema **When** the parser processes it **Then** unknown fields are silently ignored and known fields are extracted correctly (NFR14)

3. **Given** 12 months of JSONL history exists **When** `parse_date_range` is called for the full history **Then** parsing completes in under 10 seconds on typical hardware (NFR3 baseline)

## Tasks / Subtasks

- [x] Task 1: Create `src/jsonl_parser.rs` with `DailyActivity` struct and public `parse_date_range` function (AC: #1, #2)
  - [x] Define `DailyActivity` struct: `pub struct DailyActivity { pub sessions: u32, pub active_minutes: u32 }`
  - [x] Derive `serde::Serialize` + `serde::Deserialize` + `Debug` + `Default` on `DailyActivity`
  - [x] Define `ClaudeEntry` private struct for deserializing JSONL lines — use `#[serde(default)]` on every optional field to ensure schema tolerance (NFR14)
  - [x] Implement `pub fn parse_date_range(start: &str, end: &str) -> HashMap<String, DailyActivity>` that walks `~/.claude/projects/**/*.jsonl` and aggregates per-day activity

- [x] Task 2: Implement JSONL file discovery (AC: #1, #3)
  - [x] Use `std::fs::read_dir` recursively (no glob crate — see No New Dependencies section) to walk `~/.claude/projects/` and collect all `*.jsonl` paths
  - [x] Expand `~` to the real home directory using `std::env::var("HOME")` — no `dirs` crate (not in Cargo.toml)
  - [x] Skip unreadable directories or files silently (fail-open, NFR10)

- [x] Task 3: Implement JSONL file parsing — extract session date and duration (AC: #1, #2)
  - [x] Read each JSONL file line-by-line using `std::io::BufReader` for memory efficiency (NFR3)
  - [x] For each line: attempt `serde_json::from_str::<ClaudeEntry>(&line)` — on error skip the line silently
  - [x] Scan all lines to find: (a) the session date from any entry's `timestamp` field, (b) the session `durationMs` from the entry where `type == "system"` and `subtype == "turn_duration"`
  - [x] Session date = first 10 chars of the first non-None `timestamp` encountered: `timestamp.get(..10)` — never use direct indexing
  - [x] `active_minutes` = `duration_ms / 60_000`; if no `turn_duration` entry exists, use 0

- [x] Task 4: Implement per-file (per-session) activity accumulation (AC: #1)
  - [x] One `.jsonl` file = one session; count each file as +1 to `sessions` for its date (not individual lines)
  - [x] Only count the file if its session date falls within `[start, end]` (inclusive, lexicographic compare on `"YYYY-MM-DD"` strings)
  - [x] Use `HashMap::entry(date).or_default()` to accumulate across multiple files for the same date

- [x] Task 5: Wire `jsonl_parser.rs` into `main.rs` as a declared module (AC: compile)
  - [x] Add `mod jsonl_parser;` to `src/main.rs`
  - [x] No business logic in `main.rs` — only the `mod` declaration

- [x] Task 6: Write co-located unit tests (AC: #1, #2, #3)
  - [x] `#[cfg(test)]` module inside `src/jsonl_parser.rs`
  - [x] Test: parse a single JSONL file within range → sessions=1, active_minutes derived from durationMs
  - [x] Test: parse a JSONL file with unknown fields → unknown fields silently ignored, date/duration extracted correctly
  - [x] Test: parse a JSONL file with date outside the date range → not included in result (sessions=0)
  - [x] Test: parse a JSONL file with malformed lines → malformed lines skipped, file still counted if date found
  - [x] Test: parse an empty file → returns empty map (no panic, sessions=0)
  - [x] Test: two JSONL files on the same date → sessions=2, active_minutes summed from both files
  - [x] Test: file with no turn_duration entry → sessions=1, active_minutes=0 (graceful fallback)
  - [x] Run `cargo test` — must pass with 0 failures
  - [x] Run `cargo clippy --all-targets -- -D warnings` — must produce 0 warnings

## Dev Notes

### Claude Code JSONL Format — Actual Schema (Verified)

Each `.jsonl` file in `~/.claude/projects/**/*.jsonl` represents **one Claude Code session**. Lines are JSON objects with varying `type` fields. The parser must be tolerant of unknown fields (NFR14).

**Key insight: one file = one session.** Sessions are counted at the file level, not the line level. The parser should treat each `.jsonl` file as one session event for the date on which that file's session occurred.

**Observed entry types (from real files):**

| `type` | `subtype` | Purpose |
|---|---|---|
| `assistant` | — | Claude response turns (majority of lines) |
| `user` | — | User prompt turns |
| `system` | `turn_duration` | Session summary: contains `durationMs` and `messageCount` |
| `file-history-snapshot` | — | Internal snapshot; ignore |
| `attachment` | — | Hook outputs; ignore |

**Session duration entry — exact schema:**

```json
{
  "type": "system",
  "subtype": "turn_duration",
  "durationMs": 579575,
  "messageCount": 238,
  "timestamp": "2026-04-01T15:03:39.992Z",
  "uuid": "a485f263-...",
  "sessionId": "32a57c84-...",
  "isMeta": false,
  ...
}
```

**How to get the session date:** Use the `timestamp` field from the `system`/`turn_duration` entry, or fall back to the first `timestamp` field found in any entry in the file. All timestamps are ISO 8601 UTC with milliseconds: `"YYYY-MM-DDTHH:MM:SS.mmmZ"`.

**How to extract the date:** Take the first 10 chars of any `timestamp` value to get `"YYYY-MM-DD"`.

**How to compute `active_minutes`:** The `turn_duration` entry has `durationMs` (milliseconds). Convert: `active_minutes = durationMs / 60_000`. If no `turn_duration` entry exists in the file, use 0.

**ClaudeEntry struct — use these exact field names:**

```rust
#[derive(Debug, Default, Deserialize)]
struct ClaudeEntry {
    /// Entry type: "assistant", "user", "system", "attachment", etc.
    #[serde(rename = "type", default)]
    entry_type: Option<String>,

    /// Entry subtype — "turn_duration" for the session summary entry
    #[serde(default)]
    subtype: Option<String>,

    /// ISO 8601 UTC timestamp with milliseconds: "2026-04-01T15:03:39.992Z"
    #[serde(default)]
    timestamp: Option<String>,

    /// Session duration in milliseconds — only present on type=system, subtype=turn_duration
    /// JSON field name is "durationMs" (camelCase) — must use #[serde(rename)]
    #[serde(rename = "durationMs", default)]
    duration_ms: Option<u64>,
}
```

**Important:** `timestamp` uses millisecond precision (`"YYYY-MM-DDTHH:MM:SS.mmmZ"`). Use `get(..10)` to extract just the `"YYYY-MM-DD"` portion — safe with `get` since it returns `None` on out-of-bounds; do NOT use `&ts[..10]` (would panic on short strings).

**Important:** `type` is a reserved keyword in Rust. The JSON field `"type"` must be renamed using `#[serde(rename = "type")]` on a Rust field with a different name (e.g., `entry_type`).

**Schema tolerance rule (NFR14):** Use `#[serde(default)]` on every field in `ClaudeEntry`. If `serde_json::from_str` fails entirely (malformed JSON), skip the line. Never panic or exit non-zero on bad input.

### `parse_date_range` Signature and Return Type

```rust
use std::collections::HashMap;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DailyActivity {
    pub sessions: u32,
    pub active_minutes: u32,
}

/// Walk ~/.claude/projects/**/*.jsonl and aggregate per-day activity
/// for dates in [start, end] inclusive (YYYY-MM-DD strings).
/// Returns an empty map if the directory is missing or unreadable.
pub fn parse_date_range(start: &str, end: &str) -> HashMap<String, DailyActivity> {
    // implementation
}
```

**Why `HashMap<String, DailyActivity>`:** This matches the contract in `sync.rs` (Story 3.1), which iterates the map to compute hashes and push to GitHub. The key format `"YYYY-MM-DD"` is consistent with `checkpoint.rs` date keys.

### Home Directory Resolution

Do not hardcode `/Users/stephenleo`. Use `std::env::var("HOME")` to resolve `~`:

```rust
fn claude_projects_dir() -> Option<std::path::PathBuf> {
    std::env::var("HOME").ok().map(|h| {
        std::path::PathBuf::from(h).join(".claude").join("projects")
    })
}
```

**Do not use the `dirs` crate** — it is not in `Cargo.toml` and adding it would require a new dependency (not allowed for this story).

### Recursive Directory Walk (No `glob` Crate)

Use `std::fs::read_dir` recursively. A simple implementation:

```rust
fn collect_jsonl_files(dir: &std::path::Path, acc: &mut Vec<std::path::PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return, // unreadable directory — skip silently (NFR10)
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
```

**Do not add `glob` or `walkdir` crates** — they are not in `Cargo.toml`.

### Per-File (Per-Session) Parsing Strategy

Each JSONL file = one Claude Code session. The parsing logic:
1. Scan all lines in the file to find the session date and duration
2. Count the whole file as +1 session for its date
3. Add the duration (in minutes) to `active_minutes` for that date

```rust
use std::io::{BufRead, BufReader};

fn parse_file(path: &std::path::Path, start: &str, end: &str,
              result: &mut HashMap<String, DailyActivity>) {
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return, // unreadable file — skip silently
    };

    let mut session_date: Option<String> = None;
    let mut duration_ms: u64 = 0;

    for line in BufReader::new(file).lines().flatten() {
        let entry: ClaudeEntry = match serde_json::from_str(&line) {
            Ok(e) => e,
            Err(_) => continue, // malformed line — skip silently (NFR14)
        };

        // Capture session date from first timestamp we see
        if session_date.is_none() {
            if let Some(ts) = &entry.timestamp {
                if let Some(date) = ts.get(..10) {
                    session_date = Some(date.to_string());
                }
            }
        }

        // Capture duration from the turn_duration system entry
        if entry.entry_type.as_deref() == Some("system")
            && entry.subtype.as_deref() == Some("turn_duration")
        {
            if let Some(ms) = entry.duration_ms {
                duration_ms = ms;
            }
        }
    }

    // Count this file as one session for its date (if in range)
    if let Some(date) = session_date {
        if date.as_str() >= start && date.as_str() <= end {
            let activity = result.entry(date).or_default();
            activity.sessions += 1;
            activity.active_minutes += (duration_ms / 60_000) as u32;
        }
    }
}
```

### Date Range Filtering

Dates are `"YYYY-MM-DD"` strings. Lexicographic comparison works correctly for ISO date strings:

```rust
if date >= start && date <= end {
    // include this entry
}
```

Extract the date from a timestamp like `"2026-04-10T14:23:00Z"` by taking the first 10 characters:

```rust
let date = timestamp.get(..10)?;  // "YYYY-MM-DD"
```

Guard against timestamps shorter than 10 chars with `get(..10)` (returns `None` on out-of-bounds) rather than direct slice indexing (which would panic).

### Module File Location

```
src/
├── main.rs           ← add `mod jsonl_parser;` here
├── config.rs         ← story 2.1 (existing)
├── logger.rs         ← story 2.2 (existing)
├── checkpoint.rs     ← story 2.3 (existing)
└── jsonl_parser.rs   ← THIS STORY (new file)
```

Do not create or modify any other files except `src/main.rs` (to add `mod jsonl_parser;`).

### Existing Crates (No New Dependencies)

All crates needed for this story are already in `Cargo.toml`:

| Crate | Usage in this story |
|---|---|
| `serde` (with `derive`) | `#[derive(Serialize, Deserialize, Default)]` on `DailyActivity` + `ClaudeEntry` |
| `serde_json` | `serde_json::from_str` for JSONL line deserialization |

`std::collections::HashMap`, `std::fs`, `std::io::BufReader`, `std::path::Path`, `std::env::var` — all `std`, no `Cargo.toml` entry needed.

**Do NOT add `glob`, `walkdir`, `dirs`, `chrono`, or any other new crate.**

**Confirmed existing `Cargo.toml`:**
```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
ureq = "2.10"
toml = "0.8"
```

### Module Boundary (Architecture Constraint — Critical)

From `architecture.md`:

| `jsonl_parser.rs` OWNS | `jsonl_parser.rs` NEVER does |
|---|---|
| JSONL file walking | Network calls |
| Session data extraction | Config file read/write |
| Per-day aggregation | Checkpoint operations |
| `parse_date_range` public API | Business logic decisions (callers decide) |

**Anti-pattern prevention:**
- Do NOT call `std::process::exit` inside `jsonl_parser.rs` — callers handle exits (NFR10)
- Do NOT make HTTP calls or read `config.toml` or `checkpoint.toml` from this module
- Do NOT hardcode `~/.claude/projects` with a literal `~` — expand via `std::env::var("HOME")`
- Do NOT load entire files into a `String` before parsing — use `BufReader` line-by-line (NFR3)

### Error Handling Contract

All errors are silent (NFR10, NFR14):
- Unreadable directory: return empty map or skip
- Unreadable file: skip
- Malformed JSON line: skip
- Missing timestamp field: skip entry
- Date out of range: skip entry
- No files found: return empty `HashMap`

**Never call `unwrap()`, `expect()`, or `panic!()` in non-test code.**

### `allow(dead_code)` Pattern from Story 2.3

Story 2.3 required `#![allow(dead_code)]` at the top of `checkpoint.rs` to suppress clippy warnings for public API not yet consumed by callers. Apply the same pattern to `jsonl_parser.rs`:

```rust
#![allow(dead_code)]
```

This is intentional — `jsonl_parser.rs` is infrastructure; its callers (`sync.rs`) arrive in Story 3.1.

### Tests

Write tests using inline JSONL strings (no real filesystem dependency required for unit tests):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// Write JSONL content to a unique temp file, return the path.
    /// Caller must clean up.
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

    // Minimal valid JSONL session file: one assistant entry with timestamp,
    // one system/turn_duration entry with durationMs
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
        assert_eq!(day.active_minutes, 60); // 3600000ms / 60000 = 60
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
        assert_eq!(day.active_minutes, 1); // 60000ms / 60000 = 1
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
        let day = result.get("2026-04-10").expect("date found from valid line");
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
```

Use `std::env::temp_dir()` + a unique filename (combine pid + nanos) for any temp files — do not use a fixed filename (parallel test runs race on fixed names, as learned in Story 2.3 review).

### Worktree / Cargo Isolation

From Story 1.2 learnings: the worktree is nested inside the main repo. The `[workspace]` in `Cargo.toml` at the repo root prevents upward traversal. Do NOT add another `[workspace]` — it already exists.

Run `cargo build` and `cargo test` from the **repo root** (`/Users/stephenleo/Developer/vibestats`), not from the worktree directory. Cargo resolves to the workspace `Cargo.toml` at the root.

### Architecture Constraints Summary

| Constraint | NFR/Source | Impact on This Story |
|---|---|---|
| Exit 0 on all errors | NFR10, architecture.md | All parse errors skip silently; never `process::exit` in this module |
| JSONL schema tolerance | NFR14, architecture.md | `#[serde(default)]` on all fields; malformed lines skipped |
| 60s backfill performance | NFR3, architecture.md | `BufReader` line-by-line; no full-file loads into RAM |
| No network calls in jsonl_parser.rs | architecture.md#Module Responsibility | Only filesystem I/O |
| snake_case field names | architecture.md#Naming Patterns | `sessions`, `active_minutes` |
| Rust snake_case filenames | architecture.md#Naming Patterns | File: `src/jsonl_parser.rs` |
| No extra crates | Story scope | All required crates already in Cargo.toml |
| Co-located unit tests | architecture.md | `#[cfg(test)]` module inside `jsonl_parser.rs` |
| clippy --all-targets | Story 2.3 review learning | Run `cargo clippy --all-targets -- -D warnings` (not just `-- -D warnings`) |

### Project Structure Notes

- `src/jsonl_parser.rs` is a new file — no existing file to modify except `src/main.rs`
- `src/main.rs` already has `mod checkpoint;`, `mod config;`, `mod logger;` — add `mod jsonl_parser;` in alphabetical order (or consistent with existing ordering)
- `Cargo.toml` is at the repo root — no changes needed

### References

- JSONL parser spec: [Source: architecture.md#Module Responsibility Boundaries — `jsonl_parser.rs`]
- Performance baseline: [Source: architecture.md#NFR-critical constraints — NFR3]
- JSONL tolerance: [Source: architecture.md#NFR-critical constraints — NFR14 and Cross-Cutting Concern #4]
- File structure: [Source: architecture.md#Source Tree (Rust binary)]
- Data flow: [Source: architecture.md#Architectural Boundaries — data flow diagram]
- `DailyActivity` output schema: [Source: docs/schemas.md#1. Machine Day File — File Content]
- Story 2.3 learnings (allow dead_code, clippy --all-targets, unique temp files): [Source: implementation-artifacts/2-3-implement-checkpoint-module.md#Dev Agent Record]
- Story 1.2 (Cargo.toml + workspace): [Source: implementation-artifacts/1-2-initialize-rust-binary-project.md]
- GH Issue: #16

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

- Clippy `lines_filter_map_ok` warning: replaced `.lines().flatten()` with `.lines().map_while(Result::ok)` as recommended. The `flatten()` approach can loop indefinitely on persistent IO errors; `map_while` terminates correctly.

### Completion Notes List

- Implemented `src/jsonl_parser.rs` with `DailyActivity` public struct, `ClaudeEntry` private struct, `parse_date_range` public function, `collect_jsonl_files` and `parse_file` private helpers.
- All JSONL parsing uses `BufReader` line-by-line for memory efficiency (NFR3). No full-file loads into memory.
- Schema tolerance via `#[serde(default)]` on all `ClaudeEntry` fields; malformed lines silently skipped (NFR14).
- `#![allow(dead_code)]` applied as per Story 2.3 pattern — callers arrive in Story 3.1.
- `mod jsonl_parser;` added to `src/main.rs` in alphabetical order between `config` and `logger`.
- 7 unit tests added covering all required scenarios; all 36 total tests pass with 0 failures.
- `cargo clippy --all-targets -- -D warnings` passes with 0 warnings.
- No new crates added; only `serde`, `serde_json` (both already in Cargo.toml) used.

### File List

- `src/jsonl_parser.rs` (new)
- `src/main.rs` (modified — added `mod jsonl_parser;`)

## Change Log

- 2026-04-11: Implemented JSONL parser module (Story 2.4) — created `src/jsonl_parser.rs` with `DailyActivity`, `parse_date_range`, recursive file discovery, per-file session parsing, date-range filtering, and 7 co-located unit tests. Wired module into `src/main.rs`.
