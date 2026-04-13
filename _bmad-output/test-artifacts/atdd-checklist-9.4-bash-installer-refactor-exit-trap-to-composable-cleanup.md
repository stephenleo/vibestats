---
stepsCompleted: ['step-01-preflight-and-context', 'step-02-generation-mode', 'step-03-test-strategy', 'step-04-generate-tests', 'step-04c-aggregate', 'step-05-validate-and-complete']
lastStep: 'step-05-validate-and-complete'
lastSaved: '2026-04-13'
workflowType: 'testarch-atdd'
inputDocuments:
  - '_bmad-output/implementation-artifacts/9-4-bash-installer-refactor-exit-trap-to-composable-cleanup.md'
  - 'install.sh'
  - 'tests/installer/test_6_1.bats'
  - 'tests/installer/test_6_4.bats'
  - '_bmad/tea/config.yaml'
---

# ATDD Checklist - Epic 9, Story 9.4: Bash installer — Refactor EXIT trap to composable cleanup

**Date:** 2026-04-13
**Author:** Leo
**Primary Test Level:** Shell (bats-core)
**Stack:** Backend (installer shell scripts)
**TDD Phase:** RED — All 10 tests in `tests/installer/test_9_4.bats` currently fail

---

## Story Summary

As a developer maintaining `install.sh`,
I want the EXIT trap to use a composable cleanup function rather than a single override,
So that any future addition to `install.sh` can register cleanup tasks without silently discarding earlier ones.

---

## Acceptance Criteria

1. **Given** `install.sh` currently overwrites the EXIT trap in `download_and_install_binary()` **When** this story is complete **Then** the EXIT trap is replaced with a named `cleanup()` function that accumulates cleanup tasks and is registered once.

2. **Given** the `cleanup()` function is registered via `trap cleanup EXIT` **When** `install.sh` terminates (normally or via `set -e` error) **Then** all accumulated cleanup tasks execute (e.g., temp directory removal).

3. **Given** the refactored cleanup mechanism **When** `bats tests/installer/test_6_1.bats tests/installer/test_6_2.bats tests/installer/test_6_3.bats tests/installer/test_6_4.bats` is run **Then** all tests still pass with 0 failures.

4. **Given** a future developer needs to add a new cleanup action **When** they follow the established pattern **Then** they can add to the `cleanup()` function body without risk of dropping existing cleanup tasks.

---

## Stack Detection

- **Detected stack:** `backend`
- **Project type:** Rust binary with Bash installer (`install.sh`)
- **Test framework:** bats-core (existing pattern in `tests/installer/`)
- **No frontend indicators:** No `package.json`, no `playwright.config.*`
- **Generation mode:** AI generation (backend, clear ACs, no browser interaction needed)

---

## Test Strategy

### Mapping Acceptance Criteria → Test Scenarios

| AC | Test ID | Level | Priority | Description |
|----|---------|-------|----------|-------------|
| AC #1 | 9.4-UNIT-001 | Unit (bats) | P1 | `cleanup()` function defined in `install.sh` |
| AC #1 | 9.4-UNIT-002 | Unit (bats) | P1 | `download_and_install_binary()` assigns `_CLEANUP_TMPDIR` |
| AC #1 | 9.4-UNIT-003 | Unit (bats) | P1 | `trap cleanup EXIT` registered at top level |
| AC #1 | 9.4-UNIT-004 | Unit (bats) | P1 | `_CLEANUP_TMPDIR` global variable defined |
| AC #2 | 9.4-UNIT-005 | Unit (bats) | P0 | `cleanup()` removes directory when `_CLEANUP_TMPDIR` is set |
| AC #2 | 9.4-UNIT-006 | Unit (bats) | P1 | `cleanup()` no-op when `_CLEANUP_TMPDIR` empty |
| AC #2 | 9.4-INT-001 | Integration (bats) | P0 | EXIT trap fires on `set -e` error path |
| AC #1/#2 | 9.4-INT-002 | Integration (bats) | P1 | No inline `trap … EXIT` in function bodies |
| AC #4 | 9.4-UNIT-007 | Unit (bats) | P2 | `cleanup()` Bash 3.2 compatible syntax |
| AC #1/#4 | 9.4-UNIT-008 | Unit (bats) | P2 | `trap cleanup EXIT` appears exactly once |

**AC #3 (regression guard):** Not tested in this file. Verified by running the existing suite:
```bash
bats tests/installer/test_6_1.bats tests/installer/test_6_2.bats \
     tests/installer/test_6_3.bats tests/installer/test_6_4.bats
```

### Test Level Rationale (Backend — No E2E)

This is a **backend-only** project (shell installer). The test pyramid is:
- **Unit (bats)**: Test individual functions (`cleanup()`, `_CLEANUP_TMPDIR` presence, `trap cleanup EXIT`)
- **Integration (bats)**: Test function interactions (EXIT trap fires on error; no inline trap in function body)
- **No E2E**: Pure bash; no browser, no API, no UI involved

---

## Generated Test File (RED Phase)

**File:** `tests/installer/test_9_4.bats`

All 10 tests use direct assertions (no `test.skip()` — bats convention is that tests fail naturally when assertions fail, not via `skip`).

### Test List (all RED — failing before implementation)

| # | Test ID | Priority | Assertion | Why RED |
|---|---------|----------|-----------|---------|
| 1 | 9.4-UNIT-001 | P1 | `cleanup()` defined via `declare -f` | `cleanup()` does not exist yet |
| 2 | 9.4-UNIT-002 | P1 | `download_and_install_binary()` contains `_CLEANUP_TMPDIR` | No `_CLEANUP_TMPDIR` in function yet |
| 3 | 9.4-UNIT-003 | P1 | `grep "trap cleanup EXIT"` succeeds | No `trap cleanup EXIT` in file |
| 4 | 9.4-UNIT-004 | P1 | `grep "_CLEANUP_TMPDIR"` succeeds | No `_CLEANUP_TMPDIR` in file |
| 5 | 9.4-UNIT-005 | P0 | `cleanup()` removes real directory | `cleanup()` does not exist |
| 6 | 9.4-UNIT-006 | P1 | `cleanup()` exits cleanly when empty | `cleanup()` does not exist |
| 7 | 9.4-INT-001 | P0 | Temp dir removed after `set -e` abort | No EXIT trap fires |
| 8 | 9.4-INT-002 | P1 | Inline `trap '...' EXIT` count = 0 | Inline trap IS present (count = 1) |
| 9 | 9.4-UNIT-007 | P2 | No `declare -A` / `mapfile` in `cleanup()` | `cleanup()` does not exist |
| 10 | 9.4-UNIT-008 | P2 | `trap cleanup EXIT` count = 1 | Not present (count = 0) |

### TDD Red Phase Evidence

```
Command: bats tests/installer/test_9_4.bats

1..10
not ok 1 [P1][9.4-UNIT-001] install.sh defines a top-level cleanup() function
not ok 2 [P1][9.4-UNIT-002] download_and_install_binary() assigns _CLEANUP_TMPDIR (composable cleanup pattern)
not ok 3 [P1][9.4-UNIT-003] install.sh registers trap cleanup EXIT at top level
not ok 4 [P1][9.4-UNIT-004] install.sh defines _CLEANUP_TMPDIR global variable
not ok 5 [P0][9.4-UNIT-005] cleanup() removes _CLEANUP_TMPDIR directory when set
not ok 6 [P1][9.4-UNIT-006] cleanup() is a no-op when _CLEANUP_TMPDIR is empty or unset
not ok 7 [P0][9.4-INT-001] cleanup() fires and removes temp dir when script exits via set -e error
not ok 8 [P1][9.4-INT-002] install.sh source contains no inline trap … EXIT calls inside function bodies
not ok 9 [P2][9.4-UNIT-007] cleanup() uses only Bash 3.2 compatible syntax
not ok 10 [P2][9.4-UNIT-008] trap cleanup EXIT appears exactly once in install.sh (composable, not duplicated)

Summary:
- Total tests: 10
- Passing: 0 (expected in RED phase)
- Failing: 10 (expected)
- Status: RED phase confirmed ✅
```

---

## Mock Requirements

No `_gh()` stubs needed — this story tests structural properties of `install.sh` (function existence, variable assignment, trap registration). The tests do not invoke functions that make network calls.

**Mocking pattern used:**
- Source `install.sh` to load function definitions
- Call `declare -f <function>` to inspect function bodies
- Use `grep` to check for patterns in `install.sh` source
- Create real temp directories (via `mktemp -d`) for cleanup behavior tests

---

## Implementation Checklist (What Makes These Tests GREEN)

The DEV agent must implement these tasks in `install.sh` only (no test file changes needed):

### Task 1: Add `_CLEANUP_TMPDIR` global variable initialization

```bash
# Near the top of install.sh, after set -euo pipefail:
_CLEANUP_TMPDIR=""
```

### Task 2: Add `cleanup()` function

```bash
cleanup() {
  [ -n "$_CLEANUP_TMPDIR" ] && rm -rf "$_CLEANUP_TMPDIR"
}
```

**Bash 3.2 requirements:**
- No `declare -A`
- No `mapfile`/`readarray`
- No `+=` for array append
- Simple string variable — compliant ✅

### Task 3: Register `trap cleanup EXIT` once, early in `install.sh`

```bash
trap cleanup EXIT
```

Place this after the `_CLEANUP_TMPDIR=""` initialization and `cleanup()` function definition, before any function that could previously have set a trap.

### Task 4: Remove inline trap from `download_and_install_binary()`

Remove line 137:
```bash
trap 'rm -rf "$TMPDIR_WORK"' EXIT   # DELETE THIS LINE
```

Replace with:
```bash
_CLEANUP_TMPDIR="$TMPDIR_WORK"
```

### Task 5: Run full regression suite

```bash
bats tests/installer/test_6_1.bats tests/installer/test_6_2.bats \
     tests/installer/test_6_3.bats tests/installer/test_6_4.bats tests/installer/test_9_4.bats
```

All must pass with 0 failures (AC #3).

---

## Running Tests

```bash
# RED phase verification (before implementation — confirm all 10 fail)
bats tests/installer/test_9_4.bats

# GREEN phase verification (after implementation)
bats tests/installer/test_9_4.bats

# Full regression guard (AC #3)
bats tests/installer/test_6_1.bats tests/installer/test_6_2.bats \
     tests/installer/test_6_3.bats tests/installer/test_6_4.bats tests/installer/test_9_4.bats
```

---

## Red-Green-Refactor Workflow

### RED Phase (Complete) ✅

**Status:** All 10 tests in `test_9_4.bats` fail — RED phase confirmed.

- ✅ Failing tests generated and verified
- ✅ Each test maps to a specific AC or structural requirement
- ✅ Tests use direct bash assertions (bats-native pattern; no `test.skip()`)
- ✅ Mock requirements minimal — source-inspection only
- ✅ Bash 3.2 constraint documented and tested (AC #4)

### GREEN Phase (DEV Agent — Next Steps)

1. Add `_CLEANUP_TMPDIR=""` global at top of `install.sh`
2. Add `cleanup()` function using simple variable check
3. Register `trap cleanup EXIT` once at top level
4. Replace inline `trap 'rm -rf "$TMPDIR_WORK"' EXIT` with `_CLEANUP_TMPDIR="$TMPDIR_WORK"`
5. Run `bats tests/installer/test_9_4.bats` → verify 10/10 pass
6. Run full regression suite → verify 0 failures

### REFACTOR Phase

Not applicable — this story IS the refactor. No further cleanup needed beyond implementation.

---

## Next Steps

1. **Implement** the 4 tasks in `install.sh` (see Implementation Checklist above)
2. **Run** `bats tests/installer/test_9_4.bats` → must report 10/10 ok
3. **Run** full regression suite → must exit 0
4. **Mark story done** in sprint-status.yaml

---

## Knowledge Base References Applied

- **test-levels-framework.md** — Backend-only test pyramid (no E2E for shell scripts)
- **test-quality.md** — Root cause testing; no `xfail`; assert expected behavior not symptoms
- **data-factories.md** — Shell stub design (minimal mocking; source inspection preferred)
- **ci-burn-in.md** — Regression guard requirements (full suite must pass)
- **test-priorities-matrix.md** — P0/P1/P2 assignment based on risk (EXIT trap failure = data loss risk → P0)
- **component-tdd.md** — TDD red phase discipline

---

**Generated by:** BMad TEA Agent - ATDD Workflow
**Workflow:** `bmad-testarch-atdd`
**Story:** 9.4-bash-installer-refactor-exit-trap-to-composable-cleanup
**Date:** 2026-04-13
