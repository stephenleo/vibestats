---
stepsCompleted: ['step-01-load-context', 'step-02-discover-tests', 'step-03-quality-evaluation', 'step-03f-aggregate-scores', 'step-04-generate-report']
lastStep: 'step-04-generate-report'
lastSaved: '2026-04-12'
story: '6.1-implement-dependency-detection-and-gh-authentication'
inputDocuments:
  - tests/installer/test_6_1.bats
  - install.sh
  - _bmad-output/test-artifacts/test-design-epic-6.md
---

# Test Review — Story 6.1: Implement Dependency Detection and gh Authentication

## Overview

| Field | Value |
|---|---|
| Story | 6.1 — Implement dependency detection and gh authentication |
| Review Date | 2026-04-12 |
| Test File | `tests/installer/test_6_1.bats` |
| Framework | bats-core (Bash shell tests) |
| Scope | Single file — Story 6.1 acceptance criteria |
| Test Design Reference | `_bmad-output/test-artifacts/test-design-epic-6.md` |

---

## Quality Score Summary

| Dimension | Score | Grade | Weight | Weighted Score |
|---|---|---|---|---|
| Determinism | 90/100 | A- | 30% | 27.0 |
| Isolation | 90/100 | A- | 30% | 27.0 |
| Maintainability | 75/100 | C | 25% | 18.75 |
| Performance | 95/100 | A | 15% | 14.25 |
| **Overall** | **87/100** | **B** | | |

> Coverage is excluded from `test-review` scoring. Use `trace` for coverage analysis and gates.

---

## Violations Found

| Severity | Count |
|---|---|
| HIGH | 0 |
| MEDIUM | 2 |
| LOW | 0 |
| **Total** | **2** |

---

## Findings

### MEDIUM — Duplicate `case` branch in `check_gh_auth` mock (BUG — fixed)

**File:** `tests/installer/test_6_1.bats` — test `[P1] gh not authenticated → gh auth login called`

**Description:** The `_gh()` mock used a `case "$1 $2"` statement with two `"auth status"` arms. In Bash `case` statements, the first matching pattern always wins — the second `"auth status"` arm was unreachable dead code. This meant after `_gh auth login` was called, the subsequent `_gh auth status` check (which verifies authentication succeeded) still returned 1 (not authenticated), causing `check_gh_auth` to exit non-zero and the test to fail with a misleading error.

**Root Cause:** Attempting to simulate stateful behavior (auth state changes after login) using a `case` statement, which is inherently stateless.

**Fix Applied:** Replaced the duplicate-case approach with a flag-file pattern. The `_gh auth login` arm now creates `${HOME}/gh_logged_in.flag`. The `_gh auth status` arm checks for the flag file: returns 0 if it exists (post-login), returns 1 if it does not (pre-login). The heredoc delimiter was changed from `'STUB'` (single-quoted, no expansion) to unquoted `STUB` so the actual temp `$HOME` path is baked into the stub at write time, making the flag file path deterministic in subshells.

Also added a missing `[ "$status" -eq 0 ]` assertion to verify `check_gh_auth` completes successfully end-to-end.

---

### MEDIUM — Dead `_source_install_functions()` helper (removed)

**File:** `tests/installer/test_6_1.bats` — lines 28–33 (original)

**Description:** A helper function `_source_install_functions()` was defined but never called by any test. It sourced `install.sh` silently (`2>/dev/null || true`), suppressing errors. Keeping unused helpers with silent error suppression violates the principle of explicit, debuggable test infrastructure — a sourcing failure would go unnoticed.

**Fix Applied:** Removed the unused helper entirely. All tests correctly source `install.sh` directly inside their `bash --noprofile --norc -c "..."` subshells.

---

## Dimension Analyses

### Determinism (90/100 — A-)

No violations. All tests:
- Use no `Math.random()`, `Date.now()`, or `new Date()` calls
- Use no hard waits (`sleep`, `waitForTimeout`)
- Mock all `gh` CLI calls via `_gh()` overrides (no external network calls)
- Use `mktemp -d` for isolated temp directories (appropriate for shell test isolation)

The `builtin command "$@"` fallback in some mocks passes through non-targeted subcommands to real system commands (`command`, `uname`). This is low-risk since only explicitly intercepted args are tested.

### Isolation (90/100 — A-)

Strong isolation:
- `setup()` creates a fresh `mktemp -d` home directory per test
- `teardown()` `rm -rf "$HOME"` cleans up all artifacts
- Each test writes its own `stub_env.sh` inside its private `$HOME`
- No shared mutable state between tests
- Tests do not depend on each other's execution order

### Maintainability (75/100 — C)

Two issues found and fixed (see Findings above). Post-fix:
- Test names clearly identify priority `[P1]`/`[P2]` and acceptance criteria reference (`AC #N`)
- Header comments in each test block reference story, FR/NFR/risk IDs
- Consistent pattern: write stub → run function in subshell → assert
- File is 335 lines covering 9 tests — manageable

### Performance (95/100 — A)

Excellent:
- All tests are pure shell unit tests with `mktemp` setup — fast (<2 minutes total)
- No network calls, no database, no real `gh` invocations
- Tests are fully parallelizable (no shared state)
- No hard waits or sleep calls

---

## Context Alignment

| Test Design Requirement | Test Present | Notes |
|---|---|---|
| P1: `gh` not installed → `brew install gh` (Darwin) | ✅ | Covered |
| P1: `gh` not installed → `apt-get install gh` (Linux) | ✅ | Covered |
| P1: `gh` version < 2.0 → exits non-zero with message | ✅ | 2 tests (version in message, "2.0" in message) |
| P1: `gh` not authenticated → `gh auth login` called | ✅ | Fixed in this review |
| P1: Platform detection — Darwin arm64 | ✅ | Covered |
| P1: Platform detection — Darwin x86_64 | ✅ | Covered |
| P1: Platform detection — Linux x86_64 | ✅ | Covered |
| P2: `gh` already installed and version ≥ 2.0 → no install | ✅ | Covered |
| P1: Unsupported platform → exits non-zero | ✅ | Covered |

All Story 6.1 P1 acceptance criteria are covered. The P1 test for `gh` not authenticated had a mock bug that would have caused false failures — this has been fixed.

---

## Recommendations

1. **No further action required for Story 6.1** — all P1 and P2 tests are present and correct after this review.
2. **Story 6.2 (first-install path):** Follow the flag-file stateful mock pattern established in this fix when mocking multi-step `gh` interactions (e.g., `gh repo create` → `gh repo view` state change).
3. **CI matrix:** Story 6.1 platform detection tests should run on both macOS and Linux CI runners to catch any `uname` behavior differences.

---

## Gate Decision

| Gate | Status |
|---|---|
| All P0 tests for story scope | N/A (Story 6.1 has no P0 tests — P0 tests belong to 6.2+ for token/permission handling) |
| All P1 tests present | ✅ PASS |
| All P2 tests present | ✅ PASS (1 P2 test) |
| HIGH severity violations | ✅ NONE |
| MEDIUM violations fixed | ✅ Fixed in this review |
| Ready to merge | ✅ YES |

---

**Generated by:** BMad TEA Agent — Test Reviewer (Step 4)
**Workflow:** `bmad-testarch-test-review`
**Reviewer:** Claude Sonnet 4.6
