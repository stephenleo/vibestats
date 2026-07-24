const DASHBOARD_PATHS = new Set(['404', 'contact', 'privacy', 'security', 'u']);
const USERNAME_RE = /^[A-Za-z0-9](?:[A-Za-z0-9]|-(?=[A-Za-z0-9])){0,38}$/;
const SECURITY_HEADERS = {
  'Content-Security-Policy':
    "default-src 'self'; base-uri 'self'; connect-src 'self' https://raw.githubusercontent.com; font-src https://fonts.gstatic.com; form-action 'self'; frame-ancestors 'none'; img-src 'self' data: https://github.com https://avatars.githubusercontent.com; object-src 'none'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com; upgrade-insecure-requests",
  'Permissions-Policy': 'camera=(), geolocation=(), microphone=()',
  'Referrer-Policy': 'strict-origin-when-cross-origin',
  'Strict-Transport-Security': 'max-age=31536000',
  'X-Content-Type-Options': 'nosniff',
};

function withSecurityHeaders(response) {
  const secured = new Response(response.body, response);
  for (const [name, value] of Object.entries(SECURITY_HEADERS)) {
    secured.headers.set(name, value);
  }
  return secured;
}

async function dashboardExists(username) {
  try {
    const response = await fetch(
      `https://raw.githubusercontent.com/${username}/${username}/main/vibestats/data.json`,
      { method: 'HEAD', redirect: 'follow' }
    );
    return response.status !== 404;
  } catch {
    return true;
  }
}

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    const segments = url.pathname.split('/').filter(Boolean);
    const username = segments[0] ?? '';

    if (
      (request.method === 'GET' || request.method === 'HEAD') &&
      segments.length === 1 &&
      !DASHBOARD_PATHS.has(username) &&
      USERNAME_RE.test(username)
    ) {
      if (!(await dashboardExists(username))) {
        const notFound = await env.ASSETS.fetch(new Request(new URL('/404', url), request));
        return withSecurityHeaders(
          new Response(notFound.body, {
            status: 404,
            statusText: notFound.statusText,
            headers: notFound.headers,
          })
        );
      }
      return withSecurityHeaders(await env.ASSETS.fetch(new Request(new URL('/u', url), request)));
    }

    return withSecurityHeaders(await env.ASSETS.fetch(request));
  },
};
