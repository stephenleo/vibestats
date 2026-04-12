---
stepsCompleted:
  - step-01-preflight-and-context
  - step-02-generation-mode
  - step-03-test-strategy
  - step-04-generate-tests
lastStep: step-04-generate-tests
lastSaved: '2026-04-12'
inputDocuments:
  - _bmad-output/planning-artifacts/epics.md
  - _bmad-output/planning-artifacts/prd.md
  - _bmad-output/planning-artifacts/architecture.md
  - _bmad-output/test-artifacts/test-design-epic-8.md
  - _bmad/tea/config.yaml
  - action/tests/test_aggregate_yml.py
  - action/tests/test_action_yml.py
outputFiles:
  - action/tests/test_deploy_site_yml.py
---

# ATDD Checklist: Story 8.2 — Implement Cloudflare Pages Deploy Workflow

**Story:** 8.2-implement-cloudflare-pages-deploy-workflow
**GH Issue:** #40
**Date:** 2026-04-12
**Author:** Leo
**TDD Phase:** RED (failing tests generated, implementation pending)

---

## Step 1: Preflight & Context

### Stack Detection

- **Detected stack:** `backend`
- **Indicators:** `action/tests/*.py`, `pyproject.toml` (action/), pytest infrastructure from Epics 5–7

### Prerequisites

- [x] Story 8.2 approved with clear acceptance criteria (epics.md)
- [x] Test framework configured: pytest (established in Epic 5, conftest pattern present)
- [x] Development environment available
- [ ] `deploy-site.yml` does NOT exist yet at `.github/workflows/deploy-site.yml` — expected (red phase)

### Story Context (from epics.md)

**Story 8.2: Implement Cloudflare Pages deploy workflow**

As a developer deploying vibestats.dev,
I want a manually-triggered GitHub Actions workflow that deploys the Astro site to Cloudflare Pages,
So that I control exactly which version is live in production.

**Acceptance Criteria:**

- **AC1:** `deploy-site.yml` triggered only via `workflow_dispatch` with a `ref` input (branch or tag) — no automatic triggers (push, pull_request, schedule, release).
- **AC2:** Workflow checks out the specified ref, runs `npm run build` inside `site/`, and deploys to Cloudflare Pages using `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` secrets.
- **AC3:** If the build step fails, no deployment to Cloudflare occurs (build gates deploy — no `continue-on-error: true` on build step).

### Framework & Existing Patterns

- Test pattern: Python pytest, YAML schema parsing via `yaml.safe_load()`
- Existing precedent: `test_aggregate_yml.py` (Story 5.5) — YAML trigger/step assertions
- Test location: `action/tests/test_deploy_site_yml.py`

### TEA Config Flags

- `tea_use_playwright_utils`: true (not applicable — backend stack)
- `tea_use_pactjs_utils`: false
- `tea_pact_mcp`: none
- `test_stack_type`: auto → resolved to `backend`

---

## Step 2: Generation Mode

**Mode selected:** AI Generation

**Reasoning:** Story 8.2 is a pure backend YAML schema validation story. No browser interactions. All test scenarios are standard file-parse assertions on a GitHub Actions workflow file. AI generation from acceptance criteria and risk analysis (test-design-epic-8.md) is appropriate.

---

## Step 3: Test Strategy

### Acceptance Criteria → Test Scenarios

| AC | Scenario | Priority | Risk Link |
|----|----------|----------|-----------|
| AC1 | `on:` key contains only `workflow_dispatch` — no push/PR/schedule/release | P0 | R-003 |
| AC1 | `workflow_dispatch.inputs.ref` declared | P1 | R-003 |
| AC2 | `secrets.CLOUDFLARE_API_TOKEN` referenced by exact name | P0 | R-004 |
| AC2 | `secrets.CLOUDFLARE_ACCOUNT_ID` referenced by exact name | P0 | R-004 |
| AC2 | No hardcoded token values (32+ char alphanumeric outside secrets expressions) | P0 | R-004 |
| AC2 | `npm run build` step present | P0 | R-008 |
| AC2 | Checkout step uses `${{ github.event.inputs.ref }}` variable | P1 | R-003 |
| AC2 | `npm run build` runs inside `site/` working directory | P2 | R-008 |
| AC3 | `npm run build` precedes any Cloudflare Pages deploy step | P0 | R-008 |
| AC3 | Build step has no `continue-on-error: true` | P0 | R-008 |

### Test Level Selection

All tests: **Schema/Unit** — pure YAML file parsing. No I/O beyond file read. Completes in <1 second per test. Appropriate for backend stack with no browser automation needed.

### TDD Red Phase Confirmation

All tests use `@pytest.mark.skip(reason="RED PHASE — deploy-site.yml not yet implemented...")`.

Tests are designed to FAIL when the skip is removed and `.github/workflows/deploy-site.yml` does not exist or does not comply with the acceptance criteria. They will PASS only after a correct implementation is committed.

---

## Step 4: Test Generation

### Generated Test File

**Path:** `action/tests/test_deploy_site_yml.py`

**Test count:** 11 tests

| Test ID | Name | Priority | AC | Status |
|---------|------|----------|----|--------|
| 8.2-UNIT-000 | `test_preflight_deploy_site_yml_exists` | P0 | — | SKIP (red) |
| 8.2-UNIT-001 | `test_tc1_only_workflow_dispatch_trigger` | P0 | AC1 | SKIP (red) |
| 8.2-UNIT-002a | `test_tc2_cloudflare_api_token_secret_name` | P0 | AC2 | SKIP (red) |
| 8.2-UNIT-002b | `test_tc2_cloudflare_account_id_secret_name` | P0 | AC2 | SKIP (red) |
| 8.2-UNIT-002c | `test_tc2_no_hardcoded_token_values` | P0 | AC2 | SKIP (red) |
| 8.2-UNIT-003a | `test_tc3_npm_run_build_present` | P0 | AC2, AC3 | SKIP (red) |
| 8.2-UNIT-003b | `test_tc3_npm_run_build_precedes_deploy` | P0 | AC3 | SKIP (red) |
| 8.2-UNIT-003c | `test_tc3_no_continue_on_error_on_build` | P0 | AC3 | SKIP (red) |
| 8.2-UNIT-004 | `test_tc4_workflow_dispatch_has_ref_input` | P1 | AC1 | SKIP (red) |
| 8.2-UNIT-005 | `test_tc5_checkout_uses_event_inputs_ref` | P1 | AC2 | SKIP (red) |
| 8.2-UNIT-006 | `test_tc6_build_uses_site_working_directory` | P2 | AC2 | SKIP (red) |

**P0 count:** 8 tests
**P1 count:** 2 tests
**P2 count:** 1 test

### Risk Coverage

| Risk ID | Description | Covered By | Priority |
|---------|-------------|------------|----------|
| R-003 | deploy-site.yml runs on push/schedule — deploys uncommitted state | 8.2-UNIT-001, 004, 005 | P0/P1 |
| R-004 | Incorrect/missing Cloudflare secret names | 8.2-UNIT-002a, 002b, 002c | P0 |
| R-008 | Build does not gate deployment | 8.2-UNIT-003a, 003b, 003c, 006 | P0/P2 |

### TDD Verification

```
pytest action/tests/test_deploy_site_yml.py -v
# Result: 11 skipped (RED PHASE — file does not exist yet)
```

---

## GREEN Phase Instructions

When `deploy-site.yml` is implemented:

1. Remove all `@pytest.mark.skip` decorators from `test_deploy_site_yml.py`
2. Run: `python3 -m pytest action/tests/test_deploy_site_yml.py -v`
3. All 11 tests must pass
4. Run the full test suite to verify no regression: `python3 -m pytest action/tests/ -v`

**Required implementation checklist (for GREEN):**

- [ ] `.github/workflows/deploy-site.yml` exists
- [ ] `on:` key contains only `workflow_dispatch` (no push/PR/schedule/release)
- [ ] `workflow_dispatch.inputs.ref` declared (type: string, description, default: main or empty)
- [ ] Checkout step uses `${{ github.event.inputs.ref }}` or `${{ inputs.ref }}`
- [ ] `npm run build` step present with `working-directory: site`
- [ ] `npm run build` step appears before any Cloudflare Pages deploy step
- [ ] `npm run build` step has no `continue-on-error: true`
- [ ] Cloudflare Pages deploy step references `secrets.CLOUDFLARE_API_TOKEN`
- [ ] Cloudflare Pages deploy step references `secrets.CLOUDFLARE_ACCOUNT_ID`
- [ ] No hardcoded credential strings in the file

---

## Quality Gate

- **P0 pass rate required:** 100% (blocks merge)
- **P1 pass rate required:** ≥95%
- **P2 pass rate required:** ≥90% (informational)
- **Run on:** Every PR touching `.github/workflows/deploy-site.yml`

---

**Generated by:** BMad TEA Agent — ATDD Module
**Workflow:** `bmad-testarch-atdd`
**TDD Phase:** RED
