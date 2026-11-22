## Does Lynter call a model?

No. Every rule is deterministic code. The same input gives the same output, with no tokens and no network.

## Is it in the Obsidian community directory?

Not yet. You can install the plugin from a clone of the repository. The listing is on the [roadmap](/roadmap/).

## Can I bring my existing Obsidian Linter settings?

Yes. [Import settings](/import/) converts a `data.json` into a `lynter.yaml` in your browser. [Importing settings](/docs/importing-settings/) explains what is converted.

## Which languages does it lint?

The first release targets Rust, Python, TypeScript, JavaScript, SQL, Go, Java, C#, JSON, YAML, and text. Code linting is in progress. See [languages](/languages/).

## Does it work without Obsidian?

Yes. The command line runs on any folder of Markdown files. Obsidian is one place to run it.

## Does it change my files without asking?

`check` reports and never writes. `fix` rewrites in place, so commit first or use `--dry-run` to preview.

## Is there a hosted version?

No. Lynter runs on your machine and in your CI. Team features are Coming soon.

## What does it cost?

The repository is MIT licensed.

## What is the Rust core?

A rewrite of the rule core in Rust is being built so the plugin, CLI, CI, and hooks share one fast binary. Until it lands, the TypeScript core powers everything on this site.
