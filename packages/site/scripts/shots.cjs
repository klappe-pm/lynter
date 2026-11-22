// Screenshots every route at 375 and 1280, light and dark; reports overflow and console errors.
const {chromium} = require('playwright');
const http = require('node:http'); const fs = require('node:fs'); const path = require('node:path');
const root = path.join(__dirname, '..', 'out');
const types = {'.html': 'text/html', '.css': 'text/css', '.js': 'text/javascript', '.svg': 'image/svg+xml', '.png': 'image/png', '.woff2': 'font/woff2', '.json': 'application/json'};
const srv = http.createServer((q, r) => {
  let p = decodeURIComponent(q.url.split('?')[0]); if (p.endsWith('/')) p += 'index.html';
  const f = path.join(root, p);
  if (!fs.existsSync(f)) { r.statusCode = 404; return r.end(); }
  r.setHeader('content-type', types[path.extname(f)] || 'application/octet-stream'); r.end(fs.readFileSync(f));
});
const routes = ['/', '/getting-started/', '/docs/', '/examples/', '/use-cases/', '/integrations/llm/', '/integrations/ci/', '/samples/', '/faq/', '/languages/', '/obsidian/', '/roadmap/', '/playground/', '/playground/?example=client-formal', '/import/', '/nope/'];
const outDir = process.argv[2];
(async () => {
  await new Promise((r) => srv.listen(4399, r));
  const b = await chromium.launch(); let bad = 0;
  for (const scheme of ['light', 'dark']) for (const w of [375, 1280]) {
    const ctx = await b.newContext({viewport: {width: w, height: 800}, colorScheme: scheme});
    for (const route of routes) {
      const p = await ctx.newPage(); const errs = [];
      p.on('console', (m) => { if (m.type() === 'error' && !(route === '/nope/' && /404/.test(m.text()))) errs.push(m.text()); });
      p.on('pageerror', (e) => errs.push(String(e)));
      await p.goto('http://localhost:4399' + route); await p.waitForTimeout(250);
      const ov = await p.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
      if (ov > 0 || errs.length) { bad++; console.log('FAIL', scheme, w, route, 'overflow', ov, errs.join('|')); }
      if (outDir && ['/', '/examples/', '/languages/'].includes(route) && ((scheme === 'light' && w === 1280) || (scheme === 'dark' && w === 375 && route === '/') )) await p.screenshot({path: `${outDir}/site-v2-${route.replace(/\//g, '') || 'home'}-${scheme}-${w}.png`, fullPage: false});
      await p.close();
    }
    await ctx.close();
  }
  await b.close(); srv.close(); console.log('done, failures:', bad);
})();
