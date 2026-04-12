---
stepsCompleted: ['step-01-preflight-and-context', 'step-02-generation-mode', 'step-03-test-strategy']
lastStep: 'step-03-test-strategy'
lastSaved: '2026-04-12'
workflowType: 'testarch-atdd'
inputDocuments:
  - _bmad-output/planning-artifacts/epics.md
  - _bmad/tea/config.yaml
  - _bmad-output/planning-artifacts/architecture.md
---

# ATDD Checklist - Epic 7, Story 7.2: Build per-user dashboard (u/index.astro + cal-heatmap)

**Date:** 2026-04-12
**Author:** Leo
**Primary Test Level:** N/A — No tests per architecture decision

---

## Story Summary

Build the per-user dashboard at `vibestats.dev/[username]` so profile visitors can view an interactive activity heatmap with hover details, powered by `cal-heatmap` and client-side data fetching from the user's public GitHub repo.

**As a** profile visitor,
**I want** to open `vibestats.dev/username` and see an interactive activity heatmap with hover details,
**So that** I can explore a developer's Claude Code usage history beyond what the static README shows.

---

## Acceptance Criteria

1. **Given** a visitor opens `vibestats.dev/stephenleo` **When** the page loads **Then** Cloudflare serves `u/index.html` (via `_redirects`), client-side JS reads `stephenleo` from `window.location.pathname`, and fetches `https://raw.githubusercontent.com/stephenleo/stephenleo/main/vibestats/data.json` (FR29)

2. **Given** the data.json fetch succeeds **When** the heatmap renders **Then** `cal-heatmap` displays the full activity grid for the current year with Claude-orange colour scale (FR30)

3. **Given** the user hovers a day cell **When** the tooltip appears **Then** it shows the date, session count, and approximate active minutes (FR31)

4. **Given** data.json contains multiple years **When** the year toggle is rendered **Then** year buttons appear descending (newest first), current year selected by default, clicking re-renders without a new fetch

5. **Given** the data.json fetch fails **When** the error state renders **Then** the page shows "No vibestats data found for @username"

---

## No Tests Required — Architecture Decision

**Reason:** The project architecture explicitly mandates no tests for the Astro site at MVP.

> **Architecture reference (verbatim):** "Astro: no tests at MVP" (`architecture.md#Test placement`)

This is not a gap or oversight. The decision is intentional: the Astro site is a lightweight presentation layer with client-side rendering logic. While this story contains more dynamic behaviour than 7.1 or 7.4 (client-side fetch, cal-heatmap init, year toggle), the architecture decision covers the entire Astro component at MVP. Build-time validation (`npm run build` + `npm run check`) serves as the primary correctness gate; runtime behaviour is verified manually or via future E2E tests post-MVP.

---

## Verification Strategy (No Automated Tests)

The acceptance criteria for this story are validated at build time and through code review rather than automated test suites:

| Acceptance Criterion | Verification Method |
|---|---|
| AC1: Cloudflare serves `u/index.html`, client-side JS reads username, fetches `data.json` | Code review of `site/src/pages/u/index.astro` — `window.location.pathname` parsing and `fetch` URL construction are static literals; `site/public/_redirects` contains `/:username  /u/index.html  200` |
| AC2: `cal-heatmap` renders activity grid with Claude-orange colour scale | `npm run build` — build fails if `cal-heatmap` import is missing or malformed; colour config is a static value in the template |
| AC3: Tooltip shows date, session count, active minutes on hover | Code review of `cal-heatmap` tooltip configuration in `site/src/pages/u/index.astro` |
| AC4: Year buttons descend newest-first, current year selected, re-render without re-fetch | Code review of year toggle implementation — data parsed once from `data.json`, year filter applied client-side |
| AC5: Fetch failure renders "No vibestats data found for @username" | Code review of error-handling branch in client-side script |

**Build commands (run from `site/`):**
```bash
npm run build   # Build all pages — must complete with 0 errors
npm run check   # Astro TypeScript check — must report 0 errors
```

---

## Red-Green-Refactor Workflow

### RED Phase — Skipped (No Tests)

No failing tests are created for this story per the architecture decision above.

### GREEN Phase — Build-Gated Delivery

The implementation is considered correct when:
- `npm run build` exits 0 (including `dist/u/index.html` generated)
- `npm run check` reports 0 TypeScript/Astro errors
- `site/public/_redirects` contains `/:username  /u/index.html  200`
- `cal-heatmap` is declared as a dependency in `site/package.json`
- The username is read from `window.location.pathname` (not hardcoded)
- The error state string `"No vibestats data found for @"` is present in `site/src/pages/u/index.astro`

### REFACTOR Phase — Standard Code Review

Code quality is enforced through the story's code-review workflow after implementation is complete.

---

## Next Steps

1. Developer implements story tasks per `epics.md#Story 7.2` (GH Issue #36)
2. Verify `cal-heatmap` is pinned in `site/package.json`
3. Run `npm run build && npm run check` inside `site/` — must pass with 0 errors
4. Verify `dist/u/index.html` is generated
5. Verify `site/public/_redirects` contains `/:username  /u/index.html  200`
6. Submit PR with `Closes #36`
7. Run code-review workflow
8. Merge and mark story `done` in sprint-status.yaml

---

## Knowledge Base References Applied

- **architecture.md#Test placement** — Astro: no tests at MVP (primary decision driver)
- **test-quality.md** — Build-time verification as valid correctness gate for presentation layers
- **component-tdd.md** — Not applicable: Astro pages are static templates, no component test framework configured at MVP

---

**Generated by BMad TEA Agent** — 2026-04-12
