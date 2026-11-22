---
title: Overview
lede: What Lynter is, the four ways to run it, and what one run does from the config file to the rewritten note.
---

Lynter lints and fixes Markdown notes and code from a single `lynter.yaml`. Markdown notes get consistent frontmatter, headings, lists, spacing and wording. Fenced code and source files get declarative checks, and optional adapters for ruff and Biome. One Rust core does the work everywhere, so a rule behaves the same in a terminal, in Obsidian and in the browser.

## The four surfaces

All four surfaces read the same configuration file and call the same core. They differ in how they find the file, where they read notes from, and what they do with the result.

| Surface | What it is | How it finds `lynter.yaml` |
| --- | --- | --- |
| CLI | The `lynter` binary from the `lynter-cli` crate. Commands are `check`, `fix`, `explain` and `init`. | Walks up from each target file, or takes `--config PATH`. |
| Obsidian plugin | A thin TypeScript adapter that runs the core compiled to WebAssembly. | Reads the vault root only. |
| Playground | The page at [/playground/](/playground/), which runs the same WebAssembly build in your browser. | You paste or edit the config in the page. |
| CI and hooks | The CLI run from a pre-commit hook, a GitHub Action, or an editor filter such as `lynter fix --stdin`. | Same as the CLI. |

The pages for each surface are [the CLI](/docs/cli/), [the Obsidian plugin](/docs/obsidian-plugin/) and [the WebAssembly API](/docs/wasm-api/). The repository ships `.pre-commit-hooks.yaml` with `lynter-check` and `lynter-fix` hooks, which call a `lynter` binary on your `PATH`, and a composite action in `.github/actions/lynter` that downloads a release binary, verifies it against `SHA256SUMS` and runs `lynter check`.

Sources: `README.md`, `.pre-commit-hooks.yaml`, `.github/actions/lynter/action.yml`, `crates/lynter-cli/src/main.rs`, `plugin/src/main.ts`

## What one run does

A run lints one file at a time. For each Markdown or text file the host hands the core the file text, its path relative to the config directory, the config, and a date.

```flow
lynter.yaml -> Loader -> Scope resolver -> Rule runner -> Edits
```

1. The loader parses the YAML and validates it. Any structural error rejects the whole file, and nothing runs. See [configuration](/docs/configuration/).
2. The scope resolver decides which profiles govern the file and merges their rule settings over `defaults`. It also picks the note type. See [scopes and profiles](/docs/scopes-and-profiles/).
3. The rule runner selects the enabled rules, orders them, and asks each one for edits. A rule never changes text itself. It returns byte-range edits.
4. The edits are applied, and the result reports the new text, the violations, the profile chain and the note type. `check` reports and leaves the file alone. `fix` writes the new text back.

Rules that only report, such as `join-paragraph-lines`, `note-type-validate` and `kebab-case-filename`, produce violations with `fixed: false` and never rewrite the note. Regions such as frontmatter, code fences, math, tables, wikilinks and URLs are protected from content rules. See [architecture](/docs/architecture/).

Sources: `crates/lynter-core/src/lint/mod.rs`, `crates/lynter-core/src/runner.rs`, `crates/lynter-core/README.md`

## Failing closed

Loading is fail closed. If `lynter.yaml` has any structural error, Lynter reports every error with its key path and lints nothing. It never runs with a partly applied config. Unknown keys are the exception: they are ignored with a warning, so a typo is visible without blocking the run.

The CLI exits with status 2 when no config is found or the config is invalid, 1 when `check` finds violations, and 0 otherwise. See [the CLI page](/docs/cli/) for the full table.

Sources: `crates/lynter-core/src/config/validate.rs`, `crates/lynter-cli/src/main.rs`

## A minimal configuration

```yaml
version: 1
defaults:
  rules:
    yaml-key-sort:
      enabled: true
      priority-keys: [title, status, tags]
    paragraph-spacing:
      enabled: true
      blank-lines: 1
profiles:
  journal:
    match:
      path: ["journal/**"]
    rules:
      paragraph-spacing:
        enabled: false
```

A rule that is not listed, or is listed without `enabled: true`, does not run. A profile states only what differs from `defaults`, key by key. The complete key reference is at [/docs/reference/](/docs/reference/).

Sources: `docs/lynter-yaml.md`, `README.md`

## Where to go next

- Learn how the files fit together in [architecture](/docs/architecture/).
- Write your first config with [configuration](/docs/configuration/), then scope it with [scopes and profiles](/docs/scopes-and-profiles/).
- Describe the frontmatter a kind of note must carry in [note types](/docs/note-types/).
- See what each rule does in [rules overview](/docs/rules-overview/), [frontmatter rules](/docs/rules-frontmatter/), [content rules](/docs/rules-content/), [spacing and structure rules](/docs/rules-spacing-and-structure/) and [code linting](/docs/code-linting/).
- Try a config without installing anything in the [playground](/playground/).
- Bring settings from an Obsidian Linter `data.json` with [importing settings](/docs/importing-settings/).
- Understand how correctness is checked in [testing](/docs/testing/) and how builds ship in [releases](/docs/releases/).

Sources: `README.md`
