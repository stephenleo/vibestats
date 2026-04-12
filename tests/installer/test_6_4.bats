#!/usr/bin/env bats
# Story 6.4: Implement hook configuration, README markers, and backfill trigger
# ATDD Red Phase — tests assert expected behaviour; will fail until install.sh is implemented.
#
# Run: bats tests/installer/test_6_4.bats
#
# Test framework: bats-core
# Mocking strategy: override _gh() helper as a shell function to mock gh CLI calls.
# All tests use a temp $HOME to prevent mutation of the developer's real config.
#
# AC Coverage:
#   AC #1 (FR8, R-008): configure_hooks writes Stop + SessionStart hooks to ~/.claude/settings.json
#   AC #2 (FR9, R-009): inject_readme_markers adds vibestats-start/end markers with SVG embed
#   AC #3 (FR11):       run_backfill calls vibestats sync --backfill as the final step

INSTALL_SH="$(cd "$(dirname "$BATS_TEST_FILENAME")/../.." && pwd)/install.sh"

setup() {
  # Isolate from real $HOME — required for every test
  export HOME
  HOME="$(mktemp -d)"
  export BATS_TMPDIR="${HOME}/bats-tmp"
  mkdir -p "$BATS_TMPDIR"

  # Spy log for recording _gh calls
  GH_SPY_LOG="${BATS_TMPDIR}/gh_calls.log"
  export GH_SPY_LOG
}

teardown() {
  rm -rf "$HOME"
}

# ---------------------------------------------------------------------------
# P1 — AC #1 (FR8): Stop hook written to ~/.claude/settings.json
# Assert: hooks.Stop[0].hooks[0].command == "vibestats sync" and async == true
# ---------------------------------------------------------------------------
@test "[P1] configure_hooks: Stop hook with command=vibestats sync and async=true written to settings.json" {

  run bash --noprofile --norc -c "
    source '${INSTALL_SH}'
    configure_hooks
  " 2>&1

  [ "$status" -eq 0 ]

  SETTINGS_FILE="${HOME}/.claude/settings.json"
  [ -f "${SETTINGS_FILE}" ]

  run python3 -c "
import json, sys
with open('${SETTINGS_FILE}') as f:
    s = json.load(f)
stop_matchers = s.get('hooks', {}).get('Stop', [])
assert len(stop_matchers) >= 1, 'No Stop matchers in settings.json'
hooks = stop_matchers[0].get('hooks', [])
assert len(hooks) >= 1, 'No hooks in Stop matcher'
h = hooks[0]
assert h.get('command') == 'vibestats sync', f\"Stop command must be 'vibestats sync', got: {h.get('command')}\"
assert h.get('async') == True, f'Stop hook async must be true, got: {h.get(\"async\")}'
print('Stop hook valid')
"
  [ "$status" -eq 0 ]
}

# ---------------------------------------------------------------------------
# P1 — AC #1 (FR8): SessionStart hook written to ~/.claude/settings.json
# Assert: hooks.SessionStart[0].hooks[0].command == "vibestats sync"
# ---------------------------------------------------------------------------
@test "[P1] configure_hooks: SessionStart hook with command=vibestats sync written to settings.json" {

  run bash --noprofile --norc -c "
    source '${INSTALL_SH}'
    configure_hooks
  " 2>&1

  [ "$status" -eq 0 ]

  SETTINGS_FILE="${HOME}/.claude/settings.json"
  [ -f "${SETTINGS_FILE}" ]

  run python3 -c "
import json
with open('${SETTINGS_FILE}') as f:
    s = json.load(f)
session_matchers = s.get('hooks', {}).get('SessionStart', [])
assert len(session_matchers) >= 1, 'No SessionStart matchers in settings.json'
hooks = session_matchers[0].get('hooks', [])
assert len(hooks) >= 1, 'No hooks in SessionStart matcher'
h = hooks[0]
assert h.get('command') == 'vibestats sync', f\"SessionStart command must be 'vibestats sync', got: {h.get('command')}\"
assert 'async' not in h or h.get('async') is not True, 'SessionStart hook must NOT have async=true'
print('SessionStart hook valid')
"
  [ "$status" -eq 0 ]
}

# ---------------------------------------------------------------------------
# P1 — AC #1 (R-008): idempotency — running configure_hooks twice produces
# exactly one Stop matcher and one SessionStart matcher
# ---------------------------------------------------------------------------
@test "[P1] configure_hooks: idempotent — running twice produces exactly one Stop and one SessionStart entry" {

  run bash --noprofile --norc -c "
    source '${INSTALL_SH}'
    configure_hooks
    configure_hooks
  " 2>&1

  [ "$status" -eq 0 ]

  SETTINGS_FILE="${HOME}/.claude/settings.json"
  [ -f "${SETTINGS_FILE}" ]

  run python3 -c "
import json
with open('${SETTINGS_FILE}') as f:
    s = json.load(f)
stop_count = len(s.get('hooks', {}).get('Stop', []))
session_count = len(s.get('hooks', {}).get('SessionStart', []))
assert stop_count == 1, f'Expected exactly 1 Stop matcher after 2 runs, got {stop_count}'
assert session_count == 1, f'Expected exactly 1 SessionStart matcher after 2 runs, got {session_count}'
print(f'Idempotency verified: Stop={stop_count}, SessionStart={session_count}')
"
  [ "$status" -eq 0 ]
}

# ---------------------------------------------------------------------------
# P1 — AC #1 (R-008): configure_hooks does not clobber unrelated existing hooks
# Pre-seed settings.json with a pre-existing hook key; assert still present after
# ---------------------------------------------------------------------------
@test "[P1] configure_hooks: does not clobber existing unrelated hooks in settings.json" {

  # Pre-seed settings.json with an unrelated hook
  mkdir -p "${HOME}/.claude"
  cat > "${HOME}/.claude/settings.json" <<'JSON'
{
  "hooks": {
    "PreToolUse": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "echo pre-existing-hook"
          }
        ]
      }
    ]
  }
}
JSON

  run bash --noprofile --norc -c "
    source '${INSTALL_SH}'
    configure_hooks
  " 2>&1

  [ "$status" -eq 0 ]

  run python3 -c "
import json
with open('${HOME}/.claude/settings.json') as f:
    s = json.load(f)
# Pre-existing hook must still be present
pre_hooks = s.get('hooks', {}).get('PreToolUse', [])
assert len(pre_hooks) >= 1, 'Pre-existing PreToolUse hook was removed!'
h = pre_hooks[0].get('hooks', [])[0]
assert h.get('command') == 'echo pre-existing-hook', 'Pre-existing hook command was altered!'
# vibestats hooks must also be present
assert 'Stop' in s.get('hooks', {}), 'Stop hook missing after configure_hooks'
assert 'SessionStart' in s.get('hooks', {}), 'SessionStart hook missing after configure_hooks'
print('Pre-existing hooks preserved and vibestats hooks added')
"
  [ "$status" -eq 0 ]
}

# ---------------------------------------------------------------------------
# P2 — AC #2 (FR9, R-009): inject_readme_markers adds markers + SVG img + link
# Mock _gh api returning sample README content and SHA; assert PUT body contains
# <!-- vibestats-start -->, <!-- vibestats-end -->, SVG img URL, and dashboard link
# ---------------------------------------------------------------------------
@test "[P2] inject_readme_markers: markers + SVG img + dashboard link written to profile README" {

  # Build a base64-encoded README (platform-aware)
  SAMPLE_README="# Hello World

This is my GitHub profile README."

  case "$(uname -s)" in
    Darwin) ENCODED_README=$(printf '%s' "$SAMPLE_README" | base64) ;;
    Linux)  ENCODED_README=$(printf '%s' "$SAMPLE_README" | base64 -w 0) ;;
  esac
  export ENCODED_README

  # Capture the PUT body
  PUT_CAPTURE_FILE="${BATS_TMPDIR}/readme_put_capture.txt"
  export PUT_CAPTURE_FILE

  cat > "${HOME}/stub_env.sh" <<STUB
_gh() {
  echo "_gh \$*" >> "${GH_SPY_LOG}"
  case "\$1 \$2" in
    "api /user")
      echo '{"login": "testuser"}'
      ;;
    "api repos"*)
      case "\$*" in
        *"README.md"*"--method PUT"*)
          # Capture content field from PUT call for assertion
          args=("\$@")
          for i in "\${!args[@]}"; do
            if [ "\${args[\$i]}" = "--field" ] && echo "\${args[\$((i+1))]}" | grep -q "^content="; then
              CONTENT_VAL=\$(echo "\${args[\$((i+1))]}" | sed 's/^content=//')
              case "\$(uname -s)" in
                Darwin) echo "\$CONTENT_VAL" | base64 -D > "${PUT_CAPTURE_FILE}" ;;
                Linux)  echo "\$CONTENT_VAL" | base64 -d > "${PUT_CAPTURE_FILE}" ;;
              esac
            fi
          done
          echo '{"commit": {"sha": "newsha123"}}'
          return 0
          ;;
        *"README.md"*)
          # GET — return README with content and sha
          echo "{\"content\": \"${ENCODED_README}\", \"sha\": \"existingsha456\"}"
          return 0
          ;;
      esac
      return 0
      ;;
    *)
      return 0
      ;;
  esac
}
export -f _gh
STUB

  run bash --noprofile --norc -c "
    source '${HOME}/stub_env.sh'
    source '${INSTALL_SH}'
    inject_readme_markers
  " 2>&1

  [ "$status" -eq 0 ]

  # Assert the PUT was called (captured file exists)
  [ -f "${PUT_CAPTURE_FILE}" ]

  # Assert markers present in the updated README content
  run grep -q '<!-- vibestats-start -->' "${PUT_CAPTURE_FILE}"
  [ "$status" -eq 0 ]

  run grep -q '<!-- vibestats-end -->' "${PUT_CAPTURE_FILE}"
  [ "$status" -eq 0 ]

  # Assert SVG image URL contains the username
  run grep -q 'raw.githubusercontent.com/testuser/testuser/main/vibestats/heatmap.svg' "${PUT_CAPTURE_FILE}"
  [ "$status" -eq 0 ]

  # Assert dashboard link contains the username
  run grep -q 'vibestats.dev/testuser' "${PUT_CAPTURE_FILE}"
  [ "$status" -eq 0 ]
}

# ---------------------------------------------------------------------------
# P2 — AC #2 (R-009): inject_readme_markers prints warning (not error) and
# continues with exit 0 when profile repo returns 404 / non-zero exit
# ---------------------------------------------------------------------------
@test "[P2] inject_readme_markers: warning (not error) and continues when profile repo returns 404" {

  cat > "${HOME}/stub_env.sh" <<STUB
_gh() {
  echo "_gh \$*" >> "${GH_SPY_LOG}"
  case "\$1 \$2" in
    "api /user")
      echo '{"login": "testuser"}'
      ;;
    "api repos"*)
      # Simulate 404 — return non-zero for README GET
      return 1
      ;;
    *)
      return 0
      ;;
  esac
}
export -f _gh
STUB

  run bash --noprofile --norc -c "
    source '${HOME}/stub_env.sh'
    source '${INSTALL_SH}'
    inject_readme_markers
  " 2>&1

  # Must exit 0 — warning is NOT a fatal error
  [ "$status" -eq 0 ]

  # Must print a warning message
  [[ "$output" == *"Warning:"* ]]
}

# ---------------------------------------------------------------------------
# P2 — AC #2 (R-009): inject_readme_markers is idempotent — no second PUT
# when markers already present in the README
# ---------------------------------------------------------------------------
@test "[P2] inject_readme_markers: idempotent — no second PUT when markers already present" {

  # Build a README that already has vibestats markers (base64 encoded)
  SAMPLE_README="# Hello World

<!-- vibestats-start -->
[![vibestats](https://raw.githubusercontent.com/testuser/testuser/main/vibestats/heatmap.svg)](https://vibestats.dev/testuser)

[View interactive dashboard →](https://vibestats.dev/testuser)
<!-- vibestats-end -->"

  case "$(uname -s)" in
    Darwin) ENCODED_README=$(printf '%s' "$SAMPLE_README" | base64) ;;
    Linux)  ENCODED_README=$(printf '%s' "$SAMPLE_README" | base64 -w 0) ;;
  esac
  export ENCODED_README

  cat > "${HOME}/stub_env.sh" <<STUB
_gh() {
  echo "_gh \$*" >> "${GH_SPY_LOG}"
  case "\$1 \$2" in
    "api /user")
      echo '{"login": "testuser"}'
      ;;
    "api repos"*)
      case "\$*" in
        *"README.md"*"--method PUT"*)
          # PUT should NOT be called — record it so we can assert it was not called
          echo "UNEXPECTED_PUT" >> "${GH_SPY_LOG}"
          return 0
          ;;
        *"README.md"*)
          # GET — return README that already has markers
          echo "{\"content\": \"${ENCODED_README}\", \"sha\": \"existingsha789\"}"
          return 0
          ;;
      esac
      return 0
      ;;
    *)
      return 0
      ;;
  esac
}
export -f _gh
STUB

  run bash --noprofile --norc -c "
    source '${HOME}/stub_env.sh'
    source '${INSTALL_SH}'
    inject_readme_markers
  " 2>&1

  [ "$status" -eq 0 ]
  # Save output from inject_readme_markers before running the spy log check
  INJECT_OUTPUT="$output"

  # Assert PUT was NOT called (idempotency)
  run grep "UNEXPECTED_PUT" "${GH_SPY_LOG}"
  [ "$status" -ne 0 ]

  # Output should mention markers already present
  [[ "$INJECT_OUTPUT" == *"already present"* ]]
}

# ---------------------------------------------------------------------------
# P2 — AC #3 (FR11): run_backfill calls vibestats sync --backfill
# Spy on the binary call; assert sync --backfill appears in spy log
# ---------------------------------------------------------------------------
@test "[P2] run_backfill: vibestats sync --backfill is called as final step" {

  BINARY_SPY_LOG="${BATS_TMPDIR}/binary_calls.log"
  export BINARY_SPY_LOG

  # Create a mock vibestats binary at the expected path
  mkdir -p "${HOME}/.local/bin"
  cat > "${HOME}/.local/bin/vibestats" <<STUB
#!/usr/bin/env bash
echo "vibestats \$*" >> "${BINARY_SPY_LOG}"
exit 0
STUB
  chmod +x "${HOME}/.local/bin/vibestats"

  run bash --noprofile --norc -c "
    source '${INSTALL_SH}'
    run_backfill
  " 2>&1

  [ "$status" -eq 0 ]

  # Assert the binary was called with sync --backfill
  [ -f "${BINARY_SPY_LOG}" ]
  run grep "sync --backfill" "${BINARY_SPY_LOG}"
  [ "$status" -eq 0 ]
}

# ---------------------------------------------------------------------------
# P2 — AC #3 (FR11): run_backfill non-zero exit from binary prints warning
# but installer function exits 0 (backfill failure is non-fatal)
# ---------------------------------------------------------------------------
@test "[P2] run_backfill: non-zero exit from binary prints warning but installer exits 0" {

  BINARY_SPY_LOG="${BATS_TMPDIR}/binary_calls.log"
  export BINARY_SPY_LOG

  # Create a mock vibestats binary that exits non-zero
  mkdir -p "${HOME}/.local/bin"
  cat > "${HOME}/.local/bin/vibestats" <<STUB
#!/usr/bin/env bash
echo "vibestats \$*" >> "${BINARY_SPY_LOG}"
exit 1
STUB
  chmod +x "${HOME}/.local/bin/vibestats"

  run bash --noprofile --norc -c "
    source '${INSTALL_SH}'
    run_backfill
  " 2>&1

  # run_backfill must exit 0 even when binary fails
  [ "$status" -eq 0 ]

  # Must print a warning
  [[ "$output" == *"Warning:"* ]]
}
