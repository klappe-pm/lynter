//! WI-002 defaults and scaffold, and lint_file wiring that has no TypeScript twin.

use lynter_core::config::{
    default_config, default_rules, parse_config, scaffold, LoadResult, OPT_IN_RULES,
};
use lynter_core::lint::{lint_file, ScopeContext};
use serde_json::Value;

#[test]
fn scaffold_parses_clean() {
    match parse_config(&scaffold()) {
        LoadResult::Ok { warnings, .. } => assert!(warnings.is_empty(), "{warnings:?}"),
        LoadResult::Err(e) => panic!("{e:?}"),
    }
}

#[test]
fn opt_in_rules_are_off_in_defaults_and_scaffold() {
    let rules = default_rules();
    let LoadResult::Ok { config, .. } = parse_config(&scaffold()) else {
        panic!("scaffold");
    };
    let scaffolded = config.defaults_rules.expect("defaults.rules");
    for id in OPT_IN_RULES {
        assert_eq!(rules[id]["enabled"], Value::Bool(false), "{id} default");
        assert_eq!(
            scaffolded[id]["enabled"],
            Value::Bool(false),
            "{id} scaffold"
        );
    }
    assert_eq!(rules["replace-em-dash"]["enabled"], Value::Bool(true));
}

#[test]
fn default_config_leaves_opt_in_rules_alone() {
    let config = default_config();
    let text = "---\ndomain: x\n---\n# Title\n\n**bold** it is fine\n";
    let r = lint_file(
        text,
        "Some Note.md",
        &config,
        Some("2026-07-10"),
        &ScopeContext::default(),
    );
    assert!(r.text.contains("**bold** it is fine"), "{:?}", r.text);
    assert!(r.text.contains("# Title"));
    assert!(r.violations.iter().all(|v| v.rule != "kebab-case-filename"));
}

#[test]
fn folder_naming_reports_through_lint_file() {
    let yaml = "version: 1\ndefaults:\n  rules:\n    folder-naming:\n      enabled: true\n      case: kebab-case\n";
    let LoadResult::Ok { config, .. } = parse_config(yaml) else {
        panic!("config");
    };
    let r = lint_file(
        "body\n",
        "My Folder/note.md",
        &config,
        None,
        &ScopeContext::default(),
    );
    assert!(
        r.violations
            .iter()
            .any(|v| v.rule == "folder-naming" && !v.fixed),
        "{:?}",
        r.violations
    );
    assert_eq!(r.text, "body\n");
    let clean = lint_file(
        "body\n",
        "my-folder/note.md",
        &config,
        None,
        &ScopeContext::default(),
    );
    assert!(clean.violations.is_empty(), "{:?}", clean.violations);
}
