# Story 9.9: Python script hardening — update_readme.py and aggregate.py improvements

Status: review

<!-- GH Issue: #89 | Epic: #80 | PR must include: Closes #89 -->

## Story

As a developer maintaining the GitHub Actions pipeline,
I want the Python scripts to have defensive validation and clean test fixtures,
so that misconfigured workflows produce clear errors and the test suite has no dead artifacts.

## Acceptance Criteria

1. `update_readme.py --username ""` exits non-zero with a clear error message to stderr.
2. A new test in `action/tests/test_update_readme.py` covers the empty-username case (exit non-zero + message on stderr).
3. `action/tests/fixtures/expected_output/data.json` is either wired into a test OR removed (with rationale documented).
4. Full Python test suite passes with 0 failures: `cd action && python -m pytest tests/ -v`.

## Tasks / Subtasks

- [x] Task 1: Harden `update_readme.py` with empty-username validation (AC: #1)
  - [x] Open `action/update_readme.py` — validation goes in `main()` at line ~38, immediately after `args = parse_args()`
  - [x] Insert the following block BEFORE `readme_path = pathlib.Path(args.readme_path)`:
    ```python
    if not args.username or not args.username.strip():
        print("Error: --username cannot be empty", file=sys.stderr)
        sys.exit(1)
    ```
  - [x] `sys` is already imported — no new imports needed
  - [x] Confirm the message goes to `sys.stderr` (not stdout)

- [x] Task 2: Add TC-6 empty-username test to `action/tests/test_update_readme.py` (AC: #2)
  - [x] Add `test_tc6_empty_username_exits_nonzero` using the existing `_run()` helper
  - [x] Pass `["--username", ""]` as args and any README content (use `README_WITH_MARKERS`)
  - [x] Assert `result.returncode != 0`
  - [x] Assert an informative message appears in `result.stderr` (e.g., `"empty"` or `"--username"`)
  - [x] Optionally add `test_tc7_whitespace_only_username_exits_nonzero` for `--username "   "`
  - [x] Follow TC-1 through TC-5 naming and style conventions

- [x] Task 3: Remove `action/tests/fixtures/expected_output/data.json` (AC: #3)
  - [x] Delete `action/tests/fixtures/expected_output/data.json`
  - [x] If `action/tests/fixtures/expected_output/` is now empty, remove the directory too
  - [x] Verify no test loads from `expected_output/`: `grep -r "expected_output" action/tests/` — should return no results

- [x] Task 4: Verify full test suite passes (AC: #4)
  - [x] Run: `cd action && python -m pytest tests/ -v`
  - [x] All tests must pass — 0 failed, 0 errors
  - [x] New TC-6 (and TC-7 if added) must appear in output and pass

## Dev Notes

### Codebase Locations

| File | Action |
|------|--------|
| `action/update_readme.py` | Add 3-line empty-username guard in `main()` |
| `action/tests/test_update_readme.py` | Add TC-6 (and optionally TC-7) |
| `action/tests/fixtures/expected_output/data.json` | Delete — dead fixture |
| `action/tests/fixtures/expected_output/` | Delete if empty after above |
| `action/tests/test_aggregate.py` | Read-only reference — do not modify |
| `action/aggregate.py` | No changes needed in this story |

### Exact Insertion Point in `update_readme.py`

Current `main()` structure (lines 37–88):
```python
def main() -> None:
    args = parse_args()
    readme_path = pathlib.Path(args.readme_path)   # ← validation goes BEFORE this line
    ...
```

After the change:
```python
def main() -> None:
    args = parse_args()
    if not args.username or not args.username.strip():
        print("Error: --username cannot be empty", file=sys.stderr)
        sys.exit(1)
    readme_path = pathlib.Path(args.readme_path)
    ...
```

### Why `print + sys.exit(1)` not `parser.error()`

The existing error paths in `update_readme.py` (lines 43–45, 48–51, 55–61, 79–82) all use `print(f"ERROR: ...", file=sys.stderr)` + `sys.exit(1)`. Maintain that style for consistency. `parser.error()` would produce an argparse-formatted message which differs from the rest of the script.

### Test File Conventions (from `test_update_readme.py`)

- Test functions use `pytest` with `tmp_path` fixture
- The `_run()` helper at lines 22–27:
  ```python
  def _run(args, readme_content, tmp_path):
      readme = tmp_path / "README.md"
      readme.write_text(readme_content, encoding="utf-8")
      cmd = [sys.executable, str(UPDATE_README)] + args + ["--readme-path", str(readme)]
      return subprocess.run(cmd, capture_output=True, text=True)
  ```
- Existing test IDs: TC-1 through TC-5 → new is TC-6
- Assertion pattern: `assert result.returncode != 0` and `assert "some text" in result.stderr`

### Why Remove the Fixture (Not Wire It)

From `deferred-work.md` (Deferred from story 5-1, lines 74–82):
> Tests assert against the in-file `EXPECTED_DAYS` constant instead of loading the fixture. The fixture is kept because story 5.1 task list explicitly requires it to exist.

The `EXPECTED_DAYS` constant in `test_aggregate.py` (lines 29–33) is semantically identical to the fixture's `days` field. The fixture uses `"generated_at": "PLACEHOLDER_REPLACED_IN_TESTS"` — wiring it would require ugly placeholder replacement logic with no test quality improvement. Removing the dead file is the correct choice.

### No `aggregate.py` Source Changes

Despite the story title, the AC set does NOT include any changes to `aggregate.py` logic. The deferred file-size cap (`deferred-work.md` lines 83–88) is not in scope. Do not add it.

### How to Run Tests

```bash
# From repo root, full suite:
cd action && python -m pytest tests/ -v

# Target update_readme tests only:
cd action && python -m pytest tests/test_update_readme.py -v

# Verify no expected_output references remain after Task 3:
grep -r "expected_output" action/tests/
```

### Anti-Patterns to Avoid

- Do NOT implement the validation inside `parse_args()` or as an `argparse` type — use post-parse validation in `main()`
- Do NOT use `parser.error()` — it produces a different message format from the rest of the script
- Do NOT modify `_run()` helper, `EXPECTED_DAYS`, or `test_aggregate.py`
- Do NOT add any new Python package dependencies (all stdlib)
- Do NOT change `aggregate.py` implementation

### Project Structure Notes

All changes are confined to `action/`:
- One source file edit (`update_readme.py`)
- One test file addition (`test_update_readme.py`)
- One file deletion (`fixtures/expected_output/data.json`)
- Possibly one directory deletion (`fixtures/expected_output/`)

No Rust, no `.github/workflows/`, no `install.sh`, no architecture docs.

### References

- GH Issue #89: Story definition and exact validation pattern
- `_bmad-output/implementation-artifacts/deferred-work.md` lines 74–98: Both deferred items (fixture and empty-username)
- `action/update_readme.py` lines 37–88: `main()` function — insert guard at line ~38
- `action/tests/test_update_readme.py` lines 22–27: `_run()` helper to reuse in TC-6
- `action/tests/test_aggregate.py` lines 29–33: `EXPECTED_DAYS` constant — matches fixture exactly

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

None.

### Completion Notes List

- Task 1: Added 3-line empty-username guard in `update_readme.py` `main()` after `args = parse_args()`. Catches both `""` and whitespace-only strings via `not args.username or not args.username.strip()`. Error printed to `sys.stderr`, consistent with existing error paths in the script.
- Task 2: Added `test_tc6_empty_username_exits_nonzero` and `test_tc7_whitespace_only_username_exits_nonzero` to `test_update_readme.py`. Also removed `@pytest.mark.skip` from all 5 ATDD tests in `test_9_9_python_hardening.py` and updated them to pass (including fixing `test_expected_output_directory_removed` to allow `heatmap.svg` and `.gitkeep` as known fixtures, and `test_full_pytest_suite_passes` to exclude itself from the recursive invocation to avoid timeout).
- Task 3: Chose Option B (remove fixture). `data.json` was dead weight — tests use in-file `EXPECTED_DAYS` constant; the fixture used `"generated_at": "PLACEHOLDER_REPLACED_IN_TESTS"` making it unusable without brittle replacement. Directory `expected_output/` was NOT removed because it contains `heatmap.svg` (used by `test_generate_svg.py` snapshot test) and `.gitkeep`.
- Task 4: Full suite `python3 -m pytest action/tests/ -v` passes — 147 passed, 0 failed, 0 skipped.

### File List

- `action/update_readme.py` (modified — added empty-username validation guard)
- `action/tests/test_update_readme.py` (modified — added TC-6 and TC-7 tests)
- `action/tests/test_9_9_python_hardening.py` (modified — removed @pytest.mark.skip decorators, fixed directory and meta-test logic)
- `action/tests/fixtures/expected_output/data.json` (deleted — dead fixture, rationale in Completion Notes)

## Change Log

- 2026-04-13: Story 9.9 implemented — added empty-username validation to `update_readme.py`, added TC-6/TC-7 tests, removed dead `data.json` fixture, all 147 tests pass.
