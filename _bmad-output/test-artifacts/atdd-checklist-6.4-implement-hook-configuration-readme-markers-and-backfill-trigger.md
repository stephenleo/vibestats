---
stepsCompleted: ['step-01-preflight-and-context', 'step-02-generation-mode', 'step-03-test-strategy', 'step-04-generate-tests', 'step-04c-aggregate', 'step-05-validate-and-complete']
lastStep: 'step-05-validate-and-complete'
lastSaved: '2026-04-12'
storyId: '6.4-implement-hook-configuration-readme-markers-and-backfill-trigger'
tddPhase: 'RED'
inputDocuments:
  - '_bmad-output/implementation-artifacts/6-4-implement-hook-configuration-readme-markers-and-backfill-trigger.md'
  - '_bmad-output/test-artifacts/test-design-epic-6.md'
  - '_bmad/tea/config.yaml'
  - 'install.sh'
  - 'tests/installer/test_6_3.bats'
---

# ATDD Checklist: Story 6.4 — Implement Hook Configuration, README Markers, and Backfill Trigger

**Date:** 2026-04-12
**TDD Phase:** RED (failing tests generated — feature not yet implemented)
**Stack:** backend (Bash shell — bats-core framework)
**Execution Mode:** sequential

---

## Step 1: Preflight & Context

### Stack Detection

- Detected stack: **backend**
  - Evidence: `Cargo.toml` present (Rust binary); `pyproject.toml`-based action pipeline; no `playwright.config.*`; no `package.json` with frontend framework dependencies
  - No browser-based E2E tests needed — all tests are shell unit tests using `bats-core`

### Prerequisites Satisfied

- [x] Story approved with clear acceptance criteria (3 ACs mapped to FR8, FR9, FR11)
- [x] Test framework configured: `bats-core` (via npm, pattern established in `test_6_1.bats`, `test_6_3.bats`)
- [x] Development environment available (Bash 3.2-compatible environment)
- [x] Pattern established by prior stories (6.1, 6.3) for `_gh()` stub pattern

### Story Context Loaded

**File:** `_bmad-output/implementation-artifacts/6-4-implement-hook-configuration-readme-markers-and-backfill-trigger.md`

**Acceptance Criteria:**

1. **AC #1 (FR8, R-008):** `configure_hooks()` writes `Stop` hook (`command: vibestats sync`, `async: true`) and `SessionStart` hook (`command: vibestats sync`) to `~/.claude/settings.json`. Idempotency required.

2. **AC #2 (FR9, R-009):** `inject_readme_markers()` adds `<!-- vibestats-start -->` and `<!-- vibestats-end -->` markers with SVG `<img>` embed and dashboard link. Graceful 404 handling. Idempotency required.

3. **AC #3 (FR11):** `run_backfill()` runs `vibestats sync --backfill` as the final step. Non-fatal on binary failure.

### Framework & Patterns

- Framework: `bats-core` v1.x (installed via npm)
- Test directory: `tests/installer/`
- Established patterns from `test_6_3.bats`:
  - `_gh()` shell function override (define-if-not-defined guard in install.sh)
  - `export HOME=$(mktemp -d)` in `setup()` for isolation
  - `teardown()` removes temp HOME
  - `GH_SPY_LOG` for capturing and asserting `_gh` call arguments
  - `python3` stdlib for JSON parsing assertions

---

## Step 2: Generation Mode

**Mode selected:** AI generation (backend stack — no browser recording needed)

**Rationale:** All acceptance criteria involve shell function behaviour (JSON file writes, GitHub API calls via `_gh()`, binary invocation) — testable via bats-core + mock overrides without browser automation.

---

## Step 3: Test Strategy

### Acceptance Criteria → Test Scenarios Mapping

| AC | Scenario | Priority | Test Level | Risk Link |
|----|----------|----------|------------|-----------|
| AC #1 (FR8) | `configure_hooks` writes Stop hook with `command=vibestats sync` and `async=true` | P1 | Shell unit | R-008 |
| AC #1 (FR8) | `configure_hooks` writes SessionStart hook with `command=vibestats sync` (no async) | P1 | Shell unit | R-008 |
| AC #1 (R-008) | `configure_hooks` is idempotent — running twice leaves exactly one Stop + one SessionStart | P1 | Shell unit | R-008 |
| AC #1 | `configure_hooks` does not clobber pre-existing unrelated hooks in settings.json | P1 | Shell unit | R-008 |
| AC #2 (FR9) | `inject_readme_markers` adds markers + SVG img URL + dashboard link in PUT body | P2 | Shell unit | R-009 |
| AC #2 (R-009) | `inject_readme_markers` prints warning (not error) + exits 0 on 404 profile repo | P2 | Shell unit | R-009 |
| AC #2 | `inject_readme_markers` is idempotent — no PUT when markers already present | P2 | Shell unit | R-009 |
| AC #3 (FR11) | `run_backfill` calls `vibestats sync --backfill` binary | P2 | Shell unit | - |
| AC #3 | `run_backfill` exits 0 and prints warning when binary exits non-zero | P2 | Shell unit | - |

### Test Levels Selected

**Unit/Shell** for all tests — appropriate for:
- Pure bash function behaviour
- File writes and JSON structure assertions
- Mock-based `_gh()` API call verification
- Binary spy call verification

**No E2E / browser tests** — backend-only stack; no UI involved.

### Priority Distribution

- P1: 4 tests (hook configuration — FR8/R-008 critical path)
- P2: 5 tests (README markers + backfill — FR9/FR11/R-009 edge cases)
- Total: 9 tests

### TDD Red Phase Requirements

All tests are designed to FAIL before implementation:
- `configure_hooks()` function does not exist in `install.sh` yet
- `inject_readme_markers()` function does not exist in `install.sh` yet
- `run_backfill()` function does not exist in `install.sh` yet
- All tests use `source '${INSTALL_SH}'` then call the functions directly — sourcing will succeed but function calls will fail with "command not found"

---

## Step 4: Generate Tests (Sequential)

### API Tests (Shell Unit Tests)

All tests written to: `tests/installer/test_6_4.bats`

**Tests generated:**

#### P1 Tests (configure_hooks)

1. `[P1] configure_hooks: Stop hook with command=vibestats sync and async=true written to settings.json`
   - Calls `configure_hooks`, asserts `~/.claude/settings.json` exists
   - Python3 assertion: `hooks.Stop[0].hooks[0].command == "vibestats sync"` and `hooks.Stop[0].hooks[0].async == true`
   - Expected to fail: function not implemented

2. `[P1] configure_hooks: SessionStart hook with command=vibestats sync written to settings.json`
   - Calls `configure_hooks`, asserts `~/.claude/settings.json` exists
   - Python3 assertion: `hooks.SessionStart[0].hooks[0].command == "vibestats sync"` and no `async=true`
   - Expected to fail: function not implemented

3. `[P1] configure_hooks: idempotent — running twice produces exactly one Stop and one SessionStart entry`
   - Calls `configure_hooks` twice, asserts exactly 1 Stop matcher and 1 SessionStart matcher
   - Python3 assertion: `len(hooks.Stop) == 1` and `len(hooks.SessionStart) == 1`
   - Expected to fail: function not implemented (R-008 mitigation)

4. `[P1] configure_hooks: does not clobber existing unrelated hooks in settings.json`
   - Pre-seeds `settings.json` with `PreToolUse` hook
   - Calls `configure_hooks`, asserts PreToolUse hook still present + vibestats hooks added
   - Expected to fail: function not implemented

#### P2 Tests (inject_readme_markers + run_backfill)

5. `[P2] inject_readme_markers: markers + SVG img + dashboard link written to profile README`
   - Mocks `_gh api /user` → `{"login": "testuser"}`
   - Mocks `_gh api repos/.../README.md` GET → base64-encoded sample README + SHA
   - Mocks `_gh api repos/.../README.md --method PUT` → captures content field
   - Asserts PUT body contains: `<!-- vibestats-start -->`, `<!-- vibestats-end -->`, SVG URL with testuser, dashboard URL with testuser
   - Expected to fail: function not implemented (R-009 mitigation)

6. `[P2] inject_readme_markers: warning (not error) and continues when profile repo returns 404`
   - Mocks `_gh api repos/.../README.md` GET → returns non-zero exit
   - Asserts: `status == 0`, output contains `"Warning:"`
   - Expected to fail: function not implemented (R-009 mitigation)

7. `[P2] inject_readme_markers: idempotent — no second PUT when markers already present`
   - Mocks `_gh api repos/.../README.md` GET → README already containing markers
   - Asserts: `_gh api --method PUT` NOT called, output contains `"already present"`
   - Expected to fail: function not implemented

8. `[P2] run_backfill: vibestats sync --backfill is called as final step`
   - Creates mock binary at `${HOME}/.local/bin/vibestats` that logs calls to spy log
   - Asserts spy log contains `"sync --backfill"`
   - Expected to fail: function not implemented

9. `[P2] run_backfill: non-zero exit from binary prints warning but installer exits 0`
   - Creates mock binary that exits 1
   - Asserts: `status == 0`, output contains `"Warning:"`
   - Expected to fail: function not implemented

### E2E Tests

Not applicable — backend-only stack. No browser-based E2E tests generated.

---

## Step 4C: Aggregate Results

### TDD Red Phase Verification

All 9 tests:
- [x] Written in `test_6_4.bats` with proper bats-core `@test` syntax
- [x] Will FAIL when run — functions (`configure_hooks`, `inject_readme_markers`, `run_backfill`) not yet implemented in `install.sh`
- [x] Assert EXPECTED behaviour (not placeholder assertions)
- [x] Follow established patterns from `test_6_3.bats` (temp HOME, `_gh()` override, python3 JSON assertions)

### Files Written to Disk

- `tests/installer/test_6_4.bats` ✅

### Fixture Needs

- None beyond the existing bats `setup()`/`teardown()` temp HOME pattern
- `_gh()` overrides defined inline per test (established pattern)
- Mock vibestats binary created inline in `run_backfill` tests

---

## Step 5: Validate & Complete

### Validation Checklist

- [x] Prerequisites satisfied (bats-core installed, story has clear ACs)
- [x] All 9 test scenarios from AC mapping are covered
- [x] Tests follow Bash 3.2-compatible patterns (no `declare -A`, no `mapfile`)
- [x] All tests use temp `$HOME` isolation
- [x] All `gh` interactions go through `_gh()` helper (mockable)
- [x] Tests assert EXPECTED behaviour matching story acceptance criteria
- [x] Tests will FAIL until `configure_hooks`, `inject_readme_markers`, and `run_backfill` are implemented in `install.sh`
- [x] No orphaned browser sessions (N/A — no browser tests)
- [x] Temp artifacts in `${BATS_TMPDIR}` (not random locations)

### Completion Summary

**Test file created:** `tests/installer/test_6_4.bats`

**Test breakdown:**
- P1 tests: 4 (configure_hooks — Stop hook, SessionStart hook, idempotency, no-clobber)
- P2 tests: 5 (inject_readme_markers — 3 scenarios; run_backfill — 2 scenarios)
- Total: 9 failing tests (TDD RED phase)

**Key assumptions:**
- `install.sh` will define `configure_hooks()`, `inject_readme_markers()`, and `run_backfill()` as standalone functions (sourceable by bats)
- `_gh()` define-if-not-defined guard in `install.sh` allows test overrides to take precedence
- `BASH_SOURCE` guard on `main()` allows safe sourcing in tests
- Python3 is available in CI (established dependency from stories 6.1–6.3)

**Next steps (TDD Green Phase):**

1. Implement `configure_hooks()` in `install.sh`
2. Implement `inject_readme_markers()` in `install.sh`
3. Implement `run_backfill()` in `install.sh`
4. Wire into `main()` as the shared final steps (Steps 12–14)
5. Run `bats tests/installer/test_6_4.bats` → verify all 9 tests PASS
6. Run regression: `bats tests/installer/test_6_3.bats tests/installer/test_6_2.bats tests/installer/test_6_1.bats`
7. Commit passing tests

**Risk mitigations verified by these tests:**

- R-008 (duplicate hook entries): P1 idempotency test + no-clobber test
- R-009 (README 404 silent failure): P2 warning-not-error test + idempotency test

---

**Generated by:** BMad TEA Agent — ATDD Module
**Workflow:** `bmad-testarch-atdd`
**Story:** 6.4-implement-hook-configuration-readme-markers-and-backfill-trigger
