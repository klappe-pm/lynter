// Smoke test for a deployed site: node scripts/smoke.mjs [origin].
// Fetches every page in sitemap.xml and the assets the playground needs,
// checks the 404 page and the redirect from http, and exits 1 on any failure.
// A CDN can lag a deploy, so each check is retried before it counts as failed.
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

const origin = (process.argv[2] || 'https://lynter.dev').replace(/\/$/, '');
const here = path.dirname(fileURLToPath(import.meta.url));
const sitemap = fs.readFileSync(path.join(here, '..', 'public', 'sitemap.xml'), 'utf8');
const pages = [...sitemap.matchAll(/<loc>https:\/\/lynter\.dev([^<]*)<\/loc>/g)].map((m) => m[1]);

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

/** Runs one check up to five times, ten seconds apart; returns the last failure or null. */
async function attempt(check) {
  let failure = null;
  for (let i = 0; i < 5; i++) {
    try {
      failure = await check();
    } catch (error) {
      failure = String(error.cause?.message || error.message || error);
    }
    if (!failure) return null;
    if (i < 4) await sleep(10000);
  }
  return failure;
}

const checks = [
  ...pages.map((route) => [route, async () => {
    const res = await fetch(origin + route, {redirect: 'manual'});
    if (res.status !== 200) return `status ${res.status}`;
    const html = await res.text();
    if (!/<title[^>]*>[^<]*Lynter[^<]*<\/title>/.test(html)) return 'no Lynter title';
    return html.includes('id="main"') ? null : 'no main landmark';
  }]),
  ...[['/wasm/lynter_wasm_bg.wasm', 'application/wasm'], ['/wasm/lynter_wasm.mjs', 'javascript'], ['/brand/og-card.png', 'image/png'], ['/sitemap.xml', 'xml'], ['/robots.txt', 'text/plain'], ['/fonts/geist-license.txt', 'text/plain'], ['/lynter-config-builder.html', 'text/html']].map(([file, type]) => [file, async () => {
    const res = await fetch(origin + file, {redirect: 'manual'});
    if (res.status !== 200) return `status ${res.status}`;
    const got = res.headers.get('content-type') || '';
    return got.includes(type) ? null : `content-type ${got}, expected ${type}`;
  }]),
  ['/no-such-page/', async () => {
    const res = await fetch(`${origin}/no-such-page/`, {redirect: 'manual'});
    return res.status === 404 ? null : `status ${res.status}, expected 404`;
  }],
];
if (origin.startsWith('https://')) {
  checks.push(['http redirect', async () => {
    const res = await fetch(origin.replace('https://', 'http://') + '/', {redirect: 'manual'});
    const to = res.headers.get('location') || '';
    return res.status === 301 && to.startsWith(origin) ? null : `status ${res.status}, location ${to}`;
  }]);
}

let failed = 0;
for (const [name, check] of checks) {
  const failure = await attempt(check);
  if (failure) failed++;
  console.log(`${failure ? 'FAIL' : 'ok  '} ${name}${failure ? `: ${failure}` : ''}`);
}
console.log(`${checks.length - failed} of ${checks.length} checks passed against ${origin}`);
process.exit(failed ? 1 : 0);
