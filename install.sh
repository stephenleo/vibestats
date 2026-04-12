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
    GH_PAGER= gh "$@"
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
# Step 6: Detect whether this is a first-install or multi-machine install (AC #1)
# Sets INSTALL_MODE to "multi-machine" or "first-install".
# Sets GITHUB_USER to the authenticated GitHub username.
# MUST use 'if' construct (not $? check) because set -euo pipefail aborts on
# non-zero exit from _gh repo view when the repo does not exist.
# ---------------------------------------------------------------------------
detect_install_mode() {
  # Fetch the authenticated user; parse login via python to avoid relying on --jq
  # (jq is not a guaranteed dependency; python3 is available on macOS and most Linux)
  USER_JSON=$(_gh api /user)
  GITHUB_USER=$(echo "$USER_JSON" | python3 -c "import sys, json; print(json.load(sys.stdin)['login'])")
  export GITHUB_USER

  if _gh repo view "${GITHUB_USER}/vibestats-data" --json name > /dev/null 2>&1; then
    echo "Existing vibestats-data repo detected. Running multi-machine setup."
    INSTALL_MODE="multi-machine"
  else
    echo "No vibestats-data repo found. Running first-install setup."
    INSTALL_MODE="first-install"
  fi

  export INSTALL_MODE
}

# ---------------------------------------------------------------------------
# Step 7: Create the vibestats-data private repo (AC #1, FR4)
# Requires: GITHUB_USER exported by detect_install_mode (auto-detected if not set)
# ---------------------------------------------------------------------------
create_vibestats_data_repo() {
  # Detect GITHUB_USER if not already exported (e.g. when called directly in tests)
  if [ -z "${GITHUB_USER:-}" ]; then
    USER_JSON=$(_gh api /user)
    GITHUB_USER=$(echo "$USER_JSON" | python3 -c "import sys, json; print(json.load(sys.stdin)['login'])")
    export GITHUB_USER
  fi

  echo "Creating vibestats-data repository..."
  _gh repo create "${GITHUB_USER}/vibestats-data" --private \
    || { echo "Error: Failed to create vibestats-data repository." >&2; exit 1; }
  echo "Repository created: ${GITHUB_USER}/vibestats-data"
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
# Requires: GITHUB_USER exported by detect_install_mode (auto-detected if not set)
# ---------------------------------------------------------------------------
write_aggregate_workflow() {
  # Detect GITHUB_USER if not already exported
  if [ -z "${GITHUB_USER:-}" ]; then
    USER_JSON=$(_gh api /user)
    GITHUB_USER=$(echo "$USER_JSON" | python3 -c "import sys, json; print(json.load(sys.stdin)['login'])")
    export GITHUB_USER
  fi

  echo "Writing aggregate workflow to vibestats-data..."

  WORKFLOW_CONTENT=$(generate_aggregate_workflow_content)
  CONTENT=$(printf '%s' "$WORKFLOW_CONTENT" | base64 | tr -d '\n')

  echo "Workflow content:"
  printf '%s\n' "$WORKFLOW_CONTENT"

  _gh api "repos/${GITHUB_USER}/vibestats-data/contents/.github/workflows/aggregate.yml" \
    --method PUT \
    --field message="Add vibestats aggregate workflow" \
    --field "content=${CONTENT}" \
    || { echo "Error: Failed to write aggregate.yml to vibestats-data." >&2; exit 1; }

  echo "Workflow written: vibestats-data/.github/workflows/aggregate.yml"
}

# ---------------------------------------------------------------------------
# Step 10: Generate and set VIBESTATS_TOKEN Actions secret (AC #3, FR10, NFR7)
# SECURITY: VIBESTATS_TOKEN is NEVER written to disk or echoed to stdout.
# Requires: GITHUB_USER exported by detect_install_mode (auto-detected if not set)
# ---------------------------------------------------------------------------
setup_vibestats_token() {
  # Detect GITHUB_USER if not already exported
  if [ -z "${GITHUB_USER:-}" ]; then
    USER_JSON=$(_gh api /user)
    GITHUB_USER=$(echo "$USER_JSON" | python3 -c "import sys, json; print(json.load(sys.stdin)['login'])")
    export GITHUB_USER
  fi

  echo "Setting up VIBESTATS_TOKEN Actions secret..."

  # Attempt to generate a fine-grained PAT and pipe directly to gh secret set.
  # The token value is never stored in a variable that could be written to disk.
  if _gh api /user/personal_access_tokens \
      --method POST \
      --field name="vibestats-$(date +%Y)" \
      --field expiration="never" \
      --field repositories='["'"${GITHUB_USER}"'"]' \
      --field permissions='{"contents":"write"}' \
      --jq '.token' \
      | _gh secret set VIBESTATS_TOKEN --repo "${GITHUB_USER}/vibestats-data"; then
    echo "VIBESTATS_TOKEN secret set successfully."
  else
    # Fallback: enterprise may block fine-grained PAT creation
    echo "Warning: Fine-grained PAT creation blocked. Using gh auth token as VIBESTATS_TOKEN fallback."
    _gh auth token \
      | _gh secret set VIBESTATS_TOKEN --repo "${GITHUB_USER}/vibestats-data" \
      || { echo "Error: Failed to set VIBESTATS_TOKEN secret." >&2; exit 1; }
    echo "VIBESTATS_TOKEN secret set via fallback."
  fi
}

# ---------------------------------------------------------------------------
# Step 11: Register this machine in registry.json via GitHub Contents API (AC #2)
# Appends a new entry to the machines array; does NOT overwrite existing entries.
# Writes config.toml with chmod 600 (NFR6).
# Requires: GITHUB_USER exported by detect_install_mode (auto-detected if not set)
# ---------------------------------------------------------------------------
register_machine() {
  # Detect GITHUB_USER if not already exported
  if [ -z "${GITHUB_USER:-}" ]; then
    USER_JSON=$(_gh api /user)
    GITHUB_USER=$(echo "$USER_JSON" | python3 -c "import sys, json; print(json.load(sys.stdin)['login'])")
    export GITHUB_USER
  fi

  echo "Registering machine in vibestats-data/registry.json..."

  # --- Generate machine_id ---
  HOSTNAME_VAL="$(hostname)"
  case "$(uname -s)" in
    Darwin)
      SUFFIX=$(system_profiler SPHardwareDataType 2>/dev/null \
        | awk '/Hardware UUID/{print $3}' \
        | tr -d '-' \
        | cut -c1-6 \
        | tr '[:upper:]' '[:lower:]')
      # Fallback if system_profiler unavailable
      if [ -z "$SUFFIX" ]; then
        SUFFIX="$(date +%s | cut -c-6)"
      fi
      ;;
    Linux)
      SUFFIX=$(cut -c1-6 /etc/machine-id 2>/dev/null || uuidgen | tr -d '-' | cut -c1-6 | tr '[:upper:]' '[:lower:]')
      ;;
    *)
      SUFFIX="$(date +%s | cut -c-6)"
      ;;
  esac
  MACHINE_ID="${HOSTNAME_VAL}-${SUFFIX}"

  # --- Fetch existing registry.json (handle 404 = first machine) ---
  # Single API call — parse content and sha from the same response to avoid
  # race conditions and redundant network round-trips.
  # Branch on exit code, not captured output: gh api writes the error JSON body
  # to stdout on non-2xx responses, so the "|| echo NOT_FOUND" sentinel pattern
  # would produce "<error-json>\nNOT_FOUND", making the equality check fail.
  if REGISTRY_RESPONSE=$(_gh api "repos/${GITHUB_USER}/vibestats-data/contents/registry.json" 2>/dev/null); then
    ENCODED=$(echo "$REGISTRY_RESPONSE" | python3 -c "import sys, json; print(json.load(sys.stdin)['content'])")
    SHA=$(echo "$REGISTRY_RESPONSE" | python3 -c "import sys, json; print(json.load(sys.stdin)['sha'])")
    case "$(uname -s)" in
      Darwin) CURRENT_JSON=$(echo "$ENCODED" | base64 -D) ;;
      Linux)  CURRENT_JSON=$(echo "$ENCODED" | base64 -d) ;;
      *)      CURRENT_JSON=$(echo "$ENCODED" | base64 -d) ;;
    esac
  else
    CURRENT_JSON='{"machines": []}'
    SHA=""
  fi

  # --- Build updated JSON (Python 3 stdlib only — no jq required) ---
  TIMESTAMP="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
  NEW_JSON=$(python3 -c "
import sys, json
data = json.loads(sys.argv[1])
data['machines'].append({
  'machine_id': sys.argv[2],
  'hostname': sys.argv[3],
  'status': 'active',
  'last_seen': sys.argv[4]
})
print(json.dumps(data, indent=2))
" "$CURRENT_JSON" "$MACHINE_ID" "$HOSTNAME_VAL" "$TIMESTAMP")

  # --- Base64-encode for Contents API PUT ---
  case "$(uname -s)" in
    Darwin) ENCODED_NEW=$(echo "$NEW_JSON" | base64) ;;
    Linux)  ENCODED_NEW=$(echo "$NEW_JSON" | base64 -w 0) ;;
    *)      ENCODED_NEW=$(echo "$NEW_JSON" | base64 -w 0) ;;
  esac

  # --- PUT updated registry.json ---
  if [ -n "$SHA" ]; then
    _gh api "repos/${GITHUB_USER}/vibestats-data/contents/registry.json" \
      --method PUT \
      --field "message=chore: register machine ${MACHINE_ID}" \
      --field "content=${ENCODED_NEW}" \
      --field "sha=${SHA}"
  else
    _gh api "repos/${GITHUB_USER}/vibestats-data/contents/registry.json" \
      --method PUT \
      --field "message=chore: register machine ${MACHINE_ID}" \
      --field "content=${ENCODED_NEW}"
  fi

  # --- Write config.toml with 600 permissions (NFR6) ---
  CONFIG_DIR="${HOME}/.config/vibestats"
  CONFIG_FILE="${CONFIG_DIR}/config.toml"
  mkdir -p "$CONFIG_DIR"
  OAUTH_TOKEN=$(_gh auth token)
  cat > "$CONFIG_FILE" <<EOF
oauth_token = "${OAUTH_TOKEN}"
machine_id = "${MACHINE_ID}"
vibestats_data_repo = "${GITHUB_USER}/vibestats-data"
EOF
  chmod 600 "$CONFIG_FILE"

  # Clear the token from memory
  unset OAUTH_TOKEN

  echo "Machine registered: ${MACHINE_ID} (${HOSTNAME_VAL})"
}

# ---------------------------------------------------------------------------
# First-install path: runs all first-install setup steps in sequence.
# Called when INSTALL_MODE is "first-install".
# ---------------------------------------------------------------------------
first_install_path() {
  create_vibestats_data_repo
  write_aggregate_workflow
  setup_vibestats_token
  register_machine
}

# ---------------------------------------------------------------------------
# Step 12: Configure Claude Code hooks in ~/.claude/settings.json (AC #1, FR8)
# Writes Stop and SessionStart hooks; idempotent — safe to run multiple times.
# Uses python3 stdlib only (no jq required).
# ---------------------------------------------------------------------------
configure_hooks() {
  CLAUDE_SETTINGS="${HOME}/.claude/settings.json"
  mkdir -p "${HOME}/.claude"
  python3 - "$CLAUDE_SETTINGS" <<'PYEOF'
import sys, json

settings_path = sys.argv[1]
try:
    with open(settings_path, 'r') as f:
        settings = json.load(f)
except (FileNotFoundError, json.JSONDecodeError):
    settings = {}

if not isinstance(settings, dict):
    settings = {}

if 'hooks' not in settings:
    settings['hooks'] = {}

# Configure Stop hook
stop_hooks = settings['hooks'].get('Stop', [])
vibestats_stop_present = any(
    any(h.get('command') == 'vibestats sync' for h in matcher.get('hooks', []))
    for matcher in stop_hooks
)
if not vibestats_stop_present:
    stop_hooks.append({'hooks': [{'type': 'command', 'command': 'vibestats sync', 'async': True}]})
settings['hooks']['Stop'] = stop_hooks

# Configure SessionStart hook
session_hooks = settings['hooks'].get('SessionStart', [])
vibestats_session_present = any(
    any(h.get('command') == 'vibestats sync' for h in matcher.get('hooks', []))
    for matcher in session_hooks
)
if not vibestats_session_present:
    session_hooks.append({'hooks': [{'type': 'command', 'command': 'vibestats sync'}]})
settings['hooks']['SessionStart'] = session_hooks

with open(settings_path, 'w') as f:
    json.dump(settings, f, indent=2)
    f.write('\n')
PYEOF
  echo "Claude Code hooks configured in ~/.claude/settings.json"
}

# ---------------------------------------------------------------------------
# Step 13: Inject vibestats README markers into profile README (AC #2, FR9)
# Adds <!-- vibestats-start/end --> block with SVG embed and dashboard link.
# Graceful: prints warning (not error) when profile repo is inaccessible.
# Idempotent: skips if markers already present.
# Requires: GITHUB_USER (auto-detected if not set).
# ---------------------------------------------------------------------------
inject_readme_markers() {
  if [ -z "${GITHUB_USER:-}" ]; then
    USER_JSON=$(_gh api /user)
    GITHUB_USER=$(echo "$USER_JSON" | python3 -c "import sys, json; print(json.load(sys.stdin)['login'])")
    export GITHUB_USER
  fi

  # Single GET — parse content and SHA from same response (avoid race + extra API call)
  README_RESPONSE=$(_gh api "repos/${GITHUB_USER}/${GITHUB_USER}/contents/README.md" 2>/dev/null || echo "NOT_FOUND")
  if [ "$README_RESPONSE" = "NOT_FOUND" ]; then
    echo "Warning: Could not access ${GITHUB_USER}/${GITHUB_USER}/README.md. Please add vibestats markers manually. See https://vibestats.dev/docs/quickstart for instructions."
    return 0
  fi

  SHA=$(echo "$README_RESPONSE" | python3 -c "import sys, json; print(json.load(sys.stdin)['sha'])")
  ENCODED=$(echo "$README_RESPONSE" | python3 -c "import sys, json; print(json.load(sys.stdin)['content'])")
  README_CONTENT=$(echo "$ENCODED" | python3 -c "import sys, base64; print(base64.b64decode(sys.stdin.read().replace('\n','')).decode())")

  # Idempotency check
  if echo "$README_CONTENT" | grep -q '<!-- vibestats-start -->'; then
    echo "vibestats README markers already present — skipping."
    return 0
  fi

  MARKER_BLOCK="<!-- vibestats-start -->
[![vibestats](https://raw.githubusercontent.com/${GITHUB_USER}/${GITHUB_USER}/main/vibestats/heatmap.svg)](https://vibestats.dev/${GITHUB_USER})

[View interactive dashboard →](https://vibestats.dev/${GITHUB_USER})
<!-- vibestats-end -->"

  UPDATED_CONTENT="${README_CONTENT}
${MARKER_BLOCK}"

  ENCODED_NEW=$(printf '%s' "$UPDATED_CONTENT" | base64 | tr -d '\n')

  _gh api "repos/${GITHUB_USER}/${GITHUB_USER}/contents/README.md" \
    --method PUT \
    --field message="Add vibestats heatmap markers" \
    --field "content=${ENCODED_NEW}" \
    --field "sha=${SHA}"

  echo "vibestats markers added to ${GITHUB_USER}/${GITHUB_USER}/README.md"
}

# ---------------------------------------------------------------------------
# Step 14: Run post-install backfill (AC #3, FR11)
# Calls vibestats sync --backfill as the final step.
# Non-fatal: prints warning if binary exits non-zero, but installer exits 0.
# ---------------------------------------------------------------------------
run_backfill() {
  echo "Running post-install backfill (vibestats sync --backfill)..."
  if ! "${HOME}/.local/bin/vibestats" sync --backfill; then
    echo "Warning: Backfill completed with errors. Run 'vibestats sync --backfill' manually to retry."
  else
    echo "Backfill complete."
  fi
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

  # Step 6: Detect install mode (first-install or multi-machine)
  detect_install_mode

  # Step 7: Branch based on install mode
  case "$INSTALL_MODE" in
    multi-machine)
      register_machine
      ;;
    first-install)
      first_install_path
      ;;
  esac

  # Steps 12–14: shared final steps (always run)
  configure_hooks
  inject_readme_markers
  run_backfill

  echo "=== Installation complete! ==="
  echo "Run 'vibestats --help' to get started."
}

# Only run main when the script is executed directly (not sourced).
# This allows test files to source install.sh and call individual functions.
# $BASH_SOURCE is available in Bash 3.2+; equals $0 only when not sourced.
if [ "${BASH_SOURCE:-$0}" = "$0" ]; then
  main "$@"
fi
