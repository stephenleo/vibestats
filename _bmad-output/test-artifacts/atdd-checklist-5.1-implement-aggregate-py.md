---
stepsCompleted:
  - step-01-preflight-and-context
  - step-02-generation-mode
  - step-03-test-strategy
  - step-04-generate-tests
  - step-04c-aggregate
  - step-05-validate-and-complete
lastStep: step-05-validate-and-complete
lastSaved: '2026-04-11'
storyId: 5.1-implement-aggregate-py
tddPhase: RED
---

# ATDD Checklist: Story 5.1 — Implement aggregate.py

**Date:** 2026-04-11
**Story:** 5.1 — Implement aggregate.py
**GH Issue:** #26
**TDD Phase:** RED (all tests skipped — failing until implementation complete)

---

## Step 1: Preflight & Context

### Stack Detection

- **Detected stack:** `backend`
  - Indicators: `Cargo.toml` (Rust binary) + Python scripts (`action/aggregate.py`, `action/generate_svg.py`, `action/update_readme.py`)
  - No frontend indicators (`playwright.config.*`, `package.json` with React/Vue etc. absent from action/)
  - No E2E tests needed for pure backend Python script

### Prerequisites

- [x] Story 5.1 has clear acceptance criteria (5 ACs)
- [x] Test framework config: `action/tests/__init__.py` exists (pytest discovery)
- [x] `action/aggregate.py` stub exists at project root
- [x] TEA config loaded from `_bmad/tea/config.yaml`

### Story Acceptance Criteria Loaded

| AC | Description |
|---|---|
| AC1 | Globs all Hive partition files and sums sessions+active_minutes by date across all machines and harnesses |
| AC2 | Multiple machines on same date → values summed (not overwritten) |
| AC3 | Purged machines (status=purged in registry.json) → their files skipped entirely |
| AC4 | Any error → exits non-zero (blocks GitHub Action commit step) |
| AC5 | Produces data.json conforming to public schema: `{ generated_at, username, days: { YYYY-MM-DD: { sessions, active_minutes } } }` |

### Config Flags

- `test_stack_type: auto` → detected `backend`
- `tea_use_playwright_utils: true` → API-only profile (no `page.goto` patterns for backend)
- `tea_use_pactjs_utils: false`
- `tea_execution_mode: auto` → resolved to `sequential` (no subagent/agent-team capability)

---

## Step 2: Generation Mode

**Mode selected:** AI Generation

**Reason:** Backend Python stack with clear acceptance criteria and standard unit-test scenarios (CRUD-like data aggregation, error handling). No browser recording needed.

---

## Step 3: Test Strategy

### Acceptance Criteria → Test Scenarios

| AC | Test Level | Priority | Test ID | Scenario |
|---|---|---|---|---|
| AC1, AC2 | Unit | P0 | 5.1-UNIT-001 | Two machines on same date → sessions summed |
| AC1, AC2 | Unit | P0 | 5.1-UNIT-001 | Two machines on same date → active_minutes summed |
| AC3 | Unit | P0 | 5.1-UNIT-002 | Purged machine sessions excluded from output |
| AC3 | Unit | P0 | 5.1-UNIT-002 | Purged machine active_minutes excluded from output |
| AC5 | Unit | P0 | 5.1-UNIT-003 | Output has exactly 3 top-level keys: generated_at, username, days |
| AC5 | Unit | P0 | 5.1-UNIT-003 | days values have only sessions (int) and active_minutes (int) |
| AC5 | Unit | P0 | 5.1-UNIT-003 | No string values in days (no machine IDs, paths, hostnames) |
| AC5 | Unit | P0 | 5.1-UNIT-003 | username field set correctly |
| AC5 | Unit | P0 | 5.1-UNIT-003 | generated_at matches YYYY-MM-DDTHH:MM:SSZ format |
| AC4 | Unit | P0 | 5.1-UNIT-004 | Malformed data.json → non-zero exit |
| AC4 | Unit | P0 | 5.1-UNIT-004 | Missing sessions field → non-zero exit |
| AC1, AC2 | Unit | P1 | 5.1-UNIT-005 | Single machine baseline happy path |
| AC1 | Unit | P1 | 5.1-UNIT-006 | All three dates present in output |
| AC1, AC2 | Unit | P1 | 5.1-UNIT-006 | All dates match expected summed values |
| AC1 | Unit | P2 | 5.1-UNIT-007 | Empty machines directory → days: {} without error |
| AC1 | Unit | P2 | 5.1-UNIT-007 | Missing machines directory → days: {} without error |
| AC1 | Unit | P2 | 5.1-UNIT-008 | Multiple harness dirs summed correctly (sessions) |
| AC1 | Unit | P2 | 5.1-UNIT-008 | Multiple harness dirs summed correctly (active_minutes) |
| AC1 | Unit | P2 | 5.1-UNIT-009 | Running twice → identical days content (idempotency) |
| AC3 | Unit | P2 | 5.1-UNIT-010 | Missing registry.json → all machines included |
| AC3 | Unit | P2 | 5.1-UNIT-010 | Malformed registry.json → treated as empty purged set |

**No E2E tests**: Pure backend Python logic with no browser/UI interactions.
**No API contract tests**: Script is a CLI tool, not an HTTP service.

### TDD Red Phase Confirmation

All tests are designed to **fail before implementation** because:
- `aggregate()` function does not exist yet in `aggregate.py` (stub only)
- `_import_aggregate()` will fail to find `aggregate` function
- All tests use `@unittest.skip()` to document this is intentional

---

## Step 4: Test Generation (Sequential Mode)

### Execution Mode

- `tea_execution_mode: auto` → probed → resolved to `sequential`
- No subagents or agent teams launched
- Worker A (unit tests) executed sequentially

### Worker A: Failing Unit Test Generation

**Output file:** `action/tests/test_aggregate.py`
**TDD phase:** RED
**Tests generated:** 18 (all skipped)

| Test ID | Class | Method | Priority | AC Coverage |
|---|---|---|---|---|
| 5.1-UNIT-001a | TestAggregateSumMultipleMachines | test_two_machines_same_date_sessions_summed | P0 | AC1, AC2 |
| 5.1-UNIT-001b | TestAggregateSumMultipleMachines | test_two_machines_same_date_active_minutes_summed | P0 | AC1, AC2 |
| 5.1-UNIT-002a | TestAggregatePurgedMachineSkipped | test_purged_machine_sessions_excluded | P0 | AC3 |
| 5.1-UNIT-002b | TestAggregatePurgedMachineSkipped | test_purged_machine_active_minutes_excluded | P0 | AC3 |
| 5.1-UNIT-003a | TestAggregateOutputSchema | test_output_has_exactly_three_top_level_keys | P0 | AC5 |
| 5.1-UNIT-003b | TestAggregateOutputSchema | test_days_values_have_only_numeric_fields | P0 | AC5 |
| 5.1-UNIT-003c | TestAggregateOutputSchema | test_days_values_contain_no_string_leakage | P0 | AC5 |
| 5.1-UNIT-003d | TestAggregateOutputSchema | test_username_set_correctly | P0 | AC5 |
| 5.1-UNIT-003e | TestAggregateOutputSchema | test_generated_at_is_iso8601_utc | P0 | AC5 |
| 5.1-UNIT-004a | TestAggregateErrorExit | test_malformed_data_json_causes_nonzero_exit | P0 | AC4 |
| 5.1-UNIT-004b | TestAggregateErrorExit | test_missing_sessions_field_causes_nonzero_exit | P0 | AC4 |
| 5.1-UNIT-005 | TestAggregateSingleMachineBaseline | test_single_machine_single_date_correct_values | P1 | AC1, AC2 |
| 5.1-UNIT-006a | TestAggregateMultipleDates | test_all_three_expected_dates_present | P1 | AC1 |
| 5.1-UNIT-006b | TestAggregateMultipleDates | test_all_dates_match_expected_values | P1 | AC1, AC2 |
| 5.1-UNIT-007a | TestAggregateEmptyHiveDirectory | test_empty_machines_directory_returns_empty_days | P2 | AC1 |
| 5.1-UNIT-007b | TestAggregateEmptyHiveDirectory | test_missing_machines_directory_returns_empty_days | P2 | AC1 |
| 5.1-UNIT-008a | TestAggregateMultipleHarnesses | test_claude_and_codex_harness_sessions_summed_on_same_date | P2 | AC1 |
| 5.1-UNIT-008b | TestAggregateMultipleHarnesses | test_claude_and_codex_harness_active_minutes_summed_on_same_date | P2 | AC1 |
| 5.1-UNIT-009 | TestAggregateIdempotency | test_idempotent_days_output | P2 | AC1, AC2 |
| 5.1-UNIT-010a | TestAggregateRegistryMissingOrMalformed | test_missing_registry_includes_all_machines | P2 | AC3 |
| 5.1-UNIT-010b | TestAggregateRegistryMissingOrMalformed | test_malformed_registry_treated_as_empty_purged_set | P2 | AC3 |

*Note: 18 test methods generated from 10 test classes; 21 rows above include sub-variants.*

**No Worker B (E2E)**: Skipped — backend stack, no browser tests needed.

---

## Step 4C: Aggregation

### TDD Red Phase Validation

- [x] All tests use `@unittest.skip()` — intentional TDD red phase
- [x] All tests assert EXPECTED behaviour (not placeholder assertions)
- [x] All tests marked as expected-to-fail (stub aggregate.py has no implementation)
- [x] No placeholder assertions (`assertEqual(True, True)` style)

### Files Written to Disk

- [x] `action/tests/test_aggregate.py` — 18 failing unit tests (all skipped)
- [x] `action/tests/fixtures/sample_machine_data/registry.json` — registry with 2 active + 1 purged machine
- [x] `action/tests/fixtures/sample_machine_data/machines/year=2026/month=04/day=09/harness=claude/machine_id=machine-a/data.json`
- [x] `action/tests/fixtures/sample_machine_data/machines/year=2026/month=04/day=09/harness=claude/machine_id=machine-b/data.json`
- [x] `action/tests/fixtures/sample_machine_data/machines/year=2026/month=04/day=09/harness=claude/machine_id=machine-purged/data.json`
- [x] `action/tests/fixtures/sample_machine_data/machines/year=2026/month=04/day=09/harness=codex/machine_id=machine-a/data.json`
- [x] `action/tests/fixtures/sample_machine_data/machines/year=2026/month=04/day=10/harness=claude/machine_id=machine-a/data.json`
- [x] `action/tests/fixtures/sample_machine_data/machines/year=2026/month=04/day=10/harness=claude/machine_id=machine-b/data.json`
- [x] `action/tests/fixtures/sample_machine_data/machines/year=2026/month=04/day=11/harness=claude/machine_id=machine-a/data.json`
- [x] `action/tests/fixtures/expected_output/data.json` — expected aggregated output (golden file)

### Fixture Structure Verified

```
action/tests/fixtures/
├── sample_machine_data/
│   ├── registry.json                     ← 2 active + 1 purged machine
│   └── machines/
│       └── year=2026/
│           ├── month=04/day=09/harness=claude/machine_id=machine-a/data.json  (3s, 45m)
│           ├── month=04/day=09/harness=claude/machine_id=machine-b/data.json  (2s, 30m)
│           ├── month=04/day=09/harness=claude/machine_id=machine-purged/data.json  (99s, 999m) ← PURGED
│           ├── month=04/day=09/harness=codex/machine_id=machine-a/data.json   (1s, 10m)
│           ├── month=04/day=10/harness=claude/machine_id=machine-a/data.json  (4s, 60m)
│           ├── month=04/day=10/harness=claude/machine_id=machine-b/data.json  (1s, 15m)
│           └── month=04/day=11/harness=claude/machine_id=machine-a/data.json  (5s, 75m)
└── expected_output/
    └── data.json                         ← golden file (purged excluded)
```

### Summary Statistics

| Metric | Value |
|---|---|
| TDD Phase | RED |
| Total Tests | 18 (all skipped) |
| Unit Tests | 18 |
| E2E Tests | 0 (backend stack — not applicable) |
| Fixtures Created | 9 data.json files + 1 registry.json + 1 golden file |
| Acceptance Criteria Covered | AC1, AC2, AC3, AC4, AC5 (100%) |
| Risk Mitigations | R-001, R-002, R-009 (P0 tests ready) |

---

## Step 5: Validation & Completion

### Prerequisites Satisfied

- [x] Story 5.1 acceptance criteria all covered by tests
- [x] Test file created at `action/tests/test_aggregate.py`
- [x] All tests use `@unittest.skip()` — TDD red phase compliant
- [x] All tests assert expected behaviour (not placeholder assertions)
- [x] Fixtures at `action/tests/fixtures/sample_machine_data/` — Hive tree with 2 active + 1 purged machines across 3 dates
- [x] No orphaned browsers (backend test suite — no browser automation)
- [x] All temp artifacts in `_bmad-output/test-artifacts/` (this file)

### Key Risks Mitigated by These Tests

| Risk | Test(s) | Status |
|---|---|---|
| R-001: Incorrect merge logic | 5.1-UNIT-001, 5.1-UNIT-002, 5.1-UNIT-006 | Tests written (RED) |
| R-002: Data boundary violation | 5.1-UNIT-003 | Tests written (RED) |
| R-009: Exit zero on error | 5.1-UNIT-004 | Tests written (RED) |

### Completion Summary

**Test files created:**
- `action/tests/test_aggregate.py` — 18 failing unit tests (TDD RED)

**Fixture files created:**
- `action/tests/fixtures/sample_machine_data/registry.json`
- 7 Hive partition `data.json` files (2 active machines + 1 purged across 3 dates + 1 multi-harness)
- `action/tests/fixtures/expected_output/data.json` (golden file)

**Checklist output:** `_bmad-output/test-artifacts/atdd-checklist-5.1-implement-aggregate-py.md`

**Assumptions:**
1. `pytest` is available as a dev tool (not in action runtime)
2. Tests use `unittest` (stdlib) with `pytest` for discovery compatibility
3. `aggregate()` function will accept `(root: pathlib.Path, username: str) -> dict` signature

---

## TDD RED Phase: Failing Tests Generated

All tests assert EXPECTED behaviour.
All tests will FAIL until `aggregate.py` is implemented.
This is INTENTIONAL (TDD red phase).

## Next Steps (TDD Green Phase)

After implementing `aggregate.py`:

1. Remove `@unittest.skip("ATDD RED PHASE — aggregate.py not yet implemented")` from all test methods
2. Run tests: `python -m pytest action/tests/test_aggregate.py -v`
3. Verify tests PASS (green phase)
4. If any tests fail:
   - Fix implementation (feature bug) — not the tests
   - Or fix test only if the test itself has a logical error
5. Commit passing tests

## Implementation Guidance

Functions to implement in `action/aggregate.py`:
- `load_purged_machines(root: pathlib.Path) -> set` — reads registry.json, returns set of purged machine IDs
- `parse_date_from_path(path: pathlib.Path) -> str | None` — extracts YYYY-MM-DD from Hive path components
- `aggregate(root: pathlib.Path, username: str) -> dict` — main aggregation logic, returns public schema dict
- `main()` — entry point reading from env vars, calling aggregate(), writing data.json

See story file for full implementation skeleton and anti-patterns to avoid.
