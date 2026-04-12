"""Marketplace-specific acceptance tests for Story 8.3.

Story 8.3: Configure GitHub Actions Marketplace publication
GH Issue: #41 | Epic: #8

Test IDs follow: 8.3-UNIT-{SEQ}

TDD Phase: GREEN — all tests active (pytest.mark.skip removed).
  - TC-1: CONTRIBUTING.md versioning section authored in Task 2
  - TC-2, TC-3: action.yml name/description confirmed non-empty (Story 5.4 done)
  - TC-4: action.yml branding values non-empty (R-005, NFR17, test-design-epic-8.md P1)

Do NOT duplicate assertions already present in test_action_yml.py:
  - branding.icon KEY presence  (5.4-UNIT-007a) — TC-4 asserts VALUE non-empty
  - branding.color KEY presence  (5.4-UNIT-007b) — TC-4 asserts VALUE non-empty
  - runs.using == 'composite'  (5.4-UNIT-002)
  - inputs.token / inputs.profile-repo keys  (5.4-UNIT-003)

Run: python -m pytest action/tests/test_marketplace.py -v
"""

import pathlib
import re

import pytest
import yaml

# ---------------------------------------------------------------------------
# Path resolution (matches test_action_yml.py precedent)
# ---------------------------------------------------------------------------

REPO_ROOT = pathlib.Path(__file__).parent.parent.parent
ACTION_YML = REPO_ROOT / "action.yml"
CONTRIBUTING_MD = REPO_ROOT / "CONTRIBUTING.md"


def _load_action_yml() -> dict:
    """Parse action.yml and return the top-level mapping."""
    with ACTION_YML.open(encoding="utf-8") as fh:
        return yaml.safe_load(fh)


def _load_contributing_md() -> str:
    """Read CONTRIBUTING.md as text."""
    return CONTRIBUTING_MD.read_text(encoding="utf-8")


# ---------------------------------------------------------------------------
# TC-1 (P2): CONTRIBUTING.md documents semver versioning and v1 backward-compat
# (AC #3, test-design-epic-8.md P2)
# ---------------------------------------------------------------------------


def test_tc1_contributing_md_has_versioning_section() -> None:
    """[P2] 8.3-UNIT-001a: CONTRIBUTING.md must contain a versioning section heading.

    AC #3: semver-based versioning documented in CONTRIBUTING.md so that
    existing users pinned to `uses: stephenleo/vibestats@v1` continue to work
    when v2 is released (floating major tag pattern).
    """
    text = _load_contributing_md()
    assert re.search(
        r"##.*version",
        text,
        re.IGNORECASE | re.MULTILINE,
    ), (
        "CONTRIBUTING.md must contain a '## Versioning' (or '## Release Versioning') "
        "section heading (case-insensitive match for '## <anything> version')."
    )


def test_tc1_contributing_md_has_v1_backward_compat_language() -> None:
    """[P2] 8.3-UNIT-001b: CONTRIBUTING.md versioning section must reference v1 backward-compatibility.

    AC #3: v1 must continue to work when v2 is released — the backward-compat
    promise must be documented for contributors.
    """
    text = _load_contributing_md()
    assert "v1" in text, (
        "CONTRIBUTING.md must reference 'v1' to document backward-compatibility "
        "for users pinned to `uses: stephenleo/vibestats@v1`."
    )


# ---------------------------------------------------------------------------
# TC-2 (P1): action.yml name value is non-empty string
# (R-005, FR42, NFR17 — distinct from 5.4-UNIT-001 which checks existence)
# ---------------------------------------------------------------------------


def test_tc2_action_yml_name_is_non_empty() -> None:
    """[P1] 8.3-UNIT-002: action.yml 'name' field must be a non-empty string.

    The GitHub Actions Marketplace requires a non-empty 'name' value for listing.
    Story 5.4 (5.4-UNIT-001) asserts the key exists; this test asserts the VALUE
    is a non-empty string — a distinct assertion required by R-005 and NFR17.
    """
    parsed = _load_action_yml()
    name = parsed.get("name")
    assert isinstance(name, str), (
        f"action.yml 'name' must be a string, got {type(name).__name__!r}"
    )
    assert name.strip(), (
        "action.yml 'name' must be a non-empty string (required for GitHub Marketplace listing, NFR17)."
    )


# ---------------------------------------------------------------------------
# TC-3 (P1): action.yml description value is non-empty string
# (R-005, FR42, NFR17 — distinct from 5.4-UNIT-001 which checks existence)
# ---------------------------------------------------------------------------


def test_tc3_action_yml_description_is_non_empty() -> None:
    """[P1] 8.3-UNIT-003: action.yml 'description' field must be a non-empty string.

    The GitHub Actions Marketplace requires a non-empty 'description' for listing.
    Story 5.4 (5.4-UNIT-001) asserts the key exists; this test asserts the VALUE
    is a non-empty string — a distinct assertion required by R-005 and NFR17.
    """
    parsed = _load_action_yml()
    description = parsed.get("description")
    assert isinstance(description, str), (
        f"action.yml 'description' must be a string, got {type(description).__name__!r}"
    )
    assert description.strip(), (
        "action.yml 'description' must be a non-empty string "
        "(required for GitHub Marketplace listing, NFR17)."
    )


# ---------------------------------------------------------------------------
# TC-4 (P1): action.yml branding values are non-empty strings
# (R-005, NFR17 — Story 5.4 asserts branding key presence; this asserts non-empty values)
# ---------------------------------------------------------------------------


def test_tc4_action_yml_branding_icon_is_non_empty() -> None:
    """[P1] 8.3-UNIT-004a: action.yml 'branding.icon' must be a non-empty string.

    The GitHub Actions Marketplace requires a non-empty 'branding.icon' for listing.
    Story 5.4 (5.4-UNIT-007a) asserts the key exists; this test asserts the VALUE
    is a non-empty string — required by R-005 and NFR17 (test-design-epic-8.md P1).
    """
    parsed = _load_action_yml()
    branding = parsed.get("branding", {})
    icon = branding.get("icon")
    assert isinstance(icon, str), (
        f"action.yml 'branding.icon' must be a string, got {type(icon).__name__!r}"
    )
    assert icon.strip(), (
        "action.yml 'branding.icon' must be a non-empty string "
        "(required for GitHub Marketplace listing, NFR17)."
    )


def test_tc4_action_yml_branding_color_is_non_empty() -> None:
    """[P1] 8.3-UNIT-004b: action.yml 'branding.color' must be a non-empty string.

    The GitHub Actions Marketplace requires a non-empty 'branding.color' for listing.
    Story 5.4 (5.4-UNIT-007b) asserts the key exists; this test asserts the VALUE
    is a non-empty string — required by R-005 and NFR17 (test-design-epic-8.md P1).
    """
    parsed = _load_action_yml()
    branding = parsed.get("branding", {})
    color = branding.get("color")
    assert isinstance(color, str), (
        f"action.yml 'branding.color' must be a string, got {type(color).__name__!r}"
    )
    assert color.strip(), (
        "action.yml 'branding.color' must be a non-empty string "
        "(required for GitHub Marketplace listing, NFR17)."
    )
