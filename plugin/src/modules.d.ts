declare module 'lynter-wasm-glue' {
  export default function init(input?: {module_or_path: Uint8Array | ArrayBuffer}): Promise<unknown>;
  export function lint(configYaml: string, note: string, path: string, today: string): string;
  export function scaffold(): string;
  export function validate_config(yaml: string): string;
  export function version(): string;
  export function profile_chain(configYaml: string, path: string): string;
  export function convert_data_json(json: string): string | undefined;
}

declare module 'lynter-wasm-b64' {
  const b64: string;
  export default b64;
}
