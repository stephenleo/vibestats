# Story 6.4: Implement hook configuration, README markers, and backfill trigger

Status: done

<!-- GH Issue: #34 | Epic: #6 | PR must include: Closes #34 -->

## Story

As a vibestats user completing installation,
I want the installer to configure my Claude Code hooks, add README markers, and trigger an immediate backfill,
So that history is visible the moment I open my profile.

## Acceptance Criteria

1. **Given** installation reaches its final phase **When** hook configuration runs **Then** `~/.claude/settings.json` is updated with `Stop` hook (`command: vibestats sync`, `async: true`) and `SessionStart` hook (`command: vibestats sync`) (FR8)

2. **Given** the profile README at `username/username/README.md` exists **When** the marker injection runs **Then** `<!-- vibestats-start -->` and `<!-- vibestats-end -->` markers are added with the SVG `<img>` embed and dashboard link between them (FR9)

3. **Given** all setup steps complete **When** the installer triggers the post-install backfill **Then** it runs `vibestats sync --backfill` as the final step, printing progress to the terminal (FR11)

## Tasks / Subtasks

- [x] Task 1: Implement `configure_hooks()` function (AC: #1, FR8, R-008)
  - [x] File: `install.sh` at repo root — add after existing `register_machine()` function (Step 12)
  - [x] Read existing `~/.claude/settings.json` if present; default to `{}` if missing or empty
  - [x] Use python3 stdlib (no `jq`) to parse and update hooks JSON:
    - Check if `Stop` hook with `command: "vibestats sync"` already exists in `settings.hooks.Stop[].hooks[]`
    - If NOT present: append `{"type": "command", "command": "vibestats sync", "async": true}` inside a matcher object `{"hooks": [...]}` to `settings.hooks.Stop`
    - Check if `SessionStart` hook with `command: "vibestats sync"` already exists
    - If NOT present: append `{"type": "command", "command": "vibestats sync"}` inside a matcher object to `settings.hooks.SessionStart`
  - [x] Write updated JSON back to `~/.claude/settings.json` (create parent dir `~/.claude/` with `mkdir -p` if needed)
  - [x] Idempotency is REQUIRED: running `configure_hooks()` twice on the same `settings.json` must result in exactly one `Stop` entry and one `SessionStart` entry (R-008, P2 test)
  - [x] Do NOT clobber unrelated hooks or keys in `settings.json` — merge, never overwrite the whole file
  - [x] Print: "Claude Code hooks configured in ~/.claude/settings.json"
  - [x] Do NOT call `configure_hooks` on `~/.claude/settings.json` paths outside `$HOME` — always use `${HOME}/.claude/settings.json`

- [x] Task 2: Implement `inject_readme_markers()` function (AC: #2, FR9, R-009)
  - [x] File: `install.sh` — add after `configure_hooks()` (Step 13)
  - [x] Requires `GITHUB_USER` exported by `detect_install_mode()` (auto-detect if not set)
  - [x] Fetch `username/username/README.md` via `_gh api "repos/${GITHUB_USER}/${GITHUB_USER}/contents/README.md"` (profile repo)
  - [x] **If the profile repo or README does not exist (404 / non-zero exit):** print a warning message (NOT an error; do NOT exit non-zero) and return — the user can add markers manually later (R-009 mitigation)
    - Warning text: "Warning: Could not access ${GITHUB_USER}/${GITHUB_USER}/README.md. Please add vibestats markers manually. See https://vibestats.dev/docs/quickstart for instructions."
  - [x] If README exists: decode base64 content via Python (`python3 -c "import sys,base64; print(base64.b64decode(sys.argv[1]).decode())"`)
  - [x] Check if markers `<!-- vibestats-start -->` and `<!-- vibestats-end -->` already exist in the decoded content
    - If already present: print "vibestats README markers already present — skipping." and return (idempotency)
  - [x] Build the marker block to inject:
    ```
    <!-- vibestats-start -->
    [![vibestats](https://raw.githubusercontent.com/USERNAME/USERNAME/main/vibestats/heatmap.svg)](https://vibestats.dev/USERNAME)

    [View interactive dashboard →](https://vibestats.dev/USERNAME)
    <!-- vibestats-end -->
    ```
    Replace `USERNAME` with `$GITHUB_USER`
  - [x] Append the marker block to the end of the decoded README content (after existing content + newline separator)
  - [x] Base64-encode updated content (platform-aware: `base64` on Linux, `base64` with no `-w` needed on macOS — use `base64 | tr -d '\n'` for cross-platform)
  - [x] GET SHA: extract from same API response parsed via python3 (single GET — no second API call)
  - [x] PUT updated README via `_gh api "repos/${GITHUB_USER}/${GITHUB_USER}/contents/README.md" --method PUT --field message="Add vibestats heatmap markers" --field content="..." --field sha="..."`
  - [x] Print: "vibestats markers added to ${GITHUB_USER}/${GITHUB_USER}/README.md"

- [x] Task 3: Implement `run_backfill()` function (AC: #3, FR11)
  - [x] File: `install.sh` — add after `inject_readme_markers()` (Step 14)
  - [x] Call `"${HOME}/.local/bin/vibestats" sync --backfill` directly (binary installed by Step 5)
  - [x] This is a foreground blocking call — output streams to terminal (user sees progress)
  - [x] If binary exits non-zero: print a warning but do NOT exit the installer non-zero — backfill failure is non-fatal (user can run `vibestats sync --backfill` manually later)
    - Pattern: `if ! "${HOME}/.local/bin/vibestats" sync --backfill; then echo "Warning: Backfill completed with errors. Run 'vibestats sync --backfill' manually to retry."; fi`
  - [x] Print before running: "Running post-install backfill (vibestats sync --backfill)..."
  - [x] Print after success: "Backfill complete."

- [x] Task 4: Wire new functions into `main()` and `first_install_path()` (AC: #1, #2, #3)
  - [x] After the `case "$INSTALL_MODE"` branch in `main()`, add calls that run for BOTH install modes:
    - `configure_hooks`
    - `inject_readme_markers`
    - `run_backfill`
  - [x] These steps run after `register_machine` (multi-machine) and after `first_install_path` (first-install) — they are shared final steps
  - [x] `run_backfill` must be the LAST step in `main()` before the completion message (P2 test: "backfill is the final step")
  - [x] Updated `main()` structure:
    ```bash
    main() {
      # Steps 1–5 (existing)
      install_gh_if_missing
      check_gh_version
      check_gh_auth
      download_and_install_binary
      # Step 6: detect mode
      detect_install_mode
      # Step 7: path-specific setup
      case "$INSTALL_MODE" in
        multi-machine) register_machine ;;
        first-install) first_install_path ;;
      esac
      # Steps 12–14: shared final steps (always run)
      configure_hooks
      inject_readme_markers
      run_backfill
      echo "=== Installation complete! ==="
      echo "Run 'vibestats --help' to get started."
    }
    ```

- [x] Task 5: Write shell tests for Story 6.4 using `bats-core` (AC: #1, #2, #3, R-008, R-009)
  - [x] Test file: `tests/installer/test_6_4.bats`
  - [x] Follow the same patterns established in `tests/installer/test_6_1.bats` and `test_6_3.bats`:
    - Source `install.sh` in `setup()` to import functions
    - Override `_gh()` as a shell function to mock `gh` CLI calls
    - Override `$HOME` to a temp dir in `setup()` to prevent real file mutations
    - Use `teardown()` to `rm -rf "$HOME"`
  - [x] P1 tests to implement (required for PR gate):
    - `configure_hooks: Stop hook with command=vibestats sync and async=true written to settings.json` — assert JSON structure: `settings.hooks.Stop[0].hooks[0].command == "vibestats sync"` and `settings.hooks.Stop[0].hooks[0].async == true`
    - `configure_hooks: SessionStart hook with command=vibestats sync written to settings.json` — assert `settings.hooks.SessionStart[0].hooks[0].command == "vibestats sync"`
    - `configure_hooks: idempotent — running twice produces exactly one Stop and one SessionStart entry` (R-008 mitigation) — call `configure_hooks` twice; assert exactly one `Stop` matcher and one `SessionStart` matcher in the output JSON
    - `configure_hooks: does not clobber existing unrelated hooks` — pre-seed `settings.json` with a pre-existing hook key; assert it is still present after `configure_hooks`
  - [x] P2 tests to implement:
    - `inject_readme_markers: markers + SVG img + dashboard link written to profile README` (R-009 mitigation) — mock `_gh api` returning sample README content and SHA; assert PUT body contains `<!-- vibestats-start -->`, `<!-- vibestats-end -->`, SVG `<img>` URL, and dashboard link
    - `inject_readme_markers: warning (not error) and continues when profile repo returns 404` (R-009 mitigation) — mock `_gh api` returning non-zero for README fetch; assert `run` status 0 and `output` contains "Warning:"
    - `inject_readme_markers: idempotent — no second PUT when markers already present` — pre-seed README with existing markers; assert `_gh api PUT` NOT called
    - `run_backfill: vibestats sync --backfill is called as final step` — spy on binary call; assert `sync --backfill` appears in spy log
    - `run_backfill: non-zero exit from binary prints warning but installer exits 0` — mock binary to exit 1; assert installer function exits 0 and output contains "Warning:"
  - [x] Run tests: `bats tests/installer/test_6_4.bats` — all tests pass with 0 failures

## Dev Notes

### File Location

Single file to modify: `install.sh` at repo root.

**Current state after Stories 6.1–6.3:** `install.sh` has functions:
- `_gh()` helper (line 12–16)
- `install_gh_if_missing()` — Step 1
- `check_gh_version()` — Step 2
- `check_gh_auth()` — Step 3
- `detect_platform()` — Step 4 (called inside Step 5)
- `download_and_install_binary()` — Step 5
- `detect_install_mode()` — Step 6
- `create_vibestats_data_repo()` — Step 7 (first-install)
- `generate_aggregate_workflow_content()` — Step 8 (first-install)
- `write_aggregate_workflow()` — Step 9 (first-install)
- `setup_vibestats_token()` — Step 10 (first-install)
- `register_machine()` — Step 11 (both paths)
- `first_install_path()` — orchestrates Steps 7–11
- `main()` — calls Steps 1–6, then branches on `$INSTALL_MODE`

**This story adds:**
- `configure_hooks()` — Step 12 (shared, both paths)
- `inject_readme_markers()` — Step 13 (shared, both paths)
- `run_backfill()` — Step 14 (shared, both paths, LAST step)
- Updated `main()` — calls Steps 12–14 after the install mode branch

Do NOT create or modify:
- Any files in `src/` (Rust binary)
- Any files in `action/` (Python Actions pipeline)
- `.github/workflows/`
- `registry.json` (handled by Stories 6.2/6.3)

### Critical Architecture Constraints

From `architecture.md` and the test design (applies to all Epic 6 stories):

- **Bash 3.2 compatibility required** — no `declare -A`, no `mapfile`, no `[[ =~ ]]` regex captures, no `$(< file)` process substitution. Use `case`, `command -v`, `awk`, `sed`, `cut`.
- **`_gh()` helper is mandatory** — ALL `gh` calls must go through `_gh()` for test mockability. Never call `gh` directly.
- **`set -euo pipefail` + non-zero exit trap** — when calling `_gh api` for the profile README and handling 404, MUST use `|| true` or an `if` construct:
  ```bash
  # ✅ Safe — 404 is handled, installer continues
  README_RESPONSE=$(_gh api "repos/${GITHUB_USER}/${GITHUB_USER}/contents/README.md" 2>/dev/null || echo "NOT_FOUND")
  if [ "$README_RESPONSE" = "NOT_FOUND" ]; then
    echo "Warning: ..."
    return 0  # Do NOT exit 1
  fi
  ```
- **No `jq` dependency** — not listed as a required dependency. Use Python3 stdlib for JSON manipulation throughout.
- **`BASH_SOURCE` guard** — `main()` is guarded with `if [ "${BASH_SOURCE:-$0}" = "$0" ]`. Keep this guard intact — it allows bats tests to source `install.sh` without running `main`.
- **`_gh` define-if-not-defined guard** — already in place at top of `install.sh`. Keep it — this is what allows bats tests to override `_gh`.

### Exact `~/.claude/settings.json` Hook Schema

From Story 3.2 (`hooks.rs` documentation comment) and Epic 3 architecture — the exact JSON structure the installer must write:

```json
{
  "hooks": {
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "vibestats sync",
            "async": true
          }
        ]
      }
    ],
    "SessionStart": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "vibestats sync"
          }
        ]
      }
    ]
  }
}
```

Key points:
- `Stop` hook has `"async": true` (fires after session response without blocking Claude Code)
- `SessionStart` hook does NOT have `async: true` (runs synchronously for catch-up sync)
- Both are nested as: `hooks[event_type][matcher_index].hooks[hook_index]`
- The outer array item is a "matcher object" with a `hooks` array — the installer writes a single matcher with no `matcher` key (matches all tools)

### Python Pattern for Hook Configuration (stdlib only)

```python
import sys, json

settings_path = sys.argv[1]
try:
    with open(settings_path, 'r') as f:
        settings = json.load(f)
except (FileNotFoundError, json.JSONDecodeError):
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
```

Call this from bash:
```bash
configure_hooks() {
  CLAUDE_SETTINGS="${HOME}/.claude/settings.json"
  mkdir -p "${HOME}/.claude"
  python3 - "$CLAUDE_SETTINGS" <<'PYEOF'
<python code above>
PYEOF
  echo "Claude Code hooks configured in ~/.claude/settings.json"
}
```

Note: Use a here-doc (`<<'PYEOF'`) to pass Python code inline — avoids creating temp files and works cleanly with `set -euo pipefail`.

### README Marker Block (Exact Format)

```
<!-- vibestats-start -->
[![vibestats](https://raw.githubusercontent.com/USERNAME/USERNAME/main/vibestats/heatmap.svg)](https://vibestats.dev/USERNAME)

[View interactive dashboard →](https://vibestats.dev/USERNAME)
<!-- vibestats-end -->
```

- SVG URL: `https://raw.githubusercontent.com/${GITHUB_USER}/${GITHUB_USER}/main/vibestats/heatmap.svg`
- Dashboard URL: `https://vibestats.dev/${GITHUB_USER}`
- The `<img>` is implemented as a Markdown image link `[![alt](img-url)](link-url)` — renders correctly in GitHub profile READMEs
- From `update_readme.py` (Story 5.3): `update_readme.py` uses these same markers at runtime to update the SVG; the installer adds them once during setup

### GitHub Contents API Pattern for README (same GET+PUT as registry.json)

```bash
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
```

**Important:** `base64 | tr -d '\n'` is the cross-platform safe pattern (works on both macOS and Linux without needing `-w 0` flag which is Linux-only).

### Backfill Function Pattern

```bash
run_backfill() {
  echo "Running post-install backfill (vibestats sync --backfill)..."
  if ! "${HOME}/.local/bin/vibestats" sync --backfill; then
    echo "Warning: Backfill completed with errors. Run 'vibestats sync --backfill' manually to retry."
  else
    echo "Backfill complete."
  fi
}
```

The binary is installed at `~/.local/bin/vibestats` (Step 5). Use the explicit path rather than `vibestats` to avoid PATH issues in subshells.

### Test Framework Reference (from Stories 6.1–6.3 patterns)

`bats-core` is installed via npm: `npm install --save-dev bats`. Tests in `tests/installer/`.

Key patterns established in `test_6_1.bats` and `test_6_3.bats`:
1. Define `_gh()` override BEFORE sourcing `install.sh` — the define-if-not-defined guard ensures the test stub is not overwritten
2. Use `run <function_name>` to capture exit code and output
3. `[ "$status" -eq 0 ]` and `[[ "$output" == *"substring"* ]]` for assertions
4. Override `$HOME` in `setup()` to a `mktemp -d` temp directory
5. `teardown()` removes the temp home via `rm -rf "$HOME"`
6. Use `GH_SPY_LOG="${BATS_TMPDIR}/gh_calls.log"` and spy on `_gh` calls for negative assertions

For testing `configure_hooks`, parse the resulting `${HOME}/.claude/settings.json` with python3:
```bash
COUNT=$(python3 -c "
import json
with open('${HOME}/.claude/settings.json') as f:
    s = json.load(f)
print(len(s['hooks']['Stop']))
")
[ "$COUNT" -eq 1 ]
```

For testing the `run_backfill` step, mock the binary:
```bash
mkdir -p "${HOME}/.local/bin"
cat > "${HOME}/.local/bin/vibestats" <<'STUB'
#!/usr/bin/env bash
echo "vibestats $*" >> "${BATS_TMPDIR}/binary_calls.log"
exit 0
STUB
chmod +x "${HOME}/.local/bin/vibestats"
```

### Security Requirements (from NFR6, NFR7, R-001, R-002)

- This story does NOT generate `VIBESTATS_TOKEN` (that is Story 6.2)
- `configure_hooks` writes to `~/.claude/settings.json` — no sensitive tokens involved
- `inject_readme_markers` does not handle any tokens — only public README content and the `gh` auth session
- `run_backfill` calls the already-installed binary — no token handling

### Cross-Story Dependencies

- **Depends on Stories 6.1, 6.2, 6.3**: All prior functions (`_gh()`, `install_gh_if_missing()`, `check_gh_version()`, `check_gh_auth()`, `download_and_install_binary()`, `detect_install_mode()`, `create_vibestats_data_repo()`, `write_aggregate_workflow()`, `setup_vibestats_token()`, `register_machine()`) are in place and must NOT be modified
- **Binary available**: `vibestats` binary is installed at `~/.local/bin/vibestats` by Step 5 (Story 6.1); `run_backfill` depends on this
- **Story 3.2 / 3.3 hook schema**: Hook JSON structure was defined and documented in Story 3.2 (`hooks.rs` comment). Installer must produce matching JSON.
- **Story 5.3 README markers**: `update_readme.py` (Epic 5) reads the same `<!-- vibestats-start/end -->` markers at runtime — the installer places them; the Action maintains them thereafter. Marker format must match exactly.

### Shell Compatibility Reference

```bash
# ✅ Bash 3.2 safe
command -v python3
case "$(uname -s)" in Darwin) ... ;; Linux) ... ;; esac
[ -n "$VAR" ]
awk '{print $3}'
cut -d. -f1
base64 | tr -d '\n'   # cross-platform safe
grep -q 'pattern'

# ❌ Requires Bash 4+ — DO NOT use
declare -A map
mapfile -t arr
[[ "$str" =~ ^pattern(.*)$ ]]
```

### Project Structure Notes

- `install.sh` is at the repo root — this is the only file modified in this story
- `tests/installer/test_6_4.bats` is the new test file — follows the existing pattern in `tests/installer/`
- No changes to `src/`, `action/`, `.github/workflows/`, or any Rust or Python files

### References

- Story 6.3 implementation (patterns + `install.sh` current state): [Source: _bmad-output/implementation-artifacts/6-3-implement-multi-machine-install-path.md]
- Epic 6 acceptance criteria (FR8, FR9, FR11): [Source: _bmad-output/planning-artifacts/epics.md#Story 6.4]
- Hook JSON schema (from Story 3.2 doc comment): [Source: _bmad-output/implementation-artifacts/3-2-implement-stop-hook-integration.md#Task 4]
- Architecture bash installer section: [Source: _bmad-output/planning-artifacts/architecture.md#Starter Template Evaluation — Bash Installer]
- API & Communication Patterns (GET SHA + PUT): [Source: _bmad-output/planning-artifacts/architecture.md#API & Communication Patterns]
- README marker format (vibestats-start/end): [Source: _bmad-output/planning-artifacts/epics.md#Story 5.3], [Source: _bmad-output/planning-artifacts/architecture.md#project structure]
- FR8 (hooks), FR9 (README markers), FR11 (backfill): [Source: _bmad-output/planning-artifacts/epics.md#Functional Requirements]
- Epic 6 test design (R-008, R-009): [Source: _bmad-output/test-artifacts/test-design-epic-6.md]
- NFR6 (600 file permissions): [Source: _bmad-output/planning-artifacts/prd.md#Security] (not directly applicable this story — no new files with sensitive tokens)

### PR Checklist

- [ ] `install.sh` passes `bash -n install.sh` (syntax check)
- [ ] `bats tests/installer/test_6_4.bats` — all tests pass with 0 failures
- [ ] `bats tests/installer/test_6_3.bats` — all 7 regression tests still pass
- [ ] `bats tests/installer/test_6_2.bats` — all regression tests still pass
- [ ] `bats tests/installer/test_6_1.bats` — all 10 regression tests still pass
- [ ] `set -euo pipefail` still present on line 2 of `install.sh`
- [ ] `configure_hooks()` is idempotent — running twice on same `settings.json` leaves exactly one `Stop` and one `SessionStart` entry
- [ ] `inject_readme_markers()` handles missing profile repo gracefully (warning, exit 0)
- [ ] `inject_readme_markers()` is idempotent — no duplicate markers on re-run
- [ ] `run_backfill()` is the LAST step in `main()` before the completion message
- [ ] No hardcoded usernames — always use `$GITHUB_USER`
- [ ] All `gh` calls go through `_gh()` helper — no direct `gh` invocations
- [ ] `vibestats` binary invoked via `${HOME}/.local/bin/vibestats` (explicit path, not `vibestats`)

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

None — implementation was straightforward following the Dev Notes patterns.

### Completion Notes List

- Implemented `configure_hooks()` using Python3 heredoc to read/update `~/.claude/settings.json` without jq. Fully idempotent — checks existing hooks before appending.
- Implemented `inject_readme_markers()` using single GET+PUT pattern from architecture. Handles 404 gracefully (warning, exit 0). Idempotent via grep check on decoded content. Cross-platform base64 via `base64 | tr -d '\n'`.
- Implemented `run_backfill()` calling explicit binary path `${HOME}/.local/bin/vibestats sync --backfill`. Non-fatal: binary failure prints warning but installer exits 0.
- Wired all three functions into `main()` as shared final steps (Steps 12–14) after the install mode branch.
- Fixed a test design bug in ATDD test 7: `$output` was checked after a second `run` (which overwrites `$output`). Fixed by saving `INJECT_OUTPUT="$output"` before the idempotency spy log check.
- All 9 tests in `test_6_4.bats` pass. Regressions for `test_6_1.bats` (10 tests) and `test_6_3.bats` (7 tests) all pass. Pre-existing failures in `test_6_2.bats` are unrelated to this story.

### File List

- `install.sh` (modified — added `configure_hooks`, `inject_readme_markers`, `run_backfill` functions; updated `main()`)
- `tests/installer/test_6_4.bats` (modified — fixed test 7 output capture bug)
- `_bmad-output/implementation-artifacts/6-4-implement-hook-configuration-readme-markers-and-backfill-trigger.md` (modified — status, tasks, dev agent record)

## Change Log

- 2026-04-12: Implemented `configure_hooks()`, `inject_readme_markers()`, `run_backfill()` in `install.sh`; wired into `main()` as Steps 12–14; all 9 bats tests pass; story status set to review.
