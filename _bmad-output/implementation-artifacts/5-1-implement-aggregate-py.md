# Story 5.1: Implement aggregate.py

Status: done

<!-- GH Issue: #26 | Epic: #5 | PR must include: Closes #26 -->

## Story

As the GitHub Actions pipeline,
I want an aggregation script that reads all Hive partition files from vibestats-data and produces a single merged daily dataset,
so that per-machine data is combined before SVG generation.

## Acceptance Criteria

1. **Given** Hive partition files exist at `machines/year=YYYY/month=MM/day=DD/harness=claude/machine_id=<id>/data.json` **When** `aggregate.py` runs **Then** it globs all matching paths and sums `sessions` and `active_minutes` by date across all machines and harnesses

2. **Given** multiple machines pushed data for the same date **When** aggregation runs **Then** their values are summed (not overwritten)

3. **Given** a machine's status is `purged` in `registry.json` **When** aggregation runs **Then** its Hive partition files are skipped entirely

4. **Given** the aggregation script encounters any error **When** it fails **Then** it exits non-zero so the GitHub Action surfaces the failure and blocks the commit step

5. **Given** aggregation completes successfully **When** the working directory is inspected **Then** `data.json` exists and conforms to the public schema: `{ "generated_at": "<ISO 8601 UTC>", "username": "<str>", "days": { "YYYY-MM-DD": { "sessions": N, "active_minutes": N } } }` (FR22)

## Tasks / Subtasks

- [x] Task 1: Implement `aggregate.py` core logic (AC: #1, #2, #3, #4, #5)
  - [x] Read `registry.json` from the repo root to get the list of `purged` machine IDs
  - [x] Glob all Hive partition files: `machines/year=*/month=*/day=*/harness=*/machine_id=*/data.json`
  - [x] For each matched path, extract `year`, `month`, `day`, and `machine_id` from the path components
  - [x] Skip any file where `machine_id` is in the purged set (AC #3)
  - [x] Parse each file as JSON and read `sessions` and `active_minutes` fields
  - [x] Sum `sessions` and `active_minutes` by date key `YYYY-MM-DD` using a `collections.defaultdict` (AC #1, #2)
  - [x] Build the output dict conforming to the public schema (AC #5)
  - [x] Set `generated_at` to current UTC time formatted as `YYYY-MM-DDTHH:MM:SSZ` using `datetime.datetime.utcnow()`
  - [x] Read `username` from the `GITHUB_REPOSITORY_OWNER` environment variable (falls back to `GITHUB_REPOSITORY` prefix)
  - [x] Write `data.json` to the current working directory
  - [x] Exit non-zero on any unhandled exception via `sys.exit(1)` in a top-level try/except (AC #4)

- [x] Task 2: Set up test fixtures (AC: #1, #2, #3)
  - [x] Create `action/tests/fixtures/sample_machine_data/` Hive tree with ≥2 active machines and 1 purged machine across ≥3 dates
  - [x] Create `action/tests/fixtures/sample_machine_data/registry.json` with entries for all machine IDs (active and purged)
  - [x] Create `action/tests/fixtures/expected_output/data.json` — expected aggregated output matching the fixtures

- [x] Task 3: Write unit tests in `action/tests/test_aggregate.py` (AC: #1, #2, #3, #4, #5)
  - [x] Test: happy path — two active machines on same date → values summed correctly (P0, R-001)
  - [x] Test: purged machine — its data absent from output (P0, R-001)
  - [x] Test: output schema — keys are exactly `generated_at`, `username`, `days`; `days` values have only `sessions` (int) and `active_minutes` (int) — no machine IDs, paths, or hostnames (P0, R-002)
  - [x] Test: error exit — missing or malformed fixture data causes non-zero exit (P0, R-009)
  - [x] Test: single machine baseline — minimal happy path
  - [x] Test: multiple dates — all dates aggregated correctly
  - [x] Test: empty Hive directory — `days: {}` produced, no error
  - [x] Test: multiple harness dirs (`harness=claude`, `harness=codex`) summed correctly (P2)
  - [x] Run `python -m pytest action/tests/test_aggregate.py -v` — all tests must pass

## Dev Notes

### Critical Architecture Rules for This Story

**stdlib only — absolutely no pip installs.**
Use: `json`, `datetime`, `pathlib`, `collections`, `sys`, `os`. Never `import requests`, `import pandas`, or any third-party package.

**Python Actions scripts fail loudly.**
This is the OPPOSITE of the Rust binary silent-failure contract. Every error path in `aggregate.py` MUST exit non-zero. The GitHub Action surfaces the failure, blocks the commit step, and prevents corrupted outputs from being committed (NFR13). Use a top-level `try/except Exception as e: print(f"Error: {e}", file=sys.stderr); sys.exit(1)`.

**Data boundary enforcement (NFR8/NFR9).**
`data.json` must contain ONLY `generated_at`, `username`, and `days`. The `days` values must contain ONLY `sessions` (int) and `active_minutes` (int). No machine IDs, Hive paths, hostnames, or raw file contents must appear anywhere in the output. This is the security boundary test R-002 verifies.

**Hive path parsing.**
Extract date from path components, not regex on the full path string. Use `pathlib.Path.parts` or `str.split(os.sep)`. Path format:
```
machines/year=2026/month=04/day=10/harness=claude/machine_id=stephens-mbp-a1b2c3/data.json
```
Parse: `year=YYYY` → `YYYY`, `month=MM` → `MM`, `day=DD` → `DD`, then join as `YYYY-MM-DD`.

**Zero-padding in path parsing.**
Hive paths always use zero-padded month and day (`month=04`, not `month=4`). When reconstructing the date key, use the raw padded values directly from the path — no int conversion needed.

**registry.json location.**
The file lives at the vibestats-data repo root: `registry.json`. In the Actions context, `aggregate.py` is run from the checkout root of vibestats-data. If `registry.json` is absent, treat the purged set as empty (all machines included).

**registry.json schema** (from `docs/schemas.md`):
```json
{
  "machines": [
    { "machine_id": "stephens-mbp-a1b2c3", "hostname": "stephens-mbp", "status": "active", "last_seen": "2026-04-10T14:23:00Z" },
    { "machine_id": "old-laptop-x9y8z7", "hostname": "old-laptop", "status": "purged", "last_seen": "2026-03-01T10:00:00Z" }
  ]
}
```
Only `status == "purged"` entries are skipped. `retired` machines continue to be aggregated.

**Public data.json schema** (from `docs/schemas.md`):
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
`generated_at` format: `YYYY-MM-DDTHH:MM:SSZ` (UTC, never Unix timestamp).

**Idempotency.**
Running `aggregate.py` twice on the same Hive data must produce identical `days` content. Only `generated_at` changes between runs (current timestamp). This is acceptable — the dashboard only reads `days`, not `generated_at` for display.

### File Structure

```
action/
├── aggregate.py                   ← IMPLEMENT THIS (currently a stub)
├── generate_svg.py                ← EXISTING stub, do NOT modify
├── update_readme.py               ← EXISTING stub, do NOT modify
└── tests/
    ├── __init__.py                ← EXISTS, do NOT modify
    ├── test_aggregate.py          ← CREATE (unit tests)
    └── fixtures/
        ├── sample_machine_data/   ← CREATE: Hive partition tree
        │   ├── registry.json      ← CREATE: machine registry with active + purged entries
        │   └── machines/
        │       └── year=2026/
        │           ├── month=04/day=09/harness=claude/machine_id=machine-a/data.json
        │           ├── month=04/day=09/harness=claude/machine_id=machine-b/data.json
        │           ├── month=04/day=09/harness=claude/machine_id=machine-purged/data.json
        │           └── month=04/day=10/harness=claude/machine_id=machine-a/data.json
        └── expected_output/
            └── data.json          ← CREATE: expected aggregated output (machine-purged excluded)
```

**IMPORTANT:** `aggregate.py` already exists at `action/aggregate.py` as a stub. Replace the stub body — do NOT create a new file.

### Implementation Skeleton

```python
"""aggregate.py — Aggregates per-machine Hive partition files into daily totals.

Implementation: Epic 5, Story 5.1.
"""

import collections
import datetime
import json
import os
import pathlib
import sys


def load_purged_machines(root: pathlib.Path) -> set:
    """Return set of machine_id strings whose status is 'purged' in registry.json.
    Returns empty set if registry.json is absent or malformed."""
    registry_path = root / "registry.json"
    if not registry_path.exists():
        return set()
    try:
        with open(registry_path) as f:
            registry = json.load(f)
        return {m["machine_id"] for m in registry.get("machines", []) if m.get("status") == "purged"}
    except Exception:
        return set()


def parse_date_from_path(path: pathlib.Path):
    """Extract YYYY-MM-DD from a Hive partition path. Returns str date or None if path is malformed."""
    # ...


def aggregate(root: pathlib.Path, username: str) -> dict:
    """Aggregate all Hive partition files under root/machines/. Returns public data.json dict."""
    # ...


def main():
    root = pathlib.Path(".")
    username = os.environ.get("GITHUB_REPOSITORY_OWNER") or \
               os.environ.get("GITHUB_REPOSITORY", "/").split("/")[0]
    result = aggregate(root, username)
    with open("data.json", "w") as f:
        json.dump(result, f, indent=2)
    print(f"aggregate.py: wrote data.json with {len(result['days'])} day(s)")


if __name__ == "__main__":
    try:
        main()
    except Exception as e:
        print(f"aggregate.py: fatal error: {e}", file=sys.stderr)
        sys.exit(1)
```

### username Derivation

In the GitHub Actions context, `GITHUB_REPOSITORY_OWNER` is always set (e.g., `"stephenleo"`). Use it directly. Fallback to splitting `GITHUB_REPOSITORY` (`"stephenleo/stephenleo"`) on `/` if needed. For tests, mock the environment variable or pass `username` as a parameter.

### Test Setup Guidance

Use `unittest` (stdlib) for tests — do NOT use `pytest`-specific fixtures if they require pip. Standard `unittest.TestCase` with `setUp`/`tearDown` for temporary directories works without any additional install.

Alternatively, `pytest` is acceptable as a dev tool (not installed by the Action runtime) — the test design doc assumes `pytest` for discovery.

For tests that call `aggregate.py` logic (not as subprocess), extract the core aggregation into a function `aggregate(root, username)` so tests can call it directly without environment variable setup.

### Idempotency Test Pattern

```python
def test_idempotent(self):
    result1 = aggregate(fixtures_root, "testuser")
    result2 = aggregate(fixtures_root, "testuser")
    # days content must be identical; only generated_at may differ
    self.assertEqual(result1["days"], result2["days"])
```

### Anti-Patterns to Prevent

- Do NOT use `glob.glob()` with string patterns — use `pathlib.Path.glob()` for cross-platform correctness
- Do NOT use `int(month)` when parsing path components — use the raw zero-padded string directly to preserve `04` not `4`
- Do NOT include machine IDs, hostnames, or file paths in `data.json` output (NFR8/NFR9)
- Do NOT call `sys.exit(0)` explicitly on success — exit naturally; reserve `sys.exit(1)` for error paths only
- Do NOT import any non-stdlib module (no `requests`, `pandas`, `yaml`, `pytest` in the script itself)
- Do NOT overwrite rather than sum when multiple machines share the same date (AC #2)
- Do NOT create a new `aggregate.py` — the stub already exists; replace its body
- Do NOT use `str | None` union syntax in type hints — use `Optional[str]` from `typing` or omit type hints for Python 3.9 compatibility (GitHub Actions `ubuntu-latest` ships Python 3.10+ but `str | None` is not available in 3.9)

### Previous Story Learnings (Epics 1–4)

- All PRs must include `Closes #26` in the PR description
- Python `snake_case` file naming: `test_aggregate.py` (correct), `testAggregate.py` (wrong)
- Fixtures directory `action/tests/fixtures/` already exists (created in story 1.1)
- `action/tests/__init__.py` already exists — do NOT recreate it
- `docs/schemas.md` is the canonical schema reference — consult it for exact field names/types before writing any assertions

### Test Quality Gates (from test-design-epic-5.md)

These P0 tests are required before the PR can be merged:

| Test | Assertion |
|---|---|
| Multi-machine sum | Two machines on same date → `sessions` and `active_minutes` are summed |
| Purged machine skip | Purged machine's data absent from `days` output |
| Output schema | Exactly keys `generated_at`, `username`, `days`; `days` values numeric only |
| Error exit | Malformed input → process exits non-zero |

### Project Structure Notes

- `aggregate.py` lives at `action/aggregate.py` (monorepo root `action/` directory) per architecture.md
- Tests live at `action/tests/test_aggregate.py` per architecture.md
- Fixtures at `action/tests/fixtures/` per architecture.md
- `data.json` output written to current working directory (Actions checkout root of `vibestats-data`)

### References

- Story requirements: [Source: _bmad-output/planning-artifacts/epics.md#Story 5.1]
- Public data.json schema: [Source: docs/schemas.md#2. Public Aggregated data.json]
- registry.json schema: [Source: docs/schemas.md#4-registryjson]
- Hive path format: [Source: docs/schemas.md#1. Machine Day File]
- Python stdlib-only constraint: [Source: _bmad-output/planning-artifacts/architecture.md#Python GitHub Actions Script]
- Exit non-zero contract: [Source: _bmad-output/planning-artifacts/architecture.md#Python Actions scripts — fail loudly]
- Data boundary NFR8/NFR9: [Source: _bmad-output/planning-artifacts/architecture.md#Cross-Cutting Concerns Identified]
- Test design R-001/R-002/R-009: [Source: _bmad-output/test-artifacts/test-design-epic-5.md#Risk Assessment]
- File structure: [Source: _bmad-output/planning-artifacts/architecture.md#Complete Project Directory Structure]
- GH Issue: #26

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

None.

### Completion Notes List

- Implemented `aggregate.py` with stdlib-only dependencies (json, datetime, pathlib, collections, sys, os)
- `load_purged_machines()`: reads registry.json, returns set of purged machine IDs; returns empty set if absent/malformed
- `parse_date_from_path()`: extracts YYYY-MM-DD from Hive partition path using `pathlib.Path.parts`; preserves zero-padding
- `_extract_machine_id_from_path()`: extracts machine_id value from path parts
- `aggregate()`: globs `machines/year=*/month=*/day=*/harness=*/machine_id=*/data.json`, skips purged machines, sums sessions+active_minutes by date using defaultdict
- `main()`: resolves username from env vars, calls aggregate(), writes data.json, wrapped in top-level try/except with sys.exit(1) on failure
- All 21 tests pass (21/21): P0, P1, P2 coverage including multi-machine sum, purged skip, schema validation, error exit, idempotency, empty dirs, registry absent/malformed, multi-harness

### File List

- action/aggregate.py (modified — stub replaced with full implementation)
- action/tests/test_aggregate.py (modified — removed @unittest.skip decorators, entered GREEN phase)

### Review Findings

- [x] [Review][Patch] Replace deprecated `datetime.datetime.utcnow()` with timezone-aware `datetime.datetime.now(datetime.timezone.utc)` [action/aggregate.py:79] — applied
- [x] [Review][Patch] Add explicit `encoding="utf-8"` to all `open()` calls for cross-platform portability [action/aggregate.py:21,73,93] — applied
- [x] [Review][Patch] Narrow `load_purged_machines` exception handling (catch `OSError` + `json.JSONDecodeError` only, validate entry shape) to preserve NFR8/NFR9 data boundary; broad `except Exception` could silently include purged machines if registry had a schema quirk [action/aggregate.py:20-25] — applied (blind+edge+auditor)
- [x] [Review][Patch] Add `__pycache__/`, `*.py[cod]`, `.pytest_cache/`, `.mypy_cache/`, `.ruff_cache/` to `.gitignore` (untracked `action/tests/__pycache__` appeared after test runs) [.gitignore] — applied
- [x] [Review][Patch] Add `timeout=30` to `subprocess.run` calls in error-exit tests to prevent indefinite hangs [action/tests/test_aggregate.py:179-184,207-212] — applied
- [x] [Review][Defer] `action/tests/fixtures/expected_output/data.json` is unused by tests (tests compare against `EXPECTED_DAYS` constant); keep as story task requires the fixture to exist — deferred, no active defect
- [x] [Review][Defer] `aggregate()` reads data files with no size cap; potential OOM risk if malicious machine writes a huge data.json to vibestats-data — deferred, out of story scope (owner-controlled repo)

## Change Log

- 2026-04-11: Story 5.1 implemented — aggregate.py full implementation, all 21 tests passing (claude-sonnet-4-6)
- 2026-04-11: Code review complete — 5 patches applied (deprecated utcnow, utf-8 encoding, narrowed exception handling, python .gitignore entries, subprocess timeouts); 21/21 tests still pass; status → done (claude-opus-4-6)
