---
stepsCompleted: ['step-01-preflight-and-context', 'step-02-generation-mode', 'step-03-test-strategy', 'step-04-generate-tests', 'step-04c-aggregate', 'step-05-validate-and-complete']
lastStep: 'step-05-validate-and-complete'
lastSaved: '2026-04-13'
workflowType: 'testarch-atdd'
inputDocuments:
  - '_bmad-output/implementation-artifacts/9-7-aggregate-yml-add-concurrency-group-to-prevent-push-conflicts.md'
  - '.github/workflows/aggregate.yml'
  - 'action/tests/test_aggregate_yml.py'
  - '_bmad/tea/config.yaml'
---

# ATDD Checklist — Story 9.7: aggregate.yml — Add concurrency group to prevent concurrent push conflicts

**Date:** 2026-04-13
**Author:** Leo
**Primary Test Level:** Schema/Static analysis (Python/pytest)
**Stack:** Backend (GitHub Actions YAML, Python/pytest)
**TDD Phase:** RED — TC-5 fails until `concurrency:` block is added to `aggregate.yml`.
**Test File:** `action/tests/test_aggregate_yml.py`

---

## Story Summary

As a vibestats user running the pipeline on multiple machines that share a profile repo,
I want concurrent runs of the aggregate workflow to be serialized,
so that when two machines push data at the same time, the second run doesn't fail due to the first run having already advanced the remote branch.

---

## Step 1: Preflight & Context

**Stack Detection:** `backend`
- No `package.json`, no `playwright.config.ts` — this is a Python/pytest project
- Backend indicators: `action/tests/test_aggregate_yml.py` (pytest), `action/` Python scripts

**Prerequisites:**
- Story file: `_bmad-output/implementation-artifacts/9-7-aggregate-yml-add-concurrency-group-to-prevent-push-conflicts.md` — status `ready-for-dev`
- Acceptance criteria: clear (4 ACs defined)
- Test framework: Python pytest (`action/tests/`)
- All prerequisites satisfied

**Knowledge fragments loaded (core, backend):**
- `data-factories.md` (core)
- `component-tdd.md` (core)
- `test-quality.md` (core)
- `test-healing-patterns.md` (core)
- `test-levels-framework.md` (backend)
- `test-priorities-matrix.md` (backend)

---

## Step 2: Generation Mode

**Mode selected:** AI Generation
**Reason:** Backend stack — Python schema tests against a YAML file. Acceptance criteria are clear and well-specified. No browser or UI interaction needed.

---

## Step 3: Test Strategy

### Acceptance Criteria → Test Scenarios

| AC | Scenario | Test Level | Priority |
|----|----------|------------|----------|
| AC1 | `aggregate.yml` contains a `concurrency:` block at the workflow level | Schema/Unit | P1 |
| AC2 | `concurrency.group` equals `vibestats-${{ github.repository_owner }}` | Schema/Unit | P1 |
| AC2 | `concurrency.cancel-in-progress` is `False` (queue, not kill) | Schema/Unit | P1 |
| AC3 | New test TC-5 asserts presence of `concurrency:` key and its values | Schema/Unit | P1 |
| AC4 | Existing TC-1 through TC-4 continue to pass | Regression | P0 |

**Test level rationale:** This story's entire scope is a one-field YAML change. The correct test level is a schema/static analysis test (pytest loads and parses the YAML, asserts structural properties) — the same pattern used by TC-1 through TC-4 in the existing test file. No E2E, API, or integration tests are needed. E2E equivalence would require a live GitHub Actions environment which is out of scope.

**Red phase design:** TC-5 will FAIL until `aggregate.yml` contains the `concurrency:` block. This is intentional — the test is written for expected behavior that does not yet exist.

---

## Step 4: Generated Tests

### TC-5: Concurrency block — group and cancel-in-progress (Story 9.7)

**File:** `action/tests/test_aggregate_yml.py` (TC-5 appended)
**Test function:** `test_tc5_concurrency_block_present_with_correct_group_and_policy`
**Priority:** P1
**TDD Phase:** RED (fails before implementation)

**Assertions:**
1. `workflow["concurrency"]` is not `None` — block exists
2. `workflow["concurrency"]["group"] == "vibestats-${{ github.repository_owner }}"` — group string exact match
3. `workflow["concurrency"]["cancel-in-progress"] is False` — policy is queue-not-kill

**PyYAML notes:**
- `concurrency:` is an unambiguous YAML key — parses as the string `"concurrency"`, no quirks
- `cancel-in-progress: false` parses as Python `False` (boolean), not the string `"false"`
- The `${{ github.repository_owner }}` expression is a YAML string literal; exact match is asserted

**Red phase verification:**
```
$ cd action && python3 -m pytest tests/test_aggregate_yml.py::test_tc5_concurrency_block_present_with_correct_group_and_policy -v
FAILED — AssertionError: Missing 'concurrency:' block in aggregate.yml.
```
TC-5 fails as expected. Existing TC-1 through TC-4 all pass.

---

## Step 5: Validate & Complete

### Validation Checklist

- [x] Story has clear acceptance criteria (4 ACs)
- [x] Test file exists: `action/tests/test_aggregate_yml.py`
- [x] TC-5 added following existing test style (docstring, AC references, assertion messages)
- [x] TC-5 is designed to FAIL before implementation (TDD red phase)
- [x] TC-5 will PASS after implementation (when `concurrency:` block added)
- [x] AC1 covered: asserts `concurrency` key exists
- [x] AC2 covered: asserts group value and cancel-in-progress policy
- [x] AC3 covered: TC-5 is the new test referenced in AC3
- [x] AC4 covered: TC-1 through TC-4 verified still passing
- [x] Test ID follows convention (TC-5, P1)
- [x] Module docstring updated to reference Story 9.7 and TC-5
- [x] No temp artifacts or orphaned files
- [x] No E2E or browser tests generated (correct for backend schema test)

### Completion Summary

**Test file modified:** `action/tests/test_aggregate_yml.py`

**New test:** `test_tc5_concurrency_block_present_with_correct_group_and_policy` (TC-5, P1)

**TDD status:**
- RED: TC-5 FAILS (expected — `concurrency:` block not yet in `aggregate.yml`)
- GREEN: TC-5 will PASS after implementing Task 2 from the story

**Key risk / assumption:**
- PyYAML parses `cancel-in-progress: false` as Python `False` (not string). Test asserts `is False` (identity check). This is correct and consistent with PyYAML behavior.
- The `${{ github.repository_owner }}` expression is preserved as a literal string by PyYAML. Exact-match assertion is safe.

**Next recommended step:** Implementation — run `/bmad-dev-story` to add the `concurrency:` block to `.github/workflows/aggregate.yml`, then run `cd action && python3 -m pytest tests/test_aggregate_yml.py` to confirm all 5 tests pass (green phase).
