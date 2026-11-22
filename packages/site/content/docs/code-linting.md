---
title: Code and data linting
lede: How Lynter lints fenced code, JSON, YAML and source files, which languages are supported, and what runs only in the command line.
---

Lynter lints more than notes. Regex checks run over fenced code blocks inside Markdown, data rules cover JSON and YAML files, tree-sitter rules cover source files, and optional adapters call ruff and Biome. The fenced code checks are part of `lynter-core`, so they run everywhere Lynter runs. Everything else lives in `lynter-code` and runs in the command line only. This page documents the 15 rules involved and the routing that decides which of them apply to a file.

## Where each part runs

The WebAssembly build links `lynter-core` only, so the Obsidian plugin and the [playground](/playground/) lint Markdown with the note rules and the fenced code checks, and nothing else. The code and data rules and the adapters are compiled into the CLI.

| Feature | CLI | Obsidian plugin | Browser playground |
| --- | --- | --- | --- |
| Note rules ([frontmatter](/docs/rules-frontmatter/), [content](/docs/rules-content/), [spacing](/docs/rules-spacing-and-structure/)) | Yes | Yes | Yes |
| `code-checks` and `code-checks-fix` on fenced blocks | Yes | Yes | Yes |
| JSON and YAML data rules | Yes | No | No |
| Tree-sitter code rules | Yes | No | No |
| ruff and Biome adapters | Yes | No | No |

The adapters module is compiled out on `wasm32` as well as being absent from the WebAssembly crate. The plugin lints Markdown notes. See [WebAssembly API](/docs/wasm-api/) and [Obsidian plugin](/docs/obsidian-plugin/).

Sources: `crates/lynter-wasm/Cargo.toml`, `crates/lynter-code/src/lib.rs`, `crates/lynter-cli/Cargo.toml`, `plugin/src/main.ts`

## How a file is routed

The CLI lints files with the extensions `md` and `txt`, plus every extension the code host handles: `json`, `yaml`, `yml`, and the source extensions in the table below. Files ending in `.md` or `.txt` go to the note rules. Any other file goes to the code host.

```flow
Target file -> .md or .txt -> Note rules and code-checks in lynter-core
Target file -> .json .yaml .yml or source -> Code and data rules in lynter-code -> ruff or Biome adapter
```

The code host resolves the file's profile from its path and extension, so [profiles](/docs/scopes-and-profiles/) can scope code rules by folder. Frontmatter and tag matches do not apply to non-Markdown files. It then runs the enabled rules in registry order, adjusted by `rule-order`, and each rule repeats until it makes no more edits, up to 16 rounds. If no code or data rule is enabled for a file, the file is skipped silently unless an adapter ran for it.

Code and data rules are off until enabled. The compiled defaults turn none of them on.

```yaml
version: 1
defaults:
  rules:
    json-syntax: {enabled: true}
    identifier-case: {enabled: true}
profiles:
  scripts:
    match: {path: ["scripts/**"]}
    rules:
      max-function-lines: {enabled: true, max: 40}
```

Findings the rules mark as fixable are applied to the file by `lynter fix`; the rest are reports. In practice only `json-key-order` and `yaml-key-order` rewrite files. See the [command line](/docs/cli/) for output formats and exit codes.

Sources: `crates/lynter-cli/src/lint.rs`, `crates/lynter-cli/src/files.rs`, `crates/lynter-code/src/runner.rs`

## Languages and cargo features

Each grammar sits behind a cargo feature, so a build links only what it needs. Both `lynter-code` and `lynter-cli` expose the same features, and the default set is every language except SQL.

| Language | Name in options | Extensions | Cargo feature | In default build |
| --- | --- | --- | --- | --- |
| Rust | `rust` | `rs` | `lang-rust` | Yes |
| Python | `python` | `py`, `pyi` | `lang-python` | Yes |
| TypeScript | `typescript` | `ts`, `mts`, `cts` | `lang-typescript` | Yes |
| TSX | `typescript` | `tsx` | `lang-typescript` | Yes |
| JavaScript | `javascript` | `js`, `mjs`, `cjs`, `jsx` | `lang-javascript` | Yes |
| SQL | `sql` | `sql` | `lang-sql` | No |
| Go | `go` | `go` | `lang-go` | Yes |
| Java | `java` | `java` | `lang-java` | Yes |
| C# | `csharp` | `cs` | `lang-csharp` | Yes |

If a grammar is not compiled in, the language counts as absent: the tree-sitter rules do not apply to those files, while JSON and YAML rules are unaffected because they do not use a grammar. To add SQL, build the CLI with `--features lang-sql`.

```bash
cargo build --release -p lynter-cli --features lang-sql
```

### The `languages` option

Any code or data rule stanza may carry `languages`, a list of language names from the table. The rule then runs only on files of those languages. Profiles have no `languages` matcher yet, so the filter lives in the code host. `enabled` and `languages` are not passed on as rule options.

```yaml
defaults:
  rules:
    identifier-case: {enabled: true, languages: [python, rust]}
```

A file with no grammar language, such as a JSON file, never passes a `languages` filter, so do not set it on the data rules. The `languages` key inside a `code-checks` entry is a different thing: it matches the language word on a fenced code block.

Sources: `crates/lynter-code/Cargo.toml`, `crates/lynter-code/src/language.rs`, `crates/lynter-code/src/runner.rs`, `crates/lynter-cli/Cargo.toml`

## Fenced code checks

The `code-checks` key defines regex checks that run over fenced code blocks in Markdown. A check is data, not code: a pattern tested line by line inside a fence. Messages never carry the matched text. These two rules belong to `lynter-core`, so they run in the CLI, the plugin and the playground.

A check is a mapping under `code-checks`, keyed by an id.

| Key | Type | Meaning |
| --- | --- | --- |
| `enabled` | boolean | The check runs only when this is `true`. |
| `description` | string | Free text. |
| `pattern` | string | The regular expression. An invalid pattern is skipped. |
| `flags` | string | Any of `i`, `m` and `s`. Other letters are ignored. |
| `message` | string | The message for the finding. |
| `languages` | list of strings | Fence languages the check applies to, compared to the first word after the opening fence, lowercased. Absent means every fence. |
| `paths` | list of globs | Vault-relative globs of the files the check applies to. Absent means every file. |
| `fix` | mapping | `fix.replacement` makes the check a fixing check. The replacement may use `$1`, `$&` and `$$`. |

Three built-in checks exist, all disabled until you set `enabled: true`. Fields you configure override the built-in ones field by field.

| Check | What it matches | Fixes |
| --- | --- | --- |
| `no-token-shaped-strings` | Token-shaped literals such as cloud keys, bearer tokens and long hex strings. | No |
| `no-gnu-only-flags-in-sh` | GNU-only shell idioms (`sed -i` without an empty suffix argument, `readlink -f`, `flock`, `timeout N`) in `sh`, `bash`, `zsh` and `shell` fences. | No |
| `no-trailing-whitespace-in-code` | Trailing spaces and tabs in any fence. | Yes, removes them |

```yaml
code-checks:
  no-token-shaped-strings: {enabled: true}
  no-trailing-whitespace-in-code: {enabled: true}
  no-var-in-js:
    enabled: true
    languages: [js, javascript]
    pattern: "^\\s*var\\s"
    message: "use let or const"
```

Only the content of real code fences is checked. A fence that opens inside frontmatter is ignored, and lines longer than 2000 characters are skipped. If any check has `enabled: true`, both rules below are switched on for you unless a stanza sets their own `enabled`.

### `code-checks`

Reports when an enabled check without a `fix` matches a line inside a fenced code block. Reports only, one finding for the note. On by default, and a no-op until a check is enabled.

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `checks` | mapping | from the top-level `code-checks` key | The check definitions. The linter fills this in; do not set it on the rule. |

### `code-checks-fix`

Applies every enabled check that defines `fix.replacement` to the lines of fenced code blocks. Fixes. On by default, and a no-op until a fixing check is enabled.

It takes the same `checks` option as `code-checks`. This is the one rule that edits inside code on purpose, and it edits only the content lines of real fences.

````markdown
```js
foo12 here··
```
````

The `·` marks trailing spaces. With `no-trailing-whitespace-in-code` enabled:

````markdown
```js
foo12 here
```
````

Sources: `crates/lynter-core/src/rules/content/code_checks.rs`, `crates/lynter-core/src/lint/mod.rs`, `crates/lynter-core/tests/fixtures/rules-code-checks.json`, `crates/lynter-core/tests/fixtures/rules-code-checks-fix.json`

## JSON and YAML data rules

Seven rules lint data files: three for `.json` and four for `.yaml` and `.yml`. All are off by default. They need no grammar, so they work in any build.

### `json-syntax`

Reports a JSON parse error with its line and column. Reports only. Off by default. No options.

```text
line 3, column 8: <parse error message>
```

### `json-duplicate-keys`

Reports a key that repeats inside one JSON object, naming the line of its first use. Reports only. Off by default. No options.

```text
duplicate key "x" (first on line 2)
```

### `json-key-order`

Sorts the keys of JSON objects, priority keys first, and keeps the file's layout. Fixes. Off by default.

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `priority` | list of strings | `[]` | Keys that sort first, in this order. |
| `rest` | string | `alphabetical` | `alphabetical` sorts the remaining keys; `keep` leaves them in place. |
| `deep` | boolean | `false` | Also sort nested objects, including objects inside arrays. |

```json
{"b": {"y":1,"x":2}, "a": 1}
```

```json
{"a": 1, "b": {"y":1,"x":2}}
```

### `yaml-syntax`

Reports the first YAML parse error with its line and column. Reports only. Off by default. No options.

### `yaml-duplicate-keys`

Reports a key that repeats inside one YAML mapping, naming the line of its first use. Reports only. Off by default. No options.

```text
duplicate key "x" (first on line 3)
```

### `yaml-key-order`

Sorts the top-level keys of a YAML file, priority keys first, keeping comments with their key. Fixes. Off by default.

Only the top level of a single-document block mapping is reordered. Nested mappings are not touched, and a file with aliases or several documents is skipped without a finding.

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `priority` | list of strings | `[]` | Keys that sort first, in this order. |
| `rest` | string | `alphabetical` | `alphabetical` or `keep`. |

```yaml
b: 1
a: 2
```

```yaml
a: 2
b: 1
```

### `yaml-schema`

Checks that required top-level keys exist and that values have the expected types. Reports only. Off by default.

A file whose top level is not a mapping is reported. A file that is not valid YAML gets no finding here; `yaml-syntax` reports that.

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `required` | list of strings | `[]` | Top-level keys that must be present. |
| `types` | mapping | `{}` | Key to type name: `null`, `boolean`, `integer`, `number`, `string`, `array` or `object`. `number` also accepts integers. |

```text
missing required key "version"
key "name" on line 1 must be string, found integer
```

Sources: `crates/lynter-code/src/rules/json.rs`, `crates/lynter-code/src/rules/json_parse.rs`, `crates/lynter-code/src/rules/yaml.rs`, `crates/lynter-code/src/rules/mod.rs`, `crates/lynter-code/tests/data.rs`

## Source code rules

Six rules read the tree-sitter parse of a source file. They apply only to files whose language grammar is compiled in, and all of them only report. The structural rules (comments, nesting, function size) use one table of node kinds per language, kept in step with each grammar by tests.

### `identifier-case`

Reports function and variable names that do not follow the case style set for the language. Reports only, since renaming a declaration without its references would break the code. Off by default.

Styles are `snake` (no uppercase letters), `camel` (no inner underscores, first letter lowercase), `pascal` (no inner underscores, first letter uppercase) and `mixed` (no underscores, any case). Leading and trailing underscores are ignored. For variables, names in all capitals pass when `allow-constants` is true. SQL checks the names in `CREATE TABLE` and `CREATE FUNCTION`.

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `functions` | mapping | per language, see below | Language name to style for functions and methods. |
| `variables` | mapping | per language, see below | Language name to style for variables. |
| `allow-constants` | boolean | `true` | Accept all-capital variable names. |
| `ignore` | list of strings | `[]` | Names never reported. |

The defaults are `snake` for Rust, Python and SQL functions and variables; `mixed` for Go; `pascal` functions and `camel` variables for C#; and `camel` for TypeScript, JavaScript and Java.

```rust
fn BadName() {}
```

```text
function "BadName" should be snake case
```

### `syntax-error`

Reports every error and missing node in the parse, with line and column. Reports only. Off by default. No options.

```text
syntax error at line N, column N
missing <node kind> at line N, column N
```

### `comment-tag`

Requires TODO, FIXME, HACK and XXX comments to carry an owner or an issue. Reports only. Off by default.

The pattern is a list of alternatives separated by `|`. In it, `{tag}` is the tag, `{owner}` is a name such as `dana`, `{issue}` is `#123` or `ABC-123`, and every other character is literal. A tag followed by a letter, digit or underscore (such as `TODOS`) is not a tag.

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `tags` | list of strings | `[TODO, FIXME, HACK, XXX]` | Tags to check. |
| `pattern` | string | `{tag}({owner}):\|{tag} {issue}:\|{tag}({issue}):` | Accepted forms. |

```rust
// TODO tidy this up
// TODO(dana): tidy this up
```

```text
TODO needs an owner or an issue, such as TODO(name): or TODO #123:
```

Only the first comment line is reported; the second is accepted.

### `commented-out-code`

Reports comments whose text parses as code in the same language. Reports only. Off by default.

A comment, or a run of consecutive own-line line comments, is parsed again. It is reported when the parse has no errors and contains at least one statement. Doc comments, short comments and text with no code punctuation are ignored.

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `min-length` | integer | `12` | Shortest comment text that is examined. |

```rust
// let total = add(1, 2);
```

### `max-nesting-depth`

Reports blocks nested deeper than a limit. Reports only. Off by default.

Each block adds a level. For SQL, subqueries and `CASE` expressions add levels. The finding points at the first line of the block that crosses the limit.

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `max` | integer | `4` | Deepest allowed nesting. |

```text
nesting depth 5 exceeds the maximum of 4
```

### `max-function-lines`

Reports functions or methods, or SQL statements, longer than a limit. Reports only. Off by default.

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `max` | integer | `60` | Longest allowed length in lines. |

```text
function item has 70 lines, more than the maximum of 60
```

Sources: `crates/lynter-code/src/rules/identifier_case.rs`, `crates/lynter-code/src/rules/syntax_error.rs`, `crates/lynter-code/src/rules/comment_tag.rs`, `crates/lynter-code/src/rules/commented_out_code.rs`, `crates/lynter-code/src/rules/structure.rs`, `crates/lynter-code/src/rules/kinds.rs`, `crates/lynter-code/tests/identifier_case.rs`, `crates/lynter-code/tests/structure_rules.rs`

## ruff and Biome adapters

Adapters run a native tool on source files next to the Lynter rules. They are not rules with ids of their own: each finding from a tool is reported under the tool's own code, prefixed with the adapter name, such as `ruff/F401` or `biome/lint/suspicious/noDebugger`. They exist only in native builds, so they never run in the plugin or the playground.

| Adapter | Files | Command it runs |
| --- | --- | --- |
| `ruff` | `py`, `pyi` | `ruff check --output-format json --no-cache`, then `--select` with your codes, `--fix` in fix mode, then your `args` |
| `biome` | `ts`, `tsx`, `mts`, `cts`, `js`, `jsx`, `mjs`, `cjs` | `biome lint --reporter=json --colors=off --no-errors-on-unmatched`, then `--write` in fix mode, then your `args` |

Adapters are off by default. Set them under `defaults.adapters`, and override them per profile under `profiles.<name>.adapters`, key by key.

| Key | Applies to | Type | Meaning |
| --- | --- | --- | --- |
| `enabled` | both | boolean | Turn the adapter on. Default false. |
| `path` | both | string | Path to the binary. A relative path resolves against the config's folder. Without it the tool is found on `PATH`. |
| `args` | both | list of strings | Extra arguments passed before the files. |
| `select` | `ruff` only | list of strings | Rule codes for `--select`. |

```yaml
defaults:
  adapters:
    ruff: {enabled: true, select: [E, F]}
    biome: {enabled: true}
```

An unknown adapter id, an unknown key or a wrong value type is a config error. If the tool is not installed, Lynter prints one notice per adapter per run and skips the file. If the tool crashes or prints output Lynter cannot parse, the run reports a single `adapter-error` for that file and exits with code 2. In fix mode Lynter writes its own fixes first and the tool then rewrites the file in place. Pass `--no-adapters` to skip adapters for a run.

Sources: `crates/lynter-code/src/adapters/mod.rs`, `crates/lynter-code/src/adapters/config.rs`, `crates/lynter-code/src/adapters/ruff.rs`, `crates/lynter-code/src/adapters/biome.rs`, `crates/lynter-cli/src/adapt.rs`, `crates/lynter-cli/src/main.rs`
