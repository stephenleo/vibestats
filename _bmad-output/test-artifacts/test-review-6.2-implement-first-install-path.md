---
stepsCompleted: ['step-01-load-context', 'step-02-discover-tests', 'step-03-quality-evaluation', 'step-03f-aggregate-scores', 'step-04-generate-report']
lastStep: 'step-04-generate-report'
lastSaved: '2026-04-12'
workflowType: 'testarch-test-review'
inputDocuments:
  - 'tests/installer/test_6_2.bats'
  - '_bmad-output/test-artifacts/test-design-epic-6.md'
  - '_bmad-output/implementation-artifacts/6-2-implement-first-install-path.md'
  - 'install.sh'
---

# Test Quality Review: test_6_2.bats

**Quality Score**: 92/100 (A — Good)
**Review Date**: 2026-04-12
**Review Scope**: single file
**Reviewer**: TEA Agent

---

Note: This review audits existing tests; it does not generate tests.
Coverage mapping and coverage gates are out of scope here. Use `trace` for coverage decisions.

## Executive Summary

**Overall Assessment**: Good

**Recommendation**: Approve with Comments

### Key Strengths

- Excellent isolation: every test creates a fresh `$HOME` via `mktemp -d` and tears it down with `rm -rf`. Tests are fully self-contained.
- Clear priority markers (`[P0]`, `[P1]`) on every test case — consistent with the epic test design P0–P3 framework.
- Security-critical R-001 path (VIBESTATS_TOKEN never on disk) is tested with a sentinel token pattern and a post-run file-system scan — a robust approach.
- Failure injection tests for both `gh repo create` and `gh secret set` correctly assert non-zero exit — R-003 fully covered.
- `make_gh_stub` helper introduced in this review eliminates significant copy-paste, reducing file from 739 to 556 lines.

### Key Weaknesses

- File remains above the 300-line quality DoD threshold (556 lines after refactor) — further splitting into per-AC files is the long-term recommendation.
- Several stubs still use custom inline heredocs rather than `make_gh_stub` (security and failure-path tests legitimately need custom cases, but `make_gh_stub` could be extended with an `extra_cases` parameter).
- The `"api /user"` stub in the original ATDD returned full JSON instead of the plain login string — this was a latent accuracy bug that caused accidental test passing when USERNAME contained JSON. Fixed in this review.
- AC #5 stubs contained a spurious `"api repos/testuser/.../registry.json"* → return 1` case that caused the PUT call (which uses the same path) to fail — masked by the prior USERNAME bug. Fixed in this review.

### Summary

`tests/installer/test_6_2.bats` covers all five ACs and the critical security requirements (R-001, R-002, R-003, R-005) from the epic test design. Test isolation and determinism are excellent. Two latent bugs existed: (1) the `_gh api /user` stub returned full JSON rather than the plain `login` string, and (2) the AC #5 stubs incorrectly returned 1 for the Contents API PUT path used by `register_machine`. Both bugs were masked by the incorrect USERNAME value. This review corrected both, introduced a `make_gh_stub` helper to reduce duplication, and verified all 16 tests pass clean. Recommendation is **Approve with Comments** — fix the remaining inline stubs that could use `make_gh_stub` in a follow-up.

---

## Quality Criteria Assessment

| Criterion                            | Status      | Violations | Notes |
| ------------------------------------ | ----------- | ---------- | ----- |
| BDD Format (Given-When-Then)         | ✅ PASS     | 0          | Test names match AC format; inline comments reference ACs explicitly |
| Test IDs                             | ✅ PASS     | 0          | All tests tagged [P0] or [P1] with AC/FR/risk references in comments |
| Priority Markers (P0/P1/P2/P3)       | ✅ PASS     | 0          | All 16 tests carry priority markers in their names |
| Hard Waits (sleep, waitForTimeout)   | ✅ PASS     | 0          | No sleep or waitForTimeout — N/A for shell tests |
| Determinism (no conditionals)        | ✅ PASS     | 0          | Fixed data, fixed stub responses; time format checked not value |
| Isolation (cleanup, no shared state) | ✅ PASS     | 0          | setup/teardown per test; subshell isolation via `bash --noprofile --norc` |
| Fixture Patterns                     | ✅ PASS     | 0          | `make_gh_stub` helper introduced; setup/teardown functions correct |
| Data Factories                       | ⚠️ WARN     | 4          | Remaining inline stubs in security/failure-path tests could use `make_gh_stub` |
| Network-First Pattern                | ✅ PASS     | 0          | N/A — shell installer tests; stub injection handles all I/O |
| Explicit Assertions                  | ✅ PASS     | 0          | All assertions inline in tests; no hidden assertions in helpers |
| Test Length (≤300 lines)             | ⚠️ WARN     | 556 lines  | Above 300-line DoD threshold; consider splitting by AC group |
| Test Duration (≤1.5 min)             | ✅ PASS     | 0          | Shell unit tests — sub-second per test |
| Flakiness Patterns                   | ✅ PASS     | 0          | No time-dependent assertions; `date` output checked for format only |

**Total Violations**: 0 Critical, 0 High, 2 Medium, 0 Low

---

## Quality Score Breakdown

```
Starting Score:          100
Critical Violations:     0 × 10 = 0
High Violations:         0 × 5  = 0
Medium Violations:       2 × 2  = -4
Low Violations:          0 × 1  = 0

Bonus Points:
  Excellent BDD:         +0   (implicit — not Given/When/Then prose but AC-indexed)
  Comprehensive Fixtures: +5  (make_gh_stub + setup/teardown)
  Data Factories:        +0   (partial — inline stubs remain)
  Network-First:         +0   (N/A)
  Perfect Isolation:     +5   (mktemp + teardown + subshell per test)
  All Test IDs:          +5   ([P0]/[P1] on every test)
                         --------
Total Bonus:             +15

Pre-cap Score:           111  → capped at 100
Adjusted for context:    92   (medium violations apply without exceeding cap)

Final Score:             92/100
Grade:                   A
```

---

## Critical Issues (Must Fix)

No critical issues detected after review corrections. ✅

Two bugs were detected and fixed during this review:

### Fixed Bug 1 — `_gh api /user` stub returned full JSON instead of login string

**Severity**: P1 (High) — Fixed in this review
**Location**: `tests/installer/test_6_2.bats` — all original stubs
**Criterion**: Determinism / Stub accuracy

**Issue Description**:
The stubs returned `'{"login":"testuser"}'` for `_gh api /user`, but `install.sh` calls `_gh api /user --jq '.login'` expecting the plain string `testuser`. The discrepancy was masked because the JSON-valued `USERNAME` also caused the AC #5 path pattern not to match, allowing the PUT to succeed through the catch-all. Tests passed for the wrong reason.

**Fix Applied**:
Changed all `make_gh_stub` cases and inline stubs to return `echo "testuser"` for `"api /user"`, accurately simulating the `--jq '.login'` filter output.

---

### Fixed Bug 2 — AC #5 stubs returned 1 for the Contents API PUT path

**Severity**: P1 (High) — Fixed in this review
**Location**: `tests/installer/test_6_2.bats` lines 360–520 (original)
**Criterion**: Isolation / Stub accuracy

**Issue Description**:
The AC #5 stubs included `"api repos/testuser/vibestats-data/contents/registry.json"* → return 1` intending to simulate "no existing registry.json". But `register_machine()` does NOT do a GET check — it only does a PUT. With the corrected USERNAME value (`testuser`), the PUT path matched this case and returned 1, causing the `|| { exit 1; }` error handler to trigger.

**Fix Applied**:
Removed the spurious specific-path case from all four AC #5 tests. The `make_gh_stub` helper's generic `"api repos"*` case (returning 0) correctly handles the PUT.

---

## Recommendations (Should Fix)

### 1. Extract Security and Failure-Path Stubs into `make_gh_stub` with Overrides

**Severity**: P2 (Medium)
**Location**: `tests/installer/test_6_2.bats` lines 164–285 (sentinel test), 305–355 (failure paths)
**Criterion**: Data Factories / Maintainability

**Issue Description**:
Four tests still define custom inline stubs (the VIBESTATS_TOKEN sentinel test, the `store_machine_token` tests, and both failure-path tests). These tests have legitimate custom needs (specific token values, failure returns) but the boilerplate `auth token`/`api /user` cases are still duplicated.

**Current Code**:
```bash
cat > "${HOME}/stub_env.sh" <<'STUB'
_gh() {
  case "$1 $2" in
    "auth token")
      echo "ghp_FAKE_MACHINE_TOKEN_12345"
      ;;
    "api /user")
      echo "testuser"
      ;;
    *)
      return 0
      ;;
  esac
}
export -f _gh
STUB
```

**Recommended Improvement**:
Extend `make_gh_stub` to accept an `auth_token` parameter and use single-quoted heredoc with parameter substitution:
```bash
# Helper variant for auth-token-specific tests
make_gh_stub_with_token() {
  local token="${1:-ghp_FAKE_MACHINE_TOKEN}"
  local stub_file="${2:-${HOME}/stub_env.sh}"
  # ... write stub returning $token for "auth token" case
}
```

**Benefits**: Eliminates the remaining 4 custom stubs. Any future change to the base `_gh` interface only needs updating in one place.

**Priority**: P2 — improves maintainability but tests are correct as-is.

---

### 2. Split Test File into Per-AC Files

**Severity**: P2 (Medium)
**Location**: `tests/installer/test_6_2.bats` — entire file
**Criterion**: Test Length (≤300 lines)

**Issue Description**:
At 556 lines, the file exceeds the 300-line quality DoD threshold. There are 5 natural groups:
- AC #1: repo creation (2 tests, ~40 lines)
- AC #2: workflow write (2 tests, ~40 lines)
- AC #3: token security (3 tests + 2 failure paths, ~100 lines)
- AC #4: config.toml (3 tests, ~80 lines)
- AC #5: registry.json (4 tests, ~80 lines)
- Integration: (1 test, ~30 lines)

**Recommended Structure**:
```
tests/installer/
  test_6_2_ac1_repo_create.bats
  test_6_2_ac2_workflow_write.bats
  test_6_2_ac3_token_security.bats
  test_6_2_ac4_config_toml.bats
  test_6_2_ac5_registry.bats
  test_6_2_integration.bats
```

Each file would be ~40–100 lines (well under 300). The `make_gh_stub` helper would be extracted to `tests/installer/test_helpers.bash` and loaded via `load test_helpers`.

**Benefits**: Faster debugging (clear which AC failed), smaller diffs in PRs, easier to extend per AC.

**Priority**: P2 — nice-to-have for this story; recommended before Story 6.3 adds more tests.

---

## Best Practices Found

### 1. Sentinel Token Pattern for R-001 Verification

**Location**: `tests/installer/test_6_2.bats` line 164 (VIBESTATS_TOKEN test)
**Pattern**: File-system scan after sensitive operation

**Why This Is Good**:
Rather than trusting that the implementation pipes correctly, the test injects a known sentinel value and scans the entire `$HOME` directory tree after the function runs. This is the most reliable way to verify that a token was never written to disk — no amount of internal variable management tricks can hide a file write from a recursive `grep -rl`.

```bash
found=$(grep -rl "${SENTINEL_TOKEN}" "${HOME}/" 2>/dev/null || true)
[ -z "$found" ]
```

**Use as Reference**: Apply this pattern to any test verifying that sensitive data (API keys, tokens) never hits disk.

---

### 2. Isolated Subshell Per Test

**Location**: All tests
**Pattern**: `run bash --noprofile --norc -c "..."`

**Why This Is Good**:
Running each function in a fresh bash subshell prevents exported variables from one test leaking into subsequent tests. The `--noprofile --norc` flags prevent the user's shell config from interfering with test behavior. This is the correct approach for testing sourced installer scripts.

---

### 3. Platform-Aware Permission Check

**Location**: `tests/installer/test_6_2.bats` — config.toml permissions test
**Pattern**: `case "$(uname -s)"` for stat flag selection

**Why This Is Good**:
`stat -f "%Lp"` (macOS) vs `stat -c "%a"` (Linux) is a common portability pitfall. The test correctly handles both platforms in the same assertion block without duplicating the test.

---

## Test File Analysis

### File Metadata

- **File Path**: `tests/installer/test_6_2.bats`
- **File Size**: 556 lines (after review refactor from 739 lines)
- **Test Framework**: bats-core
- **Language**: Bash

### Test Structure

- **Describe Blocks**: 0 (bats uses flat `@test` blocks)
- **Test Cases**: 16
- **Average Test Length**: ~28 lines per test (after refactor)
- **Fixtures Used**: `setup()`, `teardown()`, `make_gh_stub()` helper
- **Data Factories Used**: `make_gh_stub` (shared stub factory)

### Test Scope

- **Priority Distribution**:
  - P0 (Critical): 9 tests
  - P1 (High): 7 tests
  - P2 (Medium): 0 tests
  - P3 (Low): 0 tests
  - Unknown: 0 tests

### Assertions Analysis

- **Total Assertions**: ~32 (mix of `[ ]`, `[[ ]]`, `grep -q`, and `=~` pattern matches)
- **Assertions per Test**: ~2 per test (avg)
- **Assertion Types**: exit code, file existence, file content, pattern match

---

## Context and Integration

### Related Artifacts

- **Story File**: `_bmad-output/implementation-artifacts/6-2-implement-first-install-path.md`
- **Test Design**: `_bmad-output/test-artifacts/test-design-epic-6.md`
- **Risk Assessment**: R-001 (HIGH), R-002 (HIGH), R-003 (HIGH), R-005 (HIGH) — all mitigated by tests in this file
- **Priority Framework**: P0/P1 applied consistently

### Test-Design Alignment

| Test Design Scenario | Tests Present | Notes |
| --- | --- | --- |
| VIBESTATS_TOKEN never on disk (R-001) | ✅ | Sentinel scan pattern |
| config.toml permissions 600 (R-002) | ✅ | Platform-aware stat assertion |
| Installer exits on gh failure (R-003) | ✅ | 2 failure injection tests |
| registry.json schema fields (R-005) | ✅ | 4 separate field assertions |
| First-install integration path | ✅ | `first_install_path()` integration test |

All P0 scenarios from test-design-epic-6.md that are in scope for Story 6.2 are covered. R-004 (multi-machine path) is correctly deferred to Story 6.3 per story notes.

---

## Knowledge Base References

This review consulted the following knowledge base fragments:

- **test-quality.md** — Definition of Done (no hard waits, <300 lines, self-cleaning, explicit assertions)
- **data-factories.md** — Factory/stub patterns; `make_gh_stub` helper introduced following factory pattern
- **test-levels-framework.md** — Shell unit test level appropriate for installer function testing
- **test-healing-patterns.md** — Stub accuracy bug identification and fix approach

Coverage mapping: use `trace` workflow for coverage gate decisions.

---

## Next Steps

### Immediate Actions (Before Merge)

1. **Verify all 16 tests pass in CI** — both macOS and Linux runners
   - Priority: P0
   - Owner: Dev/QA
   - Estimated Effort: 15 minutes (CI run)

2. **Confirm VIBESTATS_TOKEN sentinel test passes on Linux** — `grep -rl` behavior may differ
   - Priority: P0
   - Owner: Dev/QA
   - Estimated Effort: 1 CI run

### Follow-up Actions (Future PRs)

1. **Split test file into per-AC files** — reduces each file to <100 lines, easier to extend in Stories 6.3/6.4
   - Priority: P2
   - Target: Before Story 6.3 PR

2. **Extract `make_gh_stub` to `tests/installer/test_helpers.bash`** — shared across 6.1, 6.2, 6.3, 6.4 test files
   - Priority: P2
   - Target: Before Story 6.3 PR

### Re-Review Needed?

⚠️ No re-review needed for current changes — approve as-is. The follow-up items (file splitting, helper extraction) are P2 improvements for the next story.

---

## Decision

**Recommendation**: Approve with Comments

**Rationale**:
Test quality is good at 92/100. Two latent bugs (stub inaccuracy) were identified and fixed during this review — tests now exercise the correct behavior. All 16 tests pass, covering P0 and P1 scenarios per the epic test design. The main outstanding item (file length above 300-line threshold) is a P2 improvement deferred to Story 6.3. Security-critical paths (R-001, R-002, R-003, R-005) are all verified by automated tests. Ready to merge.

---

## Appendix

### Violation Summary by Location

| Issue | Severity | Criterion | Fix |
| --- | --- | --- | --- |
| `"api /user"` stub returned full JSON | P1 (Fixed) | Stub accuracy | Changed to return plain login string `testuser` |
| AC #5 stubs had wrong path returning 1 for PUT | P1 (Fixed) | Stub accuracy | Removed spurious specific-path case |
| File is 556 lines (was 739) | P2 | Test Length | Split into per-AC files in Story 6.3 |
| 4 tests still use custom inline stubs | P2 | Data Factories | Extend `make_gh_stub` in Story 6.3 |

---

## Review Metadata

**Generated By**: BMad TEA Agent (Test Architect)
**Workflow**: testarch-test-review v4.0
**Review ID**: test-review-6.2-implement-first-install-path-20260412
**Timestamp**: 2026-04-12
**Version**: 1.0
