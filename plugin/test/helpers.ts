import type {FileRef, Host, NoticeHandle} from '../src/lynter';
import type {WasmApi} from '../src/engine';

export interface Fake {
  host: Host<FileRef>;
  files: Map<string, string>;
  disk: Map<string, string>;
  notices: string[];
  processCalls: string[];
  yields: number;
  progress: string[];
}

export function lintJson(text: string, changed = true): string {
  return JSON.stringify({ok: true, text, changed, rules: []});
}

/** A fake host: `disk` holds root files, `files` holds notes. */
export function makeFake(api: Partial<WasmApi>, opts: {showNotices?: boolean} = {}): Fake {
  const disk = new Map<string, string>();
  const files = new Map<string, string>();
  const fake: Fake = {
    host: undefined as unknown as Host<FileRef>,
    files,
    disk,
    notices: [],
    processCalls: [],
    yields: 0,
    progress: [],
  };
  fake.host = {
    fs: {
      exists: async (p) => disk.has(p),
      read: async (p) => disk.get(p) ?? '',
      write: async (p, d) => void disk.set(p, d),
    },
    markdownFiles: () => [...files.keys()].map((path) => ({path})),
    process: async (f, fn) => {
      fake.processCalls.push(f.path);
      const next = fn(files.get(f.path) ?? '');
      files.set(f.path, next);
      return next;
    },
    read: async (f) => files.get(f.path) ?? '',
    loadWasm: async () => ({scaffold: () => 'version: 1\n', ...api}) as WasmApi,
    notice: (m): NoticeHandle => {
      fake.notices.push(m);
      return {
        setMessage: (s) => void fake.progress.push(s),
        hide: () => undefined,
      };
    },
    showNotices: () => opts.showNotices ?? true,
    yieldToUi: async () => void fake.yields++,
    today: () => '2026-01-02',
  };
  return fake;
}
