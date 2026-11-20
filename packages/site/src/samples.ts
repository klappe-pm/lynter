/** Seed content for the playground and scope demo. Every rule id is real. */

export const sampleConfig = `version: 1
defaults:
  rules:
    yaml-key-sort:
      enabled: true
      priority-keys: [title, status, tags]
    alphabetize-property-values:
      enabled: true
    header-case:
      enabled: true
      h2: First letter
    strip-strong:
      enabled: true
    contractions:
      enabled: true
profiles:
  client-notes:
    match:
      path: ["work/clients/**"]
    rules:
      contractions:
        enabled: false
`;

export const sampleNote = `---
tags: [weekly, review, planning]
status: draft
title: Weekly review
---
# Weekly review

## What Went Well This Week

It is **really** important that we do not lose momentum.
We shipped the importer and it is already in use.
`;

export const samplePath = 'notes/weekly-review.md';

/** Nested profiles from the README, used by the scope picker. */
export const scopeConfig = `version: 1
defaults:
  rules:
    contractions:
      enabled: true
profiles:
  work:
    match:
      path: ["work/**"]
    rules:
      contractions:
        enabled: false
  projects:
    match:
      path: ["work/projects/**"]
    rules:
      contractions:
        enabled: true
  client-drafts:
    match:
      path: ["work/projects/client/**"]
    rules:
      contractions:
        enabled: false
`;

/** Folder tree for the scope picker; depth drives indentation. */
export const scopeTree: {path: string; depth: number}[] = [
  {path: 'journal', depth: 0},
  {path: 'work', depth: 0},
  {path: 'work/meetings', depth: 1},
  {path: 'work/projects', depth: 1},
  {path: 'work/projects/client', depth: 2},
];

/** Whether contractions end up on for a chain, mirroring the config above. */
export function contractionsFor(chain: string[]): boolean {
  const settings: Record<string, boolean> = {
    'defaults': true,
    'work': false,
    'projects': true,
    'client-drafts': false,
  };
  return chain.reduce((on, name) => (name in settings ? settings[name] : on), true);
}

/** The CLI binary name. Change CLI_BIN and every snippet follows. */
export const CLI_BIN = 'lynter';

export const cliSnippet = `# check notes git sees as changed, before a commit
${CLI_BIN} check --changed

# fix a folder in place
${CLI_BIN} fix notes/

# show which profiles govern a path
${CLI_BIN} explain work/projects/client/brief.md`;
