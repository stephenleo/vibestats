# Story 9.9: Python script hardening — update_readme.py and aggregate.py improvements

Status: backlog

<!-- GH Issue: #89 | Epic: #80 | PR must include: Closes #89 -->

## Story

As a developer maintaining the GitHub Actions pipeline,
I want the Python scripts to have defensive validation and clean test fixtures,
So that misconfigured workflows produce clear errors and the test suite has no dead artifacts.

## Background

Two deferred items from Epic 5 code reviews:

1. **`update_readme.py` `--username` empty-string validation** (deferred from Story 5.3): `argparse` accepts `--username ""`, which produces broken URLs (`https://raw.githubusercontent.com///main/vibestats/heatmap.svg`) without surfacing an error. In the Actions context, `GITHUB_REPOSITORY_OWNER` is always set — but defence-in-depth validation is low effort.

2. **`expected_output/data.json` fixture not referenced by any test** (deferred from Story 5.1): `action/tests/fixtures/expected_output/data.json` exists (required by Story 5.1's task list) but tests assert against the in-file `EXPECTED_DAYS` constant instead of loading the fixture. The fixture is either dead weight or an opportunity for a higher-fidelity test.

Source: `deferred-work.md` (Stories 5.1 and 5.3 reviews), Epic 5 retrospective Technical Debt #2.

## Acceptance Criteria

1. **Given** `update_readme.py` is called with `--username ""` **When** the script runs **Then** it exits with a non-zero code and prints a clear error message like `"Error: --username cannot be empty"` before attempting any file I/O or URL construction.

2. **Given** the `--username` validation is added **When** `update_readme.py` is called with a valid non-empty username **Then** all existing behavior is unchanged.

3. **Given** `action/tests/fixtures/expected_output/data.json` exists but is unused **When** this story is complete **Then** either: (a) the fixture is wired into a new or existing test that loads and validates it, or (b) the fixture is removed and its existence requirement is noted as satisfied-by-removal in the dev notes.

4. **Given** the full Python test suite **When** `python3 -m pytest action/tests/` (or the project's test invocation) is run **Then** all tests pass with 0 failures after this story's changes.

5. **Given** the new `--username` validation test **When** run in isolation **Then** it passes.

## Tasks / Subtasks

- [ ] Task 1: Read `update_readme.py` to understand current argument parsing
  - [ ] Read `action/update_readme.py`
  - [ ] Identify where `--username` is declared via `argparse`
  - [ ] Identify the earliest point in the script where validation can be added without changing existing behavior

- [ ] Task 2: Add empty-string validation to `update_readme.py`
  - [ ] After `args = parser.parse_args()`, add:
    ```python
    if not args.username or not args.username.strip():
        print("Error: --username cannot be empty", file=sys.stderr)
        sys.exit(1)
    ```
  - [ ] The check must catch both `""` (empty string) and strings of whitespace only

- [ ] Task 3: Add a test for the empty-username validation
  - [ ] In `action/tests/test_update_readme.py` (or wherever update_readme.py tests live), add a test:
    - Call `update_readme.py` (via `subprocess.run` or by importing the main logic) with `--username ""`
    - Assert exit code is non-zero
    - Assert stderr contains an informative message
  - [ ] The test should run without creating any temp files or directories

- [ ] Task 4: Decide on the `expected_output/data.json` fixture
  - [ ] Read `action/tests/fixtures/expected_output/data.json`
  - [ ] Read the relevant test(s) in `action/tests/test_aggregate.py` that reference or should reference this fixture
  - [ ] **Option A (wire it):** If the fixture represents a realistic expected aggregation output, add a test that calls `aggregate.py` with controlled inputs and compares the output against the fixture.
  - [ ] **Option B (remove it):** If the fixture is purely structural (same data as the in-code constant), remove `action/tests/fixtures/expected_output/data.json` and document in the Dev Agent Record why it was removed.
  - [ ] Do NOT do both — pick one approach.

- [ ] Task 5: Run the full Python test suite and confirm 0 failures
  - [ ] Run `python3 -m pytest action/tests/` from the repo root (or the project's test invocation)
  - [ ] All tests must pass

## Dev Notes

**On the `--username` validation:**
- The validation should be after `args = parser.parse_args()`, not as a custom `argparse` type. A custom type would produce a less friendly error message.
- Use `sys.stderr` for the error output to be consistent with the script's fail-loudly contract (NFR13).
- Do not modify the `argparse` definition itself (no `required=True` — it's already effectively required; this is a belt-and-suspenders check for the empty string case).

**On the fixture decision:**
- The fixture lives at `action/tests/fixtures/expected_output/data.json`. It was noted in Story 5.1's deferred-work as "kept because story 5.1 task list explicitly requires it to exist."
- If it's removed: the test that checks for its existence (if any) should also be removed.
- If it's wired: the new test should use `aggregate.py`'s public interface, not its internals.

**All stdlib:** No new packages. `pytest` and `subprocess` are already available.

## Review Criteria

- `update_readme.py` exits non-zero with a clear error message when `--username ""` is passed
- A test covers the empty-username case and passes
- `action/tests/fixtures/expected_output/data.json` is either wired into a test or removed (with a note in the Dev Agent Record explaining which and why)
- `python3 -m pytest action/tests/` exits 0 with 0 failures
