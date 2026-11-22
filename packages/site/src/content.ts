/**
 * Every visible string and outbound link on lynter.dev lives here, so copy
 * can be reviewed in one file and pages stay layout only. Long text lives in
 * content/*.md.
 */

/** Where "Install" points. Not in the community directory yet, so this opens the install steps. */
export const INSTALL_URL = '/getting-started/#install';
/** Public origin, used for canonical links and social card URLs, which must be absolute. */
export const SITE_URL = 'https://lynter.dev';
export const LAB_URL = 'https://lappelabs.dev';
export const REPO_URL = 'https://github.com/klappe-pm/lynter';
/** The Obsidian product site. The first "Obsidian" in the home lede links here. */
export const OBSIDIAN_URL = 'https://obsidian.md';
/** Opens a new GitHub issue from the feature request template. */
export const FEATURE_REQUEST_URL = `${REPO_URL}/issues/new?template=feature_request.md`;

/** Optional Buttondown username for the notify form; hidden when unset. */
export const NOTIFY_USER = process.env.NEXT_PUBLIC_BUTTONDOWN_USER || '';

export const site = {
  name: 'Lynter',
  title: 'Lynter: one lint config for your notes and your code',
  description: 'Lynter applies one reviewed lynter.yaml in Obsidian, the CLI, CI, and agent workflows, scoped to each folder, tag, or language.',
  nav: [
    {href: '/getting-started/', label: 'Getting started'},
    {href: '/examples/', label: 'Examples'},
    {href: '/use-cases/', label: 'Use cases'},
    {href: '/docs/', label: 'Docs'},
    {href: '/playground/', label: 'Playground'},
    {href: '/configure/', label: 'Configure'},
  ],
  install: 'Install',
  footerLinks: [
    {href: '/obsidian/', label: 'Obsidian'},
    {href: '/languages/', label: 'Languages'},
    {href: '/integrations/llm/', label: 'LLM hooks'},
    {href: '/integrations/ci/', label: 'CI'},
    {href: '/samples/', label: 'Samples'},
    {href: '/faq/', label: 'FAQ'},
    {href: '/roadmap/', label: 'Roadmap'},
    {href: FEATURE_REQUEST_URL, label: 'Request a feature'},
    {href: '/import/', label: 'Import settings'},
  ],
  footer: {
    project: 'A LappeLabs project',
  },
};

export const home = {
  headline: 'One file. Every note. Every line of code.',
  lede: 'Lynter applies one reviewed lynter.yaml in Obsidian, the CLI, CI, and agent workflows, scoped to each folder, tag, or language.',
  secondary: 'Open the playground',
  benchNote: 'Edit the config or the note. This is the real rule engine, running in your browser.',
  pillars: [
    {
      title: 'The config is a file',
      body: 'Rules live in lynter.yaml at the root of your vault or repository, not in plugin settings. Every change shows up in a diff and goes through review like code.',
    },
    {
      title: 'Scopes inherit',
      body: 'Match folders, tags, properties, or languages. Each scope starts from the base and states only what differs, so a client folder can stay formal while the rest stays casual.',
    },
    {
      title: 'Same result everywhere',
      body: 'Obsidian, the command line, CI, and agent hooks run one rule core. A note that passes in Obsidian passes in the pre-commit hook and in CI, with no API tokens.',
    },
  ],
  scope: {
    title: 'Pick a folder to see which rules apply',
    body: 'Deeper scopes override broader ones. This is the same resolution the explain command prints for a path.',
    chainLabel: 'Profile chain',
    contractionsLabel: 'Contractions',
    on: 'on',
    off: 'off',
  },
  useCases: {
    title: 'Use cases',
    body: 'Lynter fits any folder where text has to follow a house style.',
    items: [
      {title: 'Personal vault', body: 'Consistent headings, tags, and frontmatter across years of notes.', href: '/use-cases/#personal-vault'},
      {title: 'Shared team vault', body: 'Review style changes in a pull request, not in a settings screen.', href: '/use-cases/#shared-team-vault'},
      {title: 'Client formal tone', body: 'Turn contractions off for one folder and leave the rest alone.', href: '/use-cases/#client-formal-tone'},
      {title: 'Docs as code', body: 'Check docs in the same CI run as the code beside them.', href: '/use-cases/#docs-as-code'},
      {title: 'Agent memory', body: 'Put what agents write into house style before anyone reads it.', href: '/use-cases/#agent-memory-and-agentsmd'},
      {title: 'Zettelkasten', body: 'Keep atomic notes, ids, and tags uniform at any size.', href: '/use-cases/#zettelkasten'},
    ],
  },
  integrations: {
    title: 'Works where you work',
    body: 'One config, four places to run it.',
    items: [
      {title: 'Obsidian', body: 'Fix notes on save from the plugin.', href: '/obsidian/'},
      {title: 'CLI', body: 'check, fix, and explain from a terminal.', href: '/docs/cli/'},
      {title: 'CI and hooks', body: 'GitHub Actions, GitLab CI, pre-commit, lefthook, husky.', href: '/integrations/ci/'},
      {title: 'Agents', body: 'Claude Code, Codex, and Cursor hooks.', href: '/integrations/llm/'},
    ],
  },
  cli: {
    title: 'Run it in hooks and CI',
    body: 'check exits 1 when a file breaks a rule. fix rewrites in place. --changed limits either to the files git sees as changed.',
    action: 'Copy',
    copied: 'Copied',
  },
  deepDives: {
    title: 'How it works, in depth',
    body: 'Each page explains one part of Lynter from the source: what it does, how to configure it, and where the code lives.',
    items: [
      {title: 'Scopes and profiles', body: 'How a file picks up its rules: matchers, inheritance, and the profile chain.', href: '/docs/scopes-and-profiles/'},
      {title: 'Note types', body: 'Frontmatter schemas, required keys, key order, and filename patterns per kind of note.', href: '/docs/note-types/'},
      {title: 'Rules', body: 'Every rule, what it changes, its options, and a before and after example.', href: '/docs/rules-overview/'},
      {title: 'Code linting', body: 'Checks for fenced code, JSON and YAML data rules, code rules, and the ruff and Biome adapters.', href: '/docs/code-linting/'},
      {title: 'Configuration', body: 'How lynter.yaml is found, validated, and applied, and why a bad file changes nothing.', href: '/docs/configuration/'},
      {title: 'Architecture', body: 'One Rust core behind the command line, the Obsidian plugin, and the browser.', href: '/docs/architecture/'},
    ],
    more: 'Browse all docs',
  },
  faq: {
    title: 'Common questions',
    items: [
      {q: 'Does Lynter call a model?', a: 'No. Every rule is deterministic code. The same input gives the same output, with no tokens and no network.'},
      {q: 'Is it available in the Obsidian directory?', a: 'Not yet. You can install the plugin from a clone of the repository today.'},
      {q: 'Which languages does it lint?', a: 'The first release targets Rust, Python, TypeScript, JavaScript, SQL, Go, Java, C#, JSON, YAML, and text. Code linting is in progress.'},
    ],
    more: 'Read the full FAQ',
  },
  notify: {
    title: 'Presets and team checks are next',
    body: 'Ready-made style packs for common vault layouts, and a GitHub check that comments on pull requests. Leave an email to hear when they ship.',
    label: 'Email',
    action: 'Notify me',
  },
};

export const playground = {
  title: 'Playground',
  lede: 'Edit lynter.yaml, the note, or its path. The output updates as you type, highlighted lines changed, and the config is checked for errors, unknown keys, and misspelled rules.',
  configLabel: 'lynter.yaml',
  noteLabel: 'Note',
  pathLabel: 'Path in vault',
  outputLabel: 'Result',
  violationsLabel: 'Rules that fired',
  noViolations: 'Nothing to change. The note already follows this config.',
  configError: 'lynter.yaml has an error, so nothing ran. Fix it and the result comes back.',
  configOk: 'Valid',
  configProblems: (n: number) => (n === 1 ? '1 problem' : `${n} problems`),
  configWarnings: (n: number) => (n === 1 ? '1 warning' : `${n} warnings`),
  configLine: 'Line',
  configErrorLabel: 'Error',
  configWarningLabel: 'Warning',
  chainLabel: 'Profiles applied',
  loading: 'Loading the engine.',
  reset: 'Reset example',
  exampleLabel: 'Example',
  exampleDefault: 'Weekly review',
};

export const builderCopy = {
  title: 'Configure',
  lede: 'Build a lynter.yaml without writing YAML. Follow the guided steps or jump to any setting; the file on the right updates as you go, and you can edit it directly at any time.',
  modeLabel: 'How to work',
  guided: 'Guided',
  guidedHint: 'One step at a time, with a short explanation.',
  free: 'All settings',
  freeHint: 'Every section on one page.',
  stepsLabel: 'Steps',
  back: 'Back',
  next: 'Next',
  stepOf: (n: number, total: number) => `Step ${n} of ${total}`,
  saved: 'Draft saved in this browser.',
  start: {
    title: 'Start',
    intro: 'Pick a starting point. Your draft is kept in this browser and comes back when you return, so nothing is lost on a refresh.',
    starter: 'Recommended starter',
    starterHint: 'The commented file that the engine ships: a quiet baseline you can adjust.',
    empty: 'Empty file',
    emptyHint: 'Only the version line. Turn on exactly what you want.',
    open: 'Open my lynter.yaml',
    openHint: 'Load a file from your computer. In Chrome and Edge, changes are written straight back to it.',
    example: 'Start from an example',
    replace: 'Replace the current draft?',
    confirm: 'Replace',
    cancel: 'Keep my draft',
  },
  rules: {
    search: 'Filter rules',
    on: 'On',
    options: (n: number) => (n === 1 ? '1 option' : `${n} options`),
    remove: 'Remove from file',
    cliOnly: 'Command line only',
    reset: 'Default',
    onePerLine: 'One per line',
    yamlHint: 'YAML',
    none: 'No rule matches that filter.',
    docs: 'Read about these rules',
  },
  scopes: {
    title: 'Scopes',
    intro: 'A scope is a named set of overrides for some of your files, such as one folder. It starts from the settings above and states only what differs.',
    name: 'Scope name',
    add: 'Add scope',
    remove: 'Remove scope',
    paths: 'Folders and files (globs)',
    extensions: 'File extensions',
    tags: 'Tags',
    overrides: 'Rule overrides',
    addRule: 'Override a rule',
    choose: 'Choose a rule',
    empty: 'No scopes yet. Add one to treat a folder differently.',
    docs: 'How scopes and matching work',
  },
  files: {
    title: 'Files',
    intro: 'Which files Lynter leaves alone, and how it treats file names.',
    rename: 'File name rules',
    renameModes: {off: 'Off', flag: 'Report only', rename: 'Rename (reports only for now)'} as Record<string, string>,
    unset: 'Not set',
    folders: 'Folders to ignore',
    filesIgnored: 'Files to ignore',
  },
  finish: {
    title: 'Save',
    intro: 'Put lynter.yaml at the root of your vault or repository. The Obsidian plugin and the command line both read it from there.',
    enabled: (n: number) => (n === 1 ? '1 rule on' : `${n} rules on`),
    scopes: (n: number) => (n === 1 ? '1 scope' : `${n} scopes`),
    advancedTitle: 'Settings you edit in the file',
    advanced: 'Note types, templates, code checks, adapters and rule order have no form yet. Edit them in the lynter.yaml pane; it is checked as you type.',
    reference: 'lynter.yaml reference',
    next: 'Then run lynter check . in that folder, or enable the plugin.',
  },
  yaml: {
    label: 'lynter.yaml',
    hint: 'Edit here or use the form. Both stay in step.',
    syntax: 'YAML error',
    open: 'Open file',
    save: 'Save to file',
    download: 'Download',
    copy: 'Copy',
    copied: 'Copied',
    reset: 'Start over',
    fileSaved: (name: string, time: string) => `Saved to ${name} at ${time}`,
    fileHeld: (name: string) => `Not saved to ${name}: fix the error first`,
    fileFailed: (name: string) => `Could not write ${name}. Use Save to file again.`,
  },
};

export const examplesPage = {
  title: 'Examples',
  lede: 'Each card shows a note before and after, produced by the real rule engine with the config beside it. Open one in the playground to change the rules and watch the result move.',
  before: 'Before',
  after: 'After',
  config: 'lynter.yaml',
  open: 'Open in playground',
  path: 'Path',
};

export const importPage = {
  title: 'Import settings',
  lede: 'Convert an Obsidian Linter data.json into a lynter.yaml. Paste the file from .obsidian/plugins/obsidian-linter/ and copy the result to the root of your vault. The conversion runs in your browser, and nothing is uploaded.',
  inputLabel: 'data.json',
  outputLabel: 'lynter.yaml',
  placeholder: '{"ruleConfigs": {...}}',
  action: 'Convert',
  copy: 'Copy lynter.yaml',
  copied: 'Copied',
  error: 'That is not valid JSON. Copy the whole data.json file, including the outer braces.',
  note: 'Settings Lynter does not map yet land in a commented block at the end, so nothing is lost.',
  more: 'How the conversion works',
};

export const docsCopy = {
  navLabel: 'Docs',
  tocLabel: 'On this page',
  prev: 'Previous',
  next: 'Next',
  edit: 'Edit this page on GitHub',
};

export const notFound = {
  title: 'No page here',
  body: 'The link may be old. The playground and docs are a click away.',
  action: 'Go to the home page',
};

/** Pages rendered from content/*.md. */
export const mdPages = {
  gettingStarted: {slug: 'getting-started', title: 'Getting started', lede: 'Install the plugin from a clone, write a first scope, and add a pre-commit hook and a CI check.'},
  useCases: {slug: 'use-cases', title: 'Use cases', lede: 'Six places where one reviewed config keeps text consistent.'},
  llm: {slug: 'integrations-llm', title: 'LLM integrations', lede: 'Put what agents write into house style with a hook. No model call, no tokens.'},
  ci: {slug: 'integrations-ci', title: 'CI integrations', lede: 'Run check on every pull request and every commit, and read the result as JSON.'},
  samples: {slug: 'samples', title: 'Samples', lede: 'Four starter configs you can copy into a vault or repository.'},
  faq: {slug: 'faq', title: 'FAQ', lede: 'Short answers about scope, status, and what Lynter does not do.'},
  languages: {slug: 'languages', title: 'Languages', lede: 'The first release covers the languages below. Everything else is Coming soon.'},
  obsidian: {slug: 'obsidian', title: 'Obsidian', lede: 'Tasks, tags, frontmatter, file and folder names, and scopes.'},
  roadmap: {slug: 'roadmap', title: 'Roadmap', lede: 'What is being built now, what comes next, and what comes later.'},
};
