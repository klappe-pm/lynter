## Install

Lynter is free and [MIT licensed](https://github.com/klappe-pm/lynter/blob/master/LICENSE). Each [release](https://github.com/klappe-pm/lynter/releases/latest) carries the plugin files and a `{{CLI}}` binary for Linux, macOS and Windows, with a `SHA256SUMS` file.

### Obsidian plugin

Download `main.js`, `manifest.json` and `styles.css` from the [latest release](https://github.com/klappe-pm/lynter/releases/latest) into a folder named `lynter` inside `.obsidian/plugins` in your vault, then enable Lynter under Settings, Community plugins. Lynter is not in the community directory yet; the listing is on the [roadmap](/roadmap/).

### Command line

Download the archive for your platform, unpack it, and put `{{CLI}}` on your `PATH`.

| Platform | Archive |
|---|---|
| macOS, Apple silicon | `lynter-0.1.0-aarch64-apple-darwin.tar.gz` |
| macOS, Intel | `lynter-0.1.0-x86_64-apple-darwin.tar.gz` |
| Linux, x86_64 | `lynter-0.1.0-x86_64-unknown-linux-gnu.tar.gz` |
| Linux, arm64 | `lynter-0.1.0-aarch64-unknown-linux-gnu.tar.gz` |
| Windows, x86_64 | `lynter-0.1.0-x86_64-pc-windows-msvc.zip` |

```bash
curl -LO https://github.com/klappe-pm/lynter/releases/download/0.1.0/lynter-0.1.0-aarch64-apple-darwin.tar.gz
tar -xzf lynter-0.1.0-aarch64-apple-darwin.tar.gz
sudo mv lynter-0.1.0-aarch64-apple-darwin/lynter /usr/local/bin/
{{CLI}} --version
```

To build from source instead, you need a stable Rust toolchain: `cargo install --git https://github.com/klappe-pm/lynter.git lynter-cli`.

Lynter reads its config as `lynter.yaml`; the older names `linter.yaml` and `lappe-linter.yaml` are still found, with a notice.

## Create the config

Run init from the root of your vault or repository. It writes a commented starter file that you edit and commit. To build the file from a form instead, use the [config builder](/configure/).

```bash
cd /path/to/vault
{{CLI}} init
```

## Write a first scope

A scope is a profile with a `match` block. It starts from `defaults` and changes only what it states. This one turns contractions off inside a client folder.

```yaml
version: 1
defaults:
  rules:
    contractions:
      enabled: true
profiles:
  client:
    match:
      path: ["work/clients/**"]
    rules:
      contractions:
        enabled: false
```

Check which profiles govern a path with `explain`.

```bash
{{CLI}} explain work/clients/acme/brief.md
```

## Add a pre-commit hook

Check only the files git sees as staged or changed. A failing check blocks the commit.

```bash
{{CLI}} check --changed
```

With the pre-commit framework, add a local hook.

```yaml
repos:
  - repo: local
    hooks:
      - id: lynter
        name: lynter
        entry: {{CLI}} check --changed
        language: system
        pass_filenames: false
```

## Add a CI check

Run `check` on every pull request with the Lynter action, which downloads the release binary and verifies its checksum. It exits 1 when a file breaks a rule.

```yaml
name: lynter
on: [pull_request]
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with: {fetch-depth: 0}
      - uses: klappe-pm/lynter/.github/actions/lynter@0.1.0
```

See [CI integrations](/integrations/ci/) for GitLab CI, lefthook, husky, and the `--json` output.
