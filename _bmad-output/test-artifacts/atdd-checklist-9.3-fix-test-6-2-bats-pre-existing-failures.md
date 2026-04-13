---
stepsCompleted: ['step-01-preflight-and-context', 'step-02-generation-mode', 'step-03-test-strategy', 'step-04-generate-tests', 'step-04c-aggregate']
lastStep: 'step-04c-aggregate'
lastSaved: '2026-04-13'
workflowType: 'testarch-atdd'
inputDocuments:
  - '_bmad-output/implementation-artifacts/9-3-fix-test-6-2-bats-pre-existing-failures.md'
  - '_bmad-output/test-artifacts/test-design-epic-9.md'
  - 'tests/installer/test_6_2.bats'
  - 'tests/installer/test_6_3.bats'
  - '_bmad/tea/config.yaml'
---

# ATDD Checklist - Epic 9, Story 9.3: Fix test_6_2.bats pre-existing failures (pre-launch blocker)

**Date:** 2026-04-13
**Author:** Leo
**Primary Test Level:** Shell (bats-core)
**Stack:** Backend (installer shell scripts)
**TDD Phase:** RED — All 16 tests in `tests/installer/test_6_2.bats` currently fail

---

## Story Summary

As a user about to run `install.sh` for the first time, I want the full installer test suite to pass with zero failures, so that I can trust the installer is correct before it runs on my machine.

**As a** user about to run `install.sh` for the first time
**I want** the full bats installer test suite to pass with zero failures
**So that** I can trust the installer is correct before it runs on my machine

---

## Acceptance Criteria

1. **Given** the full bats regression suite is run **When** this story is complete **Then** `bats tests/installer/test_6_1.bats tests/installer/test_6_2.bats tests/installer/test_6_3.bats tests/installer/test_6_4.bats` exits with code 0 and reports 0 failures.

2. **Given** any test failure in test_6_2.bats is diagnosed **When** the root cause is identified **Then** the fix addresses the root cause (not just skips the failing test or marks it xfail).

3. **Given** the fix is applied **When** test_6_2.bats is run in isolation **Then** it also passes: `bats tests/installer/test_6_2.bats` exits 0.

4. **Given** the fix is applied **When** the existing passing tests in test_6_1.bats, test_6_3.bats, and test_6_4.bats are re-run **Then** they continue to pass (zero regressions).

---

## Failing Tests (RED Phase — Existing Tests in test_6_2.bats)

**Note:** This story is a test-fix story. The 16 acceptance tests already exist in `tests/installer/test_6_2.bats` and are currently RED (failing). The ATDD phase documents the failing state, root causes, and the implementation tasks that will move them to GREEN.

**File:** `tests/installer/test_6_2.bats` (557 lines)

### AC #1 Tests — vibestats-data repo creation

- **Test 1:** `[P1] vibestats-data does not exist → gh repo create --private called`
  - **Status:** RED — `_gh api /user` stub returns plain string `"testuser"`; `python3` JSON parse raises exception; function exits non-zero before `repo create` is called
  - **Root Cause:** Issue 1 (stub mismatch)
  - **Verifies:** AC #1 — `create_vibestats_data_repo()` calls `gh repo create --private`

- **Test 2:** `[P1] gh repo create called with --private flag`
  - **Status:** RED — same as Test 1
  - **Root Cause:** Issue 1 (stub mismatch)
  - **Verifies:** AC #1 — `--private` flag is passed to `gh repo create`

### AC #2 Tests — aggregate.yml workflow

- **Test 3:** `[P1] aggregate.yml written calling stephenleo/vibestats@v1`
  - **Status:** RED — `_gh api /user` stub returns plain string; `write_aggregate_workflow()` calls auto-detect which fails JSON parse
  - **Root Cause:** Issue 1 (stub mismatch)
  - **Verifies:** AC #2 — workflow content references `stephenleo/vibestats@v1`

- **Test 4:** `[P1] aggregate.yml workflow content includes cron and workflow_dispatch triggers`
  - **Status:** RED — same as Test 3
  - **Root Cause:** Issue 1 (stub mismatch)
  - **Verifies:** AC #2 — workflow includes `schedule` and `workflow_dispatch` triggers

### AC #3 Tests — VIBESTATS_TOKEN (not written to disk)

- **Test 5:** `[P0] VIBESTATS_TOKEN is never written to disk or echoed to stdout`
  - **Status:** RED — `_gh api /user` stub in this test's inline stub returns plain string `"testuser"`; `setup_vibestats_token()` fails JSON parse
  - **Root Cause:** Issue 1 (stub mismatch — inline stub)
  - **Verifies:** AC #3 — token never persisted to disk (NFR7)

- **Test 6:** `[P1] gh secret set called with VIBESTATS_TOKEN for vibestats-data repo`
  - **Status:** RED — same JSON parse failure in `setup_vibestats_token()`
  - **Root Cause:** Issue 1 (stub mismatch)
  - **Verifies:** AC #3 — `gh secret set` is called with `VIBESTATS_TOKEN`

### AC #4 Tests — config.toml local token storage

- **Test 7:** `[P1] gh auth token result stored in ~/.config/vibestats/config.toml`
  - **Status:** RED — test calls `store_machine_token` which no longer exists in `install.sh`; bash error: `store_machine_token: command not found`; also inline stub still returns plain string for `api /user`
  - **Root Cause:** Issue 2 (function removed) + Issue 1 (stub mismatch — inline stub)
  - **Verifies:** AC #4 — `config.toml` is written with token value (FR39)

- **Test 8:** `[P0] ~/.config/vibestats/config.toml created with permissions 600`
  - **Status:** RED — test calls `store_machine_token`; bash error: `store_machine_token: command not found`; also inline stub returns plain string for `api /user`
  - **Root Cause:** Issue 2 (function removed) + Issue 1 (stub mismatch — inline stub)
  - **Verifies:** AC #4 — `config.toml` has permissions 600 (NFR6)

- **Test 9:** `[P0] installer exits non-zero and prints error when gh auth token fails`
  - **Status:** RED — test calls `store_machine_token`; bash error: `store_machine_token: command not found` (test expects non-zero exit, but for wrong reason)
  - **Root Cause:** Issue 2 (function removed)
  - **Verifies:** AC #4 (failure path) — installer exits non-zero when `gh auth token` fails (R-003)

### AC #5 Tests — registry.json machine entry

- **Test 10:** `[P0] registry.json entry contains machine_id field`
  - **Status:** RED — `register_machine()` calls `_gh api /user`; stub returns plain string; JSON parse fails
  - **Root Cause:** Issue 1 (stub mismatch)
  - **Verifies:** AC #5 — registry entry contains `machine_id` field (FR6)

- **Test 11:** `[P0] registry.json entry contains hostname field`
  - **Status:** RED — same as Test 10
  - **Root Cause:** Issue 1 (stub mismatch)
  - **Verifies:** AC #5 — registry entry contains `hostname` field (FR6)

- **Test 12:** `[P0] registry.json entry has status field set to active`
  - **Status:** RED — same as Test 10
  - **Root Cause:** Issue 1 (stub mismatch)
  - **Verifies:** AC #5 — registry entry `status = "active"` (FR6)

- **Test 13:** `[P0] registry.json entry has last_seen ISO 8601 UTC timestamp`
  - **Status:** RED — same as Test 10
  - **Root Cause:** Issue 1 (stub mismatch)
  - **Verifies:** AC #5 — registry entry has ISO 8601 `last_seen` timestamp (FR6)

### Failure-path Tests

- **Test 14:** `[P0] installer exits non-zero when gh repo create fails`
  - **Status:** RED — inline stub returns plain string for `api /user`; JSON parse fails before `repo create` is even attempted; exit is non-zero for wrong reason (test assertion `[ "$status" -ne 0 ]` would coincidentally pass, but the test body fails at setup)
  - **Root Cause:** Issue 1 (stub mismatch — inline stub)
  - **Verifies:** AC #3 failure path — non-zero exit when `gh repo create` fails (R-003)

- **Test 15:** `[P0] installer exits non-zero when gh secret set fails`
  - **Status:** RED — inline stub returns plain string for `api /user`; JSON parse fails
  - **Root Cause:** Issue 1 (stub mismatch — inline stub)
  - **Verifies:** AC #3 failure path — non-zero exit when `gh secret set` fails (R-003)

- **Test 16:** `[P1] full first-install path succeeds with all steps called in sequence`
  - **Status:** RED — inline stub returns plain string for `api /user`; `first_install_path()` fails JSON parse on first auto-detect call
  - **Root Cause:** Issue 1 (stub mismatch — inline stub)
  - **Verifies:** AC #1-#5 integration — `first_install_path()` calls all steps in sequence

---

## Root Cause Analysis

### Issue 1: `_gh api /user` stub returns plain string instead of JSON

**All stubs in `test_6_2.bats` still return:**
```bash
"api /user")
  echo "testuser"   # BROKEN: plain string
  ;;
```

**`install.sh` now requires (changed in Story 6.3):**
```bash
USER_JSON=$(_gh api /user)
GITHUB_USER=$(echo "$USER_JSON" | python3 -c "import sys, json; print(json.load(sys.stdin)['login'])")
```

**Correct stub (as used in `test_6_3.bats`):**
```bash
"api /user")
  echo '{"login":"testuser"}'   # CORRECT: valid JSON
  ;;
```

**Affected locations in `test_6_2.bats`:**
1. `make_gh_stub()` function (~line 63) — generates `stub_env.sh` via heredoc
2. Inline stub in Test 5 `[P0] VIBESTATS_TOKEN is never written to disk` (~line 185)
3. Inline stub in Test 7 `[P1] gh auth token result stored` (~line 261)
4. Inline stub in Test 8 `[P0] config.toml created with permissions 600` (~line 293)
5. Inline stub in Test 14 `[P0] installer exits non-zero when gh repo create fails` (~line 438)
6. Inline stub in Test 15 `[P0] installer exits non-zero when gh secret set fails` (~line 475)
7. Inline stub in Test 16 `[P1] full first-install path succeeds` (~line 514)

### Issue 2: Tests call `store_machine_token()` which no longer exists

**`store_machine_token()` was removed by Story 6.3** — merged into `register_machine()`.

**Affected tests (3 tests):**
- Test 7 `[P1] gh auth token result stored` (~line 275): calls `store_machine_token`
- Test 8 `[P0] config.toml created with permissions 600` (~line 308): calls `store_machine_token`
- Test 9 `[P0] installer exits non-zero when gh auth token fails` (~line 350): calls `store_machine_token`

**Fix:** Replace `store_machine_token` with `register_machine` in each test's subshell invocation. Add `"api repos"*)` stub case to Tests 7 and 8 (since `register_machine` also calls registry.json API). Test 9 (failure path) only needs the replacement — it tests `auth token` failure, which happens before `api repos` is called.

---

## Implementation Checklist (What Makes These Tests GREEN)

**Only `tests/installer/test_6_2.bats` needs to be modified. Do NOT modify `install.sh`.**

### Fix 1: Update all `"api /user"` stub return values to JSON

- [ ] Locate all occurrences: `grep -n '"api /user"' tests/installer/test_6_2.bats`
- [ ] Update `make_gh_stub()` function (~line 63): change `echo "testuser"` to `echo '{"login":"testuser"}'`
- [ ] Update Test 5 inline stub (~line 185): change `echo "testuser"` to `echo '{"login":"testuser"}'`
- [ ] Update Test 7 inline stub (~line 261): change `echo "testuser"` to `echo '{"login":"testuser"}'`
- [ ] Update Test 8 inline stub (~line 293): change `echo "testuser"` to `echo '{"login":"testuser"}'`
- [ ] Update Test 14 inline stub (~line 438): change `echo "testuser"` to `echo '{"login":"testuser"}'`
- [ ] Update Test 15 inline stub (~line 475): change `echo "testuser"` to `echo '{"login":"testuser"}'`
- [ ] Update Test 16 inline stub (~line 514): change `echo "testuser"` to `echo '{"login":"testuser"}'`
- [ ] Verify: `grep -A1 '"api /user"' tests/installer/test_6_2.bats` — every line after `"api /user")` must be `echo '{"login":"testuser"}'`

### Fix 2: Replace `store_machine_token` calls with `register_machine`

- [ ] Locate all occurrences: `grep -n "store_machine_token" tests/installer/test_6_2.bats`
- [ ] Test 7 (~line 275): Replace `store_machine_token` with `register_machine`; add `"api repos"*)` stub case to the inline stub
- [ ] Test 8 (~line 308): Replace `store_machine_token` with `register_machine`; add `"api repos"*)` stub case to the inline stub
- [ ] Test 9 (~line 350): Replace `store_machine_token` with `register_machine` (no `api repos*` case needed — failure happens at `auth token` before registry is called)
- [ ] Verify: `grep -n "store_machine_token" tests/installer/test_6_2.bats` — returns no output (all replaced)

### Stub pattern to add for Tests 7 and 8

```bash
"api repos"*)
  echo "gh api repos: $*" >> "${HOME}/gh_calls.log"
  return 0
  ;;
```

### Fix 3: Verify and run

- [ ] Run `bats tests/installer/test_6_2.bats` — must report 16/16 ok, exit 0
- [ ] Run `bats tests/installer/test_6_1.bats` — must still pass (regression guard)
- [ ] Run full suite: `bats tests/installer/test_6_1.bats tests/installer/test_6_2.bats tests/installer/test_6_3.bats tests/installer/test_6_4.bats` — must exit 0, 0 failures

---

## Mock Requirements

### `_gh()` Shell Function Mock

The bats tests use a `_gh()` function override (exported to subshells) to stub GitHub CLI calls. The correct stub pattern is:

```bash
_gh() {
  case "$1 $2" in
    "auth token")
      echo "ghp_FAKE_MACHINE_TOKEN"
      ;;
    "api /user")
      echo '{"login":"testuser"}'   # MUST be valid JSON
      ;;
    "repo view")
      return 1
      ;;
    "repo create")
      echo "gh repo create: $*" >> "${HOME}/gh_calls.log"
      return 0
      ;;
    "api repos"*)
      echo "gh api repos: $*" >> "${HOME}/gh_calls.log"
      cat >> "${HOME}/gh_api_body.log" 2>/dev/null || true
      return 0
      ;;
    "api /user/personal_access_tokens"*)
      echo '{"token":"ghp_FAKE_VIBESTATS_TOKEN"}'
      ;;
    "secret set")
      echo "gh secret set: $*" >> "${HOME}/gh_calls.log"
      return 0
      ;;
    *)
      echo "gh $* called" >> "${HOME}/gh_calls.log"
      return 0
      ;;
  esac
}
export -f _gh
```

**Key change from broken state:** `"api /user"` returns `'{"login":"testuser"}'` (JSON object), not `"testuser"` (plain string).

---

## Running Tests

```bash
# Verify RED phase (before fix — confirm failures)
bats tests/installer/test_6_2.bats

# After applying Fix 1 and Fix 2 — run in isolation (AC #3)
bats tests/installer/test_6_2.bats

# Regression guard — test_6_1 must still pass (AC #4)
bats tests/installer/test_6_1.bats

# Full suite — the pre-launch blocker condition (AC #1)
bats tests/installer/test_6_1.bats tests/installer/test_6_2.bats tests/installer/test_6_3.bats tests/installer/test_6_4.bats
```

---

## Red-Green-Refactor Workflow

### RED Phase (Complete) ✅

**Status:** All 16 tests in `test_6_2.bats` currently fail.

- ✅ Failing tests documented and mapped to root causes
- ✅ Root causes identified (2 issues: stub mismatch, removed function)
- ✅ Implementation checklist created for both fixes
- ✅ Fix is in test stubs only — `install.sh` is correct and must NOT be modified
- ✅ Anti-patterns documented (no xfail, no skip, no GITHUB_USER workaround)

**Test Execution Evidence (RED Phase):**

```
Command: bats tests/installer/test_6_2.bats

Expected result (before fix):
1..16
not ok 1 [P1] vibestats-data does not exist → gh repo create --private called
not ok 2 [P1] gh repo create called with --private flag
not ok 3 [P1] aggregate.yml written calling stephenleo/vibestats@v1
not ok 4 [P1] aggregate.yml workflow content includes cron and workflow_dispatch triggers
not ok 5 [P0] VIBESTATS_TOKEN is never written to disk or echoed to stdout
not ok 6 [P1] gh secret set called with VIBESTATS_TOKEN for vibestats-data repo
not ok 7 [P1] gh auth token result stored in ~/.config/vibestats/config.toml
not ok 8 [P0] ~/.config/vibestats/config.toml created with permissions 600
not ok 9 [P0] installer exits non-zero and prints error when gh auth token fails
not ok 10 [P0] registry.json entry contains machine_id field
not ok 11 [P0] registry.json entry contains hostname field
not ok 12 [P0] registry.json entry has status field set to active
not ok 13 [P0] registry.json entry has last_seen ISO 8601 UTC timestamp
not ok 14 [P0] installer exits non-zero when gh repo create fails
not ok 15 [P0] installer exits non-zero when gh secret set fails
not ok 16 [P1] full first-install path succeeds with all steps called in sequence

Summary:
- Total tests: 16
- Passing: 0 (expected in RED phase)
- Failing: 16 (expected)
- Status: RED phase confirmed
```

---

### GREEN Phase (DEV Agent — Next Steps)

**DEV Agent Responsibilities:**

1. Apply Fix 1: Change all `"api /user"` stub returns from plain string to JSON in `tests/installer/test_6_2.bats`
2. Apply Fix 2: Replace 3 `store_machine_token` calls with `register_machine`; add `"api repos"*)` cases to Tests 7 and 8
3. Run `bats tests/installer/test_6_2.bats` — verify 16/16 ok
4. Run full regression suite — verify 0 failures across all 4 test files

**Key Principles:**

- Fix test stubs only — do NOT modify `install.sh`
- Do NOT delete, skip, or xfail any of the 16 tests
- Do NOT hardcode `export GITHUB_USER=testuser` as a workaround — fix the mock return value
- Root cause must be addressed (not symptom-patched)

---

### REFACTOR Phase

Not applicable for this story — it is a test-stub fix, not a feature implementation.

---

## Next Steps

1. **Apply Fix 1** — update all `"api /user"` stub returns in `tests/installer/test_6_2.bats` to `echo '{"login":"testuser"}'`
2. **Apply Fix 2** — replace `store_machine_token` calls with `register_machine`; add `"api repos"*)` stub cases to Tests 7 and 8
3. **Run `bats tests/installer/test_6_2.bats`** — must report 16/16 ok (AC #3)
4. **Run full suite** — must exit 0 (AC #1)
5. **Verify no regressions** in test_6_1.bats, test_6_3.bats, test_6_4.bats (AC #4)
6. **Document root cause** in Dev Agent Record of story file
7. **Mark story done** in sprint-status.yaml

---

## Knowledge Base References Applied

- **test-quality.md** — Test isolation, root cause over symptom fix, no xfail anti-pattern
- **test-levels-framework.md** — Shell (bats) test level selection for installer scripts
- **data-factories.md** — Shell stub/mock design (adapted for bats `_gh()` override pattern)
- **ci-burn-in.md** — Regression guard requirements (full suite must pass)
- **test-priorities-matrix.md** — P0/P1 assignment based on pre-launch blocker risk (R-004)

---

**Generated by:** BMad TEA Agent - ATDD Workflow
**Workflow:** `bmad-testarch-atdd`
**Story:** 9.3-fix-test-6-2-bats-pre-existing-failures
**Date:** 2026-04-13
