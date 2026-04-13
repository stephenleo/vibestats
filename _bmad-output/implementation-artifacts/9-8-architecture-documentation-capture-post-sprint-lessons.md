# Story 9.8: Architecture documentation — Capture post-sprint lessons

Status: review

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

1. **Given** `_bmad-output/planning-artifacts/architecture.md` **When** this story is complete **Then** it contains a "Known Gotchas & Conventions" section (or equivalent heading) covering all six items listed in the Tasks below.

2. **Given** a future developer reads the architecture doc **When** they encounter the `#[serde(default)]` footgun **Then** the documentation gives them enough context to avoid it without reading Story 2.3's debug log.

3. **Given** the documentation additions **When** the architecture doc is reviewed **Then** all additions are factually accurate (cross-checked against the actual source code and story files).

## Tasks / Subtasks

- [x] Task 1: Read current `architecture.md` to find the best insertion point
  - [x] Read `_bmad-output/planning-artifacts/architecture.md`
  - [x] The natural insertion point is at the end of the document, after "Architecture Validation Results" — append a new "Known Gotchas & Conventions" section. Do NOT insert into or alter any existing sections.
  - [x] Confirm no "Known Gotchas" or "Conventions" section already exists before creating one

- [x] Task 2: Document the Cloudflare Pages `_redirects` evaluation-order rule
  - [x] Content: When adding rules to `site/public/_redirects`, pass-through rules for static assets must appear BEFORE any catch-all rewrite rules. Cloudflare Pages evaluates `_redirects` top-to-bottom and stops at the first match. A catch-all like `/:username /u  200` placed first intercepts `/_astro/`, `/favicon.ico`, and `/u` itself.
  - [x] **Additional gotcha:** Rewriting to `/u.html` instead of `/u` triggers Cloudflare's clean-URL redirect (`*.html` → `*`), which sends the browser back to `/:username` — infinite redirect loop. Always rewrite to `/u`, not `/u.html`.
  - [x] Include the correct ordering pattern. The actual file at `site/public/_redirects` is the canonical reference — read it and use it verbatim as the example.
  - [x] Source: Epic 1 retrospective, Challenge #1; `site/public/_redirects`

- [x] Task 3: Document the Rust `#[serde(default)]` vs `Default` footgun
  - [x] Content: `#[serde(default = "some_fn")]` controls what serde uses during deserialization when a field is absent from the JSON/TOML. It does NOT affect `Default::default()`. If you `#[derive(Default)]`, the derived impl uses Rust's zero values (empty string, 0, false) — not `some_fn`. Whenever a struct uses `#[serde(default = "fn")]` AND needs `Default`, write a manual `Default` impl that calls the same functions.
  - [x] Reference implementation is `src/checkpoint.rs` lines 10–33: `fn default_machine_status() -> String { "active".to_string() }` annotated with `#[serde(default = "default_machine_status")]`, plus a manual `impl Default for Checkpoint` that calls `default_machine_status()`. Read this file to confirm before writing the example.
  - [x] Source: Epic 2 retrospective, Challenge #3; `src/checkpoint.rs`

- [x] Task 4: Document the Cargo worktree `[workspace]` isolation pattern
  - [x] Content: When running `cargo` commands from within a git worktree nested inside the main repo (e.g., `.worktrees/story-X-*`), Cargo walks up the directory tree and may find the parent `Cargo.toml`. To prevent this, the project's `Cargo.toml` must include `[workspace]` with no members — this declares an isolated workspace root. The vibestats `Cargo.toml` already has this; do not remove it.
  - [x] Verify: `Cargo.toml` line 1 is `[workspace]` with no members key — confirmed intentional.
  - [x] Source: Epic 1 retrospective, Key Insight #3; `Cargo.toml`

- [x] Task 5: Document the `_gh()` define-if-not-defined pattern for testable shell helpers
  - [x] Content: When writing shell scripts that call external tools (`gh`, `curl`, `brew`, etc.), wrap each external call in a helper function using the define-if-not-defined guard. Test files can then pre-define their own stub before sourcing `install.sh`, making the entire script testable without shell binary mocking.
  - [x] Pattern (verified at `install.sh` line 28):
    ```bash
    if ! declare -f _gh > /dev/null 2>&1; then
      _gh() { gh "$@"; }
    fi
    ```
  - [x] This pattern is used throughout `install.sh` and `tests/installer/` — all external tool wrappers follow this convention. Do not call `gh` directly in `install.sh`; always route through `_gh`.
  - [x] Source: Epic 6 retrospective, Key Insight #1; `install.sh` line 28

- [x] Task 6: Document Python3 stdlib over `jq` for JSON in Bash scripts
  - [x] Content: When shell scripts need JSON manipulation, use Python3 stdlib (`import json, sys, base64`) rather than `jq`. `jq` is not installed by default on macOS or many Linux distributions; Python3 is standard on both.
  - [x] Standard pattern (verified in `install.sh` line 199 and throughout): `python3 -c "import sys, json; print(json.load(sys.stdin)['field'])"` — pipe JSON to stdin, extract field. For base64 decode (GitHub API Content responses): `python3 -c "import sys, base64; print(base64.b64decode(sys.stdin.read().replace('\n','')).decode())"`.
  - [x] Source: Epic 6 retrospective, Key Insight #2; `install.sh` lines 197–199

- [x] Task 7: Document the security negative test pattern
  - [x] Content: Security properties (e.g., "token is never written to disk") require explicit negative test assertions in bats. Implementation notes are not sufficient — add an assertion that actively detects leaks using a sentinel value.
  - [x] Pattern (verified in `tests/installer/test_6_2.bats` test "[P0] VIBESTATS_TOKEN is never written to disk or echoed to stdout"): inject a sentinel token value through the stub, run the install function, then assert `! grep -r "SENTINEL_TOKEN" "${HOME}" 2>/dev/null`. Every security requirement in `install.sh` (NFR7: token never on disk) should have a corresponding negative bats test.
  - [x] Source: Epic 6 retrospective, Key Insight #3; `tests/installer/test_6_2.bats` line 243

## Dev Notes

**This story is documentation-only — no source code changes.**

### Target File

- Single file to edit: `_bmad-output/planning-artifacts/architecture.md`
- Do NOT edit files in `docs/`, `site/`, or anywhere else — only `architecture.md`.
- The architecture.md is ~700 lines. Append the new section at the end.

### Editing Guidelines

- Each gotcha entry: 1–3 short paragraphs + code snippet where applicable. Not essays.
- **Factually verify every code example against source before writing.** The verified patterns in the Tasks section above reflect actual source code — use them.
- Do not duplicate content already in `architecture.md`. The `_redirects` routing solution is already documented in Gap Analysis (Gap 1). The new section adds the evaluation-order rule and the `/u.html` infinite-redirect trap, which are NOT already documented.
- Do not remove or modify any existing content in `architecture.md`.
- Section heading to add: `## Known Gotchas & Conventions`

### Source Code Verification Summary

All six items verified against actual source files:

| Item | Source File | Key Evidence |
|------|------------|--------------|
| `_redirects` order | `site/public/_redirects` | Comment "MUST come before" + static rules first |
| `serde(default)` footgun | `src/checkpoint.rs:10–33` | `default_machine_status()` + manual `impl Default` |
| Cargo `[workspace]` | `Cargo.toml:1` | `[workspace]` with no members key |
| `_gh()` pattern | `install.sh:28` | `if ! declare -f _gh > /dev/null 2>&1` |
| Python3 over jq | `install.sh:197–199` | Comment + `python3 -c "import sys, json"` |
| Security negative test | `tests/installer/test_6_2.bats:243` | Sentinel token + `! grep` assertion |

### Project Structure Notes

- `_bmad-output/planning-artifacts/architecture.md` is a planning artifact, not a source code file. The worktree has the same file at the same relative path.
- Architecture.md has no "Known Gotchas" section yet — the new section is purely additive.
- The `_redirects` evaluation-order content supplements (does not replace) the Gap 1 resolution already in architecture.md.

### References

- [Source: `_bmad-output/planning-artifacts/architecture.md`] — target file, "Implementation Patterns & Consistency Rules" (natural insertion point context)
- [Source: `_bmad-output/planning-artifacts/architecture.md#Gap Analysis`] — Gap 1 documents the dynamic-username solution; new entry adds evaluation-order rule
- [Source: `site/public/_redirects`] — canonical `_redirects` example
- [Source: `src/checkpoint.rs:10–33`] — `serde(default)` reference implementation
- [Source: `Cargo.toml:1`] — workspace isolation
- [Source: `install.sh:28,197–199`] — `_gh()` pattern and python3 convention
- [Source: `tests/installer/test_6_2.bats:243`] — security negative test pattern
- Epic 9 GH Issue: [#88](https://github.com/stephenleo/vibestats/issues/88)
- Epic: [#80](https://github.com/stephenleo/vibestats/issues/80)

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

None — documentation-only story, no source code changes.

### Completion Notes List

- Appended `## Known Gotchas & Conventions` section to `_bmad-output/planning-artifacts/architecture.md` (after the Implementation Handoff section, ~line 703).
- Section contains six entries, each verified against actual source files before writing:
  1. Cloudflare Pages `_redirects` evaluation order + `/u.html` infinite-redirect trap — verified against `site/public/_redirects`
  2. Rust `#[serde(default)]` vs `Default` footgun — verified against `src/checkpoint.rs` lines 10–33
  3. Cargo worktree `[workspace]` isolation — verified `Cargo.toml` line 1
  4. `_gh()` define-if-not-defined pattern — verified `install.sh` line 28
  5. Python3 stdlib over `jq` — verified `install.sh` lines 197–199
  6. Security negative test pattern — verified `tests/installer/test_6_2.bats` line 243
- No existing content in `architecture.md` was modified; addition is purely additive.
- All three Acceptance Criteria satisfied: (1) section exists with all six items, (2) serde footgun is fully explained with code example, (3) all examples cross-checked against source.

### File List

- `_bmad-output/planning-artifacts/architecture.md` (modified — appended "Known Gotchas & Conventions" section)

### Change Log

- 2026-04-13: Appended `## Known Gotchas & Conventions` section to `architecture.md` covering six implementation lessons from Epic 1, 2, and 6 retrospectives (Story 9.8)
