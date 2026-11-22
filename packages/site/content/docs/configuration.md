---
title: Configuration
lede: How lynter.yaml is discovered, parsed, validated and defaulted, why loading fails closed, and how the starter file is produced.
---

`lynter.yaml` is the only configuration Lynter reads. The core turns its text into a validated `Config` or a list of errors, never something in between. This page covers that path. For every key and its meaning, see the reference at [/docs/reference/](/docs/reference/).

## File names and discovery

Three names are accepted, tried in this order in a single directory. The first one found wins.

| Name | Status |
| --- | --- |
| `lynter.yaml` | Canonical. |
| `linter.yaml` | Deprecated alias. |
| `lappe-linter.yaml` | Deprecated alias. |

A deprecated name works, and Lynter prints a notice asking you to rename it to `lynter.yaml`. The CLI prints that notice once per config file to standard error. The Obsidian plugin shows it once per session.

Each host decides where to look:

- The CLI looks in the directory of each target file, then walks up through its parents until it finds a config. `--config PATH` skips discovery and uses that file for every target. If nothing is found, the command exits with status 2 and an error that names all three files.
- The plugin looks only at the vault root, in the same name order. It does not walk up.
- The playground and the `lint` WebAssembly export take the config text directly.

Paths passed to rules and scope matchers are relative to the directory that holds the config, with forward slashes.

Sources: `crates/lynter-core/src/config/discovery.rs`, `crates/lynter-cli/src/cfg.rs`, `plugin/src/config.ts`, `docs/lynter-yaml.md`

## Parsing

`parse_config` converts the YAML text to a JSON value tree with `serde_yaml_ng`, then validates the tree. Mapping keys that are numbers or booleans become strings. A YAML syntax error is reported as a single error whose message starts with `yaml parse error`. An empty or whitespace-only file parses to null, which fails validation with `config must be a yaml mapping`.

The validated `Config` keeps the full document as `raw`, plus typed views of the parts the scope engine reads: `defaults.rules`, `rule-order`, the profiles and the note types.

Sources: `crates/lynter-core/src/config/mod.rs`, `crates/lynter-core/src/config/types.rs`

## Fail-closed loading

Validation collects every problem, then decides. If there is at least one error, the result is `Err` with all of them, and the host lints nothing. Each error carries a key path such as `profiles.journal.match.path` and a message. There is no partial config.

```text
lynter: /vault/lynter.yaml: version: must be 1
lynter: /vault/lynter.yaml: profiles.journal.match.date-created.after: must be an ISO yyyy-MM-dd string
```

The CLI prints each error on its own line to standard error and exits with status 2. In `fix --stdin` mode it also echoes the input unchanged to standard output, so an editor filter does not erase a buffer when the config is broken.

Sources: `crates/lynter-core/src/config/validate.rs`, `crates/lynter-cli/src/cfg.rs`, `crates/lynter-cli/src/main.rs`

## Errors and warnings

The line between the two is whether the key is structurally wrong or merely unknown.

| Condition | Result |
| --- | --- |
| The document is not a mapping, or `version` is missing or not `1`. | Error. |
| A value has the wrong type, such as a rule config that is not a mapping or `enabled` that is not a boolean. | Error. |
| A date bound in `date-created` or `date-revised` is not a real `yyyy-MM-dd` date. | Error. |
| A scoped note type requires a key the `base` note type does not require. | Error that names the key. |
| `rename.mode` is missing or not `off`, `flag` or `rename`. | Error. |
| An automation has no unique name, a bad `trigger`, or a `schedule` trigger without a cron string. | Error. |
| A scoped template has a missing or duplicate `name`. | Error. |
| An unknown key at the top level, in `defaults`, in a profile, a match block, a note type, `templates`, `automations`, `rename` or `ignore`. | Warning that names the key, then the key is ignored. |
| A malformed entry under `code-checks`. | Warning, and the entry is skipped. |

In the CLI, warnings print to standard error as `lynter: PATH: warning: unknown key "foo" ignored` and do not change the exit status.

A rule id that no rule answers to is also a warning. The loader accepts any key under `rules`, so a misspelled id would otherwise be ignored without a word. The CLI and the playground report it with its key path and the closest known id, for example `defaults.rules.contraction: unknown rule "contraction" is ignored: did you mean "contractions"?`.

The CLI adds one more validation step. The core does not know the `adapters` key, so the CLI drops the core's warnings about it and validates adapter blocks itself. An unknown adapter id or an unknown adapter key is an error. See [code linting](/docs/code-linting/).

Sources: `crates/lynter-core/src/config/validate.rs`, `crates/lynter-cli/src/cfg.rs`, `crates/lynter-code/src/adapters/config.rs`

## Defaults

Lynter does not merge your file with built-in rule settings. A rule that is not listed under `defaults.rules`, or is listed without `enabled: true`, does not run. The starter file turns on a sensible baseline and leaves five rules off because they rewrite wording or names: `strip-strong`, `header-case`, `contractions`, `h1-matches-stem` and `kebab-case-filename`.

The core also has `default_config()`, a compiled config with that same baseline, a `base` note type, `rename.mode: flag` and empty ignore lists. It is a library function. The CLI, the plugin and the WebAssembly exports do not apply it, and the CLI fails with status 2 when it finds no config file.

Sources: `crates/lynter-core/src/config/defaults.rs`, `crates/lynter-cli/src/cfg.rs`, `docs/lynter-yaml.md`

## The starter file

`config::scaffold()` returns a commented `lynter.yaml` that parses without errors or warnings. Every default carries a comment saying what it does and how to change it. The plugin command "Create lynter.yaml" and the WebAssembly `scaffold` export use it. The plugin validates the text before writing and never overwrites an existing config under any of the three names.

`lynter init` writes the same text to `lynter.yaml` in the current directory, prints `wrote lynter.yaml`, and refuses to overwrite an existing file with status 2. A note fixed under the starter then passes the check.

```bash
cd /path/to/notes && lynter init
cd /path/to/notes && lynter explain notes/example.md
```

Sources: `crates/lynter-core/src/config/scaffold.rs`, `crates/lynter-cli/src/main.rs`, `plugin/src/lynter.ts`, `crates/lynter-wasm/src/lib.rs`
