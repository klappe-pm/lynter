---
title: WebAssembly API
lede: The browser and embedding contract of the lynter-wasm crate, with every export, its exact JSON shapes, and how the site and the plugin load it.
---

`crates/lynter-wasm` compiles the Rust core to WebAssembly with `wasm-bindgen`. Every export takes strings and returns a string, and results are JSON text. Exports do not throw for bad input. Failures are reported inside the JSON. The crate documents this as the stable contract. New fields may be added to the shapes below, but existing fields do not change.

## Exports

| Export | Arguments | Returns |
| --- | --- | --- |
| `lint` | `config_yaml`, `note`, `path`, `today` | JSON text, a success or failure object. |
| `validate_config` | `yaml` | JSON text with `ok`, `errors` and `warnings`. |
| `profile_chain` | `config_yaml`, `path` | JSON array of profile names. |
| `convert_data_json` | `json` | `lynter.yaml` text, or `undefined`. |
| `scaffold` | none | The starter `lynter.yaml` text. |
| `version` | none | The crate version, such as `0.1.0`. |

`today` is a `yyyy-MM-dd` date. An empty `path` is treated as `note.md`. The `path` is relative to the config's directory, so profile `path` globs and path-based rules see what they would see in the CLI.

Sources: `crates/lynter-wasm/src/lib.rs`

## lint

`lint` parses the config and, if it is valid, calls the same `lint_file` function the CLI uses for Markdown, with no backlink or alias facts. It returns one of two shapes.

Success:

```json
{"ok":true,"text":"# a\n","changed":false,"rules":[],"profileChain":["defaults"],"noteType":null,"violations":[]}
```

| Field | Type | Meaning |
| --- | --- | --- |
| `ok` | boolean | `true`. |
| `text` | string | The note after the enabled rules ran. Report-only rules do not change it. |
| `changed` | boolean | Whether `text` differs from the input. |
| `rules` | string array | Each rule id that reported, once, in order of first report. |
| `profileChain` | string array | Applied profiles, `defaults` first. |
| `noteType` | string or null | The resolved note type, or null. |
| `violations` | array | Objects with `rule` (string), `message` (string), `line` (1-based number) and `fixed` (boolean). |

Failure, for an invalid config:

```json
{"ok":false,"errors":["version: must be 1"]}
```

Each error is a string, formatted `path: message` when the error has a key path and the bare message otherwise. Config warnings are not returned. A rule that rewrites text yields one violation with `fixed: true` whose message is the rule description. A report-only finding has `fixed: false`.

Sources: `crates/lynter-wasm/src/lib.rs`, `crates/lynter-core/src/lint/mod.rs`

## validate_config

`validate_config` parses and validates a config without linting anything.

```json
{"ok":false,"errors":[{"message":"must be 1","line":1,"path":"version"}],"warnings":[]}
```

`ok` is true exactly when `errors` is empty. `warnings` has the same shape and lists what loads but is ignored: unknown keys, and rule ids that no rule answers to, with the closest known id suggested when one is near. `line` is the 1-based YAML line when the parser's message names one or the key can be found in the text, and `null` otherwise. `path` is the config key path, and is an empty string when the error is not tied to a key. Validity here means the core's rules in [configuration](/docs/configuration/). Adapter blocks are not part of it.

Sources: `crates/lynter-wasm/src/lib.rs`, `crates/lynter-core/src/config/validate.rs`

## profile_chain

`profile_chain(config_yaml, path)` returns the profile names that govern a path, as a JSON array such as `["defaults","notes"]`. It resolves against an empty note, so matchers that need frontmatter or tags can only match what the path itself provides. It returns `[]` when the config is invalid.

Sources: `crates/lynter-wasm/src/lib.rs`, `crates/lynter-core/src/scope.rs`

## convert_data_json, scaffold and version

`convert_data_json` turns an Obsidian Linter `data.json` into `lynter.yaml` text and returns `undefined` when the input cannot be converted, that is, when it is not valid JSON or not a JSON object. Input that is already a valid config comes back unchanged. See [importing settings](/docs/importing-settings/). `scaffold` returns the commented starter from `config::scaffold()`. `version` returns the crate version at compile time.

Sources: `crates/lynter-wasm/src/lib.rs`, `crates/lynter-core/src/config/migrate.rs`, `crates/lynter-core/src/config/scaffold.rs`

## What it does not do

The WebAssembly build depends only on `lynter-core`. It lints Markdown. It does not run code or data rules, adapters, the `ignore` key, file discovery or renames, because those live in the CLI and `lynter-code`. See [architecture](/docs/architecture/).

Sources: `crates/lynter-wasm/Cargo.toml`, `crates/lynter-code/src/lib.rs`

## Building and loading

`packages/site/scripts/build-wasm.sh` builds the crate for `wasm32-unknown-unknown` in release mode with size optimizations and link-time optimization, then runs `wasm-bindgen --target web`. It requires `wasm-bindgen-cli` 0.2.130 exactly and fails if the gzipped module exceeds 900,000 bytes. The output, `lynter_wasm.mjs` and `lynter_wasm_bg.wasm`, is committed under `packages/site/public/wasm`, so building the site needs no Rust toolchain.

The site loads the module once, in the browser, with a dynamic import and an explicit path to the binary.

```js
const mod = await import('/wasm/lynter_wasm.mjs');
await mod.default({module_or_path: '/wasm/lynter_wasm_bg.wasm'});
const result = JSON.parse(mod.lint(configText, noteText, 'notes/a.md', '2026-01-01'));
```

Until the module is ready the site's wrapper returns the note unchanged. The playground at [/playground/](/playground/) is a front end over `lint`, `profile_chain` and `convert_data_json`.

The plugin loads the same glue through its bundler, with the binary inlined as gzipped base64 and passed as bytes. Its wrapper accepts either a `lint` or a `lint_file` function and feature-detects `validate_config`. See [the Obsidian plugin](/docs/obsidian-plugin/).

Sources: `packages/site/scripts/build-wasm.sh`, `packages/site/src/engine.ts`, `plugin/src/wasm.ts`, `plugin/src/engine.ts`
