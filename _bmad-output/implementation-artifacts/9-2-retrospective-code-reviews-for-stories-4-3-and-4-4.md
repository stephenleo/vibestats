# Story 9.2: Retrospective code reviews for Stories 4.3 and 4.4

Status: backlog

<!-- GH Issue: #82 | Epic: #80 | PR must include: Closes #82 -->

## Story

As a developer relying on the vibestats binary,
I want Stories 4.3 (`vibestats auth`) and 4.4 (`vibestats uninstall`) to have formal code review records,
So that any security or correctness issues in the auth and uninstall commands are caught before users run them.

## Background

The Epic 4 retrospective flagged that Stories 4.3 and 4.4 closed without formal code review, while 4.1 and 4.2 each had review coverage that caught at least one real issue. The auth command (`vibestats auth`) handles OAuth tokens and calls `gh secret set` — a security-sensitive path. The uninstall command modifies `~/.claude/settings.json` and removes files — an irreversible operation. Both warrant adversarial review.

Source: Epic 4 retrospective, Challenge #1 and Action Item #2.

## Acceptance Criteria

1. **Given** Story 4.3 (`vibestats auth`) has no Review Findings section **When** this story is complete **Then** `4-3-implement-vibestats-auth-command.md` contains a `## Review Findings` section with a Blind Hunter pass, an Edge Case Hunter pass, and an Acceptance Auditor pass (even if all three report "clean review").

2. **Given** Story 4.4 (`vibestats uninstall`) has no Review Findings section **When** this story is complete **Then** `4-4-implement-vibestats-uninstall-command.md` contains a `## Review Findings` section with the same three-pass structure.

3. **Given** the review uncovers any defect (security, correctness, or quality) **When** a finding is rated P0 or P1 **Then** a corresponding fix is applied to the source code before this story is marked done, and the Review Findings section documents the fix.

4. **Given** all findings are rated P2 or lower (style/nice-to-have) **When** this story is complete **Then** findings are documented in the Review Findings section and any P2 fixes are applied at the reviewer's discretion; unaddressed P2s are added to deferred-work.md.

## Tasks / Subtasks

- [ ] Task 1: Read context files before starting review
  - [ ] Read `_bmad-output/implementation-artifacts/4-3-implement-vibestats-auth-command.md`
  - [ ] Read `_bmad-output/implementation-artifacts/4-4-implement-vibestats-uninstall-command.md`
  - [ ] Read current source: `src/commands/auth.rs` and `src/commands/uninstall.rs`
  - [ ] Read relevant module code: `src/config.rs`, `src/checkpoint.rs`

- [ ] Task 2: Invoke `bmad-code-review` on Story 4.3
  - [ ] Use the `bmad-code-review` skill in a fresh context
  - [ ] Target: `src/commands/auth.rs` plus the story file for full context
  - [ ] Three review layers: Blind Hunter (security), Edge Case Hunter (edge cases/boundaries), Acceptance Auditor (ACs)
  - [ ] Record findings with priority ratings (P0–P2) in the Review Findings section of the story file

- [ ] Task 3: Invoke `bmad-code-review` on Story 4.4
  - [ ] Use the `bmad-code-review` skill in a fresh context
  - [ ] Target: `src/commands/uninstall.rs` plus the story file for full context
  - [ ] Special attention: the `remove_vibestats_hooks` JSON surgery on `~/.claude/settings.json` — verify it cannot corrupt the file on partial write
  - [ ] Record findings in the Review Findings section of the story file

- [ ] Task 4: Apply any P0/P1 fixes
  - [ ] For each P0/P1 finding, apply the fix to the source file
  - [ ] Run `cargo test` and `cargo clippy --all-targets -- -D warnings` after fixes
  - [ ] Document each fix in the Review Findings section

- [ ] Task 5: Add deferred-work entries for any unaddressed P2+ findings

## Dev Notes

**Security focus areas for Story 4.3 (auth):**
- `gh auth token` output handling — is the token ever written to a log or printed to stdout in any path?
- `gh secret set` failure path — if this fails, does the function return an appropriate error without leaking the token value?
- The `config.toml` write — does it use `std::fs::set_permissions` with 0o600 (per NFR6), or rely on the default umask?
- `Config::save()` call — is it atomic (temp-file-then-rename) or in-place write?

**Security focus areas for Story 4.4 (uninstall):**
- `remove_vibestats_hooks` — on a malformed `~/.claude/settings.json`, does it fail gracefully or corrupt the file?
- File deletion paths — are there any race conditions where a file is deleted twice or a wrong path is constructed?
- Hook group cleanup logic — after removing vibestats hooks from a group, does it correctly handle an empty `hooks[]` array vs. removing the whole group?

**Note:** This story does NOT require implementing new features. The goal is review + fix cycle only.

## Review Criteria

- Both story files have a populated `## Review Findings` section
- No P0 or P1 findings remain unaddressed
- `cargo test` passes after any applied fixes
- `cargo clippy --all-targets -- -D warnings` passes after any applied fixes
