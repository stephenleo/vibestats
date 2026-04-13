// Cloudflare Pages edge middleware.
// Routes vibestats.dev/<username> to the dashboard (u.html)
// while keeping the original URL so client JS can extract the username.
export default {
  async fetch(request, env) {
    const url = new URL(request.url);

    // Try serving the static asset first (index.html, docs/*, _astro/*, etc.)
    const staticResponse = await env.ASSETS.fetch(request);
    if (staticResponse.status !== 404) {
      return staticResponse;
    }

    // No static file matched.  Single path segment without a file extension
    // is treated as a GitHub username → serve the dashboard shell.
    const segments = url.pathname.split('/').filter(Boolean);
    if (segments.length === 1 && !segments[0].includes('.')) {
      const assetUrl = new URL('/u.html', url.origin);
      const response = await env.ASSETS.fetch(assetUrl);
      // Override cache headers — the CDN must not edge-cache username routes
      // or redeployments won't take effect until the long TTL expires.
      const headers = new Headers(response.headers);
      headers.set('Cache-Control', 'public, max-age=0, must-revalidate');
      headers.delete('CDN-Cache-Control');
      return new Response(response.body, {
        status: 200,
        headers,
      });
    }

    // Everything else gets the original 404.
    return staticResponse;
  },
};
