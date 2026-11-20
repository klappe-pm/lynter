// Node harness for the committed WASM build: reads a JSON array of
// {config, note, path, today} from stdin, writes the lint outcomes as JSON.
import fs from 'node:fs';
import {fileURLToPath} from 'node:url';
import path from 'node:path';

const dir = path.join(path.dirname(fileURLToPath(import.meta.url)), '..', 'public', 'wasm');
const mod = await import(path.join(dir, 'lynter_wasm.mjs'));
mod.initSync({module: fs.readFileSync(path.join(dir, 'lynter_wasm_bg.wasm'))});
const cases = JSON.parse(fs.readFileSync(0, 'utf8'));
const out = cases.map((c) => JSON.parse(mod.lint(c.config, c.note, c.path, c.today)));
process.stdout.write(JSON.stringify(out));
