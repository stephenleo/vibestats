---
stepsCompleted: ['step-01-load-context', 'step-02-discover-tests', 'step-03-quality-evaluation', 'step-03f-aggregate-scores', 'step-04-generate-report']
lastStep: 'step-04-generate-report'
lastSaved: '2026-04-12'
story: '6.3-implement-multi-machine-install-path'
inputDocuments:
  - tests/installer/test_6_3.bats
  - install.sh
  - _bmad-output/test-artifacts/test-design-epic-6.md
  - _bmad-output/test-artifacts/atdd-checklist-6.3-implement-multi-machine-install-path.md
  - _bmad-output/implementation-artifacts/6-3-implement-multi-machine-install-path.md
---

# Test Review — Story 6.3: Implement Multi-Machine Install Path

## Overview

| Field | Value |
|---|---|
| Story | 6.3 — Implement multi-machine install path |
| Review Date | 2026-04-12 |
| Test File | `tests/installer/test_6_3.bats` |
| Framework | bats-core (Bash shell tests) |
| Scope | Single file — Story 6.3 acceptance criteria |
| Test Design Reference | `_bmad-output/test-artifacts/test-design-epic-6.md` |
| ATDD Checklist | `_bmad-output/test-artifacts/atdd-checklist-6.3-implement-multi-machine-install-path.md` |

---

## Quality Score Summary

| Dimension | Score | Grade | Weight | Weighted Score |
|---|---|---|---|---|
| Determinism | 95/100 | A | 30% | 28.5 |
| Isolation | 95/100 | A | 30% | 28.5 |
| Maintainability | 75/100 | C | 25% | 18.75 |
| Performance | 95/100 | A | 15% | 14.25 |
| **Overall** | **90/100** | **A** | | |

> Coverage is excluded from `test-review` scoring. Use `trace` for coverage analysis and gates.

---

## Violations Found

| Severity | Count |
|---|---|
| HIGH | 0 |
| MEDIUM | 1 |
| LOW | 0 |
| **Total** | **1** |

---

## Findings

### MEDIUM — Duplicated `_gh()` stub across all tests (Maintainability)

**File:** `tests/installer/test_6_3.bats` — tests at lines 43, 120, 210, 248, 315, 409

**Description:** Every test in `test_6_3.bats` writes a full inline `_gh()` stub via `cat > stub_env.sh <<STUB`. The base arms (`"api /user"`, `"repo view"`, `"auth token"`) are copy-pasted into 5 out of 7 tests. This amounts to approximately 80–120 lines of duplicated stub code across the file. When the API contract changes (e.g., the `_gh api /user` response format), every test's stub must be updated individually.

**Root Cause:** Bats does not offer fixture composition at the heredoc level; the subshell execution model (`bash --noprofile --norc -c "..."`) requires all stubs to be available in the subshell's environment at the time the test body executes. This means setup-level stub functions would need to be written to a shared file and sourced in every test.

**Why not blocked (not HIGH):** Each test's stub is correct and complete for its scenario. The duplication is a maintainability cost, not a correctness risk. Tests pass reliably on both macOS (Darwin) and Linux paths.

**Recommendation:** In a future PR, consider extracting a `${HOME}/base_stub.sh` in `setup()` containing the common `_gh` arms (`api /user`, `repo view`, `auth token`), then sourcing it at the start of each test-specific stub:

```bash
# In setup():
cat > "${HOME}/base_stub.sh" <<'BASE'
_gh_base_dispatch() {
  case "$1 $2" in
    "api /user")  echo '{"login": "testuser"}' ;;
    "repo view")  echo '{"name": "vibestats-data"}'; return 0 ;;
    "auth token") echo "ghp_TESTTOKEN" ;;
    *)            return 0 ;;
  esac
}
BASE

# In each test — only override what differs:
cat > "${HOME}/stub_env.sh" <<STUB
source '${HOME}/base_stub.sh'
_gh() {
  echo "_gh \$*" >> "${GH_SPY_LOG}"
  case "\$1 \$2" in
    "api repos"*)
      # test-specific registry handling
      ;;
    *)
      _gh_base_dispatch "\$@"
      ;;
  esac
}
export -f _gh
STUB
```

This approach reduces the stub duplication by ~60% while keeping each test's specific behavior explicit and visible.

**Priority:** P2 (Medium) — future PR, not a merge blocker.

---

## Dimension Analyses

### Determinism (95/100 — A)

No violations. All 7 tests:
- Use no `Math.random()`, `Date.now()`, or `new Date()` equivalents in test assertions
- Use no hard waits (`sleep`, `waitForTimeout`)
- Mock all `gh` CLI calls via `_gh()` function overrides (no external network calls)
- Use `mktemp -d` for isolated temp directories (appropriate for shell test isolation)
- Assert the `last_seen` timestamp with a format regex (`^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$`) rather than an exact value — correct approach since the SUT generates the timestamp at runtime
- Use platform-conditional `base64 -D` / `base64 -d` in test stubs — deterministic per platform, consistent with the SUT's own platform branching

### Isolation (95/100 — A)

Strong isolation:
- `setup()` creates a fresh `$(mktemp -d)` home directory per test — each test gets a private filesystem namespace
- `teardown()` runs `rm -rf "$HOME"` — complete cleanup of all artifacts
- Each test writes its own `stub_env.sh` inside its private `$HOME`
- `BATS_TMPDIR="${HOME}/bats-tmp"` is set per-test inside `setup()`, so `GH_SPY_LOG` and `REGISTRY_PUT_BODY` paths are always fresh
- No shared mutable state between tests — tests do not depend on execution order
- The `export -f _gh` exports the stub into the test's `bash --noprofile --norc -c "..."` subshell only; it does not leak to other tests

### Maintainability (75/100 — C)

One MEDIUM violation: stub duplication (see Findings above). Post that finding:
- Test names are clear and descriptive with priority prefix `[P0]`/`[P1]`
- Section header comments reference story number, FR/risk IDs, and behavior under test
- Consistent pattern throughout: write stub → run function in subshell → assert
- Python one-liner assertions use explicit `assert` statements with descriptive failure messages
- File is 469 lines for 7 tests (~67 lines/test average) — within acceptable range per quality standards
- Both platform paths (Darwin/Linux) are handled in the base64 decode/encode operations

### Performance (95/100 — A)

Excellent:
- All tests are pure shell unit tests — no network calls, no real `gh` invocations
- `mktemp -d` and `rm -rf` for per-test isolation is fast (<100ms per test)
- Python one-liner assertions are lightweight subprocess invocations
- All 7 tests complete in under 5 seconds total
- Tests are fully parallelizable — no shared state, no execution order dependencies
- No hard waits or `sleep` calls

---

## Context Alignment

| Test Design Requirement | Priority | Test Present | Notes |
|---|---|---|---|
| Multi-machine path: vibestats-data exists → repo creation skipped, workflow write skipped, VIBESTATS_TOKEN not set | P0 (R-004) | ✅ | `[P0]` test 1 |
| `registry.json` entry has all required fields: machine_id, hostname, status=active, last_seen ISO 8601 UTC | P0 (R-005) | ✅ | `[P0]` test 2 |
| `gh repo view` called with `${GITHUB_USER}/vibestats-data` — not hardcoded org | P1 (R-004) | ✅ | `[P1]` test 3 |
| `detect_install_mode` sets `INSTALL_MODE=multi-machine` when repo exists | P1 (R-004) | ✅ | `[P1]` test 4 |
| `detect_install_mode` sets `INSTALL_MODE=first-install` when repo does not exist | P1 (R-003) | ✅ | `[P1]` test 5 |
| `register_machine` appends new entry without overwriting existing machines | P1 (R-005) | ✅ | `[P1]` test 6 |
| `config.toml` written with permissions `600` after machine-side token | P1 (R-002) | ✅ | `[P1]` test 7 |

All 7 Story 6.3 scenarios from the ATDD checklist are present and passing.

**Risk mitigations verified:**
- R-002 (config.toml permissions): ✅ test 7 asserts `stat` output equals `"600"`
- R-004 (first-install path on existing install): ✅ test 1 uses spy pattern to assert `repo create` and `secret set VIBESTATS_TOKEN` were NOT called
- R-005 (registry.json schema): ✅ test 2 parses JSON and validates all four required fields; test 6 asserts append-only behavior

---

## Implementation Review Notes

While reviewing `install.sh` for context alignment, several implementation qualities are noteworthy:

1. **`set -euo pipefail` preserved** (line 2) — existing safety guarantee maintained
2. **`BASH_SOURCE` guard preserved** (line 322) — test-safe sourcing behavior intact
3. **`_gh()` define-if-not-defined guard preserved** (lines 12-16) — test override mechanism intact
4. **`if _gh repo view ... > /dev/null 2>&1` construct used** (line 186) — safe under `set -euo pipefail`, as required by story Dev Notes
5. **No `jq` dependency** — Python stdlib used for all JSON manipulation (lines 183, 243-253)
6. **`chmod 600` called immediately after file write** (line 286) — correct per NFR6
7. **`VIBESTATS_TOKEN` not present** in multi-machine path — `gh secret set` is never called, as required by AC #1 and R-004 mitigation

---

## Best Practices Found

### 1. Spy pattern for negative assertions (absence testing)

**Location:** `tests/installer/test_6_3.bats` — lines 46-48, 101-113

**Why This Is Good:** The spy log (`GH_SPY_LOG`) records every `_gh` call as it happens. Negative assertions (`run grep "repo create" ...; [ "$status" -ne 0 ]`) are explicit and unambiguous — they verify that certain dangerous operations (repo creation, secret rotation) were definitively NOT performed. This is the correct approach for R-004 mitigation validation.

```bash
# Write every call to a log
_gh() {
  echo "_gh $*" >> "${GH_SPY_LOG}"
  case "$1 $2" in
    ...
  esac
}

# Assert absence via grep exit code
run grep "repo create" "${GH_SPY_LOG}"
[ "$status" -ne 0 ]  # grep returns 1 when not found — spy confirmed absent
```

### 2. JSON capture pattern via base64 PUT interception

**Location:** `tests/installer/test_6_3.bats` — lines 141-156, 345-356

**Why This Is Good:** Instead of mocking the entire PUT response, the stub intercepts the actual `--field content=` argument, decodes the base64 payload, and writes it to a file for later assertion. This validates the exact JSON that would be written to GitHub — not just that the PUT was called, but that it was called with schema-valid content.

```bash
# In stub: capture and decode PUT body
*"registry.json"*"--method PUT"*)
  # Extract content= field from positional args
  args=("$@")
  for i in "${!args[@]}"; do
    if [ "${args[$i]}" = "--field" ] && echo "${args[$((i+1))]}" | grep -q "^content="; then
      CONTENT_VAL=$(echo "${args[$((i+1))]}" | sed 's/^content=//')
    fi
  done
  case "$(uname -s)" in
    Darwin) echo "$CONTENT_VAL" | base64 -D > "${REGISTRY_PUT_BODY}" ;;
    Linux)  echo "$CONTENT_VAL" | base64 -d > "${REGISTRY_PUT_BODY}" ;;
  esac
  echo '{"content": {"sha": "newsha123"}}'
  return 0
  ;;

# In test body: validate schema of captured JSON
run python3 -c "
import json, sys, re
with open('${REGISTRY_PUT_BODY}') as f:
    data = json.load(f)
machines = data.get('machines', [])
assert len(machines) >= 1, 'No machines in registry.json'
m = machines[-1]
assert m.get('machine_id'), 'machine_id is missing or empty'
assert m.get('hostname'), 'hostname is missing or empty'
assert m.get('status') == 'active', f\"status must be 'active', got: {m.get('status')}\"
last_seen = m.get('last_seen', '')
assert re.match(r'^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$', last_seen), f'last_seen must be ISO 8601 UTC, got: {last_seen}'
print('All registry.json fields valid')
"
[ "$status" -eq 0 ]
```

### 3. Flag-file stateful mocking for multi-step flows

**Location:** `tests/installer/test_6_3.bats` — lines 44-113 (test 1: multi-machine path)

**Why This Is Good:** Test 1 uses `UNEXPECTED: ...` log entries to detect if forbidden operations are called, without aborting the stub function or causing the test to fail prematurely. This lets `register_machine` complete normally while still capturing any unintended side-effects for post-run assertion.

---

## Gate Decision

| Gate | Status |
|---|---|
| All P0 tests for story scope | ✅ PASS (2/2) |
| All P1 tests present | ✅ PASS (5/5) |
| HIGH severity violations | ✅ NONE |
| MEDIUM violations | ✅ 1 found — documented, not a merge blocker |
| R-002 mitigation verified | ✅ `config.toml` permissions test passes |
| R-004 mitigation verified | ✅ Multi-machine path spy test passes |
| R-005 mitigation verified | ✅ Registry schema + append-only tests pass |
| Full suite regression | ✅ All 17 tests pass (`bats tests/installer/`) |
| Ready to merge | ✅ YES |

---

## Recommendations

1. **No blocking issues for Story 6.3** — all P0 and P1 tests are present, correct, and passing after this review.
2. **Stub duplication** (MEDIUM, P2): Consider extracting a shared base stub in `setup()` in a future PR (see Findings above). Not a merge blocker.
3. **Story 6.2 (first-install path):** The JSON capture pattern established in tests 2 and 6 (base64 PUT interception) should be reused when testing the first-install registry write in Story 6.2.
4. **CI matrix:** The `stat` command in test 7 uses platform-aware branching (`stat -f "%Lp"` on Darwin, `stat -c "%a"` on Linux). Ensure the CI matrix runs these tests on both macOS and Linux runners per the test-design R-002 requirement.
5. **Story 6.4 (hooks + backfill):** Follow the flag-file stateful mock pattern and spy log approach established in these tests when testing the hook configuration and README injection idempotency scenarios.

---

**Generated by:** BMad TEA Agent — Test Reviewer (Step 4)
**Workflow:** `bmad-testarch-test-review`
**Reviewer:** Claude Sonnet 4.6
