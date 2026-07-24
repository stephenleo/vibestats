import assert from 'node:assert/strict';
import test from 'node:test';

import worker from '../public/_worker.js';

test('routes dashboards, trust pages, and unknown usernames correctly', async () => {
  const requestedAssets = [];
  const env = {
    ASSETS: {
      async fetch(request) {
        requestedAssets.push(new URL(request.url).pathname);
        return new Response('asset');
      },
    },
  };
  const originalFetch = globalThis.fetch;

  try {
    globalThis.fetch = async () => new Response(null, { status: 200 });
    const dashboard = await worker.fetch(new Request('https://vibestats.dev/stephenleo'), env);
    assert.equal(dashboard.status, 200);
    assert.equal(requestedAssets.pop(), '/u');

    const privacy = await worker.fetch(new Request('https://vibestats.dev/privacy'), env);
    assert.equal(privacy.status, 200);
    assert.equal(requestedAssets.pop(), '/privacy');

    const sitemap = await worker.fetch(new Request('https://vibestats.dev/sitemap.xml'), env);
    assert.equal(sitemap.status, 200);
    assert.equal(requestedAssets.pop(), '/sitemap.xml');

    globalThis.fetch = async () => new Response(null, { status: 404 });
    const missing = await worker.fetch(
      new Request('https://vibestats.dev/not-a-real-dashboard'),
      env
    );
    assert.equal(missing.status, 404);
    assert.equal(requestedAssets.pop(), '/404');
    assert.equal(missing.headers.get('x-content-type-options'), 'nosniff');
  } finally {
    globalThis.fetch = originalFetch;
  }
});
