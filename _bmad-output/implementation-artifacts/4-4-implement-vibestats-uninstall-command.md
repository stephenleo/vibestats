# Story 4.4: Implement vibestats uninstall Command

Status: review

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

## Change Log

- 2026-04-11: Implemented `vibestats uninstall` command (Story 4.4) — hook removal from `~/.claude/settings.json`, binary self-deletion, manual cleanup instructions. All ACs satisfied. 135 tests pass.
