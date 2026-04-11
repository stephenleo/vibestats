# Story 5.3: Implement update_readme.py

Status: done

<!-- GH Issue: #28 | Epic: #5 | PR must include: Closes #28 -->

## Story

As the GitHub Actions pipeline,
I want a README updater that injects the SVG embed and dashboard link between the vibestats markers,
So that the profile README is updated automatically on every cron run.

## Acceptance Criteria

1. **Given** the profile README contains `<!-- vibestats-start -->` and `<!-- vibestats-end -->` markers **When** `update_readme.py` runs **Then** it replaces content between markers with the SVG `<img>` tag pointing to `raw.githubusercontent.com/username/username/main/vibestats/heatmap.svg` and a "View interactive dashboard →" link to `vibestats.dev/username` (FR24, FR28)

2. **Given** the markers are missing **When** `update_readme.py` runs **Then** it exits non-zero with a clear error explaining the markers must be present

3. **Given** the README content between markers has not changed **When** `update_readme.py` runs **Then** it skips the git commit step (no empty commits) (NFR13)

## Tasks / Subtasks

- [x] Task 1: Implement `action/update_readme.py` (AC: #1, #2, #3)
  - [x] Replace the stub in `action/update_readme.py` (the file already exists with a `pass` body — do NOT create a new file)
  - [x] Accept CLI args: `--username <str>` and `--readme-path <path>` (default: `README.md` in the working directory)
  - [x] Read the README file; exit non-zero with a clear error message if the file does not exist
  - [x] Locate `<!-- vibestats-start -->` and `<!-- vibestats-end -->` markers; exit non-zero with `"ERROR: vibestats markers not found in README. Add <!-- vibestats-start --> and <!-- vibestats-end --> to your profile README."` if either is missing (AC #2)
  - [x] Build the replacement block:
    ```
    <!-- vibestats-start -->
    <img src="https://raw.githubusercontent.com/{username}/{username}/main/vibestats/heatmap.svg" alt="vibestats heatmap" />

    [View interactive dashboard →](https://vibestats.dev/{username})
    <!-- vibestats-end -->
    ```
  - [x] Compare the replacement block to the existing block between the markers (normalised to strip leading/trailing whitespace); if identical, print `"vibestats: README already up to date — skipping commit"` and exit 0 (AC #3, NFR13)
  - [x] Write the updated README to disk only if content changed (AC #1)
  - [x] Print `"vibestats: README updated"` on successful write
  - [x] stdlib only — `pathlib`, `re`, `sys`, `argparse` — NO pip dependencies
  - [x] Exit non-zero on any unexpected error (Python Actions fail-loudly contract)

- [x] Task 2: Write unit tests in `action/tests/test_update_readme.py` (AC: #1, #2, #3)
  - [x] Create `action/tests/test_update_readme.py` (the `action/tests/` directory and `__init__.py` already exist)
  - [x] TC-1 (P0): markers present → content between markers is replaced with correct `<img>` tag and dashboard link (AC #1)
  - [x] TC-2 (P1): correct `raw.githubusercontent.com` URL pattern in injected content (AC #1)
  - [x] TC-3 (P0): markers absent → script exits non-zero with error message containing "vibestats markers" (AC #2, R-007)
  - [x] TC-4 (P0): identical content → script exits 0 and does NOT write file (no-op idempotent run) (AC #3, R-004)
  - [x] TC-5 (P1): markers present but content changed → file is written with new content
  - [x] Use `subprocess.run` to invoke `action/update_readme.py` via `python -m` or directly; use `tmp_path` fixture (or `tempfile.TemporaryDirectory`) for temporary README files
  - [x] Run `python -m pytest action/tests/test_update_readme.py -v` — must pass with 0 failures

### Review Findings

Code review conducted on 2026-04-11 (bmad-code-review). 3 layers ran: Blind Hunter, Edge Case Hunter, Acceptance Auditor. 1 patch applied, 1 deferred, 3 dismissed as noise/false-positive.

- [x] [Review][Patch] `re.sub` treats `new_block` as a regex replacement string — backslash sequences like `\1` or `\g<1>` in the interpolated username would be interpreted as backreferences and could corrupt output. Fixed by switching to a lambda replacement (`PATTERN.sub(lambda _m: new_block, content, count=1)`) so the block is inserted as a literal string. [action/update_readme.py:72]
- [x] [Review][Defer] No validation for empty `--username` string — argparse permits `--username ""`, producing broken URLs without failing. Low priority; Actions context always passes a real GitHub login. Deferred to deferred-work.md.

## Dev Notes

### Module Responsibility

`action/update_readme.py` is the sole owner of profile README injection. It:
1. Reads `README.md` (or a path from `--readme-path`)
2. Finds `<!-- vibestats-start -->` / `<!-- vibestats-end -->` markers
3. Builds the replacement block (SVG `<img>` + dashboard link)
4. Skips commit step if content unchanged (no-op path)
5. Writes the file if changed
6. Exits non-zero on any failure (fail-loudly contract)

### File to Modify

`action/update_readme.py` already exists as a stub:

```python
"""update_readme.py — Updates the GitHub profile README with the generated SVG heatmap.

Implementation: Epic 5, Story 5.3.
"""

if __name__ == "__main__":
    pass
```

Replace the stub entirely. Do NOT create a new file.

### Injected Block Format

```
<!-- vibestats-start -->
<img src="https://raw.githubusercontent.com/{username}/{username}/main/vibestats/heatmap.svg" alt="vibestats heatmap" />

[View interactive dashboard →](https://vibestats.dev/{username})
<!-- vibestats-end -->
```

The markers themselves are included in the replacement block (the start and end markers are preserved as the outer boundary). The `<img>` tag and dashboard link are the full content between them.

### Marker Detection Strategy

Use `re` for reliable detection across varying whitespace:

```python
import re

START_MARKER = "<!-- vibestats-start -->"
END_MARKER   = "<!-- vibestats-end -->"

pattern = re.compile(
    r"(<!-- vibestats-start -->)(.*?)(<!-- vibestats-end -->)",
    re.DOTALL
)
```

If `pattern.search(content)` finds no match → exit non-zero with the error message.

### Idempotency / Skip Logic

After building the replacement block, compare it to the existing matched region (the full match including markers) with `.strip()` normalisation:

```python
if existing_block.strip() == new_block.strip():
    print("vibestats: README already up to date — skipping commit")
    sys.exit(0)
```

Only write the file when the blocks differ. This prevents empty commits (NFR13, R-004).

### CLI Interface

```python
import argparse, pathlib, re, sys

def parse_args():
    p = argparse.ArgumentParser(description="Inject vibestats heatmap into profile README")
    p.add_argument("--username", required=True, help="GitHub username")
    p.add_argument("--readme-path", default="README.md", help="Path to profile README")
    return p.parse_args()
```

### Architecture Constraints

| Constraint | Source | Impact |
|---|---|---|
| stdlib only | architecture.md | `re`, `pathlib`, `argparse`, `sys` — no pip installs |
| Exit non-zero on failure | architecture.md, AC #2 | `sys.exit(1)` on missing markers, missing file, write error |
| Skip commit when unchanged | NFR13, AC #3 | Compare blocks before writing; exit 0 with message when identical |
| snake_case filename | architecture.md | `update_readme.py` (already correct) |
| Test file location | architecture.md | `action/tests/test_update_readme.py` |
| `git push` 3-retry loop | architecture.md (minor note) | This script does NOT handle git push — that is done by `action.yml` (Story 5.4). `update_readme.py` only writes the file. |

### Key Distinction: What This Script Does NOT Do

`update_readme.py` does **not** run `git commit` or `git push`. It only:
- Reads the README
- Updates it on disk if content changed
- Exits 0 (no-op) or exits 0 (written) or exits non-zero (error)

The git commit/push step, including the 3-retry loop for transient failures, lives in `action.yml` (Story 5.4). The skip logic here prevents `action.yml` from attempting to commit an unchanged file by exiting with a distinct message the workflow can check (or simply by not modifying the file — no diff means git commit is a no-op).

### Error Handling Contract

| Failure | Exit Code | Message |
|---|---|---|
| README file not found | 1 | `"ERROR: README not found at {path}"` |
| Markers missing | 1 | `"ERROR: vibestats markers not found in README. Add <!-- vibestats-start --> and <!-- vibestats-end --> to your profile README."` |
| Content unchanged | 0 | `"vibestats: README already up to date — skipping commit"` |
| File write error | 1 | `"ERROR: could not write {path}: {e}"` |
| Success (file written) | 0 | `"vibestats: README updated"` |

### File Structure

```
action/
├── aggregate.py         ← EXISTING (Story 5.1) — do NOT touch
├── generate_svg.py      ← EXISTING (Story 5.2) — do NOT touch
├── update_readme.py     ← MODIFY THIS — replace stub with implementation
└── tests/
    ├── __init__.py      ← EXISTING — do NOT touch
    ├── fixtures/        ← EXISTING — may add fixture README files here
    │   ├── expected_output/
    │   └── sample_machine_data/
    └── test_update_readme.py  ← NEW — create this
```

### Anti-Patterns to Prevent

- Do NOT use any non-stdlib library (no `requests`, no `yaml`, no `toml` — stdlib only)
- Do NOT modify `aggregate.py` or `generate_svg.py`
- Do NOT run `git commit` or `git push` from this script — that is `action.yml`'s job
- Do NOT silently continue when markers are missing — exit non-zero with a clear error (AC #2, R-007)
- Do NOT write the file when content has not changed — compare first (NFR13, R-004)
- Do NOT use `exit()` (use `sys.exit()`)
- Do NOT hard-code paths outside of `--readme-path` default

### References

- Story requirements: [Source: _bmad-output/planning-artifacts/epics.md#Story 5.3]
- FR24 (README marker injection), FR28 (dashboard link): [Source: _bmad-output/planning-artifacts/epics.md#Functional Requirements]
- NFR13 (Actions resilience — no empty commits): [Source: _bmad-output/planning-artifacts/epics.md#NonFunctional Requirements]
- stdlib-only constraint: [Source: _bmad-output/planning-artifacts/architecture.md#Technical Constraints]
- Fail-loudly contract for Python Actions scripts: [Source: _bmad-output/planning-artifacts/architecture.md#Implementation Patterns & Consistency Rules]
- 3-retry git push (action.yml, Story 5.4): [Source: _bmad-output/planning-artifacts/architecture.md#Architecture Completeness Checklist Minor Notes]
- Test design risks R-004, R-007: [Source: _bmad-output/test-artifacts/test-design-epic-5.md]
- GH Issue: #28

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

None.

### Completion Notes List

- Implemented `action/update_readme.py` replacing the stub with full implementation using stdlib only (argparse, pathlib, re, sys).
- Script reads README, locates vibestats markers via regex, builds replacement block with SVG img tag and dashboard link, skips write if content unchanged (idempotent), writes file if changed.
- All error paths exit non-zero with clear messages per the fail-loudly contract.
- Updated `action/tests/test_update_readme.py` — removed @pytest.mark.skip decorators to activate GREEN phase; all 5 tests pass.
- AC #1 (replace content between markers), AC #2 (exit non-zero if markers missing), AC #3 (skip commit if unchanged) all satisfied.

### File List

- action/update_readme.py (modified — stub replaced with full implementation)
- action/tests/test_update_readme.py (modified — skip decorators removed, tests activated)
- _bmad-output/implementation-artifacts/5-3-implement-update-readme-py.md (modified — tasks checked, status updated)

### Change Log

- 2026-04-11: Implemented update_readme.py (Story 5.3) — README marker injection with idempotency check; 5 unit tests added and passing.
