# Story 3.1: Implement Core Sync Orchestration

Status: done

<!-- GH Issue: #18 | Epic: #3 | PR must include: Closes #18 -->

## Story

As the vibestats binary,
I want a sync orchestration layer that coordinates JSONL parsing, hash comparison, and GitHub API push for a given date range,
so that any entry point (hook, CLI, backfill) routes through the same tested logic.

## Acceptance Criteria

1. **Given** a date range is passed to `sync::run(start_date, end_date)` **When** it runs **Then** it: (1) calls `jsonl_parser::parse_date_range` for the range, (2) computes payload hash per date, (3) skips dates where hash matches checkpoint, (4) calls `github_api::put_file` for changed dates, (5) updates checkpoint hashes on success

2. **Given** the same JSONL data for a date **When** sync runs twice **Then** the second run makes zero API calls (idempotency via hash check) (NFR12)

3. **Given** sync runs for any code path **When** it exits **Then** it always exits 0 — no exceptions propagate up (NFR10)

4. **Given** the GitHub API returns 401 **When** `sync::run` handles the error **Then** it sets `auth_error = true` in checkpoint and saves checkpoint before returning

5. **Given** any API call fails (non-401) **When** `sync::run` handles the error **Then** it logs the error via `logger::error` and continues to the next date without aborting

6. **Given** `sync::run` completes successfully for at least one date **When** it returns **Then** the checkpoint has been saved with updated hashes

## Tasks / Subtasks

- [x] Task 1: Create `src/sync.rs` with `run(start_date: &str, end_date: &str)` public function (AC: #1, #2, #3)
  - [x] Add `#![allow(dead_code)]` at top of file (callers arrive in Stories 3.2–3.4)
  - [x] Call `jsonl_parser::parse_date_range(start_date, end_date)` to get `HashMap<String, DailyActivity>`
  - [x] For each date in the result map, serialize the `DailyActivity` to a compact JSON string: `{"sessions":N,"active_minutes":N}`
  - [x] Compute SHA256 of the JSON payload bytes (std-only, no external crate — see Dev Notes for implementation)
  - [x] Compare computed hash to `checkpoint.date_hashes[date]` via `checkpoint::Checkpoint::hash_matches` — skip if equal (AC: #2)
  - [x] Call `github_api::GithubApi::put_file(hive_path, payload_json)` for dates where hash changed
  - [x] On `put_file` success: call `checkpoint.update_hash(date, hash)` and set `last_sync_date` in a local tracker
  - [x] On `put_file` error (401): call `checkpoint.set_auth_error()`, log via `logger::error`, continue to next date
  - [x] On `put_file` error (other): log via `logger::error`, continue to next date (do NOT set auth_error)
  - [x] After processing all dates, call `checkpoint.save(checkpoint_path)` once — always save even if some dates failed (AC: #6)

- [x] Task 2: Implement `hive_path` construction helper (AC: #1)
  - [x] Private `fn hive_path(date: &str, machine_id: &str) -> String`
  - [x] Input: `date` as `"YYYY-MM-DD"`, `machine_id` from config (e.g., `"stephens-mbp-a1b2c3"`)
  - [x] Output: `"machines/year=YYYY/month=MM/day=DD/harness=claude/machine_id=<id>/data.json"`
  - [x] Extract year, month, day from `date[0..4]`, `date[5..7]`, `date[8..10]`
  - [x] Month and day must be zero-padded two digits (they already are if JSONL parser outputs `YYYY-MM-DD`)
  - [x] Example: `"2026-04-10"` + `"stephens-mbp-a1b2c3"` → `"machines/year=2026/month=04/day=10/harness=claude/machine_id=stephens-mbp-a1b2c3/data.json"`

- [x] Task 3: Implement SHA256 payload hash helper (AC: #2)
  - [x] Private `fn sha256_hex(data: &[u8]) -> String` — std-only, no `sha2` crate (see Dev Notes)
  - [x] Hash the serialized JSON bytes `{"sessions":N,"active_minutes":N}` (not the full `DailyActivity` serde output)
  - [x] Return lowercase hex string (64 chars) matching the format stored in `checkpoint.toml`

- [x] Task 4: Wire `sync.rs` into `main.rs` as a declared module (AC: #1)
  - [x] Add `mod sync;` to `src/main.rs` alongside existing `mod` declarations
  - [x] No business logic in `main.rs` — only the `mod` declaration

- [x] Task 5: Write co-located unit tests (AC: #1, #2, #3, #4, #5)
  - [x] `#[cfg(test)]` module inside `src/sync.rs`
  - [x] Test `hive_path` constructs the correct path for a known date and machine_id
  - [x] Test `sha256_hex` output matches known SHA256 vectors (e.g., `sha256("")` = `"e3b0c44298fc1c149..."`, `sha256("hello")` = `"2cf24dba5fb0a30e26..."`)
  - [x] Test hash idempotency: same payload → same hash → `hash_matches` returns true → `put_file` not called (mock or test via checkpoint state)
  - [x] Run `cargo test` — must pass with 0 failures
  - [x] Run `cargo clippy --all-targets -- -D warnings` — must produce 0 warnings

## Dev Notes

### Module Responsibility Summary

`sync.rs` is the **orchestration layer** — it coordinates existing modules but owns no business logic itself:

| Module | Role in this story |
|---|---|
| `jsonl_parser::parse_date_range` | Supplies `HashMap<String, DailyActivity>` for the date range |
| `sync::sha256_hex` | Hashes serialized payload for idempotency check |
| `checkpoint::Checkpoint::hash_matches` | Returns true if stored hash equals computed hash |
| `github_api::GithubApi::put_file` | Pushes JSON to vibestats-data Hive path |
| `checkpoint::Checkpoint::update_hash` | Stores new hash after successful push |
| `checkpoint::Checkpoint::set_auth_error` | Sets `auth_error = true` on 401 |
| `checkpoint::Checkpoint::save` | Persists checkpoint to disk |
| `logger::error` | Logs all errors (no stdout/stderr in non-test code) |

### `sync::run` Signature

```rust
pub fn run(start_date: &str, end_date: &str) {
    // Loads config and checkpoint internally
    // Returns () — callers do not inspect return value
    // NEVER calls std::process::exit — caller (main.rs hook handler) does that
}
```

Config and checkpoint are loaded internally by `run`:
```rust
let config = Config::load_or_exit(); // exits 0 with message if config missing
let checkpoint_path = /* ~/.config/vibestats/checkpoint.toml */;
let mut checkpoint = Checkpoint::load(&checkpoint_path);
let api = GithubApi::new(&config.oauth_token, &config.vibestats_data_repo);
```

Use `Config::load_or_exit()` (already implemented in `config.rs` — exits 0 with helpful message on error, which is the correct NFR10-compliant behavior for callers).

### Checkpoint Path

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

If `HOME` is not set, `checkpoint_path()` returns `None` — treat as if checkpoint is empty (`Checkpoint::default()`) and skip saving.

### Payload JSON Serialization

The payload pushed to GitHub is a compact JSON string (no spaces):
```
{"sessions":4,"active_minutes":87}
```

Use `serde_json::json!` macro or manual format:
```rust
let payload = format!(
    r#"{{"sessions":{},"active_minutes":{}}}"#,
    activity.sessions, activity.active_minutes
);
```

**Critical:** Serialize field order must be deterministic. Using the explicit `format!` approach (not `serde_json::to_string`) guarantees stable field order (`sessions` always before `active_minutes`), ensuring the same JSONL data always produces the same hash. Do NOT use `serde_json::to_string` on `DailyActivity` — serde struct field order is implementation-defined (even if stable in practice, we must guarantee it explicitly).

### SHA256 Hash — Std-Only Implementation

Do NOT add a `sha2` crate. Implement SHA256 using std only:

```rust
fn sha256_hex(data: &[u8]) -> String {
    // SHA256 initial hash values (first 32 bits of fractional parts of square roots of primes 2..19)
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];
    // SHA256 round constants (first 32 bits of fractional parts of cube roots of primes 2..311)
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5,
        0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
        0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc,
        0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
        0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3,
        0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5,
        0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
        0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];

    // Pre-processing: padding
    let bit_len = (data.len() as u64).wrapping_mul(8);
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0x00);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    // Process each 512-bit chunk
    for chunk in msg.chunks(64) {
        let mut w = [0u32; 64];
        for (i, b) in chunk.chunks(4).enumerate().take(16) {
            w[i] = u32::from_be_bytes([b[0], b[1], b[2], b[3]]);
        }
        for i in 16..64 {
            let s0 = w[i-15].rotate_right(7) ^ w[i-15].rotate_right(18) ^ (w[i-15] >> 3);
            let s1 = w[i-2].rotate_right(17) ^ w[i-2].rotate_right(19) ^ (w[i-2] >> 10);
            w[i] = w[i-16].wrapping_add(s0).wrapping_add(w[i-7]).wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] =
            [h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]];
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);
            hh = g; g = f; f = e;
            e = d.wrapping_add(temp1);
            d = c; c = b; b = a;
            a = temp1.wrapping_add(temp2);
        }
        h[0] = h[0].wrapping_add(a); h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c); h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e); h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g); h[7] = h[7].wrapping_add(hh);
    }

    format!(
        "{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}",
        h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]
    )
}
```

**Test vectors:**
- `sha256_hex(b"")` → `"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"`
- `sha256_hex(b"hello")` → `"2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"`
- `sha256_hex(b"{\"sessions\":4,\"active_minutes\":87}")` → computable from spec above

### Hive Path Construction

```rust
fn hive_path(date: &str, machine_id: &str) -> String {
    // date is "YYYY-MM-DD" — indexing is safe because parse_date_range only returns
    // dates that were extracted from JSONL timestamps in that exact format.
    let year  = &date[0..4];
    let month = &date[5..7];
    let day   = &date[8..10];
    format!(
        "machines/year={}/month={}/day={}/harness=claude/machine_id={}/data.json",
        year, month, day, machine_id
    )
}
```

Month and day come from JSONL timestamps already zero-padded (they are extracted as `ts[5..7]` and `ts[8..10]` in `jsonl_parser.rs`), so no additional padding is needed.

### Error Handling Contract

| Error | Action in `sync.rs` |
|---|---|
| `github_api` returns `Err` with 401 context | `checkpoint.set_auth_error()`, `logger::error(...)`, continue to next date |
| `github_api` returns `Err` (non-401) | `logger::error(...)`, continue to next date |
| `Config::load_or_exit()` fails | Exits 0 with message (handled inside `config.rs`) |
| `checkpoint.save` fails | Log via `logger::error`, do NOT exit non-zero |

**`sync.rs` NEVER calls `std::process::exit`.** Callers (hook handlers in Stories 3.2, 3.3) handle exit after calling `sync::run`.

Note: `github_api::put_file` returns `Result<(), Box<dyn std::error::Error>>`. The error type does not carry the HTTP status code. In Story 2.5's review notes, the 401 check is the **caller's** (sync.rs) responsibility — the API module returns `Err` on 401 and logs it, but sync.rs must decide what flag to set. Since `github_api.rs` already logs the 401 error, `sync.rs` only needs to call `checkpoint.set_auth_error()` and continue. A simple approach: treat ALL `Err` from `put_file` the same way (log + continue); set `auth_error` always on error to err on the side of caution (the user can clear it via `vibestats auth`). Alternatively, parse the error message string for "401". The simplest correct approach: set `auth_error` on any API error, since the only way to clear it is `vibestats auth` and it's better to over-notify than under-notify.

### Module File Location

```
src/
├── main.rs           ← add `mod sync;` (alongside checkpoint, config, github_api, jsonl_parser, logger)
├── checkpoint.rs     ← EXISTING — provides Checkpoint struct and all checkpoint methods
├── config.rs         ← EXISTING — provides Config::load_or_exit(), oauth_token, machine_id, vibestats_data_repo
├── github_api.rs     ← EXISTING — provides GithubApi::new(), put_file()
├── jsonl_parser.rs   ← EXISTING — provides parse_date_range(), DailyActivity
├── logger.rs         ← EXISTING — provides logger::error(), logger::warn(), logger::info()
└── sync.rs           ← NEW FILE (this story)
```

### Existing Public APIs to Use

**`jsonl_parser::parse_date_range`:**
```rust
pub fn parse_date_range(start: &str, end: &str) -> HashMap<String, DailyActivity>
// DailyActivity: { sessions: u32, active_minutes: u32 }
```

**`github_api::GithubApi`:**
```rust
pub fn new(token: &str, repo: &str) -> Self
pub fn put_file(&self, path: &str, content: &str) -> Result<(), Box<dyn std::error::Error>>
```

**`checkpoint::Checkpoint`:**
```rust
pub fn load(path: &Path) -> Self        // fail-open: returns default if missing
pub fn save(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>>
pub fn hash_matches(&self, date: &str, hash: &str) -> bool
pub fn update_hash(&mut self, date: &str, hash: &str)
pub fn set_auth_error(&mut self)
```

**`config::Config`:**
```rust
pub fn load_or_exit() -> Config         // exits 0 with message on error (NFR10 compliant)
// Fields: oauth_token: String, machine_id: String, vibestats_data_repo: String
```

**`logger`:**
```rust
pub fn error(message: &str)
pub fn warn(message: &str)
pub fn info(message: &str)
```

### Imports Required in `sync.rs`

```rust
use crate::checkpoint::Checkpoint;
use crate::config::Config;
use crate::github_api::GithubApi;
use crate::jsonl_parser;
use crate::logger;
use std::path::PathBuf;
```

### Existing Crates (No New Dependencies Allowed)

All required crates are already in `Cargo.toml`:

| Crate | Usage |
|---|---|
| `serde_json 1.0` | `serde_json::json!` for payload construction (optional) |

**Do NOT add:** `sha2`, `ring`, `openssl`, or any cryptography crate. Implement SHA256 with std only (provided above).

**Confirmed `Cargo.toml` dependencies:**
```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
ureq = "2.10"
toml = "0.8"
```

### `#![allow(dead_code)]` Pattern

Add `#![allow(dead_code)]` at the top of `sync.rs`. The `run` function will not be called from `main.rs` until Stories 3.2–3.4 wire up the hook and CLI handlers. This pattern was established in Stories 2.3, 2.4, 2.5.

### Worktree / Cargo Isolation

The worktree is nested inside the main repo. `Cargo.toml` at the repo root already has `[workspace]` set. Do NOT add another `[workspace]` section.

Run all verification from the **repo root** (not from inside the worktree directory):
```bash
cargo build
cargo test
cargo clippy --all-targets -- -D warnings
```

### Unique Temp Files in Tests

If tests write any files, use unique temp file names (pid + nanos + atomic counter) to prevent races in parallel test runs — pattern established in Story 2.3:
```rust
use std::sync::atomic::{AtomicU64, Ordering};
static COUNTER: AtomicU64 = AtomicU64::new(0);
let n = COUNTER.fetch_add(1, Ordering::SeqCst);
let name = format!("vibestats_test_{}_{}_{}", std::process::id(),
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default().as_nanos(), n);
let path = std::env::temp_dir().join(name);
```

### Previous Story Learnings

From Story 2.5 (`github_api.rs`):
- `#![allow(dead_code)]` is required for infrastructure modules whose callers land in future stories
- `#![allow(clippy::result_large_err)]` is needed when `ureq::Error` is in a return type — check if needed
- No `unwrap()` or `expect()` in non-test code paths
- `cargo clippy --all-targets -- -D warnings` (not just `-- -D warnings`) catches all-targets warnings
- PRs must include `Closes #18` in the description

From Story 2.3 (`checkpoint.rs`):
- Atomic save via tmp + rename pattern (already in `checkpoint.save` — no need to replicate)
- `std::process::exit` must never be called inside modules — only by `main.rs` or hook handlers
- Use `std::env::temp_dir()` for test temp files (not `/tmp` directly)

From Story 2.4 (`jsonl_parser.rs`):
- `parse_date_range` returns an empty `HashMap` (not an error) if `HOME` is not set or projects dir is unreadable — `sync::run` must handle empty map gracefully (just save checkpoint with no changes)

### Architecture Constraints Summary

| Constraint | NFR/Source | Impact on This Story |
|---|---|---|
| Exit 0 on all errors | NFR10 | `sync.rs` returns `()` — never calls `exit` |
| Silent during session | NFR11 | Log via `logger::error` only — no stdout/stderr |
| Idempotent sync | NFR12 | Hash check before every `put_file` call |
| Single HTTP module | architecture.md | All GitHub HTTP via `github_api.rs` — sync.rs never calls `ureq` directly |
| No async runtime | architecture.md | `sync::run` is synchronous; no `tokio`, no `async fn` |
| No new crates | Story scope | SHA256 implemented std-only |
| snake_case filenames | architecture.md | File: `src/sync.rs` |
| Co-located unit tests | architecture.md | `#[cfg(test)]` module inside `sync.rs` |

### Anti-Patterns to Prevent

- Do NOT inline HTTP calls in `sync.rs` — all GitHub HTTP goes through `github_api.rs`
- Do NOT use `serde_json::to_string` on `DailyActivity` directly — use explicit `format!` for deterministic field ordering
- Do NOT add `sha2`, `ring`, or any crypto crate — implement SHA256 std-only
- Do NOT call `std::process::exit` in `sync.rs` — callers handle exit
- Do NOT write to stdout/stderr in non-test code — use `logger::error/warn/info` only
- Do NOT add a second `[workspace]` to `Cargo.toml`
- Do NOT use `unwrap()` or `expect()` in non-test code

### Project Structure Notes

- New file: `src/sync.rs`
- Modified file: `src/main.rs` (add `mod sync;` alongside existing `mod` declarations)
- No other files modified

### References

- Story requirements: [Source: _bmad-output/planning-artifacts/epics.md#Story 3.1]
- Sync operation spec (all types): [Source: _bmad-output/planning-artifacts/architecture.md#API & Communication Patterns — Sync operation]
- Error handling table: [Source: _bmad-output/planning-artifacts/architecture.md#API & Communication Patterns — Error handling]
- Idempotency spec: [Source: _bmad-output/planning-artifacts/architecture.md#API & Communication Patterns — Idempotency — three levels]
- Hive path format: [Source: docs/schemas.md#1. Machine Day File — Location]
- Machine Day File content schema: [Source: docs/schemas.md#1. Machine Day File — File Content]
- Checkpoint fields: [Source: docs/schemas.md#checkpoint.toml]
- Config fields: [Source: docs/schemas.md#config.toml]
- NFR10 (hook non-interference): [Source: _bmad-output/planning-artifacts/epics.md#NonFunctional Requirements]
- NFR11 (silent sync failure): [Source: _bmad-output/planning-artifacts/epics.md#NonFunctional Requirements]
- NFR12 (idempotent sync): [Source: _bmad-output/planning-artifacts/epics.md#NonFunctional Requirements]
- Module boundary rules: [Source: _bmad-output/planning-artifacts/architecture.md#Process Patterns]
- Previous story patterns: [Source: _bmad-output/implementation-artifacts/2-5-implement-github-api-module.md#Dev Notes]
- Story 2.3 patterns: [Source: _bmad-output/implementation-artifacts/2-3-implement-checkpoint-module.md]
- GH Issue: #18

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

None — implementation proceeded cleanly with no blocking issues.

### Completion Notes List

- Implemented `src/sync.rs` with `pub fn run(start_date, end_date)` orchestration function
- Implemented std-only SHA256 (`sha256_hex`) using the provided constants and algorithm — verified against test vectors
- Implemented `hive_path` helper producing correct Hive partition path format
- Used explicit `format!` for payload serialization (deterministic field order: `sessions` before `active_minutes`)
- All API errors set `auth_error` in checkpoint (err on the side of caution per story note)
- Checkpoint always saved after processing all dates even if some dates failed
- `mod sync;` added to `main.rs` alongside existing module declarations
- 7 new unit tests added in `#[cfg(test)]` module: SHA256 test vectors (empty string, "hello"), determinism, hive_path correctness (two cases), idempotency via checkpoint state, different-payload hashes
- All 72 tests pass (65 existing + 7 new); 0 clippy warnings

### File List

- `src/sync.rs` (new)
- `src/main.rs` (modified — added `mod sync;`)
- `_bmad-output/implementation-artifacts/3-1-implement-core-sync-orchestration.md` (updated — task checkboxes, status, Dev Agent Record)
