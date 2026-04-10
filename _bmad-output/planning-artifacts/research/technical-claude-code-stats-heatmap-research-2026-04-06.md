---
stepsCompleted: [1, 2, 3, 4, 5, 6]
inputDocuments: []
workflowType: 'research'
lastStep: 1
research_type: 'technical'
research_topic: 'Claude Code stats aggregation across machines — heat map for GitHub profile README'
research_goals: 'Understand where/how Claude Code stores /stats data locally; identify cross-machine sync/aggregation patterns; explore heat map generation approaches; understand GitHub profile README auto-update mechanisms'
user_name: 'Leo'
date: '2026-04-06'
web_research_enabled: true
source_verification: true
---

# Research Report: Technical

**Date:** 2026-04-06
**Author:** Leo
**Research Type:** technical

---

## Executive Summary

**vibestats** is a passive developer tool that aggregates Claude Code usage across all of a user's machines and renders a GitHub-contributions-style activity heat map on their GitHub profile README — installed once with a single command, requiring no manual intervention thereafter.

This research established **9 Architecture Decision Records** through iterative investigation:

| ADR | Decision |
|---|---|
| ADR-1 | Separate private `vibestats-data` repo — raw stats private, SVG public |
| ADR-2 | Three-layer sync: `Stop` hook + `SessionStart` catch-up + installer backfill (`SessionEnd` is unreliable) |
| ADR-3 | `ccusage` / direct JSONL parse as primary source; Anthropic Admin API as opt-in |
| ADR-4 | Static SVG committed to repo served via `raw.githubusercontent.com` — no server |
| ADR-5 | Python for GitHub Actions aggregation + SVG generation (stdlib only) |
| ADR-6 | HTML comment markers (`<!-- vibestats-start/end -->`) for custom README placement |
| ADR-7 | Static SVG in README + interactive GitHub Pages companion (SVG interactivity impossible in `<img>` context) |
| ADR-8 | Decouple machine sync from Actions: machines push via GitHub Contents API directly; Actions runs once daily via cron (~60 min/month) |
| ADR-9 | Rust binary for machine-side sync (zero runtime deps); Python for server-side (GitHub Actions) |

**Critical findings that shaped the design:**
- `SessionEnd` hooks are broken in production (GitHub issues #6428, #17885, #34954) — `Stop` hook is the reliable primary trigger
- GitHub sanitizes all SVG interactivity via DOMPurify — hover tooltips require GitHub Pages
- Triggering Actions on every machine push would consume ~1,800 min/month; daily cron uses ~60 min/month
- macOS ships Bash 3.2 — a compiled Rust binary eliminates shell compatibility and dependency issues entirely

**Three repos, clear separation:**
- `username/vibestats` — public, tool + installer + GitHub Actions workflow
- `username/vibestats-data` — private, machine JSON files + generated `heatmap.svg`
- `username/username` — public profile README, embeds SVG via raw URL

---

## Research Overview

This research investigates building **vibestats** — a tool that passively aggregates Claude Code usage across all of a developer's machines and renders a GitHub-contributions-style heat map on their GitHub profile README. The research covers the full technical stack: Claude Code's local JSONL data format, hook-based event capture, cross-machine sync via GitHub's Contents API, GitHub Actions-based aggregation, SVG generation, and GitHub Pages for interactive visualization.

Nine Architecture Decision Records (ADRs) were established through iterative research and user decisions. Key findings include: `SessionEnd` hooks are unreliable in production (confirmed via GitHub issues #6428, #17885) requiring a three-layer sync strategy; GitHub Actions should be decoupled from machine syncs entirely (daily cron only, ~60 min/month); SVG interactivity is impossible in GitHub README `<img>` context requiring a GitHub Pages companion; and a compiled Rust binary for the machine-side component eliminates all runtime dependencies that create cross-machine install friction.

See the ADR sections within each research phase for full rationale, and the Implementation Approaches section for production-ready code patterns.

---

## Table of Contents

1. [Technical Research Scope Confirmation](#technical-research-scope-confirmation)
2. [Technology Stack Analysis](#technology-stack-analysis) — languages, libraries, prior art
3. [Integration Patterns Analysis](#integration-patterns-analysis) — hooks system, GitHub API, sync strategy, edge cases
4. [Architectural Patterns and Design](#architectural-patterns-and-design) — ADR-1 through ADR-8, system diagram
5. [UX Architecture](#ux-architecture-readme-placement-and-interactivity) — marker placement, SVG vs GitHub Pages
6. [Implementation Approaches](#implementation-approaches-and-technology-adoption) — Rust sync binary, Python SVG, installer, ADR-9

---

## Technical Research Scope Confirmation

**Research Topic:** Claude Code stats aggregation across machines — heat map for GitHub profile README
**Research Goals:** Understand where/how Claude Code stores /stats data locally; identify cross-machine sync/aggregation patterns; explore heat map generation approaches; understand GitHub profile README auto-update mechanisms

**Technical Research Scope:**

- Architecture Analysis - design patterns, frameworks, system architecture
- Implementation Approaches - development methodologies, coding patterns
- Technology Stack - languages, frameworks, tools, platforms
- Integration Patterns - APIs, protocols, interoperability
- Performance Considerations - scalability, optimization, patterns

**Research Methodology:**

- Current web data with rigorous source verification
- Multi-source validation for critical technical claims
- Confidence level framework for uncertain information
- Comprehensive technical coverage with architecture-specific insights

**Scope Confirmed:** 2026-04-06

<!-- Content will be appended sequentially through research workflow steps -->

## Technology Stack Analysis

### Programming Languages

**Primary Options for this Tool:**

- **Python** — Most suitable for data processing scripts, JSONL parsing, and SVG generation. Mature ecosystem for both data wrangling (`json`, `pandas`) and SVG output (`cairosvg`, `svgwrite`). Used by the majority of GitHub profile auto-updaters.
- **JavaScript/TypeScript (Node.js)** — Alternative choice; used by `github-readme-stats` (50k+ stars). Strong ecosystem for SVG rendering and GitHub Actions integration. `ccusage` (the leading Claude Code JSONL reader) is TypeScript.
- **Rust** — Used by `cstats` (terminal Claude stats display); chosen for the machine-side sync binary (ADR-9) — eliminates all runtime dependencies (`jq`, `curl`, `ccusage`, Bash 4+) that cause cross-machine install failures.

_Popular Languages: Python, TypeScript/Node.js_
_Language Evolution: TypeScript dominance growing in GitHub Actions tooling; Python dominant in data pipeline scripts_
_Source: https://github.com/ryoppippi/ccusage, https://github.com/anuraghazra/github-readme-stats, https://github.com/refcell/cstats_

### Development Frameworks and Libraries

**Data Collection (reading Claude Code JSONL):**
- **ccusage** — Open-source TypeScript CLI that reads `~/.claude/projects/**/*.jsonl` files directly and provides usage analytics with `--json` flag for machine-readable output. Best existing prior art.
  - Source: https://github.com/ryoppippi/ccusage

**Heat Map / SVG Generation:**
- **cal-heatmap** (3.1k stars) — JavaScript time-series calendar heatmap library with animated navigation, timezone support, and plugin system. Most feature-complete option.
  - Source: https://github.com/wa0x6e/cal-heatmap
- **react-calendar-heatmap** — React component, SVG-based, GitHub-inspired. Good for server-side rendering.
  - Source: https://github.com/kevinsqi/react-calendar-heatmap
- **github-contributions-canvas** — Draws GitHub contribution heatmaps on HTML5 Canvas; simpler API than cal-heatmap.
  - Source: https://github.com/sallar/github-contributions-canvas
- **Python svgwrite / cairosvg** — For pure-Python SVG generation without a browser runtime; good for GitHub Actions environments.

**GitHub Actions / Automation:**
- **stefanzweifel/git-auto-commit-action** — Standard action for committing generated files back to the repo automatically.
  - Source: https://github.com/stefanzweifel/git-auto-commit-action
- **webfactory/ssh-agent** — For deploy key authentication from Actions runners.

**Existing Claude Code Stats Tools (Prior Art):**
- **claude-code-stats** (AeternaLabsHQ) — HTML dashboard, local only. Source: https://github.com/AeternaLabsHQ/claude-code-stats
- **cstats** (refcell) — Terminal stats using SQLite + Anthropic API. Source: https://github.com/refcell/cstats
- **awesome-claude-code** — Curated list of community tools. Source: https://github.com/hesreallyhim/awesome-claude-code

_Major Frameworks: Rust binary reads JSONL directly (ccusage optional fallback); Python svgwrite for server-side rendering; cal-heatmap for GitHub Pages interactive view_
_Ecosystem Maturity: High — all required building blocks exist as open-source_
_Source: https://ccusage.com/, https://cal-heatmap.com/v2/_

### Database and Storage Technologies

**Local Storage (per machine):**
- Claude Code stores session data as **JSONL files** in `~/.claude/projects/<project-hash>/` — one file per session, each line is a JSON object.
- No built-in export API; data is read directly from the filesystem.
- Storage path is hardcoded; no configuration option.
- _Source: https://milvus.io/blog/why-claude-code-feels-so-stable-a-developers-deep-dive-into-its-local-storage-design.md_

**Central Aggregation Store Options:**
- **GitHub Repository (JSON/CSV files)** — Best fit for this use case. Free, versioned, works seamlessly with GitHub Actions. Recommended for datasets <5MB. Commit aggregated stats JSON after each machine syncs.
- **GitHub Gist** — Simpler but limited; no version history branching, API rate limits, not suitable for concurrent writes from multiple machines.
- **Anthropic Admin API** — `GET /v1/admin/usage_report/claude_code` provides server-side usage data. Requires Admin API key (org-level). May be the cleanest source if user has org access.
  - Source: https://platform.claude.com/docs/en/api/admin/usage_report/retrieve_claude_code

_Recommended: GitHub repo as central JSON data store + optional Anthropic Admin API as authoritative source_
_Source: https://blog.okfn.org/2013/07/02/git-and-github-for-data/_

## Integration Patterns Analysis

> **Architectural Decision (User-Directed):** Rather than a separate cron-scheduled local script, this tool uses Claude Code's built-in **hooks system** as the data collection trigger. Users install the tool once; hooks fire automatically on relevant Claude Code events with no further user action required.

### Core Integration Pattern: Claude Code Hooks → GitHub

**Event-Driven Architecture Overview:**

```
[Machine N]                           [GitHub]
Claude Code session
    │
    ├─ Stop / SessionEnd hook fires
    │     (async: true — non-blocking)
    │
    ├─ Hook script runs:
    │     reads ~/.claude/projects/**/*.jsonl
    │     extracts today's usage delta
    │     pushes JSON to central repo via gh CLI / GitHub API
    │
[Central Repo: vibestats-data]
    │
    └─ GitHub Actions (daily cron or on push)
          aggregates all machine JSON files
          generates heat map SVG
          commits SVG to profile README repo
```

### Hook Events for Data Capture

Claude Code exposes **26 hook events**. The relevant ones for vibestats:

| Hook Event | When it fires | Suitability | Notes |
|---|---|---|---|
| `SessionEnd` | When session terminates | **Not reliable** — do not use as primary | Confirmed broken: issues #6428, #17885, #34954 |
| `Stop` | After every Claude response | **Primary trigger** (ADR-2) | Use `async: true` + checkpoint throttle |
| `SessionStart` with `startup` matcher | On new session launch | **Safety net catch-up** (ADR-2) | Syncs anything missed from previous abrupt close |
| `PostToolUse` with `matcher: "Bash(git commit *)"` | After a git commit runs | Optional enhancement | Captures coding context alongside AI usage |

**Decided hook combination (ADR-2):**
- `Stop` (primary) + checkpoint file — fires after every response, throttled to max once/5 min
- `SessionStart startup` (catch-up) — syncs missed data from previous abruptly-closed sessions

_Source: https://code.claude.com/docs/en/hooks-guide, https://code.claude.com/docs/en/hooks_

### Hook Configuration Format

Installed to `~/.claude/settings.json` (user-global scope, applies to all projects):

```json
{
  "hooks": {
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "~/.vibestats/sync",
            "async": true,
            "timeout": 30
          }
        ]
      }
    ],
    "SessionStart": [
      {
        "matcher": "startup",
        "hooks": [
          {
            "type": "command",
            "command": "~/.vibestats/sync --catchup",
            "async": true
          }
        ]
      }
    ]
  }
}
```

> Note: `~/.vibestats/sync` is the compiled Rust binary (ADR-9), not a shell script. The `--catchup` flag disables the throttle for the SessionStart catch-up run.

The hook script receives context via **stdin as JSON**:
```json
{
  "session_id": "abc123",
  "cwd": "/Users/leo/my-project",
  "hook_event_name": "SessionEnd"
}
```

_Source: https://code.claude.com/docs/en/hooks-guide_

### Data Flow: Rust Binary → GitHub

**`~/.vibestats/sync` (Rust binary, ADR-9) — what runs on each machine after every response:**

The binary is a compiled Rust executable. It reads `~/.claude/projects/**/*.jsonl` directly via `serde_json` (no `ccusage` subprocess), applies throttle logic via the checkpoint file, and pushes to the GitHub Contents API via `reqwest`. No shell, no `jq`, no `curl` subprocess.

```
sync [--catchup]
  ├── reads ~/.vibestats/checkpoint.json (last_synced_at)
  ├── [--catchup]: skip throttle check
  ├── scans ~/.claude/projects/**/*.jsonl for entries after last_synced_at
  ├── if nothing new → exit 0
  ├── GET /repos/.../contents/machines/hostname-date.json → extract SHA if exists
  ├── PUT /repos/.../contents/machines/hostname-date.json (with SHA if update)
  └── updates checkpoint.json
```

**Authentication:** Fine-grained PAT stored in `~/.vibestats/.env` (`chmod 600`), loaded at startup. Scoped to `contents:write` on `vibestats-data` only.

_Source: https://docs.github.com/en/rest/repos/contents, https://github.com/ryoppippi/ccusage_

### Central Aggregation: GitHub Actions

The `vibestats` repo (public) runs a daily GitHub Actions job — cron only, never triggered by machine pushes (ADR-8):

```yaml
name: Generate Heat Map
on:
  schedule:
    - cron: '0 1 * * *'   # Daily at 1 AM UTC — ONLY trigger (ADR-8)
  workflow_dispatch:        # Manual trigger for testing

jobs:
  generate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          repository: ${{ github.repository_owner }}/vibestats-data

      - name: Aggregate + generate SVG
        run: python generate_heatmap.py  # reads machines/*.json → produces heatmap.svg

      - name: Update profile README
        uses: stefanzweifel/git-auto-commit-action@v5
        with:
          commit_message: "stats: update heat map"
          file_pattern: heatmap.svg
```

_Source: https://github.com/stefanzweifel/git-auto-commit-action, https://dev.to/cicirello/generate-a-github-stats-svg-for-your-github-profile-readme-in-github-actions-1iaj_

### README Embedding Pattern

```markdown
<!-- vibestats-heatmap-start -->
![Claude Code Activity](https://raw.githubusercontent.com/USERNAME/vibestats-data/main/heatmap.svg)
<!-- vibestats-heatmap-end -->
```

SVG served via `raw.githubusercontent.com` — no Vercel/server needed. GitHub CDN caches it; updates propagate within minutes.

_Source: https://eugeneyan.com/writing/how-to-update-github-profile-readme-automatically/_

### API Design Patterns

**Internal tool APIs:**
- **No REST API needed** — the hook + GitHub API is the complete integration surface
- GitHub Contents API (`PUT /repos/{owner}/{repo}/contents/{path}`) — idempotent file upsert, handles conflicts via SHA, works from any machine without git clone
- Anthropic Admin API (`GET /v1/admin/usage_report/claude_code`) — alternative/supplementary data source if user has org-level API key; returns server-side verified usage, eliminating need to parse local JSONL

_Source: https://platform.claude.com/docs/en/api/admin/usage_report/retrieve_claude_code_

### Installation & Onboarding Integration Pattern

**Zero-friction install (one command):**
```bash
curl -fsSL https://raw.githubusercontent.com/USERNAME/vibestats/main/install.sh | bash
```

The installer:
1. Detects OS + architecture, downloads the correct pre-compiled Rust binary to `~/.vibestats/sync`
2. Prompts for GitHub PAT
3. Injects hook config into `~/.claude/settings.json` (merging, not overwriting)
4. Runs `~/.vibestats/sync --catchup` immediately to backfill historical data and verify connectivity

**Key constraint:** Hook injection must merge into existing `settings.json`, not overwrite — users may have existing hooks configured.

_Source: https://code.claude.com/docs/en/settings_

### Security Patterns

- GitHub PAT stored in `~/.vibestats/.env` with `chmod 600` — not in code
- Token scope: `contents:write` on `vibestats-data` repo only (fine-grained PAT)
- `vibestats-data` repo can be private — SVG is the only public artifact
- No secrets in hook scripts committed to `.claude/settings.json` — env var reference only
- Hook scripts are `async: true` and cannot block or intercept Claude Code execution

_Source: https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens_

### Edge Case Analysis: Reliability & Historical Backfill

> **User-identified critical edge cases requiring architectural solutions:**
> 1. First install must backfill all existing historical data
> 2. Abrupt terminal close (window close, force quit, crash) must not lose data

#### Edge Case 1: Historical Backfill on First Install

**Problem:** The hook system only captures future sessions. A user may have months of JSONL data already on disk.

**Solution: Installer runs a one-time full sync**

The installer script (run once via `curl | bash`) performs:
1. Read ALL `~/.claude/projects/**/*.jsonl` — no date filter
2. Parse entire history with `ccusage` or direct JSONL parse
3. Push aggregated per-day JSON files to `vibestats-data` repo (one file per date per machine)
4. Write checkpoint file at `~/.vibestats/checkpoint.json` marking the full sync complete

```bash
# install.sh (excerpt)
echo "Syncing historical Claude Code stats..."
ccusage export --json --all \
  | python3 ~/.vibestats/format_for_upload.py \
  | ~/.vibestats/bulk_upload.sh   # batched GitHub API calls

echo '{"last_synced_at": "'$(date -u +%Y-%m-%dT%H:%M:%SZ)'", "mode": "full"}' \
  > ~/.vibestats/checkpoint.json
```

The bulk upload can use the GitHub Contents API in a loop — each day's data is a separate file, so concurrent uploads are safe (no merge conflicts).

_Source: https://github.com/ryoppippi/ccusage, https://docs.github.com/en/rest/repos/contents_

#### Edge Case 2: Abrupt Terminal Close / SessionEnd Unreliability

**Problem:** `SessionEnd` hook is **unreliable** in practice:
- GitHub Issue #6428: SessionEnd doesn't fire with `/clear`, `/exit`, `Ctrl+D`, or terminal window close
- GitHub Issue #17885: SessionEnd doesn't fire on `/exit` command
- GitHub Issue #34954: Feature request for reliable SessionEnd — acknowledges it's a known limitation
- macOS specifically lacks `prctl(PR_SET_PDEATHSIG)` — SIGHUP is not forwarded to child processes on terminal close
- SIGTERM/SIGKILL (force quit) do not trigger SessionEnd

**This means we cannot rely on `SessionEnd` as the primary sync trigger.**

**Solution: Three-layer sync strategy**

| Layer | Hook/Mechanism | Frequency | Purpose |
|---|---|---|---|
| Layer 1 | `Stop` hook (async) + checkpoint | After every response | Primary sync — catches almost all cases |
| Layer 2 | `SessionStart` with `startup` matcher | Next session open | Safety net — catches abrupt-close misses |
| Layer 3 | Installer one-time full sync | Once at install | Historical backfill |

**Layer 1 — `Stop` hook with checkpoint file:**

`Stop` fires after every Claude response — even in sessions that end abruptly, it fires after the last response the user received. Combined with a local checkpoint file, it syncs incrementally and idempotently:

```bash
#!/bin/bash
# ~/.vibestats/sync.sh — called by Stop hook (async: true)

CHECKPOINT=~/.vibestats/checkpoint.json
LAST_SYNCED=$(jq -r '.last_synced_at // "1970-01-01T00:00:00Z"' "$CHECKPOINT" 2>/dev/null)

# Only sync if new JSONL data exists since last sync
NEW_DATA=$(ccusage --json --since "$LAST_SYNCED" 2>/dev/null)
if [ -z "$NEW_DATA" ] || [ "$NEW_DATA" = "[]" ]; then
  exit 0   # Nothing new, skip
fi

# Throttle: don't push more than once per 5 minutes
LAST_PUSH=$(jq -r '.last_push_at // "1970-01-01T00:00:00Z"' "$CHECKPOINT" 2>/dev/null)
NOW=$(date -u +%s)
LAST_PUSH_EPOCH=$(date -d "$LAST_PUSH" +%s 2>/dev/null || echo 0)
if [ $((NOW - LAST_PUSH_EPOCH)) -lt 300 ]; then
  exit 0   # Throttled
fi

# Push delta to GitHub and update checkpoint
~/.vibestats/upload.sh "$NEW_DATA" && \
  jq --arg ts "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
     '.last_synced_at = $ts | .last_push_at = $ts' \
     "$CHECKPOINT" > /tmp/checkpoint.tmp && \
  mv /tmp/checkpoint.tmp "$CHECKPOINT"
```

**Layer 2 — `SessionStart` catch-up:**

```json
{
  "hooks": {
    "SessionStart": [
      {
        "matcher": "startup",
        "hooks": [
          {
            "type": "command",
            "command": "~/.vibestats/catchup.sh",
            "async": true
          }
        ]
      }
    ]
  }
}
```

`catchup.sh` re-runs the same sync logic as `sync.sh` but without the throttle, ensuring any data missed during an abrupt close of the previous session is captured on the next launch.

**Checkpoint file schema:**
```json
{
  "last_synced_at": "2026-04-06T14:30:00Z",
  "last_push_at": "2026-04-06T14:30:00Z",
  "install_mode": "full",
  "machine_id": "macbook-pro-leo",
  "version": "1.0.0"
}
```

**Worst case data loss with this strategy:**
- Terminal killed during an active response (rare): the in-progress response's tokens are missed until next `SessionStart` catch-up
- Estimated max loss: < 1 session of data, recovered on next Claude Code launch

_Source: https://github.com/anthropics/claude-code/issues/6428, https://github.com/anthropics/claude-code/issues/17885, https://github.com/anthropics/claude-code/issues/34954_

---

## Architectural Patterns and Design

### System Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│  MACHINE LAYER (runs on each user machine)                  │
│                                                             │
│  ~/.claude/projects/**/*.jsonl  ←  Claude Code writes this  │
│           │                                                  │
│           ▼                                                  │
│  ~/.vibestats/sync  ←  Stop hook (Rust binary, async)        │
│  ~/.vibestats/sync --catchup  ←  SessionStart hook (launch) │
│  ~/.vibestats/checkpoint.json  ←  last_synced_at cursor     │
│           │                                                  │
│           │  GitHub Contents API (PUT, idempotent)           │
└───────────┼─────────────────────────────────────────────────┘
            │
            ▼
┌─────────────────────────────────────────────────────────────┐
│  DATA LAYER  (private GitHub repo: username/vibestats-data) │
│                                                             │
│  machines/                                                  │
│    ├── macbook-pro-2026-04-06.json                         │
│    ├── work-laptop-2026-04-06.json                         │
│    └── ...one file per machine per day                      │
│                                                             │
│  heatmap.svg  ←  generated by GitHub Actions               │
└─────────────────────────────────────────────────────────────┘
            │
            │  GitHub Actions (daily cron only — ADR-8)
            │  aggregates → renders → commits heatmap.svg
            │
            ▼
┌─────────────────────────────────────────────────────────────┐
│  PRESENTATION LAYER  (public: username/username README)      │
│                                                             │
│  ![Activity](raw.githubusercontent.com/.../heatmap.svg)    │
└─────────────────────────────────────────────────────────────┘
```

### Architecture Decision Records (ADRs)

#### ADR-1: Dedicated Private Data Repo (Decided)

**Decision:** Use a separate private `username/vibestats-data` repo as the central data store.

**Rationale:**
- Raw machine stats (token counts, session counts, project names) should be private
- The generated SVG is the only public artifact — served via `raw.githubusercontent.com`
- Separates concerns: data ingestion vs presentation
- Allows the tool repo (`vibestats`) to be public without exposing user data

**Rejected alternatives:**
- Profile README repo (`username/username`): mixing data + presentation; raw JSON would be public
- GitHub Gist: no branching, concurrent write conflicts, harder to manage per-machine files

_Source: https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/managing-repository-settings/setting-repository-visibility_

#### ADR-2: Three-Layer Sync Strategy (Decided)

**Decision:** Use `Stop` hook (primary) + `SessionStart` catch-up + installer full backfill instead of `SessionEnd`.

**Rationale:**
- `SessionEnd` is documented but unreliable in practice (GitHub issues #6428, #17885, #34954)
- `Stop` fires after every Claude response, including the last response before an abrupt close
- Checkpoint file ensures idempotent, incremental uploads (no duplicates, no gaps)
- `SessionStart startup` hook as catch-up covers the worst case: process killed before any Stop fires
- Installer backfill means day-0 historical data is never lost

**Trade-offs:**
- `Stop` hook fires frequently (every response) — mitigated by throttle (5-min minimum between pushes) and delta check (only push if new data)
- Checkpoint file is a single point of truth per machine — if corrupted, worst case is a re-upload of some data (idempotent, safe)

_Source: https://code.claude.com/docs/en/hooks-guide_

#### ADR-3: ccusage over Anthropic Admin API

**Decision:** Use `ccusage` (local JSONL parsing) as the primary data source, with Anthropic Admin API as optional enhancement.

| Factor | ccusage (local JSONL) | Anthropic Admin API |
|---|---|---|
| Requires org API key | No | Yes (admin-level) |
| Works for all users | Yes | Only org admins |
| Offline capable | Yes | No |
| Data latency | Real-time (local file) | Near real-time |
| Data richness | Full session detail | Aggregated billing data |
| Self-hostable | Yes | Depends on Anthropic uptime |

**Rationale:** ccusage works for every Claude Code user regardless of account type. Admin API is opt-in enhancement for power users who have org-level access and want server-verified data.

_Source: https://github.com/ryoppippi/ccusage, https://platform.claude.com/docs/en/api/admin/usage_report/retrieve_claude_code_

#### ADR-4: Static SVG Committed to Repo vs Dynamic Endpoint

**Decision:** Commit generated SVG to `vibestats-data` repo; embed via raw GitHub URL. No server required.

**Rationale:**
- No Vercel/server to maintain or pay for
- SVG cached by GitHub CDN; fast global delivery
- Updates on GitHub Actions schedule (daily or on push from machine) — acceptable staleness
- `raw.githubusercontent.com` URLs work from private repos (URL is effectively the access token)

**Rejected alternative — Vercel serverless endpoint:**
- Would enable real-time rendering on every README view
- Adds infrastructure dependency, rate limit concerns, cold start latency
- Unnecessary for a stats tool where daily granularity is sufficient

_Source: https://github.com/anuraghazra/github-readme-stats, https://eugeneyan.com/writing/how-to-update-github-profile-readme-automatically/_

#### ADR-5: Python for Aggregation and SVG Generation

**Decision:** Python script in GitHub Actions for data aggregation and SVG generation.

**Rationale:**
- GitHub Actions provides Python runtime out of the box (no Docker needed)
- `svgwrite` or pure f-string SVG generation — no browser runtime needed (vs cal-heatmap which requires Node + headless browser for server-side rendering)
- `json` stdlib handles all aggregation; no heavy dependencies
- Single file script deployable in < 5 seconds in Actions

**SVG generation approach:** Pure Python SVG string construction, outputting a GitHub-contributions-style grid. Each day = one `<rect>` with fill color based on activity intensity. No external rendering dependency.

_Source: https://pypi.org/project/svgwrite/, https://cal-heatmap.com/v2/_

### Data Architecture Patterns

**Per-machine daily file schema** (`machines/hostname-YYYY-MM-DD.json`):
```json
{
  "machine_id": "macbook-pro-leo",
  "hostname": "MacBook-Pro.local",
  "date": "2026-04-06",
  "synced_at": "2026-04-06T22:15:00Z",
  "sessions": 4,
  "total_tokens": 182400,
  "input_tokens": 45200,
  "output_tokens": 137200,
  "cost_usd": 1.24,
  "projects": ["vibestats", "my-api", "scripts"],
  "tool_uses": { "Bash": 42, "Edit": 31, "Read": 28, "Write": 9 }
}
```

**Aggregated heat map data** (computed by GitHub Actions):
- One activity score per calendar day across all machines
- Activity score = `log(1 + total_tokens_day)` — log scale prevents one heavy day dominating
- 52 weeks × 7 days = 364 cells rendered as SVG `<rect>` elements

**Idempotency guarantee:** Each machine file is named `hostname-YYYY-MM-DD.json`. Re-uploading the same day's data overwrites the file with the same name — no duplicates, no append conflicts.

_Source: https://docs.github.com/en/rest/repos/contents_

### Deployment and Operations Architecture

**Repos required:**
| Repo | Visibility | Purpose |
|---|---|---|
| `username/vibestats` | Public | Tool code, Rust sync binary, installer, GitHub Actions workflow |
| `username/vibestats-data` | Private | Machine data files + generated `heatmap.svg` |
| `username/username` | Public | GitHub profile README — embeds SVG URL |

**GitHub Actions in `vibestats-data`:**
```yaml
on:
  schedule:
    - cron: '0 1 * * *'       # daily at 1 AM UTC — ONLY trigger
  workflow_dispatch:           # manual trigger for testing
```

> **ADR-8: Decouple machine sync from SVG generation (Decided)**
> Machine → GitHub (data push) uses GitHub Contents API directly — no Actions trigger.
> Actions runs ONCE per day via cron only, regardless of how many machines sync.
> See ADR-8 below for full rationale.

**Secrets needed (in `vibestats-data` repo):**
- None — the Actions workflow uses `GITHUB_TOKEN` (auto-provided) to commit back to the same repo

**Secrets needed on each user machine:**
- `VIBESTATS_GITHUB_TOKEN` — fine-grained PAT with `contents:write` on `vibestats-data` only
- Stored in `~/.vibestats/.env` with `chmod 600`; loaded directly by the Rust sync binary

**Scalability:** Each machine writes one small JSON file per day (~500 bytes). 10 machines × 365 days = 3,650 files/year ≈ 1.8 MB/year. GitHub repo size is trivially small.

_Source: https://docs.github.com/en/actions/security-guides/automatic-token-authentication_

### UX Architecture: README Placement and Interactivity

#### ADR-6: Custom Marker-Based README Placement (Decided)

**Decision:** Users add HTML comment markers anywhere in their existing README; GitHub Actions replaces content between them.

**Pattern:**
```markdown
<!-- vibestats-start -->
<!-- vibestats-end -->
```

User places these two tags wherever they want the heat map to appear. The daily Actions workflow (cron, ADR-8) also updates the profile README repo by:
1. Checking out `username/username` repo
2. Finding the marker pair with regex
3. Replacing content between markers with the `<img>` embed
4. Auto-committing

```python
# In generate_heatmap.py (GitHub Actions)
import re

marker_re = re.compile(
    r'(<!-- vibestats-start -->).*?(<!-- vibestats-end -->)',
    re.DOTALL
)
replacement = (
    r'\1\n'
    r'[![Claude Code Activity]'
    r'(https://raw.githubusercontent.com/USERNAME/vibestats-data/main/heatmap.svg)]'
    r'(https://USERNAME.github.io/vibestats)\n'
    r'\2'
)
new_readme = marker_re.sub(replacement, readme_content)
```

This is the standard pattern used by `blog-post-workflow`, `github-readme-stats`, and many other auto-updating README tools. Users who don't have an existing README get one created automatically.

_Source: https://eugeneyan.com/writing/how-to-update-github-profile-readme-automatically/, https://github.com/gautamkrishnar/blog-post-workflow_

#### ADR-7: Dual Output — Static SVG in README + Interactive GitHub Pages (Decided)

**Finding:** SVG hover/tooltip interactivity does **NOT** work in GitHub READMEs. This is a hard platform constraint:
- GitHub embeds SVGs via `<img>` tags, which are a security boundary — mouse events do not pass through
- GitHub sanitizes SVGs using DOMPurify — strips all JS, event handlers (`onmouseover`, `onclick`), and `<foreignObject>` tags
- `<object>` and inline SVG are blocked entirely in GitHub Markdown
- `<title>` tooltip elements in SVG do not render inside `<img>` tags
- github-readme-stats (50k stars) itself generates static non-interactive SVGs for the same reason

**Decision:** Two complementary outputs:

| Output | Where | Interactivity | Trigger |
|---|---|---|---|
| `heatmap.svg` | `vibestats-data` repo (raw URL in README) | Static only — no hover | GitHub Actions daily cron (ADR-8) |
| `index.html` | GitHub Pages (`username.github.io/vibestats`) | Full interactive cal-heatmap — hover tooltips, date range toggle, per-project breakdown | Same GitHub Actions run |

**README embed pattern:**
```markdown
<!-- vibestats-start -->
[![Claude Code Activity](https://raw.githubusercontent.com/USERNAME/vibestats-data/main/heatmap.svg)](https://USERNAME.github.io/vibestats)
<!-- vibestats-end -->
```

The SVG itself is a **clickable link** to the interactive GitHub Pages version. Visitors who want to explore daily details click through. This mirrors the pattern used by Shields.io badges (static visual → links to richer context).

**GitHub Pages interactive view includes:**
- cal-heatmap with hover tooltips (exact date, session count, token count, cost)
- Date range toggle (last 30 days / last year / all time)
- Per-machine breakdown
- Per-project breakdown

GitHub Pages is free for public repos and deploys directly from the `vibestats` repo's `gh-pages` branch.

_Source: https://github.com/anthropics/claude-code/issues (DOMPurify sanitization), https://github.com/anuraghazra/github-readme-stats, https://cal-heatmap.com/v2/, https://docs.github.com/en/pages_

#### ADR-8: Decouple Machine Sync from SVG Generation (Decided)

**Problem:** Triggering GitHub Actions on every machine push (`push: paths: ['machines/**']`) would:
- Fire multiple times per day across all machines
- Each run ~2 min → 10 machines × 3 syncs/day × 2 min × 30 days = **1,800 min/month** — nearly exhausts the 2,000 min free tier for private repos

**Decision:** Split into two fully independent operations:

| Operation | Mechanism | Actions involved? | Frequency |
|---|---|---|---|
| Machine → `vibestats-data` | GitHub Contents API `PUT` (direct HTTP call from hook script) | **No** | Every sync (multiple/day) |
| Aggregate + render SVG | GitHub Actions cron | **Yes — once per day** | `0 1 * * *` |
| GitHub Pages interactive view | Client-side JS fetching GitHub API | **No** | Always live on page load |

**Revised data flow:**
```
[Machine hook script]
  └─ curl PUT → GitHub Contents API
       └─ machines/hostname-YYYY-MM-DD.json written directly
            (no Actions trigger)

[GitHub Actions — once daily at 1 AM UTC]
  └─ reads all machines/*.json
  └─ aggregates → generates heatmap.svg
  └─ commits heatmap.svg → profile README updated

[GitHub Pages index.html — client-side only]
  └─ fetch('https://api.github.com/repos/.../contents/machines')
  └─ renders cal-heatmap with live data in browser
  └─ always fresh, zero Actions cost
```

**Actions budget with this design:**
- 1 run/day × ~2 min = **60 min/month** — well within free tier
- Moving the workflow to the public `vibestats` repo (tool code) gives **unlimited free minutes** entirely

**GitHub Pages as live client-side view:**
The `index.html` in `vibestats` repo (`gh-pages` branch) uses the GitHub Contents API to fetch all `machines/*.json` files on page load — no build step, no Actions, always reflects the latest push from any machine. cal-heatmap renders interactively in the browser.

```html
<!-- index.html — fully client-side, no server -->
<script>
  fetch('https://api.github.com/repos/USERNAME/vibestats-data/contents/machines',
        { headers: { Authorization: 'token VIBESTATS_READ_TOKEN' }})
    .then(r => r.json())
    .then(files => Promise.all(files.map(f => fetch(f.download_url).then(r => r.json()))))
    .then(data => renderCalHeatmap(data));
</script>
```

_Source: https://docs.github.com/en/rest/repos/contents, https://docs.github.com/en/pages, https://cal-heatmap.com/v2/_

---

## Implementation Approaches and Technology Adoption

### Rust Sync Binary: `sync` (per-machine data push, ADR-9)

The machine-side component is a compiled Rust binary at `~/.vibestats/sync`. The shell script below is the **conceptual reference** for the logic — the actual implementation is Rust using `serde_json` for JSONL parsing and `reqwest` for HTTP. See ADR-9 for rationale.

**Logic reference (implemented in Rust, shown as pseudocode):**

```bash
# REFERENCE ONLY — actual implementation is Rust binary
#!/usr/bin/env bash
# Logic this Rust binary implements:

VIBESTATS_DIR="$HOME/.vibestats"
CHECKPOINT="$VIBESTATS_DIR/checkpoint.json"
ENV_FILE="$VIBESTATS_DIR/.env"

# Load credentials
[ -f "$ENV_FILE" ] && source "$ENV_FILE"
: "${VIBESTATS_GITHUB_TOKEN:?VIBESTATS_GITHUB_TOKEN not set}"
: "${VIBESTATS_REPO_OWNER:?VIBESTATS_REPO_OWNER not set}"

# Read last sync time from checkpoint
LAST_SYNCED=$(jq -r '.last_synced_at // "1970-01-01T00:00:00Z"' "$CHECKPOINT" 2>/dev/null || echo "1970-01-01T00:00:00Z")

# Throttle: skip if pushed within last 5 minutes
LAST_PUSH=$(jq -r '.last_push_at // "1970-01-01T00:00:00Z"' "$CHECKPOINT" 2>/dev/null || echo "1970-01-01T00:00:00Z")
NOW_EPOCH=$(date -u +%s)
LAST_PUSH_EPOCH=$(date -j -f "%Y-%m-%dT%H:%M:%SZ" "$LAST_PUSH" +%s 2>/dev/null \
  || date -d "$LAST_PUSH" +%s 2>/dev/null || echo 0)
[ $((NOW_EPOCH - LAST_PUSH_EPOCH)) -lt 300 ] && exit 0

# Gather today's stats via ccusage
TODAY=$(date -u +%Y-%m-%d)
PAYLOAD=$(ccusage --json --since "$LAST_SYNCED" 2>/dev/null) || exit 0
[ -z "$PAYLOAD" ] || [ "$PAYLOAD" = "[]" ] && exit 0

# Build machine file content
HOSTNAME_CLEAN=$(hostname | tr '.' '-' | tr '[:upper:]' '[:lower:]')
FILE_PATH="machines/${HOSTNAME_CLEAN}-${TODAY}.json"
CONTENT=$(echo "$PAYLOAD" | jq -c --arg h "$HOSTNAME_CLEAN" --arg d "$TODAY" \
  '{machine_id: $h, date: $d, synced_at: now|todate, data: .}')

# Cross-platform base64 (macOS + Linux)
encode_b64() { openssl base64 -A <<< "$1" 2>/dev/null || base64 -w 0 <<< "$1"; }
ENCODED=$(encode_b64 "$CONTENT")

# Get existing file SHA (needed for update, empty for create)
API_URL="https://api.github.com/repos/${VIBESTATS_REPO_OWNER}/vibestats-data/contents/${FILE_PATH}"
EXISTING_SHA=$(curl -sf -H "Authorization: Bearer $VIBESTATS_GITHUB_TOKEN" "$API_URL" \
  | jq -r '.sha // empty' 2>/dev/null || true)

# Build PUT payload
PUT_BODY=$(jq -n \
  --arg msg "stats: ${HOSTNAME_CLEAN} ${TODAY}" \
  --arg content "$ENCODED" \
  --arg sha "$EXISTING_SHA" \
  'if $sha != "" then {message: $msg, content: $content, sha: $sha}
   else {message: $msg, content: $content} end')

# Push to GitHub Contents API
curl -sf -X PUT \
  -H "Authorization: Bearer $VIBESTATS_GITHUB_TOKEN" \
  -H "Content-Type: application/json" \
  "$API_URL" -d "$PUT_BODY" > /dev/null

# Update checkpoint
TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)
jq --arg ts "$TIMESTAMP" \
   '.last_synced_at = $ts | .last_push_at = $ts' \
   "$CHECKPOINT" > "${CHECKPOINT}.tmp" && mv "${CHECKPOINT}.tmp" "$CHECKPOINT"
```

**Key implementation notes:**
- Different machines write to different file paths (`machines/hostname-date.json`) — parallel-safe, no locking needed
- SHA fetch + conditional inclusion handles both create (new day) and update (same day, multiple syncs) in one code path
- `openssl base64 -A` is the cross-platform portable solution (macOS `base64` doesn't support `-w 0`)

_Source: https://docs.github.com/en/rest/repos/contents, https://osxhub.com/macos-base64-encoding-returns-wrong-result-complete-fix-guide-for-sequoia-2025/_

### Hook Injection into settings.json

**Safe jq-based merge (appends to existing hooks, never overwrites):**

```bash
inject_hooks() {
  local settings="$HOME/.claude/settings.json"
  local new_hooks
  new_hooks=$(cat <<'EOF'
{
  "hooks": {
    "Stop": [{"hooks": [{"type": "command", "command": "~/.vibestats/sync.sh", "async": true, "timeout": 30}]}],
    "SessionStart": [{"matcher": "startup", "hooks": [{"type": "command", "command": "~/.vibestats/catchup.sh", "async": true}]}]
  }
}
EOF
)

  # Read existing (or empty object if missing)
  local existing="{}"
  [ -f "$settings" ] && existing=$(cat "$settings")

  # Atomic write: deep merge with array concatenation for hooks
  local result
  result=$(jq -s '
    .[0] as $old | .[1] as $new |
    ($old | to_entries) + ($new | to_entries) |
    group_by(.key) |
    map(
      if length == 1 then .[0]
      elif .[0].value | type == "object" then
        {key: .[0].key, value: (.[0].value * .[1].value)}
      elif .[0].value | type == "array" then
        {key: .[0].key, value: (.[0].value + .[1].value)}
      else .[1] end
    ) | from_entries
  ' <(echo "$existing") <(echo "$new_hooks"))

  # Backup existing and write atomically
  [ -f "$settings" ] && cp "$settings" "${settings}.vibestats.bak"
  echo "$result" | jq . > "${settings}.tmp" && mv "${settings}.tmp" "$settings"
}
```

_Source: https://richrose.dev/posts/linux/jq/jq-jsonmerge/, https://code.claude.com/docs/en/settings_

### SVG Heat Map Generation (GitHub Actions, Python)

**Exact GitHub contribution colors and grid generation:**

```python
#!/usr/bin/env python3
# generate_heatmap.py — runs in GitHub Actions daily
import json, calendar, math
from pathlib import Path
from datetime import date, timedelta

# GitHub light mode contribution colors (5 levels)
COLORS = ['#ebedf0', '#9be9a8', '#30c463', '#30a14e', '#216e39']

def activity_to_level(count, max_count):
    if count == 0 or max_count == 0:
        return 0
    # Log scale prevents one heavy day dominating
    log_count = math.log1p(count)
    log_max = math.log1p(max_count)
    return min(4, int((log_count / log_max) * 4) + 1)

def generate_svg(daily_totals: dict[str, int]) -> str:
    """daily_totals: {'2026-04-06': 182400, ...} (tokens per day)"""
    today = date.today()
    start = today - timedelta(weeks=52)
    # Align to Monday
    start -= timedelta(days=start.weekday())

    max_val = max(daily_totals.values(), default=1)
    cell, pad = 11, 2
    step = cell + pad
    weeks = 53

    rects = []
    for week in range(weeks):
        for day in range(7):
            d = start + timedelta(weeks=week, days=day)
            if d > today:
                continue
            ds = d.isoformat()
            count = daily_totals.get(ds, 0)
            level = activity_to_level(count, max_val)
            color = COLORS[level]
            x = week * step + 20
            y = day * step + 20
            label = f"{ds}: {count:,} tokens" if count else ds
            rects.append(
                f'<rect x="{x}" y="{y}" width="{cell}" height="{cell}" '
                f'fill="{color}" rx="2"><title>{label}</title></rect>'
            )

    w = weeks * step + 20
    h = 7 * step + 20
    return (
        f'<svg width="{w}" height="{h}" xmlns="http://www.w3.org/2000/svg">'
        f'<rect width="{w}" height="{h}" fill="#0d1117" rx="6"/>'
        + ''.join(rects)
        + '</svg>'
    )

# Aggregate all machines/*.json
daily_totals: dict[str, int] = {}
for f in Path('machines').glob('*.json'):
    data = json.loads(f.read_text())
    day = data.get('date')
    tokens = data.get('data', {})
    if isinstance(tokens, list):
        total = sum(e.get('total_tokens', 0) for e in tokens)
    else:
        total = tokens.get('total_tokens', 0)
    if day:
        daily_totals[day] = daily_totals.get(day, 0) + total

svg = generate_svg(daily_totals)
Path('heatmap.svg').write_text(svg)
print(f"Generated heatmap.svg — {len(daily_totals)} active days")
```

**Notes:**
- Log scale (`math.log1p`) prevents a single heavy day washing out the rest — same approach GitHub uses
- `<title>` elements in the SVG provide hover tooltips when viewed directly in browser (GitHub Pages), even though they're inactive in README `<img>` context
- Dark background (`#0d1117`) matches GitHub dark mode — looks native on profile pages

_Source: https://github.com/orgs/community/discussions/176081 (exact hex values verified), https://towardsdatascience.com/create-githubs-style-contributions-plot-for-your-time-series-data-79df84ec93da/_

### Installer Script Pattern

```bash
#!/usr/bin/env bash
# install.sh — curl -fsSL .../install.sh | bash
set -euo pipefail

VIBESTATS_DIR="$HOME/.vibestats"
CLAUDE_SETTINGS="$HOME/.claude/settings.json"

echo "Installing vibestats..."

# 1. Check required dependencies
for dep in curl jq openssl; do
  if ! command -v "$dep" &>/dev/null; then
    echo "Error: '$dep' is required but not installed." >&2; exit 1
  fi
done

# Check for ccusage (optional but recommended)
if ! command -v ccusage &>/dev/null; then
  echo "Note: 'ccusage' not found. Install via: npm install -g ccusage"
  echo "Without it, vibestats will parse JSONL directly (slower)."
fi

# 2. Prompt for GitHub credentials (silent input)
echo ""
echo "Create a fine-grained PAT at: https://github.com/settings/personal-access-tokens"
echo "Required permission: Contents (read & write) on your vibestats-data repo"
read -sp "GitHub PAT: " VIBESTATS_GITHUB_TOKEN; echo
read -rp "Your GitHub username: " VIBESTATS_REPO_OWNER

# 3. Create vibestats directory and scripts
mkdir -p "$VIBESTATS_DIR"
chmod 700 "$VIBESTATS_DIR"

# Write credentials (chmod 600 — owner read/write only)
cat > "$VIBESTATS_DIR/.env" <<EOF
VIBESTATS_GITHUB_TOKEN=$VIBESTATS_GITHUB_TOKEN
VIBESTATS_REPO_OWNER=$VIBESTATS_REPO_OWNER
EOF
chmod 600 "$VIBESTATS_DIR/.env"

# Detect arch and download correct Rust binary from GitHub Releases
ARCH=$(uname -m)   # arm64 or x86_64
OS=$(uname -s)     # Darwin or Linux
TARGET="${ARCH}-${OS}"
curl -fsSL "https://github.com/vibestats/vibestats/releases/latest/download/vibestats-sync-${TARGET}" \
  -o "$VIBESTATS_DIR/sync"
chmod +x "$VIBESTATS_DIR/sync"

# 4. Initialize checkpoint
echo '{"last_synced_at":"1970-01-01T00:00:00Z","last_push_at":"1970-01-01T00:00:00Z","install_mode":"full"}' \
  > "$VIBESTATS_DIR/checkpoint.json"

# 5. Inject hooks into ~/.claude/settings.json (safe merge)
# ... (inject_hooks function as defined above)
inject_hooks

# 6. Run full historical backfill immediately via Rust binary
echo "Running historical backfill (this may take a moment)..."
"$VIBESTATS_DIR/sync" --catchup --full-history

echo ""
echo "✓ vibestats installed. Hooks active for all future Claude Code sessions."
echo "  README marker: add these tags to your github.com/USERNAME/USERNAME README:"
echo ""
echo "  <!-- vibestats-start -->"
echo "  <!-- vibestats-end -->"
```

**Cross-shell PATH note:** vibestats scripts live in `~/.vibestats/` and are called by absolute path from hooks — no PATH modification needed.

_Source: https://emmer.dev/blog/reliably-detecting-command-existence-in-bash/, https://jvns.ca/blog/2025/02/13/how-to-add-a-directory-to-your-path/_

### Testing and Quality Assurance

**Testing strategy for this tool:**
- **Unit tests** (Python): SVG generation with known inputs, JSON aggregation logic, color level mapping
- **Integration tests** (bash): hook script against a test repo using a separate `vibestats-data-test` repo
- **Idempotency test**: Run sync twice with same data — verify no duplicate commits, no duplicate JSON entries
- **Edge cases to test**: Empty JSONL dir (new machine), missing `ccusage`, corrupted checkpoint file, GitHub API 422 (SHA mismatch on concurrent push from same machine)

### Risk Assessment and Mitigation

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| GitHub API rate limit (5,000 req/hr) | Low | Medium | Each sync = 2 API calls (GET SHA + PUT). 10 machines × 10 syncs/day = 200 calls — well within limit |
| ccusage not installed | Medium | Low | Installer warns; sync.sh falls back to direct JSONL parsing |
| Checkpoint corruption | Low | Low | Worst case: re-uploads existing data (idempotent) |
| `SessionEnd` never fires | High (known bug) | None | Mitigated by `Stop` hook as primary + `SessionStart` catch-up |
| Fine-grained PAT token expiry | Medium | Medium | Installer warns about expiry; sync.sh prints actionable error to stderr |
| `settings.json` merge collision | Low | Medium | Atomic write + backup before modify; jq validation before mv |

### Implementation Roadmap

**Phase 1 — Core (MVP):**
1. `sync.sh` + `catchup.sh` hook scripts
2. `install.sh` installer with hook injection
3. GitHub Actions workflow (daily cron, Python SVG)
4. README marker replacement in Actions

**Phase 2 — Interactive view:**
5. GitHub Pages `index.html` with cal-heatmap + GitHub API client
6. Link README SVG to GitHub Pages URL

**Phase 3 — Polish:**
7. Light/dark mode SVG variants
8. Per-project breakdown in GitHub Pages view
9. Anthropic Admin API opt-in for verified token counts

#### ADR-9: Rust Binary for Machine-Side Sync, Python for Server-Side (Decided)

**Decision:** The `vibestats-sync` component that runs on user machines is a compiled Rust binary. The aggregation + SVG generation that runs in GitHub Actions is Python.

**Rationale — Rust on user machines:**
- Eliminates all runtime dependencies (`jq`, `curl`, `ccusage`, `openssl`, Bash 4+) — single static binary
- macOS ships Bash 3.2 (2007) which breaks many modern shell features
- Parses `~/.claude/projects/**/*.jsonl` directly via `serde_json` — no `ccusage` subprocess
- Native HTTP client (`reqwest` async) — no curl subprocess
- Typed error handling with `Result<>` — no silent failures from exit codes
- `async: true` hook means startup speed (~5ms vs ~300ms) doesn't affect user experience, but eliminates the Node.js cold-start cost of ccusage

**Rationale — Python in GitHub Actions:**
- Python pre-installed on all GitHub Actions runners — no build step
- Controlled environment (runner OS is fixed) — no portability concerns
- Same language as SVG generation — one runtime for both tasks
- stdlib only (`json`, `pathlib`, `math`, `datetime`) — zero pip installs needed

**Binary distribution:**
```yaml
# .github/workflows/release.yml — cross-compile for all targets
strategy:
  matrix:
    target:
      - aarch64-apple-darwin      # Apple Silicon Mac
      - x86_64-apple-darwin       # Intel Mac
      - x86_64-unknown-linux-gnu  # Linux
```

`cross-rs` handles cross-compilation. Binaries attached to GitHub Releases. Installer detects architecture and downloads the correct binary:
```bash
ARCH=$(uname -m)  # arm64 or x86_64
OS=$(uname -s)    # Darwin or Linux
curl -fsSL "https://github.com/USERNAME/vibestats/releases/latest/download/vibestats-sync-${ARCH}-${OS}" \
  -o ~/.vibestats/sync
chmod +x ~/.vibestats/sync
```

**Rust crate dependencies (minimal):**
- `serde` + `serde_json` — JSONL parsing
- `reqwest` (blocking or async-lite) — GitHub API calls
- `chrono` — timestamp handling
- No async runtime needed (single-threaded background task is fine)

_Source: https://github.com/cross-rs/cross, https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html_

### Development Tools and Platforms

- **GitHub Actions** — Standard automation platform for scheduled README updates. Cron triggers (`0 0 * * *` for daily). Free for public repos.
- **Vercel (optional)** — Used by `github-readme-stats` for serverless SVG serving. Only needed if serving dynamic SVG via HTTP rather than committing static files.
- **Git + git-auto-commit-action** — For committing generated SVG/JSON back to the profile repo.
- **Version Control Pattern**: Separate `data/` branch or directory in the profile repo for raw stats JSON; `main` branch serves the rendered SVG embedded in README.
- _Source: https://dev.to/cicirello/generate-a-github-stats-svg-for-your-github-profile-readme-in-github-actions-1iaj_

### Cloud Infrastructure and Deployment

**Recommended Architecture — No External Cloud Required:**
- All computation runs inside **GitHub Actions** (free tier, public repos)
- Data stored in **GitHub repository** (no S3, no database)
- SVG committed to the same profile repo and embedded via relative path or raw GitHub URL
- Machines push data via **GitHub Actions triggers** (workflow_dispatch + PAT/deploy key)

**Authentication across machines:**
- Each machine generates a script that reads local JSONL, formats as JSON, then triggers a GitHub Actions workflow (via `gh workflow run` or `curl` to GitHub API) passing the data payload.
- The Actions runner then merges, aggregates, regenerates SVG, and commits.
- _Source: https://docs.github.com/en/authentication/connecting-to-github-with-ssh/managing-deploy-keys_

### Technology Adoption Trends

- GitHub profile README tooling is a well-established pattern (2020–present); `github-readme-stats` has 50k+ stars.
- JSONL as a local data store for AI tools is becoming a standard (Claude Code, OpenAI CLI tools follow this pattern).
- SVG-in-README is the dominant approach for dynamic content (avoids CSP issues with JavaScript).
- GitHub Actions cron jobs are replacing dedicated servers for lightweight scheduled tasks.
- The Anthropic Admin API for usage reporting is new (2025) and may be the most reliable cross-machine data source if org-level access is available.
- _Source: https://github.com/anuraghazra/github-readme-stats, https://platform.claude.com/docs/en/api/admin/usage_report/retrieve_claude_code_
