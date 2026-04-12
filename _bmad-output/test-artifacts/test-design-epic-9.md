---
stepsCompleted: ['step-01-detect-mode', 'step-02-load-context', 'step-03-risk-and-testability', 'step-04-coverage-plan', 'step-05-generate-output']
lastStep: 'step-05-generate-output'
lastSaved: '2026-04-12'
---

# Test Design: Epic 9 — Post-Sprint Quality & Technical Debt

**Date:** 2026-04-12
**Author:** Leo
**Status:** Draft

---

## Executive Summary

**Scope:** Epic-level test design for Epic 9 (Post-Sprint Quality & Technical Debt)

Epic 9 is a quality-consolidation epic with no new product features. Its 9 stories address stale artifact state, missing code reviews, a pre-launch blocker in the bats test suite, two refactors (EXIT trap composability, dead_code suppressors), the first public release (v0.1.0), a YAML workflow fix, documentation updates, and two Python script hardening items. Because Epic 9 is a debt-clearing epic, the dominant test type is verification-and-regression: confirm the fix works, confirm nothing was broken.

**Risk Summary:**

- Total risks identified: 10
- High-priority risks (≥6): 4 (R-001, R-002, R-003, R-004)
- Critical categories: OPS, TECH, SEC, BUS

**Coverage Summary:**

- P0 scenarios: 8 (~10–14 hours)
- P1 scenarios: 12 (~12–18 hours)
- P2 scenarios: 7 (~4–8 hours)
- P3 scenarios: 3 (~1–3 hours)
- **Total effort**: ~27–43 hours (~3–5 days)

---

## Not in Scope

| Item | Reasoning | Mitigation |
|------|-----------|------------|
| New feature testing | Epic 9 has no new product features | All new-feature test design was completed in Epics 5–8 |
| Performance benchmarks | No performance changes introduced | Prior NFR baselines remain valid |
| End-to-end Marketplace listing validation | GitHub Marketplace review is an external process outside CI | Story 9.6 documents submission status manually |
| Visual regression for Astro site | No site changes in Epic 9 | Epics 7/8 test designs cover site regression |

---

## Risk Assessment

### High-Priority Risks (Score ≥6)

| Risk ID | Category | Description | Probability | Impact | Score | Mitigation | Owner | Timeline |
|---------|----------|-------------|-------------|--------|-------|------------|-------|----------|
| R-001 | OPS | `release.yml` has never run in production — first real tag push may fail (TLS/cross-compilation, binary asset upload) causing a failed or incomplete v0.1.0 release | 2 | 3 | 6 | Pre-flight checklist: `cargo test`, `cargo clippy`, bats suite all green before tagging; rustls fallback documented and ready to apply | Dev | Pre-release (Story 9.6) |
| R-002 | TECH | Removing `#![allow(dead_code)]` suppressors may expose genuinely unused symbols in `sync.rs` or `hooks/mod.rs` causing `cargo clippy -D warnings` to fail | 2 | 3 | 6 | Run clippy after each file's suppressor is removed; fix each warning before moving to next file; full `cargo test` regression after all suppressors removed | Dev | Story 9.5 |
| R-003 | SEC | Story 9.2 code reviews of `vibestats auth` and `vibestats uninstall` may surface P0/P1 security or correctness issues (token leak, config file corruption) that block story completion | 2 | 3 | 6 | Three-pass review: Blind Hunter (security), Edge Case Hunter (boundaries), Acceptance Auditor (ACs); P0/P1 findings must be fixed before story done | Dev | Story 9.2 |
| R-004 | BUS | `test_6_2.bats` root cause may be a functional regression in `install.sh` logic (not just a test isolation issue), requiring changes to production code that could introduce new failures in test_6_1/6_3/6_4 | 2 | 3 | 6 | Full regression suite re-run after every fix attempt: `bats tests/installer/test_6_1.bats test_6_2.bats test_6_3.bats test_6_4.bats`; no test deletions or xfails permitted | Dev | Story 9.3 |

### Medium-Priority Risks (Score 3–4)

| Risk ID | Category | Description | Probability | Impact | Score | Mitigation | Owner |
|---------|----------|-------------|-------------|--------|-------|------------|-------|
| R-005 | TECH | EXIT trap refactor in Story 9.4 may break the bats test suite if `_CLEANUP_TMPDIR` variable scoping or timing interacts unexpectedly with the `setup`/`teardown` test lifecycle | 2 | 2 | 4 | Depend on Story 9.3 being green first; run full bats suite after refactor; pattern is Bash 3.2-compatible by design | Dev |
| R-006 | DATA | `aggregate.yml` concurrency group (Story 9.7) with `cancel-in-progress: false` may cause unexpected queue buildup if many Claude Code sessions are active simultaneously | 1 | 3 | 3 | Schema test validates `cancel-in-progress: false`; behavior intent documented in story; no behavioral test required (GitHub Actions runtime) | Dev |
| R-007 | OPS | artifact-hygiene changes in Story 9.1 (Status field updates, new story file 5-2, updated dep-graph) may accidentally overwrite or corrupt story files due to batch edits | 2 | 2 | 4 | Changes are documentation-only; verify with `grep -r "Status: review" _bmad-output/implementation-artifacts/` returning no output after completion | Dev |

### Low-Priority Risks (Score 1–2)

| Risk ID | Category | Description | Probability | Impact | Score | Action |
|---------|----------|-------------|-------------|--------|-------|--------|
| R-008 | OPS | `update_readme.py` empty-username validation (Story 9.9) may not handle Unicode whitespace variants; `str.strip()` covers ASCII whitespace but not all edge cases | 1 | 1 | 1 | Monitor — `str.strip()` covers all standard whitespace categories for this use case |
| R-009 | BUS | Architecture doc updates (Story 9.8) may contain factually incorrect code examples if not cross-checked against actual source | 1 | 2 | 2 | Each code example must be verified against actual source before writing; story Dev Notes enforce this |
| R-010 | TECH | `expected_output/data.json` fixture decision (Story 9.9) — if wired into a test, the test may be brittle if aggregate.py output format evolves | 1 | 1 | 1 | Monitor — prefer Option B (remove fixture) if it is purely structural; document the decision in Dev Agent Record |

### Risk Category Legend

- **TECH**: Technical/Architecture (flaws, integration, scalability)
- **SEC**: Security (access controls, auth, data exposure)
- **PERF**: Performance (SLA violations, degradation, resource limits)
- **DATA**: Data Integrity (loss, corruption, inconsistency)
- **BUS**: Business Impact (UX harm, logic errors, revenue)
- **OPS**: Operations (deployment, config, monitoring)

---

## Entry Criteria

- [ ] All Epic 1–8 stories are `done` in sprint-status.yaml (confirmed: yes, per sprint-status.yaml)
- [ ] Epic 9 story files (9.1–9.9) exist in `_bmad-output/implementation-artifacts/`
- [ ] `bats` is installed and available on the dev machine
- [ ] `cargo` and `cargo clippy` are available
- [ ] `python3 -m pytest` is available for Python test suite
- [ ] Developer has GitHub permissions to push tags and trigger Actions workflows (Story 9.6)

## Exit Criteria

- [ ] All 9 Epic 9 stories marked `done` in sprint-status.yaml
- [ ] `bats tests/installer/test_6_1.bats tests/installer/test_6_2.bats tests/installer/test_6_3.bats tests/installer/test_6_4.bats` exits 0 with 0 failures
- [ ] `cargo clippy --all-targets -- -D warnings` exits 0 with 0 warnings
- [ ] `cargo test` exits 0 with 0 failures
- [ ] `python3 -m pytest action/tests/` exits 0 with 0 failures
- [ ] GitHub Release at `github.com/stephenleo/vibestats/releases/tag/v0.1.0` exists with 3 binary assets
- [ ] `v1` floating tag exists and `uses: stephenleo/vibestats@v1` resolves correctly
- [ ] No story file in `_bmad-output/implementation-artifacts/` shows `Status: review` while sprint-status.yaml shows `done`
- [ ] No P0 or P1 review findings from Story 9.2 remain unaddressed

---

## Test Coverage Plan

### P0 (Critical) — Run immediately; blocks story completion

**Criteria:** Blocks pre-launch readiness + High risk (≥6) + No workaround

| Requirement / Story | Test Level | Risk Link | Scenario | Test Count | Owner | Notes |
|---------------------|------------|-----------|----------|------------|-------|-------|
| **9.3** — Full bats suite passes after fix | Shell (bats) | R-004 | `bats tests/installer/test_6_2.bats` exits 0 | 1 | Dev | Isolation fix: all test_6_2 cases pass in isolation |
| **9.3** — No regression in other test files | Shell (bats) | R-004 | Full bats suite: test_6_1 + test_6_2 + test_6_3 + test_6_4 exits 0 | 1 | Dev | Regression verification after fix |
| **9.5** — No dead_code suppressors in src/ | Static analysis | R-002 | `grep -rn "allow(dead_code)" src/` returns no output (or only item-level with comments) | 1 | Dev | Automated grep assertion |
| **9.5** — Clippy clean after suppressor removal | Static analysis | R-002 | `cargo clippy --all-targets -- -D warnings` exits 0 | 1 | Dev | Run after all suppressors removed |
| **9.5** — No test regressions after suppressor removal | Unit | R-002 | `cargo test` exits 0 with same or higher test count | 1 | Dev | Baseline test count before and after |
| **9.2** — No P0/P1 findings unresolved in auth review | Code review | R-003 | Story 4-3 Review Findings section exists; no P0/P1 items open | 1 | Dev | Three-pass adversarial review artifact |
| **9.2** — No P0/P1 findings unresolved in uninstall review | Code review | R-003 | Story 4-4 Review Findings section exists; no P0/P1 items open | 1 | Dev | Special attention: config.toml partial write risk |
| **9.6** — v0.1.0 release has all 3 binary assets | Integration (GitHub Actions) | R-001 | GitHub Release page shows `vibestats-aarch64-apple-darwin.tar.gz`, `vibestats-x86_64-apple-darwin.tar.gz`, `vibestats-x86_64-unknown-linux-gnu.tar.gz` | 1 | Dev | Manual verification post-run |

**Total P0:** 8 tests, ~10–14 hours

### P1 (High) — Run on PR to main; blocks story sign-off

**Criteria:** Critical paths + Medium/High risk + Essential acceptance criteria

| Requirement / Story | Test Level | Risk Link | Scenario | Test Count | Owner | Notes |
|---------------------|------------|-----------|----------|------------|-------|-------|
| **9.1** — All stale Status fields corrected | Documentation check | R-007 | `grep -r "Status: review" _bmad-output/implementation-artifacts/` returns no output | 1 | Dev | Verifies all 10 stale story files updated |
| **9.1** — Story 5.2 artifact exists with Dev Agent Record | File existence check | R-007 | `5-2-implement-generate-svg-py.md` exists with Status: done and non-empty Dev Agent Record | 1 | Dev | File creation verification |
| **9.1** — Stories 7.4 and 8.2 have complete Dev Agent Records | Documentation check | R-007 | Both files have non-empty Completion Notes, File List, and Change Log | 1 | Dev | Spot-check of recovered content |
| **9.4** — EXIT trap refactored; cleanup() registered once | Shell (bats) | R-005 | `install.sh` contains `cleanup()` function and `trap cleanup EXIT`; no inline trap in `download_and_install_binary()` | 1 | Dev | Pattern assertion + bats regression |
| **9.4** — Full bats suite still passes after trap refactor | Shell (bats) | R-005 | `bats tests/installer/test_6_1.bats test_6_2.bats test_6_3.bats test_6_4.bats` exits 0 | 1 | Dev | Depends on Story 9.3 green first |
| **9.6** — v1 floating tag resolves to v0.1.0 | Git tag check | R-001 | `git ls-remote origin refs/tags/v1` shows the v1 tag; points to v0.1.0 commit | 1 | Dev | Manual git verification |
| **9.7** — `aggregate.yml` concurrency block present | Schema (Python/pytest) | R-006 | New pytest test: `workflow["concurrency"]["group"] == "vibestats-${{ github.repository_owner }}"` and `cancel-in-progress == False` | 1 | Dev | Automated schema assertion |
| **9.7** — All existing aggregate.yml tests still pass | Schema (Python/pytest) | R-006 | `python3 -m pytest action/tests/test_aggregate_yml.py` exits 0 | 1 | Dev | Regression for existing schema tests |
| **9.9** — `update_readme.py --username ""` exits non-zero | Unit (Python/pytest) | R-008 | Test: `subprocess.run(["python3", "update_readme.py", "--username", ""], ...)` — exit code non-zero, stderr contains error message | 1 | Dev | New test added in story |
| **9.9** — `update_readme.py` normal behavior unchanged | Unit (Python/pytest) | - | Existing update_readme.py tests still pass with valid username | 1 | Dev | Regression guard |
| **9.9** — `expected_output/data.json` fixture decision executed | File state check | R-010 | Fixture is either wired into a new test that passes OR deleted (with reason documented) | 1 | Dev | Either outcome is acceptable |
| **9.6** — Pre-release checklist verified before tagging | Pre-flight (manual) | R-001 | `cargo test` passes + `cargo clippy` passes + bats suite passes + action.yml branding confirmed | 1 | Dev | Gate check before `git tag` |

**Total P1:** 12 tests, ~12–18 hours

### P2 (Medium) — Run nightly or before epic close

**Criteria:** Secondary verification + Low risk (1–2) + Process quality

| Requirement / Story | Test Level | Risk Link | Scenario | Test Count | Owner | Notes |
|---------------------|------------|-----------|----------|------------|-------|-------|
| **9.1** — dependency-graph.md updated to reflect all Epics 1–8 done | Documentation review | R-007 | Manual review: all Epic 1–8 entries show complete/done status matching sprint-status.yaml | 1 | Dev | No automated test possible for this |
| **9.2** — `cargo test` passes after any P0/P1 fixes applied | Unit (Rust) | R-003 | `cargo test` exits 0 after fixes to `auth.rs` or `uninstall.rs` | 1 | Dev | Post-fix regression |
| **9.5** — Hooks module dead_code also cleaned | Static analysis | R-002 | `src/hooks/mod.rs` suppressor removed; no new clippy warnings introduced | 1 | Dev | Hooks module has its own `#![allow(dead_code)]` |
| **9.8** — All six architecture doc items present | Documentation review | R-009 | Manual check: `architecture.md` contains `_redirects` ordering, serde footgun, `[workspace]` pattern, `_gh()` guard, Python3-over-jq, security negative test pattern | 1 | Dev | Spot verification — 6 items |
| **9.8** — Code examples in architecture.md are syntactically valid | Code example check | R-009 | Bash and Rust snippets cross-checked against actual source; no fabricated examples | 1 | Dev | Manual grep cross-check |
| **9.6** — Cloudflare Pages deployment succeeds | Integration (manual) | R-001 | `vibestats.dev` loads the landing page after `workflow_dispatch` of `deploy-site.yml` | 1 | Dev | Manual browser verification |
| **9.4** — Pattern is Bash 3.2 compatible | Shell analysis | R-005 | No `declare -A`, no `mapfile`, no `+=` array append in the new `cleanup()` function | 1 | Dev | `grep` assertion against install.sh |

**Total P2:** 7 tests, ~4–8 hours

### P3 (Low) — On-demand; optional quality enrichment

**Criteria:** Nice-to-have + Exploratory + Process polish

| Requirement / Story | Test Level | Scenario | Test Count | Owner | Notes |
|---------------------|------------|----------|------------|-------|-------|
| **9.6** — All 3 binary tar.gz archives extract to valid `vibestats` binary | Smoke (manual) | Download each asset, `tar -xzf`, run `./vibestats --version`; confirm prints version | 1 | Dev | Optional pre-announcement quality check |
| **9.2** — P2 findings from 4.3/4.4 reviews entered into deferred-work.md | Process check | deferred-work.md contains any unaddressed P2 items from the code reviews | 1 | Dev | Ensures feedback is not lost |
| **9.9** — `python3 -m pytest action/tests/` full suite passes after all changes | Integration (Python) | All Python action tests pass end-to-end after 9.7 + 9.9 changes combined | 1 | Dev | Final combined regression pass |

**Total P3:** 3 tests, ~1–3 hours

---

## Execution Order

### Recommended Story Order (from Epic 9 dependency analysis)

The recommended execution order respects explicit dependencies (9.3 before 9.6; 9.5 before 9.6):

```
9.1 → 9.2 → 9.3 → 9.5 → 9.4 → 9.6 → 9.7 → 9.8 → 9.9
```

Independent stories (9.7, 9.8, 9.9) can run in parallel once pre-requisite stories (9.3, 9.5) are complete.

### Per-Story Test Execution Sequence

**Story 9.1 (Artifact Hygiene):**
- [ ] After story complete: `grep -r "Status: review" _bmad-output/implementation-artifacts/` (expect no output) — P1
- [ ] Verify `5-2-implement-generate-svg-py.md` exists with Dev Agent Record — P1
- [ ] Spot-check `7-4-build-landing-page.md` and `8-2-implement-cloudflare-pages-deploy-workflow.md` for non-empty records — P1
- [ ] Review dependency-graph.md manually — P2

**Story 9.2 (Code Reviews):**
- [ ] Perform three-pass review of `src/commands/auth.rs` — P0
- [ ] Perform three-pass review of `src/commands/uninstall.rs` — P0 (special attention: JSON surgery)
- [ ] `cargo test` after any P0/P1 fixes — P2
- [ ] Any P2 findings documented in deferred-work.md — P3

**Story 9.3 (Fix test_6_2.bats):**
- [ ] `bats tests/installer/test_6_2.bats` exits 0 — P0
- [ ] `bats tests/installer/test_6_1.bats test_6_2.bats test_6_3.bats test_6_4.bats` exits 0 — P0

**Story 9.4 (EXIT trap refactor):**
- [ ] `grep "cleanup()" install.sh` and `grep "trap cleanup EXIT" install.sh` — P1
- [ ] `grep "trap.*rm.*EXIT" install.sh` returns no output — P1
- [ ] Full bats suite passes — P1
- [ ] Bash 3.2 pattern check — P2

**Story 9.5 (dead_code suppressors):**
- [ ] `grep -rn "allow(dead_code)" src/` returns no module-level suppressors — P0
- [ ] `cargo clippy --all-targets -- -D warnings` exits 0 — P0
- [ ] `cargo test` exits 0 — P0
- [ ] `src/hooks/mod.rs` suppressor also addressed — P2

**Story 9.6 (First Release):**
- [ ] Pre-release checklist (cargo test + clippy + bats) — P1 gate
- [ ] GitHub Release at v0.1.0 with 3 binary assets — P0
- [ ] `v1` floating tag resolves to v0.1.0 — P1
- [ ] Cloudflare Pages deployment check — P2
- [ ] Binary extract and `--version` smoke test — P3

**Story 9.7 (aggregate.yml concurrency):**
- [ ] New pytest test for concurrency block passes — P1
- [ ] All existing `test_aggregate_yml.py` tests pass — P1

**Story 9.8 (Architecture docs):**
- [ ] All 6 sections present in architecture.md — P2
- [ ] Code examples verified against source — P2

**Story 9.9 (Python hardening):**
- [ ] `update_readme.py --username ""` returns non-zero with error message — P1
- [ ] All existing update_readme.py tests pass — P1
- [ ] Fixture decision executed and documented — P1
- [ ] Full Python test suite passes — P3

---

## Resource Estimates

### Test Development Effort

| Priority | Count | Hours/Test | Total Hours | Notes |
|----------|-------|------------|-------------|-------|
| P0 | 8 | 1.25–1.75 | 10–14 | Code review passes + bats/clippy runs |
| P1 | 12 | 1.0–1.5 | 12–18 | Pattern checks + new pytest assertions + pre-release gate |
| P2 | 7 | 0.5–1.0 | 4–8 | Documentation checks + manual verification |
| P3 | 3 | 0.5–1.0 | 1–3 | Optional smoke tests |
| **Total** | **30** | **—** | **27–43** | **~3–5 days** |

### Prerequisites

**Test Data / Fixtures:**
- Existing bats test fixtures in `tests/installer/` (test_6_1.bats through test_6_4.bats)
- Python test fixtures in `action/tests/fixtures/` (to be resolved by Story 9.9)
- Git history accessible for Story 9.1 record recovery

**Tooling:**
- `bats-core` — shell test runner for installer tests
- `cargo` with `clippy` component — Rust lint verification
- `python3 -m pytest` — Python action test suite
- `gh` CLI — tag push and release monitoring for Story 9.6
- `git` — tag creation and verification

**Environment:**
- Local macOS dev environment with Bash 3.2+ (default on macOS)
- GitHub repository write access (push tags, trigger Actions)
- Cloudflare Pages deployment configured (Story 8.2 prerequisite)

---

## Quality Gate Criteria

### Pass/Fail Thresholds

- **P0 pass rate**: 100% (no exceptions; these are pre-launch blockers)
- **P1 pass rate**: 100% for automated tests; 95% for manual process checks
- **P2/P3 pass rate**: ≥90% (informational; waivers documented)
- **High-risk mitigations (R-001–R-004)**: 100% complete before Epic 9 is marked done

### Coverage Targets

- **Security scenarios** (Stories 9.2, 9.5): 100% — auth token handling and config file surgery are security-sensitive
- **Pre-release gate** (Story 9.6): 100% of checklist items before tag push
- **Test suite regressions**: Zero tolerance — no story may break a previously passing test

### Non-Negotiable Requirements

- [ ] All P0 tests pass before any story is marked done
- [ ] `bats tests/installer/test_6_1.bats test_6_2.bats test_6_3.bats test_6_4.bats` exits 0 (pre-launch blocker)
- [ ] `cargo clippy --all-targets -- -D warnings` exits 0 (release quality gate)
- [ ] No P0 or P1 review findings from Story 9.2 remain open
- [ ] `v0.1.0` release exists on GitHub with 3 platform binaries before Epic 9 is complete

---

## Mitigation Plans

### R-001: `release.yml` first real tag push may fail (Score: 6)

**Mitigation Strategy:** Run all local verification (cargo test, cargo clippy, bats) before pushing the tag. Have the rustls fallback documented and ready to apply immediately if the Linux cross-compilation fails. Monitor the Actions run in real time. If a tag is pushed in error, use the documented deletion procedure (`git tag -d v0.1.0 && git push origin :refs/tags/v0.1.0`) before re-pushing.
**Owner:** Dev
**Timeline:** Story 9.6 execution date
**Status:** Planned
**Verification:** GitHub Release page shows 3 binary assets after Actions run completes.

### R-002: dead_code suppressor removal may expose unused symbols in hooks/sync modules (Score: 6)

**Mitigation Strategy:** Remove suppressors file by file. After each file's suppressor is removed, immediately run `cargo clippy --all-targets -- -D warnings`. Fix each warning before moving to the next file. Use `cargo test` as the final regression gate. Any symbol that is genuinely unused and not part of the public API should be removed; any symbol kept for intentional future use should have an item-level `#[allow(dead_code)]` with a comment.
**Owner:** Dev
**Timeline:** Story 9.5 execution date
**Status:** Planned
**Verification:** `grep -rn "allow(dead_code)" src/` returns no output (or only item-level with explanatory comments); `cargo clippy --all-targets -- -D warnings` exits 0.

### R-003: Code reviews of auth/uninstall may surface P0/P1 security issues (Score: 6)

**Mitigation Strategy:** Execute three-pass adversarial review for both story files. For auth: focus on token write paths, `gh secret set` failure handling, `Config::save()` atomicity, and `set_permissions` call. For uninstall: focus on `remove_vibestats_hooks` JSON surgery on malformed input, file deletion race conditions, and empty hook array handling. Apply all P0/P1 fixes immediately; run `cargo test` and `cargo clippy` after fixes.
**Owner:** Dev
**Timeline:** Story 9.2 execution date
**Status:** Planned
**Verification:** Both story files have `## Review Findings` sections; no open P0/P1 items; `cargo test` passes.

### R-004: test_6_2.bats root cause may be a production code regression (Score: 6)

**Mitigation Strategy:** Follow the diagnostic protocol in Story 9.3: reproduce failures, identify whether isolation/mock/regression/environment issue, document root cause before applying fix. Fix the root cause — not the symptom. Never delete, skip, or xfail a test. Verify both isolated run (`bats test_6_2.bats`) and full regression suite pass before closing the story.
**Owner:** Dev
**Timeline:** Story 9.3 execution date (pre-launch blocker — must complete before Story 9.6)
**Status:** Planned
**Verification:** `bats tests/installer/test_6_2.bats` exits 0; full bats suite exits 0; root cause documented in Dev Agent Record.

---

## Assumptions and Dependencies

### Assumptions

1. All Epic 1–8 source code and test files are accessible in the working directory at their expected paths (no missing files beyond the ones identified in Story 9.1).
2. The developer has write access to push tags to `github.com/stephenleo/vibestats` (needed for Story 9.6).
3. `bats-core` is installed on the dev machine and compatible with the existing test files.
4. Python3 and `pytest` are available in the action test environment.
5. The `release.yml` workflow and its cross-compilation setup are functionally complete (Story 8.1 deliverable) — Story 9.6 is triggering, not implementing, the release.

### Dependencies

1. **Story 9.3 before Story 9.6**: Installer test suite must be green before release tag is pushed.
2. **Story 9.5 before Story 9.6** (recommended): Clippy must pass before release for code quality gate.
3. **Story 9.3 before Story 9.4**: EXIT trap refactor must be verified against a green bats suite to confirm no regressions.

### Risks to Plan

- **Risk**: Linux `cross` compilation fails with OpenSSL TLS error during Story 9.6
  - **Impact**: v0.1.0 release delayed; Linux binary missing
  - **Contingency**: Apply rustls fallback documented in Story 9.6 Dev Notes; delete and re-push tag after Cargo.toml fix

- **Risk**: Story 9.2 review surfaces a P0 security issue in `auth.rs`
  - **Impact**: Requires source code fix and re-test before Epic 9 can close
  - **Contingency**: P0 fix is mandatory scope; timeline may extend by 1–2 days if significant refactoring is needed

---

## Interworking & Regression

| Service/Component | Impact | Regression Scope |
|-------------------|--------|-----------------|
| **install.sh** | Modified by Stories 9.3 and 9.4 | Full bats suite: test_6_1–test_6_4 must pass |
| **src/commands/auth.rs** | Potentially modified by Story 9.2 (P0/P1 fixes) | `cargo test` must pass; `cargo clippy -D warnings` must pass |
| **src/commands/uninstall.rs** | Potentially modified by Story 9.2 (P0/P1 fixes) | Same as above |
| **src/*.rs (all modules)** | Modified by Story 9.5 (suppressor removal) | `cargo test` full suite; `cargo clippy --all-targets -D warnings` |
| **.github/workflows/aggregate.yml** | Modified by Story 9.7 (concurrency block) | `python3 -m pytest action/tests/test_aggregate_yml.py` |
| **action/update_readme.py** | Modified by Story 9.9 (empty-string validation) | `python3 -m pytest action/tests/test_update_readme.py` |
| **_bmad-output/planning-artifacts/architecture.md** | Modified by Story 9.8 | Manual review: no existing content removed; 6 new items present |
| **_bmad-output/implementation-artifacts/ (story files)** | Modified by Story 9.1 | `grep -r "Status: review" _bmad-output/implementation-artifacts/` returns no output |

---

## Follow-on Workflows (Manual)

- Run `*atdd` for individual stories if failing acceptance tests need to be scaffolded before implementation (especially Story 9.3 — diagnosing bats failures may benefit from writing expected behavior as a failing test first).
- Run `*trace` at Epic 9 completion to generate the final traceability matrix confirming all 9 stories' ACs are covered.

---

## Approval

**Test Design Approved By:**

- [ ] Product Manager: Leo — Date: 2026-04-12
- [ ] Tech Lead: Leo — Date: 2026-04-12
- [ ] QA Lead: Leo — Date: 2026-04-12

---

## Appendix

### Knowledge Base References

- `risk-governance.md` — Risk classification framework
- `probability-impact.md` — Risk scoring methodology
- `test-levels-framework.md` — Test level selection
- `test-priorities-matrix.md` — P0–P3 prioritization

### Related Documents

- Epic: `_bmad-output/planning-artifacts/epic-9.md`
- Sprint Status: `_bmad-output/implementation-artifacts/sprint-status.yaml`
- Stories: `_bmad-output/implementation-artifacts/9-1-*.md` through `9-9-*.md`
- Prior test designs: `test-design-epic-5.md`, `test-design-epic-6.md`, `test-design-epic-7.md`, `test-design-epic-8.md`
- Architecture: `_bmad-output/planning-artifacts/architecture.md`
- Deferred work: `_bmad-output/implementation-artifacts/deferred-work.md`

---

**Generated by**: BMad TEA Agent - Test Architect Module
**Workflow**: `bmad-testarch-test-design`
**Version**: 4.0 (BMad v6)
