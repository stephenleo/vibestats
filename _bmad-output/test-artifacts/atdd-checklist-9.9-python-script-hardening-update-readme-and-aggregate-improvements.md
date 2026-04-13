---
stepsCompleted: ['step-01-preflight-and-context', 'step-02-generation-mode', 'step-03-test-strategy', 'step-04-generate-tests', 'step-04c-aggregate', 'step-05-validate-and-complete']
lastStep: 'step-05-validate-and-complete'
lastSaved: '2026-04-13'
inputDocuments:
  - _bmad-output/implementation-artifacts/9-9-python-script-hardening-update-readme-and-aggregate-improvements.md
  - _bmad-output/planning-artifacts/epic-9.md
  - _bmad-output/test-artifacts/test-design-epic-9.md
  - action/tests/test_update_readme.py
  - action/tests/test_aggregate.py
  - action/update_readme.py
  - _bmad/tea/config.yaml
---

# ATDD Checklist: Story 9.9 — Python Script Hardening

**Story:** update_readme.py and aggregate.py improvements
**GH Issue:** [#89](https://github.com/stephenleo/vibestats/issues/89)
**Epic:** 9 (Post-Sprint Quality & Technical Debt)
**TDD Phase:** RED (current)
**Date:** 2026-04-13

---

## Stack Detection

- **Detected Stack:** `backend`
- **Detection Evidence:** Python project in `action/`; pytest-based tests; no `playwright.config.*` or browser indicators
- **Generation Mode:** AI generation (no browser recording for backend)
- **Execution Mode:** Sequential (auto-resolved, no subagent runtime available)

---

## TDD Red Phase — Failing Tests Generated

- **Test File:** `action/tests/test_9_9_python_hardening.py`
- **Total Tests:** 5 (all with `@pytest.mark.skip`)
- **TDD Phase:** RED — all tests will fail when `@pytest.mark.skip` is removed until the feature is implemented
- **Verification:** All 5 tests collected and skipped; 140 existing tests still pass (no regressions)

---

## Acceptance Criteria Coverage

| AC | Description | Test(s) | Priority |
|----|-------------|---------|----------|
| AC1 | `update_readme.py --username ""` exits non-zero with clear error to stderr | `test_tc6_empty_username_exits_nonzero`, `test_tc7_whitespace_only_username_exits_nonzero` | P1, P2 |
| AC2 | New test in `test_update_readme.py` for empty-username case | `test_tc6_empty_username_exits_nonzero` (to be added to `test_update_readme.py` in green phase) | P1 |
| AC3 | `expected_output/data.json` removed (with documented rationale) | `test_expected_output_fixture_removed`, `test_expected_output_directory_removed` | P1 |
| AC4 | Full Python test suite passes with 0 failures | `test_full_pytest_suite_passes` | P3 |

---

## Test Scenarios

### P1 Tests (Must Pass for Story Completion)

**9.9-TC-6: Empty `--username` exits non-zero with stderr message (AC1, AC2)**
- Test: `test_tc6_empty_username_exits_nonzero`
- When: `update_readme.py --username ""` is invoked with a valid README containing markers
- Then: `returncode != 0` AND `"empty"` or `"--username"` appears in `stderr`
- RED phase behavior: Current code returns exit 0 with no stderr (the bug)

**9.9-UNIT-003: Dead fixture `expected_output/data.json` is absent (AC3)**
- Test: `test_expected_output_fixture_removed`
- When: Story 9.9 Task 3 has been executed
- Then: `action/tests/fixtures/expected_output/data.json` does not exist
- RED phase behavior: File currently exists → assertion fails

**9.9-UNIT-004: Dead fixture directory is absent (AC3)**
- Test: `test_expected_output_directory_removed`
- When: Story 9.9 Task 3 has been executed
- Then: `action/tests/fixtures/expected_output/` is absent or empty
- RED phase behavior: Directory contains `data.json` and `heatmap.svg` → fails if non-empty

### P2 Tests (Should Pass for Quality)

**9.9-TC-7: Whitespace-only `--username` exits non-zero (AC1 edge case)**
- Test: `test_tc7_whitespace_only_username_exits_nonzero`
- When: `update_readme.py --username "   "` is invoked
- Then: `returncode != 0` AND `"empty"` or `"--username"` appears in `stderr`
- RED phase behavior: `str.strip()` on whitespace → empty string guard needed

### P3 Tests (Combined Regression)

**9.9-UNIT-005: Full pytest suite passes (AC4)**
- Test: `test_full_pytest_suite_passes`
- When: All AC1–AC3 changes are in place
- Then: `python3 -m pytest action/tests/ -v` returns exit 0
- RED phase behavior: Skipped because AC1–AC3 not yet implemented

---

## Test File Summary

| File | Action | Tests |
|------|--------|-------|
| `action/tests/test_9_9_python_hardening.py` | **Created (RED phase)** | 5 tests, all `@pytest.mark.skip` |

### Fixture Needs

- No new fixtures required
- AC3 requires deletion of `action/tests/fixtures/expected_output/data.json`
- AC3 requires deletion of `action/tests/fixtures/expected_output/` if empty after

---

## TDD Red Phase Compliance

- [x] All 5 tests use `@pytest.mark.skip` (documented failing tests)
- [x] All tests assert EXPECTED behavior (not placeholder assertions)
- [x] All tests marked with `expected_to_fail` semantics
- [x] `pytest` collects all 5 tests as SKIPPED (verified: `5 skipped in 0.02s`)
- [x] No regressions in existing 140 tests (verified: `140 passed, 5 skipped in 0.50s`)
- [x] TC-6 verified as proper RED: current code returns exit 0 + empty stderr (bug confirmed)

---

## Green Phase Instructions

After implementing Story 9.9 (Tasks 1–4 in the story file):

### Task 1: Implement empty-username validation in `update_readme.py`
Insert after `args = parse_args()` in `main()`:
```python
if not args.username or not args.username.strip():
    print("Error: --username cannot be empty", file=sys.stderr)
    sys.exit(1)
```

### Task 2: Add TC-6 to `test_update_readme.py`
Move `test_tc6_empty_username_exits_nonzero` from `test_9_9_python_hardening.py`
into `test_update_readme.py` (or keep it in the hardening file — both are valid).

### Task 3: Delete `action/tests/fixtures/expected_output/data.json`
Also remove the directory if empty.

### Task 4: Remove `@pytest.mark.skip` decorators from `test_9_9_python_hardening.py`
Then run: `python3 -m pytest action/tests/ -v`
Expected: all tests pass (0 failed, 0 errors)

---

## Next Steps

1. **Implement** the story using the story file:
   `_bmad-output/implementation-artifacts/9-9-python-script-hardening-update-readme-and-aggregate-improvements.md`
2. **Remove** `@pytest.mark.skip` from all 5 tests in `test_9_9_python_hardening.py`
3. **Run** `python3 -m pytest action/tests/ -v` → verify all pass (green phase)
4. **Commit** passing tests with story completion notes

---

## Risk Notes

- **R-008 (Low):** `str.strip()` covers all standard ASCII/Unicode whitespace for typical usernames — no mitigation needed
- **R-010 (Low):** Fixture removal is the correct choice per story Dev Notes (EXPECTED_DAYS constant in `test_aggregate.py` is semantically identical to fixture's `days` field); fixture removal avoids brittle placeholder logic
