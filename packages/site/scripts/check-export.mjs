// Checks the static export in out/ before it is deployed: every internal link
// and asset reference resolves to a file, every page is in sitemap.xml and the
// sitemap lists no page that does not exist, and every page carries an
// absolute canonical link and social card. Exits 1 with one line per problem.
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';

const out = path.join(path.dirname(fileURLToPath(import.meta.url)), '..', 'out');
const origin = 'https://lynter.dev';
const problems = [];

/** Every index.html under out/, as its route ("/", "/docs/"). The 404 page is not a route. */
function routes(dir = out, prefix = '/') {
  return fs.readdirSync(dir, {withFileTypes: true}).flatMap((entry) => {
    if (entry.isDirectory() && !['_next', '404'].includes(entry.name)) {
      return routes(path.join(dir, entry.name), `${prefix}${entry.name}/`);
    }
    return entry.name === 'index.html' ? [prefix] : [];
  });
}

/** Whether an internal URL path resolves to a file in the export. */
function exists(urlPath) {
  const clean = decodeURIComponent(urlPath.split(/[?#]/)[0]);
  const file = path.join(out, clean.endsWith('/') ? `${clean}index.html` : clean);
  return fs.existsSync(file) && fs.statSync(file).isFile();
}

if (!fs.existsSync(out)) {
  console.error('out/ is missing: run the site build first');
  process.exit(1);
}

const pages = routes().sort();
for (const route of pages) {
  const html = fs.readFileSync(path.join(out, route, 'index.html'), 'utf8');
  for (const match of html.matchAll(/(?:href|src)="([^"]+)"/g)) {
    const ref = match[1].replace(/&amp;/g, '&');
    if (!ref.startsWith('/') || ref.startsWith('//')) continue;
    if (!exists(ref)) problems.push(`${route}: broken reference ${ref}`);
  }
  const canonical = html.match(/<link rel="canonical" href="([^"]+)"/)?.[1];
  if (canonical !== origin + route) problems.push(`${route}: canonical is ${canonical}, expected ${origin + route}`);
  const card = html.match(/<meta property="og:image" content="([^"]+)"/)?.[1];
  if (!card?.startsWith(`${origin}/`)) problems.push(`${route}: og:image is not an absolute ${origin} URL: ${card}`);
  else if (!exists(card.slice(origin.length))) problems.push(`${route}: og:image file is missing: ${card}`);
}

const sitemap = fs.readFileSync(path.join(out, 'sitemap.xml'), 'utf8');
const listed = [...sitemap.matchAll(/<loc>([^<]+)<\/loc>/g)].map((m) => m[1]);
for (const route of pages) {
  if (!listed.includes(origin + route)) problems.push(`sitemap.xml: missing ${origin + route}`);
}
for (const loc of listed) {
  if (!loc.startsWith(`${origin}/`) || !pages.includes(loc.slice(origin.length))) problems.push(`sitemap.xml: lists ${loc}, which is not a page`);
}
for (const file of ['robots.txt', 'fonts/geist-license.txt', 'fonts/jost-license.txt','wasm/lynter_wasm_bg.wasm', '404.html', 'lynter-config-builder.html']) {
  if (!exists(`/${file}`)) problems.push(`missing ${file}`);
}

for (const problem of problems) console.error(problem);
console.log(`checked ${pages.length} pages, ${problems.length} problems`);
process.exit(problems.length ? 1 : 0);
