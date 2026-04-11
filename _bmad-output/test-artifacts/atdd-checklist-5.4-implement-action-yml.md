---
stepsCompleted:
  - step-01-preflight-and-context
  - step-02-generation-mode
  - step-03-test-strategy
  - step-04-generate-tests
  - step-04c-aggregate
  - step-05-validate-and-complete
lastStep: step-05-validate-and-complete
lastSaved: '2026-04-12'
storyId: 5.4-implement-action-yml
tddPhase: RED
---

# ATDD Checklist: Story 5.4 — Implement action.yml

**Date:** 2026-04-12
**Story:** 5.4 — Implement action.yml (composite community GitHub Action)
**GH Issue:** #29
**TDD Phase:** RED (all tests skipped — failing until implementation complete)

---

## Step 1: Preflight & Context

### Stack Detection

- **Detected stack:** `backend`
  - Indicators: `Cargo.toml` (Rust binary) + Python scripts (`action/aggregate.py`, etc.)
  - No frontend indicators (`playwright.config.*`, `package.json` with React/Vue absent)
  - No E2E tests needed — story is YAML schema/text-based validation only

### Prerequisites

- [x] Story 5.4 has clear acceptance criteria (3 ACs)
- [x] Test framework config: `action/tests/__init__.py` exists (pytest discovery)
- [x] `action.yml` stub exists at repo root
- [x] TEA config loaded from `_bmad/tea/config.yaml`
- [x] `tea_execution_mode: auto` → resolved to `sequential`

### Story Acceptance Criteria Loaded

| AC | Description |
|---|---|
| AC1 | `action.yml` declares type `composite`, inputs `token` and `profile-repo`, branding fields |
| AC2 | Step sequence: checkout vibestats-data → checkout profile-repo → setup-python → aggregate.py → generate_svg.py → update_readme.py → commit → push |
| AC3 | Any step failure exits non-zero; no partial outputs committed; no `continue-on-error: true` |

### Config Flags

- `test_stack_type: auto` → detected `backend`
- `tea_use_playwright_utils: true` → API-only profile (no browser tests for backend)
- `tea_use_pactjs_utils: false`
- `tea_execution_mode: auto` → resolved to `sequential`

---

## Step 2: Generation Mode

**Mode selected:** AI Generation

**Reason:** Backend YAML schema validation with clear acceptance criteria. No browser automation needed — action.yml is a configuration file that can be fully validated via Python text/YAML parsing.

---

## Step 3: Test Strategy

### Acceptance Criteria → Test Scenarios

| AC | Test Level | Priority | Test ID | Scenario |
|---|---|---|---|---|
| AC1 | Schema/Unit | P1 | 5.4-UNIT-001a | action.yml exists at repo root |
| AC1 | Schema/Unit | P1 | 5.4-UNIT-001b | action.yml is not empty |
| AC1 | Schema/Unit | P1 | 5.4-UNIT-001c | action.yml parses as valid YAML |
| AC1 | Schema/Unit | P1 | 5.4-UNIT-002 | runs.using == 'composite' |
| AC1, R-008 | Schema/Unit | P1 | 5.4-UNIT-003a | 'token' input declared in inputs |
| AC1, R-008 | Schema/Unit | P1 | 5.4-UNIT-003b | 'profile-repo' input declared in inputs |
| AC1, NFR17 | Schema/Unit | P1 | 5.4-UNIT-003c | 'token' input marked required: true |
| AC1, NFR17 | Schema/Unit | P1 | 5.4-UNIT-003d | 'profile-repo' input marked required: true |
| AC2, R-003 | Schema/Unit | P1 | 5.4-UNIT-004a | actions/checkout appears at least twice |
| AC2 | Schema/Unit | P1 | 5.4-UNIT-004b | actions/setup-python step present |
| AC2 | Schema/Unit | P1 | 5.4-UNIT-004c | aggregate.py referenced in steps |
| AC2 | Schema/Unit | P1 | 5.4-UNIT-004d | generate_svg.py referenced in steps |
| AC2 | Schema/Unit | P1 | 5.4-UNIT-004e | update_readme.py referenced in steps |
| AC2 | Schema/Unit | P1 | 5.4-UNIT-004f | git commit step present |
| AC2 | Schema/Unit | P1 | 5.4-UNIT-004g | git push step present |
| AC2 | Schema/Unit | P1 | 5.4-UNIT-004h | checkout precedes aggregate.py in step order |
| AC2 | Schema/Unit | P1 | 5.4-UNIT-004i | aggregate.py precedes generate_svg.py |
| AC2 | Schema/Unit | P1 | 5.4-UNIT-004j | generate_svg.py precedes update_readme.py |
| AC2 | Schema/Unit | P1 | 5.4-UNIT-004k | update_readme.py precedes git commit |
| AC2 | Schema/Unit | P1 | 5.4-UNIT-004l | git commit precedes git push |
| AC3, NFR13, R-003 | Schema/Unit | P0 | 5.4-UNIT-005 | No step uses continue-on-error: true |
| NFR composite req | Schema/Unit | P1 | 5.4-UNIT-006 | All run: steps declare shell: bash |
| NFR17, Story 8.3 | Schema/Unit | P1 | 5.4-UNIT-007a | branding.icon declared |
| NFR17, Story 8.3 | Schema/Unit | P1 | 5.4-UNIT-007b | branding.color declared |
| AC2 | Schema/Unit | P1 | 5.4-UNIT-008 | steps list is non-empty (≥8 steps) |

**No E2E tests**: Backend YAML schema validation only.
**No API contract tests**: action.yml is a configuration file, not an HTTP service.

### TDD Red Phase Confirmation

All tests are designed to **fail before implementation** because:
- `action.yml` currently contains only the stub `steps: []` with no inputs, branding, or step sequence
- All tests use `@pytest.mark.skip()` — intentional TDD red phase

---

## Step 4: Test Generation (Sequential Mode)

### Execution Mode

- `tea_execution_mode: auto` → resolved to `sequential`
- No subagents or agent teams launched
- Worker A (unit/schema tests) executed sequentially

### Worker A: Failing Schema/Unit Test Generation

**Output file:** `action/tests/test_action_yml.py`
**TDD phase:** RED
**Tests generated:** 25 (all skipped)

| Test ID | Method | Priority | AC Coverage |
|---|---|---|---|
| 5.4-UNIT-001a | test_tc1_action_yml_exists | P1 | AC1 |
| 5.4-UNIT-001b | test_tc1_action_yml_not_empty | P1 | AC1 |
| 5.4-UNIT-001c | test_tc1_action_yml_parses_as_valid_yaml | P1 | AC1 |
| 5.4-UNIT-002 | test_tc2_runs_using_composite | P1 | AC1 |
| 5.4-UNIT-003a | test_tc3_input_token_declared | P1 | AC1, R-008 |
| 5.4-UNIT-003b | test_tc3_input_profile_repo_declared | P1 | AC1, R-008 |
| 5.4-UNIT-003c | test_tc3_token_input_is_required | P1 | AC1, NFR17 |
| 5.4-UNIT-003d | test_tc3_profile_repo_input_is_required | P1 | AC1, NFR17 |
| 5.4-UNIT-004a | test_tc4_step_sequence_has_two_checkouts | P1 | AC2, R-003 |
| 5.4-UNIT-004b | test_tc4_step_sequence_has_setup_python | P1 | AC2 |
| 5.4-UNIT-004c | test_tc4_step_sequence_has_aggregate_py | P1 | AC2 |
| 5.4-UNIT-004d | test_tc4_step_sequence_has_generate_svg_py | P1 | AC2 |
| 5.4-UNIT-004e | test_tc4_step_sequence_has_update_readme_py | P1 | AC2 |
| 5.4-UNIT-004f | test_tc4_step_sequence_has_git_commit | P1 | AC2 |
| 5.4-UNIT-004g | test_tc4_step_sequence_has_git_push | P1 | AC2 |
| 5.4-UNIT-004h | test_tc4_checkout_precedes_aggregate | P1 | AC2 |
| 5.4-UNIT-004i | test_tc4_aggregate_precedes_generate_svg | P1 | AC2 |
| 5.4-UNIT-004j | test_tc4_generate_svg_precedes_update_readme | P1 | AC2 |
| 5.4-UNIT-004k | test_tc4_update_readme_precedes_git_commit | P1 | AC2 |
| 5.4-UNIT-004l | test_tc4_git_commit_precedes_git_push | P1 | AC2 |
| 5.4-UNIT-005 | test_tc5_no_continue_on_error | P0 | AC3, NFR13, R-003 |
| 5.4-UNIT-006 | test_tc6_all_run_steps_have_shell_bash | P1 | composite req |
| 5.4-UNIT-007a | test_tc7_branding_icon_declared | P1 | NFR17, Story 8.3 |
| 5.4-UNIT-007b | test_tc7_branding_color_declared | P1 | NFR17, Story 8.3 |
| 5.4-UNIT-008 | test_tc8_steps_list_is_non_empty | P1 | AC2 |

**No Worker B (E2E)**: Skipped — backend stack, no browser tests needed.

---

## Step 4C: Aggregation

### TDD Red Phase Validation

- [x] All tests use `@pytest.mark.skip()` — intentional TDD red phase
- [x] All tests assert EXPECTED behaviour (not placeholder assertions)
- [x] All tests marked as expected-to-fail (action.yml stub has no implementation)
- [x] No placeholder assertions (`assert True` style)

### Files Written to Disk

- [x] `action/tests/test_action_yml.py` — 25 failing schema/unit tests (all skipped)

### Summary Statistics

| Metric | Value |
|---|---|
| TDD Phase | RED |
| Total Tests | 25 (all skipped) |
| Unit/Schema Tests | 25 |
| E2E Tests | 0 (backend stack — not applicable) |
| Fixtures Created | 0 (no new fixtures needed — action.yml is the fixture) |
| Acceptance Criteria Covered | AC1, AC2, AC3 (100%) |
| Risk Mitigations | R-003, R-008 (P0/P1 tests ready) |

---

## Step 5: Validation & Completion

### Prerequisites Satisfied

- [x] Story 5.4 acceptance criteria all covered by tests
- [x] Test file created at `action/tests/test_action_yml.py`
- [x] All 25 tests use `@pytest.mark.skip()` — TDD red phase compliant
- [x] All tests assert expected behaviour (not placeholder assertions)
- [x] No orphaned browsers (backend test suite — no browser automation)
- [x] All temp artifacts in `_bmad-output/test-artifacts/` (this file)
- [x] Verified: `python3 -m pytest action/tests/test_action_yml.py -v` → 25 skipped, 0 errors

### Key Risks Mitigated by These Tests

| Risk | Test(s) | Status |
|---|---|---|
| R-003: Partial commit on step failure | 5.4-UNIT-005 (no continue-on-error) | Tests written (RED) |
| R-008: Missing token/profile-repo inputs | 5.4-UNIT-003a, 5.4-UNIT-003b | Tests written (RED) |

### Completion Summary

**Test files created:**
- `action/tests/test_action_yml.py` — 25 failing schema/unit tests (TDD RED)

**No new fixture files needed** — `action.yml` is the subject under test.

**Checklist output:** `_bmad-output/test-artifacts/atdd-checklist-5.4-implement-action-yml.md`

**Assumptions:**
1. `pytest` is available as a dev tool (not in action runtime)
2. `PyYAML` may or may not be installed; tests use dual-mode (PyYAML structural OR text-based fallback)
3. `action.yml` path resolved via `pathlib.Path(__file__).parent.parent.parent / "action.yml"`

---

## TDD RED Phase: Failing Tests Generated

All tests assert EXPECTED behaviour.
All tests will FAIL (be blocked by skip) until `action.yml` is fully implemented.
This is INTENTIONAL (TDD red phase).

## Next Steps (TDD Green Phase)

After implementing `action.yml` (Story 5.4):

1. Remove `@pytest.mark.skip(...)` from all test methods in `action/tests/test_action_yml.py`
2. Run tests: `python3 -m pytest action/tests/test_action_yml.py -v`
3. Verify tests PASS (green phase)
4. If any tests fail:
   - Fix `action.yml` implementation (feature bug) — not the tests
   - Or fix test only if the test itself has a logical error
5. Commit passing tests

## Implementation Guidance

Fields to implement in `action.yml`:
- `name: 'vibestats'` (already correct in stub)
- `description:` (already correct in stub)
- `branding.icon: 'activity'`
- `branding.color: 'orange'`
- `inputs.token` (required: true, with description)
- `inputs.profile-repo` (required: true, with description)
- `runs.using: 'composite'` (already correct in stub)
- `runs.steps:` — 8 steps in order:
  1. Checkout vibestats-data (`actions/checkout@v4`)
  2. Checkout profile-repo (`actions/checkout@v4` with `token:` and `path: _profile_repo`)
  3. Set up Python (`actions/setup-python@v5`)
  4. Run `aggregate.py` (shell: bash)
  5. Run `generate_svg.py` (shell: bash)
  6. Run `update_readme.py` (shell: bash)
  7. Commit outputs to profile-repo (shell: bash)
  8. Push to profile-repo with 3-retry loop (shell: bash)

See story file `_bmad-output/implementation-artifacts/5-4-implement-action-yml.md` for exact step content.
