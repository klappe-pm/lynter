# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

Primary user: anyone who keeps notes or code in a git repository and wants one reviewed configuration to format them. That covers an Obsidian user formatting notes inside the app, and the same person or their collaborators running the identical rules outside the app in git hooks and CI through the CLI.

## Product Purpose

Lynter lints and fixes Markdown notes and source code from a single `lynter.yaml`. For notes it covers YAML frontmatter keys, arrays and timestamps, heading case and increments, spacing, lists and line breaks, emphasis styles, contractions, filenames and their H1, and Markdown special regions. For code it runs declarative checks over fenced blocks and files, with optional ruff and Biome adapters. It exists so that formatting lives in one versioned configuration file rather than in hidden per-install settings.

Success means a vault or repository stays consistently formatted with no manual effort, the rules that produced it are reviewable in git, and the same rules produce identical results in the app, in a pre-commit hook, and in CI.

## Positioning

Lynter keeps its configuration in a single `lynter.yaml` that lives in your git repository, so every rule change is versioned and reviewable. Rules are scoped by path, extension, frontmatter property or tag, and each scope inherits from a shared base so you only write what differs. The same deterministic rule engine, written in Rust, runs in the Obsidian plugin through WebAssembly, in git hooks and in CI through a CLI that needs no tokens, so results match locally and in the pipeline.

## Operating Context

- The Obsidian plugin in `plugin/` is a thin adapter over the WASM build of `lynter-core`. It runs on desktop and mobile and inherits the user's active theme.
- `lynter.yaml` at the vault or repository root is the single git-tracked source of truth for rules, profiles and note types. The loader fails closed: an invalid file is reported in full and never applied in part. The reference is `docs/lynter-yaml.md`.
- The CLI is the `lynter` binary from `crates/lynter-cli`: `check` (exit 1 on violations), `fix` (including `--stdin` filter mode), `explain` and `init`. `--json` emits one JSON line per file, and `--changed` targets git-changed files for pre-commit gates. Exit codes: 0 clean or fixed, 1 violations, 2 config or usage error.
- The site at lynter.dev in `packages/site` documents the product and runs a playground on the real rule core in the browser.

## Binding Constraints

- The settings UI uses Obsidian theme variables for every dimension, font family, size and weight. No hardcoded px, family or weight.
- The plugin behaves like a normal Obsidian plugin: clean, simple, and following Obsidian plugin best practices. It adds nothing that fights the user's theme or other plugins.
- The license is MIT.

## Brand Commitments

- Name: Lynter. The y is a nod to YAML being chosen over JSON for configuration.
- Visual commitment: inherit the user's Obsidian theme. The plugin's surfaces read as part of Obsidian, never as a bolted-on panel.

## Product Principles

1. Configuration is a versioned file, never hidden plugin state. Anything the UI can change is written to `lynter.yaml`, and a hand edit to the file wins.
2. One engine, identical results everywhere. The plugin, the pre-commit hook and CI run the same core, so a note that passes locally passes in the pipeline.
3. Write only what differs. Scopes inherit from the base; duplication is a defect.
4. Behave like a normal Obsidian plugin. Inherit the theme, follow plugin best practices, keep surfaces clean and simple, and add nothing global.
5. Fail closed and never silently. An invalid configuration is reported, not partly applied.
