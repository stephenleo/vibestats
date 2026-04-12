# Story 6.1: Implement dependency detection and gh authentication

Status: done

<!-- GH Issue: #31 | Epic: #6 | PR must include: Closes #31 -->

## Story

As a new vibestats user,
I want the installer to handle all dependency and authentication checks automatically,
So that I never need to manually install gh or run auth flows.

## Acceptance Criteria

1. **Given** `gh` is not installed **When** `install.sh` runs **Then** it installs `gh` via `brew install gh` on macOS or `apt-get install gh` on Debian/Ubuntu (FR2)

2. **Given** `gh` installed version is below 2.0 **When** the version check runs **Then** the installer prints a warning and exits with a clear error message (NFR16)

3. **Given** `gh` is installed but the user is not authenticated **When** the auth check runs **Then** the installer runs `gh auth login` via the standard browser flow (FR3)

4. **Given** `uname -s` + `uname -m` identify the platform **When** the binary download step runs **Then** it downloads the correct `vibestats-<target>.tar.gz` from the latest GitHub Release and verifies the checksum before installing to `~/.local/bin/vibestats`

## Tasks / Subtasks

- [x] Task 1: Replace stub with real `install.sh` skeleton (AC: all)
  - [x] File: `install.sh` at repo root — replace the current stub (`echo "TODO: implement installer" && exit 1`)
  - [x] Keep `#!/usr/bin/env bash` shebang and `set -euo pipefail` — these are mandated by the architecture (see Dev Notes)
  - [x] Structure file following the `rustup` installer pattern: detect OS/arch → check/install `gh` → check/authenticate → download binary → verify checksum → install
  - [x] Add a `main()` function that calls each step function in sequence and a `main "$@"` at the bottom
  - [x] Do NOT implement Stories 6.2–6.4 steps (repo creation, hooks, README markers, backfill) — this story covers only dependency detection, `gh` auth, and binary download/install

- [x] Task 2: Implement `gh` detection and installation (AC: #1)
  - [x] Detect `gh` via `command -v gh` (POSIX-safe, works on Bash 3.2)
  - [x] Detect OS via `uname -s`: `Darwin` → macOS, `Linux` → Linux
  - [x] macOS install: `brew install gh` — require Homebrew; print error if `brew` not found
  - [x] Linux install: `apt-get install -y gh` — use `sudo` if not root; Debian/Ubuntu only
  - [x] After install, verify `gh` is now on `$PATH`; exit non-zero with clear message if install failed
  - [x] Do NOT use `which` — use `command -v` for POSIX portability

- [x] Task 3: Implement `gh` version check (AC: #2)
  - [x] Run `gh --version` and extract semantic version number (e.g., `2.44.1`) using `sed`/`awk` (no Python)
  - [x] Compare major version: if `< 2`, print error message including the found version and minimum required (`2.0`), then `exit 1`
  - [x] Message format: `"Error: gh version X.Y.Z is below minimum required version 2.0. Please upgrade: brew upgrade gh"`
  - [x] Version extraction must work for output format: `gh version 2.44.1 (2024-11-12)` (standard `gh --version` output)

- [x] Task 4: Implement `gh` authentication check (AC: #3)
  - [x] Check auth status via `gh auth status` (exits non-zero if not authenticated)
  - [x] If not authenticated, run `gh auth login` — this triggers the browser OAuth flow (FR3)
  - [x] After `gh auth login`, re-check `gh auth status`; exit non-zero with message if still not authenticated
  - [x] Do NOT use `gh auth token` here — that is for the machine-side token in Story 6.2

- [x] Task 5: Implement platform detection and binary download (AC: #4)
  - [x] Detect platform: `uname -s` for OS, `uname -m` for arch
  - [x] Map to release target:
    - `Darwin` + `arm64` → `aarch64-apple-darwin`
    - `Darwin` + `x86_64` → `x86_64-apple-darwin`
    - `Linux` + `x86_64` → `x86_64-unknown-linux-gnu`
    - Anything else: print "Unsupported platform: $(uname -s) $(uname -m)" and exit 1
  - [x] Download URL pattern: `https://github.com/stephenleo/vibestats/releases/latest/download/vibestats-<target>.tar.gz`
  - [x] Download via `curl -fsSL` into a temp directory (`mktemp -d`)
  - [x] Verify SHA256 checksum: download `.sha256` sidecar file from the same release URL, then run `sha256sum -c` (Linux) or `shasum -a 256 -c` (macOS)
  - [x] Extract: `tar xzf vibestats-<target>.tar.gz` in temp dir
  - [x] Install: `install -m 755 vibestats ~/.local/bin/vibestats` — create `~/.local/bin/` if it does not exist (`mkdir -p`)
  - [x] Clean up temp dir on exit using a `trap` (both success and failure)
  - [x] Verify install: run `vibestats --version` after install; exit non-zero if not found on `$PATH`

- [x] Task 6: Write shell tests for Story 6.1 using `bats-core` (AC: #1, #2, #3, #4)
  - [x] Test file: `tests/installer/test_6_1.bats` (create `tests/installer/` directory if absent)
  - [x] Use `bats-core` test framework — see Dev Notes for setup
  - [x] Mock `gh` via overriding the `_gh()` helper function (defined in `install.sh` as a wrapper around `gh`). Override `_gh()` in tests — do NOT try to mock the `gh` binary directly.
  - [x] Override `$HOME` to a temp dir in each test's `setup()` to prevent real file mutations
  - [x] P1 tests to implement (minimum required for this story's PR gate):
    - `gh not installed → brew install gh called on Darwin` (stub `command -v gh` to fail, `uname -s` returns `Darwin`)
    - `gh not installed → apt-get install gh called on Linux` (stub `command -v gh` to fail, `uname -s` returns `Linux`)
    - `gh version < 2.0 → exits non-zero with message` (mock `gh --version` returning `gh version 1.14.0 (2022-01-01)`)
    - `gh not authenticated → gh auth login called` (mock `gh auth status` exiting non-zero)
    - `platform Darwin arm64 → correct target selected` (stub `uname` outputs, assert download URL)
    - `platform Darwin x86_64 → correct target selected`
    - `platform Linux x86_64 → correct target selected`
    - `unsupported platform → exits non-zero with message`
  - [x] P2 test (idempotency/happy path):
    - `gh installed and version ≥ 2.0 → no install attempted` (mock `command -v gh` returning a path and `_gh --version` returning `gh version 2.44.1 (2024-11-12)`; assert no brew/apt-get call made)
  - [x] Run tests: `bats tests/installer/test_6_1.bats` — must pass with 0 failures before PR

## Dev Notes

### File Location

Single file to implement: `install.sh` at repo root.

**Current state:** `install.sh` is a stub (`echo "TODO: implement installer" && exit 1`). Replace the entire body — keep the shebang and `set -euo pipefail`.

Do NOT create or modify:
- Any files in `src/` (Rust binary — separate concerns)
- Any files in `action/` (Python Actions pipeline)
- `.github/workflows/` (CI/CD)
- `~/.claude/settings.json` hook configuration (Story 6.4)

### Architecture Mandates

From `architecture.md`:

- **Bash 3.2 compatibility required**: must work on macOS default shell. Avoid `[[ ]]` with regex captures requiring Bash 4+, `declare -A` (associative arrays), or `mapfile`. Use `case` statements and `command -v` instead of `which`.
- **`gh` CLI is the sole mechanism for all GitHub operations** — no direct `curl` calls to GitHub API (API calls belong in Stories 6.2–6.4)
- **`set -euo pipefail` at top** — mandatory, guards against silent failure continuation (R-003 in test design)
- **Pattern:** Follow `rustup` installer pattern — each logical step in its own function

### Shell Compatibility

```bash
# ✅ POSIX/Bash 3.2 safe
command -v gh
case "$(uname -s)" in Darwin) ... ;; Linux) ... ;; esac
[ "$major" -lt 2 ]

# ❌ Requires Bash 4+ — DO NOT use
[[ "$version" =~ ^([0-9]+) ]]  # regex capture group
declare -A platform_map
mapfile -t arr < file
```

### `gh` CLI Patterns and Testability Helper

**Critical for testability (from `test-design-epic-6.md`):** Wrap all `gh` calls in a `_gh()` helper function within `install.sh`. This makes the `gh` CLI trivially overridable in `bats-core` tests without complex binary stubbing.

```bash
# Wrap ALL gh calls through this helper — never call gh directly
_gh() {
  gh "$@"
}

# Usage in functions:
_gh auth status
_gh repo view username/vibestats-data
_gh api /user --jq .login
```

```bash
# Install detection (Bash 3.2 safe)
if ! command -v gh > /dev/null 2>&1; then
  # install gh
fi

# Version extraction (works with Bash 3.2 and GNU/BSD awk)
GH_VERSION=$(gh --version | head -1 | awk '{print $3}')
GH_MAJOR=$(echo "$GH_VERSION" | cut -d. -f1)

# Auth check
if ! _gh auth status > /dev/null 2>&1; then
  _gh auth login
fi
```

### Binary Download URL Pattern

Release assets are published by Epic 8 (`release.yml`) as:

```
https://github.com/stephenleo/vibestats/releases/latest/download/vibestats-<target>.tar.gz
https://github.com/stephenleo/vibestats/releases/latest/download/vibestats-<target>.tar.gz.sha256
```

Targets (from Epic 8 matrix):
- `aarch64-apple-darwin` (macOS Apple Silicon)
- `x86_64-apple-darwin` (macOS Intel)
- `x86_64-unknown-linux-gnu` (Linux x86_64)

### Checksum Verification

```bash
# macOS (BSD shasum)
shasum -a 256 -c vibestats-<target>.tar.gz.sha256

# Linux (GNU sha256sum)
sha256sum -c vibestats-<target>.tar.gz.sha256

# Platform-aware pattern
case "$(uname -s)" in
  Darwin) shasum -a 256 -c "$checksum_file" ;;
  Linux)  sha256sum -c "$checksum_file" ;;
esac
```

### Temp Directory and Cleanup

```bash
TMPDIR_WORK=$(mktemp -d)
trap 'rm -rf "$TMPDIR_WORK"' EXIT  # runs on both success and failure

# Download into TMPDIR_WORK
curl -fsSL "$download_url" -o "$TMPDIR_WORK/vibestats-${TARGET}.tar.gz"
```

### Test Framework Setup

**`bats-core`** is the approved shell test framework for Epic 6 (per `test-design-epic-6.md`).

Installation (for CI and local dev):
```bash
# Via npm (if already in the project)
npm install --save-dev bats

# Or via git submodule
git submodule add https://github.com/bats-core/bats-core.git tests/bats
```

Test structure pattern:
```bash
#!/usr/bin/env bats

setup() {
  export HOME="$(mktemp -d)"  # Isolate from real $HOME
  # Source install.sh functions into test scope (or run via 'run bash install.sh')
}

teardown() {
  rm -rf "$HOME"
}

@test "gh version < 2.0 → exits non-zero with message" {
  # Override the _gh() helper — install.sh wraps ALL gh calls through _gh()
  _gh() {
    case "$1" in
      --version) echo "gh version 1.14.0 (2022-01-01)" ;;
      auth) return 0 ;;
      *) return 0 ;;
    esac
  }
  export -f _gh
  run bash install.sh
  [ "$status" -ne 0 ]
  [[ "$output" == *"1.14.0"* ]]
}
```

### Security Requirements (Critical)

These apply to Stories 6.2+, but the foundation (no token handling in this story) must be clean:

- `VIBESTATS_TOKEN` must **never** be written to disk or echoed (R-001, NFR7) — not applicable in this story, but `install.sh` must not set env vars that could leak later
- `~/.config/vibestats/config.toml` will be written in Story 6.2 with `chmod 600` (NFR6) — this story does not write that file
- Token handling architecture: two independent tokens — machine-side (`gh auth token`) in `config.toml`, and `VIBESTATS_TOKEN` (fine-grained PAT) in Actions secret — both set in Story 6.2

### NFR Reference

| NFR | Requirement | Applies to this story |
|-----|-------------|----------------------|
| NFR6 | `config.toml` permissions `600` | No — file written in Story 6.2 |
| NFR7 | Token never written to disk | Partial — no token handling in 6.1, architecture must be clean |
| NFR16 | `gh` version ≥ 2.0 required | **Yes — version check and warning/exit in Task 3** |

### Cross-Story Dependencies

This story (6.1) is the **foundation** for Stories 6.2–6.4:

- Story 6.2 depends on: `install_gh_if_missing()`, `check_gh_auth()`, and binary install steps from this story
- Stories 6.2–6.4 add functions to `install.sh` built on top of the skeleton established here
- Do NOT implement any logic from 6.2–6.4 (repo creation, `VIBESTATS_TOKEN`, registry.json, hooks, README markers, backfill)

### PR Checklist

- [x] `install.sh` passes `bash -n install.sh` (syntax check, no execution)
- [x] `bats tests/installer/test_6_1.bats` — all tests pass
- [x] `set -euo pipefail` present on line 2 or 3 of `install.sh`
- [x] No hardcoded usernames in `install.sh` — use `$(gh api user --jq .login)` when needed (Story 6.2+)
- [x] Platform detection covers: macOS arm64, macOS x86_64, Linux x86_64, and unsupported (exits with clear message)

## Dev Agent Record

### Implementation Plan

Replaced the stub `install.sh` with a full rustup-style installer structured as individual step functions. Key design decisions:

1. **`_gh()` helper with "define if not defined" guard** — `install.sh` checks `declare -f _gh` before defining the helper, allowing bats tests to pre-define their own `_gh` stub without it being overwritten on `source`.
2. **Guard `main()` with `BASH_SOURCE` check** — `if [ "${BASH_SOURCE:-$0}" = "$0" ]` prevents `main "$@"` from executing when the script is sourced by tests, enabling function-level unit testing.
3. **Bash 3.2 compatibility throughout** — used `case` statements for all conditional branching, `command -v` instead of `which`, `cut`/`awk` for version parsing. Avoided `declare -A`, `mapfile`, and `[[ =~ ]]` regex captures.

### Completion Notes

All 6 tasks completed. 10/10 bats tests pass (9 P1, 1 P2):
- Task 1: `install.sh` skeleton — shebang, `set -euo pipefail`, `main()` entrypoint with BASH_SOURCE guard
- Task 2: `install_gh_if_missing()` — detects via `command -v gh`, installs via `brew` (macOS) or `apt-get` (Linux)
- Task 3: `check_gh_version()` — extracts major version with `awk`/`cut`, exits 1 with message if < 2.0
- Task 4: `check_gh_auth()` — checks `_gh auth status`, runs `_gh auth login` if unauthenticated, re-verifies
- Task 5: `detect_platform()` + `download_and_install_binary()` — maps OS/arch to target, downloads via `curl`, verifies SHA256, installs to `~/.local/bin/`
- Task 6: `tests/installer/test_6_1.bats` — 10 bats tests covering all ACs

### Change Log

| Date | Change | Author |
|------|--------|--------|
| 2026-04-12 | Story created | BMad Create-Story |
| 2026-04-12 | Implemented install.sh: gh detection, version check, auth, platform detection, binary download/install; all 10 bats tests pass | Dev Agent (Step 3) |
| 2026-04-12 | Code review: added default case in checksum verification, restored sprint-status.yaml and story file from main, updated status to done | Code Reviewer (Step 5) |

### Review Findings

- [x] [Review][Patch] Missing default case in checksum verification case statement [install.sh:139] — added `*` fallthrough that exits with error instead of silently skipping checksum
- [x] [Review][Patch] Sprint-status.yaml regressed to backlog — restored from main (branch had stale version from before story was started)
- [x] [Review][Patch] Story spec file deleted instead of updated — restored from main and updated status to done
- [x] [Review][Defer] EXIT trap override in download_and_install_binary may conflict with future cleanup traps [install.sh:129] — deferred, pre-existing architectural pattern for 6.2+ scope

## File List

- `install.sh` (modified — replaced stub with full implementation)
- `tests/installer/test_6_1.bats` (pre-existing from ATDD phase — no changes needed, all pass)
