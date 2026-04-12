#!/usr/bin/env bats
# Story 6.1: Implement dependency detection and gh authentication
# ATDD Red Phase — tests assert expected behaviour; will fail until install.sh is implemented.
#
# Run: bats tests/installer/test_6_1.bats
#
# Test framework: bats-core
# Mocking strategy: override _gh() helper and shell commands via functions exported to subshells.
# All tests use a temp $HOME to prevent mutation of the developer's real config.

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
# Helper: source install.sh functions into test scope without executing main()
# ---------------------------------------------------------------------------
_source_install_functions() {
  # Source install.sh but prevent main() from running
  # We achieve this by overriding main() before sourcing and unoverriding after
  # shellcheck disable=SC1090
  source "$INSTALL_SH" 2>/dev/null || true
}

# ---------------------------------------------------------------------------
# AC #1 — gh not installed → brew install gh called on Darwin (macOS)
# P1 — Story 6.1, FR2
# ---------------------------------------------------------------------------
@test "[P1] gh not installed → brew install gh called on Darwin" {
  # Override _gh to simulate gh not found; stub uname and brew
  cat > "${HOME}/stub_env.sh" <<'STUB'
command() {
  if [ "$1" = "-v" ] && [ "$2" = "gh" ]; then
    return 1  # gh not found
  fi
  builtin command "$@"
}
uname() {
  case "$1" in
    -s) echo "Darwin" ;;
    -m) echo "arm64" ;;
    *)  command uname "$@" ;;
  esac
}
brew() {
  echo "brew install gh called with: $*" >> "${HOME}/brew_calls.log"
}
export -f command uname brew
STUB

  run bash --noprofile --norc -c "
    source '${HOME}/stub_env.sh'
    source '${INSTALL_SH}'
    install_gh_if_missing
  " 2>&1

  [ "$status" -eq 0 ]
  [ -f "${HOME}/brew_calls.log" ]
  grep -q "install gh" "${HOME}/brew_calls.log"
}

# ---------------------------------------------------------------------------
# AC #1 — gh not installed → apt-get install gh called on Linux
# P1 — Story 6.1, FR2
# ---------------------------------------------------------------------------
@test "[P1] gh not installed → apt-get install gh called on Linux" {
  cat > "${HOME}/stub_env.sh" <<'STUB'
command() {
  if [ "$1" = "-v" ] && [ "$2" = "gh" ]; then
    return 1  # gh not found
  fi
  builtin command "$@"
}
uname() {
  case "$1" in
    -s) echo "Linux" ;;
    -m) echo "x86_64" ;;
    *)  command uname "$@" ;;
  esac
}
apt-get() {
  echo "apt-get $* called" >> "${HOME}/aptget_calls.log"
}
sudo() {
  # Pass through to the command being sudo'd
  "$@"
}
export -f command uname apt-get sudo
STUB

  run bash --noprofile --norc -c "
    source '${HOME}/stub_env.sh'
    source '${INSTALL_SH}'
    install_gh_if_missing
  " 2>&1

  [ "$status" -eq 0 ]
  [ -f "${HOME}/aptget_calls.log" ]
  grep -q "install" "${HOME}/aptget_calls.log"
  grep -q "gh" "${HOME}/aptget_calls.log"
}

# ---------------------------------------------------------------------------
# AC #2 — gh version < 2.0 → exits non-zero with message containing version
# P1 — Story 6.1, NFR16, R-006
# ---------------------------------------------------------------------------
@test "[P1] gh version < 2.0 → exits non-zero with error message" {
  cat > "${HOME}/stub_env.sh" <<'STUB'
_gh() {
  case "$1" in
    --version) echo "gh version 1.14.0 (2022-01-01)" ;;
    *)         return 0 ;;
  esac
}
export -f _gh
STUB

  run bash --noprofile --norc -c "
    source '${HOME}/stub_env.sh'
    source '${INSTALL_SH}'
    check_gh_version
  " 2>&1

  [ "$status" -ne 0 ]
  [[ "$output" == *"1.14.0"* ]]
}

# ---------------------------------------------------------------------------
# AC #2 — error message includes minimum required version (2.0)
# P1 — Story 6.1, NFR16
# ---------------------------------------------------------------------------
@test "[P1] gh version < 2.0 → error message includes minimum version 2.0" {
  cat > "${HOME}/stub_env.sh" <<'STUB'
_gh() {
  case "$1" in
    --version) echo "gh version 1.9.0 (2021-06-01)" ;;
    *)         return 0 ;;
  esac
}
export -f _gh
STUB

  run bash --noprofile --norc -c "
    source '${HOME}/stub_env.sh'
    source '${INSTALL_SH}'
    check_gh_version
  " 2>&1

  [ "$status" -ne 0 ]
  [[ "$output" == *"2.0"* ]]
}

# ---------------------------------------------------------------------------
# AC #3 — gh not authenticated → gh auth login called
# P1 — Story 6.1, FR3
# ---------------------------------------------------------------------------
@test "[P1] gh not authenticated → gh auth login called" {
  cat > "${HOME}/stub_env.sh" <<'STUB'
_gh() {
  case "$1 $2" in
    "auth status")
      return 1  # not authenticated
      ;;
    "auth login")
      echo "gh auth login called" >> "${HOME}/gh_calls.log"
      return 0
      ;;
    "auth status")
      # Second call after login succeeds
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
    check_gh_auth
  " 2>&1

  [ -f "${HOME}/gh_calls.log" ]
  grep -q "gh auth login called" "${HOME}/gh_calls.log"
}

# ---------------------------------------------------------------------------
# AC #4 — platform Darwin arm64 → correct target aarch64-apple-darwin selected
# P1 — Story 6.1, R-007
# ---------------------------------------------------------------------------
@test "[P1] platform Darwin arm64 → target is aarch64-apple-darwin" {
  cat > "${HOME}/stub_env.sh" <<'STUB'
uname() {
  case "$1" in
    -s) echo "Darwin" ;;
    -m) echo "arm64" ;;
    *)  command uname "$@" ;;
  esac
}
export -f uname
STUB

  run bash --noprofile --norc -c "
    source '${HOME}/stub_env.sh'
    source '${INSTALL_SH}'
    detect_platform
    echo \"TARGET=\${TARGET}\"
  " 2>&1

  [ "$status" -eq 0 ]
  [[ "$output" == *"TARGET=aarch64-apple-darwin"* ]]
}

# ---------------------------------------------------------------------------
# AC #4 — platform Darwin x86_64 → correct target x86_64-apple-darwin selected
# P1 — Story 6.1, R-007
# ---------------------------------------------------------------------------
@test "[P1] platform Darwin x86_64 → target is x86_64-apple-darwin" {
  cat > "${HOME}/stub_env.sh" <<'STUB'
uname() {
  case "$1" in
    -s) echo "Darwin" ;;
    -m) echo "x86_64" ;;
    *)  command uname "$@" ;;
  esac
}
export -f uname
STUB

  run bash --noprofile --norc -c "
    source '${HOME}/stub_env.sh'
    source '${INSTALL_SH}'
    detect_platform
    echo \"TARGET=\${TARGET}\"
  " 2>&1

  [ "$status" -eq 0 ]
  [[ "$output" == *"TARGET=x86_64-apple-darwin"* ]]
}

# ---------------------------------------------------------------------------
# AC #4 — platform Linux x86_64 → correct target x86_64-unknown-linux-gnu selected
# P1 — Story 6.1, R-007
# ---------------------------------------------------------------------------
@test "[P1] platform Linux x86_64 → target is x86_64-unknown-linux-gnu" {
  cat > "${HOME}/stub_env.sh" <<'STUB'
uname() {
  case "$1" in
    -s) echo "Linux" ;;
    -m) echo "x86_64" ;;
    *)  command uname "$@" ;;
  esac
}
export -f uname
STUB

  run bash --noprofile --norc -c "
    source '${HOME}/stub_env.sh'
    source '${INSTALL_SH}'
    detect_platform
    echo \"TARGET=\${TARGET}\"
  " 2>&1

  [ "$status" -eq 0 ]
  [[ "$output" == *"TARGET=x86_64-unknown-linux-gnu"* ]]
}

# ---------------------------------------------------------------------------
# AC #4 — unsupported platform → exits non-zero with clear message
# P1 — Story 6.1, R-007
# ---------------------------------------------------------------------------
@test "[P1] unsupported platform → exits non-zero with message" {
  cat > "${HOME}/stub_env.sh" <<'STUB'
uname() {
  case "$1" in
    -s) echo "FreeBSD" ;;
    -m) echo "arm64" ;;
    *)  command uname "$@" ;;
  esac
}
export -f uname
STUB

  run bash --noprofile --norc -c "
    source '${HOME}/stub_env.sh'
    source '${INSTALL_SH}'
    detect_platform
  " 2>&1

  [ "$status" -ne 0 ]
  [[ "$output" == *"Unsupported platform"* ]] || [[ "$output" == *"unsupported"* ]]
}

# ---------------------------------------------------------------------------
# AC #1 (idempotency) — gh already installed and version ≥ 2.0 → no brew/apt-get called
# P2 — Story 6.1, FR2, R-006
# ---------------------------------------------------------------------------
@test "[P2] gh installed and version >= 2.0 → no install attempted" {
  cat > "${HOME}/stub_env.sh" <<'STUB'
command() {
  if [ "$1" = "-v" ] && [ "$2" = "gh" ]; then
    echo "/usr/local/bin/gh"
    return 0  # gh is found
  fi
  builtin command "$@"
}
_gh() {
  case "$1" in
    --version) echo "gh version 2.44.1 (2024-11-12)" ;;
    *)         return 0 ;;
  esac
}
brew() {
  echo "brew called unexpectedly" >> "${HOME}/brew_calls.log"
}
apt-get() {
  echo "apt-get called unexpectedly" >> "${HOME}/aptget_calls.log"
}
export -f command _gh brew apt-get
STUB

  run bash --noprofile --norc -c "
    source '${HOME}/stub_env.sh'
    source '${INSTALL_SH}'
    install_gh_if_missing
  " 2>&1

  [ "$status" -eq 0 ]
  # Neither brew nor apt-get should have been called
  [ ! -f "${HOME}/brew_calls.log" ]
  [ ! -f "${HOME}/aptget_calls.log" ]
}
