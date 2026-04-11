# Story 4.4: Implement vibestats uninstall Command

Status: ready-for-dev

<!-- GH Issue: #25 | Epic: #4 | PR must include: Closes #25 -->

## Story

As a developer,
I want `vibestats uninstall` to cleanly remove vibestats from a machine,
so that I can remove it without leaving behind hooks or binaries.

## Acceptance Criteria

1. **Given** the user runs `vibestats uninstall` **When** it executes **Then** it removes the `Stop` and `SessionStart` hook entries from `~/.claude/settings.json` (FR37)

2. **Given** the uninstall is complete **When** the user reads the terminal output **Then** it prints instructions for the remaining optional manual steps: deleting `vibestats-data` repo and removing `<!-- vibestats-start/end -->` markers from the profile README (FR37)

3. **Given** `~/.claude/settings.json` contains other hooks (not vibestats) **When** uninstall runs **Then** only vibestats hook entries are removed; all other settings are preserved

4. **Given** the uninstall is complete **When** it executes **Then** it deletes the `vibestats` binary from its installed location (`~/.local/bin/vibestats`)

## Tasks / Subtasks

- [ ] Task 1: Add `pub mod uninstall;` to `src/commands/mod.rs` (AC: all)
  - [ ] Open `src/commands/mod.rs` and add `pub mod uninstall;`

- [ ] Task 2: Implement `src/commands/uninstall.rs` (AC: #1, #2, #3, #4)
  - [ ] Create `src/commands/uninstall.rs`
  - [ ] Implement `pub fn run()` — the entry point called from `main.rs`
  - [ ] Step 1 — remove vibestats hook entries from `~/.claude/settings.json` (AC #1, #3)
  - [ ] Step 2 — delete the vibestats binary from `~/.local/bin/vibestats` (AC #4)
  - [ ] Step 3 — print manual cleanup instructions to stdout (AC #2)
  - [ ] On any error: print descriptive message and continue (do NOT abort on non-fatal errors)
  - [ ] NEVER call `std::process::exit` — main.rs handles exit (NFR10)

- [ ] Task 3: Wire `Commands::Uninstall` in `main.rs` (AC: all)
  - [ ] In `src/main.rs` `match cli.command` arm for `Commands::Uninstall`: replace `println!("not yet implemented")` with `commands::uninstall::run();`
  - [ ] `mod commands;` is already in `main.rs` — do NOT add again

- [ ] Task 4: Write co-located unit tests (AC: #1, #3)
  - [ ] `#[cfg(test)]` module inside `src/commands/uninstall.rs`
  - [ ] Test that `settings_path()` returns `Some(path)` when `HOME` is set and path ends with `.claude/settings.json`
  - [ ] Test that `settings_path()` returns `None` when `HOME` is unset
  - [ ] Test hook removal: create a temp settings.json with vibestats Stop/SessionStart hooks + non-vibestats hooks → call hook-removal logic → verify vibestats entries removed and other entries preserved
  - [ ] Test hook removal when `~/.claude/settings.json` does not exist (no-op, no panic)
  - [ ] Test hook removal when `hooks` key is absent from settings.json (preserved as-is)
  - [ ] Run `cargo test` from repo root — must pass with 0 failures
  - [ ] Run `cargo clippy --all-targets -- -D warnings` — must produce 0 warnings
  - [ ] Run `cargo build` — must produce 0 errors

## Dev Notes

### Module Responsibility Summary

`commands/uninstall.rs` is the CLI handler for `vibestats uninstall`. It orchestrates three steps:

| Step | Action | Module Used |
|---|---|---|
| 1 | Remove vibestats hook entries from `~/.claude/settings.json` | `serde_json::Value` (std JSON manipulation) |
| 2 | Delete `~/.local/bin/vibestats` binary | `std::fs::remove_file` |
| 3 | Print manual cleanup instructions | `println!` |

No GitHub API calls. No config reads required (config.toml is left in place — user may want to re-install). No checkpoint changes.

### `commands/uninstall.rs` Entry Point Signature

```rust
pub fn run() {
    // Step 1: Remove vibestats hooks from ~/.claude/settings.json
    // Step 2: Delete ~/.local/bin/vibestats
    // Step 3: Print manual cleanup instructions
    // NEVER calls std::process::exit
}
```

### How `main.rs` calls this (already has the stub — just replace the println!)

```rust
Commands::Uninstall => commands::uninstall::run(),
```

### Step 1 — Remove vibestats hooks from `~/.claude/settings.json`

**Settings file path helper:**

```rust
fn settings_path() -> Option<std::path::PathBuf> {
    std::env::var("HOME").ok().map(|h| {
        std::path::PathBuf::from(h)
            .join(".claude")
            .join("settings.json")
    })
}
```

**Hook removal logic — preserve all non-vibestats content:**

The settings.json structure installed by vibestats:
```json
{
  "hooks": {
    "Stop": [
      { "hooks": [{ "type": "command", "command": "vibestats sync", "async": true }] }
    ],
    "SessionStart": [
      { "hooks": [{ "type": "command", "command": "vibestats session-start" }] }
    ]
  }
}
```

**CRITICAL:** The settings.json may contain many other settings (env vars, plugins, statusLine, sandbox, etc.) and other non-vibestats hooks. The uninstall MUST surgically remove only vibestats entries and preserve everything else.

**Detection strategy** — a hook object belongs to vibestats if its `"command"` field starts with `"vibestats "` (with space) OR equals `"vibestats"`:

```rust
fn is_vibestats_hook(hook_obj: &serde_json::Value) -> bool {
    hook_obj["command"]
        .as_str()
        .map(|cmd| cmd == "vibestats" || cmd.starts_with("vibestats "))
        .unwrap_or(false)
}
```

**Full hook removal implementation:**

```rust
fn remove_vibestats_hooks(path: &std::path::Path) -> Result<bool, String> {
    // Return Ok(false) if file doesn't exist — not an error
    let content = match std::fs::read_to_string(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(e) => return Err(format!("could not read {}: {e}", path.display())),
        Ok(c) => c,
    };

    let mut json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("settings.json is not valid JSON: {e}"))?;

    // If no hooks key, nothing to do
    if json.get("hooks").and_then(|h| h.as_object()).is_none() {
        return Ok(false);
    }

    let mut changed = false;

    // Process Stop and SessionStart hook types
    for hook_type in &["Stop", "SessionStart"] {
        // Check if the array exists and has entries to remove
        let has_vibestats = json["hooks"][hook_type]
            .as_array()
            .map(|entries| {
                entries.iter().any(|entry| {
                    entry["hooks"]
                        .as_array()
                        .map(|inner| inner.iter().any(is_vibestats_hook))
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false);

        if !has_vibestats {
            continue;
        }

        // Retain only non-vibestats entries
        if let Some(entries) = json["hooks"][hook_type].as_array_mut() {
            entries.retain(|entry| {
                entry["hooks"]
                    .as_array()
                    .map(|inner| !inner.iter().any(is_vibestats_hook))
                    .unwrap_or(true) // non-standard entry — preserve it
            });
            changed = true;
        }

        // Remove the hook_type key if now empty
        if json["hooks"][hook_type]
            .as_array()
            .map(|a| a.is_empty())
            .unwrap_or(false)
        {
            if let Some(hooks_obj) = json["hooks"].as_object_mut() {
                hooks_obj.remove(*hook_type);
            }
        }
    }

    if !changed {
        return Ok(false); // Nothing was vibestats-related
    }

    // Remove "hooks" key entirely if now empty
    if json["hooks"]
        .as_object()
        .map(|o| o.is_empty())
        .unwrap_or(false)
    {
        if let Some(root) = json.as_object_mut() {
            root.remove("hooks");
        }
    }

    // Write back with pretty-print (2-space indent, matching Claude Code's format)
    let new_content = serde_json::to_string_pretty(&json)
        .map_err(|e| format!("could not serialize settings.json: {e}"))?;
    std::fs::write(path, new_content + "\n")
        .map_err(|e| format!("could not write {}: {e}", path.display()))?;

    Ok(true)
}
```

**CRITICAL borrow-checker note:** The implementation above deliberately avoids holding a mutable borrow to `hooks_map` across the entire function. Instead it re-accesses `json["hooks"]` per-step. This is necessary because Rust does not allow a simultaneous `get_mut` borrow of a sub-field while also calling `as_object_mut()` on the parent. Do NOT refactor this into a single `let hooks_map = json.get_mut("hooks")` binding that spans the whole function — it will fail to compile.

**Calling remove_vibestats_hooks in `run()`:**

```rust
// Step 1: remove hooks
match settings_path() {
    None => {
        println!("vibestats: warning — HOME not set, skipping hook removal from settings.json");
    }
    Some(path) => match remove_vibestats_hooks(&path) {
        Err(e) => {
            println!("vibestats: warning — could not remove hooks from settings.json: {e}");
            // Continue — binary deletion and instructions still useful
        }
        Ok(false) => {
            println!("vibestats: no vibestats hooks found in settings.json (already clean)");
        }
        Ok(true) => {
            println!("vibestats: removed Stop and SessionStart hooks from ~/.claude/settings.json");
        }
    },
}
```

### Step 2 — Delete the Binary

The installer places the binary at `~/.local/bin/vibestats` (FR from installer epic, Story 6.1: "installing to `~/.local/bin/vibestats`").

```rust
// Step 2: delete binary
fn binary_path() -> Option<std::path::PathBuf> {
    std::env::var("HOME").ok().map(|h| {
        std::path::PathBuf::from(h)
            .join(".local")
            .join("bin")
            .join("vibestats")
    })
}

match binary_path() {
    None => {
        println!("vibestats: warning — HOME not set, skipping binary deletion");
    }
    Some(path) => match std::fs::remove_file(&path) {
        Ok(()) => {
            println!("vibestats: deleted binary at {}", path.display());
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            println!("vibestats: binary not found at {} (already removed?)", path.display());
        }
        Err(e) => {
            println!("vibestats: warning — could not delete binary at {}: {e}", path.display());
        }
    },
}
```

**Note:** The currently-running binary is being deleted. This is safe on Unix — the OS keeps the inode alive until the process exits. The binary deletion is the LAST file system operation; after this, `run()` only prints to stdout and returns.

### Step 3 — Print Manual Cleanup Instructions

```rust
// Step 3: manual steps
println!();
println!("vibestats: uninstall complete.");
println!();
println!("Optional manual cleanup steps:");
println!("  1. Delete your vibestats-data repo:");
println!("       gh repo delete <username>/vibestats-data --yes");
println!("  2. Remove the heatmap markers from your profile README:");
println!("       Delete the lines between <!-- vibestats-start --> and <!-- vibestats-end -->");
println!("  3. Remove vibestats config and logs:");
println!("       rm -rf ~/.config/vibestats");
```

### Stdout Output Contract

| Scenario | stdout |
|---|---|
| Hooks removed, binary deleted | `"vibestats: removed Stop and SessionStart hooks..."` + `"vibestats: deleted binary at ..."` + blank line + `"vibestats: uninstall complete."` + manual steps |
| No vibestats hooks found | `"vibestats: no vibestats hooks found in settings.json (already clean)"` + binary deletion line + instructions |
| settings.json not found | `"vibestats: no vibestats hooks found in settings.json (already clean)"` or similar — continue |
| Hook removal error (non-fatal) | `"vibestats: warning — could not remove hooks from settings.json: ..."` — continue to binary deletion |
| Binary not found | `"vibestats: binary not found at ... (already removed?)"` |
| Binary deletion error (non-fatal) | `"vibestats: warning — could not delete binary at ...:"` |
| HOME not set | Warnings for each skipped step, then print instructions |

### Error Handling Contract

| Failure | Fatal? | Behaviour |
|---|---|---|
| HOME not set | No | Print warning per step, skip that step, continue |
| settings.json not found | No | Print "not found/already clean", continue |
| settings.json malformed JSON | No | Print warning, skip hook removal, continue to binary deletion |
| settings.json write fails | No | Print warning, continue |
| Binary not found at path | No | Print "already removed?", continue |
| Binary delete fails | No | Print warning, continue |

**`commands/uninstall.rs` NEVER calls `std::process::exit`.** `main.rs` implicitly exits 0.

### File Structure

```
src/
├── main.rs               ← MODIFY: replace `println!("not yet implemented")` in Uninstall arm
├── commands/
│   ├── mod.rs            ← MODIFY: add `pub mod uninstall;`
│   ├── sync.rs           ← EXISTING — not touched
│   ├── status.rs         ← EXISTING — not touched
│   ├── machines.rs       ← EXISTING — not touched
│   ├── auth.rs           ← EXISTING — not touched
│   └── uninstall.rs      ← NEW — this story's implementation
├── config.rs             ← EXISTING — NOT used (config.toml left in place)
├── checkpoint.rs         ← EXISTING — NOT used
└── ...                   ← all other modules untouched
```

**Do NOT read config.toml** — `uninstall` does not need the oauth token, machine_id, or vibestats_data_repo. All operations are purely local (settings.json, binary file).

### Existing Crates (No New Dependencies Allowed)

`serde_json` is already a dependency — use `serde_json::Value` for JSON manipulation.

```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
ureq = "2.10"
toml = "0.8"
```

Do NOT add any new crates. `serde_json` already in `Cargo.toml` handles all JSON parsing and serialization.

### `serde_json` JSON Manipulation Pattern (No New Crates)

Use `serde_json::Value` for in-place JSON mutation. This is the same crate used throughout the codebase (machines.rs uses `serde_json::from_str::<serde_json::Value>`). No `use` import needed in function body — reference the type fully as `serde_json::Value`. Pattern:

```rust
let mut json: serde_json::Value = serde_json::from_str(&content)?;
// mutate json["hooks"] in place using indexing
let new_content = serde_json::to_string_pretty(&json)?;
```

`serde_json::to_string_pretty` produces 2-space indentation — consistent with Claude Code's settings.json format.

**Clippy note:** Do NOT use `.map(|o| o.remove("hooks"))` on an `Option<&mut Map>` return value — Clippy will flag the unused `Option<Option<Value>>` return. Use `if let Some(obj) = ..` instead (as shown in the implementation above).

**Index operator note:** `json["hooks"]["Stop"]` returns `serde_json::Value::Null` (not a panic) when the key doesn't exist. `as_array_mut()` on Null returns `None`. This is the fail-safe access pattern used throughout.

### `checkpoint_path()` — Do NOT Copy for This Story

Unlike `auth.rs` and `machines.rs`, `uninstall.rs` does NOT need `checkpoint_path()`. Do not copy it. This story only needs `settings_path()` and `binary_path()`.

### `#![allow(dead_code)]` — Do NOT Add

Unlike early modules, `uninstall.rs` should not have `#![allow(dead_code)]`. All functions will be used.

### Worktree / Cargo Isolation

Run all verification from the **repo root** (not from inside the worktree):

```bash
cargo build
cargo test
cargo clippy --all-targets -- -D warnings
```

### Architecture Constraints Summary

| Constraint | Source | Impact on This Story |
|---|---|---|
| Exit 0 always | NFR10 | Never call `std::process::exit`; return `()` from `run()` |
| Silent failure contract | NFR10/NFR11 | On errors: print warning to stdout and continue — never propagate |
| No async runtime | architecture.md | All code synchronous |
| No new crates | Story scope | Use `serde_json` (already in Cargo.toml) for JSON |
| snake_case filenames | architecture.md | `src/commands/uninstall.rs` |
| Co-located unit tests | architecture.md | `#[cfg(test)]` module inside `src/commands/uninstall.rs` |
| Binary install path | Epic 6.1 (FR from installer) | `~/.local/bin/vibestats` |
| PR closes GH issue | epics.md | PR description must include `Closes #25` |
| Preserve other settings | FR37, AC #3 | Surgical removal — only vibestats hook entries |

### Hook JSON Structure Reference

The exact hook entries written by the vibestats installer (for verification during removal):

**Stop hook** (async: true per NFR1):
```json
{ "hooks": [{ "type": "command", "command": "vibestats sync", "async": true }] }
```

**SessionStart hook**:
```json
{ "hooks": [{ "type": "command", "command": "vibestats session-start" }] }
```

Detection: any hook object where `command == "vibestats"` or `command.starts_with("vibestats ")`.

### Anti-Patterns to Prevent

- Do NOT call `Config::load()` or `Config::load_or_exit()` — no config needed for uninstall
- Do NOT call `github_api.rs` — uninstall is entirely local
- Do NOT call `std::process::exit` anywhere in `commands/uninstall.rs`
- Do NOT add new crates (`regex`, `serde_yaml`, etc.) — use `serde_json` already in Cargo.toml
- Do NOT use `unwrap()` or `expect()` in non-test code
- Do NOT delete `~/.config/vibestats/` — leave config and logs for user (instructions cover manual removal)
- Do NOT delete the entire `hooks` key blindly — other apps may use it; only remove vibestats entries
- Do NOT overwrite settings.json if nothing was removed (check `changed` flag before writing)
- Do NOT write binary path to stdout before deleting (self-delete is safe on Unix, inode stays alive)
- Do NOT add a second `[workspace]` to `Cargo.toml`
- Do NOT copy `checkpoint_path()` from other modules — not needed in this story

### Previous Story Learnings

From Story 4.3 (`commands/auth.rs`):
- `mod commands;` is already in `main.rs` — do NOT add again; only add the `Uninstall` arm wiring
- `commands/mod.rs` has `pub mod auth;`, `pub mod machines;`, `pub mod sync;`, `pub mod status;` — add `pub mod uninstall;`
- `std::process::exit` must never be called inside modules — only by `main.rs`
- Use `std::env::temp_dir()` for test temp files (not `/tmp` directly)
- `cargo clippy --all-targets -- -D warnings` (with `--all-targets`) catches all targets including test code
- PRs must include `Closes #25` in the PR description

From Story 4.2 (`commands/machines.rs`):
- `serde_json::Value` used for JSON parsing — same approach for settings.json
- Pattern: `serde_json::from_str::<serde_json::Value>(&content)` then drill into with `["key"]`
- `Config::load_or_exit()` is available but do NOT use in `uninstall.rs` — no config needed

From Story 3.2 (`hooks.rs`):
- Settings.json hook format is `{ "hooks": [{ "type": "command", "command": "vibestats sync", "async": true }] }` for Stop
- Settings.json hook format is `{ "hooks": [{ "type": "command", "command": "vibestats session-start" }] }` for SessionStart
- The outer array entry has a `"hooks"` key (inner array) and optional `"matcher"` key

### Project Structure Notes

- New files: `src/commands/uninstall.rs`
- Modified files: `src/commands/mod.rs` (add `pub mod uninstall;`), `src/main.rs` (replace `println!("not yet implemented")` in Uninstall arm)
- No other files modified

### References

- Story requirements: [Source: _bmad-output/planning-artifacts/epics.md#Story 4.4]
- FR37 (vibestats uninstall): [Source: _bmad-output/planning-artifacts/epics.md#Functional Requirements]
- NFR10 (exit 0): [Source: _bmad-output/planning-artifacts/epics.md#NonFunctional Requirements]
- Binary install path `~/.local/bin/vibestats`: [Source: _bmad-output/planning-artifacts/epics.md#Story 6.1]
- Hook JSON format (Stop + SessionStart): [Source: _bmad-output/implementation-artifacts/3-2-implement-stop-hook-integration.md#Hook Configuration Reference]
- settings.json hook format: [Source: _bmad-output/implementation-artifacts/3-3-implement-sessionstart-hook-integration.md]
- serde_json usage pattern: [Source: src/commands/machines.rs]
- Module responsibility boundaries: [Source: _bmad-output/planning-artifacts/architecture.md#Architectural Boundaries]
- Module file structure: [Source: _bmad-output/planning-artifacts/architecture.md#Complete Project Directory Structure]
- GH Issue: #25

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

None.

### Completion Notes List

None yet.

### File List

None yet.

## Change Log

- 2026-04-11: Story created for Story 4.4 — vibestats uninstall command implementation guide.
