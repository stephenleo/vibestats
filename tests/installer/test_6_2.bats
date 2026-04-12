#!/usr/bin/env bats
# Story 6.2: Implement first-install path
# ATDD Red Phase — tests assert expected behaviour; will fail until install.sh functions are implemented.
#
# Run: bats tests/installer/test_6_2.bats
#
# Test framework: bats-core
# Mocking strategy: override _gh() helper and shell commands via functions exported to subshells.
# All tests use a temp $HOME to prevent mutation of the developer's real config.
#
# ACs tested:
#   AC #1: vibestats-data does not exist → gh repo create --private called (FR4)
#   AC #2: repo created → aggregate.yml written into vibestats-data/.github/workflows/ (FR7)
#   AC #3: VIBESTATS_TOKEN generated via gh api, set as Actions secret, never written to disk (FR10, NFR7)
#   AC #4: local token stored in ~/.config/vibestats/config.toml with permissions 600 (FR39, NFR6)
#   AC #5: registry.json contains machine entry with required fields (FR6)

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
# AC #1 — vibestats-data does not exist → gh repo create --private called
# P1 — Story 6.2, FR4
# ---------------------------------------------------------------------------
@test "[P1] vibestats-data does not exist → gh repo create --private called" {
  # _gh repo view returns non-zero (repo not found), then _gh repo create should be called
  cat > "${HOME}/stub_env.sh" <<STUB
_gh() {
  case "\$1 \$2" in
    "auth token")
      echo "ghp_FAKE_MACHINE_TOKEN"
      ;;
    "api /user")
      echo '{"login":"testuser"}'
      ;;
    "repo view")
      # vibestats-data does not exist
      return 1
      ;;
    "repo create")
      echo "repo create called: \$*" >> "${HOME}/gh_calls.log"
      return 0
      ;;
    "api /user/personal_access_tokens"*)
      echo '{"token":"ghp_FAKE_VIBESTATS_TOKEN"}'
      ;;
    "secret set")
      echo "secret set called: \$*" >> "${HOME}/gh_calls.log"
      return 0
      ;;
    *)
      echo "gh \$* called" >> "${HOME}/gh_calls.log"
      return 0
      ;;
  esac
}
export -f _gh
STUB

  run bash --noprofile --norc -c "
    source '${HOME}/stub_env.sh'
    source '${INSTALL_SH}'
    create_vibestats_data_repo
  " 2>&1

  [ "$status" -eq 0 ]
  [ -f "${HOME}/gh_calls.log" ]
  grep -q "repo create" "${HOME}/gh_calls.log"
}

# ---------------------------------------------------------------------------
# AC #1 — gh repo create called with --private flag (not public)
# P1 — Story 6.2, FR4
# ---------------------------------------------------------------------------
@test "[P1] gh repo create called with --private flag" {
  cat > "${HOME}/stub_env.sh" <<STUB
_gh() {
  case "\$1 \$2" in
    "auth token")
      echo "ghp_FAKE_MACHINE_TOKEN"
      ;;
    "api /user")
      echo '{"login":"testuser"}'
      ;;
    "repo view")
      return 1
      ;;
    "repo create")
      echo "gh repo create args: \$*" >> "${HOME}/gh_calls.log"
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
    create_vibestats_data_repo
  " 2>&1

  [ "$status" -eq 0 ]
  [ -f "${HOME}/gh_calls.log" ]
  grep -q "\-\-private" "${HOME}/gh_calls.log"
}

# ---------------------------------------------------------------------------
# AC #2 — aggregate.yml written into vibestats-data/.github/workflows/ (FR7)
# P1 — Story 6.2, FR7
# ---------------------------------------------------------------------------
@test "[P1] aggregate.yml written calling stephenleo/vibestats@v1" {
  # Stub _gh api to accept Content PUT calls for the workflow file.
  # The function should produce a workflow YAML referencing stephenleo/vibestats@v1.
  cat > "${HOME}/stub_env.sh" <<STUB
_gh() {
  case "\$1 \$2" in
    "auth token")
      echo "ghp_FAKE_MACHINE_TOKEN"
      ;;
    "api /user")
      echo '{"login":"testuser"}'
      ;;
    "repo view")
      return 1
      ;;
    "repo create")
      return 0
      ;;
    "api repos"*)
      # Capture the JSON body written via --input - (stdin) for workflow file creation
      echo "gh api repos called with: \$*" >> "${HOME}/gh_calls.log"
      # Read stdin if piped
      cat >> "${HOME}/gh_workflow_stdin.log" 2>/dev/null || true
      return 0
      ;;
    "api /user/personal_access_tokens"*)
      echo '{"token":"ghp_FAKE_VIBESTATS_TOKEN"}'
      ;;
    "secret set")
      return 0
      ;;
    *)
      echo "gh \$* called" >> "${HOME}/gh_calls.log"
      return 0
      ;;
  esac
}
export -f _gh
STUB

  run bash --noprofile --norc -c "
    source '${HOME}/stub_env.sh'
    source '${INSTALL_SH}'
    write_aggregate_workflow
  " 2>&1

  [ "$status" -eq 0 ]
  # The workflow content passed to gh api must reference stephenleo/vibestats@v1
  [[ "$output" == *"stephenleo/vibestats@v1"* ]] || grep -rq "stephenleo/vibestats@v1" "${HOME}/" 2>/dev/null
}

# ---------------------------------------------------------------------------
# AC #2 — workflow includes schedule cron and workflow_dispatch triggers (FR25, FR26)
# P1 — Story 6.2, FR7, FR25, FR26
# ---------------------------------------------------------------------------
@test "[P1] aggregate.yml workflow content includes cron and workflow_dispatch triggers" {
  cat > "${HOME}/stub_env.sh" <<STUB
_gh() {
  case "\$1 \$2" in
    "auth token")
      echo "ghp_FAKE_MACHINE_TOKEN"
      ;;
    "api /user")
      echo '{"login":"testuser"}'
      ;;
    "repo view")
      return 1
      ;;
    "repo create")
      return 0
      ;;
    "api repos"*)
      return 0
      ;;
    "api /user/personal_access_tokens"*)
      echo '{"token":"ghp_FAKE_VIBESTATS_TOKEN"}'
      ;;
    "secret set")
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
    # Export the workflow content so we can inspect it
    generate_aggregate_workflow_content
  " 2>&1

  [ "$status" -eq 0 ]
  [[ "$output" == *"schedule"* ]] || [[ "$output" == *"cron"* ]]
  [[ "$output" == *"workflow_dispatch"* ]]
}

# ---------------------------------------------------------------------------
# AC #3 — VIBESTATS_TOKEN generated via gh api (not written to disk) (FR10, NFR7)
# P0 — Story 6.2, R-001, NFR7
# ---------------------------------------------------------------------------
@test "[P0] VIBESTATS_TOKEN is never written to disk or echoed to stdout" {
  # Use a sentinel token value to detect if it is written anywhere
  SENTINEL_TOKEN="ghp_SENTINEL_VIBESTATS_TOKEN_DETECT_ME"

  cat > "${HOME}/stub_env.sh" <<STUB
_gh() {
  case "\$1 \$2" in
    "auth token")
      echo "ghp_FAKE_MACHINE_TOKEN"
      ;;
    "api /user")
      echo '{"login":"testuser"}'
      ;;
    "repo view")
      return 1
      ;;
    "repo create")
      return 0
      ;;
    "api repos"*)
      return 0
      ;;
    "api /user/personal_access_tokens"*)
      # Return a sentinel PAT that we will scan for in files afterward
      echo '{"token":"${SENTINEL_TOKEN}"}'
      ;;
    "secret set")
      # Record that secret set was called but do NOT write the token value to a file
      echo "gh secret set called" >> "${HOME}/gh_calls.log"
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
    setup_vibestats_token
  " 2>&1

  # The sentinel must NOT appear in any file in $HOME (other than our own stub)
  # Remove the stub first to avoid false positives
  rm -f "${HOME}/stub_env.sh"

  # Scan all files under $HOME for the sentinel token value
  found=$(grep -rl "${SENTINEL_TOKEN}" "${HOME}/" 2>/dev/null || true)
  [ -z "$found" ]

  # Also assert the sentinel does NOT appear in stdout
  [[ "$output" != *"${SENTINEL_TOKEN}"* ]]
}

# ---------------------------------------------------------------------------
# AC #3 — gh secret set is called with VIBESTATS_TOKEN secret name (FR10)
# P1 — Story 6.2, FR10
# ---------------------------------------------------------------------------
@test "[P1] gh secret set called with VIBESTATS_TOKEN for vibestats-data repo" {
  cat > "${HOME}/stub_env.sh" <<STUB
_gh() {
  case "\$1 \$2" in
    "auth token")
      echo "ghp_FAKE_MACHINE_TOKEN"
      ;;
    "api /user")
      echo '{"login":"testuser"}'
      ;;
    "repo view")
      return 1
      ;;
    "repo create")
      return 0
      ;;
    "api repos"*)
      return 0
      ;;
    "api /user/personal_access_tokens"*)
      echo '{"token":"ghp_FAKE_VIBESTATS_TOKEN"}'
      ;;
    "secret set")
      echo "gh secret set: \$*" >> "${HOME}/gh_calls.log"
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
    setup_vibestats_token
  " 2>&1

  [ "$status" -eq 0 ]
  [ -f "${HOME}/gh_calls.log" ]
  grep -q "VIBESTATS_TOKEN" "${HOME}/gh_calls.log"
}

# ---------------------------------------------------------------------------
# AC #4 — local token written to ~/.config/vibestats/config.toml (FR39)
# P1 — Story 6.2, R-002, NFR6, FR39
# ---------------------------------------------------------------------------
@test "[P1] gh auth token result stored in ~/.config/vibestats/config.toml" {
  cat > "${HOME}/stub_env.sh" <<STUB
_gh() {
  case "\$1 \$2" in
    "auth token")
      echo "ghp_FAKE_MACHINE_TOKEN_12345"
      ;;
    "api /user")
      echo '{"login":"testuser"}'
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
    store_machine_token
  " 2>&1

  [ "$status" -eq 0 ]
  [ -f "${HOME}/.config/vibestats/config.toml" ]
  grep -q "token" "${HOME}/.config/vibestats/config.toml"
}

# ---------------------------------------------------------------------------
# AC #4 — config.toml has permissions 600 (NFR6)
# P0 — Story 6.2, R-002, NFR6
# ---------------------------------------------------------------------------
@test "[P0] ~/.config/vibestats/config.toml created with permissions 600" {
  cat > "${HOME}/stub_env.sh" <<STUB
_gh() {
  case "\$1 \$2" in
    "auth token")
      echo "ghp_FAKE_MACHINE_TOKEN_12345"
      ;;
    "api /user")
      echo '{"login":"testuser"}'
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
    store_machine_token
  " 2>&1

  [ "$status" -eq 0 ]
  [ -f "${HOME}/.config/vibestats/config.toml" ]

  # Platform-aware permission check
  case "$(uname -s)" in
    Darwin)
      PERMS=$(stat -f "%Lp" "${HOME}/.config/vibestats/config.toml")
      ;;
    Linux)
      PERMS=$(stat -c "%a" "${HOME}/.config/vibestats/config.toml")
      ;;
  esac

  [ "$PERMS" = "600" ]
}

# ---------------------------------------------------------------------------
# AC #4 — installer exits non-zero when gh auth token fails (R-003)
# P0 — Story 6.2, R-003, OPS
# ---------------------------------------------------------------------------
@test "[P0] installer exits non-zero and prints error when gh auth token fails" {
  cat > "${HOME}/stub_env.sh" <<'STUB'
_gh() {
  case "$1 $2" in
    "auth token")
      echo "Error: not authenticated" >&2
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
    store_machine_token
  " 2>&1

  [ "$status" -ne 0 ]
}

# ---------------------------------------------------------------------------
# AC #5 — registry.json contains machine entry with machine_id field (FR6, R-005)
# P0 — Story 6.2, R-005, FR6
# ---------------------------------------------------------------------------
@test "[P0] registry.json entry contains machine_id field" {
  cat > "${HOME}/stub_env.sh" <<STUB
_gh() {
  case "\$1 \$2" in
    "auth token")
      echo "ghp_FAKE_MACHINE_TOKEN"
      ;;
    "api /user")
      echo '{"login":"testuser"}'
      ;;
    "api repos/testuser/vibestats-data/contents/registry.json"*)
      # Simulate empty registry (first install — file does not exist yet)
      return 1
      ;;
    "api repos"*)
      # Capture PUT body to a log file for inspection
      echo "gh api put called: \$*" >> "${HOME}/gh_calls.log"
      # Read and save the JSON body passed via stdin (--input -)
      cat >> "${HOME}/gh_api_body.log" 2>/dev/null || true
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
    register_machine
  " 2>&1

  [ "$status" -eq 0 ]
  # registry.json content must include machine_id
  [[ "$output" == *"machine_id"* ]] || grep -q "machine_id" "${HOME}/gh_api_body.log" 2>/dev/null
}

# ---------------------------------------------------------------------------
# AC #5 — registry.json entry contains hostname field (FR6, R-005)
# P0 — Story 6.2, R-005, FR6
# ---------------------------------------------------------------------------
@test "[P0] registry.json entry contains hostname field" {
  cat > "${HOME}/stub_env.sh" <<STUB
_gh() {
  case "\$1 \$2" in
    "auth token")
      echo "ghp_FAKE_MACHINE_TOKEN"
      ;;
    "api /user")
      echo '{"login":"testuser"}'
      ;;
    "api repos/testuser/vibestats-data/contents/registry.json"*)
      return 1
      ;;
    "api repos"*)
      echo "gh api put called: \$*" >> "${HOME}/gh_calls.log"
      cat >> "${HOME}/gh_api_body.log" 2>/dev/null || true
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
    register_machine
  " 2>&1

  [ "$status" -eq 0 ]
  [[ "$output" == *"hostname"* ]] || grep -q "hostname" "${HOME}/gh_api_body.log" 2>/dev/null
}

# ---------------------------------------------------------------------------
# AC #5 — registry.json entry has status = "active" (FR6, R-005)
# P0 — Story 6.2, R-005, FR6
# ---------------------------------------------------------------------------
@test "[P0] registry.json entry has status field set to active" {
  cat > "${HOME}/stub_env.sh" <<STUB
_gh() {
  case "\$1 \$2" in
    "auth token")
      echo "ghp_FAKE_MACHINE_TOKEN"
      ;;
    "api /user")
      echo '{"login":"testuser"}'
      ;;
    "api repos/testuser/vibestats-data/contents/registry.json"*)
      return 1
      ;;
    "api repos"*)
      echo "gh api put called: \$*" >> "${HOME}/gh_calls.log"
      cat >> "${HOME}/gh_api_body.log" 2>/dev/null || true
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
    register_machine
  " 2>&1

  [ "$status" -eq 0 ]
  [[ "$output" == *"active"* ]] || grep -q "active" "${HOME}/gh_api_body.log" 2>/dev/null
}

# ---------------------------------------------------------------------------
# AC #5 — registry.json entry has last_seen ISO 8601 UTC timestamp (FR6, R-005)
# P0 — Story 6.2, R-005, FR6
# ---------------------------------------------------------------------------
@test "[P0] registry.json entry has last_seen ISO 8601 UTC timestamp" {
  cat > "${HOME}/stub_env.sh" <<STUB
_gh() {
  case "\$1 \$2" in
    "auth token")
      echo "ghp_FAKE_MACHINE_TOKEN"
      ;;
    "api /user")
      echo '{"login":"testuser"}'
      ;;
    "api repos/testuser/vibestats-data/contents/registry.json"*)
      return 1
      ;;
    "api repos"*)
      echo "gh api put called: \$*" >> "${HOME}/gh_calls.log"
      cat >> "${HOME}/gh_api_body.log" 2>/dev/null || true
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
    register_machine
  " 2>&1

  [ "$status" -eq 0 ]
  # ISO 8601 UTC format: YYYY-MM-DDTHH:MM:SSZ
  [[ "$output" =~ [0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z ]] || \
    grep -qE "[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z" "${HOME}/gh_api_body.log" 2>/dev/null
}

# ---------------------------------------------------------------------------
# AC #3 (failure path) — installer exits non-zero when gh repo create fails (R-003)
# P0 — Story 6.2, R-003, OPS
# ---------------------------------------------------------------------------
@test "[P0] installer exits non-zero when gh repo create fails" {
  cat > "${HOME}/stub_env.sh" <<'STUB'
_gh() {
  case "$1 $2" in
    "auth token")
      echo "ghp_FAKE_MACHINE_TOKEN"
      ;;
    "api /user")
      echo '{"login":"testuser"}'
      ;;
    "repo view")
      return 1
      ;;
    "repo create")
      echo "Error: could not create repository" >&2
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
    create_vibestats_data_repo
  " 2>&1

  [ "$status" -ne 0 ]
}

# ---------------------------------------------------------------------------
# AC #3 (failure path) — installer exits non-zero when gh secret set fails (R-003)
# P0 — Story 6.2, R-003, OPS
# ---------------------------------------------------------------------------
@test "[P0] installer exits non-zero when gh secret set fails" {
  cat > "${HOME}/stub_env.sh" <<'STUB'
_gh() {
  case "$1 $2" in
    "auth token")
      echo "ghp_FAKE_MACHINE_TOKEN"
      ;;
    "api /user")
      echo '{"login":"testuser"}'
      ;;
    "api /user/personal_access_tokens"*)
      echo '{"token":"ghp_FAKE_VIBESTATS_TOKEN"}'
      ;;
    "secret set")
      echo "Error: secret set failed" >&2
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
    setup_vibestats_token
  " 2>&1

  [ "$status" -ne 0 ]
}

# ---------------------------------------------------------------------------
# First-install path integration — full first-install path completes successfully
# P1 — Story 6.2, FR4-FR6, FR10, FR39
# ---------------------------------------------------------------------------
@test "[P1] full first-install path succeeds with all steps called in sequence" {
  cat > "${HOME}/stub_env.sh" <<STUB
_gh() {
  case "\$1 \$2" in
    "auth token")
      echo "ghp_FAKE_MACHINE_TOKEN_12345"
      ;;
    "api /user")
      echo '{"login":"testuser"}'
      ;;
    "repo view")
      # vibestats-data does not exist → first-install path
      return 1
      ;;
    "repo create")
      echo "repo create: \$*" >> "${HOME}/gh_calls.log"
      return 0
      ;;
    "api repos"*)
      echo "api repos: \$*" >> "${HOME}/gh_calls.log"
      return 0
      ;;
    "api /user/personal_access_tokens"*)
      echo '{"token":"ghp_FAKE_VIBESTATS_TOKEN_67890"}'
      ;;
    "secret set")
      echo "secret set: \$*" >> "${HOME}/gh_calls.log"
      return 0
      ;;
    *)
      echo "gh \$* called" >> "${HOME}/gh_calls.log"
      return 0
      ;;
  esac
}
export -f _gh
STUB

  run bash --noprofile --norc -c "
    source '${HOME}/stub_env.sh'
    source '${INSTALL_SH}'
    first_install_path
  " 2>&1

  [ "$status" -eq 0 ]
  # All major steps must have been called
  [ -f "${HOME}/gh_calls.log" ]
  grep -q "repo create" "${HOME}/gh_calls.log"
  grep -q "secret set" "${HOME}/gh_calls.log"
  # config.toml must be created
  [ -f "${HOME}/.config/vibestats/config.toml" ]
}
