import {execFileSync} from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import {examples} from '../src/examples';

const root = path.join(__dirname, '..', '..', '..', 'examples');
const dirs = ['starter-vault', 'team-handbook', 'agent-memory', 'docs-site'];
const today = '2026-01-01';

interface Case {config: string; note: string; path: string; today: string}
interface Out {ok: boolean; changed?: boolean; text?: string; errors?: string[]}

/** Lint cases with the committed WASM build, in a child Node process. */
function lintAll(cases: Case[]): Out[] {
  const stdout = execFileSync('node', [path.join(__dirname, '..', 'scripts', 'lint-batch.mjs')], {
    input: JSON.stringify(cases),
    maxBuffer: 1 << 28,
  });
  return JSON.parse(stdout.toString('utf8')) as Out[];
}

function notes(dir: string): string[] {
  return fs.readdirSync(dir, {withFileTypes: true}).flatMap((e) => {
    const p = path.join(dir, e.name);
    if (e.isDirectory()) return notes(p);
    return e.name.endsWith('.md') && e.name !== 'README.md' ? [p] : [];
  });
}

describe('site examples', () => {
  it.each(dirs)('%s has a config with no errors and a README', (name) => {
    const dir = path.join(root, name);
    expect(fs.existsSync(path.join(dir, 'README.md'))).toBe(true);
    const config = fs.readFileSync(path.join(dir, 'lynter.yaml'), 'utf8');
    const cases = notes(dir).map((file) => ({
      config,
      note: fs.readFileSync(file, 'utf8'),
      path: path.relative(dir, file).split(path.sep).join('/'),
      today,
    }));
    for (const out of lintAll(cases)) {
      expect(out.ok).toBe(true);
      expect(out.changed).toBe(true);
    }
  });

  it('gallery examples lint and change the note', () => {
    const outs = lintAll(examples.map((e) => ({config: e.config, note: e.note, path: e.path, today})));
    outs.forEach((out) => {
      expect(out.ok).toBe(true);
      expect(out.changed).toBe(true);
    });
  });
});
