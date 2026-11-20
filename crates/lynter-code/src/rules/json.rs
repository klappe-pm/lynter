//! JSON data rules: `json-syntax`, `json-key-order`, `json-duplicate-keys`.

use super::json_parse::{parse, Member, Node};
use super::{ext_in, key_permutation, opt_bool, opt_str, opt_strings};
use crate::rule::{report, CodeRule, SourceFile};
use crate::text::line_col;
use lynter_core::edit::Edit;
use lynter_core::Options;
use serde_json::{json, Value};

fn is_json(f: &SourceFile) -> bool {
    ext_in(f, &["json"])
}

pub struct JsonSyntax;

impl CodeRule for JsonSyntax {
    fn id(&self) -> &'static str {
        "json-syntax"
    }
    fn description(&self) -> &'static str {
        "Report JSON parse errors with line and column."
    }
    fn default_options(&self) -> Options {
        Options::new()
    }
    fn applies_to(&self, f: &SourceFile) -> bool {
        is_json(f)
    }
    fn check(&self, f: &SourceFile, _o: &Options) -> Vec<Edit> {
        match parse(f.text) {
            Ok(_) => Vec::new(),
            Err(e) => {
                let (line, col) = line_col(f.text, e.offset);
                let end = f.text[e.offset..]
                    .chars()
                    .next()
                    .map_or(e.offset, |c| e.offset + c.len_utf8());
                vec![report(
                    f.text,
                    e.offset,
                    end,
                    format!("line {line}, column {col}: {}", e.message),
                )]
            }
        }
    }
}

fn walk_objects<'a>(node: &'a Node, deep: bool, out: &mut Vec<&'a [Member]>) {
    match node {
        Node::Object { members } => {
            out.push(members);
            if deep {
                for m in members {
                    walk_objects(&m.value, deep, out);
                }
            }
        }
        Node::Array { items } => {
            if deep {
                for i in items {
                    walk_objects(i, deep, out);
                }
            }
        }
        Node::Scalar => {}
    }
}

pub struct JsonDuplicateKeys;

impl CodeRule for JsonDuplicateKeys {
    fn id(&self) -> &'static str {
        "json-duplicate-keys"
    }
    fn description(&self) -> &'static str {
        "Report repeated keys inside one JSON object."
    }
    fn default_options(&self) -> Options {
        Options::new()
    }
    fn applies_to(&self, f: &SourceFile) -> bool {
        is_json(f)
    }
    fn check(&self, f: &SourceFile, _o: &Options) -> Vec<Edit> {
        let Ok(root) = parse(f.text) else {
            return Vec::new();
        };
        let mut objects = Vec::new();
        walk_objects(&root, true, &mut objects);
        let mut edits = Vec::new();
        for members in objects {
            for (i, m) in members.iter().enumerate() {
                if let Some(first) = members[..i].iter().find(|p| p.key == m.key) {
                    let (line, _) = line_col(f.text, first.key_start);
                    edits.push(report(
                        f.text,
                        m.key_start,
                        m.key_end,
                        format!("duplicate key {:?} (first on line {line})", m.key),
                    ));
                }
            }
        }
        edits
    }
}

pub struct JsonKeyOrder;

impl CodeRule for JsonKeyOrder {
    fn id(&self) -> &'static str {
        "json-key-order"
    }
    fn description(&self) -> &'static str {
        "Sort JSON object keys, priority keys first."
    }
    fn default_options(&self) -> Options {
        let Value::Object(m) = json!({"priority": [], "rest": "alphabetical", "deep": false})
        else {
            unreachable!()
        };
        m
    }
    fn applies_to(&self, f: &SourceFile) -> bool {
        is_json(f)
    }
    fn check(&self, f: &SourceFile, o: &Options) -> Vec<Edit> {
        let Ok(root) = parse(f.text) else {
            return Vec::new();
        };
        let priority = opt_strings(o, "priority");
        let alpha = opt_str(o, "rest", "alphabetical") != "keep";
        let deep = opt_bool(o, "deep", false);
        let mut objects = Vec::new();
        walk_objects(&root, deep, &mut objects);
        let mut edits = Vec::new();
        for members in objects {
            let keys: Vec<String> = members.iter().map(|m| m.key.clone()).collect();
            let perm = key_permutation(&keys, &priority, alpha);
            for (slot, &src) in perm.iter().enumerate() {
                if slot != src {
                    let (a, b) = (&members[slot], &members[src]);
                    edits.push(Edit::new(
                        a.key_start,
                        a.end,
                        &f.text[b.key_start..b.end],
                        "keys are not in the configured order",
                    ));
                }
            }
        }
        edits
    }
}
