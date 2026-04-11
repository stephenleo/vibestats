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
storyId: 5.5-implement-aggregate-yml
tddPhase: RED
---

# ATDD Checklist: Story 5.5 — Implement aggregate.yml

**Date:** 2026-04-12
**Story:** 5.5 — Implement aggregate.yml (user vibestats-data workflow template)
**GH Issue:** #30
**TDD Phase:** RED (all tests skipped — failing until implementation complete)

---

## Step 1: Preflight & Context

### Stack Detection

- **Detected stack:** `backend`
  - Indicators: `Cargo.toml` (Rust binary) + Python scripts (`action/aggregate.py`, `action/generate_svg.py`, `action/update_readme.py`) + `pyproject.toml`-style layout
  - No frontend indicators (`playwright.config.*`, `package.json` with React/Vue etc. absent)
  - No E2E tests needed — this story creates a YAML file + a schema-level Python test
  - Stack consistent with Stories 5.1, 5.2, 5.3 (all `backend`)

### Prerequisites

- [x] Story 5.5 has clear acceptance criteria (3 ACs + 4 explicit test cases TC-1 through TC-4)
- [x] Test framework: `action/tests/__init__.py` exists (pytest discovery)
- [x] `action/tests/` directory exists with established pattern (`test_aggregate.py`, `test_update_readme.py`)
- [x] TEA config loaded from `_bmad/tea/config.yaml`
- [x] `.github/workflows/` directory exists at repo root (empty — `aggregate.yml` not yet created)

### Story Acceptance Criteria Loaded

| AC | Description |
|---|---|
| AC1 | Calls `uses: stephenleo/vibestats@v1` with `token: ${{ secrets.VIBESTATS_TOKEN }}` and `profile-repo: username/username` |
| AC2 | Triggers include both `schedule: cron` (daily) and `workflow_dispatch` (manual) — FR25, FR26 |
| AC3 | Total Actions consumption ≤60 min/month (daily cron only, no per-push triggers) — NFR5 |

### Config Flags

- `test_stack_type: auto` → detected `backend`
- `tea_use_playwright_utils: true` → API-only profile (no browser tests for YAML schema validation)
- `tea_use_pactjs_utils: false`
- `tea_pact_mcp: none`
- `tea_execution_mode: auto` → resolved to `sequential` (no subagent/agent-team capability)

---

## Step 2: Generation Mode

**Mode selected: AI Generation**

- Rationale: Backend stack; acceptance criteria are explicit (4 test cases named in story); YAML schema validation is a standard pattern; no browser recording needed.
- `tea_browser_automation: auto` — not applicable for backend/schema tests.

---

## Step 3: Test Strategy

### Acceptance Criteria → Test Scenarios

| AC | Scenario | Test Level | Priority |
|---|---|---|---|
| AC2 + AC3 + R-005 | Parse `aggregate.yml` and assert ONLY `schedule` + `workflow_dispatch` triggers present; no `push`, `pull_request`, `release`, or wildcard triggers | Unit (schema) | P0 |
| AC2 + FR26 | Assert `workflow_dispatch` trigger is present | Unit (schema) | P1 |
| AC1 | Assert step uses `stephenleo/vibestats@v1` | Unit (schema) | P1 |
| AC1 + FR10 | Assert `with.token` references `secrets.VIBESTATS_TOKEN` | Unit (schema) | P1 |

### Test Levels Selected

- **Unit (schema-level):** Parse YAML file and assert structural properties
  - Appropriate for: validating a static configuration file before it exists
  - No integration layer needed — the file is pure YAML with no runtime behavior to test at this layer
  - No E2E, no API, no component tests needed

### Priority Summary

| Priority | Count | Rationale |
|---|---|---|
| P0 | 1 | R-005 (Score 6): per-push trigger would exhaust free-tier minutes; critical regression guard |
| P1 | 3 | Direct AC coverage; low risk but required for correctness |
| P2 | 0 | N/A |
| P3 | 0 | N/A |

### TDD Red Phase Confirmation

All 4 tests are designed to **fail before implementation** (TDD red phase):
- `aggregate.yml` does NOT exist at `.github/workflows/aggregate.yml`
- When tests run, `_load_workflow()` will raise `FileNotFoundError` (or tests are skipped via `pytest.mark.skip`)
- Tests are marked `@pytest.mark.skip(reason="TDD RED PHASE — aggregate.yml not yet created")`
- Once `aggregate.yml` is created, remove `@pytest.mark.skip` to enter green phase

---

## Step 4: Generate Tests

### Execution Mode

- **Resolved mode:** `sequential`
- **Rationale:** `tea_execution_mode: auto` + no subagent/agent-team capability → falls back to sequential

### Tests Generated

**File:** `action/tests/test_aggregate_yml.py`

| Test ID | Function | Priority | TDD Status |
|---|---|---|---|
| TC-1 | `test_tc1_only_schedule_and_workflow_dispatch_triggers` | P0 | SKIP (red phase) |
| TC-2 | `test_tc2_workflow_dispatch_trigger_present` | P1 | SKIP (red phase) |
| TC-3 | `test_tc3_step_uses_vibestats_v1_action` | P1 | SKIP (red phase) |
| TC-4 | `test_tc4_token_input_references_vibestats_token_secret` | P1 | SKIP (red phase) |

### Fixture Needs

- None required — tests parse a static YAML file using `pathlib.Path`
- PyYAML (`import yaml`) required — available in GitHub Actions `ubuntu-latest` environment; for local dev: `pip install pyyaml`

### TDD Red Phase Verification

```
$ python3 -m pytest action/tests/test_aggregate_yml.py -v
collected 4 items

action/tests/test_aggregate_yml.py::test_tc1_only_schedule_and_workflow_dispatch_triggers SKIPPED [25%]
action/tests/test_aggregate_yml.py::test_tc2_workflow_dispatch_trigger_present SKIPPED [50%]
action/tests/test_aggregate_yml.py::test_tc3_step_uses_vibestats_v1_action SKIPPED [75%]
action/tests/test_aggregate_yml.py::test_tc4_token_input_references_vibestats_token_secret SKIPPED [100%]

4 skipped in 0.20s
```

All 4 tests collected and skipped — TDD red phase confirmed.

---

## Step 4C: Aggregation

**API Tests (sequential mode):**
- 4 schema-level unit tests generated (no API endpoints — YAML schema validation only)
- All tests use `@pytest.mark.skip` (TDD red phase compliant)
- All tests assert expected behavior (not placeholder assertions)

**E2E Tests:**
- None — backend stack, no browser-based testing needed for YAML schema validation

**TDD Red Phase Validation:** ✅ PASS
- All 4 tests use `@pytest.mark.skip()`
- All assertions verify expected YAML structure
- No placeholder assertions (`assert True`)
- `expected_to_fail: true` for all tests

**Generated Files:**
- `action/tests/test_aggregate_yml.py` ← CREATED (4 failing/skipped tests)

---

## Step 5: Validate & Complete

### Checklist Validation

- [x] Prerequisites satisfied (story approved, test framework configured, clear ACs)
- [x] Test file created at correct path: `action/tests/test_aggregate_yml.py`
- [x] Test path matches story spec: `action/tests/test_aggregate_yml.py`
- [x] 4 test cases created matching TC-1 through TC-4 in story
- [x] All tests use `@pytest.mark.skip()` (TDD red phase)
- [x] No placeholder assertions
- [x] P0 test covers R-005 (accidental per-push trigger)
- [x] Test locates `aggregate.yml` via `REPO_ROOT / ".github" / "workflows" / "aggregate.yml"` (matches story dev notes path)
- [x] Uses PyYAML (`import yaml`) as specified by story
- [x] Naming convention consistent with existing tests (`test_update_readme.py`, `test_aggregate.py`)
- [x] No CLI sessions to clean up (backend tests only)
- [x] Temp artifacts: none created

### Completion Summary

**Test files created:**

| File | Tests | Phase |
|---|---|---|
| `action/tests/test_aggregate_yml.py` | 4 (all skipped) | RED |

**Checklist output:** `_bmad-output/test-artifacts/atdd-checklist-5.5-implement-aggregate-yml.md`

**Key risks addressed:**
- R-005 (Score 6): TC-1 (P0) guards against accidental per-push trigger being added

**Assumptions:**
- PyYAML is available in the GitHub Actions `ubuntu-latest` runner environment
- `aggregate.yml` will be placed at `.github/workflows/aggregate.yml` (repo root)
- `profile-repo` input is not tested (AC1 focuses on `token`; `profile-repo` uses `github.repository_owner` context which is valid for any user)

**Next recommended workflow:**
1. Implement `aggregate.yml` (create `.github/workflows/aggregate.yml`)
2. Remove `@pytest.mark.skip` from all 4 tests
3. Run `python3 -m pytest action/tests/test_aggregate_yml.py -v` → verify 4 PASS (green phase)
4. Commit passing tests with implementation

---

## Summary Statistics

```
TDD Phase: RED
Total Tests: 4 (all with @pytest.mark.skip)
  - Schema/Unit Tests: 4 (RED)
  - E2E Tests: 0 (N/A — backend stack)
Fixtures Created: 0
All tests will FAIL/SKIP until aggregate.yml is created
Execution Mode: SEQUENTIAL
```
