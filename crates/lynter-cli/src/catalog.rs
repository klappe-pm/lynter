//! `lynter rules`: every rule Lynter ships, with its group, description and
//! default options. The JSON form is the source the config builder on
//! lynter.dev is generated from, so the builder cannot drift from the rules.

use lynter_core::rule::Rule;
use lynter_core::rules::{compat, content, notes_rules};
use serde_json::{json, Value};

/// One row per rule, in registry order: the core rules by family, then the
/// code and data rules.
pub fn catalog() -> Vec<Value> {
    let core = lynter_core::rules::builtin_rules();
    let family =
        |rules: Vec<Box<dyn Rule>>| -> Vec<&'static str> { rules.iter().map(|r| r.id()).collect() };
    let content_ids = family(content::rules());
    let compat_ids = family(compat::rules());
    let note_type_ids = family(lynter_core::notetypes::rules());
    let notes_ids = family(notes_rules());
    let mut rows: Vec<Value> = core
        .iter()
        .map(|r| {
            let id = r.id();
            let group = if content_ids.contains(&id) {
                "content"
            } else if compat_ids.contains(&id) {
                "formatting"
            } else if note_type_ids.contains(&id) {
                "note-types"
            } else if notes_ids.contains(&id) {
                "notes"
            } else {
                "wording"
            };
            json!({
                "id": id,
                "group": group,
                "host": "core",
                "description": r.description(),
                "options": Value::Object(r.default_options()),
            })
        })
        .collect();
    rows.extend(lynter_code::registry().iter().map(|r| {
        let id = r.id();
        let group = if id.starts_with("json-") || id.starts_with("yaml-") {
            "data"
        } else {
            "code"
        };
        json!({
            "id": id,
            "group": group,
            "host": "code",
            "description": r.description(),
            "options": Value::Object(r.default_options()),
        })
    }));
    rows
}

/// Print the catalog: a JSON array with `--json`, otherwise one line per rule.
pub fn print(as_json: bool) {
    let rows = catalog();
    if as_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&rows).expect("catalog is plain JSON")
        );
        return;
    }
    for row in &rows {
        println!(
            "{:<42} {:<11} {}",
            row["id"].as_str().unwrap_or_default(),
            row["group"].as_str().unwrap_or_default(),
            row["description"].as_str().unwrap_or_default()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_known_rule_is_listed_once_with_a_description() {
        let rows = catalog();
        let mut ids: Vec<&str> = rows.iter().map(|r| r["id"].as_str().unwrap()).collect();
        assert_eq!(ids.len(), lynter_core::config::known_rule_ids().len());
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), rows.len());
        assert!(rows
            .iter()
            .all(|r| !r["description"].as_str().unwrap().is_empty()));
    }
}
