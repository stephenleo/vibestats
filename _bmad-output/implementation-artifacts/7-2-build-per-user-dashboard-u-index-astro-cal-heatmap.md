# Story 7.2: Build per-user dashboard (u/index.astro + cal-heatmap)

Status: done

<!-- GH Issue: #36 | Epic: #7 | PR must include: Closes #36 -->

## Story

As a profile visitor,
I want to open `vibestats.dev/username` and see an interactive activity heatmap with hover details,
So that I can explore a developer's Claude Code usage history beyond what the static README shows.

## Acceptance Criteria

1. **Given** a visitor opens `vibestats.dev/stephenleo` **When** the page loads **Then** Cloudflare serves `u/index.html` (via `_redirects`), client-side JS reads `stephenleo` from `window.location.pathname`, and fetches `https://raw.githubusercontent.com/stephenleo/stephenleo/main/vibestats/data.json` (FR29)

2. **Given** the data.json fetch succeeds **When** the heatmap renders **Then** `cal-heatmap` displays the full activity grid for the current year with Claude-orange colour scale (FR30)

3. **Given** the user hovers a day cell **When** the tooltip appears **Then** it shows the date, session count, and approximate active minutes (FR31)

4. **Given** data.json contains multiple years **When** the year toggle is rendered **Then** year buttons appear descending (newest first), current year selected by default, clicking re-renders without a new fetch

5. **Given** the data.json fetch fails **When** the error state renders **Then** the page shows "No vibestats data found for @username"

## Tasks / Subtasks

- [x] Task 1: Implement `site/src/pages/u/index.astro` — dashboard shell (AC: #1, #2, #3, #4, #5)
  - [x] Replace stub content with full dashboard implementation
  - [x] Import `Base` from `'../../layouts/Base.astro'`
  - [x] Set page title to `"vibestats — @username dashboard"` (dynamic username set client-side)
  - [x] Add `<div id="cal-heatmap"></div>` as the mount point for the heatmap
  - [x] Add `<div id="year-toggle"></div>` for year navigation buttons
  - [x] Add `<div id="error-state" hidden></div>` for the error state message
  - [x] Add `<div id="loading-state">Loading...</div>` for initial load state
  - [x] Add `<script>` block with client-side logic:
    - Parse username from `window.location.pathname.split('/').filter(Boolean)[0]`
    - Update page `<title>` to include username dynamically
    - Fetch `https://raw.githubusercontent.com/{username}/{username}/main/vibestats/data.json`
    - On fetch success: parse data, render year toggle, initialise cal-heatmap
    - On fetch failure / non-OK response: show error state "No vibestats data found for @{username}"
  - [x] Import `CalHeatmap` from `'cal-heatmap'` and `Tooltip` from `'cal-heatmap/plugins/Tooltip'` in the `<script>` block
  - [x] Import cal-heatmap CSS: `import 'cal-heatmap/cal-heatmap.css'` in `<script>` block
  - [x] Configure cal-heatmap with Claude-orange colour scale: `range: ['#fff7ed', '#ea580c']` (light cream → Claude orange)
  - [x] Configure Tooltip plugin to show: date (formatted), session count, approximate active minutes
  - [x] Set `domain: { type: 'month' }` and `subDomain: { type: 'day' }` for month-grid layout showing the selected year as 12 months
  - [x] Year toggle: extract unique years from `Object.keys(data.days)`, sort descending, render `<button>` per year, default to current year, on click re-render cal-heatmap with filtered data without re-fetching
  - [x] TypeScript strict: no `interface Props` needed (no `Astro.props` used); `<script>` uses `as HTMLElement` casts where needed

- [x] Task 2: Verify `site/public/_redirects` has the correct catch-all rule (AC: #1)
  - [x] Confirm `/:username  /u/index.html  200` line exists in `site/public/_redirects`
  - [x] No modification needed if already present (added in Story 1.3)

- [x] Task 3: Verify `cal-heatmap` is pinned in `site/package.json` (AC: #2)
  - [x] Confirm `"cal-heatmap": "4.2.4"` appears in `dependencies`
  - [x] No modification needed if already present (added in Story 1.3)

- [x] Task 4: Run build and TypeScript check (AC: #1, #2, #3, #4, #5)
  - [x] Run `npm run build` from `site/` — must complete with 0 errors
  - [x] Run `npm run check` from `site/` — must report 0 TypeScript/Astro errors
  - [x] Confirm `dist/u/index.html` is generated

## Dev Notes

### Working Directory

All file operations and `npm` commands run from within `site/`. The Astro project root is `site/` — NOT the repo root.

### What Already Exists (from Stories 1.3 and 7.1)

```
site/src/layouts/Base.astro          ← base HTML wrapper: head, Header, slot, Footer
site/src/layouts/Docs.astro          ← extends Base.astro: sidebar nav + slot
site/src/components/Header.astro     ← vibestats logo + nav
site/src/components/Footer.astro     ← GitHub link + MIT License
site/src/pages/u/index.astro         ← STUB — replace contents (keep file)
site/public/_redirects               ← already has /:username /u/index.html 200
site/package.json                    ← cal-heatmap 4.2.4 already declared
```

### Architecture: Username Discovery

The Astro site is fully SSG — there is NO server-side rendering. Username is extracted at runtime:

```javascript
const username = window.location.pathname.split('/').filter(Boolean)[0];
```

For `vibestats.dev/stephenleo`, Cloudflare serves `u/index.html` (via `_redirects`), and the path is `/stephenleo`. The username is therefore `"stephenleo"`.

### Architecture: data.json Fetch

```javascript
const url = `https://raw.githubusercontent.com/${username}/${username}/main/vibestats/data.json`;
const resp = await fetch(url);
if (!resp.ok) { /* show error */ }
const data = await resp.json();
```

The public aggregated schema is:
```json
{
  "generated_at": "<ISO 8601 UTC>",
  "username": "<github username>",
  "days": {
    "YYYY-MM-DD": { "sessions": N, "active_minutes": N }
  }
}
```

### Architecture: cal-heatmap Integration

cal-heatmap 4.2.4 is bundled via npm (already pinned). Use ES module imports inside the Astro `<script>` block:

```javascript
import CalHeatmap from 'cal-heatmap';
import Tooltip from 'cal-heatmap/plugins/Tooltip';
import 'cal-heatmap/cal-heatmap.css';
```

Transform `data.days` to cal-heatmap's expected array format:
```javascript
const calData = Object.entries(data.days)
  .filter(([date]) => date.startsWith(selectedYear))
  .map(([date, v]) => ({ date, value: v.sessions }));
```

cal-heatmap configuration example:
```javascript
const cal = new CalHeatmap();
cal.paint(
  {
    itemSelector: '#cal-heatmap',
    data: {
      source: calData,
      x: datum => +new Date(datum.date),
      y: 'value',
    },
    date: { start: new Date(`${selectedYear}-01-01`) },
    range: 12,
    domain: { type: 'month', label: { text: 'MMM' } },
    subDomain: { type: 'day', radius: 2 },
    scale: {
      color: {
        type: 'linear',
        range: ['#fff7ed', '#ea580c'],
        domain: [0, 10],
      },
    },
  },
  [
    [
      Tooltip,
      {
        text: (date, value, dayjsDate) => {
          const dayData = data.days[dayjsDate.format('YYYY-MM-DD')];
          if (!dayData || !value) return `No activity on ${dayjsDate.format('MMM D, YYYY')}`;
          return `${dayjsDate.format('MMM D, YYYY')}: ${dayData.sessions} sessions, ~${dayData.active_minutes} min`;
        },
      },
    ],
  ]
);
```

### Architecture: Year Toggle

Extract years from `data.days` keys, sort descending, default to current year:
```javascript
const years = [...new Set(Object.keys(data.days).map(d => d.slice(0, 4)))]
  .sort((a, b) => b.localeCompare(a));
const currentYear = String(new Date().getFullYear());
const defaultYear = years.includes(currentYear) ? currentYear : years[0];
```

Re-render on year button click — destroy old instance and repaint (cal-heatmap pattern):
```javascript
function renderHeatmap(selectedYear) {
  if (calInstance) { calInstance.destroy(); }
  // ... paint new instance
}
```

### Claude-Orange Colour Scale

Claude's brand orange is `#ea580c` (Tailwind orange-600). The scale runs from near-white (`#fff7ed`, Tailwind orange-50) to Claude orange (`#ea580c`).

### TypeScript Notes

- `u/index.astro` has NO `Astro.props` → no `interface Props` required
- Inline `<script>` blocks in Astro support TypeScript; cast DOM elements: `document.getElementById('cal-heatmap') as HTMLElement`
- `cal-heatmap` package ships type definitions — import types will resolve correctly

### No Tests

Architecture spec: "Astro: no tests at MVP" (`architecture.md#Test placement`). No test files are required. Build validation (`npm run build` + `npm run check`) is the correctness gate.

### References

- Story AC and FR29–FR31: [Source: _bmad-output/planning-artifacts/epics.md#Story-7.2]
- Architecture data flow: [Source: _bmad-output/planning-artifacts/architecture.md — username routing, cal-heatmap]
- cal-heatmap API: [Source: cal-heatmap.com/docs — paint(), data source, tooltip plugin, scale]
- ATDD checklist verification strategy: [Source: _bmad-output/test-artifacts/atdd-checklist-7.2-build-per-user-dashboard-u-index-astro-cal-heatmap.md]
- Base layout pattern: [Source: 7-1-build-base-layouts-and-shared-astro-components.md]

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

None.

### Completion Notes List

- Replaced `site/src/pages/u/index.astro` stub with full per-user dashboard implementation
- Client-side JS reads username from `window.location.pathname`, fetches `https://raw.githubusercontent.com/{username}/{username}/main/vibestats/data.json`
- On fetch success: year toggle rendered (years descending, current year default), cal-heatmap initialised with Claude-orange colour scale (`#fff7ed` → `#ea580c`)
- Tooltip shows date, session count, and active minutes on hover
- Year toggle re-renders cal-heatmap without additional fetch (data held in module-level variable)
- On fetch failure: error state "No vibestats data found for @{username}" shown
- Created `site/src/types/cal-heatmap.d.ts` to provide ambient type declarations for `cal-heatmap` and `cal-heatmap/plugins/Tooltip` — required because the package's `exports` map lacks a `"types"` condition
- `npm run build`: 2 pages built, 0 errors — `dist/u/index.html` generated
- `npm run check`: 0 errors, 0 warnings, 0 hints across 8 Astro/TS files
- `site/public/_redirects` and `site/package.json` already correct from Stories 1.3 and 7.1 — no changes needed

### File List

- site/src/pages/u/index.astro (modified)
- site/src/types/cal-heatmap.d.ts (new)

### Change Log

- 2026-04-12: Implemented story 7.2 — full per-user dashboard with cal-heatmap, year toggle, tooltip, Claude-orange colour scale, error handling; build and TypeScript check pass with 0 errors
