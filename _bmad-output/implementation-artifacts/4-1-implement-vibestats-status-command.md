# Story 4.1: Implement vibestats status Command

Status: ready-for-dev

<!-- GH Issue: #22 | Epic: #4 | PR must include: Closes #22 -->

## Story

As a developer,
I want `vibestats status` to show me all registered machines, last sync times, and auth token validity,
so that I can diagnose sync issues without reading log files.

## Acceptance Criteria

1. **Given** the user runs `vibestats status` **When** it executes **Then** it prints each registered machine from `registry.json` with `machine_id`, `hostname`, `status`, and `last_seen` timestamp (FR32)

2. **Given** the current machine's OAuth token is valid **When** `vibestats status` runs a connectivity check **Then** it shows "Auth: OK" alongside the associated GitHub username (FR33)

3. **Given** the current machine's OAuth token is invalid **When** the connectivity check fails with 401 **Then** it shows "Auth: ERROR — run `vibestats auth` to re-authenticate" (FR33)

## Tasks / Subtasks

- [ ] Task 1: Add `pub mod status;` to `src/commands/mod.rs` (AC: all)
  - [ ] Open `src/commands/mod.rs` (currently has only `pub mod sync;`)
  - [ ] Add `pub mod status;` as a second line — do NOT add stubs for 4.2–4.4 commands yet

- [ ] Task 2: Add `get_user` method to `GithubApi` in `src/github_api.rs` (AC: #2, #3)
  - [ ] Implement `pub fn get_user(&self) -> Result<String, GithubApiError>` on `GithubApi`
  - [ ] Call `GET https://api.github.com/user` with `Authorization: Bearer {token}`, `User-Agent: vibestats`, `Accept: application/vnd.github+json`, `X-GitHub-Api-Version: 2022-11-28`
  - [ ] On 200: parse `login` field from JSON response body, return `Ok(login)`
  - [ ] On 401: return `Err(...)` immediately (do NOT retry — 401 is non-retriable per existing `is_status_retriable` logic; route through `with_retry` which handles this automatically)
  - [ ] On 429 / 5xx / transport: let `with_retry` handle retries (same pattern as `get_file_sha`)
  - [ ] Write inner function `get_user_inner(token: &str) -> Result<String, ureq::Error>` following the same pattern as `get_file_sha_inner` and `get_file_content_inner`
  - [ ] Wrap with `with_retry(|| get_user_inner(&self.token))` in the public method
  - [ ] Do NOT remove `#![allow(dead_code)]` from `github_api.rs` — the public `get_file_sha` method is not called externally and remains dead code; removing the suppression would cause a clippy warning
  - [ ] Do NOT add `#![allow(clippy::result_large_err)]` — it is already at the top of `github_api.rs`

- [ ] Task 3: Implement `src/commands/status.rs` (AC: #1, #2, #3)
  - [ ] Create `src/commands/status.rs`
  - [ ] Implement `pub fn run()` — the entry point called from `main.rs`
  - [ ] Load config via `Config::load_or_exit()` (exits 0 with message if config missing)
  - [ ] Create `GithubApi::new(&config.oauth_token, &config.vibestats_data_repo)`
  - [ ] Fetch `registry.json` via `api.get_file_content("registry.json")` — all GitHub calls through `github_api.rs`
  - [ ] Parse registry JSON; print each machine entry (see stdout contract below)
  - [ ] Handle missing registry (404 → `Ok(None)`): print "No machines registered yet."
  - [ ] Handle fetch error (`Err`): print "vibestats: failed to fetch registry — check your connection." and continue to auth check
  - [ ] Perform auth check via `api.get_user()`:
    - `Ok(login)` → print `"Auth: OK (github.com/{login})"`
    - `Err(_)` → print `"Auth: ERROR — run \`vibestats auth\` to re-authenticate"`
  - [ ] Never call `std::process::exit` in `status.rs` — `main.rs` handles exit

- [ ] Task 4: Wire `Commands::Status` into `main.rs` (AC: #1, #2, #3)
  - [ ] Replace `Commands::Status => println!("not yet implemented")` with `Commands::Status => commands::status::run()`
  - [ ] No other changes to `main.rs`

- [ ] Task 5: Write co-located unit tests (AC: #1, #2, #3)
  - [ ] `#[cfg(test)]` module inside `src/commands/status.rs`
  - [ ] Test registry JSON parsing logic: parse a valid JSON string with 2 machines and verify both are found
  - [ ] Test registry parse with empty `machines` array: should produce no machine output lines
  - [ ] Test registry parse with malformed JSON: should not panic (graceful handling)
  - [ ] Add unit tests for `get_user` JSON parsing logic in `src/github_api.rs` test module — test that `login` field is extracted correctly from a valid JSON body (same pattern as `test_parse_sha_present_in_json_body` which tests JSON parsing logic directly without network calls, since `get_user_inner` is private and cannot be called from `status.rs` tests)
  - [ ] Run `cargo test` — must pass with 0 failures (currently 101 tests pass; new tests add to that count)
  - [ ] Run `cargo clippy --all-targets -- -D warnings` — must produce 0 warnings
  - [ ] Run `cargo build` — must produce 0 errors (clean build)

## Dev Notes

### Module Responsibility Summary

`commands/status.rs` is the **CLI handler** — it translates user intent into API and config reads:

| Module | Role in this story |
|---|---|
| `commands::status::run()` | Entry point from `main.rs`; loads config, fetches registry, checks auth, prints stdout |
| `crate::config::Config::load_or_exit()` | Already implemented — load config or exit 0 with message |
| `crate::github_api::GithubApi::get_file_content("registry.json")` | Fetch registry.json from remote |
| `crate::github_api::GithubApi::get_user()` | NEW: GET /user for auth check — returns GitHub login on 200, Err on 401 |
| `crate::logger` | Only for internal error logging — NOT for stdout output |

**`config.rs`, `checkpoint.rs`, `github_api.rs` (except adding `get_user`) are COMPLETE — do not modify their logic.**

### `commands/status.rs` Entry Point Signature

```rust
pub fn run() {
    // loads config, fetches registry, checks auth, prints stdout
    // NEVER calls std::process::exit — main.rs handles exit
}
```

### Stdout Output Contract

**Registry section (FR32):**

For each machine in `registry.json["machines"]` array, print one line:
```
machine: {machine_id}  hostname: {hostname}  status: {status}  last_seen: {last_seen}
```

If `registry.json` is not found (404 → `Ok(None)`):
```
No machines registered yet.
```

If registry fetch fails (network/server error):
```
vibestats: failed to fetch registry — check your connection.
```

**Auth section (FR33):**

On successful token check:
```
Auth: OK (github.com/{login})
```

On 401 or any error:
```
Auth: ERROR — run `vibestats auth` to re-authenticate
```

**Important:** The auth check always runs regardless of registry fetch outcome (even if registry errors, still check auth).

### `get_user` Implementation Pattern

Follow the exact same pattern as `get_file_content_inner` in `github_api.rs`:

```rust
fn get_user_inner(token: &str) -> Result<String, ureq::Error> {
    let url = "https://api.github.com/user";
    let response = ureq::get(url)
        .set("Authorization", &format!("Bearer {}", token))
        .set("User-Agent", "vibestats")
        .set("Accept", "application/vnd.github+json")
        .set("X-GitHub-Api-Version", "2022-11-28")
        .call();

    match response {
        Ok(r) => {
            let body = r.into_string().map_err(ureq::Error::from)?;
            let json: serde_json::Value = serde_json::from_str(&body).map_err(|e| {
                ureq::Error::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("github_api: malformed JSON from /user: {}", e),
                ))
            })?;
            match json["login"].as_str() {
                Some(login) => Ok(login.to_string()),
                None => Err(ureq::Error::from(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "github_api: missing login field in /user response",
                ))),
            }
        }
        Err(e) => Err(e),
    }
}
```

Public method on `GithubApi`:
```rust
pub fn get_user(&self) -> Result<String, GithubApiError> {
    with_retry(|| get_user_inner(&self.token))
}
```

Note: 401 is non-retriable per `is_status_retriable` (only 429 and 5xx retry) — `with_retry` automatically propagates 401 immediately. No special 401 handling needed.

### Registry JSON Parsing Pattern

The existing pattern in `session_start.rs` shows exactly how to parse `registry.json`:

```rust
let json: serde_json::Value =
    serde_json::from_str(&content).unwrap_or(serde_json::Value::Null);
let machines = json["machines"].as_array();
if let Some(machines) = machines {
    for m in machines {
        let machine_id = m["machine_id"].as_str().unwrap_or("unknown");
        let hostname = m["hostname"].as_str().unwrap_or("unknown");
        let status = m["status"].as_str().unwrap_or("unknown");
        let last_seen = m["last_seen"].as_str().unwrap_or("never");
        println!("machine: {}  hostname: {}  status: {}  last_seen: {}",
            machine_id, hostname, status, last_seen);
    }
}
```

Use `.unwrap_or("unknown")` / `.unwrap_or("never")` for missing fields — never `.unwrap()` in non-test code.

### `registry.json` Schema (from `docs/schemas.md`)

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

Location: root of `vibestats-data` repo — fetched via `api.get_file_content("registry.json")`.

### Error Handling Contract

| Scenario | Behaviour |
|---|---|
| `Config::load_or_exit()` fails | Exits 0 with message before any stdout — never reaches `status.rs` body |
| Registry fetch → `Ok(None)` (404) | Print "No machines registered yet." |
| Registry fetch → `Err(_)` | Print "vibestats: failed to fetch registry — check your connection." |
| Registry JSON malformed | `unwrap_or(Value::Null)` — machines array is None → no machine lines printed |
| `get_user()` → `Ok(login)` | Print `"Auth: OK (github.com/{login})"` |
| `get_user()` → `Err(_)` (401 or any) | Print `"Auth: ERROR — run \`vibestats auth\` to re-authenticate"` |

**`commands/status.rs` NEVER calls `std::process::exit`.** `main.rs` implicitly exits 0 after the command returns.

**`status.rs` NEVER calls `logger::error` directly** — it prints actionable messages to stdout (this is a user-initiated CLI command, not a hook).

### File Structure

```
src/
├── main.rs               ← MODIFY: replace println! in Status arm with commands::status::run()
├── github_api.rs         ← MODIFY: add get_user() method and get_user_inner() function
├── checkpoint.rs         ← EXISTING — not touched
├── config.rs             ← EXISTING — not touched
├── sync.rs               ← EXISTING — not touched
├── jsonl_parser.rs       ← EXISTING — not touched
├── logger.rs             ← EXISTING — not touched
├── hooks/session_start.rs ← EXISTING — not touched
└── commands/
    ├── mod.rs            ← MODIFY: add `pub mod status;`
    ├── sync.rs           ← EXISTING — not touched
    └── status.rs         ← NEW — this story's implementation
```

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

Do NOT add any new crate. `ureq` handles the HTTP call; `serde_json` parses the response — both already in Cargo.toml.

### Architecture Constraints Summary

| Constraint | NFR/Source | Impact on This Story |
|---|---|---|
| Exit 0 on all errors | NFR10 | `commands/status.rs` returns `()` — never calls `exit` |
| All GitHub HTTP through `github_api.rs` | architecture.md | `get_user()` must be added to `GithubApi`, not inline in `status.rs` |
| No async runtime | architecture.md | All code synchronous; no `tokio`, no `async fn` |
| No new crates | Story scope | `ureq` and `serde_json` cover all needs |
| snake_case filenames | architecture.md | File: `src/commands/status.rs` |
| Co-located unit tests | architecture.md | `#[cfg(test)]` module inside `src/commands/status.rs` |
| PR closes GH issue | epics.md | PR description must include `Closes #22` |
| `unwrap()` / `expect()` in non-test code | architecture.md | Never — use `.unwrap_or("unknown")` for JSON field access |

### Anti-Patterns to Prevent

- Do NOT make inline HTTP calls in `status.rs` — all GitHub HTTP goes through `github_api.rs` (`get_user()` must be a method on `GithubApi`)
- Do NOT call `std::process::exit` in `status.rs` — `main.rs` handles exit
- Do NOT add `chrono`, `time`, or any date/auth crate — no new dependencies
- Do NOT add `pub mod machines;`, `pub mod auth;`, `pub mod uninstall;` to `commands/mod.rs` — only add `pub mod status;` in this story
- Do NOT add a second `[workspace]` to `Cargo.toml`
- Do NOT use `unwrap()` or `expect()` in non-test code (use `.unwrap_or("unknown")`)
- Do NOT log to `vibestats.log` for user-facing status output — stdout only
- Do NOT skip the auth check if registry fetch fails — auth check runs unconditionally after registry section

### Previous Story Learnings

From Story 3.4 (`commands/sync.rs`):
- `commands/mod.rs` currently has only `pub mod sync;` — add `pub mod status;` as the second line
- Do NOT add stubs for 4.2–4.4 — they will be added in their respective stories to avoid dead_code lint
- `main.rs` arms for `Commands::Machines`, `Commands::Auth`, `Commands::Uninstall` still print `"not yet implemented"` — leave them unchanged
- `cargo clippy --all-targets -- -D warnings` (with `--all-targets`) catches all targets including test code
- Run verification from repo root (not from inside the worktree directory)

From Story 3.3 (`hooks/session_start.rs`):
- `session_start.rs` already contains the registry.json fetch + parse pattern — copy and adapt (do NOT import from that module; `status.rs` is a separate command)
- The `parse_iso8601_utc` helper in session_start.rs is module-private — do NOT reference it from `status.rs` (and you don't need it here)
- `#![allow(dead_code)]` is on `github_api.rs` — do NOT remove it; the public `get_file_sha` method is never called externally and would generate a dead_code warning if the suppression were removed

From Story 2.5 (`github_api.rs`):
- `#![allow(clippy::result_large_err)]` is already at the top of `github_api.rs` — do not add it again
- `with_retry` takes a closure returning `Result<T, ureq::Error>` — inner function must return `ureq::Error` (not boxed) for retriability classification

From Story 2.3 (`checkpoint.rs`):
- Use `std::env::temp_dir()` for test temp files (not `/tmp` directly)
- `std::process::exit` must never be called inside modules — only by `main.rs`

### Project Structure Notes

- New file: `src/commands/status.rs`
- Modified files:
  - `src/commands/mod.rs` (add `pub mod status;`)
  - `src/github_api.rs` (add `get_user()` public method + `get_user_inner()` private function)
  - `src/main.rs` (replace `println!("not yet implemented")` in Status arm)
- No other files modified

### Worktree / Cargo Isolation

The worktree is nested inside the main repo. Run all verification from the **repo root** (not from inside the worktree directory):

```bash
cargo build
cargo test
cargo clippy --all-targets -- -D warnings
```

### References

- Story requirements: [Source: _bmad-output/planning-artifacts/epics.md#Story 4.1]
- FR32 (machines list): [Source: _bmad-output/planning-artifacts/prd.md#Functional Requirements]
- FR33 (auth token validity): [Source: _bmad-output/planning-artifacts/prd.md#Functional Requirements]
- NFR10 (exit 0): [Source: _bmad-output/planning-artifacts/prd.md#Non-Functional Requirements]
- NFR11 (silent failure): [Source: _bmad-output/planning-artifacts/prd.md#Non-Functional Requirements]
- registry.json schema: [Source: docs/schemas.md#4. registry.json]
- Architecture constraints: [Source: _bmad-output/planning-artifacts/architecture.md#Enforcement Guidelines]
- Module file structure: [Source: _bmad-output/planning-artifacts/architecture.md#Complete Project Directory Structure]
- GitHub API module boundaries: [Source: _bmad-output/planning-artifacts/architecture.md#Module responsibility boundaries]
- Registry parse pattern: [Source: src/hooks/session_start.rs#Step 1: Machine retirement check]
- Auth token lifecycle: [Source: _bmad-output/planning-artifacts/architecture.md#Key design decisions]
- GH Issue: #22

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

### Completion Notes List

### File List
