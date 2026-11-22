//! Compiled defaults for new users: what runs with no `lynter.yaml`, and the
//! baseline the scaffold writes.
//!
//! Opt-in by default: `contractions`, `header-case` (heading casing),
//! `strip-strong`, `h1-matches-stem`, and `kebab-case-filename`. Each is
//! present and disabled, so turning one on is a one-word edit.

use super::{parse_config, Config, LoadResult, RulesConfig};
use serde_json::{json, Value};

/// Priority keys, in order; aliases and tags always sort last.
pub const DEFAULT_PRIORITY_KEYS: [&str; 8] = [
    "preset",
    "domain",
    "category",
    "sub-category",
    "types",
    "date-created",
    "date-revised",
    "links",
];

/// Rule ids that are off until a user opts in.
pub const OPT_IN_RULES: [&str; 5] = [
    "contractions",
    "header-case",
    "strip-strong",
    "h1-matches-stem",
    "kebab-case-filename",
];

/// The starter `base` note type.
pub fn base_note_type() -> Value {
    json!({
        "required": {
            "domain": null, "category": null, "sub-category": null, "type": "base",
            "status": "NEW", "aliases": [], "tags": [], "relo-file": false, "relo-folder": ""
        },
        "match": {"frontmatter": {"type": "base"}}
    })
}

pub fn default_rules() -> RulesConfig {
    let v = json!({
        "yaml-key-sort": {"enabled": true, "priority-keys": DEFAULT_PRIORITY_KEYS},
        "yaml-timestamp": {"enabled": true},
        "alphabetize-property-values": {"enabled": true},
        "paragraph-spacing": {"enabled": true, "blank-lines": 1},
        "list-style": {"enabled": true, "marker": "-", "tighten": true},
        "replace-em-dash": {"enabled": true},
        "code-checks": {"enabled": true},
        "code-checks-fix": {"enabled": true},
        "join-paragraph-lines": {"enabled": true},
        "prose-list-to-sentences": {"enabled": true},
        "strip-strong": {"enabled": false},
        "header-case": {"enabled": false},
        "contractions": {"enabled": false},
        "h1-matches-stem": {"enabled": false},
        "kebab-case-filename": {"enabled": false}
    });
    v.as_object().cloned().unwrap_or_default()
}

/// The config used when no `lynter.yaml` exists.
pub fn default_config() -> Config {
    let raw = json!({
        "version": 1,
        "defaults": {"rules": default_rules()},
        "note-types": {"base": base_note_type()},
        "rename": {"mode": "flag"},
        "ignore": {"folders": [], "files": []}
    });
    match parse_config(&raw.to_string()) {
        LoadResult::Ok { config, .. } => *config,
        LoadResult::Err(e) => unreachable!("compiled defaults must validate: {e:?}"),
    }
}
