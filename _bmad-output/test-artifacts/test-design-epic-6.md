---
stepsCompleted: ['step-01-detect-mode', 'step-02-load-context', 'step-03-risk-and-testability', 'step-04-coverage-plan', 'step-05-generate-output']
lastStep: 'step-05-generate-output'
lastSaved: '2026-04-12'
mode: 'epic-level'
epic: 6
inputDocuments:
  - '_bmad-output/planning-artifacts/epics.md'
  - '_bmad-output/planning-artifacts/prd.md'
  - '_bmad-output/planning-artifacts/architecture.md'
  - '_bmad-output/implementation-artifacts/sprint-status.yaml'
  - 'install.sh'
---

# Test Design: Epic 6 — Bash Installer

**Date:** 2026-04-12
**Author:** Leo
**Status:** Active

---

## Executive Summary

**Scope:** Epic-level test design for Epic 6 — Bash Installer.

Epic 6 delivers `install.sh`: a single-command Bash installer that detects and installs `gh`, authenticates with GitHub, detects first-install vs. multi-machine paths, creates repositories and secrets, configures Claude Code hooks, injects README markers, and triggers a post-install backfill. All four stories (6.1–6.4) are in `backlog`; `install.sh` is a stub (`echo "TODO: implement installer" && exit 1`). Epic 6 is the last epic in the implementation order (depends on Epic 8 binary release and all prior epics).

**Risk Summary:**

- Total risks identified: 10
- High-priority risks (≥6): 5
- Critical categories: SEC, OPS, DATA, BUS, TECH

**Coverage Summary:**

- P0 scenarios: 8 (~14–22 hours)
- P1 scenarios: 11 (~10–18 hours)
- P2 scenarios: 6 (~3–8 hours)
- P3 scenarios: 2 (~1–3 hours)
- **Total effort:** ~28–51 hours (~1–2 weeks)

---

## Not in Scope

| Item | Reasoning | Mitigation |
| --- | --- | --- |
| Rust binary sync logic | Covered by Epics 2–4 test designs | Prior epic test designs remain authoritative |
| GitHub Action aggregation pipeline | Covered by Epic 5 test design | Existing P0 suite covers aggregate/SVG/README scripts |
| vibestats.dev dashboard | Covered by Epic 7 test design | Playwright tests cover client-side heatmap |
| CI/CD release pipeline | Covered by Epic 8 test design | Release workflow schema tests cover binary publishing |
| `vibestats auth` refresh flow (post-install) | FR40 — runtime CLI command, not installer | Covered in Epic 4 CLI tests |
| Windows support | Architecture explicitly targets macOS + Linux only | Out of scope per NFR and architecture.md |

---

## Risk Assessment

> P0/P1/P2/P3 = priority and risk level, NOT execution timing.

### High-Priority Risks (Score ≥6)

| Risk ID | Category | Description | Probability | Impact | Score | Mitigation | Owner | Timeline |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| R-001 | SEC | `VIBESTATS_TOKEN` (fine-grained PAT) is written to disk or captured in shell history during generation, violating NFR7 (token never on disk) | 2 | 3 | 6 | Test that `VIBESTATS_TOKEN` is piped directly to `gh secret set` and never appears in any file under `$HOME`; verify no token echoed to stdout | Dev/QA | Pre-merge |
| R-002 | SEC | `~/.config/vibestats/config.toml` created with permissions wider than `600`, exposing machine-side token to other OS users (NFR6) | 2 | 3 | 6 | Assert file permissions of `config.toml` equal `0600` immediately after installer writes the machine-side token; test on both macOS and Linux | Dev/QA | Pre-merge |
| R-003 | OPS | Installer silently continues past a failed step (e.g., repo creation failure, secret set failure) leaving the system in a partial-install state | 2 | 3 | 6 | Test that installer exits non-zero and prints a clear error message on `gh` command failures; use `set -euo pipefail` assertion and mock failure injection | Dev/QA | Pre-merge |
| R-004 | BUS | Installer runs the first-install path on a machine where `vibestats-data` already exists, re-generating `VIBESTATS_TOKEN` and overwriting the Actions secret mid-flight on an active installation | 3 | 2 | 6 | Test: mock `gh repo view` returning success → installer detects existing repo, skips repo creation and secret setup, and proceeds only to machine registration (Story 6.3 AC, FR5) | Dev/QA | Pre-merge |
| R-005 | DATA | `registry.json` machine entry is written with incorrect or missing fields (`machine_id`, `hostname`, `status`, `last_seen`), breaking the aggregation pipeline's ability to identify and purge machines | 2 | 3 | 6 | Test: assert `registry.json` produced by installer contains valid JSON with all four required fields; `status = "active"`, `last_seen` is ISO 8601 UTC timestamp; validate against schema | Dev/QA | Pre-merge |

### Medium-Priority Risks (Score 3–4)

| Risk ID | Category | Description | Probability | Impact | Score | Mitigation | Owner |
| --- | --- | --- | --- | --- | --- | --- | --- |
| R-006 | TECH | `gh` version check fails silently or passes a version below 2.0, allowing the installer to proceed and then fail with a cryptic error on first `gh api` call (NFR16) | 2 | 2 | 4 | Test: mock `gh --version` returning `1.14.0`; assert installer prints a warning and exits non-zero with actionable message | Dev/QA |
| R-007 | OPS | Platform/arch detection (`uname -s` + `uname -m`) produces an incorrect binary download URL, causing a failed download or silent installation of the wrong binary | 2 | 2 | 4 | Test: stub `uname` outputs for `Darwin arm64`, `Darwin x86_64`, and `Linux x86_64`; assert correct `.tar.gz` archive name selected for each | Dev/QA |
| R-008 | BUS | Hook configuration writes duplicate `Stop` or `SessionStart` entries to `~/.claude/settings.json` on re-run, causing multiple sync processes to fire per session | 2 | 2 | 4 | Test: run hook configuration step twice on the same `settings.json`; assert exactly one `Stop` and one `SessionStart` hook entry present after second run | Dev/QA |
| R-009 | BUS | README marker injection fails silently if `username/username` profile repo does not exist, leaving no visible error and no markers added (FR9) | 2 | 2 | 4 | Test: mock `gh api` returning 404 for the profile repo; assert installer prints a clear warning with manual fix instructions rather than exiting silently | Dev/QA |

### Low-Priority Risks (Score 1–2)

| Risk ID | Category | Description | Probability | Impact | Score | Action |
| --- | --- | --- | --- | --- | --- | --- |
| R-010 | TECH | Checksum verification of the downloaded binary archive fails on first run due to a stale release asset reference, blocking install for new users | 1 | 2 | 2 | Monitor — verify checksum assertion in integration smoke test (P3) |

### Risk Category Legend

- **TECH**: Technical/Architecture (integration, structure, schema)
- **SEC**: Security (data exposure, boundary enforcement, token handling)
- **PERF**: Performance (SLA violations, resource limits)
- **DATA**: Data Integrity (loss, corruption, incorrect schema)
- **BUS**: Business Impact (UX harm, logic errors, incorrect flow selection)
- **OPS**: Operations (deployment, pipeline resilience, config, partial-install)

---

## Entry Criteria

- [ ] Stories 6.1–6.4 requirements and acceptance criteria agreed upon by Dev and QA
- [ ] `install.sh` implementation in progress or complete (currently stub)
- [ ] Epic 8 binary release available at a stable GitHub Release URL (dependency — Epic 8 is `done`)
- [ ] Mock `gh` CLI harness (shell function overrides) available for unit-style shell tests
- [ ] `registry.json` schema documented and accessible (defined in Epic 1, Story 1.4)
- [ ] `~/.claude/settings.json` hook schema documented (defined in Epics 3–4)

## Exit Criteria

- [ ] All P0 tests passing (100%)
- [ ] All P1 tests passing (≥95%)
- [ ] No open SEC or OPS category risks unmitigated
- [ ] R-001 through R-005 mitigations verified via automated tests
- [ ] `registry.json` output validated against schema on every CI run
- [ ] Installer tested on macOS arm64 and Linux x86_64 (minimum two platforms)

---

## Test Coverage Plan

> P0/P1/P2/P3 = priority and risk level, NOT execution timing. Execution timing is defined in the Execution Strategy section.

### P0 (Critical)

**Criteria:** Blocks core installer correctness + High risk (≥6) + No workaround

| Requirement | Test Level | Risk Link | Test Count | Owner | Notes |
| --- | --- | --- | --- | --- | --- |
| `VIBESTATS_TOKEN` never written to disk — piped directly from `gh api` to `gh secret set` without intermediate file (Story 6.2 AC, NFR7) | Unit/Shell | R-001 | 2 | Dev/QA | Mock `gh api` to emit a test token; scan `$HOME` and temp dirs for the token value after installer run; assert absent |
| `~/.config/vibestats/config.toml` has permissions `600` after machine-side token written (Story 6.2 AC, NFR6) | Unit/Shell | R-002 | 1 | Dev/QA | Check `stat` output after installer completes; assert octal `0600` |
| Installer exits non-zero and prints error on `gh` command failure (any step) — no silent continuation (Stories 6.1–6.4 AC, OPS) | Unit/Shell | R-003 | 2 | Dev/QA | Mock `gh repo create` to exit 1; assert installer exits non-zero with message; also test `gh secret set` failure path |
| Multi-machine path: installer skips repo creation, workflow write, and `VIBESTATS_TOKEN` setup when `vibestats-data` already exists (Story 6.3 AC, FR5) | Unit/Shell | R-004 | 1 | Dev/QA | Mock `gh repo view` returning success; assert skipped steps via spy on `gh` calls |
| `registry.json` entry contains all required fields: `machine_id`, `hostname`, `status = "active"`, `last_seen` ISO 8601 UTC (Stories 6.2–6.3 AC, FR6) | Unit/Shell | R-005 | 2 | Dev/QA | Parse JSON output; assert key set and value types; validate `last_seen` format |

**Total P0:** 8 tests

### P1 (High)

**Criteria:** Core installer behaviours + Medium–high risk (3–5) + Common paths

| Requirement | Test Level | Risk Link | Test Count | Owner | Notes |
| --- | --- | --- | --- | --- | --- |
| `gh` not installed → installer installs via `brew install gh` (macOS) or `apt-get install gh` (Debian/Ubuntu) (Story 6.1 AC, FR2) | Unit/Shell | R-006 | 2 | Dev/QA | Stub `which gh` returning empty; assert correct install command called per platform |
| `gh` version < 2.0 → installer prints warning and exits non-zero with clear message (Story 6.1 AC, NFR16) | Unit/Shell | R-006 | 1 | Dev/QA | Mock `gh --version` returning `1.14.0`; assert non-zero exit and message |
| `gh` installed but not authenticated → installer runs `gh auth login` (Story 6.1 AC, FR3) | Unit/Shell | - | 1 | Dev/QA | Mock `gh auth status` returning non-zero; assert `gh auth login` called |
| Platform detection: correct binary archive selected for macOS arm64, macOS x86_64, Linux x86_64 (Story 6.1 AC) | Unit/Shell | R-007 | 3 | Dev/QA | Stub `uname -s` and `uname -m` per platform; assert correct `.tar.gz` archive name |
| First-install: `vibestats-data` created as private repo via `gh repo create` (Story 6.2 AC, FR4) | Unit/Shell | - | 1 | Dev/QA | Mock `gh repo view` returning 404; assert `gh repo create --private` called |
| First-install: `aggregate.yml` written into `vibestats-data/.github/workflows/` calling `stephenleo/vibestats@v1` (Story 6.2 AC, FR7) | Unit/Shell | - | 1 | Dev/QA | Inspect written file content; assert action reference matches |
| Machine-side token stored in `~/.config/vibestats/config.toml` via `gh auth token` (Story 6.2 AC, FR39) | Unit/Shell | R-002 | 1 | Dev/QA | Assert config.toml exists and contains token field; assert permissions `600` |
| Hook configuration writes exactly one `Stop` hook and one `SessionStart` hook to `~/.claude/settings.json` (Story 6.4 AC, FR8) | Unit/Shell | R-008 | 1 | Dev/QA | Parse `settings.json` after installer run; count hook entries by type |

**Total P1:** 11 tests

### P2 (Medium)

**Criteria:** Secondary installer behaviours + Low–medium risk (1–4) + Edge cases

| Requirement | Test Level | Risk Link | Test Count | Owner | Notes |
| --- | --- | --- | --- | --- | --- |
| Hook configuration is idempotent — running installer twice on the same `settings.json` does not duplicate hooks (Story 6.4 AC, FR8) | Unit/Shell | R-008 | 1 | Dev/QA | Run hook setup twice; assert exactly one entry per hook type |
| README marker injection: `<!-- vibestats-start -->` and `<!-- vibestats-end -->` added with SVG `<img>` embed and dashboard link (Story 6.4 AC, FR9) | Unit/Shell | R-009 | 1 | Dev/QA | Mock `gh api` returning sample README content; assert markers present with correct embed URL |
| README marker injection: installer prints warning (not error) and continues if profile repo does not exist (Story 6.4 AC) | Unit/Shell | R-009 | 1 | Dev/QA | Mock `gh api` returning 404; assert installer continues and warns |
| Post-install backfill: `vibestats sync --backfill` is the final installer step (Story 6.4 AC, FR11) | Unit/Shell | - | 1 | Dev/QA | Spy on `vibestats` binary call; assert `sync --backfill` is last command after all setup |
| `gh` already installed and version ≥ 2.0 — no install attempted (Story 6.1 AC, FR2) | Unit/Shell | R-006 | 1 | Dev/QA | Mock `which gh` returning a path and `gh --version` returning `2.44.1`; assert no `brew install` or `apt-get` called |
| `vibestats-data` repo detection: `gh repo view` used with the user's account (not a hardcoded org), and exact repo name `vibestats-data` matched (Story 6.3 AC, FR5) | Unit/Shell | R-004 | 1 | Dev/QA | Inspect `gh repo view` call arguments; assert correct repo reference |

**Total P2:** 6 tests

### P3 (Low)

**Criteria:** Nice-to-have + Exploratory + End-to-end smoke

| Requirement | Test Level | Test Count | Owner | Notes |
| --- | --- | --- | --- | --- |
| Checksum verification of downloaded binary archive passes on a real GitHub Release asset | E2E Smoke | 1 | Dev | Manual or CI-triggered against actual release; requires Epic 8 binary available |
| Full end-to-end first-install run on macOS arm64 and Linux x86_64 in an isolated sandbox environment | E2E | 1 | Dev/QA | Run against a throwaway GitHub account; verify heatmap appears within 5 minutes of install |

**Total P3:** 2 tests

---

## Execution Strategy

**Philosophy:** Shell unit tests (mocked `gh` overrides) run on every PR — they are fast, deterministic, and catch the highest-risk installer paths without network access. End-to-end integration tests require a live GitHub account and are limited to pre-release validation only.

### Every PR

- All P0 shell unit tests (~8 tests, <2 min)
- All P1 shell unit tests (~11 tests, <3 min)
- All P2 shell unit tests (~6 tests, <2 min)
- **Total PR gate:** ~25 tests, target <7 minutes

**Test harness:** Shell test framework (e.g., `bats-core`) with `gh` CLI overridden by shell function mocks. Tests run in a temp directory with `$HOME` overridden to prevent accidental mutation of the developer's real config.

### Nightly / Pre-Release Only

- P3 checksum verification smoke test (requires real GitHub Release from Epic 8)
- P3 full E2E first-install run (requires throwaway GitHub account + network)

### On-Demand / Manual

- Cross-platform verification: manual install run on macOS arm64 and Linux x86_64
- Multi-machine path: real second-machine install against an existing `vibestats-data` repo

---

## Resource Estimates

| Priority | Count | Total Effort | Notes |
| --- | --- | --- | --- |
| P0 | 8 | ~14–22 hours | Token security, file permissions, failure handling, registry schema |
| P1 | 11 | ~10–18 hours | Platform detection, auth flows, hook config, first-install steps |
| P2 | 6 | ~3–8 hours | Edge cases, idempotency, README warning path |
| P3 | 2 | ~1–3 hours | Real-environment smoke tests |
| **Total** | **27** | **~28–51 hours** | **~1–2 weeks** |

**Prerequisites:**

- Shell test framework: `bats-core` (or equivalent POSIX-compatible shell test runner)
- Mock `gh` CLI: shell function overrides — `gh()` dispatches to mock implementations per subcommand
- Temp `$HOME` fixture: isolated directory tree with sample `~/.claude/settings.json` and `~/.config/vibestats/` stubs
- `registry.json` JSON schema: formal schema from Story 1.4 (`_bmad-output/planning-artifacts/` schemas.md)
- Linux CI runner: GitHub Actions `ubuntu-latest` for Linux path coverage; macOS runner for macOS path coverage

---

## Quality Gate Criteria

### Pass/Fail Thresholds

- **P0 pass rate:** 100% (no exceptions; installer must not proceed past a security or failure-handling defect)
- **P1 pass rate:** ≥95% (failures require triage before merge)
- **P2/P3 pass rate:** ≥90% (informational)
- **R-001 to R-005 mitigations:** 100% complete before Epic 6 stories marked `done`

### Coverage Targets

- Token handling (NFR6, NFR7): 100% — every token write and pipe path asserted
- Failure handling: 100% — every `gh` command failure path tested for non-zero exit
- Platform detection: 100% of supported targets (macOS arm64, macOS x86_64, Linux x86_64)
- Hook configuration: 100% of acceptance criteria covered
- `registry.json` schema: 100% — all required fields asserted

### Non-Negotiable Requirements

- [ ] All P0 tests pass on every PR
- [ ] R-001 (token on disk) verified by automated file-scan assertion
- [ ] R-002 (config.toml permissions) verified by `stat` assertion in CI on both macOS and Linux
- [ ] R-003 (no silent continuation) verified by failure injection test
- [ ] R-004 (multi-machine path) verified by `gh repo view` mock test
- [ ] R-005 (`registry.json` schema) verified by JSON schema assertion

---

## Mitigation Plans

### R-001: VIBESTATS_TOKEN written to disk (Score: 6)

**Mitigation Strategy:**
1. In the test, mock `gh api /user/personal_access_tokens` to return a sentinel token string (e.g., `ghp_TESTTOKEN123`)
2. After installer runs, recursively scan `$HOME` (temp dir) for any file containing the sentinel string
3. Inspect shell history file (`~/.bash_history` or `~/.zsh_history` within test `$HOME`) for the sentinel — assert absent
4. Assert `VIBESTATS_TOKEN` is never echoed to stdout (capture stdout in test and grep)

**Owner:** Dev/QA
**Timeline:** Before Story 6.2 PR merge
**Status:** Planned (Story 6.2 in backlog)
**Verification:** P0 file-scan test passing in CI

---

### R-002: config.toml permissions wider than 600 (Score: 6)

**Mitigation Strategy:**
1. After installer writes `~/.config/vibestats/config.toml`, run `stat -c "%a"` (Linux) or `stat -f "%Lp"` (macOS) on the file
2. Assert output equals `600`
3. Run this assertion on both macOS runner and Linux runner in CI matrix
4. Also assert the `~/.config/vibestats/` directory itself is not world-readable

**Owner:** Dev/QA
**Timeline:** Before Story 6.2 PR merge
**Status:** Planned (Story 6.2 in backlog)
**Verification:** Permission assertion passes on both CI platform runners

---

### R-003: silent continuation past failed step (Score: 6)

**Mitigation Strategy:**
1. Verify `install.sh` has `set -euo pipefail` at the top (shell static check)
2. Mock `gh repo create` to exit 1; assert installer exit code is non-zero and stderr contains an actionable error message
3. Mock `gh secret set` to exit 1; assert same behaviour
4. Mock `gh api` (token generation) to fail; assert installer halts before writing `config.toml`

**Owner:** Dev/QA
**Timeline:** Before Story 6.2 PR merge
**Status:** Planned (Story 6.2 in backlog)
**Verification:** Failure injection tests pass in CI; `set -euo pipefail` verified by `bash -n` lint

---

### R-004: first-install path runs on existing installation (Score: 6)

**Mitigation Strategy:**
1. Mock `gh repo view username/vibestats-data` to exit 0 (repo exists)
2. Spy on all `gh` subcommand calls during installer run
3. Assert `gh repo create` was NOT called
4. Assert `gh workflow write` (or equivalent) was NOT called
5. Assert `gh secret set VIBESTATS_TOKEN` was NOT called
6. Assert only machine registration (registry.json append) and hook configuration were performed

**Owner:** Dev/QA
**Timeline:** Before Story 6.3 PR merge
**Status:** Planned (Story 6.3 in backlog)
**Verification:** Spy-call assertion test passes; no unintended `gh repo create` in multi-machine runs

---

### R-005: registry.json missing required fields (Score: 6)

**Mitigation Strategy:**
1. After installer writes `registry.json` to `vibestats-data` (via `gh api` Contents PUT mock), capture the written JSON body
2. Parse JSON and assert all four fields present: `machine_id` (non-empty string), `hostname` (non-empty string), `status` (equals `"active"`), `last_seen` (ISO 8601 UTC timestamp matching `\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z`)
3. Test for multi-machine path: assert new machine entry appended, not replaced; existing entries unchanged

**Owner:** Dev/QA
**Timeline:** Before Story 6.2 PR merge
**Status:** Planned (Story 6.2 in backlog)
**Verification:** JSON schema assertion and regex timestamp validation pass in CI

---

## Assumptions and Dependencies

### Assumptions

1. `install.sh` uses `set -euo pipefail` — this is the primary guard against silent continuation (R-003)
2. The `gh` CLI is the sole mechanism for all GitHub operations in the installer — no direct `curl` calls to GitHub API
3. `bats-core` (or equivalent) is acceptable as the shell test framework for this epic; no Python or Node test runner required
4. The `registry.json` schema from Story 1.4 is the authoritative contract for machine entry fields
5. Tests will mock `gh` via shell function override (not a binary stub) — valid for `bats-core` test environments
6. Epic 8 binary release is available at a real GitHub Release URL before P3 E2E smoke tests can run

### Dependencies

1. Story 6.1 (`install.sh` dependency detection and `gh` auth) — required before P0/P1 tests for those flows can be validated
2. Story 6.2 (first-install path) — required before token security (R-001, R-002) and registry schema (R-005) P0 tests can run
3. Story 6.3 (multi-machine path) — required before R-004 mitigation test can run
4. Story 6.4 (hook config + README markers + backfill) — required before hook idempotency (R-008) and P2 tests can run
5. Epic 8 binary available at `https://github.com/stephenleo/vibestats/releases/latest` — required before P3 checksum smoke test

### Risks to Plan

- **Risk:** Shell test mocking of `gh` is fragile if `install.sh` uses `$(gh ...)` subshells vs. direct calls
  - **Impact:** Test isolation may break in certain invocation patterns
  - **Contingency:** Wrap all `gh` calls in a `_gh()` helper function within `install.sh` — easier to override in tests

- **Risk:** macOS GitHub Actions runner availability and cost for P0 permission tests
  - **Impact:** `stat -f` macOS syntax differs from Linux `stat -c` — test matrix required
  - **Contingency:** Write a platform-aware `check_perms()` shell helper that selects the correct `stat` syntax; test helper itself in CI

---

## Interworking & Regression

| Component | Impact | Regression Scope |
| --- | --- | --- |
| `registry.json` schema | Written by installer; read by `aggregate.py` (Epic 5) | Registry schema test validates all fields; `aggregate.py` P0 tests cover `purged` machine skip |
| `~/.claude/settings.json` hooks | Written by installer; executed by Claude Code `Stop`/`SessionStart` hooks | Hook format test validates JSON structure; Epic 3 hook integration tests cover hook execution |
| `~/.config/vibestats/config.toml` | Written by installer; read by Rust binary on every sync | Permissions test covers `600`; Epic 2 config module tests cover read/write logic |
| `aggregate.yml` workflow | Written into `vibestats-data` by installer; invoked by GitHub Actions | Content assertion test verifies `stephenleo/vibestats@v1` reference; Epic 5 schema tests cover trigger assertions |
| Binary download URL | Constructed by installer from `uname` outputs; served by Epic 8 release pipeline | Platform detection tests verify URL construction; Epic 8 release matrix tests verify asset publishing |

---

## Follow-on Workflows

- Run `*atdd` to generate failing P0 shell tests before Story 6.1 implementation (TDD approach recommended for security-critical token and permissions paths)
- Run `*automate` once all installer stories are complete to expand coverage

---

## Appendix

### Knowledge Base References

- `risk-governance.md` — Risk classification framework (P×I scoring, gate decision rules)
- `probability-impact.md` — Risk scoring methodology (1–3 scale, DOCUMENT/MONITOR/MITIGATE/BLOCK)
- `test-levels-framework.md` — Unit vs. integration vs. E2E selection
- `test-priorities-matrix.md` — P0–P3 prioritisation criteria

### Related Documents

- PRD: `_bmad-output/planning-artifacts/prd.md` (FR1–FR11, FR38–FR39, NFR6, NFR7, NFR16)
- Epics: `_bmad-output/planning-artifacts/epics.md` (Epic 6, Stories 6.1–6.4)
- Architecture: `_bmad-output/planning-artifacts/architecture.md` (Bash Installer section, Authentication & Security section)
- Sprint Status: `_bmad-output/implementation-artifacts/sprint-status.yaml`

---

**Generated by:** BMad TEA Agent — Test Architect Module
**Workflow:** `bmad-testarch-test-design`
**Version:** 4.0 (BMad v6)
