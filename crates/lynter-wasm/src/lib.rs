//! Lynter for the browser and the Obsidian plugin. Every export takes and
//! returns plain strings; results are JSON text. This is the stable contract.
//!
//! Exports (all infallible; failures are reported inside the JSON):
//!
//! - `lint(config_yaml, note, path, today) -> String`. `today` is `YYYY-MM-DD`;
//!   an empty `path` means `note.md`. Runs `lynter_core::lint::lint_file`, the
//!   same call the CLI makes for Markdown. Success:
//!   `{"ok":true,"text":string,"changed":bool,"rules":[string],
//!   "profileChain":[string],"noteType":string|null,
//!   "violations":[{"rule":string,"message":string,"line":number,"fixed":bool}]}`.
//!   `rules` lists each rule id that reported, once, in order. `line` is
//!   1-based. A bad config yields `{"ok":false,"errors":[string]}`.
//! - `validate_config(yaml) -> String`:
//!   `{"ok":bool,"errors":[{"message":string,"line":number|null,"path":string}],
//!   "warnings":[{"message":string,"line":number|null,"path":string}]}`.
//!   `line` is the 1-based YAML line when it is known, else null. `path` is
//!   the config key path, empty when not applicable. `ok` is true exactly when
//!   `errors` is empty. `warnings` lists what loads but is ignored: unknown
//!   keys and rule ids no rule answers to.
//! - `profile_chain(config_yaml, path) -> String`: JSON array of profile
//!   names, `defaults` first; `[]` on config errors.
//! - `convert_data_json(json) -> string | undefined`: Obsidian Linter
//!   `data.json` text to `lynter.yaml` text; undefined when it cannot convert.
//! - `scaffold() -> String`: the starter `lynter.yaml` text.
//! - `version() -> String`: the crate version, for example `0.1.0`.
//!
//! New fields may be added to these shapes; existing fields do not change.

#![deny(unsafe_code)]

use lynter_core::config::{
    migrate_data_json, parse_config, scaffold as core_scaffold, unknown_rules, ConfigError,
    LoadResult,
};
use lynter_core::lint::{lint_file, ScopeContext};
use serde_json::{json, Value};
use wasm_bindgen::prelude::*;

fn errors_json(errors: Vec<String>) -> Value {
    json!({"ok": false, "errors": errors})
}

fn error_text(e: ConfigError) -> String {
    if e.path.is_empty() {
        e.message
    } else {
        format!("{}: {}", e.path, e.message)
    }
}

/// 1-based line from a serde_yaml message such as "... at line 3 column 5".
fn error_line(message: &str) -> Option<u64> {
    let rest = &message[message.find("line ")? + 5..];
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}

fn run(config_yaml: &str, note: &str, path: &str, today: &str) -> Value {
    match parse_config(config_yaml) {
        LoadResult::Err(errs) => errors_json(errs.into_iter().map(error_text).collect()),
        LoadResult::Ok { config, .. } => {
            let path = if path.is_empty() { "note.md" } else { path };
            let r = lint_file(note, path, &config, Some(today), &ScopeContext::default());
            let mut rules: Vec<&str> = Vec::new();
            for v in &r.violations {
                if !rules.contains(&v.rule.as_str()) {
                    rules.push(&v.rule);
                }
            }
            let violations: Vec<Value> = r
                .violations
                .iter()
                .map(|v| {
                    json!({
                        "rule": v.rule,
                        "message": v.message,
                        "line": v.line,
                        "fixed": v.fixed,
                    })
                })
                .collect();
            json!({
                "ok": true,
                "text": r.text,
                "changed": r.changed,
                "rules": rules,
                "profileChain": r.profile_chain,
                "noteType": r.note_type,
                "violations": violations,
            })
        }
    }
}

/// Lint one note; see the module contract.
#[wasm_bindgen]
pub fn lint(config_yaml: &str, note: &str, path: &str, today: &str) -> String {
    run(config_yaml, note, path, today).to_string()
}

/// The key a YAML line defines and its indent, when the line is `key:` (bare
/// or double-quoted, optionally a list item).
fn line_key(line: &str) -> Option<(usize, &str)> {
    let indent = line.len() - line.trim_start().len();
    let t = line.trim_start();
    let t = t.strip_prefix("- ").unwrap_or(t);
    let (key, rest) = match t.strip_prefix('"') {
        Some(quoted) => quoted.split_once('"')?,
        None => t.split_once(':').map(|(k, _)| (k, &t[k.len()..]))?,
    };
    rest.trim_start()
        .starts_with(':')
        .then_some((indent, key.trim_end()))
}

/// Best-effort 1-based line of a config key path such as
/// `profiles.work.rules.contractions`: each segment is looked for inside the
/// block of the one before it, so a key that appears in several stanzas
/// resolves to the right one. Flow mappings on one line resolve to that line.
fn key_line(yaml: &str, path: &str) -> Option<u64> {
    if path.is_empty() {
        return None;
    }
    let lines: Vec<&str> = yaml.lines().collect();
    let (mut from, mut parent_indent, mut found) = (0usize, None::<usize>, None::<usize>);
    for segment in path.split('.') {
        let mut hit = None;
        for (i, line) in lines.iter().enumerate().skip(from) {
            if line.trim().is_empty() || line.trim_start().starts_with('#') {
                continue;
            }
            let indent = line.len() - line.trim_start().len();
            if parent_indent.is_some_and(|p| indent <= p) {
                break;
            }
            if line_key(line).is_some_and(|(_, key)| key == segment) {
                hit = Some((i, indent));
                break;
            }
        }
        // A segment that is not on a line of its own sits in a flow mapping on
        // the last line found, which is the best answer available.
        let Some((i, indent)) = hit else { break };
        found = Some(i);
        from = i + 1;
        parent_indent = Some(indent);
    }
    found.map(|i| i as u64 + 1)
}

/// Key path quoted in a loader warning such as `unknown key "a.b" ignored`.
fn quoted_path(message: &str) -> &str {
    message.split('"').nth(1).unwrap_or("")
}

/// Validate a `lynter.yaml` text; see the module contract.
#[wasm_bindgen]
pub fn validate_config(yaml: &str) -> String {
    let mut warnings: Vec<Value> = Vec::new();
    let errors: Vec<Value> = match parse_config(yaml) {
        LoadResult::Ok {
            config,
            warnings: loader,
        } => {
            // `adapters` belongs to the command line, which validates it itself.
            for message in loader.into_iter().filter(|m| !m.contains(".adapters\"")) {
                let path = quoted_path(&message).to_owned();
                warnings.push(json!({"line": key_line(yaml, &path), "path": path, "message": message}));
            }
            for unknown in unknown_rules(&config) {
                warnings.push(json!({
                    "message": unknown.message(),
                    "line": key_line(yaml, &unknown.path),
                    "path": unknown.path,
                }));
            }
            Vec::new()
        }
        LoadResult::Err(errs) => errs
            .into_iter()
            .map(|e| json!({"message": e.message, "line": error_line(&e.message).or_else(|| key_line(yaml, &e.path)), "path": e.path}))
            .collect(),
    };
    json!({"ok": errors.is_empty(), "errors": errors, "warnings": warnings}).to_string()
}

/// Profile chain for a path, as a JSON array of names; `[]` on config errors.
#[wasm_bindgen]
pub fn profile_chain(config_yaml: &str, path: &str) -> String {
    let v = run(config_yaml, "", path, "2026-01-01");
    v.get("profileChain")
        .cloned()
        .unwrap_or_else(|| json!([]))
        .to_string()
}

/// Convert an Obsidian Linter `data.json` text to `lynter.yaml` text.
#[wasm_bindgen]
pub fn convert_data_json(json: &str) -> Option<String> {
    migrate_data_json(json).ok()
}

/// The starter `lynter.yaml`.
#[wasm_bindgen]
pub fn scaffold() -> String {
    core_scaffold()
}

/// Crate version.
#[wasm_bindgen]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use lynter_core::config::parse_config;

    #[test]
    fn lint_roundtrip() {
        let out: Value =
            serde_json::from_str(&lint(&scaffold(), "# a  \n", "a.md", "2026-01-01")).unwrap();
        assert_eq!(out["ok"], true);
        let bad: Value =
            serde_json::from_str(&lint("version: [", "x", "a.md", "2026-01-01")).unwrap();
        assert_eq!(bad["ok"], false);
    }

    #[test]
    fn lint_matches_lint_file() {
        let yaml = scaffold();
        let LoadResult::Ok { config, .. } = parse_config(&yaml) else {
            panic!("scaffold must load");
        };
        for (note, path) in [
            ("# a  \n\n\n\ntext   \n", "a.md"),
            ("---\ntitle: x\n---\n# Hello World\n* item\n", "notes/b.md"),
        ] {
            let want = lint_file(
                note,
                path,
                &config,
                Some("2026-01-01"),
                &ScopeContext::default(),
            );
            let got: Value = serde_json::from_str(&lint(&yaml, note, path, "2026-01-01")).unwrap();
            assert_eq!(got["text"], want.text);
            assert_eq!(got["changed"], want.changed);
            assert_eq!(got["profileChain"], json!(want.profile_chain));
            assert_eq!(
                got["violations"].as_array().unwrap().len(),
                want.violations.len()
            );
        }
    }

    #[test]
    fn validate_config_shapes() {
        let ok: Value = serde_json::from_str(&validate_config(&scaffold())).unwrap();
        assert_eq!(ok, json!({"ok": true, "errors": [], "warnings": []}));
        let bad: Value = serde_json::from_str(&validate_config("version: [")).unwrap();
        assert_eq!(bad["ok"], false);
        let first = &bad["errors"][0];
        assert!(first["message"]
            .as_str()
            .unwrap()
            .contains("yaml parse error"));
        assert!(first["line"].is_number() || first["line"].is_null());
    }

    #[test]
    fn validate_config_warns_about_unknown_rules_and_keys() {
        let yaml =
            "version: 1\nbogus: 1\ndefaults:\n  rules:\n    contraction:\n      enabled: true\n";
        let out: Value = serde_json::from_str(&validate_config(yaml)).unwrap();
        assert_eq!(out["ok"], true);
        let warnings = out["warnings"].as_array().unwrap();
        assert_eq!(warnings.len(), 2);
        assert_eq!(warnings[0]["path"], "bogus");
        assert_eq!(warnings[0]["line"], 2);
        assert_eq!(warnings[1]["path"], "defaults.rules.contraction");
        assert_eq!(warnings[1]["line"], 5);
        assert!(warnings[1]["message"]
            .as_str()
            .unwrap()
            .contains("did you mean \"contractions\""));
    }

    #[test]
    fn key_line_follows_the_path_into_the_right_stanza() {
        let yaml = "version: 1\ndefaults:\n  rules:\n    contractions:\n      enabled: true\nprofiles:\n  work:\n    rules:\n      contractions:\n        enabled: false\n      \"quoted\": {enabled: true}\n";
        assert_eq!(key_line(yaml, "defaults.rules.contractions"), Some(4));
        assert_eq!(key_line(yaml, "profiles.work.rules.contractions"), Some(9));
        assert_eq!(key_line(yaml, "profiles.work.rules.quoted"), Some(11));
        assert_eq!(
            key_line(yaml, "profiles.work.rules.quoted.enabled"),
            Some(11)
        );
        assert_eq!(key_line(yaml, "profiles.home"), Some(6));
        assert_eq!(key_line(yaml, "missing"), None);
        assert_eq!(key_line(yaml, ""), None);
    }

    #[test]
    fn version_is_nonempty() {
        assert!(!version().is_empty());
    }
}
