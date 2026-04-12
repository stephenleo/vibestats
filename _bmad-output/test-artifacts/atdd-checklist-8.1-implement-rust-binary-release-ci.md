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
storyId: 8.1-implement-rust-binary-release-ci
tddPhase: RED
---

# ATDD Checklist: Story 8.1 — Implement Rust binary release CI

**Date:** 2026-04-12
**Story:** 8.1 — Implement Rust binary release CI
**GH Issue:** #39
**TDD Phase:** RED (all tests skipped — failing until implementation complete)

---

## Step 1: Preflight & Context

### Stack Detection

- **Detected stack:** `backend`
  - Indicators: `Cargo.toml` (Rust binary) + Python scripts in `action/` + `pyproject.toml`-style layout
  - No frontend indicators (`playwright.config.*`, `package.json` with React/Vue etc. absent)
  - No E2E tests needed — this story creates a YAML workflow file; tests are YAML schema validation
  - Stack consistent with prior Epic 5 and Epic 8 stories (all `backend`)

### Prerequisites

- [x] Story 8.1 has clear acceptance criteria (3 ACs + P0/P1/P2 test assertions from test-design-epic-8.md)
- [x] Test framework: `action/tests/__init__.py` exists (pytest discovery)
- [x] `action/tests/` directory exists with established pattern (`test_action_yml.py`, `test_aggregate_yml.py`)
- [x] TEA config loaded from `_bmad/tea/config.yaml`
- [x] `.github/workflows/` directory exists at repo root (contains `aggregate.yml`; `release.yml` not yet created)
- [x] Epic 8 test design loaded from `_bmad-output/test-artifacts/test-design-epic-8.md`

### Story Acceptance Criteria Loaded

| AC | Description |
|---|---|
| AC1 | Given a `v*` tag push, `release.yml` runs a matrix build for `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu` using `cross` (FR41) |
| AC2 | Given all three targets compile, each binary is archived as `vibestats-<target>.tar.gz` and attached to the GitHub Release |
| AC3 | Given any compilation target fails, the workflow exits non-zero and no partial release is published |

### Config Flags

- `test_stack_type: auto` → detected `backend`
- `tea_use_playwright_utils: true` → API-only profile (no browser tests for YAML schema validation)
- `tea_use_pactjs_utils: false`
- `tea_pact_mcp: none`
- `tea_execution_mode: auto` → resolved to `sequential` (no subagent/agent-team capability)

---

## Step 2: Generation Mode

**Mode selected: AI Generation**

- Rationale: Backend stack; acceptance criteria are explicit with P0/P1/P2 test scenarios in test-design-epic-8.md; YAML schema validation is a standard pattern; no browser recording needed.
- `tea_browser_automation: auto` — not applicable for backend/schema tests.

---

## Step 3: Test Strategy

### Acceptance Criteria → Test Scenarios

| AC / Risk | Scenario | Test Level | Priority |
|---|---|---|---|
| AC1, R-002 | Parse `release.yml` matrix and assert EXACTLY the three required targets: `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu` | Unit (schema) | P0 |
| AC3, R-001 | Assert `strategy.fail-fast: true` in matrix build job | Unit (schema) | P0 |
| AC2, R-007 | Assert archive naming uses `vibestats-${{ matrix.target }}.tar.gz` template | Unit (schema) | P0 |
| AC2, R-007 | Assert archive command references `target/<target>/release/` path | Unit (schema) | P0 |
| R-006 | Assert all `uses:` action references pinned to version tags (not `@main`/`@master`) | Unit (schema) | P1 |
| AC1, R-001 | Assert workflow trigger is ONLY `push: tags: [v*]` — no branch/PR triggers | Unit (schema) | P1 |
| AC1, R-002 | Assert `cross` used for Linux target cross-compilation | Unit (schema) | P1 |
| R-001 | Assert release job declares `permissions: contents: write` | Unit (schema) | P1 |
| R-007 | Assert release step uses `${{ github.ref_name }}` — no hardcoded version | Unit (schema) | P2 |
| R-001 | Assert `upload-artifact` step present for matrix builds | Unit (schema) | P2 |
| R-001 | Assert `download-artifact` step present in release job | Unit (schema) | P2 |
| AC3, R-001 | Assert release job has `needs: build` dependency | Unit (schema) | P2 |

### Test Levels Selected

- **Unit (schema-level):** Parse YAML file and assert structural properties
  - Appropriate for: validating a static GitHub Actions workflow file before it exists
  - No integration layer needed — the file is pure YAML with no runtime behavior testable at this layer
  - No E2E, no API, no component tests needed
  - P3 (end-to-end: push tag to fork) is manual; not automated here

### Priority Summary

| Priority | Count | Rationale |
|---|---|---|
| P0 | 4 | Critical path: matrix targets, fail-fast, archive naming — directly block Epic 6 install.sh if wrong |
| P1 | 4 | High-priority: action pinning, trigger safety, cross usage, permissions |
| P2 | 4 | Supporting: ref_name usage, artifact upload/download, job dependency |
| P3 | 0 | Manual E2E (tag push to fork) — not automated in this ATDD cycle |

### TDD Red Phase Confirmation

All 17 tests are designed to **fail before implementation** (TDD red phase):
- `.github/workflows/release.yml` does NOT exist yet
- When tests run (without `@pytest.mark.skip`), they would raise `FileNotFoundError` + assertion errors
- Tests are marked `@pytest.mark.skip(reason="TDD red phase — release.yml not yet implemented (Story 8.1)")`
- Once `release.yml` is created, remove `@pytest.mark.skip` to enter green phase

---

## Step 4: Generate Tests

### Execution Mode

- **Resolved mode:** `sequential`
- **Rationale:** `tea_execution_mode: auto` + no subagent/agent-team capability → falls back to sequential

### Tests Generated

**File:** `action/tests/test_release_yml.py`

| Test ID | Function | Priority | TDD Status |
|---|---|---|---|
| 8.1-SCHEMA-000 | `test_prereq_release_yml_exists` | P0 | SKIP (red phase) |
| 8.1-SCHEMA-001 | `test_prereq_release_yml_not_empty` | P0 | SKIP (red phase) |
| 8.1-SCHEMA-002 | `test_prereq_release_yml_parses_as_valid_yaml` | P0 | SKIP (red phase) |
| 8.1-SCHEMA-010 | `test_tc1_trigger_is_tag_push_only` | P1 | SKIP (red phase) |
| 8.1-SCHEMA-011 | `test_tc1_tag_pattern_matches_v_wildcard` | P1 | SKIP (red phase) |
| 8.1-SCHEMA-020 | `test_tc2_matrix_targets_exact_set` | P0 | SKIP (red phase) |
| 8.1-SCHEMA-030 | `test_tc3_matrix_fail_fast_true` | P0 | SKIP (red phase) |
| 8.1-SCHEMA-040 | `test_tc4_archive_name_uses_matrix_target_variable` | P0 | SKIP (red phase) |
| 8.1-SCHEMA-041 | `test_tc4_archive_contains_vibestats_binary` | P0 | SKIP (red phase) |
| 8.1-SCHEMA-050 | `test_tc5_no_action_uses_main_or_master_tag` | P1 | SKIP (red phase) |
| 8.1-SCHEMA-051 | `test_tc5_required_actions_are_pinned` | P1 | SKIP (red phase) |
| 8.1-SCHEMA-060 | `test_tc6_cross_used_for_linux_target` | P1 | SKIP (red phase) |
| 8.1-SCHEMA-070 | `test_tc7_release_step_uses_github_ref_name` | P2 | SKIP (red phase) |
| 8.1-SCHEMA-080 | `test_tc8_upload_artifact_step_present` | P2 | SKIP (red phase) |
| 8.1-SCHEMA-081 | `test_tc8_download_artifact_step_present` | P2 | SKIP (red phase) |
| 8.1-SCHEMA-082 | `test_tc8_release_job_needs_build_job` | P2 | SKIP (red phase) |
| 8.1-SCHEMA-090 | `test_tc9_release_job_has_contents_write_permission` | P1 | SKIP (red phase) |

### Acceptance Criteria Coverage

| AC | Tests Covering |
|---|---|
| AC1 (matrix build, targets, cross) | SCHEMA-010, SCHEMA-011, SCHEMA-020, SCHEMA-060 |
| AC2 (archive naming, attach to release) | SCHEMA-040, SCHEMA-041, SCHEMA-080, SCHEMA-081 |
| AC3 (fail-fast, no partial release) | SCHEMA-030, SCHEMA-082 |

### Fixture Needs

- None required — tests parse a static YAML file using `pathlib.Path`
- PyYAML (`import yaml`) required — available in GitHub Actions `ubuntu-latest` environment; for local dev: `pip install pyyaml`

### TDD Red Phase Verification

```
$ python3 -m pytest action/tests/test_release_yml.py -v
collected 17 items

action/tests/test_release_yml.py::test_prereq_release_yml_exists SKIPPED [  5%]
action/tests/test_release_yml.py::test_prereq_release_yml_not_empty SKIPPED [ 11%]
action/tests/test_release_yml.py::test_prereq_release_yml_parses_as_valid_yaml SKIPPED [ 17%]
action/tests/test_release_yml.py::test_tc1_trigger_is_tag_push_only SKIPPED [ 23%]
action/tests/test_release_yml.py::test_tc1_tag_pattern_matches_v_wildcard SKIPPED [ 29%]
action/tests/test_release_yml.py::test_tc2_matrix_targets_exact_set SKIPPED [ 35%]
action/tests/test_release_yml.py::test_tc3_matrix_fail_fast_true SKIPPED [ 41%]
action/tests/test_release_yml.py::test_tc4_archive_name_uses_matrix_target_variable SKIPPED [ 47%]
action/tests/test_release_yml.py::test_tc4_archive_contains_vibestats_binary SKIPPED [ 52%]
action/tests/test_release_yml.py::test_tc5_no_action_uses_main_or_master_tag SKIPPED [ 58%]
action/tests/test_release_yml.py::test_tc5_required_actions_are_pinned SKIPPED [ 64%]
action/tests/test_release_yml.py::test_tc6_cross_used_for_linux_target SKIPPED [ 70%]
action/tests/test_release_yml.py::test_tc7_release_step_uses_github_ref_name SKIPPED [ 76%]
action/tests/test_release_yml.py::test_tc8_upload_artifact_step_present SKIPPED [ 82%]
action/tests/test_release_yml.py::test_tc8_download_artifact_step_present SKIPPED [ 88%]
action/tests/test_release_yml.py::test_tc8_release_job_needs_build_job SKIPPED [ 94%]
action/tests/test_release_yml.py::test_tc9_release_job_has_contents_write_permission SKIPPED [100%]

17 skipped in 0.01s
```

All 17 tests collected and skipped — TDD red phase confirmed.

---

## Step 4C: Aggregation

**Schema/Unit Tests (sequential mode):**
- 17 schema-level unit tests generated (no API endpoints — YAML schema validation only)
- All tests use `@pytest.mark.skip` (TDD red phase compliant)
- All tests assert expected behavior (not placeholder assertions)

**E2E Tests:**
- None — backend stack, no browser-based testing needed for YAML schema validation

**TDD Red Phase Validation:** PASS
- All 17 tests use `@pytest.mark.skip()`
- All assertions verify expected YAML structure and workflow properties
- No placeholder assertions (`assert True`)
- All tests expected to fail when `release.yml` is absent

**Generated Files:**
- `action/tests/test_release_yml.py` — CREATED (17 failing/skipped tests)

---

## Step 5: Validate & Complete

### Checklist Validation

- [x] Prerequisites satisfied (story approved, test framework configured, clear ACs)
- [x] Test file created at correct path: `action/tests/test_release_yml.py`
- [x] Naming convention consistent with existing tests (`test_action_yml.py`, `test_aggregate_yml.py`)
- [x] 17 test cases generated covering P0/P1/P2 scenarios from test-design-epic-8.md
- [x] All tests use `@pytest.mark.skip()` (TDD red phase)
- [x] No placeholder assertions
- [x] P0 tests cover R-001, R-002, R-007 (highest risk items for Story 8.1)
- [x] Test locates `release.yml` via `REPO_ROOT / ".github" / "workflows" / "release.yml"`
- [x] Uses PyYAML (`import yaml`) consistent with existing schema tests
- [x] All 3 Acceptance Criteria covered by at least one test
- [x] No CLI sessions to clean up (backend tests only)
- [x] Temp artifacts: none created

### Completion Summary

**Test files created:**

| File | Tests | Phase |
|---|---|---|
| `action/tests/test_release_yml.py` | 17 (all skipped) | RED |

**Checklist output:** `_bmad-output/test-artifacts/atdd-checklist-8.1-implement-rust-binary-release-ci.md`

**Key risks addressed:**

| Risk | Priority | Test |
|---|---|---|
| R-001 (partial release on platform failure) | P0/P1/P2 | SCHEMA-030, SCHEMA-082, SCHEMA-080, SCHEMA-081 |
| R-002 (wrong matrix targets / missing cross) | P0/P1 | SCHEMA-020, SCHEMA-060 |
| R-006 (mutable action tags break pipeline) | P1 | SCHEMA-050, SCHEMA-051 |
| R-007 (wrong archive naming breaks install.sh) | P0/P2 | SCHEMA-040, SCHEMA-041, SCHEMA-070 |

**Assumptions:**
- PyYAML is available in the GitHub Actions `ubuntu-latest` runner environment
- `release.yml` will be placed at `.github/workflows/release.yml` (repo root)
- Matrix strategy uses the `include` format as specified in story dev notes
- `cross` is invoked via `cargo install cross` + `cross build` command (not the `cross-rs/cross` GitHub Action) — test handles both patterns

**Next recommended workflow:**
1. Implement `release.yml` per story dev notes (Task 1)
2. Verify `Cargo.toml` has correct name/version (Task 2)
3. Remove `@pytest.mark.skip` from all 17 tests
4. Run `python3 -m pytest action/tests/test_release_yml.py -v` → verify all 17 PASS (green phase)
5. Commit passing tests with implementation
6. Run `bmad-testarch-ci` to wire schema tests into the PR gate pipeline

---

## Summary Statistics

```
TDD Phase: RED
Total Tests: 17 (all with @pytest.mark.skip)
  - Schema/Unit Tests: 17 (RED)
  - E2E Tests: 0 (N/A — backend stack)
  - P0: 5 tests
  - P1: 6 tests
  - P2: 4 tests
  - P3: 0 (manual only)
Fixtures Created: 0
All tests will FAIL/SKIP until release.yml is created
Execution Mode: SEQUENTIAL
```
