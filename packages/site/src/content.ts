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
    {href: '/switch/', label: 'Switch from Linter'},
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
      {title: 'CLI', body: 'check, fix, and explain from a terminal.', href: '/docs/#cli'},
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
  compare: {
    title: 'How it compares',
    columns: ['', 'Lynter', 'Obsidian Linter', 'markdownlint'],
    rows: [
      ['Config in a git-tracked file', 'Yes, lynter.yaml', 'No, plugin data.json', 'Yes'],
      ['Rules scoped by folder, tag, or property', 'Yes, with inheritance', 'No', 'No'],
      ['Frontmatter schemas per note type', 'Yes', 'No', 'No'],
      ['Same engine in Obsidian and CI', 'Yes', 'No', 'Separate plugin and CLI'],
      ['Code linting from the same file', 'In progress', 'No', 'No'],
    ],
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
  switch: {
    title: 'Already use Obsidian Linter?',
    body: 'Paste its data.json and get a lynter.yaml. Nothing leaves your browser.',
    action: 'Convert my settings',
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
  lede: 'Edit lynter.yaml, the note, or its path. The output updates as you type. Highlighted lines changed.',
  configLabel: 'lynter.yaml',
  noteLabel: 'Note',
  pathLabel: 'Path in vault',
  outputLabel: 'Result',
  violationsLabel: 'Rules that fired',
  noViolations: 'Nothing to change. The note already follows this config.',
  configError: 'lynter.yaml has an error, so nothing ran. Fix the line below and the result comes back.',
  chainLabel: 'Profiles applied',
  loading: 'Loading the engine.',
  reset: 'Reset example',
  exampleLabel: 'Example',
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

export const switchPage = {
  title: 'Switch from Obsidian Linter',
  lede: 'Lynter reads a lynter.yaml instead of plugin settings. Paste the data.json from .obsidian/plugins/ (the Obsidian Linter folder) and copy the result to the root of your vault. The conversion runs in your browser. The older names linter.yaml and lappe-linter.yaml are still read, with a notice.',
  inputLabel: 'data.json',
  outputLabel: 'lynter.yaml',
  placeholder: '{"ruleConfigs": {...}}',
  action: 'Convert',
  copy: 'Copy lynter.yaml',
  copied: 'Copied',
  error: 'That is not valid JSON. Copy the whole data.json file, including the outer braces.',
  note: 'Settings Lynter does not map yet land in a commented block at the end, so nothing is lost.',
};

export const docs = {
  title: 'Docs',
  installTitle: 'Install',
  installBody: 'Lynter is not in the Obsidian community directory yet. Build it from the repository: you need Git, Node.js 20 or later, and a Rust toolchain for the command line.',
  pluginTitle: 'Obsidian plugin',
  pluginBody: 'Copy main.js, manifest.json, and styles.css into .obsidian/plugins/lynter in your vault, then enable the plugin under Settings, Community plugins.',
  cliTitle: 'Command line',
  referenceTitle: 'lynter.yaml reference',
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
