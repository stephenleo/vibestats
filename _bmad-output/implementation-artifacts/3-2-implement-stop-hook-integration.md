# Story 3.2: Implement Stop Hook Integration

Status: ready-for-dev

<!-- GH Issue: #19 | Epic: #3 | PR must include: Closes #19 -->

## Story

As the vibestats system,
I want the Stop hook to fire after every Claude Code session response and sync today's data if not throttled,
So that the profile heatmap stays current with zero user action.

## Acceptance Criteria

1. **Given** the Stop hook fires and the last sync was under 5 minutes ago **When** the hook runs **Then** it exits 0 immediately without any API call (NFR2)

2. **Given** the Stop hook fires and the throttle is clear **When** it runs **Then** it calls `sync::run(today, today)` and updates the throttle timestamp in checkpoint on success

3. **Given** the hook is configured **When** `~/.claude/settings.json` is inspected **Then** it contains a `Stop` hook entry with `command: vibestats sync` and `async: true`

## Tasks / Subtasks

- [ ] Task 1: Create `src/hooks.rs` with `pub fn stop_hook()` function (AC: #1, #2)
  - [ ] Add `#![allow(dead_code)]` at top of `hooks.rs` — `session_start_hook()` caller arrives in Story 3.3
  - [ ] Do NOT remove `#![allow(dead_code)]` from `sync.rs` — internal helpers (`sha256_hex`, `hive_path`) remain private and the attribute stays until the compiler no longer needs it
  - [ ] Load checkpoint via `checkpoint_path()` helper (reuse the same pattern from `sync.rs`)
  - [ ] Call `checkpoint.should_throttle()` — if true, exit 0 immediately without calling `sync::run`
  - [ ] If throttle clear: call `sync::run(today, today)` where `today` is `today_utc()`
  - [ ] After `sync::run` returns (it always returns `()`): call `checkpoint.update_throttle_timestamp()`, save checkpoint, exit 0
  - [ ] NEVER call `std::process::exit` directly from hooks.rs — use `main.rs` dispatch (see Task 3 below)
  - [ ] If `checkpoint_path()` returns `None` (HOME unset): skip throttle check and call `sync::run` (fail-open)

- [ ] Task 2: Implement `today_utc()` date helper in `hooks.rs` (AC: #2)
  - [ ] Private `fn today_utc() -> String` — returns current UTC date as `"YYYY-MM-DD"`
  - [ ] Use `std::time::SystemTime::now()` and civil-date formula (same approach as `checkpoint.rs`'s `format_iso8601_utc`) — no `chrono` crate
  - [ ] Extract only the date portion `"YYYY-MM-DD"` (not the full ISO 8601 timestamp)
  - [ ] Must produce zero-padded month and day (e.g., `"2026-04-09"` not `"2026-4-9"`)

- [ ] Task 3: Wire `hooks::stop_hook()` into `main.rs` dispatch (AC: #2)
  - [ ] Add `mod hooks;` to `src/main.rs` alongside existing `mod` declarations
  - [ ] In `main()`, replace single `Commands::Sync { backfill: _ }` arm with two arms: `Commands::Sync { backfill } if !backfill` → call `hooks::stop_hook()` then `std::process::exit(0)`
  - [ ] Leave `Commands::Sync { backfill: true }` and all other commands as `println!("not yet implemented")` — Story 3.4 wires those
  - [ ] `std::process::exit(0)` is the ONLY place in the codebase it is called — always in `main.rs`, never inside modules

- [ ] Task 4: Document hook configuration for `~/.claude/settings.json` (AC: #3)
  - [ ] No code change needed — the installer (Epic 6, Story 6.4) writes this config
  - [ ] Confirm the expected JSON structure in a code comment in `hooks.rs`:
    ```json
    {
      "hooks": {
        "Stop": [{ "hooks": [{ "type": "command", "command": "vibestats sync", "async": true }] }]
      }
    }
    ```
  - [ ] This AC is verified at installer level; this story only implements the binary behaviour

- [ ] Task 5: Write co-located unit tests (AC: #1, #2)
  - [ ] `#[cfg(test)]` module inside `src/hooks.rs`
  - [ ] Test `today_utc()` returns a string matching `"YYYY-MM-DD"` format (10 chars, digit-dash pattern)
  - [ ] Test throttle branch: construct a `Checkpoint` with `update_throttle_timestamp()` just called → `should_throttle()` returns true → stop_hook logic would exit without calling sync (test via direct checkpoint inspection, not full integration)
  - [ ] Test non-throttle branch: construct a `Checkpoint` with old/absent throttle → `should_throttle()` returns false
  - [ ] Run `cargo test` — must pass with 0 failures
  - [ ] Run `cargo clippy --all-targets -- -D warnings` — must produce 0 warnings

## Dev Notes

### Module Responsibility Summary

`hooks.rs` owns **entry-point logic** for each Claude Code hook type. For this story, only the Stop hook is implemented:

| Function | Role |
|---|---|
| `hooks::stop_hook()` | Reads throttle, decides whether to sync, calls `sync::run`, updates throttle |
| `sync::run(today, today)` | Owned by Story 3.1 — does not duplicate any sync logic |
| `checkpoint::should_throttle()` | Already implemented in `checkpoint.rs` — do NOT reimplement |
| `checkpoint::update_throttle_timestamp()` | Already implemented in `checkpoint.rs` — do NOT reimplement |

### `hooks::stop_hook()` Implementation Sketch

The file top must start with:
```rust
#![allow(dead_code)]
// session_start_hook() arrives in Story 3.3
```

Then the stop_hook function:
```rust
pub fn stop_hook() {
    let path = checkpoint_path();
    let mut checkpoint = path
        .as_deref()
        .map(|p| Checkpoint::load(p))
        .unwrap_or_default();

    if checkpoint.should_throttle() {
        return; // throttle active — caller (main.rs) exits 0
    }

    let today = today_utc();
    sync::run(&today, &today);

    // Update throttle timestamp after sync (sync::run always returns ())
    checkpoint.update_throttle_timestamp();
    if let Some(p) = path.as_deref() {
        if let Err(e) = checkpoint.save(p) {
            logger::error(&format!("stop_hook: failed to save checkpoint: {e}"));
        }
    }
    // Return to main.rs which calls std::process::exit(0)
}
```

### `today_utc()` Implementation

Derive current UTC date from `SystemTime::now()`. Use the same civil-date formula already present in `checkpoint.rs`'s `format_iso8601_utc` but return only the date portion:

```rust
fn today_utc() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let z = secs / 86400 + 719468;
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

**Do NOT** use `chrono`, `time`, or any date crate — std-only.

### `main.rs` Dispatch Pattern

Replace the existing single `Commands::Sync { backfill: _ }` arm with two arms:

```rust
// In main():
Commands::Sync { backfill } if !backfill => {
    hooks::stop_hook();
    std::process::exit(0);
}
Commands::Sync { backfill: _ } => println!("not yet implemented"), // backfill=true, Story 3.4
```

Note: `{ backfill: false }` literal pattern match is also valid Rust, but `if !backfill` guard is more idiomatic and avoids clippy warnings on some versions.

`std::process::exit(0)` must be called in `main.rs` AFTER the hook function returns — never inside module code (architecture constraint: modules never call `exit`).

### Checkpoint Path Helper (reuse pattern from sync.rs)

Define the same `checkpoint_path()` private function in `hooks.rs` (copy from `sync.rs` — acceptable duplication for module isolation):

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

If `HOME` is not set: treat as no checkpoint (use `Checkpoint::default()`), skip saving — same fail-open pattern used in `sync.rs`.

### Hook Configuration Reference

The installer (Story 6.4) writes `~/.claude/settings.json`. This story's AC #3 validates the expected format at the installer level. For reference, the expected hook registration:

```json
{
  "hooks": {
    "Stop": [
      {
        "hooks": [
          { "type": "command", "command": "vibestats sync", "async": true }
        ]
      }
    ]
  }
}
```

`async: true` means Claude Code does NOT wait for the binary to finish — ensuring NFR1 (2s hook latency cap) is never breached.

### Imports Required in `hooks.rs`

```rust
use crate::checkpoint::Checkpoint;
use crate::logger;
use crate::sync;
use std::path::PathBuf;
```

### Error Handling Contract

| Scenario | Action in `hooks.rs` |
|---|---|
| `HOME` not set | Treat as no checkpoint — call `sync::run` (fail-open), skip checkpoint save |
| `checkpoint.save` fails | Log via `logger::error`, continue — do NOT exit non-zero |
| `sync::run` (any error) | Handled internally by `sync.rs` — always returns `()` |
| Throttle active | Return immediately — main.rs exits 0 |

**`hooks.rs` NEVER calls `std::process::exit`.** Only `main.rs` calls exit.

### Architecture Constraints

| Constraint | NFR/Source | Impact on This Story |
|---|---|---|
| Exit 0 on all errors | NFR10 | `stop_hook()` returns `()` — exit in `main.rs` only |
| Silent during session | NFR11 | Log via `logger::error` only — no stdout/stderr |
| Throttle: 5-minute window | NFR2 | `checkpoint.should_throttle()` already implements this |
| Hook non-interference | NFR10 | `async: true` in settings.json ensures Claude Code doesn't block |
| No async runtime | architecture.md | `stop_hook()` is synchronous |
| snake_case filenames | architecture.md | File: `src/hooks.rs` |
| Co-located unit tests | architecture.md | `#[cfg(test)]` inside `hooks.rs` |

### Anti-Patterns to Prevent

- Do NOT call `std::process::exit` in `hooks.rs` — only in `main.rs`
- Do NOT re-implement throttle logic — reuse `checkpoint.should_throttle()` from `checkpoint.rs`
- Do NOT re-implement SHA256 or sync logic — that lives in `sync.rs`
- Do NOT add `chrono`, `time`, or any date crate — implement `today_utc()` std-only
- Do NOT write to stdout/stderr in non-test code — use `logger::error/warn/info` only
- Do NOT add a second `[workspace]` to `Cargo.toml`
- Do NOT use `unwrap()` or `expect()` in non-test code
- Do NOT remove `#![allow(dead_code)]` from `sync.rs` — its private helpers (`sha256_hex`, `hive_path`) remain internal and the attribute is still needed
- Add `#![allow(dead_code)]` to `hooks.rs` — `session_start_hook()` will arrive in Story 3.3

### Module File Locations

```
src/
├── main.rs        ← add `mod hooks;`, update Sync dispatch (MODIFIED)
├── hooks.rs       ← NEW FILE (this story)
├── sync.rs        ← EXISTING — provides sync::run() — do NOT modify logic
├── checkpoint.rs  ← EXISTING — provides should_throttle(), update_throttle_timestamp()
├── config.rs      ← EXISTING — loaded internally by sync::run
├── github_api.rs  ← EXISTING — called internally by sync::run
├── jsonl_parser.rs← EXISTING — called internally by sync::run
└── logger.rs      ← EXISTING — provides logger::error()
```

### Existing Public APIs to Use

**`checkpoint::Checkpoint`** (already in `checkpoint.rs`):
```rust
pub fn load(path: &Path) -> Self
pub fn save(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>>
pub fn should_throttle(&self) -> bool        // returns true if throttle_timestamp < 5 min ago
pub fn update_throttle_timestamp(&mut self)  // sets throttle_timestamp = now UTC
```

**`sync::run`** (Story 3.1, already in `sync.rs`):
```rust
pub fn run(start_date: &str, end_date: &str) // synchronous, always returns ()
```

**`logger`** (already in `logger.rs`):
```rust
pub fn error(message: &str)
pub fn warn(message: &str)
pub fn info(message: &str)
```

### Existing Crates (No New Dependencies Allowed)

All required crates are already in `Cargo.toml` — no additions needed:

```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
ureq = "2.10"
toml = "0.8"
```

### Worktree / Cargo Isolation

Run all verification from the **repo root** (not from inside the worktree directory):
```bash
cargo build
cargo test
cargo clippy --all-targets -- -D warnings
```

### Unique Temp Files in Tests

If tests write any files, use unique temp file names (pid + nanos + atomic counter) — pattern from Story 2.3:
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
- `#![allow(dead_code)]` pattern: add at the top of `hooks.rs` until all callers arrive
- `sync::run` never panics or exits — it is safe to call from hooks
- `Config::load_or_exit()` is called inside `sync::run`, not by the caller
- The `checkpoint_path()` helper pattern is the canonical way to get the checkpoint path
- Always save checkpoint after updating throttle — even if no data changed

From Story 2.5 (`github_api.rs`):
- `#![allow(clippy::result_large_err)]` may be needed if `ureq::Error` is in a return type — check after implementation
- `cargo clippy --all-targets -- -D warnings` catches all-targets warnings (not just `-- -D warnings`)

From Story 2.3 (`checkpoint.rs`):
- `std::process::exit` must never be called inside modules — only in `main.rs`
- `Checkpoint::load` is fail-open (returns default on missing/corrupt file) — safe to call on first run

### Project Structure Notes

- New file: `src/hooks.rs`
- Modified file: `src/main.rs` (add `mod hooks;` and update `Commands::Sync` dispatch)
- No other files modified

### References

- Story requirements: [Source: _bmad-output/planning-artifacts/epics.md#Story 3.2]
- Throttle spec (5-minute window): [Source: _bmad-output/planning-artifacts/architecture.md#API & Communication Patterns — Sync operation step 1]
- Stop hook date range: [Source: _bmad-output/planning-artifacts/architecture.md#API & Communication Patterns — Date range per operation]
- NFR2 (5-min throttle): [Source: _bmad-output/planning-artifacts/epics.md#NonFunctional Requirements]
- NFR10 (hook non-interference): [Source: _bmad-output/planning-artifacts/epics.md#NonFunctional Requirements]
- NFR11 (silent sync failure): [Source: _bmad-output/planning-artifacts/epics.md#NonFunctional Requirements]
- Hook configuration schema: [Source: _bmad-output/planning-artifacts/epics.md#Story 6.4]
- Module boundary rules (exit 0): [Source: _bmad-output/planning-artifacts/architecture.md#Process Patterns]
- Previous story patterns: [Source: _bmad-output/implementation-artifacts/3-1-implement-core-sync-orchestration.md#Dev Notes]
- GH Issue: #19

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

### Completion Notes List

### File List
