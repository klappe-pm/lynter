//! The commented starter `lynter.yaml`.

pub const STARTER: &str = "# lynter.yaml is the control plane for lynter: the single git-tracked
# source of truth for rule configuration and scoped profiles.
# Canonical filename: lynter.yaml at the vault root. linter.yaml and
# lappe-linter.yaml are deprecated aliases; lynter.yaml wins when several exist.

version: 1 # config schema version; only 1 is valid, do not change

defaults:
  rules: # applied to every file; each key is a rule id, set enabled true or false
    strip-strong:
      enabled: true # removes ** and __ strong emphasis, keeping the inner text
    replace-em-dash:
      enabled: true # replaces em dashes with a configured separator
    header-case:
      enabled: true # normalizes heading capitalization
    contractions:
      enabled: true # expands or contracts phrases; set a phrase such as \"it is\": false to opt out

profiles: # named per-scope overrides; a matching file gets its rules on top of defaults
  drafts: # example profile, rename or delete it freely
    match:
      path: [\"drafts/**\"] # applies to files under drafts/
    rules:
      header-case:
        enabled: false # leave heading case alone in drafts

ignore:
  folders: [] # folders relative to this file that lynter never touches, e.g. [templates]
  files: [] # files relative to this file that lynter never touches, e.g. [inbox/scratch.md]
";
