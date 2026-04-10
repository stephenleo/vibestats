---
stepsCompleted: ['step-01-init', 'step-02-discovery', 'step-02b-vision', 'step-02c-executive-summary', 'step-03-success', 'step-04-journeys', 'step-05-domain-skipped', 'step-06-innovation', 'step-07-project-type', 'step-08-scoping', 'step-09-functional', 'step-10-nonfunctional', 'step-11-polish']
inputDocuments:
  - '_bmad-output/planning-artifacts/research/technical-claude-code-stats-heatmap-research-2026-04-06.md'
workflowType: 'prd'
classification:
  projectType: developer_tool
  domain: general
  complexity: low
  projectContext: greenfield
briefCount: 0
researchCount: 1
brainstormingCount: 0
projectDocsCount: 0
---

# Product Requirements Document - vibestats

**Author:** Leo
**Date:** 2026-04-06

## Executive Summary

**vibestats** is a passive developer tool that aggregates Claude Code usage across all of a developer's machines and renders a GitHub contributions-style activity heat map on their GitHub profile README. Installation is a single command; after that the tool operates silently — no dashboards to open, no manual exports, no cron jobs to manage. The target user is any Claude Code user who wants to make their AI-assisted development visible as a professional signal, particularly during a job search where AI fluency is increasingly a hiring differentiator.

The core problem: Claude Code usage is invisible. A developer may be spending hours per day building with AI assistance across multiple machines, but their GitHub profile — the primary surface where professional credibility is established — shows none of it. vibestats makes that invisible effort visible without any ongoing user action.

### What Makes This Special

The "reveal moment" is the differentiator: the first time a user sees their aggregated usage heat map, spanning work and personal machines across weeks or months, the numbers are often surprising. This emotional hook — *I didn't realise how much I've been using this* — makes vibestats shareable and memorable.

Two properties make this unique among developer tools: (1) **zero ongoing effort** — it installs once and updates automatically via Claude Code's native hooks system, and (2) **cross-machine aggregation** — it synthesises usage from every machine the developer works on, something no existing Claude Code stats tool does. The technical research confirmed no tool currently combines passive collection, cross-machine sync, and GitHub profile integration.

The timing is deliberate: Claude Code is new, the community has no passive usage tracker yet, and AI fluency is becoming a visible job market signal. This is a first-mover opportunity with a small, well-defined build scope.

## Project Classification

- **Project Type:** Developer Tool (installable binary + GitHub Actions workflow)
- **Domain:** General developer tooling / open source
- **Complexity:** Low — no regulated industry, standard GitHub API, SVG generation
- **Project Context:** Greenfield

## Success Criteria

### User Success

- Heatmap reflecting aggregated Claude Code usage across all of the user's machines appears on their GitHub profile README within minutes of installation
- Data updates automatically after each Claude Code session on any registered machine — no manual action required
- Cross-machine aggregation is seamless; the user never needs to know how many machines are syncing
- Interactive dashboard at `vibestats.dev/username` loads and displays per-day activity data with hover detail

### Business Success

- **1,000 GitHub stars within 3 months of public release**
- Measurable install/download count via install script telemetry or GitHub release asset downloads
- Listed on [awesome-claude-code](https://github.com/hesreallyhim/awesome-claude-code) and discoverable via GitHub topic search for Claude Code tools

### Technical Success

- Single-command installer configures both user repos, hooks, and Actions workflow without manual steps
- Machine-side sync binary has zero runtime dependencies (no `jq`, `curl`, `node`, or shell version requirements)
- GitHub Actions usage stays within free tier (~60 min/month via daily cron — not per-push)
- Hook fires reliably using `Stop` + `SessionStart` catch-up strategy (not `SessionEnd`, which is broken in production)
- SVG committed to `vibestats-data` repo and live on profile README within one Actions run of any machine syncing

### Measurable Outcomes

| Outcome | Target | Timeframe |
|---|---|---|
| GitHub stars | 1,000 | 3 months post-launch |
| Machines syncing for author | All active machines | Day 1 |
| Install-to-heatmap time | < 5 minutes | Per user |
| Actions minutes consumed | < 60 min/month | Ongoing |

## Product Scope

> **Summary view.** See [Project Scoping & Phased Development](#project-scoping--phased-development) for the authoritative breakdown including architecture decisions and install flow.

### MVP — Minimum Viable Product

The smallest thing that's genuinely useful and clearly differentiated from a `/stats` screenshot:

- **Two user repos**: `username/vibestats-data` (private, raw machine JSON + workflow) + `username/username` (public, profile README + `vibestats/heatmap.svg` + `vibestats/data.json`)
- **Rust sync binary**: reads `~/.claude/projects/**/*.jsonl`, pushes daily usage JSON to `vibestats-data` via GitHub Contents API — zero runtime deps
- **Claude Code hooks**: `Stop` hook (primary, throttled to once/5 min) + `SessionStart` catch-up safety net
- **GitHub Actions**: daily cron aggregates all machine JSON, generates `heatmap.svg` + `data.json`, commits both to `username/username/vibestats/`, updates profile README markers
- **Static SVG in README**: embedded via HTML comment markers (`<!-- vibestats-start -->` / `<!-- vibestats-end -->`), served from `raw.githubusercontent.com/username/username/main/vibestats/heatmap.svg`
- **Interactive dashboard**: `cal-heatmap`-based view served at `vibestats.dev/username`, fetches `vibestats/data.json` client-side from the public profile repo

### Growth Features (Post-MVP)

- Opt-in Anthropic Admin API as an authoritative data source (for users with org access)
- Per-project breakdown in the `vibestats.dev/[username]` dashboard
- Token usage, cost estimates, model breakdown alongside session counts
- Multi-user / team aggregate views

### Vision (Future)

**vibestats** as a broader AI coding activity platform — not just Claude Code, but any AI assistant a developer uses. The `vibestats.dev/[username]` dashboard becomes the primary surface, with the README heatmap as the entry point that drives profile discovery. The Rust binary evolves into a universal activity collector across AI tooling.

## User Journeys

### Journey 1: The First Reveal — Leo Installs vibestats

Leo is two weeks into a job search. His GitHub profile has green squares from commits, but nothing showing the hours he's been building with Claude Code across his work MacBook and home desktop. He finds vibestats via a tweet, skims the README, and runs the install command.

The installer creates `vibestats-data` (private), writes the Actions workflow into it, stores his GitHub OAuth token locally, writes hooks config to `~/.claude/settings.json`, and adds the `<!-- vibestats-start/end -->` markers to his profile README. It takes under 5 minutes.

That evening he finishes a Claude Code session. The `Stop` hook fires, the Rust binary reads his JSONL files and pushes a JSON blob to `vibestats-data` via the GitHub Contents API. The next morning, the Actions cron runs — it aggregates all machine JSON files, generates `heatmap.svg` (Claude orange) and `data.json`, and commits both to `username/username/vibestats/`. The profile README already has the markers pointing to the raw SVG URL, so the heatmap is live.

Leo opens his GitHub profile. Where there was nothing, there's now a heat map showing months of Claude Code activity across both machines — streaks he didn't know he had. The numbers are larger than he expected. He screenshots it immediately.

**This journey reveals requirements for:** installer script, hooks configuration, JSONL reader, GitHub Contents API push, Actions aggregation, SVG generation, README marker injection.

---

### Journey 2: Adding a Second Machine — Work Laptop

Leo sets vibestats up at home first. A week later he runs the same install command on his work MacBook. The installer detects that `vibestats-data` already exists under his account, skips repo creation, and registers the new machine with a unique identifier (hostname + machine UUID). It adds the work MacBook's hook config and writes a new per-machine JSON file path to `vibestats-data`.

From this point, both machines push independently. The Actions aggregator doesn't care how many machine files exist — it folds them all into a single daily total before generating the SVG. Leo never thinks about this again.

**This journey reveals requirements for:** existing-repo detection in installer, per-machine JSON namespacing, multi-source aggregation in Actions.

---

### Journey 3: Silent Failure — Token Expires on Work Machine

Three weeks after setup, Leo's GitHub auth token on his work MacBook becomes invalid (revoked or invalidated by GitHub). The Rust binary fails silently — it tries to push, gets a 401, and logs the failure locally. Leo doesn't notice because the home machine is still syncing and the heatmap keeps updating.

Two days later, Leo starts a new Claude Code session on the work machine. The `SessionStart` hook fires and the binary checks last sync status. It detects the last successful sync was 48 hours ago and prints a warning directly to the terminal:

```
vibestats: last sync was 2 days ago on this machine. Run `vibestats status` to diagnose.
```

Leo runs `vibestats status`. It shows both registered machines, their last sync timestamps, and a connectivity check result — the work machine shows a red token error with a fix instruction. He runs `vibestats sync` to force an immediate sync after updating the token. The gap in the heatmap is backfilled from local JSONL history.

**This journey reveals requirements for:** local sync failure logging, `SessionStart` staleness check, `vibestats status` command, `vibestats sync` command, backfill from JSONL on forced sync.

---

### Journey 4: The Audience — A Recruiter Visits Leo's Profile

A hiring manager at a developer tools company is reviewing Leo's GitHub profile. She sees the standard green contribution graph — and below it, an identically-shaped heatmap in Claude orange. She recognises the pattern instantly. No label needed beyond "Claude Code Activity."

The density of the orange squares tells a story without words: consistent daily usage, heavy streaks during project sprints. She clicks the "View interactive dashboard →" link beneath the heatmap. The dashboard at `vibestats.dev/leo` loads a `cal-heatmap` grid with hover tooltips — hovering a cell shows the session count and approximate active time for that day. The data is fetched client-side from Leo's public `username/username` repo.

She's seen green squares on hundreds of profiles. She hasn't seen this before. She moves Leo to the next round.

**This journey reveals requirements for:** SVG visual design (Claude orange, GitHub contributions grid shape), README link to `vibestats.dev/username`, `vibestats.dev` dashboard with `cal-heatmap`, hover tooltips with per-day session data.

---

### Journey 5: The Backfill — History Made Visible on First Install

Leo has been using Claude Code for four months before he discovers vibestats. When he runs the install command, the installer doesn't just configure hooks for future sessions — it runs an immediate backfill pass: the Rust binary walks the entire `~/.claude/projects/**/*.jsonl` directory, parses every historical session record, and pushes a complete per-day usage summary to `vibestats-data` in one shot.

The first time the Actions cron runs (or within minutes if the installer triggers an immediate sync), the SVG is generated from the full four-month history. When Leo opens his profile, the heatmap isn't sparse — it's dense with months of activity he'd forgotten about. Sprints, late nights, the week he rebuilt his portfolio. All of it visible at once.

This is the "wow" moment. Not "I'll check back in a few weeks when there's data" — but immediate gratification from existing history. The backfill is what makes the first reveal emotionally resonant rather than anticlimactic.

The same backfill logic powers `vibestats sync --backfill` for gap recovery — as seen in Journey 3 when the work machine's token expired.

**This journey reveals requirements for:** full historical JSONL backfill on first install, installer-triggered immediate sync (not waiting for first Stop hook), `vibestats sync --backfill` flag for manual gap recovery.

---

### Journey Requirements Summary

| Capability | Revealed By |
|---|---|
| Single-command installer | Journey 1 |
| Existing repo detection | Journey 2 |
| JSONL reader + GitHub Contents API push | Journeys 1 & 2 |
| Per-machine JSON namespacing | Journey 2 |
| Multi-source aggregation in Actions | Journey 2 |
| Local sync failure logging | Journey 3 |
| `SessionStart` staleness warning | Journey 3 |
| `vibestats status` + `vibestats sync` CLI | Journey 3 |
| JSONL backfill on forced sync | Journey 3 |
| Full historical backfill on first install | Journey 5 |
| Installer-triggered immediate sync | Journey 5 |
| `vibestats sync --backfill` flag | Journey 5 |
| Claude-orange GitHub-contributions-style SVG | Journey 4 |
| `vibestats.dev/[username]` dashboard (`cal-heatmap`) | Journey 4 |
| Per-day hover tooltips | Journey 4 |

## Developer Tool Specific Requirements

### Project-Type Overview

vibestats is distributed as a pre-compiled Rust binary + installer script + GitHub Actions community action. There are three surfaces: the Rust binary (machine-side, CLI), the GitHub Actions workflow (server-side, automated), and the `vibestats.dev` dashboard (universal, interactive). Users create two repos: `vibestats-data` (private, raw data) and their existing `username/username` profile repo receives a `vibestats/` subdirectory with the generated SVG and JSON.

### Language Matrix

| Component | Language | Rationale |
|---|---|---|
| Machine-side sync binary | Rust | Zero runtime dependencies — no `jq`, `curl`, `node`, or Bash 4+ required |
| GitHub Actions aggregation + SVG generation | Python (stdlib only) | No pip installs in Actions environment; stdlib `json`, `datetime`, and string formatting are sufficient |
| Installer script | Bash | Universal on macOS and Linux; WSL2 users use the Linux path |
| `vibestats.dev` dashboard | HTML + JavaScript (`cal-heatmap`) | Static Astro page served centrally; fetches per-user data client-side |
| Documentation site + universal dashboard | Astro | Static site at `vibestats.dev`; docs at `vibestats.dev/docs`, per-user dashboard at `vibestats.dev/[username]` |

### Installation Methods

| Method | Target | Notes |
|---|---|---|
| `curl -sSf https://vibestats.dev/install.sh \| bash` | Primary (macOS + Linux + WSL2) | Downloads pre-compiled binary from GitHub Releases, configures hooks, creates repos |
| GitHub Releases manual download | Fallback | Direct `.tar.gz` for users who distrust pipe-to-bash |

**Platform support via GitHub Releases:**
- `vibestats-macos-arm64` — Apple Silicon
- `vibestats-macos-x86_64` — Intel Mac
- `vibestats-linux-x86_64` — Linux and Windows via WSL2

Windows native (non-WSL2) is out of scope for MVP — Claude Code itself runs in WSL2 on Windows.

### CLI Surface

| Command | Description |
|---|---|
| `vibestats status` | Shows all registered machines, last sync timestamp per machine, GitHub connectivity check, token validity |
| `vibestats sync` | Forces immediate sync from current machine's JSONL to `vibestats-data` |
| `vibestats sync --backfill` | Full historical JSONL backfill — reads all sessions ever recorded on this machine |
| `vibestats machines list` | Lists all machines registered in `vibestats-data` with last-seen timestamps |
| `vibestats machines remove <id>` | Removes a machine's JSON file from `vibestats-data` and de-registers it |
| `vibestats uninstall` | Removes hooks from `~/.claude/settings.json` and deletes the local binary. Prints a message informing the user they can manually delete `vibestats-data` and remove the `<!-- vibestats-start/end -->` markers from their profile README if they wish to fully remove all traces. |

### Documentation Requirements

| Surface | URL | Purpose |
|---|---|---|
| Docs site | `vibestats.dev` | Astro-based documentation: quickstart, configuration reference, troubleshooting, CLI reference |
| Per-user dashboard | `vibestats.dev/[username]` | Universal Astro page fetching `username/username/vibestats/data.json` client-side — no per-user hosting required |
| Tool README | `github.com/stephenleo/vibestats` | Install command, heatmap screenshot, one-paragraph description, link to `vibestats.dev` |

**Docs site minimum content at launch:**
- Quickstart (install in 5 minutes)
- How it works (architecture overview diagram)
- CLI reference (`vibestats` subcommands)
- Troubleshooting (token expiry, hook not firing, missing machine data)
- `CONTRIBUTING.md` for open source contributors

### Implementation Considerations

- The installer must detect if `vibestats-data` already exists (multi-machine setup path) vs. first-time install (repo creation path)
- The Rust binary must store its config (GitHub OAuth token via `gh auth token`, machine ID, `vibestats-data` repo path) in a local config file — `~/.config/vibestats/config.toml`
- GitHub Releases CI pipeline (via GitHub Actions) must build all three platform binaries and attach them to each release tag

## Innovation & Novel Patterns

### Detected Innovation Areas

**1. New paradigm for developer identity signals**
The GitHub contribution graph established that coding activity can be a passive public signal. vibestats extends this paradigm to AI-assisted development for the first time. No existing tool treats Claude Code usage as a profile artifact — prior art (ccusage, cstats, claude-code-stats) requires manual invocation and produces local-only output. vibestats is the first to make AI coding activity a persistent, passive, publicly visible professional signal.

**2. Cross-machine passive aggregation via Claude Code's native hooks**
All prior Claude Code stats tools require explicit user invocation. vibestats embeds collection into the hooks system so it operates with zero ongoing user action after install. The three-layer sync strategy — `Stop` hook (primary, throttled), `SessionStart` catch-up (safety net), and installer backfill (historical) — is a novel response to `SessionEnd` being broken in production (confirmed via GitHub issues #6428, #17885, #34954). No existing tool has solved the cross-machine aggregation problem for Claude Code.

**3. Static SVG + GitHub Pages split architecture**
Rather than choosing between a static embeddable artifact and an interactive experience, vibestats delivers both from separate surfaces. GitHub's DOMPurify strips all SVG interactivity in `<img>` context — this constraint becomes a forcing function for a cleaner two-surface design: a `raw.githubusercontent.com` SVG for the README (no server, no JavaScript) and a GitHub Pages `cal-heatmap` companion for full interactivity. The split also provides a natural growth path: the GitHub Pages surface can expand into a richer stats platform without touching the README embed.

### Market Context & Competitive Landscape

- No existing tool combines passive cross-machine collection, GitHub profile integration, and a public visual artifact for Claude Code usage
- The `awesome-claude-code` ecosystem is nascent — vibestats would be among the first tools targeting profile/identity use cases rather than local productivity
- GitHub contribution graphs have trained developers to read heat map density as a proxy for effort and consistency — vibestats leverages this trained intuition at zero UX cost by replicating the exact grid shape in Claude orange

### Validation Approach

- Author (Leo) is the primary validation case — if the tool installs cleanly across his own machines and produces a compelling heatmap during his job search, the core value proposition is proven
- Star velocity in the first week post-launch is the leading indicator of product-market fit: if the reveal screenshot is shareable, organic growth follows

### Risk Mitigation

| Risk | Mitigation |
|---|---|
| `SessionEnd` hook broken | Three-layer sync strategy; `Stop` hook is the reliable primary |
| GitHub DOMPurify strips SVG interactivity | `vibestats.dev/[username]` for interactive view; static SVG for README embed |
| Actions minutes budget exceeded | Daily cron only (not per-push); ~60 min/month on free tier |
| Cross-machine auth complexity | Per-machine OAuth token stored via `gh`; `vibestats status` surfaces auth errors |
| First-install heatmap is sparse (anticlimactic) | Installer-triggered backfill from full JSONL history on first run |

## Project Scoping & Phased Development

### MVP Strategy & Philosophy

**MVP Approach:** Problem-solving MVP — ship the minimum that makes the core differentiator real. The single-machine `/stats` screenshot already exists; the value proposition is cross-machine aggregation + passive profile artifact. MVP is validated the moment the author's own heatmap appears on his GitHub profile pulling data from multiple machines.

**Resource Requirements:** Solo developer (Stephen Leo) building with Claude Code.

### Architecture Decision: Community GitHub Action (ADR-10)

**Decision:** vibestats publishes a community GitHub Action at `stephenleo/vibestats` rather than requiring users to fork the repo. Users' `vibestats-data` repos contain a minimal workflow that calls `uses: stephenleo/vibestats@v1`.

**Why:** Eliminates the fork-update problem entirely. When vibestats ships aggregation fixes or new SVG features, all users pick them up automatically on their next cron run within a major version. The `vibestats` repo is listed on the GitHub Actions Marketplace, providing an additional discoverability surface.

**Two user repos (plus the tool repo):**
- `stephenleo/vibestats` — tool repo: community GitHub Action + installer script + `vibestats.dev` Astro docs/dashboard (not forked by users)
- `username/vibestats-data` — private, raw machine JSON files + Actions workflow calling `stephenleo/vibestats@v1`
- `username/username` — public profile README + `vibestats/heatmap.svg` + `vibestats/data.json` (committed by the Action)

### Auth Architecture (ADR-11)

**Decision:** `gh` CLI is a required dependency. The installer installs `gh` if missing and runs `gh auth login` if not authenticated. All GitHub auth flows through `gh` — no vibestats OAuth App, no PAT management, no copy-pasting tokens across machines.

**`gh` as install/auth-time dependency (not runtime hot-path):**
The `Stop` hook fires after every Claude response. Spawning `gh` as a subprocess there adds latency. Instead: at install/auth time, `gh auth token` is called once and the token is stored in `~/.config/vibestats/config.toml`. The Rust binary uses the stored token for all API calls. `gh` is not invoked on the hot path.

**Two tokens, two principals, set once each:**

| Token | Lives | Used by | Scopes | Set when |
|---|---|---|---|---|
| `gh` OAuth token | `~/.config/vibestats/config.toml` per machine | Rust binary (machine → `vibestats-data` push) | `vibestats-data`: Contents Write | Every machine install |
| `VIBESTATS_TOKEN` | `vibestats-data` Actions secret | GitHub Action (commits `vibestats/` to `username/username`) | `username/username`: Contents Write | First install only |

Note: `vibestats-data` read/write within the workflow is covered by the automatic `GITHUB_TOKEN` — `VIBESTATS_TOKEN` only needs access to the separate `username/username` repo.

**Installer logic:**
```
gh installed?      → No  → install gh (brew / apt / GitHub Releases)
gh authenticated?  → No  → gh auth login (standard browser flow)
vibestats-data exists?
  NO  → create repos + set VIBESTATS_TOKEN secret + store local token
  YES → store local token only (secret already set, not overwritten)
```

**Token refresh:** `gh` OAuth tokens do not expire on a schedule. If `VIBESTATS_TOKEN` ever needs refreshing (access revoked, GitHub invalidation), `vibestats auth` on any machine re-runs `gh auth token` → updates local config AND the Actions secret. `vibestats status` surfaces auth errors with fix instructions (Journey 3).

**Why `gh` over a vibestats OAuth App:** Users trust the official GitHub CLI. No third-party OAuth App to register or maintain. `gh` handles secure token storage (macOS Keychain, etc.) and the installer can use `gh api`, `gh repo create`, and `gh secret set` for all setup operations — no hand-rolled GitHub API calls in Bash.

### MVP Feature Set (Phase 1)

**Core User Journeys Supported:** All five journeys (install + first reveal, multi-machine, broken sync recovery, profile visitor, backfill).

**Must-Have Capabilities:**

- Rust sync binary — macOS arm64/x86_64, Linux x86_64 (WSL2 = Linux binary)
- Single-command installer: installs `gh` if missing, runs `gh auth login` if needed, creates `vibestats-data`, writes workflow, stores OAuth token, configures hooks, adds markers to profile README, triggers backfill
- Existing-repo detection (multi-machine install path — no re-auth, no secret overwrite)
- Historical JSONL backfill on first install + `vibestats sync --backfill`
- Community GitHub Action (`stephenleo/vibestats@v1`) — aggregates machine JSON, generates `heatmap.svg` + `data.json`, commits both to `username/username/vibestats/`, updates profile README markers
- Static SVG in profile README via `<!-- vibestats-start/end -->` markers, served from `raw.githubusercontent.com`
- Universal dashboard at `vibestats.dev/username` — Astro page fetching `vibestats/data.json` client-side
- CLI: `vibestats status`, `vibestats sync`, `vibestats sync --backfill`, `vibestats machines list`, `vibestats machines remove`, `vibestats uninstall`
- Astro docs site at `vibestats.dev` (quickstart, how it works, CLI reference, troubleshooting)
- GitHub Releases CI: builds and attaches all three platform binaries per release tag

### Post-MVP Features (Phase 2 — Growth)

- `vibestats update` command — downloads latest binary from GitHub Releases
- Opt-in Anthropic Admin API as authoritative data source (for users with org access)
- Per-project breakdown in `vibestats.dev/[username]` dashboard
- Token usage, cost estimates, model breakdown alongside session counts
- `vibestats machines rename <id> <name>` for human-readable machine labels

### Future Vision (Phase 3 — Expansion)

- Multi-AI assistant support (Cursor, Copilot, Gemini CLI) — vibestats becomes a universal AI coding activity platform
- Team/org aggregate views
- `vibestats.dev/[username]` dashboard as primary surface; README heatmap as discovery entry point
- Plugin system for community-contributed stat collectors

## Functional Requirements

### Installation & Setup

- FR1: A user can install vibestats on a machine with a single shell command
- FR2: The installer detects whether `gh` is installed and installs it automatically if missing
- FR3: The installer detects whether the user is authenticated with GitHub and initiates `gh auth login` if not
- FR4: The installer creates the `vibestats-data` private repository on first install
- FR5: The installer detects an existing `vibestats-data` repository and skips repo creation and secret setup on subsequent machine installs
- FR6: The installer registers the current machine with a unique identifier in `vibestats-data`
- FR7: The installer writes the Actions workflow into `vibestats-data` calling `stephenleo/vibestats@v1`
- FR8: The installer configures Claude Code hooks (`Stop` and `SessionStart`) in `~/.claude/settings.json`
- FR9: The installer adds `<!-- vibestats-start -->` / `<!-- vibestats-end -->` markers to the user's profile README
- FR10: The installer sets the `VIBESTATS_TOKEN` Actions secret in `vibestats-data` on first install only
- FR11: The installer triggers a full historical JSONL backfill immediately after setup completes

### Data Collection & Sync

- FR12: The system captures Claude Code session activity automatically after every session response via the `Stop` hook
- FR13: The system performs a catch-up sync on every new Claude Code session start via the `SessionStart` hook
- FR14: The Rust binary reads Claude Code JSONL files from `~/.claude/projects/**/*.jsonl` to extract per-day usage data
- FR15: The system throttles `Stop` hook sync to at most once per 5 minutes
- FR16: The system pushes per-machine daily usage JSON to `vibestats-data` via the GitHub Contents API
- FR17: A user can force an immediate sync from any machine via `vibestats sync`
- FR18: A user can trigger a full historical JSONL backfill from any machine via `vibestats sync --backfill`
- FR19: The system logs sync failures locally and surfaces a warning on the next `SessionStart` if the last successful sync exceeds 24 hours

### Aggregation & Output Generation

- FR20: The GitHub Action aggregates all per-machine JSON files in `vibestats-data` into a single daily activity dataset
- FR21: The GitHub Action generates a static `heatmap.svg` using the GitHub contributions grid shape in Claude orange
- FR22: The GitHub Action generates a `data.json` file containing the full aggregated daily activity dataset
- FR23: The GitHub Action commits `heatmap.svg` and `data.json` to `username/username/vibestats/`
- FR24: The GitHub Action updates the user's profile README between the `<!-- vibestats-start/end -->` markers to embed the current SVG
- FR25: The GitHub Action runs on a daily cron schedule
- FR26: A user can manually trigger the GitHub Action via `workflow_dispatch`

### Profile Display

- FR27: The `heatmap.svg` is publicly accessible via `raw.githubusercontent.com/username/username/main/vibestats/heatmap.svg` and embeds in the profile README without JavaScript
- FR28: The profile README heatmap includes a link to `vibestats.dev/username`
- FR29: The `vibestats.dev/[username]` dashboard fetches `vibestats/data.json` from the user's public profile repo client-side — no per-user hosting required
- FR30: The dashboard displays the full activity heatmap using `cal-heatmap`
- FR31: The dashboard shows per-day session count and approximate active time on cell hover

### CLI & Machine Management

- FR32: A user can view all registered machines, their last sync timestamps, and GitHub connectivity status via `vibestats status`
- FR33: A user can verify their current auth token validity via `vibestats status`
- FR34: A user can list all machines registered in `vibestats-data` via `vibestats machines list`
- FR35: A user can remove a specific machine's data from `vibestats-data` via `vibestats machines remove <id>`
- FR36: A user can re-authenticate vibestats on any machine via `vibestats auth`, which updates both the local token and the `VIBESTATS_TOKEN` Actions secret
- FR37: A user can uninstall vibestats from a machine via `vibestats uninstall`, which removes hooks from `~/.claude/settings.json` and deletes the local binary, and prints instructions for manual repo and README cleanup

### Authentication

- FR38: The system uses the `gh` CLI as the authentication provider — `gh` is installed by the installer if missing
- FR39: The system stores the GitHub OAuth token in `~/.config/vibestats/config.toml` after obtaining it via `gh auth token`
- FR40: The system detects invalid or missing auth tokens and notifies the user on `SessionStart` with remediation instructions (`vibestats auth`)

### Distribution & Documentation

- FR41: The project provides pre-compiled binaries for macOS arm64, macOS x86_64, and Linux x86_64 via GitHub Releases on every tagged release
- FR42: The `stephenleo/vibestats` repo is published to the GitHub Actions Marketplace as a community action referenceable as `stephenleo/vibestats@v1`
- FR43: The project provides a public documentation and dashboard site at `vibestats.dev` covering quickstart, CLI reference, architecture, troubleshooting, and per-user dashboards at `vibestats.dev/[username]`

## Non-Functional Requirements

### Performance

- **NFR1 — Hook latency:** The `Stop` hook execution (JSONL read + GitHub API push) must complete within 2 seconds under normal network conditions. The hook runs async (`async: true`) so it does not block Claude Code, but prolonged execution degrades system resources.
- **NFR2 — Hook throttle:** Sync is throttled to once per 5 minutes maximum to prevent API rate limit exhaustion and battery drain during long sessions.
- **NFR3 — Backfill throughput:** A full historical backfill across 12 months of JSONL data must complete within 60 seconds on a standard broadband connection.
- **NFR4 — Dashboard load:** `vibestats.dev/[username]` must render the heatmap within 3 seconds on a standard connection. Data is fetched client-side from a single public JSON file — no server round-trips.
- **NFR5 — Actions runtime:** The daily GitHub Actions cron job must complete within 5 minutes and consume no more than 60 minutes of Actions minutes per month across all runs.

### Security

- **NFR6 — Token storage:** The GitHub OAuth token stored in `~/.config/vibestats/config.toml` must have file permissions set to `600` (owner read/write only) on creation.
- **NFR7 — Token scope minimisation:** `VIBESTATS_TOKEN` is scoped exclusively to `username/username` Contents write. The machine-side token is scoped to `vibestats-data` Contents write. No broader repo or org scopes are requested.
- **NFR8 — No secrets in commits:** The GitHub Action must never commit raw machine JSON (which contains file paths and timestamps) to the public `username/username` repo — only the aggregated `data.json` and `heatmap.svg` outputs.
- **NFR9 — Private data boundary:** Raw per-machine JSON files remain in `vibestats-data` (private). Only aggregated, anonymised daily totals are published to the public profile repo.

### Reliability

- **NFR10 — Hook non-interference:** A crash or unhandled error in the vibestats hook must not propagate to Claude Code or interrupt the user's session. All hook errors must be caught and logged locally.
- **NFR11 — Silent sync failure:** Sync failures (network errors, API rate limits, token issues) must fail silently during sessions. The user is notified only at the next `SessionStart`, not mid-session.
- **NFR12 — Idempotent sync:** Pushing the same daily JSON multiple times must produce identical results — no duplicate data accumulation in `vibestats-data`.
- **NFR13 — Actions resilience:** The GitHub Action must handle transient GitHub API failures with retry logic. A single failed cron run must not corrupt existing SVG or data files.

### Integration

- **NFR14 — JSONL format tolerance:** The JSONL parser must handle missing or unknown fields gracefully across Claude Code versions — future schema additions must not break existing sync.
- **NFR15 — GitHub API rate limits:** The Rust binary must respect GitHub Contents API rate limits (5,000 requests/hour for authenticated users) and implement exponential backoff on 429 responses.
- **NFR16 — `gh` CLI version compatibility:** vibestats must function with `gh` CLI version 2.0 and above. The installer must check the installed `gh` version and warn if below minimum.
- **NFR17 — Actions Marketplace compatibility:** The community action must declare all required inputs, outputs, and permissions in `action.yml` and be compatible with `ubuntu-latest` runners.
