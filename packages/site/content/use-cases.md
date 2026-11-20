## Personal vault

Years of notes drift. Lynter keeps frontmatter keys in one order, sorts tag lists, and sets heading case, so a note from last week looks like a note from five years ago. Start from the [starter-vault](/samples/) config and run `fix` on the whole vault once.

## Shared team vault

When a team shares a vault, style arguments belong in a pull request. Commit `lynter.yaml` with the vault and require `check` in CI. A change to a rule becomes a reviewed diff. A note type such as `policy` can require keys and limit `status` to a list of allowed values.

## Client formal tone

Some folders need a formal voice. A scope on `work/clients/**` turns contractions off and leaves the rest of the vault casual. Deeper scopes override broader ones, so one client can differ from another.

## Docs as code

Keep documentation beside the code and check it in the same CI run. Lynter keeps the first heading equal to the file name, flags names that are not kebab-case, and reports bullet lists that read better as prose. Code linting from the same file is Coming soon.

## Agent memory and AGENTS.md

Agents write `AGENTS.md` files, skills, memory notes, and docs. Pipe their output through `fix --stdin` in a hook so each file lands in house style before anyone reads it. A `memory/**` scope can also join hard-wrapped lines. See [LLM integrations](/integrations/llm/).

## Zettelkasten

Atomic notes depend on uniform structure. Use a note type to require the same frontmatter keys on every note, sort tags, and keep titles equal to file names. Scope by tag when permanent notes and literature notes need different rules.
