---
stepsCompleted: ['step-01-detect-mode', 'step-02-load-context', 'step-03-risk-and-testability', 'step-04-coverage-plan', 'step-05-generate-output']
lastStep: 'step-05-generate-output'
lastSaved: '2026-04-11'
mode: 'epic-level'
epic: 5
inputDocuments:
  - '_bmad-output/planning-artifacts/epics.md'
  - '_bmad-output/planning-artifacts/prd.md'
  - '_bmad-output/planning-artifacts/architecture.md'
  - '_bmad-output/implementation-artifacts/sprint-status.yaml'
  - 'action/aggregate.py'
  - 'action/generate_svg.py'
  - 'action/update_readme.py'
  - 'action.yml'
---

# Test Design: Epic 5 — GitHub Actions Pipeline

**Date:** 2026-04-12 (updated; originally 2026-04-11)
**Author:** Leo
**Status:** Active

---

## Executive Summary

**Scope:** Epic-level test design for Epic 5 — GitHub Actions Pipeline.

Epic 5 delivers the Python aggregation pipeline (aggregate.py, generate_svg.py, update_readme.py), the composite community GitHub Action (action.yml), and the user workflow template (aggregate.yml). Stories 5.1–5.3 are `done` (implemented and merged as of 2026-04-11); Stories 5.4 and 5.5 remain in `backlog`. The test design covers Stories 5.1–5.5 in full.

**Risk Summary:**

- Total risks identified: 10
- High-priority risks (≥6): 5
- Critical categories: DATA, SEC, OPS, BUS, TECH

**Coverage Summary:**

- P0 scenarios: 9 (~15–25 hours)
- P1 scenarios: 12 (~12–22 hours)
- P2 scenarios: 7 (~4–10 hours)
- P3 scenarios: 3 (~1–3 hours)
- **Total effort:** ~32–60 hours (~1–2 weeks)

---

## Not in Scope

| Item | Reasoning | Mitigation |
| --- | --- | --- |
| Dashboard rendering (FR27–31) | Epic 7 scope | Covered in Epic 7 test design |
| Bash installer (Epic 6) | Epic 6 scope | Covered in Epic 6 test design |
| Rust binary sync logic | Epics 2–4 scope | Covered in prior epic test designs |
| Performance of client-side dashboard | NFR4 — vibestats.dev | Covered in Epic 7 test design |
| GitHub Actions Marketplace publication | Epic 8 scope | Covered in Epic 8 test design |

---

## Risk Assessment

> P0/P1/P2/P3 = priority and risk level, NOT execution timing.

### High-Priority Risks (Score ≥6)

| Risk ID | Category | Description | Probability | Impact | Score | Mitigation | Owner | Timeline |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| R-001 | DATA | `aggregate.py` sums data incorrectly — double-counting machine entries or failing to skip `purged` registry machines — corrupting the public `data.json` | 2 | 3 | 6 | Unit tests covering multi-machine merge, purged-machine skip, and idempotent re-run; assert output against known fixture | Dev/QA | Pre-merge |
| R-002 | SEC | Raw machine JSON paths or hostnames leak into the public `username/username` repo, violating NFR8/NFR9 data boundary | 2 | 3 | 6 | Test that `data.json` output contains only `generated_at`, `username`, and `days` with numeric values; no machine IDs, paths, or hostnames | Dev/QA | Pre-merge |
| R-003 | OPS | Action step failure (any script) still commits partial outputs, leaving SVG/data.json in inconsistent state (violates NFR13) | 2 | 3 | 6 | Integration test: simulate mid-pipeline failure and assert no commit is made; verify action exits non-zero on any step error | Dev/QA | Pre-merge |
| R-004 | BUS | Empty commit created when README content has not changed between runs, polluting git history (NFR13 resilience) | 3 | 2 | 6 | Unit test: assert `update_readme.py` detects no change and skips commit step; test with identical input twice | Dev/QA | Pre-merge |
| R-005 | TECH | `aggregate.yml` per-push trigger accidentally added, exhausting GitHub Actions free-tier minutes (NFR5: ≤60 min/month) | 2 | 3 | 6 | Schema-level test: parse `aggregate.yml` and assert only `schedule` + `workflow_dispatch` triggers are present; no `push` or `pull_request` triggers | Dev/QA | Pre-merge |

### Medium-Priority Risks (Score 3–4)

| Risk ID | Category | Description | Probability | Impact | Score | Mitigation | Owner |
| --- | --- | --- | --- | --- | --- | --- | --- |
| R-006 | BUS | `generate_svg.py` produces SVG that GitHub's DOMPurify strips (contains JS or unsupported elements), breaking README embed (NFR17) | 2 | 2 | 4 | Assert SVG uses only `xml.etree.ElementTree`; no `<script>`, event handlers, or external refs; validate output is well-formed XML | Dev/QA |
| R-007 | DATA | `update_readme.py` silently replaces entire README if markers are absent instead of exiting non-zero | 2 | 2 | 4 | Unit test: pass a README missing markers, assert non-zero exit and clear error message | Dev/QA |
| R-008 | TECH | `action.yml` missing required `token` or `profile-repo` inputs — users get cryptic errors during Action run | 2 | 2 | 4 | Schema test: parse `action.yml`, assert `inputs.token` and `inputs.profile-repo` declared; verify `type: composite` | Dev/QA |
| R-009 | OPS | `aggregate.py` exits zero on error, masking failures and allowing broken state to be committed (Story 5.1 AC) | 2 | 2 | 4 | Unit test: inject malformed fixture data, assert non-zero exit | Dev/QA |

### Low-Priority Risks (Score 1–2)

| Risk ID | Category | Description | Probability | Impact | Score | Action |
| --- | --- | --- | --- | --- | --- | --- |
| R-010 | TECH | SVG grid dimensions do not match 52-columns × 7-rows GitHub contribution graph shape | 1 | 2 | 2 | Monitor — verify in visual snapshot test (P2) |

### Risk Category Legend

- **TECH**: Technical/Architecture (integration, structure, schema)
- **SEC**: Security (data exposure, boundary enforcement)
- **PERF**: Performance (SLA violations, resource limits)
- **DATA**: Data Integrity (loss, corruption, incorrect aggregation)
- **BUS**: Business Impact (UX harm, logic errors)
- **OPS**: Operations (deployment, pipeline resilience, config)

---

## Entry Criteria

- [x] Stories 5.1–5.5 requirements and acceptance criteria agreed upon by Dev and QA
- [x] Python scripts (`aggregate.py`, `generate_svg.py`, `update_readme.py`) implemented and accessible (Stories 5.1–5.3 done)
- [x] Test fixture directory `action/tests/fixtures/` populated with representative Hive partition data and a sample `registry.json`
- [x] Python stdlib-only constraint confirmed (no pip dependencies)
- [ ] `action.yml` composite action declared at repo root (Story 5.4 — backlog)
- [ ] `aggregate.yml` workflow template created (Story 5.5 — backlog)

## Exit Criteria

- [ ] All P0 tests passing (100%)
- [ ] All P1 tests passing (≥95%)
- [ ] No open DATA or SEC category risks unmitigated
- [ ] R-001 through R-005 mitigations verified via automated tests
- [ ] `data.json` output validated against public schema on every CI run

---

## Test Coverage Plan

> P0/P1/P2/P3 = priority and risk level, NOT execution timing. Execution timing is defined in the Execution Strategy section.

### P0 (Critical)

**Criteria:** Blocks core pipeline correctness + High risk (≥6) + No workaround

| Requirement | Test Level | Risk Link | Test Count | Owner | Notes |
| --- | --- | --- | --- | --- | --- |
| aggregate.py: sums sessions+active_minutes across multiple machines for same date (Story 5.1 AC) | Unit | R-001 | 2 | Dev/QA | Fixtures: two machines, same date; assert summed values |
| aggregate.py: skips `purged` registry machines entirely (Story 5.1 AC) | Unit | R-001 | 1 | Dev/QA | Fixture includes purged machine; assert its data absent from output |
| aggregate.py: output conforms to public schema — no machine IDs, paths, or hostnames (Story 5.1 AC, NFR8/NFR9) | Unit | R-002 | 2 | Dev/QA | Assert exact key set: `generated_at`, `username`, `days`; values numeric |
| aggregate.py: exits non-zero on any error (Story 5.1 AC) | Unit | R-009 | 1 | Dev/QA | Inject missing or malformed data; assert exit code ≠ 0 |
| update_readme.py: skips git commit when content unchanged (Story 5.3 AC, NFR13) | Unit | R-004 | 1 | Dev/QA | Run twice with identical input; assert commit step skipped on second run |
| aggregate.yml: only `schedule` + `workflow_dispatch` triggers present — no push/PR triggers (Story 5.5 AC, NFR5) | Schema/Unit | R-005 | 1 | Dev/QA | Parse YAML; assert no `push` or `pull_request` event |
| action.yml: any step failure exits non-zero, no partial commit (Story 5.4 AC, NFR13) | Integration | R-003 | 1 | Dev/QA | Mock a failing step; assert workflow exits non-zero |

**Total P0:** 9 tests

### P1 (High)

**Criteria:** Core pipeline behaviours + Medium–high risk (3–5) + Common paths

| Requirement | Test Level | Risk Link | Test Count | Owner | Notes |
| --- | --- | --- | --- | --- | --- |
| aggregate.py: merges Hive partition data from multiple machines and dates (end-to-end data flow) | Integration | R-001 | 2 | Dev/QA | Multi-date, multi-machine fixtures |
| aggregate.py: single machine — baseline happy path, correct output file written | Unit | R-001 | 1 | Dev/QA | Minimal fixture |
| generate_svg.py: produces well-formed XML SVG (no script elements, no JS event handlers) | Unit | R-006 | 2 | Dev/QA | Parse output with `xml.etree.ElementTree`; assert no `<script>` |
| generate_svg.py: uses Claude orange palette (`#fef3e8` → `#f97316`) for activity cells | Unit | R-006 | 1 | Dev/QA | Inspect cell `fill` attributes in output SVG |
| update_readme.py: replaces content between vibestats markers correctly (Story 5.3 AC) | Unit | R-007 | 2 | Dev/QA | Fixture README with markers; assert replaced content matches expected |
| update_readme.py: exits non-zero with clear error when markers are missing (Story 5.3 AC) | Unit | R-007 | 1 | Dev/QA | No-marker fixture; assert non-zero exit and error message |
| action.yml: declares `token` and `profile-repo` inputs; type is `composite` (Story 5.4 AC, NFR17) | Schema | R-008 | 1 | Dev/QA | Parse action.yml YAML |
| action.yml: step sequence — checkout → aggregate → generate_svg → update_readme → commit/push (Story 5.4 AC) | Schema/Integration | R-003 | 2 | Dev/QA | Assert ordered steps in composite action |

**Total P1:** 12 tests

### P2 (Medium)

**Criteria:** Secondary pipeline behaviours + Low–medium risk (1–4) + Edge cases

| Requirement | Test Level | Risk Link | Test Count | Owner | Notes |
| --- | --- | --- | --- | --- | --- |
| generate_svg.py: grid shape — 52 columns × 7 rows per year (Story 5.2 AC) | Unit | R-010 | 1 | Dev/QA | Count `<rect>` elements in SVG output |
| generate_svg.py: zero-activity days use neutral colour (not orange) | Unit | R-010 | 1 | Dev/QA | Fixture with zero-activity date; assert neutral fill |
| aggregate.py: multiple harness directories (`harness=claude`, `harness=codex`) aggregated correctly | Unit | R-001 | 1 | Dev/QA | Multi-harness fixture |
| aggregate.py: empty Hive partition directory — produces empty days map, not error | Unit | R-009 | 1 | Dev/QA | Assert empty `days: {}` output |
| update_readme.py: `<img>` tag points to correct `raw.githubusercontent.com` URL (Story 5.3 AC) | Unit | - | 1 | Dev/QA | Assert URL pattern in output |
| aggregate.yml: includes `workflow_dispatch` trigger for manual runs (Story 5.5 AC, FR26) | Schema | R-005 | 1 | Dev/QA | Parse YAML; assert event present |
| aggregate.yml: calls `uses: stephenleo/vibestats@v1` with correct inputs (Story 5.5 AC) | Schema | R-005 | 1 | Dev/QA | Parse YAML; assert action reference and secret input |

**Total P2:** 7 tests

### P3 (Low)

**Criteria:** Nice-to-have + Exploratory + Benchmarks

| Requirement | Test Level | Test Count | Owner | Notes |
| --- | --- | --- | --- | --- |
| generate_svg.py: visual snapshot of SVG output (pixel-level regression) | Unit/Snapshot | 1 | Dev | Compare SVG against golden file |
| aggregate.py: large dataset performance — 12 months × 3 machines completes in <10s | Unit/Benchmark | 1 | Dev | Informational only; not a hard gate |
| action.yml: idempotent — running the action twice produces identical `data.json` output | Integration | 1 | Dev/QA | Exploratory; manual verification acceptable |

**Total P3:** 3 tests

---

## Execution Strategy

**Philosophy:** Run all Python unit and schema tests on every PR. They are fast (stdlib-only, no I/O) and should complete in under 2 minutes. Reserve integration tests (mock GitHub API) for nightly if they require network setup.

### Every PR

- All P0 unit and schema tests (~9 tests, <2 min)
- All P1 unit and schema tests (~12 tests, <2 min)
- All P2 schema and unit tests (~7 tests, <1 min)
- **Total PR gate:** ~28 tests, target <5 minutes

### Nightly

- P1 integration tests requiring mocked GitHub API calls
- P3 benchmark tests

### On-Demand / Manual

- P3 visual snapshot regression (run before major SVG changes)
- Full end-to-end Action run against test `vibestats-data` repo

---

## Resource Estimates

| Priority | Count | Total Effort | Notes |
| --- | --- | --- | --- |
| P0 | 9 | ~15–25 hours | Fixture setup, schema validation, error path coverage |
| P1 | 12 | ~12–22 hours | Core pipeline paths, XML parsing |
| P2 | 7 | ~4–10 hours | Edge cases, YAML schema checks |
| P3 | 3 | ~1–3 hours | Exploratory and benchmarks |
| **Total** | **31** | **~32–60 hours** | **~1–2 weeks** |

**Prerequisites:**

- Test fixtures: representative Hive partition directory tree under `action/tests/fixtures/sample_machine_data/` with at least 3 machines (2 active, 1 purged) and at least 5 dates
- `registry.json` fixture with active and purged entries
- `README_with_markers.md` and `README_without_markers.md` fixtures
- Expected `data.json` golden file for schema assertions
- Python test runner: `pytest` (stdlib test discovery acceptable; no pip installs required beyond dev tooling)

---

## Quality Gate Criteria

### Pass/Fail Thresholds

- **P0 pass rate:** 100% (no exceptions; pipeline must not proceed on failure)
- **P1 pass rate:** ≥95% (failures require triage before merge)
- **P2/P3 pass rate:** ≥90% (informational)
- **R-001 to R-005 mitigations:** 100% complete before Epic 5 stories are marked `done`

### Coverage Targets

- Data aggregation logic: ≥80% branch coverage
- Security boundary (NFR8/NFR9): 100% — every output field asserted
- README marker handling: 100% of acceptance criteria covered
- Schema validation (action.yml, aggregate.yml): 100%

### Non-Negotiable Requirements

- [ ] All P0 tests pass on every PR
- [ ] R-002 (data boundary) verified by automated schema assertion
- [ ] R-005 (no per-push trigger) verified by YAML parse test
- [ ] `aggregate.py` exits non-zero on any error path

---

## Mitigation Plans

### R-001: aggregate.py data corruption via incorrect merge logic (Score: 6)

**Mitigation Strategy:**
1. Write parameterised unit tests with multi-machine, multi-date Hive fixtures
2. Assert output `days` values equal sum of machine values for each date
3. Add fixture for purged machine and assert its data absent from output
4. Run idempotency check: run aggregation twice on same fixtures, assert identical output

**Owner:** Dev/QA
**Timeline:** Before Story 5.1 PR merge
**Status:** Implemented (Story 5.1 done; tests exist in `action/tests/test_aggregate.py`)
**Verification:** All R-001 linked P0 tests passing in CI

---

### R-002: raw machine data leaking into public repo (Score: 6)

**Mitigation Strategy:**
1. Assert `data.json` output key set is exactly `{"generated_at", "username", "days"}`
2. Assert each `days` entry value has only `sessions` (int) and `active_minutes` (int)
3. Assert no string values in `days` entries (no machine IDs, paths, hostnames)
4. Run assertion as part of P0 CI gate

**Owner:** Dev/QA
**Timeline:** Before Story 5.1 PR merge
**Status:** Implemented (Story 5.1 done; schema assertion test in `action/tests/test_aggregate.py`)
**Verification:** Schema assertion test in P0 suite passes on every PR

---

### R-003: partial commit on step failure (Score: 6)

**Mitigation Strategy:**
1. In `action.yml`, verify each step uses `shell: bash` with `set -e`
2. Integration test: mock the `update_readme.py` step to fail; assert action exits non-zero and no commit step was reached
3. Document expected behaviour in story acceptance criteria

**Owner:** Dev/QA
**Timeline:** Before Story 5.4 PR merge
**Status:** Planned (Story 5.4 in backlog)
**Verification:** Integration test passes; manual smoke test on test repo

---

### R-004: empty commits when content unchanged (Score: 6)

**Mitigation Strategy:**
1. Unit test: call `update_readme.py` twice with same input; assert commit-skipped signal on second call
2. Review implementation to confirm hash/diff comparison before committing
3. Assert no empty commits in integration test

**Owner:** Dev/QA
**Timeline:** Before Story 5.3 PR merge
**Status:** Implemented (Story 5.3 done; tests exist in `action/tests/test_update_readme.py`)
**Verification:** P0 unit test for idempotent run passes

---

### R-005: accidental per-push trigger in aggregate.yml (Score: 6)

**Mitigation Strategy:**
1. Parse `aggregate.yml` with Python `yaml` module in test
2. Assert `on` key contains only `schedule` and `workflow_dispatch` — no `push`, `pull_request`, `release`, or wildcard triggers
3. Add this check as a required P0 CI gate

**Owner:** Dev/QA
**Timeline:** Before Story 5.5 PR merge
**Status:** Planned (Story 5.5 in backlog)
**Verification:** YAML schema test passes in CI on every PR

---

## Assumptions and Dependencies

### Assumptions

1. All three Python scripts use Python stdlib only — no external pip installs in the action runtime (confirmed in architecture.md and ADR-8)
2. `registry.json` is present in `vibestats-data` root and its `status` field is the authoritative source for `purged` machines
3. The `action.yml` composite action runs on `ubuntu-latest` (NFR17)
4. Fixture data will be generated by Dev prior to QA writing tests
5. `pytest` will be used as the test runner for the Python action scripts (dev dependency only)

### Dependencies

1. Story 5.1 (`aggregate.py`) implementation complete — required before P0 aggregation tests can be written
2. Story 5.2 (`generate_svg.py`) implementation complete — required before SVG structure tests can run
3. Story 5.3 (`update_readme.py`) implementation complete — required before marker-replacement tests can run
4. `action/tests/fixtures/` populated with representative multi-machine Hive partition data — required before integration tests

### Risks to Plan

- **Risk:** Story 5.4 integration test requires a mock GitHub API — complexity may push to nightly suite
  - **Impact:** P0 gate may not include full integration coverage on PR
  - **Contingency:** Mock using Python `unittest.mock`; keep integration test in PR gate using local mocks only

---

## Interworking & Regression

| Component | Impact | Regression Scope |
| --- | --- | --- |
| `aggregate.py` | Reads Hive partitions written by Rust binary (Epics 2–3) | Fixture schema must match `machines/year=Y/month=M/day=D/harness=H/machine_id=ID/data.json` layout confirmed in architecture.md |
| `data.json` schema | Consumed by Astro dashboard (Epic 7) | Public schema test validates `generated_at`, `username`, `days` structure; no regression risk until Epic 7 is implemented |
| `action.yml` | Invoked from user's `aggregate.yml` | Step ordering tests confirm correct invocation sequence |
| `heatmap.svg` | Embedded in profile README via `<img>` tag | SVG structure test confirms no JS/script elements that GitHub DOMPurify would strip |

---

## Follow-on Workflows

- Run `*atdd` to generate failing P0 tests before implementation (TDD approach recommended for aggregation logic)
- Run `*automate` once all scripts are implemented to expand coverage

---

## Appendix

### Knowledge Base References

- `risk-governance.md` — Risk classification framework (P×I scoring, gate decision rules)
- `probability-impact.md` — Risk scoring methodology (1–3 scale, DOCUMENT/MONITOR/MITIGATE/BLOCK)
- `test-levels-framework.md` — Unit vs. integration vs. E2E selection
- `test-priorities-matrix.md` — P0–P3 prioritisation criteria

### Related Documents

- PRD: `_bmad-output/planning-artifacts/prd.md`
- Epics: `_bmad-output/planning-artifacts/epics.md` (Epic 5, pp. 615–741)
- Architecture: `_bmad-output/planning-artifacts/architecture.md`
- Sprint Status: `_bmad-output/implementation-artifacts/sprint-status.yaml`

---

**Generated by:** BMad TEA Agent — Test Architect Module
**Workflow:** `bmad-testarch-test-design`
**Version:** 4.0 (BMad v6)
