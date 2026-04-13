"""Acceptance tests for update_readme.py — TDD Green Phase.

Story 5.3: Implement update_readme.py
AC1: markers present → content replaced with SVG img tag and dashboard link
AC2: markers absent → non-zero exit with clear error message
AC3: identical content → script exits 0 and does NOT write file
"""

import subprocess
import sys
from pathlib import Path

import pytest

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

UPDATE_README = Path(__file__).parent.parent / "update_readme.py"


def _run(args: list[str], readme_content: str, tmp_path: Path) -> subprocess.CompletedProcess:
    """Write readme_content to a temp file and invoke update_readme.py with args."""
    readme = tmp_path / "README.md"
    readme.write_text(readme_content, encoding="utf-8")
    cmd = [sys.executable, str(UPDATE_README)] + args + ["--readme-path", str(readme)]
    return subprocess.run(cmd, capture_output=True, text=True)


README_WITH_MARKERS = """\
# My Profile

<!-- vibestats-start -->
<img src="https://raw.githubusercontent.com/olduser/olduser/main/vibestats/heatmap.svg" alt="vibestats heatmap" />

[View interactive dashboard →](https://vibestats.dev/olduser)
<!-- vibestats-end -->

Some other content.
"""

README_WITHOUT_MARKERS = """\
# My Profile

Just some content, no vibestats markers here.
"""

USERNAME = "testuser"

EXPECTED_IMG = (
    f'<img src="https://raw.githubusercontent.com/{USERNAME}/{USERNAME}'
    '/main/vibestats/heatmap.svg" alt="vibestats heatmap" />'
)
EXPECTED_LINK = f"[View interactive dashboard →](https://vibestats.dev/{USERNAME})"


# ---------------------------------------------------------------------------
# TC-1 (P0): markers present → content replaced correctly (AC1)
# ---------------------------------------------------------------------------


def test_tc1_markers_present_content_replaced(tmp_path: Path) -> None:
    """[P0] AC1: When markers are present, update_readme.py replaces content
    between them with the correct <img> tag and dashboard link."""
    result = _run(["--username", USERNAME], README_WITH_MARKERS, tmp_path)

    assert result.returncode == 0, f"Expected exit 0, got {result.returncode}.\nstdout: {result.stdout}\nstderr: {result.stderr}"

    readme = tmp_path / "README.md"
    updated = readme.read_text(encoding="utf-8")

    assert "<!-- vibestats-start -->" in updated
    assert "<!-- vibestats-end -->" in updated
    assert EXPECTED_IMG in updated
    assert EXPECTED_LINK in updated
    # Old content must be gone
    assert "olduser" not in updated


# ---------------------------------------------------------------------------
# TC-2 (P1): correct raw.githubusercontent.com URL pattern (AC1)
# ---------------------------------------------------------------------------


def test_tc2_correct_raw_githubusercontent_url(tmp_path: Path) -> None:
    """[P1] AC1: The injected <img> src must point to
    raw.githubusercontent.com/<username>/<username>/main/vibestats/heatmap.svg."""
    result = _run(["--username", USERNAME], README_WITH_MARKERS, tmp_path)

    assert result.returncode == 0, f"Expected exit 0.\nstdout: {result.stdout}\nstderr: {result.stderr}"

    readme = tmp_path / "README.md"
    updated = readme.read_text(encoding="utf-8")

    expected_url = (
        f"https://raw.githubusercontent.com/{USERNAME}/{USERNAME}"
        "/main/vibestats/heatmap.svg"
    )
    assert expected_url in updated, (
        f"Expected URL '{expected_url}' not found in updated README.\nContent:\n{updated}"
    )


# ---------------------------------------------------------------------------
# TC-3 (P0): markers absent → non-zero exit with error containing "vibestats markers" (AC2, R-007)
# ---------------------------------------------------------------------------


def test_tc3_markers_absent_nonzero_exit(tmp_path: Path) -> None:
    """[P0] AC2/R-007: When markers are absent, update_readme.py must exit non-zero
    with a clear error message containing 'vibestats markers'."""
    result = _run(["--username", USERNAME], README_WITHOUT_MARKERS, tmp_path)

    assert result.returncode != 0, (
        f"Expected non-zero exit when markers are absent, got {result.returncode}.\n"
        f"stdout: {result.stdout}\nstderr: {result.stderr}"
    )

    combined_output = result.stdout + result.stderr
    assert "vibestats markers" in combined_output, (
        f"Expected error message containing 'vibestats markers' in output.\n"
        f"stdout: {result.stdout}\nstderr: {result.stderr}"
    )


# ---------------------------------------------------------------------------
# TC-4 (P0): identical content → exit 0, file NOT written (AC3, R-004)
# ---------------------------------------------------------------------------


def test_tc4_identical_content_no_op(tmp_path: Path) -> None:
    """[P0] AC3/R-004: When the README already has the correct content between
    the markers, update_readme.py exits 0 without writing the file (idempotent run)."""
    # Build the README that already contains the expected injected block
    already_injected = (
        "# My Profile\n\n"
        "<!-- vibestats-start -->\n"
        f"<img src=\"https://raw.githubusercontent.com/{USERNAME}/{USERNAME}"
        "/main/vibestats/heatmap.svg\" alt=\"vibestats heatmap\" />\n\n"
        f"[View interactive dashboard →](https://vibestats.dev/{USERNAME})\n"
        "<!-- vibestats-end -->\n\n"
        "Some other content.\n"
    )

    readme = tmp_path / "README.md"
    readme.write_text(already_injected, encoding="utf-8")
    mtime_before = readme.stat().st_mtime

    cmd = [sys.executable, str(UPDATE_README), "--username", USERNAME, "--readme-path", str(readme)]
    result = subprocess.run(cmd, capture_output=True, text=True)

    assert result.returncode == 0, (
        f"Expected exit 0 on no-op run, got {result.returncode}.\n"
        f"stdout: {result.stdout}\nstderr: {result.stderr}"
    )

    # File must NOT have been rewritten.
    # mtime equality is reliable here: subprocess.run() is blocking and introduces
    # sufficient time separation; POSIX mtime resolution is 1 ns on macOS/Linux.
    mtime_after = readme.stat().st_mtime
    assert mtime_before == mtime_after, (
        "README file was modified despite identical content — expected no-op.\n"
        f"stdout: {result.stdout}\nstderr: {result.stderr}"
    )

    combined_output = result.stdout + result.stderr
    assert "up to date" in combined_output or "skipping" in combined_output, (
        "Expected 'up to date' or 'skipping' in output for no-op run.\n"
        f"stdout: {result.stdout}\nstderr: {result.stderr}"
    )


# ---------------------------------------------------------------------------
# TC-5 (P1): markers present, content changed → file IS written
# ---------------------------------------------------------------------------


def test_tc5_content_changed_file_is_written(tmp_path: Path) -> None:
    """[P1] When markers are present and the existing content differs from the
    expected injected block, update_readme.py writes the updated file to disk."""
    result = _run(["--username", USERNAME], README_WITH_MARKERS, tmp_path)

    assert result.returncode == 0, (
        f"Expected exit 0 after writing updated content, got {result.returncode}.\n"
        f"stdout: {result.stdout}\nstderr: {result.stderr}"
    )

    readme = tmp_path / "README.md"
    updated = readme.read_text(encoding="utf-8")

    # The file should have been updated with the new username
    assert USERNAME in updated
    assert EXPECTED_IMG in updated
    assert EXPECTED_LINK in updated

    combined_output = result.stdout + result.stderr
    assert "updated" in combined_output.lower(), (
        "Expected 'updated' in output after writing file.\n"
        f"stdout: {result.stdout}\nstderr: {result.stderr}"
    )


# ---------------------------------------------------------------------------
# TC-6 (P1): empty --username exits non-zero with clear stderr message (AC1, Story 9.9)
# ---------------------------------------------------------------------------


def test_tc6_empty_username_exits_nonzero(tmp_path: Path) -> None:
    """[P1] AC1: When --username "" is passed, update_readme.py must exit non-zero
    and write a clear error message to stderr."""
    result = _run(["--username", ""], README_WITH_MARKERS, tmp_path)

    assert result.returncode != 0, (
        f"Expected non-zero exit when --username is empty, got {result.returncode}.\n"
        f"stdout: {result.stdout}\nstderr: {result.stderr}"
    )

    assert "empty" in result.stderr or "--username" in result.stderr, (
        f"Expected error message about empty username in stderr.\n"
        f"stdout: {result.stdout}\nstderr: {result.stderr}"
    )


# ---------------------------------------------------------------------------
# TC-7 (P2): whitespace-only --username exits non-zero (AC1 edge case, Story 9.9)
# ---------------------------------------------------------------------------


def test_tc7_whitespace_only_username_exits_nonzero(tmp_path: Path) -> None:
    """[P2] AC1 edge case: When --username '   ' (whitespace only) is passed,
    update_readme.py must exit non-zero with a clear error message to stderr."""
    result = _run(["--username", "   "], README_WITH_MARKERS, tmp_path)

    assert result.returncode != 0, (
        f"Expected non-zero exit when --username is whitespace-only, got {result.returncode}.\n"
        f"stdout: {result.stdout}\nstderr: {result.stderr}"
    )

    assert "empty" in result.stderr or "--username" in result.stderr, (
        f"Expected error message about empty username in stderr.\n"
        f"stdout: {result.stdout}\nstderr: {result.stderr}"
    )
