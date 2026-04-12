---
stepsCompleted: ['step-01-detect-mode', 'step-02-load-context', 'step-03-risk-and-testability', 'step-04-coverage-plan', 'step-05-generate-output']
lastStep: 'step-05-generate-output'
lastSaved: '2026-04-12'
mode: 'epic-level'
epic: 8
inputDocuments:
  - '_bmad-output/planning-artifacts/epics.md'
  - '_bmad-output/planning-artifacts/prd.md'
  - '_bmad-output/planning-artifacts/architecture.md'
  - '_bmad-output/implementation-artifacts/sprint-status.yaml'
  - 'action.yml'
  - '.github/workflows/aggregate.yml'
  - 'Cargo.toml'
---

# Test Design: Epic 8 — CI/CD & Distribution

**Date:** 2026-04-12
**Author:** Leo
**Status:** Active

---

## Executive Summary

**Scope:** Epic-level test design for Epic 8 — CI/CD & Distribution.

Epic 8 delivers three CI/CD and distribution stories: a cross-platform Rust binary release pipeline (`release.yml`, Story 8.1), a manually-triggered Cloudflare Pages deployment workflow (`deploy-site.yml`, Story 8.2), and GitHub Actions Marketplace publication for the community action (Story 8.3). All three stories are currently in `backlog`. Epic 8 is a prerequisite for Epic 6 (Bash installer).

The testing surface is primarily YAML-level schema validation and workflow-structure assertions. There is no application logic to unit-test — the deliverables are GitHub Actions workflow files and metadata configuration. Integration-level testing requires a live GitHub Actions environment, which is expensive; schema/structural tests cover the majority of observable risk with minimal CI overhead.

**Risk Summary:**

- Total risks identified: 9
- High-priority risks (score ≥6): 4
- Critical categories: OPS, TECH, BUS, SEC

**Coverage Summary:**

- P0 scenarios: 6 (~8–14 hours)
- P1 scenarios: 7 (~6–12 hours)
- P2 scenarios: 5 (~3–7 hours)
- P3 scenarios: 2 (~1–3 hours)
- **Total effort:** ~18–36 hours (~1 week)

---

## Not in Scope

| Item | Reasoning | Mitigation |
| --- | --- | --- |
| Bash installer testing (Epic 6) | Epic 6 scope — depends on binary release being available | Covered in Epic 6 test design |
| Astro site content and routing | Epic 7 scope — vibestats.dev pages already tested | Covered in Epic 7 test design |
| GitHub Actions Marketplace approval process | Manual review by GitHub; not automatable | Validated via `action.yml` schema test before submission |
| Cross-platform binary functional correctness | Epics 2–4 scope — sync/CLI logic already tested | Covered in Epics 2–4; Epic 8 tests only verify the release pipeline structure |
| Cloudflare Pages routing and DNS | Infrastructure/ops concern outside vibestats repo scope | Acceptance: successful deploy confirmed via Cloudflare dashboard post-deploy |
| Python Actions pipeline correctness | Epic 5 scope | Covered in Epic 5 test design |

---

## Risk Assessment

> P0/P1/P2/P3 = priority and risk level, NOT execution timing.

### High-Priority Risks (Score ≥6)

| Risk ID | Category | Description | Probability | Impact | Score | Mitigation | Owner | Timeline |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| R-001 | OPS | `release.yml` matrix build fails silently on one platform (e.g., Linux cross-compilation fails) but publishes a partial release, leaving install.sh with a missing binary for that platform — breaks Epic 6 installer | 2 | 3 | 6 | Assert `release.yml` uses `fail-fast: true` (or equivalent) in matrix strategy; integration test on tag push verifies all three artifacts present on GitHub Release | Dev/QA | Pre-merge |
| R-002 | TECH | Cross-compilation target misconfiguration in `release.yml` (wrong target triple or `cross` crate version pinning) causes build failures that block all platform releases | 2 | 3 | 6 | Schema test: assert matrix includes exactly `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`; assert `cross` crate usage; smoke-test binary runs on each platform post-release | Dev/QA | Pre-merge |
| R-003 | BUS | `deploy-site.yml` runs on push/schedule (not only `workflow_dispatch`) and deploys uncommitted or intermediate site state to production Cloudflare Pages, corrupting vibestats.dev | 2 | 3 | 6 | Schema test: parse `deploy-site.yml`, assert `on` key contains only `workflow_dispatch`; no `push`, `pull_request`, `schedule`, or `release` triggers | Dev/QA | Pre-merge |
| R-004 | SEC | `CLOUDFLARE_API_TOKEN` or `CLOUDFLARE_ACCOUNT_ID` secrets referenced with incorrect names in `deploy-site.yml`, causing either a failed deploy (wrong name) or accidental secret exposure in logs (interpolated incorrectly) | 2 | 3 | 6 | Schema test: parse `deploy-site.yml` and assert exact secret reference names match `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID`; assert no hardcoded token values present in file | Dev/QA | Pre-merge |

### Medium-Priority Risks (Score 3–5)

| Risk ID | Category | Description | Probability | Impact | Score | Mitigation | Owner |
| --- | --- | --- | --- | --- | --- | --- | --- |
| R-005 | TECH | `action.yml` missing `branding` block or required fields for Marketplace submission causes Marketplace rejection (NFR17) | 2 | 2 | 4 | Schema test: parse `action.yml`, assert `branding.icon` and `branding.color` present; assert `name`, `description`, `inputs`, and `runs` sections present | Dev/QA |
| R-006 | OPS | `release.yml` uses mutable action tags (e.g., `actions/upload-artifact@main`) that break on upstream changes, causing release pipeline failures | 2 | 2 | 4 | Schema test: assert all `uses:` references in `release.yml` are pinned to SHAs or major version tags (e.g., `@v4`), not `@main` or `@master` | Dev/QA |
| R-007 | BUS | `release.yml` uploads binaries without consistent naming convention (e.g., `vibestats` instead of `vibestats-<target>.tar.gz`), breaking install.sh download URL pattern in Epic 6 | 2 | 2 | 4 | Schema test: assert artifact upload step names follow `vibestats-<target>.tar.gz` pattern; assert filename template uses `${{ matrix.target }}` variable | Dev/QA |
| R-008 | OPS | `deploy-site.yml` does not gate deployment behind a passing `npm run build` step — deploys broken build output to Cloudflare Pages (Story 8.2 AC) | 1 | 3 | 3 | Schema test: assert step order in `deploy-site.yml` — `npm run build` must precede any deploy step; assert build step has no `continue-on-error: true` | Dev/QA |

### Low-Priority Risks (Score 1–2)

| Risk ID | Category | Description | Probability | Impact | Score | Action |
| --- | --- | --- | --- | --- | --- | --- |
| R-009 | TECH | `action.yml` `runs.using` value is not `composite` — breaks community action execution model | 1 | 2 | 2 | Monitor — covered by existing Epic 5 test; reconfirm in P1 schema sweep |

### Risk Category Legend

- **TECH**: Technical/Architecture (integration, structure, schema)
- **SEC**: Security (data exposure, secret handling, credential interpolation)
- **PERF**: Performance (SLA violations, resource limits)
- **DATA**: Data Integrity (loss, corruption, incorrect aggregation)
- **BUS**: Business Impact (UX harm, logic errors, distribution failures)
- **OPS**: Operations (deployment, pipeline resilience, config)

---

## Entry Criteria

- [ ] Story 8.1 (`release.yml`) workflow file exists in `.github/workflows/`
- [ ] Story 8.2 (`deploy-site.yml`) workflow file exists in `.github/workflows/`
- [ ] Story 8.3 Marketplace metadata: `action.yml` at repo root with `branding` block confirmed
- [ ] `Cargo.toml` version field matches intended release tag format (e.g., `0.1.0` → `v0.1.0`)
- [ ] Architecture document confirms expected target triples and artifact naming convention

## Exit Criteria

- [ ] All P0 tests passing (100%)
- [ ] All P1 tests passing (≥95%)
- [ ] No open OPS or SEC category risks (R-001, R-003, R-004) unmitigated
- [ ] R-001 through R-004 mitigations verified via schema/structural tests
- [ ] `action.yml` Marketplace metadata validated before GitHub Marketplace submission (Story 8.3)

---

## Test Coverage Plan

> P0/P1/P2/P3 = priority and risk level, NOT execution timing. Execution timing is defined in the Execution Strategy section.

### P0 (Critical)

**Criteria:** Blocks distribution correctness + High risk (≥6) + No workaround

| Requirement | Test Level | Risk Link | Test Count | Owner | Notes |
| --- | --- | --- | --- | --- | --- |
| `release.yml`: matrix strategy uses `fail-fast: true` (or equivalent) to prevent partial releases on platform build failure (Story 8.1 AC) | Schema/Unit | R-001 | 1 | Dev/QA | Parse `release.yml` YAML; assert matrix includes `fail-fast: true` or equivalent job dependency |
| `release.yml`: matrix targets are exactly `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu` — no missing or extra targets (Story 8.1 AC, FR41) | Schema/Unit | R-002 | 1 | Dev/QA | Parse YAML matrix; assert exact target set |
| `release.yml`: each binary archived as `vibestats-<target>.tar.gz` using matrix target variable — no hardcoded or inconsistent filenames (Story 8.1 AC) | Schema/Unit | R-007 | 1 | Dev/QA | Assert artifact name template in upload step |
| `deploy-site.yml`: only `workflow_dispatch` trigger present — no `push`, `pull_request`, `schedule`, or `release` triggers (Story 8.2 AC) | Schema/Unit | R-003 | 1 | Dev/QA | Parse `deploy-site.yml` YAML; assert `on` key |
| `deploy-site.yml`: `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` referenced by exact secret names — no hardcoded values, no misspelled names (Story 8.2 AC) | Schema/Unit | R-004 | 1 | Dev/QA | Grep parsed YAML for secret references; assert exact names |
| `deploy-site.yml`: `npm run build` step runs before any deploy step — no `continue-on-error: true` on build step (Story 8.2 AC) | Schema/Unit | R-008 | 1 | Dev/QA | Assert step ordering in parsed YAML |

**Total P0:** 6 tests

### P1 (High)

**Criteria:** Core pipeline behaviours + Medium risk (3–5) + Common paths

| Requirement | Test Level | Risk Link | Test Count | Owner | Notes |
| --- | --- | --- | --- | --- | --- |
| `release.yml`: all `uses:` action references are pinned to version tags (e.g., `@v4`) — not `@main` or `@master` (NFR stability) | Schema/Unit | R-006 | 1 | Dev/QA | Parse all `uses:` values; assert semver or SHA pinning |
| `release.yml`: workflow trigger is `on: push: tags: ['v*']` — no branch or PR triggers present | Schema/Unit | R-001 | 1 | Dev/QA | Parse `on` key; assert only tag trigger |
| `release.yml`: uses `cross` crate or equivalent cross-compilation tool for Linux target — not native `cargo build` | Schema/Unit | R-002 | 1 | Dev/QA | Assert `cross` or `cross-rs/cross` usage in workflow steps |
| `action.yml`: `branding.icon` and `branding.color` present and non-empty (required for Marketplace listing, NFR17, Story 8.3 AC) | Schema/Unit | R-005 | 1 | Dev/QA | Parse `action.yml`; assert branding block |
| `action.yml`: `name`, `description`, `inputs`, and `runs` sections present with non-empty values (FR42, NFR17) | Schema/Unit | R-005 | 1 | Dev/QA | Parse `action.yml`; assert all required fields |
| `deploy-site.yml`: `ref` input declared with `workflow_dispatch.inputs` — allows specifying branch or tag to deploy (Story 8.2 AC) | Schema/Unit | R-003 | 1 | Dev/QA | Assert `inputs.ref` in `workflow_dispatch` block |
| `deploy-site.yml`: checkout step uses `${{ github.event.inputs.ref }}` — deploys the specified ref, not `main` or default branch | Schema/Unit | R-003 | 1 | Dev/QA | Assert ref variable used in checkout step |

**Total P1:** 7 tests

### P2 (Medium)

**Criteria:** Secondary behaviours + Low–medium risk (1–4) + Edge cases

| Requirement | Test Level | Risk Link | Test Count | Owner | Notes |
| --- | --- | --- | --- | --- | --- |
| `release.yml`: GitHub Release creation step uses the triggering tag — no hardcoded version string | Schema/Unit | R-007 | 1 | Dev/QA | Assert `${{ github.ref_name }}` or equivalent in release step |
| `action.yml`: `runs.using` is `composite` — not `node20` or `docker` (required for composite action model) | Schema/Unit | R-009 | 1 | Dev/QA | Covered by Epic 5; reconfirm in schema pass |
| `deploy-site.yml`: checkout step targets `site/` subdirectory or workflow `cd`s into `site/` before `npm run build` | Schema/Unit | R-008 | 1 | Dev/QA | Assert `working-directory: site` or equivalent |
| `CONTRIBUTING.md`: documents semver-based versioning and backwards-compatibility promise — `v1` continues to work when `v2` is released (Story 8.3 AC) | Content/Schema | - | 1 | Dev/QA | Read `CONTRIBUTING.md`; assert versioning section present |
| `release.yml`: binary artifact upload step attaches all three archives to the GitHub Release — not just a single platform | Schema/Unit | R-001 | 1 | Dev/QA | Assert all three targets are referenced or that matrix.target drives upload |

**Total P2:** 5 tests

### P3 (Low)

**Criteria:** Nice-to-have + Exploratory + Benchmarks

| Requirement | Test Level | Test Count | Owner | Notes |
| --- | --- | --- | --- | --- |
| End-to-end release smoke test: push a `v*` test tag to a fork, confirm all three binary archives appear on the resulting GitHub Release | Integration/Manual | 1 | Dev | Manual — requires live GitHub Actions environment on fork; run once before first production release |
| Marketplace listing smoke test: verify action is referenceable as `uses: stephenleo/vibestats@v1` in a test workflow in a separate test repo | Integration/Manual | 1 | Dev | Manual — run once after Marketplace submission approved |

**Total P3:** 2 tests

---

## Execution Strategy

**Philosophy:** All YAML schema tests run on every PR — they are pure file-parsing with no I/O and complete in under 30 seconds. Integration tests (live tag push, Marketplace listing) are one-time manual verifications run before the first production release.

### Every PR

- All P0 schema tests (6 tests, <1 min)
- All P1 schema tests (7 tests, <1 min)
- All P2 schema/content tests (5 tests, <1 min)
- **Total PR gate:** ~18 tests, target <2 minutes

### On-Demand / Manual (Pre-Release)

- P3 end-to-end release smoke test (push tag to fork, verify GitHub Release artifacts)
- P3 Marketplace listing verification (after submission approval)

---

## Resource Estimates

| Priority | Count | Total Effort | Notes |
| --- | --- | --- | --- |
| P0 | 6 | ~8–14 hours | YAML parsing fixture setup, assertion logic for all three workflows |
| P1 | 7 | ~6–12 hours | Schema sweep, cross-tool assertion patterns |
| P2 | 5 | ~3–7 hours | Edge cases, content checks |
| P3 | 2 | ~1–3 hours | Manual exploratory runs |
| **Total** | **20** | **~18–36 hours** | **~1 week** |

**Prerequisites:**

- YAML parsing test infrastructure: Python `yaml` module (stdlib) or equivalent test tooling for parsing workflow files
- `release.yml` created in `.github/workflows/` (Story 8.1 backlog)
- `deploy-site.yml` created in `.github/workflows/` (Story 8.2 backlog)
- `action.yml` `branding` block confirmed populated (Story 8.3 backlog)
- `CONTRIBUTING.md` versioning section authored (Story 8.3 backlog)

---

## Quality Gate Criteria

### Pass/Fail Thresholds

- **P0 pass rate:** 100% (no exceptions; all distribution paths must be structurally correct before Epic 6 can proceed)
- **P1 pass rate:** ≥95% (failures require triage before merge)
- **P2/P3 pass rate:** ≥90% (informational)
- **R-001 to R-004 mitigations:** 100% complete before any story in Epic 8 is marked `done`

### Coverage Targets

- Workflow trigger configuration: 100% — every workflow file's `on:` key asserted
- Secret reference accuracy: 100% — every secret name asserted against expected value
- Matrix target completeness: 100% — all three platform targets asserted
- `action.yml` Marketplace fields: 100% — all required fields asserted before submission

### Non-Negotiable Requirements

- [ ] All P0 tests pass on every PR
- [ ] R-003 (deploy-site trigger) verified by YAML parse test before `deploy-site.yml` is merged
- [ ] R-004 (secret names) verified by schema test before `deploy-site.yml` is merged
- [ ] R-001 (fail-fast matrix) verified before first production release tag is pushed
- [ ] `action.yml` branding and metadata validated before Marketplace submission

---

## Mitigation Plans

### R-001: Partial release on platform build failure — silent partial GitHub Release (Score: 6)

**Mitigation Strategy:**
1. Assert `release.yml` matrix strategy includes `fail-fast: true`
2. Alternatively, assert jobs use explicit dependencies (`needs:`) so a failed build prevents the release upload job
3. Integration smoke test: push test tag to fork and verify all three archives appear before any are published as a draft release
4. Document in `CONTRIBUTING.md` that all three targets must succeed for a valid release

**Owner:** Dev/QA
**Timeline:** Before Story 8.1 PR merge
**Status:** Planned (Story 8.1 in backlog)
**Verification:** P0 schema test passes; pre-release smoke test (P3) confirms all three artifacts present

---

### R-002: Cross-compilation target misconfiguration (Score: 6)

**Mitigation Strategy:**
1. Schema test: parse `release.yml` matrix, assert exact target triple set matches `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`
2. Assert `cross` crate (or `cross-rs/cross` GitHub Action) is referenced for Linux target — no plain `cargo build` for cross-compilation
3. Document target triple requirements in architecture.md as authoritative source
4. Pre-release smoke test: verify each binary runs on its target platform

**Owner:** Dev/QA
**Timeline:** Before Story 8.1 PR merge
**Status:** Planned (Story 8.1 in backlog)
**Verification:** P0 matrix target assertion passes; P3 smoke test on fork

---

### R-003: deploy-site.yml accidental push/schedule trigger (Score: 6)

**Mitigation Strategy:**
1. Parse `deploy-site.yml` with Python `yaml` module in P0 test
2. Assert `on` key contains only `workflow_dispatch` — no `push`, `pull_request`, `schedule`, or `release` events
3. Assert `workflow_dispatch.inputs.ref` declared for manual ref selection
4. Add this check as required P0 CI gate

**Owner:** Dev/QA
**Timeline:** Before Story 8.2 PR merge
**Status:** Planned (Story 8.2 in backlog)
**Verification:** YAML trigger test passes in CI on every PR

---

### R-004: Incorrect or missing Cloudflare secret references in deploy-site.yml (Score: 6)

**Mitigation Strategy:**
1. Parse `deploy-site.yml` YAML and scan for all `secrets.*` references
2. Assert `secrets.CLOUDFLARE_API_TOKEN` and `secrets.CLOUDFLARE_ACCOUNT_ID` are present with exact casing
3. Assert no hardcoded token patterns in the file (regex scan for `[A-Za-z0-9_\-]{32,}` outside of `${{ secrets.* }}` context)
4. Document required secret names in story acceptance criteria and `CONTRIBUTING.md`

**Owner:** Dev/QA
**Timeline:** Before Story 8.2 PR merge
**Status:** Planned (Story 8.2 in backlog)
**Verification:** Schema secret-name assertion passes in CI; no plaintext token pattern found

---

## Assumptions and Dependencies

### Assumptions

1. YAML schema tests are implemented in Python using the stdlib `yaml` module — no additional test dependencies beyond those already established in Epic 5
2. `release.yml` will use `cross-rs/cross` or the `cross` crate for Linux cross-compilation, consistent with the architecture document
3. Cloudflare Pages secrets are named `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` in the GitHub repo settings (as specified in Story 8.2 AC)
4. The `v*` tag pattern is the authoritative release trigger (`v0.1.0`, `v1.0.0`, etc.) — no pre-release tag variants in scope for MVP
5. Epic 8 schema tests reuse the Python test runner established in Epic 5 (`pytest`)

### Dependencies

1. Story 8.1 (`release.yml`) created in `.github/workflows/` — required before P0 release pipeline tests can run
2. Story 8.2 (`deploy-site.yml`) created in `.github/workflows/` — required before P0 deploy trigger tests can run
3. `action.yml` `branding` block added — required before P1 Marketplace metadata test can run (Story 8.3)
4. `CONTRIBUTING.md` versioning section authored — required before P2 content test can run (Story 8.3)

### Risks to Plan

- **Risk:** GitHub Actions environment unavailable for fork-based P3 smoke test during development
  - **Impact:** Pre-release binary correctness not verified end-to-end
  - **Contingency:** Run smoke test on a throwaway fork or personal test repo before tagging first release; defer full verification to first actual release
- **Risk:** Cloudflare Pages requires account-specific configuration not visible in YAML schema tests
  - **Impact:** Deploy workflow passes schema tests but fails at runtime due to missing Pages project name
  - **Contingency:** Document required Cloudflare project name configuration in `CONTRIBUTING.md`; add as a P1 schema assertion if the value is deterministic

---

## Interworking & Regression

| Component | Impact | Regression Scope |
| --- | --- | --- |
| Epic 6 Bash installer (`install.sh`) | Directly depends on binary artifact URLs from GitHub Releases | Epic 8 release pipeline must produce artifacts matching the URL pattern that `install.sh` downloads; any naming change in `release.yml` breaks the installer |
| `action.yml` (Epic 5) | Marketplace metadata fields (`name`, `description`, `branding`) add to existing `runs` definition | Epic 5 schema tests assert `inputs` and `runs.using: composite`; Epic 8 adds branding assertions; no regression risk if additive |
| `.github/workflows/aggregate.yml` (Epic 5 template) | No dependency on Epic 8 workflows | No regression |
| vibestats.dev Astro site (Epic 7) | `deploy-site.yml` deploys the `site/` directory | Deployment workflow must reference `site/` working directory for `npm run build`; incorrect path would deploy empty or wrong output |

---

## Follow-on Workflows

- Run `*atdd` to generate failing P0 schema tests before implementation (TDD approach recommended for YAML validation)
- Run `*automate` once workflow files are implemented to expand schema coverage
- Run `*ci` to wire schema tests into the PR gate pipeline

---

## Appendix

### Knowledge Base References

- `risk-governance.md` — Risk classification framework (P×I scoring, gate decision rules)
- `probability-impact.md` — Risk scoring methodology (1–3 scale, DOCUMENT/MONITOR/MITIGATE/BLOCK)
- `test-levels-framework.md` — Unit vs. integration vs. E2E selection
- `test-priorities-matrix.md` — P0–P3 prioritisation criteria

### Related Documents

- PRD: `_bmad-output/planning-artifacts/prd.md` (FR41–FR43, NFR17)
- Epics: `_bmad-output/planning-artifacts/epics.md` (Epic 8, Stories 8.1–8.3)
- Architecture: `_bmad-output/planning-artifacts/architecture.md` (release.yml, deploy-site.yml layout, cross-compilation targets)
- Sprint Status: `_bmad-output/implementation-artifacts/sprint-status.yaml`
- action.yml: `action.yml` (repo root — community action definition)
- aggregate.yml: `.github/workflows/aggregate.yml` (Epic 5 user workflow template)

---

**Generated by:** BMad TEA Agent — Test Architect Module
**Workflow:** `bmad-testarch-test-design`
**Version:** 4.0 (BMad v6)
