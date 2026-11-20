//! The single lint entry: scope resolution, option injection, and the rule
//! sequence in one pure `(text, path, config)` transform. Mirrors the
//! TypeScript `lintText`: each enabled rule is applied once, in registry
//! order adjusted by `rule-order`, and report-only rules only flag.

use crate::config::Config;
use crate::document::{Document, Protection};
use crate::edit::{apply, normalize};
use crate::notetypes::frontmatter::split_document;
use crate::rule::{Options, Rule};
use crate::rules::builtin_rules;
use crate::runner::Violation;
use crate::scope::{inline_tags, FileFacts, MatchContext, Resolver};
use serde_json::{Map, Value};
use std::collections::HashMap;

pub use crate::rules::FILE_PATH_OPTION;

/// Host-provided facts the pure core cannot derive from text.
#[derive(Clone, Debug, Default)]
pub struct ScopeContext {
    pub backlinks: Vec<String>,
    pub aliases: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LintResult {
    pub text: String,
    pub violations: Vec<Violation>,
    pub changed: bool,
    /// Profile application order, `defaults` first.
    pub profile_chain: Vec<String>,
    pub note_type: Option<String>,
}

/// Registry order of the TypeScript core; rules not listed run after these.
const CANONICAL_ORDER: [&str; 21] = [
    "join-paragraph-lines",
    "join-paragraph-lines-fix",
    "strip-strong",
    "replace-em-dash",
    "prose-list-to-sentences",
    "prose-list-to-sentences-fix",
    "header-case",
    "paragraph-spacing",
    "list-style",
    "contractions",
    "yaml-key-sort",
    "alphabetize-property-values",
    "yaml-timestamp",
    "note-type-insert-keys",
    "note-type-key-sort",
    "note-type-date-keys",
    "note-type-validate",
    "h1-matches-stem",
    "kebab-case-filename",
    "code-checks",
    "code-checks-fix",
];

const REPORT_ONLY: [&str; 5] = [
    "join-paragraph-lines",
    "prose-list-to-sentences",
    "note-type-validate",
    "kebab-case-filename",
    "code-checks",
];

const NOTE_TYPE_RULES: [&str; 4] = [
    "note-type-insert-keys",
    "note-type-key-sort",
    "note-type-date-keys",
    "note-type-validate",
];

/// Rules the TypeScript core loops inside one application.
fn rounds_for(id: &str) -> usize {
    if matches!(id, "strip-strong" | "contractions") {
        64
    } else {
        1
    }
}

#[derive(Default)]
struct Stanza {
    enabled: Option<bool>,
    options: Options,
}

/// Lint one file with the Markdown-parse protection.
pub fn lint_file(
    text: &str,
    path: &str,
    config: &Config,
    today: Option<&str>,
    scope: &ScopeContext,
) -> LintResult {
    lint_file_with(text, path, config, today, scope, Protection::Parsed)
}

/// Like [`lint_file`] with an explicit protection source. `LineScan` matches
/// the TypeScript core byte for byte.
pub fn lint_file_with(
    text: &str,
    path: &str,
    config: &Config,
    today: Option<&str>,
    scope: &ScopeContext,
    protection: Protection,
) -> LintResult {
    let fm = split_document(text);
    let resolved = Resolver::new(config).resolve(&FileFacts {
        path: path.to_owned(),
        frontmatter: fm.has.then(|| fm.yaml_lines.join("\n")),
        context: MatchContext {
            tags: inline_tags(text),
            today: today.map(str::to_owned),
            backlinks: scope.backlinks.clone(),
            aliases: scope.aliases.clone(),
        },
    });

    let mut rules: HashMap<String, Stanza> = HashMap::new();
    for (id, stanza) in &resolved.rules {
        let mut s = Stanza::default();
        if let Value::Object(m) = stanza {
            for (k, v) in m {
                if k == "enabled" {
                    s.enabled = v.as_bool();
                } else {
                    s.options.insert(k.clone(), v.clone());
                }
            }
        }
        rules.insert(id.clone(), s);
    }

    let mut injected = Map::new();
    injected.insert("originalText".into(), Value::String(text.to_owned()));
    if let Some(t) = today {
        injected.insert("today".into(), Value::String(t.to_owned()));
    }
    rules
        .entry("yaml-timestamp".into())
        .or_default()
        .options
        .extend(injected);

    if let Some(checks) = config.raw.get("code-checks") {
        let any = checks.as_object().is_some_and(|m| {
            m.values()
                .any(|c| c.get("enabled") == Some(&Value::Bool(true)))
        });
        for id in ["code-checks", "code-checks-fix"] {
            let e = rules.entry(id.into()).or_default();
            e.options.insert("checks".into(), checks.clone());
            if any && e.enabled.is_none() {
                e.enabled = Some(true);
            }
        }
    }

    let strategy = config
        .raw
        .get("rename")
        .and_then(|r| r.get("strategy"))
        .and_then(Value::as_str)
        .unwrap_or("replace");
    rules
        .entry("kebab-case-filename".into())
        .or_default()
        .options
        .insert("strategy".into(), Value::String(strategy.to_owned()));

    let schema = resolved
        .note_type
        .as_ref()
        .and_then(|n| config.raw.get("note-types").and_then(|t| t.get(n)));
    if let Some(schema) = schema {
        for id in NOTE_TYPE_RULES {
            let e = rules.entry(id.into()).or_default();
            e.options.insert("schema".into(), schema.clone());
            if id == "note-type-date-keys" {
                e.options
                    .insert("originalText".into(), Value::String(text.to_owned()));
                if let Some(t) = today {
                    e.options
                        .insert("today".into(), Value::String(t.to_owned()));
                }
            }
        }
    }

    // Select enabled rules in registry order, then apply `rule-order`.
    let mut selected: Vec<(usize, Box<dyn Rule>)> = builtin_rules()
        .into_iter()
        .filter(|r| rules.get(r.id()).and_then(|s| s.enabled) == Some(true))
        .enumerate()
        .collect();
    selected.sort_by_key(|(i, r)| {
        (
            CANONICAL_ORDER
                .iter()
                .position(|c| *c == r.id())
                .unwrap_or(CANONICAL_ORDER.len()),
            *i,
        )
    });
    let mut ordered: Vec<Box<dyn Rule>> = selected.into_iter().map(|(_, r)| r).collect();
    if let Some(order) = resolved.rule_order.as_ref().filter(|o| !o.is_empty()) {
        let n = order.len();
        let mut keyed: Vec<(usize, usize, Box<dyn Rule>)> = ordered
            .into_iter()
            .enumerate()
            .map(|(i, r)| {
                (
                    order.iter().position(|o| o == r.id()).unwrap_or(n + i),
                    i,
                    r,
                )
            })
            .collect();
        keyed.sort_by_key(|(rank, i, _)| (*rank, *i));
        ordered = keyed.into_iter().map(|(_, _, r)| r).collect();
    }

    let mut current = text.to_owned();
    let mut violations = Vec::new();
    for rule in &ordered {
        let stanza = rules.get(rule.id());
        let mut options = rule.default_options();
        if let Some(s) = stanza {
            for (k, v) in &s.options {
                options.insert(k.clone(), v.clone());
            }
        }
        options
            .entry(FILE_PATH_OPTION.to_owned())
            .or_insert_with(|| Value::String(path.to_owned()));
        let report_only = REPORT_ONLY.contains(&rule.id());
        let rounds = rounds_for(rule.id());
        let mut next = current.clone();
        let mut first = true;
        for _ in 0..rounds {
            let doc = Document::parse_with(&next, protection);
            let edits = normalize(rule.check(&doc, &options));
            if edits.is_empty() {
                break;
            }
            let applied = apply(&next, &edits);
            if applied == next {
                // No-op edits are findings from report-only rules.
                if first {
                    for e in &edits {
                        violations.push(Violation {
                            rule: rule.id().to_owned(),
                            message: e.message.clone(),
                            line: doc.line_of(e.start),
                            start: e.start,
                            end: e.end,
                            fixed: false,
                        });
                    }
                }
                break;
            }
            first = false;
            next = applied;
        }
        if next != current {
            let at = first_diff(&current, &next);
            violations.push(Violation {
                rule: rule.id().to_owned(),
                message: rule.description().to_owned(),
                line: Document::parse_with(&current, protection).line_of(at),
                start: at,
                end: at,
                fixed: !report_only,
            });
            if !report_only {
                current = next;
            }
        }
    }
    let changed = current != text;
    LintResult {
        text: current,
        violations,
        changed,
        profile_chain: resolved.chain,
        note_type: resolved.note_type,
    }
}

fn first_diff(a: &str, b: &str) -> usize {
    let mut p = a.bytes().zip(b.bytes()).take_while(|(x, y)| x == y).count();
    while !a.is_char_boundary(p) {
        p -= 1;
    }
    p
}
