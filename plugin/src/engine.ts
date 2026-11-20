/** The subset of the WASM exports Lynter uses. */
export interface WasmApi {
  lint?: (configYaml: string, note: string, path: string, today: string) => string;
  lint_file?: (configYaml: string, note: string, path: string, today: string) => string;
  scaffold: () => string;
  version?: () => string;
  validate_config?: (configYaml: string) => string;
}

export interface LintFailure {
  ok: false;
  errors: string[];
}

export interface LintSuccess {
  ok: true;
  text: string;
  changed: boolean;
  rules: string[];
}

export type LintOutcome = LintSuccess | LintFailure;

export interface Validation {
  ok: boolean;
  errors: string[];
}

type ErrorItem = string | {message?: string; line?: number};

function formatError(e: ErrorItem): string {
  if (typeof e === 'string') return e;
  const msg = e.message ?? 'Unknown error';
  return typeof e.line === 'number' ? `Line ${e.line}: ${msg}` : msg;
}

function formatErrors(raw: unknown): string[] {
  return Array.isArray(raw) ? (raw as ErrorItem[]).map(formatError) : [];
}

/** Local date as YYYY-MM-DD, the `today` the WASM lint takes. */
export function localDate(d: Date = new Date()): string {
  const p = (n: number) => String(n).padStart(2, '0');
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}`;
}

/** Lint one note. `path` is vault-relative so scopes apply. */
export function lintNote(api: WasmApi, configYaml: string, note: string, path: string, today: string = localDate()): LintOutcome {
  const fn = api.lint ?? api.lint_file;
  if (!fn) {
    return {ok: false, errors: ['This Lynter build has no lint function.']};
  }
  let parsed: {ok?: boolean; text?: string; changed?: boolean; rules?: string[]; errors?: unknown};
  try {
    parsed = JSON.parse(fn(configYaml, note, path, today));
  } catch (e) {
    return {ok: false, errors: [e instanceof Error ? e.message : String(e)]};
  }
  if (parsed.ok !== true || typeof parsed.text !== 'string') {
    return {ok: false, errors: formatErrors(parsed.errors)};
  }
  return {ok: true, text: parsed.text, changed: parsed.changed === true, rules: parsed.rules ?? []};
}

/**
 * Validate config text. Uses `validate_config` when the WASM has it, and falls
 * back to linting an empty note, which reports the same config errors.
 */
export function validateConfig(api: WasmApi, configYaml: string): Validation {
  if (api.validate_config) {
    try {
      const r = JSON.parse(api.validate_config(configYaml));
      return {ok: r.ok === true, errors: formatErrors(r.errors)};
    } catch (e) {
      return {ok: false, errors: [e instanceof Error ? e.message : String(e)]};
    }
  }
  const r = lintNote(api, configYaml, '', 'note.md');
  return r.ok ? {ok: true, errors: []} : {ok: false, errors: r.errors};
}
