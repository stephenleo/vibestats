"""Schema/unit tests for .github/workflows/release.yml — TDD Red Phase.

Story 8.1: Implement Rust binary release CI
GH Issue: #39

Test IDs follow: 8.1-SCHEMA-{SEQ}

TDD Phase: RED — all tests marked with pytest.mark.skip.
          Remove pytest.mark.skip decorators after release.yml is implemented (green phase).

Run: python -m pytest action/tests/test_release_yml.py -v

Tests assert structural and schema properties of release.yml required for:
- AC1 (R-001, R-002): matrix build with fail-fast and correct targets
- AC2 (R-007): archive naming convention vibestats-<target>.tar.gz
- AC3 (R-001): non-zero exit / no partial release on failure
- P1: action pinning (R-006), tag-only trigger (R-001), cross for Linux (R-002)
- P2: ref_name usage (R-007), all three targets uploaded (R-001)
"""

import pathlib
import re

import pytest

# ---------------------------------------------------------------------------
# Path resolution
# ---------------------------------------------------------------------------

# action/tests/ -> action/ -> repo root
REPO_ROOT = pathlib.Path(__file__).parent.parent.parent
RELEASE_YML = REPO_ROOT / ".github" / "workflows" / "release.yml"

EXPECTED_TARGETS = {
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "x86_64-unknown-linux-gnu",
}


def _load_text() -> str:
    return RELEASE_YML.read_text(encoding="utf-8")


def _load_yaml() -> dict:
    """Parse release.yml as a Python dict using PyYAML."""
    import yaml  # type: ignore[import]

    with RELEASE_YML.open(encoding="utf-8") as fh:
        return yaml.safe_load(fh)


# ---------------------------------------------------------------------------
# Prerequisite: file exists
# ---------------------------------------------------------------------------


def test_prereq_release_yml_exists() -> None:
    """[P0] 8.1-SCHEMA-000: .github/workflows/release.yml must exist."""
    assert RELEASE_YML.exists(), (
        f"release.yml not found at {RELEASE_YML}. "
        "Create .github/workflows/release.yml as specified in Story 8.1 Task 1."
    )


def test_prereq_release_yml_not_empty() -> None:
    """[P0] 8.1-SCHEMA-001: release.yml must not be empty."""
    text = _load_text()
    assert text.strip(), "release.yml is empty — must contain a valid GitHub Actions workflow"


def test_prereq_release_yml_parses_as_valid_yaml() -> None:
    """[P0] 8.1-SCHEMA-002: release.yml must parse as valid YAML without errors."""
    doc = _load_yaml()
    assert isinstance(doc, dict), "release.yml top-level must be a YAML mapping"


# ---------------------------------------------------------------------------
# TC-1 (P1): Workflow trigger is push: tags: ['v*'] only — 8.1-SCHEMA-010
# Risk: R-001 — branch/PR trigger could create unintended releases
# Story 8.1 AC1 / test-design-epic-8.md P1 row 2
# ---------------------------------------------------------------------------


def test_tc1_trigger_is_tag_push_only() -> None:
    """[P1] 8.1-SCHEMA-010: release.yml trigger must be ONLY 'push.tags: [v*]'.
    No branch push, pull_request, schedule, or workflow_dispatch triggers allowed.
    Prevents accidental releases on every commit (R-001)."""
    doc = _load_yaml()
    # PyYAML parses bare 'on' as boolean True
    on_block = doc.get("on", doc.get(True))
    assert on_block is not None, "'on:' block missing from release.yml"
    assert isinstance(on_block, dict), "'on:' block must be a YAML mapping"

    trigger_keys = set(on_block.keys())
    assert trigger_keys == {"push"}, (
        f"release.yml 'on:' must contain ONLY 'push', found: {trigger_keys}. "
        "No branch, PR, schedule, or workflow_dispatch triggers allowed."
    )

    push_block = on_block.get("push", {})
    assert "tags" in push_block, (
        "release.yml on.push must include a 'tags:' filter, not a branches filter."
    )
    # Must NOT have a branches filter
    assert "branches" not in push_block, (
        "release.yml on.push must NOT include 'branches:' — only tag pushes should trigger."
    )


def test_tc1_tag_pattern_matches_v_wildcard() -> None:
    """[P1] 8.1-SCHEMA-011: release.yml on.push.tags must include 'v*' pattern.
    Ensures tags like v0.1.0 and v1.0.0 trigger the release pipeline."""
    doc = _load_yaml()
    on_block = doc.get("on", doc.get(True, {}))
    push_block = on_block.get("push", {}) if isinstance(on_block, dict) else {}
    tags = push_block.get("tags", [])
    assert tags, "release.yml on.push.tags list is empty — must contain 'v*'"
    assert any(t == "v*" or t.startswith("v") for t in tags), (
        f"release.yml on.push.tags must include 'v*' pattern, found: {tags}"
    )


# ---------------------------------------------------------------------------
# TC-2 (P0): Matrix targets are exactly the three required targets — 8.1-SCHEMA-020
# Risk: R-002 — wrong targets break install.sh download URLs (FR41)
# Story 8.1 AC1 / test-design-epic-8.md P0 row 2
# ---------------------------------------------------------------------------


def test_tc2_matrix_targets_exact_set() -> None:
    """[P0] 8.1-SCHEMA-020: release.yml build job matrix must include EXACTLY the three
    required targets: aarch64-apple-darwin, x86_64-apple-darwin, x86_64-unknown-linux-gnu.
    Missing or extra targets break Epic 6 install.sh URL construction (R-002, FR41)."""
    doc = _load_yaml()
    jobs = doc.get("jobs", {})
    assert jobs, "release.yml has no 'jobs:' defined"

    # Find the build job (may be named 'build' or similar)
    build_job = None
    for job_name, job_def in jobs.items():
        strategy = job_def.get("strategy", {})
        matrix = strategy.get("matrix", {})
        if matrix:
            build_job = job_def
            break

    assert build_job is not None, (
        "No job with a 'strategy.matrix:' found in release.yml. "
        "The build job must use a matrix strategy."
    )

    matrix = build_job["strategy"]["matrix"]
    # Matrix may use 'include' or a direct 'target' list
    found_targets = set()
    if "include" in matrix:
        for entry in matrix["include"]:
            if "target" in entry:
                found_targets.add(entry["target"])
    elif "target" in matrix:
        targets = matrix["target"]
        if isinstance(targets, list):
            found_targets.update(targets)
        else:
            found_targets.add(targets)

    assert found_targets == EXPECTED_TARGETS, (
        f"release.yml matrix targets mismatch.\n"
        f"  Expected: {sorted(EXPECTED_TARGETS)}\n"
        f"  Found:    {sorted(found_targets)}\n"
        "All three targets are required for cross-platform binary distribution (FR41)."
    )


# ---------------------------------------------------------------------------
# TC-3 (P0): fail-fast: true in matrix strategy — 8.1-SCHEMA-030
# Risk: R-001 — partial releases if one platform fails silently
# Story 8.1 AC3 / test-design-epic-8.md P0 row 1
# ---------------------------------------------------------------------------


def test_tc3_matrix_fail_fast_true() -> None:
    """[P0] 8.1-SCHEMA-030: release.yml build job strategy must have fail-fast: true.
    Prevents a partial GitHub Release when one platform compilation fails (AC3, R-001).
    A partial release (missing one binary) breaks install.sh for that platform."""
    doc = _load_yaml()
    jobs = doc.get("jobs", {})
    assert jobs, "release.yml has no 'jobs:' defined"

    build_job = None
    for _job_name, job_def in jobs.items():
        if "strategy" in job_def and "matrix" in job_def["strategy"]:
            build_job = job_def
            break

    assert build_job is not None, "No matrix build job found in release.yml"
    strategy = build_job["strategy"]
    fail_fast = strategy.get("fail-fast")
    assert fail_fast is True, (
        f"release.yml strategy.fail-fast must be true, got: {fail_fast!r}. "
        "fail-fast: true ensures no partial GitHub Release if any target fails (R-001, AC3)."
    )


# ---------------------------------------------------------------------------
# TC-4 (P0): Archive naming follows vibestats-<target>.tar.gz — 8.1-SCHEMA-040
# Risk: R-007 — wrong archive name breaks install.sh URL pattern (Epic 6)
# Story 8.1 AC2 / test-design-epic-8.md P0 row 3
# ---------------------------------------------------------------------------


def test_tc4_archive_name_uses_matrix_target_variable() -> None:
    """[P0] 8.1-SCHEMA-040: Archive step must produce vibestats-${{ matrix.target }}.tar.gz.
    The filename template must use the matrix.target variable — no hardcoded platform names.
    install.sh (Epic 6) constructs download URLs from this exact naming pattern (R-007, AC2)."""
    text = _load_text()
    # The tar command or filename reference must include matrix.target variable
    assert "matrix.target" in text, (
        "release.yml must reference '${{ matrix.target }}' for archive naming. "
        "No hardcoded platform strings allowed in archive step (R-007)."
    )
    # The archive name pattern must be vibestats-<target>.tar.gz
    assert "vibestats-" in text, (
        "release.yml must produce archives named 'vibestats-<target>.tar.gz'. "
        "This exact prefix is required by install.sh in Epic 6 (R-007, AC2)."
    )
    assert ".tar.gz" in text, (
        "release.yml must produce .tar.gz archives (AC2). "
        "install.sh expects .tar.gz format for all platform binaries."
    )


def test_tc4_archive_contains_vibestats_binary() -> None:
    """[P0] 8.1-SCHEMA-041: Archive command must package the 'vibestats' binary from
    target/<target>/release/. The binary name must be 'vibestats' (not 'vibestats.exe'
    or a path variant) so install.sh can extract and execute it directly (AC2)."""
    text = _load_text()
    # The tar command must reference the release directory with vibestats binary
    assert "target/" in text and "release" in text, (
        "release.yml archive step must reference 'target/<target>/release/' path. "
        "The built binary lives at target/${{ matrix.target }}/release/vibestats (AC2)."
    )


# ---------------------------------------------------------------------------
# TC-5 (P1): Action references pinned to version tags, not @main/@master — 8.1-SCHEMA-050
# Risk: R-006 — mutable tags break pipeline on upstream action changes
# test-design-epic-8.md P1 row 1
# ---------------------------------------------------------------------------


def test_tc5_no_action_uses_main_or_master_tag() -> None:
    """[P1] 8.1-SCHEMA-050: All 'uses:' action references must be pinned to version tags
    (e.g., @v4) — never @main or @master. Mutable tags cause pipeline breakage when
    the upstream action changes behaviour unexpectedly (R-006)."""
    text = _load_text()
    # Find all 'uses: action@ref' patterns
    uses_pattern = re.compile(r"uses\s*:\s*(\S+)")
    matches = uses_pattern.findall(text)

    bad_refs = [m for m in matches if m.endswith("@main") or m.endswith("@master")]
    assert not bad_refs, (
        f"release.yml contains action references pinned to mutable tags: {bad_refs}. "
        "Pin all 'uses:' references to major version tags (e.g., @v4) — never @main or @master (R-006)."
    )


def test_tc5_required_actions_are_pinned() -> None:
    """[P1] 8.1-SCHEMA-051: Required actions (checkout, upload-artifact, download-artifact)
    must be pinned to a specific version (e.g., @v4) — not floating refs."""
    text = _load_text()
    required_actions = [
        "actions/checkout",
        "actions/upload-artifact",
        "actions/download-artifact",
    ]
    for action in required_actions:
        assert action in text, (
            f"release.yml must use '{action}' action (required for release pipeline). "
            "Pin it to a major version tag (e.g., @v4)."
        )
        # Verify it has a version tag, not just the base name
        versioned_pattern = re.compile(rf"{re.escape(action)}@v\d+")
        assert versioned_pattern.search(text), (
            f"'{action}' in release.yml must be pinned with a major version tag "
            f"(e.g., {action}@v4), not just '{action}' without a version (R-006)."
        )


# ---------------------------------------------------------------------------
# TC-6 (P1): 'cross' crate or equivalent used for Linux target — 8.1-SCHEMA-060
# Risk: R-002 — Linux cross-compilation may fail without cross tooling
# test-design-epic-8.md P1 row 3
# ---------------------------------------------------------------------------


def test_tc6_cross_used_for_linux_target() -> None:
    """[P1] 8.1-SCHEMA-060: release.yml must use 'cross' for the Linux target
    (x86_64-unknown-linux-gnu). Native cargo build is insufficient for reliable
    Linux cross-compilation from macOS/Linux runners (R-002, architecture.md)."""
    text = _load_text()
    # The workflow must reference 'cross' — either 'cargo install cross' or 'cross build'
    assert "cross" in text.lower(), (
        "release.yml must use 'cross' for cross-compilation of the Linux target. "
        "Add 'cross build --release --target ${{ matrix.target }}' for x86_64-unknown-linux-gnu (R-002)."
    )
    # More specific: 'cross build' command should appear
    assert "cross build" in text or "cross-rs/cross" in text, (
        "release.yml must invoke 'cross build' or use 'cross-rs/cross' action for the Linux target. "
        "Plain 'cargo build' is not sufficient for reliable Linux cross-compilation (R-002)."
    )


# ---------------------------------------------------------------------------
# TC-7 (P2): Release step uses ${{ github.ref_name }} — no hardcoded version — 8.1-SCHEMA-070
# Risk: R-007 — hardcoded version breaks every release after the first
# test-design-epic-8.md P2 row 1
# ---------------------------------------------------------------------------


def test_tc7_release_step_uses_github_ref_name() -> None:
    """[P2] 8.1-SCHEMA-070: The GitHub Release creation step must use '${{ github.ref_name }}'
    for the tag/release name — no hardcoded version strings (e.g., 'v0.1.0').
    Hardcoded versions break every release after the first tag (R-007)."""
    text = _load_text()
    assert "github.ref_name" in text, (
        "release.yml must use '${{ github.ref_name }}' for the release tag/name. "
        "Hardcoded version strings (e.g., 'v0.1.0') are not allowed — they break "
        "every release after the first (R-007)."
    )


# ---------------------------------------------------------------------------
# TC-8 (P2): All three targets uploaded to GitHub Release — 8.1-SCHEMA-080
# Risk: R-001 — partial release if any target artifact is not uploaded
# test-design-epic-8.md P2 row 5
# ---------------------------------------------------------------------------


def test_tc8_upload_artifact_step_present() -> None:
    """[P2] 8.1-SCHEMA-080: release.yml must include an upload step for build artifacts.
    The matrix build must upload each vibestats-<target>.tar.gz as a build artifact so
    the release job can attach all three to the GitHub Release (R-001, AC2)."""
    text = _load_text()
    assert "upload-artifact" in text or "upload_artifact" in text, (
        "release.yml must include an 'actions/upload-artifact' step to store "
        "each platform binary as a build artifact for the release job (R-001)."
    )


def test_tc8_download_artifact_step_present() -> None:
    """[P2] 8.1-SCHEMA-081: release.yml release job must include a download-artifact step.
    The release job needs 'actions/download-artifact' to collect all three platform archives
    before attaching them to the GitHub Release (R-001, AC2)."""
    text = _load_text()
    assert "download-artifact" in text or "download_artifact" in text, (
        "release.yml release job must include 'actions/download-artifact' to collect "
        "all three platform archives before attaching to the GitHub Release (R-001)."
    )


def test_tc8_release_job_needs_build_job() -> None:
    """[P2] 8.1-SCHEMA-082: The release job must declare 'needs: build' (or equivalent).
    This dependency ensures all three platform builds complete successfully before the
    release is created — preventing partial releases (AC3, R-001)."""
    doc = _load_yaml()
    jobs = doc.get("jobs", {})
    assert jobs, "release.yml has no 'jobs:' defined"

    # Find the release job (a job that has 'needs' pointing to the build job)
    release_job = None
    for _job_name, job_def in jobs.items():
        needs = job_def.get("needs")
        if needs:
            release_job = job_def
            break

    assert release_job is not None, (
        "release.yml must have a release job with 'needs: build' dependency. "
        "The release job must wait for all matrix builds to complete (AC3, R-001)."
    )


# ---------------------------------------------------------------------------
# TC-9 (P1): permissions: contents: write on release job — 8.1-SCHEMA-090
# Required for softprops/action-gh-release to create a GitHub Release
# ---------------------------------------------------------------------------


def test_tc9_release_job_has_contents_write_permission() -> None:
    """[P1] 8.1-SCHEMA-090: The release job must declare 'permissions: contents: write'.
    GitHub Actions defaults may be read-only; without this permission,
    softprops/action-gh-release cannot create a release or upload assets."""
    doc = _load_yaml()
    jobs = doc.get("jobs", {})
    assert jobs, "release.yml has no 'jobs:' defined"

    # Find the release job (has 'needs' or is not the matrix build job)
    found_permission = False
    for _job_name, job_def in jobs.items():
        perms = job_def.get("permissions", {})
        if isinstance(perms, dict) and perms.get("contents") == "write":
            found_permission = True
            break

    assert found_permission, (
        "release.yml must have a job with 'permissions: contents: write'. "
        "This is required for softprops/action-gh-release to create a GitHub Release "
        "and upload binary assets."
    )
