//! Inline and per-line text rules: trailing and repeated spaces, ellipsis,
//! fence languages, bare URLs, and quote style.

use super::common::{fence_info, inside_except_url, prot, prot_except_url, scan, str_opt, Kind};
use crate::document::Document;
use crate::edit::Edit;
use crate::rule::{Options, Rule};
use crate::zones::ZoneKind;
use regex::Regex;
use serde_json::{json, Value};
use std::sync::LazyLock;

fn opts(v: Value) -> Options {
    v.as_object().cloned().unwrap_or_default()
}

// ---- trailing-spaces ---------------------------------------------------------

pub struct TrailingSpaces;

impl Rule for TrailingSpaces {
    fn id(&self) -> &'static str {
        "trailing-spaces"
    }

    fn description(&self) -> &'static str {
        "Remove spaces and tabs at the end of every line."
    }

    fn default_options(&self) -> Options {
        opts(json!({"two-space-linebreak": false}))
    }

    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit> {
        let keep_two = options
            .get("two-space-linebreak")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let sc = scan(doc, 2);
        let mut out = Vec::new();
        for l in &sc.lines {
            if !l.kind.is_text() {
                continue;
            }
            let trimmed = l.text.trim_end_matches([' ', '\t']);
            if trimmed.len() == l.text.len() {
                continue;
            }
            if keep_two && !trimmed.is_empty() && &l.text[trimmed.len()..] == "  " {
                continue;
            }
            let (s, e) = (l.start + trimmed.len(), l.end);
            if !prot(doc, s, e) {
                out.push(Edit::new(s, e, "", "trailing whitespace removed"));
            }
        }
        out
    }
}

// ---- remove-multiple-spaces --------------------------------------------------

pub struct RemoveMultipleSpaces;

impl Rule for RemoveMultipleSpaces {
    fn id(&self) -> &'static str {
        "remove-multiple-spaces"
    }

    fn description(&self) -> &'static str {
        "Collapse two or more consecutive spaces inside a line to one space."
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
            let t = l.text.as_bytes();
            let lead = t
                .iter()
                .take_while(|&&b| b == b' ' || b == b'\t' || b == b'>')
                .count();
            let tail = l.text.trim_end_matches([' ', '\t']).len();
            let mut i = lead;
            while i < tail {
                if t[i] != b' ' {
                    i += 1;
                    continue;
                }
                let mut j = i;
                while j < tail && t[j] == b' ' {
                    j += 1;
                }
                let (s, e) = (l.start + i, l.start + j);
                if j - i >= 2 && j < tail && i > lead && !prot(doc, s, e) {
                    out.push(Edit::new(s, e, " ", "repeated spaces collapsed"));
                }
                i = j;
            }
        }
        out
    }
}

// ---- proper-ellipsis ---------------------------------------------------------

pub struct ProperEllipsis;

impl Rule for ProperEllipsis {
    fn id(&self) -> &'static str {
        "proper-ellipsis"
    }

    fn description(&self) -> &'static str {
        "Replace three consecutive periods with the ellipsis character."
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
            let t = l.text.as_bytes();
            let mut i = 0;
            while i < t.len() {
                if t[i] != b'.' {
                    i += 1;
                    continue;
                }
                let mut j = i;
                while j < t.len() && t[j] == b'.' {
                    j += 1;
                }
                let slash = (i > 0 && t[i - 1] == b'/') || t.get(j) == Some(&b'/');
                let (s, e) = (l.start + i, l.start + j);
                if j - i == 3 && !slash && !prot(doc, s, e) {
                    out.push(Edit::new(s, e, "\u{2026}", "ellipsis"));
                }
                i = j;
            }
        }
        out
    }
}

// ---- default-language-for-code-fences ----------------------------------------

pub struct DefaultLanguageForCodeFences;

impl Rule for DefaultLanguageForCodeFences {
    fn id(&self) -> &'static str {
        "default-language-for-code-fences"
    }

    fn description(&self) -> &'static str {
        "Add a default language to opening code fences that have none."
    }

    fn default_options(&self) -> Options {
        opts(json!({"language": ""}))
    }

    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit> {
        let lang = str_opt(options.get("language"), "").trim();
        if lang.is_empty() || lang.chars().any(char::is_whitespace) {
            return Vec::new();
        }
        let sc = scan(doc, 2);
        let mut out = Vec::new();
        for l in &sc.lines {
            if l.kind != Kind::FenceOpen {
                continue;
            }
            let Some((ch, _, off, info)) = fence_info(l.inner) else {
                continue;
            };
            if !info.trim().is_empty() || (ch == '`' && lang.contains('`')) {
                continue;
            }
            let at = l.inner_start + off;
            out.push(Edit::new(at, at, lang, "default code fence language added"));
        }
        out
    }
}

// ---- no-bare-urls ------------------------------------------------------------

pub struct NoBareUrls;

static SCHEME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:https?|ftp|file|obsidian)://").expect("regex"));

fn url_end(t: &str, start: usize) -> usize {
    crate::zones::bare_url_end(t, start)
}

impl Rule for NoBareUrls {
    fn id(&self) -> &'static str {
        "no-bare-urls"
    }

    fn description(&self) -> &'static str {
        "Wrap bare URLs in angle brackets so they stay links."
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
            let t = l.text;
            let mut done = 0;
            for m in SCHEME.find_iter(t) {
                let p = m.start();
                if p < done
                    || t[..p]
                        .chars()
                        .next_back()
                        .is_some_and(char::is_alphanumeric)
                {
                    continue;
                }
                let e = url_end(t, p);
                done = e.max(m.end());
                if e <= m.end() {
                    continue;
                }
                let (abs_s, abs_e) = (l.start + p, l.start + e);
                if prot_except_url(doc, abs_s, abs_e) || t[..p].ends_with("](") {
                    continue;
                }
                let lt = t[..p].bytes().rev().take_while(|&b| b == b'<').count();
                let gt = t[e..].bytes().take_while(|&b| b == b'>').count();
                if lt > 0 {
                    if gt > 0 {
                        if lt > 1 {
                            out.push(Edit::new(
                                abs_s - (lt - 1),
                                abs_s,
                                "",
                                "duplicate angle bracket",
                            ));
                        }
                        if gt > 1 {
                            out.push(Edit::new(
                                abs_e + 1,
                                abs_e + gt,
                                "",
                                "duplicate angle bracket",
                            ));
                        }
                    }
                    continue;
                }
                if gt > 0 {
                    continue;
                }
                let (before, after) = (t[..p].chars().next_back(), t[e..].chars().next());
                if matches!(
                    (before, after),
                    (Some('['), Some(']')) | (Some('\''), Some('\'')) | (Some('"'), Some('"'))
                ) {
                    continue;
                }
                if let Some(lb) = t[..p].rfind('[') {
                    if !t[lb..p].contains(']') {
                        if let Some(rb) = t[e..].find(']') {
                            if t[e + rb + 1..].starts_with('(') {
                                continue;
                            }
                        }
                    }
                }
                // Two insertions at the edges of the URL, so the URL itself is never
                // rewritten; an edge inside another protected region leaves it alone.
                if inside_except_url(doc, abs_s) || inside_except_url(doc, abs_e) {
                    continue;
                }
                // The zone scanner reads a wider scheme here (for example
                // `words---https://` is one RFC scheme), so the URL it protects
                // starts earlier; inserting at our start would land inside it.
                if doc
                    .zones()
                    .iter()
                    .any(|z| z.kind == ZoneKind::Url && z.start < abs_s && z.end > abs_s)
                {
                    continue;
                }
                out.push(Edit::new(abs_s, abs_s, "<", "bare URL wrapped"));
                out.push(Edit::new(abs_e, abs_e, ">", "bare URL wrapped"));
            }
        }
        out
    }
}

// ---- quote-style -------------------------------------------------------------

pub struct QuoteStyle;

/// Sorted, merged protected byte intervals: zones plus Templater commands.
fn intervals(doc: &Document) -> Vec<(usize, usize)> {
    let text = doc.text();
    let mut v: Vec<(usize, usize)> = doc.zones().iter().map(|z| (z.start, z.end)).collect();
    let mut at = 0;
    while let Some(s) = text[at..].find("<%") {
        let s = at + s;
        let e = text[s + 2..]
            .find("%>")
            .map_or(text.len(), |x| s + 2 + x + 2);
        v.push((s, e));
        at = e.max(s + 2);
    }
    v.sort_unstable();
    let mut merged: Vec<(usize, usize)> = Vec::new();
    for (s, e) in v {
        match merged.last_mut() {
            Some(last) if s <= last.1 => last.1 = last.1.max(e),
            _ => merged.push((s, e)),
        }
    }
    merged
}

fn in_intervals(iv: &[(usize, usize)], pos: usize) -> bool {
    let i = iv.partition_point(|&(s, _)| s <= pos);
    i > 0 && iv[i - 1].1 > pos
}

impl QuoteStyle {
    fn single(text: &str, positions: &[usize], smart: bool, out: &mut Vec<Edit>) {
        if !smart {
            return;
        }
        let letter = |c: Option<char>| c.is_some_and(char::is_alphabetic);
        let mut rest: Vec<usize> = Vec::new();
        for &p in positions {
            let (b, a) = (text[..p].chars().next_back(), text[p + 1..].chars().next());
            if letter(b) && letter(a) {
                out.push(Edit::new(p, p + 1, "\u{2019}", "smart apostrophe"));
            } else {
                rest.push(p);
            }
        }
        if rest.len() % 2 == 1 {
            let (apos, keep): (Vec<usize>, Vec<usize>) = rest.iter().copied().partition(|&p| {
                let b = text[..p].chars().next_back();
                let a = text[p + 1..].chars().next();
                letter(b) && a.is_none_or(|c| c.is_whitespace())
            });
            if keep.len() % 2 == 0 {
                for p in apos {
                    out.push(Edit::new(p, p + 1, "\u{2019}", "smart apostrophe"));
                }
                rest = keep;
            }
        }
        Self::pair(&rest, "\u{2018}", "\u{2019}", "single", out);
    }

    fn pair(positions: &[usize], open: &str, close: &str, kind: &str, out: &mut Vec<Edit>) {
        if positions.len() % 2 == 1 {
            let p = positions[0];
            out.push(Edit::new(
                p,
                p,
                "",
                format!("odd number of straight {kind} quotes; left unchanged"),
            ));
            return;
        }
        for (i, &p) in positions.iter().enumerate() {
            out.push(Edit::new(
                p,
                p + 1,
                if i % 2 == 0 { open } else { close },
                format!("{kind} quote converted"),
            ));
        }
    }
}

impl Rule for QuoteStyle {
    fn id(&self) -> &'static str {
        "quote-style"
    }

    fn description(&self) -> &'static str {
        "Make single and double quotes in body text either all straight or all smart."
    }

    fn default_options(&self) -> Options {
        opts(json!({
            "single-quote-enabled": true,
            "single-quote-style": "straight",
            "double-quote-enabled": true,
            "double-quote-style": "straight"
        }))
    }

    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit> {
        let text = doc.text();
        let flag = |k: &str| options.get(k).and_then(Value::as_bool).unwrap_or(true);
        let (s_on, d_on) = (flag("single-quote-enabled"), flag("double-quote-enabled"));
        let s_smart = str_opt(options.get("single-quote-style"), "straight") == "smart";
        let d_smart = str_opt(options.get("double-quote-style"), "straight") == "smart";
        let iv = intervals(doc);
        let (mut singles, mut doubles) = (Vec::new(), Vec::new());
        let mut out = Vec::new();
        for (i, c) in text.char_indices() {
            if !matches!(
                c,
                '\'' | '"'
                    | '\u{201c}'
                    | '\u{201d}'
                    | '\u{201e}'
                    | '\u{ab}'
                    | '\u{bb}'
                    | '\u{2018}'
                    | '\u{2019}'
                    | '\u{201a}'
                    | '\u{2039}'
                    | '\u{203a}'
            ) || in_intervals(&iv, i)
            {
                continue;
            }
            let n = c.len_utf8();
            match c {
                '"' => doubles.push(i),
                '\'' => singles.push(i),
                '\u{201c}' | '\u{201d}' | '\u{201e}' | '\u{ab}' | '\u{bb}' => {
                    if d_on && !d_smart {
                        out.push(Edit::new(i, i + n, "\"", "straight double quote"));
                    }
                }
                _ => {
                    if s_on && !s_smart {
                        out.push(Edit::new(i, i + n, "'", "straight single quote"));
                    }
                }
            }
        }
        if d_on && d_smart {
            Self::pair(&doubles, "\u{201c}", "\u{201d}", "double", &mut out);
        }
        if s_on {
            Self::single(text, &singles, s_smart, &mut out);
        }
        out
    }
}
