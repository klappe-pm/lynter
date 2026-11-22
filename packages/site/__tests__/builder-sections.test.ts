import {execFileSync} from 'node:child_process';
import path from 'node:path';
import {catalog, EMPTY_CONFIG, getIn, parse, setIn} from '../src/builder';
import {
  ADAPTERS, AUTOMATION_ACTIONS, AUTOMATION_FAILURES, AUTOMATION_LOGS, AUTOMATION_REPORTS, AUTOMATION_TRIGGERS, BUILTIN_CHECK_IDS,
  GLOBAL_TEMPLATE, SCOPED_TEMPLATES, addAutomation, addCheck, addNoteType, addProvider, addScopedTemplate, appendEntry, customCheckIds,
  entryNameIssue, fixReplacementAt, mappingText, nameIssue, parseMapping, removeEntry, removeProviderRule, scopedEntry, setAt,
  setBuiltinCheck, setChoice, setFix, setList, setProviderRule, setTrigger, setTriState, toggleCheckFlag,
} from '../src/builder-sections';

const repo = path.join(__dirname, '..', '..', '..');

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

const clean = (text: string) => expect(validate(text)).toMatchObject({ok: true, errors: [], warnings: []});
const data = (text: string) => parse(text).data;
const ids = catalog.slice(0, 2).map((r) => r.id);

describe('code checks', () => {
  test('built-in toggles and a custom check with every key validate cleanly', () => {
    let t = EMPTY_CONFIG;
    for (const id of BUILTIN_CHECK_IDS) t = setBuiltinCheck(t, id, true);
    t = addCheck(t, 'no-todo');
    const e = ['code-checks', 'no-todo'];
    t = setAt(t, e, ['enabled'], true);
    t = setAt(t, e, ['description'], 'Flag TODO markers.');
    t = setList(t, [...e, 'paths'], ['docs/**'], e.length);
    t = setList(t, [...e, 'languages'], ['sh', 'bash'], e.length);
    t = setAt(t, e, ['pattern'], 'TODO');
    t = setAt(t, e, ['flags'], toggleCheckFlag(toggleCheckFlag('', 's', true), 'i', true));
    t = setAt(t, e, ['message'], 'Resolve the TODO.');
    t = setFix(t, 'no-todo', '');
    clean(t);
    expect(getIn(data(t), e)).toEqual({
      enabled: true, description: 'Flag TODO markers.', paths: ['docs/**'], languages: ['sh', 'bash'], pattern: 'TODO', flags: 'is',
      message: 'Resolve the TODO.', fix: {replacement: ''},
    });
    expect(customCheckIds(data(t))).toEqual(['no-todo']);
    expect(fixReplacementAt(data(t), 'no-todo')).toBe('');
  });

  test('clearing fields removes keys, and the built-ins vanish when switched off', () => {
    let t = addCheck(setBuiltinCheck(EMPTY_CONFIG, BUILTIN_CHECK_IDS[0], true), 'mine');
    const e = ['code-checks', 'mine'];
    t = setAt(t, e, ['pattern'], 'x');
    t = setFix(t, 'mine', 'y');
    t = setFix(t, 'mine', null);
    t = setAt(t, e, ['pattern'], undefined);
    expect(getIn(data(t), e)).toEqual({});
    t = setBuiltinCheck(t, BUILTIN_CHECK_IDS[0], false);
    expect(getIn(data(t), ['code-checks'])).toEqual({mine: {}});
    t = setIn(t, e, undefined, 0);
    expect(data(t)).toEqual({version: 1});
  });

  test('flags are written in a fixed order and dropped when none is left', () => {
    expect(toggleCheckFlag('s', 'i', true)).toBe('is');
    expect(toggleCheckFlag('is', 'i', false)).toBe('s');
    expect(toggleCheckFlag('s', 's', false)).toBe('');
  });
});

describe('adapters', () => {
  test('every adapter key validates cleanly and clears to nothing', () => {
    let t = EMPTY_CONFIG;
    for (const {id, select} of ADAPTERS) {
      const at = (key: string) => ['defaults', 'adapters', id, key];
      t = setIn(t, at('enabled'), true, 0);
      t = setIn(t, at('path'), '/usr/local/bin/tool', 0);
      t = setList(t, at('args'), ['--preview'], 0);
      if (select) t = setList(t, at('select'), ['F', 'E'], 0);
    }
    clean(t);
    expect(getIn(data(t), ['defaults', 'adapters', 'ruff'])).toEqual({enabled: true, path: '/usr/local/bin/tool', args: ['--preview'], select: ['F', 'E']});
    expect(getIn(data(t), ['defaults', 'adapters', 'biome'])).toEqual({enabled: true, path: '/usr/local/bin/tool', args: ['--preview']});
    for (const {id, select} of ADAPTERS) {
      const at = (key: string) => ['defaults', 'adapters', id, key];
      t = setIn(t, at('enabled'), undefined, 0);
      t = setIn(t, at('path'), undefined, 0);
      t = setList(t, at('args'), [], 0);
      if (select) t = setList(t, at('select'), [], 0);
    }
    expect(data(t)).toEqual({version: 1});
  });
});

describe('rule order', () => {
  test('a list of rule ids validates cleanly and an empty list removes the key', () => {
    let t = setList(EMPTY_CONFIG, ['rule-order'], ids, 0);
    clean(t);
    expect(getIn(data(t), ['rule-order'])).toEqual(ids);
    t = setList(t, ['rule-order'], [], 0);
    expect(data(t)).toEqual({version: 1});
  });
});

describe('note types', () => {
  test('every key of a base and a scoped type validates cleanly', () => {
    let t = addNoteType(addNoteType(EMPTY_CONFIG, 'base'), 'task');
    const base = ['note-types', 'base'];
    t = setAt(t, base, ['required'], {domain: null, type: 'base', status: 'NEW', tags: []});
    t = setList(t, [...base, 'key-order'], ['type', 'status'], base.length);
    t = setAt(t, base, ['values'], {status: ['NEW', 'DONE']});
    t = setAt(t, base, ['date-keys', 'created'], 'date-created');
    t = setAt(t, base, ['date-keys', 'revised'], 'date-revised');
    t = setList(t, [...base, 'match', 'path'], ['notes/**'], base.length);
    t = setList(t, [...base, 'match', 'extension'], ['md'], base.length);
    t = setList(t, [...base, 'match', 'tag'], ['base'], base.length);
    t = setAt(t, base, ['match', 'frontmatter'], {type: 'base'});
    t = setAt(t, ['note-types', 'task'], ['required'], {type: 'task', status: 'NEW'});
    clean(t);
    expect(getIn(data(t), [...base, 'date-keys'])).toEqual({created: 'date-created', revised: 'date-revised'});
    expect(getIn(data(t), [...base, 'match'])).toEqual({path: ['notes/**'], extension: ['md'], tag: ['base'], frontmatter: {type: 'base'}});
  });

  test('a scoped type may not add a required key the base lacks', () => {
    let t = setAt(addNoteType(addNoteType(EMPTY_CONFIG, 'base'), 'task'), ['note-types', 'base'], ['required'], {type: 'base'});
    t = setAt(t, ['note-types', 'task'], ['required'], {due: null});
    expect(validate(t).errors).not.toEqual([]);
  });

  test('clearing fields leaves only the empty type, and removing it removes note-types', () => {
    const e = ['note-types', 'base'];
    let t = addNoteType(EMPTY_CONFIG, 'base');
    t = setAt(t, e, ['date-keys', 'created'], 'c');
    t = setAt(t, e, ['match', 'frontmatter'], {type: 'base'});
    t = setList(t, [...e, 'match', 'tag'], ['x'], e.length);
    t = setAt(t, e, ['date-keys', 'created'], undefined);
    t = setAt(t, e, ['match', 'frontmatter'], undefined);
    t = setList(t, [...e, 'match', 'tag'], [], e.length);
    expect(getIn(data(t), e)).toEqual({});
    t = setIn(t, e, undefined, 0);
    expect(data(t)).toEqual({version: 1});
  });
});

describe('templates', () => {
  test('global and scoped templates with every key validate cleanly', () => {
    let t = EMPTY_CONFIG;
    const write = (text: string, sub: string[], v: unknown) => setIn(text, [...GLOBAL_TEMPLATE, ...sub], v, 0);
    t = write(t, ['pinned-keys'], ['type']);
    t = write(t, ['key-order'], ['type', 'status']);
    t = write(t, ['frontmatter'], {type: 'note', tags: [], owner: null});
    t = write(t, ['body'], '# Title\n\nWrite here.\n');
    t = write(t, ['age-line'], true);
    t = addScopedTemplate(t, 'journal');
    t = addScopedTemplate(t, 'project');
    for (let i = 0; i < 2; i++) {
      const e = scopedEntry(i);
      t = setChoice(t, e, 'extends', 'global');
      t = setList(t, [...e, 'match', 'path'], [`area-${i}/**`], e.length);
      t = setList(t, [...e, 'match', 'extension'], ['md'], e.length);
      t = setList(t, [...e, 'match', 'tag'], ['t'], e.length);
      t = setAt(t, e, ['toggles'], {tags: false, owner: 'off', type: 'on'});
      t = setAt(t, e, ['pinned-keys'], ['owner']);
      t = setAt(t, e, ['key-order'], ['owner', 'type']);
      t = setAt(t, e, ['frontmatter'], {owner: 'me'});
      t = setAt(t, e, ['body'], 'Scoped body\n');
      t = setTriState(t, [...e, 'age-line'], 'off', e.length);
    }
    clean(t);
    expect(getIn(data(t), [...GLOBAL_TEMPLATE, 'body'])).toBe('# Title\n\nWrite here.\n');
    expect(getIn(data(t), [...scopedEntry(1), 'age-line'])).toBe(false);
  });

  test('duplicate scoped names are an error and are named by the form', () => {
    let t = addScopedTemplate(addScopedTemplate(EMPTY_CONFIG, 'a'), 'b');
    t = setAt(t, scopedEntry(1), ['name'], 'a');
    expect(validate(t).errors).not.toEqual([]);
    expect(entryNameIssue('a', ['a', 'a'])).toBe('duplicate');
    expect(entryNameIssue('', [''])).toBe('empty');
    expect(entryNameIssue('a', ['a', 'b'])).toBeNull();
  });

  test('clearing and removing leave no empty mappings behind', () => {
    let t = setIn(EMPTY_CONFIG, [...GLOBAL_TEMPLATE, 'body'], 'x', 0);
    t = addScopedTemplate(t, 'one');
    t = setAt(t, scopedEntry(0), ['toggles'], {a: true});
    t = setAt(t, scopedEntry(0), ['toggles'], undefined);
    expect(getIn(data(t), scopedEntry(0))).toEqual({name: 'one'});
    t = removeEntry(t, SCOPED_TEMPLATES, 0);
    expect(getIn(data(t), ['templates'])).toEqual({global: {body: 'x'}});
    t = setIn(t, [...GLOBAL_TEMPLATE, 'body'], undefined, 0);
    expect(data(t)).toEqual({version: 1});
  });
});

describe('automations', () => {
  test('every constrained value the form offers validates cleanly', () => {
    let t = EMPTY_CONFIG;
    AUTOMATION_TRIGGERS.forEach((trigger, i) => {
      t = addAutomation(t, `job-${i}`);
      const e = ['automations', i];
      t = setTrigger(t, i, trigger);
      if (trigger === 'schedule') t = setAt(t, e, ['schedule'], '0 6 * * 1');
      t = setChoice(t, e, 'action', AUTOMATION_ACTIONS[i % AUTOMATION_ACTIONS.length]);
      t = setChoice(t, e, 'failure', AUTOMATION_FAILURES[i % AUTOMATION_FAILURES.length]);
      t = setChoice(t, e, 'log', AUTOMATION_LOGS[i % AUTOMATION_LOGS.length]);
      t = setChoice(t, e, 'report', AUTOMATION_REPORTS[i % AUTOMATION_REPORTS.length]);
      t = setList(t, [...e, 'scope', 'path'], ['notes/**'], e.length);
      t = setList(t, [...e, 'scope', 'extension'], ['md'], e.length);
      t = setList(t, [...e, 'scope', 'tag'], ['x'], e.length);
    });
    clean(t);
    expect(itemCount(t, ['automations'])).toBe(AUTOMATION_TRIGGERS.length);
  });

  test('a schedule trigger requires a cron string and other triggers drop it', () => {
    let t = setTrigger(addAutomation(EMPTY_CONFIG, 'nightly'), 0, 'schedule');
    expect(validate(t).errors).not.toEqual([]);
    t = setAt(t, ['automations', 0], ['schedule'], '0 3 * * *');
    clean(t);
    t = setTrigger(t, 0, 'ci');
    expect(getIn(data(t), ['automations', 0])).toEqual({name: 'nightly', trigger: 'ci'});
    clean(t);
  });

  test('choices return to unset, and the last removal removes the list', () => {
    let t = setChoice(addAutomation(EMPTY_CONFIG, 'x'), ['automations', 0], 'action', 'fix');
    t = setChoice(t, ['automations', 0], 'action', '');
    t = setList(setAt(t, ['automations', 0], ['scope', 'path'], ['a']), ['automations', 0, 'scope', 'path'], [], 2);
    expect(getIn(data(t), ['automations', 0])).toEqual({name: 'x', trigger: 'manual'});
    t = removeEntry(t, ['automations'], 0);
    expect(data(t)).toEqual({version: 1});
  });

  test('the form offers exactly the values the schema accepts', () => {
    expect([...AUTOMATION_TRIGGERS]).toEqual(['on-write', 'on-create', 'on-rename', 'pre-commit', 'ci', 'schedule', 'manual']);
    expect([...AUTOMATION_ACTIONS]).toEqual(['check', 'fix', 'apply-template']);
    expect([...AUTOMATION_FAILURES]).toEqual(['open', 'closed']);
    expect([...AUTOMATION_LOGS]).toEqual(['spool', 'none']);
    expect([...AUTOMATION_REPORTS]).toEqual(['md', 'json', 'none']);
  });
});

describe('providers', () => {
  test('a namespace with rules validates cleanly and unwinds to nothing', () => {
    let t = addProvider(EMPTY_CONFIG, 'acme');
    t = setProviderRule(t, 'acme', ids[0], true);
    t = setProviderRule(t, 'acme', ids[1], false);
    clean(t);
    expect(getIn(data(t), ['providers', 'acme', 'rules'])).toEqual({[ids[0]]: {enabled: true}, [ids[1]]: {enabled: false}});
    t = removeProviderRule(removeProviderRule(t, 'acme', ids[0]), 'acme', ids[1]);
    expect(getIn(data(t), ['providers'])).toEqual({acme: {}});
    t = setIn(t, ['providers', 'acme'], undefined, 0);
    expect(data(t)).toEqual({version: 1});
  });
});

describe('helpers', () => {
  test('mapping fields accept only the values their key accepts', () => {
    expect(parseMapping('', 'seed')).toEqual({ok: true, value: undefined});
    expect(parseMapping('{}', 'seed')).toEqual({ok: true, value: undefined});
    expect(parseMapping('a: null\nb: [1, x]', 'seed')).toEqual({ok: true, value: {a: null, b: [1, 'x']}});
    expect(parseMapping('a: null', 'predicate')).toEqual({ok: false, reason: 'shape'});
    expect(parseMapping('a: x', 'allowed')).toEqual({ok: false, reason: 'shape'});
    expect(parseMapping('a: on\nb: false', 'toggles')).toEqual({ok: true, value: {a: 'on', b: false}});
    expect(parseMapping('a: maybe', 'toggles')).toEqual({ok: false, reason: 'shape'});
    expect(parseMapping('- a', 'seed')).toEqual({ok: false, reason: 'shape'});
    expect(parseMapping('a: [', 'seed')).toEqual({ok: false, reason: 'syntax'});
    expect(parseMapping(mappingText({a: ['x']}), 'allowed')).toEqual({ok: true, value: {a: ['x']}});
  });

  test('names are checked and entries append and remove in place', () => {
    expect(nameIssue('ok-name', [])).toBeNull();
    expect(nameIssue('Not OK', [])).toBe('format');
    expect(nameIssue('a', ['a'])).toBe('taken');
    let t = appendEntry(appendEntry(EMPTY_CONFIG, ['automations'], {name: 'a', trigger: 'ci'}), ['automations'], {name: 'b', trigger: 'ci'});
    expect(itemCount(t, ['automations'])).toBe(2);
    t = removeEntry(t, ['automations'], 0);
    expect(getIn(data(t), ['automations', 0, 'name'])).toBe('b');
  });
});

function itemCount(text: string, at: string[]): number {
  const v = getIn(data(text), at);
  return Array.isArray(v) ? v.length : 0;
}
