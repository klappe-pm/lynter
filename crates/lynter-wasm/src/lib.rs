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
//!   `{"ok":bool,"errors":[{"message":string,"line":number|null,"path":string}]}`.
//!   `line` is the 1-based YAML line when the parser reports one, else null.
//!   `path` is the config key path, empty when not applicable. `ok` is true
//!   exactly when `errors` is empty.
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
    migrate_data_json, parse_config, scaffold as core_scaffold, ConfigError, LoadResult,
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

/// Validate a `lynter.yaml` text; see the module contract.
#[wasm_bindgen]
pub fn validate_config(yaml: &str) -> String {
    let errors: Vec<Value> = match parse_config(yaml) {
        LoadResult::Ok { .. } => Vec::new(),
        LoadResult::Err(errs) => errs
            .into_iter()
            .map(|e| json!({"message": e.message, "line": error_line(&e.message), "path": e.path}))
            .collect(),
    };
    json!({"ok": errors.is_empty(), "errors": errors}).to_string()
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
        assert_eq!(ok, json!({"ok": true, "errors": []}));
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
    fn version_is_nonempty() {
        assert!(!version().is_empty());
    }
}
