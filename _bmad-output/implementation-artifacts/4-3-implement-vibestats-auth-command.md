# Story 4.3: Implement vibestats auth Command

Status: done

<!-- GH Issue: #24 | Epic: #4 | PR must include: Closes #24 -->

## Story

As a developer,
I want `vibestats auth` to refresh my GitHub OAuth token and update the Actions secret,
so that a revoked or expired token can be fixed with one command on any machine.

## Acceptance Criteria

1. **Given** the user runs `vibestats auth` **When** it executes **Then** it calls `gh auth token` to obtain a fresh token and writes it to `~/.config/vibestats/config.toml` with permissions `600` (FR36, NFR6)

2. **Given** a fresh token is obtained **When** `vibestats auth` proceeds **Then** it updates the `VIBESTATS_TOKEN` Actions secret in `vibestats-data` via `gh secret set` (FR36)

3. **Given** the auth refresh completes **When** the user runs `vibestats status` afterwards **Then** auth shows "Auth: OK" and `checkpoint.toml` `auth_error` is cleared (FR40)

## Tasks / Subtasks

- [x] Task 1: Add `pub mod auth;` to `src/commands/mod.rs` (AC: all)
  - [x] Open `src/commands/mod.rs` and add `pub mod auth;` — do NOT add stubs for other 4.x commands not yet implemented

- [x] Task 2: Implement `src/commands/auth.rs` (AC: #1, #2, #3)
  - [x] Create `src/commands/auth.rs`
  - [x] Implement `pub fn run()` — the entry point called from `main.rs`
  - [x] Step 1 — obtain fresh token: run `gh auth token` via `std::process::Command`, capture stdout, trim whitespace → `new_token`
  - [x] Step 2 — update `config.toml`: call `Config::load()` to get existing config (for `machine_id` and `vibestats_data_repo`), update `oauth_token` field to `new_token`, call `config.save()` — this enforces `600` perms automatically (AC #1, NFR6)
  - [x] Step 3 — update `VIBESTATS_TOKEN` secret: run `gh secret set VIBESTATS_TOKEN --repo <vibestats_data_repo> --body <new_token>` via `std::process::Command`, where `vibestats_data_repo` comes from the loaded config (format `"username/vibestats-data"`) (AC #2)
  - [x] Step 4 — clear `auth_error`: call `checkpoint_path()` helper, load `Checkpoint`, call `cp.clear_auth_error()`, save checkpoint (AC #3)
  - [x] Print `"vibestats: auth complete"` to stdout on success (AC #3 — user knows token is refreshed)
  - [x] On any failure (gh not found, gh auth token fails, save fails): print descriptive error message to stdout and return — do NOT call `std::process::exit` (main.rs handles exit)

- [x] Task 3: Wire `Commands::Auth` in `main.rs` (AC: #1, #2, #3)
  - [x] In `main.rs` `match cli.command` arm for `Commands::Auth`: replace `println!("not yet implemented")` with `commands::auth::run();`
  - [x] Verify `mod commands;` is already present in `main.rs` (added in Story 3.4 — do NOT add again)

- [x] Task 4: Write co-located unit tests (AC: #1, #2, #3)
  - [x] `#[cfg(test)]` module inside `src/commands/auth.rs`
  - [x] Test that `checkpoint_path()` returns `Some(path)` when `HOME` is set and the path ends with `.config/vibestats/checkpoint.toml`
  - [x] Test that `checkpoint_path()` returns `None` when `HOME` is unset (temporarily unset for test, then restore)
  - [x] Test that running `gh` with a non-existent binary path (e.g., `"/nonexistent/gh"`) causes the subprocess to fail — verify the error branch logic by asserting `Command::new("/nonexistent/gh").args(["auth","token"]).output().is_err()` directly in test
  - [x] Run `cargo test` from repo root — must pass with 0 failures
  - [x] Run `cargo clippy --all-targets -- -D warnings` — must produce 0 warnings
  - [x] Run `cargo build` — must produce 0 errors

## Dev Notes

### Module Responsibility Summary

`commands/auth.rs` is the CLI handler for `vibestats auth`. It orchestrates three external calls (gh CLI, config write, checkpoint write):

| Step | Action | Module Used |
|---|---|---|
| 1 | `gh auth token` → fresh OAuth token | `std::process::Command` |
| 2 | Write token to `config.toml` with 600 perms | `crate::config::Config::save()` |
| 3 | `gh secret set VIBESTATS_TOKEN --repo <repo> --body <token>` | `std::process::Command` |
| 4 | Clear `auth_error` in `checkpoint.toml` | `crate::checkpoint::Checkpoint` |

`gh` CLI is the **only** authentication provider (FR38). No custom GitHub API calls for token management — all auth-related subprocess calls go through `gh`.

### `commands/auth.rs` Entry Point Signature

```rust
pub fn run() {
    // Orchestrates gh auth token → config.save() → gh secret set → checkpoint.clear_auth_error()
    // NEVER calls std::process::exit — main.rs handles exit
}
```

### How `main.rs` calls this (already has the stub — just replace the println!)

```rust
Commands::Auth => commands::auth::run(),
```

### `gh auth token` — Obtaining the Fresh Token

```rust
let new_token = match std::process::Command::new("gh")
    .args(["auth", "token"])
    .output()
{
    Err(e) => {
        println!("vibestats: auth failed — could not run 'gh': {e}");
        println!("Ensure 'gh' CLI is installed and accessible in PATH.");
        return;
    }
    Ok(out) if !out.status.success() => {
        let stderr = String::from_utf8_lossy(&out.stderr);
        println!(
            "vibestats: auth failed — 'gh auth token' returned non-zero: {}",
            stderr.trim()
        );
        println!("Run 'gh auth login' first, then retry 'vibestats auth'.");
        return;
    }
    Ok(out) => {
        let token = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if token.is_empty() {
            println!("vibestats: auth failed — 'gh auth token' returned empty token.");
            println!("Run 'gh auth login' first, then retry 'vibestats auth'.");
            return;
        }
        token
    }
};
// new_token is now the trimmed, non-empty token string
```

### `Config::load()` and `Config::save()` — Token Update

`config.rs` already has `Config::load()` (returns `Result<Config, String>`) and `Config::save()` (writes `~/.config/vibestats/config.toml` with `0o600` at creation, re-asserts `600` on existing file). Do NOT reinvent permission logic — just call `save()`.

```rust
let mut config = match crate::config::Config::load() {
    Ok(c) => c,
    Err(e) => {
        println!("vibestats: auth failed — could not load config: {e}");
        return;
    }
};
config.oauth_token = new_token.clone(); // clone because new_token is reused in gh secret set
if let Err(e) = config.save() {
    println!("vibestats: auth failed — could not save config: {e}");
    return;
}
```

### `gh secret set` — Update VIBESTATS_TOKEN

`config.vibestats_data_repo` is already the full repo slug (e.g., `"username/vibestats-data"`). Pass it directly to `--repo`:

Run `gh secret set`:

```rust
let secret_result = std::process::Command::new("gh")
    .args([
        "secret", "set", "VIBESTATS_TOKEN",
        "--repo", &config.vibestats_data_repo,
        "--body", &new_token,
    ])
    .output();

match secret_result {
    Err(e) => {
        println!("vibestats: token saved locally but could not update VIBESTATS_TOKEN secret: {e}");
        println!("Run manually: gh secret set VIBESTATS_TOKEN --repo {} --body <token>", config.vibestats_data_repo);
        // Continue — local token is updated; don't abort checkpoint clear
    }
    Ok(out) if !out.status.success() => {
        let stderr = String::from_utf8_lossy(&out.stderr);
        println!("vibestats: token saved locally but 'gh secret set' failed: {stderr}");
        println!("Run manually: gh secret set VIBESTATS_TOKEN --repo {} --body <token>", config.vibestats_data_repo);
        // Continue — local token is updated
    }
    Ok(_) => {} // success
}
```

**Important:** If `gh secret set` fails, still proceed to clear `auth_error` and print success — the local config token is refreshed, which is the primary goal. The secret can be re-set manually.

### Checkpoint `auth_error` Clear

Use the same `checkpoint_path()` helper pattern from `sync.rs`:

```rust
fn checkpoint_path() -> Option<std::path::PathBuf> {
    std::env::var("HOME").ok().map(|h| {
        std::path::PathBuf::from(h)
            .join(".config")
            .join("vibestats")
            .join("checkpoint.toml")
    })
}
```

Then load and clear:

```rust
if let Some(cp_path) = checkpoint_path() {
    let mut cp = crate::checkpoint::Checkpoint::load(&cp_path);
    cp.clear_auth_error();
    if let Err(e) = cp.save(&cp_path) {
        // Non-fatal — auth is refreshed; log but do not abort
        println!("vibestats: auth complete (note: could not clear auth_error flag: {e})");
        return;
    }
}
println!("vibestats: auth complete");
```

### Stdout Output Contract

| Scenario | stdout |
|---|---|
| All steps succeed | `"vibestats: auth complete"` |
| `gh` not in PATH | `"vibestats: auth failed — could not run 'gh': ..."` + remediation |
| `gh auth token` fails | `"vibestats: auth failed — 'gh auth token' returned non-zero: ..."` + remediation |
| `gh auth token` returns empty | `"vibestats: auth failed — 'gh auth token' returned empty token."` + remediation |
| Config load fails | `"vibestats: auth failed — could not load config: ..."` |
| Config save fails | `"vibestats: auth failed — could not save config: ..."` |
| `gh secret set` fails (non-fatal) | `"vibestats: token saved locally but ..."` + manual remediation hint, then continues to `auth complete` |
| Checkpoint save fails (non-fatal) | `"vibestats: auth complete (note: could not clear auth_error flag: ...)"` |

### Error Handling Contract

| Failure | Fatal? | Behaviour |
|---|---|---|
| `gh` not found / exec error | Yes | Print error + remediation, return |
| `gh auth token` non-zero exit | Yes | Print error + remediation, return |
| Empty token returned | Yes | Print error + remediation, return |
| `Config::load()` error | Yes | Print error, return |
| `Config::save()` error | Yes | Print error, return |
| `gh secret set` failure | No | Print warning + manual command hint, continue |
| `Checkpoint::save()` failure | No | Print note, return (token still refreshed) |

**`commands/auth.rs` NEVER calls `std::process::exit`.** `main.rs` implicitly exits 0 after the command returns.

### File Structure

```
src/
├── main.rs               ← MODIFY: replace `println!("not yet implemented")` in Auth arm
├── commands/
│   ├── mod.rs            ← MODIFY: add `pub mod auth;`
│   ├── sync.rs           ← EXISTING — not touched
│   └── auth.rs           ← NEW — this story's implementation
├── config.rs             ← EXISTING — used (Config::load, Config::save)
├── checkpoint.rs         ← EXISTING — used (Checkpoint::load, clear_auth_error, save)
├── github_api.rs         ← EXISTING — NOT used (auth uses gh CLI subprocess, not HTTP)
└── ...                   ← all other modules untouched
```

**Do NOT create stub files for `status.rs`, `machines.rs`, `uninstall.rs`** — they will be added in their respective stories.

### Existing Crates (No New Dependencies Allowed)

All required functionality uses `std` only — no new crates needed. The `gh` CLI is invoked as a subprocess via `std::process::Command`.

```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
ureq = "2.10"
toml = "0.8"
```

Do NOT add any new crates. All auth flows use the `gh` CLI as the authentication provider (FR38).

### `checkpoint_path()` — Do NOT Import from `sync.rs`

`checkpoint_path()` is a private function in `sync.rs`. Copy the same implementation (3 lines) into `auth.rs` — do NOT make it public or move it to a shared module. This matches the repo pattern (each module owns its own helpers).

### `#![allow(dead_code)]` Audit

`src/config.rs` has `#![allow(dead_code)]` at the top. Once `commands/auth.rs` calls `Config::load()` and `Config::save()`, these functions are called from user code. However, the `#![allow(dead_code)]` should NOT be removed in this story — it was added to suppress warnings for other `config.rs` functions not yet called (e.g., `generate_machine_id`). Leave it as-is.

### Worktree / Cargo Isolation

The worktree is nested inside the main repo. Run all verification from the **repo root** (not from inside the worktree):

```bash
cargo build
cargo test
cargo clippy --all-targets -- -D warnings
```

### Architecture Constraints Summary

| Constraint | Source | Impact on This Story |
|---|---|---|
| `gh` CLI as auth provider | FR38 | Use `std::process::Command::new("gh")` — no direct OAuth HTTP |
| Token storage at `config.toml` with `600` perms | FR39, NFR6 | Use `Config::save()` — already enforces permissions atomically |
| VIBESTATS_TOKEN update | FR36 | `gh secret set VIBESTATS_TOKEN --repo <vibestats_data_repo>` |
| Clear `auth_error` flag | FR40 | `Checkpoint::clear_auth_error()` + save after token refresh |
| Exit 0 always | NFR10 | Never call `std::process::exit`; return `()` from `run()` |
| No async runtime | architecture.md | All code synchronous; no `tokio`, no `async fn` |
| No new crates | Story scope | std-only subprocess calls |
| snake_case filenames | architecture.md | `src/commands/auth.rs` |
| Co-located unit tests | architecture.md | `#[cfg(test)]` module inside `src/commands/auth.rs` |
| PR closes GH issue | epics.md | PR description must include `Closes #24` |

### Anti-Patterns to Prevent

- Do NOT call `github_api.rs` for any part of the auth flow — `gh` CLI handles all OAuth operations
- Do NOT reinvent `600` permission logic — `Config::save()` already handles it atomically
- Do NOT call `std::process::exit` anywhere in `commands/auth.rs`
- Do NOT add new crates (`reqwest`, `oauth2`, etc.) — use `gh` CLI subprocess only
- Do NOT make `checkpoint_path()` from `sync.rs` public — copy the 3-line helper instead
- Do NOT create stub files for `status.rs`, `machines.rs`, `uninstall.rs`
- Do NOT add a second `[workspace]` to `Cargo.toml`
- Do NOT use `unwrap()` or `expect()` in non-test code
- Do NOT print the raw token to stdout at any point (security)
- Do NOT write the token to any file other than `config.toml` via `Config::save()`

### Previous Story Learnings

From Story 3.4 (`commands/sync.rs`):
- `mod commands;` is already in `main.rs` — do NOT add it again; only add the `Auth` arm wiring
- `commands/mod.rs` exists with `pub mod sync;` — add `pub mod auth;` to it
- `std::process::exit` must never be called inside modules — only by `main.rs`
- Use `std::env::temp_dir()` for test temp files (not `/tmp` directly)
- `cargo clippy --all-targets -- -D warnings` (with `--all-targets`) catches all targets including test code
- PRs must include `Closes #24` in the PR description

From Story 2.1 (`config.rs`):
- `Config::load()` returns `Result<Config, String>` — handle both arms
- `Config::save()` writes with `mode(0o600)` at creation AND re-asserts 600 via `set_permissions_600` — no need for explicit chmod in `auth.rs`
- `Config::load_or_exit()` exists but do NOT use it in `auth.rs` — it calls `std::process::exit(0)` internally, which breaks the no-exit rule; use `Config::load()` and handle the error manually

From Story 3.1 (`sync.rs`):
- `checkpoint_path()` returns `Option<PathBuf>` — handle `None` case (HOME not set) gracefully
- `Checkpoint::load(path)` returns a valid (default) checkpoint if file doesn't exist — never returns Err
- `Checkpoint::save(path)` returns `Result` — handle the error

### Project Structure Notes

- New files: `src/commands/auth.rs`
- Modified files: `src/commands/mod.rs` (add `pub mod auth;`), `src/main.rs` (replace `println!("not yet implemented")` in Auth arm)
- No other files modified

### References

- Story requirements: [Source: _bmad-output/planning-artifacts/epics.md#Story 4.3]
- FR36 (vibestats auth refresh): [Source: _bmad-output/planning-artifacts/epics.md#Functional Requirements]
- FR38 (gh CLI as auth provider): [Source: _bmad-output/planning-artifacts/epics.md#Functional Requirements]
- FR39 (config.toml token storage): [Source: _bmad-output/planning-artifacts/epics.md#Functional Requirements]
- FR40 (clear auth_error on refresh): [Source: _bmad-output/planning-artifacts/epics.md#Functional Requirements]
- NFR6 (token storage 600 perms): [Source: _bmad-output/planning-artifacts/epics.md#NonFunctional Requirements]
- NFR10 (exit 0): [Source: _bmad-output/planning-artifacts/epics.md#NonFunctional Requirements]
- Auth token lifecycle: [Source: _bmad-output/planning-artifacts/architecture.md#Authentication & Security]
- Two independent tokens table: [Source: _bmad-output/planning-artifacts/architecture.md#Authentication & Security]
- vibestats auth refresh: [Source: _bmad-output/planning-artifacts/architecture.md#Authentication & Security]
- Module responsibility boundaries: [Source: _bmad-output/planning-artifacts/architecture.md#Architectural Boundaries]
- Module file structure: [Source: _bmad-output/planning-artifacts/architecture.md#Complete Project Directory Structure]
- Config::save() 600-perm implementation: [Source: src/config.rs#write_file_mode_600]
- Checkpoint::clear_auth_error(): [Source: src/checkpoint.rs#clear_auth_error]
- checkpoint_path() pattern: [Source: src/sync.rs#checkpoint_path]
- GH Issue: #24

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

None.

### Completion Notes List

- Implemented `src/commands/auth.rs` with `pub fn run()` orchestrating: `gh auth token` → `Config::load/save` → `gh secret set` → `Checkpoint::clear_auth_error`
- Added `pub mod auth;` to `src/commands/mod.rs`
- Wired `Commands::Auth => commands::auth::run()` in `src/main.rs`
- All error paths are non-panicking and never call `std::process::exit`
- `gh secret set` failures are non-fatal; checkpoint clear failure is also non-fatal
- 3 unit tests added: `checkpoint_path_returns_some_when_home_is_set`, `checkpoint_path_returns_none_when_home_unset`, `nonexistent_gh_binary_causes_error`
- `cargo test`: 104 passed, 0 failed
- `cargo clippy --all-targets -- -D warnings`: 0 warnings
- `cargo build`: 0 errors

### File List

- `src/commands/auth.rs` (new)
- `src/commands/mod.rs` (modified — added `pub mod auth;`)
- `src/main.rs` (modified — replaced `println!("not yet implemented")` in `Commands::Auth` arm)

## Review Findings

**Reviewer:** Claude Sonnet 4.6 | **Date:** 2026-04-13 | **Story:** 4.3

### Blind Hunter Pass

**Focus:** Security vulnerabilities, token leakage, permission issues, injection risks

| # | Finding | Severity | Status |
|---|---------|----------|--------|
| BH-1 | S1 — Token is never passed as `--body` argument; implementation correctly uses `--body-file -` with stdin pipe | P0 | Clean |
| BH-2 | S2 — No `println!` or error path prints the raw token value; error messages contain only `{e}` or `{stderr}` (not `config.oauth_token`) | P0 | Clean |
| BH-3 | S3 — `Config::save()` calls `write_file_mode_600` (sets `mode(0o600)` at creation) then `set_permissions_600` (re-asserts on existing files) — both at-creation and pre-existing file permissions are enforced | P0 | Clean |
| BH-4 | S4 — `Config::save()` is in-place truncate (`OpenOptions::write(true).truncate(true)`), not atomic temp-file+rename. A process crash during write could leave a partially written `config.toml`. Accepted as P2 since: (a) the file is small, (b) any partial write is detected at next `Config::load()` via TOML parse error, and (c) adding atomic-write to config.rs is out of scope for this story | P2 | Deferred |
| BH-5 | S5 — `drop(child.stdin.take())` is present before `wait_with_output()` — no deadlock risk | P0 | Clean |
| BH-6 | S6 — `config.oauth_token` holds raw token in heap memory after write; Rust does not zero-on-drop. Accepted as P2 — token lifetime is process-scoped and this is standard Rust behavior | P2 | Deferred |

### Edge Case Hunter Pass

**Focus:** Boundary conditions, missing file states, race conditions, malformed inputs

| # | Finding | Severity | Status |
|---|---------|----------|--------|
| EH-1 | E1 — `gh auth token` whitespace-only: `String::from_utf8_lossy(&out.stdout).trim().to_string()` trims first, then `is_empty()` checks. Correct ordering — whitespace-only outputs are caught as empty | P0 | Clean |
| EH-2 | E2 — `Config::load()` when `config.toml` does not exist: returns `Err` with message "Config file not found at ...". `auth.rs` maps this to `println!("vibestats: auth failed — could not load config: {e}")` with actionable message | P1 | Clean |
| EH-3 | E3 — `checkpoint_path()` returns `None` when HOME unset: the `if let Some(cp_path) = checkpoint_path()` branch skips checkpoint clear silently (no message printed). This is safe — auth token refresh succeeded. Minor UX gap: no message tells the user checkpoint was skipped | P2 | Deferred |
| EH-4 | E4 — `Checkpoint::load()` on missing file returns `Default`: `clear_auth_error()` on a default checkpoint sets `auth_error = false` (already false). Idempotent and safe | P0 | Clean |
| EH-5 | E5 — `gh` binary not in PATH: error path prints "could not run 'gh': {e}" + "Ensure 'gh' CLI is installed and accessible in PATH." Actionable remediation provided | P0 | Clean |
| EH-6 | `stdin` piping: `ok_or_else` converts `stdin.as_mut()` returning `None` into a proper `io::Error` instead of using `expect()`. No non-test `expect()`/`unwrap()` present in `auth.rs` | P0 | Clean |

### Acceptance Auditor Pass

**Focus:** All ACs verified against actual implementation

| AC | Verified | Notes |
|----|----------|-------|
| AC #1 | Yes | `gh auth token` → `Config::load()` → set `config.oauth_token = new_token` → `config.save()`. `Config::save()` calls `write_file_mode_600` which opens with `mode(0o600)` and re-asserts 600 on existing files. FR36, NFR6 satisfied |
| AC #2 | Yes | `gh secret set VIBESTATS_TOKEN --repo <vibestats_data_repo> --body-file -` with token fed via stdin pipe. FR36 satisfied. Failure is non-fatal (local token still refreshed) with manual-runbook hint printed |
| AC #3 | Yes | `Checkpoint::clear_auth_error()` called and `cp.save()` called on success. Non-fatal if checkpoint save fails. `vibestats: auth complete` printed on success path |
| NFR10 | Yes | No `std::process::exit` call anywhere in `src/commands/auth.rs` (confirmed by grep) |

### Fixes Applied

No P0 or P1 findings. No source code changes required.

### Summary

`src/commands/auth.rs` is a clean, security-conscious implementation. The two key security properties — stdin-pipe token delivery and 600-perm config file — are correctly implemented and verified. Error handling follows the story's fatal/non-fatal contract precisely. No `unwrap()`/`expect()` in non-test code. Three P2 observations are deferred: (1) non-atomic config.toml write (acceptable given small file size and error detection on load), (2) no in-memory zeroing of the token (standard Rust behavior), (3) no message when checkpoint skip occurs due to unset HOME (minor UX gap). All ACs and NFR10 verified. **Recommendation: APPROVE.**

## Change Log

- 2026-04-11: Implemented `vibestats auth` command (Story 4.3) — `gh auth token` → config.toml update → `gh secret set` → checkpoint auth_error clear. All ACs satisfied. 104 tests pass.
- 2026-04-13: Retrospective code review completed (Story 9.2). Three-pass review: Blind Hunter, Edge Case Hunter, Acceptance Auditor. All P0/P1 checks pass. Three P2 observations deferred. Status updated to done.
