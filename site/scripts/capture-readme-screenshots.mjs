#!/usr/bin/env node
// One-shot Playwright capture for the README screenshots.
// Spawns `astro dev`, intercepts the public-data.json fetch with a local
// fixture, then captures four regions in both light and dark themes.
//
// Re-run when the dashboard UI changes materially:
//   node scripts/capture-readme-screenshots.mjs

import { spawn } from 'node:child_process';
import { mkdir, readFile } from 'node:fs/promises';
import { existsSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { chromium } from 'playwright';

const __dirname = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = resolve(__dirname, '..', '..');
const SITE_DIR = join(REPO_ROOT, 'site');
const FIXTURE_PATH = join(SITE_DIR, 'public', 'fixtures', 'screenshot-data.json');
const OUT_DIR = join(REPO_ROOT, 'docs', 'images');
const FIXTURE_USERNAME = 'vibestats-demo';
const PORT = 4321;
const VIEWPORT = { width: 1280, height: 800 };

async function waitForServer(url, attempts = 60) {
  for (let i = 0; i < attempts; i++) {
    try {
      const r = await fetch(url);
      if (r.ok) return;
    } catch { /* keep trying */ }
    await new Promise((r) => setTimeout(r, 500));
  }
  throw new Error(`Server did not start at ${url}`);
}

async function captureRegion(page, selector, outPath) {
  const handle = await page.locator(selector).first();
  await handle.waitFor({ state: 'visible', timeout: 10_000 });
  await handle.scrollIntoViewIfNeeded();
  await page.waitForTimeout(500); // let cal-heatmap settle
  await handle.screenshot({ path: outPath });
  console.log(`✓ ${outPath}`);
}

async function captureViewport(page, outPath) {
  await page.waitForTimeout(500);
  await page.screenshot({ path: outPath, fullPage: false });
  console.log(`✓ ${outPath}`);
}

async function main() {
  if (!existsSync(FIXTURE_PATH)) {
    throw new Error(`Fixture not found at ${FIXTURE_PATH}`);
  }
  await mkdir(OUT_DIR, { recursive: true });

  const fixtureBody = await readFile(FIXTURE_PATH, 'utf8');

  console.log('Starting astro dev…');
  const dev = spawn('npm', ['run', 'dev', '--', '--port', String(PORT)], {
    cwd: SITE_DIR,
    stdio: ['ignore', 'pipe', 'inherit'],
  });
  dev.stdout.on('data', (b) => process.stdout.write(b));

  try {
    await waitForServer(`http://localhost:${PORT}/`);

    const browser = await chromium.launch();

    // Each theme gets its own context so the theme class is set BEFORE any
    // page script runs (no toggling — toggling caused cal-heatmap to render
    // twice because the post-load repaint stacked on top of the initial one).
    for (const theme of ['light', 'dark']) {
      console.log(`\n— Theme: ${theme} —`);
      // colorScheme drives Base.astro's anti-flash script, which adds the
      // .dark class on the html element when the OS prefers dark.
      const context = await browser.newContext({ viewport: VIEWPORT, colorScheme: theme });

      const page = await context.newPage();

      // Intercept the GitHub data.json fetch and serve the local fixture.
      await page.route(/raw\.githubusercontent\.com\/.*\/vibestats\/data\.json.*/, (route) => {
        route.fulfill({
          status: 200,
          contentType: 'application/json',
          headers: { 'access-control-allow-origin': '*' },
          body: fixtureBody,
        });
      });

      // Astro dev does not honor Cloudflare _redirects, so /<username> 404s.
      // Proxy the dashboard shell from /u while keeping the URL bar pointed at
      // /<username> so the page's inline username-extraction logic still works.
      await page.route(`http://localhost:${PORT}/${FIXTURE_USERNAME}`, async (route) => {
        const upstream = await fetch(`http://localhost:${PORT}/u`);
        const body = await upstream.text();
        await route.fulfill({
          status: 200,
          contentType: upstream.headers.get('content-type') || 'text/html; charset=utf-8',
          body,
        });
      });

      await page.goto(`http://localhost:${PORT}/${FIXTURE_USERNAME}`, { waitUntil: 'domcontentloaded' });
      // Wait for the heatmap to render at least one cell.
      await page.waitForSelector('#cal-heatmap svg', { timeout: 15_000 });
      await page.waitForTimeout(800);

      await captureViewport(page, join(OUT_DIR, `dashboard-hero-${theme}.png`));
      await captureRegion(page, '.card.full-width', join(OUT_DIR, `dashboard-heatmap-${theme}.png`));
      await captureRegion(page, '#kpi-row', join(OUT_DIR, `dashboard-kpis-${theme}.png`));
      await captureRegion(page, '.grid-2', join(OUT_DIR, `dashboard-charts-${theme}.png`));

      await context.close();
    }

    await browser.close();
    console.log('\nAll screenshots captured.');
  } finally {
    dev.kill('SIGTERM');
  }
}

main().catch((err) => {
  console.error(err);
  process.exitCode = 1;
});
