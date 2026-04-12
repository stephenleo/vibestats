---
stepsCompleted: ['step-01-preflight-and-context', 'step-02-generation-mode', 'step-03-test-strategy', 'step-04-generate-tests', 'step-04c-aggregate', 'step-05-validate-and-complete']
lastStep: 'step-05-validate-and-complete'
lastSaved: '2026-04-12'
storyId: '6.1-implement-dependency-detection-and-gh-authentication'
tddPhase: 'RED'
inputDocuments:
  - '_bmad-output/implementation-artifacts/6-1-implement-dependency-detection-and-gh-authentication.md'
  - '_bmad-output/test-artifacts/test-design-epic-6.md'
  - '_bmad/tea/config.yaml'
  - 'install.sh'
---

# ATDD Checklist: Story 6.1 — Implement Dependency Detection and gh Authentication

**Date:** 2026-04-12
**TDD Phase:** RED (failing tests generated — feature not yet implemented)
**Stack:** backend (Bash shell — bats-core framework)
**Execution Mode:** sequential

---

## Step 1: Preflight & Context

### Stack Detection

- Detected stack: **backend**
  - Evidence: `Cargo.toml` present (Rust binary); no `package.json` with frontend deps; no `playwright.config.*`
  - No browser-based E2E tests needed
  - Test framework: `bats-core` (mandated by story Dev Notes and test-design-epic-6.md)

### Prerequisites

- [x] Story approved with clear acceptance criteria (4 ACs defined)
- [x] Test framework: `bats-core` specified in story Dev Notes
- [x] Development environment available
- [x] `install.sh` exists (stub at repo root — functions not yet implemented)

### Loaded Context

- Story: `_bmad-output/implementation-artifacts/6-1-implement-dependency-detection-and-gh-authentication.md`
- Epic test design: `_bmad-output/test-artifacts/test-design-epic-6.md`
- TEA config: `_bmad/tea/config.yaml`
- TEA config flags: `tea_use_playwright_utils: true` (not applicable — backend stack), `tea_use_pactjs_utils: false`, `tea_execution_mode: auto → sequential`

---

## Step 2: Generation Mode

**Mode selected:** AI generation
**Reason:** Backend stack with clear acceptance criteria — no browser recording needed. ACs describe deterministic shell function behaviors that are directly testable via mocked function overrides.

---

## Step 3: Test Strategy

### Acceptance Criteria → Test Scenario Mapping

| AC | Scenario | Test Level | Priority | Risk Link |
|----|----------|------------|----------|-----------|
| AC #1 | `gh` not installed → `brew install gh` on Darwin | Unit/Shell | P1 | R-006 |
| AC #1 | `gh` not installed → `apt-get install gh` on Linux | Unit/Shell | P1 | R-006 |
| AC #2 | `gh` version < 2.0 → exits non-zero with version in message | Unit/Shell | P1 | R-006 |
| AC #2 | `gh` version < 2.0 → error message includes "2.0" minimum | Unit/Shell | P1 | R-006 |
| AC #3 | `gh` not authenticated → `gh auth login` called | Unit/Shell | P1 | - |
| AC #4 | Darwin arm64 → target `aarch64-apple-darwin` | Unit/Shell | P1 | R-007 |
| AC #4 | Darwin x86_64 → target `x86_64-apple-darwin` | Unit/Shell | P1 | R-007 |
| AC #4 | Linux x86_64 → target `x86_64-unknown-linux-gnu` | Unit/Shell | P1 | R-007 |
| AC #4 | Unsupported platform → exits non-zero with message | Unit/Shell | P1 | R-007 |
| AC #1 (idempotency) | `gh` installed ≥ 2.0 → no install attempted | Unit/Shell | P2 | R-006 |

**Total tests:** 10 (9 P1 + 1 P2)

### Test Level Selection

- **Unit/Shell (bats-core):** All tests
  - Backend-only project; all installer logic is pure shell functions
  - No E2E or API tests needed (no web UI, no REST endpoints)
  - Mock strategy: override `_gh()` helper function + stub `uname`, `brew`, `apt-get` via exported bash functions

### TDD Red Phase Requirements

- All tests use `@test` with assertions against **expected behavior** (not placeholder `true` assertions)
- Tests will FAIL because `install.sh` functions (`install_gh_if_missing`, `check_gh_version`, `check_gh_auth`, `detect_platform`) are not yet implemented — the stub exits 1 immediately
- `test.skip()` equivalent in bats-core: tests are marked as RED-phase via comments; they run and fail (bats does not have a skip equivalent that prevents execution — tests will report FAIL which is correct for TDD red phase)

---

## Step 4: Generate Tests (Sequential)

### Subagent A: Shell/Unit Failing Tests

Tests generated in: `tests/installer/test_6_1.bats`

```
Tests generated (RED PHASE):
  [P1] gh not installed → brew install gh called on Darwin
  [P1] gh not installed → apt-get install gh called on Linux
  [P1] gh version < 2.0 → exits non-zero with error message
  [P1] gh version < 2.0 → error message includes minimum version 2.0
  [P1] gh not authenticated → gh auth login called
  [P1] platform Darwin arm64 → target is aarch64-apple-darwin
  [P1] platform Darwin x86_64 → target is x86_64-apple-darwin
  [P1] platform Linux x86_64 → target is x86_64-unknown-linux-gnu
  [P1] unsupported platform → exits non-zero with message
  [P2] gh installed and version >= 2.0 → no install attempted
```

### Subagent B: E2E Tests

**Skipped** — backend stack only. No browser-based E2E tests applicable for a shell installer.

---

## Step 4C: Aggregate

### TDD Red Phase Validation

- [x] All tests assert EXPECTED behavior (not placeholder assertions)
- [x] Tests call specific functions (`install_gh_if_missing`, `check_gh_version`, `check_gh_auth`, `detect_platform`) that will not exist until implementation
- [x] Tests will FAIL on current stub `install.sh` (exits 1 immediately)
- [x] No passing tests generated

### Files Written

| File | Type | Tests | Priority |
|------|------|-------|----------|
| `tests/installer/test_6_1.bats` | bats-core shell unit | 10 | P1×9, P2×1 |

### Fixture Needs

- `setup()` / `teardown()` in each test: temp `$HOME` directory (implemented inline in test file)
- Stub env scripts: created inline per test via heredoc into `$HOME/stub_env.sh`
- No shared fixture files needed for this story

---

## Step 5: Validate & Complete

### Validation Against Checklist

- [x] Prerequisites satisfied (story has clear ACs, bats-core specified, install.sh exists)
- [x] Test file created: `tests/installer/test_6_1.bats`
- [x] Checklist matches all 4 acceptance criteria
- [x] Tests are designed to fail before implementation
- [x] No orphaned browsers (backend only)
- [x] Temp artifacts stored in `$HOME` (test-scoped temp dir) — correct isolation

### Coverage Summary

| AC | Tests Covering | Priority |
|----|----------------|----------|
| AC #1 (gh install on Darwin) | 2 | P1, P2 |
| AC #1 (gh install on Linux) | 1 | P1 |
| AC #2 (version check) | 2 | P1 |
| AC #3 (auth check) | 1 | P1 |
| AC #4 (platform detection) | 4 | P1 |

All 4 ACs covered. Story PR gate (Task 6 from story) requires all P1 tests to pass.

### Next Steps (TDD Green Phase)

After implementing `install.sh` (Story 6.1 Tasks 1–5):

1. Run tests: `bats tests/installer/test_6_1.bats`
2. Verify all 10 tests PASS (green phase)
3. If any tests fail:
   - Either fix implementation (feature bug)
   - Or fix test (test misalignment with spec)
4. Ensure `bash -n install.sh` syntax check passes
5. Commit passing tests with implementation

### Implementation Guidance

Functions to implement in `install.sh` (in execution order):

1. `install_gh_if_missing()` — detects `gh` via `command -v gh`; installs via `brew` (Darwin) or `apt-get` (Linux)
2. `check_gh_version()` — extracts version via `_gh --version | head -1 | awk '{print $3}'`; major version < 2 → exit 1
3. `check_gh_auth()` — checks `_gh auth status`; calls `_gh auth login` if not authenticated
4. `detect_platform()` — sets `TARGET` var via `uname -s`/`uname -m` case statement
5. `_gh()` helper wrapper — wrap ALL `gh` calls through `_gh()` for testability
6. `main()` — calls each step function in sequence

---

## Run Instructions

```bash
# Install bats-core (if not already installed)
npm install --save-dev bats
# or: git submodule add https://github.com/bats-core/bats-core.git tests/bats

# Run Story 6.1 tests
bats tests/installer/test_6_1.bats

# Expected output (RED PHASE — before implementation):
# ✗ [P1] gh not installed → brew install gh called on Darwin
# ✗ [P1] gh not installed → apt-get install gh called on Linux
# ... (all 10 tests will FAIL — this is correct for TDD red phase)

# After implementation (GREEN PHASE):
# ✓ [P1] gh not installed → brew install gh called on Darwin
# ✓ ...
# 10 tests, 0 failures
```

---

**Generated by:** BMad TEA ATDD Agent
**Workflow:** `bmad-testarch-atdd`
**Story:** 6.1-implement-dependency-detection-and-gh-authentication
