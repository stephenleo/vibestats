---
stepsCompleted: ['step-01-validate-prerequisites', 'step-02-design-epics', 'step-03-write-stories']
inputDocuments:
  - '_bmad-output/planning-artifacts/prd.md'
  - '_bmad-output/planning-artifacts/architecture.md'
githubRepo: 'stephenleo/vibestats'
epicIssues:
  epic1: 1
  epic2: 2
  epic3: 3
  epic4: 4
  epic5: 5
  epic6: 6
  epic7: 7
  epic8: 8
storyIssues:
  s1_1: 9
  s1_2: 10
  s1_3: 11
  s1_4: 12
  s2_1: 13
  s2_2: 14
  s2_3: 15
  s2_4: 16
  s2_5: 17
  s3_1: 18
  s3_2: 19
  s3_3: 20
  s3_4: 21
  s4_1: 22
  s4_2: 23
  s4_3: 24
  s4_4: 25
  s5_1: 26
  s5_2: 27
  s5_3: 28
  s5_4: 29
  s5_5: 30
  s6_1: 31
  s6_2: 32
  s6_3: 33
  s6_4: 34
  s7_1: 35
  s7_2: 36
  s7_3: 37
  s7_4: 38
  s8_1: 39
  s8_2: 40
  s8_3: 41
---

# vibestats - Epic Breakdown

> **GitHub Repo:** [stephenleo/vibestats](https://github.com/stephenleo/vibestats)
>
> **Convention for PRs:** Every PR implementing a story must include `Closes #<issue>` in the PR description using the issue number from this document. Epic issues should be closed only when all their child stories are merged.

## Requirements Inventory

### Functional Requirements

FR1: A user can install vibestats on a machine with a single shell command
FR2: The installer detects whether `gh` is installed and installs it automatically if missing
FR3: The installer detects whether the user is authenticated with GitHub and initiates `gh auth login` if not
FR4: The installer creates the `vibestats-data` private repository on first install
FR5: The installer detects an existing `vibestats-data` repository and skips repo creation and secret setup on subsequent machine installs
FR6: The installer registers the current machine with a unique identifier in `vibestats-data`
FR7: The installer writes the Actions workflow into `vibestats-data` calling `stephenleo/vibestats@v1`
FR8: The installer configures Claude Code hooks (`Stop` and `SessionStart`) in `~/.claude/settings.json`
FR9: The installer adds `<!-- vibestats-start -->` / `<!-- vibestats-end -->` markers to the user's profile README
FR10: The installer sets the `VIBESTATS_TOKEN` Actions secret in `vibestats-data` on first install only
FR11: The installer triggers a full historical JSONL backfill immediately after setup completes
FR12: The system captures Claude Code session activity automatically after every session response via the `Stop` hook
FR13: The system performs a catch-up sync on every new Claude Code session start via the `SessionStart` hook
FR14: The Rust binary reads Claude Code JSONL files from `~/.claude/projects/**/*.jsonl` to extract per-day usage data
FR15: The system throttles `Stop` hook sync to at most once per 5 minutes
FR16: The system pushes per-machine daily usage JSON to `vibestats-data` via the GitHub Contents API
FR17: A user can force an immediate sync from any machine via `vibestats sync`
FR18: A user can trigger a full historical JSONL backfill from any machine via `vibestats sync --backfill`
FR19: The system logs sync failures locally and surfaces a warning on the next `SessionStart` if the last successful sync exceeds 24 hours
FR20: The GitHub Action aggregates all per-machine JSON files in `vibestats-data` into a single daily activity dataset
FR21: The GitHub Action generates a static `heatmap.svg` using the GitHub contributions grid shape in Claude orange
FR22: The GitHub Action generates a `data.json` file containing the full aggregated daily activity dataset
FR23: The GitHub Action commits `heatmap.svg` and `data.json` to `username/username/vibestats/`
FR24: The GitHub Action updates the user's profile README between the `<!-- vibestats-start/end -->` markers to embed the current SVG
FR25: The GitHub Action runs on a daily cron schedule
FR26: A user can manually trigger the GitHub Action via `workflow_dispatch`
FR27: The `heatmap.svg` is publicly accessible via `raw.githubusercontent.com/username/username/main/vibestats/heatmap.svg` and embeds in the profile README without JavaScript
FR28: The profile README heatmap includes a link to `vibestats.dev/username`
FR29: The `vibestats.dev/[username]` dashboard fetches `vibestats/data.json` from the user's public profile repo client-side — no per-user hosting required
FR30: The dashboard displays the full activity heatmap using `cal-heatmap`
FR31: The dashboard shows per-day session count and approximate active time on cell hover
FR32: A user can view all registered machines, their last sync timestamps, and GitHub connectivity status via `vibestats status`
FR33: A user can verify their current auth token validity via `vibestats status`
FR34: A user can list all machines registered in `vibestats-data` via `vibestats machines list`
FR35: A user can remove a specific machine's data from `vibestats-data` via `vibestats machines remove <id>`
FR36: A user can re-authenticate vibestats on any machine via `vibestats auth`, which updates both the local token and the `VIBESTATS_TOKEN` Actions secret
FR37: A user can uninstall vibestats from a machine via `vibestats uninstall`, which removes hooks from `~/.claude/settings.json` and deletes the local binary, and prints instructions for manual repo and README cleanup
FR38: The system uses the `gh` CLI as the authentication provider — `gh` is installed by the installer if missing
FR39: The system stores the GitHub OAuth token in `~/.config/vibestats/config.toml` after obtaining it via `gh auth token`
FR40: The system detects invalid or missing auth tokens and notifies the user on `SessionStart` with remediation instructions (`vibestats auth`)
FR41: The project provides pre-compiled binaries for macOS arm64, macOS x86_64, and Linux x86_64 via GitHub Releases on every tagged release
FR42: The `stephenleo/vibestats` repo is published to the GitHub Actions Marketplace as a community action referenceable as `stephenleo/vibestats@v1`
FR43: The project provides a public documentation and dashboard site at `vibestats.dev` covering quickstart, CLI reference, architecture, troubleshooting, and per-user dashboards at `vibestats.dev/[username]`

### NonFunctional Requirements

NFR1: Hook latency — `Stop` hook execution must complete within 2 seconds under normal network conditions (`async: true` so non-blocking)
NFR2: Hook throttle — sync is throttled to once per 5 minutes maximum
NFR3: Backfill throughput — full 12-month historical backfill must complete within 60 seconds on standard broadband
NFR4: Dashboard load — `vibestats.dev/[username]` must render within 3 seconds; data fetched client-side from a single public JSON file
NFR5: Actions runtime — daily cron must complete within 5 minutes and consume ≤60 minutes/month
NFR6: Token storage — `~/.config/vibestats/config.toml` must be created with permissions `600`
NFR7: Token scope minimisation — `VIBESTATS_TOKEN` scoped exclusively to `username/username` Contents write; machine-side token scoped to `vibestats-data` Contents write
NFR8: No secrets in commits — GitHub Action must never commit raw machine JSON to the public `username/username` repo
NFR9: Private data boundary — raw per-machine JSON files remain in `vibestats-data`; only aggregated daily totals published publicly
NFR10: Hook non-interference — crash or error in vibestats hook must not propagate to Claude Code; all hook errors caught and logged locally
NFR11: Silent sync failure — sync failures must fail silently during sessions; user notified only at next `SessionStart`
NFR12: Idempotent sync — pushing the same daily JSON multiple times produces identical results
NFR13: Actions resilience — GitHub Action handles transient API failures with retry; failed cron run must not corrupt existing SVG or data files
NFR14: JSONL format tolerance — JSONL parser handles missing or unknown fields gracefully across Claude Code versions
NFR15: GitHub API rate limits — Rust binary respects rate limits with exponential backoff on 429 responses
NFR16: `gh` CLI version compatibility — vibestats functions with `gh` CLI v2.0+; installer warns if below minimum
NFR17: Actions Marketplace compatibility — community action declares all required inputs, outputs, and permissions in `action.yml`, compatible with `ubuntu-latest`

### Additional Requirements (from Architecture)

- **Starter templates:** Rust: `cargo new vibestats --bin`; Astro: `npm create astro@latest vibestats-site -- --template minimal --typescript strict --no-install`
- **Monorepo layout:** `src/` (Rust), `action/` (Python), `site/` (Astro), `install.sh`, `action.yml`, `Cargo.toml` at root
- **Hive partition path:** `machines/year=YYYY/month=MM/day=DD/harness=claude/machine_id=<id>/data.json` (zero-padded month/day)
- **Public data.json schema:** `{ "generated_at": "<ISO 8601 UTC>", "username": "<str>", "days": { "YYYY-MM-DD": { "sessions": N, "active_minutes": N } } }`
- **Local files:** `config.toml` (600 perms), `checkpoint.toml` (throttle + hashes + machine_status + auth_error), `vibestats.log` (rolling 1MB)
- **All GitHub API calls** routed exclusively through `src/github_api.rs`
- **Exit 0** in all Rust binary error paths; **exit non-zero** in Python Actions scripts on error
- **Cloudflare Pages URL rewrite:** `site/public/_redirects` with `/:username  /u/index.html  200`
- **registry.json machine states:** active / retired / purged; `machines remove` defaults to retire (preserve history), `--purge-history` deletes Hive files
- **Remote machine retirement** via eventual consistency: SessionStart checks `registry.json` and updates local checkpoint
- **VIBESTATS_TOKEN** generated via `gh api /user/personal_access_tokens` (fine-grained PAT, never written to disk)
- **Content hash idempotency:** compare hash in `checkpoint.toml` before any API call
- **`vibestats auth`** refreshes both `config.toml` token and `VIBESTATS_TOKEN` Actions secret

### UX Design Requirements

No UX design document — vibestats has no bespoke UI beyond the Astro site and cal-heatmap integration.

### FR Coverage Map

| FR Category | Epic | Stories |
|---|---|---|
| Installation & Setup (FR1–11) | Epic 6 (#6) | #31, #32, #33, #34 |
| Data Collection & Sync (FR12–19) | Epic 3 (#3) | #18, #19, #20, #21 |
| Aggregation & Output (FR20–26) | Epic 5 (#5) | #26, #27, #28, #29, #30 |
| Profile Display (FR27–31) | Epic 7 (#7) | #35, #36, #37, #38 |
| CLI & Machine Management (FR32–37) | Epic 4 (#4) | #22, #23, #24, #25 |
| Authentication (FR38–40) | Epic 2 (#2) + Epic 6 (#6) | #13, #31, #32, #24 |
| Distribution & Docs (FR41–43) | Epic 8 (#8) + Epic 7 (#7) | #39, #40, #41, #37 |
| Foundation / Schemas | Epic 1 (#1) | #9, #10, #11, #12 |

## Epic List

| # | Epic | GH Issue | Stories |
|---|---|---|---|
| 1 | Project Foundation & Schema Definitions | [#1](https://github.com/stephenleo/vibestats/issues/1) | #9, #10, #11, #12 |
| 2 | Rust Binary — Foundation Modules | [#2](https://github.com/stephenleo/vibestats/issues/2) | #13, #14, #15, #16, #17 |
| 3 | Rust Binary — Sync Engine | [#3](https://github.com/stephenleo/vibestats/issues/3) | #18, #19, #20, #21 |
| 4 | Rust Binary — CLI Commands | [#4](https://github.com/stephenleo/vibestats/issues/4) | #22, #23, #24, #25 |
| 5 | GitHub Actions Pipeline | [#5](https://github.com/stephenleo/vibestats/issues/5) | #26, #27, #28, #29, #30 |
| 6 | Bash Installer | [#6](https://github.com/stephenleo/vibestats/issues/6) | #31, #32, #33, #34 |
| 7 | vibestats.dev Astro Site | [#7](https://github.com/stephenleo/vibestats/issues/7) | #35, #36, #37, #38 |
| 8 | CI/CD & Distribution | [#8](https://github.com/stephenleo/vibestats/issues/8) | #39, #40, #41 |

**Implementation order (dependency sequence):** Epic 1 → Epic 2 → Epic 3 + Epic 5 (parallel) → Epic 4 + Epic 7 (parallel) → Epic 8 → Epic 6

---

## Epic 1: Project Foundation & Schema Definitions
**GH Issue:** #1

Set up the monorepo structure, initialize all component projects, and define the shared data contracts that every component depends on. Must be completed first — all downstream epics rely on the schemas defined here.

### Story 1.1: Initialize monorepo directory structure
**GH Issue:** #9

As a developer,
I want the vibestats monorepo initialized with the correct directory structure,
So that all components have a defined home and the project is ready for implementation.

**Acceptance Criteria:**

**Given** the repo is empty
**When** the monorepo structure is created
**Then** `src/`, `action/`, `site/`, `install.sh`, `action.yml`, `Cargo.toml`, and `.github/workflows/` all exist at the correct paths
**And** `action/aggregate.py`, `action/generate_svg.py`, `action/update_readme.py`, and `action/tests/` directory stubs exist

**Given** the structure is created
**When** the root files are reviewed
**Then** `README.md` (with install command placeholder + vibestats.dev link), `CONTRIBUTING.md`, `LICENSE` (MIT), and `.gitignore` covering Rust build artifacts, node_modules, and site build output all exist

---

### Story 1.2: Initialize Rust binary project
**GH Issue:** #10

As a developer,
I want the Rust binary project initialized with all required crates declared,
So that the project compiles with a working CLI skeleton before any feature code is written.

**Acceptance Criteria:**

**Given** `cargo new vibestats --bin` has run
**When** `cargo build` is executed
**Then** the project compiles without errors

**Given** the project is initialized
**When** `Cargo.toml` is reviewed
**Then** `clap` (with derive feature), `serde` + `serde_json` (with derive), `ureq`, and `toml` are declared with pinned versions

**Given** the project is initialized
**When** `src/main.rs` is reviewed
**Then** a `clap` CLI skeleton defines `sync`, `status`, `machines`, `auth`, and `uninstall` subcommands, each with a stub handler that prints "not yet implemented"

---

### Story 1.3: Initialize Astro site project
**GH Issue:** #11

As a developer,
I want the Astro site project initialized with the correct structure and cal-heatmap dependency declared,
So that the dashboard and docs pages can be built without setup friction.

**Acceptance Criteria:**

**Given** `npm create astro@latest` has run with `--template minimal --typescript strict`
**When** `npm run build` executes inside `site/`
**Then** the build completes without errors

**Given** the project is initialized
**When** `site/package.json` is reviewed
**Then** `cal-heatmap` is declared as a pinned dependency (not a CDN import)

**Given** the project is initialized
**When** `site/public/_redirects` is reviewed
**Then** it contains exactly: `/:username  /u/index.html  200`

---

### Story 1.4: Define and document all JSON and TOML schemas
**GH Issue:** #12

As a developer,
I want all shared data schemas formally documented in one place,
So that every component implements the same data contracts without ambiguity.

**Acceptance Criteria:**

**Given** the schemas doc exists at `docs/schemas.md`
**When** the machine day file schema is reviewed
**Then** it defines leaf content as `{ "sessions": N, "active_minutes": N }` and the Hive path as `machines/year=YYYY/month=MM/day=DD/harness=claude/machine_id=<id>/data.json` with zero-padded month and day

**Given** the schemas doc exists
**When** the public data.json schema is reviewed
**Then** it defines `{ "generated_at": "<ISO 8601 UTC>", "username": "<str>", "days": { "YYYY-MM-DD": { "sessions": N, "active_minutes": N } } }`

**Given** the schemas doc exists
**When** the local config schemas are reviewed
**Then** `config.toml` fields (`oauth_token`, `machine_id`, `vibestats_data_repo`), `checkpoint.toml` fields (`throttle_timestamp`, `machine_status`, `auth_error`, `[date_hashes]`), and `registry.json` fields (`machines[].machine_id`, `hostname`, `status`, `last_seen`) are all documented with types and valid values

---

## Epic 2: Rust Binary — Foundation Modules
**GH Issue:** #2

Implement the foundational modules of the Rust binary: config, logger, checkpoint, JSONL parser, and GitHub API client. Depends on Epic 1.

### Story 2.1: Implement config module
**GH Issue:** #13

As the vibestats binary,
I want a config module that reads and writes `~/.config/vibestats/config.toml` with correct permissions,
So that the OAuth token, machine ID, and repo path are stored securely.

**Acceptance Criteria:**

**Given** `config.toml` does not exist
**When** `config.rs` writes it for the first time
**Then** the file is created at `~/.config/vibestats/config.toml` with permissions `600` (NFR6)

**Given** `config.toml` exists with valid content
**When** `Config::load()` is called
**Then** it returns a struct with `oauth_token`, `machine_id`, and `vibestats_data_repo` fields correctly populated

**Given** a new machine install
**When** `Config::generate_machine_id()` is called
**Then** it produces a deterministic ID from hostname + stable UUID and stores it in `config.toml`

**Given** `config.toml` is missing or malformed
**When** any command reads config
**Then** the binary exits 0, logs the error to `vibestats.log`, and prints a human-readable error with fix instructions (NFR10)

---

### Story 2.2: Implement logger module
**GH Issue:** #14

As the vibestats binary,
I want a logger module that appends structured entries to `~/.config/vibestats/vibestats.log`,
So that sync failures and diagnostics are captured without output to stdout during hooks.

**Acceptance Criteria:**

**Given** a log entry is written
**When** the log file is inspected
**Then** every line follows the format: `YYYY-MM-DDTHH:MM:SSZ LEVEL message` (UTC timestamp)

**Given** the log file reaches 1MB
**When** the next entry is written
**Then** the file is rotated (old renamed to `vibestats.log.1`, new file started)

**Given** a hook is firing during a Claude Code session
**When** any log write occurs
**Then** nothing is written to stdout or stderr (log-only, silent to the terminal) (NFR10, NFR11)

---

### Story 2.3: Implement checkpoint module
**GH Issue:** #15

As the vibestats binary,
I want a checkpoint module that persists throttle state, per-date content hashes, auth error flag, and machine status,
So that sync operations are idempotent and the Stop hook hot path makes no unnecessary API calls.

**Acceptance Criteria:**

**Given** a sync completed less than 5 minutes ago
**When** the Stop hook fires and reads the checkpoint
**Then** `Checkpoint::should_throttle()` returns `true` and no API call is made (NFR2)

**Given** a date's payload hash matches the stored hash in `checkpoint.toml`
**When** sync evaluates whether to push
**Then** no GitHub Contents API call is made for that date (NFR12)

**Given** a 401 response from the GitHub API
**When** the auth error is recorded
**Then** `auth_error = true` is written to `checkpoint.toml` and the binary exits 0

**Given** `machine_status = "retired"` is in `checkpoint.toml`
**When** the Stop hook fires
**Then** it skips entirely without any network call

---

### Story 2.4: Implement JSONL parser
**GH Issue:** #16

As the vibestats binary,
I want a JSONL parser that walks `~/.claude/projects/**/*.jsonl` and extracts per-day session activity,
So that usage data is derived from the authoritative local source.

**Acceptance Criteria:**

**Given** JSONL files exist under `~/.claude/projects/`
**When** `jsonl_parser::parse_date_range(start, end)` is called
**Then** it returns a map of `{ "YYYY-MM-DD": { sessions, active_minutes } }` aggregated across all matching files for the requested date range

**Given** a JSONL file contains fields not in the known schema
**When** the parser processes it
**Then** unknown fields are silently ignored and known fields are extracted correctly (NFR14)

**Given** 12 months of JSONL history exists
**When** `parse_date_range` is called for the full history
**Then** parsing completes in under 10 seconds on typical hardware (NFR3 baseline)

---

### Story 2.5: Implement GitHub API module
**GH Issue:** #17

As the vibestats binary,
I want a single GitHub API module that handles all Contents API calls with retry and error handling,
So that no other module makes direct HTTP calls to GitHub and the silent failure contract is enforced.

**Acceptance Criteria:**

**Given** a day file does not yet exist in vibestats-data
**When** `github_api::put_file(path, content)` is called
**Then** it performs a PUT without a SHA (first-time create) and returns success

**Given** a day file already exists
**When** `github_api::put_file(path, content)` is called
**Then** it first GETs the current SHA, then PUTs with that SHA (update pattern)

**Given** the API returns a 429 or 5xx response
**When** `github_api` handles the error
**Then** it retries with exponential backoff: 1s → 2s → 4s, max 3 attempts, logs, exits 0 (NFR15)

**Given** any module other than `github_api.rs` needs to call GitHub
**When** it is implemented
**Then** it calls functions in `github_api.rs` — no inline HTTP requests permitted elsewhere

---

## Epic 3: Rust Binary — Sync Engine
**GH Issue:** #3

Implement core sync logic: Stop hook, SessionStart catch-up, staleness warnings, throttle, and backfill. Depends on Epic 2.

### Story 3.1: Implement core sync orchestration
**GH Issue:** #18

As the vibestats binary,
I want a sync orchestration layer that coordinates JSONL parsing, hash comparison, and GitHub API push for a given date range,
So that any entry point (hook, CLI, backfill) routes through the same tested logic.

**Acceptance Criteria:**

**Given** a date range is passed to `sync::run(start_date, end_date)`
**When** it runs
**Then** it: (1) calls `jsonl_parser` for the range, (2) computes payload hash per date, (3) skips dates where hash matches checkpoint, (4) calls `github_api::put_file` for changed dates, (5) updates checkpoint hashes on success

**Given** the same JSONL data for a date
**When** sync runs twice
**Then** the second run makes zero API calls (idempotency via hash check) (NFR12)

**Given** sync runs for any code path
**When** it exits
**Then** it always exits 0 — no exceptions propagate up (NFR10)

---

### Story 3.2: Implement Stop hook integration
**GH Issue:** #19

As the vibestats system,
I want the Stop hook to fire after every Claude Code session response and sync today's data if not throttled,
So that the profile heatmap stays current with zero user action.

**Acceptance Criteria:**

**Given** the Stop hook fires and the last sync was under 5 minutes ago
**When** the hook runs
**Then** it exits 0 immediately without any API call (NFR2)

**Given** the Stop hook fires and the throttle is clear
**When** it runs
**Then** it calls `sync::run(today, today)` and updates the throttle timestamp in checkpoint on success

**Given** the hook is configured
**When** `~/.claude/settings.json` is inspected
**Then** it contains a `Stop` hook entry with `command: vibestats sync` and `async: true`

---

### Story 3.3: Implement SessionStart hook integration
**GH Issue:** #20

As the vibestats system,
I want the SessionStart hook to perform catch-up sync, check staleness, surface auth errors, and detect machine retirement,
So that missed syncs are recovered and the user is warned about issues at session start — not mid-session.

**Implementation note:** This story covers four distinct behaviours that must execute in the following order on every SessionStart: (1) machine retirement check, (2) auth error surface, (3) catch-up sync, (4) staleness warning. Complete all four before closing the story.

**Acceptance Criteria:**

**— Behaviour 1: Machine retirement detection —**

**Given** this machine's `machine_id` appears as `retired` in `registry.json`
**When** SessionStart checks the registry
**Then** it updates `machine_status = "retired"` in checkpoint.toml, prints a warning, skips catch-up sync, and exits 0

**— Behaviour 2: Auth error surface —**

**Given** `checkpoint.toml` has `auth_error = true`
**When** SessionStart fires
**Then** it prints: "vibestats: auth error detected. Run `vibestats auth` to re-authenticate." and clears the flag (FR40)

**— Behaviour 3: Catch-up sync —**

**Given** there are dates between last_sync_date and yesterday with no pushed data
**When** SessionStart fires (and machine is not retired)
**Then** it calls `sync::run(last_sync_date, yesterday)` to fill the gap (FR13)

**— Behaviour 4: Staleness warning —**

**Given** the last successful sync was more than 24 hours ago
**When** SessionStart fires
**Then** it prints: "vibestats: last sync was N days ago on this machine. Run `vibestats status` to diagnose." (FR19)

---

### Story 3.4: Implement vibestats sync and vibestats sync --backfill commands
**GH Issue:** #21

As a developer,
I want to manually trigger a sync or full historical backfill from the CLI,
So that I can recover gaps and verify my setup without waiting for a hook to fire.

**Acceptance Criteria:**

**Given** the user runs `vibestats sync`
**When** it executes
**Then** it runs sync for today (unthrottled) and prints a success or failure summary to stdout (FR17)

**Given** the user runs `vibestats sync --backfill`
**When** it executes
**Then** it calls `sync::run` for all dates present in the full JSONL history and reports the count of dates synced and any failures (FR18)

**Given** 12 months of JSONL data exists
**When** `vibestats sync --backfill` runs
**Then** it completes within 60 seconds on standard broadband (NFR3)

**Given** `vibestats sync --backfill` is interrupted and run again
**When** it resumes
**Then** dates already synced (hash match) are skipped (NFR12)

---

## Epic 4: Rust Binary — CLI Commands
**GH Issue:** #4

Implement the full CLI surface: status, machine management, auth token refresh, and uninstall. Depends on Epic 2 and 3.

### Story 4.1: Implement vibestats status command
**GH Issue:** #22

As a developer,
I want `vibestats status` to show me all registered machines, last sync times, and auth token validity,
So that I can diagnose sync issues without reading log files.

**Acceptance Criteria:**

**Given** the user runs `vibestats status`
**When** it executes
**Then** it prints each registered machine from `registry.json` with `machine_id`, `hostname`, `status`, and `last_seen` timestamp (FR32)

**Given** the current machine's OAuth token is valid
**When** `vibestats status` runs a connectivity check
**Then** it shows "Auth: OK" alongside the associated GitHub username (FR33)

**Given** the current machine's OAuth token is invalid
**When** the connectivity check fails with 401
**Then** it shows "Auth: ERROR — run `vibestats auth` to re-authenticate" (FR33)

---

### Story 4.2: Implement vibestats machines list and machines remove
**GH Issue:** #23

As a developer,
I want to list and remove machines from vibestats-data,
So that I can manage which machines contribute to my heatmap.

**Acceptance Criteria:**

**Given** the user runs `vibestats machines list`
**When** it executes
**Then** it prints all machines from `registry.json` with `machine_id`, `hostname`, `status`, and `last_seen` (FR34)

**Given** the user runs `vibestats machines remove <id>` (no flag)
**When** it executes
**Then** it sets `status = "retired"` in `registry.json` via GitHub Contents API PUT, preserving all historical Hive partition files (default retire)

**Given** the user runs `vibestats machines remove <id> --purge-history`
**When** it executes
**Then** it prompts "This will permanently remove all historical data for <hostname>. Continue? (y/N)" and on confirmation sets `status = "purged"` and bulk-deletes all Hive partition files for that `machine_id` (FR35)

---

### Story 4.3: Implement vibestats auth command
**GH Issue:** #24

As a developer,
I want `vibestats auth` to refresh my GitHub OAuth token and update the Actions secret,
So that a revoked or expired token can be fixed with one command on any machine.

**Acceptance Criteria:**

**Given** the user runs `vibestats auth`
**When** it executes
**Then** it calls `gh auth token` to obtain a fresh token and writes it to `~/.config/vibestats/config.toml` with permissions `600` (FR36, NFR6)

**Given** a fresh token is obtained
**When** `vibestats auth` proceeds
**Then** it updates the `VIBESTATS_TOKEN` Actions secret in `vibestats-data` via `gh secret set` (FR36)

**Given** the auth refresh completes
**When** the user runs `vibestats status` afterwards
**Then** auth shows "Auth: OK" and `checkpoint.toml` `auth_error` is cleared (FR40)

---

### Story 4.4: Implement vibestats uninstall command
**GH Issue:** #25

As a developer,
I want `vibestats uninstall` to cleanly remove vibestats from a machine,
So that I can remove it without leaving behind hooks or binaries.

**Acceptance Criteria:**

**Given** the user runs `vibestats uninstall`
**When** it executes
**Then** it removes the `Stop` and `SessionStart` hook entries from `~/.claude/settings.json` (FR37)
**And** it deletes the `vibestats` binary from its installed location

**Given** the uninstall is complete
**When** the user reads the terminal output
**Then** it prints instructions for the remaining optional manual steps: deleting `vibestats-data` repo and removing `<!-- vibestats-start/end -->` markers from the profile README (FR37)

**Given** `~/.claude/settings.json` contains other hooks (not vibestats)
**When** uninstall runs
**Then** only vibestats hook entries are removed; all other settings are preserved

---

## Epic 5: GitHub Actions Pipeline
**GH Issue:** #5

Implement Python aggregation scripts, SVG generator, README updater, composite community GitHub Action, and user workflow template. Depends on Epic 1.

### Story 5.1: Implement aggregate.py
**GH Issue:** #26

As the GitHub Actions pipeline,
I want an aggregation script that reads all Hive partition files from vibestats-data and produces a single merged daily dataset,
So that per-machine data is combined before SVG generation.

**Acceptance Criteria:**

**Given** Hive partition files exist at `machines/year=YYYY/month=MM/day=DD/harness=claude/machine_id=<id>/data.json`
**When** `aggregate.py` runs
**Then** it globs all matching paths and sums `sessions` and `active_minutes` by date across all machines and harnesses

**Given** multiple machines pushed data for the same date
**When** aggregation runs
**Then** their values are summed (not overwritten)

**Given** a machine's status is `purged` in `registry.json`
**When** aggregation runs
**Then** its Hive partition files are skipped entirely

**Given** the aggregation script encounters any error
**When** it fails
**Then** it exits non-zero so the GitHub Action surfaces the failure and blocks the commit step

**Given** aggregation completes successfully
**When** the working directory is inspected
**Then** `data.json` exists and conforms to the public schema: `{ "generated_at": "<ISO 8601 UTC>", "username": "<str>", "days": { "YYYY-MM-DD": { "sessions": N, "active_minutes": N } } }` (FR22)

---

### Story 5.2: Implement generate_svg.py
**GH Issue:** #27

As the GitHub Actions pipeline,
I want an SVG generator that renders a GitHub-contributions-style heatmap in Claude orange,
So that the output embeds in a profile README without JavaScript.

**Acceptance Criteria:**

**Given** the aggregated daily data map is passed to `generate_svg.py`
**When** it runs
**Then** it produces a valid SVG using `xml.etree.ElementTree` (stdlib only) with a grid layout matching GitHub's contributions graph shape (52 columns × 7 rows per year)

**Given** the SVG is generated
**When** its cell colours are inspected
**Then** activity intensity uses Claude orange shades (low: `#fef3e8` → high: `#f97316`), with neutral colour for zero-activity days

**Given** the SVG is embedded in a GitHub profile README via an `<img>` tag
**When** it renders
**Then** it displays correctly as a static image (no JavaScript — compatible with GitHub's DOMPurify) (FR27)

---

### Story 5.3: Implement update_readme.py
**GH Issue:** #28

As the GitHub Actions pipeline,
I want a README updater that injects the SVG embed and dashboard link between the vibestats markers,
So that the profile README is updated automatically on every cron run.

**Acceptance Criteria:**

**Given** the profile README contains `<!-- vibestats-start -->` and `<!-- vibestats-end -->` markers
**When** `update_readme.py` runs
**Then** it replaces content between markers with the SVG `<img>` tag pointing to `raw.githubusercontent.com/username/username/main/vibestats/heatmap.svg` and a "View interactive dashboard →" link to `vibestats.dev/username` (FR24, FR28)

**Given** the markers are missing
**When** `update_readme.py` runs
**Then** it exits non-zero with a clear error explaining the markers must be present

**Given** the README content between markers has not changed
**When** `update_readme.py` runs
**Then** it skips the git commit step (no empty commits) (NFR13)

---

### Story 5.4: Implement action.yml (composite community GitHub Action)
**GH Issue:** #29

As a vibestats user,
I want a community GitHub Action at `stephenleo/vibestats` referenceable as `uses: stephenleo/vibestats@v1`,
So that I pick up aggregation fixes and SVG updates automatically without managing Python scripts myself.

**Acceptance Criteria:**

**Given** `action.yml` exists at the repo root
**When** it is reviewed
**Then** it declares type `composite`, inputs `token` (VIBESTATS_TOKEN) and `profile-repo` (username/username), and all required permissions (NFR17)

**Given** the action runs in a user's `vibestats-data` workflow
**When** it executes
**Then** it: (1) checks out vibestats-data, (2) runs `aggregate.py`, (3) runs `generate_svg.py`, (4) runs `update_readme.py`, (5) commits and pushes outputs to `profile-repo` using the `token` input (FR23)

**Given** any action step fails
**When** the workflow exits
**Then** it exits non-zero and no partial outputs are committed (NFR13)

---

### Story 5.5: Implement aggregate.yml (user vibestats-data workflow template)
**GH Issue:** #30

As a vibestats user,
I want a ready-to-use workflow file for my `vibestats-data` repo that runs the community action on a daily cron,
So that my heatmap updates automatically every day.

**Acceptance Criteria:**

**Given** `aggregate.yml` is copied into a user's `vibestats-data/.github/workflows/`
**When** it runs
**Then** it calls `uses: stephenleo/vibestats@v1` with `token: ${{ secrets.VIBESTATS_TOKEN }}` and `profile-repo: username/username`

**Given** the workflow file is reviewed
**When** the triggers are inspected
**Then** it includes both `schedule: cron` (daily) and `workflow_dispatch` (manual trigger) (FR25, FR26)

**Given** the workflow runs over a month
**When** Actions minutes are measured
**Then** total consumption stays within 60 minutes/month (daily cron only, no per-push triggers) (NFR5)

---

## Epic 6: Bash Installer
**GH Issue:** #6

Implement `install.sh`: dependency checks, first-install path, multi-machine path, hook configuration, README marker injection, and post-install backfill. Depends on Epic 8 (binary release available).

### Story 6.1: Implement dependency detection and gh authentication
**GH Issue:** #31

As a new vibestats user,
I want the installer to handle all dependency and authentication checks automatically,
So that I never need to manually install gh or run auth flows.

**Acceptance Criteria:**

**Given** `gh` is not installed
**When** `install.sh` runs
**Then** it installs `gh` via the appropriate method: `brew install gh` (macOS) or `apt-get install gh` (Debian/Ubuntu) (FR2)

**Given** `gh` installed version is below 2.0
**When** the version check runs
**Then** the installer prints a warning and exits with a clear error message (NFR16)

**Given** `gh` is installed but the user is not authenticated
**When** the auth check runs
**Then** the installer runs `gh auth login` via the standard browser flow (FR3)

**Given** `uname -s` + `uname -m` identify the platform
**When** the binary download step runs
**Then** it downloads the correct `vibestats-<platform>.tar.gz` from the latest GitHub Release and verifies the checksum before installing to `~/.local/bin/vibestats`

---

### Story 6.2: Implement first-install path
**GH Issue:** #32

As a first-time vibestats user,
I want the installer to create my vibestats-data repo, write the workflow, and configure my tokens in one pass,
So that I can go from zero to a live heatmap without any manual GitHub steps.

**Acceptance Criteria:**

**Given** `vibestats-data` does not exist under the user's account
**When** the first-install path runs
**Then** it creates `username/vibestats-data` as a private repo via `gh repo create` (FR4)

**Given** the repo is created
**When** the workflow setup runs
**Then** it writes `aggregate.yml` into `vibestats-data/.github/workflows/` calling `stephenleo/vibestats@v1` (FR7)

**Given** the workflow is written
**When** the token setup runs
**Then** it generates `VIBESTATS_TOKEN` via `gh api /user/personal_access_tokens` (fine-grained PAT, `username/username` Contents write only), sets it as the `VIBESTATS_TOKEN` Actions secret, and the token is never written to disk (FR10, NFR7)

**Given** the local token is obtained via `gh auth token`
**When** it is stored
**Then** it is written to `~/.config/vibestats/config.toml` with permissions `600` (FR39, NFR6)

**Given** all first-install setup steps complete
**When** `vibestats-data/registry.json` is read
**Then** it contains one entry for the current machine with `machine_id`, `hostname`, `status = "active"`, and `last_seen` timestamp set to the time of install (FR6)

---

### Story 6.3: Implement multi-machine install path
**GH Issue:** #33

As a developer adding a second machine,
I want the installer to detect my existing vibestats-data repo and skip redundant setup steps,
So that adding a new machine is just as fast as the first install.

**Acceptance Criteria:**

**Given** `vibestats-data` already exists
**When** the installer runs
**Then** it detects the existing repo via `gh repo view`, skips repo creation, workflow write, and `VIBESTATS_TOKEN` secret setup (FR5)

**Given** the existing repo is detected
**When** machine registration runs
**Then** the new machine's `machine_id` and `hostname` are added to `registry.json` with `status = "active"` via Contents API PUT (FR6)

---

### Story 6.4: Implement hook configuration, README markers, and backfill trigger
**GH Issue:** #34

As a vibestats user completing installation,
I want the installer to configure my Claude Code hooks, add README markers, and trigger an immediate backfill,
So that history is visible the moment I open my profile.

**Acceptance Criteria:**

**Given** installation reaches its final phase
**When** hook configuration runs
**Then** `~/.claude/settings.json` is updated with `Stop` hook (`command: vibestats sync`, `async: true`) and `SessionStart` hook (`command: vibestats sync`) (FR8)

**Given** the profile README at `username/username/README.md` exists
**When** the marker injection runs
**Then** `<!-- vibestats-start -->` and `<!-- vibestats-end -->` markers are added with the SVG `<img>` embed and dashboard link between them (FR9)

**Given** all setup steps complete
**When** the installer triggers the post-install backfill
**Then** it runs `vibestats sync --backfill` as the final step, printing progress to the terminal (FR11)

---

## Epic 7: vibestats.dev Astro Site
**GH Issue:** #7

Build the static documentation site and per-user interactive dashboard at vibestats.dev. Depends on Epic 1 (Astro init) and Epic 1.4 (public data.json schema).

### Story 7.1: Build base layouts and shared Astro components
**GH Issue:** #35

As a developer,
I want base Astro layouts and shared components built,
So that all pages share consistent structure without duplicating markup.

**Acceptance Criteria:**

**Given** the Astro project is initialized (Story 1.3)
**When** `Base.astro` and `Docs.astro` layouts are built
**Then** they define a consistent `<head>`, `Header.astro` (vibestats logo + nav), and `Footer.astro` (GitHub link, license)

**Given** a page uses `Docs.astro` layout
**When** it renders
**Then** it includes sidebar navigation linking to all docs pages

**Given** the shared components are built
**When** `npm run build` runs
**Then** the build completes without TypeScript or Astro errors

---

### Story 7.2: Build per-user dashboard (u/index.astro + cal-heatmap)
**GH Issue:** #36

As a profile visitor,
I want to open `vibestats.dev/username` and see an interactive activity heatmap with hover details,
So that I can explore a developer's Claude Code usage history beyond what the static README shows.

**Acceptance Criteria:**

**Given** a visitor opens `vibestats.dev/stephenleo`
**When** the page loads
**Then** Cloudflare serves `u/index.html` (via `_redirects`), client-side JS reads `stephenleo` from `window.location.pathname`, and fetches `https://raw.githubusercontent.com/stephenleo/stephenleo/main/vibestats/data.json` (FR29)

**Given** the data.json fetch succeeds
**When** the heatmap renders
**Then** `cal-heatmap` displays the full activity grid for the current year with Claude-orange colour scale (FR30)

**Given** the user hovers a day cell
**When** the tooltip appears
**Then** it shows the date, session count, and approximate active minutes (FR31)

**Given** data.json contains multiple years
**When** the year toggle is rendered
**Then** year buttons appear descending (newest first), current year selected by default, clicking re-renders without a new fetch

**Given** the data.json fetch fails
**When** the error state renders
**Then** the page shows "No vibestats data found for @username"

---

### Story 7.3: Build documentation pages
**GH Issue:** #37

As a potential vibestats user,
I want clear documentation covering quickstart, architecture, CLI reference, and troubleshooting,
So that I can install and use vibestats without reading the source code.

**Acceptance Criteria:**

**Given** the docs site is built
**When** the quickstart page is viewed
**Then** it shows the install command, lists the 5-minute install steps, and links to the CLI reference (FR43)

**Given** the docs site is built
**When** the "How it works" page is viewed
**Then** it includes an architecture diagram showing the data flow: JSONL → vibestats-data → GitHub Action → profile README → vibestats.dev

**Given** the docs site is built
**When** the CLI reference page is viewed
**Then** every subcommand (`status`, `sync`, `sync --backfill`, `machines list`, `machines remove`, `auth`, `uninstall`) is documented with description, flags, and example output

**Given** the docs site is built
**When** the troubleshooting page is viewed
**Then** it covers: token expiry fix, hook not firing, missing machine data, and how to trigger a manual backfill

---

### Story 7.4: Build landing page
**GH Issue:** #38

As a developer discovering vibestats,
I want a compelling landing page at `vibestats.dev`,
So that I understand what it does and how to install it within 30 seconds.

**Acceptance Criteria:**

**Given** a visitor opens `vibestats.dev`
**When** the page loads
**Then** it shows: (1) the one-line install command in a copyable code block, (2) an example heatmap SVG, (3) a three-bullet "why vibestats" section (zero effort, cross-machine, GitHub profile)

**Given** the install command is displayed
**When** the visitor copies it
**Then** it reads exactly: `curl -sSf https://vibestats.dev/install.sh | bash`

**Given** the landing page is built
**When** `npm run build` runs
**Then** the page passes Astro's static build without errors

---

## Epic 8: CI/CD & Distribution
**GH Issue:** #8

Set up GitHub Actions pipelines for cross-platform binary releases, Cloudflare Pages deployment, and GitHub Actions Marketplace publication.

### Story 8.1: Implement Rust binary release CI
**GH Issue:** #39

As a vibestats user,
I want pre-compiled binaries for macOS arm64, macOS x86_64, and Linux x86_64 automatically published to GitHub Releases on every tag,
So that install.sh can download the correct binary without requiring Rust to be installed.

**Acceptance Criteria:**

**Given** a git tag matching `v*` is pushed
**When** `release.yml` runs
**Then** it triggers a matrix build using the `cross` crate for targets: `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu` (FR41)

**Given** all three targets compile
**When** the release step runs
**Then** each binary is archived as `vibestats-<target>.tar.gz` and attached to the GitHub Release (FR41)

**Given** any compilation target fails
**When** the workflow exits
**Then** it exits non-zero and no partial release is published

---

### Story 8.2: Implement Cloudflare Pages deploy workflow
**GH Issue:** #40

As a developer deploying vibestats.dev,
I want a manually-triggered GitHub Actions workflow that deploys the Astro site to Cloudflare Pages,
So that I control exactly which version is live in production.

**Acceptance Criteria:**

**Given** `deploy-site.yml` exists in `.github/workflows/`
**When** it is reviewed
**Then** it is triggered only via `workflow_dispatch` with a `ref` input (branch or tag) — no automatic triggers

**Given** the workflow is dispatched
**When** it runs
**Then** it checks out the specified ref, runs `npm run build` inside `site/`, and deploys to Cloudflare Pages using `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` secrets

**Given** the build step fails
**When** the workflow exits
**Then** no deployment to Cloudflare occurs (build gates deploy)

---

### Story 8.3: Configure GitHub Actions Marketplace publication
**GH Issue:** #41

As a vibestats user discovering the tool,
I want the community action published on the GitHub Actions Marketplace,
So that I can find it and reference it as `uses: stephenleo/vibestats@v1`.

**Acceptance Criteria:**

**Given** `action.yml` exists at the repo root (from Story 5.4)
**When** the Marketplace metadata is reviewed
**Then** it includes `name`, `description`, `branding` (icon + colour), and a `runs` section — NFR17

**Given** the repo is public and `action.yml` is at the root
**When** the GitHub Actions Marketplace listing is submitted
**Then** the action is referenceable as `uses: stephenleo/vibestats@v1` (FR42)

**Given** a new major version `v2` is released
**When** the tag is pushed
**Then** `v1` continues to work for existing users (semver-based versioning documented in `CONTRIBUTING.md`)
