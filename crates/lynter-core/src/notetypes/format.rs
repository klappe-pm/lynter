//! YAML scalar and block rendering that follows the `yaml` package's default
//! output for the shapes Lynter writes: plain strings unless a plain form
//! would be misread, two-space block mappings, indented block sequences.
//! Long-string folding and multi-line block scalars are not reproduced.

use regex::Regex;
use serde_json::Value;
use std::sync::LazyLock;

static NEEDS_QUOTE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?s)^$|^[\n\t ,\[\]{}#&*!|>'"%@`]|^[?-]$|^[?-][ \t]|[\n:][ \t]|[ \t]\n|[\n\t ]#|[\n\t:]$"#,
    )
    .expect("regex")
});
static CORE_NON_STRING: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(concat!(
        r"^(?:~|[Nn]ull|NULL)?$",
        r"|^(?:[Tt]rue|TRUE|[Ff]alse|FALSE)$",
        r"|^[-+]?[0-9]+$|^0o[0-7]+$|^0x[0-9a-fA-F]+$",
        r"|^[-+]?(?:\.[0-9]+|[0-9]+(?:\.[0-9]*)?)(?:[eE][-+]?[0-9]+)?$",
        r"|^[-+]?\.(?:inf|Inf|INF)$|^\.(?:nan|NaN|NAN)$",
    ))
    .expect("regex")
});

fn double_quoted(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\0' => out.push_str("\\0"),
            c if (c as u32) < 0x20 || c as u32 == 0x7f => {
                out.push_str(&format!("\\x{:02X}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn quoted(s: &str) -> String {
    let has_double = s.contains('"');
    let has_single = s.contains('\'');
    if has_double && !has_single && !s.contains('\n') {
        format!("'{s}'")
    } else {
        double_quoted(s)
    }
}

/// A string as a plain or quoted YAML scalar.
pub fn string_scalar(s: &str) -> String {
    if s.contains('\n') || NEEDS_QUOTE.is_match(s) || CORE_NON_STRING.is_match(s) {
        quoted(s)
    } else {
        s.to_owned()
    }
}

fn number_scalar(n: &serde_json::Number) -> String {
    if let Some(f) = n.as_f64().filter(|_| !n.is_i64() && !n.is_u64()) {
        if f.is_nan() {
            return ".nan".to_owned();
        }
        if f.is_infinite() {
            return if f > 0.0 { ".inf" } else { "-.inf" }.to_owned();
        }
        if f.fract() == 0.0 && f.abs() < 1e21 {
            return format!("{f:.0}");
        }
        return format!("{f}");
    }
    n.to_string()
}

/// `formatScalar`: one flat value rendered inline.
pub fn scalar(v: &Value) -> String {
    match v {
        Value::Null => "null".to_owned(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => number_scalar(n),
        Value::String(s) => string_scalar(s),
        other => other.to_string(),
    }
}

fn pad(n: usize) -> String {
    " ".repeat(n)
}

fn emit_inline(v: &Value) -> Option<String> {
    match v {
        Value::Object(m) if m.is_empty() => Some("{}".to_owned()),
        Value::Array(a) if a.is_empty() => Some("[]".to_owned()),
        Value::Object(_) | Value::Array(_) => None,
        other => Some(scalar(other)),
    }
}

fn emit_into(v: &Value, indent: usize, out: &mut Vec<String>) {
    match v {
        Value::Object(m) => {
            for (k, val) in m {
                let key = string_scalar(k);
                match emit_inline(val) {
                    Some(s) => out.push(format!("{}{key}: {s}", pad(indent))),
                    None => {
                        out.push(format!("{}{key}:", pad(indent)));
                        emit_into(val, indent + 2, out);
                    }
                }
            }
        }
        Value::Array(a) => {
            for item in a {
                match emit_inline(item) {
                    Some(s) => out.push(format!("{}- {s}", pad(indent))),
                    None => {
                        let mut sub = Vec::new();
                        emit_into(item, indent + 2, &mut sub);
                        if let Some(first) = sub.first_mut() {
                            *first = format!("{}- {}", pad(indent), first[indent + 2..].to_owned());
                        }
                        out.extend(sub);
                    }
                }
            }
        }
        other => out.push(format!("{}{}", pad(indent), scalar(other))),
    }
}

/// Block-style YAML for a JSON tree, no trailing newline.
pub fn to_yaml(v: &Value) -> String {
    let mut out = Vec::new();
    emit_into(v, 0, &mut out);
    out.join("\n")
}
