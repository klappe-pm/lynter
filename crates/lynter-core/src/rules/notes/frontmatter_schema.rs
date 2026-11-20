use super::frontmatter;
use super::{opt_list, valid_ymd, Reports};
use crate::document::Document;
use crate::edit::Edit;
use crate::rule::{Options, Rule};
use regex::Regex;
use serde_json::{json, Value};
use std::sync::LazyLock;

static DATE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(\d{4})-(\d{2})-(\d{2})(?:[T ]\d{2}:\d{2}(?::\d{2}(?:\.\d+)?)?(?:Z|[+-]\d{2}:?\d{2})?)?$")
        .expect("regex")
});

fn type_name(v: &Value) -> &'static str {
    match v {
        Value::String(_) => "string",
        Value::Number(_) => "number",
        Value::Bool(_) => "bool",
        Value::Array(_) => "list",
        Value::Object(_) => "map",
        Value::Null => "null",
    }
}

fn is_date(s: &str) -> bool {
    DATE.captures(s).is_some_and(|c| {
        let n = |i| c.get(i).and_then(|m| m.as_str().parse::<u32>().ok());
        matches!((n(1), n(2), n(3)), (Some(y), Some(m), Some(d)) if valid_ymd(y, m, d))
    })
}

fn type_ok(v: &Value, ty: &str) -> Option<bool> {
    Some(match ty {
        "string" => v.is_string(),
        "number" => v.is_number(),
        "bool" => v.is_boolean(),
        "list" => v.is_array(),
        "date" => v.as_str().is_some_and(is_date),
        _ => return None,
    })
}

fn scalar(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// Validate frontmatter against required keys, value types, and enums.
pub struct FrontmatterSchema;

impl Rule for FrontmatterSchema {
    fn id(&self) -> &'static str {
        "frontmatter-schema"
    }

    fn description(&self) -> &'static str {
        "Report frontmatter that lacks required keys, holds values of the wrong type, or uses values outside an enum. Report only."
    }

    fn default_options(&self) -> Options {
        json!({"required": [], "types": {}, "enums": {}})
            .as_object()
            .cloned()
            .unwrap_or_default()
    }

    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit> {
        let required = opt_list(options, "required");
        let types = options.get("types").and_then(Value::as_object);
        let enums = options.get("enums").and_then(Value::as_object);
        let configured = !required.is_empty()
            || types.is_some_and(|t| !t.is_empty())
            || enums.is_some_and(|e| !e.is_empty());
        if !configured {
            return Vec::new();
        }
        let text = doc.text();
        let mut reports = Reports::default();
        let Some(fm) = frontmatter::parse(text) else {
            if !required.is_empty() {
                reports.push(
                    0,
                    0,
                    format!(
                        "missing frontmatter, required keys: {}",
                        required.join(", ")
                    ),
                );
            }
            return reports.into_edits(text);
        };
        let Some(map) = &fm.value else {
            reports.push(fm.open.0, fm.open.1, "frontmatter is not valid YAML");
            return reports.into_edits(text);
        };
        for key in &required {
            if map.get(key).is_none_or(Value::is_null) {
                reports.push(
                    fm.open.0,
                    fm.open.1,
                    format!("missing required key \"{key}\""),
                );
            }
        }
        let mut keys: Vec<&String> = Vec::new();
        for m in [types, enums].into_iter().flatten() {
            for k in m.keys() {
                if !keys.contains(&k) {
                    keys.push(k);
                }
            }
        }
        for key in keys {
            let Some(v) = map.get(key).filter(|v| !v.is_null()) else {
                continue;
            };
            let (s, e) = fm.key_range(key).unwrap_or(fm.open);
            if let Some(ty) = types.and_then(|t| t.get(key)).and_then(Value::as_str) {
                if type_ok(v, ty) == Some(false) {
                    reports.push(
                        s,
                        e,
                        format!("key \"{key}\" expects {ty}, found {} {}", type_name(v), v),
                    );
                }
            }
            if let Some(Value::Array(allowed)) = enums.and_then(|m| m.get(key)) {
                let allowed: Vec<String> = allowed.iter().map(scalar).collect();
                let values: Vec<&Value> = match v {
                    Value::Array(a) => a.iter().collect(),
                    other => vec![other],
                };
                for item in values {
                    if !allowed.contains(&scalar(item)) {
                        reports.push(
                            s,
                            e,
                            format!(
                                "key \"{key}\" expects one of {}, found {item}",
                                allowed.join(", ")
                            ),
                        );
                    }
                }
            }
        }
        reports.into_edits(text)
    }
}
