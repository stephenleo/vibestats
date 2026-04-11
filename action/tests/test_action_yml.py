"""Schema/unit tests for action.yml — TDD Green Phase.

Story 5.4: Implement action.yml (composite community GitHub Action)
GH Issue: #29

Test IDs follow: 5.4-UNIT-{SEQ}

TDD Phase: GREEN — all tests active (pytest.mark.skip removed).

Run: python -m pytest action/tests/test_action_yml.py -v
"""

import pathlib
import re

import pytest

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

ACTION_YML = pathlib.Path(__file__).parent.parent.parent / "action.yml"


def _load_text() -> str:
    return ACTION_YML.read_text(encoding="utf-8")


def _load_yaml():
    """Load action.yml as a Python dict.

    PyYAML is used here as a dev test dependency — it is NOT used in the
    production action scripts (which are stdlib-only).  If PyYAML is absent
    we fall back to a simple key-extraction helper so the file can at least
    be checked structurally.
    """
    try:
        import yaml  # type: ignore[import]
        with ACTION_YML.open(encoding="utf-8") as fh:
            return yaml.safe_load(fh)
    except ImportError:
        return None


# ---------------------------------------------------------------------------
# TC-1 (P1): action.yml exists and parses as valid YAML — 5.4-UNIT-001
# ---------------------------------------------------------------------------

def test_tc1_action_yml_exists():
    """[P1] 5.4-UNIT-001a: action.yml must exist at the repo root."""
    assert ACTION_YML.exists(), f"action.yml not found at {ACTION_YML}"


def test_tc1_action_yml_not_empty():
    """[P1] 5.4-UNIT-001b: action.yml must not be empty."""
    text = _load_text()
    assert text.strip(), "action.yml is empty"


def test_tc1_action_yml_parses_as_valid_yaml():
    """[P1] 5.4-UNIT-001c: action.yml must parse as valid YAML without errors."""
    parsed = _load_yaml()
    if parsed is not None:
        # PyYAML is available — structural parse succeeded
        assert isinstance(parsed, dict), "action.yml top-level must be a YAML mapping"
    else:
        # Fallback: text-based smoke check — must contain at least 'name:' key
        text = _load_text()
        assert "name:" in text, "action.yml does not appear to contain valid YAML (no 'name:' key)"


# ---------------------------------------------------------------------------
# TC-2 (P1): action.yml declares type composite — 5.4-UNIT-002
# ---------------------------------------------------------------------------

def test_tc2_runs_using_composite():
    """[P1] 5.4-UNIT-002: action.yml must declare runs.using == 'composite' (AC1, architecture.md)."""
    parsed = _load_yaml()
    if parsed is not None:
        runs = parsed.get("runs", {})
        assert runs.get("using") == "composite", (
            f"Expected runs.using == 'composite', got: {runs.get('using')!r}"
        )
    else:
        text = _load_text()
        assert "using: 'composite'" in text or 'using: "composite"' in text or "using: composite" in text, (
            "action.yml does not declare 'using: composite'"
        )


# ---------------------------------------------------------------------------
# TC-3 (P1): action.yml declares token and profile-repo inputs — 5.4-UNIT-003
# ---------------------------------------------------------------------------

def test_tc3_input_token_declared():
    """[P1] 5.4-UNIT-003a: action.yml must declare 'token' input (AC1, R-008)."""
    parsed = _load_yaml()
    if parsed is not None:
        inputs = parsed.get("inputs", {})
        assert "token" in inputs, (
            f"'token' input not found in action.yml inputs. Found: {list(inputs.keys())}"
        )
    else:
        text = _load_text()
        assert re.search(r"^\s+token\s*:", text, re.MULTILINE), (
            "action.yml does not declare a 'token' input"
        )


def test_tc3_input_profile_repo_declared():
    """[P1] 5.4-UNIT-003b: action.yml must declare 'profile-repo' input (AC1, R-008)."""
    parsed = _load_yaml()
    if parsed is not None:
        inputs = parsed.get("inputs", {})
        assert "profile-repo" in inputs, (
            f"'profile-repo' input not found in action.yml inputs. Found: {list(inputs.keys())}"
        )
    else:
        text = _load_text()
        assert re.search(r"^\s+profile-repo\s*:", text, re.MULTILINE), (
            "action.yml does not declare a 'profile-repo' input"
        )


def test_tc3_token_input_is_required():
    """[P1] 5.4-UNIT-003c: 'token' input must be marked required: true (AC1, NFR17)."""
    parsed = _load_yaml()
    if parsed is not None:
        token_input = parsed.get("inputs", {}).get("token", {})
        assert token_input.get("required") is True, (
            f"'token' input must be required: true, got: {token_input.get('required')!r}"
        )
    else:
        text = _load_text()
        # Find the token block and assert required appears in proximity
        token_section = re.search(r"token\s*:.*?(?=\w+\s*:|$)", text, re.DOTALL)
        assert token_section and "required: true" in token_section.group(), (
            "Could not confirm 'token' input is marked required: true"
        )


def test_tc3_profile_repo_input_is_required():
    """[P1] 5.4-UNIT-003d: 'profile-repo' input must be marked required: true (AC1, NFR17)."""
    parsed = _load_yaml()
    if parsed is not None:
        profile_repo_input = parsed.get("inputs", {}).get("profile-repo", {})
        assert profile_repo_input.get("required") is True, (
            f"'profile-repo' input must be required: true, got: {profile_repo_input.get('required')!r}"
        )
    else:
        text = _load_text()
        assert "required: true" in text, (
            "Could not confirm 'profile-repo' input is marked required: true"
        )


# ---------------------------------------------------------------------------
# TC-4 (P1): step sequence is correct — 5.4-UNIT-004
# ---------------------------------------------------------------------------

def test_tc4_step_sequence_has_two_checkouts():
    """[P1] 5.4-UNIT-004a: action.yml must include actions/checkout at least twice
    (vibestats-data + profile-repo checkouts) (AC2, R-003)."""
    text = _load_text()
    checkout_count = text.count("actions/checkout")
    assert checkout_count >= 2, (
        f"Expected at least 2 actions/checkout uses (vibestats-data + profile-repo), "
        f"found {checkout_count}"
    )


def test_tc4_step_sequence_has_setup_python():
    """[P1] 5.4-UNIT-004b: action.yml must include actions/setup-python step (AC2)."""
    text = _load_text()
    assert "actions/setup-python" in text, (
        "action.yml does not include actions/setup-python step"
    )


def test_tc4_step_sequence_has_aggregate_py():
    """[P1] 5.4-UNIT-004c: action.yml must reference aggregate.py step (AC2)."""
    text = _load_text()
    assert "aggregate.py" in text, (
        "action.yml does not reference aggregate.py"
    )


def test_tc4_step_sequence_has_generate_svg_py():
    """[P1] 5.4-UNIT-004d: action.yml must reference generate_svg.py step (AC2)."""
    text = _load_text()
    assert "generate_svg.py" in text, (
        "action.yml does not reference generate_svg.py"
    )


def test_tc4_step_sequence_has_update_readme_py():
    """[P1] 5.4-UNIT-004e: action.yml must reference update_readme.py step (AC2)."""
    text = _load_text()
    assert "update_readme.py" in text, (
        "action.yml does not reference update_readme.py"
    )


def test_tc4_step_sequence_has_git_commit():
    """[P1] 5.4-UNIT-004f: action.yml must include a git commit step (AC2)."""
    text = _load_text()
    assert "git commit" in text, (
        "action.yml does not include a git commit step"
    )


def test_tc4_step_sequence_has_git_push():
    """[P1] 5.4-UNIT-004g: action.yml must include a git push step (AC2)."""
    text = _load_text()
    assert "git push" in text, (
        "action.yml does not include a git push step"
    )


def test_tc4_checkout_precedes_aggregate():
    """[P1] 5.4-UNIT-004h: checkout step(s) must appear before aggregate.py in action.yml (AC2, step order)."""
    text = _load_text()
    checkout_pos = text.find("actions/checkout")
    aggregate_pos = text.find("aggregate.py")
    assert checkout_pos != -1, "actions/checkout not found"
    assert aggregate_pos != -1, "aggregate.py not found"
    assert checkout_pos < aggregate_pos, (
        "checkout must appear before aggregate.py in step sequence"
    )


def test_tc4_aggregate_precedes_generate_svg():
    """[P1] 5.4-UNIT-004i: aggregate.py must appear before generate_svg.py in action.yml (AC2)."""
    text = _load_text()
    aggregate_pos = text.find("aggregate.py")
    generate_svg_pos = text.find("generate_svg.py")
    assert aggregate_pos != -1, "aggregate.py not found"
    assert generate_svg_pos != -1, "generate_svg.py not found"
    assert aggregate_pos < generate_svg_pos, (
        "aggregate.py must appear before generate_svg.py in step sequence"
    )


def test_tc4_generate_svg_precedes_update_readme():
    """[P1] 5.4-UNIT-004j: generate_svg.py must appear before update_readme.py in action.yml (AC2)."""
    text = _load_text()
    generate_svg_pos = text.find("generate_svg.py")
    update_readme_pos = text.find("update_readme.py")
    assert generate_svg_pos != -1, "generate_svg.py not found"
    assert update_readme_pos != -1, "update_readme.py not found"
    assert generate_svg_pos < update_readme_pos, (
        "generate_svg.py must appear before update_readme.py in step sequence"
    )


def test_tc4_update_readme_precedes_git_commit():
    """[P1] 5.4-UNIT-004k: update_readme.py must appear before git commit in action.yml (AC2)."""
    text = _load_text()
    update_readme_pos = text.find("update_readme.py")
    git_commit_pos = text.find("git commit")
    assert update_readme_pos != -1, "update_readme.py not found"
    assert git_commit_pos != -1, "git commit not found"
    assert update_readme_pos < git_commit_pos, (
        "update_readme.py must appear before git commit in step sequence"
    )


def test_tc4_git_commit_precedes_git_push():
    """[P1] 5.4-UNIT-004l: git commit must appear before git push in action.yml (AC2)."""
    text = _load_text()
    git_commit_pos = text.find("git commit")
    git_push_pos = text.find("git push")
    assert git_commit_pos != -1, "git commit not found"
    assert git_push_pos != -1, "git push not found"
    assert git_commit_pos < git_push_pos, (
        "git commit must appear before git push in step sequence"
    )


# ---------------------------------------------------------------------------
# TC-5 (P0): no step uses continue-on-error: true — 5.4-UNIT-005
# ---------------------------------------------------------------------------

def test_tc5_no_continue_on_error():
    """[P0] 5.4-UNIT-005: action.yml must not use 'continue-on-error: true' on any step.

    Any failing step must halt the workflow and exit non-zero (AC3, NFR13, R-003).
    Using continue-on-error would allow partial outputs to be committed.
    """
    text = _load_text()
    # Match both YAML boolean (true) and string ("true")
    pattern = re.compile(r"continue-on-error\s*:\s*(true|\"true\")", re.IGNORECASE)
    matches = pattern.findall(text)
    assert not matches, (
        f"action.yml must not use continue-on-error: true on any step (violates NFR13/AC3). "
        f"Found {len(matches)} occurrence(s)."
    )


# ---------------------------------------------------------------------------
# TC-6 (P1): all shell steps declare shell: bash — 5.4-UNIT-006
# ---------------------------------------------------------------------------

def test_tc6_all_run_steps_have_shell_bash():
    """[P1] 5.4-UNIT-006: Every run: step in action.yml must declare shell: bash.

    GitHub Actions composite actions require explicit shell declaration on every step.
    """
    parsed = _load_yaml()
    if parsed is not None:
        steps = parsed.get("runs", {}).get("steps", [])
        assert steps, "action.yml runs.steps must not be empty"
        for i, step in enumerate(steps):
            if "run" in step:
                assert step.get("shell") == "bash", (
                    f"Step {i} ('{step.get('name', 'unnamed')}') has a 'run:' block "
                    f"but is missing 'shell: bash'. All composite action steps with "
                    f"'run:' must declare shell."
                )
    else:
        text = _load_text()
        # Rough check: count run: occurrences vs shell: bash occurrences
        run_count = len(re.findall(r"^\s+run\s*:\s*[|>]", text, re.MULTILINE))
        shell_bash_count = text.count("shell: bash")
        assert run_count > 0, "No 'run:' steps found in action.yml"
        assert shell_bash_count >= run_count, (
            f"Found {run_count} 'run:' steps but only {shell_bash_count} 'shell: bash' "
            f"declarations — every run step must have shell: bash"
        )


# ---------------------------------------------------------------------------
# TC-7 (P1): branding fields present — 5.4-UNIT-007
# ---------------------------------------------------------------------------

def test_tc7_branding_icon_declared():
    """[P1] 5.4-UNIT-007a: action.yml must declare branding.icon (NFR17, Story 8.3 dependency)."""
    parsed = _load_yaml()
    if parsed is not None:
        branding = parsed.get("branding", {})
        assert "icon" in branding, (
            f"action.yml missing branding.icon (required for GitHub Marketplace). "
            f"Found branding keys: {list(branding.keys())}"
        )
    else:
        text = _load_text()
        assert "icon:" in text, "action.yml does not declare a branding icon"


def test_tc7_branding_color_declared():
    """[P1] 5.4-UNIT-007b: action.yml must declare branding.color (NFR17, Story 8.3 dependency)."""
    parsed = _load_yaml()
    if parsed is not None:
        branding = parsed.get("branding", {})
        assert "color" in branding, (
            f"action.yml missing branding.color (required for GitHub Marketplace). "
            f"Found branding keys: {list(branding.keys())}"
        )
    else:
        text = _load_text()
        assert "color:" in text, "action.yml does not declare a branding color"


# ---------------------------------------------------------------------------
# TC-8 (P1): steps list is non-empty — 5.4-UNIT-008
# ---------------------------------------------------------------------------

def test_tc8_steps_list_is_non_empty():
    """[P1] 5.4-UNIT-008: action.yml runs.steps must not be empty [] (AC2)."""
    parsed = _load_yaml()
    if parsed is not None:
        steps = parsed.get("runs", {}).get("steps", [])
        assert steps, (
            "action.yml runs.steps is empty — must contain the 8-step composite action pipeline"
        )
        assert len(steps) >= 8, (
            f"Expected at least 8 steps (checkout×2, setup-python, aggregate, "
            f"generate_svg, update_readme, commit, push), found {len(steps)}"
        )
    else:
        text = _load_text()
        assert "steps: []" not in text and "steps:\n  []" not in text, (
            "action.yml runs.steps is empty — must contain the full step sequence"
        )
