# Story 1.1: Initialize Monorepo Directory Structure

Status: done

## Story

As a developer,
I want the vibestats monorepo initialized with the correct directory structure,
so that all components have a defined home and the project is ready for implementation.

## Acceptance Criteria

1. **Given** the repo is empty **When** the monorepo structure is created **Then** the following paths exist at the correct locations:
   - `src/` directory (Rust binary home)
   - `action/` directory (Python GitHub Actions scripts home)
   - `site/` directory (Astro dashboard/docs site home)
   - `install.sh` (Bash installer placeholder)
   - `action.yml` (Community GitHub Action definition placeholder)
   - `Cargo.toml` (Rust workspace manifest)
   - `.github/workflows/` directory

2. **Given** the `action/` directory is created **When** its contents are reviewed **Then** the following stub files/dirs exist:
   - `action/aggregate.py` (stub — empty or `# TODO`)
   - `action/generate_svg.py` (stub)
   - `action/update_readme.py` (stub)
   - `action/tests/` directory (empty, with `.gitkeep`)

3. **Given** the `site/` directory is created **When** its contents are reviewed **Then** it contains:
   - `site/src/` directory
   - `site/public/` directory
   - `site/package.json` (placeholder JSON, no real npm install yet)

4. **Given** the root files are reviewed **Then** all of the following exist:
   - `README.md` — includes install command placeholder and `vibestats.dev` link
   - `CONTRIBUTING.md` — basic contribution guide
   - `LICENSE` — MIT license (copyright year 2026, author stephenleo)
   - `.gitignore` — covers Rust build artifacts (`/target`), node_modules, site build output (`site/dist`), and macOS `.DS_Store`

5. **Given** `Cargo.toml` is reviewed **Then** it defines a valid Rust workspace with `members = ["src"]` and the edition set to `2021`; note: full crate dependencies are added in Story 1.2.

6. **Given** the `.github/workflows/` directory exists **Then** it contains at minimum placeholder files for `release.yml`, `deploy-site.yml`, and `aggregate.yml` (stubs with minimal valid YAML).

## Tasks / Subtasks

- [x] Create top-level directory stubs (AC: #1)
  - [x] Create `src/.gitkeep` (marks Rust source home)
  - [x] Create `action/` directory with stub Python files
  - [x] Create `site/src/` and `site/public/` directories
  - [x] Create `.github/workflows/` directory

- [x] Create Python action stub files (AC: #2)
  - [x] `action/aggregate.py` — single-line `# TODO: implement aggregation`
  - [x] `action/generate_svg.py` — single-line `# TODO: implement SVG generation`
  - [x] `action/update_readme.py` — single-line `# TODO: implement README update`
  - [x] `action/tests/.gitkeep` — empty file to track dir in git

- [x] Create `Cargo.toml` workspace manifest (AC: #5)
  - [x] Set `[workspace]` with `members = ["src"]` and `resolver = "2"`
  - [x] Set `[package]` edition to `2021` (do NOT add crate deps — that is Story 1.2)

- [x] Create root documentation files (AC: #4)
  - [x] `README.md` — include one-line install command placeholder: `curl -fsSL https://vibestats.dev/install.sh | sh` and link to `https://vibestats.dev`
  - [x] `CONTRIBUTING.md` — minimal guide: how to run locally, PR process
  - [x] `LICENSE` — MIT, year 2026, copyright stephenleo

- [x] Create `.gitignore` (AC: #4)
  - [x] Add `/target` (Rust build artifacts)
  - [x] Add `node_modules/`
  - [x] Add `site/dist/` (Astro build output)
  - [x] Add `.DS_Store`

- [x] Create placeholder workflow YAML files (AC: #6)
  - [x] `.github/workflows/release.yml` — minimal stub (name + `on: push` or comment)
  - [x] `.github/workflows/deploy-site.yml` — minimal stub
  - [x] `.github/workflows/aggregate.yml` — minimal stub

- [x] Create placeholder root files (AC: #1)
  - [x] `install.sh` — `#!/usr/bin/env bash` + `# TODO: implement installer` + `set -euo pipefail`
  - [x] `action.yml` — minimal valid composite action YAML stub

- [x] Create `site/package.json` placeholder (AC: #3)
  - [x] Minimal valid JSON with `"name": "vibestats-site"`, `"version": "0.0.1"`, `"private": true`

- [x] Commit all files to the story branch

### Review Findings

- [x] [Review][Patch] Cargo.toml missing edition setting per AC #5 [Cargo.toml:1] — Added `[workspace.package]` with `edition = "2021"` to satisfy AC #5 requirement while remaining compatible with Dev Notes guidance (no crate dependencies). Downstream Story 1.2 crate can use `edition.workspace = true`.
- [x] [Review][Patch] site/src and site/public not tracked in git per AC #3 [site/src/, site/public/] — Added `.gitkeep` to both directories so they are preserved in version control, matching the pattern used by `src/.gitkeep` and `action/tests/.gitkeep`. Without this, downstream stories merging into main would not see the directories.
- [x] [Review][Patch] Dev Agent Record sections were empty after implementation [_bmad-output/implementation-artifacts/1-1-initialize-monorepo-directory-structure.md] — Populated File List, Completion Notes, and checked off all completed tasks.
- [x] [Review][Patch] Story status remained `ready-for-dev` after implementation — Advanced to `done` now that review findings are resolved.

## Dev Notes

### Project Structure Requirements

The final monorepo layout MUST match exactly (from architecture.md §Structure Patterns):

```
vibestats/
├── README.md
├── CONTRIBUTING.md
├── LICENSE
├── action.yml                  ← Community GitHub Action definition
├── install.sh                  ← Bash installer
├── Cargo.toml
├── Cargo.lock                  ← generated by cargo; include in git
├── .gitignore
├── src/                        ← Rust binary
├── action/                     ← Python Actions scripts
│   ├── aggregate.py
│   ├── generate_svg.py
│   ├── update_readme.py
│   └── tests/
├── site/                       ← Astro site
│   ├── src/
│   ├── public/
│   └── package.json
└── .github/
    └── workflows/
        ├── release.yml
        ├── deploy-site.yml
        └── aggregate.yml
```

**Critical:** This story only creates the skeleton. It does NOT:
- Run `cargo new` (that is Story 1.2 which adds the actual Rust `src/main.rs`, crate manifest with deps)
- Run `npm create astro@latest` (that is Story 1.3)
- Add any real crate dependencies to `Cargo.toml` (Story 1.2)
- Write any real Python logic (later stories in Epics 3/5)

### File-Specific Notes

**`Cargo.toml`:** Create as a workspace manifest, not a crate manifest. Do not add `[dependencies]` — those belong in a separate `src/Cargo.toml` (Story 1.2). The root `Cargo.toml` is the workspace root:
```toml
[workspace]
members = ["src"]
resolver = "2"
```

**`install.sh`:** Must start with `#!/usr/bin/env bash` and include `set -euo pipefail`. Stub content is fine for this story.

**`action.yml`:** Minimum valid composite action:
```yaml
name: vibestats
description: 'Generate Claude Code activity heatmap'
runs:
  using: composite
  steps: []
```

**`.gitignore`:** The repo root already has a `.gitignore` (from initial commit `70f3a8e update gitignore`). Review it first and ADD missing entries rather than overwriting. Check what's already there before editing.

**Workflow YAML stubs:** Must be valid YAML. Use minimal structure:
```yaml
name: <workflow-name>
on:
  workflow_dispatch:
jobs: {}
```

### Naming Conventions (from architecture.md)

- Rust source files: `snake_case` (e.g., `github_api.rs`)
- Python scripts: `snake_case` (e.g., `aggregate.py`)
- Shell scripts: `kebab-case` (e.g., `install.sh`)
- Astro/JS files: `kebab-case` files, `camelCase` vars

### Architecture Compliance

- This story establishes the canonical directory layout that ALL downstream epics depend on.
- Do NOT create any files outside the layout shown above.
- The `src/` directory will be managed by `cargo new` in Story 1.2 — for this story, only a `.gitkeep` or a minimal placeholder `src/main.rs` is acceptable.
- Story 1.3 initializes `site/` with Astro — do not add Astro-specific content here.

### Story Dependencies

- This is the first story in Epic 1 and the entire project — no prior stories to reference.
- Stories 1.2, 1.3, and 1.4 all depend on this story completing first.
- Epic 2 (Rust Foundation), Epic 5 (Actions Pipeline), and Epic 7 (Astro Site) all assume this directory layout exists.

### GitHub Issue

Story GitHub Issue: [stephenleo/vibestats#9](https://github.com/stephenleo/vibestats/issues/9)
Epic GitHub Issue: [stephenleo/vibestats#1](https://github.com/stephenleo/vibestats/issues/1)

Every PR for this story must include `Closes #9` in the PR description.

### References

- [Source: _bmad-output/planning-artifacts/epics.md — Epic 1, Story 1.1]
- [Source: _bmad-output/planning-artifacts/architecture.md — §Structure Patterns, §Complete Project Directory Structure]
- [Source: _bmad-output/planning-artifacts/architecture.md — §Starter Templates (Rust: `cargo new vibestats --bin`; Astro: `npm create astro@latest`)]

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

- None — no blockers, all work completed in a single pass.

### Completion Notes List

- All acceptance criteria satisfied. Monorepo skeleton created per architecture.md §Structure Patterns.
- `.gitignore` was already present from the initial commit; tightened existing entries (`target/` → `/target`) and added `site/dist/` rather than overwriting.
- `Cargo.toml` includes `[workspace.package] edition = "2021"` so downstream crates (Story 1.2) can inherit via `edition.workspace = true`. No crate dependencies added — deferred to Story 1.2.
- `src/`, `site/src/`, `site/public/`, and `action/tests/` use `.gitkeep` sentinels to keep the empty directories tracked in git.
- Workflow YAML files are minimal valid stubs (`jobs: {}`). They will be fleshed out in later epics.
- Code review pass: added missing `edition` in `Cargo.toml` and `.gitkeep` files under `site/src/` and `site/public/` to satisfy ACs #5 and #3.

### File List

Created:
- `.github/workflows/aggregate.yml`
- `.github/workflows/deploy-site.yml`
- `.github/workflows/release.yml`
- `CONTRIBUTING.md`
- `Cargo.toml`
- `LICENSE`
- `README.md`
- `action.yml`
- `action/aggregate.py`
- `action/generate_svg.py`
- `action/update_readme.py`
- `action/tests/.gitkeep`
- `install.sh`
- `site/package.json`
- `site/public/.gitkeep`
- `site/src/.gitkeep`
- `src/.gitkeep`

Modified:
- `.gitignore` (tightened `target/` → `/target`; added `site/dist/`)
