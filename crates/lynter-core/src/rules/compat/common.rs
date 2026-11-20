//! Shared line model for the compat rules: one pass classifies every line
//! (blank, fence, math, table, skipped) with its blockquote depth, so each
//! rule can work on lines without re-deriving block structure.

use crate::document::Document;
use crate::edit::Edit;
use crate::rules::content::util::js_trim;
use crate::zones::{Zone, ZoneKind};
use serde_json::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Ordinary text, list items, headings, rules.
    Normal,
    /// Empty or whitespace-only after the quote prefix.
    Blank,
    /// Frontmatter, HTML, indented code, other protected whole lines.
    Skip,
    FenceOpen,
    FenceBody,
    FenceClose,
    MathOpen,
    MathBody,
    MathClose,
    MathSingle,
    TableHead,
    TableDelim,
    TableRow,
}

impl Kind {
    /// Lines that rules may treat as ordinary text.
    pub fn is_text(self) -> bool {
        matches!(self, Kind::Normal | Kind::Blank)
    }
}

pub struct Info<'a> {
    pub start: usize,
    /// End of the content, before the line break.
    pub end: usize,
    /// Start of the next line.
    pub next: usize,
    pub eol: &'a str,
    pub text: &'a str,
    pub depth: usize,
    /// Absolute offset of the text after the blockquote prefix.
    pub inner_start: usize,
    pub inner: &'a str,
    pub kind: Kind,
}

pub struct Scan<'a> {
    pub lines: Vec<Info<'a>>,
    /// Index of the closing frontmatter line, when the document has frontmatter.
    pub fm_last: Option<usize>,
}

/// `(depth, byte length)` of the blockquote prefix of `s`.
pub fn quote_prefix(s: &str) -> (usize, usize) {
    quote_prefix_max(s, usize::MAX)
}

pub fn quote_prefix_max(s: &str, max: usize) -> (usize, usize) {
    let b = s.as_bytes();
    let (mut pos, mut depth) = (0, 0);
    while depth < max {
        let mut p = pos;
        let mut sp = 0;
        while p < b.len() && b[p] == b' ' && sp < 3 {
            p += 1;
            sp += 1;
        }
        if p < b.len() && b[p] == b'>' {
            p += 1;
            if p < b.len() && b[p] == b' ' {
                p += 1;
            }
            pos = p;
            depth += 1;
        } else {
            break;
        }
    }
    (depth, pos)
}

/// A bare quote line of the given depth, without a trailing space.
pub fn sep_line(depth: usize) -> String {
    vec![">"; depth].join(" ")
}

/// Fence opener on `inner`: `(marker, run length, offset after the run, info)`.
pub fn fence_info(inner: &str) -> Option<(char, usize, usize, &str)> {
    let spaces = inner.bytes().take_while(|&b| b == b' ').count();
    if spaces > 3 {
        return None;
    }
    let rest = &inner[spaces..];
    let c = rest.chars().next()?;
    if c != '`' && c != '~' {
        return None;
    }
    let n = rest.chars().take_while(|&x| x == c).count();
    if n < 3 {
        return None;
    }
    // Like the document model, an info string is not checked for backticks.
    Some((c, n, spaces + n, &rest[n..]))
}

pub fn is_fence_close(inner: &str, marker: char, min: usize) -> bool {
    let spaces = inner.bytes().take_while(|&b| b == b' ').count();
    if spaces > 3 {
        return false;
    }
    let rest = &inner[spaces..];
    let n = rest.chars().take_while(|&x| x == marker).count();
    n >= min && rest[n..].chars().all(|c| c == ' ' || c == '\t')
}

fn is_delim_cell(c: &str) -> bool {
    let t = c.trim_matches([' ', '\t']);
    let t = t.strip_prefix(':').unwrap_or(t);
    let t = t.strip_suffix(':').unwrap_or(t);
    !t.is_empty() && t.bytes().all(|b| b == b'-')
}

/// A table delimiter row: pipes, dashes and optional alignment colons.
pub fn is_delim_row(inner: &str) -> bool {
    let t = inner.trim_matches([' ', '\t']);
    if !t.contains('|') {
        return false;
    }
    let t = t.strip_prefix('|').unwrap_or(t);
    let t = t.strip_suffix('|').unwrap_or(t);
    t.split('|').all(is_delim_cell)
}

fn single_math(t: &str, n: usize) -> bool {
    if n == 0 || t.len() <= 2 * n {
        return false;
    }
    let lead = t.bytes().take_while(|&b| b == b'$').count();
    let trail = t.bytes().rev().take_while(|&b| b == b'$').count();
    lead == n && trail == n && lead + trail < t.len()
}

/// Math opener classification of one line, mirroring the document model.
fn math_open(inner: &str, dollars: usize) -> Option<Kind> {
    let t = js_trim(inner);
    if single_math(t, dollars) {
        return Some(Kind::MathSingle);
    }
    let rest = t.strip_prefix("$$")?;
    match rest.find("$$") {
        None => Some(Kind::MathOpen),
        Some(p) if p + 2 == rest.len() => Some(Kind::MathSingle),
        Some(_) => None,
    }
}

fn covered(doc: &Document, start: usize, end: usize, kinds: &[ZoneKind]) -> bool {
    doc.zones()
        .iter()
        .any(|z: &Zone| kinds.contains(&z.kind) && z.start <= start && z.end >= end)
}

/// Classify every line of the document.
pub fn scan(doc: &Document, dollars: usize) -> Scan<'_> {
    let text = doc.text();
    let fm_end = doc
        .zones()
        .iter()
        .find(|z| z.kind == ZoneKind::Frontmatter && z.start == 0)
        .map(|z| z.end);
    let mut raws: Vec<(usize, &str, &str)> = Vec::new();
    let mut start = 0;
    for raw in text.split_inclusive('\n') {
        let (content, eol) = if let Some(c) = raw.strip_suffix("\r\n") {
            (c, "\r\n")
        } else if let Some(c) = raw.strip_suffix('\n') {
            (c, "\n")
        } else {
            (raw, "")
        };
        raws.push((start, content, eol));
        start += raw.len();
    }
    let mut lines: Vec<Info> = Vec::with_capacity(raws.len());
    let mut fm_last = None;
    let mut fence: Option<(usize, char, usize)> = None;
    let mut math: Option<usize> = None;
    let mut table: Option<usize> = None;
    let mut delim_pending = false;
    for (i, &(start, content, eol)) in raws.iter().enumerate() {
        let end = start + content.len();
        let next = end + eol.len();
        let mk = |kind: Kind, depth: usize, plen: usize| Info {
            start,
            end,
            next,
            eol,
            text: content,
            depth,
            inner_start: start + plen,
            inner: &content[plen..],
            kind,
        };
        if fm_end.is_some_and(|e| start < e) {
            fm_last = Some(i);
            lines.push(mk(Kind::Skip, 0, 0));
            continue;
        }
        let hard = content.trim_end_matches('\r').contains('\r')
            || covered(doc, start, end, &[ZoneKind::Html, ZoneKind::IndentedCode]);
        let (qd, ql) = quote_prefix(content);
        if let Some((fd, ch, n)) = fence {
            if qd >= fd {
                let (_, pl) = quote_prefix_max(content, fd);
                let kind = if is_fence_close(&content[pl..], ch, n) {
                    fence = None;
                    Kind::FenceClose
                } else {
                    Kind::FenceBody
                };
                lines.push(mk(kind, fd, pl));
                continue;
            }
            fence = None;
        }
        if let Some(md) = math {
            if qd >= md {
                let (_, pl) = quote_prefix_max(content, md);
                // Closes where the document model closes its math region: a line ending in `$$`.
                let kind = if js_trim(&content[pl..]).ends_with("$$") {
                    math = None;
                    Kind::MathClose
                } else {
                    Kind::MathBody
                };
                lines.push(mk(kind, md, pl));
                continue;
            }
            math = None;
        }
        if delim_pending {
            delim_pending = false;
            if let Some(td) = table {
                let (_, pl) = quote_prefix_max(content, td);
                lines.push(mk(Kind::TableDelim, td, pl));
                continue;
            }
        }
        if let Some(td) = table {
            let (_, pl) = quote_prefix_max(content, td);
            let inner = &content[pl..];
            if qd == td && inner.contains('|') && !js_trim(inner).is_empty() {
                lines.push(mk(Kind::TableRow, td, pl));
                continue;
            }
            table = None;
        }
        if hard {
            lines.push(mk(Kind::Skip, 0, 0));
            continue;
        }
        let inner = &content[ql..];
        let kind = if js_trim(inner).is_empty() {
            Kind::Blank
        } else if let Some((ch, n, _, _)) = fence_info(inner) {
            fence = Some((qd, ch, n));
            Kind::FenceOpen
        } else if let Some(k) = math_open(inner, dollars) {
            if k == Kind::MathOpen {
                math = Some(qd);
            }
            k
        } else if inner.contains('|')
            && raws.get(i + 1).is_some_and(|&(_, nc, _)| {
                let (nd, nl) = quote_prefix(nc);
                nd == qd && is_delim_row(&nc[nl..])
            })
        {
            table = Some(qd);
            delim_pending = true;
            Kind::TableHead
        } else {
            Kind::Normal
        };
        // A block opener inside a protected region that began on an earlier line
        // (an HTML block holding a fence the line scan opened) is not an opener.
        if matches!(
            kind,
            Kind::FenceOpen | Kind::MathOpen | Kind::MathSingle | Kind::TableHead
        ) && doc
            .zones()
            .iter()
            .any(|z| z.kind.is_block() && z.start < start && z.end >= end)
        {
            (fence, math, table, delim_pending) = (None, None, None, false);
            lines.push(mk(Kind::Skip, 0, 0));
            continue;
        }
        // A whole-line block region the scan did not recognise is protected.
        let code_masked = kind.is_text()
            && doc
                .zones()
                .iter()
                .any(|z| z.kind.is_block() && z.start <= start && z.end >= end);
        if code_masked {
            lines.push(mk(Kind::Skip, 0, 0));
        } else {
            lines.push(mk(kind, qd, ql));
        }
    }
    Scan { lines, fm_last }
}

/// The document's own line ending: that of the first line break, else LF.
pub fn doc_eol(text: &str) -> &'static str {
    match text.find('\n') {
        Some(i) if i > 0 && text.as_bytes()[i - 1] == b'\r' => "\r\n",
        _ => "\n",
    }
}

/// True when `[start, end)` overlaps a protected region.
pub fn prot(doc: &Document, start: usize, end: usize) -> bool {
    doc.is_protected(start, end)
}

/// True when an insertion at `at` lands strictly inside a protected region.
pub fn prot_ins(doc: &Document, at: usize) -> bool {
    doc.zones().iter().any(|z| z.start < at && z.end > at)
}

/// True when `at` lies strictly inside a protected region other than a bare URL.
pub fn inside_except_url(doc: &Document, at: usize) -> bool {
    doc.zones()
        .iter()
        .any(|z| z.kind != ZoneKind::Url && z.start < at && z.end > at)
}

/// True when the span overlaps a protected region other than bare URLs.
pub fn prot_except_url(doc: &Document, start: usize, end: usize) -> bool {
    doc.zones()
        .iter()
        .any(|z| z.kind != ZoneKind::Url && z.start < end.max(start + 1) && z.end > start)
}

pub fn bool_opt(v: Option<&Value>, default: bool) -> bool {
    v.and_then(Value::as_bool).unwrap_or(default)
}

pub fn str_opt<'a>(v: Option<&'a Value>, default: &'a str) -> &'a str {
    v.and_then(Value::as_str).unwrap_or(default)
}

pub fn list_opt(v: Option<&Value>) -> Vec<String> {
    match v {
        Some(Value::Array(a)) => a
            .iter()
            .filter_map(|x| x.as_str().map(str::to_owned))
            .collect(),
        Some(Value::String(s)) => s
            .split(['\n', ','])
            .map(|x| x.trim().to_owned())
            .filter(|x| !x.is_empty())
            .collect(),
        _ => Vec::new(),
    }
}

/// A blank insertion edit: `sep` plus a line break, placed at `at`.
pub fn insert_line(at: usize, sep: &str, eol: &str, message: &str) -> Edit {
    Edit::new(at, at, format!("{sep}{eol}"), message)
}

// ---- line patterns ------------------------------------------------------

/// A thematic break: 3 or more of one of `*`, `-`, `_`, spaces allowed.
pub fn is_hr(inner: &str) -> bool {
    let spaces = inner.bytes().take_while(|&b| b == b' ').count();
    if spaces > 3 {
        return false;
    }
    let mut it = inner[spaces..].chars().filter(|&c| c != ' ' && c != '\t');
    let first = match it.next() {
        Some(c) if matches!(c, '*' | '-' | '_') => c,
        _ => return false,
    };
    let mut n = 1;
    for c in it {
        if c != first {
            return false;
        }
        n += 1;
    }
    n >= 3
}

pub struct Atx {
    /// Offset of the first `#` in the inner text.
    pub hash_start: usize,
    pub level: usize,
}

/// An ATX heading: up to 3 spaces, 1 to 6 `#`, then a space or the line end.
pub fn atx(inner: &str) -> Option<Atx> {
    let spaces = inner.bytes().take_while(|&b| b == b' ').count();
    if spaces > 3 {
        return None;
    }
    let level = inner[spaces..].bytes().take_while(|&b| b == b'#').count();
    if !(1..=6).contains(&level) {
        return None;
    }
    match inner[spaces + level..].chars().next() {
        None | Some(' ') | Some('\t') => Some(Atx {
            hash_start: spaces,
            level,
        }),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Marker {
    /// Bytes of indentation before the marker.
    pub marker_start: usize,
    /// End of the marker token (after the digits and delimiter, or bullet).
    pub marker_end: usize,
    /// End of the whitespace run after the marker.
    pub ws_end: usize,
    pub ordered: bool,
    pub bullet: char,
    pub number: u64,
    /// Digits of an ordered marker (start..end offsets in the inner text).
    pub digits: (usize, usize),
}

/// A list marker at the start of `inner` (after indentation).
pub fn list_marker(inner: &str) -> Option<Marker> {
    let b = inner.as_bytes();
    let ms = b.iter().take_while(|&&c| c == b' ' || c == b'\t').count();
    let rest = &inner[ms..];
    let first = rest.chars().next()?;
    let (me, ordered, number, digits) = if matches!(first, '-' | '*' | '+') {
        (ms + 1, false, 0, (ms, ms))
    } else if first.is_ascii_digit() {
        let n = rest.bytes().take_while(u8::is_ascii_digit).count();
        if n > 9 {
            return None;
        }
        let d = rest[n..].chars().next()?;
        if d != '.' && d != ')' {
            return None;
        }
        (ms + n + 1, true, rest[..n].parse().ok()?, (ms, ms + n))
    } else {
        return None;
    };
    match inner[me..].chars().next() {
        None | Some(' ') | Some('\t') => {}
        _ => return None,
    }
    let ws = inner[me..]
        .bytes()
        .take_while(|&c| c == b' ' || c == b'\t')
        .count();
    Some(Marker {
        marker_start: ms,
        marker_end: me,
        ws_end: me + ws,
        ordered,
        bullet: if ordered { ' ' } else { first },
        number,
        digits,
    })
}

/// True when the line is paragraph text that a following `-` line would
/// turn into a setext heading.
pub fn is_paragraph_text(line: &Info) -> bool {
    line.kind == Kind::Normal
        && !is_hr(line.inner)
        && atx(line.inner).is_none()
        && list_marker(line.inner).is_none()
}

/// `---`-style setext underline candidate: only dashes (spaces around).
pub fn is_dash_underline(inner: &str) -> bool {
    let t = inner.trim_matches([' ', '\t']);
    !t.is_empty() && t.bytes().all(|b| b == b'-')
}
