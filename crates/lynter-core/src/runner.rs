//! The runner: sorts and applies non-overlapping edits and loops to a fixed
//! point. `check` and `fix` share one path; check only skips the write-back.

use crate::document::{Document, Protection};
use crate::edit::{apply, normalize, Edit};
use crate::rule::{Options, Rule};
use crate::rules::get_rule;
use serde_json::{Map, Value};

/// Upper bound on rounds, so a misbehaving rule cannot loop forever.
const MAX_ROUNDS: usize = 64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Violation {
    pub rule: String,
    pub message: String,
    /// 1-based line of the edit start in the text the rule saw.
    pub line: usize,
    pub start: usize,
    pub end: usize,
    pub fixed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunResult {
    pub text: String,
    pub violations: Vec<Violation>,
    pub changed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Report violations; the returned text is the unchanged input.
    Check,
    /// Apply every edit and return the fixed text.
    Fix,
}

/// Per-rule enablement plus options, as resolved for one file. Mirrors the
/// resolved profile: `{rule-id: {enabled?, ...options}}`.
pub type ResolvedRules = Map<String, Value>;

#[derive(Default)]
pub struct RunOptions {
    pub rules: ResolvedRules,
    /// Restrict the run to these rule ids, in this order.
    pub only: Option<Vec<String>>,
    /// Ids listed first run in this order; the rest keep registry order.
    pub order: Vec<String>,
}

fn options_for(rule: &dyn Rule, cfg: &ResolvedRules) -> Options {
    let mut options = rule.default_options();
    if let Some(Value::Object(stanza)) = cfg.get(rule.id()) {
        for (k, v) in stanza {
            if k != "enabled" {
                options.insert(k.clone(), v.clone());
            }
        }
    }
    options
}

fn is_enabled(rule: &dyn Rule, cfg: &ResolvedRules) -> bool {
    cfg.get(rule.id())
        .and_then(|v| v.get("enabled"))
        .and_then(Value::as_bool)
        == Some(true)
}

fn select(opts: &RunOptions) -> Vec<Box<dyn Rule>> {
    if let Some(only) = &opts.only {
        return only.iter().filter_map(|id| get_rule(id)).collect();
    }
    let mut rules: Vec<(usize, Box<dyn Rule>)> = crate::rules::builtin_rules()
        .into_iter()
        .filter(|r| is_enabled(r.as_ref(), &opts.rules))
        .enumerate()
        .collect();
    let n = opts.order.len();
    rules.sort_by_key(|(i, r)| {
        (
            opts.order.iter().position(|o| o == r.id()).unwrap_or(n + i),
            *i,
        )
    });
    rules.into_iter().map(|(_, r)| r).collect()
}

/// Run the selected rules over `text`. Each rule loops to a fixed point in
/// order, and the whole sequence repeats until nothing changes.
pub fn run(text: &str, opts: &RunOptions, mode: Mode) -> RunResult {
    let rules = select(opts);
    let mut current = text.to_owned();
    let mut violations = Vec::new();
    if mode == Mode::Check {
        for rule in &rules {
            let options = options_for(rule.as_ref(), &opts.rules);
            let doc = Document::parse(&current);
            for e in normalize(rule.check(&doc, &options)) {
                violations.push(violation(rule.as_ref(), &doc, &e, false));
            }
        }
        return RunResult {
            text: current,
            violations,
            changed: false,
        };
    }
    for _ in 0..MAX_ROUNDS {
        let before = current.clone();
        for rule in &rules {
            let options = options_for(rule.as_ref(), &opts.rules);
            for _ in 0..MAX_ROUNDS {
                let doc = Document::parse(&current);
                let edits = normalize(rule.check(&doc, &options));
                if edits.is_empty() {
                    break;
                }
                let next = apply(&current, &edits);
                if next == current {
                    break;
                }
                violations.extend(
                    edits
                        .iter()
                        .map(|e| violation(rule.as_ref(), &doc, e, true)),
                );
                current = next;
            }
        }
        if current == before {
            break;
        }
    }
    let changed = current != text;
    RunResult {
        text: current,
        violations,
        changed,
    }
}

fn violation(rule: &dyn Rule, doc: &Document, e: &Edit, fixed: bool) -> Violation {
    Violation {
        rule: rule.id().to_owned(),
        message: e.message.clone(),
        line: doc.line_of(e.start),
        start: e.start,
        end: e.end,
        fixed,
    }
}

/// Fix a single rule with explicit options, to a fixed point.
pub fn fix_with(rule: &dyn Rule, text: &str, options: &Options) -> String {
    fix_with_protection(rule, text, options, Protection::Parsed)
}

/// Like [`fix_with`] with an explicit protection source.
pub fn fix_with_protection(
    rule: &dyn Rule,
    text: &str,
    options: &Options,
    protection: Protection,
) -> String {
    fix_rounds(rule, text, options, protection, MAX_ROUNDS)
}

/// Apply at most `max_rounds` rounds of one rule. One round is a single
/// application, which is what the TypeScript core does per rule.
pub fn fix_rounds(
    rule: &dyn Rule,
    text: &str,
    options: &Options,
    protection: Protection,
    max_rounds: usize,
) -> String {
    let mut current = text.to_owned();
    for _ in 0..max_rounds {
        let doc = Document::parse_with(&current, protection);
        let edits = normalize(rule.check(&doc, options));
        if edits.is_empty() {
            break;
        }
        let next = apply(&current, &edits);
        if next == current {
            break;
        }
        current = next;
    }
    current
}
