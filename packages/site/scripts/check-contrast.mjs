// Checks text-on-background contrast for the site's colour names. The values
// come from styles/tokens.css (generated from DESIGN.md by ds sync) and the
// names from the :root aliases in styles/globals.css.
// Exits 1 when any pair is under 4.5:1. Run: node scripts/check-contrast.mjs
import {readFileSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
import path from 'node:path';

const styles = path.join(path.dirname(fileURLToPath(import.meta.url)), '..', 'styles');
const tokensCss = readFileSync(path.join(styles, 'tokens.css'), 'utf8');
const globalsCss = readFileSync(path.join(styles, 'globals.css'), 'utf8');
const MIN = 4.5;

/** Returns the declarations of the first block in css whose selector starts with the given text. */
function block(css, selector) {
  const start = css.indexOf(selector);
  if (start < 0) throw new Error(`selector not found: ${selector}`);
  const open = css.indexOf('{', start);
  return css.slice(open + 1, css.indexOf('}', open));
}

function declarations(body) {
  const out = {};
  for (const m of body.matchAll(/(--[\w-]+)\s*:\s*([^;]+);/g)) out[m[1]] = m[2].trim();
  return out;
}

const aliases = declarations(block(globalsCss, ':root {'));
const light = {...aliases, ...declarations(block(tokensCss, ':root {'))};
const dark = {...light, ...declarations(block(tokensCss, "[data-theme='dark'] {"))};
const media = {...light, ...declarations(block(tokensCss, ":root:not([data-theme='light'])"))};

/** Resolves var(--x) chains to a hex colour. */
function resolve(tokens, name, depth = 0) {
  const v = tokens[`--${name}`];
  if (v === undefined || depth > 8) throw new Error(`cannot resolve --${name}`);
  const ref = v.match(/^var\(--([\w-]+)\)$/);
  if (ref) return resolve(tokens, ref[1], depth + 1);
  if (!/^#[0-9a-f]{6}$/i.test(v)) throw new Error(`--${name} is not a hex colour: ${v}`);
  return v;
}

function luminance(hex) {
  const c = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255)
    .map((x) => (x <= 0.03928 ? x / 12.92 : ((x + 0.055) / 1.055) ** 2.4));
  return 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
}

function ratio(a, b) {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi + 0.05) / (lo + 0.05);
}

const pairs = [
  ['ink', 'bg'], ['ink', 'surface'], ['ink', 'code-bg'],
  ['muted', 'bg'], ['muted', 'code-bg'],
  ['accent', 'bg'],
  ['accent-ink', 'accent'],
  ['violation', 'code-bg'],
  ['changed-ink', 'changed'],
  ['code-ink', 'code-bg'],
];

let failed = false;
for (const [mode, tokens] of [['light', light], ['dark', dark], ['dark (media query)', media]]) {
  for (const [fg, bg] of pairs) {
    const r = ratio(resolve(tokens, fg), resolve(tokens, bg));
    const ok = r >= MIN;
    if (!ok) failed = true;
    console.log(`${ok ? 'pass' : 'FAIL'}  ${mode.padEnd(18)} ${fg} on ${bg}`.padEnd(54) + r.toFixed(2));
  }
}
if (failed) {
  console.error(`contrast check failed: a pair is under ${MIN}:1`);
  process.exit(1);
}
console.log(`contrast check passed: every pair is at least ${MIN}:1`);
