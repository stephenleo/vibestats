# Story 7.4: Build landing page

Status: ready-for-dev

<!-- GH Issue: #38 | Epic: #7 | PR must include: Closes #38 -->

## Story

As a developer discovering vibestats,
I want a compelling landing page at `vibestats.dev`,
so that I understand what it does and how to install it within 30 seconds.

## Acceptance Criteria

1. **Given** a visitor opens `vibestats.dev` **When** the page loads **Then** it shows: (1) the one-line install command in a copyable code block, (2) an example heatmap SVG, (3) a three-bullet "why vibestats" section (zero effort, cross-machine, GitHub profile)

2. **Given** the install command is displayed **When** the visitor copies it **Then** it reads exactly: `curl -sSf https://vibestats.dev/install.sh | bash`

3. **Given** the landing page is built **When** `npm run build` runs **Then** the page passes Astro's static build without errors

## Tasks / Subtasks

- [ ] Task 1: Replace the stub `site/src/pages/index.astro` with the landing page (AC: #1, #2, #3)
  - [ ] Import `Base` from `'../layouts/Base.astro'` (not the `layout:` frontmatter shortcut — that's for `.md` files only)
  - [ ] Set page title prop to `"vibestats"` (`<Base title="vibestats">`)
  - [ ] Section 1 — Hero: headline "Track your Claude Code sessions" (or similar one-liner), sub-headline explaining the GitHub-profile heatmap angle
  - [ ] Section 2 — Install command: display `curl -sSf https://vibestats.dev/install.sh | bash` inside a `<code>` / `<pre>` block; use a copy-to-clipboard `<button>` with `navigator.clipboard.writeText()` via an inline `<script>` tag (no external JS library)
  - [ ] Section 3 — Example heatmap: embed the static `heatmap.svg` from `action/tests/fixtures/expected_output/heatmap.svg` by copying it to `site/public/heatmap-example.svg` then referencing it as `<img src="/heatmap-example.svg" alt="Example vibestats heatmap" />`
  - [ ] Section 4 — Three-bullet "why vibestats": (1) **Zero effort** — hooks fire silently on every session; (2) **Cross-machine** — all your machines, one repo; (3) **GitHub profile** — heatmap embeds in your README and links to your dashboard
  - [ ] Add `<style>` block (scoped in Astro) — minimal styling only; no Tailwind, no external CSS framework
  - [ ] TypeScript strict: index.astro has no props interface needed (no `Astro.props` used); frontmatter only needs the import

- [ ] Task 2: Copy example SVG to public assets (AC: #1)
  - [ ] Copy `action/tests/fixtures/expected_output/heatmap.svg` → `site/public/heatmap-example.svg`
  - [ ] Confirm the SVG renders correctly (it is an `<svg>` element — referencing it via `<img>` works fine for SSG)

- [ ] Task 3: Verify build passes (AC: #3)
  - [ ] Run `npm run build` from `site/` — must complete with 0 errors
  - [ ] Run `npm run check` from `site/` — must report 0 TypeScript/Astro errors
  - [ ] Confirm `dist/index.html` is generated

## Dev Notes

### Working Directory

All file operations and `npm` commands run from `site/`. The Astro project root is `site/` — NOT the repo root.

### What Already Exists — Do NOT Recreate

Story 7.1 built these — extend, don't recreate:

```
site/src/layouts/Base.astro       ← base HTML wrapper: head, Header, slot, Footer
site/src/layouts/Docs.astro       ← extends Base.astro: sidebar nav + slot
site/src/components/Header.astro  ← vibestats logo + nav (Home, Docs, GitHub)
site/src/components/Footer.astro  ← GitHub link + MIT License
site/src/pages/index.astro        ← STUB — replace contents, keep file
site/public/favicon.svg
site/public/favicon.ico
site/public/_redirects
```

The `index.astro` stub currently contains only a `<h1>` and two `<p>` tags. Replace its entire body content while keeping the `Base.astro` import pattern.

### Astro Layout Pattern — Correct Syntax

Use component import (NOT the `layout:` frontmatter property — that's for Markdown files only):

```astro
---
import Base from '../layouts/Base.astro';
---
<Base title="vibestats">
  <!-- page content here -->
</Base>
```

### Copy-to-Clipboard Pattern (Astro)

Use an inline `<script>` tag inside the `.astro` file — no separate `.js` file needed:

```astro
<button id="copy-btn">Copy</button>
<script>
  const btn = document.getElementById('copy-btn') as HTMLButtonElement;
  btn.addEventListener('click', () => {
    navigator.clipboard.writeText('curl -sSf https://vibestats.dev/install.sh | bash');
    btn.textContent = 'Copied!';
    setTimeout(() => { btn.textContent = 'Copy'; }, 2000);
  });
</script>
```

Astro bundles inline `<script>` tags — TypeScript supported, no special config needed.

### TypeScript Strict Mode

`index.astro` does **not** take props, so no `interface Props` is required. Just import and use `Base.astro`.

### Exact Install Command

The install command content must be exactly:
```
curl -sSf https://vibestats.dev/install.sh | bash
```
(per AC #2 — any deviation is a blocker)

### Example SVG Source

The fixture SVG lives at `action/tests/fixtures/expected_output/heatmap.svg` (relative to repo root). Copy it to `site/public/heatmap-example.svg` so Astro serves it as a static asset. Do NOT reference the original path — Astro only serves files from `site/public/` and `site/src/`.

### Three "Why vibestats" Bullets — Required Content

The three bullets must convey these exact concepts (wording can vary slightly):
1. **Zero effort** — hooks fire silently on every session (no manual commands)
2. **Cross-machine** — data from all your machines in one repository
3. **GitHub profile** — heatmap embeds in profile README; links to interactive dashboard

### Style Guidance

- Use scoped `<style>` in the Astro component (Astro auto-scopes component styles)
- Keep styling minimal — this is MVP; no Tailwind, no external CSS
- A simple centered layout with max-width ~800px is sufficient
- The copy button just needs to look clickable — basic border/padding is fine

### Architecture Reference

- `site/src/pages/index.astro` is the SSG entry point (`src/pages/index.astro — docs/landing (SSG)`)
- Architecture file lists this as FR43 coverage: "public documentation and dashboard site at vibestats.dev"
- No server-side rendering — pure SSG; no API calls needed on this page
- `astro.config.mjs` has `trailingSlash: 'never'` and `compressHTML: false` — do not change

### Previous Story Patterns (Story 7.1 Learnings)

- `npm run check` must pass — use TypeScript strict mode throughout
- Astro components: TypeScript interface is only required when `Astro.props` is destructured
- Build confirmed working: `npm run build` from `site/` directory
- `npm run check` = `astro check` (defined in `site/package.json`)

### Dependency Clarity

Story 7.3 (docs pages) and 7.2 (dashboard) are parallel stories — do NOT block on them and do NOT create their files. This story only modifies `site/src/pages/index.astro` and adds `site/public/heatmap-example.svg`.

### Project Structure Notes

- Target file to modify: `site/src/pages/index.astro` (already exists as stub)
- New static asset: `site/public/heatmap-example.svg` (copy from `action/tests/fixtures/expected_output/heatmap.svg`)
- No new layouts, components, or routes needed
- No changes to `astro.config.mjs`, `package.json`, `tsconfig.json`, or `_redirects`

### References

- Epic 7 story 7.4 definition [Source: _bmad-output/planning-artifacts/epics.md#Story-7.4]
- Astro routing architecture [Source: _bmad-output/planning-artifacts/architecture.md — Astro routing]
- FR43: landing page requirement [Source: _bmad-output/planning-artifacts/epics.md#Functional-Requirements]
- Base layout pattern [Source: _bmad-output/implementation-artifacts/7-1-build-base-layouts-and-shared-astro-components.md#Astro-Layout-Pattern]
- Install command format [Source: _bmad-output/planning-artifacts/epics.md#Story-7.4 AC #2]
- Example SVG location [Source: action/tests/fixtures/expected_output/heatmap.svg]

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

### Completion Notes List

### File List
