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

// 2) densify the trailing window with REALISTIC sparsity: nobody codes every
//    day. Sundays off, most Saturdays off, ~20% of weekdays off too. Days that
//    survive get boosted (heavier on the last two weeks). Today always stays so
//    the edge of the heatmap and the streak aren't empty.
const hash = (s) => [...s].reduce((a, c) => a + c.charCodeAt(0), 0);
const active = (key) => {
  const dow = new Date(key).getUTCDay();
  if (dow === 0) return false;                 // Sundays: off
  if (dow === 6 && hash(key) % 3 !== 0) return false; // Saturdays: mostly off
  return hash(key) % 10 >= 2;                   // ~20% of weekdays: off too
};
const templates = Object.values(days).map((d) => JSON.parse(JSON.stringify(d)));
for (let i = 0; i <= 34; i++) { // i = days before today
  const key = iso(addDays(TODAY, -i));
  if (i > 0 && !active(key)) { delete days[key]; continue; }
  if (!days[key]) days[key] = JSON.parse(JSON.stringify(templates[i * 5 % templates.length]));
  scale(days[key], i < 14 ? 1.5 : 1.25); // heavier on the last two weeks
}

doc.days = days;
doc.generated_at = `${TODAY}T12:00:00Z`;
writeFileSync(FIXTURE, JSON.stringify(doc, null, 2) + '\n');
const out = Object.keys(days).sort();
console.log(`shifted +${delta}d · ${out.length} days · ${out[0]} → ${out[out.length - 1]}`);
