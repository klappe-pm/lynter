---
title: Command line
lede: Every lynter subcommand and flag, the text and JSON output formats, exit codes, stdin filtering, changed-file targeting, and how files are discovered and ignored.
---

The `lynter` binary has four subcommands: `check`, `fix`, `explain` and `init`. It lints one file at a time. For each target it finds the governing config, resolves the file's scope, runs the rules, and prints what it found. Markdown and text files run through the core. Files with an extension the code host knows run through [code linting](/docs/code-linting/).

```bash
cd /path/to/notes && lynter init
cd /path/to/notes && lynter check .
cd /path/to/notes && lynter fix .
cd /path/to/notes && lynter explain notes/a.md
```

## Subcommands

| Command | What it does |
| --- | --- |
| `lynter check [PATHS]` | Reports violations without changing any file. Exits 1 when it finds any. |
| `lynter fix [PATHS]` | Applies fixes in place and prints the path of each changed file. |
| `lynter fix --stdin [--stdin-path PATH]` | Filter mode. Reads text from standard input and writes the fixed text to standard output. |
| `lynter explain PATH` | Prints the governing config, profile chain, note type and rules for one file. |
| `lynter init` | Writes a commented starter `lynter.yaml` to the current directory. |
| `lynter rules [--json]` | Lists every rule with its group and description. With `--json` it prints an array that also carries each rule's default options; the [config builder](/configure/) is generated from it. |

`check` and `fix` need at least one path or `--changed`. Without either they exit with status 2. `lynter --version` and `lynter --help` work as usual.

Sources: `crates/lynter-cli/src/main.rs`, `crates/lynter-cli/src/catalog.rs`, `crates/lynter-cli/tests/cli.rs`

## Flags

These flags are global, so they work with every subcommand.

| Flag | Meaning |
| --- | --- |
| `--config PATH` | Use this config file for every target and skip discovery. |
| `--json` | Print one JSON line per file instead of text. See the shape below. |
| `--changed` | Also target files that git reports as changed. |
| `--today yyyy-MM-dd` | The date used for age matching and for the date rules. Defaults to today in UTC. The value must have the shape `yyyy-MM-dd`, or the command exits with status 2. |
| `--no-adapters` | Skip the ruff and Biome adapters even when the config enables them. |

`fix` also takes `--stdin` and `--stdin-path PATH`. The path is relative to the config directory and defaults to `stdin.md`. It is what scope matchers and path-based rules see, so a filter in an editor can pass the real file name.

Sources: `crates/lynter-cli/src/main.rs`

## File discovery

Each path argument is a file or a directory. A directory is walked recursively in file name order, honoring `.gitignore` and the other standard ignore files, skipping hidden files and any `node_modules` folder. A file named explicitly is used even if an ignore file would hide it. A path that does not exist exits with status 2. Duplicates are removed.

Only some files are lintable: `.md` and `.txt`, plus the extensions the code host handles: `json`, `yaml`, `yml`, `rs`, `py`, `pyi`, `ts`, `mts`, `cts`, `tsx`, `js`, `mjs`, `cjs`, `jsx`, `sql`, `go`, `java` and `cs`. Other files named on the command line are skipped silently. A code or data file with no enabled rule for it is skipped too, and does not appear in `--json` output unless an adapter ran.

Sources: `crates/lynter-cli/src/files.rs`, `crates/lynter-code/src/runner.rs`, `crates/lynter-code/src/language.rs`

## Ignore handling

The config's `ignore` key is applied to every target after its config is found. `ignore.folders` entries ignore a folder and everything under it, and `ignore.files` entries ignore one file. Both are compared with the file's path relative to the config directory, using forward slashes. They are exact paths, not globs, and a trailing slash on a folder is allowed.

```yaml
ignore:
  folders: [templates, .obsidian]
  files: [inbox/scratch.md]
```

Ignored files are skipped without output. `explain` prints `ignored: true` for them, and `fix --stdin` echoes the input back unchanged.

Sources: `crates/lynter-cli/src/files.rs`, `crates/lynter-cli/src/main.rs`

## Changed files

`--changed` finds the git repository root from the current directory, then collects the files listed by `git diff --name-only HEAD` and `git diff --name-only --cached`. It keeps those that still exist and are lintable, and adds them to any paths you gave. Untracked files are not included. Outside a git repository the command exits with status 2.

```bash
cd /path/to/notes && lynter check --changed
```

Sources: `crates/lynter-cli/src/files.rs`, `crates/lynter-cli/tests/cli.rs`

## Standard input

`lynter fix --stdin` reads all of standard input, lints it as if it were the file named by `--stdin-path`, and writes the result to standard output. It discovers the config from the current directory. It takes no paths and no `--changed`, or it exits with status 2.

```bash
cd /path/to/notes && lynter fix --stdin --stdin-path notes/a.md < notes/a.md
```

The filter is safe to wire into an editor. If the config is missing or invalid it prints the errors to standard error, echoes the input unchanged and exits with status 2. If the file is ignored, or no rule applies, it echoes the input and exits 0.

Sources: `crates/lynter-cli/src/main.rs`, `crates/lynter-cli/tests/cli.rs`

## Output formats

### Text

`check` prints one line per violation as `PATH: RULE: MESSAGE`. For Markdown, the message is the rule description when the rule would rewrite text, or the rule's own finding for report-only rules. Adapter findings print as `PATH:LINE:COLUMN: RULE: MESSAGE`, and an adapter crash prints `PATH: adapter-error: MESSAGE`. `fix` prints only the path of each file it changed. Notices and warnings go to standard error.

### JSON

`--json` prints one JSON object per line, one line per linted file, in output version 1.

```json
{"path":"notes/a.md","profile":"drafts","violations":[{"rule":"strip-strong","line":null,"message":"Remove ** and __ strong emphasis, keeping the inner text; never edits inside code, math, YAML, tables, wikilinks, or URLs.","fixed":true}],"renamed_to":null,"relocated_to":null,"output-version":1}
```

| Field | Meaning |
| --- | --- |
| `path` | The path relative to the config directory. |
| `profile` | The last entry of the profile chain, the winning profile, or `defaults`. |
| `violations` | A list of entries, shaped as described below. |
| `renamed_to`, `relocated_to` | Always `null` in version 1. |
| `output-version` | Always `1`. |

Entries for Markdown and text files carry `rule`, `line`, `message` and `fixed`. There is one entry per rule that fired. The `line` is `null`, the `message` is the rule description, and `fixed` is `true`, in both `check` and `fix`. Entries for JSON, YAML and code files carry the real 1-based `line`, the rule message, and `fixed` as reported. Adapter entries add `column` and `fixable`, with `fixed` false. An adapter crash adds a single entry with the rule `adapter-error`.

Sources: `crates/lynter-cli/src/main.rs`, `crates/lynter-cli/tests/golden/check.stdout`

## Exit codes

| Code | Meaning |
| --- | --- |
| 0 | Success. `check` found nothing, or the command was `fix`, `explain` or `init`. |
| 1 | `check` found at least one violation, counting adapter findings. |
| 2 | Usage error, a missing or invalid config, an unreadable file or path, or an adapter that crashed. |

A config error stops the run at the first file that needs it. Output printed before that point stays printed. A missing adapter binary is only a notice on standard error, once per adapter, and does not change the exit status.

Sources: `crates/lynter-cli/src/main.rs`, `crates/lynter-cli/src/adapt.rs`, `crates/lynter-cli/tests/cli.rs`
