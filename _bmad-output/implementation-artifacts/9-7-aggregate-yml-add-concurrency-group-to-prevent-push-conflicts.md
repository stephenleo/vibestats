# Story 9.7: aggregate.yml — Add concurrency group to prevent concurrent push conflicts

Status: done

<!-- GH Issue: #87 | Epic: #80 | PR must include: Closes #87 -->

## Story

As a vibestats user running the pipeline on multiple machines that share a profile repo,
I want concurrent runs of the aggregate workflow to be serialized,
so that when two machines push data at the same time, the second run doesn't fail due to the first run having already advanced the remote branch.

## Background

The Epic 5 retrospective identified that the push retry loop in `action.yml` only handles transient network errors — it does NOT handle the case where the push fails because another concurrent vibestats run advanced the remote branch. The primary mitigation is a `concurrency:` group on `aggregate.yml` that prevents two runs from overlapping for the same repository owner.

This was deferred from Story 5.4's code review and again from Story 5.5. The deferred-work.md entry documents it as a medium-priority item.

Source: `deferred-work.md` (Story 5.4 review), Epic 5 retrospective Technical Debt #1.

## Acceptance Criteria

1. **Given** `aggregate.yml` lacks a `concurrency:` configuration **When** this story is complete **Then** `.github/workflows/aggregate.yml` contains a `concurrency:` block at the workflow level.

2. **Given** the concurrency group is set **When** two runs of `aggregate.yml` are triggered simultaneously for the same repository owner **Then** the second run waits for the first to complete (or is cancelled per `cancel-in-progress` policy) rather than failing with a push conflict.

3. **Given** the `aggregate.yml` test suite (`action/tests/test_aggregate_yml.py`) **When** this story is complete **Then** a new test asserts the presence of the `concurrency:` key in the workflow and its group value.

4. **Given** the existing `aggregate.yml` tests **When** the new concurrency field is added **Then** all existing tests continue to pass.

## Tasks / Subtasks

- [x] Task 1: Read the current `aggregate.yml` and test file (AC: #1, #3)
  - [x] Read `.github/workflows/aggregate.yml`
  - [x] Read `action/tests/test_aggregate_yml.py`

- [x] Task 2: Add the `concurrency:` block to `aggregate.yml` (AC: #1, #2)
  - [x] Add at the workflow level between `name:` and `on:`:
    ```yaml
    concurrency:
      group: vibestats-${{ github.repository_owner }}
      cancel-in-progress: false
    ```
  - [x] `cancel-in-progress: false` — queue the second run, never kill an in-flight push
  - [x] Place the `concurrency:` block between `name:` and `on:` per standard GitHub Actions YAML convention

- [x] Task 3: Add a schema test for the concurrency block (AC: #3)
  - [x] In `action/tests/test_aggregate_yml.py`, add a new test function:
    - Loads `aggregate.yml` as a YAML document using `_load_workflow()`
    - Asserts `workflow["concurrency"]["group"] == "vibestats-${{ github.repository_owner }}"`
    - Asserts `workflow["concurrency"]["cancel-in-progress"] == False`
  - [x] Follow existing test style (test ID `TC-5`, docstring with AC reference)

- [x] Task 4: Run the full test suite and confirm all tests pass (AC: #4)
  - [x] `cd action && python3 -m pytest tests/` (same invocation as Story 5.5)
  - [x] All 4 existing tests + new TC-5 must pass
  - [x] Fix any failures before marking done

## Dev Notes

**Why `cancel-in-progress: false` and not `true`:**
- `cancel-in-progress: true` kills a run mid-push, potentially leaving the profile repo branch in an ambiguous state.
- `cancel-in-progress: false` queues the second run. It waits, then executes after the first completes. Both runs' data is captured.

**YAML placement — exact diff to apply:**

```yaml
# BEFORE:
name: Aggregate vibestats data

on:
  schedule:
  ...

# AFTER:
name: Aggregate vibestats data

concurrency:
  group: vibestats-${{ github.repository_owner }}
  cancel-in-progress: false

on:
  schedule:
  ...
```

**YAML expression syntax note:** `${{ github.repository_owner }}` is a GitHub Actions expression — it is a string literal in YAML, not a variable. The test assertion must match this string exactly (including the `${{` and `}}`).

**Existing test infrastructure to build on:**
- `_load_workflow()` helper in `test_aggregate_yml.py` already parses aggregate.yml — reuse it, do NOT reimplement
- Existing tests: TC-1 (triggers), TC-2 (workflow_dispatch), TC-3 (uses vibestats@v1), TC-4 (VIBESTATS_TOKEN)
- New test: TC-5, priority P1
- PyYAML `safe_load` quirk: bare `on:` key is parsed as Python `True` — existing tests use `workflow.get("on", workflow.get(True, {}))`. The `concurrency:` key is unambiguous and loads as the string `"concurrency"` — no quirk.

**Note on the push retry loop:** The `action.yml` push retry loop (3 retries, deferred from Story 5.4) is NOT in scope for this story. The concurrency group and retry loop are independent mechanisms. Do NOT modify `action.yml`.

**Files NOT to touch:** `action.yml`, `action/aggregate.py`, `action/generate_svg.py`, any Rust source, any Bash installer files.

### Project Structure Notes

- **`aggregate.yml`** lives at `.github/workflows/aggregate.yml` — this is the user-side template workflow that lives in the user's `vibestats-data` repo. In the vibestats source repo it is a template checked in at this path.
- **Test file** lives at `action/tests/test_aggregate_yml.py` — follows the same module structure as `test_aggregate.py`, `test_generate_svg.py` in the same directory.
- Path resolution in test file: `REPO_ROOT = pathlib.Path(__file__).parent.parent.parent` → repo root. `AGGREGATE_YML = REPO_ROOT / ".github" / "workflows" / "aggregate.yml"` — do NOT change this path resolution.
- Test invocation: `cd action && python3 -m pytest tests/` — must be run from the `action/` directory, not repo root.
- Architecture naming: `snake_case` for Python files, consistent with existing `test_aggregate_yml.py`.

### References

- Target file: `.github/workflows/aggregate.yml`
- Test file: `action/tests/test_aggregate_yml.py`
- GitHub Actions concurrency docs: https://docs.github.com/en/actions/writing-workflows/choosing-what-your-workflow-does/control-the-concurrency-of-workflows-and-jobs
- Epic 9 context: `_bmad-output/planning-artifacts/epic-9.md` (Story 9.7, priority MEDIUM)
- Architecture CI/CD section: `_bmad-output/planning-artifacts/architecture.md` (lines 270–278, "Community GitHub Action")
- Story 5.5 (aggregate.yml origin): `_bmad-output/implementation-artifacts/5-5-implement-aggregate-yml-user-vibestats-data-workflow-template.md`
- Deferred work source: Story 5.4 code review

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

None — implementation was straightforward; no debugging needed.

### Completion Notes List

- Added `concurrency:` block to `.github/workflows/aggregate.yml` between `name:` and `on:` with `group: vibestats-${{ github.repository_owner }}` and `cancel-in-progress: false`.
- TC-5 test (`test_tc5_concurrency_block_present_with_correct_group_and_policy`) was already present in `action/tests/test_aggregate_yml.py` (authored at ATDD phase).
- All 5 aggregate_yml tests pass; full suite of 141 tests passes with zero regressions.
- `cancel-in-progress: false` chosen per Dev Notes rationale: queues second run to avoid killing an in-flight push mid-run.

### File List

- `.github/workflows/aggregate.yml` (modified)
- `action/tests/test_aggregate_yml.py` (pre-existing, no changes needed — TC-5 already present)

### Change Log

- 2026-04-13: Added workflow-level `concurrency:` block to `aggregate.yml` to serialize concurrent runs per repository owner, preventing push conflicts (Story 9.7).
