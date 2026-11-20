use super::casing::CaseStyle;
use super::frontmatter::{self, Frontmatter};
use super::{opt_bool, opt_list, opt_str, opt_usize, Reports};
use crate::document::Document;
use crate::edit::Edit;
use crate::rule::{Options, Rule};
use regex::Regex;
use serde_json::{json, Value};
use std::sync::LazyLock;

static TAG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:^|[ \t])#([\p{L}\p{N}_/\-]+)").expect("regex"));
static TAG_NAME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[\p{L}\p{N}_/\-]+$").expect("regex"));

struct Config {
    case: Option<CaseStyle>,
    max_depth: usize,
    allowed: Vec<String>,
    dedupe: bool,
    placement: String,
}

fn has_non_digit(name: &str) -> bool {
    name.chars().any(|c| !c.is_numeric() && c != '/')
}

impl Config {
    /// The name after case conversion, or the name itself when conversion
    /// would break it.
    fn converted(&self, name: &str) -> String {
        let Some(style) = self.case else {
            return name.to_owned();
        };
        let new = name
            .split('/')
            .map(|s| style.convert(s))
            .collect::<Vec<_>>()
            .join("/");
        if TAG_NAME.is_match(&new) && has_non_digit(&new) {
            new
        } else {
            name.to_owned()
        }
    }

    fn allowed_ok(&self, name: &str) -> bool {
        self.allowed.is_empty()
            || self.allowed.iter().any(|a| {
                a == name
                    || a.strip_suffix("/*").is_some_and(|p| {
                        name == p || name.strip_prefix(p).is_some_and(|r| r.starts_with('/'))
                    })
            })
    }

    /// Problems other than case, judged on the name as it will read after
    /// the case fix.
    fn problems(&self, name: &str) -> Vec<String> {
        let eff = self.converted(name);
        let mut out = Vec::new();
        let depth = eff.split('/').count();
        if self.max_depth > 0 && depth > self.max_depth {
            out.push(format!(
                "tag \"{eff}\" is nested {depth} levels deep, the maximum is {}",
                self.max_depth
            ));
        }
        if !self.allowed_ok(&eff) {
            out.push(format!("tag \"{eff}\" is not in the allowed list"));
        }
        out
    }
}

fn frontmatter_tags(fm: &Frontmatter) -> (Vec<String>, Option<(usize, usize)>) {
    let Some(map) = &fm.value else {
        return (Vec::new(), None);
    };
    for key in ["tags", "tag"] {
        let Some(v) = map.get(key) else { continue };
        let raw: Vec<String> = match v {
            Value::String(s) => s
                .split(|c: char| c == ',' || c.is_whitespace())
                .map(str::to_owned)
                .collect(),
            Value::Array(a) => a
                .iter()
                .filter_map(|x| match x {
                    Value::String(s) => Some(s.clone()),
                    Value::Number(n) => Some(n.to_string()),
                    _ => None,
                })
                .collect(),
            _ => Vec::new(),
        };
        let tags = raw
            .iter()
            .map(|t| t.trim().trim_start_matches('#').to_owned())
            .filter(|t| !t.is_empty())
            .collect();
        return (tags, fm.key_range(key));
    }
    (Vec::new(), None)
}

/// Check and normalize tags, inline and in frontmatter.
pub struct TagFormat;

impl Rule for TagFormat {
    fn id(&self) -> &'static str {
        "tag-format"
    }

    fn description(&self) -> &'static str {
        "Apply a case style to inline tags, and report tags that are too deeply nested, not on the allowed list, duplicated in frontmatter, or placed against the preferred location."
    }

    fn default_options(&self) -> Options {
        json!({
            "case": "none",
            "max-depth": 0,
            "allowed": [],
            "dedupe": true,
            "placement": "any",
        })
        .as_object()
        .cloned()
        .unwrap_or_default()
    }

    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit> {
        let cfg = Config {
            case: opt_str(options, "case").and_then(CaseStyle::parse),
            max_depth: opt_usize(options, "max-depth"),
            allowed: opt_list(options, "allowed"),
            dedupe: opt_bool(options, "dedupe", true),
            placement: opt_str(options, "placement").unwrap_or("any").to_owned(),
        };
        let text = doc.text();
        let mut edits = Vec::new();
        let mut reports = Reports::default();
        for line in doc.lines() {
            if doc.is_line_block_masked(line) {
                continue;
            }
            let lt = line.text(text);
            for c in TAG.captures_iter(lt) {
                let Some(m) = c.get(1) else { continue };
                let name = m.as_str().trim_end_matches('/');
                if !has_non_digit(name) {
                    continue;
                }
                let start = line.start + m.start();
                let end = start + name.len();
                if doc.is_protected(start - 1, end) {
                    continue;
                }
                let new = cfg.converted(name);
                if new != name {
                    edits.push(Edit::new(
                        start,
                        end,
                        new,
                        format!(
                            "tag \"{name}\" is not {} case",
                            cfg.case.map_or("", |s| s.name())
                        ),
                    ));
                }
                for p in cfg.problems(name) {
                    reports.push(start, end, p);
                }
                if cfg.placement == "frontmatter" {
                    reports.push(start, end, format!("tag \"{name}\" belongs in frontmatter"));
                }
            }
        }
        if let Some(fm) = frontmatter::parse(text) {
            let (tags, range) = frontmatter_tags(&fm);
            let (rs, re) = range.unwrap_or(fm.open);
            let mut seen: Vec<&str> = Vec::new();
            for t in &tags {
                if let Some(style) = cfg.case {
                    if cfg.converted(t) != *t {
                        reports.push(
                            rs,
                            re,
                            format!("frontmatter tag \"{t}\" is not {} case", style.name()),
                        );
                    }
                }
                for p in cfg.problems(t) {
                    reports.push(rs, re, p);
                }
                if cfg.dedupe && seen.contains(&t.as_str()) {
                    reports.push(rs, re, format!("frontmatter tag \"{t}\" is duplicated"));
                }
                seen.push(t);
            }
            if cfg.placement == "inline" && !tags.is_empty() {
                reports.push(rs, re, "tags belong inline, not in frontmatter");
            }
        }
        edits.extend(reports.into_edits(text));
        edits
    }
}
