---
title: Scopes and profiles
lede: How profiles match files, in what order their rule settings are layered, how rule-order works, and how explain shows the result.
---

A profile is a named set of rule overrides plus a `match` block that says which files it governs. For each file, the scope engine decides which profiles match, sorts them, and merges their settings over `defaults.rules`. The result is the chain of profiles and the rule settings that the rule runner uses.

## Matchers

A `match` block can hold any of nine kinds. Every kind that is present must hold (AND). Within one list, any entry is enough (OR). An empty list counts as absent. A match with no kinds never matches, so such a profile applies only through the `linter-profile` override described below.

| Key | Type | A file matches when |
| --- | --- | --- |
| `path` | list of globs | Its config-relative path matches one glob. `**` matches zero or more whole segments, so `a/**` also matches `a`. `*` and `?` match within one segment. Brackets and braces are literal. |
| `extension` | list of strings | Its extension, lower-cased and without the dot, equals one entry. A dotfile such as `.hidden` has no extension. |
| `frontmatter` | mapping | Every listed key matches its value (see below). |
| `tag` | list of strings | It has at least one of the tags, from frontmatter `tags` (or `tag`) or from inline `#tag` text. A leading `#` is ignored. |
| `age` | list of strings | Its age bucket is listed. The bucket comes from `date-created` and the run date, in 5-day steps such as `1-5` and `6-10`. |
| `date-created` | mapping | Its `date-created` falls within `after` and `before`, both inclusive. |
| `date-revised` | mapping | The same test against `date-revised`. |
| `backlink` | list of strings | A host-supplied list of linking files contains one entry, compared case-insensitively. |
| `alias` | list of strings | A host-supplied list of aliases contains one entry, compared case-insensitively. |

Frontmatter values compare as follows. A scalar expected value matches an equal scalar, or a list that contains it. A list expected value matches a list that contains every entry. Strings and booleans compare exactly, and numbers compare numerically, so `n: 3` matches `3.0` but the string `"false"` does not match `false`. A date range test fails when the file has no valid date. A date with a time suffix, such as `2024-01-01T10:00:00`, uses its date part.

The CLI and the WebAssembly build pass no backlink or alias facts, so `backlink` and `alias` matchers do not match there. A host that calls the core library can supply both through `ScopeContext`.

Sources: `crates/lynter-core/src/scope.rs`, `crates/lynter-core/src/config/validate.rs`, `crates/lynter-cli/src/lint.rs`, `crates/lynter-wasm/src/lib.rs`

## Order of application

The engine builds the effective rules in layers, and later layers win key by key.

1. `defaults.rules`.
2. Profiles whose match uses only `extension`.
3. Profiles whose match uses `path`, with or without `extension`. The deeper glob wins over a shallower one, where depth is the number of `/` separated segments in the glob.
4. Profiles whose match uses any of `frontmatter`, `tag`, `age`, `date-created`, `date-revised`, `backlink` or `alias`.
5. The profile named by the file's own `linter-profile` frontmatter key, applied last.

Internally each matching profile gets a rank (1, 2 or 3 for the tiers above), and the list is sorted by rank, then glob depth, then declaration order. Profiles in the same tier and depth apply in the order they appear in the file, so the later one wins.

The `linter-profile` value must be a string that names an existing profile. It moves that profile to the end of the chain even when its own match did not hold, which is how a match-less profile gets used. Other values, and names that do not exist, are ignored.

Merging is shallow per rule. A profile's `paragraph-spacing: {blank-lines: 0}` replaces only `blank-lines` and keeps the `enabled` value from `defaults`. A list value replaces the list from the earlier layer. It does not append.

```yaml
version: 1
defaults:
  rules:
    paragraph-spacing: {enabled: true, blank-lines: 1}
    replace-em-dash: {enabled: true}
profiles:
  notes:
    match: {path: ["notes/**"]}
    rules:
      paragraph-spacing: {blank-lines: 2}
  deep:
    match: {path: ["notes/projects/**"]}
    rules:
      paragraph-spacing: {blank-lines: 0}
  journal:
    match: {frontmatter: {category: journal}}
    rules:
      replace-em-dash: {enabled: false}
```

For `notes/projects/a.md` with `category: journal`, the chain is `defaults`, `notes`, `deep`, `journal`, and `blank-lines` resolves to 0.

Sources: `crates/lynter-core/src/scope.rs`, `crates/lynter-core/tests/fixtures/scope.json`, `docs/lynter-yaml.md`

## The profile chain

The resolved chain always starts with `defaults`, followed by the matched profiles in the order they were applied. The last entry is the winning profile. The result of a lint carries the chain as `profile_chain` in Rust and `profileChain` in the WebAssembly JSON. The CLI `--json` output reports the last entry as `profile`. The `profile_chain(config_yaml, path)` WebAssembly export returns the chain for a path as a JSON array.

The note type is resolved from the same facts. A file takes the first note type, in declaration order, whose own `match` holds. See [note types](/docs/note-types/).

Sources: `crates/lynter-core/src/scope.rs`, `crates/lynter-core/src/lint/mod.rs`, `crates/lynter-cli/src/main.rs`, `crates/lynter-wasm/src/lib.rs`

## rule-order

`rule-order` is a list of rule ids. The top-level list applies to every file. A profile may set its own, and a non-empty profile list replaces the top-level one. When several profiles in a chain set one, the last non-empty list wins.

Rules named in the list run first, in the listed order. Every other enabled rule keeps its built-in order after them. Ids that name disabled or unknown rules are ignored. Code and data rules honor the same list.

```yaml
rule-order: [replace-em-dash, strip-strong]
profiles:
  prose:
    match: {extension: [md]}
    rule-order: [header-case]
```

Sources: `crates/lynter-core/src/lint/mod.rs`, `crates/lynter-core/src/scope.rs`, `crates/lynter-code/src/runner.rs`

## Seeing the result with explain

`lynter explain PATH` loads the governing config for one file, resolves it, and prints the outcome without changing anything. It exits with status 2 when the file does not exist or the config is invalid.

```text
path: notes/projects/a.md
config: /vault/lynter.yaml
ignored: false
profile chain: defaults -> notes -> deep -> journal
note type: (none)
rules:
  paragraph-spacing: enabled  Normalize blank lines between blocks to a fixed count (0, 1, or 2); trims leading and trailing blank lines and preserves blank lines inside code, math, tables, and frontmatter.
  replace-em-dash: disabled  Replace em dashes and connector en dashes with a configured separator (default comma plus space), preserving numeric en dash ranges.
```

Each rule line shows its resolved state and the rule description. The output does not list rule options or the effective rule order. Run `explain` with `--today yyyy-MM-dd` to test age matchers on a fixed date.

Sources: `crates/lynter-cli/src/main.rs`, `crates/lynter-cli/tests/cli.rs`
