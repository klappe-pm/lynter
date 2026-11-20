use super::util::{diff_edit, is_masked_line, js_pattern, js_trim};
use crate::document::Document;
use crate::edit::Edit;
use crate::rule::{Options, Rule};
use fancy_regex::Regex;
use serde_json::json;
use std::sync::LazyLock;

static BULLET: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(&js_pattern(r"^(\s*)([-*+])(\s+)(\SDOT*)$")).expect("regex")
});
static THEMATIC_BREAK: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&js_pattern(r"^\s*([-*_])(\s*\1){2,}\s*$")).expect("regex"));

/// Normalize unordered list bullets to one marker and tighten lists.
pub struct ListStyle;

impl Rule for ListStyle {
    fn id(&self) -> &'static str {
        "list-style"
    }

    fn description(&self) -> &'static str {
        "Normalize unordered list bullets to a single marker (\"- \" or \"* \") and, by default, remove blank lines between items; ordered lists, thematic breaks, and masked blocks are untouched."
    }

    fn default_options(&self) -> Options {
        json!({"marker": "-", "tighten": true})
            .as_object()
            .cloned()
            .unwrap_or_default()
    }

    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit> {
        let marker = if options.get("marker").and_then(|v| v.as_str()) == Some("*") {
            "*"
        } else {
            "-"
        };
        let tighten = options.get("tighten").and_then(|v| v.as_bool()) != Some(false);
        let text = doc.text();
        let lines: Vec<&str> = doc.lines().iter().map(|l| l.text(text)).collect();
        let masked: Vec<bool> = doc.lines().iter().map(|l| is_masked_line(doc, l)).collect();
        let is_bullet = |i: usize| {
            !masked[i]
                && !THEMATIC_BREAK.is_match(lines[i]).unwrap_or(false)
                && BULLET.is_match(lines[i])
        };
        let mut rendered: Vec<Option<String>> = (0..lines.len())
            .map(|i| {
                if !is_bullet(i) {
                    return Some(lines[i].to_owned());
                }
                let m = BULLET.captures(lines[i]).expect("bullet");
                Some(format!("{}{} {}", &m[1], marker, &m[4]))
            })
            .collect();
        if tighten {
            for i in 0..lines.len() {
                if rendered[i].is_some() && js_trim(lines[i]).is_empty() && !masked[i] {
                    let prev = (0..i).rev().find(|&k| {
                        rendered[k]
                            .as_deref()
                            .is_some_and(|r| !js_trim(r).is_empty())
                    });
                    let next = (i + 1..lines.len())
                        .find(|&k| rendered[k].is_some() && !js_trim(lines[k]).is_empty());
                    if let (Some(p), Some(n)) = (prev, next) {
                        if is_bullet(p)
                            && is_bullet(n)
                            && (p + 1..n).all(|k| !masked[k] && js_trim(lines[k]).is_empty())
                        {
                            rendered[i] = None;
                        }
                    }
                }
            }
        }
        let out = rendered
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join("\n");
        diff_edit(text, &out, "list style normalized")
    }
}
