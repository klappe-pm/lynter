/**
 * Pure helpers behind the form sections of the config builder. Every write
 * goes through `setIn`, so the YAML text stays the single source of truth and
 * a field returned to empty removes its key instead of leaving an empty value.
 */
import {parse as parseYaml, stringify} from 'yaml';
import {asList, getIn, parse, setIn, type Path} from './builder';

export const isRecord = (v: unknown): v is Record<string, unknown> => v !== null && typeof v === 'object' && !Array.isArray(v);

/** Keys of the mapping at a path, in file order. */
export const keysAt = (data: unknown, path: Path): string[] => {
  const v = getIn(data, path);
  return isRecord(v) ? Object.keys(v) : [];
};

/** Items of the list at a path. */
export const itemsAt = (data: unknown, path: Path): unknown[] => {
  const v = getIn(data, path);
  return Array.isArray(v) ? v : [];
};

export const textAt = (data: unknown, path: Path): string => {
  const v = getIn(data, path);
  return typeof v === 'string' ? v : '';
};

export const listAt = (data: unknown, path: Path): string[] => asList(getIn(data, path));

/** True when the key holds a boolean true; a missing key is off. */
export const flagAt = (data: unknown, path: Path): boolean => getIn(data, path) === true;

/** Write a string, or drop the key when the field is empty. */
export const setText = (text: string, path: Path, value: string, keep = 0): string => setIn(text, path, value === '' ? undefined : value, keep);

/** Write a list, or drop the key when the list is empty. */
export const setList = (text: string, path: Path, value: string[], keep = 0): string => setIn(text, path, value.length ? value : undefined, keep);

/** Write true, or drop the key: every switch in the schema is off when absent. */
export const setFlag = (text: string, path: Path, on: boolean, keep = 0): string => setIn(text, path, on ? true : undefined, keep);

/** A three-way choice for a boolean that may also be left unset. */
export type TriState = '' | 'on' | 'off';

export const triStateAt = (data: unknown, path: Path): TriState => {
  const v = getIn(data, path);
  return v === true ? 'on' : v === false ? 'off' : '';
};

export const setTriState = (text: string, path: Path, value: TriState, keep = 0): string =>
  setIn(text, path, value === '' ? undefined : value === 'on', keep);

/**
 * Write below one list entry or mapping entry. Pruning stops at the entry, so
 * clearing its last field leaves the entry in place instead of shifting a list.
 */
export const setAt = (text: string, entry: Path, sub: Path, value: unknown): string => setIn(text, [...entry, ...sub], value, entry.length);

/** Why a new name is refused, or null when it is fine. */
export type NameIssue = 'format' | 'taken' | null;

export function nameIssue(name: string, taken: string[]): NameIssue {
  if (!/^[a-z0-9][a-z0-9-]*$/.test(name)) return 'format';
  return taken.includes(name) ? 'taken' : null;
}

/** Why an entry's own name is wrong once it is in the file. */
export function entryNameIssue(name: string, all: string[]): 'empty' | 'duplicate' | null {
  if (name.trim() === '') return 'empty';
  return all.filter((n) => n === name).length > 1 ? 'duplicate' : null;
}

/** Append one entry to a list, creating the list when it is missing. */
export function appendEntry(text: string, listPath: Path, entry: unknown): string {
  const list = getIn(parse(text).data, listPath);
  return setIn(text, Array.isArray(list) ? [...listPath, list.length] : listPath, Array.isArray(list) ? entry : [entry]);
}

/** Remove one list entry, and the list itself when it was the last one. */
export function removeEntry(text: string, listPath: Path, index: number): string {
  const list = getIn(parse(text).data, listPath);
  const count = Array.isArray(list) ? list.length : 0;
  return setIn(text, count <= 1 ? listPath : [...listPath, index], undefined, 0);
}

/* ---------- code checks ---------- */

/** Built-in checks the engine merges under `code-checks`; all are off until enabled. */
export const BUILTIN_CHECK_IDS = ['no-token-shaped-strings', 'no-gnu-only-flags-in-sh', 'no-trailing-whitespace-in-code'] as const;

export const isBuiltinCheck = (id: string): boolean => (BUILTIN_CHECK_IDS as readonly string[]).includes(id);

/** Checks the user defined, which the form lets them edit and remove. */
export const customCheckIds = (data: unknown): string[] => keysAt(data, ['code-checks']).filter((id) => !isBuiltinCheck(id));

/** Regex flags the engine honours, in the order it writes them. */
export const CHECK_FLAGS = ['i', 'm', 's'] as const;

export function toggleCheckFlag(current: string, flag: string, on: boolean): string {
  const set = new Set(current.split(''));
  if (on) set.add(flag);
  else set.delete(flag);
  return CHECK_FLAGS.filter((f) => set.has(f)).join('');
}

export const addCheck = (text: string, id: string): string => setIn(text, ['code-checks', id], {});

/** Enable or disable a built-in check; the default is off, so off removes the key. */
export const setBuiltinCheck = (text: string, id: string, on: boolean): string => setFlag(text, ['code-checks', id, 'enabled'], on, 0);

/** The fix replacement is the one place an empty string is a real value (delete the match). */
export const fixReplacementAt = (data: unknown, id: string): string | null => {
  const v = getIn(data, ['code-checks', id, 'fix', 'replacement']);
  return typeof v === 'string' ? v : null;
};

export function setFix(text: string, id: string, replacement: string | null): string {
  return setIn(text, ['code-checks', id, 'fix'], replacement === null ? undefined : {replacement}, 2);
}

/* ---------- adapters ---------- */

export interface AdapterSpec {
  id: string;
  /** ruff takes rule codes for --select; biome has no such key. */
  select: boolean;
}

export const ADAPTERS: AdapterSpec[] = [
  {id: 'ruff', select: true},
  {id: 'biome', select: false},
];

/* ---------- note types and templates ---------- */

/** What a mapping field accepts for its values; each mirrors one check in the validator. */
export type MappingKind = 'seed' | 'predicate' | 'allowed' | 'toggles';

export type MappingResult = {ok: true; value: Record<string, unknown> | undefined} | {ok: false; reason: 'syntax' | 'shape'};

const isScalar = (v: unknown): boolean => ['string', 'number', 'boolean'].includes(typeof v);
const isScalarList = (v: unknown): boolean => Array.isArray(v) && v.every(isScalar);

const valueFits = (kind: MappingKind, v: unknown): boolean => {
  if (kind === 'seed') return v === null || isScalar(v) || isScalarList(v);
  if (kind === 'predicate') return isScalar(v) || isScalarList(v);
  if (kind === 'allowed') return isScalarList(v);
  return typeof v === 'boolean' || v === 'on' || v === 'off';
};

/** Parse a YAML mapping typed into a form; empty text, or `{}`, means the key is unset. */
export function parseMapping(source: string, kind: MappingKind): MappingResult {
  if (source.trim() === '') return {ok: true, value: undefined};
  let value: unknown;
  try {
    value = parseYaml(source);
  } catch {
    return {ok: false, reason: 'syntax'};
  }
  if (value === null || value === undefined) return {ok: true, value: undefined};
  if (!isRecord(value)) return {ok: false, reason: 'shape'};
  if (!Object.values(value).every((v) => valueFits(kind, v))) return {ok: false, reason: 'shape'};
  return {ok: true, value: Object.keys(value).length ? value : undefined};
}

/** The text a mapping field starts from. */
export const mappingText = (value: unknown): string => (isRecord(value) && Object.keys(value).length ? stringify(value, {lineWidth: 0}).trimEnd() : '');

/** Keys a `match` block edits through lists; other match keys stay in the YAML pane. */
export const MATCH_LISTS = ['path', 'extension', 'tag'] as const;

/** Note types a scoped schema may not add required keys to: the base type bounds them. */
export const BASE_NOTE_TYPE = 'base';

export const DATE_KEYS = ['created', 'revised'] as const;

export const TEMPLATE_LISTS = ['pinned-keys', 'key-order'] as const;

/** Where the global template and each scoped template live. */
export const GLOBAL_TEMPLATE: Path = ['templates', 'global'];
export const SCOPED_TEMPLATES: Path = ['templates', 'by-scope'];

/** Where one scoped template sits in the data. */
export const scopedEntry = (index: number): Path => [...SCOPED_TEMPLATES, index];

export const addNoteType = (text: string, name: string): string => setIn(text, ['note-types', name], {});

export const addScopedTemplate = (text: string, name: string): string => appendEntry(text, SCOPED_TEMPLATES, {name});

/* ---------- automations ---------- */

export const AUTOMATION_TRIGGERS = ['on-write', 'on-create', 'on-rename', 'pre-commit', 'ci', 'schedule', 'manual'] as const;
export const AUTOMATION_ACTIONS = ['check', 'fix', 'apply-template'] as const;
export const AUTOMATION_FAILURES = ['open', 'closed'] as const;
export const AUTOMATION_LOGS = ['spool', 'none'] as const;
export const AUTOMATION_REPORTS = ['md', 'json', 'none'] as const;

/** Only a schedule trigger reads the cron string. */
export const triggerNeedsSchedule = (trigger: string): boolean => trigger === 'schedule';

/** A new automation starts on a trigger that needs nothing else. */
export const addAutomation = (text: string, name: string): string => appendEntry(text, ['automations'], {name, trigger: 'manual'});

/** Change the trigger, and drop a schedule the new trigger no longer reads. */
export function setTrigger(text: string, index: number, trigger: string): string {
  const entry: Path = ['automations', index];
  const next = setAt(text, entry, ['trigger'], trigger);
  return triggerNeedsSchedule(trigger) ? next : setAt(next, entry, ['schedule'], undefined);
}

/** Set a constrained key, or drop it when the select is back on "not set". */
export const setChoice = (text: string, entry: Path, key: string, value: string): string => setAt(text, entry, [key], value === '' ? undefined : value);

/* ---------- providers ---------- */

export const addProvider = (text: string, name: string): string => setIn(text, ['providers', name], {});

/** Turn a provider rule on or off. Unlike elsewhere off is written, so a provider can switch a rule off. */
export const setProviderRule = (text: string, ns: string, id: string, enabled: boolean): string => setAt(text, ['providers', ns], ['rules', id, 'enabled'], enabled);

export const removeProviderRule = (text: string, ns: string, id: string): string => setAt(text, ['providers', ns], ['rules', id], undefined);
