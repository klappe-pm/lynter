# lynter-core

The Lynter core: a pulldown-cmark document model, an edit runner, the `lynter.yaml` config loader, the scope engine, and the rule set. Rules are pure functions from a document and options to byte-range edits.

## Entry points

- `lint::lint_file(text, path, config, today, scope_context)` resolves the profile chain and note type, injects rule options, runs the enabled rules once each in registry order (adjusted by `rule-order`), and returns the text, violations, profile chain, and note type. Report-only rules flag and never rewrite. The file path reaches rules through the `file-path` option.
- `config::scaffold()` returns a commented starter `lynter.yaml` that parses without errors or warnings.
- `config::default_config()` is the config used when no file exists.
- `config::migrate_data_json(text)` turns a plugin `data.json` into `lynter.yaml` text.

## Defaults for new users

A new user gets a quiet baseline. These rules are present in the defaults and the scaffold but disabled, so enabling one is a one-word edit (`enabled: true`):

| Rule | What it does |
| --- | --- |
| `contractions` | rewrites phrases such as "it is" to "it's" |
| `header-case` | applies a casing style per heading level, including H1 |
| `strip-strong` | removes `**` and `__` emphasis |
| `h1-matches-stem` | keeps the first H1 equal to the filename |
| `kebab-case-filename` | flags filenames that are not kebab-case |

Frontmatter key sort, timestamps, value alphabetizing, paragraph spacing, list style, and em dash replacement stay on. `join-paragraph-lines` and `prose-list-to-sentences` run report-only until their `-fix` twins are enabled.

## Parity with the TypeScript core

The fixtures under `tests/fixtures` are frozen and hash-checked; `tests/parity.rs` and `tests/parity_slice2.rs` replay them against Rust with the line-scan protection that matches TypeScript.

Known differences, each intended:

- `paragraph-spacing` adds a trailing newline only when missing. TypeScript appends one more on every pass when a code or math block runs to the end of the file.
- `code-checks` and `code-checks-fix` ignore fences that open inside YAML frontmatter.
- `migrate` writes a `lynter.yaml` header naming Lynter.
- List value sorting uses an approximation of locale collation (punctuation, digits, letters, accents as ties).
- Rule patterns in `code-checks` use the `regex` and `fancy-regex` engines, so JavaScript-only syntax is skipped as an invalid pattern.
- Long YAML strings are never folded and multi-line strings are written double-quoted.
- Provider config merging is not part of `lint_file` yet.

## Checks

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --check
```
