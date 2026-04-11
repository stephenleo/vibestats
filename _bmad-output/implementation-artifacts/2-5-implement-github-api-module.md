# Story 2.5: Implement GitHub API Module

Status: done

<!-- GH Issue: #17 | Epic: #2 | PR must include: Closes #17 -->

## Story

As the vibestats binary,
I want a single GitHub API module that handles all Contents API calls with retry and error handling,
So that no other module makes direct HTTP calls to GitHub and the silent failure contract is enforced.

## Acceptance Criteria

1. **Given** a day file does not yet exist in vibestats-data **When** `github_api::put_file(path, content)` is called **Then** it performs a PUT without a SHA (first-time create) and returns success

2. **Given** a day file already exists **When** `github_api::put_file(path, content)` is called **Then** it first GETs the current SHA, then PUTs with that SHA (update pattern)

3. **Given** the API returns a 429 or 5xx response **When** `github_api` handles the error **Then** it retries with exponential backoff: 1s → 2s → 4s, max 3 attempts, logs, exits 0 (NFR15)

4. **Given** any module other than `github_api.rs` needs to call GitHub **When** it is implemented **Then** it calls functions in `github_api.rs` — no inline HTTP requests permitted elsewhere

## Tasks / Subtasks

- [x] Task 1: Create `src/github_api.rs` with `GithubApi` struct and core types (AC: #1, #2, #3, #4)
  - [x] Define `GithubApi` struct holding `token: String` and `repo: String` (from config)
  - [x] Define `GithubApiError` type for error propagation (return `Result<T, Box<dyn std::error::Error>>`)
  - [x] Implement `GithubApi::new(token: &str, repo: &str) -> Self`

- [x] Task 2: Implement `get_file_sha` helper (AC: #2)
  - [x] `GithubApi::get_file_sha(&self, path: &str) -> Result<Option<String>, Box<dyn std::error::Error>>`
  - [x] GET `https://api.github.com/repos/{repo}/contents/{path}` with `Authorization: Bearer {token}` and `User-Agent: vibestats`
  - [x] 200: parse `sha` field from JSON response body — return `Some(sha_string)`
  - [x] 404: file does not exist — return `Ok(None)`
  - [x] Other status: propagate as error (will be caught by retry wrapper)

- [x] Task 3: Implement `put_file` (AC: #1, #2)
  - [x] `GithubApi::put_file(&self, path: &str, content: &str) -> Result<(), Box<dyn std::error::Error>>`
  - [x] Call `get_file_sha` first to determine current SHA
  - [x] 404 (None): PUT body = `{ "message": "vibestats sync", "content": "<base64>" }` (no `sha` field)
  - [x] Exists (Some(sha)): PUT body = `{ "message": "vibestats sync", "content": "<base64>", "sha": "<sha>" }`
  - [x] Use `base64_encode` helper (std-only, no external crate) to base64-encode `content`
  - [x] PUT to `https://api.github.com/repos/{repo}/contents/{path}` with same auth headers
  - [x] 200 or 201: return `Ok(())`
  - [x] Other status: propagate as error

- [x] Task 4: Implement `with_retry` wrapper (AC: #3)
  - [x] `fn with_retry<F, T>(f: F) -> Result<T, Box<dyn std::error::Error>> where F: Fn() -> Result<T, ureq::Error>`
  - [x] Max 3 attempts; delays: 1s before attempt 1, 2s before attempt 2 (exponential backoff)
  - [x] Retry on: HTTP 429 (rate limit), HTTP 5xx (server error)
  - [x] Do NOT retry on: 401, 404, 422, other 4xx — propagate immediately
  - [x] On final attempt failure: log to `vibestats.log` via `logger::error(...)`, return error to caller
  - [x] Call `put_file` and `get_file_sha` through `with_retry`

- [x] Task 5: Implement base64 encoding helper (AC: #1, #2)
  - [x] `fn base64_encode(input: &[u8]) -> String` — std-only, no external crate
  - [x] Standard Base64 alphabet (`A–Z`, `a–z`, `0–9`, `+`, `/`) with `=` padding
  - [x] This is the standard alphabet used by GitHub Contents API

- [x] Task 6: Implement error handling contract (AC: #3)
  - [x] On 401: log error via `logger::error(...)`, return `Err` — do NOT touch checkpoint here; caller (`sync.rs`) sets auth_error in checkpoint
  - [x] On network timeout / DNS failure (`ureq::Error::Transport`): log error, return `Err`
  - [x] `github_api.rs` does NOT import or reference `checkpoint.rs` — module boundary enforced
  - [x] Caller (future `sync.rs`) is responsible for `std::process::exit(0)` — `github_api.rs` does NOT call `exit` itself
  - [x] No `unwrap()` or `expect()` in non-test code paths

- [x] Task 7: Wire `github_api.rs` into `main.rs` as a declared module
  - [x] Add `mod github_api;` to `src/main.rs`
  - [x] No business logic in `main.rs` — only the `mod` declaration

- [x] Task 8: Write co-located unit tests (AC: #1, #2, #3, #4)
  - [x] `#[cfg(test)]` module inside `src/github_api.rs`
  - [x] Test `base64_encode` against known vectors (e.g. `""` → `""`, `"Man"` → `"TWFu"`, `"Ma"` → `"TWE="`, `"M"` → `"TQ=="`)
  - [x] Test `get_file_sha` 404 returns `Ok(None)` (use a mock/stub approach or test the parsing logic)
  - [x] Test `with_retry` invokes `f` once on success, up to 3 times on retriable errors
  - [x] Run `cargo test` — must pass with 0 failures
  - [x] Run `cargo clippy --all-targets -- -D warnings` — must produce 0 warnings

### Review Findings

- [x] [Review][Patch] `with_retry` used `last_err.unwrap()` in non-test code, violating Task 6's "no unwrap/expect in non-test code paths" contract [src/github_api.rs:111] — fixed by seeding `last_err` with a synthetic fallback error so the fallthrough cannot panic even if `max_attempts == 0`.
- [x] [Review][Patch] `get_file_sha_inner` silently collapsed body-read and JSON-parse failures into `Ok(None)` [src/github_api.rs:199-213] — this would cause a subsequent PUT-without-sha against an existing file, trigger a 422 from GitHub, and mask the real underlying transient/server-side issue. Fixed by propagating both failures as `ureq::Error::Transport` (via `From<io::Error>`) so `with_retry` classifies them as retriable and the caller logs them.
- [x] [Review][Defer] URL path/repo components are not percent-encoded in `get_file_sha_inner` / `put_file_inner` — callers (future `sync.rs`) control the Hive path and repo slug, which are composed from alphanumerics, `=`, `/`, and `-`. Risk is low, but a defensive escape pass would harden the module. Deferred to follow-up hardening.
- [x] [Review][Defer] `test_retry_transport_error_exhausts_3_attempts` and `test_retry_succeeds_after_two_transport_errors` sleep ~3s of real time due to the hardcoded backoff delays. Not a correctness concern and the suite still finishes in ~3s, but the delay array could become an injectable parameter for faster tests. Deferred.

## Dev Notes

### GitHub Contents API — Exact Specification

**GET file (to retrieve SHA):**
```
GET https://api.github.com/repos/{owner}/{repo}/contents/{path}
Authorization: Bearer {token}
User-Agent: vibestats
Accept: application/vnd.github+json
X-GitHub-Api-Version: 2022-11-28
```
Response 200:
```json
{ "sha": "abc123...", "content": "...", "encoding": "base64", ... }
```
Response 404: file does not exist → first-time create path.

**PUT file (create or update):**
```
PUT https://api.github.com/repos/{owner}/{repo}/contents/{path}
Authorization: Bearer {token}
User-Agent: vibestats
Accept: application/vnd.github+json
X-GitHub-Api-Version: 2022-11-28
Content-Type: application/json
```
Body (create — no sha):
```json
{ "message": "vibestats sync", "content": "<base64-encoded content>" }
```
Body (update — with sha):
```json
{ "message": "vibestats sync", "content": "<base64-encoded content>", "sha": "<current sha>" }
```
Response 200 (update) or 201 (create) = success.

**Rate limit headers:**
- `X-RateLimit-Remaining`: remaining requests in current window
- `X-RateLimit-Reset`: Unix timestamp when window resets (for logging only — do not use for sleep duration)

### Hive Path Format

The `path` argument to `put_file` is the full Hive partition path within the repo:
```
machines/year=2026/month=04/day=10/harness=claude/machine_id=stephens-mbp-a1b2c3/data.json
```
Zero-padded month and day (two digits). `github_api.rs` does NOT construct this path — the caller (`sync.rs`) constructs it and passes it in.

### `ureq` Usage (Existing Dependency)

`ureq 2.10` is already in `Cargo.toml`. No new dependencies allowed.

**Making requests with ureq:**
```rust
use ureq;

// GET request
let response = ureq::get(&url)
    .set("Authorization", &format!("Bearer {}", token))
    .set("User-Agent", "vibestats")
    .set("Accept", "application/vnd.github+json")
    .set("X-GitHub-Api-Version", "2022-11-28")
    .call();

// PUT request with JSON body
let response = ureq::put(&url)
    .set("Authorization", &format!("Bearer {}", token))
    .set("User-Agent", "vibestats")
    .set("Accept", "application/vnd.github+json")
    .set("X-GitHub-Api-Version", "2022-11-28")
    .set("Content-Type", "application/json")
    .send_string(&json_body);
```

**ureq 2.x error handling:**
`ureq::call()` returns `Result<ureq::Response, ureq::Error>`.
- `Ok(response)` — HTTP response received (any status code including 4xx/5xx)
- `Err(ureq::Error::Status(code, response))` — HTTP error response
- `Err(ureq::Error::Transport(e))` — network-level error (timeout, DNS, etc.)

**Getting HTTP status code:**
```rust
match response {
    Ok(r) => r.status(), // u16
    Err(ureq::Error::Status(code, _)) => code, // u16
    Err(ureq::Error::Transport(_)) => 0, // treat as retriable
}
```

**Reading response body as string:**
```rust
let body: String = response.into_string()?;
```

**Parsing `sha` from GET response:**
Use `serde_json` (already in Cargo.toml):
```rust
let json: serde_json::Value = serde_json::from_str(&body)?;
let sha = json["sha"].as_str().map(|s| s.to_string());
```

**Building PUT body:**
Build the JSON body as a string or use `serde_json::json!` macro:
```rust
let body = if let Some(sha) = &current_sha {
    serde_json::json!({
        "message": "vibestats sync",
        "content": encoded_content,
        "sha": sha
    }).to_string()
} else {
    serde_json::json!({
        "message": "vibestats sync",
        "content": encoded_content
    }).to_string()
};
```

### Base64 Encoding — Std-Only Implementation

Do NOT add a `base64` crate. Implement standard Base64 (RFC 4648) using std only:

```rust
fn base64_encode(input: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    let mut i = 0;
    while i + 2 < input.len() {
        let b0 = input[i] as usize;
        let b1 = input[i + 1] as usize;
        let b2 = input[i + 2] as usize;
        out.push(ALPHABET[b0 >> 2] as char);
        out.push(ALPHABET[((b0 & 0x3) << 4) | (b1 >> 4)] as char);
        out.push(ALPHABET[((b1 & 0xf) << 2) | (b2 >> 6)] as char);
        out.push(ALPHABET[b2 & 0x3f] as char);
        i += 3;
    }
    match input.len() - i {
        1 => {
            let b0 = input[i] as usize;
            out.push(ALPHABET[b0 >> 2] as char);
            out.push(ALPHABET[(b0 & 0x3) << 4] as char);
            out.push('=');
            out.push('=');
        }
        2 => {
            let b0 = input[i] as usize;
            let b1 = input[i + 1] as usize;
            out.push(ALPHABET[b0 >> 2] as char);
            out.push(ALPHABET[((b0 & 0x3) << 4) | (b1 >> 4)] as char);
            out.push(ALPHABET[(b1 & 0xf) << 2] as char);
            out.push('=');
        }
        _ => {}
    }
    out
}
```

**Test vectors:**
- `""` → `""`
- `"M"` → `"TQ=="`
- `"Ma"` → `"TWE="`
- `"Man"` → `"TWFu"`
- `"Many"` → `"TWFueQ=="`
- `"hello"` → `"aGVsbG8="`

### `with_retry` Implementation Pattern

From `architecture.md`:
> All GitHub API calls go through this — no inline retry logic elsewhere

Use `ureq::Error` directly inside the retry loop (before boxing) so retriability can be checked by pattern-matching on the variant. The pattern below makes exactly 3 attempts with delays before attempts 1 and 2 (not after the final attempt):
```rust
fn call_with_retry<F, T>(f: F) -> Result<T, Box<dyn std::error::Error>>
where
    F: Fn() -> Result<T, ureq::Error>,
{
    let delays = [1u64, 2, 4]; // delay BEFORE attempt index+1 (not after last attempt)
    let max_attempts = 3;
    let mut last_err: Option<ureq::Error> = None;
    for attempt in 0..max_attempts {
        // Sleep before retry (not before the first attempt)
        if attempt > 0 {
            let delay = delays[attempt - 1];
            std::thread::sleep(std::time::Duration::from_secs(delay));
        }
        match f() {
            Ok(val) => return Ok(val),
            Err(e) => {
                let retriable = match &e {
                    ureq::Error::Status(code, _) => *code == 429 || *code >= 500,
                    ureq::Error::Transport(_) => true,
                };
                if retriable {
                    last_err = Some(e);
                    // continue to next attempt
                } else {
                    return Err(e.into()); // non-retriable: fail immediately
                }
            }
        }
    }
    // All 3 attempts exhausted
    Err(last_err.unwrap().into())
}
```

**Delay schedule:** attempt 0 → no delay, attempt 1 → 1s delay, attempt 2 → 2s delay. Total wait before 3rd attempt: 3 seconds. The 4s delay value is not used in 3-attempt scheme — use `[1u64, 2]` for the pre-attempt delay array (the architecture spec says 1s → 2s → 4s max backoff but means the delays *between* attempts, so with 3 attempts you sleep 1s then 2s).

### Error Handling Contract

From `architecture.md` — `github_api.rs` responsibility:

| HTTP Status | Action in `github_api.rs` |
|---|---|
| 200 / 201 | Return `Ok(())` |
| 401 | Log to `vibestats.log` via `logger::error`, return `Err` — caller (`sync.rs`) sets `auth_error` in checkpoint and exits 0 |
| 404 on GET | First push for this date — `Ok(None)` from `get_file_sha` |
| 429 / 5xx | Retry with exponential backoff: 1s → 2s → 4s, max 3 retries, log, return `Err` |
| Network timeout | Log to `vibestats.log`, return `Err` |

**This module does NOT call `std::process::exit`.** Callers handle exit.

### Module File Location

```
src/
├── main.rs         ← add `mod github_api;` here (alongside existing mod declarations)
└── github_api.rs   ← THIS STORY (new file)
```

No other files should be created or modified except `src/main.rs` (to add `mod github_api;`).

### Existing Crates (No New Dependencies)

All required crates are already in `Cargo.toml`:

| Crate | Usage in this story |
|---|---|
| `ureq 2.10` | HTTP GET and PUT to GitHub Contents API |
| `serde_json 1.0` | Parse SHA from GET response; build PUT body with `serde_json::json!` |
| `serde 1.0` | Not directly needed but available |

Do NOT add `base64`, `reqwest`, `tokio`, `hyper`, or any other HTTP/async crate.

**Confirmed existing `Cargo.toml`:**
```toml
[dependencies]
clap = { version = "4.5", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
ureq = "2.10"
toml = "0.8"
```

### Integration with Logger Module

`github_api.rs` uses `logger::error` (from `src/logger.rs`, Story 2.2) to log errors before returning.

The logger module is already implemented and wired into `main.rs`. Import it with:
```rust
use crate::logger;
```

Public logger API (from `src/logger.rs`):
- `logger::error(message: &str)` — logs at ERROR level
- `logger::warn(message: &str)` — logs at WARN level
- `logger::info(message: &str)` — logs at INFO level

Call pattern:
```rust
logger::error(&format!("github_api: PUT failed: {}", e));
```

Do NOT write to stdout or stderr — log-only (NFR10, NFR11).

### Architecture Module Boundary (Critical)

From `architecture.md`:

| `github_api.rs` OWNS | `github_api.rs` NEVER does |
|---|---|
| All HTTP to GitHub, retry logic, SHA handling | Parse JSONL files |
| base64 encoding of content for PUT bodies | Read config directly |
| Logging API errors | Make sync decisions (callers decide what to push) |
| 401 error detection | Call `std::process::exit` |

**Anti-patterns to prevent:**
- Do NOT inline HTTP calls in `sync.rs` or any other module — all GitHub HTTP goes through `github_api.rs`
- Do NOT add a `reqwest` or `tokio` dependency — `ureq` is sync HTTP as designed
- Do NOT add a `base64` crate — implement std-only base64
- Do NOT call `std::process::exit(0)` inside `github_api.rs` — callers handle exit (NFR10)
- Do NOT write to stdout/stderr — use `logger::log_error` only

### `#![allow(dead_code)]` Pattern

From Story 2.3 learnings: public API functions not yet called by callers trigger Clippy `dead_code` warnings. Add `#![allow(dead_code)]` at the top of `github_api.rs` to suppress these — the module is infrastructure whose callers land in Story 3.1 (`sync.rs`).

```rust
#![allow(dead_code)]
```

### Worktree / Cargo Isolation

From Story 1.2 and 2.3 learnings: the worktree is nested inside the main repo. `Cargo.toml` at the repo root already has `[workspace]` set. Do NOT add another `[workspace]` section.

Run `cargo build` and `cargo test` from the repo root:
```bash
cargo build
cargo test
cargo clippy --all-targets -- -D warnings
```

### Tests — Unique Temp File Names

From Story 2.3 code review: use unique temp file names in tests (pid + nanos + atomic counter) to prevent race conditions in parallel test runs. Apply same pattern if any test writes files.

### Project Structure Notes

- New file: `src/github_api.rs`
- Modified file: `src/main.rs` (add `mod github_api;`)
- No other files modified

### Previous Story Context

Story 2.3 (`checkpoint.rs`) established:
- `#![allow(dead_code)]` pattern for modules whose callers are in future stories
- `std::env::temp_dir()` + unique name for test temp files (not fixed names — avoids parallel test races)
- `cargo clippy --all-targets -- -D warnings` (not just `-- -D warnings`) to catch all-targets warnings
- Atomic `save` via tmp + rename pattern (not needed in this story, but good precedent)
- `std::process::exit` must never be called inside modules — only by `main.rs` or hook handlers

Story 2.2 (`logger.rs`) established:
- `logger::log_error(message)` — how to log errors from any module
- Log format: `YYYY-MM-DDTHH:MM:SSZ ERROR message`
- Silent to stdout/stderr; writes only to `~/.config/vibestats/vibestats.log`

Story 2.1 (`config.rs`) established:
- Config is loaded once by the caller; `github_api.rs` receives token + repo as constructor args
- `oauth_token` and `vibestats_data_repo` are the relevant config fields

Story 1.2 established:
- `Cargo.toml` has all 5 dependencies — no new deps permitted
- `[workspace]` already present — do not add another

### Git Intelligence

Recent commit patterns confirm:
- All story work in dedicated worktrees
- `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings` are standard verification steps
- PRs must include `Closes #17` in the description
- Clippy must pass with `--all-targets` flag (not just `-- -D warnings`)

### Architecture Constraints Summary

| Constraint | NFR/Source | Impact on This Story |
|---|---|---|
| Exit 0 on all errors | NFR10 | `github_api.rs` returns `Err` — never calls `exit` |
| Silent during session | NFR11 | Log via `logger::log_error` only — no stdout/stderr |
| Rate limit backoff | NFR15 | `with_retry`: 1s → 2s → 4s, max 3 attempts on 429/5xx |
| Single HTTP module | architecture.md | All GitHub HTTP in `github_api.rs` — no inline calls elsewhere |
| No async runtime | architecture.md | Use `ureq` (sync); no `tokio`, no `reqwest` |
| No new crates | Story scope | All required crates already in Cargo.toml |
| snake_case filenames | architecture.md | File: `src/github_api.rs` |
| Co-located unit tests | architecture.md | `#[cfg(test)]` module inside `github_api.rs` |

### References

- GitHub API module spec: [Source: architecture.md#Module Responsibility Boundaries — `github_api.rs`]
- GitHub Contents API error handling table: [Source: architecture.md#API & Communication Patterns — Error handling]
- Retry pattern: [Source: architecture.md#Process Patterns — Retry pattern]
- Single module enforcement: [Source: architecture.md#Process Patterns — GitHub API access]
- HTTP client crate selection: [Source: architecture.md#Starter Template Evaluation — Rust Sync Binary (`ureq`)]
- Hive path format: [Source: docs/schemas.md#1. Machine Day File — Location]
- NFR15 (rate limit backoff): [Source: epics.md#NonFunctional Requirements]
- NFR10 (hook non-interference): [Source: epics.md#NonFunctional Requirements]
- NFR11 (silent sync failure): [Source: epics.md#NonFunctional Requirements]
- Story 1.2 (Cargo.toml + crate versions): [Source: implementation-artifacts/1-2-initialize-rust-binary-project.md]
- Story 2.1 (config module + oauth_token field): [Source: implementation-artifacts/2-1-implement-config-module.md]
- Story 2.2 (logger module + log_error API): [Source: implementation-artifacts/2-2-implement-logger-module.md]
- Story 2.3 (checkpoint module + dead_code pattern + clippy --all-targets): [Source: implementation-artifacts/2-3-implement-checkpoint-module.md]
- GH Issue: #17

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

None — implementation completed without debugging incidents.

### Completion Notes List

- Implemented `src/github_api.rs` with `GithubApi` struct, `GithubApiError` type alias, and all public/private functions.
- `GithubApi::new(token, repo)` stores credentials passed by caller (config not read directly — module boundary enforced).
- `GithubApi::put_file(path, content)` base64-encodes content, calls `get_file_sha` to detect create vs update, then PUTs with or without SHA accordingly.
- `GithubApi::get_file_sha(path)` GETs the Contents API, returns `Ok(Some(sha))` for 200, `Ok(None)` for 404, `Err` for other status codes.
- `with_retry<F,T>(f)` implements 3-attempt exponential backoff (1s then 2s delay) for retriable errors (429, 5xx, Transport).
- `classify(err)` helper extracts retriability flag from `ureq::Error` before boxing, avoiding the `clippy::result_large_err` issue at call sites.
- `#![allow(clippy::result_large_err)]` added because `ureq::Error` is a third-party type at 272 bytes — cannot be reduced.
- `base64_encode` implemented std-only (RFC 4648 standard alphabet) — no `base64` crate added.
- `#![allow(dead_code)]` added per Story 2.3 pattern — callers arrive in Story 3.1.
- No calls to `std::process::exit`, no stdout/stderr writes, no reference to `checkpoint.rs`.
- 58 total tests pass (29 pre-existing + 29 new github_api tests). `cargo clippy --all-targets -- -D warnings` passes with 0 warnings.
- Test approach for retry count: used `std::io::Error` → `ureq::Error::Transport` conversion (public API) to construct retriable errors without requiring network access or ureq test-mode server (which fails in sandbox).

### File List

- src/github_api.rs (new)
- src/main.rs (modified — added `mod github_api;`)

## Change Log

- 2026-04-11: Story implemented by claude-sonnet-4-6. Created `src/github_api.rs` with `GithubApi` struct, `put_file`, `get_file_sha`, `with_retry`, `base64_encode`, and 29 co-located unit tests. Added `mod github_api;` to `src/main.rs`. All 58 tests pass; clippy clean. Status set to review.
