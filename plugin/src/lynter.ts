import {anyConfigExists, CONFIG_NAME, discoverConfig, type ConfigFs, type FoundConfig} from './config';
import {lintNote, localDate, validateConfig, type Validation, type WasmApi} from './engine';
import {SaveGuard} from './save-guard';

export interface FileRef {
  path: string;
}

export interface NoticeHandle {
  setMessage(message: string): unknown;
  hide(): void;
}

export interface Host<F extends FileRef> {
  fs: ConfigFs & {write(path: string, data: string): Promise<void>};
  markdownFiles(): F[];
  /** Atomic read-modify-write, like Vault.process. */
  process(file: F, fn: (data: string) => string): Promise<string>;
  read(file: F): Promise<string>;
  loadWasm(): Promise<WasmApi>;
  notice(message: string, timeout?: number): NoticeHandle;
  showNotices(): boolean;
  yieldToUi(): Promise<void>;
  today?(): string;
  /** True when the file is open in the active editor. */
  isActive?(file: F): boolean;
}

export type FileResult = 'changed' | 'unchanged' | 'error' | 'no-config';

export const YIELD_EVERY = 25;
/** How long the active file must stay unmodified before lint on save runs. */
export const DEBOUNCE_MS = 2000;

/** Lynter's behavior, independent of the Obsidian classes. */
export class Lynter<F extends FileRef> {
  readonly guard = new SaveGuard();
  private deprecatedNoticeShown = false;
  private lastConfigKey: string | undefined;
  private pending = new Map<string, ReturnType<typeof setTimeout>>();

  constructor(private host: Host<F>) {}

  private say(message: string, timeout?: number): void {
    if (this.host.showNotices()) this.host.notice(message, timeout);
  }

  /** The active config, with a one-time notice for deprecated names. */
  async activeConfig(): Promise<FoundConfig | null> {
    const found = await discoverConfig(this.host.fs);
    // A changed config can change the output, so remembered outputs are stale.
    const key = found ? `${found.name}\n${found.text}` : '';
    if (key !== this.lastConfigKey) {
      this.guard.clear();
      this.lastConfigKey = key;
    }
    if (found?.deprecated && !this.deprecatedNoticeShown) {
      this.deprecatedNoticeShown = true;
      // Always shown: it asks the person to act.
      this.host.notice(`Lynter is reading ${found.name}. Rename it to ${CONFIG_NAME}.`, 8000);
    }
    return found;
  }

  async lintFile(file: F, config?: FoundConfig | null): Promise<FileResult> {
    const cfg = config === undefined ? await this.activeConfig() : config;
    if (!cfg) {
      this.host.notice(`No ${CONFIG_NAME} found. Run "Create ${CONFIG_NAME}" first.`);
      return 'no-config';
    }
    const api = await this.host.loadWasm();
    const today = this.host.today ? this.host.today() : localDate();
    const state: {result: FileResult; failure?: string} = {result: 'unchanged'};
    await this.host.process(file, (data) => {
      const r = lintNote(api, cfg.text, data, file.path, today);
      if (!r.ok) {
        state.result = 'error';
        state.failure = r.errors[0];
        return data;
      }
      this.guard.recordOutput(file.path, r.text);
      if (r.text !== data) state.result = 'changed';
      return r.text;
    });
    if (state.result === 'error') this.host.notice(`Lynter could not lint ${file.path}: ${state.failure ?? 'unknown error'}`);
    return state.result;
  }

  async lintActive(file: F | null): Promise<void> {
    if (!file) {
      this.say('Lynter: open a note to lint it.');
      return;
    }
    const r = await this.lintFile(file);
    if (r === 'changed') this.say(`Lynter: tidied ${file.path}.`);
    else if (r === 'unchanged') this.say(`Lynter: ${file.path} is already tidy.`);
  }

  async lintAll(): Promise<{changed: number; errors: number; total: number} | null> {
    const cfg = await this.activeConfig();
    if (!cfg) {
      this.host.notice(`No ${CONFIG_NAME} found. Run "Create ${CONFIG_NAME}" first.`);
      return null;
    }
    const files = this.host.markdownFiles();
    const progress = this.host.showNotices() ? this.host.notice(`Lynter: 0 of ${files.length} notes`, 0) : null;
    let changed = 0;
    let errors = 0;
    for (let i = 0; i < files.length; i++) {
      try {
        const r = await this.lintFile(files[i], cfg);
        if (r === 'changed') changed++;
        if (r === 'error') errors++;
      } catch {
        errors++;
      }
      if ((i + 1) % YIELD_EVERY === 0) {
        progress?.setMessage(`Lynter: ${i + 1} of ${files.length} notes`);
        await this.host.yieldToUi();
      }
    }
    progress?.hide();
    this.say(`Lynter: tidied ${changed} of ${files.length} notes${errors ? `, ${errors} failed` : ''}.`);
    return {changed, errors, total: files.length};
  }

  /**
   * Lint on save for one modified file. A file open in the active editor waits
   * until it has been idle for DEBOUNCE_MS, so typing is never rewritten.
   * Background files lint at once. Returns true when it wrote now.
   */
  async onModify(file: F): Promise<boolean> {
    this.cancel(file.path);
    if (this.host.isActive?.(file)) {
      const timer = setTimeout(() => {
        this.pending.delete(file.path);
        void this.lintIdle(file).catch((e) => console.error('Lynter', e));
      }, DEBOUNCE_MS);
      this.pending.set(file.path, timer);
      return false;
    }
    // Reading the config first drops remembered outputs when it changed.
    const cfg = await this.activeConfig();
    const content = await this.host.read(file);
    if (!this.guard.shouldProcess(file.path, content)) return false;
    const r = await this.lintFile(file, cfg);
    return r === 'changed';
  }

  /** Cancel a pending debounced lint for a path. */
  cancel(path: string): void {
    const t = this.pending.get(path);
    if (t !== undefined) {
      clearTimeout(t);
      this.pending.delete(path);
    }
  }

  /** Cancel every pending lint, for unload. */
  dispose(): void {
    for (const t of this.pending.values()) clearTimeout(t);
    this.pending.clear();
  }

  /** Lint content read after the idle wait; write only if it is still current. */
  private async lintIdle(file: F): Promise<void> {
    const cfg = await this.activeConfig();
    if (!cfg) return;
    const content = await this.host.read(file);
    if (!this.guard.shouldProcess(file.path, content)) return;
    const api = await this.host.loadWasm();
    const r = lintNote(api, cfg.text, content, file.path, this.host.today ? this.host.today() : localDate());
    if (!r.ok) return;
    if (r.text === content) {
      this.guard.recordOutput(file.path, r.text);
      return;
    }
    // A newer modify may have landed while linting; process sees the live text.
    let applied = false;
    await this.host.process(file, (data) => {
      if (data !== content) return data;
      applied = true;
      return r.text;
    });
    if (applied) this.guard.recordOutput(file.path, r.text);
  }

  /** Write the starter config. Never overwrites, and refuses invalid output. */
  async createConfig(): Promise<'created' | 'exists' | 'invalid'> {
    if (await anyConfigExists(this.host.fs)) {
      this.host.notice(`A Lynter config already exists at the vault root.`);
      return 'exists';
    }
    const api = await this.host.loadWasm();
    const text = api.scaffold();
    const v = validateConfig(api, text);
    if (!v.ok) {
      this.host.notice(`Lynter did not write ${CONFIG_NAME}: ${v.errors[0] ?? 'invalid starter config'}`);
      return 'invalid';
    }
    await this.host.fs.write(CONFIG_NAME, text);
    this.guard.clear();
    this.say(`Lynter: created ${CONFIG_NAME}.`);
    return 'created';
  }

  async validateActive(): Promise<{config: FoundConfig | null; validation: Validation | null}> {
    const config = await discoverConfig(this.host.fs);
    if (!config) return {config: null, validation: null};
    const api = await this.host.loadWasm();
    return {config, validation: validateConfig(api, config.text)};
  }
}
