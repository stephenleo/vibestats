# Story Dependency Graph
_Last updated: 2026-04-11T18:00:00Z_

## Stories

| Story | Epic | Title | Sprint Status | Issue | PR | PR Status | Dependencies | Ready to Work |
|-------|------|-------|--------------|-------|----|-----------|--------------|---------------|
| 1.1 | 1 | Initialize monorepo directory structure | done | #9 | #43 | merged | none | ✅ Yes (done) |
| 1.2 | 1 | Initialize Rust binary project | done | #10 | #44 | merged | 1.1 | ✅ Yes (done) |
| 1.3 | 1 | Initialize Astro site project | done | #11 | #46 | merged | 1.1 | ✅ Yes (done) |
| 1.4 | 1 | Define and document all JSON and TOML schemas | done | #12 | #45 | merged | 1.1 | ✅ Yes (done) |
| 2.1 | 2 | Implement config module | backlog | #13 | — | — | epic 1 complete | ✅ Yes |
| 2.2 | 2 | Implement logger module | backlog | #14 | — | — | epic 1 complete | ✅ Yes |
| 2.3 | 2 | Implement checkpoint module | backlog | #15 | — | — | epic 1 complete | ✅ Yes |
| 2.4 | 2 | Implement JSONL parser | backlog | #16 | — | — | epic 1 complete | ✅ Yes |
| 2.5 | 2 | Implement GitHub API module | backlog | #17 | — | — | epic 1 complete | ✅ Yes |
| 3.1 | 3 | Implement core sync orchestration | backlog | #18 | — | — | epic 2 complete | ❌ No (epic 2 not complete) |
| 3.2 | 3 | Implement Stop hook integration | backlog | #19 | — | — | 3.1 | ❌ No (epic 2 not complete, 3.1 not merged) |
| 3.3 | 3 | Implement SessionStart hook integration | backlog | #20 | — | — | 3.1 | ❌ No (epic 2 not complete, 3.1 not merged) |
| 3.4 | 3 | Implement vibestats sync and sync --backfill commands | backlog | #21 | — | — | 3.1 | ❌ No (epic 2 not complete, 3.1 not merged) |
| 4.1 | 4 | Implement vibestats status command | backlog | #22 | — | — | epics 2+3 complete | ❌ No (epics 2+3 not complete) |
| 4.2 | 4 | Implement vibestats machines list and machines remove | backlog | #23 | — | — | epics 2+3 complete | ❌ No (epics 2+3 not complete) |
| 4.3 | 4 | Implement vibestats auth command | backlog | #24 | — | — | epics 2+3 complete | ❌ No (epics 2+3 not complete) |
| 4.4 | 4 | Implement vibestats uninstall command | backlog | #25 | — | — | epics 2+3 complete | ❌ No (epics 2+3 not complete) |
| 5.1 | 5 | Implement aggregate.py | backlog | #26 | — | — | epic 1 complete | ✅ Yes |
| 5.2 | 5 | Implement generate_svg.py | backlog | #27 | — | — | epic 1 complete | ✅ Yes |
| 5.3 | 5 | Implement update_readme.py | backlog | #28 | — | — | epic 1 complete | ✅ Yes |
| 5.4 | 5 | Implement action.yml (composite community GitHub Action) | backlog | #29 | — | — | epic 1 complete | ✅ Yes |
| 5.5 | 5 | Implement aggregate.yml (user vibestats-data workflow template) | backlog | #30 | — | — | epic 1 complete | ✅ Yes |
| 6.1 | 6 | Implement dependency detection and gh authentication | backlog | #31 | — | — | epic 8 complete | ❌ No (epic 8 not complete) |
| 6.2 | 6 | Implement first-install path | backlog | #32 | — | — | 6.1 | ❌ No (epic 8 not complete, 6.1 not merged) |
| 6.3 | 6 | Implement multi-machine install path | backlog | #33 | — | — | 6.1 | ❌ No (epic 8 not complete, 6.1 not merged) |
| 6.4 | 6 | Implement hook configuration, README markers, and backfill trigger | backlog | #34 | — | — | 6.1, 6.2, 6.3 | ❌ No (epic 8 not complete, 6.1/6.2/6.3 not merged) |
| 7.1 | 7 | Build base layouts and shared Astro components | backlog | #35 | — | — | epic 1 complete | ✅ Yes |
| 7.2 | 7 | Build per-user dashboard (u/index.astro + cal-heatmap) | backlog | #36 | — | — | 7.1 | ❌ No (7.1 not merged) |
| 7.3 | 7 | Build documentation pages | backlog | #37 | — | — | 7.1 | ❌ No (7.1 not merged) |
| 7.4 | 7 | Build landing page | backlog | #38 | — | — | 7.1 | ❌ No (7.1 not merged) |
| 8.1 | 8 | Implement Rust binary release CI | backlog | #39 | — | — | epics 1–7 complete | ❌ No (epics 1–7 not complete) |
| 8.2 | 8 | Implement Cloudflare Pages deploy workflow | backlog | #40 | — | — | epics 1–7 complete | ❌ No (epics 1–7 not complete) |
| 8.3 | 8 | Configure GitHub Actions Marketplace publication | backlog | #41 | — | — | epics 1–7 complete | ❌ No (epics 1–7 not complete) |

## Dependency Chains

### Epic-level sequencing (from epics.md)
- **Epic 1** → **Epic 2** → **Epic 3** + **Epic 5** (parallel) → **Epic 4** + **Epic 7** (parallel) → **Epic 8** → **Epic 6**

### Intra-epic story dependencies

**Epic 1:**
- **1.2** depends on: 1.1 (directory structure must exist before initializing Rust project within it)
- **1.3** depends on: 1.1 (directory structure must exist before initializing Astro site within it)
- **1.4** depends on: 1.1 (schema docs placed in repo structure)

**Epic 2:**
- All stories (2.1–2.5) depend on: epic 1 complete (independent of each other, can run in parallel within the epic)

**Epic 3:**
- **3.1** depends on: epic 2 complete
- **3.2** depends on: 3.1 (Stop hook routes through sync orchestration)
- **3.3** depends on: 3.1 (SessionStart hook routes through sync orchestration)
- **3.4** depends on: 3.1 (CLI sync command routes through sync orchestration)

**Epic 4:**
- All stories (4.1–4.4) depend on: epics 2+3 complete (independent of each other within the epic)

**Epic 5:**
- All stories (5.1–5.5) depend on: epic 1 complete (independent of each other within the epic)

**Epic 6:**
- **6.1** depends on: epic 8 complete
- **6.2** depends on: 6.1 (first-install path needs dependency detection scaffolded)
- **6.3** depends on: 6.1 (multi-machine path needs dependency detection scaffolded)
- **6.4** depends on: 6.1, 6.2, 6.3 (hook config and backfill trigger need all install paths complete)

**Epic 7:**
- **7.1** depends on: epic 1 complete
- **7.2** depends on: 7.1 (dashboard builds on base layouts)
- **7.3** depends on: 7.1 (docs pages build on base layouts)
- **7.4** depends on: 7.1 (landing page builds on base layouts)

**Epic 8:**
- All stories (8.1–8.3) depend on: epics 1–7 complete (independent of each other within the epic)

## Notes

- **Epic 1 complete** — all four PRs merged: #43 (1.1), #44 (1.2), #46 (1.3), #45 (1.4). Worktrees and remote branches cleaned up.
- **Stories 2.1–2.5 are now "Ready to Work"** — Epic 2 stories can all be worked in parallel now that Epic 1 is complete.
- **Stories 5.1–5.5 are now "Ready to Work"** — Epic 5 can begin in parallel with Epic 2 work.
- **Story 7.1 is now "Ready to Work"** — Epic 7 can begin (7.1 is the first story; 7.2–7.4 depend on 7.1).
- **Parallelization opportunities:** 2.1–2.5 (parallel), 5.1–5.5 (parallel), and 7.1 can all start now. Epics 2, 5, and 7 can proceed concurrently.
- **Current bottleneck:** Epic 2 must fully complete before Epic 3 can start. Epic 3 story 3.1 is a serial bottleneck within that epic.
- **GitHub auth:** `gh auth status` reports keyring token invalid. PR/issue status verified via GitHub MCP plugin. Run `gh auth login` to re-authenticate the CLI.
- **`bad` label:** All story issues (#9–#41) and epic issues (#1–#8) already exist in the repo. No new issues need to be created.
