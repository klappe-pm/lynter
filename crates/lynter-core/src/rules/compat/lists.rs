//! List rules: ordered numbering, bullet style, empty items, and spacing
//! after markers.

use super::common::{
    is_hr, is_paragraph_text, list_marker, prot, scan, str_opt, Info, Kind, Marker,
};
use crate::document::Document;
use crate::edit::Edit;
use crate::rule::{Options, Rule};
use serde_json::{json, Value};

fn opts(v: Value) -> Options {
    v.as_object().cloned().unwrap_or_default()
}

/// Indentation width: a space is one column, a tab is two.
fn indent_width(s: &str) -> usize {
    s.bytes()
        .map(|b| match b {
            b' ' => 1,
            b'\t' => 2,
            _ => 0,
        })
        .sum()
}

/// A checkbox at the start of `rest`: `[ ]`, `[x]` or `[X]`. Returns its length.
fn checkbox(rest: &str) -> Option<usize> {
    let b = rest.as_bytes();
    (b.len() >= 3
        && b[0] == b'['
        && matches!(b[1], b' ' | b'x' | b'X')
        && b[2] == b']'
        && (b.len() == 3 || b[3] == b' ' || b[3] == b'\t'))
        .then_some(3)
}

/// Any single-character task marker, for the bullet style rule.
fn is_task(rest: &str) -> bool {
    let mut it = rest.chars();
    it.next() == Some('[')
        && it.next().is_some_and(|c| c != '[' && c != ']')
        && it.next() == Some(']')
        && it.next().is_none_or(|c| c == ' ' || c == '\t')
}

// ---- ordered-list-style --------------------------------------------------

pub struct OrderedListStyle;

struct Level {
    depth: usize,
    indent: usize,
    last: u64,
}

impl Rule for OrderedListStyle {
    fn id(&self) -> &'static str {
        "ordered-list-style"
    }

    fn description(&self) -> &'static str {
        "Normalize ordered list numbering (ascending or lazy) and the character after the number."
    }

    fn default_options(&self) -> Options {
        opts(json!({"number-style": "ascending", "end-style": "."}))
    }

    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit> {
        let lazy = str_opt(options.get("number-style"), "ascending") == "lazy";
        let end = if str_opt(options.get("end-style"), ".") == ")" {
            ')'
        } else {
            '.'
        };
        let sc = scan(doc, 2);
        let mut stack: Vec<Level> = Vec::new();
        let mut out = Vec::new();
        for l in &sc.lines {
            match l.kind {
                Kind::Blank
                | Kind::Skip
                | Kind::FenceBody
                | Kind::FenceClose
                | Kind::MathBody
                | Kind::MathClose
                | Kind::TableDelim
                | Kind::TableRow => continue,
                _ => {}
            }
            let m = (l.kind == Kind::Normal)
                .then(|| list_marker(l.inner).filter(|m| m.ordered))
                .flatten();
            let Some(m) = m else {
                let w = indent_width(&l.inner[..l.inner.len() - l.inner.trim_start().len()]);
                while stack
                    .last()
                    .is_some_and(|t| t.depth > l.depth || (t.depth == l.depth && t.indent >= w))
                {
                    stack.pop();
                }
                continue;
            };
            let w = indent_width(&l.inner[..m.marker_start]);
            while stack
                .last()
                .is_some_and(|t| t.depth > l.depth || (t.depth == l.depth && t.indent > w))
            {
                stack.pop();
            }
            let number = match stack.last_mut() {
                Some(t) if t.depth == l.depth && t.indent == w => {
                    t.last = if lazy {
                        1
                    } else {
                        (t.last + 1).min(999_999_999)
                    };
                    t.last
                }
                _ => {
                    let n = if lazy { 1 } else { m.number };
                    stack.push(Level {
                        depth: l.depth,
                        indent: w,
                        last: n,
                    });
                    n
                }
            };
            let s = l.inner_start + m.digits.0;
            let e = l.inner_start + m.marker_end;
            let new = format!("{number}{end}");
            if doc.text()[s..e] != new && !prot(doc, s, e) {
                out.push(Edit::new(s, e, new, "ordered list item renumbered"));
            }
        }
        out
    }
}

// ---- unordered-list-style ------------------------------------------------

pub struct UnorderedListStyle;

fn bullet_item(l: &Info) -> Option<Marker> {
    if l.kind != Kind::Normal || is_hr(l.inner) {
        return None;
    }
    let m = list_marker(l.inner).filter(|m| !m.ordered)?;
    if m.marker_end >= l.inner.len() || is_task(&l.inner[m.ws_end..]) {
        return None;
    }
    Some(m)
}

impl Rule for UnorderedListStyle {
    fn id(&self) -> &'static str {
        "unordered-list-style"
    }

    fn description(&self) -> &'static str {
        "Use one bullet marker for all unordered list items."
    }

    fn default_options(&self) -> Options {
        opts(json!({"style": "consistent"}))
    }

    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit> {
        let sc = scan(doc, 2);
        let target = match str_opt(options.get("style"), "consistent") {
            "-" => Some('-'),
            "*" => Some('*'),
            "+" => Some('+'),
            _ => sc
                .lines
                .iter()
                .find_map(|l| bullet_item(l).map(|m| m.bullet)),
        };
        let Some(target) = target else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for l in &sc.lines {
            let Some(m) = bullet_item(l) else { continue };
            if m.bullet == target {
                continue;
            }
            let s = l.inner_start + m.marker_start;
            if !prot(doc, s, s + 1) {
                out.push(Edit::new(
                    s,
                    s + 1,
                    target.to_string(),
                    "bullet marker normalized",
                ));
            }
        }
        out
    }
}

// ---- remove-empty-list-markers ---------------------------------------------

pub struct RemoveEmptyListMarkers;

impl Rule for RemoveEmptyListMarkers {
    fn id(&self) -> &'static str {
        "remove-empty-list-markers"
    }

    fn description(&self) -> &'static str {
        "Delete list items that have a marker but no content."
    }

    fn default_options(&self) -> Options {
        Options::new()
    }

    fn check(&self, doc: &Document, _options: &Options) -> Vec<Edit> {
        let sc = scan(doc, 2);
        let mut out = Vec::new();
        for (i, l) in sc.lines.iter().enumerate() {
            if l.kind != Kind::Normal {
                continue;
            }
            let Some(m) = list_marker(l.inner) else {
                continue;
            };
            let rest = &l.inner[m.ws_end..];
            let boxed = checkbox(rest);
            let tail = &rest[boxed.unwrap_or(0)..];
            if !tail.trim_matches([' ', '\t']).is_empty() {
                continue;
            }
            if m.bullet == '-'
                && boxed.is_none()
                && i > 0
                && sc.lines[i - 1].depth == l.depth
                && is_paragraph_text(&sc.lines[i - 1])
            {
                continue;
            }
            if !prot(doc, l.start, l.next) {
                out.push(Edit::new(l.start, l.next, "", "empty list item removed"));
            }
        }
        out
    }
}

// ---- space-after-list-markers ----------------------------------------------

pub struct SpaceAfterListMarkers;

impl Rule for SpaceAfterListMarkers {
    fn id(&self) -> &'static str {
        "space-after-list-markers"
    }

    fn description(&self) -> &'static str {
        "Use exactly one space after list markers and after checkboxes."
    }

    fn default_options(&self) -> Options {
        Options::new()
    }

    fn check(&self, doc: &Document, _options: &Options) -> Vec<Edit> {
        let sc = scan(doc, 2);
        let text = doc.text();
        let mut out = Vec::new();
        for l in &sc.lines {
            if l.kind != Kind::Normal || is_hr(l.inner) {
                continue;
            }
            let Some(m) = list_marker(l.inner) else {
                continue;
            };
            let rest = &l.inner[m.ws_end..];
            if rest.is_empty() {
                continue;
            }
            let mut fix = |from: usize, to: usize| {
                let (s, e) = (l.inner_start + from, l.inner_start + to);
                if &text[s..e] != " " && !prot(doc, s, e) {
                    out.push(Edit::new(s, e, " ", "one space after list marker"));
                }
            };
            fix(m.marker_end, m.ws_end);
            if let Some(n) = checkbox(rest) {
                let after = &rest[n..];
                let ws = after.len() - after.trim_start_matches([' ', '\t']).len();
                if ws > 0 && ws < after.len() {
                    fix(m.ws_end + n, m.ws_end + n + ws);
                }
            }
        }
        out
    }
}
