import {gzipSync} from 'fflate';
import {base64ToBytes, inflateBase64Gzip} from '../src/codec';
import {initCalls} from './glue-stub';
import {loadWasm} from '../src/wasm';

describe('base64 and gzip round trip', () => {
  it('decodes base64 to the same bytes', () => {
    const bytes = new Uint8Array(300).map((_, i) => (i * 7) % 256);
    const b64 = Buffer.from(bytes).toString('base64');
    expect(Array.from(base64ToBytes(b64))).toEqual(Array.from(bytes));
  });

  it('inflates what the build step embeds', () => {
    const wasm = new Uint8Array([0, 97, 115, 109, 1, 0, 0, 0, ...new Array(500).fill(9)]);
    const b64 = Buffer.from(gzipSync(wasm, {level: 9})).toString('base64');
    expect(Array.from(inflateBase64Gzip(b64))).toEqual(Array.from(wasm));
  });

  it('loads lazily, once, passing inflated bytes to init', async () => {
    expect(initCalls).toHaveLength(0);
    const [a, b] = await Promise.all([loadWasm(), loadWasm()]);
    expect(a).toBe(b);
    expect(initCalls).toHaveLength(1);
    const arg = initCalls[0] as {module_or_path: Uint8Array};
    expect(Array.from(arg.module_or_path)).toEqual([0, 97, 115, 109]);
  });
});
