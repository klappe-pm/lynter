## Install

Lynter is not in the Obsidian community directory yet. Today you install the plugin from a clone of the repository. You need Git and Node.js 20 or later.

```bash
git clone https://github.com/klappe-pm/lynter.git
cd lynter/plugin && npm ci && npm run build
cd .. && cargo install --path crates/lynter-cli
```

Copy `plugin/main.js`, `plugin/manifest.json`, and `plugin/styles.css` into `.obsidian/plugins/lynter` in your vault, then enable the plugin under Settings, Community plugins. The command line installs as `{{CLI}}`. It reads the config as `lynter.yaml`; the older names `linter.yaml` and `lappe-linter.yaml` are still found, with a notice.

## Create the config

Run init from the root of your vault or repository. It writes a commented starter file that you edit and commit.

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

Run `check` on every pull request. It exits 1 when a file breaks a rule.

```yaml
name: lynter
on: [pull_request]
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with: {fetch-depth: 0}
      - run: cargo install --git https://github.com/klappe-pm/lynter.git lynter-cli
      - run: lynter check .
```

See [CI integrations](/integrations/ci/) for GitLab CI, lefthook, husky, and the `--json` output.
