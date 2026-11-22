---
title: Rules overview
lede: What a rule is, how rules are enabled, ordered and applied, what they never touch, and an index of every rule id with its default state.
---

Lynter registers 69 rule ids: 56 note rules in `lynter-core` and 13 code and data rules in `lynter-code`. Each rule checks one thing and either rewrites it or reports it. Rules are switched on and tuned in `lynter.yaml`; this page explains the model, and the pages linked from the index at the bottom document each rule with its options and an example.

## What a rule is

A rule is a pure function from a parsed document and an options map to a list of edits. It does no I/O and never writes a file; the runner applies the edits. The contract is the `Rule` trait in `crates/lynter-core/src/rule.rs`.

```text
id() -> stable kebab-case identifier
description() -> one-line description
default_options() -> options used when config does not override them
check(document, options) -> edits for one round
edit_scope() -> Body (default) or Frontmatter
```

An edit replaces a byte range with new text and carries a message. An edit whose replacement equals the text it covers changes nothing and is a report-only finding. That is how a rule reports without fixing.

Code and data rules have the same shape in `crates/lynter-code/src/rule.rs`. They receive a `SourceFile` (text, path, language and parse tree) instead of a document, and add `applies_to`, which says whether the rule wants a file by extension or language. They return the same edits, and the `report` helper builds the no-op edit.

| Part | Note rule (`lynter-core`) | Code rule (`lynter-code`) |
| --- | --- | --- |
| Input | `Document`: text, lines, protected regions, list items | `SourceFile`: text, path, language, tree-sitter tree |
| Selects files | Markdown and text files | `applies_to`, by extension or language |
| Output | Edits | Edits, with `report` for findings |
| Built-in list | `builtin_rules()` in `rules/mod.rs` | `registry()` in `runner.rs` |

Sources: `crates/lynter-core/src/rule.rs`, `crates/lynter-core/src/edit.rs`, `crates/lynter-core/src/rules/mod.rs`, `crates/lynter-code/src/rule.rs`, `crates/lynter-code/src/runner.rs`

## Enabling rules and options

A rule runs only when its stanza says `enabled: true`. Every other key in the stanza is an option handed to the rule, replacing the rule's own default for that key. A rule that is not listed, or is listed without `enabled: true`, does not run. Rules set under `defaults.rules` apply everywhere, and profiles override them key by key for the files they match; see [Scopes and profiles](/docs/scopes-and-profiles/) and [Configuration](/docs/configuration/).

```yaml
version: 1
defaults:
  rules:
    paragraph-spacing: {enabled: true, blank-lines: 1}
    contractions:
      enabled: true
      "it is": false
profiles:
  journal:
    match: {path: ["journal/**"]}
    rules:
      replace-em-dash: {enabled: false}
```

The linter also fills in a few options itself, so that rules stay pure. These are `file-path` (for the filename and folder rules), `today` and `originalText` (for the timestamp rules), `schema` (the matched note type, for the note type rules), `checks` (the top-level `code-checks` mapping) and `strategy` (from `rename.strategy`). They come from the host and are not meant to be set by hand. The full set of keys is in the [configuration reference](/docs/reference/).

Code and data rules take their options the same way, plus one extra key, `languages`, described in [Code and data linting](/docs/code-linting/).

Sources: `docs/lynter-yaml.md`, `crates/lynter-core/src/lint/mod.rs`, `crates/lynter-core/src/runner.rs`, `crates/lynter-code/src/runner.rs`

## Run order

For a note, the linter first picks the enabled rules and sorts them by a built-in order: 21 rules have a fixed place, and any other rule runs after them in registry order. A non-empty `rule-order` then moves the ids it names to the front, in the order listed; unlisted rules keep their relative order behind them. A profile's non-empty `rule-order` replaces the top-level list for the files it governs.

```flow
Resolve profiles -> Pick enabled rules -> Sort by built-in order
Apply rule-order -> Run each rule once -> Collect violations
```

The built-in order of the fixed 21 is:

```text
join-paragraph-lines, join-paragraph-lines-fix, strip-strong, replace-em-dash, prose-list-to-sentences, prose-list-to-sentences-fix, header-case, paragraph-spacing, list-style, contractions, yaml-key-sort, alphabetize-property-values, yaml-timestamp, note-type-insert-keys, note-type-key-sort, note-type-date-keys, note-type-validate, h1-matches-stem, kebab-case-filename, code-checks, code-checks-fix
```

Each enabled rule is applied once to the text left by the previous rule. Two rules repeat internally, up to 64 rounds, because one pass can expose another match: `strip-strong` and `contractions`. Put a rule earlier with `rule-order` when its output feeds a later rule.

```yaml
rule-order:
  - strip-strong
  - header-case
  - header-increment
```

Code and data rules run in registry order, then `rule-order` is applied the same way, and each rule repeats until it makes no more edits, up to 16 rounds. Use `lynter explain PATH` to see the governing profile chain and the resulting rules for a file ([command line](/docs/cli/)).

Sources: `crates/lynter-core/src/lint/mod.rs`, `crates/lynter-core/src/rules/mod.rs`, `crates/lynter-code/src/runner.rs`, `crates/lynter-core/README.md`

## Fixing and report-only rules

Most rules fix: `lynter fix` applies their edits and `lynter check` lists them without writing. A smaller group only reports, and `lynter fix` never changes the text on their account.

- Some rules are report-only twins of a fixing rule: `join-paragraph-lines` and `prose-list-to-sentences` flag, while `join-paragraph-lines-fix` and `prose-list-to-sentences-fix` rewrite. They share one detector, so enabling the `-fix` twin is the opt-in to the rewrite.
- `code-checks` reports matches in fenced code, and `code-checks-fix` applies checks that define a replacement.
- `note-type-validate`, `kebab-case-filename`, `folder-naming`, `filename-pattern` and `frontmatter-schema` only report.
- `tag-format` and `task-format` do both: they rewrite what is safe (tag case, checkbox spacing, date format and order) and report the rest.
- Every code and data rule only reports, except `json-key-order` and `yaml-key-order`, which sort keys.

A violation carries the rule id, a message, a line and a flag `fixed`. For a rule that rewrote the text, the linter records one violation with the rule's description at the first changed line and `fixed` true. For a report-only rule the violations keep `fixed` false.

Sources: `crates/lynter-core/src/lint/mod.rs`, `crates/lynter-core/src/rules/notes/mod.rs`, `crates/lynter-core/src/rules/content/join_paragraph_lines.rs`, `crates/lynter-core/src/rules/content/prose_list.rs`, `crates/lynter-code/src/runner.rs`

## Protected regions

Rules never edit inside regions where Markdown or YAML has its own syntax. The document model marks these regions once per parse, and rules ask whether a range is protected before they edit it.

| Region | Examples |
| --- | --- |
| Frontmatter | The YAML block at the top |
| Code | Fenced code, indented code, inline code |
| Math | Display math blocks, inline math |
| Tables | Table rows and delimiter rows |
| Links | Wikilinks, URLs, link targets |
| HTML | HTML blocks and inline HTML |

Protection comes from two sources that are combined: a line scan that masks frontmatter, fences, indented code, block math, tables, inline code, inline math, wikilinks and URLs, and the Markdown parse (tables, footnotes, strikethrough, task lists, math, wikilinks and YAML metadata blocks), which adds code, math, table, metadata and HTML ranges. Protection only ever grows when both are applied. Rules that work on whole lines skip a line when it falls inside a block region.

Two cases edit protected content on purpose. The frontmatter rules edit the frontmatter block on purpose, and property tests check that they leave the body alone. `code-checks-fix` edits the content lines of real code fences. `header-case` rewrites whole heading lines, so for it only whole-line regions count as protected.

Sources: `crates/lynter-core/src/zones.rs`, `crates/lynter-core/src/document.rs`, `crates/lynter-core/src/rule.rs`, `crates/lynter-core/tests/props.rs`

## Idempotence

Fixing is idempotent: running a rule on its own output makes no further edit, and a rule's check finds nothing to do after its fix. Property tests assert this for every rule on generated documents. They also assert that edits do not touch protected regions, that frontmatter rules do not change the body, and that check mode never changes the text. The edit runner supports this by sorting edits, dropping any that overlap an earlier one, and applying the rest as splices on the original text.

Sources: `crates/lynter-core/tests/props.rs`, `crates/lynter-core/tests/compat_props.rs`, `crates/lynter-core/tests/props_slice2.rs`, `crates/lynter-core/src/edit.rs`

## Compiled defaults

With no `lynter.yaml`, the compiled defaults apply, and `lynter init` writes a starter file with the same set. Ten rules are on, and five more are present but off because they rewrite wording or names. Any rule not listed here is off until you add it to your config.

| Rule | State | Options in the defaults |
| --- | --- | --- |
| `yaml-key-sort` | On | `priority-keys`: `preset`, `domain`, `category`, `sub-category`, `types`, `date-created`, `date-revised`, `links` |
| `yaml-timestamp` | On | none |
| `alphabetize-property-values` | On | none |
| `paragraph-spacing` | On | `blank-lines: 1` |
| `list-style` | On | `marker: "-"`, `tighten: true` |
| `replace-em-dash` | On | none |
| `code-checks` | On | none, no-op until a check is enabled |
| `code-checks-fix` | On | none, no-op until a fixing check is enabled |
| `join-paragraph-lines` | On, reports only | none |
| `prose-list-to-sentences` | On, reports only | none |
| `strip-strong` | Off | none |
| `header-case` | Off | none |
| `contractions` | Off | none |
| `h1-matches-stem` | Off | none |
| `kebab-case-filename` | Off | none |

The defaults also add a `base` note type, set `rename.mode` to `flag` and ignore nothing. Code and data rules have no compiled defaults. In the index below, "On" means on in this baseline.

Sources: `crates/lynter-core/src/config/defaults.rs`, `crates/lynter-core/src/config/scaffold.rs`, `crates/lynter-core/README.md`, `docs/lynter-yaml.md`

## Rule index

Every rule id, the page section that documents it, whether it fixes or only reports, and its default state. The anchors are the rule ids, so `/docs/rules-content/#contractions` is the entry for `contractions`.

| Rule | Group | Action | Default |
| --- | --- | --- | --- |
| [`yaml-key-sort`](/docs/rules-frontmatter/#yaml-key-sort) | Frontmatter keys and values | Fixes | On |
| [`yaml-timestamp`](/docs/rules-frontmatter/#yaml-timestamp) | Frontmatter keys and values | Fixes | On |
| [`alphabetize-property-values`](/docs/rules-frontmatter/#alphabetize-property-values) | Frontmatter keys and values | Fixes | On |
| [`dedupe-yaml-array-values`](/docs/rules-frontmatter/#dedupe-yaml-array-values) | Frontmatter keys and values | Fixes | Off |
| [`escape-yaml-special-characters`](/docs/rules-frontmatter/#escape-yaml-special-characters) | Frontmatter keys and values | Fixes | Off |
| [`format-yaml-arrays`](/docs/rules-frontmatter/#format-yaml-arrays) | Frontmatter keys and values | Fixes | Off |
| [`format-tags-in-yaml`](/docs/rules-frontmatter/#format-tags-in-yaml) | Frontmatter keys and values | Fixes | Off |
| [`insert-yaml-attributes`](/docs/rules-frontmatter/#insert-yaml-attributes) | Frontmatter keys and values | Fixes | Off |
| [`remove-yaml-keys`](/docs/rules-frontmatter/#remove-yaml-keys) | Frontmatter keys and values | Fixes | Off |
| [`move-tags-to-yaml`](/docs/rules-frontmatter/#move-tags-to-yaml) | Frontmatter keys and values | Fixes | Off |
| [`note-type-insert-keys`](/docs/rules-frontmatter/#note-type-insert-keys) | Note types | Fixes | Off |
| [`note-type-key-sort`](/docs/rules-frontmatter/#note-type-key-sort) | Note types | Fixes | Off |
| [`note-type-date-keys`](/docs/rules-frontmatter/#note-type-date-keys) | Note types | Fixes | Off |
| [`note-type-validate`](/docs/rules-frontmatter/#note-type-validate) | Note types | Reports | Off |
| [`kebab-case-filename`](/docs/rules-frontmatter/#kebab-case-filename) | Filenames and folders | Reports | Off |
| [`h1-matches-stem`](/docs/rules-frontmatter/#h1-matches-stem) | Filenames and folders | Fixes | Off |
| [`folder-naming`](/docs/rules-frontmatter/#folder-naming) | Filenames and folders | Reports | Off |
| [`filename-pattern`](/docs/rules-frontmatter/#filename-pattern) | Filenames and folders | Reports | Off |
| [`tag-format`](/docs/rules-frontmatter/#tag-format) | Tags, tasks and schema | Fixes and reports | Off |
| [`task-format`](/docs/rules-frontmatter/#task-format) | Tags, tasks and schema | Fixes and reports | Off |
| [`frontmatter-schema`](/docs/rules-frontmatter/#frontmatter-schema) | Tags, tasks and schema | Reports | Off |
| [`strip-strong`](/docs/rules-content/#strip-strong) | Wording | Fixes | Off |
| [`replace-em-dash`](/docs/rules-content/#replace-em-dash) | Wording | Fixes | On |
| [`contractions`](/docs/rules-content/#contractions) | Wording | Fixes | Off |
| [`header-case`](/docs/rules-content/#header-case) | Wording | Fixes | Off |
| [`remove-trailing-punctuation-in-heading`](/docs/rules-content/#remove-trailing-punctuation-in-heading) | Wording | Fixes | Off |
| [`emphasis-style`](/docs/rules-content/#emphasis-style) | Inline text | Fixes | Off |
| [`strong-style`](/docs/rules-content/#strong-style) | Inline text | Fixes | Off |
| [`quote-style`](/docs/rules-content/#quote-style) | Inline text | Fixes | Off |
| [`proper-ellipsis`](/docs/rules-content/#proper-ellipsis) | Inline text | Fixes | Off |
| [`no-bare-urls`](/docs/rules-content/#no-bare-urls) | Inline text | Fixes | Off |
| [`default-language-for-code-fences`](/docs/rules-content/#default-language-for-code-fences) | Inline text | Fixes | Off |
| [`join-paragraph-lines`](/docs/rules-content/#join-paragraph-lines) | Prose reshaping | Reports | On |
| [`join-paragraph-lines-fix`](/docs/rules-content/#join-paragraph-lines-fix) | Prose reshaping | Fixes | Off |
| [`prose-list-to-sentences`](/docs/rules-content/#prose-list-to-sentences) | Prose reshaping | Reports | On |
| [`prose-list-to-sentences-fix`](/docs/rules-content/#prose-list-to-sentences-fix) | Prose reshaping | Fixes | Off |
| [`consecutive-blank-lines`](/docs/rules-spacing-and-structure/#consecutive-blank-lines) | Blank lines | Fixes | Off |
| [`empty-line-around-blockquotes`](/docs/rules-spacing-and-structure/#empty-line-around-blockquotes) | Blank lines | Fixes | Off |
| [`empty-line-around-code-fences`](/docs/rules-spacing-and-structure/#empty-line-around-code-fences) | Blank lines | Fixes | Off |
| [`empty-line-around-horizontal-rules`](/docs/rules-spacing-and-structure/#empty-line-around-horizontal-rules) | Blank lines | Fixes | Off |
| [`empty-line-around-math-block`](/docs/rules-spacing-and-structure/#empty-line-around-math-block) | Blank lines | Fixes | Off |
| [`empty-line-around-tables`](/docs/rules-spacing-and-structure/#empty-line-around-tables) | Blank lines | Fixes | Off |
| [`heading-blank-lines`](/docs/rules-spacing-and-structure/#heading-blank-lines) | Blank lines | Fixes | Off |
| [`line-break-at-document-end`](/docs/rules-spacing-and-structure/#line-break-at-document-end) | Blank lines | Fixes | Off |
| [`paragraph-spacing`](/docs/rules-spacing-and-structure/#paragraph-spacing) | Blank lines | Fixes | On |
| [`trailing-spaces`](/docs/rules-spacing-and-structure/#trailing-spaces) | Spaces within lines | Fixes | Off |
| [`remove-multiple-spaces`](/docs/rules-spacing-and-structure/#remove-multiple-spaces) | Spaces within lines | Fixes | Off |
| [`list-style`](/docs/rules-spacing-and-structure/#list-style) | Lists | Fixes | On |
| [`unordered-list-style`](/docs/rules-spacing-and-structure/#unordered-list-style) | Lists | Fixes | Off |
| [`ordered-list-style`](/docs/rules-spacing-and-structure/#ordered-list-style) | Lists | Fixes | Off |
| [`remove-empty-list-markers`](/docs/rules-spacing-and-structure/#remove-empty-list-markers) | Lists | Fixes | Off |
| [`space-after-list-markers`](/docs/rules-spacing-and-structure/#space-after-list-markers) | Lists | Fixes | Off |
| [`header-increment`](/docs/rules-spacing-and-structure/#header-increment) | Headings | Fixes | Off |
| [`headings-start-line`](/docs/rules-spacing-and-structure/#headings-start-line) | Headings | Fixes | Off |
| [`code-checks`](/docs/code-linting/#code-checks) | Fenced code checks | Reports | On |
| [`code-checks-fix`](/docs/code-linting/#code-checks-fix) | Fenced code checks | Fixes | On |
| [`json-syntax`](/docs/code-linting/#json-syntax) | Data rules | Reports | Off |
| [`json-duplicate-keys`](/docs/code-linting/#json-duplicate-keys) | Data rules | Reports | Off |
| [`json-key-order`](/docs/code-linting/#json-key-order) | Data rules | Fixes | Off |
| [`yaml-syntax`](/docs/code-linting/#yaml-syntax) | Data rules | Reports | Off |
| [`yaml-duplicate-keys`](/docs/code-linting/#yaml-duplicate-keys) | Data rules | Reports | Off |
| [`yaml-key-order`](/docs/code-linting/#yaml-key-order) | Data rules | Fixes | Off |
| [`yaml-schema`](/docs/code-linting/#yaml-schema) | Data rules | Reports | Off |
| [`identifier-case`](/docs/code-linting/#identifier-case) | Source code rules | Reports | Off |
| [`syntax-error`](/docs/code-linting/#syntax-error) | Source code rules | Reports | Off |
| [`comment-tag`](/docs/code-linting/#comment-tag) | Source code rules | Reports | Off |
| [`commented-out-code`](/docs/code-linting/#commented-out-code) | Source code rules | Reports | Off |
| [`max-nesting-depth`](/docs/code-linting/#max-nesting-depth) | Source code rules | Reports | Off |
| [`max-function-lines`](/docs/code-linting/#max-function-lines) | Source code rules | Reports | Off |

Sources: `crates/lynter-core/src/rules/mod.rs`, `crates/lynter-core/src/rules/compat/mod.rs`, `crates/lynter-core/src/rules/content/mod.rs`, `crates/lynter-core/src/rules/notes/mod.rs`, `crates/lynter-core/src/notetypes/mod.rs`, `crates/lynter-code/src/runner.rs`
