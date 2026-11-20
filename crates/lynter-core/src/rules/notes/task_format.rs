use super::{opt_bool, opt_list, opt_str, valid_ymd, Reports};
use crate::document::Document;
use crate::edit::Edit;
use crate::rule::{Options, Rule};
use regex::Regex;
use serde_json::json;
use std::sync::LazyLock;

static TASK: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^([ \t]*(?:[-*+]|\d{1,9}[.)]))([ \t]+)\[([^\]\r\n])\]").expect("regex")
});
static DATE_TOKEN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        "(\u{1F6EB}|\u{23F3}|\u{1F4C5}|\u{2705})(\u{FE0F}?)[ \\t]*(?:(\\d{4})[-/.](\\d{1,2})[-/.](\\d{1,2})|(\\d{4})(\\d{2})(\\d{2}))",
    )
    .expect("regex")
});

fn kind_of(emoji: &str) -> &'static str {
    match emoji {
        "\u{1F6EB}" => "start",
        "\u{23F3}" => "scheduled",
        "\u{1F4C5}" => "due",
        _ => "done",
    }
}

struct Token {
    start: usize,
    end: usize,
    rank: usize,
    text: String,
}

/// Normalize Obsidian task checkboxes and task dates.
pub struct TaskFormat;

impl Rule for TaskFormat {
    fn id(&self) -> &'static str {
        "task-format"
    }

    fn description(&self) -> &'static str {
        "Normalize task checkbox spacing and marker case, restrict status characters, write due, scheduled, start, and done dates as yyyy-mm-dd, and keep date tokens in a consistent order."
    }

    fn default_options(&self) -> Options {
        json!({
            "normalize-spacing": true,
            "lowercase-marker": true,
            "allowed-statuses": "",
            "date-format": true,
            "date-order": ["start", "scheduled", "due", "done"],
        })
        .as_object()
        .cloned()
        .unwrap_or_default()
    }

    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit> {
        let norm = opt_bool(options, "normalize-spacing", true);
        let lower = opt_bool(options, "lowercase-marker", true);
        let allowed = opt_str(options, "allowed-statuses").unwrap_or("");
        let fmt = opt_bool(options, "date-format", true);
        let order = opt_list(options, "date-order");
        let text = doc.text();
        let mut edits = Vec::new();
        let mut reports = Reports::default();
        for line in doc.lines() {
            if doc.is_line_block_masked(line) {
                continue;
            }
            let lt = line.text(text);
            let Some(c) = TASK.captures(lt) else {
                continue;
            };
            let (Some(m1), Some(m2), Some(m3)) = (c.get(1), c.get(2), c.get(3)) else {
                continue;
            };
            let whole_end = c.get(0).map_or(0, |m| m.end());
            let rest = &lt[whole_end..];
            let status = m3.as_str();
            let status_ch = status.chars().next().unwrap_or(' ');
            if !rest.is_empty()
                && !rest.starts_with([' ', '\t', '\r'])
                && !matches!(status_ch, ' ' | 'x' | 'X')
            {
                continue;
            }
            if rest.starts_with(['(', '[', ':']) {
                continue;
            }
            let base = line.start;
            let new_status = if lower && status_ch == 'X' {
                "x".to_owned()
            } else {
                status.to_owned()
            };
            let ws2_len = rest.len() - rest.trim_start_matches([' ', '\t']).len();
            let has_content = !rest.trim().is_empty();
            let ws1 = if norm { " " } else { m2.as_str() };
            let ws2 = if has_content && norm {
                " "
            } else if has_content {
                &rest[..ws2_len]
            } else {
                ""
            };
            let start = base + m1.end();
            let end = base + whole_end + if has_content { ws2_len } else { 0 };
            let new_prefix = format!("{ws1}[{new_status}]{ws2}");
            if !doc.is_protected(start, end) && text[start..end] != new_prefix {
                edits.push(Edit::new(
                    start,
                    end,
                    new_prefix,
                    "task checkbox normalized",
                ));
            }
            let cs = new_status.chars().next().unwrap_or(' ');
            if !allowed.is_empty() && !allowed.contains(cs) {
                reports.push(
                    base + m3.start(),
                    base + m3.end(),
                    format!("task status \"{cs}\" is not in the allowed statuses \"{allowed}\""),
                );
            }
            if !has_content || (!fmt && order.is_empty()) {
                continue;
            }
            let content_start = base + whole_end + ws2_len;
            let mut toks: Vec<Token> = Vec::new();
            let content = &text[content_start..line.end];
            for m in DATE_TOKEN.captures_iter(content) {
                let (Some(all), Some(em), Some(fe)) = (m.get(0), m.get(1), m.get(2)) else {
                    continue;
                };
                let num = |i: usize| m.get(i).and_then(|x| x.as_str().parse::<u32>().ok());
                let (y, mo, d) = if m.get(3).is_some() {
                    (num(3), num(4), num(5))
                } else {
                    (num(6), num(7), num(8))
                };
                let (Some(y), Some(mo), Some(d)) = (y, mo, d) else {
                    continue;
                };
                if !valid_ymd(y, mo, d) {
                    continue;
                }
                let rest_after = &content[all.end()..];
                if rest_after.starts_with(|ch: char| ch.is_ascii_digit()) {
                    continue;
                }
                let s = content_start + all.start();
                let e = content_start + all.end();
                if doc.is_protected(s, e) {
                    continue;
                }
                let kind = kind_of(em.as_str());
                let new_text = if fmt {
                    format!("{}{} {y:04}-{mo:02}-{d:02}", em.as_str(), fe.as_str())
                } else {
                    all.as_str().to_owned()
                };
                let rank = order.iter().position(|o| o == kind).unwrap_or(order.len());
                toks.push(Token {
                    start: s,
                    end: e,
                    rank,
                    text: new_text,
                });
            }
            let mut sorted: Vec<&Token> = toks.iter().collect();
            sorted.sort_by_key(|t| t.rank);
            for (slot, src) in toks.iter().zip(sorted) {
                if text[slot.start..slot.end] != src.text {
                    edits.push(Edit::new(
                        slot.start,
                        slot.end,
                        src.text.clone(),
                        "task date normalized",
                    ));
                }
            }
        }
        edits.extend(reports.into_edits(text));
        edits
    }
}
