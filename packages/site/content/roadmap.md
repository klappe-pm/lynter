Lynter 0.1 ships the Rust core, the command line, the Obsidian plugin and this site. This page lists what comes after it, in order. It carries no dates: an item moves up when it is ready.

## Now

- **Obsidian community directory listing.** Install and update Lynter from inside Obsidian.
- **One-line installs for the command line.** `npm install -g lynter` and a Homebrew formula, both using the release binaries.
- **GitHub Action.** Run `lynter check` on a pull request without a Rust toolchain.

## Next

- **Task formatting.** Consistent checkbox, date and priority style for tasks.
- **Tag rules.** Normalize tag case and separators, and limit a vault to an allowed list.
- **Code linting for the first languages.** Rust, Python, TypeScript, JavaScript, SQL, Go, Java, C#, JSON and YAML, from the same `lynter.yaml`. See [languages](/languages/).

## Later

- **VS Code extension.** Fix on save and inline diagnostics from the same core.
- **JetBrains plugin.** The same for IntelliJ-based editors.
- **More languages.** C, C++, PHP, Kotlin, Swift, Ruby, Bash, HTML and CSS, Dart, Lua, TOML, XML and CSV.
- **Presets.** Ready-made style packs for common vault and repository layouts.
- **Team checks.** A GitHub check that comments on a pull request with what Lynter would change.

## Notes and code, together

Lynter exists because notes and code live in the same repositories and depend on each other. A decision note explains a module, a spec names the files it governs, and an instruction file shapes what an agent writes next. Today Lynter gives both one config and one result. What comes after is the links between them.

- **Cross references.** Check that a note's links to files and headings still resolve, and report the notes a code change leaves stale.
- **Intent documents.** Build on note types so a product document keeps the sections, ids and frontmatter that other tools read.

This is the groundwork for [Product Intent Runtime](https://github.com/klappe-pm/product-intent-runtime), a companion project that treats product intent as tracked files beside the code: compiled, locked, delivered to every working session, and checked against every change. Lynter keeps those files well formed, and the runtime checks each change against what they say. The runtime is an early proof of concept, and its repository may not be public yet.

## Request a feature

Missing a rule, an option, a language or an integration? [Open a feature request](https://github.com/klappe-pm/lynter/issues/new?template=feature_request.md) on GitHub. A before and after example is the most useful thing to include. To see what others have asked for, browse the [open requests](https://github.com/klappe-pm/lynter/issues?q=is%3Aissue+is%3Aopen+label%3Aenhancement).
