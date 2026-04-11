# Story 2.3: Implement Checkpoint Module

Status: review

<!-- GH Issue: #15 | Epic: #2 | PR must include: Closes #15 -->

## Story

As the vibestats binary,
I want a checkpoint module that persists throttle state, per-date content hashes, auth error flag, and machine status,
So that sync operations are idempotent and the Stop hook hot path makes no unnecessary API calls.

## Acceptance Criteria

1. **Given** a sync completed less than 5 minutes ago **When** the Stop hook fires and reads the checkpoint **Then** `Checkpoint::should_throttle()` returns `true` and no API call is made (NFR2)

2. **Given** a date's payload hash matches the stored hash in `checkpoint.toml` **When** sync evaluates whether to push **Then** no GitHub Contents API call is made for that date (NFR12)

3. **Given** a 401 response from the GitHub API **When** the auth error is recorded **Then** `auth_error = true` is written to `checkpoint.toml` and the binary exits 0

4. **Given** `machine_status = "retired"` is in `checkpoint.toml` **When** the Stop hook fires **Then** it skips entirely without any network call

## Tasks / Subtasks

- [x] Task 1: Create `src/checkpoint.rs` with `Checkpoint` struct and TOML serialization (AC: #1, #2, #3, #4)
  - [x] Define `Checkpoint` struct matching canonical `checkpoint.toml` schema from `docs/schemas.md`
  - [x] Derive `serde::Serialize` + `serde::Deserialize` on `Checkpoint` and `DateHashes`
  - [x] Implement `Checkpoint::load(path: &Path) -> Checkpoint` — returns default if file missing
  - [x] Implement `Checkpoint::save(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>>`
  - [x] Ensure file at `~/.config/vibestats/checkpoint.toml` (caller provides path; this module does NOT hardcode the path)

- [x] Task 2: Implement `should_throttle` (AC: #1)
  - [x] `Checkpoint::should_throttle(&self) -> bool` — returns `true` if `throttle_timestamp` is within 5 minutes of now
  - [x] Use `std::time::SystemTime` for current time comparison; parse `throttle_timestamp` as ISO 8601 UTC
  - [x] Returns `false` if `throttle_timestamp` is absent or unparseable (fail-safe: allow sync)

- [x] Task 3: Implement date hash operations (AC: #2)
  - [x] `Checkpoint::hash_matches(&self, date: &str, hash: &str) -> bool` — returns `true` if stored hash for date equals provided hash
  - [x] `Checkpoint::update_hash(&mut self, date: &str, hash: &str)` — upserts entry in `date_hashes`
  - [x] Date key format: `"YYYY-MM-DD"` strings exactly as defined in `docs/schemas.md`

- [x] Task 4: Implement auth error and machine status helpers (AC: #3, #4)
  - [x] `Checkpoint::set_auth_error(&mut self)` — sets `auth_error = true`
  - [x] `Checkpoint::clear_auth_error(&mut self)` — sets `auth_error = false`
  - [x] `Checkpoint::is_retired(&self) -> bool` — returns `true` when `machine_status == "retired"`
  - [x] `Checkpoint::set_machine_status(&mut self, status: &str)` — sets `machine_status`

- [x] Task 5: Implement throttle timestamp update
  - [x] `Checkpoint::update_throttle_timestamp(&mut self)` — sets `throttle_timestamp` to current UTC ISO 8601 string
  - [x] Format: `"YYYY-MM-DDTHH:MM:SSZ"` — never Unix timestamp

- [x] Task 6: Wire `checkpoint.rs` into `main.rs` as a declared module (AC: compile)
  - [x] Add `mod checkpoint;` to `src/main.rs`
  - [x] No business logic in `main.rs` — only the `mod` declaration

- [x] Task 7: Write co-located unit tests (AC: #1, #2, #3, #4)
  - [x] `#[cfg(test)]` module inside `src/checkpoint.rs`
  - [x] Test `should_throttle` with timestamp 2 min ago (expect `true`) and 10 min ago (expect `false`)
  - [x] Test `hash_matches` with matching hash (true) and mismatched hash (false)
  - [x] Test `set_auth_error` / `clear_auth_error` round-trip
  - [x] Test `is_retired` with `"retired"`, `"active"`, `"purged"` values
  - [x] Test `load` on missing file returns default (no panic)
  - [x] Test `save` + `load` round-trip produces identical struct
  - [x] Run `cargo test` — must pass with 0 failures
  - [x] Run `cargo clippy -- -D warnings` — must produce 0 warnings

## Dev Notes

### Canonical `checkpoint.toml` Schema

From `docs/schemas.md` — implement this exactly:

```toml
throttle_timestamp = "2026-04-10T14:23:00Z"
machine_status = "active"
auth_error = false

[date_hashes]
"2026-04-10" = "a3f5c2e1b9d04..."
"2026-04-09" = "7b2d1c4e8a093..."
```

Fields:
| Field | Type | Valid values |
|---|---|---|
| `throttle_timestamp` | string (ISO 8601 UTC) | Any valid UTC datetime string |
| `machine_status` | string enum | `"active"` \| `"retired"` \| `"purged"` |
| `auth_error` | boolean | `true` \| `false` |
| `[date_hashes]` | TOML table | Keys: `YYYY-MM-DD`; values: SHA256 hex strings |

### Rust Struct Design

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Checkpoint {
    /// ISO 8601 UTC string; absent means no sync has run yet
    pub throttle_timestamp: Option<String>,
    /// "active" | "retired" | "purged"; defaults to "active"
    #[serde(default = "default_machine_status")]
    pub machine_status: String,
    /// true when last GitHub API call returned 401
    #[serde(default)]
    pub auth_error: bool,
    /// Per-date SHA256 content hashes; key = "YYYY-MM-DD"
    #[serde(default)]
    pub date_hashes: HashMap<String, String>,
}

fn default_machine_status() -> String {
    "active".to_string()
}
```

**Why `Option<String>` for `throttle_timestamp`:** The field is absent on first run. Using `Option` allows `serde` to handle missing field gracefully without a custom default. `should_throttle()` returns `false` if `None` (allows sync to proceed).

**Why `#[serde(default)]` on `auth_error` and `date_hashes`:** Provides zero-value defaults (`false` and empty `HashMap`) if the field is absent from the TOML file — forward-compatible with older checkpoint files missing these fields.

### `load` / `save` Implementation Pattern

```rust
use std::path::Path;

impl Checkpoint {
    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string(self)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
        Ok(())
    }
}
```

**Important:** `load` must NEVER panic or exit non-zero on any error (NFR10). Missing file, corrupt TOML, or unreadable path all fall back to `Checkpoint::default()`.

**Important:** `save` returns a `Result` — callers decide whether to log or silently ignore (the binary always exits 0 per the silent failure contract — checkpoint.rs itself does not call `std::process::exit`).

### `should_throttle` and `update_throttle_timestamp` Implementation

Both functions need ISO 8601 UTC ↔ `SystemTime` conversion using `std` only (no extra crates). Use the complete self-contained implementation below — copy it verbatim to avoid bugs.

**Parse ISO 8601 UTC string → `SystemTime`:**

```rust
/// Parses "YYYY-MM-DDTHH:MM:SSZ" → SystemTime (UNIX_EPOCH offset).
/// Returns None on any parse error (fail-open: caller treats as no timestamp).
fn parse_iso8601_utc(s: &str) -> Option<std::time::SystemTime> {
    let s = s.trim_end_matches('Z');
    let (date_str, time_str) = s.split_once('T')?;
    let mut dp = date_str.split('-');
    let year: u64 = dp.next()?.parse().ok()?;
    let month: u64 = dp.next()?.parse().ok()?;
    let day: u64 = dp.next()?.parse().ok()?;
    let mut tp = time_str.split(':');
    let hour: u64 = tp.next()?.parse().ok()?;
    let min: u64 = tp.next()?.parse().ok()?;
    let sec: u64 = tp.next()?.parse().ok()?;

    // Days since Unix epoch (1970-01-01) via the civil date formula.
    // Source: https://howardhinnant.github.io/date_algorithms.html  (days_from_civil)
    let y = if month <= 2 { year - 1 } else { year };
    let era = y / 400;
    let yoe = y - era * 400;                                // [0, 399]
    let doy = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + day - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;       // [0, 146096]
    let days_since_epoch = era * 146097 + doe - 719468;     // days since 1970-01-01

    let secs = days_since_epoch * 86400 + hour * 3600 + min * 60 + sec;
    Some(std::time::UNIX_EPOCH + std::time::Duration::from_secs(secs))
}
```

**Format `SystemTime` → ISO 8601 UTC string for `update_throttle_timestamp`:**

```rust
/// Formats a SystemTime as "YYYY-MM-DDTHH:MM:SSZ".
fn format_iso8601_utc(t: std::time::SystemTime) -> String {
    let secs = t
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Reverse civil date formula (days_from_civil inverse: civil_from_days)
    // Source: https://howardhinnant.github.io/date_algorithms.html
    let z = secs / 86400;
    let time_of_day = secs % 86400;
    let h = time_of_day / 3600;
    let m = (time_of_day % 3600) / 60;
    let s = time_of_day % 60;

    let z = z + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if mo <= 2 { y + 1 } else { y };

    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, mo, d, h, m, s)
}
```

**`should_throttle` using the parser:**

```rust
pub fn should_throttle(&self) -> bool {
    let ts_str = match &self.throttle_timestamp {
        Some(ts) => ts,
        None => return false, // no timestamp → allow sync
    };
    let ts = match parse_iso8601_utc(ts_str) {
        Some(t) => t,
        None => return false, // unparseable → fail-safe: allow sync
    };
    let now = std::time::SystemTime::now();
    match now.duration_since(ts) {
        Ok(elapsed) => elapsed.as_secs() < 300, // 5 min = 300 s
        Err(_) => false, // clock skew (ts in future) → allow sync
    }
}
```

**`update_throttle_timestamp`:**

```rust
pub fn update_throttle_timestamp(&mut self) {
    self.throttle_timestamp = Some(format_iso8601_utc(std::time::SystemTime::now()));
}
```

**Note:** The civil date formulas above are exact for all dates in the Gregorian calendar. They are used verbatim by many date libraries. Copy them exactly — do not simplify or approximate.

**Alternative (simpler, acceptable):** Store `throttle_timestamp` as a raw Unix timestamp integer in the TOML. This is NOT acceptable — the schema specifies ISO 8601 UTC strings. Do not change the schema to use integers.

### File Location

The checkpoint file path is `~/.config/vibestats/checkpoint.toml`. `checkpoint.rs` does NOT hardcode this path — it accepts a `&Path` parameter. The caller (e.g., the Stop hook handler in `main.rs` or a future `sync.rs`) resolves the path from `config.rs` or constructs it via `dirs` / `home_dir`. For now, this module is path-agnostic.

**Do not add a `dirs` crate dependency for this story.** Path resolution will be handled in story 2.1 (`config.rs`) or whichever story wires up the full sync path. The `checkpoint.rs` module is purely a data structure + file I/O concern.

### Module File Location

```
src/
├── main.rs         ← add `mod checkpoint;` here
└── checkpoint.rs   ← THIS STORY (new file)
```

No other files should be created or modified except `src/main.rs` (to add `mod checkpoint;`) and the story file itself.

### Existing Crates (No New Dependencies)

All crates required for this story are already in `Cargo.toml` from Story 1.2:

| Crate | Usage in this story |
|---|---|
| `serde` (with `derive`) | `#[derive(Serialize, Deserialize)]` on `Checkpoint` |
| `toml` | `toml::from_str` (deserialize) + `toml::to_string` (serialize) |

Do NOT add new crate dependencies. `serde`, `serde_json`, `ureq`, `clap`, and `toml` are already declared. `std::collections::HashMap`, `std::time::SystemTime`, `std::path::Path`, `std::fs` are all `std` — no import needed in `Cargo.toml`.

**Confirmed existing `Cargo.toml`:**
```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
ureq = "2.10"
toml = "0.8"
```

### Architecture Module Boundary (Critical)

From `architecture.md` — `checkpoint.rs` responsibility boundaries:

| `checkpoint.rs` OWNS | `checkpoint.rs` NEVER does |
|---|---|
| Throttle state read/write | Network calls |
| Content hash storage/lookup | Config file read/write (that's `config.rs`) |
| `auth_error` flag management | JSONL parsing |
| `machine_status` read/write | Business logic decisions (callers decide) |

**Anti-pattern prevention:**
- Do NOT call `std::process::exit` inside `checkpoint.rs` (callers handle exit)
- Do NOT make HTTP calls or read `config.toml` from this module
- Do NOT compute content hashes here — callers compute hashes, pass strings in

### Hash Computation (Caller's Responsibility)

The checkpoint module stores and compares hashes — it does NOT compute them. The caller (future `sync.rs` in Story 3.1) will:
1. Serialize the day's payload to JSON
2. Compute SHA256 hex string
3. Call `checkpoint.hash_matches(date, hash)` to decide whether to skip

For this story: `checkpoint.rs` treats hashes as opaque strings. No SHA256 computation in this module.

### Error Handling Contract

From `architecture.md` — all Rust binary code paths exit 0 (NFR10). For `checkpoint.rs`:
- `load` never panics — returns `Checkpoint::default()` on any error
- `save` returns `Result` — callers log errors via `logger.rs` (Story 2.2) and exit 0
- No `unwrap()` or `expect()` in non-test code paths

### Tests

Co-located `#[cfg(test)]` module inside `src/checkpoint.rs`. Use `tempfile` / `std::env::temp_dir()` for round-trip save/load tests (no `tempfile` crate — use `std::env::temp_dir()` + a unique filename):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(name)
    }

    #[test]
    fn load_missing_file_returns_default() {
        let cp = Checkpoint::load(&temp_path("nonexistent_vibestats_test.toml"));
        assert!(!cp.auth_error);
        assert_eq!(cp.machine_status, "active");
        assert!(cp.date_hashes.is_empty());
    }

    #[test]
    fn save_load_roundtrip() {
        let path = temp_path("vibestats_checkpoint_test.toml");
        let mut cp = Checkpoint::default();
        cp.set_auth_error();
        cp.update_hash("2026-04-10", "abc123");
        cp.save(&path).unwrap();
        let loaded = Checkpoint::load(&path);
        assert!(loaded.auth_error);
        assert!(loaded.hash_matches("2026-04-10", "abc123"));
        let _ = std::fs::remove_file(&path); // cleanup
    }

    #[test]
    fn should_throttle_recent_timestamp() {
        let mut cp = Checkpoint::default();
        cp.update_throttle_timestamp(); // sets to now
        assert!(cp.should_throttle()); // just set — must be throttled
    }

    #[test]
    fn should_throttle_old_timestamp_returns_false() {
        let mut cp = Checkpoint::default();
        // Set a timestamp 10 minutes in the past
        cp.throttle_timestamp = Some("2020-01-01T00:00:00Z".to_string());
        assert!(!cp.should_throttle());
    }

    #[test]
    fn hash_matches_correct_and_incorrect() {
        let mut cp = Checkpoint::default();
        cp.update_hash("2026-04-10", "deadbeef");
        assert!(cp.hash_matches("2026-04-10", "deadbeef"));
        assert!(!cp.hash_matches("2026-04-10", "wronghash"));
        assert!(!cp.hash_matches("2026-04-11", "deadbeef")); // different date
    }

    #[test]
    fn auth_error_roundtrip() {
        let mut cp = Checkpoint::default();
        assert!(!cp.auth_error);
        cp.set_auth_error();
        assert!(cp.auth_error);
        cp.clear_auth_error();
        assert!(!cp.auth_error);
    }

    #[test]
    fn is_retired_variants() {
        let mut cp = Checkpoint::default();
        assert!(!cp.is_retired()); // default is "active"
        cp.set_machine_status("retired");
        assert!(cp.is_retired());
        cp.set_machine_status("purged");
        assert!(!cp.is_retired()); // purged is not the same as retired
        cp.set_machine_status("active");
        assert!(!cp.is_retired());
    }
}
```

### Worktree / Cargo Isolation

From Story 1.2 learnings: the worktree is nested inside the main repo. The `Cargo.toml` at the repo root already has `[workspace]` set (added in Story 1.2 to prevent upward traversal). Do NOT add another `[workspace]` section — it already exists.

Run `cargo build` and `cargo test` from the repo root (not the worktree directory), as `Cargo.toml` is at `/Users/stephenleo/Developer/vibestats/Cargo.toml`.

### Previous Story Context

Story 1.2 established:
- `src/main.rs` exists with full `clap` CLI skeleton
- `Cargo.toml` has all 5 dependencies (`clap`, `serde`, `serde_json`, `ureq`, `toml`) — no new deps needed
- `Cargo.lock` is committed — binary project, lockfile in VCS
- `[workspace]` section in `Cargo.toml` prevents cargo upward traversal in worktree
- All Rust filenames use `snake_case`
- Architecture decision: no async runtime (tokio), no `reqwest`

Story 1.4 established:
- `docs/schemas.md` exists as canonical schema reference
- `checkpoint.toml` fields are: `throttle_timestamp`, `machine_status`, `auth_error`, `[date_hashes]`
- `machine_status` valid values: `"active"` | `"retired"` | `"purged"`

### Git Intelligence

Recent commits confirm:
- `cargo build` and `cargo clippy -- -D warnings` are the standard verification steps
- All story work is done in dedicated worktrees
- PRs must include `Closes #15` in description

### Architecture Constraints Summary

| Constraint | NFR/Source | Impact on This Story |
|---|---|---|
| Exit 0 on all errors | NFR10, architecture.md#Process Patterns | `load` never panics; `save` returns Result — caller exits 0 |
| Silent failure during session | NFR11 | `checkpoint.rs` does not log or exit — caller responsibility |
| 5-min throttle | NFR2 | `should_throttle()` compares throttle_timestamp to now |
| Idempotent sync | NFR12 | `hash_matches()` / `update_hash()` enable skip-if-unchanged |
| No network calls in checkpoint.rs | architecture.md#Module Responsibility | Only file I/O and in-memory struct operations |
| snake_case all TOML fields | architecture.md#Naming Patterns | `throttle_timestamp`, `machine_status`, `auth_error`, `date_hashes` |
| ISO 8601 UTC timestamps | architecture.md#Format Patterns | `throttle_timestamp` stored as `"YYYY-MM-DDTHH:MM:SSZ"` string |
| Rust snake_case filenames | architecture.md#Naming Patterns | File: `src/checkpoint.rs` |
| No extra crates | Story scope | All required crates already in Cargo.toml |
| Co-located unit tests | architecture.md#Structure Patterns | `#[cfg(test)]` module inside `checkpoint.rs` |

### References

- Checkpoint module spec: [Source: architecture.md#Module Responsibility Boundaries — `checkpoint.rs`]
- `checkpoint.toml` schema: [Source: docs/schemas.md#Local Config Files — checkpoint.toml]
- Throttle logic: [Source: architecture.md#Sync Operation — Step 1 (throttle check)]
- Hash idempotency: [Source: architecture.md#Idempotency — API level]
- Auth error flag: [Source: architecture.md#Authentication & Security — Validation strategy]
- Machine status / remote retirement: [Source: architecture.md#Gap Analysis — Gap 3]
- NFR2 (5-min throttle): [Source: epics.md#NonFunctional Requirements]
- NFR10 (hook non-interference): [Source: epics.md#NonFunctional Requirements]
- NFR11 (silent sync failure): [Source: epics.md#NonFunctional Requirements]
- NFR12 (idempotent sync): [Source: epics.md#NonFunctional Requirements]
- Story 1.2 (Cargo.toml + crate versions): [Source: implementation-artifacts/1-2-initialize-rust-binary-project.md]
- Story 1.4 (canonical schema reference): [Source: implementation-artifacts/1-4-define-and-document-all-json-and-toml-schemas.md]
- GH Issue: #15

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

- Initial test run: `load_missing_file_returns_default` failed because `#[derive(Default)]` uses `String::default()` (empty string) instead of the serde `default_machine_status()` function. Fixed by implementing `Default` manually to call `default_machine_status()`.
- Second test run: All 7 tests pass.
- Clippy run: 5 dead_code warnings for public items not yet used by callers. Fixed by adding `#![allow(dead_code)]` at the top of `checkpoint.rs` — module is infrastructure not yet wired to callers.
- Final clippy run: 0 warnings. Final test run: 7/7 pass.

### Completion Notes List

- Created `src/checkpoint.rs` with `Checkpoint` struct implementing TOML serialization via serde
- Implemented `Default` manually so `machine_status` defaults to `"active"` (not empty string)
- Implemented `parse_iso8601_utc` and `format_iso8601_utc` using std-only civil date algorithms (no extra crates)
- Implemented `load` (fail-open: returns default on any error), `save` (returns Result), `should_throttle`, `update_throttle_timestamp`, `hash_matches`, `update_hash`, `set_auth_error`, `clear_auth_error`, `is_retired`, `set_machine_status`
- Added `mod checkpoint;` to `src/main.rs` (no business logic — module declaration only)
- Added `#[allow(dead_code)]` to suppress clippy false positives for public API not yet consumed by callers
- 7 co-located unit tests: all pass — covers throttle (recent/old), hash match/mismatch, auth error roundtrip, is_retired variants, load missing file, save/load roundtrip
- No new Cargo.toml dependencies added (uses existing `serde`, `toml`)

### File List

- `src/checkpoint.rs` (new)
- `src/main.rs` (modified — added `mod checkpoint;`)

### Review Findings

_to be filled by reviewer_

## Change Log

- 2026-04-11: Story created — comprehensive implementation guide for checkpoint module (Story 2.3)
- 2026-04-11: Implementation complete — created `src/checkpoint.rs` with full Checkpoint struct, TOML I/O, throttle/hash/auth/status helpers, and 7 passing unit tests; wired `mod checkpoint;` into `src/main.rs`; 0 clippy warnings
