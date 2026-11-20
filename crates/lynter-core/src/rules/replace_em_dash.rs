use super::next_char;
use crate::document::Document;
use crate::edit::Edit;
use crate::rule::{Options, Rule};
use regex::Regex;
use serde_json::json;
use std::sync::LazyLock;

const EM: char = '\u{2014}';
const EN: char = '\u{2013}';

static DASH_RUN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("[ \\t]*[\u{2013}\u{2014}]+[ \\t]*").expect("regex"));

static ATX_OPENER: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^ {0,3}#{1,6}$").expect("regex"));

fn is_word(c: Option<char>) -> bool {
    c.is_some_and(|c| c.is_ascii_alphanumeric())
}

fn is_digit(c: Option<char>) -> bool {
    c.is_some_and(|c| c.is_ascii_digit())
}

/// Replace em dashes and connector en dashes with a configured separator.
pub struct ReplaceEmDash;

impl Rule for ReplaceEmDash {
    fn id(&self) -> &'static str {
        "replace-em-dash"
    }

    fn description(&self) -> &'static str {
        "Replace em dashes and connector en dashes with a configured separator (default comma plus space), preserving numeric en dash ranges."
    }

    fn default_options(&self) -> Options {
        json!({"replacement": ", ", "word-joining": false})
            .as_object()
            .cloned()
            .unwrap_or_default()
    }

    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit> {
        let replacement = options
            .get("replacement")
            .and_then(|v| v.as_str())
            .unwrap_or(", ");
        let word_joining = options.get("word-joining").and_then(|v| v.as_bool()) == Some(true);
        let text = doc.text();
        let mut edits = Vec::new();
        let mut pos = 0;
        while pos <= text.len() {
            let Some(m) = DASH_RUN.find_at(text, pos) else {
                break;
            };
            let (start, end) = (m.start(), m.end());
            if doc.is_protected(start, end) {
                pos = next_char(text, start);
                continue;
            }
            pos = end;
            // A dash right after an ATX marker would eat the marker's
            // separating space and turn the heading into a paragraph.
            let line_start = text[..start].rfind(['\n', '\r']).map_or(0, |i| i + 1);
            if ATX_OPENER.is_match(&text[line_start..start]) {
                continue;
            }
            let matched = m.as_str();
            let before = text[..start].chars().next_back();
            let after = text[end..].chars().next();
            if before.is_none() || before == Some('\n') {
                continue;
            }
            let dashes: String = matched
                .chars()
                .filter(|c| *c != ' ' && *c != '\t')
                .collect();
            if dashes.chars().eq([EN]) && is_digit(before) && is_digit(after) {
                continue;
            }
            let no_space = !matched.contains([' ', '\t']);
            let new: String = if word_joining && no_space && is_word(before) && is_word(after) {
                "-".to_owned()
            } else if after.is_none() || after == Some('\n') {
                replacement.trim_end_matches([' ', '\t']).to_owned()
            } else {
                replacement.to_owned()
            };
            if new != matched {
                edits.push(Edit::new(start, end, new, "dash replaced"));
            }
        }
        let _ = EM;
        edits
    }
}
