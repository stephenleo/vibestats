---
stepsCompleted: ['step-01-preflight-and-context', 'step-02-generation-mode', 'step-03-test-strategy', 'step-04-generate-tests', 'step-04c-aggregate', 'step-05-validate-and-complete']
lastStep: 'step-05-validate-and-complete'
lastSaved: '2026-04-13'
workflowType: 'testarch-atdd'
inputDocuments:
  - '_bmad-output/implementation-artifacts/9-5-rust-remove-dead-code-suppressors-and-verify-lint-clean.md'
  - '_bmad-output/test-artifacts/test-design-epic-9.md'
  - '_bmad/tea/config.yaml'
  - 'src/config.rs'
  - 'src/logger.rs'
  - 'src/checkpoint.rs'
  - 'src/jsonl_parser.rs'
  - 'src/github_api.rs'
  - 'src/hooks/mod.rs'
---

# ATDD Checklist — Epic 9, Story 9.5: Rust Remove dead_code Suppressors and Verify Lint Clean

**Date:** 2026-04-13
**Author:** Leo
**Primary Test Level:** Shell (bats-core) — integration tests invoking cargo toolchain
**Stack:** Backend (Rust/Cargo) — pure backend, no frontend
**TDD Phase:** RED — tests assert expected post-implementation behaviour; 8 of 10 tests fail before implementation

---

## Story Summary

As a Rust developer maintaining the vibestats binary, I want all `#![allow(dead_code)]` suppressors removed from the codebase now that all public APIs have callers, so that clippy can detect genuinely unused code and the false safety of blanket suppression is eliminated.

---

## Acceptance Criteria

1. No `#![allow(dead_code)]` attribute exists anywhere in `src/` after this story is complete.
2. `cargo clippy --all-targets -- -D warnings` exits 0 with 0 warnings.
3. Any residual dead_code warning is resolved per the decision tree (remove item, add targeted `#[allow(dead_code)]` with comment, or use `cfg_attr`) — no module-level blanket suppressor re-added.
4. `cargo test` exits 0 with no regressions.

---

## Stack Detection Result

- **Detected stack:** `backend`
- **Indicators:** `Cargo.toml` present; no `package.json`, no frontend framework files
- **Generation mode:** AI generation (backend — no browser recording needed)
- **Test framework:** bats-core (shell) calling cargo; Rust built-in `#[cfg(test)]` modules not modified
- **Playwright Utils:** not applicable (backend stack)

---

## Test Strategy

### AC Mapping

| AC | Scenario | Test Level | Priority | Test ID |
|----|----------|-----------|----------|---------|
| AC #1 | No module-level `#![allow(dead_code)]` in any `src/` file | Shell (grep) | P0 | 9.5-SHELL-001 |
| AC #1 | `src/config.rs` has no `#![allow(dead_code)]` | Shell (grep) | P0 | 9.5-SHELL-002 |
| AC #1 | `src/logger.rs` has no `#![allow(dead_code)]` | Shell (grep) | P0 | 9.5-SHELL-003 |
| AC #1 | `src/checkpoint.rs` has no `#![allow(dead_code)]` | Shell (grep) | P0 | 9.5-SHELL-004 |
| AC #1 | `src/jsonl_parser.rs` has no `#![allow(dead_code)]` | Shell (grep) | P0 | 9.5-SHELL-005 |
| AC #1 | `src/github_api.rs` has no `#![allow(dead_code)]` | Shell (grep) | P0 | 9.5-SHELL-006 |
| AC #1 | `src/github_api.rs` retains `#![allow(clippy::result_large_err)]` | Shell (grep) | P1 | 9.5-SHELL-007 |
| AC #1 | `src/hooks/mod.rs` has no `#![allow(dead_code)]` | Shell (grep) | P0 | 9.5-SHELL-008 |
| AC #3 | Any item-level `#[allow(dead_code)]` is commented | Shell (grep+awk) | P1 | 9.5-SHELL-009 |
| AC #2 | `cargo clippy --all-targets -- -D warnings` exits 0 | Cargo (integration) | P0 | 9.5-SHELL-010 |
| AC #4 | `cargo test` exits 0 with no failures | Cargo (integration) | P0 | 9.5-SHELL-011 |

### Test Level Rationale

- **Shell tests (bats-core):** Appropriate for lint/code-quality acceptance criteria. Grep assertions verify structural constraints (no suppressor attributes). Cargo invocations verify the build toolchain outcomes (AC #2, AC #4).
- **No E2E/API/UI tests:** This story is a pure Rust source refactor — no web endpoints or user interface involved.
- **Existing Rust `#[cfg(test)]` modules:** Not modified by this story. The story only removes module-level attributes; it does not change test logic.

### Red Phase Rationale

At story creation time, all six module-level suppressors are confirmed present:

| File | Line | Suppressor |
|------|------|-----------|
| `src/config.rs` | 1 | `#![allow(dead_code)]` |
| `src/logger.rs` | 16 | `#![allow(dead_code)]` |
| `src/checkpoint.rs` | 1 | `#![allow(dead_code)]` |
| `src/jsonl_parser.rs` | 1 | `#![allow(dead_code)]` |
| `src/github_api.rs` | 22 | `#![allow(dead_code)]` |
| `src/hooks/mod.rs` | 1 | `#![allow(dead_code)]` |

Tests 9.5-SHELL-002 through 9.5-SHELL-008 will **FAIL** (grep finds matches) until suppressors are removed.
Test 9.5-SHELL-001 will **FAIL** (grep exits 0 = found matches) until all suppressors are removed.
Test 9.5-SHELL-010 (clippy) will **FAIL** only if removing suppressors exposes real dead_code warnings — likely FAIL on first attempt.
Test 9.5-SHELL-011 (cargo test) is likely **GREEN** already but must remain green after code deletion.
Test 9.5-SHELL-007 (`result_large_err` retained) is **GREEN** already — validates no over-removal.

---

## Generated Test Files

### `tests/rust/test_9_5.bats`

**Description:** ATDD acceptance tests for Story 9.5. Shell tests using bats-core that verify:
- No module-level `#![allow(dead_code)]` suppressors remain in any `src/` file (AC #1)
- The critical `#![allow(clippy::result_large_err)]` suppressor in `github_api.rs` is preserved (AC #1 negative guard)
- Any residual item-level `#[allow(dead_code)]` attributes are accompanied by explanatory comments (AC #3)
- `cargo clippy --all-targets -- -D warnings` exits 0 (AC #2)
- `cargo test` exits 0 with no regressions (AC #4)

**TDD Phase:** RED — 8 tests fail before implementation, 2–3 tests currently green (green tests are guards against over-removal)

**Run command:**
```bash
bats tests/rust/test_9_5.bats
```

**Priority coverage:**
- P0: 8 tests (core correctness — no suppressor, clippy clean, tests pass)
- P1: 3 tests (guard tests — result_large_err retained, item-level allows commented)

---

## Checklist Validation

- [x] Story approved with clear acceptance criteria
- [x] Backend stack detected (`Cargo.toml` present, no frontend indicators)
- [x] AI generation mode selected (backend stack — no browser recording)
- [x] All 4 acceptance criteria mapped to test scenarios
- [x] Tests designed to FAIL before implementation (TDD red phase)
- [x] Tests use realistic assertions (grep for actual attribute text, cargo exit codes)
- [x] No placeholder assertions (`[ "$status" -eq 0 ]` is real, not `[ "1" = "1" ]`)
- [x] Test file located in `tests/rust/` (appropriate subdirectory for Rust integration tests)
- [x] Negative guard test included (AC #1 — `result_large_err` must NOT be removed)
- [x] AC #3 decision-tree compliance verified by test (item-level allows require comments)
- [x] No new crates added (pure grep/cargo assertions)
- [x] CLI sessions / temp artifacts: N/A (no browser automation, no temp files created)

---

## Completion Summary

**Test files created:**
- `tests/rust/test_9_5.bats` — 11 tests (8 P0, 3 P1)

**Checklist output path:**
- `_bmad-output/test-artifacts/atdd-checklist-9.5-rust-remove-dead-code-suppressors-and-verify-lint-clean.md`

**Key risks / assumptions:**
1. `cargo` must be on PATH when bats tests run (CI should ensure this)
2. Some tests may be slow (cargo clippy, cargo test) — expected for Rust integration tests
3. The `cargo test` test (AC #4) is likely already green — it becomes the regression guard after code deletion

**Next recommended workflow:** Implementation (dev story 9.5) — run `bats tests/rust/test_9_5.bats` after each change to track progress from RED to GREEN.
