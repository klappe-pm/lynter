Code linting runs from the same `lynter.yaml` as note formatting. It is in progress. The matrix lists the first release.

## First release

| Language | Status |
|---|---|
| Rust | In progress |
| Python | In progress |
| TypeScript | In progress |
| JavaScript | In progress |
| SQL | In progress |
| Go | In progress |
| Java | In progress |
| C# | In progress |
| JSON | In progress |
| YAML | In progress |
| Plain text | In progress |

Markdown notes, frontmatter, and file names are covered today by the note rules in the current core.

## Coming soon

C, C++, PHP, Kotlin, Swift, Ruby, Bash, HTML and CSS, Dart, Lua, TOML, XML, and CSV are Coming soon.

## Scope by language

A scope can match a language, so one file can hold rules for notes and for code.

```yaml
profiles:
  python:
    match:
      language: python
```
