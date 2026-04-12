# Story 9.1: Artifact Hygiene — Fix stale story statuses and missing records

Status: ready-for-dev

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
- **dependency-graph.md not updated** to reflect all stories complete (Story 6.4 shows `backlog` / no PR despite being done)

## Acceptance Criteria

1. **Given** any story marked `done` in sprint-status.yaml **When** its story file is read **Then** `Status: done` appears in the header — no story shows `Status: review` or `Status: ready-for-dev` while sprint-status marks it done.

2. **Given** Story 5.2 has no implementation artifact file **When** this story is complete **Then** `_bmad-output/implementation-artifacts/5-2-implement-generate-svg-py.md` exists with at minimum: Status, a Dev Agent Record section with best-effort completion notes (recoverable from git history / PR #65), and the list of files created.

3. **Given** Stories 7.4 and 8.2 have empty Dev Agent Records **When** this story is complete **Then** both story files have filled Completion Notes (recovered from git history), File List, and a Change Log entry.

4. **Given** the dependency-graph.md still shows Story 6.4 as backlog and no PR **When** this story is complete **Then** dependency-graph.md shows Story 6.4 as done with its correct PR number, and all Epic 1–8 stories are shown as complete.

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
  - [ ] `7-4-build-landing-page.md` — change `Status: ready-for-dev` → `Status: done`

- [ ] Task 2: Create missing Story 5.2 artifact file
  - [ ] Run `git log --all --oneline -- action/generate_svg.py` to identify the commit and PR
  - [ ] Run `gh pr view 65` to recover PR description, file list, and merge details
  - [ ] Run `git show <commit> -- action/generate_svg.py` to recover implementation context
  - [ ] Create `_bmad-output/implementation-artifacts/5-2-implement-generate-svg-py.md` with:
    - `Status: done` in the header
    - Comment: `<!-- GH Issue: #27 | Epic: #5 | PR: #65 -->`
    - Story user story statement (recover from epics.md Story 5.2 section)
    - Dev Agent Record with best-effort Completion Notes, File List (minimum: `action/generate_svg.py`, confirm others via git), and Change Log entry

- [ ] Task 3: Retroactively complete Dev Agent Record for Story 7.4
  - [ ] Run `git log --all --oneline -- site/src/pages/index.astro` to find the landing page commit(s)
  - [ ] Run `gh pr view 70` to recover PR description and merged file list
  - [ ] Update `7-4-build-landing-page.md`:
    - Set `Status: done`
    - Fill `### Completion Notes List` from git/PR context
    - Fill `### File List` from PR diff
    - Add `## Change Log` entry dated per PR merge date (2026-04-12T01:15:26Z)

- [ ] Task 4: Retroactively complete Dev Agent Record for Story 8.2
  - [ ] Run `git log --all --oneline -- .github/workflows/deploy-site.yml` to find the commit (PR #74)
  - [ ] Run `gh pr view 74` to recover PR description and file list
  - [ ] Update `8-2-implement-cloudflare-pages-deploy-workflow.md`:
    - Set `Status: done`
    - Fill `### Completion Notes List` from git/PR context
    - Fill `### File List` from PR diff (primary: `.github/workflows/deploy-site.yml`)
    - Add `## Change Log` entry dated per PR merge date (2026-04-12T07:16:27Z)

- [ ] Task 5: Update dependency-graph.md
  - [ ] Read `_bmad-output/implementation-artifacts/dependency-graph.md`
  - [ ] Confirm Story 6.4 PR number: run `gh pr list --state merged --search "6.4"` or check `_bmad-output/implementation-artifacts/epic-6-retro-2026-04-12.md`
  - [ ] Update Story 6.4 row: Sprint Status `done`, PR to confirmed number, PR Status `merged`, Ready to Work `✅ Yes (done)`
  - [ ] Update Notes section: remove "Story 6.4 now unblocked" and "Current bottleneck" notes, add Epic 6 completion note and Epic 9 start note

## Dev Notes

### Overview

All changes in this story are **documentation edits only** — no source code, no tests, no configuration changes. The dev agent edits `.md` files in `_bmad-output/implementation-artifacts/` only.

### File Locations

All target files are in `_bmad-output/implementation-artifacts/`:

```
_bmad-output/implementation-artifacts/
├── 1-3-initialize-astro-site-project.md              ← fix Status: review → done
├── 1-4-define-and-document-all-json-and-toml-schemas.md  ← fix Status: review → done
├── 2-4-implement-jsonl-parser.md                     ← fix Status: review → done
├── 3-1-implement-core-sync-orchestration.md          ← fix Status: review → done
├── 3-2-implement-stop-hook-integration.md            ← fix Status: review → done
├── 3-3-implement-sessionstart-hook-integration.md    ← fix Status: review → done
├── 4-3-implement-vibestats-auth-command.md           ← fix Status: review → done
├── 4-4-implement-vibestats-uninstall-command.md      ← fix Status: review → done
├── 5-5-implement-aggregate-yml-user-vibestats-data-workflow-template.md  ← fix Status: review → done
├── 7-4-build-landing-page.md                         ← fix Status: ready-for-dev → done + fill Dev Agent Record
├── 8-2-implement-cloudflare-pages-deploy-workflow.md ← fix Status: review → done + fill Dev Agent Record
├── 5-2-implement-generate-svg-py.md                  ← CREATE (does not exist yet)
└── dependency-graph.md                               ← update Story 6.4 row + Notes section
```

### Story File Structure Reference

Every story file follows this structure (use `8-3-configure-github-actions-marketplace-publication.md` as the canonical model):

```markdown
# Story X.Y: <Title>

Status: done

<!-- GH Issue: #N | Epic: #N | PR must include: Closes #N -->

## Story
As a ...
I want ...
So that ...

## Acceptance Criteria
...

## Tasks / Subtasks
...

## Dev Notes
...

## Dev Agent Record

### Agent Model Used
<model name>

### Debug Log References
<none or references>

### Completion Notes List
- Task 1: <what was done>
- Task 2: <what was done>
...

### File List
- path/to/file1
- path/to/file2

## Change Log
- YYYY-MM-DD: <description>
```

### Recovering Git History — Commands to Run

**Do NOT fabricate implementation details.** Use these exact commands:

```bash
# Find commits for Story 5.2 file
git log --all --oneline -- action/generate_svg.py

# Find commits for Story 7.4 file
git log --all --oneline -- site/src/pages/index.astro

# Find commits for Story 8.2 file
git log --all --oneline -- .github/workflows/deploy-site.yml

# View the full diff for a commit
git show <commit-hash>

# Get PR details from GitHub
gh pr view 65   # Story 5.2 — generate_svg.py
gh pr view 70   # Story 7.4 — landing page
gh pr view 74   # Story 8.2 — deploy-site.yml

# Find Story 6.4 PR number
gh pr list --state merged --limit 20
```

If a detail is unrecoverable, write: `"recovered from git history — full details not available"` rather than fabricating.

### Story 5.2 Recovery Context

Known facts from dependency-graph.md and sprint-status.yaml:
- PR #65 merged `2026-04-11T14:32:23Z`
- GH Issue: #27, Epic Issue: #5
- Files expected: `action/generate_svg.py` plus any test files (confirm with git)
- Purpose: Python script that reads aggregated JSONL data and produces the SVG heatmap

### Story 7.4 Recovery Context

Known facts from dependency-graph.md:
- PR #70 merged `2026-04-12T01:15:26Z`
- GH Issue: #38, Epic Issue: #7
- Primary file: `site/src/pages/index.astro`

### Story 8.2 Recovery Context

Known facts from dependency-graph.md:
- PR #74 merged `2026-04-12T07:16:27Z`
- GH Issue: #40, Epic Issue: #8
- Primary file: `.github/workflows/deploy-site.yml`

### Story 6.4 PR Recovery Context

- Check `_bmad-output/implementation-artifacts/epic-6-retro-2026-04-12.md` for PR number
- Sprint-status.yaml confirms status is `done`
- Run `gh pr list --state merged --limit 20` to identify the PR

### Anti-Patterns to Prevent

- Do NOT fabricate completion notes, file lists, or commit hashes — use git/gh commands only
- Do NOT change any non-`Status:` content in the 11 stories in Task 1 — change only the `Status:` line (first 5 lines of file)
- Do NOT touch `sprint-status.yaml` — all statuses there are already correct; only the `.md` files and `dependency-graph.md` need fixing
- Do NOT modify any source code files (`action/`, `src/`, `site/`, `.github/workflows/`) — documentation only
- Do NOT add Dev Agent Records to the 10 simple status-fix stories (Tasks 1) if they already have a section — only fix the `Status:` line for those
- For 7.4 and 8.2: the Dev Agent Record sections already exist but are empty — fill them, do not create duplicate sections

### Verification Commands

Run these after completing all tasks to confirm AC satisfaction:

```bash
# AC #1: Should return no output for done stories
grep -rn "Status: review" _bmad-output/implementation-artifacts/*.md
grep -rn "Status: ready-for-dev" _bmad-output/implementation-artifacts/*.md

# AC #2: Must exist
ls _bmad-output/implementation-artifacts/5-2-implement-generate-svg-py.md

# AC #3: Dev Agent Records must be non-empty
grep -A 5 "Completion Notes List" _bmad-output/implementation-artifacts/7-4-build-landing-page.md
grep -A 5 "Completion Notes List" _bmad-output/implementation-artifacts/8-2-implement-cloudflare-pages-deploy-workflow.md

# AC #4: Story 6.4 must show done in dependency-graph
grep "6\.4" _bmad-output/implementation-artifacts/dependency-graph.md
```

### References

- Epic 9 definition: `_bmad-output/planning-artifacts/epic-9.md`
- GH Issue: #81 (Story 9.1), Epic Issue: #80
- sprint-status.yaml: `_bmad-output/implementation-artifacts/sprint-status.yaml`
- dependency-graph.md: `_bmad-output/implementation-artifacts/dependency-graph.md`
- Story 5.2 PR: #65 (merged 2026-04-11T14:32:23Z)
- Story 7.4 PR: #70 (merged 2026-04-12T01:15:26Z)
- Story 8.2 PR: #74 (merged 2026-04-12T07:16:27Z)
- Canonical complete story example: `_bmad-output/implementation-artifacts/8-3-configure-github-actions-marketplace-publication.md`
- Epic 6 retro (Story 6.4 PR number): `_bmad-output/implementation-artifacts/epic-6-retro-2026-04-12.md`

## Dev Agent Record

### Agent Model Used

_to be filled by dev agent_

### Debug Log References

_none_

### Completion Notes List

_to be filled by dev agent_

### File List

_to be filled by dev agent_

## Change Log

- 2026-04-12: Story created — comprehensive context for artifact hygiene fixes across Epic 1–8 story files and dependency-graph.md.
