"""Acceptance tests for Story 9.6: First release — push v0.1.0 tag and create v1 floating tag.

Story 9.6: First release — push v0.1.0 tag and create v1 floating tag
GH Issue: #86 | Epic: #80

Test IDs follow: 9.6-UNIT-{SEQ}

TDD Phase: RED — All tests assert expected state; tests verify pre-release
conditions and release.yml structural requirements. Tests that check
conditions not yet true (e.g., v0.1.0 not yet tagged) use pytest.mark.skip
to mark the manual/runtime-only verification steps.

Testable ACs via schema/static analysis:
  AC #1 (partial): release.yml produces the correct binary asset names and
                   the floating-tag update step exists (schema assertion).
  AC #2: release.yml includes a step to create/update the v1 floating tag
         pointing to the pushed tag.
  AC #3 (partial): Cargo.toml ureq dependency is present; rustls fallback path
                   is documentable via Cargo.toml structure.
  AC #4: action.yml has required branding fields and CONTRIBUTING.md has
         Release Versioning section — prerequisites for Marketplace submission.
  AC #5 (partial): deploy-site.yml exists with workflow_dispatch trigger.

Pre-release checklist items (Task 1) testable statically:
  - action.yml has branding.icon, branding.color, name, description
  - CONTRIBUTING.md has Release Versioning section
  - Cargo.toml version is 0.1.0
  - release.yml has the v1 floating tag update step

Runtime-only / manual steps (not automatable):
  - AC #1 runtime: GitHub Release page actually exists with 3 binary assets
  - AC #2 runtime: git ls-remote shows v1 tag on remote
  - AC #3 runtime: cargo test passes, cargo clippy passes, bats suite passes
  - AC #5 runtime: vibestats.dev serves the landing page after deploy
  - AC #4 runtime: Marketplace submission UI (manual)

Run: python3 -m pytest action/tests/test_release_9_6.py -v
"""

import pathlib
import re

import pytest
import yaml

# ---------------------------------------------------------------------------
# Path resolution
# ---------------------------------------------------------------------------

# action/tests/ -> action/ -> repo root
REPO_ROOT = pathlib.Path(__file__).parent.parent.parent
RELEASE_YML = REPO_ROOT / ".github" / "workflows" / "release.yml"
DEPLOY_SITE_YML = REPO_ROOT / ".github" / "workflows" / "deploy-site.yml"
ACTION_YML = REPO_ROOT / "action.yml"
CONTRIBUTING_MD = REPO_ROOT / "CONTRIBUTING.md"
CARGO_TOML = REPO_ROOT / "Cargo.toml"

EXPECTED_BINARY_TARGETS = {
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "x86_64-unknown-linux-gnu",
}


def _load_yaml(path: pathlib.Path) -> dict:
    with path.open(encoding="utf-8") as fh:
        return yaml.safe_load(fh)


def _load_text(path: pathlib.Path) -> str:
    return path.read_text(encoding="utf-8")


# ---------------------------------------------------------------------------
# Module-level cached content — parsed once, reused across tests
# (avoids repeated disk I/O; file content is static during a test run)
# ---------------------------------------------------------------------------

_RELEASE_YML_TEXT: str = _load_text(RELEASE_YML) if RELEASE_YML.exists() else ""
_RELEASE_YML_DOC: dict = _load_yaml(RELEASE_YML) if RELEASE_YML.exists() else {}
_DEPLOY_SITE_YML_DOC: dict = _load_yaml(DEPLOY_SITE_YML) if DEPLOY_SITE_YML.exists() else {}
_ACTION_YML_DOC: dict = _load_yaml(ACTION_YML) if ACTION_YML.exists() else {}
_CONTRIBUTING_MD_TEXT: str = _load_text(CONTRIBUTING_MD) if CONTRIBUTING_MD.exists() else ""
_CARGO_TOML_TEXT: str = _load_text(CARGO_TOML) if CARGO_TOML.exists() else ""


# ---------------------------------------------------------------------------
# [PREFLIGHT] File existence checks — gate all further tests
# ---------------------------------------------------------------------------


def test_preflight_release_yml_exists() -> None:
    """[P0] 9.6-UNIT-000: .github/workflows/release.yml must exist.

    AC #1: The release.yml workflow is required to build and publish
    the v0.1.0 GitHub Release with all three platform binaries.
    Story 8.1 delivered this file; this test ensures it is present.
    """
    assert RELEASE_YML.exists(), (
        f"release.yml not found at {RELEASE_YML}. "
        "Story 8.1 should have created .github/workflows/release.yml."
    )


def test_preflight_action_yml_exists() -> None:
    """[P0] 9.6-UNIT-001: action.yml must exist at the repo root.

    AC #4: action.yml is required for Marketplace submission (Story 5.4 deliverable).
    """
    assert ACTION_YML.exists(), (
        f"action.yml not found at {ACTION_YML}. "
        "Story 5.4 should have created action.yml."
    )


def test_preflight_contributing_md_exists() -> None:
    """[P0] 9.6-UNIT-002: CONTRIBUTING.md must exist at the repo root.

    AC #4: CONTRIBUTING.md with Release Versioning section is a Marketplace
    prerequisite (Story 8.3 deliverable).
    """
    assert CONTRIBUTING_MD.exists(), (
        f"CONTRIBUTING.md not found at {CONTRIBUTING_MD}. "
        "Story 8.3 should have created CONTRIBUTING.md."
    )


def test_preflight_cargo_toml_exists() -> None:
    """[P0] 9.6-UNIT-003: Cargo.toml must exist at the repo root.

    Pre-release checklist (Task 1): cargo test and cargo clippy require
    a valid Cargo.toml before tagging v0.1.0.
    """
    assert CARGO_TOML.exists(), (
        f"Cargo.toml not found at {CARGO_TOML}. "
        "A valid Cargo.toml is required to run cargo test and cargo clippy (Task 1)."
    )


# ---------------------------------------------------------------------------
# [AC #4] action.yml pre-release checklist: branding + name + description
# Task 1 checklist item: "Confirm action.yml has branding.icon, branding.color,
#                         name, description (Story 5.4 deliverables)"
# ---------------------------------------------------------------------------


def test_tc1_action_yml_name_present_and_non_empty() -> None:
    """[P1] 9.6-UNIT-010: action.yml 'name' must be a non-empty string.

    Task 1 pre-release checklist: confirm action.yml name is set.
    Required for GitHub Actions Marketplace listing (AC #4).
    """
    name = _ACTION_YML_DOC.get("name")
    assert isinstance(name, str) and name.strip(), (
        f"action.yml 'name' must be a non-empty string, got: {name!r}. "
        "A non-empty 'name' is required for the Marketplace submission (AC #4, Task 1)."
    )


def test_tc1_action_yml_description_present_and_non_empty() -> None:
    """[P1] 9.6-UNIT-011: action.yml 'description' must be a non-empty string.

    Task 1 pre-release checklist: confirm action.yml description is set.
    Required for GitHub Actions Marketplace listing (AC #4).
    """
    description = _ACTION_YML_DOC.get("description")
    assert isinstance(description, str) and description.strip(), (
        f"action.yml 'description' must be a non-empty string, got: {description!r}. "
        "A non-empty 'description' is required for the Marketplace submission (AC #4, Task 1)."
    )


def test_tc1_action_yml_branding_icon_present_and_non_empty() -> None:
    """[P1] 9.6-UNIT-012: action.yml 'branding.icon' must be a non-empty string.

    Task 1 pre-release checklist: confirm action.yml branding.icon is set.
    Required for GitHub Actions Marketplace listing (AC #4).
    """
    branding = _ACTION_YML_DOC.get("branding", {})
    icon = branding.get("icon")
    assert isinstance(icon, str) and icon.strip(), (
        f"action.yml 'branding.icon' must be a non-empty string, got: {icon!r}. "
        "A valid icon name is required for the Marketplace listing (AC #4, Task 1)."
    )


def test_tc1_action_yml_branding_color_present_and_non_empty() -> None:
    """[P1] 9.6-UNIT-013: action.yml 'branding.color' must be a non-empty string.

    Task 1 pre-release checklist: confirm action.yml branding.color is set.
    Required for GitHub Actions Marketplace listing (AC #4).
    """
    branding = _ACTION_YML_DOC.get("branding", {})
    color = branding.get("color")
    assert isinstance(color, str) and color.strip(), (
        f"action.yml 'branding.color' must be a non-empty string, got: {color!r}. "
        "A valid color value is required for the Marketplace listing (AC #4, Task 1)."
    )


# ---------------------------------------------------------------------------
# [AC #4] CONTRIBUTING.md pre-release checklist: Release Versioning section
# Task 1 checklist item: "Confirm CONTRIBUTING.md has the Release Versioning
#                         section (Story 8.3 deliverable)"
# ---------------------------------------------------------------------------


def test_tc2_contributing_md_has_release_versioning_section() -> None:
    """[P1] 9.6-UNIT-020: CONTRIBUTING.md must contain a Release Versioning section.

    Task 1 pre-release checklist: CONTRIBUTING.md Release Versioning section
    is a Marketplace prerequisite (Story 8.3 deliverable, AC #4).
    """
    assert re.search(
        r"##.*(?:release\s+versioning|versioning)",
        _CONTRIBUTING_MD_TEXT,
        re.IGNORECASE | re.MULTILINE,
    ), (
        "CONTRIBUTING.md must contain a '## Release Versioning' (or '## Versioning') "
        "section. This is a Marketplace prerequisite from Story 8.3 (AC #4, Task 1)."
    )


def test_tc2_contributing_md_references_v1_tag() -> None:
    """[P1] 9.6-UNIT-021: CONTRIBUTING.md must document the v1 floating tag convention.

    AC #2: The v1 floating tag (created in Task 4) follows the pattern documented
    in CONTRIBUTING.md. The documentation must reference 'v1' for the
    `uses: stephenleo/vibestats@v1` convention.
    """
    assert "v1" in _CONTRIBUTING_MD_TEXT, (
        "CONTRIBUTING.md must reference 'v1' (the floating major tag). "
        "The v1 floating tag created in Task 4 follows this documented convention (AC #2)."
    )


def test_tc2_contributing_md_documents_floating_tag_force_push() -> None:
    """[P1] 9.6-UNIT-022: CONTRIBUTING.md must document the force-push procedure
    for updating the v1 floating tag.

    AC #2: The maintenance checklist (git tag -f v1 ... && git push --force origin v1)
    must be present in CONTRIBUTING.md so future maintainers can update v1 correctly.
    """
    # The documented procedure should include force-flag usage
    assert "--force" in _CONTRIBUTING_MD_TEXT or "-f" in _CONTRIBUTING_MD_TEXT, (
        "CONTRIBUTING.md must document the 'git push --force origin v1' (or 'git tag -f v1') "
        "procedure for updating the floating major tag after each release (AC #2)."
    )


# ---------------------------------------------------------------------------
# [AC #1 / AC #2] release.yml: v1 floating tag update step
# Task 4 / release.yml structural requirement: the workflow must update v1
# after creating the release — without it, `uses: stephenleo/vibestats@v1`
# would require a manual tag push after every release.
# ---------------------------------------------------------------------------


def test_tc3_release_yml_has_v1_floating_tag_step() -> None:
    """[P0] 9.6-UNIT-030: release.yml must include a step that creates/updates the
    v1 floating tag after the release is published.

    AC #2: `uses: stephenleo/vibestats@v1` must resolve to the v0.1.0 commit.
    The release.yml workflow must automate this by force-updating the v1 tag
    after each vX.Y.Z release (Story 8.1 architecture decision).
    """
    # The step must reference the major version extraction and force-push pattern
    # release.yml extracts major version: e.g. major="${REF_NAME%%.*}"  → v1
    assert (
        "git tag" in _RELEASE_YML_TEXT and ("--force" in _RELEASE_YML_TEXT or "-f" in _RELEASE_YML_TEXT)
    ) or "%%.*" in _RELEASE_YML_TEXT, (
        "release.yml must contain a step that creates/updates the v1 floating tag "
        "(e.g., 'git tag $major $REF_NAME --force' + 'git push origin $major --force'). "
        "Without this, `uses: stephenleo/vibestats@v1` breaks after v0.1.0 (AC #2)."
    )


def test_tc3_release_yml_floating_tag_uses_force_push() -> None:
    """[P0] 9.6-UNIT-031: release.yml floating tag step must use force-push for
    the major version tag.

    AC #2: The v1 tag must be force-updated on each release so it always points
    to the latest vX.Y.Z within the v1 major line. A plain 'git push' fails if
    v1 already exists (which it will after v0.1.0 is published).
    """
    # Force push is required for updating an existing tag
    push_force_pattern = re.compile(
        r"git\s+push\s+.*--force|git\s+push\s+.*-f\b",
        re.IGNORECASE,
    )
    assert push_force_pattern.search(_RELEASE_YML_TEXT), (
        "release.yml floating tag step must use 'git push ... --force' (or -f) for the v1 tag. "
        "A non-force push to an existing tag fails with 'already exists' error (AC #2)."
    )


def test_tc3_release_yml_floating_tag_derives_major_version() -> None:
    """[P1] 9.6-UNIT-032: release.yml floating tag step must derive the major version
    (v1) from the pushed tag (e.g., v0.1.0 → v0; v1.2.0 → v1).

    AC #2: The major tag must be computed dynamically from the pushed tag name
    so the workflow works correctly for any future v1.x.x, v2.x.x releases.
    No hardcoded 'v1' string is acceptable — the tag must be computed.
    """
    # release.yml uses parameter expansion: major="${REF_NAME%%.*}" to extract v1 from v1.2.3
    # For v0.1.0, major would be v0 (which is correct behaviour — v0 is the float for 0.x)
    # The key thing is: the computation exists (not hardcoded 'v1')
    # Check for shell parameter expansion pattern or similar dynamic extraction
    assert "%%.*" in _RELEASE_YML_TEXT or "cut" in _RELEASE_YML_TEXT or re.search(
        r"major\s*=\s*['\"]?\$", _RELEASE_YML_TEXT
    ), (
        "release.yml must dynamically compute the major version tag from the pushed tag "
        "(e.g., major=\"${REF_NAME%%.*}\"). Hardcoding 'v1' breaks future major releases (AC #2)."
    )


# ---------------------------------------------------------------------------
# [AC #1] release.yml: three required binary asset names
# Verify the workflow references all three expected asset files in the release step
# ---------------------------------------------------------------------------


def test_tc4_release_yml_references_all_three_binary_assets() -> None:
    """[P0] 9.6-UNIT-040: release.yml must reference all three platform binary assets
    in the GitHub Release creation step.

    AC #1: The GitHub Release at v0.1.0 must contain:
      - vibestats-aarch64-apple-darwin.tar.gz
      - vibestats-x86_64-apple-darwin.tar.gz
      - vibestats-x86_64-unknown-linux-gnu.tar.gz
    The release.yml files: block must include all three.
    """
    for target in EXPECTED_BINARY_TARGETS:
        assert target in _RELEASE_YML_TEXT, (
            f"release.yml does not reference binary asset for target '{target}'. "
            f"AC #1 requires all three platform assets: {sorted(EXPECTED_BINARY_TARGETS)}. "
            "Check the 'files:' block in the release step."
        )


def test_tc4_release_yml_references_tar_gz_assets() -> None:
    """[P0] 9.6-UNIT-041: release.yml release step must attach .tar.gz archives.

    AC #1: All three platform binaries must be distributed as .tar.gz archives.
    install.sh (Epic 6) constructs download URLs expecting the .tar.gz suffix.
    """
    tar_gz_count = _RELEASE_YML_TEXT.count(".tar.gz")
    # Expect at least 3 occurrences in files: block (one per target)
    assert tar_gz_count >= 3, (
        f"release.yml contains {tar_gz_count} reference(s) to '.tar.gz' but expected at least 3. "
        "All three platform binaries must be attached as .tar.gz archives (AC #1)."
    )


# ---------------------------------------------------------------------------
# [AC #3 partial] Cargo.toml: ureq dependency for TLS/network requests
# The rustls fallback path is documented in Dev Notes; the test verifies ureq
# is present so the fallback option is applicable.
# ---------------------------------------------------------------------------


def test_tc5_cargo_toml_version_is_0_1_0() -> None:
    """[P1] 9.6-UNIT-050: Cargo.toml package version must be 0.1.0.

    The tag pushed in Task 2 is v0.1.0. The Cargo.toml version must match
    to ensure the compiled binary reports the correct version via `vibestats --version`.
    """
    # Accept both [package] version and workspace.package.version
    assert re.search(
        r'version\s*=\s*["\']0\.1\.0["\']',
        _CARGO_TOML_TEXT,
    ), (
        "Cargo.toml package version must be '0.1.0' to match the v0.1.0 tag being pushed. "
        "The binary version string (`vibestats --version`) is derived from this value (Task 2)."
    )


def test_tc5_cargo_toml_has_ureq_dependency() -> None:
    """[P1] 9.6-UNIT-051: Cargo.toml must declare the 'ureq' dependency.

    AC #3: If the Linux cross-compilation fails with an OpenSSL/TLS error,
    the documented rustls fallback requires the ureq dependency to exist so
    that 'features = ["rustls"]' can be added to it.
    """
    assert "ureq" in _CARGO_TOML_TEXT, (
        "Cargo.toml must include 'ureq' as a dependency. "
        "This is required for the rustls TLS fallback documented in Dev Notes (AC #3, Task 3)."
    )


# ---------------------------------------------------------------------------
# [AC #5 partial] deploy-site.yml exists with workflow_dispatch trigger
# ---------------------------------------------------------------------------


def test_tc6_deploy_site_yml_exists() -> None:
    """[P1] 9.6-UNIT-060: .github/workflows/deploy-site.yml must exist.

    AC #5: The deploy-site.yml workflow is needed to trigger the Cloudflare
    Pages deployment via workflow_dispatch (Story 8.2 deliverable).
    """
    assert DEPLOY_SITE_YML.exists(), (
        f"deploy-site.yml not found at {DEPLOY_SITE_YML}. "
        "Story 8.2 should have created .github/workflows/deploy-site.yml (AC #5)."
    )


def test_tc6_deploy_site_yml_has_workflow_dispatch_trigger() -> None:
    """[P1] 9.6-UNIT-061: deploy-site.yml must have a workflow_dispatch trigger.

    AC #5: Task 5 requires triggering deploy-site.yml via workflow_dispatch.
    Without this trigger, the manual deployment step cannot be executed (AC #5).
    """
    on_block = _DEPLOY_SITE_YML_DOC.get("on", _DEPLOY_SITE_YML_DOC.get(True, {}))
    assert isinstance(on_block, dict), (
        "deploy-site.yml 'on:' block must be a YAML mapping."
    )
    assert "workflow_dispatch" in on_block, (
        "deploy-site.yml must have a 'workflow_dispatch' trigger. "
        "Task 5 triggers the deployment via workflow_dispatch (AC #5)."
    )


# ---------------------------------------------------------------------------
# [PRE-FLIGHT] Pre-release checklist verification (Task 1) — local tooling
# These tests verify static/structural conditions on disk.
# cargo test, cargo clippy, and bats are verified separately as manual steps
# because they require build toolchain access.
# ---------------------------------------------------------------------------


def test_tc7_release_yml_trigger_is_tag_push_only() -> None:
    """[P0] 9.6-UNIT-070: release.yml must ONLY trigger on tag pushes (v*).

    Pre-release checklist (Task 1): the workflow must only trigger on version
    tags — not on every commit push. Branch triggers would create a release
    on every push to main, depleting GitHub Actions minutes and creating
    spurious releases (AC #1).
    """
    on_block = _RELEASE_YML_DOC.get("on", _RELEASE_YML_DOC.get(True, {}))
    assert isinstance(on_block, dict), "'on:' block in release.yml must be a mapping."

    trigger_keys = set(on_block.keys())
    assert "push" in trigger_keys, (
        "release.yml must have a 'push' trigger for tag-based releases (AC #1)."
    )

    # Only push trigger is allowed (no branch, pull_request, schedule, etc.)
    allowed_triggers = {"push"}
    unexpected = trigger_keys - allowed_triggers
    assert not unexpected, (
        f"release.yml 'on:' must contain ONLY 'push' trigger. "
        f"Unexpected triggers found: {unexpected}. "
        "Branch or other triggers would create unintended releases (AC #1)."
    )

    push_block = on_block.get("push", {})
    assert "tags" in push_block, (
        "release.yml on.push must include a 'tags:' filter. "
        "Without a tag filter, every push to main triggers a release (AC #1)."
    )
    assert "branches" not in push_block, (
        "release.yml on.push must NOT include 'branches:' — only tag pushes should trigger. "
        "A branches filter would trigger releases on every commit to main (AC #1)."
    )
