---
title: Content rules
lede: The rules that rewrite wording and inline text, such as contractions, heading case, dashes, quotes, emphasis, URLs and hard-wrapped prose.
---

Content rules change what the text says or how its inline marks are written. They never edit inside code, math, tables, frontmatter, wikilinks or URLs (see protected regions in [Rules overview](/docs/rules-overview/)). The rules that reshape paragraphs and lists come in pairs: a report-only rule that flags and a `-fix` twin that rewrites. Enable a rule with `enabled: true` under `defaults.rules` or in a profile ([Scopes and profiles](/docs/scopes-and-profiles/)).

## Wording

Rules that change words, headings and dashes. `replace-em-dash` is on in the compiled defaults; the other four are present but off, because they change wording.

### `strip-strong`

Removes `**` and `__` strong emphasis and keeps the inner text. Fixes. Off by default.

Only the two delimiters are deleted. A `__` pair that touches a word character, a span that crosses a blank line, and a delimiter that would expose a fence, table row, math fence or heading marker at the start of a line are skipped. `***key***` becomes `*key*`.

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `keep-heading-strong` | boolean | `false` | Leave strong emphasis on heading lines. |

```markdown
This is **very** important and __also__ this.
```

```markdown
This is very important and also this.
```

### `replace-em-dash`

Replaces em dashes and connector en dashes with a separator. Fixes. On by default.

Spaces around the dash are absorbed into the replacement. An en dash between two digits (a numeric range) is kept. A dash at the start of a line or directly after a heading marker is left alone. A dash at the end of a line gets the replacement without trailing spaces.

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `replacement` | string | `", "` | Text that replaces each dash run. |
| `word-joining` | boolean | `false` | When true, a dash with no spaces between two letters or digits becomes a hyphen. |

```text
one thing — another—and a third
pages 10–12 cover setup – the rest is reference
```

```text
one thing, another, and a third
pages 10–12 cover setup, the rest is reference
```

### `contractions`

Replaces expanded English phrases with contractions. Fixes. Off by default.

There are 107 phrases, such as `do not`, `it is`, `we have` and `could have`. Matching ignores case and keeps it: `It is` becomes `It's` and `IT IS` becomes `IT'S`. Phrases inside protected regions, and phrases that touch a letter, digit or apostrophe on either side, are skipped.

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| each phrase, for example `it is` | boolean | `true` | Set to `false` to leave that phrase expanded. |

```markdown
It is ready. We are ready.
```

With `it is: false`:

```markdown
It is ready. We're ready.
```

### `header-case`

Normalizes ATX heading text to a case style, per heading level. Fixes. Off by default.

A level with no style is left alone, so the rule does nothing until at least one of `h1` to `h6` is set. Words are split on spaces, hyphens, underscores and camel case boundaries before the style is applied. Headings inside code, math, tables and frontmatter are never edited.

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `h1` to `h6` | string | empty | One of `camelCase`, `First letter`, `kebab-case`, `Title Case` or `underscore_formatted`. Empty leaves the level unchanged. |

```markdown
# My Great Note

## Some Section Here
```

With `h1: kebab-case` and `h2: camelCase`:

```markdown
# my-great-note

## someSectionHere
```

### `remove-trailing-punctuation-in-heading`

Strips trailing punctuation from headings, keeping HTML entities. Fixes. Off by default.

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `trailing-punctuation` | string | `.,;:!` plus the full-width forms of `. , ; : !` | The characters to strip. An empty string disables the rule. |

```markdown
# Heading ends in a period.
## Other heading ends in an exclamation mark! ##
```

```markdown
# Heading ends in a period
## Other heading ends in an exclamation mark ##
```

Sources: `crates/lynter-core/src/rules/strip_strong.rs`, `crates/lynter-core/src/rules/replace_em_dash.rs`, `crates/lynter-core/src/rules/contractions.rs`, `crates/lynter-core/src/rules/contractions_table.rs`, `crates/lynter-core/src/rules/header_case.rs`, `crates/lynter-core/src/rules/compat/headings.rs`, `crates/lynter-core/tests/fixtures/rules-strip-strong.json`, `crates/lynter-core/tests/fixtures/rules-replace-em-dash.json`, `crates/lynter-core/tests/fixtures/rules-contractions.json`, `crates/lynter-core/tests/fixtures/rules-header-case.json`

## Inline text

Rules for emphasis delimiters, quotes, ellipses, bare URLs and fence languages. All five are off by default.

### `emphasis-style`

Makes emphasis delimiters consistently asterisks or underscores. Fixes. Off by default.

An underscore is not used where it would touch a letter or digit, since that would stop it being emphasis. Strong text (`**`) is handled by `strong-style`.

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `style` | string | `consistent` | `asterisk`, `underscore`, or `consistent`, which follows the first emphasis in the note. |

```markdown
*Test emphasis*
This is *emphasized* mid sentence
```

With `style: underscore`:

```markdown
_Test emphasis_
This is _emphasized_ mid sentence
```

### `strong-style`

Makes strong delimiters consistently double asterisks or double underscores. Fixes. Off by default.

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `style` | string | `consistent` | `asterisk`, `underscore`, or `consistent`, which follows the first strong span in the note. |

```markdown
**Test bold**
This is **bold** mid sentence
```

With `style: underscore`:

```markdown
__Test bold__
This is __bold__ mid sentence
```

### `quote-style`

Makes quotes in body text all straight or all smart. Fixes. Off by default.

Straight style turns curly and guillemet quotes into `"` and `'`. Smart style pairs straight quotes into opening and closing curly quotes and turns apostrophes inside words into `’`. If the straight double quotes in a note do not pair up, they are reported and left unchanged. Templater commands (`<% ... %>`) and protected regions are skipped.

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `single-quote-enabled` | boolean | `true` | Process single quotes. |
| `single-quote-style` | string | `straight` | `straight` or `smart`. |
| `double-quote-enabled` | boolean | `true` | Process double quotes. |
| `double-quote-style` | string | `straight` | `straight` or `smart`. |

```markdown
"As you can see"
Pam's dog and can't stop
```

With both styles set to `smart`:

```markdown
“As you can see”
Pam’s dog and can’t stop
```

### `proper-ellipsis`

Replaces three consecutive periods with the ellipsis character. Fixes. Off by default. No options.

```markdown
Wait...
```

```markdown
Wait…
```

### `no-bare-urls`

Wraps bare URLs in angle brackets so they stay links. Fixes. Off by default. No options.

A URL already in angle brackets, a link target, a URL inside `[ ]` and a URL inside matching quotes are left alone. Doubled angle brackets are collapsed to one pair.

```markdown
Mid-sentence like https://google.com will be.
```

```markdown
Mid-sentence like <https://google.com> will be.
```

### `default-language-for-code-fences`

Adds a language to opening code fences that have none. Fixes. Off by default.

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `language` | string | empty | The language to add. Empty, or a value with whitespace, disables the rule. |

````markdown
```
var temp = 'text';
```
````

With `language: javascript`:

````markdown
```javascript
var temp = 'text';
```
````

Sources: `crates/lynter-core/src/rules/compat/emphasis.rs`, `crates/lynter-core/src/rules/compat/text.rs`, `crates/lynter-core/tests/fixtures/compat/emphasis-style.json`, `crates/lynter-core/tests/fixtures/compat/strong-style.json`, `crates/lynter-core/tests/fixtures/compat/quote-style.json`, `crates/lynter-core/tests/fixtures/compat/proper-ellipsis.json`, `crates/lynter-core/tests/fixtures/compat/no-bare-urls.json`, `crates/lynter-core/tests/fixtures/compat/default-language-for-code-fences.json`

## Prose reshaping

Two reshaping rules, each as a report-only rule and a `-fix` twin that shares its detection and applies the rewrite. The report-only rules are on in the compiled defaults, so a new user sees a finding without losing any line breaks. The `-fix` twins are off by default and are the opt-in way to rewrite. None of the four takes options.

### `join-paragraph-lines`

Flags hard-wrapped prose lines inside a paragraph that could be one line. Reports only. On by default.

### `join-paragraph-lines-fix`

Joins hard-wrapped prose lines inside a paragraph into one line, separated by a single space. Fixes. Off by default.

Headings, blockquotes, lists, tables, setext underlines, horizontal rules, fences, math, reference definitions, lines that are only a wikilink, lines starting with `<`, and indented lines are never joined. A line that ends with two or more spaces, a backslash or a `<br>` is an intentional break and keeps its line break.

```markdown
wrapped
at a column
by an
old editor.
```

```markdown
wrapped at a column by an old editor.
```

### `prose-list-to-sentences`

Flags simple one-level bullet lists that could be written as a sentence. Reports only. On by default.

A list qualifies when it has at least two items with the same marker, no checkboxes and no nested lists, and a blank line or the end of the file follows it.

### `prose-list-to-sentences-fix`

Rewrites a qualifying bullet list as comma-joined prose. Fixes. Off by default.

Two items are joined with `and`, three or more with commas and a final `, and`. Trailing periods, commas and semicolons on items are dropped and a final period is added unless the last item already ends a sentence. When the line before the list is joinable prose, the sentence continues that line.

```markdown
The kit includes:
- a tent
- a stove
- two mats
```

```markdown
The kit includes: a tent, a stove, and two mats.
```

Sources: `crates/lynter-core/src/rules/content/join_paragraph_lines.rs`, `crates/lynter-core/src/rules/content/prose_list.rs`, `crates/lynter-core/src/rules/content/util.rs`, `crates/lynter-core/tests/fixtures/rules-join-paragraph-lines-fix.json`, `crates/lynter-core/tests/fixtures/rules-prose-list-to-sentences-fix.json`
