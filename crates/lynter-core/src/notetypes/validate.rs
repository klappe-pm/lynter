use super::frontmatter::{parse_frontmatter_data, split_document};
use super::schema::schema_from;
use crate::document::Document;
use crate::edit::Edit;
use crate::rule::{Options, Rule};
use serde_json::{Map, Value};

fn is_map(v: &Value) -> bool {
    v.is_object()
}

/// JavaScript `String(value)` for the shapes a schema can hold.
fn js_string(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Array(a) => a.iter().map(js_string).collect::<Vec<_>>().join(","),
        Value::Object(_) => "[object Object]".to_owned(),
        Value::Number(n) => super::format::scalar(&Value::Number(n.clone())),
        Value::Bool(b) => b.to_string(),
    }
}

/// `JSON.stringify` of a flat scalar.
fn js_json(v: &Value) -> String {
    match v {
        Value::Number(n) => super::format::scalar(&Value::Number(n.clone())),
        other => other.to_string(),
    }
}

fn strict_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x.as_f64() == y.as_f64(),
        _ => a == b,
    }
}

/// Structured violation messages, one per problem, formatted `key: problem`.
pub fn collect_note_type_violations(text: &str, schema: &Map<String, Value>) -> Vec<String> {
    let fm = split_document(text);
    let Some(data) = parse_frontmatter_data(&fm.yaml_lines) else {
        return vec!["frontmatter: invalid YAML or not a flat mapping".to_owned()];
    };
    let mut messages = Vec::new();
    let required = schema.get("required").and_then(Value::as_object);
    let values = schema.get("values").and_then(Value::as_object);

    let mut managed: Vec<String> = Vec::new();
    let mut add = |k: &str| {
        if !managed.iter().any(|m| m == k) {
            managed.push(k.to_owned());
        }
    };
    for k in required.into_iter().flat_map(Map::keys) {
        add(k);
    }
    for k in values.into_iter().flat_map(Map::keys) {
        add(k);
    }
    if let Some(dk) = schema.get("date-keys").and_then(Value::as_object) {
        for name in ["created", "revised"] {
            if let Some(s) = dk
                .get(name)
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
            {
                add(s);
            }
        }
    }

    for key in &managed {
        let Some(value) = data.get(key) else { continue };
        if is_map(value) {
            messages.push(format!(
                "{key}: nested maps are not allowed in managed keys"
            ));
            continue;
        }
        if let Value::Array(a) = value {
            if a.iter().any(|i| i.is_array() || i.is_object()) {
                messages.push(format!("{key}: list items must be flat scalars"));
            }
        }
    }

    for (key, allowed) in values.into_iter().flatten() {
        let (Some(value), Value::Array(allowed)) = (data.get(key), allowed) else {
            continue;
        };
        if value.is_null() || is_map(value) {
            continue;
        }
        let items: Vec<&Value> = match value {
            Value::Array(a) => a.iter().collect(),
            v => vec![v],
        };
        for item in items {
            if item.is_null() || item.is_array() || item.is_object() {
                continue;
            }
            if !allowed.iter().any(|c| strict_eq(c, item)) {
                let list = allowed.iter().map(js_string).collect::<Vec<_>>().join(", ");
                messages.push(format!(
                    "{key}: value {} is not one of [{list}]",
                    js_json(item)
                ));
            }
        }
    }

    for (key, default) in required.into_iter().flatten() {
        if default.is_null() && !data.contains_key(key) {
            messages.push(format!("{key}: required key is missing and has no default"));
        }
    }
    messages
}

/// Report-only. The edit appends the violation trailer; the runner never
/// writes it back.
pub struct NoteTypeValidate;

impl Rule for NoteTypeValidate {
    fn id(&self) -> &'static str {
        "note-type-validate"
    }
    fn description(&self) -> &'static str {
        "Report note-type frontmatter violations: enum constraints, nested maps in managed keys, and missing required keys without defaults."
    }
    fn default_options(&self) -> Options {
        Options::new()
    }
    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit> {
        let Some(schema) = schema_from(options) else {
            return Vec::new();
        };
        let messages = collect_note_type_violations(doc.text(), schema);
        if messages.is_empty() {
            return Vec::new();
        }
        let end = doc.text().len();
        vec![Edit::new(
            end,
            end,
            format!("\n<!-- note-type-validate\n{}\n-->", messages.join("\n")),
            "note type violations",
        )]
    }
}
