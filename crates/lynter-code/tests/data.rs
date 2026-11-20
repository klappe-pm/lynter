mod common;
use common::{messages, run};

fn cfg(rules: &str) -> String {
    format!("version: 1\ndefaults:\n  rules:\n{rules}")
}

#[test]
fn json_syntax_reports_line_and_column() {
    let c = cfg("    json-syntax: {enabled: true}\n");
    let r = run(&c, "a.json", "{\n  \"a\": 1,\n  \"b\": ,\n}\n").unwrap();
    assert_eq!(r.violations.len(), 1);
    let v = &r.violations[0];
    assert_eq!(v.line, 3);
    assert!(v.message.starts_with("line 3, column 8:"), "{}", v.message);
    assert!(!v.fixed && !r.changed);
    assert!(run(&c, "a.json", "{\"a\": [1, 2.5e3, null, \"x\\n\"]}")
        .unwrap()
        .violations
        .is_empty());
    assert!(!run(&c, "a.json", "{\"a\": 1} x")
        .unwrap()
        .violations
        .is_empty());
}

#[test]
fn json_duplicate_keys_reports_second_occurrence() {
    let c = cfg("    json-duplicate-keys: {enabled: true}\n");
    let r = run(
        &c,
        "a.json",
        "{\"a\": 1,\n \"b\": {\"x\": 1, \"x\": 2},\n \"a\": 3}",
    )
    .unwrap();
    assert_eq!(
        messages(&r),
        vec![
            "json-duplicate-keys:2:duplicate key \"x\" (first on line 2)",
            "json-duplicate-keys:3:duplicate key \"a\" (first on line 1)"
        ]
    );
}

#[test]
fn json_key_order_sorts_with_priority_and_keeps_layout() {
    let c = cfg("    json-key-order: {enabled: true, priority: [name, version], deep: true}\n");
    let src = "{\n  \"z\": 1,\n  \"version\": \"1\",\n  \"b\": {\"y\": 1, \"x\": [ {\"q\":1,\"p\":2} ]},\n  \"name\": \"n\"\n}\n";
    let r = run(&c, "package.json", src).unwrap();
    assert!(r.changed);
    assert_eq!(
        r.text,
        "{\n  \"name\": \"n\",\n  \"version\": \"1\",\n  \"b\": {\"x\": [ {\"p\":2,\"q\":1} ], \"y\": 1},\n  \"z\": 1\n}\n"
    );
    // Fixed point.
    let again = run(&c, "package.json", &r.text).unwrap();
    assert!(!again.changed && again.violations.is_empty());
    // Top level only by default.
    let c2 = cfg("    json-key-order: {enabled: true}\n");
    let r2 = run(&c2, "a.json", "{\"b\": {\"y\":1,\"x\":2}, \"a\": 1}").unwrap();
    assert_eq!(r2.text, "{\"a\": 1, \"b\": {\"y\":1,\"x\":2}}");
}

#[test]
fn yaml_syntax_and_duplicates() {
    let c = cfg("    yaml-syntax: {enabled: true}\n    yaml-duplicate-keys: {enabled: true}\n");
    let bad = run(&c, "a.yml", "a: 1\nb: [1, 2\nc: 3\n").unwrap();
    assert_eq!(bad.violations.len(), 1);
    assert_eq!(bad.violations[0].rule, "yaml-syntax");
    assert!(bad.violations[0].message.starts_with("line "));
    let dup = run(&c, "a.yaml", "a: 1\nn:\n  x: 1\n  x: 2\na: 3\n").unwrap();
    assert_eq!(
        messages(&dup)
            .into_iter()
            .filter(|m| m.starts_with("yaml-dup"))
            .collect::<Vec<_>>(),
        vec![
            "yaml-duplicate-keys:4:duplicate key \"x\" (first on line 3)",
            "yaml-duplicate-keys:5:duplicate key \"a\" (first on line 1)"
        ]
    );
}

#[test]
fn yaml_key_order_keeps_comments_with_their_key() {
    let c = cfg("    yaml-key-order: {enabled: true, priority: [title]}\n");
    let src = "# header\n\nzeta: 1\n# about b\nbeta:\n  - one # tail\n  - two\ntitle: T\nalpha: |\n  text\n  more\n";
    let r = run(&c, "a.yaml", src).unwrap();
    assert_eq!(
        r.text,
        "# header\n\ntitle: T\nalpha: |\n  text\n  more\n# about b\nbeta:\n  - one # tail\n  - two\nzeta: 1\n"
    );
    let again = run(&c, "a.yaml", &r.text).unwrap();
    assert!(!again.changed);
    // No trailing newline on the last key.
    let r2 = run(&c, "a.yaml", "b: 1\na: 2").unwrap();
    assert_eq!(r2.text, "a: 2\nb: 1");
    // Aliases and multi-document files are left alone.
    assert!(!run(&c, "a.yaml", "b: &x 1\na: *x\n").unwrap().changed);
    assert!(!run(&c, "a.yaml", "b: 1\n---\na: 2\n").unwrap().changed);
}

#[test]
fn yaml_schema_checks_required_keys_and_types() {
    let c = cfg("    yaml-schema: {enabled: true, required: [name, version], types: {name: string, count: integer, tags: array}}\n");
    let r = run(&c, "a.yaml", "name: 5\ncount: 2.5\ntags: [a]\n").unwrap();
    let m = messages(&r);
    assert_eq!(m.len(), 3, "{m:?}");
    assert!(m
        .iter()
        .any(|x| x.contains("missing required key \"version\"")));
    assert!(m
        .iter()
        .any(|x| x.contains("\"name\" on line 1 must be string, found integer")));
    assert!(m
        .iter()
        .any(|x| x.contains("\"count\" on line 2 must be integer, found number")));
    assert!(run(&c, "a.yaml", "name: n\nversion: 1\ncount: 3\n")
        .unwrap()
        .violations
        .is_empty());
}

#[test]
fn profiles_scope_rule_options_by_path() {
    let c = "version: 1
defaults:
  rules:
    yaml-schema: {enabled: true, required: [name]}
profiles:
  ci:
    match: {path: [\".github/**\"]}
    rules:
      yaml-schema: {enabled: true, required: [on]}
";
    assert!(messages(&run(c, "a.yml", "x: 1\n").unwrap())[0].contains("\"name\""));
    assert!(messages(&run(c, ".github/w.yml", "x: 1\n").unwrap())[0].contains("\"on\""));
    // Nothing enabled for the file type: no result at all.
    assert!(run(c, "a.json", "{}").is_none());
}
