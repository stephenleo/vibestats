---
stepsCompleted: [1, 2, 3, 4, 5, 6, 7, 8]
lastStep: 8
status: 'complete'
completedAt: '2026-04-10'
inputDocuments:
  - '_bmad-output/planning-artifacts/prd.md'
  - '_bmad-output/planning-artifacts/research/technical-claude-code-stats-heatmap-research-2026-04-06.md'
workflowType: 'architecture'
project_name: 'vibestats'
user_name: 'Leo'
date: '2026-04-10'
---

# Architecture Decision Document

_This document builds collaboratively through step-by-step discovery. Sections are appended as we work through each architectural decision together._

## Project Context Analysis

### Requirements Overview

**Functional Requirements:**
43 FRs across 7 categories: installer (FR1–11), data collection/sync (FR12–19), aggregation/output
(FR20–26), profile display (FR27–31), CLI/machine management (FR32–37), authentication (FR38–40),
distribution/documentation (FR41–43). The installer and sync categories carry the most architectural
weight — they define the machine-side component contract. Aggregation and output define the Actions
pipeline contract. Profile display defines the public-facing data contract.

**Non-Functional Requirements:**
17 NFRs across performance (NFR1–5), security (NFR6–9), reliability (NFR10–13), and integration
(NFR14–17). The most architecturally constraining are:
- NFR1 (2s hook latency cap) — forces async execution and minimal in-band work
- NFR2 (5-min throttle) — requires a checkpoint mechanism persisted to disk
- NFR3 (60s backfill of 12 months) — sets throughput floor for the JSONL parser + API push path
- NFR6 (600 file permissions on config) — must be enforced at write time, not as documentation
- NFR8/NFR9 (private/public data boundary) — enforced at the Actions layer; raw JSON never leaves vibestats-data
- NFR12 (idempotent sync) — daily push to vibestats-data must be a PUT/upsert, not accumulating appends
- NFR14 (JSONL schema tolerance) — parser must use optional field access, not hard-typed structs

**Scale & Complexity:**
- Primary domain: CLI tool + automation pipeline + static web
- Complexity level: Low–Medium (bounded scope, no multi-tenancy, async by design)
- Estimated architectural components: 5 (Rust binary, Bash installer, Python Actions script, Astro static site, GitHub Actions workflow YAML)

### Technical Constraints & Dependencies

- **Rust binary**: zero runtime deps; all GitHub API calls are HTTP inside the binary (reqwest or similar); checkpoint file for throttle state; config at `~/.config/vibestats/config.toml` (600 perms)
- **Python Actions**: stdlib only — no pip installs; `json`, `datetime`, `pathlib`, string formatting
- **Bash installer**: must work on macOS (Bash 3.2) and Linux; relies on `gh` CLI for all GitHub operations
- **`gh` CLI**: required dependency; minimum version 2.0 (NFR16); installer installs if missing
- **GitHub Contents API**: rate limit 5,000 req/hr; binary implements exponential backoff on 429 (NFR15)
- **GitHub Actions free tier**: ~2,000 min/month; daily cron only (~60 min/month) — per-push triggers are excluded by design (ADR-8)
- **SVG in README**: GitHub DOMPurify strips interactivity; static SVG only for README embed; cal-heatmap used on vibestats.dev dashboard (ADR-7)

### Cross-Cutting Concerns Identified

1. **Silent failure contract**: hook errors must never propagate to Claude Code (NFR10); sync failures must be silent during sessions, surfaced only at next SessionStart (NFR11) — this contract must be upheld across both the Stop hook and SessionStart hook code paths
2. **Idempotency**: both machine-side push (daily JSON PUT) and Actions aggregation must be idempotent — running twice produces identical output (NFR12/NFR13)
3. **Auth token lifecycle**: machine-side token and VIBESTATS_TOKEN are independent; either can expire separately; `vibestats status` and `vibestats auth` must handle partial auth failure states
4. **JSONL schema tolerance**: parser must handle unknown fields and missing optional fields across Claude Code versions without panicking (NFR14)
5. **GitHub API rate limiting**: exponential backoff required at the binary level; Actions also subject to limits but less frequently called (NFR15)
6. **Data boundary enforcement**: the Actions layer is the sole gatekeeper ensuring raw machine JSON never reaches the public repo — this must be explicit in the aggregation script design (NFR8/NFR9)

## Starter Template Evaluation

### Primary Technology Domain

Multi-surface system — each component has its own appropriate toolchain. There is no single
"project starter"; initialization is per-component.

### Starter Options Considered

This project's pre-decided language matrix (from PRD/research ADRs) eliminates most starter
template decisions. Each surface uses the canonical bootstrapper for its language.

### Selected Starters by Component

#### 1. Rust Sync Binary

**Initialization Command:**
```bash
cargo new vibestats --bin
```

**Key crate decisions:**
- `clap` — CLI argument parsing (`status`, `sync`, `machines`, `auth`, `uninstall` subcommands)
- `serde` + `serde_json` — JSONL parsing with `#[serde(default)]` for schema tolerance (NFR14)
- `ureq` — minimal HTTP client for GitHub Contents API calls (smaller than `reqwest`, no async runtime needed since hook runs async at the Claude Code level)
- `toml` — config file read/write (`~/.config/vibestats/config.toml`)
- No async runtime (tokio) — sync HTTP is sufficient; async is handled by Claude Code's `async: true` hook flag

#### 2. Python GitHub Actions Script

**Initialization:** Plain Python files — no starter.

**Structure decision:** Two scripts in `action/`:
- `aggregate.py` — reads all machine JSON from `vibestats-data`, merges into daily totals
- `generate_svg.py` — renders the heatmap SVG from aggregated data

stdlib only: `json`, `datetime`, `pathlib`, `collections`, `xml.etree.ElementTree` (for SVG output).

#### 3. Bash Installer

**Initialization:** Hand-written `install.sh` — no starter.

Structured after `rustup` installer pattern: detect OS/arch, download correct binary from GitHub
Releases, verify, install to `~/.local/bin/vibestats`, configure `gh` auth, create repos, set
secrets, write hooks, trigger backfill.

#### 4. Astro Documentation + Dashboard Site

**Initialization Command:**
```bash
npm create astro@latest vibestats-site -- --template minimal --typescript strict --no-install
```

**Architectural decisions provided:**
- Static site generation (SSG) — no server required
- File-based routing: `src/pages/index.astro` (docs), `src/pages/[username].astro` (per-user dashboard)
- `[username].astro` fetches `data.json` client-side from user's public GitHub repo — no per-user hosting, no build-time data dependency
- `cal-heatmap` bundled via npm into Astro build — served from Cloudflare Pages, exact version pinned in `package.json`, no CDN runtime dependency

**Note:** Project initialization using these commands should be the first implementation stories per component.

## Core Architectural Decisions

### Decision Priority Analysis

**Critical Decisions (Block Implementation):**
- Machine JSON schema and Hive partition file layout in vibestats-data
- Public data.json schema (defines the contract between Actions and dashboard)
- GitHub Contents API update pattern (GET SHA + conditional PUT)
- Auth token validation strategy (validate-on-failure, not validate-on-every-sync)
- VIBESTATS_TOKEN generation via programmatic fine-grained PAT

**Important Decisions (Shape Architecture):**
- One-file-per-day Hive partition layout (date-first for query optimization)
- Merge-not-replace sync pattern with content-hash idempotency guard
- SessionStart catch-up writes all dates from last_sync_date to yesterday
- cal-heatmap bundled into Astro build (not CDN-loaded)
- Year toggle derived client-side from data.json — no schema changes
- Cloudflare Pages free tier for vibestats.dev; GitHub Actions (manual dispatch) for deploy CI/CD
- Cross crate for Rust multi-target CI builds

**Deferred Decisions (Post-MVP):**
- Token count / cost tracking fields in JSON schema (Growth phase)
- Per-project breakdown in data.json (Growth phase)
- Team aggregate views (Vision phase)

### Data Architecture

**Hive partition file layout** (`vibestats-data`):
```
machines/year=2026/month=04/day=10/harness=claude/machine_id=stephens-mbp-a1b2c3/data.json
```
- Date partitions are outermost — optimises dominant query pattern ("all activity for date range X")
- Harness directory (`harness=claude`) enables future multi-tool support (Codex, Cursor, Copilot) with zero schema changes — Actions aggregator globs `harness=*` automatically
- Partition columns encoded in path, not file — enables Athena/BigQuery external tables with no transformation
- Leaf `data.json`: `{ "sessions": 4, "active_minutes": 87 }` (minimal; partition metadata in path)
- One file per machine per day — each push is an independent overwrite, no merge of historical data required

**Public aggregated schema** (`username/username/vibestats/data.json`):
```json
{
  "generated_at": "<ISO 8601 UTC>",
  "username": "<github username>",
  "days": {
    "YYYY-MM-DD": { "sessions": N, "active_minutes": N }
  }
}
```
- Full history all years in single file (~73KB for 5 years — single fetch, filter client-side)
- Aggregated totals only — no machine IDs, hostnames, file paths (NFR8/NFR9)

**Local checkpoint** (`~/.config/vibestats/`):
- `config.toml` — OAuth token, machine ID, vibestats-data repo path (permissions: `600`)
- `checkpoint.toml` — throttle timestamp + per-date content hash (skip PUT if data unchanged)
- `vibestats.log` — rolling error log, max 1MB, append-only

### Authentication & Security

**VIBESTATS_TOKEN generation (first install):**
```bash
VIBESTATS_TOKEN=$(gh api /user/personal_access_tokens \
  --method POST \
  --field name="vibestats-$(date +%Y)" \
  --field expiration="never" \
  --field repositories='["username"]' \
  --field permissions='{"contents":"write"}' \
  --jq '.token')
gh secret set VIBESTATS_TOKEN --repo username/vibestats-data --body "$VIBESTATS_TOKEN"
```
Token piped directly to `gh secret set` — never written to disk. Scoped exclusively to `username/username` Contents write (NFR7).
Fallback: if fine-grained PAT API blocked (enterprise restriction), fall back to `gh auth token` with printed warning.

**Two independent tokens:**

| Token | How created | Scope | Lives |
|---|---|---|---|
| Machine-side | `gh auth token` | `vibestats-data` Contents write | `~/.config/vibestats/config.toml` (600) |
| `VIBESTATS_TOKEN` | `gh api /user/personal_access_tokens` | `username/username` Contents write only | Actions secret in `vibestats-data` |

**Validation strategy:** Validate-on-failure — 401 response sets `auth_error = true` in `checkpoint.toml`; `SessionStart` reads flag and prints warning with `vibestats auth` fix instruction (NFR11, silent during sessions).

**`vibestats auth` refresh:** updates `config.toml` + re-sets `VIBESTATS_TOKEN` Actions secret.

### API & Communication Patterns

**Sync operation (all types — Stop hook, SessionStart, explicit sync):**
1. Check throttle timestamp in `checkpoint.toml` — skip entirely if < 5 min (NFR2, Stop hook only)
2. Parse JSONL for relevant date range → compute `{ "sessions": N, "active_minutes": N }`
3. Hash the computed payload
4. Compare hash to cached hash in `checkpoint.toml` for that date
5. If unchanged: skip — no API call, no git commit
6. If changed: GET day file SHA from GitHub → PUT new content → update checkpoint hash

**Date range per operation:**
- `Stop` hook: today only
- `SessionStart` catch-up: `last_sync_date` (from remote file's `last_updated`) → yesterday
- `vibestats sync`: today only (same as Stop hook, unthrottled)
- `vibestats sync --backfill`: all dates present in JSONL history

**Idempotency — three levels:**
| Level | Mechanism |
|---|---|
| Data | Same JSONL → same computed payload |
| API | Content hash check before GET+PUT — skips if unchanged |
| Git | No PUT issued → no commit created in vibestats-data |

**Error handling (all paths exit 0 — never propagate to Claude Code):**
| HTTP Status | Action |
|---|---|
| 401 | Set `auth_error = true` in checkpoint, exit 0 |
| 404 on GET | First push for this date — PUT without SHA |
| 429 / 5xx | Exponential backoff: 1s → 2s → 4s, max 3 retries, log, exit 0 |
| Network timeout | Log to `vibestats.log`, exit 0 |

### Frontend Architecture

**`vibestats.dev/[username]` data flow:**
- Static Astro shell served from Cloudflare Pages
- Client-side `fetch("https://raw.githubusercontent.com/{username}/{username}/main/vibestats/data.json")` at runtime
- On success: render cal-heatmap with full history data
- On failure: render "No vibestats data found for @{username}" — no server-side handling
- Library and data fetch are fully independent — bundling cal-heatmap does not affect data source

**cal-heatmap:** bundled via `npm install cal-heatmap` into Astro build — served from Cloudflare Pages, version pinned in `package.json`, no third-party CDN runtime dependency

**Year toggle:**
- Derived from `Object.keys(data.days)` — extract unique years client-side
- Render year buttons descending, current year selected by default
- On toggle: filter `days` map by selected year, re-render cal-heatmap in place
- No additional fetches, no schema changes required

**Astro routing:**
- `src/pages/index.astro` — docs/landing (SSG)
- `src/pages/[username].astro` — dashboard shell (SSG + client-side fetch)
- `src/pages/docs/[...slug].astro` — documentation pages

### Infrastructure & Deployment

**Rust binary CI/CD:**
- Triggered on `git tag v*`
- Matrix build using `cross` crate for cross-compilation
- Targets: `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`
- Produces `.tar.gz` archives attached to GitHub Release
- Install script selects correct archive via `uname -s` + `uname -m`

**Community GitHub Action:**
- Type: composite (not Docker) — faster startup, no image pull
- `action.yml` inputs: `token` (VIBESTATS_TOKEN), `profile-repo` (username/username)
- Steps: checkout vibestats-data → `python action/aggregate.py` → `python action/generate_svg.py` → commit outputs to profile-repo via VIBESTATS_TOKEN

**`vibestats.dev` hosting:**
- Cloudflare Pages free tier (unlimited bandwidth/requests, 500 builds/month, automatic SSL)
- Direct GitHub integration for automatic builds on push to `main`
- Deploy CI/CD: separate GitHub Actions workflow, manually triggered via `workflow_dispatch` with branch/tag input — user controls which version gets pushed to production
- Requires `CLOUDFLARE_API_TOKEN` + `CLOUDFLARE_ACCOUNT_ID` as repo secrets

**Custom domain:**
- `vibestats.dev` registered on Namecheap
- DNS managed via Cloudflare (nameservers updated in Namecheap → Cloudflare)
- Custom domain added in Cloudflare Pages dashboard — SSL provisioned automatically
- Domain setup is a one-time operational task — detailed steps provided at implementation time

### Decision Impact Analysis

**Implementation Sequence (dependency order):**
1. JSON schemas — all components depend on these contracts
2. Rust binary — JSONL reader + GitHub Contents API + checkpoint logic
3. Python Actions scripts — aggregate + SVG generation
4. GitHub Actions workflow YAML — cron + community action wrapper
5. Bash installer — depends on binary release being available
6. Astro site — depends on public data.json schema
7. Rust release CI/CD — enables installer binary downloads
8. Astro deploy CI/CD — Cloudflare Pages manual dispatch workflow

**Cross-Component Dependencies:**
- Hive path schema → Rust binary write path + Python aggregate glob (must use identical path format)
- Public data.json schema → Python generate script + Astro dashboard (must agree on field names)
- Checkpoint.toml → Stop hook + SessionStart hook (shared local state, same binary)
- VIBESTATS_TOKEN scope → installer setup + `vibestats auth` refresh path

## Implementation Patterns & Consistency Rules

### Critical Conflict Points (6 identified)

1. JSON field naming across Rust/Python/JS boundary
2. Date/timestamp format across all components
3. Hive path zero-padding for lexicographic sort correctness
4. Error handling contract (exit 0 in binary, exit non-zero in Actions)
5. GitHub API access centralisation in binary
6. Repo layout — where each component lives

### Naming Patterns

**JSON fields:** `snake_case` everywhere
- `sessions`, `active_minutes`, `generated_at`, `machine_id`, `last_updated`
- Rust: `serde` default serialisation (snake_case struct fields, no rename needed)
- Python: native snake_case
- JavaScript: access as `data.active_minutes` (not camelCase mapping)

**File naming by language:**

| Language | Convention | Examples |
|---|---|---|
| Rust | `snake_case` | `jsonl_parser.rs`, `github_api.rs`, `checkpoint.rs` |
| Python | `snake_case` | `aggregate.py`, `generate_svg.py` |
| Astro/JS | `kebab-case` files, `camelCase` vars | `[username].astro`, `activeMins` |
| Shell | `kebab-case` | `install.sh` |

**CLI commands:** `kebab-case` subcommands, `--kebab-case` flags
- `vibestats sync --backfill`
- `vibestats machines list`
- `vibestats machines remove <id>`

### Structure Patterns

**Monorepo layout (`stephenleo/vibestats`):**
```
vibestats/
  src/                  ← Rust binary
  action/               ← Python Actions scripts
    aggregate.py
    generate_svg.py
    tests/
  site/                 ← Astro site
    src/
    public/
    package.json
  install.sh
  action.yml            ← Community GitHub Action definition
  Cargo.toml
  .github/
    workflows/
      release.yml       ← Rust binary CI (triggered on git tag v*)
      deploy-site.yml   ← Cloudflare Pages (manual workflow_dispatch)
      aggregate.yml     ← Template workflow for users' vibestats-data repos
```

**Test placement:**
- Rust: co-located `#[cfg(test)]` modules within each `.rs` file
- Python: `action/tests/test_aggregate.py`, `action/tests/test_generate_svg.py`
- Astro: no tests at MVP

### Format Patterns

**Dates and timestamps:**
- Day keys: `"YYYY-MM-DD"` ISO date string (e.g., `"2026-04-10"`)
- Timestamps: `"YYYY-MM-DDTHH:MM:SSZ"` ISO 8601 UTC (e.g., `"2026-04-10T14:23:00Z"`)
- Never Unix timestamps, never locale-formatted strings, never without timezone

**Hive path zero-padding:** always two digits — `month=04` not `month=4`, `day=09` not `day=9`
Ensures correct lexicographic sort order for glob patterns and partition pruning.

**Error log format** (`vibestats.log`):
```
YYYY-MM-DDTHH:MM:SSZ LEVEL message
```
Example:
```
2026-04-10T14:23:01Z ERROR sync failed: 401 Unauthorized — run `vibestats auth`
2026-04-10T14:28:07Z INFO  sync skipped: throttle active (last sync 3m ago)
```

### Process Patterns

**Silent failure contract (Rust binary — all code paths):**
1. Log to `vibestats.log` with timestamp + level
2. Set actionable flag in `checkpoint.toml` if recovery requires user action
3. `std::process::exit(0)` — never propagate errors to Claude Code (NFR10)

**Retry pattern — centralised, not per-call-site:**
```rust
// All GitHub API calls go through this — no inline retry logic elsewhere
fn with_retry<F, T>(f: F) -> Result<T>  // 3 retries: 1s, 2s, 4s delays
```

**GitHub API access — single module:**
All GitHub Contents API calls live in `src/github_api.rs`. No other module constructs HTTP requests to GitHub. This is the single enforcement point for auth headers, backoff, and error handling.

**Python Actions scripts — fail loudly:**
Opposite contract from the binary — scripts exit non-zero on any failure. GitHub Actions surfaces the error, blocks the workflow, and prevents corrupted outputs from being committed.

### Enforcement Guidelines

**All AI agents MUST:**
- Use `snake_case` for all JSON field names — no exceptions across any component
- Use zero-padded month/day in all Hive paths — `month=04`, never `month=4`
- Format all dates as ISO 8601 UTC strings — never Unix timestamps
- Route all GitHub API calls through `src/github_api.rs` in the Rust binary
- Exit 0 in the Rust binary on all error paths
- Exit non-zero in Python Actions scripts on all error paths

**Anti-patterns to avoid:**
- `camelCase` JSON fields (breaks Python/Rust serde compatibility)
- Non-zero exit in Rust binary error paths (breaks Claude Code hook contract)
- Inline GitHub API HTTP calls outside `github_api.rs`
- Un-padded Hive paths (`month=4`) — breaks lexicographic sort and partition pruning
- Unix timestamps in JSON — breaks human readability and Athena partition detection

## Project Structure & Boundaries

### Complete Project Directory Structure

```
stephenleo/vibestats/
│
├── README.md                        ← Install command, heatmap screenshot, vibestats.dev link
├── CONTRIBUTING.md
├── LICENSE
├── action.yml                       ← Community GitHub Action definition (FR42)
├── install.sh                       ← Bash installer (FR1–FR11, FR38–FR40)
├── Cargo.toml
├── Cargo.lock
├── .gitignore
│
├── src/                             ← Rust binary (FR12–FR19, FR32–FR37)
│   ├── main.rs                      ← Entry point, clap CLI routing
│   ├── config.rs                    ← Read/write ~/.config/vibestats/config.toml (FR39, NFR6)
│   ├── checkpoint.rs                ← Throttle timestamp + content hash (NFR2, NFR12)
│   ├── logger.rs                    ← Append to vibestats.log, TIMESTAMP LEVEL msg format
│   ├── jsonl_parser.rs              ← Walk ~/.claude/projects/**/*.jsonl, extract per-day data (FR14, NFR14)
│   ├── github_api.rs                ← ALL GitHub Contents API calls: GET SHA, PUT, backoff (FR16, NFR15)
│   ├── sync.rs                      ← Core sync orchestration: date range → hash check → push (FR12–FR18)
│   └── commands/
│       ├── mod.rs
│       ├── sync.rs                  ← `vibestats sync [--backfill]` (FR17, FR18)
│       ├── status.rs                ← `vibestats status` (FR32, FR33)
│       ├── machines.rs              ← `vibestats machines list/remove` (FR34, FR35)
│       ├── auth.rs                  ← `vibestats auth` (FR36, FR40)
│       └── uninstall.rs             ← `vibestats uninstall` (FR37)
│
├── action/                          ← Python Actions scripts (FR20–FR26, stdlib only)
│   ├── aggregate.py                 ← Glob Hive partition files, sum by date across machines/harnesses
│   ├── generate_svg.py              ← Render heatmap.svg (Claude orange, GitHub contributions grid)
│   ├── update_readme.py             ← Inject SVG URL between <!-- vibestats-start/end --> markers (FR24)
│   └── tests/
│       ├── test_aggregate.py
│       ├── test_generate_svg.py
│       └── fixtures/
│           ├── sample_machine_data/ ← Sample Hive partition files for test input
│           └── expected_output/     ← Expected SVG + data.json for assertion
│
├── site/                            ← Astro docs + dashboard (FR27–FR31, FR43)
│   ├── astro.config.mjs
│   ├── package.json                 ← cal-heatmap pinned here
│   ├── tsconfig.json
│   ├── public/
│   │   ├── favicon.svg
│   │   └── og-image.png
│   └── src/
│       ├── layouts/
│       │   ├── Base.astro
│       │   └── Docs.astro
│       ├── components/
│       │   ├── Heatmap.astro        ← cal-heatmap wrapper, receives days data as prop
│       │   ├── YearToggle.astro     ← Year filter buttons, derived from days keys
│       │   ├── Header.astro
│       │   └── Footer.astro
│       └── pages/
│           ├── index.astro          ← Landing / quickstart
│           ├── [username].astro     ← Dashboard shell + client-side fetch of data.json (FR29–FR31)
│           └── docs/
│               ├── quickstart.astro
│               ├── how-it-works.astro
│               ├── cli-reference.astro
│               └── troubleshooting.astro
│
└── .github/
    └── workflows/
        ├── release.yml              ← Rust binary CI: cross-compile 3 targets, attach to GitHub Release (FR41)
        ├── deploy-site.yml          ← Cloudflare Pages: manual workflow_dispatch with ref input
        └── aggregate.yml            ← Template for users' vibestats-data repos (calls stephenleo/vibestats@v1)
```

### Architectural Boundaries

**Data flow through the system:**
```
~/.claude/projects/**/*.jsonl
  → jsonl_parser.rs     (local read)
  → sync.rs             (compute sessions + active_minutes per day)
  → github_api.rs       (PUT to vibestats-data Hive path)
  → aggregate.py        (glob all Hive files, sum by date across harnesses + machines)
  → generate_svg.py     (render heatmap.svg)
  → update_readme.py    (inject SVG URL between README markers)
  → committed to username/username/vibestats/ by GitHub Action
  → fetched by [username].astro client-side at vibestats.dev/username
```

**Integration boundaries:**

| Boundary | From | To | Protocol |
|---|---|---|---|
| Machine → vibestats-data | `github_api.rs` | Hive partition files | GitHub Contents API (HTTPS) |
| vibestats-data → profile repo | Python Action scripts | `username/username/vibestats/` | `VIBESTATS_TOKEN` git push |
| Profile repo → dashboard | `[username].astro` | `raw.githubusercontent.com` | Client-side `fetch()` |
| Installer → GitHub | `install.sh` | Repos, secrets, hooks | `gh` CLI |
| Hook → binary | Claude Code `Stop`/`SessionStart` | `vibestats` binary | OS process spawn |

**Module responsibility boundaries (Rust):**

| Module | Owns | Never does |
|---|---|---|
| `github_api.rs` | All HTTP to GitHub, retry logic, SHA handling | Parse JSONL, read config |
| `jsonl_parser.rs` | JSONL file walking, session extraction | Network calls |
| `sync.rs` | Date range logic, hash comparison, orchestration | Direct HTTP, file parsing |
| `checkpoint.rs` | Throttle state, content hashes, auth_error flag | Network calls |
| `config.rs` | `config.toml` read/write, `600` perms enforcement | Business logic |
| `logger.rs` | Append to `vibestats.log` | Anything else |

### Requirements to Structure Mapping

| FR Category | Primary Files |
|---|---|
| Installation & Setup (FR1–11) | `install.sh` |
| Data Collection & Sync (FR12–19) | `src/sync.rs`, `src/jsonl_parser.rs`, `src/github_api.rs`, `src/checkpoint.rs` |
| Aggregation & Output (FR20–26) | `action/aggregate.py`, `action/generate_svg.py`, `action/update_readme.py` |
| Profile Display (FR27–31) | `site/src/pages/[username].astro`, `site/src/components/Heatmap.astro`, `YearToggle.astro` |
| CLI & Machine Management (FR32–37) | `src/commands/status.rs`, `machines.rs`, `uninstall.rs` |
| Authentication (FR38–40) | `src/commands/auth.rs`, `src/config.rs`, `install.sh` |
| Distribution & Docs (FR41–43) | `.github/workflows/release.yml`, `action.yml`, `site/src/pages/docs/` |

## Architecture Validation Results

### Coherence Validation ✅

All technology decisions are compatible and non-conflicting. `ureq` (sync HTTP) is
correct since async is handled at the Claude Code hook level. Python stdlib +
`xml.etree.ElementTree` for SVG is compatible with the Actions environment. Astro +
cal-heatmap via npm is standard bundling. Hive `=` in paths is supported by GitHub's
Contents API, web UI, and Git.

Patterns consistently support decisions across all 4 languages and 5 components.
Silent failure contract is correctly differentiated: exit 0 in binary, exit non-zero
in Python Actions scripts. All GitHub API calls routed through `github_api.rs`.

### Requirements Coverage Validation ✅

All 43 functional requirements and 17 non-functional requirements are architecturally
supported. Complete mapping documented in Project Structure section.

### Gap Analysis & Resolutions

**Gap 1 — CRITICAL (resolved): Dynamic username routing**

Astro SSG cannot pre-generate pages for arbitrary usernames at build time.

Resolution:
- Rename `[username].astro` → `site/src/pages/u/index.astro` (static shell, SSG)
- Add `site/public/_redirects`: `/:username  /u/index.html  200`
- Client-side JS reads username from `window.location.pathname.split('/').filter(Boolean)[0]`
- Cloudflare Pages serves the static shell for all `vibestats.dev/{username}` requests
- No SSR, no Cloudflare Workers, no server required

Updated structure additions:
```
site/src/pages/u/index.astro     ← dashboard shell (replaces [username].astro)
site/public/_redirects           ← Cloudflare URL rewrite rules
```

**Gap 2 — IMPORTANT (resolved): `machines remove` design**

Two-tier behaviour via optional flag:

`vibestats machines remove <id>` (default — retire):
- Sets machine status to `retired` in `vibestats-data/machines/registry.json` (single PUT)
- Historical data continues to be aggregated and shown on dashboard
- Use case: changing laptops — old machine's streaks remain visible

`vibestats machines remove <id> --purge-history` (explicit — purge):
- Sets status to `purged` in registry.json
- Bulk-deletes all Hive partition files for that machine_id via sequential Contents API DELETE calls
- Requires confirmation prompt: `"This will permanently remove all historical data for <hostname>. Continue? (y/N)"`
- Historical data disappears from dashboard on next cron run

`registry.json` machine states:

| Status | Set by | Aggregator behaviour |
|---|---|---|
| `active` | Installer | Include historical + future data |
| `retired` | `machines remove` (default) | Include historical data; no future pushes expected |
| `purged` | `machines remove --purge-history` | Skip entirely; Hive files deleted |

**Gap 3 — IMPORTANT (resolved): Remote retirement propagation**

When Machine A retires Machine B, Machine B continues pushing until it discovers
its own retired status. The only shared channel is `vibestats-data`. Resolution
via eventual consistency:

On **SessionStart** (already making network calls):
1. GET `registry.json` → check own `machine_id` status
2. If `retired`: set `machine_status = "retired"` in local `checkpoint.toml`, print
   terminal warning, skip catch-up sync, exit 0
3. If `active`: proceed normally

On **Stop hook** (hot path — no network calls):
1. Check throttle timestamp (local)
2. Check `machine_status` in `checkpoint.toml` — skip entirely if `"retired"`
3. If `active`: proceed with sync

Self-retire (same machine): binary updates both `registry.json` (remote) and
`checkpoint.toml` (local) in the same operation — immediately effective.

Remote retire lag: at most one session's worth of data after retirement. Acceptable.

Updated `checkpoint.toml` schema:
```toml
throttle_timestamp = "2026-04-10T14:23:01Z"
machine_status = "active"   # "active" | "retired"
auth_error = false

[date_hashes]
"2026-04-10" = "a3f8c2..."
"2026-04-09" = "b1d4e9..."
```

**Minor (noted, not blocking):**
- Actions workflow needs `git config user.name "vibestats[bot]"` + `user.email` before commit step
- `update_readme.py` commit step should wrap git push in a 3-retry loop for transient failures

### Architecture Completeness Checklist

**✅ Requirements Analysis**
- [x] Project context thoroughly analysed (43 FRs, 17 NFRs)
- [x] Scale and complexity assessed (Low–Medium, 5 components, 4 languages)
- [x] Technical constraints identified (zero-dep binary, stdlib Python, Bash 3.2)
- [x] Cross-cutting concerns mapped (6 identified and addressed)

**✅ Architectural Decisions**
- [x] 11 pre-existing ADRs validated and extended
- [x] Data architecture: Hive partition layout, schemas, merge strategy
- [x] Auth: dual token model, fine-grained PAT generation, validate-on-failure
- [x] API: GET+PUT pattern, content hash idempotency, error handling table
- [x] Infrastructure: cross crate CI, composite Action, Cloudflare Pages

**✅ Implementation Patterns**
- [x] 6 conflict points identified and resolved
- [x] Naming conventions per language documented with examples
- [x] Error handling contract differentiated (binary vs Actions)
- [x] Anti-patterns explicitly listed

**✅ Project Structure**
- [x] Complete directory tree with FR annotations
- [x] Module responsibility boundaries (owns / never does)
- [x] Integration boundaries table (5 boundaries defined)
- [x] FR category → file mapping (all 7 categories)

### Architecture Readiness Assessment

**Overall Status: READY FOR IMPLEMENTATION**

**Confidence Level: High** — all critical decisions are made, all FRs have a home,
patterns prevent the most likely agent conflicts, and all 3 gaps are resolved.

**Key strengths:**
- Hive partition layout is future-proof for multi-harness support and analytics
- Dual idempotency (content hash + day-file overwrite) prevents data corruption
- Silent failure contract clearly differentiated between binary and Actions layer
- Eventual consistency for machine retirement keeps Stop hook hot path network-free
- registry.json two-tier remove (retire/purge) preserves data by default
- Zero-dependency constraints baked into structure (stdlib Python, ureq over reqwest)

**Areas for future enhancement (post-MVP):**
- `machines purge` bulk cleanup command
- `machines reactivate <id>` to re-enable a retired machine
- Per-project breakdown field in data.json schema
- Token usage / cost fields in machine day files
- `vibestats update` self-update command

### Implementation Handoff

**First implementation priorities (in dependency order):**
1. Define and commit JSON schemas + registry.json format as documentation
2. `cargo new vibestats --bin` → implement `src/github_api.rs`, `src/jsonl_parser.rs`, `src/checkpoint.rs`
3. `action/aggregate.py` + `action/generate_svg.py` + `action/update_readme.py`
4. `.github/workflows/aggregate.yml` + `action.yml`
5. `install.sh`
6. `site/` Astro setup → `u/index.astro` + `public/_redirects`
7. `.github/workflows/release.yml` + `.github/workflows/deploy-site.yml`

---

## Known Gotchas & Conventions

This section captures non-obvious lessons and footguns discovered during implementation. Each item is cross-referenced to the source file that demonstrates the correct pattern.

---

### 1. Cloudflare Pages `_redirects` evaluation order

Cloudflare Pages evaluates `_redirects` top-to-bottom and stops at the first match. Pass-through rules for static assets **must appear before** any catch-all rewrite rules. A catch-all like `/:username /u  200` placed first will intercept `/_astro/`, `/favicon.ico`, and every other path — nothing else will ever be served.

**Additional footgun:** Rewriting to `/u.html` instead of `/u` triggers Cloudflare's clean-URL redirect (`*.html` → `*`), which sends the browser back to `/:username` — causing an infinite redirect loop. Always rewrite to `/u`, not `/u.html`.

Correct ordering (canonical reference: `site/public/_redirects`):

```
# Pass-through rules: keep static assets untouched.
# These MUST come before the /:username catch-all below.
/favicon.ico  /favicon.ico  200
/favicon.svg  /favicon.svg  200
/install.sh   /install.sh   200
/_astro/*     /_astro/:splat  200

# Catch-all: map vibestats.dev/<username> to the per-user dashboard shell.
# Target is /u (the clean URL), not /u.html.  Rewriting to /u.html would
# trigger Cloudflare's clean-URL redirect (*.html → *), sending the browser
# to /u, which re-matches /:username — causing an infinite redirect loop.
/:username    /u  200
```

Source: Epic 1 retrospective, Challenge #1; `site/public/_redirects`

---

### 2. Rust `#[serde(default)]` vs `Default` footgun

`#[serde(default = "some_fn")]` controls what serde uses during **deserialization** when a field is absent from the JSON/TOML. It does **not** affect `Default::default()`. If you `#[derive(Default)]`, the derived impl uses Rust's zero values (empty string, 0, false) — not `some_fn`.

Whenever a struct uses `#[serde(default = "fn")]` **and** needs `Default`, write a manual `Default` impl that calls the same functions.

Reference implementation (`src/checkpoint.rs` lines 10–33):

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Checkpoint {
    /// "active" | "retired" | "purged"; defaults to "active"
    #[serde(default = "default_machine_status")]
    pub machine_status: String,
    // ...
}

fn default_machine_status() -> String {
    "active".to_string()
}

impl Default for Checkpoint {
    fn default() -> Self {
        Self {
            throttle_timestamp: None,
            machine_status: default_machine_status(),
            auth_error: false,
            date_hashes: HashMap::new(),
        }
    }
}
```

Source: Epic 2 retrospective, Challenge #3; `src/checkpoint.rs`

---

### 3. Cargo worktree `[workspace]` isolation pattern

When running `cargo` commands from within a git worktree nested inside the main repo (e.g., `.worktrees/story-X-*`), Cargo walks up the directory tree and may find the parent `Cargo.toml`. To prevent this, the project's `Cargo.toml` must include `[workspace]` with no members — this declares an isolated workspace root and stops Cargo's upward search.

The vibestats `Cargo.toml` line 1 is `[workspace]` with no members key — this is intentional. Do not remove it.

Source: Epic 1 retrospective, Key Insight #3; `Cargo.toml`

---

### 4. `_gh()` define-if-not-defined pattern for testable shell helpers

When writing shell scripts that call external tools (`gh`, `curl`, `brew`, etc.), wrap each external call in a helper function using the define-if-not-defined guard. Test files can then pre-define their own stub before sourcing `install.sh`, making the entire script testable without shell binary mocking.

Pattern (from `install.sh` line 28):

```bash
if ! declare -f _gh > /dev/null 2>&1; then
  _gh() {
    GH_PAGER= gh "$@"
  }
fi
```

This pattern is used throughout `install.sh` and `tests/installer/` — all external tool wrappers follow this convention. Do **not** call `gh` directly in `install.sh`; always route through `_gh`.

Source: Epic 6 retrospective, Key Insight #1; `install.sh` line 28

---

### 5. Python3 stdlib over `jq` for JSON in Bash scripts

When shell scripts need JSON manipulation, use Python3 stdlib (`import json, sys, base64`) rather than `jq`. `jq` is not installed by default on macOS or many Linux distributions; Python3 is standard on both.

Standard pattern (from `install.sh` lines 197–199):

```bash
USER_JSON=$(_gh api /user)
GITHUB_USER=$(echo "$USER_JSON" | python3 -c "import sys, json; print(json.load(sys.stdin)['login'])")
```

For base64 decode (GitHub API Content responses):

```bash
python3 -c "import sys, base64; print(base64.b64decode(sys.stdin.read().replace('\n','')).decode())"
```

Pipe JSON to stdin and extract the required field. This avoids the `--jq` flag dependency on `gh` as well.

Source: Epic 6 retrospective, Key Insight #2; `install.sh` lines 197–199

---

### 6. Security negative test pattern

Security properties (e.g., "token is never written to disk") require **explicit negative test assertions** in bats. Implementation notes are not sufficient — add an assertion that actively detects leaks using a sentinel value.

Pattern (from `tests/installer/test_6_2.bats` around line 243):

```bash
@test "[P0] VIBESTATS_TOKEN is never written to disk or echoed to stdout" {
  SENTINEL_TOKEN="ghp_SENTINEL_VIBESTATS_TOKEN_DETECT_ME"

  # Stub returns the sentinel so we can scan for it afterward
  # ... (stub setup) ...

  # Remove the stub before scanning to avoid false positives
  rm -f "${HOME}/stub_env.sh"

  # Assert the sentinel does NOT appear in any file under $HOME
  found=$(grep -rl "${SENTINEL_TOKEN}" "${HOME}/" 2>/dev/null || true)
  [ -z "$found" ]

  # Also assert the sentinel does NOT appear in stdout
  [[ "$output" != *"${SENTINEL_TOKEN}"* ]]
}
```

Every security requirement in `install.sh` (NFR7: token never on disk) should have a corresponding negative bats test.

Source: Epic 6 retrospective, Key Insight #3; `tests/installer/test_6_2.bats` line 243
