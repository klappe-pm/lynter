//! The document model: one parse, protected regions, and list item offsets.

use crate::zones::{in_zone, line_spans, scan_zones_with, LineSpan, Zone, ZoneKind};
use pulldown_cmark::{Event, Options, Parser, Tag};
use std::ops::Range;

/// A list item with its range corrected to start at the marker.
///
/// pulldown-cmark reports nested item ranges beginning in the parent's
/// indentation; `marker_start` skips that leading whitespace.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListItem {
    /// Range as reported by the parser.
    pub raw: Range<usize>,
    /// Byte offset of the list marker (`-`, `*`, `+`, or digits).
    pub marker_start: usize,
}

#[derive(Debug)]
pub struct Document {
    text: String,
    lines: Vec<LineSpan>,
    zones: Vec<Zone>,
    items: Vec<ListItem>,
}

fn parser_options() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_MATH
        | Options::ENABLE_WIKILINKS
        | Options::ENABLE_YAML_STYLE_METADATA_BLOCKS
}

/// Where protected regions come from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Protection {
    /// The line scan only. Matches the TypeScript core byte for byte.
    LineScan,
    /// The line scan plus code, math, table, metadata, and HTML ranges from
    /// the Markdown parse. Protection only ever grows.
    Parsed,
}

/// End of a parsed table: the Markdown parse reads a non-blank line after the
/// last row as one more row, so cut the range back to the last line that holds
/// a pipe. This agrees with the line scan's table zone.
fn table_end(text: &str, range: std::ops::Range<usize>) -> usize {
    let body = &text[range.start..range.end];
    let mut end = range.start;
    let mut pos = 0;
    for line in body.split_inclusive('\n') {
        let content = line.trim_end_matches(['\n', '\r']);
        if content.contains('|') && !content.trim().is_empty() {
            end = range.start + pos + content.len();
        }
        pos += line.len();
    }
    end.max(range.start)
}

impl Document {
    /// Parse once and compute every protected region and list item offset.
    pub fn parse(text: &str) -> Document {
        Document::parse_with(text, Protection::Parsed)
    }

    /// Like [`Document::parse`] with an explicit protection source.
    pub fn parse_with(text: &str, protection: Protection) -> Document {
        let lines = line_spans(text);
        let mut zones = scan_zones_with(text, &lines, protection == Protection::Parsed);
        let mut items = Vec::new();
        for (event, range) in Parser::new_ext(text, parser_options()).into_offset_iter() {
            let kind = match &event {
                Event::Start(Tag::CodeBlock(_))
                | Event::Start(Tag::MetadataBlock(_))
                | Event::Start(Tag::Table(_)) => Some(ZoneKind::ParsedBlock),
                Event::Start(Tag::HtmlBlock) | Event::Html(_) | Event::InlineHtml(_) => {
                    Some(ZoneKind::Html)
                }
                Event::Code(_) => Some(ZoneKind::InlineCode),
                Event::InlineMath(_) | Event::DisplayMath(_) => Some(ZoneKind::InlineMath),
                Event::Start(Tag::Item) => {
                    let skip = text[range.start..range.end]
                        .bytes()
                        .take_while(|&b| b == b' ' || b == b'\t')
                        .count();
                    items.push(ListItem {
                        raw: range.clone(),
                        marker_start: range.start + skip,
                    });
                    None
                }
                _ => None,
            };
            if let Some(kind) = kind {
                let mut range = range;
                if matches!(event, Event::Start(Tag::Table(_))) {
                    range.end = table_end(text, range.clone());
                }
                if protection == Protection::Parsed && range.start < range.end {
                    zones.push(Zone {
                        start: range.start,
                        end: range.end,
                        kind,
                    });
                }
            }
        }
        zones.sort_by_key(|z| (z.start, z.end));
        Document {
            text: text.to_owned(),
            lines,
            zones,
            items,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn lines(&self) -> &[LineSpan] {
        &self.lines
    }

    pub fn zones(&self) -> &[Zone] {
        &self.zones
    }

    pub fn list_items(&self) -> &[ListItem] {
        &self.items
    }

    /// True when `[start, end)` overlaps a protected region.
    pub fn is_protected(&self, start: usize, end: usize) -> bool {
        in_zone(&self.zones, start, end)
    }

    /// True when the line lies inside a whole-line protected region.
    pub fn is_line_block_masked(&self, line: &LineSpan) -> bool {
        self.zones
            .iter()
            .any(|z| z.kind.is_block() && z.start <= line.start && z.end >= line.end)
    }

    /// 1-based line number of a byte offset.
    pub fn line_of(&self, offset: usize) -> usize {
        self.lines.partition_point(|l| l.start <= offset).max(1)
    }
}
