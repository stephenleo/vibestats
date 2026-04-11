# Story 5.4: Implement action.yml (composite community GitHub Action)

Status: review

<!-- GH Issue: #29 | Epic: #5 | PR must include: Closes #29 -->

## Story

As a vibestats user,
I want a community GitHub Action at `stephenleo/vibestats` referenceable as `uses: stephenleo/vibestats@v1`,
So that I pick up aggregation fixes and SVG updates automatically without managing Python scripts myself.

## Acceptance Criteria

1. **Given** `action.yml` exists at the repo root **When** it is reviewed **Then** it declares type `composite`, inputs `token` (VIBESTATS_TOKEN) and `profile-repo` (username/username), and all required permissions (NFR17)

2. **Given** the action runs in a user's `vibestats-data` workflow **When** it executes **Then** it: (1) checks out vibestats-data, (2) runs `aggregate.py`, (3) runs `generate_svg.py`, (4) runs `update_readme.py`, (5) commits and pushes outputs to `profile-repo` using the `token` input (FR23)

3. **Given** any action step fails **When** the workflow exits **Then** it exits non-zero and no partial outputs are committed (NFR13)

## Tasks / Subtasks

- [x] Task 1: Implement `action.yml` at the repo root (AC: #1, #2, #3)
  - [x] `action.yml` already exists at the repo root as a stub — modify it in place (do NOT create a new file)
  - [x] Current stub content:
    ```yaml
    name: 'vibestats'
    description: 'Aggregate Claude Code session activity and update your GitHub profile heatmap'
    runs:
      using: 'composite'
      steps: []
    ```
  - [x] Set `name: 'vibestats'` (already correct)
  - [x] Add Marketplace branding fields: `branding.icon: 'activity'` and `branding.color: 'orange'` (required for GitHub Actions Marketplace — NFR17, Story 8.3 dependency)
  - [x] Declare `inputs` section with:
    - `token`: required, description `"Fine-grained PAT with Contents write access to profile-repo (VIBESTATS_TOKEN)"`
    - `profile-repo`: required, description `"GitHub profile repo in format username/username"`
  - [x] Set `runs.using: 'composite'` (already correct)
  - [x] Implement `runs.steps` in this exact order (AC #2 step sequence):
    1. **Checkout vibestats-data** — use `actions/checkout@v4`, no token needed (action runs inside the user's `vibestats-data` workflow, so the GITHUB_TOKEN has access; the checkout path is the default `$GITHUB_WORKSPACE`)
    2. **Checkout profile-repo** — use `actions/checkout@v4` with `repository: ${{ inputs.profile-repo }}`, `token: ${{ inputs.token }}`, and `path: _profile_repo` (checkout to a subdirectory to avoid overwriting the vibestats-data checkout)
    3. **Set up Python** — use `actions/setup-python@v5` with `python-version: '3.x'`
    4. **Run aggregate.py** — `shell: bash`, run inside the vibestats-data checkout (default CWD), write output directly to profile-repo path:
       ```bash
       mkdir -p _profile_repo/vibestats
       python ${{ github.action_path }}/action/aggregate.py
       mv data.json _profile_repo/vibestats/data.json
       ```
       (`aggregate.py` reads `./machines/...` and writes `./data.json` to CWD — vibestats-data root. Output must be moved to `_profile_repo/vibestats/data.json`.)
    5. **Run generate_svg.py** — `shell: bash`, with explicit paths:
       ```bash
       python ${{ github.action_path }}/action/generate_svg.py \
         --input _profile_repo/vibestats/data.json \
         --output _profile_repo/vibestats/heatmap.svg
       ```
    6. **Run update_readme.py** — `shell: bash`, derive username and pass profile repo README path:
       ```bash
       USERNAME=$(echo "${{ inputs.profile-repo }}" | cut -d'/' -f1)
       python ${{ github.action_path }}/action/update_readme.py \
         --username "$USERNAME" \
         --readme-path _profile_repo/README.md
       ```
    7. **Commit outputs to profile-repo** — `shell: bash`, run inside `_profile_repo`:
       ```bash
       cd _profile_repo
       git config user.name "vibestats[bot]"
       git config user.email "vibestats[bot]@users.noreply.github.com"
       git add vibestats/data.json vibestats/heatmap.svg README.md
       git diff --cached --quiet && echo "vibestats: no changes to commit" && exit 0
       git commit -m "chore(vibestats): update heatmap [skip ci]"
       ```
    8. **Push to profile-repo** with 3-retry loop — `shell: bash`, run inside `_profile_repo`:
       ```bash
       cd _profile_repo
       for i in 1 2 3; do
         git push && exit 0
         sleep $((i * 2))
       done
       exit 1
       ```
       (The profile-repo was checked out with the `token` input as credential — `git push` uses that token automatically. No need to embed the token in the push URL.)
  - [x] Every step that runs a shell command MUST include `shell: bash` (required for composite action steps)
  - [x] Each `run` block in the composite action implicitly uses `set -e` behaviour when `shell: bash` is specified — every command failure exits the step non-zero, halting the workflow (AC #3, NFR13)
  - [x] Step 7 uses `exit 0` for no-change early return — this exits ONLY that step successfully. Step 8 still runs but `git push` with nothing new to push will be a no-op. This is correct behaviour for composite actions.

- [x] Task 2: Write schema/unit tests in `action/tests/test_action_yml.py` (AC: #1, #2, #3)
  - [x] Create `action/tests/test_action_yml.py` (the `action/tests/` directory and `__init__.py` already exist)
  - [x] TC-1 (P1): `action.yml` parses as valid YAML — load with `yaml` stdlib equivalent or `import yaml` (stdlib: use `import tomllib` is not applicable; use `python -c "import yaml"` from PyYAML if available, or parse manually) — **NOTE:** `yaml` (PyYAML) is NOT stdlib. The test must use stdlib only. Use `subprocess` to validate via a Python one-liner, OR implement the schema test by loading the file with a YAML-safe parse. Because the project uses stdlib-only for action scripts, test files may use `PyYAML` if it is already installed in the test environment — check `action/tests/` for precedent. If no YAML library is available, parse `action.yml` as text and assert key substrings. See Dev Notes.
  - [x] TC-2 (P1): `action.yml` declares `type: composite` — assert `runs.using == 'composite'`
  - [x] TC-3 (P1): `action.yml` declares `token` and `profile-repo` inputs — assert both keys present in `inputs`
  - [x] TC-4 (P1): step sequence is correct — assert steps contain (in order) references to: `actions/checkout` (twice: vibestats-data + profile-repo), `actions/setup-python`, `aggregate.py`, `generate_svg.py`, `update_readme.py`, git commit, git push (AC #2, test-design R-003, R-008)
  - [x] TC-5 (P0): any failing step blocks commit — assert no step uses `continue-on-error: true` (AC #3, NFR13)
  - [x] Run `python -m pytest action/tests/test_action_yml.py -v` — must pass with 0 failures

### Deferred Work (Do NOT implement in this story)

- `aggregate.yml` workflow template (Story 5.5)
- Any changes to `aggregate.py`, `generate_svg.py`, or `update_readme.py` — all are DONE and must not be touched
- Any changes to existing test files (`test_aggregate.py`, `test_generate_svg.py`, `test_update_readme.py`)

## Dev Notes

### What Exists

`action.yml` already exists at the repo root as a stub. Modify it in place — do NOT delete and recreate.

```
vibestats/
├── action.yml                       ← MODIFY THIS stub (repo root)
├── action/
│   ├── aggregate.py                 ← EXISTING (Story 5.1) — do NOT touch
│   ├── generate_svg.py              ← EXISTING (Story 5.2) — do NOT touch
│   ├── update_readme.py             ← EXISTING (Story 5.3) — do NOT touch
│   └── tests/
│       ├── __init__.py              ← EXISTING — do NOT touch
│       ├── test_aggregate.py        ← EXISTING — do NOT touch
│       ├── test_generate_svg.py     ← EXISTING — do NOT touch
│       ├── test_update_readme.py    ← EXISTING — do NOT touch
│       └── test_action_yml.py       ← NEW — create this
│       └── fixtures/
│           ├── expected_output/
│           └── sample_machine_data/
```

### Architecture Constraints

| Constraint | Source | Value |
|---|---|---|
| Action type | architecture.md | `composite` (not Docker) — faster startup, no image pull |
| Inputs | architecture.md | `token` (VIBESTATS_TOKEN), `profile-repo` (username/username) |
| Step order | epics.md AC #2 | checkout vibestats-data → checkout profile-repo → aggregate → generate_svg → update_readme → commit → push |
| Git config | architecture.md minor note | `git config user.name "vibestats[bot]"` + email before commit |
| Push retry | architecture.md minor note | 3-retry loop for transient git push failures |
| Fail loudly | NFR13, AC #3 | Any step failure = non-zero exit, no partial commit |
| Marketplace branding | NFR17, Story 8.3 | `branding.icon` + `branding.color` required for Marketplace listing |
| Shell declaration | GitHub Actions composite requirement | Every step must have `shell: bash` |

### Script CLI Interface (Critical — Read Before Implementing)

**aggregate.py** (from implementation):
- Takes no CLI args; uses `pathlib.Path(".")` as root
- Reads `./machines/year=*/month=*/day=*/harness=*/machine_id=*/data.json`
- Writes `./data.json` to CWD
- Gets username from `GITHUB_REPOSITORY_OWNER` env var (set automatically in GitHub Actions)

**generate_svg.py** (from implementation):
- `--input vibestats/data.json` (default)
- `--output vibestats/heatmap.svg` (default)
- Must be called with explicit paths when not running from profile-repo root

**update_readme.py** (from implementation):
- `--username <str>` (required)
- `--readme-path <path>` (default: `README.md` in CWD)

### Two-Checkout Pattern

The action requires TWO checkouts because:
1. **vibestats-data** is the source of Hive partition files that `aggregate.py` reads
2. **profile-repo** (`username/username`) is the destination where outputs (`data.json`, `heatmap.svg`) and README updates are committed

```
$GITHUB_WORKSPACE/          ← vibestats-data checkout (Step 1)
  machines/
    year=.../...
  registry.json
  data.json                  ← aggregate.py output (before mv)
_profile_repo/               ← profile-repo checkout (Step 2, path: _profile_repo)
  README.md
  vibestats/
    data.json                ← moved here from $GITHUB_WORKSPACE/data.json
    heatmap.svg              ← generate_svg.py output
```

### Deriving `--username` from `profile-repo`

`update_readme.py` requires `--username <str>`. The input `profile-repo` is `username/username`. Extract in bash:

```bash
USERNAME=$(echo "${{ inputs.profile-repo }}" | cut -d'/' -f1)
```

### git push 3-Retry Loop

The profile-repo is checked out with `token: ${{ inputs.token }}` in the checkout step. GitHub Actions persists the token as a git credential for the checked out repo. Therefore `git push` (no URL needed) uses the token credential automatically.

Architecture.md calls for a 3-retry loop:

```bash
cd _profile_repo
for i in 1 2 3; do
  git push && exit 0
  sleep $((i * 2))
done
exit 1
```

The `VIBESTATS_TOKEN` has Contents write scope for `username/username` only (NFR7).

### Early-Return on No Changes (Step 7 → Step 8)

When `git diff --cached --quiet` returns 0 (no staged changes), Step 7 exits 0 to skip the commit. In composite actions, `exit 0` inside a `run:` block exits ONLY that step successfully — Step 8 (push) still runs. This is fine: `git push` with nothing new to push is a no-op (returns 0). No harm done.

Alternative: skip Step 8 entirely by setting an output flag from Step 7 and using `if:` on Step 8. This is cleaner but requires `outputs` in the action — not required by AC. Use the simpler approach (exit 0 in commit, allow push to no-op).

### Commit Outputs Location

FR23: outputs committed to `username/username/vibestats/`:
- `vibestats/data.json` (aggregate.py output, moved from vibestats-data root)
- `vibestats/heatmap.svg` (generate_svg.py output)

Profile README also updated by update_readme.py (`_profile_repo/README.md`).

### Test Strategy for action.yml

Because `action.yml` is YAML, not Python, the test approach is schema/text-based. Use Python's stdlib to parse the file:

```python
# action/tests/test_action_yml.py
import pathlib
import re

# action.yml location relative to the test file
ACTION_YML = pathlib.Path(__file__).parent.parent.parent / "action.yml"

def test_action_yml_exists():
    assert ACTION_YML.exists()

def _load_text():
    return ACTION_YML.read_text()
```

If PyYAML is available (check existing test files for imports), parse structurally. If not, use text-based assertions. Precedent from existing tests (test_aggregate.py, test_generate_svg.py) should guide the approach.

Check existing tests to confirm whether PyYAML is already used:
```
action/tests/test_aggregate.py
action/tests/test_generate_svg.py
```

### What update_readme.py Does NOT Do

`update_readme.py` does NOT run `git commit` or `git push`. It only writes the README file to disk. The git commit and push steps live entirely in `action.yml`. This is the design established in Story 5.3 (`update_readme.py` exit 0 when file changed, git operations are the action's responsibility).

### Key References: Previous Story Patterns

From Story 5.3 dev notes and implementation:
- Python scripts use stdlib only (`argparse`, `pathlib`, `re`, `sys`)
- Exit non-zero on any failure (fail-loudly contract)
- Test files use `subprocess.run` to invoke scripts, `tmp_path` fixture for temp files
- `action/tests/__init__.py` already exists — do NOT recreate
- Review Findings pattern: `# [Review][Patch]` inline comments in story file

From Stories 5.1-5.3 git history:
- All PRs titled: `story-5.X-slug - fixes #N (#PR)`
- Story files updated: tasks checked, status set to `done`, file list populated

### Error Handling Contract for action.yml

| Step | Failure Behaviour |
|---|---|
| checkout vibestats-data | `actions/checkout` fails loudly by default |
| checkout profile-repo | `actions/checkout` fails loudly; invalid token = auth error |
| setup-python | `actions/setup-python` fails loudly by default |
| aggregate.py | exits non-zero → composite action stops, no subsequent steps run |
| generate_svg.py | exits non-zero → composite action stops, no subsequent steps run |
| update_readme.py | exits non-zero → composite action stops, no commit/push attempted |
| git commit | `git diff --cached --quiet` empty = skip with exit 0 (NFR13 — no empty commits) |
| git push (3-retry) | all 3 retries fail → `exit 1` → workflow fails, no partial output committed |

### Anti-Patterns to Prevent

- Do NOT use `continue-on-error: true` on any step (breaks NFR13 fail-loudly contract)
- Do NOT use `runs-on` in the action — that belongs in the calling workflow (aggregate.yml), not the composite action
- Do NOT use Docker-based action (`using: docker`) — must be `composite` for Marketplace speed
- Do NOT try to push to profile-repo without checking it out first — `update_readme.py` needs the profile README file from `_profile_repo/README.md`
- Do NOT run aggregate.py from the `_profile_repo` directory — it reads `./machines/...` which only exists in the vibestats-data checkout
- Do NOT call generate_svg.py without explicit `--input` and `--output` args — the defaults (`vibestats/data.json`, `vibestats/heatmap.svg`) only work if CWD is the profile-repo root
- Do NOT add pip install steps — Python scripts use stdlib only (NFR, architecture.md)
- Do NOT add `outputs` section unless explicitly specified — not required by AC
- Do NOT modify `aggregate.py`, `generate_svg.py`, `update_readme.py`, or existing test files
- Do NOT embed the token in the git push URL — the checkout step with `token:` handles auth automatically

### References

- Story requirements: [Source: _bmad-output/planning-artifacts/epics.md#Story 5.4]
- FR23 (commit heatmap.svg + data.json to username/username/vibestats/): [Source: _bmad-output/planning-artifacts/epics.md#Functional Requirements]
- NFR5 (≤60 min/month): [Source: _bmad-output/planning-artifacts/epics.md#NonFunctional Requirements]
- NFR13 (no partial commits, retry): [Source: _bmad-output/planning-artifacts/epics.md#NonFunctional Requirements]
- NFR17 (Marketplace compatibility, `ubuntu-latest`): [Source: _bmad-output/planning-artifacts/epics.md#NonFunctional Requirements]
- Composite action architecture: [Source: _bmad-output/planning-artifacts/architecture.md#Infrastructure & Deployment]
- git config + retry note: [Source: _bmad-output/planning-artifacts/architecture.md#Architecture Completeness Checklist Minor Notes]
- Test design R-003 (any step failure exits non-zero), R-008 (inputs schema): [Source: _bmad-output/test-artifacts/test-design-epic-5.md]
- Story 8.3 dependency (branding fields for Marketplace): [Source: _bmad-output/planning-artifacts/epics.md#Story 8.3]
- GH Issue: #29

### Project Structure Notes

- `action.yml` lives at the **repo root** (`stephenleo/vibestats/action.yml`), not inside `action/`
- Python scripts are in `action/` and are referenced via `${{ github.action_path }}/action/<script>.py` in action steps — `github.action_path` resolves to the root of the `stephenleo/vibestats` checkout
- Tests live in `action/tests/` — existing pattern from Stories 5.1-5.3

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

None.

### Completion Notes List

- Implemented `action.yml` as a composite GitHub Action with 8 steps: checkout vibestats-data, checkout profile-repo (with token auth), setup-python, aggregate.py, generate_svg.py, update_readme.py, commit outputs, push with 3-retry loop.
- Added `branding.icon: 'activity'` and `branding.color: 'orange'` for GitHub Marketplace (NFR17).
- Declared `token` and `profile-repo` as required inputs.
- Every `run:` step includes `shell: bash` (composite action requirement).
- No `continue-on-error: true` on any step (NFR13, AC3).
- Removed `@pytest.mark.skip` from all 25 tests in `action/tests/test_action_yml.py` (TDD green phase).
- Full test suite: 80 passed, 0 failed, 0 skipped.

### File List

- `action.yml` (modified)
- `action/tests/test_action_yml.py` (modified — TDD green phase, skip decorators removed)

## Change Log

- 2026-04-12: Implemented `action.yml` composite GitHub Action (8 steps: checkout×2, setup-python, aggregate.py, generate_svg.py, update_readme.py, commit, push with retry). Added branding fields for Marketplace. Activated 25 ATDD schema/unit tests (TDD green phase). All 80 tests pass.
