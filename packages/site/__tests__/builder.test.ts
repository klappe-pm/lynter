import {execFileSync} from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import {
  addProfile, catalog, EMPTY_CONFIG, getIn, optionKind, parse, profileNameError, profileNames, removeRule, ruleState,
  setIn, setRuleEnabled, setRuleOption,
} from '../src/builder';

const repo = path.join(__dirname, '..', '..', '..');
const rule = (id: string) => catalog.find((r) => r.id === id)!;

/** Validate a config with the committed WASM build, as the page does. */
function validate(text: string): {ok: boolean; errors: unknown[]; warnings: unknown[]} {
  const script = `
    import fs from 'node:fs';
    const dir = ${JSON.stringify(path.join(repo, 'packages', 'site', 'public', 'wasm'))};
    const mod = await import(dir + '/lynter_wasm.mjs');
    mod.initSync({module: fs.readFileSync(dir + '/lynter_wasm_bg.wasm')});
    process.stdout.write(mod.validate_config(fs.readFileSync(0, 'utf8')));`;
  return JSON.parse(execFileSync('node', ['--input-type=module', '-e', script], {input: text}).toString('utf8'));
}

describe('rule catalog', () => {
  test('is exactly what `lynter rules --json` prints', () => {
    const printed = execFileSync(path.join(repo, 'target', 'debug', 'lynter'), ['rules', '--json']).toString('utf8');
    expect(JSON.parse(printed)).toEqual(JSON.parse(fs.readFileSync(path.join(repo, 'packages', 'site', 'src', 'rule-catalog.json'), 'utf8')));
  });

  test('every rule has a group the builder shows and options it can edit', () => {
    const groups = ['wording', 'content', 'formatting', 'note-types', 'notes', 'data', 'code'];
    for (const r of catalog) {
      expect(groups).toContain(r.group);
      for (const value of Object.values(r.options)) expect(['toggle', 'number', 'text', 'list', 'yaml']).toContain(optionKind(value));
    }
  });
});

describe('editing the YAML text', () => {
  test('turning a rule on writes a stanza the engine accepts without warnings', () => {
    let text = EMPTY_CONFIG;
    for (const r of catalog) text = setRuleEnabled(text, null, r.id, true);
    const out = validate(text);
    expect(out.errors).toEqual([]);
    expect(out.warnings).toEqual([]);
    expect(catalog.every((r) => ruleState(parse(text).data, null, r).enabled)).toBe(true);
  });

  test('an option is written when it differs from the default and removed when it returns to it', () => {
    const r = rule('paragraph-spacing');
    let text = setRuleOption(setRuleEnabled(EMPTY_CONFIG, null, r.id, true), null, r, 'blank-lines', 2);
    expect(getIn(parse(text).data, ['defaults', 'rules', r.id])).toEqual({enabled: true, 'blank-lines': 2});
    text = setRuleOption(text, null, r, 'blank-lines', 1);
    expect(getIn(parse(text).data, ['defaults', 'rules', r.id])).toEqual({enabled: true});
  });

  test('removing the last rule removes the empty mappings above it, and comments survive edits', () => {
    const start = '# house style\nversion: 1 # schema\n';
    let text = setRuleEnabled(start, null, 'contractions', true);
    expect(text).toContain('# house style');
    expect(text).toContain('# schema');
    text = removeRule(text, null, 'contractions');
    expect(parse(text).data).toEqual({version: 1});
  });

  test('a scope keeps its own overrides apart from the defaults', () => {
    let text = addProfile(setRuleEnabled(EMPTY_CONFIG, null, 'contractions', true), 'work');
    text = setRuleEnabled(text, 'work', 'contractions', false);
    const data = parse(text).data;
    expect(profileNames(data)).toEqual(['work']);
    expect(ruleState(data, null, rule('contractions')).enabled).toBe(true);
    expect(ruleState(data, 'work', rule('contractions'))).toMatchObject({present: true, enabled: false});
    expect(validate(text)).toMatchObject({errors: [], warnings: []});
  });

  test('text that does not parse is left alone', () => {
    const broken = 'version: 1\ndefaults: [\n';
    expect(parse(broken).error).not.toBeNull();
    expect(setRuleEnabled(broken, null, 'contractions', true)).toBe(broken);
    expect(setIn('- a\n- b\n', ['version'], 1)).toBe('- a\n- b\n');
  });

  test('scope names are checked', () => {
    expect(profileNameError('work', [])).toBeNull();
    expect(profileNameError('Work Notes', [])).not.toBeNull();
    expect(profileNameError('work', ['work'])).not.toBeNull();
  });
});
