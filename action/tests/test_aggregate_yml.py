"""test_aggregate_yml.py — Acceptance tests for aggregate.yml (TDD RED PHASE).

Story 5.5: Implement aggregate.yml (user vibestats-data workflow template)
GH Issue: #30

All tests are marked with pytest.mark.skip() — this is the TDD red phase.
aggregate.yml does NOT exist yet. Tests will fail until the file is created.

Test IDs follow the story task list:
  TC-1 (P0): Only 'schedule' and 'workflow_dispatch' triggers present — no push/PR
  TC-2 (P1): 'workflow_dispatch' trigger is present
  TC-3 (P1): Step uses 'stephenleo/vibestats@v1'
  TC-4 (P1): 'token' input references 'secrets.VIBESTATS_TOKEN'
"""

import pathlib

import pytest
import yaml

# ---------------------------------------------------------------------------
# Path resolution
# ---------------------------------------------------------------------------

# action/tests/ → action/ → repo root
REPO_ROOT = pathlib.Path(__file__).parent.parent.parent
AGGREGATE_YML = REPO_ROOT / ".github" / "workflows" / "aggregate.yml"


def _load_workflow() -> dict:
    """Parse aggregate.yml and return the top-level dict."""
    with AGGREGATE_YML.open("r", encoding="utf-8") as fh:
        return yaml.safe_load(fh)


# ---------------------------------------------------------------------------
# TC-1 (P0): Only 'schedule' and 'workflow_dispatch' triggers — no push/PR
# R-005: per-push trigger accidentally added exhausts free-tier minutes (NFR5)
# ---------------------------------------------------------------------------


@pytest.mark.skip(reason="TDD RED PHASE — aggregate.yml not yet created (Story 5.5)")
def test_tc1_only_schedule_and_workflow_dispatch_triggers() -> None:
    """[P0] AC2/AC3/R-005/NFR5: Parse aggregate.yml and assert the 'on' key contains
    ONLY 'schedule' and 'workflow_dispatch'. No 'push', 'pull_request', 'release',
    or wildcard triggers are allowed — they would exhaust GitHub Actions free-tier
    minutes (NFR5: ≤60 min/month)."""
    workflow = _load_workflow()

    on_block = workflow.get("on", workflow.get(True, {}))  # PyYAML parses bare 'on' as True
    assert on_block is not None, "'on:' block is missing from aggregate.yml"

    trigger_keys = set(on_block.keys()) if isinstance(on_block, dict) else set()

    # Must have both required triggers
    assert "schedule" in trigger_keys, (
        "Missing 'schedule' trigger in aggregate.yml 'on:' block. "
        "Daily cron is required (FR25)."
    )
    assert "workflow_dispatch" in trigger_keys, (
        "Missing 'workflow_dispatch' trigger in aggregate.yml 'on:' block. "
        "Manual trigger is required (FR26)."
    )

    # Must NOT have any other triggers
    forbidden_triggers = trigger_keys - {"schedule", "workflow_dispatch"}
    assert not forbidden_triggers, (
        f"Forbidden triggers found in aggregate.yml: {forbidden_triggers}. "
        "Only 'schedule' and 'workflow_dispatch' are allowed (R-005, NFR5)."
    )


# ---------------------------------------------------------------------------
# TC-2 (P1): 'workflow_dispatch' trigger is present (AC2, FR26)
# ---------------------------------------------------------------------------


@pytest.mark.skip(reason="TDD RED PHASE — aggregate.yml not yet created (Story 5.5)")
def test_tc2_workflow_dispatch_trigger_present() -> None:
    """[P1] AC2/FR26: The 'workflow_dispatch' trigger must be present so users
    can run the workflow manually from the GitHub Actions UI."""
    workflow = _load_workflow()

    on_block = workflow.get("on", workflow.get(True, {}))
    assert on_block is not None, "'on:' block is missing from aggregate.yml"

    trigger_keys = set(on_block.keys()) if isinstance(on_block, dict) else set()
    assert "workflow_dispatch" in trigger_keys, (
        "Missing 'workflow_dispatch' trigger in aggregate.yml 'on:' block (FR26)."
    )


# ---------------------------------------------------------------------------
# TC-3 (P1): Step uses 'stephenleo/vibestats@v1' (AC1, architecture.md)
# ---------------------------------------------------------------------------


@pytest.mark.skip(reason="TDD RED PHASE — aggregate.yml not yet created (Story 5.5)")
def test_tc3_step_uses_vibestats_v1_action() -> None:
    """[P1] AC1: The job step must use 'stephenleo/vibestats@v1'. This resolves
    to action.yml in the same repo via the Marketplace tag 'v1' (Story 8.3)."""
    workflow = _load_workflow()

    jobs = workflow.get("jobs", {})
    assert jobs, "No 'jobs:' defined in aggregate.yml"

    # Find the 'uses:' field in any step across all jobs
    found_uses = None
    for _job_name, job_def in jobs.items():
        steps = job_def.get("steps", [])
        for step in steps:
            if "uses" in step:
                found_uses = step["uses"]
                break
        if found_uses:
            break

    assert found_uses is not None, (
        "No 'uses:' step found in any job in aggregate.yml. "
        "Expected a step that calls 'stephenleo/vibestats@v1'."
    )
    assert found_uses == "stephenleo/vibestats@v1", (
        f"Step 'uses:' is '{found_uses}', expected 'stephenleo/vibestats@v1' (AC1)."
    )


# ---------------------------------------------------------------------------
# TC-4 (P1): 'token' input references 'secrets.VIBESTATS_TOKEN' (AC1, FR10)
# ---------------------------------------------------------------------------


@pytest.mark.skip(reason="TDD RED PHASE — aggregate.yml not yet created (Story 5.5)")
def test_tc4_token_input_references_vibestats_token_secret() -> None:
    """[P1] AC1/FR10: The step's 'with.token' must reference '${{ secrets.VIBESTATS_TOKEN }}'.
    This is the secret set by the installer (Story 6.1). Using the wrong secret name
    would cause silent authentication failures for every user."""
    workflow = _load_workflow()

    jobs = workflow.get("jobs", {})
    assert jobs, "No 'jobs:' defined in aggregate.yml"

    token_value = None
    for _job_name, job_def in jobs.items():
        steps = job_def.get("steps", [])
        for step in steps:
            if "uses" in step and "with" in step:
                token_value = step["with"].get("token")
                break
        if token_value is not None:
            break

    assert token_value is not None, (
        "No 'with.token' input found in any step in aggregate.yml. "
        "The step using 'stephenleo/vibestats@v1' must pass 'token' (AC1)."
    )
    # Allow minor whitespace variation inside the expression
    assert "secrets.VIBESTATS_TOKEN" in str(token_value), (
        f"'with.token' is '{token_value}', expected it to reference "
        "'secrets.VIBESTATS_TOKEN' (AC1, FR10)."
    )
