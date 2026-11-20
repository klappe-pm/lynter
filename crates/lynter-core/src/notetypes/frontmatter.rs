//! Line-preserving frontmatter access. The document is split into lines once;
//! the body is carried through verbatim, so a file the rules do not change is
//! returned byte-identical.

use crate::rules::content::util::{js_pattern, js_trim};
use regex::Regex;
use serde_json::{Map, Value};
use std::sync::LazyLock;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FrontmatterDoc {
    pub has: bool,
    pub yaml_lines: Vec<String>,
    pub body_lines: Vec<String>,
}

pub fn split_document(text: &str) -> FrontmatterDoc {
    let lines: Vec<&str> = text.split('\n').collect();
    if lines[0] == "---" {
        if let Some(i) = (1..lines.len()).find(|&i| lines[i] == "---") {
            return FrontmatterDoc {
                has: true,
                yaml_lines: lines[1..i].iter().map(|s| (*s).to_owned()).collect(),
                body_lines: lines[i + 1..].iter().map(|s| (*s).to_owned()).collect(),
            };
        }
    }
    FrontmatterDoc {
        has: false,
        yaml_lines: Vec::new(),
        body_lines: lines.iter().map(|s| (*s).to_owned()).collect(),
    }
}

pub fn join_document(doc: &FrontmatterDoc) -> String {
    if !doc.has {
        return doc.body_lines.join("\n");
    }
    let mut all: Vec<&str> = vec!["---"];
    all.extend(doc.yaml_lines.iter().map(String::as_str));
    all.push("---");
    all.extend(doc.body_lines.iter().map(String::as_str));
    all.join("\n")
}

/// One top-level mapping entry with its exact source lines. `key: None` holds
/// loose lines before the first key (comments, blanks).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct YamlEntry {
    pub key: Option<String>,
    pub lines: Vec<String>,
}

static KEY_LINE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&js_pattern(
        r#"^(?:"([^"]+)"|'([^']+)'|([^\s#:-](?:[^:]*[^\s:])?))\s*:(?:\s|$)"#,
    ))
    .expect("regex")
});

pub fn split_entries(yaml_lines: &[String]) -> Vec<YamlEntry> {
    let mut entries: Vec<YamlEntry> = Vec::new();
    let mut current: Option<YamlEntry> = None;
    for line in yaml_lines {
        let m = if line.contains(':') {
            KEY_LINE.captures(line)
        } else {
            None
        };
        if let Some(m) = m {
            if let Some(c) = current.take() {
                entries.push(c);
            }
            let key = m
                .get(1)
                .or_else(|| m.get(2))
                .map(|g| g.as_str().to_owned())
                .or_else(|| m.get(3).map(|g| js_trim(g.as_str()).to_owned()));
            current = Some(YamlEntry {
                key,
                lines: vec![line.clone()],
            });
        } else if let Some(c) = current.as_mut() {
            c.lines.push(line.clone());
        } else {
            current = Some(YamlEntry {
                key: None,
                lines: vec![line.clone()],
            });
        }
    }
    if let Some(c) = current {
        entries.push(c);
    }
    entries
}

pub fn entries_to_lines(entries: &[YamlEntry]) -> Vec<String> {
    entries.iter().flat_map(|e| e.lines.clone()).collect()
}

/// Parse the frontmatter mapping. `{}` when there is none, `None` when the
/// YAML is invalid or not a mapping, so rules can bail safely.
pub fn parse_frontmatter_data(yaml_lines: &[String]) -> Option<Map<String, Value>> {
    if yaml_lines.is_empty() {
        return Some(Map::new());
    }
    match crate::config::yaml_to_json(&yaml_lines.join("\n")).ok()? {
        Value::Null => Some(Map::new()),
        Value::Object(m) => Some(m),
        _ => None,
    }
}

/// Render one entry in the house style: empty arrays as `key:` with no `[]`,
/// non-empty arrays as two-space-indented block sequences.
pub fn format_entry_lines(key: &str, value: &Value) -> Vec<String> {
    match value {
        Value::Null => vec![format!("{key}:")],
        Value::Array(a) if a.is_empty() => vec![format!("{key}:")],
        Value::Array(a) => {
            let mut out = vec![format!("{key}:")];
            out.extend(
                a.iter()
                    .map(|i| format!("  - {}", super::format::scalar(i))),
            );
            out
        }
        v => vec![format!("{key}: {}", super::format::scalar(v))],
    }
}
