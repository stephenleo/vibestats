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
const REPO_ROOT = resolve(__dirname, '..');
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

async function setTheme(page, theme) {
  await page.evaluate((t) => {
    document.documentElement.classList.toggle('dark', t === 'dark');
    try { localStorage.setItem('theme', t); } catch { /* ignore */ }
  }, theme);
  // Trigger the dashboard's theme-toggle handler so cal-heatmap repaints.
  await page.evaluate(() => {
    document.getElementById('theme-toggle')?.dispatchEvent(new MouseEvent('click', { bubbles: true }));
  });
  await page.waitForTimeout(800);
  // Force back to the requested theme in case the click toggled it the other way.
  await page.evaluate((t) => {
    document.documentElement.classList.toggle('dark', t === 'dark');
  }, theme);
  await page.waitForTimeout(400);
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
    const context = await browser.newContext({ viewport: VIEWPORT });
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

    for (const theme of ['light', 'dark']) {
      console.log(`\n— Theme: ${theme} —`);
      await page.goto(`http://localhost:${PORT}/${FIXTURE_USERNAME}`, { waitUntil: 'networkidle' });
      await setTheme(page, theme);
      // Wait for the heatmap to render at least one cell.
      await page.waitForSelector('#cal-heatmap svg', { timeout: 15_000 });
      await page.waitForTimeout(800);

      await captureViewport(page, join(OUT_DIR, `dashboard-hero-${theme}.png`));
      await captureRegion(page, '.card.full-width', join(OUT_DIR, `dashboard-heatmap-${theme}.png`));
      await captureRegion(page, '#kpi-row', join(OUT_DIR, `dashboard-kpis-${theme}.png`));
      await captureRegion(page, '.grid-2', join(OUT_DIR, `dashboard-charts-${theme}.png`));
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
