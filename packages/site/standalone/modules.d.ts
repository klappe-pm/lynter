declare module 'lynter-wasm-gz' {
  /** The engine's .wasm file, gzipped and base64 encoded at build time. */
  const packed: string;
  export default packed;
}

declare module '*/lynter_wasm.mjs' {
  export function initSync(input: {module: ArrayBuffer}): void;
  export function lint(config: string, note: string, path: string, today: string): string;
  export function validate_config(yaml: string): string;
  export function profile_chain(config: string, path: string): string;
  export function convert_data_json(json: string): string | undefined;
  export function scaffold(): string;
  export function version(): string;
}
