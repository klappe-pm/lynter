---
title: Spacing and structure rules
lede: The rules for blank lines, spaces, list markers and numbering, heading levels and the end of the document.
---

These rules change layout rather than wording: how many blank lines separate blocks, where spaces sit, which marker a list uses and how headings nest. They skip frontmatter, code, math and tables except where a rule exists to put blank lines around them. Enable a rule with `enabled: true` under `defaults.rules` or in a profile; see [Rules overview](/docs/rules-overview/) for how rules run and [Scopes and profiles](/docs/scopes-and-profiles/) for per-folder overrides. Every rule on this page fixes, and only `paragraph-spacing` and `list-style` are on in the compiled defaults.

## Blank lines

Nine rules control blank lines. The `empty-line-around-*` rules add a blank line where one is missing, inside blockquotes too (the added line carries the `>`).

### `consecutive-blank-lines`

Collapses runs of blank lines outside code, math and frontmatter to a single blank line. Fixes. Off by default. No options.

Lines that hold only spaces or tabs count as blank.

```markdown
Some text


Some more text
```

```markdown
Some text

Some more text
```

### `empty-line-around-blockquotes`

Ensures a blank line before and after blockquotes and callouts. Fixes. Off by default. No options.

```markdown
# Heading 1
> Quote content here
> quote content continued
```

```markdown
# Heading 1

> Quote content here
> quote content continued
```

### `empty-line-around-code-fences`

Ensures a blank line before and after fenced code blocks. Fixes. Off by default. No options.

````markdown
# Heading 1
```
Here is a code block
```
````

````markdown
# Heading 1

```
Here is a code block
```
````

### `empty-line-around-horizontal-rules`

Ensures exactly one blank line before and after horizontal rules. Fixes. Off by default. No options.

A `---` line directly under a paragraph is a setext heading underline and is not treated as a rule.

```markdown
***
Content
***
```

```markdown
***

Content

***
```

### `empty-line-around-math-block`

Ensures a blank line before and after display math blocks. Fixes. Off by default.

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `number-of-dollar-signs` | integer | `2` | How many `$` open and close a block. Values are limited to 1 through 16. |

```markdown
$$
\boldsymbol{a}=1
$$
some more text
```

```markdown
$$
\boldsymbol{a}=1
$$

some more text
```

### `empty-line-around-tables`

Ensures a blank line before and after tables. Fixes. Off by default. No options.

```markdown
Intro
| a | b |
|---|---|
| 1 | 2 |
Outro
```

```markdown
Intro

| a | b |
|---|---|
| 1 | 2 |

Outro
```

### `heading-blank-lines`

Ensures one blank line before and after every heading, and collapses longer runs to one. Fixes. Off by default.

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `bottom` | boolean | `true` | Also enforce the blank line after each heading. |
| `empty-line-between-yaml-and-header` | boolean | `true` | Keep one blank line between the frontmatter and a heading that follows it. When false, that blank line is removed. |

```markdown
# H1
## H2


# H1
line
```

```markdown
# H1

## H2

# H1

line
```

### `line-break-at-document-end`

Ends the document with exactly one line break and no trailing blank lines. Fixes. Off by default. No options.

```markdown
Lorem ipsum dolor sit amet.



```

```markdown
Lorem ipsum dolor sit amet.
```

### `paragraph-spacing`

Normalizes the blank lines between blocks to a fixed count and trims blank lines at the start and end. Fixes. On by default.

Blank lines inside code, math, tables and frontmatter are preserved. A final newline that is already present is kept.

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `blank-lines` | integer | `1` | Blank lines between blocks. Values are limited to 0, 1 or 2. |

```markdown
one



two
```

```markdown
one

two
```

Sources: `crates/lynter-core/src/rules/compat/spacing.rs`, `crates/lynter-core/src/rules/content/paragraph_spacing.rs`, `crates/lynter-core/tests/fixtures/compat/consecutive-blank-lines.json`, `crates/lynter-core/tests/fixtures/compat/empty-line-around-blockquotes.json`, `crates/lynter-core/tests/fixtures/compat/empty-line-around-code-fences.json`, `crates/lynter-core/tests/fixtures/compat/empty-line-around-horizontal-rules.json`, `crates/lynter-core/tests/fixtures/compat/empty-line-around-math-block.json`, `crates/lynter-core/tests/fixtures/compat/empty-line-around-tables.json`, `crates/lynter-core/tests/fixtures/compat/heading-blank-lines.json`, `crates/lynter-core/tests/fixtures/compat/line-break-at-document-end.json`, `crates/lynter-core/tests/fixtures/rules-paragraph-spacing.json`

## Spaces within lines

Two rules clean up stray spaces. Both are off by default.

### `trailing-spaces`

Removes spaces and tabs at the end of every line. Fixes. Off by default.

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `two-space-linebreak` | boolean | `false` | Keep exactly two trailing spaces, which Markdown reads as a hard line break. |

```text
Line with trailing spaces and tabs.··
```

The `·` marks a trailing space or tab. After the rule:

```text
Line with trailing spaces and tabs.
```

### `remove-multiple-spaces`

Collapses two or more consecutive spaces inside a line to one. Fixes. Off by default. No options.

Leading indentation and trailing spaces are left alone, so a hard line break survives.

```markdown
Lorem ipsum   dolor  sit amet.
```

```markdown
Lorem ipsum dolor sit amet.
```

Sources: `crates/lynter-core/src/rules/compat/text.rs`, `crates/lynter-core/tests/fixtures/compat/trailing-spaces.json`, `crates/lynter-core/tests/fixtures/compat/remove-multiple-spaces.json`

## Lists

Five rules cover bullet markers, numbering, empty items and the space after a marker. `list-style` is on by default and `unordered-list-style` is its off-by-default sibling. Checklist items (`- [ ]`) are left alone by the marker rules.

### `list-style`

Normalizes unordered list bullets to one marker and, by default, removes blank lines between items. Fixes. On by default.

Ordered lists, thematic breaks and protected blocks are untouched. Only blank lines between two bullet items are removed, so a gap between a bullet list and the next paragraph stays.

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `marker` | string | `-` | `-` or `*`. Any other value is treated as `-`. |
| `tighten` | boolean | `true` | Remove blank lines between bullet items. |

```markdown
* a

* b
```

```markdown
- a
- b
```

### `unordered-list-style`

Uses one bullet marker for all unordered list items, at every nesting level. Fixes. Off by default.

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `style` | string | `consistent` | `-`, `*`, `+`, or `consistent`, which follows the first bullet in the note. |

```markdown
- Item 1
  * Sub
+ Item 2
```

With `style: "*"`:

```markdown
* Item 1
  * Sub
* Item 2
```

### `ordered-list-style`

Renumbers ordered lists and sets the character after the number. Fixes. Off by default.

Each list keeps its first number and counts up from it. Nested lists are numbered on their own.

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `number-style` | string | `ascending` | `ascending` counts up; `lazy` numbers every item `1`. |
| `end-style` | string | `.` | `.` or `)`, the character after the number. |

```markdown
1. Item 1
2. Item 2
4. Item 3
```

```markdown
1. Item 1
2. Item 2
3. Item 3
```

### `remove-empty-list-markers`

Deletes list items that have a marker but no content. Fixes. Off by default. No options.

Empty checkbox items such as `- [ ]` count as empty. Numbered items after a deleted one keep their numbers, so pair this rule with `ordered-list-style` to renumber.

```markdown
- item 1
-
- item 2
```

```markdown
- item 1
- item 2
```

### `space-after-list-markers`

Uses exactly one space after list markers and after checkboxes. Fixes. Off by default. No options.

```markdown
1.   Item 1
-   [ ] Item 2
- [x]    Item 3
```

```markdown
1. Item 1
- [ ] Item 2
- [x] Item 3
```

Sources: `crates/lynter-core/src/rules/compat/lists.rs`, `crates/lynter-core/src/rules/content/list_style.rs`, `crates/lynter-core/tests/fixtures/compat/unordered-list-style.json`, `crates/lynter-core/tests/fixtures/compat/ordered-list-style.json`, `crates/lynter-core/tests/fixtures/compat/remove-empty-list-markers.json`, `crates/lynter-core/tests/fixtures/compat/space-after-list-markers.json`, `crates/lynter-core/tests/fixtures/rules-list-style.json`

## Headings

Two rules keep heading structure valid. Both are off by default and reshape only ATX headings (`#` markers).

### `header-increment`

Limits each heading to one level deeper than its parent. Fixes. Off by default.

A heading that skips levels is moved up to the parent level plus one, and the headings under it follow. Going back up to a shallower level is allowed.

| Option | Type | Default | Meaning |
| --- | --- | --- | --- |
| `start-at-level-2` | boolean | `false` | Give headings that have no parent heading level 2 instead of keeping their level. |

```markdown
# H1
### H3
#### H4
```

```markdown
# H1
## H3
### H4
```

### `headings-start-line`

Removes the whitespace before a heading marker so the line is a heading. Fixes. Off by default. No options.

Only indentation of fewer than four columns is removed, since four or more is an indented code block. A `#` with no space after it is not a heading and is left as written.

```markdown
   ## Other heading preceded by 2 spaces ##
```

```markdown
## Other heading preceded by 2 spaces ##
```

Sources: `crates/lynter-core/src/rules/compat/headings.rs`, `crates/lynter-core/tests/fixtures/compat/header-increment.json`, `crates/lynter-core/tests/fixtures/compat/headings-start-line.json`
