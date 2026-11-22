---
title: Frontmatter and note rules
lede: Every rule that reads or rewrites YAML frontmatter, note types, filenames, folders, tags and tasks, with options and before and after examples.
---

These rules work on the metadata of a note: the YAML block at the top, the keys a note type requires, the name and folder of the file, and the tags and task checkboxes in the body. Frontmatter rules edit only inside the frontmatter block, and a note without frontmatter is left alone unless the rule says it creates one. Turn a rule on with `enabled: true` under `defaults.rules` or a profile, as described in [Rules overview](/docs/rules-overview/). Rules that need a note type read the schema of the type the file matches, see [Note types](/docs/note-types/).

## Frontmatter keys and values

Ten rules cover key order, timestamps, array formatting, quoting, insertion and removal. Three of them (`yaml-key-sort`, `yaml-timestamp`, `alphabetize-property-values`) are on in the compiled defaults; the rest are off until enabled.

### `yaml-key-sort`

Sorts frontmatter keys and optionally inserts default keys when they are absent. Fixes. On by default.

Order: keys named in `priority-keys` first, in the listed order, then `preset`, `domain`, `category`, `sub-category`, `types`, `date-created`, `date-revised` and `links`, then every other key alphabetically (case insensitive), then `aliases`, `tags`, `relo-file` and `relo-folder` last. Frontmatter that is not a flat YAML mapping is left alone.

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `priority-keys` | list of strings | `[]` (the compiled defaults set the eight built-in keys) | Keys that sort first, in this order. |
| `defaults` | mapping | `{}` | Key and value pairs inserted, in ranked position, when the key is missing. |

```markdown
---
status: DRAFT
domain: development
category: notes
tags:
  - x
---
Body.
```

```markdown
---
domain: development
category: notes
status: DRAFT
tags:
  - x
---
Body.
```

### `yaml-timestamp`

Keeps `date-created` and `date-revised` current in `yyyy-MM-dd` form. Fixes. On by default.

`date-created` is added when absent. `date-revised` is set to today only when the run changed something else in the note, so an already clean note is not touched. The rule needs today's date and the text as it was before the run; hosts supply both (the CLI takes `--today`). Without a date it does nothing. If the note has no frontmatter, the rule creates one.

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `schema` | mapping | `{date-keys: {created: date-created, revised: date-revised}}` | Names the frontmatter keys that hold the two dates. |

```markdown
---
---
```

```markdown
---
date-created: 2026-07-10
date-revised: 2026-07-10
---
```

### `alphabetize-property-values`

Sorts the items of list properties, leaving keys and scalar values alone. Fixes. On by default.

It sorts block lists and single-line `[a, b]` lists of plain values, ignoring case. A flow list containing quotes or brackets, and any list with one item, is skipped. No options.

```markdown
---
tags:
  - zeta
  - alpha
aliases: [second, first]
---
```

```markdown
---
tags:
  - alpha
  - zeta
aliases: [first, second]
---
```

### `dedupe-yaml-array-values`

Removes repeated values from YAML arrays, comparing case sensitively. Fixes. Off by default.

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `dedupe-aliases` | boolean | `true` | Process the `aliases` key. |
| `dedupe-tags` | boolean | `true` | Process the `tags` key. |
| `dedupe-arrays` | boolean | `true` | Process every other array key. |
| `keys-to-ignore` | list of strings | `[]` | Keys never processed. |

```markdown
---
tags: [computer, research, computer, Computer]
---
```

```markdown
---
tags: [computer, research, Computer]
---
```

### `escape-yaml-special-characters`

Quotes frontmatter values that contain a colon followed by a space, a single quote or a double quote. Fixes. Off by default.

A value with a colon and a space is wrapped in the default quote character, a value with a single quote is wrapped in double quotes, and a value with a double quote is wrapped in single quotes. A value that is already quoted, or holds both kinds of quote, is left as written.

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `default-escape-character` | string | `"` | The quote to use, `"` or `'`. |
| `escape-single-line-arrays` | boolean | `false` | Also quote the items of single-line `[a, b]` arrays. |

```markdown
---
key: value: with colon in the middle
---
```

```markdown
---
key: "value: with colon in the middle"
---
```

### `format-yaml-arrays`

Rewrites array properties, `tags` and `aliases` in the chosen layout. Fixes. Off by default.

Styles are `single-line`, `multi-line`, `single string comma delimited`, `single string space delimited`, `single string to single-line` and `single string to multi-line`. The two `to` styles turn arrays with more than one item into the named layout and keep a single item as a plain string. Any other style value acts as `single-line`. A scalar `tags` value is split on commas and spaces and a scalar `aliases` value on commas; a scalar under any other key is only handled, as a single item, when the key is listed in a force option.

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `format-aliases` | boolean | `true` | Format the `aliases` key. |
| `aliases-style` | string | `single-line` | Style for `aliases`. |
| `format-tags` | boolean | `true` | Format the `tags` key. |
| `tags-style` | string | `single-line` | Style for `tags`. |
| `format-arrays` | boolean | `true` | Format every other array key. |
| `default-array-style` | string | `single-line` | Style for other arrays. |
| `force-single-line-keys` | list of strings | `[]` | Keys always written single-line, even a plain string. |
| `force-multi-line-keys` | list of strings | `[]` | Keys always written as a multi-line list. |

```markdown
---
tags:
  - computer
  - research
aliases: Title 1, Title2
---
```

With `aliases-style: multi-line`:

```markdown
---
tags: [computer, research]
aliases:
  - Title 1
  - Title2
---
```

### `format-tags-in-yaml`

Removes the leading `#` from tags in the `tags` or `tag` frontmatter key. Fixes. Off by default. No options.

```markdown
---
tags: [#one #two #three]
---
```

```markdown
---
tags: [one two three]
---
```

### `insert-yaml-attributes`

Adds missing frontmatter keys from a template, creating the block when the note has none. Fixes. Off by default.

Keys already present are never overwritten or duplicated. New lines are inserted at the top of the block.

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `text-to-insert` | string | `"aliases: \ntags: "` | Template lines; the text before the first colon is the key. |

```markdown
Body
```

```markdown
---
aliases:
tags:
---
Body
```

### `remove-yaml-keys`

Deletes the listed top-level keys, with their values, from the frontmatter. Fixes. Off by default.

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `keys` | list of strings | `[]` | Keys to remove. A trailing colon is accepted, so `status:` and `status` are the same. |

```markdown
---
a: 1
b: 2
---
```

With `keys: ["a:"]`:

```markdown
---
b: 2
---
```

### `move-tags-to-yaml`

Collects `#tags` from the body into the frontmatter `tags` list. Fixes. Off by default.

Tags inside code, inline code and other protected regions are ignored. It creates the frontmatter or the `tags` key when needed and appends only tags not already listed.

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `body-tag-operation` | string | `nothing` | What happens to the body tags: `nothing`, `remove-hashtag` (keep the word, drop the `#`) or `remove-whole-tag`. |
| `tags-to-ignore` | list of strings | `[]` | Tags that stay in the body and are not moved. |

```markdown
---
tags: [test, tag2]
---
Text has to do with #test and #markdown
```

```markdown
---
tags: [test, tag2, markdown]
---
Text has to do with #test and #markdown
```

Sources: `crates/lynter-core/src/notetypes/yaml_rules.rs`, `crates/lynter-core/src/notetypes/key_rank.rs`, `crates/lynter-core/src/notetypes/locale.rs`, `crates/lynter-core/src/rules/compat/yaml_rules.rs`, `crates/lynter-core/src/config/defaults.rs`, `crates/lynter-core/tests/fixtures/compat/dedupe-yaml-array-values.json`, `crates/lynter-core/tests/fixtures/compat/move-tags-to-yaml.json`

## Note type rules

These four rules apply the schema of the note type a file matches (`required`, `key-order`, `values`, `date-keys`). When no note type matches, or the rule is not enabled, nothing happens. All are off by default and take no options of their own; the schema is passed in by the linter. See [Note types](/docs/note-types/) for the schema keys.

### `note-type-insert-keys`

Inserts the type's required keys that are missing, using the schema default as the value. Fixes. Off by default.

Existing values are never overwritten. A required key whose default is `null` is not inserted; `note-type-validate` reports it instead.

```markdown
---
domain: x
---
```

With `required: {status: NEW}`:

```markdown
---
domain: x
status: NEW
---
```

### `note-type-key-sort`

Orders frontmatter keys by the type's `key-order`, then by the built-in order. Fixes. Off by default.

The built-in order is `preset`, `domain`, `category`, `sub-category`, `types`, `date-created`, `date-revised`, `links`, then the remaining keys alphabetically, then `aliases`, `tags`, `relo-file` and `relo-folder`.

```markdown
---
tags: [a]
domain: work
status: NEW
---
```

With `key-order: [status]`:

```markdown
---
status: NEW
domain: work
tags: [a]
---
```

### `note-type-date-keys`

Sets the type's created date key when it is absent and bumps the revised key only when the run changed other content. Fixes. Off by default.

It works like `yaml-timestamp`, but the key names come from the type's `date-keys` (`created` and `revised`). It needs today's date from the host and does nothing without it.

```markdown
---
---
```

With `date-keys: {created: created}` and today as 2026-07-10:

```markdown
---
created: 2026-07-10
---
```

### `note-type-validate`

Reports frontmatter that breaks the note type. Reports only, it never changes the file. Off by default.

Findings, each as `key: problem`: invalid YAML or a frontmatter that is not a flat mapping; nested maps in a managed key; list items that are not flat scalars; a value outside the type's `values` list; a required key that is missing and has no default. Managed keys are those named in `required`, `values` and `date-keys`.

Sources: `crates/lynter-core/src/notetypes/yaml_rules.rs`, `crates/lynter-core/src/notetypes/validate.rs`, `crates/lynter-core/src/notetypes/key_rank.rs`, `crates/lynter-core/src/notetypes/schema.rs`, `crates/lynter-core/src/lint/mod.rs`, `crates/lynter-core/tests/fixtures/rules-note-type-insert-keys.json`, `crates/lynter-core/tests/fixtures/rules-note-type-date-keys.json`

## Filename and folder rules

These rules read the file path the host supplies. `kebab-case-filename` and `h1-matches-stem` come from the note type family; `folder-naming` and `filename-pattern` are report-only checks. Without a path they do nothing.

### `kebab-case-filename`

Flags a filename stem that is not kebab-case and proposes the compliant name. Reports only. Off by default.

The slug lowercases, splits camel case, strips accents and punctuation, and joins words with hyphens. Reserved stems such as `README`, `CLAUDE`, `SKILL`, `MEMORY`, `AGENTS` and `CODEX` are kept as they are. The rule never renames a file. The top-level `rename.strategy` key (`replace`, `prepend` or `append`) decides how a leading or trailing number or date is handled in the proposal: with `prepend` or `append` it is kept and moved to the front or the end. No options of its own.

```text
notes/My Note.md  ->  my-note
12 notes.md       ->  12-notes
```

### `h1-matches-stem`

Keeps the first H1 equal to the filename stem. Fixes. Off by default.

The first H1 is rewritten in place. When there is none, `# stem` is inserted after the frontmatter. No options.

```markdown
# Title

body
```

For a file named `c.md`:

```markdown
# c

body
```

### `folder-naming`

Reports folders in the file path that break a case style, a maximum depth or an allowed list. Reports only. Off by default.

Only folder names are checked, not the filename. When `allowed-names` is set, folders are checked against that list and `case` is not applied.

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `case` | string | `none` | `kebab`, `snake`, `camel` or `lower` (the long forms `kebab-case`, `snake_case`, `camelCase` and `lowercase` also work). |
| `max-depth` | integer | `0` | Largest number of folders; `0` means no limit. |
| `allowed-names` | list of strings | `[]` | Folder names that are permitted. |
| `ignore-hidden` | boolean | `true` | Stop checking at the first folder that starts with a dot. |

```text
My Notes/sub/x.md with case: kebab
folder "My Notes" is not kebab case, expected "my-notes"
```

### `filename-pattern`

Reports file names, without the extension, that do not match a preset or a regular expression. Reports only. Off by default.

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `preset` | string | `none` | `kebab` requires lowercase words joined by hyphens; `dated` requires a valid `yyyy-mm-dd-` prefix. |
| `pattern` | string | empty | A regular expression the stem must match. An invalid expression is itself reported. |

```text
My Note.md with preset: kebab
file name "My Note" is not kebab-case
```

Sources: `crates/lynter-core/src/notetypes/filename.rs`, `crates/lynter-core/src/rules/notes/folder_naming.rs`, `crates/lynter-core/src/rules/notes/filename_pattern.rs`, `crates/lynter-core/src/rules/notes/casing.rs`, `crates/lynter-core/tests/notes.rs`, `crates/lynter-core/tests/fixtures/rules-h1-matches-stem.json`, `crates/lynter-core/tests/fixtures/rules-kebab-case-filename.json`

## Tag, task and schema rules

Three rules cover inline tags, task checkboxes and a lightweight frontmatter schema. They are off by default. Tag and task rules rewrite what is safe and report the rest; the schema rule only reports.

### `tag-format`

Applies a case style to inline `#tags` and reports tags that are too deep, not allowed, duplicated or in the wrong place. Fixes and reports. Off by default.

Case conversion is the only rewrite. Everything else is a finding. Purely numeric tags such as `#123`, and tags inside code, wikilinks or other protected regions, are skipped. Frontmatter tags are checked and reported, never rewritten.

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `case` | string | `none` | `kebab`, `snake`, `camel` or `lower`, applied to each segment of a nested tag. |
| `max-depth` | integer | `0` | Largest number of `/` levels; `0` means no limit. |
| `allowed` | list of strings | `[]` | Permitted tags; an entry ending in `/*` permits a whole branch. Empty permits all. |
| `dedupe` | boolean | `true` | Report repeated tags in frontmatter. |
| `placement` | string | `any` | `frontmatter` reports inline tags, `inline` reports frontmatter tags, `any` reports neither. |

```markdown
a #MyTag b
```

With `case: kebab`:

```markdown
a #my-tag b
```

### `task-format`

Normalizes task checkboxes and the dates written after the task. Fixes and reports. Off by default.

It sets one space around the checkbox, lowercases `[X]`, writes dates as `yyyy-mm-dd` after the start, scheduled, due and done emoji, and reorders those tokens. A status character outside `allowed-statuses` is reported, not changed. Impossible dates and tasks in code blocks are left alone.

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `normalize-spacing` | boolean | `true` | Use one space between the list marker and the checkbox, and between the checkbox and the text. |
| `lowercase-marker` | boolean | `true` | Rewrite `[X]` as `[x]`. |
| `allowed-statuses` | string | empty | Permitted status characters; empty permits all. |
| `date-format` | boolean | `true` | Rewrite dates as `yyyy-mm-dd` with one space after the emoji. |
| `date-order` | list of strings | `[start, scheduled, due, done]` | Order of the date tokens; an empty list keeps the written order. |

```markdown
-  [X]   buy milk 📅 2026/12/1
```

```markdown
- [x] buy milk 📅 2026-12-01
```

### `frontmatter-schema`

Reports frontmatter that lacks required keys, has values of the wrong type, or uses values outside an enum. Reports only. Off by default.

It does nothing until at least one of its options is set. A required key counts as missing when it is absent or `null`. Type names are `string`, `number`, `bool`, `list` and `date` (a calendar date, optionally with a time).

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `required` | list of strings | `[]` | Keys that must be present. |
| `types` | mapping | `{}` | Key to type name. |
| `enums` | mapping | `{}` | Key to the list of allowed values. Lists are checked item by item. |

```text
status: wip  with enums: {status: [draft, done]}
key "status" expects one of draft, done, found "wip"
```

Sources: `crates/lynter-core/src/rules/notes/tag_format.rs`, `crates/lynter-core/src/rules/notes/task_format.rs`, `crates/lynter-core/src/rules/notes/frontmatter_schema.rs`, `crates/lynter-core/src/rules/notes/casing.rs`, `crates/lynter-core/src/rules/notes/mod.rs`, `crates/lynter-core/tests/notes.rs`
