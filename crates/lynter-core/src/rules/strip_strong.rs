use super::next_char;
use crate::document::Document;
use crate::edit::Edit;
use crate::rule::{Options, Rule};
use fancy_regex::Regex;
use serde_json::json;
use std::sync::LazyLock;

static STRONG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(\*\*|__)(?!\s)([\s\S]*?\S)\1").expect("regex"));
static HEADING_LINE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"^ {0,3}#{1,6}(\s|$)").expect("regex"));
static BLANK_GAP: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"\n[ \t]*\n").expect("regex"));

/// True when deleting the opening delimiter at `start` would leave block syntax at
/// the start of its line (a code fence, a math fence, a table row, or a heading),
/// which would change the document structure instead of only removing emphasis.
fn exposes_block_start(text: &str, start: usize, delim_len: usize) -> bool {
    let line_start = text[..start].rfind('\n').map_or(0, |p| p + 1);
    if !text[line_start..start].chars().all(|c| c == ' ') {
        return false;
    }
    let rest = &text[start + delim_len..];
    let line = rest.split('\n').next().unwrap_or("");
    let probe = format!("{}{}", &text[line_start..start], line);
    crate::zones::fence_open(&probe).is_some()
        || probe.trim_start().starts_with("$$")
        || probe.trim_start().starts_with('|')
        || HEADING_LINE.is_match(&probe)
}

fn is_word_char(c: Option<char>) -> bool {
    c.is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Remove `**` and `__` strong emphasis, keeping the inner text.
pub struct StripStrong;

impl Rule for StripStrong {
    fn id(&self) -> &'static str {
        "strip-strong"
    }

    fn description(&self) -> &'static str {
        "Remove ** and __ strong emphasis, keeping the inner text; never edits inside code, math, YAML, tables, wikilinks, or URLs."
    }

    fn default_options(&self) -> Options {
        json!({"keep-heading-strong": false})
            .as_object()
            .cloned()
            .unwrap_or_default()
    }

    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit> {
        let keep_heading =
            options.get("keep-heading-strong").and_then(|v| v.as_bool()) == Some(true);
        let text = doc.text();
        let heading_lines: Vec<_> = if keep_heading {
            doc.lines()
                .iter()
                .filter(|l| HEADING_LINE.is_match(l.text(text)))
                .collect()
        } else {
            Vec::new()
        };
        let on_heading = |i: usize| heading_lines.iter().any(|l| l.start <= i && i <= l.end);
        let mut edits = Vec::new();
        let mut pos = 0;
        while pos <= text.len() {
            let Ok(Some(caps)) = STRONG.captures_from_pos(text, pos) else {
                break;
            };
            let whole = caps.get(0).expect("group 0");
            let (start, end) = (whole.start(), whole.end());
            let delim = caps.get(1).expect("group 1").as_str();
            let content = caps.get(2).expect("group 2").as_str();
            let skip = doc.is_protected(start, start + delim.len())
                || doc.is_protected(end - delim.len(), end)
                || BLANK_GAP.is_match(content)
                || exposes_block_start(text, start, delim.len())
                || (delim == "__"
                    && (is_word_char(text[..start].chars().next_back())
                        || is_word_char(text[end..].chars().next())))
                || (keep_heading && on_heading(start));
            if skip {
                pos = next_char(text, start);
                continue;
            }
            // Delete only the two delimiters, so the inner text is never part of an edit.
            edits.push(Edit::new(
                start,
                start + delim.len(),
                "",
                "strong emphasis removed",
            ));
            edits.push(Edit::new(
                end - delim.len(),
                end,
                "",
                "strong emphasis removed",
            ));
            pos = end;
        }
        edits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_delimiters_that_hide_a_code_fence() {
        let doc = Document::parse("__```\n__under__ score\n```\nIt is\n");
        let edits = StripStrong.check(&doc, &StripStrong.default_options());
        assert!(edits.iter().all(|e| e.start != 0), "{edits:?}");
    }
}
