---
title: Architecture
lede: How the workspace is split into crates and packages, and how one document moves through loading, scope resolution, the rule runner and edits.
---

Lynter is a Cargo workspace with a TypeScript plugin and a Next.js site around it. The Rust core owns every Markdown rule, the config loader and the scope engine. Each surface is a thin host that gathers a file, a path and a date, calls the core, and decides what to do with the answer.

## The workspace

| Path | Owns |
| --- | --- |
| `crates/lynter-core` | The document model, the rule contract, the runner, the config loader, the scope engine, note types, and every Markdown rule. |
| `crates/lynter-code` | A tree-sitter language registry keyed by file extension, the JSON and YAML data rules, the code rules, and the ruff and Biome adapters. |
| `crates/lynter-cli` | The `lynter` binary: argument parsing, config discovery, file discovery, output formats and exit codes. |
| `crates/lynter-wasm` | A `wasm-bindgen` wrapper that exposes a few string-in, string-out functions to JavaScript. |
| `plugin/` | The Obsidian plugin. It bundles the WebAssembly build into `main.js`. |
| `packages/site` | The lynter.dev site and playground. It serves the same WebAssembly build from `public/wasm`. |

Dependencies point one way. `lynter-core` depends on no other workspace member. `lynter-code` depends on `lynter-core`. `lynter-cli` depends on both. `lynter-wasm` depends only on `lynter-core`, which is why the plugin and the playground lint Markdown but do not run code rules or adapters. The adapters module in `lynter-code` is compiled out on `wasm32`.

```flow
lynter-core -> lynter-code -> lynter-cli
lynter-core -> lynter-wasm -> plugin and site
```

Sources: `Cargo.toml`, `crates/lynter-core/Cargo.toml`, `crates/lynter-code/Cargo.toml`, `crates/lynter-cli/Cargo.toml`, `crates/lynter-wasm/Cargo.toml`, `crates/lynter-code/src/lib.rs`

## The document model

`Document::parse` runs one `pulldown-cmark` parse and computes three things: line spans, protected zones, and list item offsets. Rules read from a `Document` and never parse on their own.

A zone is a half-open byte range that content rules must not edit. A line scan finds frontmatter, fenced code, indented code, block math, tables, inline code, inline math, wikilinks and URLs. The Markdown parse then adds HTML, code, math, table and metadata ranges on top, so protection only grows. Zone kinds that cover whole lines (frontmatter, fences, indented code, math blocks, tables and parsed blocks) are called block zones, and line-based rules skip those lines entirely. A `Protection::LineScan` mode keeps only the line scan, which the test suite uses to replay frozen fixtures.

Sources: `crates/lynter-core/src/document.rs`, `crates/lynter-core/src/zones.rs`

## The rule contract

A rule implements the `Rule` trait: a stable kebab-case `id`, a one-line `description`, `default_options`, and `check`, which maps a document and options to a list of edits. An edit is a byte range, a replacement string and a message. Rules are pure, so check and fix share one code path.

A rule declares an edit scope, `Body` by default, or `Frontmatter` for rules that rewrite the YAML block. `builtin_rules()` returns every rule in registry order: the wording rules first, then the content rules, the compat rule family, the frontmatter and note-type rules, and the notes rules.

Sources: `crates/lynter-core/src/rule.rs`, `crates/lynter-core/src/rules/mod.rs`

## Edits

`edit::normalize` sorts edits by start offset and drops any edit that overlaps an earlier one, or that repeats the same range. `edit::apply` then splices the surviving edits into the original text in one pass. Because overlap is resolved before applying, two rules can never corrupt each other's output within a round.

Report-only rules emit a no-op edit whose replacement equals the original text. The runner treats an unchanged result as a finding rather than a fix.

Sources: `crates/lynter-core/src/edit.rs`, `crates/lynter-core/src/lint/mod.rs`

## From document to result

`lint::lint_file(text, path, config, today, scope_context)` is the single entry point for a Markdown file. The CLI calls it for `.md` and `.txt` files, and the WebAssembly `lint` export calls it too.

```flow
Text and path -> Split frontmatter -> Scope resolver -> Option injection -> Rule loop -> LintResult
```

1. It splits the frontmatter and builds file facts: the path, the raw frontmatter, inline tags, and the date.
2. The `Resolver` merges `defaults.rules` with every matching profile and picks the note type and rule order. See [scopes and profiles](/docs/scopes-and-profiles/).
3. It injects options that only the host knows: the original text and date for timestamp rules, the `code-checks` block, the `rename.strategy` value, the matched note type's schema for the four `note-type-*` rules, and the file path as the `file-path` option.
4. It selects the rules whose `enabled` is `true`, sorts them into a fixed canonical order, then applies `rule-order` on top. Ids listed in `rule-order` run first, in that order. The rest keep canonical order.
5. Each rule runs once. `strip-strong` and `contractions` may loop up to 64 times inside their single turn. After each rule the text is re-parsed.
6. A rule that changed the text yields one violation with `fixed: true`. A report-only rule yields violations with `fixed: false`, and its output is discarded.

The result carries the new text, whether it changed, the violations, the profile chain (`defaults` first) and the note type. A separate `runner::run` API, with `Check` and `Fix` modes, loops the selected rules to a fixed point with a ceiling of 64 rounds.

Sources: `crates/lynter-core/src/lint/mod.rs`, `crates/lynter-core/src/runner.rs`

## How the hosts differ

The CLI decides by extension. Files ending in `.md` or `.txt` go to `lint_file`. Everything else goes to `lynter_code::lint_source`, which resolves a profile with the same `Resolver`, runs the enabled code and data rules, and returns the same `LintResult` shape. After that the CLI optionally runs a ruff or Biome adapter and reports its findings alongside.

The plugin and the site never reimplement a rule. They call `lint`, `validate_config`, `profile_chain`, `convert_data_json`, `scaffold` and `version` through the WebAssembly module. See [the WebAssembly API](/docs/wasm-api/).

Sources: `crates/lynter-cli/src/lint.rs`, `crates/lynter-code/src/runner.rs`, `crates/lynter-wasm/src/lib.rs`, `plugin/src/engine.ts`, `packages/site/src/engine.ts`
