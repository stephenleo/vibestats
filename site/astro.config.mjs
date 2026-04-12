// @ts-check
import { defineConfig } from 'astro/config';
import tailwindcss from '@tailwindcss/vite';

// https://astro.build/config
export default defineConfig({
  site: 'https://vibestats.dev',
  trailingSlash: 'never',
  compressHTML: false,
  build: {
    // Generate pages as standalone .html files (e.g. dist/u.html) instead of
    // directory indexes (dist/u/index.html).  Cloudflare treats directory indexes
    // as having a canonical URL (/u/), which causes it to redirect any 200-rewrite
    // pointing there — changing the browser URL from /<username> to /u/.
    // Standalone files have no such canonical redirect.
    format: 'file',
  },
  vite: {
    plugins: [tailwindcss()],
  },
});
