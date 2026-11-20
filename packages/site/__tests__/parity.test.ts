import {execFileSync, spawnSync} from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {examples} from '../src/examples';
import {sampleConfig, sampleNote, samplePath} from '../src/samples';

const repo = path.join(__dirname, '..', '..', '..');
const bin = path.join(repo, 'target', 'debug', 'lynter');
const today = '2026-01-01';
const vaults = ['starter-vault', 'team-handbook', 'agent-memory', 'docs-site'];

interface Case {name: string; config: string; note: string; path: string}

function notes(dir: string): string[] {
  return fs.readdirSync(dir, {withFileTypes: true}).flatMap((e) => {
    const p = path.join(dir, e.name);
    if (e.isDirectory()) return notes(p);
    return e.name.endsWith('.md') && e.name !== 'README.md' ? [p] : [];
  });
}

function cases(): Case[] {
  const out: Case[] = [{name: 'seeded sample', config: sampleConfig, note: sampleNote, path: samplePath}];
  for (const e of examples) out.push({name: `gallery ${e.id}`, config: e.config, note: e.note, path: e.path});
  for (const v of vaults) {
    const dir = path.join(repo, 'examples', v);
    const config = fs.readFileSync(path.join(dir, 'lynter.yaml'), 'utf8');
    for (const f of notes(dir)) {
      const rel = path.relative(dir, f).split(path.sep).join('/');
      out.push({name: `${v}/${rel}`, config, note: fs.readFileSync(f, 'utf8'), path: rel});
    }
  }
  return out;
}

// Parity needs the CLI built (cargo build -p lynter-cli); CI without Rust skips it.
const run = fs.existsSync(bin) ? describe : describe.skip;

run('WASM and CLI parity', () => {
  it('fix --stdin output equals the WASM build for every sample and example note', () => {
    const all = cases();
    const wasm = JSON.parse(
        execFileSync('node', [path.join(__dirname, '..', 'scripts', 'lint-batch.mjs')], {
          input: JSON.stringify(all.map((c) => ({...c, today}))),
          maxBuffer: 1 << 28,
        }).toString('utf8'),
    ) as {ok: boolean; text: string}[];
    const tmp = fs.mkdtempSync(path.join(os.tmpdir(), 'lynter-parity-'));
    const mismatches: string[] = [];
    all.forEach((c, i) => {
      fs.writeFileSync(path.join(tmp, 'lynter.yaml'), c.config);
      const r = spawnSync(bin, ['--today', today, 'fix', '--stdin', '--stdin-path', c.path], {cwd: tmp, input: c.note});
      if (!wasm[i].ok || r.stdout.toString('utf8') !== wasm[i].text) mismatches.push(c.name);
    });
    fs.rmSync(tmp, {recursive: true, force: true});
    expect(mismatches).toEqual([]);
    expect(all.length).toBeGreaterThan(10);
  });
});
