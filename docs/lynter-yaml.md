# lynter.yaml

`lynter.yaml` is the single configuration file for Lynter. It lives in the root of a vault or project, is tracked in git, and controls which rules run, which files each rule applies to, and what shape each note type must have. The same file is read by the CLI, the Obsidian plugin, and the WASM build used on lynter.dev.

Create a commented starter with `lynter init`. It refuses to overwrite an existing file.

## File names and discovery

| Name | Status |
| --- | --- |
| `lynter.yaml` | Canonical. |
| `linter.yaml` | Deprecated alias. |
| `lappe-linter.yaml` | Deprecated alias. |

For each target file the CLI looks for a config in the file's directory, then walks up through parent directories until it finds one. In a single directory the names are tried in the order above and the first one found wins. A deprecated name prints one notice asking you to rename it to `lynter.yaml`. `--config PATH` skips discovery and uses that file for every target. If no config is found, the command fails with an error that lists the three names.

## Loading is fail closed

A structural error anywhere in the file rejects the whole config with every error listed, each with its key path. Lynter never runs with a partial config. An unknown key is not an error: it is ignored with a warning naming the key. Errors and warnings come from validating the parsed YAML, so a YAML syntax error is reported as `yaml parse error`. An empty file is not a mapping and is rejected.

## Top-level keys

| Key | Purpose |
| --- | --- |
| `version` | Schema version. Required, and must be `1`. |
| `defaults` | Settings applied to every file. Holds `rules` and `adapters`. |
| `rule-order` | List of rule ids giving the order rules run in. |
| `profiles` | Named per-scope overrides. |
| `note-types` | Schemas for kinds of notes. |
| `templates` | Frontmatter and body templates, global and per scope. |
| `automations` | Named triggers that run a check, fix, or template. |
| `rename` | Filename behavior. |
| `ignore` | Folders and files Lynter never touches. |
| `providers` | Rule settings grouped by provider namespace. |
| `code-checks` | Declarative regex checks over fenced code blocks. |

## Version

```yaml
version: 1
```

Any other value, or a missing value, is an error at `version`.

## Defaults

`defaults.rules` is a mapping of rule id to that rule's settings. A rule's settings are a mapping. `enabled` must be a boolean when present, and every other key is an option passed to the rule. A rule that is not listed, or is listed without `enabled: true`, does not run.

```yaml
defaults:
  rules:
    yaml-key-sort:
      enabled: true
      priority-keys: [domain, category, types, date-created]
    paragraph-spacing:
      enabled: true
      blank-lines: 1
    replace-em-dash:
      enabled: false
```

With no `lynter.yaml` at all, the compiled defaults apply. These turn on `yaml-key-sort`, `yaml-timestamp`, `alphabetize-property-values`, `paragraph-spacing`, `list-style`, `replace-em-dash`, `code-checks`, `code-checks-fix`, `join-paragraph-lines`, and `prose-list-to-sentences`. Five rules are present but off until you opt in, because they rewrite wording or names: `strip-strong`, `header-case`, `contractions`, `h1-matches-stem`, and `kebab-case-filename`. The default `yaml-key-sort` priority keys are `preset`, `domain`, `category`, `sub-category`, `types`, `date-created`, `date-revised`, and `links`, with `aliases` and `tags` always last. The defaults also set `rename.mode` to `flag` and a `base` note type.

`join-paragraph-lines` and `prose-list-to-sentences` only report by default. Their `-fix` variants rewrite text when enabled.

## Rule order

`rule-order` is a list of rule ids. A profile may set its own `rule-order`, and a non-empty one replaces the top-level list for files that profile governs.

## Profiles and scopes

A profile is a named set of rule overrides plus a `match` block that says which files it governs. Each profile accepts `match`, `rules`, and `rule-order`. `rules` has the same shape as `defaults.rules`, and a profile may also carry `adapters`.

```yaml
profiles:
  tasks:
    match:
      frontmatter: {category: task}
    rules:
      paragraph-spacing: {enabled: true, blank-lines: 0}
  journal:
    match:
      path: ["journal/**"]
      extension: [md]
    rules:
      replace-em-dash: {enabled: false}
```

### Match keys

| Key | Type | Meaning |
| --- | --- | --- |
| `path` | list of globs | Vault-relative globs. `**` matches zero or more whole segments, `*` and `?` match within one segment. Brackets and braces are literal. |
| `extension` | list of strings | File extensions, compared in lower case. |
| `frontmatter` | mapping | Each key must match its value. A value is a scalar or a list of scalars. |
| `tag` | list of strings | The file has at least one of these tags, in frontmatter or inline. |
| `age` | list of strings | Age bucket of the file, computed from `date-created` in 5-day buckets such as `1-5`. |
| `date-created` | mapping | `after` and/or `before`, ISO `yyyy-MM-dd`, inclusive. |
| `date-revised` | mapping | Same, against `date-revised`. |
| `backlink` | list of strings | At least one of these files links to the file. |
| `alias` | list of strings | The file has at least one of these aliases. |

All kinds present in a match must hold (AND). Within one list, any entry is enough (OR). Empty lists count as absent. A match with no kinds never matches, so such a profile applies only through the override key below. Unknown keys inside a match are warnings. Date bounds that are not real calendar dates are errors.

### Resolution

For one file the effective rules are built in layers, later layers winning key by key:

1. `defaults.rules`.
2. Profiles matched by extension.
3. Profiles matched by path, with the deeper glob winning over a shallower one.
4. Profiles matched by frontmatter, tag, age, date, backlink, or alias.
5. The profile named in the file's own `linter-profile` frontmatter key, applied last.

Profiles in the same tier apply in declaration order. `lynter explain PATH` prints the governing config, the profile chain, and the resulting rules for a file.

## Note types

`note-types` maps a type name to a schema that describes what frontmatter notes of that type should carry.

| Key | Meaning |
| --- | --- |
| `required` | Mapping of key to default value. Values are scalars, lists of scalars, or null. |
| `key-order` | List of keys giving their order in frontmatter. |
| `values` | Mapping of key to a list of allowed scalar values. |
| `date-keys` | Mapping with optional `created` and `revised`, each naming the frontmatter key that holds that date. |
| `match` | A match block, as for profiles. A file takes the first note type whose match holds. |

```yaml
note-types:
  base:
    required: {domain: null, category: null, type: base, status: NEW, aliases: [], tags: []}
    match: {frontmatter: {type: base}}
  task:
    required: {type: task, status: NEW, tags: []}
    values: {status: [NEW, ACTIVE, DONE]}
    match: {frontmatter: {type: task}}
```

The `base` type governs the others. A scoped type may drop or reorder the keys that `base` requires but may not introduce a required key that `base` lacks. Doing so is an error naming the key.

## Templates

`templates.global` and `templates.by-scope` seed new notes. A global template accepts `pinned-keys`, `key-order`, `frontmatter`, `body`, and `age-line`. Each entry of `by-scope` is a list item that also needs a unique non-empty `name` and may add `extends` (only `global` is supported), `match`, and `toggles`, a mapping of key to a boolean or `on` or `off`. `frontmatter` values are scalars, lists of scalars, or null, `body` is a string, and `age-line` is a boolean.

## Automations

`automations` is a list. Each entry needs a unique non-empty `name` and a `trigger` of `on-write`, `on-create`, `on-rename`, `pre-commit`, `ci`, `schedule`, or `manual`. A `schedule` trigger also needs a cron string in `schedule`. Optional keys are `action` (`check`, `fix`, or `apply-template`), `failure` (`open` or `closed`), `log` (`spool` or `none`), `report` (`md`, `json`, or `none`), and `scope`, a match block.

## Rename and ignore

```yaml
rename:
  mode: flag        # off, flag, or rename
  strategy: replace # replace, prepend, or append
ignore:
  folders: [templates, .obsidian]
  files: [inbox/scratch.md]
```

`rename.mode` is required when `rename` is present: `off` disables filename rules, `flag` reports only, and `rename` fixes names and updates links. `strategy` is optional. `ignore.folders` and `ignore.files` are lists of vault-relative paths Lynter never touches.

## Providers

`providers` groups rule settings by namespace. Each namespace is a mapping with a `rules` key shaped like `defaults.rules`.

## Code checks

`code-checks` defines checks over fenced code blocks. Each entry is keyed by an id and may set `enabled`, `description`, `paths`, `languages`, `pattern`, `flags`, `message`, and `fix`. A malformed check is skipped with a warning rather than failing the config. The `code-checks` and `code-checks-fix` rules run them; both do nothing until a check is enabled.

```yaml
code-checks:
  no-token-shaped-strings: {enabled: true}
```

## Adapters

Adapters run native tools for source files next to the markdown rules. They are off by default, and Lynter skips a file with a one-line notice if the tool is not installed. Two adapters exist: `ruff` for Python and `biome` for TypeScript and JavaScript.

| Key | Applies to | Meaning |
| --- | --- | --- |
| `enabled` | both | Boolean. Default false. |
| `path` | both | Path to the binary. Relative paths resolve against the config's directory. Without it, the tool is found on `PATH`. |
| `args` | both | Extra arguments passed before the files. A list of strings. |
| `select` | `ruff` only | Rule codes passed to `--select`. A list of strings. |

```yaml
defaults:
  adapters:
    ruff: {enabled: true, select: [E, F]}
profiles:
  scripts:
    match: {path: ["scripts/**"]}
    adapters:
      ruff: {select: [F]}
```

Adapters are set in `defaults.adapters` and then overridden by `profiles.<name>.adapters` for each profile in a file's chain, key by key. An unknown adapter id or an unknown key is an error. Pass `--no-adapters` to skip them for a run.

## Command line

| Command | Purpose |
| --- | --- |
| `lynter check [PATHS]` | Report violations, exit 1 if any. |
| `lynter fix [PATHS]` | Apply fixes and print changed files. |
| `lynter fix --stdin [--stdin-path PATH]` | Filter mode: read stdin, write fixed text to stdout. |
| `lynter explain PATH` | Show the governing config, profile chain, and rules for a file. |
| `lynter init` | Write a commented starter `lynter.yaml` to the current directory. |

Global flags: `--config PATH`, `--json` (one JSON line per file, output version 1), `--changed` (also target git-changed `.md` and `.txt` files), `--today yyyy-MM-dd` (the date used for age matching), and `--no-adapters`.

## Importing an Obsidian Linter data.json

The loader can convert a plugin `data.json` into `lynter.yaml` text. `ruleConfigs` become `defaults.rules`, `foldersToIgnore` becomes `ignore.folders`, interface-only settings stay behind, and everything else is carried over as comments so nothing is dropped silently. Input that is already a valid `lynter.yaml` is returned unchanged.
