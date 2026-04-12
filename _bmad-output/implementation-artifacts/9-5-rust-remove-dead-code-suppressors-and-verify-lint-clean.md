# Story 9.5: Rust — Remove dead_code suppressors and verify lint clean

Status: backlog

<!-- GH Issue: #85 | Epic: #80 | PR must include: Closes #85 -->

## Story

As a Rust developer maintaining the vibestats binary,
I want all `#![allow(dead_code)]` suppressors removed from the codebase now that all public APIs have callers,
So that clippy can detect genuinely unused code and the false safety of blanket suppression is eliminated.

## Background

`#![allow(dead_code)]` was added to each Rust module during incremental delivery (Epics 2–4) because callers didn't exist yet. This was the correct pattern at the time. By the end of Epic 4, every public method in every module is called from user-facing code. The retrospectives for Epics 2, 3, and 4 all tracked removal of these suppressors as action items — none were completed.

Affected modules (per retro analysis):
- `src/config.rs` — `#![allow(dead_code)]` added in Epic 2
- `src/logger.rs` — `#![allow(dead_code)]` added in Epic 2
- `src/checkpoint.rs` — `#![allow(dead_code)]` added in Epic 2
- `src/jsonl_parser.rs` — `#![allow(dead_code)]` added in Epic 2
- `src/github_api.rs` — `#![allow(dead_code)]` added in Epic 2
- `src/sync.rs` — `#![allow(dead_code)]` added in Epic 3

Source: Epic 2 retro Technical Debt #1, Epic 3 retro Technical Debt #1, Epic 4 retro Technical Debt #1.

## Acceptance Criteria

1. **Given** `#![allow(dead_code)]` exists at the top of any source file **When** this story is complete **Then** no `#![allow(dead_code)]` attribute exists anywhere in `src/`.

2. **Given** the suppressors are removed **When** `cargo clippy --all-targets -- -D warnings` is run from the repo root **Then** it exits with code 0 and reports 0 warnings.

3. **Given** clippy reports any dead code warning after suppressor removal **When** the warning is analyzed **Then** either: (a) the function is genuinely unused and is removed, or (b) the function is part of the public API and its suppressor is replaced with a targeted `#[allow(dead_code)]` on that specific item with a comment explaining why it's intentionally kept (e.g., future extensibility).

4. **Given** the changes are applied **When** `cargo test` is run **Then** all tests pass with no regressions.

## Tasks / Subtasks

- [ ] Task 1: Search for all dead_code suppressors
  - [ ] Run `grep -rn "allow(dead_code)" src/` to find every instance
  - [ ] List each file and line number

- [ ] Task 2: Remove all `#![allow(dead_code)]` module-level suppressors
  - [ ] Remove from `src/config.rs`
  - [ ] Remove from `src/logger.rs`
  - [ ] Remove from `src/checkpoint.rs`
  - [ ] Remove from `src/jsonl_parser.rs`
  - [ ] Remove from `src/github_api.rs`
  - [ ] Remove from `src/sync.rs`
  - [ ] Check for any other files in `src/` (e.g., `src/commands/*.rs`)

- [ ] Task 3: Run `cargo clippy --all-targets -- -D warnings` and address any warnings
  - [ ] For each dead_code warning:
    - If the function has no callers and is not part of the intended public API: remove the function and its tests
    - If the function is intentionally present (future use, part of a public interface): add `#[allow(dead_code)]` at the item level with a `// intentionally kept: <reason>` comment
  - [ ] Repeat until `cargo clippy --all-targets -- -D warnings` exits 0

- [ ] Task 4: Run `cargo test` and confirm all tests pass
  - [ ] `cargo test` from repo root must exit 0
  - [ ] Record the final test count in the Dev Agent Record

## Dev Notes

**Expected state after all callers wired (Epic 4 complete):**

Per the Epic 4 retrospective: "every public method in `github_api.rs` (`get_file_content`, `get_file_sha`, `put_file`, `get_user`, `delete_file`, `list_directory`), `config.rs`, and `checkpoint.rs` is now called from user-facing code." This means the suppressors should be entirely removable with no dead code warnings.

**If clippy reports dead code on a function:**
First check whether the function is called indirectly via a test path only (`#[cfg(test)]`). A function called only in tests is considered dead in release builds. That is acceptable — add `#[cfg_attr(not(test), allow(dead_code))]` or restructure as a test helper. Do NOT remove a function just because clippy flags it without confirming it has no callers.

**The `#[cfg(test)]` test modules themselves** may contain test-only functions. These are expected to have dead_code warnings suppressed by default (Rust doesn't warn on `#[cfg(test)]` items) — but double-check if any warnings appear.

**Arch constraint:** No new crates. No changes to public function signatures. Only suppressor removal and any resulting dead code removal.

## Review Criteria

- `grep -rn "allow(dead_code)" src/` returns no output (or only targeted item-level allows with comments)
- `cargo clippy --all-targets -- -D warnings` exits 0
- `cargo test` exits 0
- Test count after this story is equal to or greater than the count before (no tests accidentally removed)
