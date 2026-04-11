# Story 2.1: Implement config module

Status: review

<!-- GH Issue: #13 | Epic: #2 | PR must include: Closes #13 -->

## Story

As the vibestats binary,
I want a config module that reads and writes `~/.config/vibestats/config.toml` with correct permissions,
so that the OAuth token, machine ID, and repo path are stored securely.

## Acceptance Criteria

1. **Given** `config.toml` does not exist **When** `config.rs` writes it for the first time **Then** the file is created at `~/.config/vibestats/config.toml` with permissions `600` (NFR6)

2. **Given** `config.toml` exists with valid content **When** `Config::load()` is called **Then** it returns a struct with `oauth_token`, `machine_id`, and `vibestats_data_repo` fields correctly populated

3. **Given** a new machine install **When** `Config::generate_machine_id()` is called **Then** it produces a deterministic ID from hostname + stable UUID and stores it in `config.toml`

4. **Given** `config.toml` is missing or malformed **When** any command reads config **Then** the binary exits 0 and prints a human-readable error with fix instructions (NFR10) — **Note:** `vibestats.log` integration is deferred to Story 2.2; for this story, `println!` to stdout is the correct implementation

## Tasks / Subtasks

- [x] Task 1: Create `src/config.rs` with `Config` struct and TOML schema (AC: #1, #2)
  - [x] Define `Config` struct with fields: `oauth_token: String`, `machine_id: String`, `vibestats_data_repo: String` — all using `#[derive(Serialize, Deserialize, Debug)]`
  - [x] Implement `Config::load() -> Result<Config>` — reads `~/.config/vibestats/config.toml` and deserializes via `toml` crate
  - [x] Implement `Config::save(&self) -> Result<()>` — serializes to TOML and writes to `~/.config/vibestats/config.toml`; creates parent directory if needed
  - [x] Enforce `600` permissions (owner read/write only) at write time via `std::fs::set_permissions` with `PermissionsExt::from_mode(0o600)` (NFR6)

- [x] Task 2: Implement `Config::generate_machine_id()` (AC: #3)
  - [x] Read hostname via `std::process::Command::new("hostname").output()` or `gethostname` syscall
  - [x] Compute a deterministic 6-hex-char suffix using the FNV-1a algorithm (see Dev Notes) — **do NOT use `uuid` crate** (not in `Cargo.toml`; no new crates), and **do NOT use `DefaultHasher`** (not stable across Rust versions)
  - [x] Slug the hostname: lowercase, replace non-alphanumeric with `-`, truncate at 20 chars before appending hash suffix — result format: `"stephens-mbp-a1b2c3"`
  - [x] Write the generated ID into `config.toml` by calling `self.machine_id = id; self.save()?`

- [x] Task 3: Implement error handling per silent failure contract (AC: #4)
  - [x] On any `config.rs` error: print human-readable message to `stdout` with fix instruction (e.g., `"Config error: run 'vibestats auth' to initialize config"`) — **do NOT use logger in this story**, that is Story 2.2
  - [x] Call `std::process::exit(0)` on all error paths — never propagate panic or non-zero exit (NFR10)
  - [x] Handle both "file not found" and "TOML parse error" cases with distinct messages

- [x] Task 4: Wire `config` module into `main.rs` (AC: #1–#4)
  - [x] Add `mod config;` declaration to `src/main.rs`
  - [x] Do NOT call `Config::load()` from `main()` yet — that happens in later stories when commands need it. Just declare the module so it compiles.
  - [x] Ensure `cargo build` compiles with 0 errors and `cargo clippy -- -D warnings` produces 0 warnings

- [x] Task 5: Write co-located unit tests in `src/config.rs` (AC: #1–#4)
  - [x] Test `Config::load()` with a valid TOML fixture — verify all three fields populated correctly
  - [x] Test `Config::load()` with a missing file — verify it returns an `Err` (not panic)
  - [x] Test `Config::load()` with malformed TOML — verify it returns an `Err`
  - [x] Test `Config::generate_machine_id()` — verify the result is non-empty, contains only `[a-z0-9-]`, and is deterministic (same hostname → same ID on repeated calls)
  - [x] Test file creation sets `600` permissions — write a temp file, check `metadata().permissions().mode() & 0o777 == 0o600` (use `std::os::unix::fs::PermissionsExt`)
  - [x] Use `tempfile`-free approach: write to `std::env::temp_dir()` + unique suffix for test isolation; clean up in test teardown

## Dev Notes

### Context: What Epic 1 Established

Story 1.2 initialized the Rust binary project. The current state is:
- `Cargo.toml` has `[workspace]` section and all required dependencies
- `src/main.rs` has the full clap CLI skeleton with 5 subcommands
- All 5 crates are available: `clap`, `serde` + `serde_json`, `ureq`, `toml`
- `Cargo.lock` is committed

**CRITICAL: Do NOT run `cargo new`.** The project already exists. Only create `src/config.rs` and modify `src/main.rs` to add `mod config;`.

### File to Create

```
src/config.rs    ← THIS STORY (new file)
```

**Modify only:**
```
src/main.rs      ← add `mod config;` declaration at top
```

**Do NOT create or touch:**
- `src/logger.rs` — Story 2.2
- `src/checkpoint.rs` — Story 2.3
- Any file under `src/commands/` — Epic 4

### `config.toml` Schema (from `docs/schemas.md`)

**Path:** `~/.config/vibestats/config.toml`
**Permissions:** `600` (enforced at write time — NFR6)

```toml
oauth_token = "gho_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
machine_id = "stephens-mbp-a1b2c3"
vibestats_data_repo = "stephenleo/vibestats-data"
```

| Field | Type | Description |
|-------|------|-------------|
| `oauth_token` | string | Machine-side GitHub OAuth token via `gh auth token`; scoped to `vibestats-data` Contents write |
| `machine_id` | string | Deterministic ID from hostname + stable hash; format `"hostname-slug-hexhash"` |
| `vibestats_data_repo` | string | `"username/vibestats-data"` format |

All field names are `snake_case` — no exceptions (JSON/TOML naming rule from architecture).

### Required `Config` Struct

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub oauth_token: String,
    pub machine_id: String,
    pub vibestats_data_repo: String,
}
```

### Permissions Enforcement Pattern (NFR6)

```rust
use std::os::unix::fs::PermissionsExt;

fn set_permissions_600(path: &std::path::Path) -> std::io::Result<()> {
    let mut perms = std::fs::metadata(path)?.permissions();
    perms.set_mode(0o600);
    std::fs::set_permissions(path, perms)
}
```

Call `set_permissions_600` immediately after writing `config.toml` — every write, not just first-time creation.

### Directory Creation Pattern

```rust
if let Some(parent) = path.parent() {
    std::fs::create_dir_all(parent)?;
}
```

The `~/.config/vibestats/` directory may not exist on a fresh install. Always `create_dir_all` before writing.

### Error Handling: Silent Failure Contract (NFR10)

**This story does NOT have access to the logger yet (Story 2.2).** For AC#4:
1. Print a human-readable message to `stdout` (not `stderr`) with fix instruction
2. Call `std::process::exit(0)`

Example pattern:
```rust
pub fn load_or_exit() -> Config {
    match Config::load() {
        Ok(c) => c,
        Err(e) => {
            println!("vibestats: config error: {e}");
            println!("Run 'vibestats auth' to set up your configuration.");
            std::process::exit(0);
        }
    }
}
```

**Note:** Once Story 2.2 (logger) is complete, the logger call will be added here. For now, `println!` is acceptable.

### No New Crates

The `Cargo.toml` must not gain new dependencies in this story. Use only what is already declared:
- `serde` + `serde_json` for struct derive macros
- `toml` for config file serialization/deserialization
- `std` for file I/O, permissions, process commands, temp dirs

Do NOT add `uuid`, `hostname`, `dirs`, `home`, or any other crate.

**Home directory resolution:** Use `std::env::var("HOME")` or `dirs`-free approach:
```rust
fn config_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").expect("HOME env not set");
    std::path::PathBuf::from(home)
        .join(".config")
        .join("vibestats")
        .join("config.toml")
}
```

### Machine ID Generation (No `uuid` Crate)

**CRITICAL:** Do NOT use `std::collections::hash_map::DefaultHasher` — its output is NOT stable across Rust versions or platforms (explicitly not guaranteed by std docs). Use a simple manual FNV-1a hash instead for cross-version determinism:

```rust
fn fnv1a_hash(s: &str) -> u64 {
    let mut hash: u64 = 14695981039346656037;
    for byte in s.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(1099511628211);
    }
    hash
}

fn generate_machine_id() -> String {
    let hostname = std::process::Command::new("hostname")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_lowercase())
        .unwrap_or_else(|_| "unknown".to_string());

    let slug: String = hostname
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .chars()
        .take(20)
        .collect();

    let hash = fnv1a_hash(&hostname);
    format!("{}-{:06x}", slug, hash & 0xffffff)
}
```

Result format: `"stephens-mbp-a1b2c3"` — lowercase, alphanumeric + hyphens, 6-hex-char suffix for uniqueness. Deterministic: same hostname always produces same ID across Rust versions and platforms.

### Architecture Constraints Summary

| Constraint | Source | Impact |
|---|---|---|
| `config.toml` at `~/.config/vibestats/config.toml` | architecture.md#Data Architecture | Hardcoded path — no env override |
| `600` permissions enforced at write time | architecture.md + NFR6 | `set_permissions` after every write |
| All errors exit 0 | architecture.md#Process Patterns + NFR10 | `std::process::exit(0)` on all error paths |
| No new crates | Story 2.1 scope | Use only `serde`, `toml`, `std` |
| `snake_case` struct field names | architecture.md#Naming Patterns | `oauth_token` not `oauthToken` |
| Co-located `#[cfg(test)]` modules | architecture.md#Test Placement | Tests inside `config.rs`, not separate file |

### Testing Notes

- Tests MUST be co-located as `#[cfg(test)]` module in `src/config.rs` (architecture.md requirement)
- Use `std::env::temp_dir()` + unique filename for file tests — do NOT write to `~/.config/vibestats/` in tests
- On macOS/Linux both — `PermissionsExt` works on both platforms; these tests should pass in CI
- No test framework beyond `std` test harness needed — just `#[test]` functions

### Previous Story Intelligence (Story 1.2 Learnings)

From Story 1.2 review:
- The worktree is nested inside the main repo. The `[workspace]` section in `Cargo.toml` prevents upward traversal issues during `cargo build`. Do not remove it.
- `cargo clippy -- -D warnings` must produce 0 warnings — treat as blocking gate
- `Cargo.lock` changes from new compiled deps are expected and should be committed
- `.gitignore` excludes only `/target/` — `Cargo.lock` is tracked, do not add it to ignore

### References

- Story 2.1 acceptance criteria: [Source: epics.md#Story 2.1: Implement config module]
- `config.toml` schema (all fields + permissions): [Source: docs/schemas.md#3. Local Configuration Files]
- NFR6 (`600` permissions at write time): [Source: epics.md#Non-Functional Requirements]
- NFR10 (all errors exit 0, human-readable output): [Source: epics.md#Non-Functional Requirements]
- Silent failure contract: [Source: architecture.md#Process Patterns]
- No async runtime: [Source: architecture.md#Selected Starters by Component - Rust Sync Binary]
- Test placement: co-located `#[cfg(test)]` in each `.rs` file: [Source: architecture.md#Test Placement]
- Module file layout: [Source: architecture.md#Complete Project Directory Structure]
- `snake_case` naming: [Source: architecture.md#Naming Patterns]
- Story 1.2 dev notes (workspace + cargo conventions): [Source: implementation-artifacts/1-2-initialize-rust-binary-project.md#Dev Notes]
- GH Issue: #13

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

No issues encountered. Build succeeded with 0 errors, clippy produced 0 warnings, all 6 tests pass.

### Completion Notes List

- Implemented `src/config.rs` with `Config` struct (`oauth_token`, `machine_id`, `vibestats_data_repo`) using `serde` + `toml`.
- `Config::load()` reads `~/.config/vibestats/config.toml`, returning distinct `Err` messages for missing file vs malformed TOML.
- `Config::save()` creates parent directories, writes TOML, then enforces `600` permissions via `PermissionsExt`.
- `Config::generate_machine_id()` uses FNV-1a hash (not `DefaultHasher`) for cross-version determinism; format: `"slug-hexhash"`.
- `Config::load_or_exit()` implements the silent failure contract: `println!` to stdout + `std::process::exit(0)`.
- `#[allow(dead_code)]` added at module level since public APIs are not called yet (used by future stories).
- Added `mod config;` to `src/main.rs` without calling `Config::load()` from `main()`.
- 6 co-located unit tests in `#[cfg(test)]` module — all pass; use `std::env::temp_dir()` for isolation, no external test crates.

### File List

- `src/config.rs` (new)
- `src/main.rs` (modified — added `mod config;`)

### Change Log

- 2026-04-11: Implemented Story 2.1 — config module with `Config` struct, `load`/`save`/`generate_machine_id`/`load_or_exit`, 600-permission enforcement, FNV-1a machine ID generation, silent failure contract, and 6 unit tests.
