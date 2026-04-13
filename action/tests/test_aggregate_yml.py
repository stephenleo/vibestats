"""test_aggregate_yml.py — Acceptance tests for aggregate.yml.

Story 5.5: Implement aggregate.yml (user vibestats-data workflow template)
GH Issue: #30

Story 9.7: Add concurrency group to prevent concurrent push conflicts
GH Issue: #87

Test IDs follow the story task list:
  TC-1 (P0): Only 'schedule' and 'workflow_dispatch' triggers present — no push/PR
  TC-2 (P1): 'workflow_dispatch' trigger is present
  TC-3 (P1): Step uses 'stephenleo/vibestats@v1'
  TC-4 (P1): 'token' input references 'secrets.VIBESTATS_TOKEN'
  TC-5 (P1): 'concurrency:' block present with correct group and cancel-in-progress=False
"""

import pathlib
from typing import Optional

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


def _find_uses_step(workflow: dict) -> Optional[dict]:
    """Return the first step that has a 'uses:' key, searching all jobs.

    TC-3 and TC-4 both need to locate the composite-action step. Centralising
    the search here keeps the test bodies focused on their own assertions and
    makes the traversal pattern easy to update if the YAML structure changes.
    """
    for _job_name, job_def in workflow.get("jobs", {}).items():
        for step in job_def.get("steps", []):
            if "uses" in step:
                return step
    return None


# ---------------------------------------------------------------------------
# TC-1 (P0): Only 'schedule' and 'workflow_dispatch' triggers — no push/PR
# R-005: per-push trigger accidentally added exhausts free-tier minutes (NFR5)
# ---------------------------------------------------------------------------


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


def test_tc3_step_uses_vibestats_v1_action() -> None:
    """[P1] AC1: The job step must use 'stephenleo/vibestats@v1'. This resolves
    to action.yml in the same repo via the Marketplace tag 'v1' (Story 8.3)."""
    workflow = _load_workflow()

    step = _find_uses_step(workflow)
    assert step is not None, (
        "No 'uses:' step found in any job in aggregate.yml. "
        "Expected a step that calls 'stephenleo/vibestats@v1'."
    )
    assert step["uses"] == "stephenleo/vibestats@v1", (
        f"Step 'uses:' is '{step['uses']}', expected 'stephenleo/vibestats@v1' (AC1)."
    )


# ---------------------------------------------------------------------------
# TC-4 (P1): 'token' input references 'secrets.VIBESTATS_TOKEN' (AC1, FR10)
# ---------------------------------------------------------------------------


def test_tc4_token_input_references_vibestats_token_secret() -> None:
    """[P1] AC1/FR10: The step's 'with.token' must reference '${{ secrets.VIBESTATS_TOKEN }}'.
    This is the secret set by the installer (Story 6.1). Using the wrong secret name
    would cause silent authentication failures for every user."""
    workflow = _load_workflow()

    step = _find_uses_step(workflow)
    assert step is not None, (
        "No 'uses:' step found in any job in aggregate.yml. "
        "The step using 'stephenleo/vibestats@v1' must be present to have a 'token' input (AC1)."
    )

    token_value = step.get("with", {}).get("token")
    assert token_value is not None, (
        "No 'with.token' input found in any step in aggregate.yml. "
        "The step using 'stephenleo/vibestats@v1' must pass 'token' (AC1)."
    )
    # Allow minor whitespace variation inside the expression
    assert "secrets.VIBESTATS_TOKEN" in str(token_value), (
        f"'with.token' is '{token_value}', expected it to reference "
        "'secrets.VIBESTATS_TOKEN' (AC1, FR10)."
    )


# ---------------------------------------------------------------------------
# TC-5 (P1): 'concurrency:' block present with correct group and policy (Story 9.7)
# AC1: workflow-level concurrency block serialises concurrent runs for same owner
# AC3: test asserts presence of concurrency key, group value, and cancel-in-progress
# ---------------------------------------------------------------------------


def test_tc5_concurrency_block_present_with_correct_group_and_policy() -> None:
    """[P1] Story-9.7/AC1/AC3: aggregate.yml must declare a workflow-level
    'concurrency:' block that serialises runs for the same repository owner.

    The group key must be 'vibestats-${{ github.repository_owner }}' so that
    concurrent runs triggered by different machines for the same profile repo
    are queued rather than raced. cancel-in-progress must be False so that
    an in-flight push is never killed mid-run (Dev Notes: cancel-in-progress
    rationale in story 9.7).

    """
    workflow = _load_workflow()

    # The 'concurrency:' key is unambiguous YAML — no PyYAML quirk (unlike 'on:')
    concurrency = workflow.get("concurrency")
    assert concurrency is not None, (
        "Missing 'concurrency:' block in aggregate.yml. "
        "Story 9.7 requires a workflow-level concurrency group to serialise "
        "concurrent runs and prevent push conflicts (AC1)."
    )

    group = concurrency.get("group")
    assert group == "vibestats-${{ github.repository_owner }}", (
        f"'concurrency.group' is '{group}', expected "
        "'vibestats-${{ github.repository_owner }}'. "
        "The group must be owner-scoped so that runs from different machines "
        "sharing the same profile repo are serialised (AC2, Story 9.7)."
    )

    cancel_in_progress = concurrency.get("cancel-in-progress")
    assert cancel_in_progress is False, (
        f"'concurrency.cancel-in-progress' is '{cancel_in_progress}', expected False. "
        "Setting cancel-in-progress: false queues the second run instead of killing "
        "an in-flight push, ensuring both machines' data is captured (AC2, Dev Notes)."
    )
