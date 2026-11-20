import {gunzipSync} from 'fflate';

/** Decode base64 text to bytes without Buffer, so it works on mobile. */
export function base64ToBytes(b64: string): Uint8Array {
  const bin = atob(b64);
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) {
    out[i] = bin.charCodeAt(i);
  }
  return out;
}

/** Decode base64 and gunzip it: the shape the build step embeds. */
export function inflateBase64Gzip(b64: string): Uint8Array {
  return gunzipSync(base64ToBytes(b64));
}
