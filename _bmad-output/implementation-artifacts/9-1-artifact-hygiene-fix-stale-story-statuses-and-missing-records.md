# Story 9.1: Artifact Hygiene — Fix stale story statuses and missing records

Status: backlog

<!-- GH Issue: #81 | Epic: #80 | PR must include: Closes #81 -->

## Story

As a developer reading the implementation history,
I want every story file's `Status` field to match `sprint-status.yaml` and every story to have a complete Dev Agent Record,
So that the artifact layer is a reliable source of truth and retrospectives don't have to reconcile conflicting states.

## Background

This problem was flagged in the retrospectives for Epics 1, 2, 3, 4, 5, 7, and 8 — it is the single most recurring cross-epic failure. Specific gaps identified:

- **Status field drift (Status: review when sprint-status says done):** Stories 1.3, 1.4, 2.4, 3.1, 3.2, 3.3, 4.3, 4.4, 5.5, 8.2
- **Missing story artifact file entirely:** Story 5.2 (generate_svg.py)
- **Empty Dev Agent Record (no completion notes, file list, or change log):** Stories 7.4, 8.2
- **dependency-graph.md not updated** to reflect all 8 epics complete

## Acceptance Criteria

1. **Given** any story marked `done` in sprint-status.yaml **When** its story file is read **Then** `Status: done` appears in the header — no story shows `Status: review` or `Status: ready-for-dev` while sprint-status marks it done.

2. **Given** Story 5.2 has no implementation artifact file **When** this story is complete **Then** `_bmad-output/implementation-artifacts/5-2-implement-generate-svg-py.md` exists with at minimum: Status, a Dev Agent Record section with best-effort completion notes (recoverable from git history / PR #65), and the list of files created.

3. **Given** Stories 7.4 and 8.2 have empty Dev Agent Records **When** this story is complete **Then** both story files have filled Completion Notes (recovered from git history), File List, and a Change Log entry.

4. **Given** the dependency-graph.md still shows some stories as backlog or in-progress **When** this story is complete **Then** all Epic 1–8 stories and epics are marked as done/complete in dependency-graph.md.

## Tasks / Subtasks

- [ ] Task 1: Fix Status fields — update all stale story files from `review`/`ready-for-dev` to `done`
  - [ ] `1-3-initialize-astro-site-project.md` — change `Status: review` → `Status: done`
  - [ ] `1-4-define-and-document-all-json-and-toml-schemas.md` — change `Status: review` → `Status: done`
  - [ ] `2-4-implement-jsonl-parser.md` — change `Status: review` → `Status: done`
  - [ ] `3-1-implement-core-sync-orchestration.md` — change `Status: review` → `Status: done`
  - [ ] `3-2-implement-stop-hook-integration.md` — change `Status: review` → `Status: done`
  - [ ] `3-3-implement-sessionstart-hook-integration.md` — change `Status: review` → `Status: done`
  - [ ] `4-3-implement-vibestats-auth-command.md` — change `Status: review` → `Status: done`
  - [ ] `4-4-implement-vibestats-uninstall-command.md` — change `Status: review` → `Status: done`
  - [ ] `5-5-implement-aggregate-yml-user-vibestats-data-workflow-template.md` — change `Status: review` → `Status: done`
  - [ ] `8-2-implement-cloudflare-pages-deploy-workflow.md` — change `Status: review` → `Status: done`

- [ ] Task 2: Create missing Story 5.2 artifact file
  - [ ] Use `git log --all --oneline -- action/generate_svg.py` to identify the commit and PR
  - [ ] Use `git show <commit> -- action/generate_svg.py` to recover implementation context
  - [ ] Create `5-2-implement-generate-svg-py.md` with Status: done, best-effort Dev Agent Record, and file list
  - [ ] Minimum file list: `action/generate_svg.py`, `action/tests/test_generate_svg.py` (if present)

- [ ] Task 3: Retroactively complete Dev Agent Record for Story 7.4
  - [ ] Use `git log --all --oneline -- site/src/pages/index.astro` to find the landing page commit(s)
  - [ ] Recover file list from the commit diff: should include `site/src/pages/index.astro` and related assets
  - [ ] Update `7-4-build-landing-page.md`: set `Status: done`, fill Completion Notes, File List, Change Log entry

- [ ] Task 4: Retroactively complete Dev Agent Record for Story 8.2
  - [ ] Use `git log --all --oneline -- .github/workflows/deploy-site.yml` to find the commit (PR #74)
  - [ ] Recover file list: `.github/workflows/deploy-site.yml`
  - [ ] Update `8-2-implement-cloudflare-pages-deploy-workflow.md`: set `Status: done`, fill Completion Notes, File List, Change Log entry

- [ ] Task 5: Update dependency-graph.md
  - [ ] Read `_bmad-output/implementation-artifacts/dependency-graph.md`
  - [ ] Update all Epics 1–8 and their stories to show complete/done status
  - [ ] Verify the "Sprint Status" column matches sprint-status.yaml for every row

## Dev Notes

- All changes in this story are documentation edits only — no source code changes.
- Use `git log --oneline --follow -- <file>` to trace file history when recovering Dev Agent Records.
- For Story 5.2: PR #65 was merged on 2026-04-11 per the Epic 5 retrospective. Use `gh pr view 65` or `git log --all` to find it.
- For Story 8.2: PR #74 was mentioned in the Epic 8 retrospective. Use `gh pr view 74` or `git log --all` to find it.
- Do NOT attempt to reconstruct Dev Agent Records from memory — use actual git history only. If a detail is unrecoverable, mark it as "recovered from git history — details not available" rather than fabricating.

## Review Criteria

- `grep -r "Status: review" _bmad-output/implementation-artifacts/` returns no output (or only stories legitimately still in review)
- `_bmad-output/implementation-artifacts/5-2-implement-generate-svg-py.md` exists
- `7-4-build-landing-page.md` and `8-2-implement-cloudflare-pages-deploy-workflow.md` both have non-empty Dev Agent Records
- dependency-graph.md shows all Epics 1–8 as complete
