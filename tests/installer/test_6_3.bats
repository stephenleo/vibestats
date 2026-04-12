#!/usr/bin/env bats
# Story 6.3: Implement multi-machine install path
# ATDD Red Phase — tests assert expected behaviour; will fail until install.sh is implemented.
#
# Run: bats tests/installer/test_6_3.bats
#
# Test framework: bats-core
# Mocking strategy: override _gh() helper as a shell function to mock gh CLI calls.
# All tests use a temp $HOME to prevent mutation of the developer's real config.
#
# TDD RED PHASE: All tests are prefixed with skip until detect_install_mode() and
# register_machine() functions exist in install.sh.
#
# AC Coverage:
#   AC #1 (FR5): detect existing vibestats-data repo → skip repo creation, workflow write, VIBESTATS_TOKEN
#   AC #2 (FR6): register new machine → registry.json entry with all required fields

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
# P0 — AC #1 (FR5, R-004): multi-machine path skips first-install steps
# When vibestats-data repo already exists, installer must NOT call:
#   - gh repo create
#   - gh api (workflow write)
#   - gh secret set VIBESTATS_TOKEN
# ---------------------------------------------------------------------------
@test "[P0] multi-machine path: vibestats-data exists → repo creation skipped, workflow write skipped, VIBESTATS_TOKEN not set" {
  skip "RED: detect_install_mode() and register_machine() not yet implemented in install.sh"

  cat > "${HOME}/stub_env.sh" <<STUB
_gh() {
  echo "_gh \$*" >> "${GH_SPY_LOG}"
  case "\$1 \$2" in
    "api /user")
      echo '{"login": "testuser"}'
      ;;
    "repo view")
      # vibestats-data exists — simulate success
      echo '{"name": "vibestats-data"}'
      return 0
      ;;
    "repo create")
      echo "UNEXPECTED: gh repo create called" >> "${GH_SPY_LOG}"
      return 0
      ;;
    "secret set")
      echo "UNEXPECTED: gh secret set called" >> "${GH_SPY_LOG}"
      return 0
      ;;
    "api repos"*)
      # Simulate registry.json not found (first machine registration)
      case "\$*" in
        *"registry.json"*"--method PUT"*)
          echo '{"content": {"sha": "abc123"}}'
          return 0
          ;;
        *"registry.json"*)
          # GET returns 404 — no existing registry
          return 1
          ;;
      esac
      return 0
      ;;
    "auth token")
      echo "ghp_TESTMACHINETOKEN"
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
    detect_install_mode
    case \"\$INSTALL_MODE\" in
      multi-machine) register_machine ;;
      first-install) echo 'First-install path (stub)' ;;
    esac
  " 2>&1

  [ "$status" -eq 0 ]

  # Assert repo creation was NOT called
  run grep "repo create" "${GH_SPY_LOG}"
  [ "$status" -ne 0 ]

  # Assert secret set VIBESTATS_TOKEN was NOT called
  run grep "secret set VIBESTATS_TOKEN" "${GH_SPY_LOG}"
  [ "$status" -ne 0 ]

  # Assert workflow write was NOT called (no api with aggregate.yml path)
  run grep "aggregate.yml" "${GH_SPY_LOG}"
  [ "$status" -ne 0 ]
}

# ---------------------------------------------------------------------------
# P0 — AC #2 (FR6, R-005): registry.json entry has all required fields
# machine_id, hostname, status=active, last_seen ISO 8601 UTC
# ---------------------------------------------------------------------------
@test "[P0] registry.json entry has all required fields: machine_id, hostname, status=active, last_seen ISO 8601 UTC" {
  skip "RED: register_machine() not yet implemented in install.sh"

  # Capture the JSON that would be PUT to the Contents API
  REGISTRY_PUT_BODY="${BATS_TMPDIR}/registry_put_body.json"
  export REGISTRY_PUT_BODY

  cat > "${HOME}/stub_env.sh" <<STUB
_gh() {
  echo "_gh \$*" >> "${GH_SPY_LOG}"
  case "\$1 \$2" in
    "api /user")
      echo '{"login": "testuser"}'
      ;;
    "repo view")
      echo '{"name": "vibestats-data"}'
      return 0
      ;;
    "auth token")
      echo "ghp_TESTTOKEN"
      ;;
    "api repos"*)
      # Intercept the PUT for registry.json and capture the --field content= value
      case "\$*" in
        *"registry.json"*"--method PUT"*)
          # Extract the content field from the arguments and save for assertion
          CONTENT_VAL=""
          args=("\$@")
          for i in "\${!args[@]}"; do
            if [ "\${args[\$i]}" = "--field" ] && echo "\${args[\$((i+1))]}" | grep -q "^content="; then
              CONTENT_VAL=\$(echo "\${args[\$((i+1))]}" | sed 's/^content=//')
            fi
          done
          # Decode base64 and save
          case "\$(uname -s)" in
            Darwin) echo "\$CONTENT_VAL" | base64 -D > "${REGISTRY_PUT_BODY}" ;;
            Linux)  echo "\$CONTENT_VAL" | base64 -d > "${REGISTRY_PUT_BODY}" ;;
          esac
          echo '{"content": {"sha": "newsha123"}}'
          return 0
          ;;
        *"registry.json"*)
          # GET — return empty registry (no existing machines)
          # Return 404 to trigger empty state handling
          return 1
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
    detect_install_mode
    register_machine
  " 2>&1

  [ "$status" -eq 0 ]

  # Verify registry PUT was captured
  [ -f "${REGISTRY_PUT_BODY}" ]

  # Assert all required fields present and correct
  # machine_id: non-empty string
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
}

# ---------------------------------------------------------------------------
# P1 — AC #1: vibestats-data detection uses correct repo name (user/vibestats-data)
# ---------------------------------------------------------------------------
@test "[P1] vibestats-data repo detection uses correct repo name (username/vibestats-data not hardcoded org)" {
  skip "RED: detect_install_mode() not yet implemented in install.sh"

  cat > "${HOME}/stub_env.sh" <<STUB
_gh() {
  echo "_gh \$*" >> "${GH_SPY_LOG}"
  case "\$1 \$2" in
    "api /user")
      echo '{"login": "testuser"}'
      ;;
    "repo view")
      echo '{"name": "vibestats-data"}'
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
    detect_install_mode
  " 2>&1

  [ "$status" -eq 0 ]

  # Assert 'gh repo view' was called with testuser/vibestats-data (not a hardcoded org)
  run grep "repo view" "${GH_SPY_LOG}"
  [ "$status" -eq 0 ]
  [[ "$(cat "${GH_SPY_LOG}")" == *"testuser/vibestats-data"* ]]
}

# ---------------------------------------------------------------------------
# P1 — AC #1: detect_install_mode sets INSTALL_MODE=multi-machine when repo exists
# ---------------------------------------------------------------------------
@test "[P1] detect_install_mode sets INSTALL_MODE=multi-machine when vibestats-data repo exists" {
  skip "RED: detect_install_mode() not yet implemented in install.sh"

  cat > "${HOME}/stub_env.sh" <<STUB
_gh() {
  case "\$1 \$2" in
    "api /user")
      echo '{"login": "testuser"}'
      ;;
    "repo view")
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
    detect_install_mode
    echo \"INSTALL_MODE=\${INSTALL_MODE}\"
  " 2>&1

  [ "$status" -eq 0 ]
  [[ "$output" == *"INSTALL_MODE=multi-machine"* ]]
}

# ---------------------------------------------------------------------------
# P1 — AC #1: detect_install_mode sets INSTALL_MODE=first-install when repo does not exist
# ---------------------------------------------------------------------------
@test "[P1] detect_install_mode sets INSTALL_MODE=first-install when vibestats-data repo does not exist" {
  skip "RED: detect_install_mode() not yet implemented in install.sh"

  cat > "${HOME}/stub_env.sh" <<STUB
_gh() {
  case "\$1 \$2" in
    "api /user")
      echo '{"login": "testuser"}'
      ;;
    "repo view")
      # Repo does not exist — return non-zero
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
    detect_install_mode
    echo \"INSTALL_MODE=\${INSTALL_MODE}\"
  " 2>&1

  [ "$status" -eq 0 ]
  [[ "$output" == *"INSTALL_MODE=first-install"* ]]
}

# ---------------------------------------------------------------------------
# P1 — AC #2 (R-005): register_machine appends new entry without overwriting existing machines
# ---------------------------------------------------------------------------
@test "[P1] register_machine appends new entry without overwriting existing machines" {
  skip "RED: register_machine() not yet implemented in install.sh"

  REGISTRY_PUT_BODY="${BATS_TMPDIR}/registry_put_body.json"
  export REGISTRY_PUT_BODY

  # Existing registry JSON with one pre-existing machine
  EXISTING_REGISTRY='{"machines":[{"machine_id":"old-machine","hostname":"old-host","status":"active","last_seen":"2026-01-01T00:00:00Z"}]}'
  EXISTING_ENCODED=""
  case "$(uname -s)" in
    Darwin) EXISTING_ENCODED=$(echo "$EXISTING_REGISTRY" | base64) ;;
    Linux)  EXISTING_ENCODED=$(echo "$EXISTING_REGISTRY" | base64 -w 0) ;;
  esac
  export EXISTING_ENCODED

  cat > "${HOME}/stub_env.sh" <<STUB
_gh() {
  echo "_gh \$*" >> "${GH_SPY_LOG}"
  case "\$1 \$2" in
    "api /user")
      echo '{"login": "testuser"}'
      ;;
    "repo view")
      echo '{"name": "vibestats-data"}'
      return 0
      ;;
    "auth token")
      echo "ghp_TESTTOKEN"
      ;;
    "api repos"*)
      case "\$*" in
        *"registry.json"*"--method PUT"*)
          # Capture the content field
          args=("\$@")
          for i in "\${!args[@]}"; do
            if [ "\${args[\$i]}" = "--field" ] && echo "\${args[\$((i+1))]}" | grep -q "^content="; then
              CONTENT_VAL=\$(echo "\${args[\$((i+1))]}" | sed 's/^content=//')
            fi
          done
          case "\$(uname -s)" in
            Darwin) echo "\$CONTENT_VAL" | base64 -D > "${REGISTRY_PUT_BODY}" ;;
            Linux)  echo "\$CONTENT_VAL" | base64 -d > "${REGISTRY_PUT_BODY}" ;;
          esac
          echo '{"content": {"sha": "newsha999"}}'
          return 0
          ;;
        *"registry.json"*"--jq .sha"*)
          echo "existingsha123"
          ;;
        *"registry.json"*"--jq .content"*)
          # Return base64-encoded existing registry
          echo "${EXISTING_ENCODED}"
          ;;
        *"registry.json"*)
          # GET without jq — return JSON with content and sha
          echo "{\"content\": \"${EXISTING_ENCODED}\", \"sha\": \"existingsha123\"}"
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
    detect_install_mode
    register_machine
  " 2>&1

  [ "$status" -eq 0 ]
  [ -f "${REGISTRY_PUT_BODY}" ]

  # Assert both old and new machines are present in the PUT body
  run python3 -c "
import json
with open('${REGISTRY_PUT_BODY}') as f:
    data = json.load(f)
machines = data.get('machines', [])
assert len(machines) >= 2, f'Expected at least 2 machines (old + new), got {len(machines)}'
ids = [m['machine_id'] for m in machines]
assert 'old-machine' in ids, f'Existing machine old-machine was removed! machines: {ids}'
print(f'OK: {len(machines)} machines found, old-machine preserved')
"
  [ "$status" -eq 0 ]
}

# ---------------------------------------------------------------------------
# P1 — NFR6: config.toml written with 600 permissions
# ---------------------------------------------------------------------------
@test "[P1] register_machine writes config.toml with 600 permissions" {
  skip "RED: register_machine() not yet implemented in install.sh"

  REGISTRY_PUT_BODY="${BATS_TMPDIR}/registry_put_body.json"
  export REGISTRY_PUT_BODY

  cat > "${HOME}/stub_env.sh" <<STUB
_gh() {
  case "\$1 \$2" in
    "api /user")
      echo '{"login": "testuser"}'
      ;;
    "repo view")
      echo '{"name": "vibestats-data"}'
      return 0
      ;;
    "auth token")
      echo "ghp_TESTMACHINETOKEN"
      ;;
    "api repos"*)
      case "\$*" in
        *"registry.json"*"--method PUT"*)
          echo '{"content": {"sha": "abc123"}}'
          return 0
          ;;
        *)
          return 1
          ;;
      esac
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
    detect_install_mode
    register_machine
  " 2>&1

  [ "$status" -eq 0 ]

  CONFIG_FILE="${HOME}/.config/vibestats/config.toml"
  [ -f "${CONFIG_FILE}" ]

  # Check permissions are 600
  case "$(uname -s)" in
    Darwin)
      PERMS="$(stat -f "%Lp" "${CONFIG_FILE}")"
      ;;
    Linux)
      PERMS="$(stat -c "%a" "${CONFIG_FILE}")"
      ;;
  esac

  [ "$PERMS" = "600" ]
}
