/**
 * Browser wrapper over the Rust core compiled to WebAssembly (crates/lynter-wasm).
 * The site never reimplements a rule. The module loads once from
 * /wasm/ (built by scripts/build-wasm.sh and committed); until it is ready,
 * lint returns the note unchanged.
 */
import {useEffect, useState} from 'react';

export interface LintOk {
  ok: true;
  text: string;
  changed: boolean;
  rules: string[];
  profileChain: string[];
  noteType: string | null;
}

export interface LintFailed {
  ok: false;
  errors: string[];
}

export type LintOutcome = LintOk | LintFailed;

export interface ConfigProblem {
  message: string;
  /** 1-based line in lynter.yaml when the parser reports one. */
  line: number | null;
  /** Config key path such as `profiles.work.rules`, empty when not applicable. */
  path: string;
  /** An error stops the config from loading; a warning loads but is ignored. */
  severity: 'error' | 'warning';
}

interface WasmApi {
  lint(config: string, note: string, path: string, today: string): string;
  validate_config(yaml: string): string;
  profile_chain(config: string, path: string): string;
  convert_data_json(json: string): string | undefined;
  scaffold(): string;
  version(): string;
}

let api: WasmApi | null = null;
let loading: Promise<void> | null = null;

/** Fetches the module from the site. The standalone builder swaps this for one that reads embedded bytes. */
let loader: () => Promise<WasmApi> = async () => {
  const url = '/wasm/lynter_wasm.mjs';
  const mod = await import(/* webpackIgnore: true */ url);
  await mod.default({module_or_path: '/wasm/lynter_wasm_bg.wasm'});
  return mod as WasmApi;
};

/** Replace how the engine is loaded. Call before the first render. */
export function setEngineLoader(next: () => Promise<unknown>): void {
  loader = next as () => Promise<WasmApi>;
}

/** Load and instantiate the WASM module once; later calls share the promise. */
export function initEngine(): Promise<void> {
  if (!loading) {
    loading = loader().then((mod) => {
      api = mod;
    }).catch((error) => {
      loading = null;
      throw error;
    });
  }
  return loading;
}

/** Whether the module is loaded. */
export function engineReady(): boolean {
  return api !== null;
}

/**
 * React hook: starts the load and reports state. `slow` turns true only when
 * loading takes over 100 ms, so a fast load never flashes a loading state.
 */
export function useEngine(): {ready: boolean; slow: boolean} {
  const [ready, setReady] = useState(engineReady());
  const [slow, setSlow] = useState(false);
  useEffect(() => {
    if (engineReady()) {
      setReady(true);
      return undefined;
    }
    const timer = setTimeout(() => setSlow(true), 100);
    let live = true;
    initEngine().then(
        () => live && setReady(true),
        () => undefined,
    );
    return () => {
      live = false;
      clearTimeout(timer);
    };
  }, []);
  return {ready, slow: slow && !ready};
}

/**
 * Lint one note against a linter.yaml text.
 * @param {string} configText linter.yaml contents
 * @param {string} note Markdown note contents
 * @param {string} path vault-relative path used for scope matching
 * @param {string} today ISO date for timestamp rules
 * @return {LintOutcome} the fixed note, or the config errors that stopped it
 */
export function lint(configText: string, note: string, path: string, today: string): LintOutcome {
  if (!api) return {ok: true, text: note, changed: false, rules: [], profileChain: [], noteType: null};
  return JSON.parse(api.lint(configText, note, path || 'note.md', today)) as LintOutcome;
}

/**
 * Lint a lynter.yaml text itself: syntax, unknown keys, and bad values.
 * @param {string} configText lynter.yaml contents
 * @return {ConfigProblem[]} errors first, then warnings; empty when the file is clean
 */
export function validateConfig(configText: string): ConfigProblem[] {
  if (!api) return [];
  type Raw = Omit<ConfigProblem, 'severity'>;
  const out = JSON.parse(api.validate_config(configText)) as {errors: Raw[]; warnings?: Raw[]};
  return [
    ...out.errors.map((p) => ({...p, severity: 'error' as const})),
    ...(out.warnings ?? []).map((p) => ({...p, severity: 'warning' as const})),
  ];
}

/**
 * Resolve the profile chain for a path without caring about note content.
 * @param {string} configText linter.yaml contents
 * @param {string} path vault-relative path
 * @return {string[]} profiles applied in order, base first
 */
export function profileChain(configText: string, path: string): string[] {
  if (!api) return [];
  return JSON.parse(api.profile_chain(configText, path)) as string[];
}

/**
 * Convert an Obsidian Linter data.json text to a lynter.yaml text.
 * @param {string} dataJson the plugin's data.json contents
 * @return {string | null} lynter.yaml text, or null when the input cannot be converted
 */
export function convertDataJson(dataJson: string): string | null {
  if (!api) return null;
  return api.convert_data_json(dataJson) ?? null;
}

/** The commented starter lynter.yaml the engine ships, or null until the engine has loaded. */
export function starterConfig(): string | null {
  return api ? api.scaffold() : null;
}

/** Today as yyyy-MM-dd in the viewer's local time. */
export function todayIso(): string {
  const d = new Date();
  const pad = (n: number) => String(n).padStart(2, '0');
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}
