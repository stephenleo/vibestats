# Story Dependency Graph
_Last updated: 2026-04-13T06:00:00Z (phase0 run — epic 9 stories added; 9.1–9.6 done, 9.7–9.9 backlog)_

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
| 3.1 | 3 | Implement core sync orchestration | done | #18 | #54 | merged | epic 2 complete | ✅ Yes (done) |
| 3.2 | 3 | Implement Stop hook integration | done | #19 | #56 | merged | 3.1 | ✅ Yes (done) |
| 3.3 | 3 | Implement SessionStart hook integration | done | #20 | #55 | merged | 3.1 | ✅ Yes (done) |
| 3.4 | 3 | Implement vibestats sync and sync --backfill commands | done | #21 | #57 | merged | 3.1 | ✅ Yes (done) |
| 4.1 | 4 | Implement vibestats status command | done | #22 | #58 | merged | epics 2+3 complete | ✅ Yes (done) |
| 4.2 | 4 | Implement vibestats machines list and machines remove | done | #23 | #61 | merged | epics 2+3 complete | ✅ Yes (done) |
| 4.3 | 4 | Implement vibestats auth command | done | #24 | #62 | merged | epics 2+3 complete | ✅ Yes (done) |
| 4.4 | 4 | Implement vibestats uninstall command | done | #25 | #63 | merged | epics 2+3 complete | ✅ Yes (done) |
| 5.1 | 5 | Implement aggregate.py | done | #26 | #64 | merged | epic 1 complete | ✅ Yes (done) |
| 5.2 | 5 | Implement generate_svg.py | done | #27 | #65 | merged | epic 1 complete | ✅ Yes (done) |
| 5.3 | 5 | Implement update_readme.py | done | #28 | #66 | merged | epic 1 complete | ✅ Yes (done) |
| 5.4 | 5 | Implement action.yml (composite community GitHub Action) | done | #29 | #68 | merged | epic 1 complete | ✅ Yes (done) |
| 5.5 | 5 | Implement aggregate.yml (user vibestats-data workflow template) | done | #30 | #67 | merged | epic 1 complete | ✅ Yes (done) |
| 6.1 | 6 | Implement dependency detection and gh authentication | done | #31 | #76 | merged | epic 8 complete | ✅ Yes (done) |
| 6.2 | 6 | Implement first-install path | done | #32 | #77 | merged | 6.1 | ✅ Yes (done) |
| 6.3 | 6 | Implement multi-machine install path | done | #33 | #78 | merged | 6.1 | ✅ Yes (done) |
| 6.4 | 6 | Implement hook configuration, README markers, and backfill trigger | done | #34 | #79 | merged | 6.1, 6.2, 6.3 | ✅ Yes (done) |
| 7.1 | 7 | Build base layouts and shared Astro components | done | #35 | #69 | merged | epic 1 complete | ✅ Yes (done) |
| 7.2 | 7 | Build per-user dashboard (u/index.astro + cal-heatmap) | done | #36 | #71 | merged | 7.1 | ✅ Yes (done) |
| 7.3 | 7 | Build documentation pages | done | #37 | #72 | merged | 7.1 | ✅ Yes (done) |
| 7.4 | 7 | Build landing page | done | #38 | #70 | merged | 7.1 | ✅ Yes (done) |
| 8.1 | 8 | Implement Rust binary release CI | done | #39 | #73 | merged | epics 1–7 complete | ✅ Yes (done) |
| 8.2 | 8 | Implement Cloudflare Pages deploy workflow | done | #40 | #74 | merged | epics 1–7 complete | ✅ Yes (done) |
| 8.3 | 8 | Configure GitHub Actions Marketplace publication | done | #41 | #75 | merged | epics 1–7 complete | ✅ Yes (done) |
| 9.1 | 9 | Artifact hygiene — fix stale story statuses and missing records | done | #81 | #94 | merged | none (independent) | ✅ Yes (done) |
| 9.2 | 9 | Retrospective code reviews for Stories 4.3 and 4.4 | done | #82 | #95 | merged | none (independent) | ✅ Yes (done) |
| 9.3 | 9 | Fix test_6_2.bats pre-existing failures | done | #83 | #96 | merged | none (independent) | ✅ Yes (done) |
| 9.4 | 9 | Bash installer — refactor EXIT trap to composable cleanup | done | #84 | #97 | merged | none (independent) | ✅ Yes (done) |
| 9.5 | 9 | Rust — remove dead_code suppressors and verify lint clean | done | #85 | #99 | merged | none (independent) | ✅ Yes (done) |
| 9.6 | 9 | First release — push v0.1.0 tag and create v1 floating tag | done | #86 | #98 | merged | 9.3 (recommended) | ✅ Yes (done) |
| 9.7 | 9 | aggregate.yml — add concurrency group to prevent push conflicts | backlog | #87 | — | — | none (independent) | ✅ Yes |
| 9.8 | 9 | Architecture documentation — capture post-sprint lessons | backlog | #88 | — | — | none (independent) | ✅ Yes |
| 9.9 | 9 | Python script hardening — update_readme.py and aggregate.py improvements | backlog | #89 | — | — | none (independent) | ✅ Yes |

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

**Epic 9:**
- All stories (9.1–9.9) are independent of each other and can be worked in any order
- Recommended order: 9.1 → 9.2 → 9.3 → 9.5 → 9.4 → 9.6 → 9.7 → 9.8 → 9.9
- 9.6 should only follow 9.3 (broken installer tests should not be on a released version)

## Notes

- **Epic 1 complete** — all four PRs merged: #43 (1.1), #44 (1.2), #46 (1.3), #45 (1.4). Worktrees and remote branches cleaned up.
- **Epic 2 complete** — all five stories done. PRs #48 (2.1), #49 (2.2), #50 (2.3) merged via GitHub. Stories 2.4 and 2.5 implemented via direct commits to main (deb76aa, 23b8443); PRs #53 and #51 closed accordingly. Worktrees and remote branches for 2.4 and 2.5 cleaned up.
- **Story 3.1 done** — PR #54 merged 2026-04-11. Epic 3 stories 3.2, 3.3, 3.4 all unblocked.
- **Story 3.2 done** — PR #56 merged 2026-04-11T07:56:38Z. Worktree and remote branch cleaned up.
- **Story 3.3 done** — PR #55 merged 2026-04-11T07:57:49Z. Worktree cleaned up; remote branch auto-deleted by GitHub on merge.
- **Story 3.4 done** — PR #57 merged 2026-04-11T11:19:43Z. Worktree cleaned up; remote branch auto-deleted by GitHub on merge.
- **Epic 3 complete** — All stories (3.1–3.4) done.
- **Story 4.1 done** — PR #58 merged 2026-04-11T12:31:07Z. Worktree cleaned up; remote branch auto-deleted by GitHub on merge.
- **Story 4.2 done** — PR #61 merged 2026-04-11T12:39:01Z. Worktree cleaned up; remote branch auto-deleted by GitHub on merge.
- **Story 4.3 done** — PR #62 merged 2026-04-11T12:42:30Z. Worktree cleaned up; remote branch auto-deleted by GitHub on merge.
- **Story 4.4 done** — PR #63 merged 2026-04-11T13:32:22Z. Worktree cleaned up; remote branch auto-deleted by GitHub on merge.
- **Epic 4 complete** — All stories (4.1–4.4) done.
- **Story 5.1 done** — PR #64 merged 2026-04-11T14:25:26Z. Worktree and remote branch cleaned up.
- **Story 5.2 done** — PR #65 merged 2026-04-11T14:32:23Z. Worktree and remote branch cleaned up.
- **Story 5.3 done** — PR #66 merged 2026-04-11T14:34:02Z. Worktree and remote branch cleaned up.
- **Story 5.4 done** — PR #68 merged 2026-04-11T23:54:53Z. Worktree and remote branch cleaned up.
- **Story 5.5 done** — PR #67 merged 2026-04-11T23:56:08Z. Worktree cleaned up; remote branch auto-deleted by GitHub on merge.
- **Epic 5 complete** — All stories (5.1–5.5) done.
- **Story 7.1 done** — PR #69 merged 2026-04-12T00:37:53Z. Worktree and remote branch cleaned up.
- **Story 7.2 done** — PR #71 merged 2026-04-12T01:13:29Z.
- **Story 7.3 done** — PR #72 merged 2026-04-12T01:14:22Z.
- **Story 7.4 done** — PR #70 merged 2026-04-12T01:15:26Z.
- **Epic 7 complete** — All stories (7.1–7.4) done.
- **Story 8.1 done** — PR #73 merged 2026-04-12T07:15:17Z.
- **Story 8.2 done** — PR #74 merged 2026-04-12T07:16:27Z.
- **Story 8.3 done** — PR #75 merged 2026-04-12T07:17:43Z.
- **Epic 8 complete** — All stories (8.1–8.3) done.
- **Story 6.1 done** — PR #76 merged 2026-04-12T09:28:17Z. Issue #31 closed. Remote branch deleted. Worktree already cleaned up.
- **Story 6.2 done** — PR #77 merged 2026-04-12T10:09:15Z. Issue #32 closed. Worktree and remote branch cleaned up.
- **Story 6.3 done** — PR #78 merged 2026-04-12T10:12:42Z. Issue #33 closed. Worktree and remote branch cleaned up.
- **Story 6.4 done** — PR #79 merged 2026-04-13. Hook configuration, README markers, and backfill trigger implemented. Issue #34 closed.
- **Epic 6 complete** — All stories (6.1–6.4) done.
- **All Epics 1–8 complete** — The sprint is fully done. Epic 9 (Post-Sprint Quality & Technical Debt) is in progress.
- **GitHub auth:** `gh auth status` confirmed working (keyring). All PR/issue lookups use `gh` CLI directly.
- **`bad` label:** Exists in repo. All story issues (#9–#41) and epic issues (#1–#8) already created. Epic 9 issues #80–#89 already created.
- **Epic 9 stories 9.1–9.6 done** — PRs #94 (9.1), #95 (9.2), #96 (9.3), #97 (9.4), #99 (9.5), #98 (9.6) all merged 2026-04-13. Worktrees for 9.4, 9.5, 9.6 removed; remote branches deleted.
- **Epic 9 stories 9.7–9.9** — backlog, no PRs yet. All three are independent and ready to work.
- **No open PRs** as of 2026-04-13.
