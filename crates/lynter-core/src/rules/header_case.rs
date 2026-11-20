use crate::document::Document;
use crate::edit::Edit;
use crate::rule::{Options, Rule};
use regex::Regex;
use serde_json::json;
use std::sync::LazyLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeaderCaseStyle {
    CamelCase,
    FirstLetter,
    KebabCase,
    TitleCase,
    UnderscoreFormatted,
}

impl HeaderCaseStyle {
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "camelCase" => Self::CamelCase,
            "First letter" => Self::FirstLetter,
            "kebab-case" => Self::KebabCase,
            "Title Case" => Self::TitleCase,
            "underscore_formatted" => Self::UnderscoreFormatted,
            _ => return None,
        })
    }
}

static CAMEL_BOUNDARY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("([a-z0-9])([A-Z])").expect("regex"));
static WORD_SPLIT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[\s_-]+").expect("regex"));
static ATX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(\s{0,3})(#{1,6})(\s+)([^\n\r\u{2028}\u{2029}]*?)(\s*)$").expect("regex")
});

fn to_words(text: &str) -> Vec<String> {
    let spaced = CAMEL_BOUNDARY.replace_all(text, "$1 $2");
    WORD_SPLIT
        .split(&spaced)
        .map(|w| w.trim().to_owned())
        .filter(|w| !w.is_empty())
        .collect()
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase(),
    }
}

/// Format one heading's text (the part after the `#` markers) into a style.
pub fn format_heading_text(text: &str, style: HeaderCaseStyle) -> String {
    let words = to_words(text);
    if words.is_empty() {
        return text.trim().to_owned();
    }
    match style {
        HeaderCaseStyle::CamelCase => words
            .iter()
            .enumerate()
            .map(|(i, w)| {
                if i == 0 {
                    w.to_lowercase()
                } else {
                    capitalize(w)
                }
            })
            .collect(),
        HeaderCaseStyle::FirstLetter => capitalize(
            &words
                .iter()
                .map(|w| w.to_lowercase())
                .collect::<Vec<_>>()
                .join(" "),
        ),
        HeaderCaseStyle::KebabCase => words
            .iter()
            .map(|w| w.to_lowercase())
            .collect::<Vec<_>>()
            .join("-"),
        HeaderCaseStyle::TitleCase => words
            .iter()
            .map(|w| capitalize(w))
            .collect::<Vec<_>>()
            .join(" "),
        HeaderCaseStyle::UnderscoreFormatted => words
            .iter()
            .map(|w| w.to_lowercase())
            .collect::<Vec<_>>()
            .join("_"),
    }
}

/// Normalize each ATX heading level to a chosen case style.
pub struct HeaderCase;

impl Rule for HeaderCase {
    fn id(&self) -> &'static str {
        "header-case"
    }

    fn description(&self) -> &'static str {
        "Normalize each ATX heading level to a chosen case style (camelCase, First letter, kebab-case, Title Case, underscore_formatted); levels left unset are untouched, and headings in code, math, tables, or frontmatter are never edited."
    }

    fn default_options(&self) -> Options {
        json!({"h1": "", "h2": "", "h3": "", "h4": "", "h5": "", "h6": ""})
            .as_object()
            .cloned()
            .unwrap_or_default()
    }

    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit> {
        let style_for = |level: usize| {
            options
                .get(&format!("h{level}"))
                .and_then(|v| v.as_str())
                .and_then(HeaderCaseStyle::parse)
        };
        if !(1..=6).any(|l| style_for(l).is_some()) {
            return Vec::new();
        }
        let text = doc.text();
        let mut edits = Vec::new();
        for span in doc.lines() {
            if doc.is_line_block_masked(span) {
                continue;
            }
            let line = span.text(text);
            // A lone carriage return inside a line is a line break to the
            // Markdown parse; what follows may be a different block.
            if line.trim_end_matches('\r').contains('\r') {
                continue;
            }
            let Some(caps) = ATX.captures(line) else {
                continue;
            };
            let Some(style) = style_for(caps[2].len()) else {
                continue;
            };
            let body = &caps[4];
            if body.is_empty() {
                continue;
            }
            let formatted = format_heading_text(body, style);
            if formatted == body {
                continue;
            }
            let rebuilt = format!("{}{}{}{}", &caps[1], &caps[2], &caps[3], formatted);
            edits.push(Edit::new(span.start, span.end, rebuilt, "heading case"));
        }
        edits
    }
}
