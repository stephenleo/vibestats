"""test_deploy_site_yml.py — Acceptance tests for deploy-site.yml.

Story 8.2: Implement Cloudflare Pages deploy workflow
GH Issue: #40

Test IDs follow: 8.2-UNIT-{SEQ}

TDD Phase: RED — all tests marked pytest.mark.skip until deploy-site.yml is
implemented in .github/workflows/.

Run: python -m pytest action/tests/test_deploy_site_yml.py -v

Acceptance Criteria (from epics.md):
  AC1: deploy-site.yml triggered only via workflow_dispatch with a ref input —
       no automatic triggers (push, pull_request, schedule, release).
  AC2: workflow checks out the specified ref, runs npm run build inside site/,
       and deploys to Cloudflare Pages using CLOUDFLARE_API_TOKEN and
       CLOUDFLARE_ACCOUNT_ID secrets.
  AC3: if the build step fails, no deployment to Cloudflare occurs
       (build gates deploy — no continue-on-error on build step).

Coverage targets from test-design-epic-8.md (P0 + P1 + P2):
  P0 (R-003): only workflow_dispatch trigger present
  P0 (R-004): exact secret names CLOUDFLARE_API_TOKEN / CLOUDFLARE_ACCOUNT_ID
  P0 (R-008): npm run build precedes any deploy step; no continue-on-error on build
  P1 (R-003): workflow_dispatch.inputs.ref declared
  P1 (R-003): checkout step uses github.event.inputs.ref variable
  P2 (R-008): checkout / build step uses working-directory: site or cd site/
"""

import pathlib
import re

import pytest

# ---------------------------------------------------------------------------
# Path resolution
# ---------------------------------------------------------------------------

# action/tests/ → action/ → repo root
REPO_ROOT = pathlib.Path(__file__).parent.parent.parent
DEPLOY_SITE_YML = REPO_ROOT / ".github" / "workflows" / "deploy-site.yml"


def _load_text() -> str:
    """Read deploy-site.yml as raw text."""
    return DEPLOY_SITE_YML.read_text(encoding="utf-8")


def _load_yaml() -> dict:
    """Parse deploy-site.yml as a Python dict using PyYAML."""
    import yaml  # test dependency only — not used in production action scripts

    with DEPLOY_SITE_YML.open("r", encoding="utf-8") as fh:
        return yaml.safe_load(fh)


# ---------------------------------------------------------------------------
# Preflight: deploy-site.yml must exist (fails during red phase)
# ---------------------------------------------------------------------------


@pytest.mark.skip(reason="RED PHASE — deploy-site.yml not yet implemented (Story 8.2 backlog)")
def test_preflight_deploy_site_yml_exists() -> None:
    """[P0] 8.2-UNIT-000: deploy-site.yml must exist at .github/workflows/.

    This test is the gatekeeper — all other tests depend on the file existing.
    """
    assert DEPLOY_SITE_YML.exists(), (
        f"deploy-site.yml not found at {DEPLOY_SITE_YML}. "
        "Create .github/workflows/deploy-site.yml to enter the GREEN phase."
    )


# ---------------------------------------------------------------------------
# TC-1 (P0): only workflow_dispatch trigger — no push/PR/schedule/release
# R-003: accidental trigger deploys uncommitted state to production (Score: 6)
# ---------------------------------------------------------------------------


@pytest.mark.skip(reason="RED PHASE — deploy-site.yml not yet implemented (Story 8.2 backlog)")
def test_tc1_only_workflow_dispatch_trigger() -> None:
    """[P0] 8.2-UNIT-001: deploy-site.yml 'on' key must contain ONLY workflow_dispatch.

    No push, pull_request, schedule, or release triggers are allowed.
    Any automatic trigger risks deploying uncommitted or intermediate site state
    to production Cloudflare Pages (R-003, AC1).
    """
    workflow = _load_yaml()

    # PyYAML parses bare 'on' as Python bool True
    on_block = workflow.get("on", workflow.get(True, {}))
    assert on_block is not None, "'on:' block is missing from deploy-site.yml"

    trigger_keys = set(on_block.keys()) if isinstance(on_block, dict) else set()

    assert "workflow_dispatch" in trigger_keys, (
        "Missing 'workflow_dispatch' trigger in deploy-site.yml 'on:' block (AC1)."
    )

    forbidden_triggers = trigger_keys - {"workflow_dispatch"}
    assert not forbidden_triggers, (
        f"Forbidden triggers found in deploy-site.yml: {forbidden_triggers}. "
        "Only 'workflow_dispatch' is allowed — automatic triggers risk "
        "deploying uncommitted state to production (R-003, AC1)."
    )


# ---------------------------------------------------------------------------
# TC-2 (P0): exact Cloudflare secret names — CLOUDFLARE_API_TOKEN and
#            CLOUDFLARE_ACCOUNT_ID with no hardcoded values
# R-004: incorrect secret name causes failed deploy or accidental exposure (Score: 6)
# ---------------------------------------------------------------------------


@pytest.mark.skip(reason="RED PHASE — deploy-site.yml not yet implemented (Story 8.2 backlog)")
def test_tc2_cloudflare_api_token_secret_name() -> None:
    """[P0] 8.2-UNIT-002a: CLOUDFLARE_API_TOKEN must be referenced by exact name.

    Misspelled or wrong-cased secret names cause silent deploy failures (R-004, AC2).
    """
    text = _load_text()
    assert "secrets.CLOUDFLARE_API_TOKEN" in text, (
        "deploy-site.yml does not reference 'secrets.CLOUDFLARE_API_TOKEN'. "
        "Check exact casing — GitHub secrets are case-sensitive (R-004, AC2)."
    )


@pytest.mark.skip(reason="RED PHASE — deploy-site.yml not yet implemented (Story 8.2 backlog)")
def test_tc2_cloudflare_account_id_secret_name() -> None:
    """[P0] 8.2-UNIT-002b: CLOUDFLARE_ACCOUNT_ID must be referenced by exact name.

    Misspelled or wrong-cased secret names cause silent deploy failures (R-004, AC2).
    """
    text = _load_text()
    assert "secrets.CLOUDFLARE_ACCOUNT_ID" in text, (
        "deploy-site.yml does not reference 'secrets.CLOUDFLARE_ACCOUNT_ID'. "
        "Check exact casing — GitHub secrets are case-sensitive (R-004, AC2)."
    )


@pytest.mark.skip(reason="RED PHASE — deploy-site.yml not yet implemented (Story 8.2 backlog)")
def test_tc2_no_hardcoded_token_values() -> None:
    """[P0] 8.2-UNIT-002c: No hardcoded token/credential patterns in deploy-site.yml.

    Any plaintext credential (32+ char alphanumeric string outside a secrets.*
    expression) constitutes a secret exposure risk (R-004, SEC).
    """
    text = _load_text()
    # Remove all valid ${{ secrets.* }} expressions before scanning
    stripped = re.sub(r"\$\{\{\s*secrets\.[A-Za-z0-9_]+\s*\}\}", "", text)
    # Remove ${{ github.* }} and ${{ env.* }} expressions
    stripped = re.sub(r"\$\{\{[^}]+\}\}", "", stripped)
    # Look for suspicious 32+ char alphanumeric strings (potential tokens/keys)
    matches = re.findall(r"[A-Za-z0-9_\-]{32,}", stripped)
    assert not matches, (
        f"Potential hardcoded credential(s) found in deploy-site.yml after removing "
        f"${{{{ secrets.* }}}} expressions: {matches[:3]}. "
        "All credentials must be referenced via ${{{{ secrets.NAME }}}} (R-004)."
    )


# ---------------------------------------------------------------------------
# TC-3 (P0): npm run build precedes any deploy step; no continue-on-error on build
# R-008: deploying broken build output to Cloudflare Pages (Score: 3, AC3)
# ---------------------------------------------------------------------------


@pytest.mark.skip(reason="RED PHASE — deploy-site.yml not yet implemented (Story 8.2 backlog)")
def test_tc3_npm_run_build_present() -> None:
    """[P0] 8.2-UNIT-003a: deploy-site.yml must include an 'npm run build' step (AC2, AC3)."""
    text = _load_text()
    assert "npm run build" in text, (
        "deploy-site.yml does not contain 'npm run build'. "
        "The build step is required to gate deployment on a successful build (AC3)."
    )


@pytest.mark.skip(reason="RED PHASE — deploy-site.yml not yet implemented (Story 8.2 backlog)")
def test_tc3_npm_run_build_precedes_deploy() -> None:
    """[P0] 8.2-UNIT-003b: npm run build must appear before the deploy step (AC3, R-008).

    If deploy runs before build, a failed build would not block the deployment.
    """
    text = _load_text()
    build_pos = text.find("npm run build")
    # Deploy step is identified by Cloudflare Pages action or wrangler invocation
    deploy_markers = [
        "cloudflare/pages-action",
        "cloudflare/wrangler-action",
        "wrangler pages deploy",
        "pages deploy",
    ]
    deploy_pos = -1
    for marker in deploy_markers:
        pos = text.find(marker)
        if pos != -1:
            deploy_pos = pos
            break

    assert build_pos != -1, "npm run build not found in deploy-site.yml"
    assert deploy_pos != -1, (
        "No Cloudflare Pages deploy action/command found in deploy-site.yml. "
        "Expected one of: " + ", ".join(deploy_markers)
    )
    assert build_pos < deploy_pos, (
        f"'npm run build' (pos {build_pos}) must appear before the deploy step "
        f"(pos {deploy_pos}) to gate deployment on a successful build (AC3, R-008)."
    )


@pytest.mark.skip(reason="RED PHASE — deploy-site.yml not yet implemented (Story 8.2 backlog)")
def test_tc3_no_continue_on_error_on_build() -> None:
    """[P0] 8.2-UNIT-003c: build step must not have continue-on-error: true (AC3, R-008).

    continue-on-error on the build step would allow a broken build to proceed
    to deployment, corrupting vibestats.dev.
    """
    workflow = _load_yaml()

    jobs = workflow.get("jobs", {})
    assert jobs, "No 'jobs:' defined in deploy-site.yml"

    for job_name, job_def in jobs.items():
        steps = job_def.get("steps", [])
        for i, step in enumerate(steps):
            run_cmd = step.get("run", "")
            if "npm run build" in str(run_cmd):
                coe = step.get("continue-on-error", False)
                assert coe is not True, (
                    f"Step {i} ('{step.get('name', 'unnamed')}') in job '{job_name}' "
                    f"runs 'npm run build' but has continue-on-error: true. "
                    "This allows broken builds to reach deployment (AC3, R-008)."
                )


# ---------------------------------------------------------------------------
# TC-4 (P1): workflow_dispatch.inputs.ref declared
# R-003: allows deploying specific branch or tag to production (AC1)
# ---------------------------------------------------------------------------


@pytest.mark.skip(reason="RED PHASE — deploy-site.yml not yet implemented (Story 8.2 backlog)")
def test_tc4_workflow_dispatch_has_ref_input() -> None:
    """[P1] 8.2-UNIT-004: workflow_dispatch must declare a 'ref' input (AC1, R-003).

    The ref input allows the operator to specify exactly which branch or tag
    is deployed to production Cloudflare Pages.
    """
    workflow = _load_yaml()

    on_block = workflow.get("on", workflow.get(True, {}))
    assert on_block is not None, "'on:' block missing"

    dispatch_block = on_block.get("workflow_dispatch", {}) if isinstance(on_block, dict) else {}
    inputs = dispatch_block.get("inputs", {}) if isinstance(dispatch_block, dict) else {}

    assert "ref" in inputs, (
        f"workflow_dispatch.inputs.ref not declared in deploy-site.yml. "
        f"Found inputs: {list(inputs.keys())}. "
        "A 'ref' input is required to control which commit/tag is deployed (AC1)."
    )


# ---------------------------------------------------------------------------
# TC-5 (P1): checkout step uses github.event.inputs.ref variable
# R-003: ensures the dispatched ref (not main) is actually checked out (AC2)
# ---------------------------------------------------------------------------


@pytest.mark.skip(reason="RED PHASE — deploy-site.yml not yet implemented (Story 8.2 backlog)")
def test_tc5_checkout_uses_event_inputs_ref() -> None:
    """[P1] 8.2-UNIT-005: checkout step must use ${{ github.event.inputs.ref }} (AC2, R-003).

    Without this, the workflow checks out the default branch regardless of the
    ref input, meaning only 'main' is ever deployed to production.
    """
    text = _load_text()
    # Accept both event.inputs.ref and inputs.ref (GitHub expressions vary by workflow syntax)
    pattern = re.compile(
        r"\$\{\{\s*(?:github\.event\.inputs\.ref|inputs\.ref)\s*\}\}"
    )
    assert pattern.search(text), (
        "deploy-site.yml checkout step does not use "
        "${{ github.event.inputs.ref }} or ${{ inputs.ref }}. "
        "Without this, the specified 'ref' input is ignored and main is always deployed (AC2)."
    )


# ---------------------------------------------------------------------------
# TC-6 (P2): site/ working directory for npm run build
# R-008: Astro site lives in site/ subdirectory — build must run there (AC2)
# ---------------------------------------------------------------------------


@pytest.mark.skip(reason="RED PHASE — deploy-site.yml not yet implemented (Story 8.2 backlog)")
def test_tc6_build_uses_site_working_directory() -> None:
    """[P2] 8.2-UNIT-006: npm run build must run inside the site/ directory (AC2, R-008).

    The Astro site is in the site/ subdirectory. Running npm run build from the
    repo root would fail (no package.json there) or deploy wrong output.
    """
    workflow = _load_yaml()

    jobs = workflow.get("jobs", {})
    assert jobs, "No 'jobs:' defined in deploy-site.yml"

    build_step_found = False
    site_dir_found = False

    for _job_name, job_def in jobs.items():
        steps = job_def.get("steps", [])
        for step in steps:
            run_cmd = str(step.get("run", ""))
            if "npm run build" in run_cmd:
                build_step_found = True
                working_dir = step.get("working-directory", "")
                # Accept working-directory: site or cd site/ inside the run block
                if "site" in str(working_dir) or "cd site" in run_cmd:
                    site_dir_found = True
                break
        if build_step_found:
            break

    assert build_step_found, "npm run build step not found in deploy-site.yml jobs"
    assert site_dir_found, (
        "npm run build step does not specify working-directory: site (or cd site/). "
        "The Astro site must be built from the site/ subdirectory (AC2, R-008)."
    )
