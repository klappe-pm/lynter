import {SaveGuard} from '../src/save-guard';
import {Lynter} from '../src/lynter';
import {lintJson, makeFake} from './helpers';

describe('SaveGuard', () => {
  it('skips content equal to the last recorded output', () => {
    const g = new SaveGuard();
    expect(g.shouldProcess('a.md', 'x')).toBe(true);
    g.recordOutput('a.md', 'x');
    expect(g.shouldProcess('a.md', 'x')).toBe(false);
    expect(g.shouldProcess('a.md', 'y')).toBe(true);
    expect(g.shouldProcess('b.md', 'x')).toBe(true);
  });

  it('follows renames and forgets deletes', () => {
    const g = new SaveGuard();
    g.recordOutput('a.md', 'x');
    g.rename('a.md', 'b.md');
    expect(g.shouldProcess('b.md', 'x')).toBe(false);
    g.forget('b.md');
    expect(g.shouldProcess('b.md', 'x')).toBe(true);
  });
});

describe('lint on save loop guard', () => {
  it('does not re-lint its own write', async () => {
    const lint = jest.fn((_c: string, note: string) => lintJson(note.trim() + '\n'));
    const fake = makeFake({lint});
    fake.disk.set('lynter.yaml', 'version: 1\n');
    fake.files.set('a.md', '# a  \n\n\n');
    const l = new Lynter(fake.host);
    const file = {path: 'a.md'};

    expect(await l.onModify(file)).toBe(true);
    expect(fake.files.get('a.md')).toBe('# a\n');
    // The write fires a modify event: it must be skipped.
    expect(await l.onModify(file)).toBe(false);
    expect(lint).toHaveBeenCalledTimes(1);
  });

  it('skips when the content is unchanged by lint, on the next save too', async () => {
    const lint = jest.fn((_c: string, note: string) => lintJson(note, false));
    const fake = makeFake({lint});
    fake.disk.set('lynter.yaml', 'version: 1\n');
    fake.files.set('a.md', 'tidy\n');
    const l = new Lynter(fake.host);
    expect(await l.onModify({path: 'a.md'})).toBe(false);
    expect(await l.onModify({path: 'a.md'})).toBe(false);
    expect(lint).toHaveBeenCalledTimes(1);
  });

  it('lints again after the person edits the note', async () => {
    const lint = jest.fn((_c: string, note: string) => lintJson(note.trim() + '\n'));
    const fake = makeFake({lint});
    fake.disk.set('lynter.yaml', 'version: 1\n');
    fake.files.set('a.md', 'a \n');
    const l = new Lynter(fake.host);
    await l.onModify({path: 'a.md'});
    fake.files.set('a.md', 'b \n');
    expect(await l.onModify({path: 'a.md'})).toBe(true);
    expect(lint).toHaveBeenCalledTimes(2);
  });

  it('lints again when the config changes', async () => {
    const lint = jest.fn((_c: string, note: string) => lintJson(note, false));
    const fake = makeFake({lint});
    fake.disk.set('lynter.yaml', 'version: 1\n');
    fake.files.set('a.md', 'tidy\n');
    const l = new Lynter(fake.host);
    await l.onModify({path: 'a.md'});
    fake.disk.set('lynter.yaml', 'version: 1\n# changed\n');
    await l.onModify({path: 'a.md'});
    expect(lint).toHaveBeenCalledTimes(2);
  });
});
