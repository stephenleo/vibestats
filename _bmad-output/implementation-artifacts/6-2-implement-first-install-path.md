# Story 6.2: Implement first-install path

Status: review

<!-- GH Issue: #32 | Epic: #6 | PR must include: Closes #32 -->

## Story

As a first-time vibestats user,
I want the installer to create my vibestats-data repo, write the workflow, and configure my tokens in one pass,
So that I can go from zero to a live heatmap without any manual GitHub steps.

## Acceptance Criteria

1. **Given** `vibestats-data` does not exist under the user's account **When** the first-install path runs **Then** it creates `username/vibestats-data` as a private repo via `gh repo create` (FR4)

2. **Given** the repo is created **When** the workflow setup runs **Then** it writes `aggregate.yml` into `vibestats-data/.github/workflows/` calling `stephenleo/vibestats@v1` (FR7)

3. **Given** the workflow is written **When** the token setup runs **Then** it generates `VIBESTATS_TOKEN` via `gh api /user/personal_access_tokens` (fine-grained PAT, `username/username` Contents write only), sets it as the `VIBESTATS_TOKEN` Actions secret, and the token is **never written to disk** (FR10, NFR7)

4. **Given** the local token is obtained via `gh auth token` **When** it is stored **Then** it is written to `~/.config/vibestats/config.toml` with permissions `600` (FR39, NFR6)

5. **Given** all first-install setup steps complete **When** `vibestats-data/registry.json` is read **Then** it contains one entry for the current machine with `machine_id`, `hostname`, `status = "active"`, and `last_seen` ISO 8601 UTC timestamp set to the time of install (FR6)

## Tasks / Subtasks

- [x] Task 1: Add `detect_first_install()` function to `install.sh` (AC: #1)
  - [x] File: `install.sh` at repo root — add new function after existing Step 5 (`download_and_install_binary`)
  - [x] Obtain GitHub username via `_gh api /user --jq '.login'` — **never hardcode usernames**
  - [x] Export the username: `export USERNAME` — so subsequent functions can access it without re-calling the API
  - [x] Check if repo exists: `_gh repo view "${USERNAME}/vibestats-data" > /dev/null 2>&1`
  - [x] Return 0 if repo does NOT exist (first-install path — used with `if detect_first_install; then`), return 1 if it DOES exist (multi-machine path)
  - [x] Note on shell convention: shell functions return 0 for "success/true" in `if` statements. Here 0 means "first install IS needed" — this is intentional

- [x] Task 2: Implement `create_vibestats_data_repo()` function (AC: #1)
  - [x] Call `_gh repo create "${USERNAME}/vibestats-data" --private` — use the `--private` flag (FR4)
  - [x] Exit non-zero with clear error message if `gh repo create` fails (set -euo pipefail already handles this, but add explicit error message via `|| { echo "Error: ..."; exit 1; }`)
  - [x] Print progress: `"Creating vibestats-data repository..."`

- [x] Task 3: Implement `write_workflow()` function (AC: #2)
  - [x] **IMPORTANT**: `install.sh` is run via `curl -sSf https://vibestats.dev/install.sh | bash` — workflow content embedded as heredoc, NOT read from disk
  - [x] Embed the `aggregate.yml` content as a heredoc in `generate_aggregate_workflow_content()` helper — content includes `stephenleo/vibestats@v1`
  - [x] Base64-encode the heredoc content — Bash 3.2 safe approach using `printf '%s' ... | base64 | tr -d '\n'`
  - [x] `tr -d '\n'` applied — macOS `base64` wrap removed for GitHub API compatibility
  - [x] Write file via `gh api` Contents PUT
  - [x] Print progress: `"Writing aggregate workflow to vibestats-data..."`

- [x] Task 4: Implement `setup_vibestats_token()` function (AC: #3, R-001)
  - [x] **CRITICAL SECURITY REQUIREMENT**: `VIBESTATS_TOKEN` must NEVER be written to disk or echoed to stdout
  - [x] Generate fine-grained PAT and pipe directly to `gh secret set` — token never stored in variable written to disk
  - [x] Fallback: if fine-grained PAT API returns non-zero, fall back to `gh auth token` with printed warning
  - [x] Print progress: `"Setting up VIBESTATS_TOKEN Actions secret..."`

- [x] Task 5: Implement `store_machine_token()` function (AC: #4, R-002)
  - [x] Compute `MACHINE_ID` using `cksum`-based approach — computed and exported for reuse by `register_machine()`
  - [x] Obtain machine-side token via `LOCAL_TOKEN=$(_gh auth token)`
  - [x] Create config directory: `mkdir -p "${HOME}/.config/vibestats"`
  - [x] Write `~/.config/vibestats/config.toml` with oauth_token, machine_id, vibestats_data_repo
  - [x] Set permissions immediately: `chmod 600 "${HOME}/.config/vibestats/config.toml"` (NFR6)
  - [x] Unset the token variable after writing: `unset LOCAL_TOKEN`
  - [x] Print progress: `"Storing machine token in ~/.config/vibestats/config.toml..."`

- [x] Task 6: Implement `register_machine()` function (AC: #5, R-005)
  - [x] Reuses `MACHINE_ID` exported by `store_machine_token()`, or self-computes if called directly
  - [x] Build `registry.json` with machine_id, hostname, status="active", last_seen ISO 8601 UTC
  - [x] Base64-encode with line-wrap removal
  - [x] Write `registry.json` to `vibestats-data` via Contents API PUT
  - [x] Print progress: `"Registering machine in vibestats-data/registry.json..."`

- [x] Task 7: Wire first-install path into `main()` (AC: all)
  - [x] After `download_and_install_binary` in `main()`: `if detect_first_install; then create_vibestats_data_repo; write_aggregate_workflow; setup_vibestats_token; fi`
  - [x] `store_machine_token` and `register_machine` run on BOTH paths
  - [x] Implemented `first_install_path()` convenience function (combines all first-install steps including store/register)
  - [x] Story 6.3 logic NOT implemented here

- [x] Task 8: Write bats tests for Story 6.2 (AC: #1–#5, R-001, R-002, R-005)
  - [x] Test file: `tests/installer/test_6_2.bats` (pre-existing from ATDD phase — 16 tests)
  - [x] All 16 bats tests pass: `bats tests/installer/test_6_2.bats` — 16/16 ok
  - [x] All 10 story 6.1 regression tests pass: `bats tests/installer/test_6_1.bats` — 10/10 ok

## Dev Notes

### File Location

Single file to modify: `install.sh` at repo root.

**Add new functions AFTER** the existing `download_and_install_binary()` function. Do NOT modify existing Story 6.1 functions (`install_gh_if_missing`, `check_gh_version`, `check_gh_auth`, `detect_platform`, `download_and_install_binary`).

Do NOT create or modify:
- Any files in `src/` (Rust binary)
- `.github/workflows/` files
- `~/.claude/settings.json` (Story 6.4's responsibility)
- `tests/installer/test_6_1.bats` (Story 6.1's test file — do not touch)

### Critical Pattern from Story 6.1 — The `_gh()` Helper

ALL GitHub CLI calls MUST go through `_gh()` (already defined in `install.sh`). This is required for bats-core testability:

```bash
# CORRECT — testable
_gh api /user --jq .login
_gh repo create "${USERNAME}/vibestats-data" --private

# WRONG — NOT testable
gh api /user --jq .login
```

The `_gh()` override guard is already in place:
```bash
if ! declare -f _gh > /dev/null 2>&1; then
  _gh() { gh "$@"; }
fi
```

Tests pre-define their own `_gh()` before sourcing `install.sh`, so the guard prevents overwriting.

### Bash 3.2 Compatibility (macOS Default Shell)

```bash
# SAFE — Bash 3.2 compatible
case "$(uname -s)" in Darwin) ... ;; Linux) ... ;; esac
[ "$status" -eq 0 ]
command -v gh

# FORBIDDEN — Bash 4+ only
declare -A mymap       # associative arrays
mapfile -t arr < file
[[ "$str" =~ ^pattern ]]  # regex capture groups
```

### GitHub Username Detection

**Always dynamically detect — never hardcode:**

```bash
USERNAME=$(_gh api /user --jq '.login')
export USERNAME
```

Call this once in `detect_first_install()` and export it. Do not call `_gh api /user` again in subsequent functions — reuse `$USERNAME`.

The `--jq` flag is supported by `gh` CLI (wraps `jq` syntax). The `.login` jq expression extracts the `login` field from the `/user` endpoint response.

### Machine ID Generation

`machine_id` must be **deterministic** (same machine always produces the same ID). Follow the pattern from `docs/schemas.md` (e.g., `"stephens-mbp-a1b2c3"`):

```bash
# Recommended cross-platform pattern (Bash 3.2 safe)
HOSTNAME_LOWER=$(hostname | tr '[:upper:]' '[:lower:]' | sed 's/\..*$//' | sed 's/[^a-z0-9-]/-/g')
# cksum is POSIX — available on both macOS and Linux (unlike md5sum/md5)
HASH=$(hostname | cksum | awk '{print $1}' | cut -c1-6)
MACHINE_ID="${HOSTNAME_LOWER}-${HASH}"
export MACHINE_ID
```

Cross-platform hash notes:
- `md5sum` — Linux only (not available on macOS)
- `md5 -q` — macOS only (not available on Linux)
- `openssl dgst -md5` — requires OpenSSL, not always present
- `cksum` — POSIX standard, available on all targets (macOS arm64, macOS x86_64, Linux x86_64)

`MACHINE_ID` must be computed **once** and used in both `store_machine_token()` (write to config.toml) and `register_machine()` (write to registry.json). Export it so both functions can access the same value.

### config.toml Schema (from `docs/schemas.md`)

```toml
oauth_token = "gho_xxxxxxxxxxxxxxxxxxxx"
machine_id = "stephens-mbp-a1b2c3"
vibestats_data_repo = "stephenleo/vibestats-data"
```

File location: `~/.config/vibestats/config.toml`
Required permissions: `600` (owner read/write only — NFR6)

### registry.json Schema (from `docs/schemas.md`)

This is the authoritative schema — match exactly:

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

- All JSON field names are `snake_case` (not `camelCase`)
- `last_seen` format: `YYYY-MM-DDTHH:MM:SSZ` (ISO 8601 UTC, not Unix timestamp)
- `status` must be the string `"active"` on first install

### aggregate.yml Template

The template is already committed at `.github/workflows/aggregate.yml` in the repo root. However, since `install.sh` is executed via `curl | bash`, the template file is NOT on disk during user installation. The workflow content must be embedded as a heredoc in `install.sh`.

The exact content to embed (from the committed file):
```yaml
# aggregate.yml — Copy this file to your vibestats-data/.github/workflows/ directory.
name: Aggregate vibestats data
on:
  schedule:
    - cron: '0 2 * * *'
  workflow_dispatch:
jobs:
  aggregate:
    runs-on: ubuntu-latest
    steps:
      - uses: stephenleo/vibestats@v1
        with:
          token: ${{ secrets.VIBESTATS_TOKEN }}
          profile-repo: ${{ github.repository_owner }}/${{ github.repository_owner }}
```

### GitHub Contents API PUT (Writing Files)

Pattern for writing a file to a GitHub repo via `gh api`:

```bash
# Base64-encode content — ALWAYS pipe through tr -d '\n'
# macOS base64 wraps at 76 chars; GitHub API requires unwrapped base64
CONTENT=$(printf '%s' "$CONTENT_STRING" | base64 | tr -d '\n')

# PUT the file via gh api
_gh api "repos/${USERNAME}/vibestats-data/contents/.github/workflows/aggregate.yml" \
  --method PUT \
  --field message="Add vibestats aggregate workflow" \
  --field "content=${CONTENT}"
```

Note: No SHA field needed for new files. For updates (Story 6.3's registry.json append), you first GET the SHA (`_gh api repos/${USERNAME}/vibestats-data/contents/registry.json --jq '.sha'`), then PUT with `--field sha="$SHA_VALUE"`.

### VIBESTATS_TOKEN Security Pattern (R-001, NFR7)

**Must never touch disk.** The architecture mandates piping from `gh api` directly to `gh secret set`:

```bash
# CORRECT — token never hits disk
_gh api /user/personal_access_tokens \
  --method POST \
  --field name="vibestats-$(date +%Y)" \
  --field expiration="never" \
  --field repositories='["'"${USERNAME}"'"]' \
  --field permissions='{"contents":"write"}' \
  --jq '.token' \
  | _gh secret set VIBESTATS_TOKEN --repo "${USERNAME}/vibestats-data"
```

Pipe approach is preferred over the two-step variable approach. If the piped approach is used, no variable assignment to disk risk.

Fallback if enterprise blocks fine-grained PAT API:
```bash
echo "Warning: Fine-grained PAT creation blocked. Using gh auth token as VIBESTATS_TOKEN fallback."
_gh auth token | _gh secret set VIBESTATS_TOKEN --repo "${USERNAME}/vibestats-data"
```

### Error Handling

`set -euo pipefail` is already active from Story 6.1. Additionally add explicit messages for each step:

```bash
create_vibestats_data_repo() {
  echo "Creating vibestats-data repository..."
  _gh repo create "${USERNAME}/vibestats-data" --private \
    || { echo "Error: Failed to create vibestats-data repository." >&2; exit 1; }
  echo "Repository created: ${USERNAME}/vibestats-data"
}
```

### Test Framework Patterns (from Story 6.1)

Copy the exact bats setup/teardown pattern from `tests/installer/test_6_1.bats`:

```bash
#!/usr/bin/env bats

INSTALL_SH="$(cd "$(dirname "$BATS_TEST_FILENAME")/../.." && pwd)/install.sh"

setup() {
  export HOME
  HOME="$(mktemp -d)"
  export BATS_TMPDIR="${HOME}/bats-tmp"
  mkdir -p "$BATS_TMPDIR"
}

teardown() {
  rm -rf "$HOME"
}
```

Mock `_gh()` using the same pre-define-before-source pattern:

```bash
@test "[P0] config.toml has permissions 600 after store_machine_token" {
  cat > "${HOME}/stub_env.sh" <<'STUB'
_gh() {
  case "$1 $2" in
    "auth token") echo "gho_testtoken123" ;;
    "api /user")  echo "testuser" ;;
    *)            return 0 ;;
  esac
}
export -f _gh
STUB

  run bash --noprofile --norc -c "
    source '${HOME}/stub_env.sh'
    USERNAME=testuser
    source '${INSTALL_SH}'
    MACHINE_ID=test-machine-abc123
    store_machine_token
  " 2>&1

  [ "$status" -eq 0 ]
  [ -f "${HOME}/.config/vibestats/config.toml" ]
  PERMS=$(stat -f "%OLp" "${HOME}/.config/vibestats/config.toml" 2>/dev/null || \
          stat -c "%a" "${HOME}/.config/vibestats/config.toml")
  [ "$PERMS" = "600" ]
}
```

For R-001 (VIBESTATS_TOKEN never on disk), scan temp dirs and `$HOME` after the function runs:
```bash
# Assert token not in any file
! grep -r "test-secret-token" "${HOME}" 2>/dev/null
```

### Cross-Story Dependencies

This story adds to `install.sh`; Story 6.3 will extend it further:

- **Story 6.3** adds the multi-machine detection branch — `detect_first_install()` returning 1 triggers 6.3's path
- **Story 6.4** adds hook configuration, README markers, and backfill — they run after all steps in this story
- Do NOT implement Stories 6.3 or 6.4 logic in this PR

### Security Requirements Summary

| NFR | Requirement | How |
|-----|-------------|-----|
| NFR6 | `config.toml` permissions `600` | `chmod 600` immediately after `cat > config.toml` |
| NFR7 | `VIBESTATS_TOKEN` never on disk | Pipe `gh api` output directly to `gh secret set` |
| NFR7 | Token scope minimised | PAT scoped to `username/username` Contents write only |

### PR Checklist

- [ ] `bash -n install.sh` passes (syntax check)
- [ ] `bats tests/installer/test_6_2.bats` — all tests pass
- [ ] `bats tests/installer/test_6_1.bats` — still passes (no regressions)
- [ ] `VIBESTATS_TOKEN` is never assigned to a shell variable that gets written to disk
- [ ] `config.toml` permissions are set to `600` (verified by test)
- [ ] `registry.json` contains all four required fields: `machine_id`, `hostname`, `status`, `last_seen`
- [ ] `aggregate.yml` content reads from `.github/workflows/aggregate.yml` — not hardcoded inline
- [ ] No hardcoded GitHub usernames — all use `$(_gh api /user --jq .login)`

## Dev Agent Record

### Agent Actions Log

| Date | Action | Notes |
|------|--------|-------|
| 2026-04-12 | Story created | BMad Create-Story |
| 2026-04-12 | Implementation complete | All 8 tasks done, 16/16 bats tests pass, 10/10 regression tests pass |

### Completion Notes

Implemented all 8 tasks for Story 6.2 first-install path in `install.sh`:

- `detect_first_install()`: Gets USERNAME via `_gh api /user --jq '.login'`, checks if vibestats-data repo exists, returns 0 for first-install, 1 for multi-machine
- `create_vibestats_data_repo()`: Creates private repo via `_gh repo create --private` with error handling
- `generate_aggregate_workflow_content()`: Helper that echoes workflow YAML (heredoc) for testability
- `write_aggregate_workflow()`: Encodes workflow via base64+tr, writes to repo via Contents API PUT
- `setup_vibestats_token()`: Pipes fine-grained PAT directly to `gh secret set` (never touches disk); fallback to `gh auth token` if enterprise blocks PAT creation
- `store_machine_token()`: Computes deterministic MACHINE_ID via cksum, writes config.toml with chmod 600
- `register_machine()`: Builds registry.json with machine_id/hostname/status/last_seen, writes via Contents API
- `first_install_path()`: Convenience integration function combining all first-install steps
- `main()` updated: calls `detect_first_install`, conditionally calls first-install functions, always calls store/register

All functions auto-detect USERNAME if not already exported (enables independent testing).
Security: VIBESTATS_TOKEN never written to disk — piped directly from gh api to gh secret set.

### Change Log

| Date | Change | Author |
|------|--------|--------|
| 2026-04-12 | Story created | BMad Create-Story |
| 2026-04-12 | Implemented first-install path functions in install.sh | Dev Agent |

## File List

- `install.sh` (modified — add first-install path functions)
- `tests/installer/test_6_2.bats` (new — bats tests for Story 6.2)
