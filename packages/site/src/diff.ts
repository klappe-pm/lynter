/**
 * Line diff for the playground: which output lines are new or changed
 * relative to the input. Longest common subsequence over lines; notes are
 * small, so the quadratic table is fine.
 * @param {string} before input text
 * @param {string} after output text
 * @return {boolean[]} one flag per output line, true when the line changed
 */
export function changedLines(before: string, after: string): boolean[] {
  const a = before.split('\n');
  const b = after.split('\n');
  const rows = a.length + 1;
  const cols = b.length + 1;
  const table = new Uint32Array(rows * cols);
  for (let i = a.length - 1; i >= 0; i--) {
    for (let j = b.length - 1; j >= 0; j--) {
      table[i * cols + j] = a[i] === b[j] ?
        table[(i + 1) * cols + j + 1] + 1 :
        Math.max(table[(i + 1) * cols + j], table[i * cols + j + 1]);
    }
  }
  const changed = b.map(() => true);
  let i = 0;
  let j = 0;
  while (i < a.length && j < b.length) {
    if (a[i] === b[j]) {
      changed[j] = false;
      i++;
      j++;
    } else if (table[(i + 1) * cols + j] >= table[i * cols + j + 1]) {
      i++;
    } else {
      j++;
    }
  }
  return changed;
}
