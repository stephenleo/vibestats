---
stepsCompleted:
  - step-01-preflight-and-context
  - step-02-generation-mode
  - step-03-test-strategy
  - step-04-generate-tests
lastStep: step-04-generate-tests
lastSaved: '2026-04-12'
inputDocuments:
  - _bmad-output/implementation-artifacts/8-3-configure-github-actions-marketplace-publication.md
  - _bmad-output/test-artifacts/test-design-epic-8.md
  - action/tests/test_action_yml.py
  - action/tests/test_aggregate_yml.py
  - action.yml
  - CONTRIBUTING.md
  - _bmad/tea/config.yaml
---

# ATDD Checklist: Story 8.3 — Configure GitHub Actions Marketplace Publication

**Date:** 2026-04-12
**Story ID:** 8.3-configure-github-actions-marketplace-publication
**GH Issue:** #41
**TDD Phase:** RED (failing/skipped tests written before implementation)

---

## Step 1: Preflight & Context

### Stack Detection

- **Detected stack:** `backend` (Python)
- **Indicators found:** `pyproject.toml` / `action/tests/*.py`, no `playwright.config.*` or browser test files
- **Tea config:** `test_stack_type: auto` → resolved to `backend`
- **`tea_use_playwright_utils`:** true (ignored for backend stack)
- **`tea_use_pactjs_utils`:** false
- **`tea_pact_mcp`:** none

### Prerequisites Verified

- [x] Story 8.3 approved with clear acceptance criteria
- [x] Python test framework configured (`action/tests/` with `pytest`)
- [x] `action.yml` exists at repo root (Story 5.4 — DONE)
- [x] `CONTRIBUTING.md` exists at repo root (versioning section NOT yet present — red phase target)

### Story Acceptance Criteria Extracted

1. **AC #1:** `action.yml` includes `name`, `description`, `branding` (icon + colour), and `runs` section — NFR17
2. **AC #2:** Action is referenceable as `uses: stephenleo/vibestats@v1` — FR42
3. **AC #3:** `v1` continues to work when `v2` released — semver-based versioning documented in `CONTRIBUTING.md`

### Existing Test Coverage (DO NOT DUPLICATE)

From `action/tests/test_action_yml.py` (Story 5.4, 25 tests):
- `5.4-UNIT-007a`: `branding.icon` declared
- `5.4-UNIT-007b`: `branding.color` declared
- `5.4-UNIT-002`: `runs.using == 'composite'`
- `5.4-UNIT-003a/b/c/d`: `inputs.token` and `inputs.profile-repo` with `required: true`
- `5.4-UNIT-001a/b/c`: file existence and YAML validity

---

## Step 2: Generation Mode

- **Selected mode:** AI Generation (backend Python project; no browser recording needed)
- **Rationale:** Acceptance criteria are YAML schema/content checks; no UI interaction

---

## Step 3: Test Strategy

### Acceptance Criteria → Test Scenarios

| AC | Scenario | Priority | Level | Status |
|---|---|---|---|---|
| AC #1 (NFR17) | `action.yml.name` is non-empty string | P1 | Unit/Schema | TC-2 — new assertion (distinct from 5.4-UNIT-001) |
| AC #1 (NFR17) | `action.yml.description` is non-empty string | P1 | Unit/Schema | TC-3 — new assertion (distinct from 5.4-UNIT-001) |
| AC #3 | `CONTRIBUTING.md` versioning section heading present | P2 | Content | TC-1a — RED (section not yet authored) |
| AC #3 | `CONTRIBUTING.md` references `v1` backward-compat language | P2 | Content | TC-1b — RED (section not yet authored) |

### Skipped (already covered by Story 5.4)

- branding.icon presence — 5.4-UNIT-007a
- branding.color presence — 5.4-UNIT-007b
- runs.using == composite — 5.4-UNIT-002
- inputs section keys — 5.4-UNIT-003

### TDD Red Phase Requirement

- **TC-1 (P2):** `CONTRIBUTING.md` versioning section — marked `pytest.mark.skip` — will fail until Task 2 is implemented
- **TC-2 (P1):** `action.yml name non-empty` — passes immediately (Story 5.4 already set `name: 'vibestats'`)
- **TC-3 (P1):** `action.yml description non-empty` — passes immediately (Story 5.4 already set description)

---

## Step 4: Generated Tests

### Output Files

| File | Status | Tests | TDD Phase |
|---|---|---|---|
| `action/tests/test_marketplace.py` | CREATED | 4 | RED (2 skipped, 2 passing) |

### Test Inventory

| Test ID | Function | Priority | Status | Covers |
|---|---|---|---|---|
| 8.3-UNIT-001a | `test_tc1_contributing_md_has_versioning_section` | P2 | SKIPPED (RED) | AC #3 |
| 8.3-UNIT-001b | `test_tc1_contributing_md_has_v1_backward_compat_language` | P2 | SKIPPED (RED) | AC #3 |
| 8.3-UNIT-002 | `test_tc2_action_yml_name_is_non_empty` | P1 | PASS | AC #1, R-005, NFR17 |
| 8.3-UNIT-003 | `test_tc3_action_yml_description_is_non_empty` | P1 | PASS | AC #1, R-005, NFR17 |

### Test Run Results

```
python3 -m pytest action/tests/test_marketplace.py -v
4 tests: 2 passed, 2 skipped
```

```
python3 -m pytest action/tests/ -v
86 passed, 2 skipped — existing suite unaffected
```

### Red Phase Compliance

- [x] TC-1 tests use `pytest.mark.skip` (TDD red phase — CONTRIBUTING.md versioning section not yet authored)
- [x] TC-2 and TC-3 assert EXPECTED behavior (non-empty name/description)
- [x] No duplication of Story 5.4 test assertions
- [x] PyYAML (`import yaml`) used for YAML parsing — consistent with existing pattern
- [x] Path resolution: `pathlib.Path(__file__).parent.parent.parent / "action.yml"` — matches `test_action_yml.py`

---

## Green Phase Checklist (for Dev Agent)

When implementing Story 8.3 Task 2 (CONTRIBUTING.md versioning section):

- [ ] Remove `@pytest.mark.skip` from `test_tc1_contributing_md_has_versioning_section`
- [ ] Remove `@pytest.mark.skip` from `test_tc1_contributing_md_has_v1_backward_compat_language`
- [ ] Verify `python3 -m pytest action/tests/test_marketplace.py -v` — 4 passed, 0 skipped, 0 failed
- [ ] Verify `python3 -m pytest action/tests/ -v` — full suite still green

---

## Quality Gate

- **P0 pass rate:** N/A (no P0 tests for this story — existing P0s covered by Story 5.4)
- **P1 pass rate:** 100% (2/2 passing immediately)
- **P2 pass rate:** 0% until CONTRIBUTING.md versioning section authored (expected red phase)
- **Acceptance:** Green when all 4 tests pass (after Task 2 implementation)
