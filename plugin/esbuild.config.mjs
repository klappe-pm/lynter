import esbuild from 'esbuild';
import {gzipSync} from 'node:zlib';
import {readFileSync, statSync, writeFileSync} from 'node:fs';
import {dirname, resolve, relative, sep} from 'node:path';
import {fileURLToPath} from 'node:url';

const MAX_BYTES = 1300000;
const here = dirname(fileURLToPath(import.meta.url));
const wasmDir = resolve(here, '../packages/site/public/wasm');
const glue = resolve(wasmDir, 'lynter_wasm.mjs');
const wasmFile = resolve(wasmDir, 'lynter_wasm_bg.wasm');

/** Resolves the WASM glue and an inline base64 of the gzipped binary. */
const wasmInline = {
  name: 'lynter-wasm-inline',
  setup(build) {
    build.onResolve({filter: /^lynter-wasm-glue$/}, () => ({path: glue}));
    build.onResolve({filter: /^lynter-wasm-b64$/}, () => ({path: 'lynter-wasm-b64', namespace: 'lynter-wasm'}));
    build.onLoad({filter: /.*/, namespace: 'lynter-wasm'}, () => {
      const gz = gzipSync(readFileSync(wasmFile), {level: 9});
      return {contents: `export default ${JSON.stringify(gz.toString('base64'))};`, loader: 'js'};
    });
  },
};

const result = await esbuild.build({
  entryPoints: [resolve(here, 'src/main.ts')],
  outfile: resolve(here, 'main.js'),
  bundle: true,
  format: 'cjs',
  platform: 'browser',
  target: 'es2020',
  external: ['obsidian', 'electron', '@codemirror/*', '@lezer/*'],
  define: {'import.meta.url': '""'},
  minify: true,
  legalComments: 'none',
  metafile: true,
  logLevel: 'info',
  plugins: [wasmInline],
});

const allowed = (input) => {
  const abs = resolve(input);
  const rel = relative(here, abs);
  if (!rel.startsWith('..' + sep) && rel !== '..') {
    if (!rel.startsWith('node_modules' + sep)) return true;
    return /^node_modules[\\/](obsidian|fflate)[\\/]/.test(rel);
  }
  return abs.startsWith(wasmDir + sep);
};
const inputs = Object.keys(result.metafile.inputs).filter((i) => !i.includes(':'));
const outside = inputs.filter((i) => !allowed(resolve(here, i)));
writeFileSync(resolve(here, 'metafile.json'), JSON.stringify(result.metafile));
console.log(`metafile inputs: ${inputs.length}, outside the allowed roots: ${outside.length}`);
if (outside.length > 0) {
  console.error('Unexpected inputs:\n' + outside.join('\n'));
  process.exit(1);
}

const size = statSync(resolve(here, 'main.js')).size;
console.log(`main.js size: ${size} bytes (limit ${MAX_BYTES})`);
if (size > MAX_BYTES) {
  console.error(`main.js exceeds ${MAX_BYTES} bytes`);
  process.exit(1);
}
