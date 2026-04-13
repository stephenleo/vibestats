---
stepsCompleted: ['step-01-load-context', 'step-02-discover-tests', 'step-03-quality-evaluation', 'step-03f-aggregate-scores', 'step-04-generate-report']
lastStep: 'step-04-generate-report'
lastSaved: '2026-04-13'
workflowType: 'testarch-test-review'
inputDocuments:
  - 'tests/installer/test_6_2.bats'
  - '_bmad-output/test-artifacts/atdd-checklist-9.3-fix-test-6-2-bats-pre-existing-failures.md'
  - '_bmad-output/test-artifacts/test-design-epic-9.md'
  - '_bmad/tea/config.yaml'
---

# Test Quality Review: test_6_2.bats

**Quality Score**: 99/100 (A — Excellent)
**Review Date**: 2026-04-13
**Review Scope**: Single file
**Reviewer**: TEA Agent (Story 9.3)

---

Note: This review audits existing tests; it does not generate tests.
Coverage mapping and coverage gates are out of scope here. Use `trace` for coverage decisions.

## Executive Summary

**Overall Assessment**: Excellent

**Recommendation**: Approve

### Key Strengths

- Perfect test isolation: every test creates a fresh `$HOME` via `mktemp -d` in `setup()` and destroys it with `rm -rf` in `teardown()` — parallel-safe with zero state leakage
- Fully deterministic: all 16 tests run the same code path on every invocation with no conditionals, no `sleep`, no random data, and no external network calls
- Correct stub implementation: `make_gh_stub()` produces the proper JSON return (`'{"login":"testuser"}'`) for `_gh api /user`, matching the `python3 -c "import sys, json; print(json.load(sys.stdin)['login'])"` parse in `install.sh`
- Descriptive test names with explicit P0/P1 priority markers and AC mapping comments above each test block
- All 16 tests are GREEN and the full regression suite (42 tests across test_6_1–test_6_4) passes with 0 failures

### Key Weaknesses

- Tests 7 and 8 have nearly identical inline heredoc stubs — a LOW-severity duplication that could be extracted to a shared `make_register_machine_stub` helper (cosmetic; does not affect reliability)
- File is 630 lines total, above the 300-line soft limit for a single test file; however, this is standard for a bats installer test suite covering 5 acceptance criteria with 16 test cases and is not a maintainability risk

### Summary

Story 9.3 fixed two pre-existing root causes in `tests/installer/test_6_2.bats`: (1) the `_gh api /user` stub returned a plain string `"testuser"` instead of the JSON object `'{"login":"testuser"}'` required by `install.sh`'s `python3` JSON parse, causing all 16 tests to fail before the function under test was reached; (2) three tests called `store_machine_token()` which was removed in Story 6.3 and merged into `register_machine()`.

The implementation correctly addresses both root causes in the test stubs only, without touching `install.sh`. The resulting tests are of excellent quality — fully deterministic, perfectly isolated, and fast. The single minor finding (duplication in two inline stubs) is cosmetic and should be addressed in a follow-up PR.

---

## Quality Criteria Assessment

| Criterion                            | Status    | Violations | Notes                                                        |
| ------------------------------------ | --------- | ---------- | ------------------------------------------------------------ |
| BDD Format (Given-When-Then)         | ✅ PASS   | 0          | AC comments above each test serve as Given/When/Then context |
| Test IDs                             | ✅ PASS   | 0          | P0/P1 markers + AC# references in all test names            |
| Priority Markers (P0/P1/P2)          | ✅ PASS   | 0          | All 16 tests have [P0] or [P1] prefix                       |
| Hard Waits (sleep, waitForTimeout)   | ✅ PASS   | 0          | No sleep, no timeout, no arbitrary waits anywhere            |
| Determinism (no conditionals)        | ✅ PASS   | 0          | All assertions are unconditional; stubs are pure dispatchers |
| Isolation (cleanup, no shared state) | ✅ PASS   | 0          | setup/teardown correctly scope $HOME per test                |
| Fixture Patterns                     | ✅ PASS   | 0          | make_gh_stub() is a clean fixture factory                    |
| Data Factories                       | ✅ PASS   | 0          | Fixed test tokens (ghp_FAKE_*) are intentional, not magic    |
| Network-First Pattern                | N/A       | 0          | Shell/bats tests; no browser navigation                      |
| Explicit Assertions                  | ✅ PASS   | 0          | All assertions in test bodies, none hidden in helpers        |
| Test Length (≤300 lines per test)    | ✅ PASS   | 0          | Avg ~35 lines per test block                                 |
| Test Duration (≤1.5 min)            | ✅ PASS   | 0          | Each test runs a subshell with stub fns — sub-second         |
| Flakiness Patterns                   | ✅ PASS   | 0          | No race conditions, no timing dependencies                   |

**Total Violations**: 0 Critical, 0 High, 0 Medium, 1 Low

---

## Quality Score Breakdown

```
Starting Score:          100
Critical Violations:       0 × 10 =   0
High Violations:           0 × 5  =   0
Medium Violations:         0 × 2  =   0
Low Violations:            1 × 1  =  -1

Bonus Points:
  Excellent BDD structure:           +0  (inline comments, not formal BDD)
  Comprehensive Fixtures:            +5  (make_gh_stub helper is excellent)
  Data Factories:                    +0  (N/A — shell tests use fixed stubs)
  Network-First:                     +0  (N/A — shell/bats)
  Perfect Isolation:                 +5  (mktemp HOME isolation is textbook)
  All Test IDs (P0/P1 markers):      +5  (all 16 tests labelled)
                                    --------
Total Bonus:                         +15  (capped at score = 99 after deduction)

Final Score:             99/100
Grade:                   A
```

---

## Critical Issues (Must Fix)

No critical issues detected. All 16 tests pass. Full regression suite (42 tests) passes with 0 failures.

---

## Recommendations (Should Fix)

### 1. Extract shared inline stub for register_machine tests (Tests 7, 8, 9)

**Severity**: P3 (Low)
**Location**: `tests/installer/test_6_2.bats:282–413`
**Criterion**: Maintainability — minor duplication

**Issue Description**:
Tests 7, 8, and 9 each write an inline heredoc `stub_env.sh` with nearly identical `_gh()` bodies. Tests 7 and 8 are essentially the same stub; Test 9 differs only in the `auth token` case. Consolidating into a named helper (e.g., `make_register_machine_stub`) would remove ~60 lines of duplication and make future stub changes a single-edit operation.

**Current Code** (abridged — same pattern in Tests 7, 8):

```bash
cat > "${HOME}/stub_env.sh" <<'STUB'
_gh() {
  case "$1 $2" in
    "auth token")
      echo "ghp_FAKE_MACHINE_TOKEN_12345"
      ;;
    "api /user")
      echo '{"login":"testuser"}'
      ;;
    "api repos"*)
      # ... PUT/GET dispatch logic
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

Add a `make_register_machine_stub()` helper below `make_gh_stub()`:

```bash
# Stub for tests that call register_machine (Tests 7, 8, 9).
# $1 (optional): stub file path (default: ${HOME}/stub_env.sh)
# $2 (optional): auth_token return value (default: "ghp_FAKE_MACHINE_TOKEN_12345")
# $3 (optional): auth_token exit code (default: 0)
make_register_machine_stub() {
  local stub_file="${1:-${HOME}/stub_env.sh}"
  local auth_token="${2:-ghp_FAKE_MACHINE_TOKEN_12345}"
  local auth_exit="${3:-0}"

  cat > "$stub_file" <<STUB
_gh() {
  case "\$1 \$2" in
    "auth token")
      [ "${auth_exit}" -eq 0 ] || { echo "Error: not authenticated" >&2; return 1; }
      echo "${auth_token}"
      ;;
    "api /user")
      echo '{"login":"testuser"}'
      ;;
    "api repos"*)
      case "\$*" in
        *"--method PUT"*) return 0 ;;
        *)                return 1 ;;
      esac
      ;;
    *)
      return 0
      ;;
  esac
}
export -f _gh
STUB
}
```

Then replace inline heredocs in Tests 7, 8, 9 with calls:
- Test 7: `make_register_machine_stub`
- Test 8: `make_register_machine_stub`
- Test 9: `make_register_machine_stub "${HOME}/stub_env.sh" "" 1` (non-zero auth exit)

**Benefits**:
- Removes ~60 lines of duplicated heredoc content
- A single function handles all register_machine stub variants
- Consistent with the existing `make_gh_stub()` pattern

**Priority**: P3 — low urgency; tests pass as-is. Recommended for a follow-up cleanup PR.

---

## Best Practices Found

### 1. Isolated `$HOME` per test via `mktemp -d`

**Location**: `tests/installer/test_6_2.bats:24–34`
**Pattern**: Complete process isolation with per-test temp directories

**Why This Is Good**:
Each test gets a brand-new empty `$HOME` directory that is guaranteed not to exist before or after. This prevents any state leakage between tests and makes the suite fully parallelizable. The paired `rm -rf "$HOME"` teardown ensures clean state. This is the canonical pattern for installer bats tests.

**Code Example**:

```bash
setup() {
  export HOME
  HOME="$(mktemp -d)"
  export BATS_TMPDIR="${HOME}/bats-tmp"
  mkdir -p "$BATS_TMPDIR"
}

teardown() {
  rm -rf "$HOME"
}
```

### 2. Reusable `make_gh_stub()` fixture factory

**Location**: `tests/installer/test_6_2.bats:51–120`
**Pattern**: Fixture factory with optional case injection

**Why This Is Good**:
`make_gh_stub()` generates a complete `stub_env.sh` that can be sourced in subshells. It accepts an optional `extra_cases` parameter so callers can inject test-specific overrides without rewriting the entire stub. This reduces boilerplate for the majority of tests (Tests 1–6, 10–13) while keeping test-specific variants readable.

**Code Example**:

```bash
make_gh_stub() {
  local stub_file="${1:-${HOME}/stub_env.sh}"
  local extra_cases="${2:-}"

  cat > "$stub_file" <<STUB
_gh() {
  case "\$1 \$2" in
    "api /user")
      echo '{"login":"testuser"}'   # JSON required by install.sh python3 parse
      ;;
    # ... other cases
  esac
}
export -f _gh
${extra_cases}
STUB
}
```

### 3. Correct JSON stub for `_gh api /user`

**Location**: `tests/installer/test_6_2.bats:62`
**Pattern**: Stub return value matches the production code's parsing requirement

**Why This Is Good**:
The original bug in test_6_2.bats was that `"api /user"` returned a plain string, while `install.sh` uses `python3 -c "import sys, json; print(json.load(sys.stdin)['login'])"` to parse it. The fix correctly returns a JSON object. This is a good example of aligning stubs with the actual interface contract rather than the test author's mental model of the interface.

```bash
"api /user")
  echo '{"login":"testuser"}'   # Must be valid JSON for python3 parse in install.sh
  ;;
```

### 4. Platform-aware file permission check

**Location**: `tests/installer/test_6_2.bats:366–375`
**Pattern**: Defensive platform detection in assertions

**Why This Is Good**:
The `stat` command uses different flags on macOS (`-f "%Lp"`) vs Linux (`-c "%a"`). The test correctly branches on `uname -s` to use the right invocation. This makes the test portable across CI platforms.

```bash
case "$(uname -s)" in
  Darwin)
    PERMS=$(stat -f "%Lp" "${HOME}/.config/vibestats/config.toml")
    ;;
  Linux)
    PERMS=$(stat -c "%a" "${HOME}/.config/vibestats/config.toml")
    ;;
esac
[ "$PERMS" = "600" ]
```

---

## Test File Analysis

### File Metadata

- **File Path**: `tests/installer/test_6_2.bats`
- **File Size**: 630 lines, ~16 KB
- **Test Framework**: bats-core (v1.13.0)
- **Language**: Bash/shell

### Test Structure

- **Describe Blocks**: 0 (bats uses `@test` directly; AC mapping via comments)
- **Test Cases (@test)**: 16
- **Average Test Length**: ~35 lines per test block
- **Fixtures Used**: 1 (`make_gh_stub()` helper function)
- **Data Factories Used**: 0 (fixed test tokens; appropriate for security-sensitive installer tests)

### Test Scope

- **Priority Distribution**:
  - P0 (Critical): 9 tests
  - P1 (High): 7 tests
  - P2 (Medium): 0 tests
  - P3 (Low): 0 tests

### Assertions Analysis

- **Total Assertions**: ~40 (`[ ]`, `[[ ]]`, `grep -q` checks)
- **Assertions per Test**: ~2.5 (avg)
- **Assertion Types**: exit code checks, file existence, grep content, regex pattern match

---

## Context and Integration

### Related Artifacts

- **ATDD Checklist**: `_bmad-output/test-artifacts/atdd-checklist-9.3-fix-test-6-2-bats-pre-existing-failures.md`
- **Test Design**: `_bmad-output/test-artifacts/test-design-epic-9.md`
- **Risk Level**: Pre-launch blocker (R-004)
- **Priority Framework**: P0/P1 applied to all 16 tests

The ATDD checklist documented both root causes accurately:
1. Issue 1 — `_gh api /user` returning plain string instead of JSON (7 stub locations)
2. Issue 2 — `store_machine_token()` removed in Story 6.3, calls not updated (3 tests)

The implementation addresses both root causes exactly as specified, with no `install.sh` modifications and no xfail/skip workarounds.

---

## Knowledge Base References

- **[test-quality.md](../../../.claude/skills/bmad-testarch-test-review/resources/knowledge/test-quality.md)** — Determinism, isolation, explicit assertions, no hard waits, self-cleaning tests
- **[data-factories.md](../../../.claude/skills/bmad-testarch-test-review/resources/knowledge/data-factories.md)** — Shell stub/mock design adapted for bats `_gh()` override pattern
- **[test-levels-framework.md](../../../.claude/skills/bmad-testarch-test-review/resources/knowledge/test-levels-framework.md)** — Shell (bats) as correct test level for installer scripts
- **[ci-burn-in.md](../../../.claude/skills/bmad-testarch-test-review/resources/knowledge/ci-burn-in.md)** — Regression guard requirements (full suite must pass)

---

## Next Steps

### Immediate Actions (Before Merge)

None required. Tests are production-ready and the story is complete.

### Follow-up Actions (Future PRs)

1. **Extract `make_register_machine_stub()` helper** — Consolidate the three inline heredoc stubs in Tests 7, 8, 9 into a shared factory function.
   - Priority: P3
   - Target: Backlog / cleanup sprint
   - Estimated Effort: 15 minutes

### Re-Review Needed?

No re-review needed — approve as-is.

---

## Decision

**Recommendation**: Approve

**Rationale**:
Test quality is excellent with 99/100 score. All 16 tests in `test_6_2.bats` pass (GREEN), and the full regression suite of 42 tests across `test_6_1.bats`–`test_6_4.bats` passes with zero failures. Both root causes documented in the ATDD checklist are correctly addressed in the test stubs without modifying `install.sh`. Tests are deterministic, isolated, fast, and follow all quality criteria.

The single LOW-severity finding (minor duplication in two inline stubs) is cosmetic and does not affect reliability or correctness. It can be addressed in a future cleanup PR.

> Test quality is excellent with 99/100 score. Tests are production-ready and follow best practices. The pre-launch blocker condition (AC #1) is satisfied: `bats tests/installer/test_6_1.bats tests/installer/test_6_2.bats tests/installer/test_6_3.bats tests/installer/test_6_4.bats` exits 0 with 42/42 passing.

---

## Appendix

### Violation Summary by Location

| Line    | Severity | Criterion       | Issue                                      | Fix                                         |
| ------- | -------- | --------------- | ------------------------------------------ | ------------------------------------------- |
| 282–413 | LOW      | Maintainability | Inline stubs in Tests 7/8/9 are duplicated | Extract `make_register_machine_stub()` helper |

### Test Execution Evidence (GREEN Phase)

```
Command: bats tests/installer/test_6_2.bats
Result:
1..16
ok 1 [P1] vibestats-data does not exist → gh repo create --private called
ok 2 [P1] gh repo create called with --private flag
ok 3 [P1] aggregate.yml written calling stephenleo/vibestats@v1
ok 4 [P1] aggregate.yml workflow content includes cron and workflow_dispatch triggers
ok 5 [P0] VIBESTATS_TOKEN is never written to disk or echoed to stdout
ok 6 [P1] gh secret set called with VIBESTATS_TOKEN for vibestats-data repo
ok 7 [P1] gh auth token result stored in ~/.config/vibestats/config.toml
ok 8 [P0] ~/.config/vibestats/config.toml created with permissions 600
ok 9 [P0] installer exits non-zero and prints error when gh auth token fails
ok 10 [P0] registry.json entry contains machine_id field
ok 11 [P0] registry.json entry contains hostname field
ok 12 [P0] registry.json entry has status field set to active
ok 13 [P0] registry.json entry has last_seen ISO 8601 UTC timestamp
ok 14 [P0] installer exits non-zero when gh repo create fails
ok 15 [P0] installer exits non-zero when gh secret set fails
ok 16 [P1] full first-install path succeeds with all steps called in sequence

Summary: 16/16 ok — EXIT CODE 0
```

```
Command: bats tests/installer/test_6_1.bats tests/installer/test_6_2.bats tests/installer/test_6_3.bats tests/installer/test_6_4.bats
Result: 1..42 — 42/42 ok — EXIT CODE 0
```

---

## Review Metadata

**Generated By**: BMad TEA Agent (Test Architect) — Step 4 Test Reviewer
**Workflow**: testarch-test-review
**Review ID**: test-review-9.3-fix-test-6-2-bats-pre-existing-failures-20260413
**Story**: 9.3-fix-test-6-2-bats-pre-existing-failures
**Timestamp**: 2026-04-13
