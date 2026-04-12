# Story 9.7: aggregate.yml — Add concurrency group to prevent concurrent push conflicts

Status: backlog

<!-- GH Issue: #87 | Epic: #80 | PR must include: Closes #87 -->

## Story

As a vibestats user running the pipeline on multiple machines that share a profile repo,
I want concurrent runs of the aggregate workflow to be serialized,
So that when two machines push data at the same time, the second run doesn't fail due to the first run having already advanced the remote branch.

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

- [ ] Task 1: Read the current `aggregate.yml` and test file
  - [ ] Read `.github/workflows/aggregate.yml`
  - [ ] Read `action/tests/test_aggregate_yml.py` (or wherever aggregate.yml schema tests live)

- [ ] Task 2: Add the `concurrency:` block to `aggregate.yml`
  - [ ] Add at the workflow level (after `name:`, before `on:`):
    ```yaml
    concurrency:
      group: vibestats-${{ github.repository_owner }}
      cancel-in-progress: false
    ```
  - [ ] `cancel-in-progress: false` is preferred over `true` — let the in-progress run finish; queue the next one. This prevents data loss from a cancelled in-flight push.
  - [ ] Place the `concurrency:` block between `name:` and `on:` per standard GitHub Actions YAML convention

- [ ] Task 3: Add a schema test for the concurrency block
  - [ ] In the aggregate.yml test file, add a test that:
    - Loads `aggregate.yml` as a YAML document
    - Asserts `workflow["concurrency"]["group"] == "vibestats-${{ github.repository_owner }}"`
    - Asserts `workflow["concurrency"]["cancel-in-progress"] == False`

- [ ] Task 4: Run the full test suite and confirm all tests pass
  - [ ] `cd action && python3 -m pytest tests/` (or the appropriate test invocation from Story 5.5)
  - [ ] All tests must pass including the new concurrency assertion

## Dev Notes

**Why `cancel-in-progress: false` and not `true`:**
- `cancel-in-progress: true` would kill a run that's already mid-push. This could leave the profile repo in a state where a partial commit was attempted, potentially causing the next run to encounter an unexpected branch state.
- `cancel-in-progress: false` queues the second run. It waits, then executes after the first completes. The profile repo is always updated by both runs, just not simultaneously.

**YAML placement:**
```yaml
name: Aggregate vibestats data

concurrency:
  group: vibestats-${{ github.repository_owner }}
  cancel-in-progress: false

on:
  schedule:
    ...
  workflow_dispatch:
```

**Note on expression syntax in YAML:** `${{ github.repository_owner }}` in a YAML value is a GitHub Actions expression — it's a string literal in YAML (not a variable). The test assertion should match this string exactly.

**Note on the push retry loop:** The `action.yml` push retry loop (3 retries for transient network errors, deferred from Story 5.4) is NOT addressed in this story. The `concurrency:` group is the correct primary mitigation for concurrent-run conflicts; the retry loop handles transient errors. Both mechanisms are independent.

## Review Criteria

- `.github/workflows/aggregate.yml` contains a `concurrency:` block at the workflow level
- `concurrency.group` is `vibestats-${{ github.repository_owner }}`
- `concurrency.cancel-in-progress` is `false`
- New schema test for concurrency passes
- All existing aggregate.yml tests pass
