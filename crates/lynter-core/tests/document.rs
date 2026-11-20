use lynter_core::document::Document;
use lynter_core::runner::{run, Mode, RunOptions};
use serde_json::json;

#[test]
fn list_item_offsets_start_at_the_marker() {
    let text = "  - a\n  - b\n\n- c\n\n    - d\n\n* e\n * f\n";
    let doc = Document::parse(text);
    assert!(!doc.list_items().is_empty());
    for item in doc.list_items() {
        let rest = &text[item.marker_start..];
        let first = rest.chars().next().expect("marker");
        assert!(
            first == '-' || first == '*' || first == '+' || first.is_ascii_digit(),
            "item starts at {first:?}"
        );
    }
    // At least one nested item had its raw range start in indentation.
    assert!(doc
        .list_items()
        .iter()
        .any(|i| i.raw.start != i.marker_start));
}

#[test]
fn protected_regions_cover_code_math_frontmatter_tables_html() {
    let text = "---\na: 1\n---\n\n```\ncode\n```\n\n$$\nx\n$$\n\n| a | b |\n| - | - |\n| 1 | 2 |\n\n<div>\nhtml\n</div>\n\nplain `inline` text\n";
    let doc = Document::parse(text);
    for needle in ["a: 1", "code", "x\n", "| 1 | 2 |", "html", "inline"] {
        let at = text.find(needle).expect("needle");
        assert!(
            doc.is_protected(at, at + needle.len()),
            "{needle:?} should be protected"
        );
    }
    let at = text.find("plain").expect("plain");
    assert!(!doc.is_protected(at, at + 5));
}

#[test]
fn check_reports_and_fix_applies_through_one_path() {
    let text = "It is **very** good \u{2014} really.\n";
    let rules = json!({
        "strip-strong": {"enabled": true},
        "replace-em-dash": {"enabled": true},
        "contractions": {"enabled": true},
        "header-case": {"enabled": false},
    });
    let opts = RunOptions {
        rules: rules.as_object().cloned().unwrap_or_default(),
        ..Default::default()
    };
    let checked = run(text, &opts, Mode::Check);
    assert_eq!(checked.text, text);
    assert!(!checked.changed);
    // strip-strong reports its two delimiters, the dash and the contraction report one each.
    assert_eq!(checked.violations.len(), 4);
    let fixed = run(text, &opts, Mode::Fix);
    assert_eq!(fixed.text, "It's very good, really.\n");
    assert!(fixed.changed);
    // A second run finds nothing: check and fix agree on the fixed point.
    assert!(run(&fixed.text, &opts, Mode::Check).violations.is_empty());
}

#[test]
fn rule_order_and_only_select_rules() {
    let rules = json!({"strip-strong": {"enabled": true}, "contractions": {"enabled": true}});
    let opts = RunOptions {
        rules: rules.as_object().cloned().unwrap_or_default(),
        only: Some(vec!["contractions".to_owned()]),
        ..Default::default()
    };
    let out = run("it is **x**\n", &opts, Mode::Fix);
    assert_eq!(out.text, "it's **x**\n");
}

#[test]
fn discovery_prefers_lynter_yaml_and_notes_deprecated_names() {
    use lynter_core::config::discover_config;
    let dir = std::env::temp_dir().join(format!("lynter-discovery-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("mkdir");
    assert!(discover_config(&dir).is_none());
    std::fs::write(dir.join("lappe-linter.yaml"), "version: 1\n").expect("write");
    let d = discover_config(&dir).expect("found");
    assert!(d.path.ends_with("lappe-linter.yaml") && d.notice.is_some());
    std::fs::write(dir.join("linter.yaml"), "version: 1\n").expect("write");
    assert!(discover_config(&dir)
        .expect("found")
        .path
        .ends_with("linter.yaml"));
    std::fs::write(dir.join("lynter.yaml"), "version: 1\n").expect("write");
    let d = discover_config(&dir).expect("found");
    assert!(d.path.ends_with("lynter.yaml") && d.notice.is_none());
    std::fs::remove_dir_all(&dir).ok();
}
