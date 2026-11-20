use super::next_char;
use crate::document::Document;
use crate::edit::Edit;
use crate::rule::{Options, Rule};
use fancy_regex::Regex;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::LazyLock;

include!("contractions_table.rs");

static REPLACEMENTS: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| CONTRACTIONS.iter().copied().collect());

fn is_negation(source: &str) -> bool {
    source.ends_with(" not") || source == "cannot"
}

fn is_have(source: &str) -> bool {
    matches!(
        source,
        "could have" | "might have" | "must have" | "should have" | "would have"
    )
}

/// Three passes preserve precedence: negations, then modal "have", then the rest.
static PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    let groups: [fn(&str) -> bool; 3] = [is_negation, is_have, |s| !is_negation(s) && !is_have(s)];
    groups
        .iter()
        .map(|pick| {
            let alts: Vec<String> = CONTRACTIONS
                .iter()
                .filter(|(s, _)| pick(s))
                .map(|(s, _)| s.split(' ').collect::<Vec<_>>().join("[ \\t]+"))
                .collect();
            Regex::new(&format!(
                "(?i)(?<![\\p{{L}}\\p{{N}}_'\u{2019}])(?:{})(?![\\p{{L}}\\p{{N}}_'\u{2019}])",
                alts.join("|")
            ))
            .expect("regex")
        })
        .collect()
});

fn preserve_case(source: &str, replacement: &str) -> String {
    if source == source.to_uppercase() {
        return replacement.to_uppercase();
    }
    if source.starts_with(|c: char| c.is_ascii_uppercase()) {
        let mut chars = replacement.chars();
        return match chars.next() {
            Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
            None => String::new(),
        };
    }
    replacement.to_owned()
}

/// Replace selected expanded English phrases with contractions.
pub struct Contractions;

impl Rule for Contractions {
    fn id(&self) -> &'static str {
        "contractions"
    }

    fn description(&self) -> &'static str {
        "Replace selected expanded English phrases with contractions, preserving capitalization and protected Markdown regions."
    }

    fn default_options(&self) -> Options {
        CONTRACTIONS
            .iter()
            .map(|(s, _)| ((*s).to_owned(), Value::Bool(true)))
            .collect()
    }

    /// Edits for the first pass that has any; later passes run on the next round.
    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit> {
        let text = doc.text();
        for pattern in PATTERNS.iter() {
            let mut edits = Vec::new();
            let mut pos = 0;
            while pos <= text.len() {
                let Ok(Some(m)) = pattern.find_from_pos(text, pos) else {
                    break;
                };
                if m.start() == m.end() {
                    pos = next_char(text, m.end());
                    continue;
                }
                if doc.is_protected(m.start(), m.end()) {
                    pos = next_char(text, m.start());
                    continue;
                }
                pos = m.end();
                let key = collapse_ws(&m.as_str().to_lowercase());
                if options.get(&key) == Some(&Value::Bool(false)) {
                    continue;
                }
                if let Some(repl) = REPLACEMENTS.get(key.as_str()) {
                    edits.push(Edit::new(
                        m.start(),
                        m.end(),
                        preserve_case(m.as_str(), repl),
                        "contraction",
                    ));
                }
            }
            if !edits.is_empty() {
                return edits;
            }
        }
        Vec::new()
    }
}

fn collapse_ws(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_ws = false;
    for c in s.chars() {
        if c == ' ' || c == '\t' {
            if !in_ws {
                out.push(' ');
            }
            in_ws = true;
        } else {
            out.push(c);
            in_ws = false;
        }
    }
    out
}
