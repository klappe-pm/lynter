// Renders og-card.png and apple-touch-icon.png from the HTML templates. Run: node scripts/brand.cjs
const {chromium} = require('playwright');
const path = require('node:path');
(async () => {
  const b = await chromium.launch();
  const out = path.join(__dirname, '..', 'public', 'brand');
  let p = await b.newPage({viewport: {width: 1200, height: 630}});
  await p.goto('file://' + path.join(__dirname, 'brand.html'));
  await p.evaluate(() => document.fonts.ready);
  await p.screenshot({path: path.join(out, 'og-card.png')});
  p = await b.newPage({viewport: {width: 180, height: 180}});
  await p.goto('file://' + path.join(__dirname, 'icon.html'));
  await p.screenshot({path: path.join(out, 'apple-touch-icon.png')});
  await b.close();
})();
