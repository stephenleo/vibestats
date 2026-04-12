---
stepsCompleted: ['step-01-preflight-and-context', 'step-02-generation-mode', 'step-03-test-strategy']
lastStep: 'step-03-test-strategy'
lastSaved: '2026-04-12'
workflowType: 'testarch-atdd'
inputDocuments:
  - _bmad-output/implementation-artifacts/7-3-build-documentation-pages.md
  - _bmad/tea/config.yaml
  - _bmad-output/planning-artifacts/architecture.md
  - _bmad-output/test-artifacts/test-design-epic-7.md
---

# ATDD Checklist - Epic 7, Story 7.3: Build documentation pages

**Date:** 2026-04-12
**Author:** Leo
**Primary Test Level:** N/A — No tests per architecture decision

---

## Story Summary

Build four static documentation pages (`quickstart.astro`, `how-it-works.astro`, `cli-reference.astro`, `troubleshooting.astro`) under `site/src/pages/docs/` using the existing `Docs.astro` layout, so that potential vibestats users can install and use vibestats without reading the source code.

**As a** potential vibestats user,
**I want** clear documentation covering quickstart, architecture, CLI reference, and troubleshooting,
**So that** I can install and use vibestats without reading the source code.

---

## Acceptance Criteria

1. Quickstart page shows the install command, lists the 5-minute install steps, and links to the CLI reference (FR43)
2. "How it works" page includes an architecture diagram showing the data flow: JSONL → vibestats-data → GitHub Action → profile README → vibestats.dev
3. CLI reference page documents every subcommand (`status`, `sync`, `sync --backfill`, `machines list`, `machines remove`, `auth`, `uninstall`) with description, flags, and example output
4. Troubleshooting page covers: token expiry fix, hook not firing, missing machine data, and how to trigger a manual backfill
5. `npm run build` inside `site/` completes without TypeScript or Astro errors after all four doc pages are added

---

## No Tests Required — Architecture Decision

**Reason:** The project architecture explicitly mandates no tests for the Astro site at MVP.

> **Story dev notes (verbatim):** "Architecture spec: 'Astro: no tests at MVP' (`architecture.md#Test placement`). No test files required for this story."

**Architecture reference:** `architecture.md#Test placement` — Astro: no tests at MVP

This is not a gap or oversight. The decision is intentional: the Astro site is a lightweight presentation layer (static content pages) with no business logic to unit-test. Build-time validation (`npm run build` + `npm run check`) serves as the primary correctness gate, and content correctness is verified through manual review.

**Epic-level test design reference:** `test-design-epic-7.md` — Story 7.3 content correctness (R-008) is classified P1/P2 content tests, intentionally deferred to the epic-level test pass rather than per-story automated tests.

---

## Verification Strategy (No Automated Tests)

The acceptance criteria for this story are validated at build time and through manual content review rather than automated test suites:

| Acceptance Criterion | Verification Method |
|---|---|
| AC1: Quickstart page with install command and install steps | `npm run build` — Astro compile catches broken imports and template errors; content verified by manual review |
| AC2: How-it-works page with architecture diagram | `npm run build` — build fails on malformed Astro/HTML; diagram presence verified by manual review |
| AC3: CLI reference documents all 7 subcommands | `npm run build` — build validates page structure; all 7 subcommand names verified by manual review of built HTML |
| AC4: Troubleshooting page covers all 4 scenarios | `npm run build` — build validates page structure; four scenario headings verified by manual review |
| AC5: Build completes without TypeScript/Astro errors | `npm run build && npm run check` — explicit build and type-check gate |

**Build commands (run from `site/`):**
```bash
npm run build   # Build all pages — must complete with 0 errors
npm run check   # Astro TypeScript check — must report 0 errors
```

**Expected build output (6 pages minimum):**
- `dist/index.html`
- `dist/u/index.html`
- `dist/docs/quickstart/index.html` (or `/docs/quickstart` with `trailingSlash: 'never'`)
- `dist/docs/how-it-works/index.html`
- `dist/docs/cli-reference/index.html`
- `dist/docs/troubleshooting/index.html`

---

## Red-Green-Refactor Workflow

### RED Phase — Skipped (No Tests)

No failing tests are created for this story per the architecture decision above.

### GREEN Phase — Build-Gated Delivery

The implementation is considered correct when:
- `npm run build` exits 0 with all 4 new doc pages in `dist/`
- `npm run check` reports 0 TypeScript/Astro errors
- Manual review confirms all required content sections are present in each page

### REFACTOR Phase — Standard Code Review

Code quality is enforced through the story's code-review workflow after implementation is complete.

---

## Epic-Level Test Coverage (Deferred)

Story 7.3 content correctness (R-008, score 4) is covered by the epic-level test plan in `test-design-epic-7.md`. The following P1/P2 content tests are planned for the epic-level automated test pass (not this story's ATDD):

| Test | Priority | Risk | Description |
|---|---|---|---|
| Docs quickstart page: install command + 5-step list + CLI link present | P1 | R-008 | Parse built HTML; assert headings and code block present |
| Docs "How it works" page: architecture diagram/description present | P1 | R-008 | Parse built HTML; assert diagram or description present |
| Docs CLI reference: all 7 subcommand names present in built HTML | P1 | R-008 | Assert all subcommand names in built HTML |
| Docs troubleshooting page: 4 section headings present | P2 | R-008 | Parse built HTML; assert four section headings |

These tests are scoped to the epic-level test pass (post-implementation of all four Epic 7 stories) and are not blocking gates for this individual story.

---

## Next Steps

1. Developer implements story tasks per `7-3-build-documentation-pages.md`
2. Run `npm run build && npm run check` inside `site/` — must pass with 0 errors
3. Confirm 4 new pages appear in `dist/docs/`
4. Manually verify all required content is present (install command, 7 subcommands, 4 troubleshooting scenarios, architecture diagram)
5. Submit PR with `Closes #37`
6. Run code-review workflow
7. Merge and mark story `done` in sprint-status.yaml

---

## Knowledge Base References Applied

- **architecture.md#Test placement** — Astro: no tests at MVP (primary decision driver)
- **test-quality.md** — Build-time verification as valid correctness gate for presentation layers
- **component-tdd.md** — Not applicable: Astro components are static templates, no component test framework configured
- **test-design-epic-7.md** — R-008 content tests deferred to epic-level test pass

---

**Generated by BMad TEA Agent** — 2026-04-12
