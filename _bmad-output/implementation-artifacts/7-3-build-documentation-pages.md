# Story 7.3: Build documentation pages

Status: done

<!-- GH Issue: #37 | Epic: #7 | PR must include: Closes #37 -->

## Story

As a potential vibestats user,
I want clear documentation covering quickstart, architecture, CLI reference, and troubleshooting,
So that I can install and use vibestats without reading the source code.

## Acceptance Criteria

1. **Given** the docs site is built **When** the quickstart page is viewed **Then** it shows the install command (`curl -sSf https://vibestats.dev/install.sh | bash`), lists the 5-minute install steps, and links to the CLI reference (FR43)

2. **Given** the docs site is built **When** the "How it works" page is viewed **Then** it includes an architecture diagram showing the data flow: JSONL → vibestats-data → GitHub Action → profile README → vibestats.dev

3. **Given** the docs site is built **When** the CLI reference page is viewed **Then** every subcommand (`status`, `sync`, `sync --backfill`, `machines list`, `machines remove`, `auth`, `uninstall`) is documented with description, flags, and example output

4. **Given** the docs site is built **When** the troubleshooting page is viewed **Then** it covers: token expiry fix, hook not firing, missing machine data, and how to trigger a manual backfill

5. **Given** all four doc pages are built **When** `npm run build` runs inside `site/` **Then** the build completes without TypeScript or Astro errors

## Tasks / Subtasks

- [x] Task 1: Create `site/src/pages/docs/quickstart.astro` (AC: #1, #5)
  - [x] Use `Docs.astro` layout (`import Docs from '../../layouts/Docs.astro'`)
  - [x] Pass `title="Quickstart — vibestats"` to the layout
  - [x] Show the one-line install command in a `<pre><code>` block: `curl -sSf https://vibestats.dev/install.sh | bash`
  - [x] List the 5-minute install steps (installer creates `vibestats-data` repo, writes Actions workflow, stores OAuth token, configures `~/.claude/settings.json` hooks, adds README markers, triggers backfill)
  - [x] Include a link to `/docs/cli-reference`
  - [x] TypeScript strict — no props needed (static page, no `interface Props` required)

- [x] Task 2: Create `site/src/pages/docs/how-it-works.astro` (AC: #2, #5)
  - [x] Use `Docs.astro` layout
  - [x] Pass `title="How it works — vibestats"`
  - [x] Include an ASCII or HTML text-based architecture diagram showing the data flow:
    - JSONL files (`~/.claude/projects/**/*.jsonl`) → Rust binary → `vibestats-data` (private GitHub repo)
    - `vibestats-data` → GitHub Action cron (daily) → `heatmap.svg` + `data.json`
    - `data.json` → `username/username` (profile README via `<!-- vibestats-start/end -->` markers)
    - `data.json` → `vibestats.dev/[username]` (client-side dashboard)
  - [x] Keep the diagram simple and readable — text-based using `<pre>` or HTML is acceptable for MVP

- [x] Task 3: Create `site/src/pages/docs/cli-reference.astro` (AC: #3, #5)
  - [x] Use `Docs.astro` layout
  - [x] Pass `title="CLI Reference — vibestats"`
  - [x] Document all 7 subcommands with description, flags, and example output:
    - `vibestats status` — shows sync health, last sync time, auth status, machine list
    - `vibestats sync` — forces an immediate sync of JSONL data to vibestats-data
    - `vibestats sync --backfill` — triggers full historical JSONL backfill
    - `vibestats machines list` — lists all registered machines with hostname and last-seen
    - `vibestats machines remove <id>` — removes a machine from registry and purges its remote data
    - `vibestats auth` — re-authenticates the GitHub OAuth token
    - `vibestats uninstall` — removes hooks from `~/.claude/settings.json`, optionally removes config

- [x] Task 4: Create `site/src/pages/docs/troubleshooting.astro` (AC: #4, #5)
  - [x] Use `Docs.astro` layout
  - [x] Pass `title="Troubleshooting — vibestats"`
  - [x] Cover all four troubleshooting scenarios:
    - **Token expiry fix** — run `vibestats auth` to re-authenticate; token stored at `~/.config/vibestats/config.toml`
    - **Hook not firing** — verify `~/.claude/settings.json` has Stop and SessionStart hooks; re-run installer or manually add hooks
    - **Missing machine data** — check `vibestats status`; machine may not have synced yet; run `vibestats sync` to force
    - **How to trigger manual backfill** — run `vibestats sync --backfill`

- [x] Task 5: Verify `npm run build` and TypeScript check pass (AC: #5)
  - [x] Run `npm run build` inside `site/` — must complete with 0 errors
  - [x] Run `npm run check` inside `site/` — must report 0 TypeScript/Astro errors
  - [x] Confirm output includes the 4 new doc pages: `/docs/quickstart/index.html` etc. (or non-trailing-slash equivalents per `trailingSlash: 'never'` config)

## Dev Notes

### Working Directory

All file creation and `npm` commands run from within `site/`. The Astro project root is `site/` — not the repo root.

### What Already Exists (from Stories 1.3 and 7.1)

Do NOT recreate or overwrite these files:

```
site/
  astro.config.mjs         ← site: 'https://vibestats.app', trailingSlash: 'never', compressHTML: false
  package.json             ← astro ^6.1.5, cal-heatmap 4.2.4, @astrojs/check, typescript
  tsconfig.json            ← strict TypeScript
  .prettierrc              ← 2-space indentation, Astro parser
  src/layouts/Base.astro   ← base HTML wrapper (head + Header + slot + Footer)
  src/layouts/Docs.astro   ← two-column layout (sidebar nav + main slot), wraps Base.astro
  src/components/Header.astro  ← vibestats logo/wordmark + nav (Home, Docs, GitHub)
  src/components/Footer.astro  ← GitHub link + MIT License
  src/pages/index.astro        ← uses Base.astro layout
  src/pages/u/index.astro      ← uses Base.astro layout
  public/_redirects            ← Cloudflare Pages rewrite rules — DO NOT MODIFY
```

`site/src/pages/docs/` does NOT exist yet — create the directory and all 4 pages inside it.

### Target File Structure After This Story

```
site/src/pages/docs/
  quickstart.astro
  how-it-works.astro
  cli-reference.astro
  troubleshooting.astro
```

**Do NOT use a dynamic `[...slug].astro` approach.** The architecture routing table mentions `[...slug].astro` as shorthand, but the definitive project directory tree explicitly lists 4 individual static files. The sidebar in `Docs.astro` already has hardcoded links — individual static pages is the correct MVP approach.

### Docs.astro Layout — How to Use It

The `Docs.astro` layout already exists with a sidebar that links to all 4 doc pages. Use it in every doc page:

```astro
---
import Docs from '../../layouts/Docs.astro';
---
<Docs title="Quickstart — vibestats">
  <h1>Quickstart</h1>
  <!-- page content here -->
</Docs>
```

The import path from `site/src/pages/docs/` to `site/src/layouts/Docs.astro` is `../../layouts/Docs.astro`.

Do NOT use Base.astro directly — use Docs.astro, which wraps Base.astro with the sidebar navigation.

### Docs.astro Sidebar (Already Implemented — Do Not Change)

The sidebar in `Docs.astro` already contains the correct links:
- `/docs/quickstart`
- `/docs/how-it-works`
- `/docs/cli-reference`
- `/docs/troubleshooting`

These links must match the file-based routes that Astro generates from the filenames you create. With `trailingSlash: 'never'` in `astro.config.mjs`, the routes will be `/docs/quickstart`, `/docs/how-it-works`, `/docs/cli-reference`, `/docs/troubleshooting` — matching exactly.

### TypeScript Strict Mode

Pages that accept NO props (all 4 doc pages in this story are fully static) do NOT need an `interface Props` declaration. Their frontmatter can be minimal:

```astro
---
import Docs from '../../layouts/Docs.astro';
---
```

The `Docs.astro` layout accepts `title: string` as a prop — pass it via the JSX-style syntax `<Docs title="...">`.

### No CSS Framework

Architecture does not mandate a CSS framework. Use minimal inline `<style>` blocks if needed. Keep styling simple — the Docs.astro layout already handles two-column grid with responsive breakpoint. Focus on correct HTML structure and content.

### Indentation and Formatting

- 2-space indentation (enforced by `.prettierrc`)
- Follow patterns established in `Base.astro`, `Docs.astro`, `Header.astro`, `Footer.astro`

### Architecture Diagram Guidance (how-it-works.astro)

A text-based diagram in a `<pre>` block is fully acceptable for MVP. Example structure (adapt as needed):

```
Claude Code sessions
        │
        ▼ Stop/SessionStart hooks
  Rust binary (vibestats)
        │ reads ~/.claude/projects/**/*.jsonl
        │ pushes per-machine daily JSON
        ▼
  vibestats-data (private repo)
  └── machines/year=.../day=.../machine_id=.../data.json
        │
        ▼ daily GitHub Actions cron
  stephenleo/vibestats@v1
  (aggregate.py + generate_svg.py + update_readme.py)
        │
        ├──► username/username/vibestats/heatmap.svg
        │     └──► profile README (<!-- vibestats-start/end -->)
        │
        └──► username/username/vibestats/data.json
              └──► vibestats.dev/[username] (client-side dashboard)
```

### Install Command (Exact)

The exact install command for the quickstart page is:
```
curl -sSf https://vibestats.dev/install.sh | bash
```

This is defined by FR43 / prd.md and must be reproduced exactly.

### CLI Reference Content — Exact Subcommands

Document these exact subcommands (from the Rust binary implemented in Epics 3 and 4):

| Command | Description |
|---|---|
| `vibestats status` | Shows sync health, last sync time, auth token status, and registered machine list |
| `vibestats sync` | Forces an immediate sync from JSONL to vibestats-data (respects 5-min throttle) |
| `vibestats sync --backfill` | Walks all historical JSONL files and pushes complete history to vibestats-data |
| `vibestats machines list` | Lists all machines registered in `vibestats-data/registry.json` |
| `vibestats machines remove <id>` | Removes a machine from registry and purges its remote data files |
| `vibestats auth` | Re-authenticates the GitHub OAuth token via `gh auth login` |
| `vibestats uninstall` | Removes hooks from `~/.claude/settings.json`, optionally removes config/binary |

### Troubleshooting Content — Exact Scenarios

Cover these four scenarios verbatim from the acceptance criteria:

1. **Token expiry fix** — `vibestats auth` re-authenticates. Config stored at `~/.config/vibestats/config.toml`.
2. **Hook not firing** — Verify `~/.claude/settings.json` has `Stop` hook (`vibestats sync`, `async: true`) and `SessionStart` hook (`vibestats sync`). Re-run installer or add manually.
3. **Missing machine data** — Run `vibestats status` to inspect. Run `vibestats sync` to push immediately.
4. **Manual backfill** — Run `vibestats sync --backfill` to push all historical JSONL data.

### Build Verification

Run from inside `site/`:
```bash
npm run build && npm run check
```

Expected: build succeeds (6 pages minimum: index, u/index, docs/quickstart, docs/how-it-works, docs/cli-reference, docs/troubleshooting), check reports 0 errors.

### No Tests

Architecture spec: "Astro: no tests at MVP" (`architecture.md#Test placement`). No test files required for this story.

### Project Structure Notes

- All new files in this story live under `site/src/pages/docs/`
- Do NOT touch `src/` (Rust), `action/` (Python), `Cargo.toml`, `.github/`
- Do NOT modify `site/public/_redirects`
- Do NOT modify `site/astro.config.mjs` or `site/tsconfig.json`
- Do NOT modify `site/src/layouts/Docs.astro` — the sidebar already links to all 4 pages
- Do NOT create `site/src/pages/[username].astro` — that is Story 7.2's scope

### Previous Story Intelligence (from 7.1)

Key learnings from Story 7.1 that apply directly here:

1. **`@astrojs/check` and `typescript` are already in `devDependencies`** — no new packages needed for this story
2. **Import path depth matters** — from `site/src/pages/docs/*.astro`, layouts are at `../../layouts/`, components at `../../components/`
3. **`npm run check` is the TypeScript checker** — always run before marking done
4. **`npm run build` + `npm run check` both run from inside `site/`**, not from repo root
5. **`compressHTML: false`** in astro.config.mjs — build output is readable HTML (not minified)
6. **`trailingSlash: 'never'`** — routes are `/docs/quickstart` not `/docs/quickstart/`
7. **Docs.astro wraps Base.astro** — never duplicate the `<html>` shell in pages; always use Docs.astro for doc pages

### References

- File locations: [Source: architecture.md#Complete Project Directory Structure — `site/src/pages/docs/`]
- Routing: [Source: architecture.md#Astro routing — `src/pages/docs/[...slug].astro` pattern]
- Naming convention: [Source: architecture.md#Naming Patterns — "Astro/JS: kebab-case files"]
- FR43 docs requirements: [Source: prd.md#FR43 — quickstart, CLI reference, architecture, troubleshooting]
- Docs minimum content: [Source: prd.md#Docs site minimum content at launch]
- No Astro tests at MVP: [Source: architecture.md#Test placement]
- TypeScript strict: [Source: architecture.md#Selected Starters by Component — Astro: `--typescript strict`]
- Install command: [Source: prd.md#Installation Methods — `curl -sSf https://vibestats.dev/install.sh | bash`]
- Story requirements and AC: [Source: epics.md#Story 7.3: Build documentation pages]
- Layouts and components: [Source: 7-1-build-base-layouts-and-shared-astro-components.md]

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

None.

### Completion Notes List

- Created 4 static Astro documentation pages under `site/src/pages/docs/`
- `quickstart.astro`: includes exact install command `curl -sSf https://vibestats.dev/install.sh | bash`, 6-step install list, and link to `/docs/cli-reference`
- `how-it-works.astro`: includes text-based ASCII architecture diagram in `<pre>` block showing full data flow from JSONL to vibestats.dev dashboard
- `cli-reference.astro`: documents all 7 subcommands (`status`, `sync`, `sync --backfill`, `machines list`, `machines remove`, `auth`, `uninstall`) with descriptions, flags tables, and example output
- `troubleshooting.astro`: covers all 4 scenarios (token expiry, hook not firing, missing machine data, manual backfill)
- All pages use `Docs.astro` layout with correct import path `../../layouts/Docs.astro`
- `npm run build` passed: 6 pages built with 0 errors (index, u/index, docs/quickstart, docs/how-it-works, docs/cli-reference, docs/troubleshooting)
- `npm run check` passed: 11 files checked, 0 errors, 0 warnings, 0 hints
- No new dependencies required; no existing files modified

### File List

- `site/src/pages/docs/quickstart.astro` (new)
- `site/src/pages/docs/how-it-works.astro` (new)
- `site/src/pages/docs/cli-reference.astro` (new)
- `site/src/pages/docs/troubleshooting.astro` (new)

### Change Log

- 2026-04-12: Implemented story 7.3 — created 4 documentation pages (quickstart, how-it-works, cli-reference, troubleshooting) under `site/src/pages/docs/`. Build and TypeScript checks pass with 0 errors.
