//! Adapter options from lynter.yaml. Core's config loader does not know the
//! `adapters` key, so it is parsed here from the raw config: `defaults.adapters`
//! first, then `profiles.<name>.adapters` for each profile in the resolved
//! chain, later layers overriding earlier ones key by key.

use serde_json::{Map, Value};
use std::collections::BTreeMap;

const ADAPTERS: &[(&str, &[&str])] = &[
    ("ruff", &["enabled", "path", "args", "select"]),
    ("biome", &["enabled", "path", "args"]),
];

/// Effective options for one adapter. Adapters are off by default.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AdapterSettings {
    pub enabled: bool,
    /// Explicit binary path; PATH lookup when absent.
    pub path: Option<String>,
    /// Extra arguments passed to the tool before the files.
    pub args: Vec<String>,
    /// ruff only: rule codes for `--select`.
    pub select: Vec<String>,
}

fn string_list(v: &Value) -> Option<Vec<String>> {
    v.as_array()?
        .iter()
        .map(|s| s.as_str().map(str::to_owned))
        .collect()
}

/// Validate one `adapters` mapping; unknown adapters and keys are errors.
fn validate_block(block: &Value, at: &str, errors: &mut Vec<String>) {
    let Some(map) = block.as_object() else {
        errors.push(format!("{at}: must be a mapping of adapter id to options"));
        return;
    };
    for (id, opts) in map {
        let Some((_, keys)) = ADAPTERS.iter().find(|(n, _)| n == id) else {
            errors.push(format!(
                "{at}.{id}: unknown adapter (expected one of: ruff, biome)"
            ));
            continue;
        };
        let Some(o) = opts.as_object() else {
            errors.push(format!("{at}.{id}: must be a mapping"));
            continue;
        };
        for (k, v) in o {
            if !keys.contains(&k.as_str()) {
                errors.push(format!("{at}.{id}.{k}: unknown key"));
                continue;
            }
            let (ok, want) = match k.as_str() {
                "enabled" => (v.is_boolean(), "a boolean"),
                "path" => (v.is_string(), "a string"),
                _ => (string_list(v).is_some(), "a list of strings"),
            };
            if !ok {
                errors.push(format!("{at}.{id}.{k}: must be {want}"));
            }
        }
    }
}

/// Every `adapters` block in a raw config, as errors. Empty when valid.
pub fn validate_config(raw: &Value) -> Vec<String> {
    let mut errors = Vec::new();
    if let Some(b) = raw.get("defaults").and_then(|d| d.get("adapters")) {
        validate_block(b, "defaults.adapters", &mut errors);
    }
    if let Some(profiles) = raw.get("profiles").and_then(Value::as_object) {
        for (name, p) in profiles {
            if let Some(b) = p.get("adapters") {
                validate_block(b, &format!("profiles.{name}.adapters"), &mut errors);
            }
        }
    }
    errors
}

fn layer(target: &mut BTreeMap<String, AdapterSettings>, block: Option<&Value>) {
    let Some(Value::Object(map)) = block else {
        return;
    };
    for (id, opts) in map {
        let Some(o) = opts.as_object() else { continue };
        let s = target.entry(id.clone()).or_default();
        apply(s, o);
    }
}

fn apply(s: &mut AdapterSettings, o: &Map<String, Value>) {
    if let Some(b) = o.get("enabled").and_then(Value::as_bool) {
        s.enabled = b;
    }
    if let Some(p) = o.get("path").and_then(Value::as_str) {
        s.path = Some(p.to_owned());
    }
    if let Some(a) = o.get("args").and_then(string_list) {
        s.args = a;
    }
    if let Some(a) = o.get("select").and_then(string_list) {
        s.select = a;
    }
}

/// Effective settings per adapter id for a file whose resolved profile chain is
/// `chain` (`defaults` first, as `ResolvedProfile::chain` returns it).
pub fn resolve_settings(raw: &Value, chain: &[String]) -> BTreeMap<String, AdapterSettings> {
    let mut out = BTreeMap::new();
    layer(
        &mut out,
        raw.get("defaults").and_then(|d| d.get("adapters")),
    );
    for name in chain.iter().filter(|n| n.as_str() != "defaults") {
        layer(
            &mut out,
            raw.get("profiles")
                .and_then(|p| p.get(name))
                .and_then(|p| p.get("adapters")),
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn off_by_default() {
        assert!(resolve_settings(&json!({"version": 1}), &["defaults".to_owned()]).is_empty());
        assert!(!AdapterSettings::default().enabled);
    }

    #[test]
    fn scopes_inherit_key_by_key() {
        let raw = json!({
            "defaults": {"adapters": {"ruff": {"enabled": true, "select": ["F"], "args": ["--preview"]}}},
            "profiles": {
                "legacy": {"adapters": {"ruff": {"select": ["E"]}}},
                "off": {"adapters": {"ruff": {"enabled": false}}}
            }
        });
        let chain = |n: &str| vec!["defaults".to_owned(), n.to_owned()];
        let s = &resolve_settings(&raw, &chain("legacy"))["ruff"];
        assert!(s.enabled);
        assert_eq!(s.select, vec!["E"]);
        assert_eq!(s.args, vec!["--preview"]);
        assert!(!resolve_settings(&raw, &chain("off"))["ruff"].enabled);
    }

    #[test]
    fn unknown_keys_and_adapters_are_errors() {
        let raw = json!({
            "defaults": {"adapters": {"ruff": {"enabled": true, "bogus": 1}, "pylint": {}}},
            "profiles": {"p": {"adapters": {"biome": {"select": ["x"], "path": 3}}}}
        });
        let e = validate_config(&raw);
        assert_eq!(e.len(), 4, "{e:?}");
        assert!(e
            .iter()
            .any(|m| m.contains("defaults.adapters.ruff.bogus: unknown key")));
        assert!(e
            .iter()
            .any(|m| m.contains("defaults.adapters.pylint: unknown adapter")));
        assert!(e.iter().any(|m| m.contains("biome.select: unknown key")));
        assert!(e.iter().any(|m| m.contains("biome.path: must be a string")));
    }

    #[test]
    fn valid_config_has_no_errors() {
        let raw = json!({"defaults": {"adapters": {"ruff": {"enabled": true, "path": "/x", "args": [], "select": ["F"]}, "biome": {"enabled": false}}}});
        assert!(validate_config(&raw).is_empty());
    }
}
