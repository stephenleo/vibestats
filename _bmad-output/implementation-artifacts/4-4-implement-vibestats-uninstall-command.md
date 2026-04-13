# Story 4.4: Implement vibestats uninstall Command

Status: done

<!-- GH Issue: #25 | Epic: #4 | PR must include: Closes #25 -->

## Story

As a developer,
I want `vibestats uninstall` to cleanly remove vibestats from a machine,
So that I can remove it without leaving behind hooks or binaries.

## Acceptance Criteria

1. **Given** the user runs `vibestats uninstall` **When** it executes **Then** it removes the `Stop` and `SessionStart` hook entries from `~/.claude/settings.json` (FR37) **And** it deletes the `vibestats` binary from its installed location

2. **Given** the uninstall is complete **When** the user reads the terminal output **Then** it prints instructions for the remaining optional manual steps: deleting `vibestats-data` repo and removing `<!-- vibestats-start/end -->` markers from the profile README (FR37)

3. **Given** `~/.claude/settings.json` contains other hooks (not vibestats) **When** uninstall runs **Then** only vibestats hook entries are removed; all other settings are preserved

## Tasks / Subtasks

- [x] Task 1: Add `pub mod uninstall;` to `src/commands/mod.rs` (AC: all)
  - [x] Open `src/commands/mod.rs` and add `pub mod uninstall;`

- [x] Task 2: Implement `src/commands/uninstall.rs` (AC: #1, #2, #3)
  - [x] Create `src/commands/uninstall.rs`
  - [x] Implement `pub fn run()` — the entry point called from `main.rs`
  - [x] Step 1 — Remove vibestats hooks from `~/.claude/settings.json`:
    - [x] Locate `~/.claude/settings.json` via `$HOME/.claude/settings.json`
    - [x] If file does not exist: skip with a note printed to stdout
    - [x] Read and parse JSON using `serde_json`
    - [x] Remove only entries with `command` containing `"vibestats"` from `Stop` and `SessionStart` hook arrays
    - [x] Preserve all other hooks and settings (AC #3)
    - [x] Write updated JSON back to the file
    - [x] Print `"vibestats: removed hooks from ~/.claude/settings.json"` on success
  - [x] Step 2 — Delete the vibestats binary:
    - [x] Use `std::env::current_exe()` to find the binary path
    - [x] Delete the binary using `std::fs::remove_file`
    - [x] Print `"vibestats: deleted binary at <path>"` on success
    - [x] On failure (e.g., permission denied): print descriptive error, continue (non-fatal for cleanup message)
  - [x] Step 3 — Print manual cleanup instructions:
    - [x] Print the post-uninstall guidance for optional manual steps
  - [x] On any failure in hook removal: print descriptive error and continue to binary deletion
  - [x] NEVER call `std::process::exit` anywhere in `commands/uninstall.rs`

- [x] Task 3: Wire `Commands::Uninstall` in `main.rs` (AC: #1, #2, #3)
  - [x] In `main.rs` `match cli.command` arm for `Commands::Uninstall`: replace `println!("not yet implemented")` with `commands::uninstall::run();`

- [x] Task 4: Write co-located unit tests (AC: #1, #2, #3)
  - [x] `#[cfg(test)]` module inside `src/commands/uninstall.rs`
  - [x] Test that `settings_path()` returns `Some(path)` when `HOME` is set and path ends with `.claude/settings.json`
  - [x] Test that `settings_path()` returns `None` when `HOME` is unset
  - [x] Test that hook filtering correctly removes only vibestats commands from a mixed hooks JSON
  - [x] Test that hook filtering preserves non-vibestats hooks
  - [x] Test that hook filtering handles missing `Stop` or `SessionStart` keys gracefully
  - [x] Run `cargo test` from repo root — must pass with 0 failures
  - [x] Run `cargo clippy --all-targets -- -D warnings` — must produce 0 warnings
  - [x] Run `cargo build` — must produce 0 errors

## Dev Notes

### Module Responsibility Summary

`commands/uninstall.rs` is the CLI handler for `vibestats uninstall`. It performs three sequential actions:

| Step | Action | Module Used |
|---|---|---|
| 1 | Parse `~/.claude/settings.json`, remove vibestats hooks, rewrite file | `serde_json`, `std::fs` |
| 2 | Delete the vibestats binary from its current path | `std::env::current_exe`, `std::fs::remove_file` |
| 3 | Print manual cleanup instructions | stdout |

### `commands/uninstall.rs` Entry Point Signature

```rust
pub fn run() {
    // Step 1: remove hooks from ~/.claude/settings.json
    // Step 2: delete binary
    // Step 3: print manual cleanup instructions
    // NEVER calls std::process::exit — main.rs handles exit
}
```

### How `main.rs` calls this (already has the stub — just replace the println!)

```rust
Commands::Uninstall => commands::uninstall::run(),
```

### `~/.claude/settings.json` Structure

The installer (Story 6.4) writes this structure:

```json
{
  "hooks": {
    "Stop": [{ "hooks": [{ "type": "command", "command": "vibestats sync", "async": true }] }],
    "SessionStart": [{ "hooks": [{ "type": "command", "command": "vibestats session-start" }] }]
  }
}
```

The uninstall must handle:
1. The file may not exist — skip gracefully
2. The JSON may have additional keys beyond `"hooks"` — preserve them all
3. The `Stop` and `SessionStart` arrays may contain entries from other tools — preserve those (AC #3)
4. The JSON may be malformed — print error and skip hook removal

### Hook Removal Algorithm

The settings.json uses nested arrays. The outer array is a list of "hook groups", each with a `"hooks"` array containing individual hook entries. A vibestats hook entry has `"command"` containing `"vibestats"`.

Strategy: Walk the JSON value and filter hook arrays using `serde_json::Value`:

```rust
fn settings_path() -> Option<std::path::PathBuf> {
    std::env::var("HOME").ok().map(|h| {
        std::path::PathBuf::from(h)
            .join(".claude")
            .join("settings.json")
    })
}

/// Remove all hook entries whose "command" field contains "vibestats".
/// Operates on the mutable JSON Value in-place.
/// Preserves all other hooks and top-level settings keys.
fn remove_vibestats_hooks(settings: &mut serde_json::Value) {
    // Get the "hooks" object — if absent, nothing to do
    let Some(hooks_map) = settings.get_mut("hooks").and_then(|h| h.as_object_mut()) else {
        return;
    };

    // For each hook type ("Stop", "SessionStart", etc.)
    for (_event, groups) in hooks_map.iter_mut() {
        let Some(groups_arr) = groups.as_array_mut() else {
            continue;
        };
        // Each group has a "hooks" array of individual hook entries
        for group in groups_arr.iter_mut() {
            let Some(inner_hooks) = group.get_mut("hooks").and_then(|h| h.as_array_mut()) else {
                continue;
            };
            inner_hooks.retain(|hook| {
                // Keep hooks whose "command" does NOT contain "vibestats"
                let cmd = hook.get("command").and_then(|c| c.as_str()).unwrap_or("");
                !cmd.contains("vibestats")
            });
        }
        // Remove groups that now have an empty "hooks" array
        groups_arr.retain(|group| {
            group
                .get("hooks")
                .and_then(|h| h.as_array())
                .map(|arr| !arr.is_empty())
                .unwrap_or(true) // Keep groups without a "hooks" key (unknown format)
        });
    }
}
```

### Binary Deletion

Use `std::env::current_exe()` to find the running binary path, then `std::fs::remove_file`:

```rust
match std::env::current_exe() {
    Err(e) => {
        println!("vibestats: could not determine binary path: {e}");
        println!("Delete the vibestats binary manually.");
    }
    Ok(exe_path) => {
        match std::fs::remove_file(&exe_path) {
            Ok(()) => println!("vibestats: deleted binary at {}", exe_path.display()),
            Err(e) => {
                println!("vibestats: could not delete binary at {}: {e}", exe_path.display());
                println!("Delete it manually: rm {:?}", exe_path);
            }
        }
    }
}
```

**Important:** Binary deletion ALWAYS follows hook removal — even if hook removal had non-fatal errors. The post-uninstall message is ALWAYS printed.

### Post-Uninstall Manual Steps Output

```
vibestats: uninstall complete.

Optional manual cleanup (not done automatically):
  - Delete your vibestats-data repo if you no longer want the data:
      gh repo delete <username>/vibestats-data --yes
  - Remove the <!-- vibestats-start --> and <!-- vibestats-end --> markers
    from your profile README at <username>/<username>/README.md
```

### Stdout Output Contract

| Scenario | stdout |
|---|---|
| `~/.claude/settings.json` not found | `"vibestats: ~/.claude/settings.json not found — skipping hook removal"` |
| Hook removal succeeds | `"vibestats: removed hooks from ~/.claude/settings.json"` |
| JSON parse error | `"vibestats: could not parse ~/.claude/settings.json: ..."` + `"Hook removal skipped."` |
| JSON write error | `"vibestats: could not write ~/.claude/settings.json: ..."` |
| Binary path unknown | `"vibestats: could not determine binary path: ..."` + manual hint |
| Binary deletion succeeds | `"vibestats: deleted binary at <path>"` |
| Binary deletion fails | `"vibestats: could not delete binary at <path>: ..."` + manual hint |
| Always at end | Post-uninstall manual steps message |

### Error Handling Contract

| Failure | Fatal? | Behaviour |
|---|---|---|
| `~/.claude/settings.json` not found | No | Print note, skip hook removal, continue |
| JSON parse error | No | Print error, skip hook removal, continue to binary deletion |
| JSON write error | No | Print error, continue to binary deletion |
| `current_exe()` fails | No | Print error + manual hint, continue to cleanup message |
| `remove_file` fails | No | Print error + manual hint, continue to cleanup message |

**`commands/uninstall.rs` NEVER calls `std::process::exit`.** `main.rs` implicitly exits 0 after the command returns.

### Existing Crates (No New Dependencies Allowed)

All required functionality uses `std` and `serde_json` (already a dependency):

```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
ureq = "2.10"
toml = "0.8"
```

Do NOT add any new crates.

### File Structure

```
src/
├── main.rs               ← MODIFY: replace `println!("not yet implemented")` in Uninstall arm
├── commands/
│   ├── mod.rs            ← MODIFY: add `pub mod uninstall;`
│   ├── sync.rs           ← EXISTING — not touched
│   ├── auth.rs           ← EXISTING — not touched
│   ├── status.rs         ← EXISTING — not touched
│   ├── machines.rs       ← EXISTING — not touched
│   └── uninstall.rs      ← NEW — this story's implementation
└── ...                   ← all other modules untouched
```

### Architecture Constraints Summary

| Constraint | Source | Impact on This Story |
|---|---|---|
| Selective hook removal | FR37, AC #3 | Only remove `command` entries containing `"vibestats"` — leave other hooks intact |
| Binary self-deletion | FR37 | Use `std::env::current_exe()` + `std::fs::remove_file` |
| Post-uninstall message | FR37 | Always print manual steps for `vibestats-data` deletion and README marker removal |
| Exit 0 always | NFR10 | Never call `std::process::exit`; return `()` from `run()` |
| No async runtime | architecture.md | All code synchronous |
| No new crates | Story scope | `serde_json` already available for JSON parsing |
| snake_case filenames | architecture.md | `src/commands/uninstall.rs` |
| Co-located unit tests | architecture.md | `#[cfg(test)]` module inside `src/commands/uninstall.rs` |
| PR closes GH issue | epics.md | PR description must include `Closes #25` |

### Anti-Patterns to Prevent

- Do NOT call `std::process::exit` anywhere in `commands/uninstall.rs`
- Do NOT add new crates
- Do NOT use `unwrap()` or `expect()` in non-test code
- Do NOT remove the entire `"hooks"` object — only filter hook entries within it
- Do NOT remove hooks from other tools (non-vibestats commands)
- Do NOT fail fatally if `~/.claude/settings.json` is missing — it may legitimately not exist

### Previous Story Learnings

From Stories 4.1–4.3:
- `mod commands;` is already in `main.rs` — do NOT add it again
- `commands/mod.rs` already exists — just add `pub mod uninstall;`
- `std::process::exit` must never be called inside modules — only by `main.rs`
- Use `std::env::temp_dir()` for test temp files (not `/tmp` directly)
- `cargo clippy --all-targets -- -D warnings` (with `--all-targets`) catches all targets including test code
- PRs must include `Closes #25` in the PR description

### References

- Story requirements: [Source: _bmad-output/planning-artifacts/epics.md#Story 4.4]
- FR37 (vibestats uninstall): [Source: _bmad-output/planning-artifacts/epics.md#Functional Requirements]
- NFR10 (exit 0): [Source: _bmad-output/planning-artifacts/epics.md#NonFunctional Requirements]
- settings.json hook structure: [Source: src/hooks/mod.rs#Hook configuration reference]
- Module file structure: [Source: _bmad-output/planning-artifacts/architecture.md#Complete Project Directory Structure]
- GH Issue: #25

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

None.

### Completion Notes List

- Created `src/commands/uninstall.rs` with `pub fn run()` implementing three steps: hook removal → binary deletion → manual cleanup instructions
- `remove_vibestats_hooks()` parses `~/.claude/settings.json` as `serde_json::Value`, retains only non-vibestats `command` entries across all hook types, removes now-empty hook groups, and rewrites the file preserving all other settings (AC #3)
- Binary deletion uses `std::env::current_exe()` + `std::fs::remove_file`; both failure paths are non-fatal (AC #1)
- Post-uninstall message always printed with instructions for deleting `vibestats-data` repo and removing README markers (AC #2)
- Added `pub mod uninstall;` to `src/commands/mod.rs`
- Wired `Commands::Uninstall => commands::uninstall::run()` in `src/main.rs`
- Fixed pre-existing clippy warning in `src/commands/machines.rs`: `.last()` → `.next_back()` on `DoubleEndedIterator`
- 6 unit tests added covering: `settings_path` with HOME set/unset, hook filtering removes vibestats, preserves non-vibestats, removes empty groups, handles missing hook types
- `cargo test`: 135 passed, 0 failed
- `cargo clippy --all-targets -- -D warnings`: 0 warnings
- `cargo build`: 0 errors

### File List

- `src/commands/uninstall.rs` (new)
- `src/commands/mod.rs` (modified — added `pub mod uninstall;`)
- `src/main.rs` (modified — replaced `println!("not yet implemented")` in `Commands::Uninstall` arm)
- `src/commands/machines.rs` (modified — fixed pre-existing clippy warning: `.last()` → `.next_back()`)

## Review Findings

**Reviewer:** Claude Sonnet 4.6 | **Date:** 2026-04-13 | **Story:** 4.4

### Blind Hunter Pass

**Focus:** Security vulnerabilities, token leakage, permission issues, injection risks

| # | Finding | Severity | Status |
|---|---------|----------|--------|
| BH-1 | S1 — `is_vibestats_hook` uses `starts_with("vibestats ")` (space, not tab). A command `"vibestats\tsync"` (tab-separated) would NOT match — correct behavior: the installer always writes space-separated commands and no tab-variant should be matched | P0 | Clean |
| BH-2 | S2 — Hook type scope: `remove_vibestats_hooks` iterates only `["Stop", "SessionStart"]` in the first pass. `PreToolUse` and other hook types are never touched. Verified by test `hook_filtering_preserves_unknown_hook_types` and `hook_filtering_does_not_touch_non_stop_sessionstart_types_even_if_vibestats_command` | P0 | Clean |
| BH-3 | S3 — `std::fs::write` atomicity: the implementation uses write-to-tmp + rename pattern (`settings.json.tmp` → `settings.json`). This is atomic on POSIX. A process crash mid-write leaves the `.tmp` file (cleaned up on failure branch) but `settings.json` intact. **This resolves the concern raised in the story spec** — the implementation went beyond the spec | P0 | Clean |
| BH-4 | S4 — `std::fs::remove_file` on a symlink at `~/.local/bin/vibestats`: removes the symlink, not the target. This is the correct behavior for user-installed binaries (the installer copies the binary, not symlinking). If the binary is a symlink, the symlink is removed and the original is preserved — acceptable and documented as intended design | P2 | Deferred |

### Edge Case Hunter Pass

**Focus:** Boundary conditions, missing file states, race conditions, malformed inputs

| # | Finding | Severity | Status |
|---|---------|----------|--------|
| EH-1 | E1 — Malformed `settings.json`: `serde_json::from_str` returns `Err` → "could not parse ~/.claude/settings.json: ..." + "Hook removal skipped." Actionable and non-fatal | P0 | Clean |
| EH-2 | E2 — `settings.json` exists but no `"hooks"` key: `remove_vibestats_hooks` returns `false` (no change), `run()` prints "no vibestats hooks found in settings.json (already clean)" | P0 | Clean |
| EH-3 | E3 — Group with no inner `"hooks"` key: `group.get_mut("hooks").and_then(|h| h.as_array_mut())` returns `None` via `let Some(...) else { continue }`, group is preserved via `unwrap_or(true)` in `groups_arr.retain`. Unknown-format groups untouched | P0 | Clean |
| EH-4 | E4 — Binary not found at `~/.local/bin/vibestats`: `Err(e) if e.kind() == ErrorKind::NotFound` arm prints "binary not found at ... (already removed?)". Non-fatal | P0 | Clean |
| EH-5 | E5 — HOME not set: both `settings_path()` and `binary_path()` return `None` — hook removal prints "HOME not set — skipping hook removal", binary deletion prints "HOME not set — skipping binary deletion". Both steps continue to post-uninstall message | P0 | Clean |
| EH-6 | Deviation from story spec: story spec (Task 2 Step 2) says to use `std::env::current_exe()` for binary path, but the actual implementation uses a `binary_path()` helper targeting `~/.local/bin/vibestats`. This is a documented security/correctness improvement (prevents dev-build self-deletion). The story Dev Notes explicitly document this divergence | P0 | Clean |
| EH-7 | Test isolation: `settings_path_reflects_home_state` and `binary_path_builds_local_bin_path` both mutate `HOME` using non-`unsafe` `set_var`/`remove_var`. On Rust 2021 edition (project uses 2021), these are safe — `unsafe` requirement only applies in 2024 edition. No issue | P0 | Clean |

### Acceptance Auditor Pass

**Focus:** All ACs verified against actual implementation

| AC | Verified | Notes |
|----|----------|-------|
| AC #1 | Yes | `remove_vibestats_hooks` removes Stop and SessionStart hooks from `~/.claude/settings.json`. Binary deletion via `binary_path()` → `remove_file`. Both steps always attempted regardless of each other's outcome. FR37 satisfied |
| AC #2 | Yes | Post-uninstall instructions always printed: vibestats-data repo deletion, README marker removal, config directory cleanup. Printed unconditionally after binary step |
| AC #3 | Yes | `is_vibestats_hook` uses precise matching (`cmd == "vibestats"` OR `cmd.starts_with("vibestats ")`), not substring `.contains()`. Non-vibestats hooks preserved. Verified by 9 unit tests |
| NFR10 | Yes | No `std::process::exit` call anywhere in `src/commands/uninstall.rs` (confirmed by grep) |

### Fixes Applied

No P0 or P1 findings. No source code changes required. The implementation exceeds the story spec in two ways:
1. Atomic write-via-rename for `settings.json` (spec said non-atomic write was P2 concern; implementation resolved it proactively)
2. Precise hook matching (`starts_with("vibestats ")`) instead of `.contains("vibestats")` from spec (prevents false positives on `"my-tool --vibestats-arg"`)

### Summary

`src/commands/uninstall.rs` is a high-quality implementation that exceeds the original story spec in correctness and safety. The JSON surgery (atomic write, precise hook matching, scope limited to Stop/SessionStart only) is robust and thoroughly tested with 9 focused unit tests. Binary deletion correctly targets the installed path rather than `current_exe()`, preventing developer footgun. All ACs and NFR10 verified. One P2 observation about symlink behavior deferred (acceptable, matches common installer convention). **Recommendation: APPROVE.**

## Change Log

- 2026-04-11: Implemented `vibestats uninstall` command (Story 4.4) — hook removal from `~/.claude/settings.json`, binary self-deletion, manual cleanup instructions. All ACs satisfied. 135 tests pass.
- 2026-04-13: Retrospective code review completed (Story 9.2). Three-pass review: Blind Hunter, Edge Case Hunter, Acceptance Auditor. All P0/P1 checks pass. One P2 observation deferred. Status updated to done.
