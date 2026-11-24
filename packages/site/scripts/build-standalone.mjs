// Builds public/lynter-config-builder.html: the config builder as one file
// that runs from disk with no network. JavaScript, styles and the engine
// (gzipped, base64) are all inline. Run before `next build`, which then
// publishes it at https://lynter.dev/lynter-config-builder.html.
import {gzipSync} from 'node:zlib';
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import esbuild from 'esbuild';

const root = path.join(path.dirname(fileURLToPath(import.meta.url)), '..');
const out = path.join(root, 'public', 'lynter-config-builder.html');
const wasm = fs.readFileSync(path.join(root, 'public', 'wasm', 'lynter_wasm_bg.wasm'));

const result = await esbuild.build({
  entryPoints: [path.join(root, 'standalone', 'main.tsx')],
  bundle: true,
  write: false,
  minify: true,
  format: 'iife',
  target: 'es2020',
  jsx: 'automatic',
  tsconfig: path.join(root, 'tsconfig.json'),
  define: {
    'process.env.NODE_ENV': '"production"',
    'process.env.NEXT_PUBLIC_BUTTONDOWN_USER': '""',
    'import.meta.url': '""',
  },
  legalComments: 'none',
  plugins: [{
    name: 'lynter-wasm-gz',
    setup(build) {
      build.onResolve({filter: /^lynter-wasm-gz$/}, () => ({path: 'lynter-wasm-gz', namespace: 'lynter'}));
      build.onLoad({filter: /.*/, namespace: 'lynter'}, () => ({
        contents: `export default ${JSON.stringify(gzipSync(wasm, {level: 9}).toString('base64'))};`,
        loader: 'js',
      }));
    },
  }],
});

// Fonts are not embedded: the file falls back to the system font, which keeps it small.
// tokens.css is generated from DESIGN.md by ds sync and must come first.
const css = ['tokens.css', 'globals.css']
  .map((file) => fs.readFileSync(path.join(root, 'styles', file), 'utf8'))
  .join('\n')
  .replace(/@font-face\s*\{[^}]*\}\s*/g, '');
// A closing script tag inside the bundle would end the inline script early.
const js = result.outputFiles[0].text.replace(/<\/script/gi, '<\\/script');
const version = JSON.parse(fs.readFileSync(path.join(root, 'package.json'), 'utf8')).version;

fs.writeFileSync(out, `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Lynter config builder ${version}</title>
<meta name="description" content="Build a lynter.yaml from a form. This file works offline: nothing is uploaded.">
<style>${css}</style>
</head>
<body>
<div id="root"></div>
<script>${js}</script>
</body>
</html>
`);
const size = fs.statSync(out).size;
console.log(`wrote ${path.relative(process.cwd(), out)}: ${size} bytes, ${gzipSync(fs.readFileSync(out)).length} gzipped`);
if (size > 3_000_000) {
  console.error('standalone builder is over 3 MB');
  process.exit(1);
}
