---
stepsCompleted: ['step-01-detect-mode', 'step-02-load-context', 'step-03-risk-and-testability', 'step-04-coverage-plan', 'step-05-generate-output']
lastStep: 'step-05-generate-output'
lastSaved: '2026-04-12'
mode: 'epic-level'
epic: 7
inputDocuments:
  - '_bmad-output/planning-artifacts/epics.md'
  - '_bmad-output/planning-artifacts/prd.md'
  - '_bmad-output/planning-artifacts/architecture.md'
  - '_bmad-output/implementation-artifacts/sprint-status.yaml'
  - 'site/astro.config.mjs'
  - 'site/package.json'
  - 'site/src/pages/index.astro'
  - 'site/src/pages/u/index.astro'
  - 'site/public/_redirects'
---

# Test Design: Epic 7 — vibestats.dev Astro Site

**Date:** 2026-04-12
**Author:** Leo
**Status:** Active

---

## Executive Summary

**Scope:** Epic-level test design for Epic 7 — vibestats.dev Astro Site.

Epic 7 builds the `vibestats.dev` static site on Astro/Cloudflare Pages: shared base layouts and components (Story 7.1), the per-user interactive dashboard at `/username` with `cal-heatmap` and client-side data fetch (Story 7.2), documentation pages (Story 7.3), and the landing page (Story 7.4). All four stories are currently in `backlog`. The site initialisation (Story 1.3) is already complete, including `_redirects` and the Astro configuration.

**Risk Summary:**

- Total risks identified: 9
- High-priority risks (≥6): 4
- Critical categories: BUS, TECH, PERF, OPS

**Coverage Summary:**

- P0 scenarios: 8 (~14–22 hours)
- P1 scenarios: 11 (~11–20 hours)
- P2 scenarios: 8 (~4–10 hours)
- P3 scenarios: 3 (~1–3 hours)
- **Total effort:** ~30–55 hours (~1–2 weeks)

---

## Not in Scope

| Item | Reasoning | Mitigation |
| --- | --- | --- |
| Cloudflare Pages deploy pipeline | Epic 8 scope (Story 8.2) | Covered in Epic 8 test design |
| GitHub Actions Marketplace listing | Epic 8 scope (Story 8.3) | Covered in Epic 8 test design |
| Bash installer | Epic 6 scope | Covered in Epic 6 test design |
| `data.json` generation correctness | Epic 5 scope (Stories 5.1–5.2) | Covered in Epic 5 test design |
| Rust binary sync logic | Epics 2–4 scope | Covered in prior epic test designs |

---

## Risk Assessment

> P0/P1/P2/P3 = priority and risk level, NOT execution timing.

### High-Priority Risks (Score ≥6)

| Risk ID | Category | Description | Probability | Impact | Score | Mitigation | Owner | Timeline |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| R-001 | BUS | `vibestats.dev/username` dashboard fails silently — blank page with no error message — when `data.json` is inaccessible or the fetch fails, violating the "No vibestats data found for @username" requirement (Story 7.2 AC, FR29) | 3 | 3 | 9 | E2E test: mock `fetch` to reject; assert error banner text matches spec exactly; test with both 404 and network error | Dev/QA | Pre-merge |
| R-002 | TECH | `_redirects` catch-all maps `/username` to `u/index.html` but Cloudflare serves a static file for `/favicon.ico` or `/_astro/*` instead of passing through, breaking bundled assets (Story 7.1 AC, architecture routing) | 3 | 2 | 6 | Build test: run `npm run build`, assert all `_astro/*` bundle paths resolve correctly; integration test: verify `_redirects` pass-through rules are ordered before catch-all | Dev/QA | Pre-merge |
| R-003 | PERF | Dashboard render time exceeds 3-second NFR4 target — `cal-heatmap` bundle or client-side `data.json` fetch is too slow on a standard connection | 2 | 3 | 6 | Performance test: measure fetch + heatmap render time with a ~73KB fixture `data.json`; assert total paint <3s; verify `cal-heatmap` is bundled (not CDN-loaded) | Dev/QA | Pre-Epic-7-done |
| R-004 | OPS | `npm run build` fails due to TypeScript or Astro config errors introduced during component implementation, breaking the Cloudflare Pages deploy gate (Story 7.1 AC, Story 7.4 AC) | 2 | 3 | 6 | CI check: run `npm run build` and `astro check` on every PR; treat non-zero exit as gate failure | Dev/QA | Pre-merge |

### Medium-Priority Risks (Score 3–4)

| Risk ID | Category | Description | Probability | Impact | Score | Mitigation | Owner |
| --- | --- | --- | --- | --- | --- | --- | --- |
| R-005 | BUS | Year toggle on dashboard re-fetches `data.json` instead of filtering client-side, causing multiple network calls and violating the "clicking re-renders without a new fetch" acceptance criterion (Story 7.2 AC) | 2 | 2 | 4 | Unit/component test: spy on `fetch`; toggle year; assert fetch call count remains 1 | Dev/QA |
| R-006 | BUS | `data.json` containing multiple years does not render year buttons in descending order (newest first), or current year is not selected by default (Story 7.2 AC) | 2 | 2 | 4 | Component test: inject multi-year fixture; assert button order descending; assert current year button has `aria-pressed` or active class | Dev/QA |
| R-007 | TECH | `cal-heatmap` is loaded from a CDN at runtime instead of being bundled into the Astro build, creating a hard dependency on third-party availability (architecture requirement, ADR) | 2 | 2 | 4 | Build test: inspect `dist/_astro/` output; assert `cal-heatmap` module present in bundle; assert no CDN URL in HTML output | Dev/QA |
| R-008 | BUS | Docs pages are missing one or more required sections (quickstart install command, architecture diagram, all CLI subcommands, all four troubleshooting topics) as specified in Story 7.3 ACs | 2 | 2 | 4 | Content test: parse built HTML for each docs page; assert required headings and code blocks are present | Dev/QA |

### Low-Priority Risks (Score 1–2)

| Risk ID | Category | Description | Probability | Impact | Score | Action |
| --- | --- | --- | --- | --- | --- | --- |
| R-009 | BUS | Landing page install command renders as text that cannot be copied or differs from the spec string `curl -sSf https://vibestats.dev/install.sh \| bash` (Story 7.4 AC) | 1 | 2 | 2 | Monitor — content test: assert exact install command string in built HTML |

### Risk Category Legend

- **TECH**: Technical/Architecture (integration, routing, bundling, structure)
- **SEC**: Security (data exposure, access control)
- **PERF**: Performance (SLA violations, NFR4 dashboard load)
- **DATA**: Data Integrity (loss, corruption, schema mismatch)
- **BUS**: Business Impact (UX harm, logic errors, missing content)
- **OPS**: Operations (build pipeline, deployment, config)

---

## Entry Criteria

- [ ] Story 1.3 (Astro site init) marked `done` — confirmed; `site/` directory exists with `astro.config.mjs`, `package.json`, and stub pages
- [ ] `cal-heatmap@4.2.4` locked in `package.json` — confirmed
- [ ] `_redirects` catch-all rule in place — confirmed
- [ ] Epic 7 stories (7.1–7.4) requirements and acceptance criteria agreed upon by Dev and QA
- [ ] A representative `data.json` fixture (~73KB, multiple years, multiple days) available for dashboard tests
- [ ] Test environment (Node ≥22.12.0, Playwright or similar for E2E) provisioned

## Exit Criteria

- [ ] All P0 tests passing (100%)
- [ ] All P1 tests passing (≥95%)
- [ ] No open BUS or PERF category risks unmitigated
- [ ] `npm run build` passes cleanly with zero TypeScript/Astro errors
- [ ] NFR4 dashboard render <3s verified against fixture `data.json`
- [ ] All four docs sections (quickstart, how-it-works, CLI reference, troubleshooting) confirmed present in built output

---

## Test Coverage Plan

> P0/P1/P2/P3 = priority and risk level, NOT execution timing. Execution timing is defined in the Execution Strategy section.

### P0 (Critical)

**Criteria:** Blocks core user journey + High risk (≥6) + No workaround

| Requirement | Test Level | Risk Link | Test Count | Owner | Notes |
| --- | --- | --- | --- | --- | --- |
| Dashboard shows "No vibestats data found for @username" when `data.json` fetch fails with 404 (Story 7.2 AC, FR29) | E2E/Component | R-001 | 1 | Dev/QA | Mock `fetch` → 404; assert error message exact text |
| Dashboard shows "No vibestats data found for @username" when `data.json` fetch fails with network error (Story 7.2 AC, FR29) | E2E/Component | R-001 | 1 | Dev/QA | Mock `fetch` → reject; assert error message |
| Dashboard renders `cal-heatmap` grid with full activity for current year when `data.json` fetch succeeds (Story 7.2 AC, FR30) | E2E/Component | R-001 | 1 | Dev/QA | Inject fixture data; assert heatmap element present and populated |
| `npm run build` completes with zero TypeScript/Astro errors after all Epic 7 components are added (Story 7.1 AC, Story 7.4 AC) | Build | R-004 | 1 | Dev/QA | CI: run `astro check && npm run build`; assert exit code 0 |
| `_redirects` pass-through rules: `/favicon.ico`, `/favicon.svg`, `/_astro/*`, `/u`, `/u/*` serve static assets without being swallowed by catch-all (Story 7.1 AC) | Build/Integration | R-002 | 2 | Dev/QA | Assert rules appear before catch-all in `_redirects`; verify build output paths |
| Dashboard `data.json` fetch constructs URL from `window.location.pathname`, not hardcoded username (Story 7.2 AC, FR29) | Component | R-001 | 1 | Dev/QA | Inject `pathname=/stephenleo`; assert fetch URL contains `stephenleo` |
| `cal-heatmap` is bundled into Astro build output — no CDN reference in HTML (architecture requirement) | Build | R-007 | 1 | Dev/QA | Inspect `dist/` for cal-heatmap chunk; assert no `cdn.jsdelivr.net` or `unpkg.com` in HTML |

**Total P0:** 8 tests

### P1 (High)

**Criteria:** Core site features + Medium–high risk (3–5) + Primary user paths

| Requirement | Test Level | Risk Link | Test Count | Owner | Notes |
| --- | --- | --- | --- | --- | --- |
| Year toggle shows buttons descending (newest first), current year selected by default (Story 7.2 AC) | Component | R-006 | 2 | Dev/QA | Multi-year fixture; assert button order and default selection |
| Clicking year toggle re-renders heatmap without a new `data.json` fetch (Story 7.2 AC) | Component | R-005 | 1 | Dev/QA | Spy on `fetch`; toggle year; assert call count = 1 |
| Hover tooltip on day cell shows date, session count, and approximate active minutes (Story 7.2 AC, FR31) | E2E | R-001 | 2 | Dev/QA | Playwright hover on cell; assert tooltip text format |
| Base layout (`Base.astro`): all pages share consistent `<head>`, `Header.astro` (logo + nav), `Footer.astro` (GitHub link, license) (Story 7.1 AC) | Component | R-004 | 2 | Dev/QA | Render each page; assert header/footer presence and nav links |
| Docs layout (`Docs.astro`): sidebar navigation links to all four docs pages when rendered (Story 7.1 AC) | Component | R-004 | 1 | Dev/QA | Assert sidebar links exist for quickstart, how-it-works, CLI reference, troubleshooting |
| Docs quickstart page: contains install command and 5-minute install steps, links to CLI reference (Story 7.3 AC, FR43) | Content | R-008 | 1 | Dev/QA | Parse built HTML; assert headings and code block present |
| Docs "How it works" page: contains architecture diagram data flow (JSONL → vibestats-data → GitHub Action → README → vibestats.dev) (Story 7.3 AC) | Content | R-008 | 1 | Dev/QA | Parse built HTML; assert diagram or description present |
| Docs CLI reference: documents all 7 subcommands with description, flags, and example output (Story 7.3 AC) | Content | R-008 | 1 | Dev/QA | Assert all subcommand names present in built HTML |

**Total P1:** 11 tests

### P2 (Medium)

**Criteria:** Secondary features + Low–medium risk (1–4) + Edge cases

| Requirement | Test Level | Risk Link | Test Count | Owner | Notes |
| --- | --- | --- | --- | --- | --- |
| Dashboard render time <3s on fixture `data.json` (~73KB, 5 years of data) (NFR4) | Performance | R-003 | 1 | Dev/QA | Playwright `performance.now()`; assert <3000ms from load to heatmap painted |
| Docs troubleshooting page: covers all four required topics (token expiry, hook not firing, missing machine data, manual backfill) (Story 7.3 AC) | Content | R-008 | 1 | Dev/QA | Parse built HTML; assert four section headings present |
| Landing page: shows one-line install command in copyable code block, example heatmap SVG, three-bullet "why" section (Story 7.4 AC) | Content | R-009 | 2 | Dev/QA | Parse built HTML; assert code block, SVG img, and bullet list present |
| Landing page install command text exactly matches `curl -sSf https://vibestats.dev/install.sh \| bash` (Story 7.4 AC) | Content | R-009 | 1 | Dev/QA | Assert exact string in built HTML |
| Dashboard: `data.json` containing a single year renders without year toggle buttons (graceful degrade) | Component | R-006 | 1 | Dev/QA | Single-year fixture; assert no year buttons or only one button |
| `cal-heatmap` colour scale uses Claude orange (`#f97316` at max intensity) for active day cells (Story 7.2 AC, FR30) | Component | R-003 | 1 | Dev/QA | Inspect rendered cell styles; assert orange fill at maximum activity level |
| `/u/index.html` is the correct target for `/_redirects` catch-all (routing correctness) | Build | R-002 | 1 | Dev/QA | Assert `u/index.html` present in `dist/` after build |

**Total P2:** 8 tests

### P3 (Low)

**Criteria:** Nice-to-have + Exploratory + Benchmarks

| Requirement | Test Level | Test Count | Owner | Notes |
| --- | --- | --- | --- | --- |
| Visual snapshot regression of landing page (full-page screenshot) | Visual | 1 | Dev | Golden file comparison; run before major design changes |
| Visual snapshot regression of dashboard heatmap with fixture `data.json` | Visual | 1 | Dev | Golden file for cal-heatmap rendered output |
| Accessibility audit: landing page and dashboard score ≥90 on Lighthouse accessibility (aria labels, contrast) | Accessibility | 1 | Dev/QA | Informational; not a hard gate at MVP |

**Total P3:** 3 tests

---

## Execution Strategy

**Philosophy:** Epic 7 is a pure frontend (Astro SSG + client-side JS). The primary gate is `npm run build` + `astro check` on every PR. Component and content tests are fast and should run on every PR. E2E tests requiring a browser require Playwright setup but are short (<5 min).

### Every PR

- All P0 tests (~8 tests, <5 min)
- All P1 tests (~11 tests, <5 min)
- P2 build and content tests (~6 tests, <3 min)
- **Total PR gate:** ~25 tests, target <10 minutes

### Nightly

- P2 performance test (NFR4 dashboard render <3s) — requires real network conditions or realistic network throttling
- P3 accessibility audit

### On-Demand / Manual

- P3 visual snapshot regressions (run before any design changes or major refactors)
- Full smoke test of deployed Cloudflare Pages preview URL (post-deploy validation)

---

## Resource Estimates

| Priority | Count | Total Effort | Notes |
| --- | --- | --- | --- |
| P0 | 8 | ~14–22 hours | Fetch mocking, build assertions, routing validation |
| P1 | 11 | ~11–20 hours | Component tests, content assertions, hover E2E |
| P2 | 8 | ~4–10 hours | Performance, edge cases, colour assertions |
| P3 | 3 | ~1–3 hours | Snapshots, accessibility |
| **Total** | **30** | **~30–55 hours** | **~1–2 weeks** |

**Prerequisites:**

- **Test fixtures:**
  - `site/tests/fixtures/data-multi-year.json` — ~73KB, 5 years, realistic daily data
  - `site/tests/fixtures/data-single-year.json` — one year, minimal entries
- **Tooling:**
  - Playwright for E2E and component tests (already gated by `tea_use_playwright_utils: true`)
  - `astro check` for TypeScript validation
  - `npm run build` with build output inspection
- **Environment:**
  - Node ≥22.12.0 (matches `package.json` `engines`)
  - Playwright browser binaries (Chromium sufficient for MVP)

---

## Quality Gate Criteria

### Pass/Fail Thresholds

- **P0 pass rate:** 100% (no exceptions; build failures and fetch-error-handling failures are release blockers)
- **P1 pass rate:** ≥95% (failures require triage and documented waiver before merge)
- **P2/P3 pass rate:** ≥90% (informational; failures logged but do not block merge)
- **R-001 to R-004 mitigations:** 100% complete before Epic 7 stories are marked `done`

### Coverage Targets

- Dashboard client-side fetch flow (success + error paths): 100%
- `cal-heatmap` bundling and routing: 100%
- Docs content (all required sections): 100%
- Landing page required elements: 100%
- Performance (NFR4): measured and recorded (≥1 sample)

### Non-Negotiable Requirements

- [ ] All P0 tests pass on every PR
- [ ] `npm run build` exits 0 with zero TypeScript/Astro errors (R-004)
- [ ] R-001 (error state) verified by automated component/E2E test
- [ ] NFR4 (<3s dashboard render) measured and within threshold

---

## Mitigation Plans

### R-001: Dashboard silent failure on `data.json` fetch error (Score: 9)

**Mitigation Strategy:** Implement `try/catch` around the client-side `fetch` call in `u/index.astro`. Display "No vibestats data found for @username" banner in the catch block and on HTTP error status. Cover both 404 and network rejection in automated tests.
**Owner:** Dev/QA
**Timeline:** Pre-merge (Story 7.2)
**Status:** Planned
**Verification:** E2E test with mocked fetch failure asserts banner text

### R-002: Routing — static assets swallowed by `_redirects` catch-all (Score: 6)

**Mitigation Strategy:** Confirm pass-through rules for `/favicon.ico`, `/favicon.svg`, `/_astro/*`, `/u`, and `/u/*` appear before the `/:username` catch-all in `_redirects`. Add a build-time test that parses `_redirects` and validates ordering.
**Owner:** Dev/QA
**Timeline:** Pre-merge (Story 7.1)
**Status:** Planned (partial — `_redirects` already has correct ordering; needs automated validation)
**Verification:** Build test parses `_redirects` line order; assert catch-all is last

### R-003: Dashboard render >3s (Score: 6)

**Mitigation Strategy:** Bundle `cal-heatmap` into Astro build (already done per `package.json`). Run Playwright performance measurement with `~73KB` fixture `data.json`. If >3s, investigate bundle size and consider lazy initialisation.
**Owner:** Dev/QA
**Timeline:** Pre-Epic-7-done
**Status:** Planned
**Verification:** Playwright performance test asserts render time <3000ms

### R-004: `npm run build` failure (Score: 6)

**Mitigation Strategy:** Add `astro check && npm run build` as mandatory CI step on every PR targeting `main`. Treat non-zero exit as a blocking gate failure.
**Owner:** Dev/QA
**Timeline:** Pre-merge (Story 7.1, ongoing)
**Status:** Planned
**Verification:** CI step in GitHub Actions workflow exits non-zero on any TypeScript or Astro error

---

## Assumptions and Dependencies

### Assumptions

1. All Epic 7 stories (7.1–7.4) will be implemented in the `site/` directory using Astro SSG with TypeScript strict mode.
2. `cal-heatmap@4.2.4` (already pinned in `package.json`) will remain the bundled library — no CDN runtime dependency.
3. Client-side `fetch` in `u/index.astro` will use `window.location.pathname` to derive the username, consistent with the `_redirects` catch-all pattern.
4. No per-user server-side rendering — all pages are static shells; data is fetched client-side.
5. The `vibestats.dev/install.sh` URL is a fixed constant at the time Epic 7 is implemented (Epic 8 dependency).

### Dependencies

1. `data.json` public schema (from Epic 5, Stories 5.1–5.2) — required by Story 7.2 dashboard implementation; schema must be stable before dashboard fetch logic is tested
2. Playwright test infrastructure — must be bootstrapped (via `bmad-testarch-framework`) before E2E tests can run
3. Epic 8 (Cloudflare Pages deploy workflow, Story 8.2) — required before production performance and routing can be validated end-to-end

### Risks to Plan

- **Risk:** Cloudflare Pages routing may behave differently from `_redirects` local simulation
  - **Impact:** `/username` routing may not map to `u/index.html` as expected
  - **Contingency:** Test `_redirects` ordering in build; validate with Cloudflare Pages preview deployment before marking Epic 7 done

- **Risk:** `cal-heatmap` API changes between 4.2.4 and a future version during development
  - **Impact:** Heatmap rendering breaks silently
  - **Contingency:** Version is pinned in `package.json`; do not upgrade without a dedicated test run

---

## Interworking & Regression

| Service/Component | Impact | Regression Scope |
| --- | --- | --- |
| Epic 5 `data.json` schema | Dashboard reads `generated_at`, `username`, `days` fields | If field names change, dashboard fetch logic breaks; assert schema contract in Epic 7 component tests |
| Story 1.3 Astro init (`_redirects`, `astro.config.mjs`) | Routing and build config already established | Run `npm run build` on every PR to catch config regressions; do not modify `_redirects` without a routing test update |
| Epic 8 Cloudflare Pages deploy | Hosting and CDN cache for static assets | Epic 7 tests validate build output only; production routing validated post-deploy in Epic 8 |

---

## Follow-on Workflows (Manual)

- Run `*atdd` to generate failing P0 tests for Story 7.2 (client-side fetch flow) before implementation begins.
- Run `*framework` to scaffold Playwright configuration in `site/` before writing E2E/component tests.
- Run `*automate` for broader coverage once all four stories are implemented.

---

## Approval

**Test Design Approved By:**

- [ ] Product Manager: Leo  Date: ___
- [ ] Tech Lead: Leo  Date: ___
- [ ] QA Lead: Leo  Date: ___

**Comments:**

---

## Appendix

### Knowledge Base References

- `risk-governance.md` — Risk classification framework
- `probability-impact.md` — Risk scoring methodology
- `test-levels-framework.md` — Test level selection
- `test-priorities-matrix.md` — P0–P3 prioritisation

### Related Documents

- PRD: `_bmad-output/planning-artifacts/prd.md` (FR27–31, FR43, NFR4)
- Epic: `_bmad-output/planning-artifacts/epics.md` (Epic 7, lines 848–955)
- Architecture: `_bmad-output/planning-artifacts/architecture.md` (Frontend Architecture, Routing, Hosting sections)
- Sprint Status: `_bmad-output/implementation-artifacts/sprint-status.yaml`

---

**Generated by**: BMad TEA Agent - Test Architect Module
**Workflow**: `bmad-testarch-test-design`
**Version**: 4.0 (BMad v6)
