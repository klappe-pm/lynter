//! Heading rules: level increments, start-of-line markers, trailing punctuation.

use super::common::{atx, bool_opt, prot, scan, str_opt, Kind};
use crate::document::Document;
use crate::edit::Edit;
use crate::rule::{Options, Rule};
use regex::Regex;
use serde_json::{json, Value};
use std::sync::LazyLock;

fn opts(v: Value) -> Options {
    v.as_object().cloned().unwrap_or_default()
}

// ---- header-increment --------------------------------------------------------

pub struct HeaderIncrement;

struct Seen {
    orig: usize,
    new: usize,
}

impl Rule for HeaderIncrement {
    fn id(&self) -> &'static str {
        "header-increment"
    }

    fn description(&self) -> &'static str {
        "Heading levels may only go one level deeper than their parent heading."
    }

    fn default_options(&self) -> Options {
        opts(json!({"start-at-level-2": false}))
    }

    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit> {
        let start2 = bool_opt(options.get("start-at-level-2"), false);
        let sc = scan(doc, 2);
        let mut stack: Vec<Seen> = Vec::new();
        let mut out = Vec::new();
        for l in &sc.lines {
            if l.kind != Kind::Normal || l.depth != 0 {
                continue;
            }
            let Some(h) = atx(l.inner) else { continue };
            let level = h.level;
            while stack.last().is_some_and(|e| e.orig >= level) {
                stack.pop();
            }
            let new = match stack.last() {
                Some(p) => (p.new + 1).min(level),
                None if start2 => 2,
                None => level,
            };
            stack.push(Seen { orig: level, new });
            if new != level {
                let s = l.inner_start + h.hash_start;
                if !prot(doc, s, s + level) {
                    out.push(Edit::new(
                        s,
                        s + level,
                        "#".repeat(new),
                        "heading level adjusted",
                    ));
                }
            }
        }
        out
    }
}

// ---- headings-start-line -----------------------------------------------------

pub struct HeadingsStartLine;

fn columns(lead: &str) -> usize {
    lead.bytes()
        .fold(0, |c, b| if b == b'\t' { (c / 4 + 1) * 4 } else { c + 1 })
}

impl Rule for HeadingsStartLine {
    fn id(&self) -> &'static str {
        "headings-start-line"
    }

    fn description(&self) -> &'static str {
        "Remove whitespace before a heading marker so the line is a heading."
    }

    fn default_options(&self) -> Options {
        Options::new()
    }

    fn check(&self, doc: &Document, _options: &Options) -> Vec<Edit> {
        let sc = scan(doc, 2);
        let mut out = Vec::new();
        for l in &sc.lines {
            if l.kind != Kind::Normal {
                continue;
            }
            let n = l
                .inner
                .bytes()
                .take_while(|&b| b == b' ' || b == b'\t')
                .count();
            if n == 0 || columns(&l.inner[..n]) >= 4 {
                continue;
            }
            let rest = &l.inner[n..];
            let hashes = rest.bytes().take_while(|&b| b == b'#').count();
            if !(1..=6).contains(&hashes) || !rest[hashes..].starts_with(' ') {
                continue;
            }
            let (s, e) = (l.inner_start, l.inner_start + n);
            if !prot(doc, s, e) {
                out.push(Edit::new(s, e, "", "heading moved to line start"));
            }
        }
        out
    }
}

// ---- remove-trailing-punctuation-in-heading ----------------------------------

pub struct RemoveTrailingPunctuationInHeading;

static ENTITY_END: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"&(?:[A-Za-z][A-Za-z0-9]*|#[0-9]+|#[xX][0-9A-Fa-f]+);$").expect("regex")
});

impl Rule for RemoveTrailingPunctuationInHeading {
    fn id(&self) -> &'static str {
        "remove-trailing-punctuation-in-heading"
    }

    fn description(&self) -> &'static str {
        "Strip configured punctuation from the end of headings, keeping HTML entities."
    }

    fn default_options(&self) -> Options {
        opts(json!({"trailing-punctuation": ".,;:!\u{3002}\u{ff0c}\u{ff1b}\u{ff1a}\u{ff01}"}))
    }

    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit> {
        let set: Vec<char> = str_opt(
            options.get("trailing-punctuation"),
            ".,;:!\u{3002}\u{ff0c}\u{ff1b}\u{ff1a}\u{ff01}",
        )
        .chars()
        .collect();
        if set.is_empty() {
            return Vec::new();
        }
        let sc = scan(doc, 2);
        let mut out = Vec::new();
        for l in &sc.lines {
            if l.kind != Kind::Normal {
                continue;
            }
            let Some(h) = atx(l.inner) else { continue };
            let after = h.hash_start + h.level;
            let rest = &l.inner[after..];
            let lead = rest.len() - rest.trim_start_matches([' ', '\t']).len();
            let body = &rest[lead..];
            let trimmed = body.trim_end_matches([' ', '\t']);
            let hs = trimmed.trim_end_matches('#');
            let text_end =
                if hs.len() < trimmed.len() && (hs.is_empty() || hs.ends_with([' ', '\t'])) {
                    hs.trim_end_matches([' ', '\t']).len()
                } else {
                    trimmed.len()
                };
            let text = &body[..text_end];
            let mut end = text.len();
            while let Some(c) = text[..end].chars().next_back() {
                if !set.contains(&c) || (c == ';' && ENTITY_END.is_match(&text[..end])) {
                    break;
                }
                end -= c.len_utf8();
            }
            if end == text.len() || end == 0 {
                continue;
            }
            let base = l.inner_start + after + lead;
            let (s, e) = (base + end, base + text.len());
            if !prot(doc, s, e) {
                out.push(Edit::new(
                    s,
                    e,
                    "",
                    "trailing punctuation removed from heading",
                ));
            }
        }
        out
    }
}
