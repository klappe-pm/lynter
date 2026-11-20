/**
 * Stops lint on save from looping. Lynter writes a file, which fires another
 * modify event; the guard remembers the text it wrote and skips that event,
 * and skips any content that is already the lint output.
 */
export class SaveGuard {
  private written = new Map<string, string>();

  /** True when this content should be linted. */
  shouldProcess(path: string, content: string): boolean {
    return this.written.get(path) !== content;
  }

  /** Record the text Lynter produced for a path, whether or not it wrote. */
  recordOutput(path: string, text: string): void {
    this.written.set(path, text);
  }

  /** Drop everything, for example after the config changes. */
  clear(): void {
    this.written.clear();
  }

  forget(path: string): void {
    this.written.delete(path);
  }

  rename(from: string, to: string): void {
    const v = this.written.get(from);
    if (v !== undefined) {
      this.written.delete(from);
      this.written.set(to, v);
    }
  }
}
