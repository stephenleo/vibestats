# Epic 9: Post-Sprint Quality & Technical Debt

**GH Issue:** [#80](https://github.com/stephenleo/vibestats/issues/80)
**Status:** backlog
**Date Created:** 2026-04-12
**Source:** Synthesized from retrospectives for Epics 1–8

---

## Epic Goal

Address every actionable recommendation, technical debt item, and process gap surfaced across the eight Epic retrospectives. This epic has no new product features — it exists to close quality gaps, resolve pre-launch blockers, and leave the codebase in a state ready for production launch and ongoing maintenance.

## Priority Classification

| Priority | Stories |
|---|---|
| PRE-LAUNCH BLOCKER | 9.3 (test_6_2.bats failures) |
| HIGH | 9.1 (artifact hygiene), 9.2 (missing code reviews), 9.6 (first release + v1 tag) |
| MEDIUM | 9.4 (EXIT trap), 9.5 (dead_code suppressors), 9.7 (aggregate.yml concurrency) |
| LOW | 9.8 (architecture docs), 9.9 (Python hardening) |

## Dependencies

All stories in this epic are independent and can be worked in any order, with one exception: Story 9.6 (first release) should only run after Story 9.3 (installer test failures) is resolved, since a broken installer test suite should not be present on a released version.

## Stories

| # | Story | Priority | Source |
|---|---|---|---|
| 9.1 | [#81](https://github.com/stephenleo/vibestats/issues/81) Artifact hygiene — fix stale story statuses and missing records | HIGH | Epics 1–5, 7–8 retros |
| 9.2 | [#82](https://github.com/stephenleo/vibestats/issues/82) Retrospective code reviews for Stories 4.3 and 4.4 | HIGH | Epic 4 retro |
| 9.3 | [#83](https://github.com/stephenleo/vibestats/issues/83) Fix test_6_2.bats pre-existing failures (pre-launch blocker) | CRITICAL | Epic 6 retro |
| 9.4 | [#84](https://github.com/stephenleo/vibestats/issues/84) Bash installer — refactor EXIT trap to composable cleanup | MEDIUM | Epic 6 retro (deferred from Story 6.1) |
| 9.5 | [#85](https://github.com/stephenleo/vibestats/issues/85) Rust — remove dead_code suppressors and verify lint clean | MEDIUM | Epics 2–4 retros |
| 9.6 | [#86](https://github.com/stephenleo/vibestats/issues/86) First release — push v0.1.0 tag and create v1 floating tag | HIGH | Epic 8 retro |
| 9.7 | [#87](https://github.com/stephenleo/vibestats/issues/87) aggregate.yml — add concurrency group to prevent push conflicts | MEDIUM | Epic 5 retro (deferred from Story 5.4) |
| 9.8 | [#88](https://github.com/stephenleo/vibestats/issues/88) Architecture documentation — capture post-sprint lessons | LOW | Epics 1–2, 6 retros |
| 9.9 | [#89](https://github.com/stephenleo/vibestats/issues/89) Python script hardening — update_readme.py and aggregate.py improvements | LOW | Epics 5 retros |

## Completion Criteria

Epic 9 is complete when:
- All 9 stories are marked `done` in sprint-status.yaml
- `bats tests/installer/test_6_1.bats tests/installer/test_6_2.bats tests/installer/test_6_3.bats tests/installer/test_6_4.bats` passes with 0 failures
- `cargo clippy --all-targets -- -D warnings` passes with 0 warnings
- `v0.1.0` release is live on GitHub Releases with all three platform binaries
- `v1` floating tag exists and Marketplace listing is active
- No story file in implementation-artifacts shows `Status: review` while sprint-status.yaml shows `done`
