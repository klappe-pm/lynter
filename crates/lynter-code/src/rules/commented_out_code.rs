//! `commented-out-code`: a comment, or a run of consecutive own-line line
//! comments, whose text re-parses in the same language with no ERROR nodes and
//! at least one statement-level node. Doc comments and short comments are
//! ignored. Option: `min-length` (default 12), the shortest comment text that
//! is examined. Text with no code punctuation is ignored too, so a lone word
//! that happens to parse as an expression is not reported.

use super::kinds::{comment_lines, is_line_comment, kinds, Kinds};
use crate::language::Language;
use crate::rule::{report, CodeRule, SourceFile};
use lynter_core::edit::Edit;
use lynter_core::Options;
use serde_json::{json, Value};
use tree_sitter::Node;

pub struct CommentedOutCode;

struct Item {
    start: usize,
    end: usize,
    row: usize,
    end_row: usize,
    own_line: bool,
    line: bool,
    lines: Vec<String>,
}

fn dedent(lines: &[String]) -> String {
    let indent = lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);
    lines
        .iter()
        .map(|l| l.get(indent..).unwrap_or(l.trim_start()))
        .collect::<Vec<_>>()
        .join("\n")
}

/// True when a statement-level node lies inside `[lo, hi)`.
fn has_statement(k: &Kinds, root: Node, lo: usize, hi: usize) -> bool {
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        if n.start_byte() >= lo && n.end_byte() <= hi && k.is_statement(n.kind()) {
            return true;
        }
        let mut c = n.walk();
        stack.extend(n.children(&mut c));
    }
    false
}

/// Contexts a comment's text is tried in: the text alone first, then inside a
/// wrapper for languages that only allow statements or members in a body.
fn contexts(lang: Language) -> &'static [(&'static str, &'static str)] {
    match lang {
        Language::Go => &[("package p\n", ""), ("package p\nfunc w() {\n", "\n}")],
        Language::Java => &[("class W {\n", "\n}"), ("class W { void w() {\n", "\n}}")],
        Language::CSharp => &[("", ""), ("class W {\n", "\n}")],
        _ => &[("", "")],
    }
}

fn looks_like_code(lang: Language, text: &str, min: usize) -> bool {
    let t = text.trim();
    if t.chars().count() < min || !t.contains(['(', ')', '{', '}', '[', ']', ';', '=']) {
        return false;
    }
    contexts(lang).iter().any(|(pre, post)| {
        let src = format!("{pre}{t}{post}");
        let Some(tree) = lang.parse(&src) else {
            return false;
        };
        let root = tree.root_node();
        !root.has_error() && has_statement(kinds(lang), root, pre.len(), pre.len() + t.len())
    })
}

impl CodeRule for CommentedOutCode {
    fn id(&self) -> &'static str {
        "commented-out-code"
    }
    fn description(&self) -> &'static str {
        "Comments whose text parses as code in the same language are commented-out code."
    }
    fn default_options(&self) -> Options {
        let Value::Object(m) = json!({ "min-length": 12 }) else {
            unreachable!()
        };
        m
    }
    fn applies_to(&self, f: &SourceFile) -> bool {
        f.language.is_some()
    }
    fn check(&self, f: &SourceFile, o: &Options) -> Vec<Edit> {
        let (Some(lang), Some(tree)) = (f.language, f.tree) else {
            return Vec::new();
        };
        let k = kinds(lang);
        let min = o
            .get("min-length")
            .and_then(Value::as_u64)
            .map_or(12, |n| n as usize);
        let mut items: Vec<Item> = Vec::new();
        let mut stack = vec![tree.root_node()];
        while let Some(n) = stack.pop() {
            if k.is_comment(n.kind()) {
                let text = &f.text[n.byte_range()];
                let before = &f.text
                    [f.text[..n.start_byte()].rfind('\n').map_or(0, |i| i + 1)..n.start_byte()];
                items.push(Item {
                    start: n.start_byte(),
                    end: n.end_byte(),
                    row: n.start_position().row,
                    end_row: n.end_position().row,
                    own_line: before.trim().is_empty(),
                    line: is_line_comment(text),
                    lines: if k.is_doc_comment(text) {
                        Vec::new()
                    } else {
                        comment_lines(text).into_iter().map(|l| l.text).collect()
                    },
                });
                continue;
            }
            let mut c = n.walk();
            stack.extend(n.children(&mut c));
        }
        items.sort_by_key(|i| i.start);
        let mut edits = Vec::new();
        let mut i = 0;
        while i < items.len() {
            let mut j = i + 1;
            if items[i].line && items[i].own_line {
                while j < items.len()
                    && items[j].line
                    && items[j].own_line
                    && items[j].row == items[j - 1].end_row + 1
                {
                    j += 1;
                }
            }
            let run = &items[i..j];
            let doc = run.iter().any(|c| c.lines.is_empty());
            if !doc {
                let lines: Vec<String> = run.iter().flat_map(|c| c.lines.iter().cloned()).collect();
                if looks_like_code(lang, &dedent(&lines), min) {
                    let (s, e) = (run[0].start, run[run.len() - 1].end);
                    edits.push(report(f.text, s, e, "comment contains commented-out code"));
                }
            }
            i = j;
        }
        edits
    }
}
