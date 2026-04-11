# Story 1.4: Define and Document All JSON and TOML Schemas

Status: review

<!-- GH Issue: #12 | Epic: #1 | PR must include: Closes #12 -->

## Story

As a developer,
I want all shared data schemas formally documented in one place,
so that every component implements the same data contracts without ambiguity.

## Acceptance Criteria

1. **Given** the schemas doc exists at `docs/schemas.md` **When** the machine day file schema is reviewed **Then** it defines leaf content as `{ "sessions": N, "active_minutes": N }` and the Hive path as `machines/year=YYYY/month=MM/day=DD/harness=claude/machine_id=<id>/data.json` with zero-padded month and day.

2. **Given** the schemas doc exists **When** the public data.json schema is reviewed **Then** it defines `{ "generated_at": "<ISO 8601 UTC>", "username": "<str>", "days": { "YYYY-MM-DD": { "sessions": N, "active_minutes": N } } }`.

3. **Given** the schemas doc exists **When** the local config schemas are reviewed **Then** `config.toml` fields (`oauth_token`, `machine_id`, `vibestats_data_repo`), `checkpoint.toml` fields (`throttle_timestamp`, `machine_status`, `auth_error`, `[date_hashes]`), and `registry.json` fields (`machines[].machine_id`, `hostname`, `status`, `last_seen`) are all documented with types and valid values.

## Tasks / Subtasks

- [x] Task 1: Create `docs/` directory and `docs/schemas.md` file (AC: #1, #2, #3)
  - [x] Create `docs/` directory at the monorepo root
  - [x] Create `docs/schemas.md` with all schema sections below

- [x] Task 2: Document machine day file schema (AC: #1)
  - [x] Document Hive partition path: `machines/year=YYYY/month=MM/day=DD/harness=claude/machine_id=<id>/data.json`
  - [x] Specify zero-padding: month and day always two digits (e.g., `month=04`, `day=09`)
  - [x] Document leaf file content: `{ "sessions": N, "active_minutes": N }`
  - [x] Document field types: `sessions` (integer ≥ 0), `active_minutes` (integer ≥ 0)
  - [x] Include a concrete example with real-looking values

- [x] Task 3: Document public data.json schema (AC: #2)
  - [x] Document all top-level fields: `generated_at` (ISO 8601 UTC string), `username` (GitHub username string), `days` (object)
  - [x] Document the `days` sub-object: keys are `YYYY-MM-DD` date strings, values are `{ "sessions": N, "active_minutes": N }`
  - [x] Specify `generated_at` timestamp format: `YYYY-MM-DDTHH:MM:SSZ` (UTC, never Unix timestamp)
  - [x] Document expected file location: `username/username/vibestats/data.json` in the user's public profile repo
  - [x] Include a concrete JSON example

- [x] Task 4: Document local config schemas (AC: #3)
  - [x] Document `config.toml` schema:
    - `oauth_token` (string) — machine-side GitHub OAuth token; obtained via `gh auth token`
    - `machine_id` (string) — deterministic ID from hostname + UUID; e.g., `"stephens-mbp-a1b2c3"`
    - `vibestats_data_repo` (string) — `"username/vibestats-data"` format
    - File path: `~/.config/vibestats/config.toml`
    - Required permissions: `600` (owner read/write only, NFR6)
  - [x] Document `checkpoint.toml` schema:
    - `throttle_timestamp` (string) — ISO 8601 UTC; last successful sync time; Stop hook skips if < 5 min ago (NFR2)
    - `machine_status` (string enum) — `"active"` | `"retired"` | `"purged"`; if `"retired"`, Stop hook skips all network calls
    - `auth_error` (boolean) — `true` if last GitHub API call returned 401; triggers warning on next SessionStart
    - `[date_hashes]` (TOML table) — keys are `YYYY-MM-DD`, values are content hashes (SHA256 hex string) of the last pushed payload; used to skip PUT if data unchanged (NFR12)
    - File path: `~/.config/vibestats/checkpoint.toml`
  - [x] Document `registry.json` schema:
    - Lives in `vibestats-data` repo root: `registry.json`
    - Top-level field: `machines` (array of machine objects)
    - Per-machine fields: `machine_id` (string), `hostname` (string), `status` (string enum: `"active"` | `"retired"` | `"purged"`), `last_seen` (ISO 8601 UTC string)
    - Include a concrete JSON example

- [x] Task 5: Verify `docs/schemas.md` completeness
  - [x] All three AC checks pass: machine day file, public data.json, local config schemas
  - [x] No placeholder text remains
  - [x] All examples use real-looking values (not `"TBD"` or `"TODO"`)

## Dev Notes

### What This Story Delivers

This story creates exactly one new file: `docs/schemas.md`. No code is written — this is a documentation-only story. All downstream epics (Rust binary, Python Actions, Astro site) depend on this document as the canonical contract reference.

**Output file:** `docs/schemas.md` (new file, monorepo root `docs/` directory)

### Why This Story Exists

All five system components (Rust binary, Python Actions, Bash installer, Astro site, GitHub Actions YAML) must agree on the same field names, path formats, and data types. Without a single authoritative reference, developers will independently guess and diverge. This document is the single source of truth that prevents cross-component field-name mismatches.

### Critical Schema Rules (Must Be in docs/schemas.md)

These rules come directly from `architecture.md` and MUST be reflected in the schema documentation:

**JSON field naming: `snake_case` everywhere**
- `sessions`, `active_minutes`, `generated_at`, `machine_id`, `last_updated`, `last_seen`
- No `camelCase` — this breaks Python/Rust `serde` compatibility
- Rust uses serde default serialization (snake_case struct fields, no rename needed)
- JavaScript accesses as `data.active_minutes` (not `data.activeMinutes`)

**Date/timestamp format: ISO 8601 UTC only**
- Day keys: `"YYYY-MM-DD"` (e.g., `"2026-04-10"`)
- Timestamps: `"YYYY-MM-DDTHH:MM:SSZ"` (e.g., `"2026-04-10T14:23:00Z"`)
- Never Unix timestamps — breaks human readability and Athena partition detection

**Hive path zero-padding: always two digits**
- `month=04` not `month=4`, `day=09` not `day=9`
- Critical for correct lexicographic sort order in glob patterns and partition pruning

### Schemas to Document

#### 1. Machine Day File (in `vibestats-data`)

**Path format:**
```
machines/year=YYYY/month=MM/day=DD/harness=claude/machine_id=<id>/data.json
```

**Example path:**
```
machines/year=2026/month=04/day=10/harness=claude/machine_id=stephens-mbp-a1b2c3/data.json
```

**Leaf file content:**
```json
{ "sessions": 4, "active_minutes": 87 }
```

**Field definitions:**
- `sessions` (integer, ≥ 0): number of Claude Code sessions active on this machine on this day
- `active_minutes` (integer, ≥ 0): approximate active working minutes (derived from session durations)

**Design rationale (include in docs):**
- Partition metadata is encoded in the path, not the file — enables Athena/BigQuery external tables with no transformation
- `harness=claude` enables future multi-tool support (Codex, Cursor, Copilot) with zero schema changes; Actions aggregator globs `harness=*` automatically
- One file per machine per day — each push is an independent overwrite; no merge of historical data required

#### 2. Public Aggregated data.json (in `username/username/vibestats/`)

**Schema:**
```json
{
  "generated_at": "2026-04-10T14:23:00Z",
  "username": "stephenleo",
  "days": {
    "2026-04-01": { "sessions": 3, "active_minutes": 42 },
    "2026-04-10": { "sessions": 4, "active_minutes": 87 }
  }
}
```

**Field definitions:**
- `generated_at` (string, ISO 8601 UTC): timestamp when the GitHub Action produced this file
- `username` (string): GitHub username of the repo owner
- `days` (object): keys are `YYYY-MM-DD` date strings; values are `{ "sessions": N, "active_minutes": N }`

**Design rationale (include in docs):**
- Full history all years in single file (~73KB for 5 years) — single client-side fetch, filter by year client-side
- Aggregated totals only — no machine IDs, hostnames, file paths (NFR8/NFR9: private data boundary)

#### 3. Local Config Files

**`~/.config/vibestats/config.toml`** (permissions: `600`):
```toml
oauth_token = "gho_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
machine_id = "stephens-mbp-a1b2c3"
vibestats_data_repo = "stephenleo/vibestats-data"
```

Fields:
- `oauth_token` (string): machine-side GitHub OAuth token; obtained via `gh auth token`; scoped to `vibestats-data` Contents write
- `machine_id` (string): deterministic ID from hostname + stable UUID (generated once on first install)
- `vibestats_data_repo` (string): `"username/vibestats-data"` format

**IMPORTANT for schema doc:** Mention that the binary enforces `600` permissions at write time, not documentation time (NFR6).

---

**`~/.config/vibestats/checkpoint.toml`**:
```toml
throttle_timestamp = "2026-04-10T14:23:00Z"
machine_status = "active"
auth_error = false

[date_hashes]
"2026-04-10" = "a3f5c2e1b9d04..."
"2026-04-09" = "7b2d1c4e8a093..."
```

Fields:
- `throttle_timestamp` (string, ISO 8601 UTC): last successful sync time; Stop hook skips sync if within 5 minutes (NFR2)
- `machine_status` (string enum): `"active"` | `"retired"` | `"purged"` — if `"retired"`, Stop hook skips all network calls
- `auth_error` (boolean): set to `true` on 401 response from GitHub API; triggers warning at next SessionStart; cleared on successful `vibestats auth`
- `[date_hashes]` (TOML table): keys are `YYYY-MM-DD` date strings; values are SHA256 hex strings of last-pushed payload; used to skip PUT if data unchanged (NFR12)

---

**`registry.json`** (in `vibestats-data` repo root):
```json
{
  "machines": [
    {
      "machine_id": "stephens-mbp-a1b2c3",
      "hostname": "Stephens-MacBook-Pro.local",
      "status": "active",
      "last_seen": "2026-04-10T14:23:00Z"
    },
    {
      "machine_id": "work-ubuntu-d4e5f6",
      "hostname": "work-ubuntu",
      "status": "retired",
      "last_seen": "2026-03-15T09:10:00Z"
    }
  ]
}
```

Fields (per machine object):
- `machine_id` (string): matches the `machine_id` in `config.toml` and the Hive path
- `hostname` (string): human-readable machine name from OS
- `status` (string enum): `"active"` | `"retired"` | `"purged"` — `"purged"` means Hive partition files also deleted
- `last_seen` (string, ISO 8601 UTC): timestamp of last successful sync from this machine

### File Structure

```
vibestats/
└── docs/
    └── schemas.md    ← ONLY file created by this story
```

**Do NOT** create any other files. `docs/` does not exist yet — create the directory as part of this story.

### Story 1.1 Learnings

From Story 1.1 review:
- Story 1.1 notes that `docs/schemas.md` is explicitly delegated to Story 1.4 (see Story 1.1 Project Structure Notes: "docs/schemas.md → Story 1.4"). The `docs/` directory was not created in 1.1, so this story must create it.
- Git tracking: `docs/schemas.md` will be a new tracked file; run `git status` to confirm before finalizing.
- Use ISO 8601 UTC timestamps consistently throughout the document — no Unix timestamps anywhere (established pattern from Story 1.1 and architecture).

### No Tests Required

This story creates only a documentation file. No executable logic. Verify correctness by reviewing `docs/schemas.md` against the acceptance criteria checklist:
- [ ] `docs/schemas.md` exists and is tracked by git
- [ ] Machine day file section: Hive path with zero-padded month/day present
- [ ] Machine day file section: leaf content `{ "sessions": N, "active_minutes": N }` present
- [ ] Public data.json section: all three top-level fields documented (`generated_at`, `username`, `days`)
- [ ] `config.toml` section: all three fields with types (`oauth_token`, `machine_id`, `vibestats_data_repo`) + `600` permission note
- [ ] `checkpoint.toml` section: all four fields with types (`throttle_timestamp`, `machine_status`, `auth_error`, `[date_hashes]`)
- [ ] `registry.json` section: all four per-machine fields with types (`machine_id`, `hostname`, `status`, `last_seen`)

### Cross-Component Impact

This document is the primary reference for:
- **Epic 2 Story 2.1** (`config.rs`): must implement `config.toml` exactly as documented here
- **Epic 2 Story 2.3** (`checkpoint.rs`): must implement `checkpoint.toml` exactly as documented here
- **Epic 2 Story 2.4** (`jsonl_parser.rs`): produces `{ "sessions": N, "active_minutes": N }` per day
- **Epic 2 Story 2.5** (`github_api.rs`): pushes to Hive path exactly as documented here
- **Epic 5 Story 5.1** (`aggregate.py`): reads Hive partition files and produces public `data.json`
- **Epic 7 Story 7.2** (`[username].astro`): fetches and renders public `data.json`

Incorrect schema documentation here creates bugs in 6+ downstream stories — get it right.

### Anti-Patterns to Avoid in Schema Doc

- `camelCase` field names (must be `snake_case` everywhere — `active_minutes` not `activeMinutes`)
- Un-padded month/day in Hive paths (`month=4` is wrong — must be `month=04`)
- Unix timestamps (must be ISO 8601 UTC strings — `"2026-04-10T14:23:00Z"` not `1712754180`)
- Missing permission note for `config.toml` (must document `600` perms — critical security requirement NFR6)
- Leaving `[date_hashes]` undocumented (checkpoint module depends on this for idempotent sync NFR12)

### References

- Story 1.4 acceptance criteria: [Source: epics.md#Story 1.4: Define and document all JSON and TOML schemas]
- Machine day file schema: [Source: architecture.md#Data Architecture — Hive partition file layout]
- Public data.json schema: [Source: architecture.md#Data Architecture — Public aggregated schema]
- Local config files: [Source: architecture.md#Data Architecture — Local checkpoint]
- `registry.json` machine states: [Source: epics.md#Additional Requirements — registry.json machine states]
- `checkpoint.toml` fields: [Source: epics.md#Additional Requirements — Local files]
- JSON naming conventions: [Source: architecture.md#Naming Patterns — JSON fields]
- Zero-padding requirement: [Source: architecture.md#Format Patterns — Hive path zero-padding]
- Date/timestamp format: [Source: architecture.md#Format Patterns — Dates and timestamps]
- NFR6 (config.toml 600 perms): [Source: epics.md#NonFunctional Requirements]
- NFR8/NFR9 (private data boundary): [Source: epics.md#NonFunctional Requirements]
- NFR12 (idempotent sync): [Source: epics.md#NonFunctional Requirements]
- docs/ directory delegation from Story 1.1: [Source: implementation-artifacts/1-1-initialize-monorepo-directory-structure.md#Project Structure Notes]
- GH Issue: #12

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

### Completion Notes List

- Created `docs/` directory at monorepo root (did not exist before this story).
- Created `docs/schemas.md` as the single canonical schema reference for all vibestats components.
- Documented all three schema groups: machine day file (Hive partition path + leaf content), public aggregated `data.json`, and local config files (`config.toml`, `checkpoint.toml`, `registry.json`).
- All field types, valid values, constraints, and concrete examples are present.
- Zero-padding rule for Hive path month/day enforced and documented.
- `config.toml` `600` permission requirement (NFR6) documented.
- `[date_hashes]` idempotent sync mechanism (NFR12) documented.
- No TBD/TODO placeholders present.
- No tests required — documentation-only story.
- All acceptance criteria verified via grep checks on the output file.

### File List

- docs/schemas.md (new)

## Change Log

- 2026-04-11: Created `docs/schemas.md` with all schema definitions for machine day file, public data.json, and local config files (`config.toml`, `checkpoint.toml`, `registry.json`). All acceptance criteria satisfied.

## Status

review
