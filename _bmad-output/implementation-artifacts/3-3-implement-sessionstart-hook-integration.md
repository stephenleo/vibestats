# Story 3.3: Implement SessionStart Hook Integration

Status: done

<!-- GH Issue: #20 | Epic: #3 | PR must include: Closes #20 -->

## Story

As the vibestats system,
I want the SessionStart hook to perform catch-up sync, check staleness, surface auth errors, and detect machine retirement,
so that missed syncs are recovered and the user is warned about issues at session start — not mid-session.

## Acceptance Criteria

**Behaviour 1: Machine retirement detection**

1. **Given** this machine's `machine_id` appears as `retired` in `registry.json` **When** SessionStart checks the registry **Then** it updates `machine_status = "retired"` in `checkpoint.toml`, prints a warning, skips catch-up sync, and exits 0

**Behaviour 2: Auth error surface**

2. **Given** `checkpoint.toml` has `auth_error = true` **When** SessionStart fires **Then** it prints: `"vibestats: auth error detected. Run \`vibestats auth\` to re-authenticate."` and clears the flag (FR40)

**Behaviour 3: Catch-up sync**

3. **Given** there are dates between `last_sync_date` and yesterday with no pushed data **When** SessionStart fires (and machine is not retired) **Then** it calls `sync::run(last_sync_date, yesterday)` to fill the gap (FR13)

**Behaviour 4: Staleness warning**

4. **Given** the last successful sync was more than 24 hours ago **When** SessionStart fires **Then** it prints: `"vibestats: last sync was N days ago on this machine. Run \`vibestats status\` to diagnose."` (FR19)

## Tasks / Subtasks

- [x] Task 1: Add `get_file_content` method to `src/github_api.rs` (AC: #1)
  - [x] Add `pub fn get_file_content(&self, path: &str) -> Result<Option<String>, GithubApiError>` — returns base64-decoded file content, or `Ok(None)` on 404
  - [x] Extract base64-decoded content from the `"content"` field in the GitHub Contents API JSON response (field uses `base64` encoding with newlines — strip `\n` before decoding)
  - [x] Use the existing `with_retry` wrapper for backoff on 429/5xx
  - [x] On 401: return `Err` (same as `get_file_sha`)
  - [x] Add a `base64_decode` helper (std-only, no new crates — see Dev Notes for implementation)
  - [x] Write co-located unit test for `base64_decode` using a known vector

- [x] Task 2: Add `get_last_sync_date` helper to `src/checkpoint.rs` (AC: #3, #4)
  - [x] Add `pub fn get_last_sync_date(&self) -> Option<String>` — returns the most recent date key in `date_hashes` (format `"YYYY-MM-DD"`)
  - [x] Return `None` if `date_hashes` is empty
  - [x] Use `date_hashes.keys().max().cloned()` — lexicographic max is correct for `"YYYY-MM-DD"` strings (dates are already zero-padded by `jsonl_parser.rs`)
  - [x] Write co-located unit test verifying the max-key logic with multiple date entries

- [x] Task 3: Create `src/hooks/session_start.rs` with `pub fn run()` (AC: #1, #2, #3, #4)
  - [x] Step 1 — Machine retirement check:
    - [x] Load config via `Config::load_or_exit()` and checkpoint via `Checkpoint::load(&checkpoint_path)`
    - [x] Create `GithubApi::new(&config.oauth_token, &config.vibestats_data_repo)`
    - [x] Call `api.get_file_content("registry.json")` — if `Ok(None)`, skip retirement check and continue to Step 2
    - [x] If registry fetched: parse JSON, find the machine entry where `"machine_id" == config.machine_id`
    - [x] If entry has `"status": "retired"`: call `checkpoint.set_machine_status("retired")`, save checkpoint, print `"vibestats: this machine has been retired. Sync skipped."`, and **return early** (Steps 2–4 are skipped; `main.rs` calls `exit(0)`)
    - [x] If registry fetch fails (`Err`): log via `logger::error`, continue to Step 2 (retirement check failure is non-fatal)
  - [x] Step 2 — Auth error surface:
    - [x] If `checkpoint.auth_error == true`: print `"vibestats: auth error detected. Run \`vibestats auth\` to re-authenticate."`, call `checkpoint.clear_auth_error()`, save checkpoint before returning (auth_error must be cleared even if subsequent steps also call save)
  - [x] Step 3 — Catch-up sync:
    - [x] Guard: only proceed if `checkpoint.machine_status != "retired"`
    - [x] Compute `last_sync_date = checkpoint.get_last_sync_date()` — if `None`, skip catch-up (no previous sync recorded)
    - [x] Compute `yesterday` as today's date minus one day in `"YYYY-MM-DD"` format (see Dev Notes)
    - [x] If `last_sync_date < yesterday` (string comparison is correct for `YYYY-MM-DD`): call `sync::run(&last_sync_date, &yesterday)`
    - [x] If `last_sync_date >= yesterday`: no gap — skip catch-up
  - [x] Step 4 — Staleness warning:
    - [x] Reload checkpoint from disk after `sync::run` (sync may have updated `throttle_timestamp`)
    - [x] If `checkpoint.throttle_timestamp` is `None`: skip staleness warning (no sync has ever run)
    - [x] Compute elapsed seconds since `checkpoint.throttle_timestamp` (see Dev Notes)
    - [x] If elapsed > 86400 seconds (24 hours): print `"vibestats: last sync was N days ago on this machine. Run \`vibestats status\` to diagnose."` where `N = elapsed_seconds / 86400`

- [x] Task 4: Create `src/hooks/mod.rs` declaring `pub mod session_start;`

- [x] Task 5: Wire `hooks` module into `src/main.rs` (AC: #1, #2, #3, #4)
  - [x] Add `mod hooks;` to `src/main.rs` alongside existing `mod` declarations
  - [x] Add a `SessionStart` subcommand to the `Commands` enum in `main.rs` (used by Claude Code hook: `command: vibestats session-start`)
  - [x] In the `Commands::SessionStart` match arm: call `hooks::session_start::run(); std::process::exit(0);`
  - [x] Do NOT add `#![allow(dead_code)]` to `session_start.rs` — it is a new file that is immediately called from `main.rs`

- [x] Task 6: Write co-located unit tests in `src/hooks/session_start.rs` (AC: #1, #2, #3, #4)
  - [x] Test `yesterday()` helper returns a valid `"YYYY-MM-DD"` string
  - [x] Test `days_since_timestamp` returns correct values for known inputs
  - [x] Test that auth error message is printed and flag cleared when `auth_error = true`
  - [x] Test that catch-up sync is skipped when `machine_status == "retired"`
  - [x] Test that staleness warning fires when elapsed > 24 hours
  - [x] Run `cargo test` — must pass with 0 failures
  - [x] Run `cargo clippy --all-targets -- -D warnings` — must produce 0 warnings

## Dev Notes

### Execution Order (CRITICAL — Must Not Reorder)

The four behaviours must execute in this fixed order on every SessionStart:

1. Machine retirement check — if retired: save checkpoint, print warning, **return early** (Steps 2–4 skipped)
2. Auth error surface — print message and clear `auth_error` flag if set
3. Catch-up sync — calls `sync::run(last_sync_date, yesterday)` if gap exists and not retired
4. Staleness warning — reloads checkpoint, prints warning if last sync > 24h ago

**Retirement causes early return: Steps 2–4 do NOT execute if the machine is retired.**

### Module File Locations

```
src/
├── main.rs              ← add `mod hooks;`, add `SessionStart` command
├── hooks/
│   ├── mod.rs           ← NEW: `pub mod session_start;`
│   └── session_start.rs ← NEW: `pub fn run()`
├── checkpoint.rs        ← MODIFIED: add `get_last_sync_date()`
├── github_api.rs        ← MODIFIED: add `get_file_content()` + `base64_decode`
└── sync.rs              ← EXISTING: call `sync::run(last_sync_date, yesterday)`
```

**Note:** Story 3.2 (Stop hook integration) may create `src/hooks/mod.rs` and the `hooks/` directory. If it does, do NOT create a second `mod.rs`. Instead, add `pub mod session_start;` to the existing `src/hooks/mod.rs`. Check whether `src/hooks/mod.rs` exists before creating it.

### SessionStart Hook Registration

The hook is registered in `~/.claude/settings.json` (done by installer in Epic 6). The command is `vibestats session-start` (kebab-case per CLI conventions). This story implements the binary handler — hook registration is handled by Story 6.4.

```json
{
  "hooks": {
    "SessionStart": [{ "hooks": [{ "type": "command", "command": "vibestats session-start" }] }]
  }
}
```

The `session-start` subcommand maps to `Commands::SessionStart` via clap's automatic kebab-case conversion.

### `get_file_content` Implementation

The GitHub Contents API response includes a base64-encoded `"content"` field with embedded newlines (GitHub wraps at 60 chars). Strip `\n` before decoding:

```rust
pub fn get_file_content(&self, path: &str) -> Result<Option<String>, GithubApiError> {
    with_retry(|| get_file_content_inner(&self.token, &self.repo, path))
}
```

Inner function follows the same pattern as `get_file_sha_inner` but also extracts `"content"`:

```rust
fn get_file_content_inner(token: &str, repo: &str, path: &str) -> Result<Option<String>, ureq::Error> {
    // ... same GET call as get_file_sha_inner ...
    // On 200: extract json["content"].as_str(), strip '\n', base64_decode, return Ok(Some(decoded))
    // On 404: return Ok(None)
    // On other error: return Err
}
```

### Base64 Decode — Std-Only Implementation

GitHub Contents API uses standard base64 (RFC 4648 Table 1, with `+` and `/`, padded with `=`). Implement without `base64` crate:

```rust
fn base64_decode(input: &str) -> Result<String, &'static str> {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    // Build reverse lookup: 256-byte array mapping ASCII -> 6-bit value (255 = invalid)
    let mut rev = [255u8; 256];
    for (i, &c) in TABLE.iter().enumerate() {
        rev[c as usize] = i as u8;
    }
    let input: Vec<u8> = input.bytes().filter(|&b| b != b'=').collect();
    let mut out = Vec::new();
    for chunk in input.chunks(4) {
        // Map each byte through reverse table
        let vals: Vec<u8> = chunk.iter().map(|&b| rev[b as usize]).collect();
        if vals.iter().any(|&v| v == 255) {
            return Err("invalid base64 character");
        }
        match vals.len() {
            4 => { out.push((vals[0] << 2) | (vals[1] >> 4)); out.push((vals[1] << 4) | (vals[2] >> 2)); out.push((vals[2] << 6) | vals[3]); }
            3 => { out.push((vals[0] << 2) | (vals[1] >> 4)); out.push((vals[1] << 4) | (vals[2] >> 2)); }
            2 => { out.push((vals[0] << 2) | (vals[1] >> 4)); }
            _ => {}
        }
    }
    String::from_utf8(out).map_err(|_| "base64 decoded bytes are not valid UTF-8")
}
```

**Test vector:** `base64_decode("aGVsbG8=")` → `"hello"` (strip `=` → `"aGVsbG8"`, decode → `[104, 101, 108, 108, 111]`)

### `yesterday()` Helper

Compute yesterday's date as `"YYYY-MM-DD"` using std only:

```rust
fn yesterday() -> String {
    let now = std::time::SystemTime::now();
    let secs = now.duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
    // Subtract 86400 seconds (one day), then format
    let yesterday_secs = secs.saturating_sub(86400);
    // Use the same civil-date formula as checkpoint.rs format_iso8601_utc
    // but only return the date portion "YYYY-MM-DD"
    let z = yesterday_secs / 86400 + 719468;
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
```

**Do NOT import `chrono` or any date crate.** The civil-date formula above is already established in `checkpoint.rs` — replicate the pattern.

### Staleness Calculation

To determine `N` days since last sync (for the staleness warning):

```rust
fn days_since_timestamp(ts_str: &str) -> Option<u64> {
    // Parse ISO 8601 UTC string using the same parse_iso8601_utc logic from checkpoint.rs
    // (checkpoint.rs has a private parse_iso8601_utc — do NOT re-export; re-implement inline or copy pattern)
    // Compute: (SystemTime::now() - ts) / 86400 seconds
}
```

Use `checkpoint.throttle_timestamp` as the proxy for last sync time. If `None`, skip the staleness warning (no sync has ever run). If > 24 hours, print with `N = elapsed_seconds / 86400`.

**Note:** `parse_iso8601_utc` is a private function in `checkpoint.rs`. Implement the date parsing inline in `session_start.rs` or use a simpler approach: parse the UNIX epoch offset from the `"YYYY-MM-DDTHH:MM:SSZ"` string manually (same algorithm, just copy-and-adapt).

### Registry JSON Parsing

The `registry.json` content returned by `get_file_content` is parsed as `serde_json::Value`:

```rust
let json: serde_json::Value = serde_json::from_str(&content).unwrap_or(serde_json::Value::Null);
let machines = json["machines"].as_array();
if let Some(machines) = machines {
    for m in machines {
        if m["machine_id"].as_str() == Some(&config.machine_id) {
            if m["status"].as_str() == Some("retired") {
                // handle retirement
            }
            break;
        }
    }
}
```

On any JSON parse failure: log and continue — do not abort SessionStart.

### Error Handling Contract

| Error | Action in `session_start.rs` |
|---|---|
| `get_file_content` returns `Err` (registry fetch fail) | `logger::error(...)`, continue (retirement check skipped) |
| `get_file_content` returns `Ok(None)` (404) | Skip retirement check — no registry yet |
| `serde_json` parse failure on registry | `logger::error(...)`, continue |
| `sync::run` internal errors | Handled inside `sync::run` — exits 0 per NFR10 |
| `checkpoint.save` fails | `logger::error(...)`, do NOT exit non-zero |

**All code paths exit 0.** `session_start::run()` returns `()` — `main.rs` calls `std::process::exit(0)`.

### Existing Public APIs to Use

**`checkpoint::Checkpoint`** (add `get_last_sync_date` in this story):
```rust
pub fn load(path: &Path) -> Self
pub fn save(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>>
pub fn auth_error: bool          // field — read directly
pub fn clear_auth_error(&mut self)
pub fn set_machine_status(&mut self, status: &str)
pub fn is_retired(&self) -> bool
pub fn throttle_timestamp: Option<String>  // field — read directly
// NEW in this story:
pub fn get_last_sync_date(&self) -> Option<String>
```

**`config::Config`:**
```rust
pub fn load_or_exit() -> Config
// Fields: oauth_token: String, machine_id: String, vibestats_data_repo: String
```

**`github_api::GithubApi`** (add `get_file_content` in this story):
```rust
pub fn new(token: &str, repo: &str) -> Self
pub fn put_file(&self, path: &str, content: &str) -> Result<(), GithubApiError>
pub fn get_file_sha(&self, path: &str) -> Result<Option<String>, GithubApiError>
// NEW in this story:
pub fn get_file_content(&self, path: &str) -> Result<Option<String>, GithubApiError>
```

**`sync::run`:**
```rust
pub fn run(start_date: &str, end_date: &str)
// Loads config/checkpoint internally; always exits 0; no return value to inspect
```

**`logger`:**
```rust
pub fn error(message: &str)
pub fn warn(message: &str)
pub fn info(message: &str)
```

### Checkpoint Path Helper

Use the same checkpoint path pattern established in `sync.rs`:

```rust
fn checkpoint_path() -> Option<std::path::PathBuf> {
    std::env::var("HOME").ok().map(|h| {
        std::path::PathBuf::from(h)
            .join(".config")
            .join("vibestats")
            .join("checkpoint.toml")
    })
}
```

If `HOME` is not set, use `Checkpoint::default()` and skip saving.

### Existing Crates (No New Dependencies Allowed)

All required crates are already in `Cargo.toml`:

```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
ureq = "2.10"
toml = "0.8"
```

**Do NOT add:** `chrono`, `base64`, `time`, or any date/encoding crate.

### `#![allow(dead_code)]` Pattern

Do NOT add `#![allow(dead_code)]` to `session_start.rs` — this function is called from `main.rs` directly in this story.

### Worktree / Cargo Isolation

The worktree is nested inside the main repo. `Cargo.toml` at the repo root already has `[workspace]` set. Do NOT add another `[workspace]` section.

Run all verification from the **repo root** (not from inside the worktree):
```bash
cargo build
cargo test
cargo clippy --all-targets -- -D warnings
```

### Unique Temp Files in Tests

If tests write any files, use the established pattern from prior stories:

```rust
use std::sync::atomic::{AtomicU64, Ordering};
static COUNTER: AtomicU64 = AtomicU64::new(0);
let n = COUNTER.fetch_add(1, Ordering::SeqCst);
let name = format!("vibestats_test_{}_{}_{}", std::process::id(),
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default().as_nanos(), n);
let path = std::env::temp_dir().join(name);
```

### Previous Story Learnings

From Story 3.1 (`sync.rs`):
- `sync::run(start_date, end_date)` returns `()` — callers do NOT inspect return values
- `sync.rs` NEVER calls `std::process::exit` — only `main.rs` and hook handlers do
- Config and checkpoint are loaded internally by `sync::run` — do NOT pass them in
- `#![allow(dead_code)]` is required for callers that arrive in future stories; NOT needed here (this is called from main.rs)
- `cargo clippy --all-targets -- -D warnings` catches all-targets warnings
- PRs must include `Closes #20` in the description

From Story 2.3 (`checkpoint.rs`):
- Atomic save via tmp + rename pattern (already in `checkpoint.save` — do not replicate)
- `std::process::exit` must never be called inside modules — only by `main.rs` or hook handlers
- Use `std::env::temp_dir()` for test temp files (not `/tmp` directly)

From Story 2.5 (`github_api.rs`):
- `#![allow(clippy::result_large_err)]` is needed when `ureq::Error` is in a return type — use on `get_file_content_inner`
- `with_retry` wraps the inner function — do NOT inline retry logic

### Architecture Constraints Summary

| Constraint | NFR/Source | Impact on This Story |
|---|---|---|
| Exit 0 on all errors | NFR10 | `session_start::run()` returns `()` — `main.rs` calls `exit(0)` |
| Silent during session (surface at SessionStart) | NFR11 | Auth error + staleness printed to stdout at SessionStart only |
| No async runtime | architecture.md | `session_start::run()` is synchronous |
| Single HTTP module | architecture.md | All GitHub API calls via `github_api.rs` — session_start.rs never calls `ureq` directly |
| No new crates | story scope | Base64, date math: std-only |
| snake_case filenames | architecture.md | Files: `src/hooks/mod.rs`, `src/hooks/session_start.rs` |
| Co-located unit tests | architecture.md | `#[cfg(test)]` module inside `session_start.rs` |

### Anti-Patterns to Prevent

- Do NOT call `std::process::exit` in `session_start.rs` — only `main.rs` calls exit
- Do NOT add `chrono`, `base64`, or any crate — implement everything std-only
- Do NOT write to stdout/stderr in non-test code EXCEPT for the four prescribed user-facing messages (warning messages are intentional stdout output at SessionStart)
- Do NOT inline GitHub API HTTP calls — all calls go through `github_api.rs`
- Do NOT re-export or `pub use` the private `parse_iso8601_utc` from `checkpoint.rs` — re-implement the date parsing inline
- Do NOT call `sync::run` if machine is retired
- Do NOT skip early return on machine retirement — Steps 2–4 must not execute if machine is retired
- Do NOT add a second `[workspace]` to `Cargo.toml`

### Project Structure Notes

- New files: `src/hooks/mod.rs`, `src/hooks/session_start.rs`
- Modified files:
  - `src/main.rs` — add `mod hooks;`, add `SessionStart` arm to `Commands` enum
  - `src/checkpoint.rs` — add `get_last_sync_date()` method
  - `src/github_api.rs` — add `get_file_content()` public method + `get_file_content_inner` private fn + `base64_decode` private fn

### References

- Story requirements: [Source: _bmad-output/planning-artifacts/epics.md#Story 3.3]
- Epic 3 context: [Source: _bmad-output/planning-artifacts/epics.md#Epic 3]
- Sync operation spec (SessionStart catch-up): [Source: _bmad-output/planning-artifacts/architecture.md#API & Communication Patterns — Date range per operation]
- Auth validation strategy: [Source: _bmad-output/planning-artifacts/architecture.md#Authentication & Security — Validation strategy]
- Silent failure contract: [Source: _bmad-output/planning-artifacts/architecture.md#Cross-Cutting Concerns — Silent failure contract]
- registry.json schema: [Source: docs/schemas.md#4. registry.json]
- checkpoint.toml schema (auth_error, machine_status, throttle_timestamp): [Source: docs/schemas.md#checkpoint.toml]
- FR13 (catch-up sync on SessionStart): [Source: _bmad-output/planning-artifacts/epics.md#Functional Requirements]
- FR19 (staleness warning): [Source: _bmad-output/planning-artifacts/epics.md#Functional Requirements]
- FR40 (auth error on SessionStart): [Source: _bmad-output/planning-artifacts/epics.md#Functional Requirements]
- NFR10 (hook non-interference): [Source: _bmad-output/planning-artifacts/epics.md#Non-Functional Requirements]
- NFR11 (silent sync failure): [Source: _bmad-output/planning-artifacts/epics.md#Non-Functional Requirements]
- Remote machine retirement via eventual consistency: [Source: _bmad-output/planning-artifacts/epics.md#Additional Decisions & Clarifications]
- Module boundary rules: [Source: _bmad-output/planning-artifacts/architecture.md#Module responsibility boundaries (Rust)]
- Previous story patterns (sync.rs): [Source: _bmad-output/implementation-artifacts/3-1-implement-core-sync-orchestration.md#Dev Notes]
- GH Issue: #20

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

None — implementation completed without issues.

### Completion Notes List

- Implemented `get_file_content` method on `GithubApi` with inner helper `get_file_content_inner` following the same pattern as `get_file_sha_inner`. Added `base64_decode` std-only helper using RFC 4648 reverse-lookup table. Fixed clippy warning: used `vals.contains(&255)` instead of `vals.iter().any(|&v| v == 255)`.
- Added `get_last_sync_date()` to `Checkpoint` using `date_hashes.keys().max().cloned()` — lexicographic max is correct for zero-padded ISO 8601 dates.
- Created `src/hooks/session_start.rs` with `pub fn run()` implementing all four behaviours in the correct order: retirement check → auth error surface → catch-up sync → staleness warning. Re-implemented `parse_iso8601_utc` inline per architecture constraints (private function in checkpoint.rs must not be re-exported).
- Created `src/hooks/mod.rs` declaring `pub mod session_start;`.
- Wired `mod hooks;` into `main.rs`, added `Commands::SessionStart` variant with clap's automatic kebab-case conversion, and wired it to `hooks::session_start::run(); std::process::exit(0);`.
- All 93 tests pass (74 pre-existing + 19 new). Zero clippy warnings.

### File List

- `src/github_api.rs` — added `get_file_content()`, `get_file_content_inner()`, `base64_decode()` + 5 new unit tests
- `src/checkpoint.rs` — added `get_last_sync_date()` + 3 new unit tests
- `src/hooks/mod.rs` — NEW: `pub mod session_start;`
- `src/hooks/session_start.rs` — NEW: `pub fn run()` + 11 unit tests
- `src/main.rs` — added `mod hooks;`, `Commands::SessionStart` variant, match arm

### Change Log

- 2026-04-11: Implemented Story 3.3 — SessionStart hook integration. Added `get_file_content` to `GithubApi`, `get_last_sync_date` to `Checkpoint`, created `src/hooks/session_start.rs` with four-step SessionStart behaviour (retirement check, auth error surface, catch-up sync, staleness warning), wired `SessionStart` subcommand into `main.rs`. 93 tests pass, 0 clippy warnings.
