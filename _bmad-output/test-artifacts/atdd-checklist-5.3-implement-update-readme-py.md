---
stepsCompleted:
  - step-01-preflight-and-context
  - step-02-generation-mode
  - step-03-test-strategy
  - step-04-generate-tests
  - step-04c-aggregate
  - step-05-validate-and-complete
lastStep: step-05-validate-and-complete
lastSaved: '2026-04-11'
story_id: 5.3-implement-update-readme-py
tdd_phase: RED
---

# ATDD Checklist: Story 5.3 — Implement update_readme.py

**Date:** 2026-04-11
**Story:** 5.3 — Implement update_readme.py
**TDD Phase:** RED (failing tests generated, awaiting implementation)

---

## Step 1: Preflight & Context

### Stack Detection

- **Detected Stack:** `backend`
- **Indicators:** `Cargo.toml` (Rust binary), Python action scripts (`action/update_readme.py`), no `playwright.config.*` or `cypress.config.*`
- **Test Framework:** pytest (stdlib-compatible; dev dependency only)

### Prerequisites Check

- [x] Story has clear acceptance criteria (AC1, AC2, AC3)
- [x] Story file loaded: `_bmad-output/implementation-artifacts/5-3-implement-update-readme-py.md`
- [x] Test directory exists: `action/tests/`
- [x] `action/tests/__init__.py` present
- [x] No pip dependencies required (stdlib only: `re`, `pathlib`, `argparse`, `sys`)
- [x] Test design loaded: `_bmad-output/test-artifacts/test-design-epic-5.md`

### Loaded Inputs

- Story: `_bmad-output/implementation-artifacts/5-3-implement-update-readme-py.md`
- Test Design: `_bmad-output/test-artifacts/test-design-epic-5.md`
- Stub: `action/update_readme.py` (pass-body only — not implemented)
- Config: `_bmad/tea/config.yaml`

---

## Step 2: Generation Mode

**Mode selected: AI Generation** (backend stack — no browser recording needed)

Rationale: acceptance criteria are explicit CLI/file operations; no UI or API endpoints involved.

---

## Step 3: Test Strategy

### Acceptance Criteria → Test Scenarios

| AC | Scenario | Level | Priority | Test ID |
|---|---|---|---|---|
| AC1 | Markers present → SVG img + dashboard link injected | Unit | P1 | TC-1 |
| AC1 | Correct `raw.githubusercontent.com` URL format in output | Unit | P1 | TC-2 |
| AC2 | Markers absent → non-zero exit + error containing "vibestats markers" | Unit | P0 | TC-3 |
| AC3 | Identical content → exit 0, file NOT written (idempotent) | Unit | P0 | TC-4 |
| AC1 | Content changed → file IS written to disk | Unit | P1 | TC-5 |

### Risk Linkage

| Test | Risk | Description |
|---|---|---|
| TC-3 | R-007 | `update_readme.py` must NOT silently replace README when markers are absent |
| TC-4 | R-004 | Empty commit prevention — script must not write file when content unchanged |

### Red Phase Requirements

All tests use `@pytest.mark.skip` — they assert EXPECTED behavior but will fail until implementation is complete.

---

## Step 4: TDD Red Phase — Generated Tests

### Test File

**Path:** `action/tests/test_update_readme.py`

**Tests generated (all `@pytest.mark.skip`):**

| ID | Test Name | Priority | AC | Risk |
|---|---|---|---|---|
| TC-1 | `test_tc1_markers_present_content_replaced` | P0 | AC1 | — |
| TC-2 | `test_tc2_correct_raw_githubusercontent_url` | P1 | AC1 | — |
| TC-3 | `test_tc3_markers_absent_nonzero_exit` | P0 | AC2 | R-007 |
| TC-4 | `test_tc4_identical_content_no_op` | P0 | AC3 | R-004 |
| TC-5 | `test_tc5_content_changed_file_is_written` | P1 | AC1 | — |

**Total:** 5 tests (3 P0, 2 P1) — all skipped (TDD RED phase)

### Fixture Needs

- `tmp_path` pytest built-in fixture (no additional fixtures required)
- `README_WITH_MARKERS`: inline string constant in test file
- `README_WITHOUT_MARKERS`: inline string constant in test file

No separate fixture files needed — all inline.

### TDD Red Phase Validation

- [x] All tests use `@pytest.mark.skip` decorator
- [x] All tests assert EXPECTED behavior (not placeholders)
- [x] All tests invoke `update_readme.py` via `subprocess.run` (as specified in story Task 2)
- [x] Uses `tmp_path` fixture for temp README files (as specified in story Task 2)
- [x] No non-stdlib imports in test file (`subprocess`, `sys`, `pathlib`, `pytest`)
- [x] No passing tests (strict red phase compliance)

---

## Step 5: Validation

### Checklist

- [x] Test file written to disk: `action/tests/test_update_readme.py`
- [x] All 5 ACs from story Task 2 covered (TC-1 through TC-5)
- [x] All tests designed to fail before implementation (stub has `pass` body)
- [x] Tests do NOT create fixture files (no orphaned resources)
- [x] ATDD checklist saved to `_bmad-output/test-artifacts/`

### Acceptance Criteria Coverage

| AC | Covered By | Status |
|---|---|---|
| AC1: markers present → SVG img + link injected | TC-1, TC-2, TC-5 | Covered |
| AC2: markers absent → non-zero exit + clear error | TC-3 | Covered |
| AC3: identical content → no write, exit 0 | TC-4 | Covered |

### Risks Mitigated

| Risk | Mitigation Test | Status |
|---|---|---|
| R-004 (empty commit) | TC-4 | Covered |
| R-007 (silent README replacement) | TC-3 | Covered |

---

## Completion Summary

### Generated Files

- `action/tests/test_update_readme.py` — 5 failing acceptance tests (RED phase)
- `_bmad-output/test-artifacts/atdd-checklist-5.3-implement-update-readme-py.md` — this document

### Next Steps (TDD Green Phase)

After implementing `action/update_readme.py`:

1. Remove `@pytest.mark.skip` from each test in `action/tests/test_update_readme.py`
2. Run: `python -m pytest action/tests/test_update_readme.py -v`
3. Verify all 5 tests PASS (green phase)
4. If any test fails — fix the implementation, not the test
5. Commit passing tests

### Implementation Guidance

Story Task 1 specifies the full implementation in `action/update_readme.py`:
- CLI args: `--username` (required) and `--readme-path` (default: `README.md`)
- Marker detection via `re.compile(r"(<!-- vibestats-start -->)(.*?)(<!-- vibestats-end -->)", re.DOTALL)`
- Idempotency: compare new block to existing with `.strip()` normalization before writing
- stdlib only: `re`, `pathlib`, `argparse`, `sys`
- Exit non-zero on all error paths (fail-loudly contract)

### Recommended Next Workflow

Run `bmad-dev-story` to implement `action/update_readme.py` (Story 5.3, Task 1), then remove `@pytest.mark.skip` decorators and verify green phase.
