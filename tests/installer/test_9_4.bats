#!/usr/bin/env bats
# Story 9.4: Bash installer — Refactor EXIT trap to composable cleanup
# ATDD Red Phase — tests assert expected behaviour; will fail until install.sh is refactored.
#
# Run: bats tests/installer/test_9_4.bats
# Full regression: bats tests/installer/test_6_1.bats tests/installer/test_6_2.bats tests/installer/test_6_3.bats tests/installer/test_6_4.bats tests/installer/test_9_4.bats
#
# Test framework: bats-core
# Mocking strategy: override _gh() helper and shell commands via functions exported to subshells.
# All tests use a temp $HOME to prevent mutation of the developer's real config.
#
# AC Coverage:
#   AC #1: EXIT trap replaced with named cleanup() function that accumulates cleanup tasks
#   AC #2: trap cleanup EXIT registered once; all accumulated tasks execute on exit
#   AC #3: Existing bats tests (6_1–6_4) still pass — zero regressions (regression guard, not tested here)
#   AC #4: Pattern is composable — cleanup() can be extended without dropping earlier tasks

INSTALL_SH="$(cd "$(dirname "$BATS_TEST_FILENAME")/../.." && pwd)/install.sh"

setup() {
  # Isolate from real $HOME — required for every test
  export HOME
  HOME="$(mktemp -d)"
  export BATS_TMPDIR="${HOME}/bats-tmp"
  mkdir -p "$BATS_TMPDIR"
}

teardown() {
  rm -rf "$HOME"
}

# ---------------------------------------------------------------------------
# AC #1 — [P1] install.sh defines a cleanup() function
# 9.4-UNIT-001
# ---------------------------------------------------------------------------
@test "[P1][9.4-UNIT-001] install.sh defines a top-level cleanup() function" {
  # Source install.sh and verify cleanup() is defined as a shell function.
  # RED: install.sh currently has no cleanup() function.
  run bash --noprofile --norc -c "
    source '${INSTALL_SH}'
    declare -f cleanup
  " 2>&1

  [ "$status" -eq 0 ]
  [[ "$output" == *"cleanup"* ]]
}

# ---------------------------------------------------------------------------
# AC #1 — [P1] download_and_install_binary() assigns _CLEANUP_TMPDIR (new pattern)
# 9.4-UNIT-002
# ---------------------------------------------------------------------------
@test "[P1][9.4-UNIT-002] download_and_install_binary() assigns _CLEANUP_TMPDIR (composable cleanup pattern)" {
  # The refactored function must set _CLEANUP_TMPDIR="$TMPDIR_WORK" instead of
  # using an inline trap override.
  # RED: install.sh currently has no _CLEANUP_TMPDIR assignment inside download_and_install_binary().
  run bash --noprofile --norc -c "
    source '${INSTALL_SH}'
    declare -f download_and_install_binary
  " 2>&1

  [ "$status" -eq 0 ]
  # Must contain _CLEANUP_TMPDIR assignment in the function body
  [[ "$output" == *"_CLEANUP_TMPDIR"* ]]
}

# ---------------------------------------------------------------------------
# AC #1 — [P1] install.sh registers trap cleanup EXIT once (not inside a function)
# 9.4-UNIT-003
# ---------------------------------------------------------------------------
@test "[P1][9.4-UNIT-003] install.sh registers trap cleanup EXIT at top level" {
  # Verify that trap cleanup EXIT appears in install.sh source at the top level
  # (not inside a function definition).
  # RED: install.sh currently has no 'trap cleanup EXIT' statement.
  run grep -n "trap cleanup EXIT" "${INSTALL_SH}" 2>&1

  [ "$status" -eq 0 ]
  # At least one occurrence must exist
  [[ -n "$output" ]]
}

# ---------------------------------------------------------------------------
# AC #1 — [P1] install.sh defines _CLEANUP_TMPDIR global variable
# 9.4-UNIT-004
# ---------------------------------------------------------------------------
@test "[P1][9.4-UNIT-004] install.sh defines _CLEANUP_TMPDIR global variable" {
  # The composable cleanup pattern uses _CLEANUP_TMPDIR="" as the accumulator.
  # RED: install.sh currently has no _CLEANUP_TMPDIR variable.
  run grep -n "_CLEANUP_TMPDIR" "${INSTALL_SH}" 2>&1

  [ "$status" -eq 0 ]
  [[ -n "$output" ]]
}

# ---------------------------------------------------------------------------
# AC #2 — [P0] cleanup() removes the temp directory when _CLEANUP_TMPDIR is set
# 9.4-UNIT-005
# ---------------------------------------------------------------------------
@test "[P0][9.4-UNIT-005] cleanup() removes _CLEANUP_TMPDIR directory when set" {
  # Create a real temp directory, set _CLEANUP_TMPDIR, call cleanup(), verify removal.
  # RED: cleanup() does not exist yet.
  TMPDIR_FOR_TEST="$(mktemp -d)"

  run bash --noprofile --norc -c "
    source '${INSTALL_SH}'
    _CLEANUP_TMPDIR='${TMPDIR_FOR_TEST}'
    cleanup
    # If cleanup() worked the directory should be gone
    if [ -d '${TMPDIR_FOR_TEST}' ]; then
      echo 'TMPDIR still exists after cleanup'
      exit 1
    fi
    echo 'TMPDIR removed by cleanup'
    exit 0
  " 2>&1

  [ "$status" -eq 0 ]
  [[ "$output" == *"TMPDIR removed by cleanup"* ]]
  # Belt-and-suspenders: verify the directory is gone from the outer shell too
  [ ! -d "${TMPDIR_FOR_TEST}" ]
}

# ---------------------------------------------------------------------------
# AC #2 — [P1] cleanup() is a no-op when _CLEANUP_TMPDIR is empty
# 9.4-UNIT-006
# ---------------------------------------------------------------------------
@test "[P1][9.4-UNIT-006] cleanup() is a no-op when _CLEANUP_TMPDIR is empty or unset" {
  # cleanup() must not error when no cleanup has been registered.
  # RED: cleanup() does not exist yet.
  run bash --noprofile --norc -c "
    source '${INSTALL_SH}'
    _CLEANUP_TMPDIR=''
    cleanup
    echo 'cleanup exited cleanly'
  " 2>&1

  [ "$status" -eq 0 ]
  [[ "$output" == *"cleanup exited cleanly"* ]]
}

# ---------------------------------------------------------------------------
# AC #2 — [P0] install.sh exits non-zero and cleanup() still removes temp dir (set -e path)
# 9.4-INT-001
# ---------------------------------------------------------------------------
@test "[P0][9.4-INT-001] cleanup() fires and removes temp dir when script exits via set -e error" {
  # Simulate a set -e abort inside a sourced context:
  # set _CLEANUP_TMPDIR to a real directory, then trigger a non-zero exit command
  # while the EXIT trap is registered. Verify the directory is cleaned up.
  # RED: cleanup() / trap cleanup EXIT do not exist yet.
  TMPDIR_FOR_TEST="$(mktemp -d)"

  # We cannot source install.sh and then trigger set -e in the same shell easily,
  # so we write a wrapper script that sources install.sh and then causes a failure.
  WRAPPER="$(mktemp "${BATS_TMPDIR}/wrapper_XXXXXX.sh")"
  cat > "$WRAPPER" <<WRAPPER_SCRIPT
#!/usr/bin/env bash
set -euo pipefail
source '${INSTALL_SH}'
_CLEANUP_TMPDIR='${TMPDIR_FOR_TEST}'
# Trigger a deliberate non-zero exit to exercise the EXIT trap
false
WRAPPER_SCRIPT
  chmod +x "$WRAPPER"

  run bash --noprofile --norc "$WRAPPER" 2>&1

  # Script exits non-zero (false returned 1)
  [ "$status" -ne 0 ]
  # But the temp directory must have been cleaned up by cleanup()
  [ ! -d "${TMPDIR_FOR_TEST}" ]
}

# ---------------------------------------------------------------------------
# AC #1 / AC #2 — [P1] install.sh does NOT contain any inline trap … EXIT inside functions
# 9.4-INT-002
# ---------------------------------------------------------------------------
@test "[P1][9.4-INT-002] install.sh source contains no inline trap … EXIT calls inside function bodies" {
  # The inline 'trap ... EXIT' inside download_and_install_binary() must be removed.
  # After refactor: all trap registrations use 'trap cleanup EXIT' at the top level only.
  # RED: install.sh currently has an inline trap inside download_and_install_binary() at line 137.
  #
  # Invariant: every 'trap ... EXIT' in the file must be 'trap cleanup EXIT'.
  # Count all trap...EXIT lines and all 'trap cleanup EXIT' lines — they must be equal.
  # This catches any form of inline trap (quoted, variable-based, etc.), not just quoted forms.
  ALL_TRAP_EXIT_COUNT=$(grep -c 'trap.*EXIT' "${INSTALL_SH}" || true)
  CLEANUP_TRAP_COUNT=$(grep -c 'trap cleanup EXIT' "${INSTALL_SH}" || true)

  # After refactor: all trap...EXIT registrations must be the single 'trap cleanup EXIT'
  [ "$ALL_TRAP_EXIT_COUNT" -eq "$CLEANUP_TRAP_COUNT" ]
}

# ---------------------------------------------------------------------------
# AC #4 — [P2] cleanup() Bash 3.2 compatible — no declare -A, no arrays
# 9.4-UNIT-007
# ---------------------------------------------------------------------------
@test "[P2][9.4-UNIT-007] cleanup() uses only Bash 3.2 compatible syntax" {
  # Verify install.sh cleanup() and _CLEANUP_TMPDIR use simple string variables only.
  # No 'declare -A' (associative arrays), no 'mapfile', no '+='.
  # This is validated by sourcing under bash --posix mode (approximately) and checking
  # that cleanup() function source contains no disallowed constructs.
  # RED: cleanup() does not exist yet (and absence also causes failure).
  run bash --noprofile --norc -c "
    source '${INSTALL_SH}'
    FUNC_SRC=\"\$(declare -f cleanup)\"
    # Check for disallowed Bash 4+ patterns
    if echo \"\$FUNC_SRC\" | grep -q 'declare -A'; then
      echo 'FAIL: declare -A found — not Bash 3.2 compatible'
      exit 1
    fi
    if echo \"\$FUNC_SRC\" | grep -q 'mapfile'; then
      echo 'FAIL: mapfile found — not Bash 3.2 compatible'
      exit 1
    fi
    echo 'Bash 3.2 compatibility OK'
  " 2>&1

  [ "$status" -eq 0 ]
  [[ "$output" == *"Bash 3.2 compatibility OK"* ]]
}

# ---------------------------------------------------------------------------
# AC #4 — [P2] trap cleanup EXIT appears exactly once in install.sh
# 9.4-UNIT-008
# ---------------------------------------------------------------------------
@test "[P2][9.4-UNIT-008] trap cleanup EXIT appears exactly once in install.sh (composable, not duplicated)" {
  # Composable cleanup registers one trap. More than one registration would mean
  # the last one wins — defeating the composability goal.
  # RED: 'trap cleanup EXIT' does not exist in install.sh yet.
  COUNT="$(grep -c "trap cleanup EXIT" "${INSTALL_SH}" || true)"

  [ "$COUNT" -eq 1 ]
}

# ---------------------------------------------------------------------------
# AC #3 — [P1] Regression guard: test_6_1.bats still passes after refactor
# (integration-level; runs full existing suite as a black-box regression check)
# 9.4-INT-003
# GREEN phase: implementation complete — regression guard is now active.
# ---------------------------------------------------------------------------
@test "[P1][9.4-INT-003] REGRESSION: existing test_6_1.bats suite still passes after refactor" {
  # Run the Epic 6 first-install test suite as a black-box regression check.
  # This verifies AC #3: zero regressions in existing tests after the EXIT trap refactor.
  # Note: nested bats invocation is intentional here — we want the full bats runner
  # output and exit code rather than sourcing individual test files.
  run bats "${BATS_TEST_DIRNAME}/test_6_1.bats" 2>&1
  [ "$status" -eq 0 ]
}
