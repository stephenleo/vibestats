# Story 4.2: Implement vibestats machines list and machines remove

Status: done

<!-- GH Issue: #23 | Epic: #4 | PR must include: Closes #23 -->

## Story

As a developer,
I want to list and remove machines from vibestats-data,
so that I can manage which machines contribute to my heatmap.

## Acceptance Criteria

1. **Given** the user runs `vibestats machines list` **When** it executes **Then** it prints all machines from `registry.json` with `machine_id`, `hostname`, `status`, and `last_seen` (FR34)

2. **Given** the user runs `vibestats machines remove <id>` (no flag) **When** it executes **Then** it sets `status = "retired"` in `registry.json` via GitHub Contents API PUT, preserving all historical Hive partition files (default retire)

3. **Given** the user runs `vibestats machines remove <id> --purge-history` **When** it executes **Then** it prompts `"This will permanently remove all historical data for <hostname>. Continue? (y/N)"` and on confirmation sets `status = "purged"` and bulk-deletes all Hive partition files for that `machine_id` (FR35)

4. **Given** the user runs `vibestats machines remove <id>` on the **current machine** (self-retire) **When** it executes **Then** it updates both `registry.json` (remote) and `checkpoint.toml` (local) `machine_status = "retired"` in the same operation — immediately effective (architecture constraint)

## Tasks / Subtasks

- [x] Task 1: Add `pub mod machines;` to `src/commands/mod.rs` (AC: all)
  - [x] Open `src/commands/mod.rs` — currently only `pub mod sync;`
  - [x] Append `pub mod machines;` — final file has exactly two lines

- [x] Task 2: Add `delete_file` to `src/github_api.rs` (AC: #3)
  - [x] Add `pub fn delete_file(&self, path: &str) -> Result<(), GithubApiError>` to `GithubApi` impl block
  - [x] Implementation: GET SHA first (reuse `get_file_sha`), then DELETE via GitHub Contents API DELETE endpoint
  - [x] DELETE body: `{ "message": "vibestats: remove machine data", "sha": "<sha>" }` — sha is required for DELETE
  - [x] If file returns 404 (already deleted): treat as success (`Ok(())`) — idempotent
  - [x] Wrap both GET-SHA and DELETE in `with_retry` — same 3-attempt exponential backoff as PUT
  - [x] Add inner function `delete_file_inner(token, repo, path, sha)` returning `Result<(), ureq::Error>` (same pattern as `put_file_inner`)
  - [x] **Do NOT remove** `#![allow(dead_code)]` from `src/github_api.rs` — `get_file_sha` and `list_directory` (if added) are not yet called from all upcoming stories; keep the allow until all public API methods are fully exercised
  - [x] Add unit tests for `delete_file` body construction (parallel to `test_put_body_*` tests)

- [x] Task 3: Add `list_directory` to `src/github_api.rs` (AC: #3 — purge-history remote machine bulk delete)
  - [x] Add `pub fn list_directory(&self, path: &str) -> Result<Vec<String>, GithubApiError>` to `GithubApi` impl block
  - [x] Use GitHub Contents API GET on a directory path — returns JSON array of objects with `"type"` and `"path"` fields
  - [x] Return only `"file"` type entries' `"path"` values; skip `"dir"` entries (caller handles recursion)
  - [x] GitHub Contents API returns up to 1000 entries per directory call — for a machine with many years of data, the year-level directory has ≤52 entries (one per month), month-level ≤31 entries (one per day), so pagination is not a concern at the leaf level; each `list_directory` call covers one month or day directory
  - [x] **Self-purge preferred path** (same machine, no listing needed): build Hive paths deterministically from checkpoint date hashes — see Dev Notes for the preferred path enumeration strategy
  - [x] Add `list_directory_inner` helper returning `Result<Vec<String>, ureq::Error>` — same pattern as other inner functions
  - [x] Note: `list_directory` is only needed for remote machine purge; self-purge uses checkpoint date hashes instead

- [x] Task 4: Implement `src/commands/machines.rs` (AC: #1, #2, #3, #4)
  - [x] Create file `src/commands/machines.rs`
  - [x] Implement `pub fn list()` — entry point from `main.rs` for `vibestats machines list`
  - [x] Implement `pub fn remove(machine_id: &str, purge_history: bool)` — entry point from `main.rs` for `vibestats machines remove`

  - [x] **`list()` implementation:**
    - [x] Call `Config::load_or_exit()` to get config
    - [x] Construct `GithubApi::new(&config.oauth_token, &config.vibestats_data_repo)`
    - [x] Call `api.get_file_content("registry.json")`
    - [x] If `Ok(None)`: print `"vibestats: no machines registered"` and return
    - [x] If `Ok(Some(content))`: parse as `serde_json::Value`, iterate `json["machines"].as_array()`
    - [x] Print each machine as: `"  <machine_id>  <hostname>  <status>  <last_seen>"` (tab-separated or aligned)
    - [x] If parse fails or `machines` key missing: print `"vibestats: registry.json is malformed"` and return
    - [x] If `Err(e)`: log via `logger::error` and print `"vibestats: failed to fetch registry — check vibestats.log"`, return

  - [x] **`remove(machine_id, purge_history)` implementation:**
    - [x] Call `Config::load_or_exit()` to get config
    - [x] Construct `GithubApi::new(&config.oauth_token, &config.vibestats_data_repo)`
    - [x] GET `registry.json` via `api.get_file_content("registry.json")`
    - [x] If `Ok(None)`: print `"vibestats: no machines registered"` and return
    - [x] If `Err(e)`: log via `logger::error`, print error message, return
    - [x] Parse registry JSON; find the machine with matching `machine_id`
    - [x] If machine not found: print `"vibestats: machine '<id>' not found in registry"` and return
    - [x] Extract `hostname` from found machine entry (needed for purge confirmation message)
    - [x] **Default retire path** (`purge_history == false`):
      - [x] Set `status = "retired"` for the matching machine in the JSON
      - [x] Serialize updated JSON back to string (use `serde_json::to_string_pretty`)
      - [x] Call `api.put_file("registry.json", &updated_json)` to update remote
      - [x] If `machine_id == config.machine_id` (self-retire): load local checkpoint, set `machine_status = "retired"`, save
      - [x] Print `"vibestats: machine '<id>' retired"` on success
    - [x] **Purge path** (`purge_history == true`):
      - [x] Print confirmation prompt: `"This will permanently remove all historical data for <hostname>. Continue? (y/N): "`
      - [x] Read line from stdin; accept only `"y"` or `"Y"` as confirmation — anything else aborts
      - [x] If aborted: print `"vibestats: purge cancelled"` and return
      - [x] Set `status = "purged"` for the machine in registry JSON
      - [x] Call `api.put_file("registry.json", &updated_json)`
      - [x] Enumerate and delete all Hive partition files for this machine (see Dev Notes for strategy)
      - [x] If `machine_id == config.machine_id` (self-purge): set local `checkpoint.toml` `machine_status = "purged"`
      - [x] Print `"vibestats: machine '<id>' purged — N file(s) deleted"` on completion

- [x] Task 5: Wire `machines` commands into `main.rs` (AC: #1, #2, #3)
  - [x] In `main.rs`, replace the `MachinesSubcommand::List` arm's `println!("not yet implemented")` with `commands::machines::list()`
  - [x] Replace the `MachinesSubcommand::Remove { machine_id: _ }` arm: change `machine_id: _` to `machine_id`, add `purge_history` arg to the `Remove` variant, call `commands::machines::remove(&machine_id, purge_history)`
  - [x] Add `--purge-history` flag to `MachinesSubcommand::Remove` variant in the `clap` enum: `#[arg(long)] purge_history: bool`

- [x] Task 6: Write co-located unit tests (AC: all)
  - [x] `#[cfg(test)]` module inside `src/commands/machines.rs`
  - [x] Test registry JSON parsing: given valid registry JSON, `list()` helper extracts machine fields correctly
  - [x] Test retire mutation: given registry JSON with one active machine, updating `status = "retired"` produces correct JSON
  - [x] Test machine-not-found path: machine_id not in registry returns appropriate result
  - [x] Test stdin confirmation acceptance: `"y"` and `"Y"` accepted; all other inputs (including empty) cancel
  - [x] Test `delete_file` body construction in `github_api.rs` (inline with `#[cfg(test)]`)
  - [x] Run `cargo test` from repo root — must pass with 0 failures
  - [x] Run `cargo clippy --all-targets -- -D warnings` — must produce 0 warnings
  - [x] Run `cargo build` — must produce 0 errors

## Dev Notes

### Module Responsibility Summary

`commands/machines.rs` is the CLI handler — it translates user intent into `github_api.rs` calls:

| Module | Role in this story |
|---|---|
| `commands::machines::list()` | Entry point for `vibestats machines list`; fetches and prints registry |
| `commands::machines::remove(id, purge)` | Entry point for `vibestats machines remove`; retire or purge |
| `crate::github_api::GithubApi` | All HTTP calls — GET registry, PUT registry, DELETE Hive files |
| `crate::config::Config::load_or_exit()` | Config loading — already implemented, do NOT modify |
| `crate::checkpoint::Checkpoint` | Local checkpoint update for self-retire/self-purge |
| `crate::logger::error` | Error logging to `vibestats.log` — never stdout for errors |

### `commands/machines.rs` Entry Point Signatures

```rust
pub fn list() {
    // Fetches registry.json, prints machine table — never calls std::process::exit
}

pub fn remove(machine_id: &str, purge_history: bool) {
    // Retire or purge machine — never calls std::process::exit
}
```

### How `main.rs` wires these (current stubs to replace)

Current `main.rs` stubs (to be replaced):
```rust
MachinesSubcommand::List => println!("not yet implemented"),
MachinesSubcommand::Remove { machine_id: _ } => println!("not yet implemented"),
```

Replace with:
```rust
MachinesSubcommand::List => commands::machines::list(),
MachinesSubcommand::Remove { machine_id, purge_history } => {
    commands::machines::remove(&machine_id, purge_history)
}
```

Add `purge_history` to the `Remove` variant in the clap enum:
```rust
Remove {
    /// Machine ID to remove
    machine_id: String,
    /// Permanently delete all historical Hive partition files
    #[arg(long)]
    purge_history: bool,
},
```

### `registry.json` Schema (from docs/schemas.md)

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

Machine `status` enum: `"active"` | `"retired"` | `"purged"`.

**`machines list` output format** (one machine per line, columns: machine_id, hostname, status, last_seen):
```
stephens-mbp-a1b2c3  Stephens-MacBook-Pro.local  active  2026-04-10T14:23:00Z
work-ubuntu-d4e5f6   work-ubuntu                 retired 2026-03-15T09:10:00Z
```

### `delete_file` in `github_api.rs` — Implementation Pattern

GitHub Contents API DELETE requires the file SHA:

```
DELETE /repos/{owner}/{repo}/contents/{path}
Body: { "message": "vibestats: remove machine data", "sha": "<current sha>" }
```

Follow the exact same pattern as `put_file`:

```rust
pub fn delete_file(&self, path: &str) -> Result<(), GithubApiError> {
    // Step 1: get SHA (with retry)
    let sha = match with_retry(|| get_file_sha_inner(&self.token, &self.repo, path)) {
        Ok(Some(sha)) => sha,
        Ok(None) => return Ok(()), // Already deleted — idempotent
        Err(e) => {
            logger::error(&format!("github_api: get_file_sha failed for {}: {}", path, e));
            return Err(e);
        }
    };
    // Step 2: DELETE (with retry)
    match with_retry(|| delete_file_inner(&self.token, &self.repo, path, &sha)) {
        Ok(()) => Ok(()),
        Err(e) => {
            logger::error(&format!("github_api: delete_file failed for {}: {}", path, e));
            Err(e)
        }
    }
}
```

`delete_file_inner` uses `ureq::delete(&url).send_string(&body)`:

```rust
#[allow(clippy::result_large_err)]
fn delete_file_inner(token: &str, repo: &str, path: &str, sha: &str) -> Result<(), ureq::Error> {
    let url = format!("https://api.github.com/repos/{}/contents/{}", repo, path);
    let body = serde_json::json!({
        "message": "vibestats: remove machine data",
        "sha": sha
    })
    .to_string();
    let response = ureq::delete(&url)
        .set("Authorization", &format!("Bearer {}", token))
        .set("User-Agent", "vibestats")
        .set("Accept", "application/vnd.github+json")
        .set("X-GitHub-Api-Version", "2022-11-28")
        .set("Content-Type", "application/json")
        .send_string(&body);
    match response {
        Ok(_) => Ok(()),
        Err(ureq::Error::Status(404, _)) => Ok(()), // already deleted
        Err(e) => Err(e),
    }
}
```

### Hive File Enumeration for `--purge-history`

**Preferred strategy: deterministic path building from checkpoint date hashes.**

The local `checkpoint.toml` `[date_hashes]` table contains all dates that were ever synced for this machine. For each date key `"YYYY-MM-DD"`, the Hive path is:

```
machines/year=YYYY/month=MM/day=DD/harness=claude/machine_id=<id>/data.json
```

Zero-padding is required: `month=04`, `day=09` (never `month=4`).

For self-purge (same machine), enumerate checkpoint date keys:
```rust
let checkpoint = Checkpoint::load(path).unwrap_or_default();
for date in checkpoint.date_hashes.keys() {
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() == 3 {
        let hive_path = format!(
            "machines/year={}/month={}/day={}/harness=claude/machine_id={}/data.json",
            parts[0], parts[1], parts[2], machine_id
        );
        // api.delete_file(&hive_path) — ignore individual errors, log them
    }
}
```

**For remote machine purge** (different machine, no local checkpoint): use the GitHub Contents API to list the directory tree. Add `list_directory` to `github_api.rs`:

```rust
pub fn list_directory(&self, path: &str) -> Result<Vec<String>, GithubApiError> {
    // GET /repos/{owner}/{repo}/contents/{path}
    // Returns JSON array; each element has "type" ("file"/"dir") and "path"
    // Return only "file" entries' paths
}
```

Then recursively list `machines/` and filter by `machine_id=<id>` partition segment. Walk depth-first to collect all `data.json` paths under the target machine's partition. This is more network-intensive but necessary for remote purge.

**Implementation recommendation**: For Story 4.2, implement:
1. Self-purge via checkpoint date enumeration (low network cost)
2. Remote purge via `list_directory` recursive walk (required for different-machine purge)

Both paths must call `api.delete_file(path)` for each found file — log errors but continue (best-effort cleanup).

### Self-Retire / Self-Purge: Checkpoint Update Pattern

When `machine_id == config.machine_id`, also update local `checkpoint.toml`:

```rust
use crate::checkpoint::Checkpoint;

fn checkpoint_path() -> Option<std::path::PathBuf> {
    std::env::var("HOME").ok().map(|home| {
        std::path::PathBuf::from(home)
            .join(".config")
            .join("vibestats")
            .join("checkpoint.toml")
    })
}

// In self-retire / self-purge path:
if let Some(ref path) = checkpoint_path() {
    let mut cp = Checkpoint::load(path).unwrap_or_default();
    cp.set_machine_status("retired"); // or "purged"
    if let Err(e) = cp.save(path) {
        logger::error(&format!("machines: failed to save checkpoint: {}", e));
    }
}
```

The `checkpoint_path()` helper must be defined privately in `machines.rs` — do NOT modify `checkpoint.rs`. Pattern is identical to `session_start.rs` — copy it, do not cross-import.

### Stdout Output Contract

| Scenario | stdout |
|---|---|
| `vibestats machines list` — has machines | One line per machine: `machine_id  hostname  status  last_seen` |
| `vibestats machines list` — no machines / 404 | `"vibestats: no machines registered"` |
| `vibestats machines list` — API error | `"vibestats: failed to fetch registry — check vibestats.log"` |
| `vibestats machines remove <id>` — success | `"vibestats: machine '<id>' retired"` |
| `vibestats machines remove <id>` — not found | `"vibestats: machine '<id>' not found in registry"` |
| `vibestats machines remove <id> --purge-history` — confirm | prompts, then `"vibestats: machine '<id>' purged — N file(s) deleted"` |
| `vibestats machines remove <id> --purge-history` — cancel | `"vibestats: purge cancelled"` |

### Error Handling Contract

| Scenario | Behaviour |
|---|---|
| `Config::load_or_exit()` fails | exits 0 with message — never reaches `commands/machines.rs` |
| `registry.json` 404 (not found) | treat as no machines registered — print friendly message |
| `registry.json` fetch error | log via `logger::error`, print user-facing message, return |
| machine_id not in registry | print not-found message, return |
| PUT registry.json fails | log via `logger::error`, print `"vibestats: failed to update registry — check vibestats.log"`, return |
| DELETE file fails (individual) | log via `logger::error` and continue — best-effort cleanup; report total files deleted |
| stdin read fails (purge confirm) | treat as `"N"` — abort purge safely |

**`commands/machines.rs` NEVER calls `std::process::exit`.** `main.rs` exits 0 after command returns. This is NFR10.

### Existing Crates (No New Dependencies Allowed)

All required crates are already in `Cargo.toml`:
```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
ureq = "2.10"
toml = "0.8"
```

`serde_json` handles registry JSON parsing and mutation. `ureq` handles DELETE (already used for GET/PUT). Do NOT add any new crates.

### `#![allow(dead_code)]` Status

`src/github_api.rs` currently has `#![allow(dead_code)]` (added in Story 2.5). **Do NOT remove it in this story.** The `get_file_sha` public method and any new `list_directory` method are not yet called from all upcoming stories (4.3, 4.4 arrive later). Keep the allow until all public API surface is exercised.

`src/config.rs` also has `#![allow(dead_code)]` — **do NOT remove it** in this story either.

`src/checkpoint.rs` has `#![allow(dead_code)]` — **do NOT remove it** in this story.

### File Structure

```
src/
├── main.rs               ← MODIFY: add purge_history to Remove variant, wire machines arms
├── github_api.rs         ← MODIFY: add delete_file, delete_file_inner, list_directory
├── config.rs             ← EXISTING — not touched (except load_or_exit is called)
├── checkpoint.rs         ← EXISTING — not touched (methods called via crate::checkpoint)
├── logger.rs             ← EXISTING — not touched
├── sync.rs               ← EXISTING — not touched
└── commands/
    ├── mod.rs            ← MODIFY: append `pub mod machines;`
    ├── sync.rs           ← EXISTING — not touched
    └── machines.rs       ← NEW — this story's implementation
```

**Do NOT create stubs for `status.rs`, `auth.rs`, `uninstall.rs`** — these arrive in Stories 4.1, 4.3, 4.4 respectively.

### Architecture Constraints Summary

| Constraint | NFR/Source | Impact on This Story |
|---|---|---|
| All GitHub HTTP through `github_api.rs` | architecture.md | `commands/machines.rs` calls `GithubApi` — no inline HTTP |
| Exit 0 on all errors | NFR10 | `commands/machines.rs` returns `()` — never calls `exit` |
| Silent errors during sessions | NFR11 | Errors logged via `logger::error` only |
| No async runtime | architecture.md | All code synchronous; no `tokio`, no `async fn` |
| No new crates | story scope | No new `Cargo.toml` entries |
| snake_case filenames | architecture.md | `src/commands/machines.rs` |
| Co-located unit tests | architecture.md | `#[cfg(test)]` inside `src/commands/machines.rs` |
| registry.json path | architecture.md | Root of `vibestats-data` repo — `"registry.json"` (no prefix) |
| Hive path zero-padding | architecture.md | `month=04`, `day=09` — never `month=4` |
| PR closes GH issue | epics.md | PR description must include `Closes #23` |
| Self-retire updates checkpoint | architecture.md Gap 2 | Update local `checkpoint.toml` in same operation |

### Anti-Patterns to Prevent

- Do NOT construct HTTP requests directly in `commands/machines.rs` — ALL GitHub calls must go through `github_api.rs`
- Do NOT call `std::process::exit` anywhere in `commands/machines.rs`
- Do NOT add `chrono`, `time`, or any new crate
- Do NOT modify `sync.rs`, `session_start.rs`, or any existing module logic
- Do NOT stub out `status.rs`, `auth.rs`, `uninstall.rs` — leave `println!("not yet implemented")` for those arms in `main.rs`
- Do NOT add a second `[workspace]` to `Cargo.toml`
- Do NOT use `unwrap()` or `expect()` in non-test production code
- Do NOT create `checkpoint_path()` in `checkpoint.rs` — define it privately in `machines.rs` (matches `session_start.rs` pattern)
- Do NOT inline retry logic in `commands/machines.rs` — `GithubApi` methods already wrap in `with_retry`

### Previous Story Intelligence (from Story 3.4)

- `commands/mod.rs` currently has only `pub mod sync;` — add `pub mod machines;` on a new line
- `Config::load_or_exit()` exits 0 with message if config missing — `commands/machines.rs` never needs to handle config failure
- All GitHub HTTP errors are already logged inside `GithubApi` methods via `logger::error` — callers can check `Err(e)` without re-logging the technical details (but may log a higher-level context message)
- `cargo clippy --all-targets -- -D warnings` must pass — run with `--all-targets` to catch test code lint
- `cargo test` must pass from the repo root (not from inside the worktree directory)
- Worktree is nested inside the main repo — `Cargo.toml` already has `[workspace]`; do NOT add another

### Previous Story Intelligence (from Story 3.3 / session_start.rs)

- `session_start.rs` already implements GET + parse of `registry.json` — follow that exact pattern for `list()` and `remove()`
- `registry.json` is at path `"registry.json"` (root of `vibestats-data`, no subdirectory prefix)
- `json["machines"].as_array()` is the correct access pattern — field is `machines` (snake_case array)
- `serde_json::from_str(&content).unwrap_or(serde_json::Value::Null)` is the pattern used in `session_start.rs` — prefer handling `Err` explicitly in `machines.rs` for better error surfacing
- The `checkpoint_path()` private helper is defined in `session_start.rs` — copy it to `machines.rs`, do not share across modules

### Project Structure Notes

- New files: `src/commands/machines.rs`
- Modified files: `src/main.rs` (add `purge_history` to `Remove` variant, wire `machines::list` and `machines::remove`), `src/commands/mod.rs` (append `pub mod machines;`), `src/github_api.rs` (add `delete_file`, `delete_file_inner`, optionally `list_directory`)
- No other files modified

### References

- Story requirements: [Source: _bmad-output/planning-artifacts/epics.md#Story 4.2]
- FR34 (machines list): [Source: _bmad-output/planning-artifacts/epics.md#Functional Requirements]
- FR35 (machines remove): [Source: _bmad-output/planning-artifacts/epics.md#Functional Requirements]
- registry.json schema: [Source: docs/schemas.md#4-registryjson]
- machines remove design (retire/purge): [Source: _bmad-output/planning-artifacts/architecture.md#Gap 2]
- Hive partition path format: [Source: docs/schemas.md#1-machine-day-file]
- Self-retire checkpoint update: [Source: _bmad-output/planning-artifacts/architecture.md#Gap 2]
- Module file structure: [Source: _bmad-output/planning-artifacts/architecture.md#Complete Project Directory Structure]
- Single HTTP module constraint: [Source: _bmad-output/planning-artifacts/architecture.md#GitHub API access — single module]
- NFR10 (exit 0): [Source: _bmad-output/planning-artifacts/epics.md#NonFunctional Requirements]
- GH Issue: #23

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

None.

### Completion Notes List

- Implemented `commands/machines.rs` with `list()` and `remove(machine_id, purge_history)` entry points
- Added `delete_file`, `delete_file_inner`, `list_directory`, `list_directory_inner`, `list_directory_all`, and `list_directory_all_inner` to `github_api.rs`
- Self-purge uses local checkpoint date hashes for deterministic, low-network-cost enumeration
- Remote purge uses `list_directory_all` depth-first walk through the Hive partition tree
- Self-retire and self-purge both update local `checkpoint.toml` atomically via `Checkpoint::save`
- Confirmation prompt accepts only `"y"` or `"Y"`; stdin read failure treated as cancel (safe default)
- All 118 tests pass; 0 clippy warnings; 0 build errors
- `#![allow(dead_code)]` preserved in `github_api.rs`, `config.rs`, and `checkpoint.rs` per story constraints

### File List

- src/commands/mod.rs (modified)
- src/commands/machines.rs (new)
- src/github_api.rs (modified)
- src/main.rs (modified)

## Change Log

- 2026-04-11: Implemented story 4.2 — `vibestats machines list` and `vibestats machines remove` commands with retire/purge support. Added `delete_file`, `list_directory`, and `list_directory_all` to `github_api.rs`. Wired commands in `main.rs` with `--purge-history` flag. 118 tests pass, 0 clippy warnings.
- 2026-04-11: Code review (Step 3) — 1 patch applied (list() output column alignment); 2 findings deferred; build/clippy/test clean.

### Review Findings

- [x] [Review][Patch] `list()` output columns not aligned per spec Dev Notes example [src/commands/machines.rs:57] — fixed: now computes max column widths and pads with `{:<N$}` format specifiers
- [x] [Review][Defer] Remote purge walks entire `machines/` tree for every purge target [src/commands/machines.rs:279] — deferred, architecturally intended per story ("more network-intensive but necessary for remote purge")
- [x] [Review][Defer] No unit tests for `purge_self` / `purge_remote` / `update_local_checkpoint` [src/commands/machines.rs:231-385] — deferred, would require a GithubApi mocking framework not allowed by story constraints
