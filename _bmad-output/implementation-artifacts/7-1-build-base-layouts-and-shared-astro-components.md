# Story 7.1: Build base layouts and shared Astro components

Status: ready-for-dev

<!-- GH Issue: #35 | Epic: #7 | PR must include: Closes #35 -->

## Story

As a developer,
I want base Astro layouts and shared components built,
so that all pages share consistent structure without duplicating markup.

## Acceptance Criteria

1. **Given** the Astro project is initialized (Story 1.3) **When** `Base.astro` and `Docs.astro` layouts are built **Then** they define a consistent `<head>`, `Header.astro` (vibestats logo + nav), and `Footer.astro` (GitHub link, license)

2. **Given** a page uses `Docs.astro` layout **When** it renders **Then** it includes sidebar navigation linking to all docs pages

3. **Given** the shared components are built **When** `npm run build` runs **Then** the build completes without TypeScript or Astro errors

## Tasks / Subtasks

- [ ] Task 1: Create `site/src/layouts/Base.astro` (AC: #1, #3)
  - [ ] Create `site/src/layouts/` directory
  - [ ] Implement `Base.astro` with a complete `<head>` section: charset, viewport, favicon links (already in `site/public/`), `<title>{title}</title>` prop, generator meta tag
  - [ ] Import and render `Header.astro` and `Footer.astro` components (created in Task 3)
  - [ ] Accept `title: string` as a prop via the frontmatter (`const { title } = Astro.props`)
  - [ ] Use `<slot />` for page body content
  - [ ] TypeScript strict — define an `interface Props { title: string }` in frontmatter

- [ ] Task 2: Create `site/src/layouts/Docs.astro` (AC: #1, #2, #3)
  - [ ] Extend `Base.astro` — import and use it as the outer layout wrapper
  - [ ] Accept `title: string` and pass it through to `Base.astro`
  - [ ] Implement a two-column layout: sidebar + main content area
  - [ ] Sidebar must link to all four docs pages (use relative or root-relative `/docs/` paths):
    - Quickstart → `/docs/quickstart`
    - How it works → `/docs/how-it-works`
    - CLI reference → `/docs/cli-reference`
    - Troubleshooting → `/docs/troubleshooting`
  - [ ] Use `<slot />` for the main docs content
  - [ ] TypeScript strict — define `interface Props { title: string }` in frontmatter

- [ ] Task 3: Create `Header.astro` and `Footer.astro` shared components (AC: #1, #3)
  - [ ] Create `site/src/components/Header.astro`:
    - vibestats logo/wordmark (text-based `<a href="/">vibestats</a>` is acceptable for MVP)
    - Nav links: Home (`/`), Docs (`/docs/quickstart`), GitHub (`https://github.com/stephenleo/vibestats`)
    - GitHub link opens in new tab (`target="_blank" rel="noopener noreferrer"`)
  - [ ] Create `site/src/components/Footer.astro`:
    - GitHub link: `https://github.com/stephenleo/vibestats`
    - License notice: "MIT License"
    - Keep minimal — single line or small footer block

- [ ] Task 4: Update existing page stubs to use `Base.astro` layout (AC: #3)
  - [ ] Update `site/src/pages/index.astro` to use `Base.astro` layout (pass `title="vibestats"`)
  - [ ] Update `site/src/pages/u/index.astro` to use `Base.astro` layout (pass `title="vibestats dashboard"`)
  - [ ] Both pages must continue to build without errors

- [ ] Task 5: Verify `npm run build` and TypeScript check pass (AC: #3)
  - [ ] Run `npm run build` inside `site/` — must complete with 0 errors
  - [ ] Run `npm run check` inside `site/` — must report 0 TypeScript/Astro errors
  - [ ] Confirm output includes at least 2 pages: `/index.html` and `/u/index.html`

## Dev Notes

### Working Directory

All file creation and `npm` commands run from within `site/`. The Astro project root is `site/` — not the repo root.

### What Already Exists (from Story 1.3)

Story 1.3 scaffolded the Astro project. The following are already present — do NOT recreate or overwrite without reason:

```
site/
  astro.config.mjs         ← configured with site URL, trailingSlash: 'never', compressHTML: false
  package.json             ← astro ^6.1.5, cal-heatmap 4.2.4 (exact), "check": "astro check" script
  tsconfig.json            ← strict TypeScript, excludes: node_modules, public, .astro, dist
  .prettierrc              ← 2-space indentation, Astro parser
  src/pages/index.astro    ← minimal stub: title "vibestats", h1, p
  src/pages/u/index.astro  ← minimal stub: "Per-user dashboard — implementation in Epic 7 (story 7.2)"
  public/_redirects        ← Cloudflare Pages rewrite rules (pass-throughs + /:username /u/index.html 200)
  public/favicon.svg       ← exists
  public/favicon.ico       ← exists
```

`site/src/layouts/` does NOT exist yet — create it.
`site/src/components/` does NOT exist yet — create it.

### Architecture Requirements

**File naming:** Astro/JS uses `kebab-case` files. Exception: Astro components by convention use `PascalCase` (`Base.astro`, `Docs.astro`, `Header.astro`, `Footer.astro`). This is standard Astro convention and the architecture explicitly lists these filenames — follow them exactly.

**Complete target structure after this story:**
```
site/src/
  layouts/
    Base.astro           ← base HTML wrapper: head, Header, slot, Footer
    Docs.astro           ← extends Base.astro: sidebar nav + slot
  components/
    Header.astro         ← vibestats logo + nav
    Footer.astro         ← GitHub link + license
  pages/
    index.astro          ← updated to use Base.astro layout
    u/
      index.astro        ← updated to use Base.astro layout
```

**Future components** (`Heatmap.astro`, `YearToggle.astro`) are created in Story 7.2. Do NOT create them in this story.

**Future doc pages** (`docs/quickstart.astro`, `docs/how-it-works.astro`, etc.) are created in Story 7.3. This story only creates the `Docs.astro` layout that those pages will use.

### TypeScript Strict Mode

The project uses `--typescript strict`. Every `.astro` frontmatter block that accepts props MUST define a typed `interface Props`:

```astro
---
interface Props {
  title: string;
}
const { title } = Astro.props;
---
```

Failure to type props will cause `npm run check` to fail.

### Astro Layout Pattern (Correct)

Use `<Layout>` import syntax in pages, not the `layout` frontmatter property (the frontmatter `layout:` shortcut is for Markdown files):

```astro
---
// In site/src/pages/index.astro
import Base from '../layouts/Base.astro';
---
<Base title="vibestats">
  <h1>Content here</h1>
</Base>
```

```astro
---
// In site/src/pages/u/index.astro (one level deeper — note ../../)
import Base from '../../layouts/Base.astro';
---
<Base title="vibestats dashboard">
  <!-- content -->
</Base>
```

```astro
---
// In Base.astro (site/src/layouts/Base.astro)
import Header from '../components/Header.astro';
import Footer from '../components/Footer.astro';
interface Props {
  title: string;
}
const { title } = Astro.props;
---
<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <link rel="icon" type="image/svg+xml" href="/favicon.svg" />
    <link rel="icon" href="/favicon.ico" />
    <meta name="viewport" content="width=device-width" />
    <meta name="generator" content={Astro.generator} />
    <title>{title}</title>
  </head>
  <body>
    <Header />
    <slot />
    <Footer />
  </body>
</html>
```

### Docs.astro Layout Pattern

`Docs.astro` wraps `Base.astro` — do NOT duplicate the `<html>` shell. Use a slot-within-slot approach:

```astro
---
import Base from './Base.astro';
interface Props {
  title: string;
}
const { title } = Astro.props;
---
<Base title={title}>
  <div class="docs-layout">
    <nav class="sidebar">
      <ul>
        <li><a href="/docs/quickstart">Quickstart</a></li>
        <li><a href="/docs/how-it-works">How it works</a></li>
        <li><a href="/docs/cli-reference">CLI reference</a></li>
        <li><a href="/docs/troubleshooting">Troubleshooting</a></li>
      </ul>
    </nav>
    <main>
      <slot />
    </main>
  </div>
</Base>
```

### Indentation and Formatting

- 2-space indentation (enforced by `.prettierrc`)
- The existing pages use 2-space indentation — maintain consistency

### `npm run check` Script

`"check": "astro check"` was added to `site/package.json` in Story 1.3's code review. This runs Astro's TypeScript type-checker. Always run it before marking the story done.

### Build Verification

Run from inside `site/`:
```bash
npm run build && npm run check
```

Expected: build succeeds (≥2 pages), check reports 0 errors.

### No CSS Framework

Architecture does not mandate a CSS framework. Simple inline styles or a minimal `<style>` block in each component is acceptable for MVP. Keep styling minimal — the focus is on correct component structure and build pass.

### Tests

Architecture spec: "Astro: no tests at MVP" (`architecture.md#Test placement`). No test files required for this story.

### Project Structure Notes

- All new files in this story live under `site/src/layouts/` and `site/src/components/`
- Do NOT touch `src/` (Rust), `action/` (Python), `Cargo.toml`, `.github/`, or `install.sh`
- Do NOT modify `site/public/_redirects` — it was hardened in Story 1.3's code review
- Do NOT modify `site/astro.config.mjs` or `site/tsconfig.json` — already correctly configured
- `site/src/pages/docs/` page stubs are out of scope for this story (Story 7.3)

### References

- File locations: [Source: architecture.md#Complete Project Directory Structure — `site/src/layouts/`, `site/src/components/`]
- Component list: [Source: architecture.md#Complete Project Directory Structure — `Base.astro`, `Docs.astro`, `Header.astro`, `Footer.astro`]
- File naming convention: [Source: architecture.md#Naming Patterns — "Astro/JS: kebab-case files"]
- Astro routing and frontend architecture: [Source: architecture.md#Frontend Architecture]
- Story requirements and AC: [Source: epics.md#Story 7.1: Build base layouts and shared Astro components]
- Astro site initialization: [Source: 1-3-initialize-astro-site-project.md — existing file inventory]
- No Astro tests at MVP: [Source: architecture.md#Test placement]
- TypeScript strict: [Source: architecture.md#Selected Starters by Component — Astro: `--typescript strict`]
- FR43 (docs + dashboard site): [Source: prd.md#Functional Requirements]

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

### Completion Notes List

### File List
