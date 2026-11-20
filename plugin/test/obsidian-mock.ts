/** A minimal hand-written stand-in for the parts of Obsidian Lynter imports. */
export class Notice {
  static log: string[] = [];
  constructor(public message: string, public timeout?: number) {
    Notice.log.push(message);
  }
  setMessage(m: string): this {
    this.message = m;
    return this;
  }
  hide(): void {}
}

export class TFile {
  extension = 'md';
  constructor(public path: string) {}
}

export class Plugin {
  constructor(public app: unknown, public manifest: unknown) {}
}

export class PluginSettingTab {
  containerEl = {empty: () => undefined};
  constructor(public app: unknown, public plugin: unknown) {}
}

export class Setting {}

export const Platform = {isMobile: false};
