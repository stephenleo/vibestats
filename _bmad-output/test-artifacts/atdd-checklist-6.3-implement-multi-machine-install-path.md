---
stepsCompleted: ['step-01-preflight-and-context', 'step-02-generation-mode', 'step-03-test-strategy', 'step-04-generate-tests', 'step-04c-aggregate', 'step-05-validate-and-complete']
lastStep: 'step-05-validate-and-complete'
lastSaved: '2026-04-12'
storyId: '6.3-implement-multi-machine-install-path'
tddPhase: 'RED'
inputDocuments:
  - '_bmad-output/implementation-artifacts/6-3-implement-multi-machine-install-path.md'
  - '_bmad-output/test-artifacts/test-design-epic-6.md'
  - '_bmad/tea/config.yaml'
  - 'install.sh'
  - 'tests/installer/test_6_1.bats'
---

# ATDD Checklist: Story 6.3 — Implement Multi-Machine Install Path

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

- [x] Story approved with clear acceptance criteria (2 ACs defined)
- [x] Test framework: `bats-core` specified in story Dev Notes
- [x] Development environment available
- [x] `install.sh` exists with Story 6.1 functions (`_gh()`, `install_gh_if_missing()`, `check_gh_version()`, `check_gh_auth()`, `detect_platform()`, `download_and_install_binary()`)
- [x] Existing test patterns available in `tests/installer/test_6_1.bats`
- [x] Epic 6 test design with risk assessments (R-004, R-005) loaded

### Loaded Context

- Story: `_bmad-output/implementation-artifacts/6-3-implement-multi-machine-install-path.md`
- Epic test design: `_bmad-output/test-artifacts/test-design-epic-6.md`
- TEA config: `_bmad/tea/config.yaml`
- TEA config flags: `tea_use_playwright_utils: true` (not applicable — backend stack), `tea_use_pactjs_utils: false`, `tea_execution_mode: auto → sequential`

---

## Step 2: Generation Mode

**Mode selected:** AI generation
**Reason:** Backend stack with clear acceptance criteria — no browser recording needed. ACs describe deterministic shell function behaviors testable via mocked `_gh()` function overrides, following the same patterns established in `test_6_1.bats`.

---

## Step 3: Test Strategy

### Acceptance Criteria → Test Scenario Mapping

| AC | Scenario | Test Level | Priority | Risk Link |
|----|----------|------------|----------|-----------|
| AC #1 (FR5) | `vibestats-data` exists → repo creation skipped, workflow write skipped, `VIBESTATS_TOKEN` not set | Unit/Shell | P0 | R-004 |
| AC #2 (FR6) | `registry.json` entry has all required fields: `machine_id`, `hostname`, `status="active"`, `last_seen` ISO 8601 UTC | Unit/Shell | P0 | R-005 |
| AC #1 | `gh repo view` called with `${GITHUB_USER}/vibestats-data` — not hardcoded org | Unit/Shell | P1 | R-004 |
| AC #1 | `detect_install_mode` sets `INSTALL_MODE=multi-machine` when repo exists | Unit/Shell | P1 | R-004 |
| AC #1 | `detect_install_mode` sets `INSTALL_MODE=first-install` when repo does not exist | Unit/Shell | P1 | R-003 |
| AC #2 | `register_machine` appends new entry — existing machines in `registry.json` preserved | Unit/Shell | P1 | R-005 |
| NFR6 | `config.toml` written with permissions `600` after machine-side token | Unit/Shell | P1 | R-002 |

### Test Level Rationale

**Unit/Shell (bats-core) — all 7 tests**

- `detect_install_mode()` and `register_machine()` are pure shell functions called with mocked `_gh()`
- No external network calls — all `gh` CLI interactions intercepted via override
- Shell spy pattern (recording calls to `gh_calls.log`) confirms which commands were/were not invoked
- `$HOME` override isolates file system mutations (temp dir per test)

### Red Phase Design

All tests use `skip "RED: <function> not yet implemented in install.sh"` to enforce TDD red phase. Tests will:
1. Fail at function-not-found (`detect_install_mode: command not found`) when `skip` is removed
2. Pass only after the implementation is complete

---

## Step 4: Test Generation

### TDD Red Phase — Execution Mode

**Mode:** sequential (config: `tea_execution_mode: auto`, resolved to `sequential` — backend stack, no subagent needed)

### Generated Test Files

**File:** `tests/installer/test_6_3.bats`

---

## Step 4C: Aggregated Results

### TDD Red Phase Verification

All 7 tests use `skip "RED: ..."` directive.

- **Test count:** 7 (2 P0, 5 P1)
- **All tests:** Skipped in current state (TDD red phase)
- **Expected failure when skip removed:** `detect_install_mode: command not found` or `register_machine: command not found`
- **Tests assert expected behavior** — not placeholder assertions

---

## Failing Tests Created (RED Phase)

### Shell/Unit Tests (7 tests)

**File:** `tests/installer/test_6_3.bats`

- **[P0] `multi-machine path: vibestats-data exists → repo creation skipped, workflow write skipped, VIBESTATS_TOKEN not set`**
  - **Status:** RED — `detect_install_mode` function not yet in `install.sh`
  - **Verifies:** AC #1 (FR5), R-004 mitigation — installer does not perform first-install steps when repo already exists
  - **Mock:** `_gh repo view` returns 0; spy asserts `repo create`, `secret set VIBESTATS_TOKEN`, and `aggregate.yml` write NOT called

- **[P0] `registry.json entry has all required fields: machine_id, hostname, status=active, last_seen ISO 8601 UTC`**
  - **Status:** RED — `register_machine` function not yet in `install.sh`
  - **Verifies:** AC #2 (FR6), R-005 mitigation — all four schema fields present and correct
  - **Mock:** `_gh api` PUT captured; Python stdlib parses JSON and validates field types + ISO 8601 timestamp regex

- **[P1] `vibestats-data repo detection uses correct repo name (username/vibestats-data not hardcoded org)`**
  - **Status:** RED — `detect_install_mode` not yet implemented
  - **Verifies:** AC #1 (FR5) — `gh repo view` called with `${GITHUB_USER}/vibestats-data` using dynamic user from `gh api /user`

- **[P1] `detect_install_mode sets INSTALL_MODE=multi-machine when vibestats-data repo exists`**
  - **Status:** RED — `detect_install_mode` not yet implemented
  - **Verifies:** AC #1 — correct mode set on existing repo detection

- **[P1] `detect_install_mode sets INSTALL_MODE=first-install when vibestats-data repo does not exist`**
  - **Status:** RED — `detect_install_mode` not yet implemented
  - **Verifies:** AC #1 — correct mode set when repo absent; `set -euo pipefail` safe (no abort on `_gh repo view` non-zero)

- **[P1] `register_machine appends new entry without overwriting existing machines`**
  - **Status:** RED — `register_machine` not yet implemented
  - **Verifies:** AC #2 (FR6), R-005 — append-only rule; existing machine entries preserved

- **[P1] `register_machine writes config.toml with 600 permissions`**
  - **Status:** RED — `register_machine` not yet implemented
  - **Verifies:** NFR6 (R-002) — `chmod 600` applied immediately after writing `~/.config/vibestats/config.toml`

---

## Mock Requirements

### `_gh()` Shell Function Override

Defined in test `setup()` as an inline `cat > stub_env.sh` block, sourced before `install.sh`.

**Calls mocked:**

| `_gh` invocation | Mock response | Notes |
|---|---|---|
| `_gh api /user` | `{"login": "testuser"}` | Sets `GITHUB_USER` |
| `_gh repo view testuser/vibestats-data` | exit 0 (multi-machine) or exit 1 (first-install) | Simulates repo existence check |
| `_gh repo create` | Should NOT be called in multi-machine path | Spy: `UNEXPECTED` log entry |
| `_gh secret set VIBESTATS_TOKEN` | Should NOT be called | Spy: `UNEXPECTED` log entry |
| `_gh api repos/.../registry.json` GET | exit 1 (no existing) or base64-encoded JSON (existing) | Two test variants |
| `_gh api repos/.../registry.json --method PUT` | Captures `--field content=` value; returns SHA | Used to assert PUT body |
| `_gh auth token` | `ghp_TESTMACHINETOKEN` | Machine-side token for config.toml |

**Spy Pattern:**

```bash
_gh() {
  echo "_gh $*" >> "${GH_SPY_LOG}"
  case "$1 $2" in ...
  esac
}
export -f _gh
```

Assertions check `${GH_SPY_LOG}` for presence/absence of specific subcommand calls.

---

## Implementation Checklist

### Test: `[P0] multi-machine path: vibestats-data exists → repo creation skipped`

**File:** `tests/installer/test_6_3.bats`

**Tasks to make this test pass:**

- [ ] Implement `detect_install_mode()` in `install.sh` after existing Step 5 functions
  - [ ] Call `_gh api /user --jq .login` to get `GITHUB_USER`
  - [ ] Use `if _gh repo view "${GITHUB_USER}/vibestats-data" ...` construct (safe under `set -euo pipefail`)
  - [ ] Set `INSTALL_MODE="multi-machine"` when repo exists
  - [ ] Set `INSTALL_MODE="first-install"` when repo does not exist
  - [ ] Export both `INSTALL_MODE` and `GITHUB_USER`
- [ ] Implement `register_machine()` in `install.sh`
  - [ ] Does NOT call `gh repo create`
  - [ ] Does NOT call `gh secret set VIBESTATS_TOKEN`
  - [ ] Does NOT write `aggregate.yml`
- [ ] Update `main()` to call `detect_install_mode` then branch on `INSTALL_MODE`
- [ ] Remove `skip` from test
- [ ] Run test: `bats tests/installer/test_6_3.bats --filter "P0.*multi-machine"`
- [ ] Test passes (green phase)

**Estimated Effort:** 2–3 hours

---

### Test: `[P0] registry.json entry has all required fields`

**File:** `tests/installer/test_6_3.bats`

**Tasks to make this test pass:**

- [ ] Implement `register_machine()` function with `machine_id` generation:
  - [ ] `case "$(uname -s)"` branch: macOS uses `system_profiler SPHardwareDataType` UUID suffix; Linux uses `/etc/machine-id`
  - [ ] Format: `<hostname>-<6char-suffix>`
- [ ] Fetch `registry.json` via `_gh api` Contents GET (handle 404 → empty `{"machines": []}`)
- [ ] Decode base64 content (`base64 -D` macOS, `base64 -d` Linux)
- [ ] Append new entry via Python stdlib one-liner (no `jq`)
- [ ] Encode updated JSON back to base64
- [ ] PUT via `_gh api` Contents PUT (with SHA if file existed)
- [ ] Ensure `last_seen` is ISO 8601 UTC: `$(date -u '+%Y-%m-%dT%H:%M:%SZ')`
- [ ] Remove `skip` from test
- [ ] Run test: `bats tests/installer/test_6_3.bats --filter "P0.*registry"`
- [ ] Test passes (green phase)

**Estimated Effort:** 3–4 hours

---

### Tests: `[P1]` group

**Tasks to make all P1 tests pass:**

- [ ] `GITHUB_USER` obtained via `_gh api /user --jq .login` (not hardcoded)
- [ ] `detect_install_mode` outputs `INSTALL_MODE=multi-machine` or `INSTALL_MODE=first-install`
- [ ] `register_machine` appends to `machines` array (never overwrites existing entries)
- [ ] `config.toml` written with `chmod 600` immediately after file creation
- [ ] Remove `skip` from each P1 test
- [ ] Run: `bats tests/installer/test_6_3.bats`
- [ ] All tests pass (green phase)

**Estimated Effort:** 1–2 hours

---

## Running Tests

```bash
# Run all failing tests for this story
bats tests/installer/test_6_3.bats

# Run only P0 tests
bats tests/installer/test_6_3.bats --filter "P0"

# Run only P1 tests
bats tests/installer/test_6_3.bats --filter "P1"

# Run full installer test suite (6.1 + 6.3)
bats tests/installer/

# Syntax check install.sh before running tests
bash -n install.sh
```

---

## Red-Green-Refactor Workflow

### RED Phase (Complete) ✅

**TEA Agent Responsibilities:**

- ✅ All tests written with `skip "RED: ..."` directive
- ✅ Mocking strategy documented (shell function overrides)
- ✅ Implementation checklist created
- ✅ Tests assert EXPECTED behavior (not placeholders)
- ✅ Tests designed to fail when `skip` removed (function-not-found)

**Verification:**

- All 7 tests are in SKIP state (bats reports "7 skipped")
- When `skip` lines are removed before implementation: tests fail with "command not found"
- When `skip` lines are removed after implementation: tests should pass

---

### GREEN Phase (DEV Agent — Next Steps)

**DEV Agent Responsibilities:**

1. Pick the first `[P0]` test from implementation checklist
2. Read the test to understand expected behavior
3. Implement `detect_install_mode()` minimal code
4. Remove `skip` from that test and run: `bats tests/installer/test_6_3.bats --filter "P0.*multi-machine"`
5. Verify it passes (green)
6. Implement `register_machine()` minimal code
7. Remove `skip` from the second P0 test and run
8. Verify it passes (green)
9. Continue with P1 tests

**Key Principles:**

- All `gh` calls must go through `_gh()` helper — never call `gh` directly
- Use `if _gh repo view ...` construct — NOT `$?` check — for `set -euo pipefail` safety
- No `jq` dependency — use Python stdlib for JSON manipulation
- Bash 3.2 compatibility — no `declare -A`, no `mapfile`

---

### REFACTOR Phase (DEV Agent — After All Tests Pass)

1. Verify `bash -n install.sh` passes (syntax check)
2. Verify `set -euo pipefail` still present on line 2 of `install.sh`
3. Verify no hardcoded usernames (always uses `$GITHUB_USER`)
4. Verify `config.toml` written with `chmod 600` immediately after creation
5. Run full suite: `bats tests/installer/` — all tests pass
6. Code review ready

---

## Summary Statistics

- **TDD Phase:** RED
- **Total Tests:** 7 (all skipped)
  - P0: 2 (R-004, R-005 mitigations)
  - P1: 5 (repo detection, mode branching, append-only, permissions)
- **E2E Tests:** 0 (backend stack — no browser testing)
- **Shell Unit Tests:** 7
- **Acceptance Criteria Covered:** AC #1 (FR5), AC #2 (FR6)
- **Risk Mitigations Covered:** R-002 (partial), R-004, R-005

---

## Next Steps

1. Share this checklist and `tests/installer/test_6_3.bats` with the dev workflow
2. Run tests to confirm RED phase: `bats tests/installer/test_6_3.bats` (expect 7 skipped)
3. Begin implementation using implementation checklist
4. Work test by test: remove `skip` → implement → verify green
5. When all tests pass, run full suite: `bats tests/installer/`
6. When refactoring complete, update story status to `dev-done` in sprint-status.yaml

---

## Knowledge Base References Applied

- `test-levels-framework.md` — Shell unit tests selected for backend-only stack
- `test-priorities-matrix.md` — P0/P1 assignment by risk score from R-004 and R-005
- `test-quality.md` — Deterministic, isolated, explicit assertions
- `data-factories.md` — Shell mock overrides follow factory pattern principles
- `test-healing-patterns.md` — `_gh()` helper wrapper ensures test mockability
- `ci-burn-in.md` — P0 tests block CI gate

---

**Generated by BMad TEA Agent** — 2026-04-12
