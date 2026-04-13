"""ATDD tests for Story 9.9: Python script hardening.

Story 9.9: update_readme.py and aggregate.py improvements
GH Issue: #89 | Epic: #80

GREEN PHASE — implementation complete, all tests pass.

Acceptance Criteria:
  AC1: `update_readme.py --username ""` exits non-zero with a clear error message to stderr.
  AC2: A new test in test_update_readme.py covers the empty-username case.
  AC3: action/tests/fixtures/expected_output/data.json is either wired into a test
       OR removed (with rationale documented).
  AC4: Full Python test suite passes with 0 failures.

Test IDs: 9.9-UNIT-{SEQ}
"""

import subprocess
import sys
from pathlib import Path

import pytest

# ---------------------------------------------------------------------------
# Helpers (mirrors test_update_readme.py conventions)
# ---------------------------------------------------------------------------

UPDATE_README = Path(__file__).parent.parent / "update_readme.py"
FIXTURES_ROOT = Path(__file__).parent / "fixtures"
EXPECTED_OUTPUT_DIR = FIXTURES_ROOT / "expected_output"
EXPECTED_OUTPUT_FIXTURE = EXPECTED_OUTPUT_DIR / "data.json"

README_WITH_MARKERS = """\
# My Profile

<!-- vibestats-start -->
<img src="https://raw.githubusercontent.com/olduser/olduser/main/vibestats/heatmap.svg" alt="vibestats heatmap" />

[View interactive dashboard →](https://vibestats.dev/olduser)
<!-- vibestats-end -->

Some other content.
"""


def _run(args: list[str], readme_content: str, tmp_path: Path) -> subprocess.CompletedProcess:
    """Write readme_content to a temp file and invoke update_readme.py with args."""
    readme = tmp_path / "README.md"
    readme.write_text(readme_content, encoding="utf-8")
    cmd = [sys.executable, str(UPDATE_README)] + args + ["--readme-path", str(readme)]
    return subprocess.run(cmd, capture_output=True, text=True, timeout=30)


# ---------------------------------------------------------------------------
# TC-6 (P1): empty --username exits non-zero with clear stderr message (AC1, AC2)
# ---------------------------------------------------------------------------


def test_tc6_empty_username_exits_nonzero(tmp_path: Path) -> None:
    """[P1] AC1/AC2: When --username "" is passed, update_readme.py must exit non-zero
    and write a clear error message to stderr.

    Expected: returncode != 0; stderr contains 'empty' or '--username'
    """
    result = _run(["--username", ""], README_WITH_MARKERS, tmp_path)

    # ATDD assertion: must exit non-zero
    assert result.returncode != 0, (
        f"Expected non-zero exit when --username is empty, got {result.returncode}.\n"
        f"stdout: {result.stdout}\nstderr: {result.stderr}"
    )

    # ATDD assertion: stderr must contain a meaningful error message
    assert "empty" in result.stderr or "--username" in result.stderr, (
        f"Expected error message about empty username in stderr.\n"
        f"stdout: {result.stdout}\nstderr: {result.stderr}"
    )


# ---------------------------------------------------------------------------
# TC-7 (P2): whitespace-only --username exits non-zero (AC1 edge case)
# ---------------------------------------------------------------------------


def test_tc7_whitespace_only_username_exits_nonzero(tmp_path: Path) -> None:
    """[P2] AC1 edge case: When --username '   ' (whitespace only) is passed,
    update_readme.py must exit non-zero with a clear error message to stderr.

    str.strip() covers all standard ASCII whitespace characters.
    Expected: returncode != 0; stderr contains 'empty' or '--username'
    """
    result = _run(["--username", "   "], README_WITH_MARKERS, tmp_path)

    assert result.returncode != 0, (
        f"Expected non-zero exit when --username is whitespace-only, got {result.returncode}.\n"
        f"stdout: {result.stdout}\nstderr: {result.stderr}"
    )

    assert "empty" in result.stderr or "--username" in result.stderr, (
        f"Expected error message about empty username in stderr.\n"
        f"stdout: {result.stdout}\nstderr: {result.stderr}"
    )


# ---------------------------------------------------------------------------
# 9.9-UNIT-003 (P1): dead fixture expected_output/data.json must be absent (AC3)
# ---------------------------------------------------------------------------


def test_expected_output_fixture_removed() -> None:
    """[P1] AC3: The dead fixture action/tests/fixtures/expected_output/data.json
    must be absent from the repository after Story 9.9 is implemented.

    Rationale for removal (from deferred-work.md and story Dev Notes):
      - test_aggregate.py asserts against the in-file EXPECTED_DAYS constant,
        not this fixture file.
      - The fixture has 'generated_at': 'PLACEHOLDER_REPLACED_IN_TESTS', meaning
        wiring it would require brittle placeholder replacement with no quality gain.
      - Removing the dead file is the correct choice per story 9.9 Task 3.

    This test verifies that the cleanup was actually performed.
    """
    assert not EXPECTED_OUTPUT_FIXTURE.exists(), (
        f"Dead fixture {EXPECTED_OUTPUT_FIXTURE} still exists — Task 3 of Story 9.9 not completed.\n"
        "Delete action/tests/fixtures/expected_output/data.json (and the directory if empty)."
    )


# ---------------------------------------------------------------------------
# 9.9-UNIT-004 (P1): expected_output directory absent after fixture removal (AC3)
# ---------------------------------------------------------------------------


def test_expected_output_directory_removed() -> None:
    """[P1] AC3: After removing data.json, the expected_output/ directory must
    not contain unexpected files.

    heatmap.svg is a known fixture used by test_generate_svg.py (snapshot test)
    and is intentionally retained. data.json was the only dead artifact to remove.
    This test verifies data.json is gone and no other unexpected files were introduced.
    """
    # Known fixtures allowed to remain in expected_output/ (used by other tests)
    KNOWN_FIXTURES = {"heatmap.svg", ".gitkeep"}

    if EXPECTED_OUTPUT_DIR.exists():
        remaining = {f.name for f in EXPECTED_OUTPUT_DIR.iterdir()}
        unexpected = remaining - KNOWN_FIXTURES
        assert len(unexpected) == 0, (
            f"expected_output/ directory contains unexpected files after data.json removal: "
            f"{sorted(unexpected)}\n"
            "Remove or explain any unexpected files."
        )


# ---------------------------------------------------------------------------
# 9.9-UNIT-005 (P3): full Python test suite passes — regression guard (AC4)
# ---------------------------------------------------------------------------


def test_full_pytest_suite_passes() -> None:
    """[P3] AC4: The full Python test suite `cd action && python -m pytest tests/ -v`
    must pass with 0 failures after all story 9.9 changes are applied.

    This meta-test verifies the suite as a whole — it passes once:
    - TC-6 (empty username guard) is implemented
    - data.json fixture is deleted
    - No regressions in existing TC-1 through TC-5 or aggregate tests
    """
    action_dir = UPDATE_README.parent
    # Exclude this file from the recursive run to avoid infinite recursion / timeout.
    # test_9_9_python_hardening.py is the meta-test harness; all its substantive
    # assertions (TC-6, TC-7, fixture removal) are also mirrored in the other test
    # files (test_update_readme.py), so the exclusion does not reduce coverage.
    result = subprocess.run(
        [
            sys.executable, "-m", "pytest", "tests/",
            "--ignore=tests/test_9_9_python_hardening.py",
            "-v", "--tb=short",
        ],
        cwd=str(action_dir),
        capture_output=True,
        text=True,
        timeout=120,
    )

    assert result.returncode == 0, (
        f"Full pytest suite failed with return code {result.returncode}.\n"
        f"stdout:\n{result.stdout}\nstderr:\n{result.stderr}"
    )

    # Confirm TC-6 appears in the output (verifies the new test was collected)
    assert "test_tc6" in result.stdout or "TC-6" in result.stdout or "tc6" in result.stdout.lower(), (
        "TC-6 test was not found in pytest output — it may not have been added to test_update_readme.py.\n"
        f"stdout:\n{result.stdout}"
    )
