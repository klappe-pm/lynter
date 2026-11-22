---
title: Obsidian plugin
lede: How the Lynter plugin loads the WebAssembly core, reads lynter.yaml from the vault root, lints on save without looping, and runs on mobile.
---

The Obsidian plugin is a thin TypeScript adapter. It contains no rules. It reads `lynter.yaml` from the vault, passes the config text, a note and its path to the same Rust core the CLI uses (compiled to WebAssembly), and writes the result back. Rules live in the config file, not in plugin settings.

## Loading the WebAssembly core

The build embeds the core inside `main.js`, so the plugin needs no network access and no separate `.wasm` file in the vault. The esbuild config reads `lynter_wasm_bg.wasm` from `packages/site/public/wasm`, gzips it, encodes it as base64, and exposes it as a module. The same build bundles the `wasm-bindgen` glue.

On first use, `loadWasm` decodes the base64 text with `atob`, gunzips the bytes with `fflate`, and passes them to the glue's `init`. The result is cached for the session, so the module loads once. If loading fails, the cached promise is cleared and the next use tries again.

The build step enforces two limits. `main.js` must be no larger than 1,300,000 bytes, and the bundle may only include the plugin sources, the `obsidian` and `fflate` packages, and the committed WebAssembly files. The plugin CI workflow also fails if the bundle requires `fs` or `zlib` or uses `Buffer`.

Sources: `plugin/esbuild.config.mjs`, `plugin/src/wasm.ts`, `plugin/src/codec.ts`, `.github/workflows/plugin.yml`

## Reading lynter.yaml

The plugin looks for the config at the vault root only. It tries `lynter.yaml`, then the deprecated names `linter.yaml` and `lappe-linter.yaml`, and uses the first that exists. It does not walk up or down. See [configuration](/docs/configuration/) for the names and the loading rules.

The config is read again for each command and each save event, so edits to the file take effect without a reload. If the config text or name changed since the last run, the plugin forgets the output it remembered for the save guard. A deprecated name triggers one notice per session asking you to rename the file.

The plugin calls the WebAssembly `lint(config, note, path, today)` export. The path is vault-relative, so profile `path` globs work as they do in the CLI. The date is the local date, formatted `yyyy-MM-dd`. A config error comes back as a list of messages, and the note is left untouched. The plugin shows the first one as a notice.

The plugin does not apply the config's `ignore` key, and the `backlink` and `alias` matchers do not match here, because the plugin passes no such facts. See [scopes and profiles](/docs/scopes-and-profiles/).

Sources: `plugin/src/config.ts`, `plugin/src/lynter.ts`, `plugin/src/engine.ts`, `crates/lynter-wasm/src/lib.rs`

## Commands

| Command | What it does |
| --- | --- |
| Lint current file | Lints the active note. Also available from the ribbon icon. |
| Lint all files in the vault | Lints every Markdown file in turn, shows progress, and yields to the interface every 25 files. |
| Create lynter.yaml | Writes the starter config to the vault root. |

Writes go through `Vault.process`, an atomic read-modify-write, so a note that changes while the core runs is not overwritten with stale text. "Create lynter.yaml" validates the starter text first and refuses to write if any config file already exists under any of the three names.

Sources: `plugin/src/main.ts`, `plugin/src/lynter.ts`

## Lint on save

Lint on save is off by default. When you turn it on, the plugin reacts to Obsidian's `modify` event for Markdown files.

```flow
modify event -> Save guard check -> Lint -> Write only if changed
```

A note open in the active editor waits until it has been left alone for 2 seconds, so typing is never rewritten under you. A new modify event restarts the wait. A note in the background is linted at once. After the wait the plugin reads the file again and writes only if the text is still the text it linted.

Sources: `plugin/src/lynter.ts`, `plugin/src/main.ts`

## The save guard

Writing a file fires another modify event, which would lint again, forever. The save guard prevents that. It remembers, per path, the text Lynter last produced, whether or not it wrote it. When a modify event arrives and the file's content equals that remembered text, the plugin skips it. Content that is already the lint output is skipped even if Lynter did not write it.

The guard follows file renames, forgets deleted files, and clears entirely when the config changes. Deleting a file also cancels its pending debounced lint, and unloading the plugin cancels all of them.

Sources: `plugin/src/save-guard.ts`, `plugin/src/main.ts`, `plugin/src/lynter.ts`, `plugin/test/save-guard.test.ts`

## Settings

| Setting | Default | Effect |
| --- | --- | --- |
| Lint on save | Off | Tidies a note when Obsidian saves it. |
| Show notices | On | Shows a notice after each lint command. Error notices and the deprecated-name notice always show. |

Both are stored with `saveData`, and a missing or wrongly typed value falls back to its default. The settings tab also shows which config is active, a note when it uses a deprecated name, the validation result with each error, the config text, and a button to open the file. It does not edit rules.

Sources: `plugin/src/settings.ts`, `plugin/src/main.ts`

## Mobile

The manifest sets `isDesktopOnly` to `false` and requires Obsidian 1.12.0 or later. The plugin is built to run on mobile, where Node built-ins do not exist. It decodes base64 without `Buffer`, decompresses with a pure JavaScript library, uses no `fs` or `zlib`, and reads the vault only through the Obsidian adapter. The WebAssembly module is instantiated from bytes already in memory.

Sources: `plugin/manifest.json`, `plugin/src/codec.ts`, `plugin/esbuild.config.mjs`, `.github/workflows/plugin.yml`
