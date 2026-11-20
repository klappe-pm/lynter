//! Blank-line rules: collapsing runs, blank lines around blocks, and the
//! document ending.

use super::common::{
    bool_opt, doc_eol, insert_line, is_dash_underline, is_hr, is_paragraph_text, prot, prot_ins,
    scan, sep_line, Info, Kind, Scan,
};
use crate::document::Document;
use crate::edit::Edit;
use crate::rule::{Options, Rule};
use crate::rules::content::util::js_trim;
use crate::zones::ZoneKind;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

fn opts(v: Value) -> Options {
    v.as_object().cloned().unwrap_or_default()
}

fn blankish(l: &Info) -> bool {
    l.kind == Kind::Blank || (l.kind == Kind::Skip && js_trim(l.text).is_empty())
}

/// Collects the blank lines to add between line pairs and the byte ranges to drop.
#[derive(Default)]
struct Plan {
    ins: BTreeMap<usize, String>,
    del: BTreeSet<(usize, usize)>,
}

impl Plan {
    /// Ensure a blank line between line `p` and line `p + 1`.
    fn ensure(&mut self, lines: &[Info], p: usize) {
        let (Some(a), Some(b)) = (lines.get(p), lines.get(p + 1)) else {
            return;
        };
        if blankish(a) || blankish(b) {
            return;
        }
        self.ins.insert(p + 1, sep_line(a.depth.min(b.depth)));
    }

    /// Reduce the run of blank lines that ends at `last` (when it is blank)
    /// or starts at `first`, to one line.
    fn collapse_run(&mut self, lines: &[Info], a: usize, b: usize) {
        if b > a {
            self.del.insert((lines[a + 1].start, lines[b].next));
        }
    }

    fn collapse_before(&mut self, lines: &[Info], i: usize) {
        let mut a = i;
        while a > 0 && lines[a - 1].kind == Kind::Blank {
            a -= 1;
        }
        if a < i {
            self.collapse_run(lines, a, i - 1);
        }
    }

    fn collapse_after(&mut self, lines: &[Info], i: usize) {
        let mut b = i;
        while b + 1 < lines.len() && lines[b + 1].kind == Kind::Blank {
            b += 1;
        }
        if b > i {
            self.collapse_run(lines, i + 1, b);
        }
    }

    fn edits(self, doc: &Document, sc: &Scan, message: &str) -> Vec<Edit> {
        let lines = &sc.lines;
        let mut out = Vec::new();
        for (idx, sep) in self.ins {
            let at = lines[idx].start;
            if prot_ins(doc, at) {
                continue;
            }
            out.push(insert_line(at, &sep, lines[idx - 1].eol, message));
        }
        for (s, e) in self.del {
            if !prot(doc, s, e) {
                out.push(Edit::new(s, e, "", message));
            }
        }
        out
    }
}

macro_rules! unit_rule {
    ($name:ident, $id:literal) => {
        pub struct $name;
        impl $name {
            const ID: &'static str = $id;
        }
    };
}

unit_rule!(ConsecutiveBlankLines, "consecutive-blank-lines");

impl Rule for ConsecutiveBlankLines {
    fn id(&self) -> &'static str {
        Self::ID
    }

    fn description(&self) -> &'static str {
        "Collapse runs of blank lines outside code, math, and frontmatter to a single blank line."
    }

    fn default_options(&self) -> Options {
        Options::new()
    }

    fn check(&self, doc: &Document, _options: &Options) -> Vec<Edit> {
        let sc = scan(doc, 2);
        let lines = &sc.lines;
        let is_blank = |l: &Info| l.kind == Kind::Blank && l.depth == 0;
        let mut out = Vec::new();
        let mut i = 0;
        while i < lines.len() {
            if !is_blank(&lines[i]) {
                i += 1;
                continue;
            }
            let mut j = i;
            while j + 1 < lines.len() && is_blank(&lines[j + 1]) {
                j += 1;
            }
            if j > i {
                let (s, e) = (lines[i].start, lines[j].next);
                let keep = lines[i].eol;
                if &doc.text()[s..e] != keep && !prot(doc, s, e) {
                    out.push(Edit::new(s, e, keep, "consecutive blank lines collapsed"));
                }
            }
            i = j + 1;
        }
        out
    }
}

unit_rule!(EmptyLineAroundBlockquotes, "empty-line-around-blockquotes");

impl Rule for EmptyLineAroundBlockquotes {
    fn id(&self) -> &'static str {
        Self::ID
    }

    fn description(&self) -> &'static str {
        "Ensure a blank line before and after blockquotes and callouts."
    }

    fn default_options(&self) -> Options {
        Options::new()
    }

    fn check(&self, doc: &Document, _options: &Options) -> Vec<Edit> {
        let sc = scan(doc, 2);
        let mut plan = Plan::default();
        for p in 0..sc.lines.len().saturating_sub(1) {
            if sc.lines[p].depth != sc.lines[p + 1].depth {
                plan.ensure(&sc.lines, p);
            }
        }
        plan.edits(doc, &sc, "blank line added around blockquote")
    }
}

unit_rule!(EmptyLineAroundCodeFences, "empty-line-around-code-fences");

impl Rule for EmptyLineAroundCodeFences {
    fn id(&self) -> &'static str {
        Self::ID
    }

    fn description(&self) -> &'static str {
        "Ensure a blank line before and after fenced code blocks."
    }

    fn default_options(&self) -> Options {
        Options::new()
    }

    fn check(&self, doc: &Document, _options: &Options) -> Vec<Edit> {
        let sc = scan(doc, 2);
        let mut plan = Plan::default();
        for (i, l) in sc.lines.iter().enumerate() {
            match l.kind {
                Kind::FenceOpen if i > 0 => plan.ensure(&sc.lines, i - 1),
                Kind::FenceClose => plan.ensure(&sc.lines, i),
                _ => {}
            }
        }
        plan.edits(doc, &sc, "blank line added around code fence")
    }
}

unit_rule!(
    EmptyLineAroundHorizontalRules,
    "empty-line-around-horizontal-rules"
);

fn is_rule_line(lines: &[Info], i: usize) -> bool {
    let l = &lines[i];
    if l.kind != Kind::Normal || !is_hr(l.inner) {
        return false;
    }
    if is_dash_underline(l.inner) && i > 0 {
        let p = &lines[i - 1];
        if p.depth == l.depth && is_paragraph_text(p) {
            return false;
        }
    }
    true
}

impl Rule for EmptyLineAroundHorizontalRules {
    fn id(&self) -> &'static str {
        Self::ID
    }

    fn description(&self) -> &'static str {
        "Ensure a blank line before and after horizontal rules."
    }

    fn default_options(&self) -> Options {
        Options::new()
    }

    fn check(&self, doc: &Document, _options: &Options) -> Vec<Edit> {
        let sc = scan(doc, 2);
        let mut plan = Plan::default();
        for i in 0..sc.lines.len() {
            if !is_rule_line(&sc.lines, i) {
                continue;
            }
            if i > 0 {
                plan.ensure(&sc.lines, i - 1);
                plan.collapse_before(&sc.lines, i);
            }
            plan.ensure(&sc.lines, i);
            plan.collapse_after(&sc.lines, i);
        }
        plan.edits(doc, &sc, "blank line added around horizontal rule")
    }
}

unit_rule!(EmptyLineAroundMathBlock, "empty-line-around-math-block");

impl Rule for EmptyLineAroundMathBlock {
    fn id(&self) -> &'static str {
        Self::ID
    }

    fn description(&self) -> &'static str {
        "Ensure a blank line before and after display math blocks."
    }

    fn default_options(&self) -> Options {
        opts(json!({"number-of-dollar-signs": 2}))
    }

    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit> {
        let dollars = options
            .get("number-of-dollar-signs")
            .and_then(Value::as_u64)
            .map_or(2, |n| n.clamp(1, 16) as usize);
        let sc = scan(doc, dollars);
        let mut plan = Plan::default();
        for (i, l) in sc.lines.iter().enumerate() {
            match l.kind {
                Kind::MathOpen | Kind::MathSingle if i > 0 => plan.ensure(&sc.lines, i - 1),
                _ => {}
            }
            if matches!(l.kind, Kind::MathClose | Kind::MathSingle) {
                plan.ensure(&sc.lines, i);
            }
        }
        plan.edits(doc, &sc, "blank line added around math block")
    }
}

unit_rule!(EmptyLineAroundTables, "empty-line-around-tables");

impl Rule for EmptyLineAroundTables {
    fn id(&self) -> &'static str {
        Self::ID
    }

    fn description(&self) -> &'static str {
        "Ensure a blank line before and after tables."
    }

    fn default_options(&self) -> Options {
        Options::new()
    }

    fn check(&self, doc: &Document, _options: &Options) -> Vec<Edit> {
        let sc = scan(doc, 2);
        let mut plan = Plan::default();
        for (i, l) in sc.lines.iter().enumerate() {
            if l.kind == Kind::TableHead && i > 0 {
                plan.ensure(&sc.lines, i - 1);
            }
            if matches!(l.kind, Kind::TableDelim | Kind::TableRow)
                && sc.lines.get(i + 1).map(|n| n.kind) != Some(Kind::TableRow)
            {
                plan.ensure(&sc.lines, i);
            }
        }
        plan.edits(doc, &sc, "blank line added around table")
    }
}

unit_rule!(HeadingBlankLines, "heading-blank-lines");

impl Rule for HeadingBlankLines {
    fn id(&self) -> &'static str {
        Self::ID
    }

    fn description(&self) -> &'static str {
        "Ensure a blank line before and after every heading."
    }

    fn default_options(&self) -> Options {
        opts(json!({"bottom": true, "empty-line-between-yaml-and-header": true}))
    }

    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit> {
        let bottom = bool_opt(options.get("bottom"), true);
        let yaml_gap = bool_opt(options.get("empty-line-between-yaml-and-header"), true);
        let sc = scan(doc, 2);
        let lines = &sc.lines;
        let mut plan = Plan::default();
        for i in 0..lines.len() {
            if lines[i].kind != Kind::Normal || super::common::atx(lines[i].inner).is_none() {
                continue;
            }
            let after_yaml = sc
                .fm_last
                .filter(|&f| f < i && lines[f + 1..i].iter().all(|l| l.kind == Kind::Blank));
            if let Some(f) = after_yaml {
                let blanks = i - f - 1;
                if yaml_gap {
                    if blanks == 0 {
                        plan.ensure(lines, f);
                    } else {
                        plan.collapse_run(lines, f + 1, i - 1);
                    }
                } else if blanks > 0 {
                    plan.del.insert((lines[f + 1].start, lines[i].start));
                }
            } else if i > 0 {
                plan.ensure(lines, i - 1);
                plan.collapse_before(lines, i);
            }
            if bottom {
                plan.ensure(lines, i);
                plan.collapse_after(lines, i);
            }
        }
        plan.edits(doc, &sc, "blank lines around heading")
    }
}

unit_rule!(LineBreakAtDocumentEnd, "line-break-at-document-end");

impl Rule for LineBreakAtDocumentEnd {
    fn id(&self) -> &'static str {
        Self::ID
    }

    fn description(&self) -> &'static str {
        "End the document with exactly one line break and no trailing blank lines."
    }

    fn default_options(&self) -> Options {
        Options::new()
    }

    fn check(&self, doc: &Document, _options: &Options) -> Vec<Edit> {
        let text = doc.text();
        let sc = scan(doc, 2);
        let lines = &sc.lines;
        let message = "document ends with one line break";
        let Some(k) = lines.iter().rposition(|l| !js_trim(l.text).is_empty()) else {
            return if text.is_empty() {
                Vec::new()
            } else {
                vec![Edit::new(0, text.len(), "", message)]
            };
        };
        let mut cut = lines[k].end;
        for z in doc.zones() {
            let tail_zone = z.kind.is_block() || z.kind == ZoneKind::Html;
            if tail_zone && z.end > cut && z.start < cut {
                cut = z.end.min(text.len());
            }
        }
        let eol = if lines[k].eol.is_empty() {
            doc_eol(text)
        } else {
            lines[k].eol
        };
        if cut == text.len() {
            // A protected region runs to the end: only the final break is ours.
            return if text.ends_with('\n') {
                Vec::new()
            } else {
                vec![Edit::new(cut, cut, eol, message)]
            };
        }
        let b = text.as_bytes();
        let mut eol = eol;
        if cut > 0 && b[cut - 1] == b'\r' && b[cut] == b'\n' {
            if doc.is_protected(cut - 1, cut) {
                // A CRLF zone keeps the `\r`; only the `\n` after it is ours.
                eol = "\n";
            } else {
                cut -= 1;
            }
        }
        let tail = &text[cut..];
        if tail == eol {
            return Vec::new();
        }
        vec![Edit::new(cut, text.len(), eol, message)]
    }
}
