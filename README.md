<p align="center">
  <a href="https://lynter.dev">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="packages/site/public/brand/lockup-dark.svg">
      <img src="packages/site/public/brand/lockup.svg" alt="Lynter" width="196" height="48">
    </picture>
  </a>
</p>

<p align="center">
  <a href="https://github.com/klappe-pm/lynter/actions/workflows/ci.yml"><img src="https://github.com/klappe-pm/lynter/actions/workflows/ci.yml/badge.svg?branch=master" alt="CI status"></a>
  <a href="https://github.com/klappe-pm/lynter/actions/workflows/site.yml"><img src="https://github.com/klappe-pm/lynter/actions/workflows/site.yml/badge.svg?branch=master" alt="Site deploy status"></a>
  <a href="https://github.com/klappe-pm/lynter/releases/latest"><img src="https://img.shields.io/github/v/release/klappe-pm/lynter" alt="Latest release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/github/license/klappe-pm/lynter" alt="MIT license"></a>
  <a href="https://lynter.dev/playground/"><img src="https://img.shields.io/badge/playground-lynter.dev-2563AD" alt="Playground at lynter.dev"></a>
</p>

# Lynter

Lynter lints and fixes notes and code from one `lynter.yaml`. Markdown notes get consistent frontmatter, headings, lists, spacing and wording. Fenced code and source files get declarative checks, with optional adapters for ruff and Biome. The same Rust core runs in a command line tool, in an Obsidian plugin (compiled to WebAssembly), and in the playground on [lynter.dev](https://lynter.dev).

Rules are scoped. Profiles match files by path, extension, frontmatter, tag or age, and each profile overrides the defaults key by key. Note types describe the frontmatter a kind of note must carry. Loading is fail closed: an invalid `lynter.yaml` is reported in full and never applied in part.

## Install

### Obsidian plugin

Download `main.js`, `manifest.json` and `styles.css` from the [latest release](https://github.com/klappe-pm/lynter/releases/latest) into a folder named `lynter` in the vault's plugin directory (`.obsidian/plugins/lynter`), then enable Lynter under Settings, Community plugins.

To build it yourself, the plugin lives in `plugin/`. Build it, then copy `main.js`, `manifest.json` and `styles.css` into a folder named `lynter` in the vault's plugin directory and enable Lynter under Settings, Community plugins. You need Node.js 20 or later.

```bash
cd plugin && npm ci && npm run build
mkdir -p /path/to/vault/.obsidian/plugins/lynter
cp main.js manifest.json styles.css /path/to/vault/.obsidian/plugins/lynter/
```

### Command line

Each [release](https://github.com/klappe-pm/lynter/releases/latest) has a `lynter` binary for Linux and macOS (x86_64 and arm64) and Windows (x86_64), with a `SHA256SUMS` file. Unpack the archive for your platform and put `lynter` on your `PATH`.

To build from source, the CLI is the `lynter-cli` crate. You need a stable Rust toolchain.

```bash
cargo install --git https://github.com/klappe-pm/lynter.git lynter-cli
cd /path/to/notes && lynter init
cd /path/to/notes && lynter check .
cd /path/to/notes && lynter fix .
```

`lynter explain PATH` shows which profiles and rules govern a file. `lynter fix --stdin` filters text from stdin for editors and hooks.

## Configuration

`lynter init` writes a commented starter, and the [config builder](https://lynter.dev/configure/) builds the file from a form with every rule, option and section. Each release also carries the builder as `lynter-config-builder.html`, one file that runs offline. A small file looks like this:

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

The full key reference is [docs/lynter-yaml.md](docs/lynter-yaml.md). `lynter check` warns about unknown keys and misspelled rule ids in the config itself.

## Documentation

The docs at [lynter.dev/docs](https://lynter.dev/docs/) explain each part from the source:

- [Architecture](https://lynter.dev/docs/architecture/): the crates and how a document moves through them.
- [Scopes and profiles](https://lynter.dev/docs/scopes-and-profiles/) and [note types](https://lynter.dev/docs/note-types/).
- [Rules](https://lynter.dev/docs/rules-overview/): every rule with its options and a before and after example.
- [Code and data linting](https://lynter.dev/docs/code-linting/), the [command line](https://lynter.dev/docs/cli/), and the [Obsidian plugin](https://lynter.dev/docs/obsidian-plugin/).

The [playground](https://lynter.dev/playground/) runs the real engine in your browser.

## Roadmap

Lynter 0.1 ships the Rust core, the command line, the Obsidian plugin and the site. Next, in order:

- **Now:** the Obsidian community directory listing, `npm` and Homebrew installs for the command line, and the GitHub Action.
- **Next:** task formatting, tag rules, and code linting for Rust, Python, TypeScript, JavaScript, SQL, Go, Java, C#, JSON and YAML.
- **Later:** a VS Code extension, a JetBrains plugin, more languages, presets, and team checks on pull requests.

The full list is on [lynter.dev/roadmap](https://lynter.dev/roadmap/). To ask for a rule, an option, a language or an integration, [open a feature request](https://github.com/klappe-pm/lynter/issues/new?template=feature_request.md).

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

MIT. See [LICENSE](LICENSE). Third-party material and its licenses are listed in [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md).
