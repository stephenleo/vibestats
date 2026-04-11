# Story Dependency Graph
_Last updated: 2026-04-11T21:00:00Z_

## Stories

| Story | Epic | Title | Sprint Status | Issue | PR | PR Status | Dependencies | Ready to Work |
|-------|------|-------|--------------|-------|----|-----------|--------------|---------------|
| 1.1 | 1 | Initialize monorepo directory structure | done | #9 | #43 | merged | none | ✅ Yes (done) |
| 1.2 | 1 | Initialize Rust binary project | done | #10 | #44 | merged | 1.1 | ✅ Yes (done) |
| 1.3 | 1 | Initialize Astro site project | done | #11 | #46 | merged | 1.1 | ✅ Yes (done) |
| 1.4 | 1 | Define and document all JSON and TOML schemas | done | #12 | #45 | merged | 1.1 | ✅ Yes (done) |
| 2.1 | 2 | Implement config module | done | #13 | #48 | merged | epic 1 complete | ✅ Yes (done) |
| 2.2 | 2 | Implement logger module | done | #14 | #49 | merged | epic 1 complete | ✅ Yes (done) |
| 2.3 | 2 | Implement checkpoint module | done | #15 | #50 | merged | epic 1 complete | ✅ Yes (done) |
| 2.4 | 2 | Implement JSONL parser | done | #16 | #53 | closed (direct) | epic 1 complete | ✅ Yes (done) |
| 2.5 | 2 | Implement GitHub API module | done | #17 | #51 | merged | epic 1 complete | ✅ Yes (done) |
| 3.1 | 3 | Implement core sync orchestration | backlog | #18 | — | — | epic 2 complete | ✅ Yes |
| 3.2 | 3 | Implement Stop hook integration | backlog | #19 | — | — | 3.1 | ❌ No (3.1 not merged) |
| 3.3 | 3 | Implement SessionStart hook integration | backlog | #20 | — | — | 3.1 | ❌ No (3.1 not merged) |
| 3.4 | 3 | Implement vibestats sync and sync --backfill commands | backlog | #21 | — | — | 3.1 | ❌ No (3.1 not merged) |
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
- **Epic 2 complete** — all five stories done. PRs #48 (2.1), #49 (2.2), #50 (2.3) merged via GitHub. Stories 2.4 and 2.5 implemented via direct commits to main (deb76aa, 23b8443); PRs #53 and #51 closed accordingly. Worktrees and remote branches for 2.4 and 2.5 cleaned up.
- **Story 3.1 is now "Ready to Work"** — epic 2 complete unblocks epic 3. 3.1 is the serial gatekeeper: 3.2, 3.3, 3.4 all depend on it.
- **Stories 5.1–5.5 are "Ready to Work"** — Epic 5 can proceed in parallel with Epic 3.
- **Story 7.1 is "Ready to Work"** — Epic 7 can begin in parallel (7.2–7.4 depend on 7.1).
- **Parallelization opportunities:** 3.1, 5.1–5.5 (parallel with each other and with 3.1), and 7.1 can all be worked concurrently. Epics 3, 5, and 7 can proceed in parallel.
- **Current bottleneck:** Epic 3 story 3.1 is the serial gatekeeper within epic 3 — 3.2, 3.3, and 3.4 cannot start until 3.1 is merged. Epic 4 requires both epics 2 and 3 complete.
- **GitHub auth:** `gh auth status` reports keyring token invalid. PR/issue status verified via GitHub MCP plugin. Run `gh auth login` to re-authenticate the CLI.
- **`bad` label:** Exists in repo. All story issues (#9–#41) and epic issues (#1–#8) already created.
