# vibestats Schema Reference

This document is the canonical data contract for all vibestats components. Every component — the Rust binary, GitHub Actions Python scripts, Bash installer, Astro site, and GitHub Actions YAML — must implement these schemas exactly.

**Naming convention:** All JSON and TOML fields use `snake_case` everywhere (`active_minutes`, not `activeMinutes`). This ensures compatibility with Rust `serde` default serialization and Python's conventional attribute naming.

**Timestamp format:** All timestamps are ISO 8601 UTC strings (`"YYYY-MM-DDTHH:MM:SSZ"`). Unix timestamps are never used.

---

## 1. Machine Day File

### Location

Stored in the user's `vibestats-data` GitHub repository using a Hive-style partition path:

```
machines/year=YYYY/month=MM/day=DD/harness=<harness>/machine_id=<id>/data.json
```

**Zero-padding rule:** `month` and `day` are always two digits (e.g., `month=04`, `day=09`). This is required for correct lexicographic sort order in glob patterns and Athena/BigQuery partition pruning.

**Example path:**
```
machines/year=2026/month=04/day=10/harness=claude/machine_id=stephens-mbp-a1b2c3/data.json
```

### File Content

```json
{
  "sessions": 4,
  "active_minutes": 87,
  "input_tokens": 12500,
  "output_tokens": 3800,
  "cache_read_tokens": 4200,
  "cache_creation_tokens": 800,
  "models": { "claude-sonnet-4-5": 3800 },
  "longest_session_minutes": 45,
  "message_count": 32,
  "tool_uses": 18
}
```

### Fields

| Field | Type | Constraints | Description |
|-------|------|-------------|-------------|
| `sessions` | integer | ≥ 0 | Number of AI coding sessions active on this machine on this day |
| `active_minutes` | integer | ≥ 0 | Approximate active working minutes (derived from session durations) |
| `input_tokens` | integer | ≥ 0 | Total input tokens across all assistant turns |
| `output_tokens` | integer | ≥ 0 | Total output tokens across all assistant turns |
| `cache_read_tokens` | integer | ≥ 0 | Total cache_read_input_tokens from the usage object |
| `cache_creation_tokens` | integer | ≥ 0 | Total cache_creation_input_tokens from the usage object |
| `models` | object | — | Map of model name → output_tokens for that model. Keys are alphabetically ordered. |
| `longest_session_minutes` | integer | ≥ 0 | Duration in minutes of the longest single session on this day |
| `message_count` | integer | ≥ 0 | Total message count where available (Claude uses `turn_duration.messageCount`; Codex currently reports `0`) |
| `tool_uses` | integer | ≥ 0 | Total tool use count where available (Codex currently reports `0`) |

**Backward compatibility:** Old files with only `sessions` and `active_minutes` remain valid. The aggregation layer uses `.get(field, 0)` / `.get("models", {})` for all new fields, defaulting to zero.

### Design Notes

- Partition metadata is encoded in the path, not the file. This enables Athena/BigQuery external tables with no transformation.
- `harness=<harness>` identifies the source tool. Supported values are currently `claude` and `codex`. The Actions aggregator globs `harness=*` automatically.
- One file per machine per day. Each push is an independent overwrite; no merge of historical data is required.

---

## 2. Public Aggregated `data.json`

### Location

Stored in the user's public GitHub profile repository:

```
username/username/vibestats/data.json
```

For example, user `stephenleo` stores at `stephenleo/stephenleo/vibestats/data.json`.

### Schema

```json
{
  "generated_at": "2026-04-10T14:23:00Z",
  "username": "stephenleo",
  "days": {
    "2026-04-01": {
      "sessions": 3, "active_minutes": 42,
      "input_tokens": 8000, "output_tokens": 2500,
      "cache_read_tokens": 1200, "cache_creation_tokens": 300,
      "models": { "claude-sonnet-4-5": 2500 },
      "longest_session_minutes": 30, "message_count": 18, "tool_uses": 10
    }
  }
}
```

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `generated_at` | string (ISO 8601 UTC) | Timestamp when the GitHub Action produced this file. Format: `YYYY-MM-DDTHH:MM:SSZ`. Never a Unix timestamp. |
| `username` | string | GitHub username of the repo owner |
| `days` | object | Keys are `YYYY-MM-DD` date strings; values are day objects (see below) |

### `days` Value Object

| Field | Type | Aggregation | Description |
|-------|------|-------------|-------------|
| `sessions` | integer | sum | Total sessions across all machines on this date |
| `active_minutes` | integer | sum | Total active minutes across all machines |
| `input_tokens` | integer | sum | Total input tokens across all machines |
| `output_tokens` | integer | sum | Total output tokens across all machines |
| `cache_read_tokens` | integer | sum | Total cache read tokens across all machines |
| `cache_creation_tokens` | integer | sum | Total cache creation tokens across all machines |
| `models` | object | merge-sum | Map of model name → output_tokens (merged across machines) |
| `longest_session_minutes` | integer | max | Longest single session across all machines on this date |
| `message_count` | integer | sum | Total message count across all machines |
| `tool_uses` | integer | sum | Total tool use count across all machines |

### Design Notes

- Full history (all years) in a single file (~73 KB for 5 years). This allows a single client-side fetch with year filtering done client-side.
- Aggregated totals only — no machine IDs, hostnames, or file paths are included. This enforces the private data boundary (NFR8/NFR9).

---

## 3. Local Configuration Files

These files are stored on each machine and are never committed to any repository.

> **Note:** `registry.json` is stored in the remote `vibestats-data` repository, not on the local machine. It is documented in [Section 4](#4-registryjson) below.

### `config.toml`

**Path:** `~/.config/vibestats/config.toml`
**Permissions:** `600` (owner read/write only — enforced by the binary at write time, NFR6)

```toml
oauth_token = "gho_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
machine_id = "stephens-mbp-a1b2c3"
vibestats_data_repo = "stephenleo/vibestats-data"
```

**Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `oauth_token` | string | Machine-side GitHub OAuth token. Obtained via `gh auth token`. Must have Contents write permission scoped to `vibestats-data`. |
| `machine_id` | string | Deterministic ID generated once on first install from hostname + stable UUID. Example: `"stephens-mbp-a1b2c3"`. |
| `vibestats_data_repo` | string | Repository in `"username/vibestats-data"` format where machine day files are pushed. |

---

### `checkpoint.toml`

**Path:** `~/.config/vibestats/checkpoint.toml`

```toml
throttle_timestamp = "2026-04-10T14:23:00Z"
machine_status = "active"
auth_error = false

[date_hashes]
"claude:2026-04-10" = "a3f5c2e1b9d04e8f7c2a1b3d5e6f9012345678901234567890abcdef01234567"
"codex:2026-04-10" = "7b2d1c4e8a093f5c6d2e1b4a9f0e3d8c7a5b6c2d1e0f4a3b9c8d7e6f5a4b3c21"
```

**Fields:**

| Field | Type | Valid Values | Description |
|-------|------|--------------|-------------|
| `throttle_timestamp` | string (ISO 8601 UTC) | Any valid UTC timestamp | Last successful sync time. The Stop hook skips sync if the current time is within 5 minutes of this value (NFR2). |
| `machine_status` | string enum | `"active"` \| `"retired"` \| `"purged"` | Current machine state. If `"retired"`, the Stop hook skips all network calls. |
| `auth_error` | boolean | `true` \| `false` | Set to `true` when the GitHub API returns 401. Triggers a warning at the next SessionStart. Cleared on successful `vibestats auth`. |
| `[date_hashes]` | TOML table | — | Keys are `<harness>:YYYY-MM-DD`; legacy Claude-only `YYYY-MM-DD` keys remain readable. Values are SHA256 hex strings of the last-pushed payload. Used to skip PUT requests when data is unchanged (NFR12). |

---

## 4. `registry.json`

**Location:** Root of the user's `vibestats-data` GitHub repository (`registry.json`)

This file is stored remotely in the `vibestats-data` repo, not on the local machine.

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

**Top-level fields:**

| Field | Type | Description |
|-------|------|-------------|
| `machines` | array | Array of machine objects, one entry per registered machine |

**Per-machine object fields:**

| Field | Type | Valid Values | Description |
|-------|------|--------------|-------------|
| `machine_id` | string | — | Matches the `machine_id` in `config.toml` and the Hive partition path. |
| `hostname` | string | — | Human-readable machine name from the OS. |
| `status` | string enum | `"active"` \| `"retired"` \| `"purged"` | Machine state. `"purged"` means the Hive partition files for this machine have also been deleted from `vibestats-data`. |
| `last_seen` | string (ISO 8601 UTC) | Any valid UTC timestamp | Timestamp of the last successful sync from this machine. |

---

## Naming and Format Rules (Summary)

| Rule | Correct | Wrong |
|------|---------|-------|
| JSON/TOML field names | `snake_case` (`active_minutes`) | `camelCase` (`activeMinutes`) |
| Hive path month/day | Two digits (`month=04`, `day=09`) | One digit (`month=4`, `day=9`) |
| Timestamps | ISO 8601 UTC string (`"2026-04-10T14:23:00Z"`) | Unix timestamp (`1712754180`) |
| Day keys in `days` object | `"YYYY-MM-DD"` (`"2026-04-10"`) | Any other format |
