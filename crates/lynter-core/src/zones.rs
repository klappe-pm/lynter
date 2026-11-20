//! Protected regions: spans of a document that content rules must never edit.
//!
//! The line-based scan mirrors the owner's TypeScript masking (frontmatter,
//! fences, indented code, block math, tables, inline code, inline math,
//! wikilinks, URLs). The Markdown parse adds HTML, code, math, table, and
//! metadata ranges on top, so protection only ever grows.

use fancy_regex::Regex;
use std::sync::LazyLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZoneKind {
    Frontmatter,
    CodeFence,
    IndentedCode,
    InlineCode,
    MathBlock,
    InlineMath,
    Table,
    Wikilink,
    Url,
    /// HTML block or inline HTML found by the Markdown parse.
    Html,
    /// Code block, math, table, or metadata block found by the Markdown parse.
    ParsedBlock,
}

impl ZoneKind {
    /// Whole-line kinds: line-based rules skip these lines entirely.
    pub fn is_block(self) -> bool {
        matches!(
            self,
            ZoneKind::Frontmatter
                | ZoneKind::CodeFence
                | ZoneKind::IndentedCode
                | ZoneKind::MathBlock
                | ZoneKind::Table
                | ZoneKind::ParsedBlock
        )
    }
}

/// Half-open byte range `[start, end)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Zone {
    pub start: usize,
    pub end: usize,
    pub kind: ZoneKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LineSpan {
    pub start: usize,
    /// End of the line content, before the newline.
    pub end: usize,
}

impl LineSpan {
    pub fn text<'a>(&self, doc: &'a str) -> &'a str {
        &doc[self.start..self.end]
    }
}

pub fn line_spans(text: &str) -> Vec<LineSpan> {
    let mut spans = Vec::new();
    let mut start = 0;
    loop {
        match text[start..].find('\n') {
            Some(i) => {
                spans.push(LineSpan {
                    start,
                    end: start + i,
                });
                start += i + 1;
            }
            None => {
                spans.push(LineSpan {
                    start,
                    end: text.len(),
                });
                return spans;
            }
        }
    }
}

fn re(p: &str) -> Regex {
    Regex::new(p).expect("static regex")
}

static TABLE_DELIM_ROW: LazyLock<Regex> =
    LazyLock::new(|| re(r"^ {0,3}\|?[ \t]*:?-+:?[ \t]*(\|[ \t]*:?-+:?[ \t]*)*\|?[ \t]*$"));
static INLINE_PATTERNS: LazyLock<Vec<(ZoneKind, Regex)>> = LazyLock::new(|| {
    vec![
        (
            ZoneKind::InlineMath,
            re(r"\$\$[^\n\r\u{2028}\u{2029}]+?\$\$"),
        ),
        (
            ZoneKind::InlineMath,
            re(r"\$(?![\s$])(?:[^$\n]*?[^\s$\\])?\$(?!\d)"),
        ),
        (ZoneKind::Wikilink, re(r"!?\[\[[^\]\n]*\]\]")),
        (ZoneKind::Url, re(r"[a-zA-Z][a-zA-Z0-9+.-]*://[^\s<>]+")),
        (ZoneKind::Url, re(r"\]\([^)\n]*\)")),
    ]
});

/// Length of a fence opener (up to three spaces, then a run of 3 or more
/// backticks or tildes): `(marker, run length)`.
pub fn fence_open(t: &str) -> Option<(char, usize)> {
    let rest = strip_indent(t, 3)?;
    let c = rest.chars().next()?;
    if c != '`' && c != '~' {
        return None;
    }
    let n = rest.chars().take_while(|&x| x == c).count();
    (n >= 3).then_some((c, n))
}

/// Strip at most `max` leading spaces.
fn strip_indent(t: &str, max: usize) -> Option<&str> {
    let n = t.bytes().take_while(|&b| b == b' ').count();
    Some(&t[n.min(max)..])
}

pub fn is_fence_close(t: &str, marker: char, min: usize, crlf: bool) -> bool {
    let Some(rest) = strip_indent(t, 3) else {
        return false;
    };
    let n = rest.chars().take_while(|&x| x == marker).count();
    n >= min
        && rest[n..]
            .chars()
            .all(|c| c == ' ' || c == '\t' || (crlf && c == '\r'))
}

fn js_trim_end(s: &str) -> &str {
    s.trim_end_matches(|c: char| c.is_whitespace() || c == '\u{feff}')
}

fn has_indent4(t: &str) -> bool {
    t.starts_with("    ") || t.starts_with('\t')
}

fn block_zones(text: &str, lines: &[LineSpan], crlf: bool) -> Vec<Zone> {
    let mut zones = Vec::new();
    let last_end = lines.last().map_or(0, |l| l.end);
    let mut i = 0;
    if lines.len() > 1 && js_trim_end(lines[0].text(text)) == "---" {
        for (j, l) in lines.iter().enumerate().skip(1) {
            let t = js_trim_end(l.text(text));
            if t == "---" || t == "..." {
                zones.push(Zone {
                    start: 0,
                    end: l.end,
                    kind: ZoneKind::Frontmatter,
                });
                i = j + 1;
                break;
            }
        }
    }
    let mut prev_blank = true;
    while i < lines.len() {
        let line = lines[i];
        let t = line.text(text);
        let trimmed = t.trim();
        if let Some((marker, n)) = fence_open(t) {
            let mut j = i + 1;
            while j < lines.len() && !is_fence_close(lines[j].text(text), marker, n, crlf) {
                j += 1;
            }
            let end = if j < lines.len() {
                lines[j].end
            } else {
                last_end
            };
            zones.push(Zone {
                start: line.start,
                end,
                kind: ZoneKind::CodeFence,
            });
            i = j + 1;
            prev_blank = false;
            continue;
        }
        if let Some(after_open) = trimmed.strip_prefix("$$") {
            let close_in_line = after_open.find("$$").map(|p| p + 2);
            match close_in_line {
                None => {
                    let mut j = i + 1;
                    while j < lines.len() && !lines[j].text(text).trim().ends_with("$$") {
                        j += 1;
                    }
                    let end = if j < lines.len() {
                        lines[j].end
                    } else {
                        last_end
                    };
                    zones.push(Zone {
                        start: line.start,
                        end,
                        kind: ZoneKind::MathBlock,
                    });
                    i = j + 1;
                    prev_blank = false;
                    continue;
                }
                Some(p) if p == trimmed.len() - 2 => {
                    zones.push(Zone {
                        start: line.start,
                        end: line.end,
                        kind: ZoneKind::MathBlock,
                    });
                    i += 1;
                    prev_blank = false;
                    continue;
                }
                Some(_) => {}
            }
        }
        let delim_next = i + 1 < lines.len() && {
            let n = lines[i + 1].text(text);
            n.contains('|') && TABLE_DELIM_ROW.is_match(n).unwrap_or(false)
        };
        let starts_pipe = strip_indent(t, 3).is_some_and(|r| r.starts_with('|'));
        if starts_pipe || (!trimmed.is_empty() && t.contains('|') && delim_next) {
            let mut j = i;
            while j < lines.len()
                && lines[j].text(text).contains('|')
                && !lines[j].text(text).trim().is_empty()
            {
                j += 1;
            }
            // j > i is guaranteed only when line i contains '|'; guard the degenerate case.
            let end_line = if j > i { j - 1 } else { i };
            zones.push(Zone {
                start: line.start,
                end: lines[end_line].end,
                kind: ZoneKind::Table,
            });
            i = j.max(i + 1);
            prev_blank = false;
            continue;
        }
        if prev_blank && !trimmed.is_empty() && has_indent4(t) {
            let mut j = i;
            while j < lines.len()
                && (lines[j].text(text).trim().is_empty() || has_indent4(lines[j].text(text)))
            {
                j += 1;
            }
            while j > i && lines[j - 1].text(text).trim().is_empty() {
                j -= 1;
            }
            zones.push(Zone {
                start: line.start,
                end: lines[j - 1].end,
                kind: ZoneKind::IndentedCode,
            });
            i = j;
            prev_blank = false;
            continue;
        }
        prev_blank = trimmed.is_empty();
        i += 1;
    }
    zones
}

/// CommonMark code spans within one line: a run of N backticks closes only on
/// the next run of exactly N backticks. Offsets are line-relative.
pub fn inline_code_spans(t: &str) -> Vec<(usize, usize)> {
    let b = t.as_bytes();
    let mut spans = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] != b'`' {
            i += 1;
            continue;
        }
        let mut n = 1;
        while i + n < b.len() && b[i + n] == b'`' {
            n += 1;
        }
        let mut j = i + n;
        let mut closed = false;
        while j < b.len() {
            if b[j] != b'`' {
                j += 1;
                continue;
            }
            let mut m = 1;
            while j + m < b.len() && b[j + m] == b'`' {
                m += 1;
            }
            if m == n {
                spans.push((i, j + m));
                i = j + m;
                closed = true;
                break;
            }
            j += m;
        }
        if !closed {
            i += n;
        }
    }
    spans
}

/// End of the bare URL that starts at `start` in `t`: it stops at whitespace,
/// angle brackets, quotes, backticks and a link end `](`, then drops trailing
/// sentence punctuation and unbalanced closing brackets.
pub fn bare_url_end(t: &str, start: usize) -> usize {
    let mut end = t.len();
    for (i, c) in t[start..].char_indices() {
        let link_end = c == ']' && t[start + i + 1..].starts_with('(');
        if c.is_whitespace() || matches!(c, '<' | '>' | '"' | '\'' | '`') || link_end {
            end = start + i;
            break;
        }
    }
    loop {
        let u = &t[start..end];
        let Some(c) = u.chars().next_back() else {
            break;
        };
        let unbalanced = match c {
            ')' => u.matches(')').count() > u.matches('(').count(),
            ']' => u.matches(']').count() > u.matches('[').count(),
            _ => matches!(c, '.' | ',' | ';' | ':' | '!' | '?'),
        };
        if !unbalanced {
            break;
        }
        end -= c.len_utf8();
    }
    end
}

fn inline_zones(text: &str, lines: &[LineSpan], blocks: &[Zone], corrected: bool) -> Vec<Zone> {
    let mut zones = Vec::new();
    for line in lines {
        if line.start == line.end {
            continue;
        }
        if blocks
            .iter()
            .any(|z| z.start <= line.start && z.end >= line.end)
        {
            continue;
        }
        let lt = line.text(text);
        let mut local: Vec<Zone> = Vec::new();
        for (s, e) in inline_code_spans(lt) {
            let z = Zone {
                start: line.start + s,
                end: line.start + e,
                kind: ZoneKind::InlineCode,
            };
            local.push(z);
            zones.push(z);
        }
        for (kind, rx) in INLINE_PATTERNS.iter() {
            let mut pos = 0;
            while pos <= lt.len() {
                let Ok(Some(m)) = rx.find_from_pos(lt, pos) else {
                    break;
                };
                if m.start() == m.end() {
                    pos = next_char(lt, m.end());
                    continue;
                }
                let start = line.start + m.start();
                let mut end = line.start + m.end();
                if corrected && *kind == ZoneKind::Url && !lt[m.start()..].starts_with(']') {
                    end = end.min(line.start + bare_url_end(lt, m.start()));
                }
                if local.iter().any(|z| z.start < end && z.end > start) {
                    pos = next_char(lt, m.start());
                    continue;
                }
                let z = Zone {
                    start,
                    end,
                    kind: *kind,
                };
                local.push(z);
                zones.push(z);
                pos = m.end();
            }
        }
    }
    zones
}

fn next_char(s: &str, i: usize) -> usize {
    i + s[i..].chars().next().map_or(1, char::len_utf8)
}

/// Line-scan zones, sorted by start then end. A fence line ending in `\r` does
/// not close its fence, byte for byte like the TypeScript core.
pub fn scan_zones(text: &str, lines: &[LineSpan]) -> Vec<Zone> {
    scan_zones_with(text, lines, false)
}

/// Like [`scan_zones`]; with `corrected` a closing fence may end in `\r` (CRLF
/// documents close their fences) and a bare URL zone stops before trailing
/// sentence punctuation, closing quotes and an unbalanced closing bracket.
pub fn scan_zones_with(text: &str, lines: &[LineSpan], corrected: bool) -> Vec<Zone> {
    let blocks = block_zones(text, lines, corrected);
    let mut all = blocks.clone();
    all.extend(inline_zones(text, lines, &blocks, corrected));
    all.sort_by_key(|z| (z.start, z.end));
    all
}

/// True when `[start, end)` overlaps any zone (zones sorted by start).
pub fn in_zone(zones: &[Zone], start: usize, end: usize) -> bool {
    for z in zones {
        if z.start >= end {
            break;
        }
        if z.end > start {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corrected_close() {
        let text = "before\r\n```\r\ncode\r\n```\r\nafter\r\n";
        let lines = line_spans(text);
        let zones = scan_zones_with(text, &lines, true);
        let fence = zones
            .iter()
            .find(|z| z.kind == ZoneKind::CodeFence)
            .expect("fence zone");
        assert_eq!(&text[fence.start..fence.end], "```\r\ncode\r\n```\r");
        assert!(!in_zone(&zones, text.find("after").unwrap(), text.len()));
    }

    #[test]
    fn corrected_url_zone_stops_before_trailing_punctuation() {
        let text = "See https://example.com/b. Then \"https://x.y/z\".\n";
        let lines = line_spans(text);
        let urls = |corrected| -> Vec<&str> {
            scan_zones_with(text, &lines, corrected)
                .iter()
                .filter(|z| z.kind == ZoneKind::Url)
                .map(|z| &text[z.start..z.end])
                .collect()
        };
        assert_eq!(urls(true), ["https://example.com/b", "https://x.y/z"]);
        assert_eq!(urls(false)[0], "https://example.com/b.");
    }
}
