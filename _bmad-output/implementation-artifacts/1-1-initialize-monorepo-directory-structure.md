# Story 1.1: Initialize Monorepo Directory Structure

Status: done

<!-- GH Issue: #9 | Epic: #1 | PR must include: Closes #9 -->

## Story

As a developer,
I want the vibestats monorepo initialized with the correct directory structure,
so that all components have a defined home and the project is ready for implementation.

## Acceptance Criteria

1. **Given** the repo is empty **When** the monorepo structure is created **Then** the following paths all exist:
   - `src/` (Rust binary home)
   - `action/` (Python Actions scripts home)
   - `site/` (Astro site home)
   - `install.sh` (stub, executable)
   - `action.yml` (community action stub)
   - `Cargo.toml` (root Cargo manifest stub)
   - `.github/workflows/` directory

2. **Given** the structure is created **When** `action/` is reviewed **Then** the following stubs exist:
   - `action/aggregate.py`
   - `action/generate_svg.py`
   - `action/update_readme.py`
   - `action/tests/` directory (with `.gitkeep` or empty `__init__.py`)

3. **Given** the structure is created **When** root files are reviewed **Then** the following all exist:
   - `README.md` with install command placeholder (`curl -fsSL https://vibestats.dev/install.sh | bash`) and `vibestats.dev` link
   - `CONTRIBUTING.md` with basic contribution guidelines
   - `LICENSE` containing the MIT license text (copyright holder: stephenleo)
   - `.gitignore` covering:
     - Rust build artifacts (`/target`, `Cargo.lock` is kept — binary project)
     - Node/Astro build output (`node_modules/`, `site/dist/`, `site/.astro/`)
     - macOS artifacts (`.DS_Store`)
     - Editor artifacts (`.idea/`, `.vscode/`)

## Tasks / Subtasks

- [x] Task 1: Create top-level directory skeleton (AC: #1)
  - [x] Create `src/` directory with `.gitkeep`
  - [x] Create `action/` directory
  - [x] Create `site/` directory with `.gitkeep`
  - [x] Create `.github/workflows/` directory with `.gitkeep`

- [x] Task 2: Create action/ Python stub files (AC: #2)
  - [x] Create `action/aggregate.py` (module docstring + `if __name__ == "__main__": pass`)
  - [x] Create `action/generate_svg.py` (module docstring + `if __name__ == "__main__": pass`)
  - [x] Create `action/update_readme.py` (module docstring + `if __name__ == "__main__": pass`)
  - [x] Create `action/tests/` directory with empty `__init__.py`
  - [x] Create `action/tests/fixtures/sample_machine_data/` directory with `.gitkeep`
  - [x] Create `action/tests/fixtures/expected_output/` directory with `.gitkeep`

- [x] Task 3: Create root stub files (AC: #1)
  - [x] Create `install.sh` stub (shebang `#!/usr/bin/env bash`, comment "TODO: implement installer")
  - [x] Create `action.yml` stub (minimal composite action YAML with `name`, `description`, `runs.using: composite`)
  - [x] Create `Cargo.toml` stub (package name `vibestats`, edition `2021`, version `0.1.0` — no dependencies yet; those are added in story 1.2)

- [x] Task 4: Create root documentation and config files (AC: #3)
  - [x] Create `README.md` with install command placeholder and `vibestats.dev` link
  - [x] Create `CONTRIBUTING.md`
  - [x] Create `LICENSE` (MIT, year 2026, copyright stephenleo)
  - [x] Create `.gitignore` covering Rust, Node/Astro, macOS, editor artifacts

- [x] Task 5: Verify structure integrity
  - [x] Run `git status` to confirm all files are tracked
  - [x] Confirm no files land in wrong directories

### Review Findings

- [x] [Review][Patch] Restore `.env`/`.env.local`/`*.toml.bak` ignore entries [.gitignore] — The rewrite of `.gitignore` dropped the pre-existing secret-file protections. Removing env-file ignores is a security regression. Fix: re-added `.env`, `.env.local`, `*.toml.bak` under a "Local env / secrets" section.
- [x] [Review][Patch] Restore BMAD tooling directory ignores [.gitignore] — The rewrite dropped `_bmad/`, `.claude/`, `.agents/` entries. These directories were explicitly removed from version control in commit 85957e6 and still exist in the parent working tree. Fix: re-added them under a "BMAD tooling" section.

## Dev Notes

### Architecture Context

This story establishes the monorepo root. Every subsequent story depends on this layout being correct. The architecture defines a strict component-to-directory mapping that must be followed exactly.

**Monorepo layout (canonical — from Architecture doc):**

```
vibestats/
├── README.md
├── CONTRIBUTING.md
├── LICENSE
├── action.yml          ← Community GitHub Action definition (composite)
├── install.sh          ← Bash installer
├── Cargo.toml          ← Rust workspace/binary manifest
├── Cargo.lock          ← Keep in VCS (binary project, not library)
├── .gitignore
├── src/                ← Rust binary (stories 1.2 and Epic 2+)
├── action/             ← Python Actions scripts (Epic 5)
│   ├── aggregate.py
│   ├── generate_svg.py
│   ├── update_readme.py
│   └── tests/
│       ├── __init__.py
│       └── fixtures/
│           ├── sample_machine_data/   ← test input: sample Hive partition files
│           └── expected_output/       ← test assertions: expected SVG + data.json
├── site/               ← Astro site (Epic 7)
└── .github/
    └── workflows/      ← CI/CD YAMLs (Epics 3, 5, 8)
```

**Do NOT create files inside `src/`, `site/`, or `.github/workflows/` beyond directory placeholders** — those are fully handled in stories 1.2 (Rust), 1.3 (Astro), and later epics respectively.

### File Naming and Conventions

| Component | Convention | Examples |
|---|---|---|
| Rust | `snake_case` | `jsonl_parser.rs`, `github_api.rs` |
| Python | `snake_case` | `aggregate.py`, `generate_svg.py` |
| Shell | `kebab-case` | `install.sh` |
| YAML | `kebab-case` | `action.yml`, `release.yml` |

### Cargo.toml Notes

- Story 1.2 runs `cargo new vibestats --bin` which will generate `src/main.rs` and the full `Cargo.toml` with `[dependencies]`. This story only creates a stub `Cargo.toml` to mark the package root.
- Do NOT add crate dependencies in this story — that is AC in story 1.2.
- The stub `Cargo.toml` should be valid TOML but minimal:

```toml
[package]
name = "vibestats"
version = "0.1.0"
edition = "2021"
```

- **`Cargo.lock` should be committed** — this is a binary (application), not a library. Add `Cargo.lock` to `.gitignore` only for libraries.

### action.yml Stub

Minimal valid composite action YAML. Story 5.4 will fill in the full implementation:

```yaml
name: 'vibestats'
description: 'Aggregate Claude Code session activity and update your GitHub profile heatmap'
runs:
  using: 'composite'
  steps: []
```

### install.sh Stub

Must be executable (`chmod +x install.sh`). Minimal stub:

```bash
#!/usr/bin/env bash
# vibestats installer — implementation in Epic 6
set -euo pipefail
echo "TODO: implement installer" && exit 1
```

### Python Stub Pattern

Each Python stub should be importable and runnable, with a module docstring explaining purpose:

```python
"""aggregate.py — Aggregates per-machine Hive partition files into daily totals.

Implementation: Epic 5, Story 5.1.
"""

if __name__ == "__main__":
    pass
```

### README.md Content Requirements

The README must include these elements (even as placeholders):
1. Install command: `curl -fsSL https://vibestats.dev/install.sh | bash`
2. Link to `vibestats.dev`
3. Brief one-line description: "Track your Claude Code session activity and display a GitHub contributions-style heatmap on your profile."
4. Placeholder for heatmap screenshot/SVG (use alt text: `<!-- vibestats heatmap screenshot placeholder -->`)
5. `<!-- vibestats-start -->` / `<!-- vibestats-end -->` markers (required by FR9, FR24 — installer adds SVG between these)

### .gitignore Requirements

Must cover all four component build systems:

```gitignore
# Rust
/target/

# Node / Astro
node_modules/
site/dist/
site/.astro/

# macOS
.DS_Store

# Editor
.idea/
.vscode/
*.swp
*.swo
```

**Note:** Do NOT ignore `Cargo.lock` — binary projects commit the lockfile.

### Project Structure Notes

- This story covers the repo root and directory scaffold only.
- `src/main.rs` and Rust crate structure → Story 1.2
- `site/` Astro initialization → Story 1.3
- `docs/schemas.md` → Story 1.4
- `.github/workflows/*.yml` workflow files → Epics 3, 5, 8

### No Tests Required

This story creates only directory structure and stub files — no executable logic to test. Verify correctness via `git status` and `ls` checks in the task checklist.

### References

- Monorepo layout: [Source: architecture.md#Complete Project Directory Structure]
- File naming conventions: [Source: architecture.md#Naming Patterns]
- FR9, FR24 (README markers): [Source: epics.md#Functional Requirements]
- Story 1.1 ACs: [Source: epics.md#Story 1.1: Initialize monorepo directory structure]
- Cargo.toml initialization: [Source: architecture.md#Selected Starters by Component - Rust Sync Binary]
- action.yml definition: [Source: architecture.md#Community GitHub Action]

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

None — implementation was straightforward with no errors.

### Completion Notes List

All tasks completed on 2026-04-11. The monorepo directory structure was created with all required stub files. All acceptance criteria verified:
- AC1: `src/`, `action/`, `site/`, `install.sh`, `action.yml`, `Cargo.toml`, `.github/workflows/` all exist at correct paths.
- AC2: `action/aggregate.py`, `action/generate_svg.py`, `action/update_readme.py`, `action/tests/` with `__init__.py` and fixture subdirectories all exist.
- AC3: `README.md` (with install command placeholder and vibestats.dev link + vibestats-start/end markers), `CONTRIBUTING.md`, `LICENSE` (MIT 2026 stephenleo), `.gitignore` (Rust, Node/Astro, macOS, editor) all exist.
- No tests required for this story (directory structure only). Structure integrity verified via git status.

### File List

- `.github/workflows/.gitkeep`
- `.gitignore`
- `action.yml`
- `action/aggregate.py`
- `action/generate_svg.py`
- `action/update_readme.py`
- `action/tests/__init__.py`
- `action/tests/fixtures/expected_output/.gitkeep`
- `action/tests/fixtures/sample_machine_data/.gitkeep`
- `Cargo.toml`
- `CONTRIBUTING.md`
- `install.sh`
- `LICENSE`
- `README.md`
- `site/.gitkeep`
- `src/.gitkeep`
- `_bmad-output/implementation-artifacts/1-1-initialize-monorepo-directory-structure.md`
