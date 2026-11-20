import {PluginSettingTab, Setting, type App} from 'obsidian';
import {CONFIG_NAME} from './config';
import type LynterPlugin from './main';

export interface LynterSettings {
  lintOnSave: boolean;
  showNotices: boolean;
}

export const DEFAULT_SETTINGS: LynterSettings = {
  lintOnSave: false,
  showNotices: true,
};

export class LynterSettingTab extends PluginSettingTab {
  constructor(app: App, private plugin: LynterPlugin) {
    super(app, plugin);
  }

  display(): void {
    const {containerEl} = this;
    containerEl.empty();

    new Setting(containerEl)
        .setName('Lint on save')
        .setDesc('Tidy a note when Obsidian saves it. Lynter skips its own writes and notes that are already tidy.')
        .addToggle((t) => t.setValue(this.plugin.settings.lintOnSave).onChange(async (v) => {
          this.plugin.settings.lintOnSave = v;
          await this.plugin.saveSettings();
        }));

    new Setting(containerEl)
        .setName('Show notices')
        .setDesc('Show a notice after each lint command.')
        .addToggle((t) => t.setValue(this.plugin.settings.showNotices).onChange(async (v) => {
          this.plugin.settings.showNotices = v;
          await this.plugin.saveSettings();
        }));

    new Setting(containerEl).setName('Configuration').setHeading();
    const status = containerEl.createDiv();
    const view = containerEl.createEl('pre', {cls: 'lynter-config-view'});
    status.setText('Checking the config...');

    new Setting(containerEl)
        .setName(`Open ${CONFIG_NAME}`)
        .setDesc('Rules live in this file, not in plugin settings. Edit it in any text editor.')
        .addButton((b) => b.setButtonText(`Open ${CONFIG_NAME}`).onClick(() => this.plugin.openConfig()));

    void this.refresh(status, view);
  }

  private async refresh(status: HTMLElement, view: HTMLElement): Promise<void> {
    try {
      const {config, validation} = await this.plugin.lynter.validateActive();
      if (!config || !validation) {
        status.setText(`No ${CONFIG_NAME} at the vault root. Run the "Create ${CONFIG_NAME}" command.`);
        view.setText('');
        return;
      }
      view.setText(config.text);
      status.empty();
      status.createSpan({text: `Active config: ${config.name}. `});
      if (config.deprecated) status.createSpan({text: `Deprecated name, rename it to ${CONFIG_NAME}. `});
      if (validation.ok) {
        status.createSpan({cls: 'lynter-validation-ok', text: 'The config is valid.'});
      } else {
        status.createSpan({cls: 'lynter-validation-error', text: 'The config has errors:'});
        const ul = status.createEl('ul');
        validation.errors.forEach((e) => ul.createEl('li', {text: e}));
      }
    } catch (e) {
      status.setText(`Lynter could not check the config: ${e instanceof Error ? e.message : String(e)}`);
    }
  }
}

