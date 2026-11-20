The Obsidian plugin runs the same core as the CLI. Settings in the plugin are a view over `lynter.yaml`, and edits to the file win.

## Frontmatter

Shipped today: sort keys with a priority list, sort the values of list properties, manage `date-created` and `date-revised`, and define note types with required keys, key order, and allowed values.

```yaml
note-types:
  task:
    match:
      frontmatter: {type: task}
    required:
      status: NEW
    values:
      status: [NEW, DONE, ARCHIVED]
```

## Tags

Shipped today: sort the values of the `tags` property, and scope rules by tag. Tag normalization and a tag allow list are Coming soon.

## Tasks

Task formatting is Coming soon.

## File and folder naming

Shipped today: flag file names that are not kebab-case, keep the first heading equal to the file name, and rename with link updates when `rename.mode` is `rename`. Folder name rules are Coming soon.

## Scopes

A scope matches a path, tag, or property, starts from the defaults, and states only what differs. Deeper scopes override broader ones. Use the [playground](/playground/) to try one, or run `explain` on a path in the CLI.
