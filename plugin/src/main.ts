import {Notice, Plugin, TFile, type Editor, type MarkdownFileInfo, type MarkdownView} from 'obsidian';
import {CONFIG_CANDIDATES, CONFIG_NAME} from './config';
import {Lynter, type Host} from './lynter';
import {DEFAULT_SETTINGS, LynterSettingTab, type LynterSettings} from './settings';
import {loadWasm} from './wasm';

export default class LynterPlugin extends Plugin {
  settings: LynterSettings = {...DEFAULT_SETTINGS};
  lynter!: Lynter<TFile>;

  async onload(): Promise<void> {
    await this.loadSettings();

    const host: Host<TFile> = {
      fs: this.app.vault.adapter,
      markdownFiles: () => this.app.vault.getMarkdownFiles(),
      process: (file, fn) => this.app.vault.process(file, fn),
      read: (file) => this.app.vault.cachedRead(file),
      loadWasm,
      notice: (message, timeout) => new Notice(message, timeout),
      showNotices: () => this.settings.showNotices,
      isActive: (file) => this.app.workspace.getActiveFile()?.path === file.path,
      yieldToUi: () => new Promise((resolve) => window.setTimeout(resolve, 0)),
    };
    this.lynter = new Lynter(host);

    this.addRibbonIcon('wand-sparkles', 'Lint current file with Lynter', () => {
      void this.guarded(() => this.lynter.lintActive(this.app.workspace.getActiveFile()));
    });

    this.addCommand({
      id: 'lint-current-file',
      name: 'Lint current file',
      editorCheckCallback: (checking: boolean, _editor: Editor, ctx: MarkdownView | MarkdownFileInfo) => {
        if (!ctx.file) return false;
        if (!checking) void this.guarded(() => this.lynter.lintActive(ctx.file));
        return true;
      },
    });
    this.addCommand({
      id: 'lint-all-files',
      name: 'Lint all files in the vault',
      callback: () => void this.guarded(() => this.lynter.lintAll()),
    });
    this.addCommand({
      id: 'create-config',
      name: `Create ${CONFIG_NAME}`,
      callback: () => void this.guarded(() => this.lynter.createConfig()),
    });

    this.registerEvent(this.app.vault.on('modify', (file) => {
      if (CONFIG_CANDIDATES.includes(file.path)) this.lynter.guard.clear();
      if (!this.settings.lintOnSave || !(file instanceof TFile) || file.extension !== 'md') return;
      void this.guarded(() => this.lynter.onModify(file));
    }));
    this.registerEvent(this.app.vault.on('rename', (file, oldPath) => this.lynter.guard.rename(oldPath, file.path)));
    this.registerEvent(this.app.vault.on('delete', (file) => {
      this.lynter.cancel(file.path);
      this.lynter.guard.forget(file.path);
    }));
    this.register(() => this.lynter.dispose());

    this.addSettingTab(new LynterSettingTab(this.app, this));
  }

  /** Surface unexpected failures instead of losing them in a floating promise. */
  private async guarded(fn: () => Promise<unknown>): Promise<void> {
    try {
      await fn();
    } catch (e) {
      console.error('Lynter', e);
      new Notice(`Lynter failed: ${e instanceof Error ? e.message : String(e)}`);
    }
  }

  async openConfig(): Promise<void> {
    const found = await this.lynter.activeConfig();
    const name = found?.name ?? CONFIG_NAME;
    const app = this.app as unknown as {openWithDefaultApp?: (path: string) => Promise<void>};
    if (found && typeof app.openWithDefaultApp === 'function') {
      await app.openWithDefaultApp(name);
    } else {
      new Notice(found ? `Open ${name} in a text editor from the vault folder.` : `No ${CONFIG_NAME} yet. Run "Create ${CONFIG_NAME}" first.`);
    }
  }

  async loadSettings(): Promise<void> {
    const saved = (await this.loadData()) as Partial<LynterSettings> | null;
    this.settings = {
      lintOnSave: typeof saved?.lintOnSave === 'boolean' ? saved.lintOnSave : DEFAULT_SETTINGS.lintOnSave,
      showNotices: typeof saved?.showNotices === 'boolean' ? saved.showNotices : DEFAULT_SETTINGS.showNotices,
    };
  }

  async saveSettings(): Promise<void> {
    await this.saveData(this.settings);
  }
}
