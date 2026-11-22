import {sampleConfig, sampleNote, samplePath} from './samples';

/** Before and after gallery. Every config uses rule ids that exist in the core. */
export interface Example {
  id: string;
  title: string;
  blurb: string;
  path: string;
  config: string;
  note: string;
}

export const examples: Example[] = [
  {
    id: 'weekly-review',
    title: 'Weekly review',
    blurb: 'Sorted frontmatter, sentence case headings, no bold emphasis, and contractions.',
    path: samplePath,
    config: sampleConfig,
    note: sampleNote,
  },
  {
    id: 'client-formal',
    title: 'Client folder stays formal',
    blurb: 'A scope on the client folder turns contractions off. This path is outside it, so contractions apply. In the playground, change the path to work/clients/acme/brief.md and the text stays formal.',
    path: 'work/internal/brief.md',
    config: `version: 1
defaults:
  rules:
    contractions:
      enabled: true
profiles:
  client:
    match:
      path: ["work/clients/**"]
    rules:
      contractions:
        enabled: false
`,
    note: `# Acme brief

We do not expect changes to the scope. It is ready for review.
`,
  },
  {
    id: 'em-dashes',
    title: 'Em dashes replaced',
    blurb: 'Punctuation that reads as machine written becomes plain commas and periods.',
    path: 'notes/status.md',
    config: `version: 1
defaults:
  rules:
    replace-em-dash:
      enabled: true
`,
    note: `# Status

The import finished — every file moved.
Two notes need a manual check — they have no title.
`,
  },
  {
    id: 'lists-and-spacing',
    title: 'Lists and spacing',
    blurb: 'Dash bullets, tight lists, and one blank line between blocks.',
    path: 'notes/shopping.md',
    config: `version: 1
defaults:
  rules:
    paragraph-spacing:
      enabled: true
      blank-lines: 1
    list-style:
      enabled: true
      marker: '-'
      tighten: true
`,
    note: `# Shopping
Things to buy:


* Bread

* Milk
* Eggs
## Later
Nothing yet.
`,
  },
  {
    id: 'sorted-tags',
    title: 'Sorted properties',
    blurb: 'Frontmatter keys follow a priority order and list values sort alphabetically.',
    path: 'notes/reading.md',
    config: `version: 1
defaults:
  rules:
    yaml-key-sort:
      enabled: true
      priority-keys: [title, status, tags]
    alphabetize-property-values:
      enabled: true
`,
    note: `---
status: reading
tags: [zettel, books, ideas]
title: Reading list
---
# Reading list
`,
  },
  {
    id: 'agent-output',
    title: 'Agent output in house style',
    blurb: 'Hard wrapped lines join, bold text goes, and the memory scope sorts its keys.',
    path: 'memory/project-notes.md',
    config: `version: 1
defaults:
  rules:
    strip-strong:
      enabled: true
profiles:
  memory:
    match:
      path: ["memory/**"]
    rules:
      join-paragraph-lines-fix:
        enabled: true
      yaml-key-sort:
        enabled: true
        priority-keys: [name, description, type]
`,
    note: `---
type: project
name: project-notes
---
# Project notes

Run the **full suite** before a push. It takes
about two minutes on a laptop.
`,
  },
  {
    id: 'code-blocks',
    title: 'Code blocks by language',
    blurb: 'Checks written in lynter.yaml run inside fenced code and are scoped by language: var becomes let in JavaScript, tabs become spaces in Python, and the Rust block is left alone.',
    path: 'docs/snippets.md',
    config: `version: 1
defaults:
  rules:
    code-checks:
      enabled: true
    code-checks-fix:
      enabled: true
code-checks:
  no-trailing-whitespace-in-code:
    enabled: true
  let-not-var:
    enabled: true
    languages: [javascript, typescript]
    pattern: '\\bvar\\b'
    message: Use let or const.
    fix:
      replacement: let
  tabs-to-spaces:
    enabled: true
    languages: [python]
    pattern: '\\t'
    message: Indent Python with spaces.
    fix:
      replacement: '    '
`,
    note: `# Snippets

\`\`\`javascript
var total = 0;
for (var i = 0; i < 3; i++) total += i;
\`\`\`

\`\`\`python
def greet(name):
	return f"hello {name}"   
\`\`\`

\`\`\`rust
let total = 0;
\`\`\`
`,
  },
];

export function findExample(id: string): Example | undefined {
  return examples.find((e) => e.id === id);
}
