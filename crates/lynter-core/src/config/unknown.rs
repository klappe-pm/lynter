//! Finds rule ids in a config that no rule answers to. The loader accepts any
//! key under `rules`, so a misspelled id is silently ignored; this is the
//! check that catches it. It is separate from `parse_config` so loading keeps
//! its exact results and hosts opt in to the extra findings.

use super::Config;
use crate::rules::builtin_rules;
use serde_json::Value;

/// Ids of the code and data rules hosted by the `lynter-code` crate. The core
/// cannot depend on that crate, so the list is repeated here and a test in
/// `lynter-code` keeps the two in step.
pub const CODE_RULE_IDS: [&str; 13] = [
    "json-syntax",
    "json-duplicate-keys",
    "json-key-order",
    "yaml-syntax",
    "yaml-duplicate-keys",
    "yaml-key-order",
    "yaml-schema",
    "identifier-case",
    "syntax-error",
    "comment-tag",
    "commented-out-code",
    "max-nesting-depth",
    "max-function-lines",
];

/// A rule id used in a config that no rule answers to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnknownRule {
    /// Config key path of the stanza, for example `profiles.work.rules.contraction`.
    pub path: String,
    pub id: String,
    /// The closest known id, when one is near enough to be a likely typo.
    pub suggestion: Option<String>,
}

impl UnknownRule {
    pub fn message(&self) -> String {
        match &self.suggestion {
            Some(s) => format!(
                "unknown rule \"{}\" is ignored: did you mean \"{s}\"?",
                self.id
            ),
            None => format!("unknown rule \"{}\" is ignored", self.id),
        }
    }
}

/// Every rule id a config may name: the core rules and the code rules.
pub fn known_rule_ids() -> Vec<String> {
    let mut ids: Vec<String> = builtin_rules().iter().map(|r| r.id().to_owned()).collect();
    ids.extend(CODE_RULE_IDS.iter().map(|id| (*id).to_owned()));
    ids
}

/// Levenshtein distance over chars, small inputs only.
fn distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut prev = row[0];
        row[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = if ca == *cb { prev } else { prev + 1 };
            prev = row[j + 1];
            row[j + 1] = cost.min(row[j] + 1).min(prev + 1);
        }
    }
    row[b.len()]
}

/// The closest known id, when it is within two edits and the edits are at
/// most a third of what was typed, so a short or unrelated id suggests nothing.
fn suggest(id: &str, known: &[String]) -> Option<String> {
    let typed = id.chars().count();
    known
        .iter()
        .map(|k| (distance(id, k), k))
        .filter(|(d, _)| *d <= 2 && d * 3 <= typed)
        .min_by_key(|(d, _)| *d)
        .map(|(_, k)| k.clone())
}

fn scan(rules: Option<&Value>, path: &str, known: &[String], out: &mut Vec<UnknownRule>) {
    let Some(map) = rules.and_then(Value::as_object) else {
        return;
    };
    for id in map.keys() {
        if !known.iter().any(|k| k == id) {
            out.push(UnknownRule {
                path: format!("{path}.{id}"),
                id: id.clone(),
                suggestion: suggest(id, known),
            });
        }
    }
}

/// Unknown rule ids under `defaults.rules`, each `profiles.<name>.rules`, and
/// each `providers.<namespace>.rules`, in document order.
pub fn unknown_rules(config: &Config) -> Vec<UnknownRule> {
    let known = known_rule_ids();
    let mut out = Vec::new();
    let raw = &config.raw;
    scan(
        raw.get("defaults").and_then(|d| d.get("rules")),
        "defaults.rules",
        &known,
        &mut out,
    );
    for section in ["profiles", "providers"] {
        if let Some(entries) = raw.get(section).and_then(Value::as_object) {
            for (name, entry) in entries {
                scan(
                    entry.get("rules"),
                    &format!("{section}.{name}.rules"),
                    &known,
                    &mut out,
                );
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{default_config, parse_config, scaffold, LoadResult};

    fn load(yaml: &str) -> Config {
        match parse_config(yaml) {
            LoadResult::Ok { config, .. } => *config,
            LoadResult::Err(e) => panic!("config must load: {e:?}"),
        }
    }

    #[test]
    fn scaffold_and_defaults_name_only_known_rules() {
        assert_eq!(unknown_rules(&load(&scaffold())), vec![]);
        assert_eq!(unknown_rules(&default_config()), vec![]);
    }

    #[test]
    fn a_typo_is_reported_with_its_path_and_a_suggestion() {
        let config = load(
            "version: 1\ndefaults:\n  rules:\n    contraction:\n      enabled: true\nprofiles:\n  work:\n    match:\n      path: [\"work/**\"]\n    rules:\n      header-cas:\n        enabled: true\n      identifier-case:\n        enabled: true\n",
        );
        let found = unknown_rules(&config);
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].path, "defaults.rules.contraction");
        assert_eq!(found[0].suggestion.as_deref(), Some("contractions"));
        assert_eq!(found[1].path, "profiles.work.rules.header-cas");
        assert_eq!(found[1].suggestion.as_deref(), Some("header-case"));
        assert!(found[0].message().contains("did you mean \"contractions\""));
    }

    #[test]
    fn a_short_id_gets_no_suggestion() {
        let config = load("version: 1\ndefaults:\n  rules:\n    abc:\n      enabled: true\n");
        assert_eq!(unknown_rules(&config)[0].suggestion, None);
    }

    #[test]
    fn an_id_near_nothing_gets_no_suggestion() {
        let config =
            load("version: 1\ndefaults:\n  rules:\n    zzzzzzzzzzzz:\n      enabled: true\n");
        let found = unknown_rules(&config);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].suggestion, None);
    }
}
