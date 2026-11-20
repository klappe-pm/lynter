use super::util::{is_masked_line, js_pattern, js_trim};
use crate::document::Document;
use crate::edit::Edit;
use crate::rule::{Options, Rule};
use regex::Regex as Re;
use std::sync::LazyLock;

fn re(p: &str) -> Re {
    Re::new(&js_pattern(p)).expect("regex")
}

/// A line ending in two or more spaces, a `<br>` tag, or a trailing
/// backslash is an intentional hard break.
pub static INTENTIONAL_BREAK: LazyLock<Re> =
    LazyLock::new(|| re(r"(?i)( {2,}|<br[ \t]*/?>[ \t]*|\\)$"));
static SETEXT: LazyLock<Re> = LazyLock::new(|| re(r"^ {0,3}(=+|-+)[ \t]*$"));
static HEADING: LazyLock<Re> = LazyLock::new(|| re(r"^#{1,6}\s"));
static LIST: LazyLock<Re> = LazyLock::new(|| re(r"^([-*+]|[0-9]{1,9}[.)])(\s|$)"));
static STARS: LazyLock<Re> = LazyLock::new(|| re(r"^(\*[ \t]*){3,}$"));
static UNDERSCORES: LazyLock<Re> = LazyLock::new(|| re(r"^(_[ \t]*){3,}$"));
static FENCE: LazyLock<Re> = LazyLock::new(|| re(r"^(`{3,}|~{3,})"));
static REF_DEF: LazyLock<Re> = LazyLock::new(|| re(r"^\[[^\]]*\]:"));
static WIKI_LINE: LazyLock<Re> = LazyLock::new(|| re(r"^!?\[\[[^\]]+\]\][ \t]*$"));

pub fn is_joinable_prose(line: &str) -> bool {
    if js_trim(line).is_empty() {
        return false;
    }
    if line.chars().next().is_some_and(super::util::is_js_ws) {
        return false;
    }
    !(HEADING.is_match(line)
        || line.starts_with('>')
        || LIST.is_match(line)
        || line.starts_with('|')
        || SETEXT.is_match(line)
        || STARS.is_match(line)
        || UNDERSCORES.is_match(line)
        || FENCE.is_match(line)
        || line.starts_with("$$")
        || REF_DEF.is_match(line)
        || WIKI_LINE.is_match(line)
        || line.starts_with('<'))
}

fn join_edits(doc: &Document) -> Vec<Edit> {
    let text = doc.text();
    let spans = doc.lines();
    let lines: Vec<&str> = spans.iter().map(|l| l.text(text)).collect();
    let mut edits = Vec::new();
    // End of the current joined line's content, with trailing blanks trimmed.
    let mut cur_end: Option<usize> = None;
    let mut can_extend = false;
    for i in 0..lines.len() {
        let next_setext = i + 1 < lines.len() && SETEXT.is_match(lines[i + 1]);
        let joinable =
            !is_masked_line(doc, &spans[i]) && is_joinable_prose(lines[i]) && !next_setext;
        // Parsed protection also covers multi-line inline math and the like.
        let separator_free = cur_end.is_none_or(|from| !doc.is_protected(from, spans[i].start));
        if cur_end.is_some() && can_extend && joinable && separator_free {
            // The separator: trailing blanks, the newline, and nothing else.
            let from = cur_end.unwrap_or(0);
            edits.push(Edit::new(
                from,
                spans[i].start,
                " ",
                "hard-wrapped lines joined",
            ));
            can_extend = !INTENTIONAL_BREAK.is_match(lines[i]);
        } else {
            can_extend = joinable && !INTENTIONAL_BREAK.is_match(lines[i]);
        }
        cur_end = Some(spans[i].start + lines[i].trim_end_matches([' ', '\t']).len());
    }
    edits
}

/// Report-only twin: flags hard-wrapped prose.
pub struct JoinParagraphLines;
/// Destructive twin: unwraps hard-wrapped prose.
pub struct JoinParagraphLinesFix;

impl Rule for JoinParagraphLines {
    fn id(&self) -> &'static str {
        "join-paragraph-lines"
    }
    fn description(&self) -> &'static str {
        "Flag hard-wrapped prose lines inside a paragraph that could be unwrapped into a single line; enable join-paragraph-lines-fix to apply the rewrite."
    }
    fn default_options(&self) -> Options {
        Options::new()
    }
    fn check(&self, doc: &Document, _options: &Options) -> Vec<Edit> {
        join_edits(doc)
    }
}

impl Rule for JoinParagraphLinesFix {
    fn id(&self) -> &'static str {
        "join-paragraph-lines-fix"
    }
    fn description(&self) -> &'static str {
        "Unwrap hard-wrapped prose lines inside a paragraph into a single line, leaving lists, blockquotes, tables, headings, code, math, YAML, and intentional breaks untouched; destructive opt-in twin of join-paragraph-lines."
    }
    fn default_options(&self) -> Options {
        Options::new()
    }
    fn check(&self, doc: &Document, _options: &Options) -> Vec<Edit> {
        join_edits(doc)
    }
}
