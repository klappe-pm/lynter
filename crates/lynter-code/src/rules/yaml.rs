//! YAML data rules: `yaml-syntax`, `yaml-duplicate-keys`, `yaml-key-order`,
//! `yaml-schema`.
//!
//! Syntax and duplicate detection run on the `saphyr-parser` event stream.
//! Key ordering works on top-level blocks of the source text, so comments and
//! blank lines travel with their key. It only reorders the top level of a
//! single-document block mapping without aliases; nested mappings are left
//! alone, and anything else is skipped without a finding.

use super::{ext_in, key_permutation, opt_str, opt_strings};
use crate::rule::{report, CodeRule, SourceFile};
use crate::text::{line_col, offset_of};
use lynter_core::edit::Edit;
use lynter_core::Options;
use saphyr_parser::{Event, Parser, Span};
use serde_json::{json, Value};
use std::collections::HashMap;

fn is_yaml(f: &SourceFile) -> bool {
    ext_in(f, &["yaml", "yml"])
}

fn span_range(text: &str, span: &Span) -> (usize, usize) {
    let s = offset_of(text, span.start.line(), span.start.col());
    let e = offset_of(text, span.end.line(), span.end.col());
    (s, e.max(s))
}

pub struct YamlSyntax;

impl CodeRule for YamlSyntax {
    fn id(&self) -> &'static str {
        "yaml-syntax"
    }
    fn description(&self) -> &'static str {
        "Report YAML parse errors with line and column."
    }
    fn default_options(&self) -> Options {
        Options::new()
    }
    fn applies_to(&self, f: &SourceFile) -> bool {
        is_yaml(f)
    }
    fn check(&self, f: &SourceFile, _o: &Options) -> Vec<Edit> {
        for item in Parser::new_from_str(f.text) {
            if let Err(e) = item {
                let m = e.marker();
                let at = offset_of(f.text, m.line(), m.col());
                let end = f.text[at..]
                    .chars()
                    .next()
                    .filter(|c| *c != '\n')
                    .map_or(at, |c| at + c.len_utf8());
                return vec![report(
                    f.text,
                    at,
                    end,
                    format!("line {}, column {}: {}", m.line(), m.col() + 1, e.info()),
                )];
            }
        }
        Vec::new()
    }
}

enum Frame {
    Map {
        keys: HashMap<String, usize>,
        is_key: bool,
    },
    Seq,
}

pub struct YamlDuplicateKeys;

impl CodeRule for YamlDuplicateKeys {
    fn id(&self) -> &'static str {
        "yaml-duplicate-keys"
    }
    fn description(&self) -> &'static str {
        "Report repeated keys inside one YAML mapping."
    }
    fn default_options(&self) -> Options {
        Options::new()
    }
    fn applies_to(&self, f: &SourceFile) -> bool {
        is_yaml(f)
    }
    fn check(&self, f: &SourceFile, _o: &Options) -> Vec<Edit> {
        let mut stack: Vec<Frame> = Vec::new();
        let mut edits = Vec::new();
        for item in Parser::new_from_str(f.text) {
            let Ok((ev, span)) = item else { break };
            let node = matches!(
                ev,
                Event::Scalar(..)
                    | Event::Alias(_)
                    | Event::MappingStart(..)
                    | Event::SequenceStart(..)
            );
            if node {
                if let Some(Frame::Map { keys, is_key }) = stack.last_mut() {
                    let was_key = *is_key;
                    *is_key = !*is_key;
                    if let (true, Event::Scalar(k, ..)) = (was_key, &ev) {
                        if k != "<<" {
                            if let Some(first) = keys.get(k.as_ref()) {
                                let (s, e) = span_range(f.text, &span);
                                edits.push(report(
                                    f.text,
                                    s,
                                    e,
                                    format!("duplicate key {:?} (first on line {first})", k),
                                ));
                            } else {
                                keys.insert(k.to_string(), span.start.line());
                            }
                        }
                    }
                }
            }
            match ev {
                Event::MappingStart(..) => stack.push(Frame::Map {
                    keys: HashMap::new(),
                    is_key: true,
                }),
                Event::SequenceStart(..) => stack.push(Frame::Seq),
                Event::MappingEnd | Event::SequenceEnd => {
                    stack.pop();
                }
                _ => {}
            }
        }
        edits
    }
}

struct Slot {
    key: String,
    start: usize,
    end: usize,
    key_line_start: usize,
}

fn unquote(k: &str) -> String {
    let k = k.trim();
    for q in ['"', '\''] {
        if k.len() >= 2 && k.starts_with(q) && k.ends_with(q) {
            return k[1..k.len() - 1].to_owned();
        }
    }
    k.to_owned()
}

/// Key text of a column-0 `key:` line, or `None` when the line is not one.
fn key_of_line(line: &str) -> Option<String> {
    let first = line.chars().next()?;
    if matches!(
        first,
        ' ' | '\t' | '-' | '#' | '?' | '[' | '{' | '&' | '*' | '!' | '|' | '>' | '%' | '@' | '`'
    ) {
        return None;
    }
    if first == '"' || first == '\'' {
        let close = line[1..].find(first)? + 1;
        let rest = &line[close + 1..];
        return (rest.trim_start().starts_with(':')).then(|| line[1..close].to_owned());
    }
    for (i, c) in line.char_indices() {
        if c == ':' {
            let after = &line[i + 1..];
            if after.is_empty() || after.starts_with([' ', '\t', '\r']) {
                return Some(unquote(&line[..i]));
            }
        }
        if c == '#' && i > 0 && line[..i].ends_with([' ', '\t']) {
            return None;
        }
    }
    None
}

/// Top-level blocks of a single-document block mapping, or `None` when the
/// layout is not one this scanner handles safely.
fn top_level_slots(text: &str) -> Option<Vec<Slot>> {
    let mut lines: Vec<(usize, &str)> = Vec::new();
    let mut at = 0;
    for l in text.split_inclusive('\n') {
        lines.push((at, l.trim_end_matches(['\n', '\r'])));
        at += l.len();
    }
    let mut keys: Vec<(usize, String, usize)> = Vec::new();
    for (n, (off, l)) in lines.iter().enumerate() {
        if *l == "---" && keys.is_empty() && n == lines.iter().position(|x| x.1 == "---")? {
            continue;
        }
        if l.starts_with("---") || l.starts_with("...") {
            return None;
        }
        if l.is_empty() || l.trim().is_empty() || l.starts_with(' ') || l.starts_with('\t') {
            continue;
        }
        if l.starts_with('#') {
            continue;
        }
        let key = key_of_line(l)?;
        keys.push((n, key, *off));
    }
    if keys.is_empty() {
        return None;
    }
    let mut slots = Vec::new();
    for (i, (n, key, off)) in keys.iter().enumerate() {
        // Pull in the comment block directly above the key line.
        let floor = if i == 0 { 0 } else { keys[i - 1].0 + 1 };
        let mut first = *n;
        while first > floor && lines[first - 1].1.starts_with('#') {
            first -= 1;
        }
        if i == 0 {
            // Keep a leading `---` and detached header comments in place.
            let doc_start = lines.iter().position(|x| x.1 == "---").map_or(0, |p| p + 1);
            first = first.max(doc_start);
        }
        slots.push(Slot {
            key: key.clone(),
            start: lines[first].0,
            end: 0,
            key_line_start: *off,
        });
    }
    for i in 0..slots.len() {
        slots[i].end = slots.get(i + 1).map_or(text.len(), |s| s.start);
    }
    Some(slots)
}

fn has_alias(text: &str) -> bool {
    for item in Parser::new_from_str(text) {
        match item {
            Ok((Event::Alias(_), _)) => return true,
            Ok(_) => {}
            Err(_) => return true,
        }
    }
    false
}

pub struct YamlKeyOrder;

impl CodeRule for YamlKeyOrder {
    fn id(&self) -> &'static str {
        "yaml-key-order"
    }
    fn description(&self) -> &'static str {
        "Sort top-level YAML keys, priority keys first, keeping comments with their key."
    }
    fn default_options(&self) -> Options {
        let Value::Object(m) = json!({"priority": [], "rest": "alphabetical"}) else {
            unreachable!()
        };
        m
    }
    fn applies_to(&self, f: &SourceFile) -> bool {
        is_yaml(f)
    }
    fn check(&self, f: &SourceFile, o: &Options) -> Vec<Edit> {
        let Some(slots) = top_level_slots(f.text) else {
            return Vec::new();
        };
        if has_alias(f.text) {
            return Vec::new();
        }
        let keys: Vec<String> = slots.iter().map(|s| s.key.clone()).collect();
        let perm = key_permutation(
            &keys,
            &opt_strings(o, "priority"),
            opt_str(o, "rest", "alphabetical") != "keep",
        );
        let eol = if f.text.contains("\r\n") {
            "\r\n"
        } else {
            "\n"
        };
        let last = slots.len() - 1;
        let mut edits = Vec::new();
        for (slot, &src) in perm.iter().enumerate() {
            if slot == src {
                continue;
            }
            let mut body = f.text[slots[src].start..slots[src].end].to_owned();
            let src_open = !body.ends_with('\n');
            if src_open && slot != last {
                body.push_str(eol);
            } else if !src_open && slot == last && !f.text.ends_with('\n') {
                body.truncate(body.trim_end_matches(['\n', '\r']).len());
            }
            let msg = format!("key {:?} is not in the configured order", slots[src].key);
            edits.push(Edit::new(slots[slot].start, slots[slot].end, body, msg));
        }
        edits
    }
}

fn type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(n) if n.is_i64() || n.is_u64() => "integer",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn type_ok(want: &str, v: &Value) -> bool {
    let got = type_name(v);
    got == want || (want == "number" && got == "integer")
}

pub struct YamlSchema;

impl CodeRule for YamlSchema {
    fn id(&self) -> &'static str {
        "yaml-schema"
    }
    fn description(&self) -> &'static str {
        "Check required top-level keys and value types."
    }
    fn default_options(&self) -> Options {
        let Value::Object(m) = json!({"required": [], "types": {}}) else {
            unreachable!()
        };
        m
    }
    fn applies_to(&self, f: &SourceFile) -> bool {
        is_yaml(f)
    }
    fn check(&self, f: &SourceFile, o: &Options) -> Vec<Edit> {
        let Ok(doc) = serde_yaml_ng::from_str::<Value>(f.text) else {
            return Vec::new();
        };
        let first_line_end = f.text.find('\n').unwrap_or(f.text.len());
        let Value::Object(map) = doc else {
            return vec![report(
                f.text,
                0,
                first_line_end,
                "document must be a mapping".to_owned(),
            )];
        };
        let slots = top_level_slots(f.text).unwrap_or_default();
        let at = |key: &str| {
            slots
                .iter()
                .find(|s| s.key == key)
                .map_or((0, first_line_end), |s| {
                    let e = f.text[s.key_line_start..]
                        .find('\n')
                        .map_or(f.text.len(), |i| s.key_line_start + i);
                    (s.key_line_start, e)
                })
        };
        let mut edits = Vec::new();
        for key in opt_strings(o, "required") {
            if !map.contains_key(&key) {
                edits.push(report(
                    f.text,
                    0,
                    first_line_end,
                    format!("missing required key {key:?}"),
                ));
            }
        }
        if let Some(Value::Object(types)) = o.get("types") {
            for (key, want) in types {
                let (Some(want), Some(v)) = (want.as_str(), map.get(key)) else {
                    continue;
                };
                if !type_ok(want, v) {
                    let (s, e) = at(key);
                    let (line, _) = line_col(f.text, s);
                    edits.push(report(
                        f.text,
                        s,
                        e,
                        format!(
                            "key {key:?} on line {line} must be {want}, found {}",
                            type_name(v)
                        ),
                    ));
                }
            }
        }
        edits
    }
}
