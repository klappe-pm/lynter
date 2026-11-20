//! Line-preserving access to frontmatter: lines between the delimiters, and
//! top-level entries with their value lines.

use crate::document::Document;
use crate::zones::ZoneKind;

pub struct FmLine<'a> {
    pub start: usize,
    pub next: usize,
    pub eol: &'a str,
    pub text: &'a str,
}

pub struct Fm<'a> {
    /// Lines strictly between the opening and closing delimiters.
    pub lines: Vec<FmLine<'a>>,
    /// Start of the closing delimiter line.
    pub close_start: usize,
    /// Line break of the opening delimiter line.
    pub eol: &'a str,
}

pub fn frontmatter(doc: &Document) -> Option<Fm<'_>> {
    let zone = doc
        .zones()
        .iter()
        .find(|z| z.kind == ZoneKind::Frontmatter && z.start == 0)?;
    let text = doc.text();
    let mut all = Vec::new();
    let mut start = 0;
    for raw in text.split_inclusive('\n') {
        if start >= zone.end {
            break;
        }
        let (content, eol) = if let Some(c) = raw.strip_suffix("\r\n") {
            (c, "\r\n")
        } else if let Some(c) = raw.strip_suffix('\n') {
            (c, "\n")
        } else {
            (raw, "")
        };
        all.push(FmLine {
            start,
            next: start + raw.len(),
            eol,
            text: content,
        });
        start += raw.len();
    }
    if all.len() < 2 {
        return None;
    }
    let close_start = all[all.len() - 1].start;
    let eol = all[0].eol;
    let n = all.len();
    all.truncate(n - 1);
    all.remove(0);
    Some(Fm {
        lines: all,
        close_start,
        eol,
    })
}

pub struct Entry {
    /// The key without quotes.
    pub key: String,
    /// The key line up to (not including) the colon, as written.
    pub key_text: String,
    pub first: usize,
    pub last: usize,
    /// Inline value after the colon, trimmed.
    pub value: String,
    /// Absolute offset of the inline value.
    pub value_start: usize,
}

/// `(key, key text, offset after the colon)` for a top-level key line.
fn key_of(t: &str) -> Option<(String, String, usize)> {
    let first = t.chars().next()?;
    if first.is_whitespace() || matches!(first, '#' | '[' | '{') || t == "-" || t.starts_with("- ")
    {
        return None;
    }
    let colon = if first == '"' || first == '\'' {
        let q = t[1..].find(first)? + 1;
        let rest = &t[q + 1..];
        let ws = rest.len() - rest.trim_start_matches([' ', '\t']).len();
        if !rest[ws..].starts_with(':') {
            return None;
        }
        q + 1 + ws
    } else {
        let mut at = 0;
        loop {
            let i = at + t[at..].find(':')?;
            match t[i + 1..].chars().next() {
                None | Some(' ') | Some('\t') => break i,
                _ => at = i + 1,
            }
        }
    };
    let raw = t[..colon].trim_end();
    if raw.is_empty() {
        return None;
    }
    let key = if (raw.starts_with('"') && raw.ends_with('"')
        || raw.starts_with('\'') && raw.ends_with('\''))
        && raw.len() >= 2
    {
        raw[1..raw.len() - 1].to_owned()
    } else {
        raw.to_owned()
    };
    Some((key, raw.to_owned(), colon + 1))
}

fn continues(t: &str, value_empty: bool) -> bool {
    !t.trim().is_empty()
        && (t.starts_with([' ', '\t']) || (value_empty && (t == "-" || t.starts_with("- "))))
}

pub fn entries(fm: &Fm) -> Vec<Entry> {
    let n = fm.lines.len();
    let mut out = Vec::new();
    let mut i = 0;
    while i < n {
        let l = &fm.lines[i];
        let Some((key, key_text, after)) = key_of(l.text) else {
            i += 1;
            continue;
        };
        let full = &l.text[after..];
        let lead = full.len() - full.trim_start_matches([' ', '\t']).len();
        let value = full[lead..].trim_end().to_owned();
        let mut last = i;
        let mut j = i + 1;
        while j < n {
            let t = fm.lines[j].text;
            if continues(t, value.is_empty()) {
                last = j;
                j += 1;
            } else if t.trim().is_empty() {
                let k = (j..n)
                    .find(|&k| !fm.lines[k].text.trim().is_empty())
                    .unwrap_or(n);
                if k < n && continues(fm.lines[k].text, value.is_empty()) {
                    j = k;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        out.push(Entry {
            key,
            key_text,
            first: i,
            last,
            value,
            value_start: l.start + after + lead,
        });
        i = last + 1;
    }
    out
}

pub struct BlockItem {
    pub line: usize,
    /// Bytes before the dash.
    pub indent: usize,
    pub text: String,
}

/// The items of a block list entry whose lines are all plain `- item` lines.
pub fn block_items(fm: &Fm, e: &Entry) -> Option<Vec<BlockItem>> {
    if !e.value.is_empty() || e.first == e.last {
        return None;
    }
    let mut out = Vec::new();
    for (idx, l) in fm
        .lines
        .iter()
        .enumerate()
        .take(e.last + 1)
        .skip(e.first + 1)
    {
        let indent = l.text.len() - l.text.trim_start_matches([' ', '\t']).len();
        let rest = l.text[indent..].strip_prefix('-')?;
        let ws = rest.len() - rest.trim_start_matches([' ', '\t']).len();
        if !rest.is_empty() && ws == 0 {
            return None;
        }
        let body = rest[ws..].trim_end();
        if body.starts_with("- ") || body == "-" {
            return None;
        }
        out.push(BlockItem {
            line: idx,
            indent,
            text: body.to_owned(),
        });
    }
    Some(out)
}

/// An item of a flow array with its span relative to the parsed string.
#[derive(Clone, Debug)]
pub struct FlowItem {
    pub text: String,
    pub start: usize,
    pub end: usize,
}

/// Split a single-line flow array `[a, b]` at top-level commas.
pub fn parse_flow(value: &str) -> Option<Vec<FlowItem>> {
    let v = value.trim_end();
    if !v.starts_with('[') || !v.ends_with(']') || v.len() < 2 {
        return None;
    }
    let inner_start = 1;
    let inner = &v[1..v.len() - 1];
    let mut items = Vec::new();
    let mut depth = 0usize;
    let mut quote: Option<char> = None;
    let mut seg = 0;
    let push = |items: &mut Vec<FlowItem>, from: usize, to: usize| {
        let s = &inner[from..to];
        let lead = s.len() - s.trim_start().len();
        let t = s.trim();
        items.push(FlowItem {
            text: t.to_owned(),
            start: inner_start + from + lead,
            end: inner_start + from + lead + t.len(),
        });
    };
    let mut prev = '\0';
    for (i, c) in inner.char_indices() {
        match quote {
            Some(q) => {
                if c == q && prev != '\\' {
                    quote = None;
                }
            }
            None => match c {
                '"' | '\'' => quote = Some(c),
                '[' | '{' => depth += 1,
                ']' | '}' => depth = depth.saturating_sub(1),
                ',' if depth == 0 => {
                    push(&mut items, seg, i);
                    seg = i + 1;
                }
                _ => {}
            },
        }
        prev = c;
    }
    if quote.is_some() || depth != 0 {
        return None;
    }
    if !inner[seg..].trim().is_empty() || !items.is_empty() {
        push(&mut items, seg, inner.len());
    }
    Some(items)
}

/// Remove one level of matching quotes.
pub fn unquote(s: &str) -> String {
    let t = s.trim();
    if t.len() >= 2 {
        if t.starts_with('"') && t.ends_with('"') {
            return t[1..t.len() - 1].replace("\\\"", "\"");
        }
        if t.starts_with('\'') && t.ends_with('\'') {
            return t[1..t.len() - 1].replace("''", "'");
        }
    }
    t.to_owned()
}

/// True for an item that reads as `key: value` (a mapping, not a scalar).
pub fn mapping_like(item: &str) -> bool {
    !item.starts_with(['"', '\'', '[', '{']) && (item.contains(": ") || item.ends_with(':'))
}
