import assert from 'node:assert/strict';
import { existsSync } from 'node:fs';
import test from 'node:test';

import worker from '../public/_worker.js';

test('routes dashboards, trust pages, and unknown usernames correctly', async () => {
  const requestedAssets = [];
  const env = {
    ASSETS: {
      async fetch(request) {
        const path = new URL(request.url).pathname;
        requestedAssets.push(path);
        return new Response(`asset:${path}`, { status: path === '/404' ? 404 : 200 });
      },
    },
  };
  const originalFetch = globalThis.fetch;

  try {
    let dashboardChecks = 0;
    globalThis.fetch = async () => {
      dashboardChecks += 1;
      return new Response(null, { status: 200 });
    };
    const dashboard = await worker.fetch(new Request('https://vibestats.dev/stephenleo'), env);
    assert.equal(dashboard.status, 200);
    assert.equal(requestedAssets.pop(), '/u');

    for (const path of [
      '/404',
      '/contact',
      '/privacy',
      '/robots.txt',
      '/security',
      '/sitemap.xml',
      '/u',
    ]) {
      const response = await worker.fetch(new Request(`https://vibestats.dev${path}`), env);
      assert.equal(response.status, path === '/404' ? 404 : 200);
      assert.equal(requestedAssets.pop(), path);
    }
    assert.equal(dashboardChecks, 1);

    globalThis.fetch = async () => new Response(null, { status: 404 });
    const missing = await worker.fetch(
      new Request('https://vibestats.dev/not-a-real-dashboard'),
      env
    );
    assert.equal(missing.status, 404);
    assert.equal(requestedAssets.pop(), '/404');
    assert.equal(await missing.text(), 'asset:/404');
    assert.equal(missing.headers.get('x-content-type-options'), 'nosniff');
  } finally {
    globalThis.fetch = originalFetch;
  }
});

test('does not ship Pages redirects ahead of worker routing', () => {
  assert.equal(existsSync(new URL('../public/_redirects', import.meta.url)), false);
});
