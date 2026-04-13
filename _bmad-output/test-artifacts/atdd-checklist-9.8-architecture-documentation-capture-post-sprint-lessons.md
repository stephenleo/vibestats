---
stepsCompleted: ['step-01-preflight-and-context', 'step-02-generation-mode', 'step-03-test-strategy', 'step-04-generate-tests', 'step-04c-aggregate', 'step-05-validate-and-complete']
lastStep: 'step-05-validate-and-complete'
lastSaved: '2026-04-13'
inputDocuments:
  - _bmad-output/implementation-artifacts/9-8-architecture-documentation-capture-post-sprint-lessons.md
  - _bmad/tea/config.yaml
  - _bmad-output/planning-artifacts/architecture.md
  - src/checkpoint.rs
  - Cargo.toml
  - install.sh
  - tests/installer/test_6_2.bats
  - tests/docs/test_9_8.bats
---

# ATDD Checklist: Story 9.8 — Architecture Documentation: Capture Post-Sprint Lessons

## TDD Red Phase (Current)

Tests generated and confirmed failing (9 of 18 tests fail — the 9 documentation-content tests).

- Documentation Tests: 9 tests (all failing — content not yet written)
- Accuracy / Integrity Tests: 9 tests (all passing — source facts verified)

## Acceptance Criteria Coverage

| AC | Description | Tests | Status |
|----|-------------|-------|--------|
| AC #1 | architecture.md contains "Known Gotchas & Conventions" section covering all 6 items | 9.8-DOC-001 through 9.8-DOC-009, 9.8-INT-003 | RED |
| AC #2 | serde(default) footgun documented with enough context to avoid it | 9.8-DOC-004, 9.8-DOC-005 | RED |
| AC #3 | All additions factually accurate (cross-checked against source) | 9.8-ACC-001 through 9.8-ACC-005 | GREEN (source facts confirmed) |

## Test Strategy

**Detected Stack:** `backend` — documentation-only story, no UI/API endpoints involved

**Execution Mode:** `sequential`

**Test Levels Used:**
- **Documentation content tests** (equivalent to unit tests): verify specific string patterns in architecture.md that correspond to each of the 6 gotcha items
- **Structural integrity tests** (equivalent to integration tests): verify existing content is preserved and ordering is correct
- **Factual accuracy tests** (equivalent to contract tests): verify source files contain the patterns that the documentation will reference

**No E2E or API tests generated** — this story has no UI or API surface. The test type is file content assertions using bats.

## Test File Location

`tests/docs/test_9_8.bats`

## Test Summary

| Test ID | Priority | Description | Phase |
|---------|----------|-------------|-------|
| 9.8-PRE-001 | P0 | architecture.md file exists | GREEN |
| 9.8-DOC-001 | P0 | Contains "Known Gotchas & Conventions" heading | RED |
| 9.8-DOC-002 | P0 | _redirects top-to-bottom evaluation order rule | RED |
| 9.8-DOC-003 | P1 | /u.html infinite-redirect trap | RED |
| 9.8-DOC-004 | P0 | serde(default = "fn") vs Default::default() footgun | RED |
| 9.8-DOC-005 | P0 | serde(default) does NOT affect Default::default() | RED |
| 9.8-DOC-006 | P1 | Cargo [workspace] worktree isolation pattern | RED |
| 9.8-DOC-007 | P1 | _gh() define-if-not-defined pattern | RED |
| 9.8-DOC-008 | P1 | Python3 stdlib over jq convention | RED |
| 9.8-DOC-009 | P1 | Security negative test pattern (sentinel) | RED |
| 9.8-ACC-001 | P1 | Cargo.toml line 1 is [workspace] | GREEN |
| 9.8-ACC-002 | P1 | src/checkpoint.rs has default_machine_status() | GREEN |
| 9.8-ACC-003 | P1 | install.sh has _gh() declare-if-not-defined guard | GREEN |
| 9.8-ACC-004 | P1 | install.sh uses python3 -c for JSON | GREEN |
| 9.8-ACC-005 | P1 | test_6_2.bats has SENTINEL_TOKEN pattern | GREEN |
| 9.8-INT-001 | P1 | "Architecture Readiness Assessment" section preserved | GREEN |
| 9.8-INT-002 | P1 | "Implementation Patterns" section preserved | GREEN |
| 9.8-INT-003 | P2 | Known Gotchas appears after Architecture Validation | RED |

**Totals: 18 tests — 9 RED (failing), 9 GREEN (passing)**

## Next Steps (TDD Green Phase)

After implementing the story (adding the "Known Gotchas & Conventions" section to architecture.md):

1. Run tests: `bats tests/docs/test_9_8.bats`
2. Verify all 18 tests PASS (green phase)
3. If any tests fail, fix the documentation content to match the test assertions
4. Commit the architecture.md changes with passing tests

## Implementation Guidance

The single file to edit is `_bmad-output/planning-artifacts/architecture.md`.

Append a new `## Known Gotchas & Conventions` section at the end of the file (after the Implementation Handoff section).

The section must include all 6 subsections with exact content patterns that will satisfy the failing tests:

1. **_redirects evaluation order**: Must include phrases matching `top.to.bottom|first.match|pass.through.*before|before.*catch.all|evaluation.order`
2. **/u.html infinite-redirect**: Must include `u\.html|infinite.redirect|clean.URL`
3. **serde(default) footgun**: Must include `serde(default = "` and `default_machine_status` as the reference example
4. **serde Default::default()**: Must include `Default` alongside serde content
5. **Cargo [workspace] isolation**: Must include `[workspace]|worktree.*cargo|cargo.*worktree`
6. **_gh() pattern**: Must include `declare -f|define-if-not-defined|_gh()`
7. **Python3 over jq**: Must include `python3.*json|jq.*python|Python3 stdlib|python3 -c`
8. **Security negative test**: Must include `SENTINEL|sentinel.*token|inject.*sentinel`

## Performance Report

- Execution Mode: SEQUENTIAL (documentation story — no parallel subagents needed)
- Test Generation: Single bats file, 18 tests
- TDD Phase: RED (9 content tests failing as expected)
