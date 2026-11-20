/// A commented starter `lynter.yaml`. Every default carries a comment saying
/// what it does and how to change it. Rules that change prose or names are
/// present and disabled; set `enabled: true` to opt in. Always parses clean.
pub fn scaffold() -> String {
    r#"# lynter.yaml is the control plane for Lynter: the single git-tracked source of
# truth for rule configuration, profiles, and note-type schemas.
# Canonical filename: lynter.yaml at the vault root. linter.yaml and
# lappe-linter.yaml are deprecated aliases; when several exist, lynter.yaml wins.

version: 1 # config schema version; only 1 is valid, do not change

defaults:
  rules: # rules applied to every file; each key is a rule id, set enabled true/false
    yaml-key-sort:
      enabled: true # sorts frontmatter keys; set false to leave key order alone
      priority-keys: [preset, domain, category, sub-category, types, date-created, date-revised, links] # keys sort in this order; aliases and tags always follow last
    yaml-timestamp:
      enabled: true # date-created on first lint, date-revised only when content changed; dates yyyy-MM-dd
    alphabetize-property-values:
      enabled: true # sorts the values of list properties (aliases, tags, links); never the keys
    paragraph-spacing:
      enabled: true # one blank line between blocks; change blank-lines to 0, 1, or 2
      blank-lines: 1
    list-style:
      enabled: true # dash bullets with no blank lines between items; marker accepts "-" or "*"
      marker: "-"
      tighten: true
    replace-em-dash:
      enabled: true # replaces em dashes and connector en dashes with a configured separator
    code-checks:
      enabled: true # reports matches of the checks listed under code-checks below; a no-op until one is enabled
    code-checks-fix:
      enabled: true # applies the checks that define fix.replacement; a no-op until one is enabled
    join-paragraph-lines:
      enabled: true # report-only: flags hard-wrapped prose but never rewrites it; enable join-paragraph-lines-fix to rewrite
    prose-list-to-sentences:
      enabled: true # report-only: flags simple bullet lists that could be prose; enable prose-list-to-sentences-fix to rewrite

    # Opt-in rules. They are off by default because they rewrite wording or names.
    strip-strong:
      enabled: false # opt in: removes ** and __ strong emphasis, keeping the inner text
    header-case:
      enabled: false # opt in: heading casing; also set a style per level, for example h1: Title Case
    contractions:
      enabled: false # opt in: rewrites phrases such as "it is" to "it's"; set a phrase to false to exempt it
    h1-matches-stem:
      enabled: false # opt in: keeps the first H1 equal to the filename stem, inserted when missing
    kebab-case-filename:
      enabled: false # opt in: reports filenames that are not kebab-case; rename.mode controls fixing

profiles: # named per-scope overrides; a file matching a profile gets its rules on top of defaults
  tasks-notes: # example profile, rename or delete it freely
    match:
      frontmatter: {category: task} # applies to files whose frontmatter has category: task
    rules: {} # per-rule overrides for matched files, same shape as defaults.rules

note-types: {base: {required: {domain: null, category: null, sub-category: null, type: base, status: NEW, aliases: [], tags: [], relo-file: false, relo-folder: ""}, match: {frontmatter: {type: base}}}} # base schema keeps relocation keys required; scoped types may only use a subset of these required-key names

code-checks: {} # declarative code linting over fenced code blocks; enable built-ins like "no-token-shaped-strings: {enabled: true}" or define your own {pattern, languages, message}; patterns are bounded regex, never executed code

rename:
  mode: flag # filename rule behavior: off = disabled, flag = report only, rename = fix filenames and update links
  strategy: replace # prepend/append move a leading or trailing date/number; without one they match replace

ignore:
  folders: [] # vault-relative folders the linter never touches, e.g. [templates, .obsidian]
  files: [] # vault-relative files the linter never touches, e.g. [inbox/scratch.md]
"#
    .to_owned()
}
