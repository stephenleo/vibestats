# Story 9.8: Architecture documentation — Capture post-sprint lessons

Status: backlog

<!-- GH Issue: #88 | Epic: #80 | PR must include: Closes #88 -->

## Story

As a developer (or future agent) returning to the vibestats codebase,
I want the architecture documentation to capture the non-obvious lessons and footguns discovered during implementation,
So that the same mistakes are not repeated and the codebase conventions are discoverable without reading 8 retrospectives.

## Background

Multiple retrospectives identified patterns and gotchas that should be documented but weren't — they were noted as action items that persisted across epics without being resolved. This story consolidates all documentation debt into one pass.

Sources:
- Epic 1 retro: `_redirects` evaluation-order guidance (Action Item #2)
- Epic 2 retro: `#[serde(default = "fn")]` vs `Default` footgun (Action Item #2)
- Epic 1/2 retros: Cargo worktree `[workspace]` pattern documentation
- Epic 6 retro: `_gh()` define-if-not-defined pattern as project convention (Documentation #1)
- Epic 6 retro: Python3 stdlib for JSON in Bash scripts as convention
- Epic 6 retro: Security requirements need explicit negative test coverage

## Acceptance Criteria

1. **Given** `_bmad-output/planning-artifacts/architecture.md` **When** this story is complete **Then** it contains a "Known Gotchas & Lessons" section (or equivalent heading) covering all six items listed in the Tasks below.

2. **Given** a future developer reads the architecture doc **When** they encounter the `#[serde(default)]` footgun **Then** the documentation gives them enough context to avoid it without reading Story 2.3's debug log.

3. **Given** the documentation additions **When** the architecture doc is reviewed **Then** all additions are factually accurate (cross-checked against the actual source code and story files).

## Tasks / Subtasks

- [ ] Task 1: Read current `architecture.md` to find the best insertion point
  - [ ] Read `_bmad-output/planning-artifacts/architecture.md`
  - [ ] Identify whether a "Conventions" or "Dev Notes" section already exists, or where to add one

- [ ] Task 2: Document the Cloudflare Pages `_redirects` evaluation-order rule
  - [ ] Content: When adding rules to `site/public/_redirects`, pass-through rules for static assets must appear BEFORE any catch-all rewrite rules. Cloudflare Pages evaluates `_redirects` top-to-bottom and stops at the first match. A catch-all like `/:username /u/index.html 200` placed first will intercept `/_astro/`, `/favicon.ico`, and `/u` itself.
  - [ ] Include the correct ordering pattern with an example
  - [ ] Source: Epic 1 retrospective, Challenge #1

- [ ] Task 3: Document the Rust `#[serde(default)]` vs `Default` footgun
  - [ ] Content: `#[serde(default = "some_fn")]` controls what serde uses during deserialization when a field is absent from the JSON/TOML. It does NOT affect `Default::default()`. If you also `#[derive(Default)]`, the derived `Default` impl uses Rust's default (empty string, 0, false) — not `some_fn`. Whenever a struct uses `#[serde(default = "fn")]` AND implements `Default`, write a manual `Default` impl that calls the same functions.
  - [ ] Include a minimal before/after code example
  - [ ] Source: Epic 2 retrospective, Challenge #3; Story 2.3 debug log

- [ ] Task 4: Document the Cargo worktree `[workspace]` isolation pattern
  - [ ] Content: When running `cargo` commands from within a git worktree that is nested inside the main repo, Cargo walks up the directory tree and may find the parent `Cargo.toml`. To prevent this, the nested project's `Cargo.toml` must include `[workspace]` with no members. This tells Cargo to treat it as an isolated workspace root. The vibestats `Cargo.toml` already has this — do not remove it.
  - [ ] Source: Epic 1 retrospective, Key Insight #3; Story 1.2 debug log

- [ ] Task 5: Document the `_gh()` define-if-not-defined pattern for testable shell helpers
  - [ ] Content: When writing shell scripts that call external tools (`gh`, `curl`, `brew`, etc.), wrap each external call in a helper function using the define-if-not-defined guard:
    ```bash
    if ! declare -f _gh > /dev/null 2>&1; then
      _gh() { gh "$@"; }
    fi
    ```
    Test files can then pre-define their own stub before sourcing `install.sh`, making the entire script testable without shell binary mocking. This pattern is used throughout `install.sh` and `tests/installer/`.
  - [ ] Source: Epic 6 retrospective, Key Insight #1; Story 6.1

- [ ] Task 6: Document Python3 stdlib over `jq` for JSON in Bash scripts
  - [ ] Content: When shell scripts need JSON manipulation (parse, append, merge), use Python3 stdlib (`import json, sys, base64`) rather than `jq`. Rationale: `jq` is not installed by default on macOS or many Linux distributions. Python3 is standard on both. The pattern: `python3 -c "import json,sys; ..."`. Used throughout `install.sh` for registry.json and settings.json manipulation.
  - [ ] Source: Epic 6 retrospective, Key Insight #2

- [ ] Task 7: Document the security negative test pattern
  - [ ] Content: Security properties (e.g., "token is never written to disk") require explicit negative test assertions, not just implementation notes. Example from `test_6_2.bats`: `! grep -r "test-secret-token" "${HOME}" 2>/dev/null`. Every security requirement in `install.sh` should have a corresponding negative test.
  - [ ] Source: Epic 6 retrospective, Key Insight #3

## Dev Notes

- All changes in this story are documentation edits only — no source code changes.
- The architecture.md may already contain some conventions. Find the natural extension point; don't create duplicate sections.
- Keep each item concise — 1–3 paragraphs with a code example where helpful. These are quick reference items, not essays.
- Verify any code examples against actual source code (not from memory). Use `grep` to confirm function signatures before writing them in docs.

## Review Criteria

- `architecture.md` contains all six documented items
- Each item is factually accurate (cross-checked against source code)
- Code examples compile or are syntactically valid
- No existing content in `architecture.md` was accidentally removed or overwritten
