# Story 9.3: Fix test_6_2.bats pre-existing failures (pre-launch blocker)

Status: backlog

<!-- GH Issue: #83 | Epic: #80 | PR must include: Closes #83 -->

## Story

As a user about to run `install.sh` for the first time,
I want the full installer test suite to pass with zero failures,
So that I can trust the installer is correct before it runs on my machine.

## Background

Epic 6 closed with `test_6_2.bats` producing failures in the full regression suite. Story 6.4's completion notes stated: "Pre-existing failures in `test_6_2.bats` are unrelated to this story" — but this means the suite was never resolved. The Epic 6 retrospective rated this as a **pre-launch blocker**: "Before any public release of the installer, `test_6_2.bats` must pass cleanly in the full regression suite."

The root cause was not diagnosed during Epic 6 — it may be a test isolation issue, a mock pattern inconsistency between the test file and the final `install.sh` implementation, or a genuine functional regression introduced when Story 6.3 or 6.4 modified the first-install path functions that Story 6.2 tests cover.

Source: Epic 6 retrospective, Challenge #1, Technical Debt #1.

## Acceptance Criteria

1. **Given** the full bats regression suite is run **When** this story is complete **Then** `bats tests/installer/test_6_1.bats tests/installer/test_6_2.bats tests/installer/test_6_3.bats tests/installer/test_6_4.bats` exits with code 0 and reports 0 failures.

2. **Given** any test failure in test_6_2.bats is diagnosed **When** the root cause is identified **Then** the fix addresses the root cause (not just skips the failing test or marks it xfail).

3. **Given** the fix is applied **When** test_6_2.bats is run in isolation **Then** it also passes: `bats tests/installer/test_6_2.bats` exits 0.

4. **Given** the fix modifies `install.sh` **When** the existing passing tests in test_6_1.bats, test_6_3.bats, and test_6_4.bats are re-run **Then** they continue to pass (zero regressions).

## Tasks / Subtasks

- [ ] Task 1: Reproduce the failures
  - [ ] Run `bats tests/installer/test_6_2.bats` and capture the output
  - [ ] Run the full suite `bats tests/installer/test_6_1.bats tests/installer/test_6_2.bats tests/installer/test_6_3.bats tests/installer/test_6_4.bats` and capture the output
  - [ ] List the specific failing test names and failure messages

- [ ] Task 2: Diagnose the root cause
  - [ ] Read `tests/installer/test_6_2.bats` completely
  - [ ] Read the functions tested by test_6_2.bats in `install.sh` (first-install path functions)
  - [ ] Identify whether failures are: (a) test isolation — shared state from another test file polluting test_6_2; (b) mock mismatch — `_gh()` stub doesn't match how the function is now called; (c) functional regression — `install.sh` logic changed after test_6_2 was written and no longer matches test expectations; or (d) environment issue — test assumptions about directory structure don't hold
  - [ ] Document the root cause clearly before applying any fix

- [ ] Task 3: Apply the fix
  - [ ] Fix the root cause (not the symptom)
  - [ ] If the bug is in `install.sh`: fix the logic, ensure no regressions in passing stories
  - [ ] If the bug is in test_6_2.bats: fix the test to match current correct behavior (only if the production code is correct and the test expectation is wrong)
  - [ ] If the bug is test isolation: add `setup` / `teardown` cleanup to prevent cross-test contamination

- [ ] Task 4: Verify full suite passes
  - [ ] Run `bats tests/installer/test_6_2.bats` — must be 0 failures
  - [ ] Run `bats tests/installer/test_6_1.bats tests/installer/test_6_2.bats tests/installer/test_6_3.bats tests/installer/test_6_4.bats` — must be 0 failures across all test files

## Dev Notes

**Key context from Epic 6 development:**

- Story 6.2 implemented the first-install path: `create_vibestats_data_repo()`, `setup_actions_secret()`, `store_machine_token()`, `register_machine()`, `write_workflow()`.
- Story 6.3 implemented the multi-machine path and was "developed in parallel with 6.2 after 6.1 merged" — there is a possibility that 6.3 modified shared functions that test_6_2.bats was already testing.
- Story 6.4 implemented shared final steps: hook configuration, README markers, backfill trigger. Story 6.4's completion notes explicitly noted test_6_2.bats failures as "pre-existing."
- The `_gh()` define-if-not-defined guard (from Story 6.1) is the backbone of testability — if any story changed how `_gh()` is called (different arguments, different mock expectations), that would cause test_6_2.bats failures.

**Test isolation patterns to check:**
- Does test_6_2.bats use a `setup()` function that creates a clean HOME directory?
- Does test_6_2.bats `teardown()` clean up temp files? If not, state from test N could affect test N+1.
- When run after test_6_1.bats or test_6_3.bats, does environment pollution occur?

**Do NOT skip or xfail tests** — the goal is a green suite, not a reduced suite.

## Review Criteria

- `bats tests/installer/test_6_2.bats` exits 0 with all tests passing
- `bats tests/installer/test_6_1.bats tests/installer/test_6_2.bats tests/installer/test_6_3.bats tests/installer/test_6_4.bats` exits 0 with all tests passing
- Root cause is documented in this story file's Dev Agent Record
- No tests were deleted, skipped, or marked xfail as the fix
