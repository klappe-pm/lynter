//! `max-nesting-depth` and `max-function-lines`: size limits per function,
//! measured on the parse tree with the node kinds of `kinds.rs`.

use super::kinds::{kinds, Kinds};
use crate::rule::{report, CodeRule, SourceFile};
use lynter_core::edit::Edit;
use lynter_core::Options;
use serde_json::{json, Value};
use tree_sitter::Node;

pub struct MaxNestingDepth;
pub struct MaxFunctionLines;

fn max_option(o: &Options, default: usize) -> usize {
    o.get("max")
        .and_then(Value::as_u64)
        .map_or(default, |n| n as usize)
}

fn default_max(n: u64) -> Options {
    let Value::Object(m) = json!({ "max": n }) else {
        unreachable!()
    };
    m
}

fn depth_walk(k: &Kinds, node: Node, depth: usize, max: usize, text: &str, out: &mut Vec<Edit>) {
    let mut c = node.walk();
    for child in node.children(&mut c) {
        let counts =
            k.is_block(child.kind()) && !node.parent().is_none_or(|_| k.owns_body(node.kind()));
        let d = if counts { depth + 1 } else { depth };
        if counts && d == max + 1 {
            let (start, end) = (child.start_byte(), child.end_byte());
            let line_end = text[start..end].find('\n').map_or(end, |i| start + i);
            out.push(report(
                text,
                start,
                line_end,
                format!("nesting depth {d} exceeds the maximum of {max}"),
            ));
        }
        depth_walk(k, child, d, max, text, out);
    }
}

impl CodeRule for MaxNestingDepth {
    fn id(&self) -> &'static str {
        "max-nesting-depth"
    }
    fn description(&self) -> &'static str {
        "Block nesting depth, or subquery and CASE depth in SQL, stays within `max` (default 4)."
    }
    fn default_options(&self) -> Options {
        default_max(4)
    }
    fn applies_to(&self, f: &SourceFile) -> bool {
        f.language.is_some()
    }
    fn check(&self, f: &SourceFile, o: &Options) -> Vec<Edit> {
        let (Some(lang), Some(tree)) = (f.language, f.tree) else {
            return Vec::new();
        };
        let mut edits = Vec::new();
        depth_walk(
            kinds(lang),
            tree.root_node(),
            0,
            max_option(o, 4),
            f.text,
            &mut edits,
        );
        edits
    }
}

fn lines_walk(k: &Kinds, node: Node, max: usize, text: &str, out: &mut Vec<Edit>) {
    if k.is_function(node.kind()) {
        let lines = node.end_position().row - node.start_position().row + 1;
        if lines > max {
            let (start, end) = (node.start_byte(), node.end_byte());
            let line_end = text[start..end].find('\n').map_or(end, |i| start + i);
            out.push(report(
                text,
                start,
                line_end,
                format!(
                    "{} has {lines} lines, more than the maximum of {max}",
                    node.kind().replace('_', " ")
                ),
            ));
        }
        // SQL statements nest (CTEs, function bodies); count the outer one only.
        if node.kind() == "statement" {
            return;
        }
    }
    let mut c = node.walk();
    for child in node.children(&mut c) {
        lines_walk(k, child, max, text, out);
    }
}

impl CodeRule for MaxFunctionLines {
    fn id(&self) -> &'static str {
        "max-function-lines"
    }
    fn description(&self) -> &'static str {
        "Lines per function or method, or per SQL statement, stay within `max` (default 60)."
    }
    fn default_options(&self) -> Options {
        default_max(60)
    }
    fn applies_to(&self, f: &SourceFile) -> bool {
        f.language.is_some()
    }
    fn check(&self, f: &SourceFile, o: &Options) -> Vec<Edit> {
        let (Some(lang), Some(tree)) = (f.language, f.tree) else {
            return Vec::new();
        };
        let mut edits = Vec::new();
        lines_walk(
            kinds(lang),
            tree.root_node(),
            max_option(o, 60),
            f.text,
            &mut edits,
        );
        edits
    }
}
