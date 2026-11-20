//! `comment-tag`: a TODO, FIXME, HACK or XXX comment must name an owner or an
//! issue. Options: `tags` (list) and `pattern`, a template of alternatives
//! separated by `|`. In a template `{tag}` is the tag, `{owner}` is a name
//! such as `kevin` and `{issue}` is `#123` or `ABC-123`; every other character
//! is literal. The default accepts `TODO(name):`, `TODO #123:` and
//! `TODO(#123):`.

use super::kinds::{comment_lines, kinds};
use super::opt_strings;
use crate::rule::{report, CodeRule, SourceFile};
use lynter_core::edit::Edit;
use lynter_core::Options;
use serde_json::{json, Value};

pub struct CommentTag;

const DEFAULT_PATTERN: &str = "{tag}({owner}):|{tag} {issue}:|{tag}({issue}):";

fn owner_char(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '_' | '.' | '@' | '-' | '/')
}

/// Length of an issue reference at the start of `s`: `#123` or `ABC-123`.
fn issue_len(s: &str) -> Option<usize> {
    if let Some(r) = s.strip_prefix('#') {
        let d = r.bytes().take_while(u8::is_ascii_digit).count();
        return (d > 0).then_some(1 + d);
    }
    let letters = s
        .bytes()
        .take_while(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
        .count();
    let rest = s[letters..].strip_prefix('-')?;
    let d = rest.bytes().take_while(u8::is_ascii_digit).count();
    (letters > 0 && d > 0).then_some(letters + 1 + d)
}

/// True when `rest` matches one template alternative from its start.
fn matches_alt(alt: &str, tag: &str, mut rest: &str) -> bool {
    let mut t = alt;
    while !t.is_empty() {
        if let Some(r) = t.strip_prefix("{tag}") {
            let Some(after) = rest.strip_prefix(tag) else {
                return false;
            };
            rest = after;
            t = r;
        } else if let Some(r) = t.strip_prefix("{owner}") {
            let n: usize = rest
                .chars()
                .take_while(|&c| owner_char(c))
                .map(char::len_utf8)
                .sum();
            if n == 0 {
                return false;
            }
            rest = &rest[n..];
            t = r;
        } else if let Some(r) = t.strip_prefix("{issue}") {
            let Some(n) = issue_len(rest) else {
                return false;
            };
            rest = &rest[n..];
            t = r;
        } else {
            let c = t.chars().next().unwrap_or(' ');
            let Some(after) = rest.strip_prefix(c) else {
                return false;
            };
            rest = after;
            t = &t[c.len_utf8()..];
        }
    }
    true
}

impl CodeRule for CommentTag {
    fn id(&self) -> &'static str {
        "comment-tag"
    }
    fn description(&self) -> &'static str {
        "TODO, FIXME, HACK and XXX comments must carry an owner or an issue."
    }
    fn default_options(&self) -> Options {
        let Value::Object(m) = json!({
            "tags": ["TODO", "FIXME", "HACK", "XXX"],
            "pattern": DEFAULT_PATTERN,
        }) else {
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
        let tags = opt_strings(o, "tags");
        let pattern = super::opt_str(o, "pattern", DEFAULT_PATTERN);
        let alts: Vec<&str> = pattern.split('|').collect();
        let mut edits = Vec::new();
        let mut stack = vec![tree.root_node()];
        while let Some(n) = stack.pop() {
            if k.is_comment(n.kind()) {
                let base = n.start_byte();
                let text = &f.text[n.byte_range()];
                for line in comment_lines(text) {
                    let trimmed = line.text.trim_start();
                    let at = base + line.offset + (line.text.len() - trimmed.len());
                    for tag in &tags {
                        let Some(after) = trimmed.strip_prefix(tag.as_str()) else {
                            continue;
                        };
                        if after
                            .chars()
                            .next()
                            .is_some_and(|c| c.is_alphanumeric() || c == '_')
                        {
                            continue;
                        }
                        if !alts.iter().any(|a| matches_alt(a, tag, trimmed)) {
                            edits.push(report(
                                f.text,
                                at,
                                at + tag.len(),
                                format!("{tag} needs an owner or an issue, such as {tag}(name): or {tag} #123:"),
                            ));
                        }
                        break;
                    }
                }
                continue;
            }
            let mut c = n.walk();
            stack.extend(n.children(&mut c));
        }
        edits.sort_by_key(|e| e.start);
        edits
    }
}
