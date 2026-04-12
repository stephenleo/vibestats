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

  # Verify gh is now accessible on PATH (brew may install to /opt/homebrew/bin
  # which is not always in PATH in non-interactive shells).
  if ! command -v gh > /dev/null 2>&1; then
    echo "Error: gh was installed but is not accessible on \$PATH." >&2
    echo "Add the install location to your PATH and re-run the installer." >&2
    exit 1
  fi

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
# Step 6: Detect if this is a first install (no vibestats-data repo exists)
# Returns 0 if first install is needed, 1 if vibestats-data already exists.
# Also detects and exports USERNAME for use by subsequent functions.
# ---------------------------------------------------------------------------
detect_first_install() {
  USERNAME=$(_gh api /user --jq '.login')
  export USERNAME

  if _gh repo view "${USERNAME}/vibestats-data" > /dev/null 2>&1; then
    # Repo exists — multi-machine path
    return 1
  fi

  # Repo does not exist — first-install path
  return 0
}

# ---------------------------------------------------------------------------
# Step 7: Create the vibestats-data private repo (AC #1, FR4)
# Requires: USERNAME exported by detect_first_install (auto-detected if not set)
# ---------------------------------------------------------------------------
create_vibestats_data_repo() {
  # Detect USERNAME if not already exported (e.g. when called directly in tests)
  if [ -z "${USERNAME:-}" ]; then
    USERNAME=$(_gh api /user --jq '.login')
    export USERNAME
  fi

  echo "Creating vibestats-data repository..."
  _gh repo create "${USERNAME}/vibestats-data" --private \
    || { echo "Error: Failed to create vibestats-data repository." >&2; exit 1; }
  echo "Repository created: ${USERNAME}/vibestats-data"
}

# ---------------------------------------------------------------------------
# Step 8: Generate aggregate workflow YAML content (AC #2, FR7)
# Echoes the workflow YAML to stdout for embedding or processing.
# ---------------------------------------------------------------------------
generate_aggregate_workflow_content() {
  cat <<'WORKFLOW'
# aggregate.yml — Copy this file to your vibestats-data/.github/workflows/ directory.
# It runs the vibestats community action daily to aggregate your Claude Code session data
# and update your GitHub profile heatmap automatically.
name: Aggregate vibestats data

on:
  schedule:
    - cron: '0 2 * * *'   # Daily at 02:00 UTC
  workflow_dispatch:        # Allow manual runs

jobs:
  aggregate:
    runs-on: ubuntu-latest
    steps:
      - uses: stephenleo/vibestats@v1
        with:
          token: ${{ secrets.VIBESTATS_TOKEN }}
          profile-repo: ${{ github.repository_owner }}/${{ github.repository_owner }}
WORKFLOW
}

# ---------------------------------------------------------------------------
# Step 9: Write aggregate.yml into vibestats-data/.github/workflows/ (AC #2, FR7)
# Requires: USERNAME exported by detect_first_install (auto-detected if not set)
# ---------------------------------------------------------------------------
write_aggregate_workflow() {
  # Detect USERNAME if not already exported
  if [ -z "${USERNAME:-}" ]; then
    USERNAME=$(_gh api /user --jq '.login')
    export USERNAME
  fi

  echo "Writing aggregate workflow to vibestats-data..."

  WORKFLOW_CONTENT=$(generate_aggregate_workflow_content)
  CONTENT=$(printf '%s' "$WORKFLOW_CONTENT" | base64 | tr -d '\n')

  echo "Workflow content:"
  printf '%s\n' "$WORKFLOW_CONTENT"

  _gh api "repos/${USERNAME}/vibestats-data/contents/.github/workflows/aggregate.yml" \
    --method PUT \
    --field message="Add vibestats aggregate workflow" \
    --field "content=${CONTENT}" \
    || { echo "Error: Failed to write aggregate.yml to vibestats-data." >&2; exit 1; }

  echo "Workflow written: vibestats-data/.github/workflows/aggregate.yml"
}

# ---------------------------------------------------------------------------
# Step 10: Generate and set VIBESTATS_TOKEN Actions secret (AC #3, FR10, NFR7)
# SECURITY: VIBESTATS_TOKEN is NEVER written to disk or echoed to stdout.
# Requires: USERNAME exported by detect_first_install (auto-detected if not set)
# ---------------------------------------------------------------------------
setup_vibestats_token() {
  # Detect USERNAME if not already exported
  if [ -z "${USERNAME:-}" ]; then
    USERNAME=$(_gh api /user --jq '.login')
    export USERNAME
  fi

  echo "Setting up VIBESTATS_TOKEN Actions secret..."

  # Attempt to generate a fine-grained PAT and pipe directly to gh secret set.
  # The token value is never stored in a variable that could be written to disk.
  if _gh api /user/personal_access_tokens \
      --method POST \
      --field name="vibestats-$(date +%Y)" \
      --field expiration="never" \
      --field repositories='["'"${USERNAME}"'"]' \
      --field permissions='{"contents":"write"}' \
      --jq '.token' \
      | _gh secret set VIBESTATS_TOKEN --repo "${USERNAME}/vibestats-data"; then
    echo "VIBESTATS_TOKEN secret set successfully."
  else
    # Fallback: enterprise may block fine-grained PAT creation
    echo "Warning: Fine-grained PAT creation blocked. Using gh auth token as VIBESTATS_TOKEN fallback."
    _gh auth token \
      | _gh secret set VIBESTATS_TOKEN --repo "${USERNAME}/vibestats-data" \
      || { echo "Error: Failed to set VIBESTATS_TOKEN secret." >&2; exit 1; }
    echo "VIBESTATS_TOKEN secret set via fallback."
  fi
}

# ---------------------------------------------------------------------------
# Step 11: Store machine-side token in ~/.config/vibestats/config.toml (AC #4, NFR6)
# Computes and exports MACHINE_ID for reuse by register_machine().
# ---------------------------------------------------------------------------
store_machine_token() {
  echo "Storing machine token in ~/.config/vibestats/config.toml..."

  # Compute deterministic machine ID (POSIX cksum — works on macOS and Linux)
  HOSTNAME_LOWER=$(hostname | tr '[:upper:]' '[:lower:]' | sed 's/\..*$//' | sed 's/[^a-z0-9-]/-/g')
  HASH=$(hostname | cksum | awk '{print $1}' | cut -c1-6)
  MACHINE_ID="${HOSTNAME_LOWER}-${HASH}"
  export MACHINE_ID

  # Obtain machine-side token
  LOCAL_TOKEN=$(_gh auth token) \
    || { echo "Error: Failed to obtain machine token via 'gh auth token'. Ensure gh is authenticated." >&2; exit 1; }

  # Detect USERNAME if not already exported
  if [ -z "${USERNAME:-}" ]; then
    USERNAME=$(_gh api /user --jq '.login')
    export USERNAME
  fi

  # Create config directory and write config
  mkdir -p "${HOME}/.config/vibestats"
  cat > "${HOME}/.config/vibestats/config.toml" <<TOML
oauth_token = "${LOCAL_TOKEN}"
machine_id = "${MACHINE_ID}"
vibestats_data_repo = "${USERNAME}/vibestats-data"
TOML

  # Immediately set secure permissions (NFR6)
  chmod 600 "${HOME}/.config/vibestats/config.toml"

  # Clear the token from memory
  unset LOCAL_TOKEN

  echo "Machine token stored at ~/.config/vibestats/config.toml"
}

# ---------------------------------------------------------------------------
# Step 12: Register machine in vibestats-data/registry.json (AC #5, FR6)
# Reuses MACHINE_ID exported by store_machine_token().
# ---------------------------------------------------------------------------
register_machine() {
  echo "Registering machine in vibestats-data/registry.json..."

  # Compute MACHINE_ID if not already set (e.g. when called directly in tests)
  if [ -z "${MACHINE_ID:-}" ]; then
    HOSTNAME_LOWER=$(hostname | tr '[:upper:]' '[:lower:]' | sed 's/\..*$//' | sed 's/[^a-z0-9-]/-/g')
    HASH=$(hostname | cksum | awk '{print $1}' | cut -c1-6)
    MACHINE_ID="${HOSTNAME_LOWER}-${HASH}"
    export MACHINE_ID
  fi

  # Detect USERNAME if not already exported
  if [ -z "${USERNAME:-}" ]; then
    USERNAME=$(_gh api /user --jq '.login')
    export USERNAME
  fi

  LAST_SEEN=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
  HOSTNAME_VAL=$(hostname)

  # Build registry.json payload (Bash 3.2 safe — no jq required)
  REGISTRY_JSON='{"machines":[{"machine_id":"'"${MACHINE_ID}"'","hostname":"'"${HOSTNAME_VAL}"'","status":"active","last_seen":"'"${LAST_SEEN}"'"}]}'

  echo "Registry entry: ${REGISTRY_JSON}"

  # Base64-encode with line-wrap removal (macOS base64 wraps at 76 chars)
  CONTENT=$(printf '%s' "$REGISTRY_JSON" | base64 | tr -d '\n')

  _gh api "repos/${USERNAME}/vibestats-data/contents/registry.json" \
    --method PUT \
    --field message="Register machine ${MACHINE_ID}" \
    --field "content=${CONTENT}" \
    || { echo "Error: Failed to register machine in registry.json." >&2; exit 1; }

  echo "Machine registered: ${MACHINE_ID}"
}

# ---------------------------------------------------------------------------
# First-install path: runs all first-install setup steps in sequence.
# Called when detect_first_install() returns 0.
# ---------------------------------------------------------------------------
first_install_path() {
  create_vibestats_data_repo
  write_aggregate_workflow
  setup_vibestats_token
  store_machine_token
  register_machine
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

  if detect_first_install; then
    create_vibestats_data_repo
    write_aggregate_workflow
    setup_vibestats_token
  fi
  store_machine_token
  register_machine

  echo "=== Installation complete! ==="
  echo "Run 'vibestats --help' to get started."
}

# Only run main when the script is executed directly (not sourced).
# This allows test files to source install.sh and call individual functions.
# $BASH_SOURCE is available in Bash 3.2+; equals $0 only when not sourced.
if [ "${BASH_SOURCE:-$0}" = "$0" ]; then
  main "$@"
fi
