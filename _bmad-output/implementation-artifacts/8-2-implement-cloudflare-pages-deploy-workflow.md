# Story 8.2: Implement Cloudflare Pages Deploy Workflow

**GH Issue:** #40

Status: ready-for-dev

## Story

As a developer deploying vibestats.dev,
I want a manually-triggered GitHub Actions workflow that deploys the Astro site to Cloudflare Pages,
so that I control exactly which version is live in production.

## Acceptance Criteria

1. **Given** `deploy-site.yml` exists in `.github/workflows/`  
   **When** it is reviewed  
   **Then** it is triggered only via `workflow_dispatch` with a `ref` input (branch or tag) — no automatic triggers

2. **Given** the workflow is dispatched  
   **When** it runs  
   **Then** it checks out the specified ref, runs `npm run build` inside `site/`, and deploys to Cloudflare Pages using `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` secrets

3. **Given** the build step fails  
   **When** the workflow exits  
   **Then** no deployment to Cloudflare occurs (build gates deploy)

## Tasks / Subtasks

- [ ] Create `.github/workflows/deploy-site.yml` (AC: 1, 2, 3)
  - [ ] Add `workflow_dispatch` trigger with `ref` input (branch or tag name, default `main`)
  - [ ] Add checkout step using specified ref
  - [ ] Add Node.js setup step (Node >= 22.12.0 per `site/package.json` engines field)
  - [ ] Add npm install step inside `site/` working directory
  - [ ] Add `npm run build` step inside `site/` working directory
  - [ ] Add Cloudflare Pages deploy step using `cloudflare/pages-action@v1` with `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` secrets
  - [ ] Ensure deploy step comes AFTER build step so build failure prevents deploy (AC: 3)
- [ ] Verify no automatic triggers (push/pull_request/schedule) are present (AC: 1)
- [ ] Confirm workflow file is at correct path: `.github/workflows/deploy-site.yml` (not inside `site/`)

## Dev Notes

### File to Create

**Single new file:** `.github/workflows/deploy-site.yml`

The ONLY deliverable for this story. Do not modify any existing files.

### Exact File Location

```
vibestats/
  .github/
    workflows/
      aggregate.yml      ← existing, do not touch
      deploy-site.yml    ← CREATE THIS (Cloudflare Pages manual deploy)
      release.yml        ← will be created in story 8.1, does not exist yet
```

[Source: architecture.md#Monorepo layout]

### Trigger Specification

```yaml
on:
  workflow_dispatch:
    inputs:
      ref:
        description: 'Branch or tag to deploy'
        required: true
        default: 'main'
```

- `workflow_dispatch` ONLY — no `push`, no `schedule`, no `pull_request`, no `release` triggers
- The `ref` input is `${{ github.event.inputs.ref }}` — use this exact expression in the checkout step

[Source: epics.md#Story 8.2 AC1] [Source: architecture.md#Infrastructure & Deployment] [Source: test-design-epic-8.md R-003]

### Build Command

- Working directory: `site/`
- Install: `npm ci` (or `npm install`)
- Build: `npm run build` (defined in `site/package.json` → calls `astro build`)
- Output directory: `site/dist/` (Astro default, confirmed by `site/dist/` existing in repo)
- Node version: `>=22.12.0` (enforced in `site/package.json` engines field)

[Source: site/package.json]

### Cloudflare Pages Deploy

Use the official Cloudflare Pages action. The recommended action as of 2026 is `cloudflare/wrangler-action` (the older `cloudflare/pages-action` is deprecated). Use whichever is current at implementation time — check [https://github.com/cloudflare/wrangler-action](https://github.com/cloudflare/wrangler-action) for latest version.

Example using `cloudflare/wrangler-action`:

```yaml
- uses: cloudflare/wrangler-action@v3
  with:
    apiToken: ${{ secrets.CLOUDFLARE_API_TOKEN }}
    accountId: ${{ secrets.CLOUDFLARE_ACCOUNT_ID }}
    command: pages deploy site/dist --project-name=vibestats
```

Alternatively with `cloudflare/pages-action@v1` (older, still functional):

```yaml
- uses: cloudflare/pages-action@v1
  with:
    apiToken: ${{ secrets.CLOUDFLARE_API_TOKEN }}
    accountId: ${{ secrets.CLOUDFLARE_ACCOUNT_ID }}
    projectName: vibestats   # Cloudflare Pages project name (verify exact name in Cloudflare dashboard)
    directory: site/dist      # Astro build output relative to repo root
    gitHubToken: ${{ secrets.GITHUB_TOKEN }}
```

Required secrets (already referenced in architecture):
- `CLOUDFLARE_API_TOKEN` — Cloudflare API token with Pages edit permission
- `CLOUDFLARE_ACCOUNT_ID` — Cloudflare account ID

**projectName note:** The Cloudflare Pages project name must match the project name registered in the Cloudflare dashboard. It is likely `vibestats` but verify in the dashboard — do not guess.

[Source: architecture.md#Infrastructure & Deployment: "Requires `CLOUDFLARE_API_TOKEN` + `CLOUDFLARE_ACCOUNT_ID` as repo secrets"]

### Build Gates Deploy — Critical AC3

Steps MUST be sequential (not parallel):
1. checkout
2. setup-node
3. npm install
4. npm run build ← if this fails, workflow exits non-zero here
5. cloudflare deploy ← only reached if build succeeds

Do NOT use `continue-on-error: true` on the build step.

### Style Reference — Existing Workflow

Refer to `.github/workflows/aggregate.yml` for YAML style (name casing, indentation). Use `ubuntu-latest` as the runner (consistent with existing workflow).

### Action Version Pinning (Critical)

ALL `uses:` references MUST be pinned to semver version tags — never `@main` or `@master`.

Minimum required:
- `actions/checkout@v4`
- `actions/setup-node@v4`
- Cloudflare action at a pinned tag (e.g., `cloudflare/wrangler-action@v3` or `cloudflare/pages-action@v1`)

[Source: test-design-epic-8.md P1 table — "all `uses:` action references are pinned to version tags"]

### Checkout Step

```yaml
- uses: actions/checkout@v4
  with:
    ref: ${{ github.event.inputs.ref }}
```

Use `${{ github.event.inputs.ref }}` exactly — not `${{ inputs.ref }}` (the longer form is required for `workflow_dispatch`).

[Source: test-design-epic-8.md P1 — "checkout step uses `${{ github.event.inputs.ref }}`"]

### Node.js Setup

```yaml
- uses: actions/setup-node@v4
  with:
    node-version: '22'
    cache: 'npm'
    cache-dependency-path: site/package-lock.json
```

### Anti-Patterns to Avoid

- Do NOT add `push` or `schedule` triggers — this must be manual-only
- Do NOT set `working-directory` at the job level — set it per-step or use `cd site &&` in run commands to be explicit
- Do NOT reference `site/dist` as the Cloudflare directory with a leading slash
- Do NOT create any other files — this story is one workflow YAML only
- Do NOT modify `astro.config.mjs` — build output is already `dist/` by Astro default

### Project Structure Notes

- Workflow file lives at `.github/workflows/deploy-site.yml` (repo root `.github/`, not inside `site/`)
- Astro build output: `site/dist/` — Astro SSG default, confirmed by existing `site/dist/` directory
- `site/public/_redirects` contains Cloudflare URL rewrite rules for `/:username` → `/u/index.html 200` — this is bundled by Astro into `dist/` automatically, no extra action needed

[Source: architecture.md#Monorepo layout] [Source: epics.md line 135]

### Testing Requirements (from test-design-epic-8.md)

The test plan defines schema/unit tests that will validate the YAML file after implementation. Write the workflow so it passes these assertions:

**P0 (must pass before merge):**
- `on` key contains ONLY `workflow_dispatch` — no `push`, `pull_request`, `schedule`, or `release`
- `CLOUDFLARE_API_TOKEN` and `CLOUDFLARE_ACCOUNT_ID` referenced by exact names — no hardcoded values, no misspellings
- Build step (`npm run build`) precedes the deploy step; build step has no `continue-on-error: true`

**P1 (should pass before merge):**
- `workflow_dispatch.inputs.ref` declared
- Checkout step uses `${{ github.event.inputs.ref }}` — not hardcoded branch
- All `uses:` action refs pinned to semver tags (e.g., `@v4`) — not `@main` or `@master`

**P2 (nice to have):**
- Checkout or run steps use `working-directory: site` (or `cd site`) before `npm run build`

[Source: test-design-epic-8.md — R-003, R-004, R-008 + P0/P1/P2 test tables]

### References

- [Source: epics.md#Story 8.2] — Story AC and description
- [Source: architecture.md#Infrastructure & Deployment] — Cloudflare Pages hosting, secrets, manual dispatch pattern
- [Source: architecture.md#Monorepo layout] — `deploy-site.yml` exact path
- [Source: site/package.json] — `npm run build` script, Node engine requirement
- [Source: .github/workflows/aggregate.yml] — YAML style reference
- [Source: test-design-epic-8.md] — R-003 (trigger), R-004 (secrets), R-008 (build gates deploy)

## Dev Agent Record

### Agent Model Used

claude-sonnet-4-6

### Debug Log References

### Completion Notes List

### File List
