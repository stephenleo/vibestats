---
stepsCompleted: ['step-01-preflight-and-context', 'step-02-generation-mode', 'step-03-test-strategy', 'step-04-generate-tests', 'step-04c-aggregate', 'step-05-validate-and-complete']
lastStep: 'step-05-validate-and-complete'
lastSaved: '2026-04-13'
workflowType: 'testarch-atdd'
inputDocuments:
  - '_bmad-output/implementation-artifacts/9-6-first-release-push-v0-1-0-tag-and-create-v1-floating-tag.md'
  - '_bmad-output/planning-artifacts/epic-9.md'
  - '_bmad-output/test-artifacts/test-design-epic-9.md'
  - '.github/workflows/release.yml'
  - 'action.yml'
  - 'CONTRIBUTING.md'
  - 'Cargo.toml'
  - '_bmad/tea/config.yaml'
---

# ATDD Checklist — Story 9.6: First release — push v0.1.0 tag and create v1 floating tag

**Date:** 2026-04-13
**Author:** Leo
**Primary Test Level:** Schema/Static analysis (Python/pytest)
**Stack:** Backend (CI/CD workflows, Cargo, shell)
**TDD Phase:** GREEN — All 21 pre-release checklist/structural tests pass.
**Test File:** `action/tests/test_release_9_6.py`

---

## Story Summary

As a user who wants to install vibestats,
I want a published GitHub Release with downloadable binaries and a working Marketplace action reference,
So that `curl -sSf https://vibestats.dev/install.sh | bash` and `uses: stephenleo/vibestats@v1` both work end-to-end.

---

## Step 1: Preflight & Context

### Stack Detection

**Detected stack:** `backend` (Cargo.toml present; no package.json with frontend framework; bats installer tests; Python action tests)

**Framework config:** No Playwright/Cypress config. Test framework: `pytest` (Python) + `bats-core` (shell).

**TEA config:** `tea_use_playwright_utils: true` (overridden to API-only profile — backend detected), `test_stack_type: auto`.

### Prerequisites Verified

- [x] Story file exists: `_bmad-output/implementation-artifacts/9-6-first-release-push-v0-1-0-tag-and-create-v1-floating-tag.md`
- [x] Clear acceptance criteria present (5 ACs)
- [x] Test framework configured: `action/tests/` with existing pytest tests
- [x] Development environment available

### Story Context

**AC #1** — `git push origin v0.1.0` triggers `release.yml` → GitHub Release with 3 binary assets  
**AC #2** — `v1` floating tag created: `git tag v1 v0.1.0 && git push origin v1` succeeds; `@v1` resolves to v0.1.0 commit  
**AC #3** — Linux `cross` compilation succeeds OR `rustls` fallback is applied and build succeeds  
**AC #4** — All Marketplace prerequisites verified; submission initiated (manual step)  
**AC #5** — `deploy-site.yml` `workflow_dispatch` triggered; `vibestats.dev` serves landing page

---

## Step 2: Generation Mode

**Mode selected:** AI generation (backend project; no browser UI; acceptance criteria are operational/schema-verifiable)

**Rationale:** Story 9.6 is a release-trigger story — the infrastructure already exists from Epics 5, 8. The testable surface area is:
1. **Pre-release checklist assertions** (static schema tests in pytest): verify that all conditions required before pushing the tag are already true.
2. **release.yml structural requirements** (pytest): verify that the workflow will correctly create the GitHub Release AND update the v1 floating tag.
3. **Runtime-only steps** (manual verification): pushing the tag, monitoring CI, verifying the GitHub Release page, checking git ls-remote for v1.

Recording mode is not applicable (no browser UI involved).

---

## Step 3: Test Strategy

### Acceptance Criteria → Test Scenario Mapping

| AC | Test Level | Priority | Scenario | Automatable |
|----|------------|----------|----------|-------------|
| AC #1 — release.yml creates Release with 3 binaries | Schema (pytest) | P0 | release.yml references all 3 target names and .tar.gz assets | Yes |
| AC #1 — release.yml trigger is tag-push only | Schema (pytest) | P0 | on.push.tags = ['v*'], no branch triggers | Yes |
| AC #2 — v1 floating tag created | Schema (pytest) | P0 | release.yml has force-push step that derives major version | Yes |
| AC #2 — v1 floating tag uses force-push | Schema (pytest) | P0 | git push --force pattern present in release.yml | Yes |
| AC #2 — v1 CONTRIBUTING.md documented | Schema (pytest) | P1 | CONTRIBUTING.md has versioning section + v1 reference + force-push | Yes |
| AC #3 — ureq dependency present for rustls fallback | Schema (pytest) | P1 | Cargo.toml has ureq entry | Yes |
| AC #3 — Cargo.toml version is 0.1.0 | Schema (pytest) | P1 | version = "0.1.0" in Cargo.toml | Yes |
| AC #4 — action.yml branding complete | Schema (pytest) | P1 | branding.icon, branding.color non-empty | Yes |
| AC #4 — action.yml name/description set | Schema (pytest) | P1 | name, description non-empty strings | Yes |
| AC #5 — deploy-site.yml exists | File check (pytest) | P1 | deploy-site.yml at expected path | Yes |
| AC #5 — deploy-site.yml has workflow_dispatch | Schema (pytest) | P1 | on.workflow_dispatch trigger present | Yes |
| AC #1 runtime — GitHub Release with 3 binaries | Manual/CI | P0 | GitHub Releases page shows 3 assets | No |
| AC #2 runtime — v1 tag on remote | Manual/git | P1 | git ls-remote shows v1 → v0.1.0 commit | No |
| AC #3 runtime — cargo test passes | Manual/shell | P1 | cargo test exits 0 (pre-flight gate) | No |
| AC #3 runtime — cargo clippy passes | Manual/shell | P1 | cargo clippy --all-targets -- -D warnings exits 0 | No |
| AC #3 runtime — bats suite passes | Manual/shell | P0 | bats test_6_1..test_6_4.bats exits 0 | No |
| AC #5 runtime — vibestats.dev loads | Manual/browser | P2 | Landing page served from current main | No |
| AC #4 runtime — Marketplace submission | Manual/UI | P2 | Submission form submitted (documented in Dev Agent Record) | No |

### Test Level Rationale

Story 9.6 is a **release trigger story** — the implementation is operational (pushing a tag), not a code feature. This drives the test level distribution:

- **Schema/static (pytest)**: 21 tests — verify all pre-release preconditions are already true before the dev agent pushes the tag. These tests are **already GREEN** because the underlying infrastructure (release.yml, action.yml, CONTRIBUTING.md, Cargo.toml) was delivered by Epics 5 and 8.
- **Manual/runtime verification**: GitHub Release creation, git tag checks, cargo/bats pre-flight — these require a live environment (GitHub API, build toolchain, remote git).
- **No E2E (browser)**: Not applicable for a CI/CD release story.

### TDD Red Phase Assessment

This story is unusual in the TDD cycle: the schema/static tests are already GREEN because Story 9.6 does not modify any source files — it TRIGGERS the existing workflow. The pre-release checklist tests verify infrastructure already in place.

The RED condition for Story 9.6 is: **the GitHub Release does not yet exist** (runtime-only verification). The pytest tests verify all preconditions are satisfied so the dev agent can safely push the tag.

---

## Step 4: Test Generation

### Test File Created

**File:** `action/tests/test_release_9_6.py`

**Test count:** 21 tests

**TDD Phase:** All schema/static tests are GREEN (infrastructure pre-built by Epics 5, 8). Runtime verification steps are manual.

### Test Groups

#### Group 1: Pre-flight file existence (P0) — 9.6-UNIT-000–003

Tests that all required files exist before pushing the tag:
- `test_preflight_release_yml_exists` — `.github/workflows/release.yml`
- `test_preflight_action_yml_exists` — `action.yml`
- `test_preflight_contributing_md_exists` — `CONTRIBUTING.md`
- `test_preflight_cargo_toml_exists` — `Cargo.toml`

#### Group 2: action.yml branding pre-release checklist (P1) — 9.6-UNIT-010–013

Task 1 checklist items from story: verify branding is set before Marketplace submission:
- `test_tc1_action_yml_name_present_and_non_empty`
- `test_tc1_action_yml_description_present_and_non_empty`
- `test_tc1_action_yml_branding_icon_present_and_non_empty`
- `test_tc1_action_yml_branding_color_present_and_non_empty`

#### Group 3: CONTRIBUTING.md versioning documentation (P1) — 9.6-UNIT-020–022

Task 1 checklist + AC #2 prerequisite: versioning section present, v1 referenced, force-push procedure documented:
- `test_tc2_contributing_md_has_release_versioning_section`
- `test_tc2_contributing_md_references_v1_tag`
- `test_tc2_contributing_md_documents_floating_tag_force_push`

#### Group 4: release.yml v1 floating tag step (P0/P1) — 9.6-UNIT-030–032

AC #2: release.yml automatically creates/updates the v1 floating tag:
- `test_tc3_release_yml_has_v1_floating_tag_step` [P0]
- `test_tc3_release_yml_floating_tag_uses_force_push` [P0]
- `test_tc3_release_yml_floating_tag_derives_major_version` [P1]

#### Group 5: release.yml binary asset names (P0) — 9.6-UNIT-040–041

AC #1: release.yml references all three platform targets and .tar.gz format:
- `test_tc4_release_yml_references_all_three_binary_assets` [P0]
- `test_tc4_release_yml_references_tar_gz_assets` [P0]

#### Group 6: Cargo.toml version + ureq dependency (P1) — 9.6-UNIT-050–051

AC #3 pre-flight: version matches tag, ureq present for optional rustls fallback:
- `test_tc5_cargo_toml_version_is_0_1_0`
- `test_tc5_cargo_toml_has_ureq_dependency`

#### Group 7: deploy-site.yml presence (P1) — 9.6-UNIT-060–061

AC #5: deploy-site.yml exists with workflow_dispatch trigger:
- `test_tc6_deploy_site_yml_exists`
- `test_tc6_deploy_site_yml_has_workflow_dispatch_trigger`

#### Group 8: release.yml trigger validation (P0) — 9.6-UNIT-070

Pre-flight: release.yml only triggers on tag pushes (not branch pushes):
- `test_tc7_release_yml_trigger_is_tag_push_only`

---

## Step 5: Validate & Complete

### Checklist Validation

- [x] Prerequisites satisfied (story file, test framework, clear ACs)
- [x] Test file created: `action/tests/test_release_9_6.py`
- [x] Checklist matches all automatable acceptance criteria
- [x] Tests verify pre-release state (schema assertions)
- [x] No CLI sessions to clean up (no browser automation used)
- [x] Test file in `action/tests/` not a random location

### Test Execution Results (GREEN Phase — Pre-release conditions already met)

```
Command: python3 -m pytest action/tests/test_release_9_6.py -v
Platform: darwin -- Python 3.9.6, pytest-8.4.2

21 passed in 0.04s

All 21 schema/static tests pass:
  PASSED action/tests/test_release_9_6.py::test_preflight_release_yml_exists
  PASSED action/tests/test_release_9_6.py::test_preflight_action_yml_exists
  PASSED action/tests/test_release_9_6.py::test_preflight_contributing_md_exists
  PASSED action/tests/test_release_9_6.py::test_preflight_cargo_toml_exists
  PASSED action/tests/test_release_9_6.py::test_tc1_action_yml_name_present_and_non_empty
  PASSED action/tests/test_release_9_6.py::test_tc1_action_yml_description_present_and_non_empty
  PASSED action/tests/test_release_9_6.py::test_tc1_action_yml_branding_icon_present_and_non_empty
  PASSED action/tests/test_release_9_6.py::test_tc1_action_yml_branding_color_present_and_non_empty
  PASSED action/tests/test_release_9_6.py::test_tc2_contributing_md_has_release_versioning_section
  PASSED action/tests/test_release_9_6.py::test_tc2_contributing_md_references_v1_tag
  PASSED action/tests/test_release_9_6.py::test_tc2_contributing_md_documents_floating_tag_force_push
  PASSED action/tests/test_release_9_6.py::test_tc3_release_yml_has_v1_floating_tag_step
  PASSED action/tests/test_release_9_6.py::test_tc3_release_yml_floating_tag_uses_force_push
  PASSED action/tests/test_release_9_6.py::test_tc3_release_yml_floating_tag_derives_major_version
  PASSED action/tests/test_release_9_6.py::test_tc4_release_yml_references_all_three_binary_assets
  PASSED action/tests/test_release_9_6.py::test_tc4_release_yml_references_tar_gz_assets
  PASSED action/tests/test_release_9_6.py::test_tc5_cargo_toml_version_is_0_1_0
  PASSED action/tests/test_release_9_6.py::test_tc5_cargo_toml_has_ureq_dependency
  PASSED action/tests/test_release_9_6.py::test_tc6_deploy_site_yml_exists
  PASSED action/tests/test_release_9_6.py::test_tc6_deploy_site_yml_has_workflow_dispatch_trigger
  PASSED action/tests/test_release_9_6.py::test_tc7_release_yml_trigger_is_tag_push_only
```

**TDD Note:** The schema/static tests are GREEN because Story 9.6 is a release-trigger story. The infrastructure (release.yml, action.yml, CONTRIBUTING.md, Cargo.toml) was already delivered by Epics 5 and 8. The TRUE RED condition for this story is: the GitHub Release does not yet exist at `github.com/stephenleo/vibestats/releases/tag/v0.1.0`. That condition is verified manually after the dev agent pushes the tag.

### Manual Verification Checklist (Dev Agent — Task execution order)

The dev agent must complete these manual steps (not automatable):

**Task 1: Pre-release checklist** (all conditions are now verified GREEN by pytest):
- [ ] `cargo test` — run from repo root, must be 0 failures
- [ ] `cargo clippy --all-targets -- -D warnings` — must be 0 warnings
- [ ] `bats tests/installer/test_6_1.bats tests/installer/test_6_2.bats tests/installer/test_6_3.bats tests/installer/test_6_4.bats` — must be 0 failures
- [ ] `python3 -m pytest action/tests/test_release_9_6.py -v` — must pass 21/21 (already confirmed)

**Task 2: Tag and push v0.1.0** (gates AC #1):
- [ ] `git status` — working tree clean on main
- [ ] `git tag -a v0.1.0 -m "Initial release — vibestats v0.1.0"`
- [ ] `git push origin v0.1.0`
- [ ] Monitor `release.yml` Actions run on GitHub

**Task 3: Handle TLS/cross-compilation** (gates AC #3):
- [ ] If Linux build succeeds: document in Dev Agent Record
- [ ] If Linux build fails with OpenSSL error: apply rustls fallback (see Dev Notes)
- [ ] Verify all 3 platform binaries present in GitHub Release

**Task 4: Create v1 floating tag** (gates AC #2):
- [ ] `git tag v1 v0.1.0`
- [ ] `git push origin v1`
- [ ] Verify: `git ls-remote origin refs/tags/v1` shows tag pointing to v0.1.0 commit

**Task 5: Trigger Cloudflare Pages deployment** (gates AC #5):
- [ ] Navigate to `deploy-site.yml` workflow in GitHub Actions UI
- [ ] Trigger via `workflow_dispatch` with `ref: main`
- [ ] Confirm `vibestats.dev` loads correctly

**Task 6: Marketplace submission** (gates AC #4 — manual UI step):
- [ ] Follow Marketplace submission process from Story 8.3 Dev Notes
- [ ] Submit action for review
- [ ] Document submission status in Dev Agent Record

### Completion Summary

- **Test file created:** `action/tests/test_release_9_6.py` (21 tests)
- **Checklist output:** `_bmad-output/test-artifacts/atdd-checklist-9.6-first-release-push-v0-1-0-tag-and-create-v1-floating-tag.md`
- **Key finding:** All pre-release preconditions are already satisfied (Epics 5 and 8 delivered the required infrastructure). The dev agent's implementation task is entirely operational: run the pre-flight checklist, push the tag, monitor CI, create the v1 tag, trigger Pages deploy, submit to Marketplace.
- **Assumption:** Story 9.3 (bats failures) and Story 9.5 (dead_code suppressors) should be complete before pushing the tag. The `cargo clippy` pre-flight check will gate on Story 9.5 completion.
- **Risk:** The Linux cross-compilation (R-001) is the only technical risk during tag push. The rustls fallback is documented and ready.
- **Next recommended workflow:** `bmad-dev-story` for execution of Tasks 1–6 above.

---

## Mock Requirements

No mocks required for pytest schema tests (file-system reads only). The manual pre-flight steps use real tooling:
- `cargo` CLI (test + clippy)
- `bats` CLI (installer tests)
- `git` CLI (tag creation + push)
- `gh` CLI (release monitoring — optional)

---

## Running Tests

```bash
# Verify all pre-release preconditions are met
python3 -m pytest action/tests/test_release_9_6.py -v

# Pre-flight manual steps (run before git tag)
cargo test
cargo clippy --all-targets -- -D warnings
bats tests/installer/test_6_1.bats tests/installer/test_6_2.bats tests/installer/test_6_3.bats tests/installer/test_6_4.bats

# After task completion: verify runtime conditions (manual)
git ls-remote origin refs/tags/v0.1.0
git ls-remote origin refs/tags/v1
```

---

## Red-Green-Refactor Workflow

### RED Phase (For runtime-only conditions)

The TRUE RED conditions that will become GREEN after implementation:
1. `github.com/stephenleo/vibestats/releases/tag/v0.1.0` — does not exist yet → becomes GREEN after Task 2/3
2. `git ls-remote origin refs/tags/v1` — v1 tag does not exist on remote yet → becomes GREEN after Task 4
3. `vibestats.dev` — may not be current with main branch → becomes GREEN after Task 5

### GREEN Phase (Static/schema tests — already GREEN)

All 21 pytest schema/static tests pass. The pre-release infrastructure is complete:
- `release.yml` references all 3 binary targets, has fail-fast, has v1 floating tag step with force-push
- `action.yml` has all Marketplace-required fields
- `CONTRIBUTING.md` has Release Versioning section documenting v1 floating tag convention
- `Cargo.toml` version is 0.1.0 and ureq dependency is present

### REFACTOR Phase

Not applicable — this story is operational, not a code refactor.

---

## Knowledge Base References Applied

- **test-quality.md** — Pre-flight verification pattern; separating static from runtime assertions
- **test-levels-framework.md** — Schema test level for CI/CD workflow files; "no E2E for backend"
- **test-priorities-matrix.md** — P0 for release blockers (missing binary asset); P1 for pre-flight checks
- **ci-burn-in.md** — Pre-release gate requirements (all three toolchain checks before tagging)
- **data-factories.md** — Not applicable (no data fixtures needed for schema tests)

---

**Generated by:** BMad TEA Agent - ATDD Workflow
**Workflow:** `bmad-testarch-atdd`
**Story:** 9.6-first-release-push-v0-1-0-tag-and-create-v1-floating-tag
**Date:** 2026-04-13
