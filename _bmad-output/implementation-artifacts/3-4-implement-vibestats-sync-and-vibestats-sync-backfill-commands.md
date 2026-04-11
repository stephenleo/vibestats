# Story 3.4: Implement vibestats sync and vibestats sync --backfill Commands

Status: review

<!-- GH Issue: #21 | Epic: #3 | PR must include: Closes #21 -->

## Story

As a developer,
I want to manually trigger a sync or full historical backfill from the CLI,
so that I can recover gaps and verify my setup without waiting for a hook to fire.

## Acceptance Criteria

1. **Given** the user runs `vibestats sync` **When** it executes **Then** it calls `sync::run(today, today)` (unthrottled) and prints `"vibestats: sync complete"` to stdout (FR17)

2. **Given** the user runs `vibestats sync --backfill` **When** it executes **Then** it calls `sync::run` for all dates present in the full JSONL history (start = earliest date, end = today) and reports the count of dates synced and any failures (FR18)

3. **Given** 12 months of JSONL data exists **When** `vibestats sync --backfill` runs **Then** it completes within 60 seconds on standard broadband (NFR3)

4. **Given** `vibestats sync --backfill` is interrupted and run again **When** it resumes **Then** dates already synced (hash match in checkpoint) are skipped — zero API calls for unchanged dates (NFR12)

5. **Given** `vibestats sync` or `vibestats sync --backfill` is run **When** it exits **Then** it exits 0 regardless of any sync errors (NFR10)

## Tasks / Subtasks

- [x] Task 1: Create `src/commands/` directory and `src/commands/mod.rs` (AC: all)
  - [x] Create directory `src/commands/`
  - [x] Create `src/commands/mod.rs` with only `pub mod sync;` — do NOT add stubs for 4.x commands yet (they will be added in their respective stories to avoid dead_code lint)
  - [x] Add `mod commands;` to `src/main.rs` alongside existing `mod` declarations

- [x] Task 2: Implement `src/commands/sync.rs` (AC: #1, #2, #3, #4, #5)
  - [x] Create `src/commands/sync.rs`
  - [x] Implement `pub fn run(backfill: bool)` — the entry point called from `main.rs`
  - [x] For `backfill = false`: compute `today` using std-only UTC date helper (see Dev Notes), call `sync::run(&today, &today)`, print summary to stdout
  - [x] For `backfill = true`: compute `today` using same helper, call `jsonl_parser::parse_date_range("0000-00-00", &today)` to discover all dates, find the earliest date, call `sync::run(&earliest, &today)`, print summary with date count
  - [x] Print summary to stdout after `sync::run` returns: `"vibestats: sync complete"` — `sync::run` returns `()` so there is no "changed vs. skipped" count available at the caller level; all error details are in `vibestats.log`
  - [x] Note: `sync::run` internally logs any failures via `logger::error`; the stdout summary is always positive (the user initiated this deliberately and should check the log if they suspect errors)
  - [x] Do NOT set throttle timestamp — `vibestats sync` is unthrottled by design (only Stop hook throttle applies)
  - [x] No `std::process::exit` in `commands/sync.rs` — `main.rs` handles exit

- [x] Task 3: Wire commands into `main.rs` (AC: #1, #2)
  - [x] In `main.rs` `match cli.command` arm for `Commands::Sync { backfill }`:
    - Replace `println!("not yet implemented")` with `commands::sync::run(backfill);`
  - [x] Remove `#![allow(dead_code)]` from `src/sync.rs` (it is now called from `commands/sync.rs`)
  - [x] Verify `backfill: bool` is passed through correctly from `clap` (already declared in `main.rs` as `#[arg(long)] backfill: bool`)

- [x] Task 4: Implement std-only UTC date helper (AC: #1, #2)
  - [x] Add private `fn today_utc() -> String` to `src/commands/sync.rs`
  - [x] Use `std::time::SystemTime::now()` + `UNIX_EPOCH` to get seconds since epoch
  - [x] Apply civil-from-days algorithm to compute year/month/day (same algorithm already used in `checkpoint.rs::format_iso8601_utc` and `logger.rs::epoch_to_datetime` — do NOT add chrono or any date crate)
  - [x] Return `"YYYY-MM-DD"` ISO date string (no time component needed)
  - [x] See Dev Notes for the exact implementation to use

- [x] Task 5: Implement backfill date discovery (AC: #2, #3, #4)
  - [x] For backfill: call `jsonl_parser::parse_date_range("0000-00-00", &today)` — this returns all dates present in JSONL since the `start` bound is effectively the epoch floor
  - [x] Count the keys in the returned HashMap — that is the total date count
  - [x] Find the earliest key by sorting and taking `dates[0]` (or use `keys().min()`)
  - [x] Call `sync::run(&earliest_date, &today)` with the full range
  - [x] Print: `"vibestats: backfill complete — processed N date(s)"` where N = total dates found in JSONL

- [x] Task 6: Write co-located unit tests (AC: #1, #2, #4, #5)
  - [x] `#[cfg(test)]` module inside `src/commands/sync.rs`
  - [x] Test `today_utc()` returns a string matching `YYYY-MM-DD` pattern (length == 10, correct separators)
  - [x] Test `today_utc()` year is >= 2026 (sanity check against epoch bug)
  - [x] Run `cargo test` from repo root — must pass with 0 failures (76 tests pass)
  - [x] Run `cargo clippy --all-targets -- -D warnings` — must produce 0 warnings (0 warnings)
  - [x] Run `cargo build` — must produce 0 errors (clean build)

## Dev Notes

### Module Responsibility Summary

`commands/sync.rs` is the **CLI handler** — it translates user intent into `sync::run` calls:

| Module | Role in this story |
|---|---|
| `commands::sync::run(backfill)` | Entry point from `main.rs`; computes date range, calls `sync::run`, prints stdout summary |
| `crate::sync::run(start, end)` | Already implemented in Story 3.1 — the core orchestration; do NOT modify `sync.rs` |
| `crate::jsonl_parser::parse_date_range` | Used in backfill to discover all historical dates |
| `crate::logger::error/info` | For error logging (not stdout) |

**`sync.rs` is COMPLETE — do not modify its logic.** This story only adds the CLI wiring.

### `commands/sync.rs` Entry Point Signature

```rust
pub fn run(backfill: bool) {
    // computes date range, calls sync::run, prints stdout summary
    // NEVER calls std::process::exit — main.rs handles exit
}
```

### How `main.rs` calls this (already wired in main.rs, just needs the `println!` replaced)

```rust
Commands::Sync { backfill } => commands::sync::run(backfill),
```

The `backfill: bool` is already declared in `main.rs` with `#[arg(long)] backfill: bool`.

### `today_utc()` — Std-Only UTC Date Helper

Do NOT add `chrono`, `time`, or any date crate. Use the same civil-from-days algorithm already used in `src/checkpoint.rs::format_iso8601_utc` and `src/logger.rs::epoch_to_datetime`:

```rust
fn today_utc() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Days since Unix epoch
    let z = secs / 86400;

    // Civil-from-days: https://howardhinnant.github.io/date_algorithms.html
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

    format!("{:04}-{:02}-{:02}", y, mo, d)
}
```

**This is identical logic to `checkpoint.rs::format_iso8601_utc` minus the time component.** Copy and adapt — do not import across modules.

### Backfill Date Discovery

For `vibestats sync --backfill`, use the JSONL parser to discover all historical dates:

```rust
// "0000-00-00" is before any real ISO date — parse_date_range is inclusive and
// will return every date found in JSONL history.
let activities = jsonl_parser::parse_date_range("0000-00-00", &today);
if activities.is_empty() {
    println!("vibestats: backfill complete — no JSONL data found");
    return;
}
let mut dates: Vec<&String> = activities.keys().collect();
dates.sort();
let earliest = dates[0].clone();
let count = dates.len();
crate::sync::run(&earliest, &today);
println!("vibestats: backfill complete — processed {} date(s)", count);
```

`parse_date_range` returns an empty `HashMap` (not error) if `HOME` is not set or the JSONL directory is missing — handle this gracefully.

### Stdout Output Contract

| Scenario | stdout message |
|---|---|
| `vibestats sync` — any outcome | `"vibestats: sync complete"` |
| `vibestats sync --backfill` — data found | `"vibestats: backfill complete — processed N date(s)"` where N = JSONL date count |
| `vibestats sync --backfill` — no JSONL data | `"vibestats: backfill complete — no JSONL data found"` |

**Key constraint:** `sync::run` returns `()` — the caller has no visibility into how many dates changed vs. were skipped. For `vibestats sync`, a simple `"vibestats: sync complete"` is correct and honest. For `vibestats sync --backfill`, we DO have the JSONL date count from our `parse_date_range` discovery call (before calling `sync::run`), so we can report `N` there. Any errors are silently logged to `vibestats.log` by `sync::run`; the CLI does not need to re-surface them in stdout.

### Error Handling Contract

| Scenario | Behaviour |
|---|---|
| `Config::load_or_exit()` fails (called inside `sync::run`) | `sync::run` exits 0 with message — never reaches `commands/sync.rs` summary |
| Any GitHub API error | `sync::run` logs via `logger::error` and continues; `commands/sync.rs` still prints summary |
| `HOME` not set / no JSONL | `parse_date_range` returns empty map; commands/sync.rs handles gracefully |

**`commands/sync.rs` NEVER calls `std::process::exit`.** `main.rs` will implicitly exit 0 after the command returns.

### File Structure

```
src/
├── main.rs               ← MODIFY: add `mod commands;`, wire Sync arm
├── sync.rs               ← MODIFY: remove #![allow(dead_code)] only; no logic changes
├── checkpoint.rs         ← EXISTING — not touched
├── config.rs             ← EXISTING — not touched
├── github_api.rs         ← EXISTING — not touched
├── jsonl_parser.rs       ← EXISTING — not touched
├── logger.rs             ← EXISTING — not touched
└── commands/
    ├── mod.rs            ← NEW — `pub mod sync;` only
    └── sync.rs           ← NEW — this story's implementation
```

Do NOT create stub files for `status.rs`, `machines.rs`, `auth.rs`, `uninstall.rs` — these will be added in Epic 4 stories. The `main.rs` arms for `Commands::Status`, `Commands::Machines`, `Commands::Auth`, `Commands::Uninstall` already print `"not yet implemented"` — leave them unchanged.

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

Do NOT add `chrono`, `time`, or any date crate. The civil-from-days algorithm covers all date needs std-only.

### Removing `#![allow(dead_code)]` from `sync.rs`

`src/sync.rs` currently has `#![allow(dead_code)]` at the top (added in Story 3.1 because no callers existed yet). Once `commands/sync.rs` calls `crate::sync::run(...)`, this allow attribute is no longer needed. Remove it in this story to restore full dead_code linting.

**Check:** After removing, `cargo clippy --all-targets -- -D warnings` must still pass. If any private helper in `sync.rs` triggers dead_code, either add a targeted `#[allow(dead_code)]` on that specific item or remove the unused item.

### Worktree / Cargo Isolation

The worktree is nested inside the main repo. `Cargo.toml` at the repo root already has `[workspace]` set. Do NOT add another `[workspace]` section.

Run all verification from the **repo root** (not from inside the worktree directory):
```bash
cargo build
cargo test
cargo clippy --all-targets -- -D warnings
```

### Previous Story Learnings

From Story 3.1 (`sync.rs`):
- `sync::run` returns `()` — callers do not inspect return value for sync success/failure; all errors are logged via `logger::error`
- `Config::load_or_exit()` is called inside `sync::run` — if config is missing, `sync::run` exits 0 with message before any caller code continues
- `parse_date_range` returns an empty `HashMap` (not error) when HOME is unset or projects dir unreadable

From Story 2.5 (`github_api.rs`):
- `#![allow(clippy::result_large_err)]` may be needed if a `ureq::Error` is in a return type — not applicable here since commands/sync.rs doesn't use ureq directly
- `cargo clippy --all-targets -- -D warnings` (with `--all-targets`) catches all targets including test code
- PRs must include `Closes #21` in the PR description

From Story 2.3 (`checkpoint.rs`):
- `std::process::exit` must never be called inside modules — only by `main.rs`
- Use `std::env::temp_dir()` for test temp files (not `/tmp` directly)

From Story 2.4 (`jsonl_parser.rs`):
- `parse_date_range("0000-00-00", end)` is safe — the start bound comparison is string lexicographic; `"0000-00-00"` is less than any real date and will include all historical JSONL dates

### Architecture Constraints Summary

| Constraint | NFR/Source | Impact on This Story |
|---|---|---|
| Exit 0 on all errors | NFR10 | `commands/sync.rs` returns `()` — never calls `exit` |
| Silent sync errors during sessions | NFR11 | Errors logged via `logger::error` only; stdout summary is user-initiated (CLI, not hook) |
| Idempotent sync | NFR12 | Hash check is inside `sync::run` — backfill re-runs skip unchanged dates automatically |
| Backfill ≤ 60s for 12 months | NFR3 | No additional rate-limiting beyond `github_api.rs` backoff; ~365 files × ~200ms ≈ 73s worst case — acceptable since most dates are already synced (hash skip) |
| Single HTTP module | architecture.md | `commands/sync.rs` calls `sync::run` which calls `github_api.rs` — no direct HTTP |
| No async runtime | architecture.md | All code synchronous; no `tokio`, no `async fn` |
| No new crates | Story scope | std-only date helper, no chrono |
| snake_case filenames | architecture.md | Files: `src/commands/mod.rs`, `src/commands/sync.rs` |
| Co-located unit tests | architecture.md | `#[cfg(test)]` module inside `src/commands/sync.rs` |
| PR closes GH issue | epics.md | PR description must include `Closes #21` |

### Anti-Patterns to Prevent

- Do NOT modify `src/sync.rs` business logic — it is complete and tested
- Do NOT add throttle check in `commands/sync.rs` — `vibestats sync` is unthrottled by design (throttle is Stop hook only)
- Do NOT call `std::process::exit` in `commands/sync.rs`
- Do NOT add `chrono` or any date crate — use civil-from-days (std-only)
- Do NOT inline GitHub API calls — all HTTP must go through `github_api.rs`
- Do NOT add a second `[workspace]` to `Cargo.toml`
- Do NOT create stub files for commands 4.1–4.4 (unnecessary complexity)
- Do NOT use `unwrap()` or `expect()` in non-test code

### Project Structure Notes

- New files: `src/commands/mod.rs`, `src/commands/sync.rs`
- Modified files: `src/main.rs` (add `mod commands;`, replace `println!("not yet implemented")` in Sync arm), `src/sync.rs` (remove `#![allow(dead_code)]` only — no logic changes)
- No other files modified

### References

- Story requirements: [Source: _bmad-output/planning-artifacts/epics.md#Story 3.4]
- FR17 (manual sync): [Source: _bmad-output/planning-artifacts/epics.md#Functional Requirements]
- FR18 (manual backfill): [Source: _bmad-output/planning-artifacts/epics.md#Functional Requirements]
- NFR3 (backfill ≤ 60s): [Source: _bmad-output/planning-artifacts/epics.md#NonFunctional Requirements]
- NFR10 (exit 0): [Source: _bmad-output/planning-artifacts/epics.md#NonFunctional Requirements]
- NFR12 (idempotent sync): [Source: _bmad-output/planning-artifacts/epics.md#NonFunctional Requirements]
- Date range per operation: [Source: _bmad-output/planning-artifacts/architecture.md#API & Communication Patterns — Date range per operation]
- Sync operation spec: [Source: _bmad-output/planning-artifacts/architecture.md#API & Communication Patterns — Sync operation]
- Module file structure: [Source: _bmad-output/planning-artifacts/architecture.md#Complete Project Directory Structure]
- sync.rs public API: [Source: _bmad-output/implementation-artifacts/3-1-implement-core-sync-orchestration.md#sync::run Signature]
- civil-from-days algorithm: [Source: src/checkpoint.rs#format_iso8601_utc]
- GH Issue: #21

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

No issues encountered. Clean implementation following story spec exactly.

### Completion Notes List

- Created `src/commands/mod.rs` with `pub mod sync;` only (no stubs for 4.x commands).
- Created `src/commands/sync.rs` implementing `pub fn run(backfill: bool)`:
  - Uses std-only civil-from-days algorithm for `today_utc()` — no external date crates.
  - `backfill = false`: calls `crate::sync::run(&today, &today)`, prints `"vibestats: sync complete"`.
  - `backfill = true`: discovers dates via `jsonl_parser::parse_date_range("0000-00-00", &today)`, handles empty JSONL case gracefully, calls `crate::sync::run(&earliest, &today)`, prints count.
  - Never calls `std::process::exit` — `main.rs` handles exit (AC #5 / NFR10).
  - No throttle check — `vibestats sync` is unthrottled by design.
- Wired `Commands::Sync { backfill } => commands::sync::run(backfill)` in `main.rs`.
- Added `mod commands;` to `main.rs` module declarations.
- Removed `#![allow(dead_code)]` from `src/sync.rs` — now fully called.
- Added 2 unit tests for `today_utc()`: format validation and year >= 2026 sanity check.
- All 76 tests pass; 0 clippy warnings; clean build.

### File List

- `src/commands/mod.rs` (new)
- `src/commands/sync.rs` (new)
- `src/main.rs` (modified: added `mod commands;`, wired Sync arm)
- `src/sync.rs` (modified: removed `#![allow(dead_code)]`)

### Change Log

- 2026-04-11: Implemented `vibestats sync` and `vibestats sync --backfill` CLI commands (Story 3.4)
