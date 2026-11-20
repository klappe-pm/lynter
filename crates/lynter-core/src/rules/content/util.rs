//! Helpers shared by the whole-text rules: JavaScript whitespace and a
//! minimal-span diff that turns a transformed text into one edit.

use crate::document::Document;
use crate::edit::Edit;
use crate::zones::{LineSpan, ZoneKind};

/// The JavaScript `\s` class, as the body of a regex character class.
pub const WS: &str = "\t\n\u{b}\u{c}\r \u{a0}\u{1680}\u{2000}-\u{200a}\u{2028}\u{2029}\u{202f}\u{205f}\u{3000}\u{feff}";
/// JavaScript `.`: anything but a line terminator.
pub const DOT: &str = "[^\n\r\u{2028}\u{2029}]";

/// True when a line must not be rewritten: inside a whole-line protected
/// region, or inside an HTML block found by the Markdown parse.
pub fn is_masked_line(doc: &Document, line: &LineSpan) -> bool {
    // A lone carriage return inside a line is a line break to the Markdown
    // parse; what follows it may be a different block.
    doc.text()[line.start..line.end]
        .trim_end_matches('\r')
        .contains('\r')
        || doc.is_line_block_masked(line)
        || doc
            .zones()
            .iter()
            .any(|z| z.kind == ZoneKind::Html && z.start <= line.start && z.end >= line.end)
}

/// True for the characters JavaScript's `\s` and `String.prototype.trim` accept.
pub fn is_js_ws(c: char) -> bool {
    matches!(
        c,
        '\t' | '\n' | '\u{b}' | '\u{c}' | '\r' | ' ' | '\u{a0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200a}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202f}'
                | '\u{205f}'
                | '\u{3000}'
                | '\u{feff}'
    )
}

pub fn js_trim(s: &str) -> &str {
    s.trim_matches(is_js_ws)
}

pub fn js_trim_end(s: &str) -> &str {
    s.trim_end_matches(is_js_ws)
}

/// Regex source with `\s` and `\S` replaced by the JavaScript classes.
pub fn js_pattern(src: &str) -> String {
    src.replace("\\S", &format!("[^{WS}]"))
        .replace("\\s", &format!("[{WS}]"))
        .replace("DOT", DOT)
}

/// Longest line-table size the diff will build before it falls back to one
/// edit for the whole changed middle.
const MAX_DIFF_CELLS: usize = 4_000_000;

fn units(s: &str) -> (Vec<&str>, Vec<usize>) {
    let mut out = Vec::new();
    let mut offsets = vec![0];
    for u in s.split_inclusive('\n') {
        out.push(u);
        offsets.push(offsets[offsets.len() - 1] + u.len());
    }
    (out, offsets)
}

/// Trim the common prefix and suffix of two strings at char boundaries and
/// return one edit for what remains, shifted by `base`.
fn trimmed_edit(old: &str, new: &str, base: usize, message: &str) -> Option<Edit> {
    if old == new {
        return None;
    }
    let mut p = old
        .bytes()
        .zip(new.bytes())
        .take_while(|(a, b)| a == b)
        .count();
    while !old.is_char_boundary(p) || !new.is_char_boundary(p) {
        p -= 1;
    }
    let max = old.len().min(new.len()) - p;
    let mut s = old
        .bytes()
        .rev()
        .zip(new.bytes().rev())
        .take(max)
        .take_while(|(a, b)| a == b)
        .count();
    while !old.is_char_boundary(old.len() - s) || !new.is_char_boundary(new.len() - s) {
        s -= 1;
    }
    Some(Edit::new(
        base + p,
        base + old.len() - s,
        &new[p..new.len() - s],
        message,
    ))
}

/// Edits turning `old` into `new`: a line diff, each changed run trimmed to
/// the bytes that differ, so unchanged lines (and the code, math, and links
/// inside changed ones) are never part of an edit. Empty when equal.
pub fn diff_edit(old: &str, new: &str, message: &str) -> Vec<Edit> {
    if old == new {
        return Vec::new();
    }
    let (o, oo) = units(old);
    let (n, no) = units(new);
    let pre = o.iter().zip(&n).take_while(|(a, b)| a == b).count();
    let max = o.len().min(n.len()) - pre;
    let suf = o
        .iter()
        .rev()
        .zip(n.iter().rev())
        .take(max)
        .take_while(|(a, b)| a == b)
        .count();
    let (oe, ne) = (o.len() - suf, n.len() - suf);
    let (a, b) = (oe - pre, ne - pre);
    let whole = |out: &mut Vec<Edit>| {
        out.extend(trimmed_edit(
            &old[oo[pre]..oo[oe]],
            &new[no[pre]..no[ne]],
            oo[pre],
            message,
        ));
    };
    let mut out = Vec::new();
    if a == 0 || b == 0 || a.saturating_mul(b) > MAX_DIFF_CELLS {
        whole(&mut out);
        return out;
    }
    let w = b + 1;
    let mut dp = vec![0u32; (a + 1) * w];
    for i in (0..a).rev() {
        for j in (0..b).rev() {
            dp[i * w + j] = if o[pre + i] == n[pre + j] {
                dp[(i + 1) * w + j + 1] + 1
            } else {
                dp[(i + 1) * w + j].max(dp[i * w + j + 1])
            };
        }
    }
    let (mut i, mut j) = (0, 0);
    let mut hunk: Option<(usize, usize)> = None; // (i, j) where the run began
    let flush = |out: &mut Vec<Edit>, h: &mut Option<(usize, usize)>, i: usize, j: usize| {
        if let Some((hi, hj)) = h.take() {
            out.extend(trimmed_edit(
                &old[oo[pre + hi]..oo[pre + i]],
                &new[no[pre + hj]..no[pre + j]],
                oo[pre + hi],
                message,
            ));
        }
    };
    while i < a && j < b {
        if o[pre + i] == n[pre + j] {
            flush(&mut out, &mut hunk, i, j);
            i += 1;
            j += 1;
        } else {
            hunk.get_or_insert((i, j));
            if dp[(i + 1) * w + j] >= dp[i * w + j + 1] {
                i += 1;
            } else {
                j += 1;
            }
        }
    }
    if i < a || j < b {
        hunk.get_or_insert((i, j));
    }
    flush(&mut out, &mut hunk, a, b);
    out
}
