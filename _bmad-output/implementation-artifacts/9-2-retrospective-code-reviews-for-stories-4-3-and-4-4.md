# Story 9.2: Retrospective code reviews for Stories 4.3 and 4.4

Status: ready-for-dev

<!-- GH Issue: #82 | Epic: #80 | PR must include: Closes #82 -->

## Story

As a developer relying on the vibestats binary,
I want Stories 4.3 (`vibestats auth`) and 4.4 (`vibestats uninstall`) to have formal code review records,
So that any security or correctness issues in the auth and uninstall commands are caught before users run them.

## Background

The Epic 4 retrospective flagged that Stories 4.3 and 4.4 closed without formal code review, while 4.1 and 4.2 each had review coverage that caught at least one real issue. The auth command (`vibestats auth`) handles OAuth tokens and calls `gh secret set` — a security-sensitive path. The uninstall command modifies `~/.claude/settings.json` and removes files — an irreversible operation. Both warrant adversarial review.

Source: Epic 4 retrospective, Challenge #1 and Action Item #2.

**CRITICAL: This story does NOT implement new features. The goal is code review + fix cycle only.**

## Acceptance Criteria

1. **Given** Story 4.3 (`vibestats auth`) has no Review Findings section **When** this story is complete **Then** `4-3-implement-vibestats-auth-command.md` contains a `## Review Findings` section with a Blind Hunter pass, an Edge Case Hunter pass, and an Acceptance Auditor pass (even if all three report "clean review").

2. **Given** Story 4.4 (`vibestats uninstall`) has no Review Findings section **When** this story is complete **Then** `4-4-implement-vibestats-uninstall-command.md` contains a `## Review Findings` section with the same three-pass structure.

3. **Given** the review uncovers any defect (security, correctness, or quality) **When** a finding is rated P0 or P1 **Then** a corresponding fix is applied to the source code before this story is marked done, and the Review Findings section documents the fix.

4. **Given** all findings are rated P2 or lower (style/nice-to-have) **When** this story is complete **Then** findings are documented in the Review Findings section and any P2 fixes are applied at the reviewer's discretion; unaddressed P2s are added to `_bmad-output/implementation-artifacts/deferred-work.md`.

5. **Given** any source code fix is applied **When** the fix is complete **Then** `cargo test` passes with 0 failures AND `cargo clippy --all-targets -- -D warnings` passes with 0 warnings.

6. **Given** both reviews are complete **When** this story is marked done **Then** the status header in `4-3-implement-vibestats-auth-command.md` is `Status: done` AND the status header in `4-4-implement-vibestats-uninstall-command.md` is `Status: done`.

## Tasks / Subtasks

- [ ] Task 1: Read context files before starting review
  - [ ] Read `_bmad-output/implementation-artifacts/4-3-implement-vibestats-auth-command.md` (full story + dev notes)
  - [ ] Read `_bmad-output/implementation-artifacts/4-4-implement-vibestats-uninstall-command.md` (full story + dev notes)
  - [ ] Read current source: `src/commands/auth.rs`
  - [ ] Read current source: `src/commands/uninstall.rs`
  - [ ] Read supporting modules: `src/config.rs`, `src/checkpoint.rs`

- [ ] Task 2: Invoke `bmad-code-review` on Story 4.3 (`vibestats auth`)
  - [ ] Use the `bmad-code-review` skill (`/bmad-code-review`) in a fresh context
  - [ ] Primary target file: `src/commands/auth.rs`
  - [ ] Context files to provide: `4-3-implement-vibestats-auth-command.md` + `src/config.rs` + `src/checkpoint.rs`
  - [ ] Three review layers: Blind Hunter (security-first), Edge Case Hunter, Acceptance Auditor
  - [ ] Record ALL findings with priority ratings (P0–P2) in a new `## Review Findings` section appended to `4-3-implement-vibestats-auth-command.md`

- [ ] Task 3: Invoke `bmad-code-review` on Story 4.4 (`vibestats uninstall`)
  - [ ] Use the `bmad-code-review` skill in a fresh context
  - [ ] Primary target file: `src/commands/uninstall.rs`
  - [ ] Context files to provide: `4-4-implement-vibestats-uninstall-command.md` + `src/config.rs`
  - [ ] Special attention: `remove_vibestats_hooks` JSON surgery — verify it cannot corrupt the file on partial write or OS crash mid-write
  - [ ] Record ALL findings in a new `## Review Findings` section appended to `4-4-implement-vibestats-uninstall-command.md`

- [ ] Task 4: Apply any P0/P1 fixes
  - [ ] For each P0/P1 finding in either review, apply the fix to the source file
  - [ ] Run `cargo test` from repo root after ALL fixes — must pass with 0 failures
  - [ ] Run `cargo clippy --all-targets -- -D warnings` from repo root — must produce 0 warnings
  - [ ] Document each fix applied in the corresponding Review Findings section

- [ ] Task 5: Handle P2 findings
  - [ ] Apply P2 fixes at reviewer's discretion
  - [ ] For any unaddressed P2 findings, add entries to `_bmad-output/implementation-artifacts/deferred-work.md` (create if not present)

- [ ] Task 6: Update story file status headers (AC #6)
  - [ ] Change `Status: review` to `Status: done` in `4-3-implement-vibestats-auth-command.md`
  - [ ] Change `Status: review` to `Status: done` in `4-4-implement-vibestats-uninstall-command.md`

## Dev Notes

### Current Implementation State (as of 2026-04-11)

Both story files currently show `Status: review` which is inconsistent with `sprint-status.yaml` marking them `done`. Correcting both status headers to `done` is a required subtask of this story (AC #6).

### Actual Source Code vs. Story Spec — KEY DIFFERENCES the Reviewer Must Know

The implementation in `src/commands/auth.rs` **diverges from the original story spec** in one security-motivated way the reviewer must understand — not flag as a bug:

**auth.rs — Token passed via stdin pipe, not `--body` flag:**
The story spec showed `gh secret set --body <token>`, but the actual implementation uses `--body-file -` with stdin piping. This was a deliberate security improvement: it prevents the token from appearing in `ps` output or `/proc/<pid>/cmdline`.

```rust
// ACTUAL implementation — token passed via stdin pipe (NOT --body <token>)
let mut child = std::process::Command::new("gh")
    .args(["secret", "set", "VIBESTATS_TOKEN", "--repo", &config.vibestats_data_repo, "--body-file", "-"])
    .stdin(std::process::Stdio::piped())
    .stdout(std::process::Stdio::piped())
    .stderr(std::process::Stdio::piped())
    .spawn()?;
child.stdin.as_mut().expect("stdin is piped").write_all(config.oauth_token.as_bytes())?;
drop(child.stdin.take()); // MUST close stdin before wait_with_output() or it deadlocks
child.wait_with_output()
```

**uninstall.rs — Binary deletion targets `~/.local/bin/vibestats`, not `current_exe()`:**
The story spec showed `std::env::current_exe()`. The actual implementation uses a `binary_path()` helper that always targets `~/.local/bin/vibestats`. This prevents a developer running `cargo run -- uninstall` from deleting their dev build.

```rust
// ACTUAL implementation — targets installer path, not running process path
fn binary_path() -> Option<std::path::PathBuf> {
    std::env::var("HOME").ok().map(|h| {
        std::path::PathBuf::from(h).join(".local").join("bin").join("vibestats")
    })
}
```

### Security Focus Areas for Story 4.3 (`auth`) — Blind Hunter Must Check

| # | Check | Expected |
|---|-------|----------|
| S1 | Is `new_token` ever passed as a CLI `--body` argument? | No — must use stdin pipe only |
| S2 | Does any `println!` or error path print the raw token value? | No — token must never appear in stdout |
| S3 | Does `Config::save()` enforce `0o600` at creation AND on existing files? | Yes — `write_file_mode_600` + `set_permissions_600` |
| S4 | Is `Config::save()` atomic (temp-file + rename) or in-place? | In-place truncate — note if P2 concern |
| S5 | Is `drop(child.stdin.take())` present BEFORE `wait_with_output()`? | Yes — if missing this is a P0 deadlock bug |
| S6 | Does `config.oauth_token` hold the raw token after write? | Yes (Rust does not zero memory) — acceptable, note if concern |

### Edge Case Focus Areas for Story 4.3 (`auth`) — Edge Case Hunter Must Check

| # | Check | Expected |
|---|-------|----------|
| E1 | `gh auth token` returns whitespace-only string | `trim()` before `is_empty()` — verify ordering is correct |
| E2 | `Config::load()` when `config.toml` does not exist | Returns actionable error message |
| E3 | `checkpoint_path()` returns `None` (HOME not set) | Checkpoint clear silently skipped — verify this is acceptable |
| E4 | `checkpoint.toml` does not exist at path | `Checkpoint::load` returns default — `clear_auth_error()` on default must be safe/idempotent |
| E5 | `gh` binary not in PATH | Error path prints actionable remediation hint |

### Security Focus Areas for Story 4.4 (`uninstall`) — Blind Hunter Must Check

| # | Check | Expected |
|---|-------|----------|
| S1 | `is_vibestats_hook` matching: does `"vibestats\tsync"` (tab) match? | Should NOT match — `starts_with("vibestats ")` uses space, not tab |
| S2 | Hook type scope: does `PreToolUse` vibestats command get removed? | No — scope is Stop/SessionStart only (by design) |
| S3 | `std::fs::write` atomicity: if process dies mid-write, is settings.json corrupted? | Yes — non-atomic write is a P1/P2 concern to document |
| S4 | If `~/.local/bin/vibestats` is a symlink, does `remove_file` remove the symlink or the target? | Removes symlink (not target) — verify if this is intended |

### Edge Case Focus Areas for Story 4.4 (`uninstall`) — Edge Case Hunter Must Check

| # | Check | Expected |
|---|-------|----------|
| E1 | Malformed `settings.json` | `serde_json::from_str` error — skips with actionable message |
| E2 | `settings.json` exists but no `"hooks"` key | `remove_vibestats_hooks` returns `false`, no-op |
| E3 | Group with no inner `"hooks"` key | `unwrap_or(true)` preserves unknown-format groups |
| E4 | Binary not found at `~/.local/bin/vibestats` | `ErrorKind::NotFound` handled with "already removed?" message |
| E5 | HOME not set | Both `settings_path()` and `binary_path()` return `None` — both steps skip with message |

### Acceptance Auditor Check Matrix

**Story 4.3 ACs to verify against `src/commands/auth.rs`:**

| AC | Requirement | Verify |
|----|-------------|--------|
| AC #1 | `gh auth token` → token written to `config.toml` with `600` perms (FR36, NFR6) | Call to `Config::save()` which uses `write_file_mode_600` |
| AC #2 | `VIBESTATS_TOKEN` Actions secret updated via `gh secret set` (FR36) | `gh secret set VIBESTATS_TOKEN --repo ... --body-file -` |
| AC #3 | After auth, `checkpoint.toml` `auth_error` cleared (FR40) | `Checkpoint::clear_auth_error()` + `save()` |
| NFR10 | `std::process::exit` never called | grep `auth.rs` for `process::exit` |

**Story 4.4 ACs to verify against `src/commands/uninstall.rs`:**

| AC | Requirement | Verify |
|----|-------------|--------|
| AC #1 | `Stop` and `SessionStart` hooks removed from `~/.claude/settings.json` AND binary deleted (FR37) | `remove_vibestats_hooks` + `binary_path()` + `remove_file` |
| AC #2 | Post-uninstall manual cleanup instructions printed (FR37) | Always-printed block at end of `run()` |
| AC #3 | Non-vibestats hooks preserved | `is_vibestats_hook` filter + unit tests |
| NFR10 | `std::process::exit` never called | grep `uninstall.rs` for `process::exit` |

### `bmad-code-review` Skill Usage

The `bmad-code-review` skill is installed at `.claude/skills/bmad-code-review/`. Invoke with `/bmad-code-review` in a fresh Claude Code session. Provide the source file as the target and the story file + supporting modules as context.

### Review Findings Section Format to Append

Append the following template to the end of each story file:

```markdown
## Review Findings

**Reviewer:** [Claude model] | **Date:** [date] | **Story:** [4.3 or 4.4]

### Blind Hunter Pass

**Focus:** Security vulnerabilities, token leakage, permission issues, injection risks

| # | Finding | Severity | Status |
|---|---------|----------|--------|
| BH-1 | [finding description] | P0/P1/P2 | Fixed / Deferred / Clean |

### Edge Case Hunter Pass

**Focus:** Boundary conditions, missing file states, race conditions, malformed inputs

| # | Finding | Severity | Status |
|---|---------|----------|--------|
| EH-1 | [finding description] | P0/P1/P2 | Fixed / Deferred / Clean |

### Acceptance Auditor Pass

**Focus:** All ACs verified against actual implementation

| AC | Verified | Notes |
|----|----------|-------|
| AC #1 | Yes/No | [notes] |
| AC #2 | Yes/No | [notes] |
| AC #3 | Yes/No | [notes] |
| NFR10 | Yes/No | [notes] |

### Fixes Applied

[List any P0/P1 fixes: what was changed, why, and which file]

### Summary

[Overall assessment: any critical issues, code quality level, recommendation]
```

### File Locations

| File | Role |
|------|------|
| `src/commands/auth.rs` | Story 4.3 target for review |
| `src/commands/uninstall.rs` | Story 4.4 target for review |
| `src/config.rs` | Supporting — `Config::load()`, `Config::save()`, `write_file_mode_600` |
| `src/checkpoint.rs` | Supporting — `Checkpoint::load()`, `clear_auth_error()`, `save()` |
| `_bmad-output/implementation-artifacts/4-3-implement-vibestats-auth-command.md` | Append `## Review Findings` here |
| `_bmad-output/implementation-artifacts/4-4-implement-vibestats-uninstall-command.md` | Append `## Review Findings` here |
| `_bmad-output/implementation-artifacts/deferred-work.md` | Deferred P2 items (create if missing) |

### Architecture Constraints (Do Not Violate)

- `auth.rs` and `uninstall.rs` NEVER call `std::process::exit` (NFR10)
- Token (`new_token`) must NEVER appear in stdout, not even in error messages
- `Config::save()` is the ONLY authorized path for writing `config.toml`
- No new crates may be added (`Cargo.toml` is frozen for this story)
- All code must remain synchronous — no `async fn`, no `tokio`
- `cargo test` and `cargo clippy --all-targets -- -D warnings` must pass from repo root after any fix

### Verification Commands

Run from REPO ROOT (not from inside worktree):

```bash
cargo test                                       # Must pass: ≥135 tests, 0 failures
cargo clippy --all-targets -- -D warnings        # Must produce: 0 warnings
```

## Review Criteria

- Both `4-3-implement-vibestats-auth-command.md` and `4-4-implement-vibestats-uninstall-command.md` have a populated `## Review Findings` section (all three passes completed)
- No P0 or P1 findings remain unaddressed (fixed + documented, or downgraded with written rationale)
- `cargo test` passes with 0 failures after any fixes
- `cargo clippy --all-targets -- -D warnings` passes with 0 warnings after any fixes
- Both story files show `Status: done` in their header (AC #6)
- Any unaddressed P2 items recorded in `deferred-work.md`

## Dev Agent Record

### Agent Model Used

<!-- To be filled in by dev agent -->

### Debug Log References

None.

### Completion Notes List

<!-- To be filled in by dev agent -->

### File List

<!-- To be filled in by dev agent -->

## Change Log

- 2026-04-12: Story created — retrospective code reviews for Stories 4.3 and 4.4
