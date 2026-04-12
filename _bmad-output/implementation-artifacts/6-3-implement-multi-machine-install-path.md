# Story 6.3: Implement multi-machine install path

Status: review

<!-- GH Issue: #33 | Epic: #6 | PR must include: Closes #33 -->

## Story

As a developer adding a second machine,
I want the installer to detect my existing vibestats-data repo and skip redundant setup steps,
So that adding a new machine is just as fast as the first install.

## Acceptance Criteria

1. **Given** `vibestats-data` already exists **When** the installer runs **Then** it detects the existing repo via `gh repo view`, skips repo creation, workflow write, and `VIBESTATS_TOKEN` secret setup (FR5)

2. **Given** the existing repo is detected **When** machine registration runs **Then** the new machine's `machine_id` and `hostname` are added to `registry.json` with `status = "active"` via Contents API PUT (FR6)

## Tasks / Subtasks

- [x] Task 1: Implement `detect_install_mode()` function (AC: #1)
  - [x] File: `install.sh` at repo root — add this function after the existing Step 5 functions from Story 6.1
  - [x] Obtain GitHub username via `_gh api /user` and parse login with python3 (no --jq dependency) and store in `GITHUB_USER` variable
  - [x] Run `if _gh repo view "${GITHUB_USER}/vibestats-data" --json name > /dev/null 2>&1; then` to check if repo exists — MUST use `if` construct (not `$?` check) because `set -euo pipefail` aborts on non-zero exit
  - [x] If repo exists: print "Existing vibestats-data repo detected. Running multi-machine setup." and set `INSTALL_MODE="multi-machine"`
  - [x] If repo does not exist: print "No vibestats-data repo found. Running first-install setup." and set `INSTALL_MODE="first-install"`
  - [x] Export both `INSTALL_MODE` and `GITHUB_USER`
  - [x] Do NOT implement Story 6.2 first-install logic (repo creation, `aggregate.yml` write, `VIBESTATS_TOKEN` generation) — only multi-machine path is in scope here
  - [x] Do NOT implement Story 6.4 steps (hooks, README markers, backfill)

- [x] Task 2: Implement `register_machine()` function for multi-machine path (AC: #2)
  - [x] Generate `machine_id` deterministically: `$(hostname)-$(cat /etc/machine-id 2>/dev/null || uuidgen | tr -d '-' | head -c 6)`
    - macOS: use `system_profiler SPHardwareDataType 2>/dev/null | awk '/Hardware UUID/{print $3}' | head -c 6` as the suffix (no `/etc/machine-id` on macOS)
    - Linux: use the first 6 chars of `/etc/machine-id`
    - Bash 3.2 safe: use `case "$(uname -s)"` to branch
    - Full `machine_id` format: `<hostname>-<6char-suffix>` (e.g., `stephens-mbp-a1b2c3`)
  - [x] Get `hostname` via `hostname` command
  - [x] Get current timestamp in ISO 8601 UTC format: `$(date -u '+%Y-%m-%dT%H:%M:%SZ')`
  - [x] Fetch existing `registry.json` via `_gh api "repos/${GITHUB_USER}/vibestats-data/contents/registry.json"` — decode base64 content
    - If file doesn't exist (404): start from `{"machines": []}`
    - Decode base64: `echo "$content" | base64 -d` (Linux) or `echo "$content" | base64 -D` (macOS) — use `case "$(uname -s)"` to branch
  - [x] Build the new machine JSON entry:
    ```json
    {"machine_id": "<id>", "hostname": "<hostname>", "status": "active", "last_seen": "<ISO8601Z>"}
    ```
  - [x] Append the new entry to the `machines` array — use Python stdlib (no `jq` required; `jq` is not a guaranteed dependency)
    - Safe pattern: use Python one-liner since Python is available on macOS and most Linux distros: `python3 -c "import sys,json; ..."`
    - Do NOT use `jq` — it is not installed by default and is not listed as a dependency
  - [x] Write updated `registry.json` back via `_gh api` Contents PUT (same pattern as GitHub Contents API updates):
    - GET SHA first: `_gh api "repos/${GITHUB_USER}/vibestats-data/contents/registry.json" --jq .sha`
    - PUT with base64-encoded content and SHA
  - [x] Write `machine_id` and `vibestats_data_repo` to `~/.config/vibestats/config.toml` using `gh auth token` for `oauth_token`, with `chmod 600` (NFR6)
  - [x] Print confirmation: "Machine registered: <machine_id> (<hostname>)"

- [x] Task 3: Wire `detect_install_mode()` and multi-machine path into `main()` (AC: #1, #2)
  - [x] After `download_and_install_binary`, call `detect_install_mode`
  - [x] Add a `case "$INSTALL_MODE" in` branch in `main()`:
    - `multi-machine`: call `register_machine`
    - `first-install`: print "First-install steps will be handled by the full installer (run with --first-install flag or see Story 6.2)" — placeholder stub only; do NOT implement 6.2 logic
  - [x] `main()` must still complete without error in both branches for this story's scope (the first-install stub path is acceptable as a no-op for testing 6.3 in isolation)

- [x] Task 4: Write shell tests for Story 6.3 using `bats-core` (AC: #1, #2)
  - [x] Test file: `tests/installer/test_6_3.bats`
  - [x] Follow the same patterns established in `tests/installer/test_6_1.bats`:
    - Source `install.sh` in `setup()` to import functions
    - Override `_gh()` as a shell function to mock `gh` CLI calls
    - Override `$HOME` to a temp dir in `setup()` to prevent real file mutations
    - Use `teardown()` to `rm -rf "$HOME"`
  - [x] P0 tests to implement (required for PR gate — these match R-004 and R-005 from the test design):
    - `multi-machine path: vibestats-data exists → repo creation skipped, workflow write skipped, VIBESTATS_TOKEN not set` (mock `_gh repo view` returning 0; spy that `_gh repo create` NOT called, `_gh secret set VIBESTATS_TOKEN` NOT called)
    - `registry.json entry has all required fields: machine_id, hostname, status=active, last_seen ISO 8601 UTC` (mock `_gh api` contents GET returning empty `{"machines":[]}` and PUT returning 200; parse the JSON written and assert all four fields)
  - [x] P1 tests to implement:
    - `vibestats-data repo detection uses correct repo name (username/vibestats-data) not a hardcoded org` (inspect `_gh repo view` call arguments; assert `${GITHUB_USER}/vibestats-data` pattern used)
    - `detect_install_mode sets INSTALL_MODE=multi-machine when repo exists` (mock `_gh repo view` returning 0; assert `INSTALL_MODE` equals `"multi-machine"`)
    - `detect_install_mode sets INSTALL_MODE=first-install when repo does not exist` (mock `_gh repo view` returning non-zero; assert `INSTALL_MODE` equals `"first-install"`)
    - `register_machine appends new entry without overwriting existing machines` (mock GET returning existing `{"machines":[{"machine_id":"old-machine","hostname":"old","status":"active","last_seen":"2026-01-01T00:00:00Z"}]}`; assert PUT body contains both old and new entries)
  - [x] Run tests: `bats tests/installer/test_6_3.bats` — all 7 tests pass with 0 failures

## Dev Notes

### File Location

Single file to modify: `install.sh` at repo root.

**Current state after Story 6.1:** `install.sh` has functions:
- `_gh()` helper (line 12–16)
- `install_gh_if_missing()` — Step 1
- `check_gh_version()` — Step 2
- `check_gh_auth()` — Step 3
- `detect_platform()` — Step 4
- `download_and_install_binary()` — Step 5
- `main()` — calls Steps 1–5 in sequence

**This story adds:**
- `detect_install_mode()` — Step 6
- `register_machine()` — Step 7 (multi-machine path only)
- Updated `main()` — calls Step 6, then branches on `$INSTALL_MODE`

Do NOT create or modify:
- Any files in `src/` (Rust binary)
- Any files in `action/` (Python Actions pipeline)
- `.github/workflows/`
- `~/.claude/settings.json` (Story 6.4)

### Critical Architecture Constraints

From `architecture.md` and the test design:

- **Bash 3.2 compatibility required** — no `declare -A`, no `mapfile`, no `[[ =~ ]]` regex captures, no `$(< file)` process substitution. Use `case`, `command -v`, `awk`, `sed`, `cut`.
- **`_gh()` helper is mandatory** — ALL `gh` calls must go through `_gh()` for test mockability (R-003, R-004 from test design). Never call `gh` directly.
- **`set -euo pipefail` + non-zero exit trap** — `set -e` causes any non-zero exit to abort the script. When checking `_gh repo view` to detect if the repo exists, you MUST use `|| true` or an `if` construct to prevent `set -e` from exiting when the repo does not exist:
  ```bash
  # ✅ Safe — non-zero exit from _gh is caught by if, not set -e
  if _gh repo view "${GITHUB_USER}/vibestats-data" --json name > /dev/null 2>&1; then
    INSTALL_MODE="multi-machine"
  else
    INSTALL_MODE="first-install"
  fi

  # ❌ Broken under set -euo pipefail — script aborts when repo does not exist
  _gh repo view "${GITHUB_USER}/vibestats-data" > /dev/null 2>&1
  if [ $? -eq 0 ]; then ...
  ```
  Similarly for `_gh api` 404 handling — use `|| true` or `if` construct.
- **No `jq` dependency** — not listed as a required dependency (only `gh` CLI is required). Use Python one-liner for JSON manipulation.
- **`BASH_SOURCE` guard** — `main()` is already guarded with `if [ "${BASH_SOURCE:-$0}" = "$0" ]`. Keep this guard intact so tests can source `install.sh` without running `main`.
- **`_gh` define-if-not-defined guard** — already in place at top of `install.sh`. Keep it — it's what allows bats tests to override `_gh`.

### GitHub API Patterns for registry.json

The multi-machine path uses the same GET-SHA + PUT pattern used throughout the system (see `architecture.md#API & Communication Patterns`):

```bash
# Step 1: Fetch existing registry.json (handle 404 = first-time, create empty)
REGISTRY_RESPONSE=$(_gh api "repos/${GITHUB_USER}/vibestats-data/contents/registry.json" 2>/dev/null || echo "NOT_FOUND")
if [ "$REGISTRY_RESPONSE" = "NOT_FOUND" ]; then
  CURRENT_JSON='{"machines": []}'
  SHA=""
else
  ENCODED=$(_gh api "repos/${GITHUB_USER}/vibestats-data/contents/registry.json" --jq .content)
  case "$(uname -s)" in
    Darwin) CURRENT_JSON=$(echo "$ENCODED" | base64 -D) ;;
    Linux)  CURRENT_JSON=$(echo "$ENCODED" | base64 -d) ;;
  esac
  SHA=$(_gh api "repos/${GITHUB_USER}/vibestats-data/contents/registry.json" --jq .sha)
fi

# Step 2: Build new JSON (Python 3, stdlib only)
NEW_JSON=$(python3 -c "
import sys, json, datetime
data = json.loads(sys.argv[1])
data['machines'].append({
  'machine_id': sys.argv[2],
  'hostname': sys.argv[3],
  'status': 'active',
  'last_seen': datetime.datetime.utcnow().strftime('%Y-%m-%dT%H:%M:%SZ')
})
print(json.dumps(data, indent=2))
" "$CURRENT_JSON" "$MACHINE_ID" "$HOSTNAME_VAL")

# Step 3: PUT back (base64 encode for Contents API)
case "$(uname -s)" in
  Darwin) ENCODED_NEW=$(echo "$NEW_JSON" | base64) ;;
  Linux)  ENCODED_NEW=$(echo "$NEW_JSON" | base64 -w 0) ;;
esac

if [ -n "$SHA" ]; then
  _gh api "repos/${GITHUB_USER}/vibestats-data/contents/registry.json" \
    --method PUT \
    --field message="chore: register machine ${MACHINE_ID}" \
    --field content="$ENCODED_NEW" \
    --field sha="$SHA"
else
  _gh api "repos/${GITHUB_USER}/vibestats-data/contents/registry.json" \
    --method PUT \
    --field message="chore: register machine ${MACHINE_ID}" \
    --field content="$ENCODED_NEW"
fi
```

**Important:** The above is a reference pattern — do NOT copy-paste verbatim without adapting to actual variable names in `install.sh`. It shows the exact flow: 404-handling, base64 decode by platform, Python JSON manipulation (stdlib only), base64 encode by platform, PUT with optional SHA.

### machine_id Generation

```bash
generate_machine_id() {
  HOSTNAME_VAL="$(hostname)"
  case "$(uname -s)" in
    Darwin)
      # macOS: use Hardware UUID suffix (no /etc/machine-id)
      SUFFIX=$(system_profiler SPHardwareDataType 2>/dev/null \
        | awk '/Hardware UUID/{print $3}' \
        | tr -d '-' \
        | cut -c1-6 \
        | tr '[:upper:]' '[:lower:]')
      ;;
    Linux)
      # Linux: use first 6 chars of /etc/machine-id
      SUFFIX=$(cut -c1-6 /etc/machine-id 2>/dev/null || echo "$(uuidgen | tr -d '-' | cut -c1-6 | tr '[:upper:]' '[:lower:]')")
      ;;
    *)
      SUFFIX="$(date +%s | cut -c-6)"
      ;;
  esac
  MACHINE_ID="${HOSTNAME_VAL}-${SUFFIX}"
  echo "$MACHINE_ID"
}
```

The `machine_id` is then stored in `~/.config/vibestats/config.toml` (written by this story for multi-machine path).

### config.toml Write (NFR6: 600 permissions)

```bash
write_config_toml() {
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
}
```

`chmod 600` must be called immediately after writing — never before. The token must not be echoed to stdout or stored in a shell variable longer than necessary.

### registry.json Schema (from docs/schemas.md and Story 1.4)

```json
{
  "machines": [
    {
      "machine_id": "stephens-mbp-a1b2c3",
      "hostname": "Stephens-MacBook-Pro.local",
      "status": "active",
      "last_seen": "2026-04-10T14:23:00Z"
    }
  ]
}
```

Field requirements (all four required — R-005):
- `machine_id` (string): non-empty, matches `config.toml` and Hive path format
- `hostname` (string): non-empty, from `hostname` command
- `status` (string): must be exactly `"active"` on first registration
- `last_seen` (string): ISO 8601 UTC, format `YYYY-MM-DDTHH:MM:SSZ`

**Multi-machine append rule:** New entry is APPENDED to the `machines` array — existing entries must NOT be overwritten or removed (R-005 mitigation, test case: "append without overwrite").

### Shell Compatibility Reference

```bash
# ✅ Bash 3.2 safe
command -v python3
case "$(uname -s)" in Darwin) ... ;; Linux) ... ;; esac
[ -n "$VAR" ]
awk '{print $3}'
cut -d. -f1
base64 -d   # Linux
base64 -D   # macOS

# ❌ Requires Bash 4+ — DO NOT use
declare -A map
mapfile -t arr
[[ "$str" =~ ^pattern(.*)$ ]]
```

### Test Framework Reference (from Story 6.1 patterns)

`bats-core` is installed via npm: `npm install --save-dev bats`. Tests in `tests/installer/`.

Key patterns from `test_6_1.bats`:
1. Define `_gh()` override BEFORE sourcing `install.sh` — the define-if-not-defined guard ensures the test stub is not overwritten
2. Use `run <function_name>` to capture exit code and output
3. `[ "$status" -eq 0 ]` and `[[ "$output" == *"substring"* ]]` for assertions
4. Override `$HOME` in `setup()` to a `mktemp -d` temp directory

For spying (asserting a `gh` subcommand was/was not called), record calls in a temp file:

```bash
_gh() {
  echo "_gh $*" >> "$BATS_TMPDIR/gh_calls.log"
  case "$1 $2" in
    "repo view") return 0 ;;  # simulate repo exists
    "repo create") return 1 ;; # should not be called
    *) return 0 ;;
  esac
}
export -f _gh
```

Then assert:
```bash
run grep "repo create" "$BATS_TMPDIR/gh_calls.log"
[ "$status" -ne 0 ]  # gh repo create was NOT called
```

### Security Requirements (from R-001, R-002, NFR6, NFR7)

- `VIBESTATS_TOKEN` is NOT generated in this story (only in Story 6.2 first-install path)
- Multi-machine path must NOT call `gh secret set VIBESTATS_TOKEN` — this is the R-004 risk to mitigate
- `oauth_token` (machine-side token from `gh auth token`) IS written to `config.toml` in this story
- `config.toml` must be created with `chmod 600` immediately (NFR6)
- Token from `gh auth token` must not be echoed to stdout or written to any file other than `config.toml`

### Cross-Story Dependencies

- **Depends on Story 6.1**: `_gh()` helper, `install_gh_if_missing()`, `check_gh_version()`, `check_gh_auth()`, `download_and_install_binary()` are all in place and must not be modified
- **Does NOT implement Story 6.2**: repo creation, `aggregate.yml` write, `VIBESTATS_TOKEN` generation/secret-set are all Story 6.2 scope
- **Does NOT implement Story 6.4**: `~/.claude/settings.json` hooks, README markers, backfill trigger are all Story 6.4 scope
- Stories 6.2 and 6.3 can run in parallel (per dependency-graph.md update after 6.1 merged)
- Story 6.4 depends on both 6.2 and 6.3 being complete

### References

- Story 6.1 implementation and patterns: [Source: _bmad-output/implementation-artifacts/6-1-implement-dependency-detection-and-gh-authentication.md]
- Epic 6 acceptance criteria and FR5/FR6: [Source: _bmad-output/planning-artifacts/epics.md#Epic 6, Story 6.3]
- Architecture bash installer section: [Source: _bmad-output/planning-artifacts/architecture.md#Starter Template Evaluation — Bash Installer]
- Authentication & Security patterns: [Source: _bmad-output/planning-artifacts/architecture.md#Authentication & Security]
- API & Communication Patterns (GET SHA + PUT): [Source: _bmad-output/planning-artifacts/architecture.md#API & Communication Patterns]
- registry.json schema: [Source: _bmad-output/implementation-artifacts/1-4-define-and-document-all-json-and-toml-schemas.md#registry.json]
- config.toml schema: [Source: _bmad-output/implementation-artifacts/1-4-define-and-document-all-json-and-toml-schemas.md#config.toml]
- Epic 6 test design (R-001 through R-005): [Source: _bmad-output/test-artifacts/test-design-epic-6.md]
- PR5, FR6 requirements: [Source: _bmad-output/planning-artifacts/prd.md#Functional Requirements]
- NFR6 (600 file perms), NFR7 (token scope): [Source: _bmad-output/planning-artifacts/prd.md#Security]

### PR Checklist

- [ ] `install.sh` passes `bash -n install.sh` (syntax check)
- [ ] `bats tests/installer/test_6_3.bats` — all tests pass with 0 failures
- [ ] `set -euo pipefail` still present on line 2 of `install.sh`
- [ ] No hardcoded usernames — always use `$GITHUB_USER` (obtained via `_gh api /user --jq .login`)
- [ ] All `gh` calls go through `_gh()` helper — no direct `gh` invocations
- [ ] `config.toml` written with `chmod 600` immediately after creation
- [ ] `VIBESTATS_TOKEN` secret setup NOT present (this is multi-machine only — skip that step)
- [ ] `registry.json` entry contains all four required fields with correct types
- [ ] Append-only: existing machines in `registry.json` preserved

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

None — implementation proceeded without blockers.

### Completion Notes List

- Implemented `detect_install_mode()` (Step 6): fetches GitHub username via `_gh api /user` and parses JSON with python3 (avoids `--jq` dependency issue in test mocks), then uses `if` construct with `_gh repo view` to set `INSTALL_MODE` to `"multi-machine"` or `"first-install"`.
- Implemented `register_machine()` (Step 7): generates `machine_id` as `<hostname>-<6char-suffix>` using platform-specific branch (`case "$(uname -s)"`), fetches and appends to `registry.json` via GitHub Contents API GET+PUT pattern, builds JSON with python3 stdlib, base64-encodes by platform, writes `config.toml` with `chmod 600`.
- Updated `main()` to call `detect_install_mode` after `download_and_install_binary`, then branch on `$INSTALL_MODE` with `case` statement.
- Removed `skip` statements from all 7 tests in `test_6_3.bats`; all 7 tests pass GREEN.
- All 10 regression tests in `test_6_1.bats` continue to pass.
- Key decision: used `_gh api /user` (without `--jq`) and parsed the JSON output with python3 inline — this ensures the test mocks (which return full JSON regardless of `--jq`) work correctly while real `gh` calls also work.
- `set -euo pipefail` and `BASH_SOURCE` guard preserved intact.
- No `jq` dependency introduced.
- `VIBESTATS_TOKEN` secret setup NOT present in multi-machine path (correct per AC #1, R-004).

### File List

- `install.sh` (modified — added `detect_install_mode()`, `register_machine()`, updated `main()`)
- `tests/installer/test_6_3.bats` (modified — removed skip statements, tests now GREEN)

## Change Log

- 2026-04-12: Story 6.3 implemented — `detect_install_mode()` and `register_machine()` added to `install.sh`; `main()` updated to branch on `$INSTALL_MODE`; all 7 bats tests GREEN; status set to review.
