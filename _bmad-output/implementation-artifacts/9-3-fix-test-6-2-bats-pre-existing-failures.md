# Story 9.3: Fix test_6_2.bats pre-existing failures (pre-launch blocker)

Status: ready-for-dev

<!-- GH Issue: #83 | Epic: #80 | PR must include: Closes #83 -->

## Story

As a user about to run `install.sh` for the first time,
I want the full installer test suite to pass with zero failures,
So that I can trust the installer is correct before it runs on my machine.

## Background

Epic 6 closed with `test_6_2.bats` producing failures in the full regression suite. Story 6.4's completion notes stated: "Pre-existing failures in `test_6_2.bats` are unrelated to this story" — but this means the suite was never resolved. The Epic 6 retrospective rated this as a **pre-launch blocker**: "Before any public release of the installer, `test_6_2.bats` must pass cleanly in the full regression suite."

Source: Epic 6 retrospective, Challenge #1, Technical Debt #1.

## Acceptance Criteria

1. **Given** the full bats regression suite is run **When** this story is complete **Then** `bats tests/installer/test_6_1.bats tests/installer/test_6_2.bats tests/installer/test_6_3.bats tests/installer/test_6_4.bats` exits with code 0 and reports 0 failures.

2. **Given** any test failure in test_6_2.bats is diagnosed **When** the root cause is identified **Then** the fix addresses the root cause (not just skips the failing test or marks it xfail).

3. **Given** the fix is applied **When** test_6_2.bats is run in isolation **Then** it also passes: `bats tests/installer/test_6_2.bats` exits 0.

4. **Given** the fix is applied **When** the existing passing tests in test_6_1.bats, test_6_3.bats, and test_6_4.bats are re-run **Then** they continue to pass (zero regressions).

## Tasks / Subtasks

- [ ] Task 1: Reproduce the failures (AC: #1, #3)
  - [ ] Run `bats tests/installer/test_6_2.bats` and capture the full output
  - [ ] Run the full suite and capture output: `bats tests/installer/test_6_1.bats tests/installer/test_6_2.bats tests/installer/test_6_3.bats tests/installer/test_6_4.bats`
  - [ ] Confirm that the failures match the root cause described in Dev Notes (python3 JSON parse error from plain-string stub)

- [ ] Task 2: Fix Issue 1 — `_gh api /user` stub returns plain string instead of JSON (AC: #2)
  - [ ] Find every `_gh()` stub in `test_6_2.bats` that has a `"api /user"` case returning a plain string
  - [ ] Change each occurrence from `echo "testuser"` to `echo '{"login":"testuser"}'`
  - [ ] Primary location: `make_gh_stub()` function at the top of the file; also check every inline `cat > "${HOME}/stub_env.sh"` block in individual tests
  - [ ] Run this grep to locate all occurrences: `grep -n '"api /user"' tests/installer/test_6_2.bats`
  - [ ] Do NOT modify `install.sh`, `test_6_1.bats`, `test_6_3.bats`, or `test_6_4.bats`

- [ ] Task 3: Fix Issue 2 — tests call `store_machine_token` which no longer exists in `install.sh` (AC: #2)
  - [ ] Story 6.3 merged `store_machine_token()` into `register_machine()` — the function no longer exists as a standalone function
  - [ ] Identify all tests in `test_6_2.bats` that call `store_machine_token` directly (3 tests: lines ~275, ~308, ~350)
  - [ ] Replace each `store_machine_token` call with `register_machine`
  - [ ] Verify the tests' assertions still apply: `register_machine` writes `config.toml` with `chmod 600` and reads `gh auth token` — the same behavior that `store_machine_token` had
  - [ ] Adjust any stub patterns in those tests if `register_machine` requires additional stub cases (e.g., `api repos/*` for registry.json PUT)

- [ ] Task 4: Verify all tests pass (AC: #1, #3, #4)
  - [ ] Run `bats tests/installer/test_6_2.bats` — must report 16/16 ok, exit 0
  - [ ] Run `bats tests/installer/test_6_1.bats` — must still pass (regression guard)
  - [ ] Run full suite: `bats tests/installer/test_6_1.bats tests/installer/test_6_2.bats tests/installer/test_6_3.bats tests/installer/test_6_4.bats` — must exit 0, 0 failures

## Dev Notes

### Root Causes (Pre-Diagnosed — Do Not Re-Diagnose)

There are **two distinct failure causes** in `test_6_2.bats`, both introduced by Story 6.3's refactoring. `test_6_2.bats` was never updated after Story 6.3 merged.

#### Root Cause 1: `_gh api /user` stub returns plain string instead of JSON

Story 6.2 was written when `install.sh` used `_gh api /user --jq '.login'`, which returns a plain login string. Story 6.3 changed ALL `_gh api /user` calls to use `python3` JSON parsing:

```bash
# Current install.sh pattern (introduced in Story 6.3):
USER_JSON=$(_gh api /user)
GITHUB_USER=$(echo "$USER_JSON" | python3 -c "import sys, json; print(json.load(sys.stdin)['login'])")
```

The test stubs in `test_6_2.bats` still return plain strings:

```bash
# BROKEN stub (current state in test_6_2.bats):
"api /user")
  echo "testuser"   # <-- plain string; python3 JSON parse fails on this
  ;;
```

When any `install.sh` function calls `_gh api /user`, the stub returns `testuser`, then `python3 -c "...json.load(sys.stdin)['login']"` raises a JSON parse exception, causing the bash subshell to exit non-zero.

Confirmation: `test_6_3.bats` (written after the Story 6.3 refactoring) correctly stubs this as `echo '{"login": "testuser"}'`.

#### Root Cause 2: Tests call `store_machine_token()` which no longer exists in `install.sh`

Story 6.2 implemented `store_machine_token()` as a standalone function. Story 6.3 merged its logic into `register_machine()` — `store_machine_token` was removed entirely and is not present in the current `install.sh`.

Three tests in `test_6_2.bats` call `store_machine_token` directly:
- `[P1] gh auth token result stored in ~/.config/vibestats/config.toml` (~line 275)
- `[P0] ~/.config/vibestats/config.toml created with permissions 600` (~line 308)
- `[P0] installer exits non-zero and prints error when gh auth token fails` (~line 350)

These tests fail with a bash error: `store_machine_token: command not found`.

The fix: replace `store_machine_token` calls with `register_machine`. The `register_machine()` function performs all the same operations: calls `_gh auth token`, writes `~/.config/vibestats/config.toml` with `chmod 600`, and registers the machine in `registry.json`. Stubs for those tests may need additional cases (e.g. `api repos/*`) to handle `register_machine`'s registry.json API calls.

### The Fixes

**Only modify `tests/installer/test_6_2.bats`. Do NOT change `install.sh`.**

#### Fix 1: Update `"api /user"` stub return value to JSON

Every `"api /user"` case in every `_gh()` stub must return valid JSON:

```bash
# CORRECT stub — returns JSON matching install.sh's python3 parsing:
"api /user")
  echo '{"login":"testuser"}'
  ;;
```

The file has two types of stub definitions:

1. **`make_gh_stub()` function** (top of file, ~line 55): generates `stub_env.sh` using a heredoc. The `"api /user"` case here must be changed.
2. **Inline stubs in individual tests**: Several tests bypass `make_gh_stub()` and write their own `cat > "${HOME}/stub_env.sh" <<'STUB'` or `<<STUB` blocks. Every one that has `"api /user"` must be updated.

Locate all occurrences before fixing:

```bash
grep -n '"api /user"' tests/installer/test_6_2.bats
grep -n 'echo "testuser"' tests/installer/test_6_2.bats
```

Verify after fixing:

```bash
grep -A1 '"api /user"' tests/installer/test_6_2.bats
# Every line after "api /user") should be: echo '{"login":"testuser"}'
```

#### Fix 2: Replace `store_machine_token` calls with `register_machine`

Three tests call `store_machine_token` which no longer exists. Replace the function call in each test's subshell invocation:

```bash
# BEFORE (broken):
run bash --noprofile --norc -c "
  source '${HOME}/stub_env.sh'
  source '${INSTALL_SH}'
  store_machine_token
" 2>&1

# AFTER (correct):
run bash --noprofile --norc -c "
  source '${HOME}/stub_env.sh'
  source '${INSTALL_SH}'
  register_machine
" 2>&1
```

`register_machine()` calls `_gh auth token` AND makes `api repos/*` API calls for registry.json. The inline stubs for those three tests only stub `auth token` and `api /user`. Add an `api repos*` catch-all case if not already present:

```bash
"api repos"*)
  echo "gh api repos: \$*" >> "${HOME}/gh_calls.log"
  return 0
  ;;
```

The test assertions check for `config.toml` existence and permissions — `register_machine()` still writes that file, so the assertions remain valid.

Locate the three tests:

```bash
grep -n "store_machine_token" tests/installer/test_6_2.bats
```

### Files to Change

| File | Changes |
|------|---------|
| `tests/installer/test_6_2.bats` | (1) Change all `"api /user"` stub returns from plain string to JSON; (2) Replace 3x `store_machine_token` calls with `register_machine`; (3) Add `api repos*` stub cases to the 3 affected tests |

**Do NOT touch:**
- `install.sh` — production code is correct; both bugs are in the tests
- `tests/installer/test_6_1.bats` — unrelated, do not modify
- `tests/installer/test_6_3.bats` — unrelated, do not modify
- `tests/installer/test_6_4.bats` — unrelated, do not modify

### Current install.sh Functions (Post-Story 6.3)

Story 6.3 refactored `install.sh`. The current function list relevant to `test_6_2.bats`:

| Function | Status | Notes |
|----------|--------|-------|
| `create_vibestats_data_repo()` | Exists | Auto-detects GITHUB_USER via `_gh api /user` + python3 |
| `write_aggregate_workflow()` | Exists | Same auto-detect |
| `setup_vibestats_token()` | Exists | Same auto-detect |
| `register_machine()` | Exists | Same auto-detect + writes config.toml + registry.json |
| `first_install_path()` | Exists | Calls all 4 above in sequence |
| `store_machine_token()` | **REMOVED** | Merged into `register_machine()` by Story 6.3 |
| `detect_first_install()` | **REMOVED** | Replaced by `detect_install_mode()` in Story 6.3 |
| `generate_aggregate_workflow_content()` | Exists | Helper, no auto-detect needed |

Every test that calls these functions without pre-setting `export GITHUB_USER=testuser` triggers the auto-detect path (python3 JSON parse). All such tests fail with the broken plain-string stub.

### Preservation Requirements

Do NOT change any of the following — they are correct:

- `setup()` / `teardown()` isolation (temp `$HOME` via `mktemp -d`) — already correct
- The `_gh()` define-if-not-defined guard in `install.sh` — already correct
- The `export -f _gh` at the end of stub files — required for subshell export
- `make_gh_stub()` helper structure — correct, only the `"api /user"` return value needs fixing
- All other case patterns in the stubs (`repo view`, `repo create`, `api repos*`, etc.) — correct

### Anti-Patterns to Avoid

- Do NOT skip or xfail any tests — goal is 16/16 green
- Do NOT hardcode `GITHUB_USER=testuser` as a workaround in stub preambles — fix the mock return value instead
- Do NOT change `install.sh` to return to `--jq` parsing — the python3 approach is correct and intentional (Story 6.3 notes: "avoids `--jq` dependency issue in test mocks")
- Do NOT add new tests — the 16 existing tests cover the required ACs

### Verification Commands

```bash
# Before fix — verify failures exist
bats tests/installer/test_6_2.bats

# Apply fix to tests/installer/test_6_2.bats

# After fix — must pass in isolation
bats tests/installer/test_6_2.bats

# Regression guard — must not break other suites
bats tests/installer/test_6_1.bats

# Full suite — the blocker condition
bats tests/installer/test_6_1.bats tests/installer/test_6_2.bats tests/installer/test_6_3.bats tests/installer/test_6_4.bats
```

## Review Criteria

- `bats tests/installer/test_6_2.bats` exits 0 with all 16 tests passing
- `bats tests/installer/test_6_1.bats tests/installer/test_6_2.bats tests/installer/test_6_3.bats tests/installer/test_6_4.bats` exits 0 with all tests passing
- Root cause is documented in this story file's Dev Agent Record
- No tests were deleted, skipped, or marked xfail as the fix
- `install.sh` was NOT modified (fix is entirely in test stubs)

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Agent Actions Log

| Date | Action | Notes |
|------|--------|-------|
| 2026-04-12 | Story created | BMad Create-Story — Story 9.3 |

### Completion Notes List

(To be filled by dev agent after implementation)

### File List

- `tests/installer/test_6_2.bats` (modified — fix `"api /user"` stub return value to JSON; replace 3x `store_machine_token` calls with `register_machine`; add `api repos*` stub cases to the 3 affected tests)
