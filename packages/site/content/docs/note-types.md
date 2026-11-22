---
title: Note types
lede: How note types describe the frontmatter a kind of note carries, which rules enforce them, and how keys are ranked, dated and written.
---

A note type is a schema for a kind of note. It lists the frontmatter keys the note should have, the order they appear in, the values some keys may take, and which keys hold the created and revised dates. Four rules read the schema of the note type a file resolves to. Everything else about frontmatter handling, such as key sorting, is shared with files that have no note type.

## Defining a note type

Note types live under `note-types`. Each entry accepts five keys, and the loader warns about any other key.

| Key | Meaning |
| --- | --- |
| `required` | Mapping of key to default value. A value is a scalar, a list of scalars, or null. |
| `key-order` | List of keys, in the order they should appear. |
| `values` | Mapping of key to a list of allowed scalar values. |
| `date-keys` | Mapping with optional `created` and `revised`, each the name of the frontmatter key that holds that date. |
| `match` | A match block, as for profiles. |

```yaml
note-types:
  task:
    required: {type: task, status: NEW, tags: []}
    key-order: [type, status]
    values: {status: [NEW, ACTIVE, DONE]}
    date-keys: {created: date-created, revised: date-revised}
    match: {path: ["tasks/**"]}
```

A file takes the first note type, in declaration order, whose `match` holds. The match tests are the same ones profiles use, described in [scopes and profiles](/docs/scopes-and-profiles/). A note type without a `match` never resolves to a file, because an empty match never holds. The resolved name is returned with every lint result as `note_type`, or `noteType` in the WebAssembly JSON.

A `base` note type governs the others. A scoped type may drop or reorder the keys that `base` requires, but listing a required key that `base` lacks is an error naming the key, such as `note-types.task.required.owner`. The check runs only when a `base` type with a `required` mapping exists.

Sources: `crates/lynter-core/src/config/validate.rs`, `crates/lynter-core/src/scope.rs`, `docs/lynter-yaml.md`

## Rules that use a schema

The schema reaches rules as an option. When a file resolves to a note type, the linter hands its schema to four rules. Each rule still has to be enabled in `defaults.rules` or a profile, like any rule.

| Rule | Behavior |
| --- | --- |
| `note-type-insert-keys` | Inserts each required key whose default is not null and that is absent, at its ranked position. It never overwrites a value. |
| `note-type-key-sort` | Reorders frontmatter keys by the ranking described below. |
| `note-type-date-keys` | Sets the created key when it is absent and refreshes the revised key when the run changed other content. |
| `note-type-validate` | Reports problems. It is report-only and never rewrites the note. |

For `tasks/a.md` containing only `title: Pay rent`, `note-type-insert-keys` with the `task` type above produces:

```text
---
type: task
status: NEW
title: Pay rent
tags:
---
```

A required key with a null default is never inserted. `note-type-validate` reports it instead when it is missing.

Sources: `crates/lynter-core/src/notetypes/yaml_rules.rs`, `crates/lynter-core/src/notetypes/validate.rs`, `crates/lynter-core/src/lint/mod.rs`

## Validation

`note-type-validate` checks the keys the schema manages: every key in `required`, every key in `values`, and the keys named in `date-keys`. It builds one message per problem, formatted `key: problem`.

| Problem | Message form |
| --- | --- |
| A managed key holds a nested mapping. | `key: nested maps are not allowed in managed keys` |
| A managed list holds a list or mapping item. | `key: list items must be flat scalars` |
| A value is outside the `values` list. | `key: value "x" is not one of [NEW, ACTIVE, DONE]` |
| A required key with a null default is missing. | `key: required key is missing and has no default` |
| The frontmatter is not a valid YAML mapping. | `frontmatter: invalid YAML or not a flat mapping` |

The rule reports through a no-op edit, so the note is never changed. The lint result carries one violation with `fixed: false` and the rule description as its message. The individual messages are available from the library function `collect_note_type_violations`.

Sources: `crates/lynter-core/src/notetypes/validate.rs`, `crates/lynter-core/src/lint/mod.rs`

## Key ranking

Key sorting uses one ranking for `yaml-key-sort`, which takes `priority-keys`, and for `note-type-key-sort`, which takes the schema's `key-order`. Keys sort into groups, and within a group by the rule below.

| Group | Keys | Order within the group |
| --- | --- | --- |
| 0 | Keys listed in the ordering list (`priority-keys` or `key-order`). | Position in that list. |
| 1 | `preset`, `domain`, `category`, `sub-category`, `types`, `date-created`, `date-revised`, `links`. | That fixed sequence. |
| 2 | Every other key. | Alphabetical, lower-cased, by UTF-16 code unit. |
| 3 | `aliases`, then `tags`. | Always after group 2. |
| 4 | `relo-file`, then `relo-folder`. | Always last. |

Each key travels with the lines that follow it, so multi-line lists and indented comments stay attached. Loose lines before the first key, such as comments, stay at the top.

`yaml-key-sort` can also insert keys. Its `defaults` option is a mapping of key to value, and any key missing from a note that has frontmatter is added at its ranked position.

Sources: `crates/lynter-core/src/notetypes/key_rank.rs`, `crates/lynter-core/src/notetypes/yaml_rules.rs`, `crates/lynter-core/src/config/defaults.rs`

## Dates

`date-keys` and the built-in `yaml-timestamp` rule share one engine. It needs a date, which is the `--today` value in the CLI, the `today` argument in the WebAssembly `lint` call, and the local date in the plugin. Without a date it changes nothing.

- The created key is added as `key: yyyy-MM-dd` when it is absent. An existing value is never changed.
- The revised key is set to the date when the note, with that key removed, differs from the original text. A run that changes nothing else leaves the revised date alone. An absent revised key is inserted at its ranked position.

`yaml-timestamp` defaults to `date-created` and `date-revised`. A note type's `date-keys` use whatever names you give them.

Sources: `crates/lynter-core/src/notetypes/yaml_rules.rs`, `crates/lynter-core/src/lint/mod.rs`

## Locale, values and formatting

`alphabetize-property-values` sorts the items of list properties, both block lists with one indent and unquoted flow lists such as `[b, a]`. It compares case-folded text with an approximation of root locale collation: punctuation first, then digits, then letters, with accented letters ordered after their base letters on ties. Keys and non-list values are untouched.

When a rule writes a value, plain strings stay plain unless a plain form would be misread, in which case the string is quoted. Empty and null values are written as `key:`. Non-empty lists are written as block sequences indented two spaces.

Filenames are checked by separate rules, `kebab-case-filename` and `h1-matches-stem`, not by the note type schema, which has no filename key. See [frontmatter rules](/docs/rules-frontmatter/) and [rules overview](/docs/rules-overview/).

Sources: `crates/lynter-core/src/notetypes/yaml_rules.rs`, `crates/lynter-core/src/notetypes/locale.rs`, `crates/lynter-core/src/notetypes/format.rs`, `crates/lynter-core/src/notetypes/frontmatter.rs`
