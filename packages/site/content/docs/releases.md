---
title: Releases
lede: How Lynter is versioned, what the release workflow builds and publishes, how checksums work, and how third-party notices are produced.
---

A release is cut by pushing a version tag. The workflow builds the command line binary for five platforms, builds the Obsidian plugin, writes a checksum file, and publishes a GitHub release. Publishing to npm and crates.io stays a manual step, and the workspace marks its crates as unpublished.

## Versioning

The current version is 0.1.0. The same number appears in several places, and the release workflow refuses to run unless they agree.

| File | Field |
| --- | --- |
| `plugin/manifest.json` and the root `manifest.json` | `version`. The root file must be an exact copy of the plugin's. |
| `packaging/npm/package.json` | `version`. |
| `crates/lynter-cli/Cargo.toml` | `version`. |

`versions.json`, also mirrored at the root, maps each plugin version to the minimum Obsidian version it needs. A CI job compares the two copies of both files.

The tag is the bare version, such as `0.1.0`, with no `v` prefix. That is because the Obsidian community directory reads the release whose tag equals the version in `manifest.json`. The workflow triggers on tags shaped like `NUMBER.NUMBER.NUMBER`.

Sources: `.github/workflows/release.yml`, `.github/workflows/ci.yml`, `manifest.json`, `versions.json`, `Cargo.toml`

## The release workflow

`.github/workflows/release.yml` runs five kinds of job in a chain.

```flow
version -> binaries (five targets) -> release
version -> plugin -> release
```

1. **version** checks that the tag equals the version in both manifests, in the npm package and in the CLI crate.
2. **binaries** builds `cargo build --release --locked -p lynter-cli` on a native runner for each target, with `fail-fast` on. A failure in one target stops the release.
3. **plugin** installs plugin dependencies, builds `main.js`, runs the plugin tests and uploads the result.
4. **release** downloads every artifact, writes `SHA256SUMS`, and creates the GitHub release with generated notes.

Release notes are grouped by pull request label through `.github/release.yml`: Rules, Plugin, CLI and CI, Fixes, and Other changes. The WebAssembly build is not rebuilt during a release. The plugin embeds the committed build from `packages/site/public/wasm`.

Sources: `.github/workflows/release.yml`, `.github/release.yml`

## Artifacts

| Target | Runner | Archive |
| --- | --- | --- |
| `x86_64-unknown-linux-gnu` | `ubuntu-latest` | `.tar.gz` |
| `aarch64-unknown-linux-gnu` | `ubuntu-24.04-arm` | `.tar.gz` |
| `x86_64-apple-darwin` | `macos-15-intel` | `.tar.gz` |
| `aarch64-apple-darwin` | `macos-14` | `.tar.gz` |
| `x86_64-pc-windows-msvc` | `windows-latest` | `.zip` |

Each archive is named `lynter-<version>-<target>` and holds one folder with the same name. The folder contains the `lynter` binary (`lynter.exe` on Windows), `LICENSE`, `THIRD-PARTY-NOTICES.md`, and `THIRD-PARTY-LICENSES.txt`, which is `packaging/licenses/rust-crates.txt` renamed. The release also attaches the plugin files `main.js`, `manifest.json` and `styles.css`, and the checksum file.

Sources: `.github/workflows/release.yml`

## Checksums

The release job runs `sha256sum lynter-*` over the downloaded archives and publishes the output as `SHA256SUMS`. The file covers the five binary archives. It does not list the plugin files.

Consumers verify against it. The GitHub Action in `.github/actions/lynter` downloads the archive for the runner and `SHA256SUMS`, runs `sha256sum -c` on the matching line, and only then unpacks it. The npm package in `packaging/npm` has a postinstall script that downloads the archive for your platform, checks it against `SHA256SUMS`, and unpacks the binary. Set `LYNTER_SKIP_DOWNLOAD=1` to skip that download. To check a download by hand, compare its SHA-256 digest with the line for that file in `SHA256SUMS`.

Sources: `.github/workflows/release.yml`, `.github/actions/lynter/action.yml`, `packaging/npm/install.js`

## Packaging

`packaging/` holds the ways a release reaches users beyond the GitHub page.

- `packaging/npm` is an npm package named `lynter`. It ships `bin/lynter.js` and `install.js`, needs Node 18 or later, and fetches the matching release archive when installed.
- `packaging/homebrew/lynter.rb` is a Homebrew formula template. Replace `VERSION` and each `SHA256` placeholder with values from `SHA256SUMS`, then publish it to a tap as `Formula/lynter.rb`. The template does not publish itself.
- `packaging/licenses` holds the license tooling, described next.

Sources: `packaging/npm/package.json`, `packaging/homebrew/lynter.rb`, `packaging/npm/install.js`

## Third-party notices

`THIRD-PARTY-NOTICES.md` at the repository root lists the third-party material that Lynter includes or ships with, each under its own license. Its sections cover material used for compatible rule behavior, the Rust crates, the fonts the site serves, and the JavaScript libraries in the plugin bundle and on the site.

The Rust crate texts live in `packaging/licenses/rust-crates.txt`, generated by `packaging/licenses/generate.py` from `Cargo.lock`:

- The crate set is the normal, non-development, non-build dependency closure of `lynter-cli` and `lynter-wasm` on every platform, read from `cargo metadata`. Workspace members are excluded.
- License texts come from each crate's own published source. A crate published without a license file falls back to a copy under `packaging/licenses/extra/<crate>.txt`. Currently that covers `tree-sitter-java` and `tree-sitter-typescript`.
- Identical texts are printed once and referenced by a short hash. A crate with no license file and no fallback fails the run.

```bash
cd /path/to/lynter && python3 packaging/licenses/generate.py
cd /path/to/lynter && python3 packaging/licenses/generate.py --check
```

The first command rewrites the file. The second exits with status 1 when the file is stale, and CI runs it on every pull request, so a change to `Cargo.lock` needs a regenerated file. The file ships inside every release archive.

Sources: `THIRD-PARTY-NOTICES.md`, `packaging/licenses/generate.py`, `packaging/licenses/rust-crates.txt`, `.github/workflows/ci.yml`
