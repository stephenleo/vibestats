#!/usr/bin/env bash
set -euo pipefail

# vibestats installer
# Installs the vibestats binary, handling gh dependency detection and auth.
# Pattern: rustup-style installer — each logical step in its own function.

# ---------------------------------------------------------------------------
# gh CLI wrapper — ALL gh calls go through this helper for testability.
# Only define _gh if it hasn't already been defined (allows test overrides).
# ---------------------------------------------------------------------------
if ! declare -f _gh > /dev/null 2>&1; then
  _gh() {
    gh "$@"
  }
fi

# ---------------------------------------------------------------------------
# Step 1: Detect and install gh if missing (AC #1, FR2)
# ---------------------------------------------------------------------------
install_gh_if_missing() {
  if command -v gh > /dev/null 2>&1; then
    echo "gh is already installed."
    return 0
  fi

  echo "gh not found. Installing..."

  OS="$(uname -s)"
  case "$OS" in
    Darwin)
      if ! command -v brew > /dev/null 2>&1; then
        echo "Error: Homebrew is required to install gh on macOS but was not found." >&2
        echo "Install Homebrew from https://brew.sh then re-run this installer." >&2
        exit 1
      fi
      brew install gh
      ;;
    Linux)
      if [ "$(id -u)" -eq 0 ]; then
        apt-get install -y gh
      else
        sudo apt-get install -y gh
      fi
      ;;
    *)
      echo "Error: Unsupported OS '$OS'. Cannot install gh automatically." >&2
      exit 1
      ;;
  esac

  echo "gh installed successfully."
}

# ---------------------------------------------------------------------------
# Step 2: Check gh version >= 2.0 (AC #2, NFR16)
# ---------------------------------------------------------------------------
check_gh_version() {
  GH_VERSION=$(_gh --version | head -1 | awk '{print $3}')
  GH_MAJOR=$(echo "$GH_VERSION" | cut -d. -f1)

  if [ "$GH_MAJOR" -lt 2 ]; then
    echo "Error: gh version ${GH_VERSION} is below minimum required version 2.0. Please upgrade: brew upgrade gh" >&2
    exit 1
  fi

  echo "gh version ${GH_VERSION} meets minimum requirement (>= 2.0)."
}

# ---------------------------------------------------------------------------
# Step 3: Ensure gh is authenticated (AC #3, FR3)
# ---------------------------------------------------------------------------
check_gh_auth() {
  if _gh auth status > /dev/null 2>&1; then
    echo "gh is already authenticated."
    return 0
  fi

  echo "gh is not authenticated. Launching browser OAuth flow..."
  _gh auth login

  if ! _gh auth status > /dev/null 2>&1; then
    echo "Error: gh authentication failed. Please run 'gh auth login' manually and re-run the installer." >&2
    exit 1
  fi

  echo "gh authentication successful."
}

# ---------------------------------------------------------------------------
# Step 4: Detect platform and set TARGET (AC #4)
# ---------------------------------------------------------------------------
detect_platform() {
  OS="$(uname -s)"
  ARCH="$(uname -m)"

  case "${OS}-${ARCH}" in
    Darwin-arm64)
      TARGET="aarch64-apple-darwin"
      ;;
    Darwin-x86_64)
      TARGET="x86_64-apple-darwin"
      ;;
    Linux-x86_64)
      TARGET="x86_64-unknown-linux-gnu"
      ;;
    *)
      echo "Unsupported platform: ${OS} ${ARCH}" >&2
      exit 1
      ;;
  esac

  export TARGET
  echo "Platform detected: ${OS} ${ARCH} → target: ${TARGET}"
}

# ---------------------------------------------------------------------------
# Step 5: Download, verify, extract, and install the vibestats binary (AC #4)
# ---------------------------------------------------------------------------
download_and_install_binary() {
  detect_platform

  REPO="stephenleo/vibestats"
  BASE_URL="https://github.com/${REPO}/releases/latest/download"
  ARCHIVE="vibestats-${TARGET}.tar.gz"
  CHECKSUM="${ARCHIVE}.sha256"

  TMPDIR_WORK="$(mktemp -d)"
  trap 'rm -rf "$TMPDIR_WORK"' EXIT

  echo "Downloading ${ARCHIVE}..."
  curl -fsSL "${BASE_URL}/${ARCHIVE}" -o "${TMPDIR_WORK}/${ARCHIVE}"
  curl -fsSL "${BASE_URL}/${CHECKSUM}" -o "${TMPDIR_WORK}/${CHECKSUM}"

  echo "Verifying checksum..."
  (
    cd "$TMPDIR_WORK"
    case "$(uname -s)" in
      Darwin) shasum -a 256 -c "${CHECKSUM}" ;;
      Linux)  sha256sum -c "${CHECKSUM}" ;;
      *)
        echo "Error: Cannot verify checksum — unsupported OS for checksum tool." >&2
        exit 1
        ;;
    esac
  )

  echo "Extracting archive..."
  tar xzf "${TMPDIR_WORK}/${ARCHIVE}" -C "${TMPDIR_WORK}"

  echo "Installing to ~/.local/bin/vibestats..."
  mkdir -p "${HOME}/.local/bin"
  install -m 755 "${TMPDIR_WORK}/vibestats" "${HOME}/.local/bin/vibestats"

  if ! "${HOME}/.local/bin/vibestats" --version > /dev/null 2>&1; then
    echo "Error: vibestats installed but --version check failed. Binary may not be on \$PATH." >&2
    echo "Add ~/.local/bin to your PATH: export PATH=\"\$HOME/.local/bin:\$PATH\"" >&2
    exit 1
  fi

  echo "vibestats installed successfully to ~/.local/bin/vibestats"
}

# ---------------------------------------------------------------------------
# Main entrypoint
# ---------------------------------------------------------------------------
main() {
  echo "=== vibestats installer ==="

  install_gh_if_missing
  check_gh_version
  check_gh_auth
  download_and_install_binary

  echo "=== Installation complete! ==="
  echo "Run 'vibestats --help' to get started."
}

# Only run main when the script is executed directly (not sourced).
# This allows test files to source install.sh and call individual functions.
# $BASH_SOURCE is available in Bash 3.2+; equals $0 only when not sourced.
if [ "${BASH_SOURCE:-$0}" = "$0" ]; then
  main "$@"
fi
