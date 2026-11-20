import {anyConfigExists, CONFIG_CANDIDATES, discoverConfig} from '../src/config';
import {Lynter} from '../src/lynter';
import {makeFake} from './helpers';

describe('config discovery order', () => {
  const fsOf = (names: string[]) => {
    const m = new Map(names.map((n) => [n, `from ${n}`]));
    return {exists: async (p: string) => m.has(p), read: async (p: string) => m.get(p) ?? ''};
  };

  it('prefers lynter.yaml over the deprecated aliases', async () => {
    const found = await discoverConfig(fsOf(['lappe-linter.yaml', 'linter.yaml', 'lynter.yaml']));
    expect(found).toEqual({name: 'lynter.yaml', text: 'from lynter.yaml', deprecated: false});
  });

  it('reads linter.yaml before lappe-linter.yaml', async () => {
    const found = await discoverConfig(fsOf(['lappe-linter.yaml', 'linter.yaml']));
    expect(found?.name).toBe('linter.yaml');
    expect(found?.deprecated).toBe(true);
  });

  it('falls back to lappe-linter.yaml', async () => {
    expect((await discoverConfig(fsOf(['lappe-linter.yaml'])))?.name).toBe('lappe-linter.yaml');
  });

  it('returns null with no config', async () => {
    expect(await discoverConfig(fsOf([]))).toBeNull();
    expect(await anyConfigExists(fsOf([]))).toBe(false);
  });

  it('lists the names in order', () => {
    expect(CONFIG_CANDIDATES).toEqual(['lynter.yaml', 'linter.yaml', 'lappe-linter.yaml']);
  });

  it('shows the deprecated notice once', async () => {
    const fake = makeFake({});
    fake.disk.set('linter.yaml', 'x');
    const l = new Lynter(fake.host);
    await l.activeConfig();
    await l.activeConfig();
    expect(fake.notices.filter((n) => n.includes('linter.yaml'))).toHaveLength(1);
  });

  it('shows no deprecated notice for lynter.yaml', async () => {
    const fake = makeFake({});
    fake.disk.set('lynter.yaml', 'x');
    await new Lynter(fake.host).activeConfig();
    expect(fake.notices).toHaveLength(0);
  });
});
