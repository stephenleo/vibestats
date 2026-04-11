# Story 5.5: Implement aggregate.yml (user vibestats-data workflow template)

Status: review

<!-- GH Issue: #30 | Epic: #5 | PR must include: Closes #30 -->

## Story

As a vibestats user,
I want a ready-to-use workflow file for my `vibestats-data` repo that runs the community action on a daily cron,
so that my heatmap updates automatically every day.

## Acceptance Criteria

1. **Given** `aggregate.yml` is copied into a user's `vibestats-data/.github/workflows/` **When** it runs **Then** it calls `uses: stephenleo/vibestats@v1` with `token: ${{ secrets.VIBESTATS_TOKEN }}` and `profile-repo: username/username`

2. **Given** the workflow file is reviewed **When** the triggers are inspected **Then** it includes both `schedule: cron` (daily) and `workflow_dispatch` (manual trigger) (FR25, FR26)

3. **Given** the workflow runs over a month **When** Actions minutes are measured **Then** total consumption stays within 60 minutes/month (daily cron only, no per-push triggers) (NFR5)

## Tasks / Subtasks

- [x] Task 1: Create `.github/workflows/aggregate.yml` in the `stephenleo/vibestats` repo (AC: #1, #2, #3)
  - [x] Create `.github/workflows/aggregate.yml` — this file does NOT exist yet; create it at this path
  - [x] Set triggers: `schedule` with a daily cron expression AND `workflow_dispatch` — ONLY these two triggers; do NOT add `push`, `pull_request`, or any other trigger (AC #2, AC #3, NFR5)
  - [x] Use daily cron: `'0 2 * * *'` (2 AM UTC daily — avoids peak hours, keeps monthly consumption well under 60 min)
  - [x] Set `jobs.<job>.runs-on: ubuntu-latest`
  - [x] Single job step: `uses: stephenleo/vibestats@v1` with inputs (AC #1):
    - `token: ${{ secrets.VIBESTATS_TOKEN }}`
    - `profile-repo: ${{ github.repository_owner }}/${{ github.repository_owner }}`
  - [x] Add a clear YAML comment at the top explaining this is a template for users' `vibestats-data` repos
  - [x] Set workflow `name:` to something descriptive, e.g. `Aggregate vibestats data`

- [x] Task 2: Write schema-level test in `action/tests/test_aggregate_yml.py` (AC: #1, #2, #3)
  - [x] Create `action/tests/test_aggregate_yml.py`
  - [x] TC-1 (P0): Parse `aggregate.yml` with `yaml` module and assert `on` key contains ONLY `schedule` and `workflow_dispatch` — no `push`, `pull_request`, `release`, or wildcard triggers (AC #2, AC #3, R-005 from test-design-epic-5.md)
  - [x] TC-2 (P1): Assert `workflow_dispatch` trigger is present (AC #2, FR26)
  - [x] TC-3 (P1): Assert the step uses `stephenleo/vibestats@v1` (or `stephenleo/vibestats` pinned to `v1`) (AC #1)
  - [x] TC-4 (P1): Assert `token` input references `secrets.VIBESTATS_TOKEN` (AC #1)
  - [x] Use `PyYAML` (`import yaml`) to parse the YAML file — it is available in the Actions runner environment; for local test runs install with `pip install pyyaml` (stdlib `json` is not suitable for YAML parsing)
  - [x] Locate `aggregate.yml` relative to the test file: go up to repo root then `.github/workflows/aggregate.yml`
  - [x] Run `python -m pytest action/tests/test_aggregate_yml.py -v` — must pass with 0 failures

## Dev Notes

### What This File Is

`aggregate.yml` is a **workflow template** that lives in the `stephenleo/vibestats` repo at `.github/workflows/aggregate.yml`. Its purpose is to be copied into a user's `vibestats-data/.github/workflows/` directory, where it orchestrates the daily heatmap pipeline by calling the community action (`action.yml`) from the same repo.

This is a YAML workflow file only — no Python logic, no Rust, no shell scripts. The implementation is a single YAML file (~30 lines) plus a test file.

### File to Create

`.github/workflows/aggregate.yml` — this path does NOT currently exist. The `.github/` directory does exist at the repo root (`/Users/stephenleo/Developer/vibestats/.github/`), but `workflows/` inside it is currently empty.

Create this file. Do NOT modify any existing file.

### Reference YAML Structure

```yaml
# aggregate.yml — Copy this file to your vibestats-data/.github/workflows/ directory.
# It runs the vibestats community action daily to aggregate your Claude Code session data
# and update your GitHub profile heatmap automatically.
name: Aggregate vibestats data

on:
  schedule:
    - cron: '0 2 * * *'   # Daily at 02:00 UTC
  workflow_dispatch:        # Allow manual runs

jobs:
  aggregate:
    runs-on: ubuntu-latest
    steps:
      - uses: stephenleo/vibestats@v1
        with:
          token: ${{ secrets.VIBESTATS_TOKEN }}
          profile-repo: ${{ github.repository_owner }}/${{ github.repository_owner }}
```

Key points about this structure:
- `on:` has exactly two keys: `schedule` and `workflow_dispatch` — nothing else
- No `push:` or `pull_request:` trigger — absence of these is required by NFR5 (≤60 min/month) and tested by TC-1
- `profile-repo` uses `github.repository_owner` context so it works for any user who copies the file without editing
- `token` references `secrets.VIBESTATS_TOKEN` — the secret set by the installer (FR10, Story 6.1)

### Architecture Constraints

| Constraint | Source | Impact |
|---|---|---|
| No push/PR triggers | NFR5, ADR-8 | Daily cron only; adding `push:` would exhaust free-tier minutes |
| `uses: stephenleo/vibestats@v1` | architecture.md §Infrastructure | Exact action reference; version tag `v1` (Story 8.3 publishes Marketplace listing) |
| `token` input name | architecture.md §Community GitHub Action | Must be `token` (not `github-token` or `pat`) |
| `profile-repo` input name | architecture.md §Community GitHub Action | Must be `profile-repo` (username/username format) |
| `ubuntu-latest` runner | NFR17 | Required for Marketplace compatibility |
| File location in repo | architecture.md §Complete Project Directory Structure | `.github/workflows/aggregate.yml` |

### Cross-Story Context

- **Story 5.4 (action.yml)**: `aggregate.yml` calls `stephenleo/vibestats@v1`, which resolves to `action.yml`. Story 5.4 defines the inputs (`token`, `profile-repo`) that `aggregate.yml` must pass. Story 5.4 is still in `backlog` — implement `aggregate.yml` to call the action as specified; do not wait for Story 5.4 to be done first.
- **Story 6.1 (installer)**: The installer writes `aggregate.yml` into the user's `vibestats-data/.github/workflows/` directory and sets the `VIBESTATS_TOKEN` secret. Story 6.2 references the exact path `vibestats-data/.github/workflows/aggregate.yml`.
- **Stories 5.1–5.3**: All Python scripts are implemented. `aggregate.yml` invokes them indirectly via `action.yml`.

### Testing Notes

The test uses `PyYAML` (`import yaml`) to parse the workflow file. `PyYAML` is available via `pip install pyyaml` and is present in the GitHub Actions `ubuntu-latest` environment. For local development: `pip install pyyaml`.

Locate `aggregate.yml` from the test at runtime:
```python
import pathlib
REPO_ROOT = pathlib.Path(__file__).parent.parent.parent  # action/tests/ → action/ → repo root
AGGREGATE_YML = REPO_ROOT / ".github" / "workflows" / "aggregate.yml"
```

The test file path follows the established pattern: `action/tests/test_aggregate_yml.py` — consistent with `test_update_readme.py` location.

### Anti-Patterns to Prevent

- Do NOT add `push:` or `pull_request:` triggers — this is the most critical mistake (R-005, NFR5, tested by TC-1)
- Do NOT use `on: push` alone — the workflow MUST have `workflow_dispatch` for manual runs (FR26)
- Do NOT hardcode the username in `profile-repo` — use `${{ github.repository_owner }}` so any user can copy the file without editing
- Do NOT use `actions/checkout` separately — the composite action handles checkout internally
- Do NOT add `permissions:` block at the workflow level unless required — the composite action manages its own permissions
- Do NOT create the file in the wrong location — it must be `.github/workflows/aggregate.yml`, NOT `action/aggregate.yml` (that is `aggregate.py`)
- Do NOT confuse `aggregate.yml` (this story) with `aggregate.py` (Story 5.1)

### File Structure

```
stephenleo/vibestats/
├── .github/
│   └── workflows/
│       └── aggregate.yml        ← CREATE THIS (this story)
├── action/
│   ├── aggregate.py             ← EXISTING (Story 5.1) — do NOT touch
│   ├── generate_svg.py          ← EXISTING (Story 5.2) — do NOT touch
│   ├── update_readme.py         ← EXISTING (Story 5.3) — do NOT touch
│   └── tests/
│       ├── __init__.py          ← EXISTING — do NOT touch
│       ├── test_aggregate_yml.py ← CREATE THIS (this story)
│       ├── test_update_readme.py ← EXISTING — do NOT touch
│       └── fixtures/
├── action.yml                   ← EXISTING stub (Story 5.4 will fill it) — do NOT touch
└── Cargo.toml
```

### References

- Story requirements: [Source: _bmad-output/planning-artifacts/epics.md#Story 5.5]
- FR25 (daily cron), FR26 (workflow_dispatch): [Source: _bmad-output/planning-artifacts/epics.md#Functional Requirements]
- NFR5 (≤60 min/month, daily cron only): [Source: _bmad-output/planning-artifacts/epics.md#NonFunctional Requirements]
- Community Action inputs (`token`, `profile-repo`): [Source: _bmad-output/planning-artifacts/architecture.md#Infrastructure & Deployment]
- File location `.github/workflows/aggregate.yml`: [Source: _bmad-output/planning-artifacts/architecture.md#Complete Project Directory Structure]
- ADR-8 (no per-push triggers): [Source: _bmad-output/planning-artifacts/architecture.md#Cross-Cutting Concerns]
- R-005 (accidental per-push trigger risk, Score 6): [Source: _bmad-output/test-artifacts/test-design-epic-5.md#R-005]
- P0 test: parse `aggregate.yml` and assert no push/PR triggers: [Source: _bmad-output/test-artifacts/test-design-epic-5.md#P0 Tests]
- GH Issue: #30

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

None.

### Completion Notes List

- Created `.github/workflows/aggregate.yml` with exactly two triggers (`schedule` cron `0 2 * * *` and `workflow_dispatch`); no push/PR triggers (NFR5/R-005 compliance). Single job step calls `stephenleo/vibestats@v1` with `token: ${{ secrets.VIBESTATS_TOKEN }}` and `profile-repo: ${{ github.repository_owner }}/${{ github.repository_owner }}`.
- Removed `@pytest.mark.skip` from all 4 tests in `action/tests/test_aggregate_yml.py` (TDD green phase). All 4 tests now pass; 59/59 total tests pass with no regressions.
- All 3 ACs satisfied: AC1 (correct action + inputs), AC2 (schedule + workflow_dispatch triggers), AC3 (daily cron only — no per-push triggers).

### File List

- `.github/workflows/aggregate.yml` (created)
- `action/tests/test_aggregate_yml.py` (modified — removed @pytest.mark.skip decorators)

## Change Log

- 2026-04-12: Implemented story 5.5 — created `.github/workflows/aggregate.yml` (daily cron + workflow_dispatch triggers, calls stephenleo/vibestats@v1); activated 4 schema-level tests in `action/tests/test_aggregate_yml.py` (TDD green phase). All 59 tests pass.
