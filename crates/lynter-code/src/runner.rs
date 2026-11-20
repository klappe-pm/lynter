//! Applies code and data rules to one file using the options of the profile
//! the core scope engine resolves for it.

use crate::language::{extension_of, Language};
use crate::rule::{CodeRule, SourceFile};
use crate::rules::comment_tag::CommentTag;
use crate::rules::commented_out_code::CommentedOutCode;
use crate::rules::identifier_case::IdentifierCase;
use crate::rules::json::{JsonDuplicateKeys, JsonKeyOrder, JsonSyntax};
use crate::rules::structure::{MaxFunctionLines, MaxNestingDepth};
use crate::rules::syntax_error::SyntaxError;
use crate::rules::yaml::{YamlDuplicateKeys, YamlKeyOrder, YamlSchema, YamlSyntax};
use crate::text::line_col;
use lynter_core::edit::{apply, normalize, Edit};
use lynter_core::lint::LintResult;
use lynter_core::scope::{FileFacts, MatchContext, Resolver};
use lynter_core::{Options, Violation};
use serde_json::Value;

/// Upper bound on rounds per rule so a misbehaving rule cannot loop forever.
const MAX_ROUNDS: usize = 16;

/// Every built-in code and data rule, in default run order.
pub fn registry() -> Vec<Box<dyn CodeRule>> {
    vec![
        Box::new(JsonSyntax),
        Box::new(JsonDuplicateKeys),
        Box::new(JsonKeyOrder),
        Box::new(YamlSyntax),
        Box::new(YamlDuplicateKeys),
        Box::new(YamlKeyOrder),
        Box::new(YamlSchema),
        Box::new(IdentifierCase),
        Box::new(SyntaxError),
        Box::new(CommentTag),
        Box::new(CommentedOutCode),
        Box::new(MaxNestingDepth),
        Box::new(MaxFunctionLines),
    ]
}

/// True when some rule here can lint files with this extension.
pub fn handles_extension(ext: &str) -> bool {
    matches!(ext, "json" | "yaml" | "yml") || Language::from_extension(ext).is_some()
}

fn is_enabled(stanza: Option<&Value>) -> bool {
    stanza
        .and_then(|s| s.get("enabled"))
        .and_then(Value::as_bool)
        == Some(true)
}

/// `languages: [rust, python]` on a rule stanza limits it to those languages.
/// Core has no `languages` profile matcher yet, so the filter lives here.
fn language_allowed(stanza: Option<&Value>, language: Option<Language>) -> bool {
    let Some(list) = stanza
        .and_then(|s| s.get("languages"))
        .and_then(Value::as_array)
    else {
        return true;
    };
    language.is_some_and(|l| list.iter().any(|v| v.as_str() == Some(l.name())))
}

fn rule_options(rule: &dyn CodeRule, stanza: Option<&Value>) -> Options {
    let mut options = rule.default_options();
    if let Some(Value::Object(m)) = stanza {
        for (k, v) in m {
            if k != "enabled" && k != "languages" {
                options.insert(k.clone(), v.clone());
            }
        }
    }
    options
}

fn violation(rule: &str, text: &str, e: &Edit, fixed: bool) -> Violation {
    Violation {
        rule: rule.to_owned(),
        message: e.message.clone(),
        line: line_col(text, e.start).0,
        start: e.start,
        end: e.end,
        fixed,
    }
}

/// Lint one non-Markdown file. Returns `None` when no code or data rule is
/// enabled for it, so callers can skip files this crate has nothing to say
/// about. Fixable findings are applied to the returned text with `fixed` set;
/// report-only findings keep `fixed: false`.
pub fn lint_source(
    text: &str,
    path: &str,
    resolver: &Resolver,
    today: Option<&str>,
) -> Option<LintResult> {
    let resolved = resolver.resolve(&FileFacts {
        path: path.to_owned(),
        frontmatter: None,
        context: MatchContext {
            today: today.map(str::to_owned),
            ..MatchContext::default()
        },
    });
    let language = Language::from_path(path);
    let probe = SourceFile {
        text,
        path,
        language: language.filter(|l| l.is_available()),
        tree: None,
    };
    let mut selected: Vec<(usize, Box<dyn CodeRule>)> = registry()
        .into_iter()
        .filter(|r| {
            let stanza = resolved.rules.get(r.id());
            is_enabled(stanza) && language_allowed(stanza, probe.language) && r.applies_to(&probe)
        })
        .enumerate()
        .collect();
    if selected.is_empty() {
        return None;
    }
    let order = resolved.rule_order.clone().unwrap_or_default();
    let n = order.len();
    selected.sort_by_key(|(i, r)| (order.iter().position(|o| o == r.id()).unwrap_or(n + i), *i));

    let mut current = text.to_owned();
    let mut violations = Vec::new();
    for (_, rule) in &selected {
        let options = rule_options(rule.as_ref(), resolved.rules.get(rule.id()));
        for round in 0..MAX_ROUNDS {
            let tree = probe.language.and_then(|l| l.parse(&current));
            let file = SourceFile {
                text: &current,
                path,
                language: probe.language,
                tree: tree.as_ref(),
            };
            let (noop, real): (Vec<Edit>, Vec<Edit>) = rule
                .check(&file, &options)
                .into_iter()
                .partition(|e| current[e.start..e.end] == e.replacement);
            if round == 0 {
                let mut noop = noop;
                noop.sort_by_key(|e| (e.start, e.end));
                violations.extend(
                    noop.iter()
                        .map(|e| violation(rule.id(), &current, e, false)),
                );
            }
            let real = normalize(real);
            if real.is_empty() {
                break;
            }
            violations.extend(real.iter().map(|e| violation(rule.id(), &current, e, true)));
            current = apply(&current, &real);
        }
    }
    let changed = current != text;
    Some(LintResult {
        text: current,
        violations,
        changed,
        profile_chain: resolved.chain,
        note_type: None,
    })
}

/// Extension of a path, lowercase; re-exported for hosts that route files.
pub fn extension(path: &str) -> Option<String> {
    extension_of(path)
}
