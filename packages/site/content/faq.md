## Does Lynter call a model?

No. Every rule is deterministic code. The same input gives the same output, with no tokens and no network.

## Is it in the Obsidian community directory?

Not yet. Download the plugin files from the [latest release](https://github.com/klappe-pm/lynter/releases/latest); [getting started](/getting-started/#install) has the steps. The listing is on the [roadmap](/roadmap/).

## Can I bring my existing Obsidian Linter settings?

Yes. [Import settings](/import/) converts a `data.json` into a `lynter.yaml` in your browser. [Importing settings](/docs/importing-settings/) explains what is converted.

## Which languages does it lint?

Markdown everywhere, including its YAML frontmatter. The command line also lints plain text, Rust, Python, TypeScript, JavaScript, Go, Java, C#, JSON, and YAML files. See [languages](/languages/).

## Does it work without Obsidian?

Yes. The command line runs on any folder or repository, with Markdown, plain text, source and data files, and the same `lynter.yaml` drives [CI and pre-commit](/integrations/ci/) and [agent hooks](/integrations/llm/). Obsidian is one place to run it.

## Does it change my files without asking?

`check` reports and never writes. `fix` rewrites in place, so commit first or use `--dry-run` to preview.

## Is there a hosted version?

No. Lynter runs on your machine and in your CI. Team features are Coming soon.

## Can it convert YAML to JSON, TOML or Markdown?

No. Lynter lints and fixes files in the format they are already in; it does not translate data from one format to another. Convert with a tool such as `yq` or CUE, then run `lynter fix` on the output so it matches your rules. TOML files are not linted yet; TOML support is on the [roadmap](/roadmap/).

## What does fix change in JSON and YAML files?

Order and layout, not values. In JSON and YAML files only `json-key-order` and `yaml-key-order` rewrite anything; syntax, duplicate key and schema rules report findings. In Markdown frontmatter, `yaml-key-sort` orders keys and can add missing keys from its `defaults`, and `alphabetize-property-values` sorts list values. See [code and data linting](/docs/code-linting/) and [note types](/docs/note-types/).

## What does it cost?

Nothing. Lynter is free and MIT licensed: the CLI, the Obsidian plugin and the core. See [LICENSE](https://github.com/klappe-pm/lynter/blob/master/LICENSE) and [third-party notices](https://github.com/klappe-pm/lynter/blob/master/THIRD-PARTY-NOTICES.md).

## What is the Rust core?

Every rule lives in one Rust core. The command line is a native binary built from it, and the Obsidian plugin and this site run the same core compiled to WebAssembly, so a note gets the same result everywhere. See [architecture](/docs/architecture/).
