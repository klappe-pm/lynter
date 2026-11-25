Code linting runs from the same `lynter.yaml` as note formatting. Markdown is linted everywhere Lynter runs. Plain text files, source files and data files are linted by the command line, so they are covered in hooks and CI; the Obsidian plugin and the playground lint Markdown and the code inside its fences.

## In the 0.1 release

| Language | Where it runs | What is checked |
|---|---|---|
| Markdown | Everywhere | Every note rule, frontmatter rules such as `yaml-key-sort` and `alphabetize-property-values`, and code checks inside fenced blocks. |
| Plain text | Command line | The same note rules as Markdown. |
| Rust | Command line | Identifier case, syntax errors, comment tags, commented-out code, nesting depth, function length. |
| Python | Command line | The same code rules, plus the optional ruff adapter. |
| TypeScript and TSX | Command line | The same code rules, plus the optional Biome adapter. |
| JavaScript | Command line | The same code rules, plus the optional Biome adapter. |
| Go | Command line | The same code rules. |
| Java | Command line | The same code rules. |
| C# | Command line | The same code rules. |
| JSON | Command line | Syntax, duplicate keys, and key order (`json-key-order`, which `fix` rewrites). |
| YAML | Command line | Syntax, duplicate keys, schema, and key order (`yaml-key-order`, which `fix` rewrites). |

In data files, `fix` changes key order and nothing else; it never edits values or converts one format to another. SQL has a grammar in the source but is not part of the release build. [Code and data linting](/docs/code-linting/) describes each rule and its options.

## Coming soon

C, C++, PHP, Kotlin, Swift, Ruby, Bash, HTML and CSS, Dart, Lua, TOML, XML, and CSV are Coming soon, along with source file linting in the playground and the plugin.

## Limit a rule to some languages

A code rule can carry a `languages` list, so one file can hold different settings per language. A profile can also match by file extension.

```yaml
version: 1
defaults:
  rules:
    max-function-lines:
      enabled: true
      languages: [python, typescript]
profiles:
  scripts:
    match:
      extension: [py]
    rules:
      identifier-case:
        enabled: true
```
