use super::join_paragraph_lines::{is_joinable_prose, INTENTIONAL_BREAK};
use super::util::{is_masked_line, js_pattern, js_trim};
use crate::document::Document;
use crate::edit::Edit;
use crate::rule::{Options, Rule};
use fancy_regex::Regex;
use std::sync::LazyLock;

static SIMPLE_ITEM: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&js_pattern(r"^([-*+]) (?!\[[ xX]\](?: |$))(\SDOT*)$")).expect("regex")
});
static LIST_LIKE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"^[ \t]*([-*+]|[0-9]{1,9}[.)])([ \t]|$)").expect("regex"));
static TRAILING_PUNCT: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"[.,;]+$").expect("regex"));
static SENTENCE_END: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"[.!?]$").expect("regex"));

fn simple_item(line: &str) -> Option<(String, String)> {
    let caps = SIMPLE_ITEM.captures(line).ok().flatten()?;
    Some((caps[1].to_owned(), caps[2].to_owned()))
}

/// The item text without trailing `.,;` and blanks.
fn item_part(text: &str) -> &str {
    let end = TRAILING_PUNCT.find(text).map_or(text.len(), |m| m.start());
    js_trim(&text[..end])
}

fn convert_edits(doc: &Document) -> Vec<Edit> {
    let text = doc.text();
    let spans = doc.lines();
    let lines: Vec<&str> = spans.iter().map(|l| l.text(text)).collect();
    let masked = |i: usize| is_masked_line(doc, &spans[i]);
    let msg = "list can be written as a sentence";
    let mut edits: Vec<Edit> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let first = if masked(i) {
            None
        } else {
            simple_item(lines[i])
        };
        let Some((marker, _)) = first else {
            i += 1;
            continue;
        };
        let mut items: Vec<String> = Vec::new();
        let mut j = i;
        while j < lines.len() && !masked(j) {
            match simple_item(lines[j]) {
                Some((m, body)) if m == marker => items.push(js_trim(&body).to_owned()),
                _ => break,
            }
            j += 1;
        }
        let prev = (i > 0).then(|| lines[i - 1]);
        let term = (j < lines.len()).then(|| lines[j]);
        let valid = items.len() >= 2
            && term.is_none_or(|t| js_trim(t).is_empty())
            && prev.is_none_or(|p| {
                js_trim(p).is_empty() || (!LIST_LIKE.is_match(p) && !p.starts_with([' ', '\t']))
            })
            && items
                .iter()
                .all(|t| !t.is_empty() && !LIST_LIKE.is_match(t));
        if !valid {
            while i < lines.len() && !js_trim(lines[i]).is_empty() {
                i += 1;
            }
            continue;
        }
        let mut block: Vec<Edit> = Vec::new();
        // Start of each item's text (after "- ") and the end of its kept part.
        let starts: Vec<usize> = (i..j).map(|k| spans[k].start + 2).collect();
        let ends: Vec<usize> = items
            .iter()
            .zip(&starts)
            .map(|(t, s)| s + item_part(t).len())
            .collect();
        let n = items.len();
        let prev_masked = i > 0 && masked(i - 1);
        let merge_prev = prev.is_some_and(|p| {
            !js_trim(p).is_empty()
                && !prev_masked
                && is_joinable_prose(p)
                && !INTENTIONAL_BREAK.is_match(p)
        });
        if merge_prev {
            let from = spans[i - 1].start + lines[i - 1].trim_end_matches([' ', '\t']).len();
            block.push(Edit::new(from, starts[0], " ", msg));
        } else {
            block.push(Edit::new(spans[i].start, starts[0], "", msg));
        }
        for k in 0..n - 1 {
            let sep = if n == 2 {
                " and "
            } else if k + 1 == n - 1 {
                ", and "
            } else {
                ", "
            };
            block.push(Edit::new(ends[k], starts[k + 1], sep, msg));
        }
        let last_part = item_part(&items[n - 1]);
        let tail = if SENTENCE_END.is_match(last_part) {
            ""
        } else {
            "."
        };
        let last_end = spans[j - 1].end;
        if !(ends[n - 1] == last_end && tail.is_empty()) {
            block.push(Edit::new(ends[n - 1], last_end, tail, msg));
        }
        let touches_protected = block
            .iter()
            .any(|e| e.start < e.end && doc.is_protected(e.start, e.end));
        if !touches_protected {
            edits.extend(block);
        }
        i = j;
    }
    edits
}

/// Report-only twin: flags simple bullet lists.
pub struct ProseListToSentences;
/// Destructive twin: converts simple bullet lists to sentences.
pub struct ProseListToSentencesFix;

impl Rule for ProseListToSentences {
    fn id(&self) -> &'static str {
        "prose-list-to-sentences"
    }
    fn description(&self) -> &'static str {
        "Flag simple one-level bullet lists that could be comma-joined prose; enable prose-list-to-sentences-fix to apply the rewrite."
    }
    fn default_options(&self) -> Options {
        Options::new()
    }
    fn check(&self, doc: &Document, _options: &Options) -> Vec<Edit> {
        convert_edits(doc)
    }
}

impl Rule for ProseListToSentencesFix {
    fn id(&self) -> &'static str {
        "prose-list-to-sentences-fix"
    }
    fn description(&self) -> &'static str {
        "Convert simple one-level bullet lists into comma-joined prose sentences; destructive opt-in twin of prose-list-to-sentences."
    }
    fn default_options(&self) -> Options {
        Options::new()
    }
    fn check(&self, doc: &Document, _options: &Options) -> Vec<Edit> {
        convert_edits(doc)
    }
}
