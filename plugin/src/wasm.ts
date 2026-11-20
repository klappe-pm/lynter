import init, * as glue from 'lynter-wasm-glue';
import b64 from 'lynter-wasm-b64';
import {inflateBase64Gzip} from './codec';
import type {WasmApi} from './engine';

let loading: Promise<WasmApi> | undefined;

/** Load the inline WASM once, on first use. Retries after a failure. */
export function loadWasm(): Promise<WasmApi> {
  if (!loading) {
    loading = (async () => {
      const bytes = inflateBase64Gzip(b64);
      await init({module_or_path: bytes});
      return glue as unknown as WasmApi;
    })();
    loading.catch(() => {
      loading = undefined;
    });
  }
  return loading;
}
