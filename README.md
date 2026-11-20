# Lynter

Lynter lints and fixes notes and code from one `lynter.yaml`. Markdown notes get consistent frontmatter, headings, lists, spacing and wording. Fenced code and source files get declarative checks, with optional adapters for ruff and Biome. The same Rust core runs in a command line tool, in an Obsidian plugin (compiled to WebAssembly), and in the playground on [lynter.dev](https://lynter.dev).

Rules are scoped. Profiles match files by path, extension, frontmatter, tag, age, backlink or alias, and each profile overrides the defaults key by key. Note types describe the frontmatter a kind of note must carry. Loading is fail closed: an invalid `lynter.yaml` is reported in full and never applied in part.

## Install

### Obsidian plugin

The plugin lives in `plugin/`. Build it, then copy `main.js`, `manifest.json` and `styles.css` into a folder named `lynter` in the vault's plugin directory and enable Lynter under Settings, Community plugins. You need Node.js 20 or later.

```bash
cd plugin && npm ci && npm run build
mkdir -p /path/to/vault/.obsidian/plugins/lynter
cp main.js manifest.json styles.css /path/to/vault/.obsidian/plugins/lynter/
```

### Command line

The CLI is the `lynter-cli` crate and installs a binary named `lynter`. You need a stable Rust toolchain.

```bash
cargo install --path crates/lynter-cli
cd /path/to/notes && lynter init
cd /path/to/notes && lynter check .
cd /path/to/notes && lynter fix .
```

`lynter explain PATH` shows which profiles and rules govern a file. `lynter fix --stdin` filters text from stdin for editors and hooks.

## Configuration

`lynter init` writes a commented starter. A small file looks like this:

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
    replace-em-dash:
      enabled: true
profiles:
  journal:
    match:
      path: ["journal/**"]
    rules:
      replace-em-dash:
        enabled: false
  python:
    match:
      extension: [py]
    adapters:
      ruff: {enabled: true, select: [E, F]}
```

The full reference is [docs/lynter-yaml.md](docs/lynter-yaml.md). Guides, examples and a playground are at [lynter.dev](https://lynter.dev).

## Repository layout

| Path | What it holds |
| --- | --- |
| `crates/lynter-core` | Rules, scope engine, config loader and note types. |
| `crates/lynter-code` | Code rules and the ruff and Biome adapters. |
| `crates/lynter-cli` | The `lynter` command. |
| `crates/lynter-wasm` | The WebAssembly build used by the plugin and the site. |
| `plugin/` | The Obsidian plugin. |
| `packages/site` | The lynter.dev site. |
| `examples/` | Sample vaults with a `lynter.yaml` each. |

See [CONTRIBUTING.md](CONTRIBUTING.md) to run the checks and [SECURITY.md](SECURITY.md) to report a vulnerability.

## License

MIT. See [LICENSE](LICENSE).
