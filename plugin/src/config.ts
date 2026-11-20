/** Config file names in discovery order. The first is canonical. */
export const CONFIG_NAME = 'lynter.yaml';
export const DEPRECATED_CONFIG_NAMES = ['linter.yaml', 'lappe-linter.yaml'] as const;
export const CONFIG_CANDIDATES = [CONFIG_NAME, ...DEPRECATED_CONFIG_NAMES];

export interface ConfigFs {
  exists(path: string): Promise<boolean>;
  read(path: string): Promise<string>;
}

export interface FoundConfig {
  name: string;
  text: string;
  deprecated: boolean;
}

/** Find the active config at the vault root, or null when there is none. */
export async function discoverConfig(fs: ConfigFs): Promise<FoundConfig | null> {
  for (const name of CONFIG_CANDIDATES) {
    if (await fs.exists(name)) {
      return {name, text: await fs.read(name), deprecated: name !== CONFIG_NAME};
    }
  }
  return null;
}

export async function anyConfigExists(fs: ConfigFs): Promise<boolean> {
  for (const name of CONFIG_CANDIDATES) {
    if (await fs.exists(name)) return true;
  }
  return false;
}
