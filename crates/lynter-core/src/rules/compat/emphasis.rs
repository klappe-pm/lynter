//! Emphasis and strong delimiter rules. A line-local pass of the CommonMark
//! delimiter algorithm finds matched spans; the rules rewrite their delimiters.

use super::common::{prot, scan, str_opt, Kind, Scan};
use crate::document::Document;
use crate::edit::Edit;
use crate::rule::{Options, Rule};
use serde_json::{json, Value};

fn opts(v: Value) -> Options {
    v.as_object().cloned().unwrap_or_default()
}

/// One matched span: `len` delimiter characters at `open` and at `close`.
#[derive(Clone, Copy, Debug)]
pub struct Span {
    pub ch: char,
    pub len: usize,
    pub open: usize,
    pub close: usize,
}

struct Delim {
    ch: char,
    start: usize,
    orig: usize,
    count: usize,
    can_open: bool,
    can_close: bool,
    active: bool,
}

fn is_ws(c: Option<char>) -> bool {
    c.is_none_or(char::is_whitespace)
}

fn is_punct(c: Option<char>) -> bool {
    c.is_some_and(|c| !c.is_alphanumeric() && !c.is_whitespace())
}

fn runs(doc: &Document, text: &str, base: usize) -> Vec<Delim> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < text.len() {
        let Some(c) = text[i..].chars().next() else {
            break;
        };
        if c == '\\' {
            i += 1;
            if let Some(n) = text[i..].chars().next() {
                i += n.len_utf8();
            }
            continue;
        }
        if (c != '*' && c != '_') || prot(doc, base + i, base + i + 1) {
            i += c.len_utf8();
            continue;
        }
        let mut j = i;
        while text[j..].starts_with(c) && !prot(doc, base + j, base + j + 1) {
            j += 1;
        }
        let prev = text[..i].chars().next_back();
        let next = text[j..].chars().next();
        let left = !is_ws(next) && (!is_punct(next) || is_ws(prev) || is_punct(prev));
        let right = !is_ws(prev) && (!is_punct(prev) || is_ws(next) || is_punct(next));
        let (can_open, can_close) = if c == '*' {
            (left, right)
        } else {
            (
                left && (!right || is_punct(prev)),
                right && (!left || is_punct(next)),
            )
        };
        out.push(Delim {
            ch: c,
            start: base + i,
            orig: j - i,
            count: j - i,
            can_open,
            can_close,
            active: true,
        });
        i = j;
    }
    out
}

fn match_line(mut ds: Vec<Delim>, out: &mut Vec<Span>) {
    let mut ci = 0;
    while ci < ds.len() {
        if !ds[ci].active || !ds[ci].can_close || ds[ci].count == 0 {
            ci += 1;
            continue;
        }
        let found = (0..ci).rev().find(|&oi| {
            let (o, c) = (&ds[oi], &ds[ci]);
            o.active
                && o.count > 0
                && o.ch == c.ch
                && o.can_open
                && !((o.can_close || c.can_open)
                    && (o.orig + c.orig) % 3 == 0
                    && !(o.orig % 3 == 0 && c.orig % 3 == 0))
        });
        let Some(oi) = found else {
            if !ds[ci].can_open {
                ds[ci].active = false;
            }
            ci += 1;
            continue;
        };
        let len = if ds[oi].count >= 2 && ds[ci].count >= 2 {
            2
        } else {
            1
        };
        out.push(Span {
            ch: ds[oi].ch,
            len,
            open: ds[oi].start + ds[oi].count - len,
            close: ds[ci].start + (ds[ci].orig - ds[ci].count),
        });
        ds[oi].count -= len;
        ds[ci].count -= len;
        for d in &mut ds[oi + 1..ci] {
            d.active = false;
        }
        if ds[oi].count == 0 {
            ds[oi].active = false;
        }
        if ds[ci].count == 0 {
            ci += 1;
        }
    }
}

/// Every matched span in the document, in order of the opening delimiter.
pub fn spans(doc: &Document, sc: &Scan) -> Vec<Span> {
    let mut out = Vec::new();
    for l in &sc.lines {
        if l.kind != Kind::Normal {
            continue;
        }
        match_line(runs(doc, l.text, l.start), &mut out);
    }
    out.sort_by_key(|s| s.open);
    out
}

fn style_edits(doc: &Document, options: &Options, len: usize, message: &str) -> Vec<Edit> {
    let sc = scan(doc, 2);
    let all: Vec<Span> = spans(doc, &sc)
        .into_iter()
        .filter(|s| s.len == len)
        .collect();
    let target = match str_opt(options.get("style"), "consistent") {
        "asterisk" => Some('*'),
        "underscore" => Some('_'),
        _ => all.first().map(|s| s.ch),
    };
    let Some(target) = target else {
        return Vec::new();
    };
    let text = doc.text();
    let mut out = Vec::new();
    for s in all.iter().filter(|s| s.ch != target) {
        if target == '_' {
            let before = text[..s.open].chars().next_back();
            let after = text[s.close + len..].chars().next();
            if before.is_some_and(char::is_alphanumeric) || after.is_some_and(char::is_alphanumeric)
            {
                continue;
            }
        }
        let rep = target.to_string().repeat(len);
        out.push(Edit::new(s.open, s.open + len, rep.clone(), message));
        out.push(Edit::new(s.close, s.close + len, rep, message));
    }
    out
}

pub struct EmphasisStyle;

impl Rule for EmphasisStyle {
    fn id(&self) -> &'static str {
        "emphasis-style"
    }

    fn description(&self) -> &'static str {
        "Make emphasis delimiters consistently asterisks or underscores."
    }

    fn default_options(&self) -> Options {
        opts(json!({"style": "consistent"}))
    }

    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit> {
        style_edits(doc, options, 1, "emphasis delimiter normalized")
    }
}

pub struct StrongStyle;

impl Rule for StrongStyle {
    fn id(&self) -> &'static str {
        "strong-style"
    }

    fn description(&self) -> &'static str {
        "Make strong delimiters consistently double asterisks or double underscores."
    }

    fn default_options(&self) -> Options {
        opts(json!({"style": "consistent"}))
    }

    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit> {
        style_edits(doc, options, 2, "strong delimiter normalized")
    }
}
