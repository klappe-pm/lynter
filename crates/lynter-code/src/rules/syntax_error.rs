//! `syntax-error`: every ERROR and MISSING node in the parse tree.

use crate::rule::{report, CodeRule, SourceFile};
use crate::text::line_col;
use lynter_core::edit::Edit;
use lynter_core::Options;
use tree_sitter::Node;

pub struct SyntaxError;

fn walk(node: Node, text: &str, edits: &mut Vec<Edit>) {
    if !node.has_error() && !node.is_missing() {
        return;
    }
    let (start, end) = (node.start_byte(), node.end_byte());
    if node.is_missing() || node.is_error() {
        let (line, col) = line_col(text, start);
        let what = if node.is_missing() {
            format!("missing {}", node.kind())
        } else {
            "syntax error".to_owned()
        };
        edits.push(report(
            text,
            start,
            end,
            format!("{what} at line {line}, column {col}"),
        ));
        if node.is_error() {
            return;
        }
    }
    let mut c = node.walk();
    for child in node.children(&mut c) {
        walk(child, text, edits);
    }
}

impl CodeRule for SyntaxError {
    fn id(&self) -> &'static str {
        "syntax-error"
    }
    fn description(&self) -> &'static str {
        "Report every ERROR and MISSING node of the tree-sitter parse, with line and column."
    }
    fn default_options(&self) -> Options {
        Options::new()
    }
    fn applies_to(&self, f: &SourceFile) -> bool {
        f.language.is_some()
    }
    fn check(&self, f: &SourceFile, _: &Options) -> Vec<Edit> {
        let Some(tree) = f.tree else {
            return Vec::new();
        };
        let mut edits = Vec::new();
        walk(tree.root_node(), f.text, &mut edits);
        edits
    }
}
