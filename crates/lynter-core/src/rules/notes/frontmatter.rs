//! A minimal frontmatter reader: key line positions plus parsed YAML.

use crate::zones::line_spans;
use serde_json::{Map, Value};

pub struct Frontmatter {
    /// Byte range of the opening `---` line.
    pub open: (usize, usize),
    /// Top-level keys with the byte range of their line.
    pub keys: Vec<(String, usize, usize)>,
    /// Parsed mapping; `None` when the YAML is invalid.
    pub value: Option<Map<String, Value>>,
}

impl Frontmatter {
    pub fn key_range(&self, key: &str) -> Option<(usize, usize)> {
        self.keys.iter().find(|k| k.0 == key).map(|k| (k.1, k.2))
    }
}

pub fn parse(text: &str) -> Option<Frontmatter> {
    let lines = line_spans(text);
    let first = lines.first()?;
    if first.text(text).trim_end() != "---" {
        return None;
    }
    let close = (1..lines.len()).find(|&i| {
        let t = lines[i].text(text).trim_end();
        t == "---" || t == "..."
    })?;
    let mut keys = Vec::new();
    for l in &lines[1..close] {
        let t = l.text(text);
        if t.starts_with([' ', '\t', '-', '#']) {
            continue;
        }
        if let Some(i) = t.find(':') {
            let after = &t[i + 1..];
            let key = t[..i].trim().trim_matches(['"', '\'']);
            if !key.is_empty() && (after.is_empty() || after.starts_with([' ', '\t', '\r'])) {
                keys.push((key.to_owned(), l.start, l.end));
            }
        }
    }
    let yaml = &text[lines[1].start.min(text.len())..lines[close].start];
    let value = match serde_yaml_ng::from_str::<Value>(yaml) {
        Ok(Value::Object(m)) => Some(m),
        Ok(Value::Null) => Some(Map::new()),
        _ => None,
    };
    Some(Frontmatter {
        open: (first.start, first.end),
        keys,
        value,
    })
}
