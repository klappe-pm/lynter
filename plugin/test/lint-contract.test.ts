import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {lintNote, localDate, validateConfig} from '../src/engine';
import {Lynter, YIELD_EVERY} from '../src/lynter';
import {lintJson, makeFake} from './helpers';

describe('lint call contract', () => {
  it('passes config, note, vault-relative path and a local date', () => {
    const lint = jest.fn(() => lintJson('out', true));
    const r = lintNote({lint, scaffold: () => ''}, 'cfg', 'note', 'folder/a.md', '2026-03-04');
    expect(lint).toHaveBeenCalledWith('cfg', 'note', 'folder/a.md', '2026-03-04');
    expect(r).toEqual({ok: true, text: 'out', changed: true, rules: []});
  });

  it('uses lint_file when lint is absent', () => {
    const lint_file = jest.fn(() => lintJson('x'));
    expect(lintNote({lint_file, scaffold: () => ''}, 'c', 'n', 'p.md', 'd').ok).toBe(true);
    expect(lint_file).toHaveBeenCalled();
  });

  it('reports string errors and object errors', () => {
    const a = lintNote({lint: () => JSON.stringify({ok: false, errors: ['bad']}), scaffold: () => ''}, 'c', 'n', 'p', 'd');
    expect(a).toEqual({ok: false, errors: ['bad']});
    const b = lintNote({lint: () => JSON.stringify({ok: false, errors: [{message: 'bad', line: 3}]}), scaffold: () => ''}, 'c', 'n', 'p', 'd');
    expect(b).toEqual({ok: false, errors: ['Line 3: bad']});
  });

  it('turns thrown errors and bad JSON into failures', () => {
    const r = lintNote({lint: () => 'not json', scaffold: () => ''}, 'c', 'n', 'p', 'd');
    expect(r.ok).toBe(false);
  });

  it('formats the date locally', () => {
    expect(localDate(new Date(2026, 0, 5))).toBe('2026-01-05');
  });

  it('feature-detects validate_config', () => {
    const validate_config = jest.fn(() => JSON.stringify({ok: false, errors: [{message: 'oops', line: 2}]}));
    expect(validateConfig({validate_config, scaffold: () => ''}, 'c')).toEqual({ok: false, errors: ['Line 2: oops']});
    const lint = jest.fn(() => JSON.stringify({ok: false, errors: ['cfg bad']}));
    expect(validateConfig({lint, scaffold: () => ''}, 'c')).toEqual({ok: false, errors: ['cfg bad']});
    expect(lint).toHaveBeenCalledWith('c', '', 'note.md', expect.any(String));
  });
});

describe('Lynter commands', () => {
  it('passes the vault path to the WASM lint', async () => {
    const lint = jest.fn((_c: string, n: string) => lintJson(n, false));
    const fake = makeFake({lint});
    fake.disk.set('lynter.yaml', 'cfg');
    fake.files.set('a/b.md', 'x');
    await new Lynter(fake.host).lintFile({path: 'a/b.md'});
    expect(lint).toHaveBeenCalledWith('cfg', 'x', 'a/b.md', '2026-01-02');
  });

  it('refuses to lint without a config', async () => {
    const fake = makeFake({lint: jest.fn()});
    fake.files.set('a.md', 'x');
    expect(await new Lynter(fake.host).lintFile({path: 'a.md'})).toBe('no-config');
    expect(fake.processCalls).toHaveLength(0);
  });

  it('leaves the note alone on config errors', async () => {
    const fake = makeFake({lint: () => JSON.stringify({ok: false, errors: ['bad']})});
    fake.disk.set('lynter.yaml', 'cfg');
    fake.files.set('a.md', 'keep');
    expect(await new Lynter(fake.host).lintFile({path: 'a.md'})).toBe('error');
    expect(fake.files.get('a.md')).toBe('keep');
  });

  it('yields to the UI every 25 files and reports progress', async () => {
    const fake = makeFake({lint: (_c: string, n: string) => lintJson(n.toUpperCase())});
    fake.disk.set('lynter.yaml', 'cfg');
    for (let i = 0; i < 60; i++) fake.files.set(`n${i}.md`, 'a');
    const r = await new Lynter(fake.host).lintAll();
    expect(r).toEqual({changed: 60, errors: 0, total: 60});
    expect(YIELD_EVERY).toBe(25);
    expect(fake.yields).toBe(2);
    expect(fake.progress).toEqual(['Lynter: 25 of 60 notes', 'Lynter: 50 of 60 notes']);
  });

  it('creates the starter config once and never overwrites', async () => {
    const fake = makeFake({scaffold: () => 'version: 1\n', lint: () => lintJson('')});
    const l = new Lynter(fake.host);
    expect(await l.createConfig()).toBe('created');
    expect(fake.disk.get('lynter.yaml')).toBe('version: 1\n');
    expect(await l.createConfig()).toBe('exists');
  });

  it('refuses to write an invalid starter config', async () => {
    const fake = makeFake({scaffold: () => 'bad', validate_config: () => JSON.stringify({ok: false, errors: ['no']})});
    expect(await new Lynter(fake.host).createConfig()).toBe('invalid');
    expect(fake.disk.has('lynter.yaml')).toBe(false);
  });

  it('does not create when a deprecated config exists', async () => {
    const fake = makeFake({});
    fake.disk.set('linter.yaml', 'x');
    expect(await new Lynter(fake.host).createConfig()).toBe('exists');
  });

  it('stays quiet when notices are off, except for required prompts', async () => {
    const fake = makeFake({lint: (_c: string, n: string) => lintJson(n, false)}, {showNotices: false});
    fake.disk.set('lynter.yaml', 'cfg');
    fake.files.set('a.md', 'x');
    await new Lynter(fake.host).lintActive({path: 'a.md'});
    expect(fake.notices).toHaveLength(0);
  });
});

describe('validate_config path', () => {
  it('calls validate_config and never the lint fallback when it exists', () => {
    const validate_config = jest.fn(() => JSON.stringify({ok: true, errors: []}));
    const lint = jest.fn();
    expect(validateConfig({validate_config, lint, scaffold: () => ''}, 'c')).toEqual({ok: true, errors: []});
    expect(validate_config).toHaveBeenCalledWith('c');
    expect(lint).not.toHaveBeenCalled();
  });

  it('is exported by the checked-in WASM glue', () => {
    const glue = readFileSync(resolve(__dirname, '../../packages/site/public/wasm/lynter_wasm.mjs'), 'utf8');
    expect(glue).toMatch(/export function validate_config\(/);
  });
});
