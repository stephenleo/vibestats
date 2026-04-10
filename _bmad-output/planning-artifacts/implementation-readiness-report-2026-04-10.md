---
stepsCompleted: [step-01-document-discovery, step-02-prd-analysis, step-03-epic-coverage-validation, step-04-ux-alignment, step-05-epic-quality-review, step-06-final-assessment]
documentsUsed:
  prd: planning-artifacts/prd.md
  architecture: planning-artifacts/architecture.md
  epics: planning-artifacts/epics.md
  ux: null
---

# Implementation Readiness Assessment Report

**Date:** 2026-04-10
**Project:** vibestats

---

## Document Inventory

| Type | File | Status |
|---|---|---|
| PRD | `planning-artifacts/prd.md` | ✅ Found |
| Architecture | `planning-artifacts/architecture.md` | ✅ Found |
| Epics & Stories | `planning-artifacts/epics.md` | ✅ Found |
| UX Design | — | ⚠️ Not found (acceptable for this project type) |

---

## PRD Analysis

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

**Total FRs: 43**

---

### Non-Functional Requirements

NFR1 (Performance — Hook latency): The `Stop` hook execution must complete within 2 seconds under normal network conditions
NFR2 (Performance — Hook throttle): Sync is throttled to once per 5 minutes maximum
NFR3 (Performance — Backfill throughput): Full historical backfill across 12 months must complete within 60 seconds on standard broadband
NFR4 (Performance — Dashboard load): `vibestats.dev/[username]` must render the heatmap within 3 seconds on a standard connection
NFR5 (Performance — Actions runtime): Daily GitHub Actions cron must complete within 5 minutes and consume no more than 60 minutes/month
NFR6 (Security — Token storage): `~/.config/vibestats/config.toml` must have file permissions `600` on creation
NFR7 (Security — Token scope minimisation): Tokens scoped to minimum required permissions only — no broader repo or org scopes
NFR8 (Security — No secrets in commits): The GitHub Action must never commit raw machine JSON to the public profile repo
NFR9 (Security — Private data boundary): Raw per-machine JSON remains in `vibestats-data` (private); only aggregated daily totals published publicly
NFR10 (Reliability — Hook non-interference): A crash in the vibestats hook must not propagate to Claude Code or interrupt the user's session
NFR11 (Reliability — Silent sync failure): Sync failures must fail silently during sessions; user notified only at next `SessionStart`
NFR12 (Reliability — Idempotent sync): Pushing the same daily JSON multiple times must produce identical results — no duplicate data
NFR13 (Reliability — Actions resilience): The GitHub Action must handle transient API failures with retry logic; a failed cron run must not corrupt existing files
NFR14 (Integration — JSONL format tolerance): The JSONL parser must handle missing or unknown fields gracefully across Claude Code versions
NFR15 (Integration — GitHub API rate limits): The Rust binary must respect GitHub Contents API rate limits and implement exponential backoff on 429 responses
NFR16 (Integration — `gh` CLI version compatibility): vibestats must function with `gh` CLI version 2.0+; installer must check and warn if below minimum
NFR17 (Integration — Actions Marketplace compatibility): The community action must declare all required inputs, outputs, and permissions in `action.yml` and be compatible with `ubuntu-latest` runners

**Total NFRs: 17**

---

### Additional Requirements & Constraints

- **Platform support:** macOS (arm64, x86_64) and Linux x86_64 (WSL2). Windows native is out of scope for MVP.
- **Language constraints:** Rust binary (zero runtime deps), Python stdlib only in Actions, Bash installer, Astro + JS dashboard
- **Auth architecture:** `gh` CLI is a required dependency; token extracted once and stored in config (not invoked on hot path)
- **Two-token model:** Machine-side `gh` OAuth token (local config) + `VIBESTATS_TOKEN` Actions secret (first install only)
- **Multi-machine architecture:** Per-machine JSON namespacing; aggregation is machine-count-agnostic
- **Installer backfill:** First install must trigger immediate historical backfill (not wait for first `Stop` hook)
- **Community Action:** Users reference `stephenleo/vibestats@v1` — no forking required; updates are automatic within major version

---

## Epic Coverage Validation

### Coverage Matrix

| FR | PRD Requirement (summary) | Epic | Story | Status |
|---|---|---|---|---|
| FR1 | Single shell command install | Epic 6 | #31, #32, #33, #34 | ✅ Covered |
| FR2 | Auto-install `gh` if missing | Epic 6 | #31 | ✅ Covered |
| FR3 | Run `gh auth login` if unauthenticated | Epic 6 | #31 | ✅ Covered |
| FR4 | Create `vibestats-data` private repo | Epic 6 | #32 | ✅ Covered |
| FR5 | Detect existing `vibestats-data`, skip setup | Epic 6 | #33 | ✅ Covered |
| FR6 | Register current machine with unique ID | Epic 6 | #33 | ⚠️ **PARTIAL — gap in first-install path** |
| FR7 | Write Actions workflow to `vibestats-data` | Epic 6 | #32 | ✅ Covered |
| FR8 | Configure `Stop` + `SessionStart` hooks | Epic 6 | #34 | ✅ Covered |
| FR9 | Add `<!-- vibestats-start/end -->` markers to profile README | Epic 6 | #34 | ✅ Covered |
| FR10 | Set `VIBESTATS_TOKEN` Actions secret (first install only) | Epic 6 | #32 | ✅ Covered |
| FR11 | Trigger full historical JSONL backfill post-install | Epic 6 | #34 | ✅ Covered |
| FR12 | Capture activity via `Stop` hook automatically | Epic 3 | #19 | ✅ Covered |
| FR13 | Catch-up sync via `SessionStart` hook | Epic 3 | #20 | ✅ Covered |
| FR14 | Read JSONL from `~/.claude/projects/**/*.jsonl` | Epic 2 | #16 | ✅ Covered |
| FR15 | Throttle `Stop` hook to once per 5 min | Epic 2, 3 | #15, #19 | ✅ Covered |
| FR16 | Push per-machine daily JSON via GitHub Contents API | Epic 2, 3 | #17, #18 | ✅ Covered |
| FR17 | `vibestats sync` — force immediate sync | Epic 3 | #21 | ✅ Covered |
| FR18 | `vibestats sync --backfill` — full historical backfill | Epic 3 | #21 | ✅ Covered |
| FR19 | Log sync failures + warn on `SessionStart` if >24h | Epic 2, 3 | #14, #20 | ✅ Covered |
| FR20 | Action aggregates all machine JSON into daily dataset | Epic 5 | #26 | ✅ Covered |
| FR21 | Action generates `heatmap.svg` (GitHub-style, Claude orange) | Epic 5 | #27 | ✅ Covered |
| FR22 | Action generates `data.json` with aggregated daily dataset | Epic 5 | #26 | ⚠️ **PARTIAL — no explicit AC in Story 5.1 for writing data.json output** |
| FR23 | Action commits `heatmap.svg` + `data.json` to `username/username/vibestats/` | Epic 5 | #29 | ✅ Covered (Story 5.4 commits "outputs") |
| FR24 | Action updates README between markers | Epic 5 | #28 | ✅ Covered |
| FR25 | Action runs on daily cron schedule | Epic 5 | #30 | ✅ Covered |
| FR26 | Manual `workflow_dispatch` trigger | Epic 5 | #30 | ✅ Covered |
| FR27 | `heatmap.svg` publicly accessible via `raw.githubusercontent.com`, embeds without JS | Epic 5 | #27 | ✅ Covered |
| FR28 | Profile README includes link to `vibestats.dev/username` | Epic 5 | #28 | ✅ Covered |
| FR29 | Dashboard fetches `data.json` client-side (no per-user hosting) | Epic 7 | #36 | ✅ Covered |
| FR30 | Dashboard displays `cal-heatmap` activity grid | Epic 7 | #36 | ✅ Covered |
| FR31 | Dashboard hover shows per-day session count + active minutes | Epic 7 | #36 | ✅ Covered |
| FR32 | `vibestats status` — registered machines, sync timestamps, connectivity | Epic 4 | #22 | ✅ Covered |
| FR33 | `vibestats status` — verify auth token validity | Epic 4 | #22 | ✅ Covered |
| FR34 | `vibestats machines list` | Epic 4 | #23 | ✅ Covered |
| FR35 | `vibestats machines remove <id>` | Epic 4 | #23 | ✅ Covered |
| FR36 | `vibestats auth` — refresh local token + Actions secret | Epic 4 | #24 | ✅ Covered |
| FR37 | `vibestats uninstall` — remove hooks + binary + print cleanup instructions | Epic 4 | #25 | ✅ Covered |
| FR38 | Use `gh` CLI as auth provider | Epic 6 | #31 | ✅ Covered |
| FR39 | Store OAuth token in `config.toml` with `600` perms | Epic 2, 6 | #13, #32 | ✅ Covered |
| FR40 | Detect invalid tokens on `SessionStart`, prompt `vibestats auth` | Epic 3, 4 | #20, #24 | ✅ Covered |
| FR41 | Pre-compiled binaries for macOS arm64/x86_64 + Linux x86_64 on Releases | Epic 8 | #39 | ✅ Covered |
| FR42 | Publish community action to GitHub Actions Marketplace | Epic 8 | #41 | ✅ Covered |
| FR43 | `vibestats.dev` docs site (quickstart, CLI ref, architecture, troubleshooting) | Epic 7, 8 | #37, #38 | ✅ Covered |

**Total PRD FRs: 43 | FRs with full story coverage: 41 | FRs with gaps: 2**
**Coverage: 95.3% (41/43 fully covered; 2 with partial/gap)**

---

### Missing / Partially Covered Requirements

#### ⚠️ GAP 1 — FR6: Machine registration missing from first-install path

**FR6:** The installer registers the current machine with a unique identifier in `vibestats-data`

- **Impact:** Story 6.2 (first-install path) covers repo creation, workflow writing, and token setup — but has **no acceptance criterion for writing the initial machine entry to `registry.json`**. Story 6.3 (multi-machine path) explicitly adds a new machine's `machine_id`/`hostname` to `registry.json`, but that logic is scoped to "existing repo detected." On first install there is no AC ensuring `registry.json` is initialised with machine 1.
- **Risk:** A developer implementing Story 6.2 strictly by its ACs would produce a valid install with no machine registered. The Actions aggregator (Story 5.1) skips machines whose status is `purged` — if no entry exists at all, the first push from the binary may still work (registry is optional at read time), but `vibestats machines list` and `vibestats status` would show no machines.
- **Recommendation:** Add an AC to Story 6.2: "Given setup completes, when `vibestats-data/registry.json` is inspected, then it contains the first machine entry with `machine_id`, `hostname`, `status = 'active'`, and `last_seen` timestamp."

#### ⚠️ GAP 2 — FR22: `data.json` generation responsibility unclear

**FR22:** The GitHub Action generates a `data.json` file containing the full aggregated daily activity dataset

- **Impact:** Story 5.1 (`aggregate.py`) describes reading Hive partition files and summing values by date, but its ACs **do not include an output contract** — no AC says "writes merged data as `data.json` to a working directory path." Story 5.4 (`action.yml`) says the action "commits and pushes outputs to `profile-repo`" but doesn't specify that `data.json` is one of those outputs. Story 5.2 (`generate_svg.py`) only generates the SVG.
- **Risk:** A developer implementing `aggregate.py` strictly by its ACs would produce internal aggregation logic with no defined output artifact. The `data.json` that powers `vibestats.dev/[username]` (FR29–31) has no traceable implementation story step.
- **Recommendation:** Add an AC to Story 5.1: "Given aggregation completes, when the working directory is inspected, then `data.json` exists conforming to the public schema: `{ 'generated_at', 'username', 'days': { 'YYYY-MM-DD': { sessions, active_minutes } } }`." Alternatively, this could be a dedicated sub-task of Story 5.4.

---

### NFR Coverage Notes

| NFR | Coverage | Note |
|---|---|---|
| NFR1 (hook latency 2s) | Story 3.2 — `async: true` AC | ✅ |
| NFR2 (throttle 5min) | Stories 2.3, 3.2 | ✅ |
| NFR3 (backfill 60s) | Stories 3.4, 2.4 | ✅ |
| NFR4 (dashboard 3s render) | Story 7.2 — no performance AC | ⚠️ No explicit test criterion for 3-second load target |
| NFR5 (Actions ≤60min/month) | Story 5.5 | ✅ |
| NFR6 (config.toml 600 perms) | Stories 2.1, 6.2, 4.3 | ✅ |
| NFR7 (token scope minimisation) | Story 6.2 | ✅ |
| NFR8 (no raw machine JSON in public repo) | Implied by architecture; no explicit story AC | ⚠️ No story enforces this with an explicit "never commit raw JSON" check |
| NFR9 (private data boundary) | Architectural — implicit | ⚠️ No story AC explicitly validates raw JSON stays in `vibestats-data` |
| NFR10 (hook non-interference / exit 0) | Stories 2.1, 3.1, 3.2 | ✅ |
| NFR11 (silent sync failure) | Stories 2.2, 3.2, 3.3 | ✅ |
| NFR12 (idempotent sync) | Stories 2.3, 3.1, 3.4 | ✅ |
| NFR13 (Actions retry logic) | Story 5.4 exits non-zero on failure; no retry AC in any story | ⚠️ Retry-with-backoff for transient API failures not implemented in any story |
| NFR14 (JSONL format tolerance) | Story 2.4 | ✅ |
| NFR15 (rate limits + exponential backoff) | Story 2.5 | ✅ |
| NFR16 (`gh` v2.0+ check) | Story 6.1 | ✅ |
| NFR17 (Marketplace compatibility) | Stories 5.4, 8.3 | ✅ |

**NFR Coverage: 13/17 fully covered | 4 with gaps/partial coverage (NFR4, NFR8, NFR9, NFR13)**

---

### Coverage Statistics

- **Total PRD FRs:** 43
- **FRs fully covered in epics/stories:** 41
- **FRs with story-level gaps:** 2 (FR6, FR22)
- **FR Coverage:** 95.3%
- **Total PRD NFRs:** 17
- **NFRs fully covered:** 13
- **NFRs with gaps:** 4 (NFR4, NFR8, NFR9, NFR13)
- **NFR Coverage:** 76.5%

---

---

## UX Alignment Assessment

### UX Document Status

**Not Found** — deliberately omitted. The epics document explicitly states: *"No UX design document — vibestats has no bespoke UI beyond the Astro site and cal-heatmap integration."*

### Assessment

UX/UI is implied by the product (two user-facing surfaces: static SVG in README, interactive dashboard), but the decision to skip a formal UX document is **appropriate and well-justified** given:

1. **SVG embed (primary surface):** No interactive design required. Visual spec is fully defined in PRD: GitHub contributions grid shape, Claude orange colour scale (`#fef3e8` → `#f97316`), 52×7 grid. Story 5.2 ACs implement this directly.
2. **Dashboard (secondary surface):** Uses `cal-heatmap` library (off-the-shelf). PRD specifies tooltip content (date, session count, active minutes) and year-toggle behaviour. Story 7.2 ACs implement all specified interactions.
3. **Docs site:** Documentation pages (Story 7.3) and landing page (Story 7.4) have no novel UX complexity — standard Astro site with nav and prose.

### Alignment Issues

None. All UX requirements specified in the PRD (visual design, interactions, dashboard behaviour) are mapped to story ACs in Epic 7.

### Warnings

- ⚠️ **Minor:** The landing page (Story 7.4) has ACs for structure and content but no AC covering visual design quality or mobile responsiveness. For a tool targeting developers during a job search, mobile-friendly README rendering and a polished landing page are brand-credibility concerns. Consider adding a responsive design check.
- ℹ️ **Not blocking:** Year-toggle on the dashboard (year buttons, click re-render without re-fetch) is covered in Story 7.2 ACs but not traced to a PRD FR — it is implied by FR30 and is a good user experience addition.

---

---

## Epic Quality Review

### 1. Epic Structure Validation

#### A. User Value Focus Check

| # | Epic Title | User-Centric? | Can User Benefit Alone? | Verdict |
|---|---|---|---|---|
| 1 | Project Foundation & Schema Definitions | ❌ Technical | ❌ No standalone user value | 🟠 Technical epic |
| 2 | Rust Binary — Foundation Modules | ❌ Technical | ❌ No standalone user value | 🟠 Technical epic |
| 3 | Rust Binary — Sync Engine | ⚠️ Mixed | ✅ Sync works after this epic | 🟡 Acceptable |
| 4 | Rust Binary — CLI Commands | ✅ User-facing CLI | ✅ Yes — users can manage machines | ✅ Good |
| 5 | GitHub Actions Pipeline | ⚠️ Mixed | ✅ Heatmap generated after this epic | 🟡 Acceptable |
| 6 | Bash Installer | ✅ User-facing | ✅ Yes — end-to-end install works | ✅ Good |
| 7 | vibestats.dev Astro Site | ✅ User-facing | ✅ Yes — dashboard + docs live | ✅ Good |
| 8 | CI/CD & Distribution | ❌ Technical | ❌ No standalone user value | 🟠 Technical epic |

**Assessment — Epics 1, 2, 8 are technically-framed:** These epics are infrastructure/tooling milestones rather than user-value deliveries. However, for this project type (greenfield Rust + Astro monorepo), this is a **known and acceptable trade-off**:
- Epic 1 is the mandatory setup required in any statically-compiled greenfield project. ✅ The step file confirms greenfield projects should have "initial project setup story" — Epic 1 satisfies this.
- Epic 2 isolates foundational Rust modules (config, logger, checkpoint, parser, API client) — they must exist before any user-facing feature can function.
- Epic 8 provides distribution that enables Epic 6 (installer). Without it, the tool can't be shipped.
- **Mitigating factor:** The dependency ordering (Epic 1 → 2 → 3+5 → 4+7 → 8 → 6) ensures Epic 6 (the first genuinely shippable user-facing deliverable) comes only after all foundations are in place. This is an appropriate architecture for a tool with this complexity.
- **Verdict:** Not ideal per pure agile principles, but pragmatic and correct for a Rust binary + CI/CD distribution pipeline.

#### B. Epic Independence Validation

| Transition | Forward Dependency Risk | Verdict |
|---|---|---|
| Epic 1 → 2 | Epic 2 depends on schemas/structure from Epic 1. Correct. | ✅ |
| Epic 1 → 5 (parallel with 3) | Epic 5 Python scripts depend on schemas from Epic 1.4 only. Correct. | ✅ |
| Epic 2 → 3 | Epic 3 calls Epic 2 modules (jsonl_parser, github_api, checkpoint). Correct dependency chain. | ✅ |
| Epic 3 → 4 | Epic 4 CLI commands wrap Epic 3 sync logic. Correct. | ✅ |
| Epic 1.3 → 7 | Epic 7 depends on Astro init from Story 1.3. Correct. | ✅ |
| Epic 8 → 6 | **Epic 6 (installer) depends on Epic 8 (binary releases available).** Documented and ordering resolves it. | ✅ (documented) |
| Circularity check | No circular dependencies found. | ✅ |

---

### 2. Story Quality Assessment

#### A. Story Sizing and Independence

**🟠 Story 3.3 (SessionStart hook) — Oversized**
Covers 4 distinct behaviours: (1) catch-up sync, (2) staleness warning, (3) auth error surface + clear, (4) machine retirement detection. Each is independently testable and could reasonably be a separate story. This creates implementation risk: a developer completing "3 of 4 behaviours" cannot close the story. Recommendation: Split into 3.3a (catch-up sync), 3.3b (staleness + auth error warning), 3.3c (machine retirement detection).

**🟡 Story 6.2 (first-install path) — Dense but cohesive**
Covers repo creation + workflow write + `VIBESTATS_TOKEN` setup + local token storage. All are tightly coupled to the "first install" path and cannot be meaningfully split. Acceptable size given the cohesion.

**🟡 Story 5.4 (action.yml) — Orchestration story**
Orchestrates all prior Epic 5 stories into a single composite action. This is inherently dependent on Stories 5.1–5.3 completing first. Within-epic sequential dependency is expected and acceptable.

**All other stories:** Appropriately sized — 3–5 ACs each, independently completable.

#### B. Acceptance Criteria Review

**🟢 Strengths across the epics:**
- Near-universal use of Given/When/Then BDD format
- NFR references in ACs (e.g., "NFR6", "NFR10") — excellent traceability
- Error paths covered: 401 handling in 2.3/2.5, missing file handling in 2.1/2.4, marker-missing error in 5.3
- Quantitative criteria throughout (60s, 2s, 5min, 600 perms)
- Exit-code contracts explicit ("exits 0", "exits non-zero") — critical for hooks and CI

**🟠 Missing: Story 6.2 — no AC for initialising `registry.json` with first machine**
As noted in Gap 1 (FR6), Story 6.2 ACs do not include creating the initial `registry.json` entry. A developer following Story 6.2 strictly would produce a valid first install with no machine registered. This is an AC completeness defect.

**🟡 Story 7.4 (landing page) — thin ACs**
3 ACs cover structure, content, and build success only. Missing:
- No mobile/responsive design check
- No "install command is copyable" accessibility test (clipboard API)
- No error state (e.g., broken image for placeholder SVG)

**🟡 Story 8.2 (Cloudflare Pages deploy) — happy-path only**
3 ACs, all happy path. Missing: "Given the Cloudflare deploy fails, when the workflow exits, then no partial deployment is live and the failure is surfaced in the Actions log."

**🟡 Story 1.1 (CONTRIBUTING.md) — content undefined**
Story 1.1 AC requires `CONTRIBUTING.md` to exist but doesn't specify required content. Given that `CONTRIBUTING.md` is referenced from other stories (e.g., Story 8.3 documents semver behaviour there), its content should be specified.

---

### 3. Dependency Analysis

#### Forward Reference Check: All Clear (with one note)

No story contains a forward reference to a story that hasn't been completed in the defined sequence. The one notable case:

- **Story 3.2 ACs** reference the hook entry `command: vibestats sync` — the `sync` CLI command is implemented in Story 3.4. However, Story 3.2 merely tests that the settings.json *string* is present. The actual functionality exercised by the hook is from Story 3.1 (sync::run). No implementation forward dependency exists. ✅

#### Database/Schema Timing

N/A — no database. The equivalent (JSON/TOML schema definitions) is handled in Story 1.4, which defines all data contracts upfront. For a project where Rust, Python, and JavaScript all share the same data schemas, upfront definition in a shared `docs/schemas.md` is **the correct approach** — not a violation. ✅

---

### 4. Best Practices Compliance Checklist

| Epic | User Value | Independent | Stories Sized Right | No Forward Deps | Clear ACs | FR Traceability |
|---|---|---|---|---|---|---|
| 1 | 🟠 Technical | ✅ | ✅ | ✅ | ✅ | ✅ |
| 2 | 🟠 Technical | ✅ | ✅ | ✅ | ✅ | ✅ |
| 3 | 🟡 Mixed | ✅ | 🟠 Story 3.3 oversized | ✅ | ✅ | ✅ |
| 4 | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| 5 | 🟡 Mixed | ✅ | ✅ | ✅ | 🟠 FR22 gap in Story 5.1 | 🟠 FR22 partial |
| 6 | ✅ | ✅ (after Epic 8) | ✅ | ✅ | 🟠 FR6 gap in Story 6.2 | 🟠 FR6 partial |
| 7 | ✅ | ✅ | ✅ | ✅ | 🟡 Story 7.4 thin | ✅ |
| 8 | 🟠 Technical | ✅ | ✅ | ✅ | 🟡 Story 8.2 happy-path only | ✅ |

---

### 5. Quality Findings Summary

#### 🔴 Critical Violations
None.

#### 🟠 Major Issues

1. **Story 6.2 missing AC for `registry.json` initialisation (FR6 gap)** — A developer following Story 6.2 ACs strictly will produce a first install with no machine entry in `registry.json`. This is a functional defect that would surface as "no machines listed" when the user runs `vibestats machines list` after first install.

2. **Story 5.1 missing AC for `data.json` output artifact (FR22 gap)** — `aggregate.py` has no AC specifying that it writes a `data.json` conforming to the public schema. The dashboard (FR29–31) and action.yml commit step (FR23) both depend on this file existing, but no story owns its creation with a specific AC.

3. **Story 3.3 oversized** — Four distinct, independently testable behaviours in one story creates implementation risk. Recommend splitting.

#### 🟡 Minor Concerns

4. **Story 7.4 thin ACs** — Landing page lacks mobile responsiveness and error state coverage.
5. **Story 8.2 happy-path only ACs** — Cloudflare deploy has no failed-deploy or rollback scenario.
6. **Story 1.1 CONTRIBUTING.md content undefined** — Required to exist but content not specified despite being referenced downstream.
7. **Epics 1, 2, 8 are technically framed** — Acceptable for this project type, but teams should understand no working user feature is deliverable until Epic 3+5 complete.

---

### PRD Completeness Assessment

The PRD is **exceptionally complete and well-structured** for a greenfield developer tool of this complexity. It includes:
- ✅ Clear executive summary with differentiation
- ✅ Detailed user journeys that trace directly to requirements
- ✅ 43 numbered, specific, testable Functional Requirements
- ✅ 17 numbered, specific, measurable Non-Functional Requirements
- ✅ Explicit architecture decision records (ADR-10, ADR-11) in-line
- ✅ Phased scope with clear MVP vs. post-MVP delineation
- ✅ Risk register with mitigations
- ✅ Language matrix with rationale per component

Minor gap: No UX design document, but the PRD adequately describes the visual and interaction requirements for this tool type (SVG shape, colour, hover tooltips).

---

## Summary and Recommendations

### Overall Readiness Status

**🟡 NEEDS MINOR WORK**

The planning is thorough, well-structured, and 95%+ complete. The project has an exceptional PRD with 43 traced FRs, a coherent 8-epic breakdown with proper dependency ordering, and story ACs that consistently reference NFR numbers and use precise BDD format. This is not a "not ready" situation — it is a "fix 2–3 specific story ACs before implementation begins" situation.

No epic redesigns or architectural changes are required. The gaps are surgical AC additions.

---

### Critical Issues Requiring Immediate Action

#### Issue 1 — Story 6.2: Add `registry.json` initialisation AC (FR6 gap)

**Problem:** Story 6.2 (first-install path) has no acceptance criterion for creating the initial `registry.json` entry with the first machine's `machine_id`, `hostname`, `status = "active"`, and `last_seen` timestamp.

**Impact:** Without this AC, a developer implementing Story 6.2 strictly by its ACs could complete the story and ship a first install that passes all ACs, yet `vibestats machines list` and `vibestats status` would show zero machines. The heatmap would still generate (the Rust binary pushes Hive partition files regardless of registry), but the observability story (Journey 3) breaks immediately.

**Fix:** Add to Story 6.2 (#32):
> *"Given setup completes on first install, when `vibestats-data/registry.json` is read, then it contains one entry with this machine's `machine_id`, `hostname`, `status = 'active'`, and `last_seen` timestamp."*

---

#### Issue 2 — Story 5.1: Add `data.json` output artifact AC (FR22 gap)

**Problem:** Story 5.1 (`aggregate.py`, #26) describes reading and summing Hive partition data but has no AC specifying what file it writes as output. Story 5.4's action.yml commits "outputs" without defining what those outputs are. No story owns `data.json` creation with a contractual acceptance criterion.

**Impact:** `data.json` is the file that powers `vibestats.dev/[username]` (FR29–31). If `aggregate.py` doesn't write it, the dashboard fetches nothing. The gap in ownership could cause the file to be implemented as an implicit side effect that's never tested.

**Fix:** Add to Story 5.1 (#26):
> *"Given aggregation completes successfully, when the working directory is inspected, then `data.json` exists at the path the action.yml will commit, conforming to the public schema: `{ 'generated_at': '<ISO 8601 UTC>', 'username': '<str>', 'days': { 'YYYY-MM-DD': { 'sessions': N, 'active_minutes': N } } }`."*

---

#### Issue 3 — Story 3.3: Split into sub-stories or add clear ordering

**Problem:** Story 3.3 (SessionStart hook, #20) covers four distinct, independently testable behaviours: (1) catch-up sync, (2) staleness warning, (3) auth error surface + flag clear, (4) machine retirement detection. This is the largest story in the project by behaviour count.

**Impact:** Implementation risk. A developer can complete 3 of 4 behaviours, all ACs would technically pass if the fourth isn't checked carefully. Code review overhead is high.

**Fix (two options):**
- Split into Stories 3.3a, 3.3b, 3.3c within Epic 3 (preferred)
- Or add explicit section headers to Story 3.3 ACs making the four distinct behaviour blocks explicit and ordering them with "must complete in sequence" notes

---

### Recommended Next Steps

1. **Before writing any code:** Add the two missing ACs to Stories 6.2 (#32) and 5.1 (#26) — these are 2-minute edits to the GitHub issues that prevent functional defects in the delivered MVP.

2. **Before Epic 3 implementation:** Split Story 3.3 (#20) or annotate it clearly. This reduces implementation risk for the SessionStart hook, which is one of the most complex stories in the project.

3. **Before Epic 7 implementation:** Add a responsive design AC to Story 7.4 (#38) — the landing page is a brand-credibility surface during a job-search-targeted launch.

4. **Post-MVP backlog consideration:** Stories for NFR4 (dashboard 3s performance test), NFR8/9 (automated check that raw JSON never reaches the public repo), and NFR13 (Actions retry logic) can be deferred but should be tracked as known gaps.

5. **Proceed with confidence:** Epics 1 → 2 → 3 → 5 can begin immediately. The dependency ordering is sound, the schemas are well-defined, and the ACs for all other stories are implementation-ready.

---

### Issues Found: 10 across 4 categories

| Severity | Count | Category |
|---|---|---|
| 🟠 Major | 3 | FR Coverage (2) + Story sizing (1) |
| 🟡 Minor | 4 | Story AC quality |
| ℹ️ NFR gaps | 4 | NFR4, NFR8, NFR9, NFR13 |
| **Total** | **10** | |

**Address the 3 Major issues before proceeding to implementation. The 4 Minor and 4 NFR items can be addressed during or after MVP delivery.**

---

**Assessment completed:** 2026-04-10
**Assessor:** Implementation Readiness Skill (BMAD)
**Report file:** `_bmad-output/planning-artifacts/implementation-readiness-report-2026-04-10.md`
