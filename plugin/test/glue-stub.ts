export const initCalls: unknown[] = [];
export default async function init(input?: unknown): Promise<void> {
  initCalls.push(input);
}
export function lint(): string {
  return JSON.stringify({ok: true, text: '', changed: false, rules: []});
}
export function scaffold(): string {
  return 'version: 1\n';
}
export function version(): string {
  return 'test';
}
