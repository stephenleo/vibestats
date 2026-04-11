# Story 2.2: Implement Logger Module

Status: done

<!-- GH Issue: #14 | Epic: #2 | PR must include: Closes #14 -->

## Story

As the vibestats binary,
I want a logger module that appends structured entries to `~/.config/vibestats/vibestats.log`,
so that sync failures and diagnostics are captured without output to stdout during hooks.

## Acceptance Criteria

1. **Given** a log entry is written **When** the log file is inspected **Then** every line follows the format: `YYYY-MM-DDTHH:MM:SSZ LEVEL message` (UTC timestamp)

2. **Given** the log file reaches 1MB **When** the next entry is written **Then** the file is rotated (old renamed to `vibestats.log.1`, new file started)

3. **Given** a hook is firing during a Claude Code session **When** any log write occurs **Then** nothing is written to stdout or stderr (log-only, silent to the terminal) (NFR10, NFR11)

## Tasks / Subtasks

- [x] Task 1: Create `src/logger.rs` with the logger implementation (AC: #1, #2, #3)
  - [x] Define public `log(level: &str, message: &str)` function (or equivalent API — see Dev Notes)
  - [x] Resolve log directory path as `~/.config/vibestats/vibestats.log` (same dir as `config.toml`)
  - [x] Write formatted log lines: `YYYY-MM-DDTHH:MM:SSZ LEVEL message\n` (UTC, no local timezone)
  - [x] Open file in append mode — never truncate existing log content
  - [x] Ensure no output goes to stdout or stderr on any code path (swallow all IO errors silently)

- [x] Task 2: Implement 1MB rotation (AC: #2)
  - [x] Before writing, check file size (using `metadata().len()`)
  - [x] If size >= 1,048,576 bytes (1MB), rename `vibestats.log` → `vibestats.log.1` (overwrite if exists)
  - [x] Open a fresh `vibestats.log` after rotation
  - [x] If rotation fails (e.g., permission error), silently discard and continue — never panic

- [x] Task 3: Wire logger into `src/main.rs` (AC: #3)
  - [x] Add `mod logger;` declaration in `src/main.rs`
  - [x] Do NOT call `logger::log(...)` from `main.rs` yet — just declare the module so it compiles
  - [x] Verify `cargo build` and `cargo clippy -- -D warnings` pass with 0 errors/warnings

- [x] Task 4: Write co-located unit tests in `src/logger.rs` (AC: #1, #2)
  - [x] Test: log entry format matches `YYYY-MM-DDTHH:MM:SSZ LEVEL message` regex pattern
  - [x] Test: multiple entries are appended (not overwritten) to the file
  - [x] Test: rotation renames file at exactly 1MB threshold
  - [x] Use `tempdir` or `std::env::temp_dir()` for test isolation — do NOT write to the real `~/.config/vibestats/` in tests
  - [x] All tests pass with `cargo test`

## Dev Notes

### Context: What Previous Stories Created

Story 1.2 created `src/main.rs` and `Cargo.toml` with these dependencies already declared:
```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
ureq = "2.10"
toml = "0.8"
```

Story 1.1 established the monorepo structure. The `src/` directory currently contains only `src/main.rs`.

**IMPORTANT: Do NOT add `[workspace]` to `Cargo.toml`** — the Story 1.2 dev agent added it to the worktree-local copy to avoid cargo worktree traversal. The main repo's `Cargo.toml` already has the `[workspace]` entry. Do not change `Cargo.toml` unless adding a new dependency.

**No new crates needed** — all functionality (file I/O, timestamps) is achievable with Rust stdlib (`std::fs`, `std::time`). Do not add `chrono`, `log`, `env_logger`, or any logging crate. This is an intentional architectural decision: keep the binary dependency surface minimal.

### File to Create

**`src/logger.rs`** — the only new file this story creates.

Architecture spec location: `src/logger.rs` — "Append to vibestats.log, TIMESTAMP LEVEL msg format"

### Log File Location

Log file: `~/.config/vibestats/vibestats.log`

The config module (Story 2.1, parallel story) writes to the same directory `~/.config/vibestats/`. For logger.rs, resolve the path independently using `dirs`-free stdlib approach:

```rust
fn log_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".config").join("vibestats").join("vibestats.log")
}
```

Create the directory if it doesn't exist — use `if let Some(parent) = log_path().parent() { let _ = std::fs::create_dir_all(parent); }` — no `unwrap()`, ignore errors (silent failure contract).

**Do NOT import or call functions from `config.rs`** — Story 2.1 is a parallel story being developed simultaneously. `logger.rs` must be fully self-contained with no dependencies on other vibestats modules.

### Log Line Format

From architecture.md:
```
YYYY-MM-DDTHH:MM:SSZ LEVEL message
```

Examples:
```
2026-04-10T14:23:01Z ERROR sync failed: 401 Unauthorized — run `vibestats auth`
2026-04-10T14:28:07Z INFO  sync skipped: throttle active (last sync 3m ago)
```

**Timestamp requirements:**
- UTC only — never local timezone
- Format: `YYYY-MM-DDTHH:MM:SSZ` (T separator, Z suffix for UTC)
- Use `std::time::SystemTime::now()` and convert manually, or format via stdlib

**LEVEL field:** Right-pad or left-align levels so columns align visually. The spec shows `INFO ` with trailing space in the example — this is a cosmetic preference. The strict requirement is `LEVEL` followed by a space and then the message.

**Newline:** Each log entry ends with `\n`.

### Timestamp Generation Without chrono

Use `std::time::SystemTime` + `std::time::UNIX_EPOCH`:

```rust
use std::time::{SystemTime, UNIX_EPOCH};

fn utc_timestamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    // Convert epoch seconds to YYYY-MM-DDTHH:MM:SSZ
    let (y, mo, d, h, min, s) = epoch_to_datetime(secs);
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, mo, d, h, min, s)
}
```

Implement `epoch_to_datetime` using standard Julian Day / Gregorian calendar arithmetic, or use `std::time::OffsetDateTime` from the time crate — but do NOT add any new crate to Cargo.toml. A pure stdlib approach is preferred to keep the dependency surface minimal. A correct implementation of `epoch_to_datetime` using basic arithmetic is acceptable.

### Rotation Logic

1MB threshold = 1,048,576 bytes (1024 * 1024).

```rust
fn rotate_if_needed(log_path: &Path) {
    if let Ok(meta) = std::fs::metadata(log_path) {
        if meta.len() >= 1_048_576 {
            let rotated = log_path.with_extension("log.1");
            let _ = std::fs::rename(log_path, rotated); // ignore errors
        }
    }
}
```

Only keep one rotated file (`.log.1`). If `.log.1` already exists, it is overwritten by rename. No gzip compression required.

### Silent Failure Contract (Critical)

**NFR10 — hook errors must never propagate to Claude Code:**

The logger is called from hot paths (Stop hook, SessionStart hook). Any IO error in the logger itself must be silently discarded:

```rust
pub fn log(level: &str, message: &str) {
    // All operations wrapped — errors silently ignored
    let _ = write_log_entry(level, message); // If this returns Err, drop it
}
```

**Never:**
- `unwrap()` on file operations
- `expect(...)` on path resolution
- `panic!()` or propagate `Result` out of the public API
- Write to stdout or stderr under any circumstances

**Always:**
- Use `let _ = ...` or `.ok()` to discard IO errors
- Return unit from the public API

### Public API Design

Minimal public surface:

```rust
// Required: called by all other modules to log events
pub fn log(level: &str, message: &str) { ... }

// Optional convenience wrappers (recommended for cleaner call sites):
pub fn info(message: &str) { log("INFO", message); }
pub fn error(message: &str) { log("ERROR", message); }
pub fn warn(message: &str) { log("WARN", message); }
```

Callers in other modules will use:
```rust
use crate::logger;
logger::error("sync failed: 401 Unauthorized — run `vibestats auth`");
logger::info("sync skipped: throttle active (last sync 3m ago)");
```

### Module Declaration in main.rs

Add at the top of `src/main.rs`:
```rust
mod logger;
```

No `use` statement needed in main.rs itself (other modules will import via `use crate::logger`). Just declare the module to make it part of the crate.

### Testing Without Writing to ~/.config/vibestats/

Tests MUST use temporary directories:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_entry_to(path: &std::path::Path, level: &str, msg: &str) {
        // Internal helper that accepts a path argument for testability
    }

    #[test]
    fn test_log_format() {
        let dir = std::env::temp_dir().join("vibestats_test_logger");
        let _ = fs::create_dir_all(&dir);
        let log_file = dir.join("test.log");
        // Write entry and verify format
        // ...
        let _ = fs::remove_dir_all(&dir);
    }
}
```

Design the internal functions to accept a `&Path` argument for testability. The public `log()` function calls the internal function with the real log path.

### Architecture Constraints Summary

| Constraint | Source | Requirement |
|---|---|---|
| No external logging crates | architecture.md | No `chrono`, `log`, `env_logger`, `tracing` |
| Exit 0 on all error paths | architecture.md#Process Patterns | Swallow all IO errors in logger |
| No stdout/stderr output | NFR10, NFR11 | Logger writes ONLY to file |
| Log format | architecture.md#Format Patterns | `YYYY-MM-DDTHH:MM:SSZ LEVEL message` |
| UTC timestamps | architecture.md#Format Patterns | Never local timezone |
| File location | architecture.md#Data Architecture | `~/.config/vibestats/vibestats.log` |
| Rolling 1MB max | architecture.md#Data Architecture | Rename to `.log.1`, keep only one backup |
| snake_case filenames | architecture.md#Naming Patterns | `logger.rs` (already correct) |
| Co-located tests | architecture.md#Test Placement | `#[cfg(test)]` module in `logger.rs` |

### Files to Touch

- `src/logger.rs` — **create** (new file, entire implementation + tests)
- `src/main.rs` — **modify** (add `mod logger;` declaration only)
- `_bmad-output/implementation-artifacts/2-2-implement-logger-module.md` — **modify** (story status + dev record)

**Do NOT touch:**
- `Cargo.toml` — no new dependencies needed
- `Cargo.lock` — auto-updated by cargo if needed (should not change since no new deps)
- Any file outside `src/` except the story file itself

### Sprint Context

- Story 2.2 is part of Epic 2: Rust Binary — Foundation Modules (GitHub Issue #2)
- Story 2.1 (config module) and Story 2.3 (checkpoint module) are being developed in parallel worktrees
- `logger.rs` must be self-contained with NO imports from `config.rs`, `checkpoint.rs`, or any other not-yet-implemented vibestats module
- Stories 2.3–2.5 will call into `logger.rs` once it is merged — the public API defined here becomes a contract

### References

- Log format: [Source: architecture.md#Format Patterns — Error log format]
- File location: [Source: architecture.md#Data Architecture — Local checkpoint]
- Silent failure contract: [Source: architecture.md#Process Patterns — Silent failure contract]
- NFR10: "crash or error in vibestats hook must not propagate to Claude Code; all hook errors caught and logged locally"
- NFR11: "sync failures must fail silently during sessions; user notified only at next SessionStart"
- Story 2.2 ACs: [Source: epics.md#Story 2.2: Implement logger module]
- Module file layout: [Source: architecture.md#Complete Project Directory Structure]
- Previous story file list: [Source: implementation-artifacts/1-2-initialize-rust-binary-project.md#File List]

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

- Clippy `manual_div_ceil` lint triggered on Julian Day Number algorithm; replaced with a simple iterative year/month extraction that avoids the pattern entirely.
- `#[allow(dead_code)]` module attribute added at the top of `logger.rs` because `main.rs` only declares `mod logger;` without calling any functions (per story spec); this suppresses dead-code lints so `cargo clippy -- -D warnings` passes.
- Epoch test constant for 2025-04-10 (1744243200) was initially labelled 2026-04-10 — corrected with proper constants verified against `utc_timestamp()` live output.

### Completion Notes List

- Implemented `src/logger.rs` with fully self-contained stdlib-only implementation (no chrono, no external logging crates).
- Public API: `log(level, message)`, `info(message)`, `error(message)`, `warn(message)`.
- Timestamp generation via `epoch_to_datetime` using iterative Gregorian calendar arithmetic.
- Log rotation at exactly 1,048,576 bytes — old file renamed to `vibestats.log.1`, new file started.
- Silent failure contract enforced throughout: all IO errors discarded via `let _ = ...` / `.ok()`, no `unwrap()` or `panic!()` on any IO path.
- Added `mod logger;` to `src/main.rs` — no calls made (per spec).
- 8 unit tests added in `#[cfg(test)]` module within `logger.rs`, all using `std::env::temp_dir()` for isolation.
- `cargo build`, `cargo clippy -- -D warnings`, and `cargo test` all pass with 0 errors.

### File List

- `src/logger.rs` — created (new file: full implementation + co-located unit tests)
- `src/main.rs` — modified (added `mod logger;` declaration at top)
- `_bmad-output/implementation-artifacts/2-2-implement-logger-module.md` — modified (story status + dev record)

## Change Log

- 2026-04-11: Story created — ready for dev implementation.
- 2026-04-11: Story implemented — all tasks complete, 8 tests passing, status set to review.
- 2026-04-11: Code review (Step 3) applied fixes — `cargo fmt` compliance; rotation derives rotated file name from the caller-supplied log path (no longer hard-coded to `vibestats.log.1`); cleaned self-contradictory rotation comment; `unique_test_dir` test helper now pre-cleans stale state from prior failed runs; added `test_rotation_preserves_file_stem` asserting the rotation fix; added `.truncate(true)` to three test filler file opens to satisfy `clippy::suspicious_open_options` under `--all-targets`. 9 tests passing; `cargo build`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, and `cargo test` all green. Status set to done.
