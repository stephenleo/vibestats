#!/usr/bin/env node
// One-shot: shift the screenshot fixture forward so it ends at "today", then
// fill + boost the most recent weeks so the heatmap/KPIs read dense on the
// latest days (a static fixture goes blank once wall-clock passes its last day).
//   node scripts/densify-fixture.mjs
import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const FIXTURE = join(dirname(fileURLToPath(import.meta.url)), '..', 'public', 'fixtures', 'screenshot-data.json');
const TODAY = '2026-07-01'; // matches the capture machine's clock; edit if re-running much later
const iso = (d) => d.toISOString().slice(0, 10);
const addDays = (dateStr, n) => { const d = new Date(dateStr); d.setUTCDate(d.getUTCDate() + n); return d; };

// Seeded PRNG (mulberry32) — reproducible re-runs, but irregular enough that
// the gaps and intensities don't fall into a visible weekly grid.
let _seed = 0x1a2b3c4d;
const rand = () => {
  _seed = (_seed + 0x6d2b79f5) | 0;
  let t = Math.imul(_seed ^ (_seed >>> 15), 1 | _seed);
  t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
  return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
};

const scale = (day, f) => {
  for (const k of ['sessions', 'active_minutes', 'input_tokens', 'output_tokens',
    'cache_read_tokens', 'cache_creation_tokens', 'message_count', 'tool_uses', 'longest_session_minutes']) {
    if (typeof day[k] === 'number') day[k] = Math.max(1, Math.round(day[k] * f));
  }
  for (const m of Object.keys(day.models ?? {})) day.models[m] = Math.round(day.models[m] * f);
  for (const h of Object.values(day.harnesses ?? {})) {
    for (const k of Object.keys(h)) if (typeof h[k] === 'number') h[k] = Math.round(h[k] * f);
  }
};

const doc = JSON.parse(readFileSync(FIXTURE, 'utf8'));
const keys = Object.keys(doc.days).sort();
const delta = Math.round((new Date(TODAY) - new Date(keys[keys.length - 1])) / 864e5);

// 1) shift every existing day forward so the series ends on TODAY.
const days = {};
for (const k of keys) days[iso(addDays(k, delta))] = doc.days[k];

// A static fixture must not show future days, so drop anything past today.
for (const k of Object.keys(days)) if (k > TODAY) delete days[k];

// Jitter every surviving day's intensity so the heatmap has real day-to-day
// colour variation instead of flat bands.
for (const k of Object.keys(days)) scale(days[k], 0.75 + rand() * 0.6);

// 2) densify the trailing window with REALISTIC, irregular sparsity: nobody
//    codes every day. Activity is drawn per-day against a weekday-weighted
//    probability (weekends far less likely), so gaps land unevenly rather than
//    on a clean weekly grid. Surviving days get a boost with a wide random
//    multiplier (heavier on the last two weeks). Today always stays so the edge
//    of the heatmap and the streak aren't empty.
const activeProb = [0.12, 0.9, 0.82, 0.88, 0.82, 0.78, 0.35]; // Sun..Sat
const templates = Object.values(days).map((d) => JSON.parse(JSON.stringify(d)));
for (let i = 0; i <= 34; i++) { // i = days before today
  const key = iso(addDays(TODAY, -i));
  const on = rand() < activeProb[new Date(key).getUTCDay()];
  if (i > 0 && !on) { delete days[key]; continue; }
  if (!days[key]) days[key] = JSON.parse(JSON.stringify(templates[Math.floor(rand() * templates.length)]));
  scale(days[key], (i < 14 ? 1.5 : 1.2) * (0.55 + rand())); // 0.55–1.55× spread
}

doc.days = days;
doc.generated_at = `${TODAY}T12:00:00Z`;
writeFileSync(FIXTURE, JSON.stringify(doc, null, 2) + '\n');
const out = Object.keys(days).sort();
console.log(`shifted +${delta}d · ${out.length} days · ${out[0]} → ${out[out.length - 1]}`);
