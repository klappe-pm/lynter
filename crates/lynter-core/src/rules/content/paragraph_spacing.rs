use super::util::{diff_edit, js_trim};
use crate::document::Document;
use crate::edit::Edit;
use crate::rule::{Options, Rule};
use serde_json::{json, Value};

fn clamp_count(v: Option<&Value>) -> usize {
    let n = match v {
        Some(Value::Number(n)) => n.as_f64(),
        Some(Value::String(s)) => {
            let t = js_trim(s);
            if t.is_empty() {
                Some(0.0)
            } else {
                t.parse::<f64>().ok()
            }
        }
        Some(Value::Bool(b)) => Some(f64::from(u8::from(*b))),
        Some(Value::Null) => Some(0.0),
        _ => None,
    };
    match n {
        Some(n) if n.is_finite() => (n + 0.5).floor().clamp(0.0, 2.0) as usize,
        _ => 1,
    }
}

/// Normalize blank lines between blocks to a fixed count.
pub struct ParagraphSpacing;

impl Rule for ParagraphSpacing {
    fn id(&self) -> &'static str {
        "paragraph-spacing"
    }

    fn description(&self) -> &'static str {
        "Normalize blank lines between blocks to a fixed count (0, 1, or 2); trims leading and trailing blank lines and preserves blank lines inside code, math, tables, and frontmatter."
    }

    fn default_options(&self) -> Options {
        json!({"blank-lines": 1})
            .as_object()
            .cloned()
            .unwrap_or_default()
    }

    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit> {
        let count = clamp_count(options.get("blank-lines"));
        let text = doc.text();
        let mut out: Vec<&str> = Vec::new();
        let mut blank_run = 0;
        let mut seen_content = false;
        for line in doc.lines() {
            let t = line.text(text);
            let masked = doc.is_line_block_masked(line);
            if !masked && js_trim(t).is_empty() {
                blank_run += 1;
                continue;
            }
            if seen_content && blank_run > 0 {
                out.extend(std::iter::repeat_n("", count));
            }
            blank_run = 0;
            out.push(t);
            seen_content = true;
        }
        let mut result = out.join("\n");
        // The TypeScript rule appends a newline here even when a masked block
        // running to the end already supplied one, so its output grows on
        // every pass. Rust adds it only when missing.
        if text.ends_with('\n') && !result.is_empty() && !result.ends_with('\n') {
            result.push('\n');
        }
        diff_edit(text, &result, "blank lines normalized")
    }
}
