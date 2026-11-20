import {DEBOUNCE_MS, Lynter} from '../src/lynter';
import {lintJson, makeFake} from './helpers';

const trim = (_c: string, note: string) => lintJson(note.trim() + '\n');

function setup(activePath: string | null) {
  const lint = jest.fn(trim);
  const fake = makeFake({lint});
  fake.disk.set('lynter.yaml', 'cfg');
  fake.host.isActive = (f) => f.path === activePath;
  return {lint, fake, l: new Lynter(fake.host)};
}

/** Let queued microtasks (awaits inside the timer callback) finish. */
const flush = async () => {
  for (let i = 0; i < 10; i++) await Promise.resolve();
};

describe('debounced lint on save for the active file', () => {
  beforeEach(() => jest.useFakeTimers());
  afterEach(() => jest.useRealTimers());

  it('waits for the idle period before linting', async () => {
    const {lint, fake, l} = setup('a.md');
    fake.files.set('a.md', 'hi  ');
    expect(await l.onModify({path: 'a.md'})).toBe(false);
    jest.advanceTimersByTime(DEBOUNCE_MS - 1);
    await flush();
    expect(lint).not.toHaveBeenCalled();
    expect(fake.files.get('a.md')).toBe('hi  ');
    jest.advanceTimersByTime(1);
    await flush();
    expect(lint).toHaveBeenCalledTimes(1);
    expect(fake.files.get('a.md')).toBe('hi\n');
  });

  it('restarts the wait on a newer modify of the same path', async () => {
    const {lint, fake, l} = setup('a.md');
    fake.files.set('a.md', 'a ');
    await l.onModify({path: 'a.md'});
    jest.advanceTimersByTime(1500);
    fake.files.set('a.md', 'ab ');
    await l.onModify({path: 'a.md'});
    jest.advanceTimersByTime(1500);
    await flush();
    expect(lint).not.toHaveBeenCalled();
    jest.advanceTimersByTime(500);
    await flush();
    expect(lint).toHaveBeenCalledTimes(1);
    expect(lint).toHaveBeenCalledWith('cfg', 'ab ', 'a.md', '2026-01-02');
  });

  it('does not apply the result when the text changed while linting', async () => {
    const {fake, l} = setup('a.md');
    fake.files.set('a.md', 'hi  ');
    await l.onModify({path: 'a.md'});
    const realProcess = fake.host.process;
    fake.host.process = async (f, fn) => {
      // The person types between the read and the write.
      fake.files.set('a.md', 'hi there');
      return realProcess(f, fn);
    };
    jest.advanceTimersByTime(DEBOUNCE_MS);
    await flush();
    expect(fake.files.get('a.md')).toBe('hi there');
    // Nothing was recorded, so the next modify event lints the new text.
    expect(l.guard.shouldProcess('a.md', 'hi there')).toBe(true);
  });

  it('lints background files immediately', async () => {
    const {lint, fake, l} = setup('other.md');
    fake.files.set('a.md', 'hi  ');
    expect(await l.onModify({path: 'a.md'})).toBe(true);
    expect(lint).toHaveBeenCalledTimes(1);
  });

  it('skips its own write event on the active file', async () => {
    const {lint, fake, l} = setup('a.md');
    fake.files.set('a.md', 'hi  ');
    await l.onModify({path: 'a.md'});
    jest.advanceTimersByTime(DEBOUNCE_MS);
    await flush();
    // The write fires a modify event, which schedules a lint that then skips.
    await l.onModify({path: 'a.md'});
    jest.advanceTimersByTime(DEBOUNCE_MS);
    await flush();
    expect(lint).toHaveBeenCalledTimes(1);
  });

  it('cancels pending lints on dispose and on cancel', async () => {
    const {lint, fake, l} = setup('a.md');
    fake.files.set('a.md', 'hi  ');
    await l.onModify({path: 'a.md'});
    l.cancel('a.md');
    jest.advanceTimersByTime(DEBOUNCE_MS);
    await flush();
    await l.onModify({path: 'a.md'});
    l.dispose();
    jest.advanceTimersByTime(DEBOUNCE_MS);
    await flush();
    expect(lint).not.toHaveBeenCalled();
  });
});
