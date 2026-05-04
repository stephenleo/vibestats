// cal-heatmap 4.x ships typings at src/index.d.ts but its package.json
// `exports` field doesn't expose them, so TS can't resolve the declarations.
// Until that's fixed upstream, declare a minimal shim covering the surface
// we actually use (constructor, paint, destroy).
declare module 'cal-heatmap' {
  export default class CalHeatmap {
    paint(options?: unknown, plugins?: unknown): Promise<unknown>;
    destroy(): void;
  }
}

declare module 'cal-heatmap/plugins/Tooltip' {
  const Tooltip: unknown;
  export default Tooltip;
}

// Set in the inline early-fetch script in u.astro and read once the dashboard
// module loads. Stored as `unknown` so callers cast to the local data shape.
interface Window {
  __vibestatsData?: Promise<unknown>;
}
