//! Behavior tables and property tests for the notes rules.
//! Behavior is defined by these tests.

use lynter_core::document::{Document, Protection};
use lynter_core::edit::normalize;
use lynter_core::rule::Options;
use lynter_core::rules::get_rule;
use lynter_core::runner::fix_with_protection;
use proptest::prelude::*;
use serde_json::{json, Value};

fn opts(id: &str, v: Value) -> Options {
    let mut o = get_rule(id).expect("rule").default_options();
    for (k, val) in v.as_object().cloned().unwrap_or_default() {
        o.insert(k, val);
    }
    o
}

fn fix(id: &str, text: &str, v: Value) -> String {
    let rule = get_rule(id).expect("rule");
    fix_with_protection(rule.as_ref(), text, &opts(id, v), Protection::Parsed)
}

fn report(id: &str, text: &str, v: Value) -> Vec<String> {
    let rule = get_rule(id).expect("rule");
    let doc = Document::parse(text);
    normalize(rule.check(&doc, &opts(id, v)))
        .into_iter()
        .map(|e| e.message)
        .collect()
}

#[test]
fn rules_are_registered() {
    for id in [
        "task-format",
        "tag-format",
        "folder-naming",
        "filename-pattern",
        "frontmatter-schema",
    ] {
        assert!(get_rule(id).is_some(), "{id}");
    }
}

#[test]
fn task_format_table() {
    let cases: &[(&str, &str, Value)] = &[
        ("-  [ ]   buy milk\n", "- [ ] buy milk\n", json!({})),
        ("- [X] done\n", "- [x] done\n", json!({})),
        ("- [x]done\n", "- [x] done\n", json!({})),
        ("  * [/]  half\n", "  * [/] half\n", json!({})),
        ("1.  [-] skipped\n", "1. [-] skipped\n", json!({})),
        (
            "- [X] keep\n",
            "- [X] keep\n",
            json!({"lowercase-marker": false}),
        ),
        (
            "-  [ ]  x\n",
            "-  [ ]  x\n",
            json!({"normalize-spacing": false}),
        ),
        ("- [ ]\n", "- [ ]\n", json!({})),
        (
            "- [x](http://a.b) link\n",
            "- [x](http://a.b) link\n",
            json!({}),
        ),
        ("- [ab] not a task\n", "- [ab] not a task\n", json!({})),
        ("- [a]b not a task\n", "- [a]b not a task\n", json!({})),
        (
            "- [ ] t \u{1F4C5} 2026/12/1\n",
            "- [ ] t \u{1F4C5} 2026-12-01\n",
            json!({}),
        ),
        (
            "- [ ] t \u{23F3}20261201 \u{1F6EB}2026.1.2\n",
            "- [ ] t \u{1F6EB} 2026-01-02 \u{23F3} 2026-12-01\n",
            json!({}),
        ),
        (
            "- [x] t \u{2705} 2026-12-02 \u{1F4C5} 2026-12-01\n",
            "- [x] t \u{1F4C5} 2026-12-01 \u{2705} 2026-12-02\n",
            json!({}),
        ),
        (
            "- [ ] t \u{1F4C5}2026-02-30\n",
            "- [ ] t \u{1F4C5}2026-02-30\n",
            json!({}),
        ),
        (
            "- [ ] t \u{1F4C5} 2026-12-011\n",
            "- [ ] t \u{1F4C5} 2026-12-011\n",
            json!({}),
        ),
        (
            "- [ ] `\u{1F4C5}2026/12/1` \u{1F4C5}2026/12/2\n",
            "- [ ] `\u{1F4C5}2026/12/1` \u{1F4C5} 2026-12-02\n",
            json!({}),
        ),
        (
            "- [ ] t \u{1F4C5} 2026/12/1 \u{1F6EB} 2026/11/1\n",
            "- [ ] t \u{1F4C5} 2026/12/1 \u{1F6EB} 2026/11/1\n",
            json!({"date-format": false, "date-order": []}),
        ),
        (
            "- [ ] t \u{1F4C5} 2026/12/1 \u{1F6EB} 2026/11/1\n",
            "- [ ] t \u{1F6EB} 2026/11/1 \u{1F4C5} 2026/12/1\n",
            json!({"date-format": false}),
        ),
        (
            "```\n-  [X] code \u{1F4C5} 2026/1/1\n```\n",
            "```\n-  [X] code \u{1F4C5} 2026/1/1\n```\n",
            json!({}),
        ),
    ];
    for (input, want, o) in cases {
        assert_eq!(&fix("task-format", input, o.clone()), want, "{input:?}");
    }
}

#[test]
fn task_format_reports_disallowed_status() {
    let m = report(
        "task-format",
        "- [?] odd\n- [x] ok\n",
        json!({"allowed-statuses": " x/-"}),
    );
    assert_eq!(m.len(), 1);
    assert!(m[0].contains('?'));
    assert!(report("task-format", "- [?] odd\n", json!({})).is_empty());
}

#[test]
fn tag_format_table() {
    let cases: &[(&str, &str, Value)] = &[
        ("a #MyTag b\n", "a #my-tag b\n", json!({"case": "kebab"})),
        (
            "a #My-Tag/SubTag\n",
            "a #my_tag/sub_tag\n",
            json!({"case": "snake"}),
        ),
        ("a #my-tag b\n", "a #myTag b\n", json!({"case": "camel"})),
        (
            "a #MyTag #x_Y\n",
            "a #mytag #x_y\n",
            json!({"case": "lower"}),
        ),
        (
            "#Top at start\n",
            "#top at start\n",
            json!({"case": "kebab"}),
        ),
        (
            "`#MyTag` and [[n#Head]]\n",
            "`#MyTag` and [[n#Head]]\n",
            json!({"case": "kebab"}),
        ),
        (
            "# Heading #Tag\n",
            "# Heading #tag\n",
            json!({"case": "kebab"}),
        ),
        (
            "issue #123 ok\n",
            "issue #123 ok\n",
            json!({"case": "kebab"}),
        ),
        ("x #Ab\n", "x #Ab\n", json!({})),
        (
            "---\ntags: [Ab]\n---\n#Ab\n",
            "---\ntags: [Ab]\n---\n#ab\n",
            json!({"case": "lower"}),
        ),
    ];
    for (input, want, o) in cases {
        assert_eq!(&fix("tag-format", input, o.clone()), want, "{input:?}");
    }
}

#[test]
fn tag_format_reports() {
    let r = report("tag-format", "#a/b/c #ok\n", json!({"max-depth": 2}));
    assert_eq!(r.len(), 1);
    assert!(r[0].contains("3 levels"));
    let r = report(
        "tag-format",
        "#work #other #proj/x\n",
        json!({"allowed": ["work", "proj/*"]}),
    );
    assert_eq!(r.len(), 1);
    assert!(r[0].contains("other"));
    let r = report(
        "tag-format",
        "---\ntags:\n  - a\n  - '#a'\n  - b\n---\n",
        json!({}),
    );
    assert_eq!(r.len(), 1);
    assert!(r[0].contains("duplicated"));
    assert!(report(
        "tag-format",
        "---\ntags: [a, a]\n---\n",
        json!({"dedupe": false})
    )
    .is_empty());
    let r = report("tag-format", "x #a\n", json!({"placement": "frontmatter"}));
    assert!(r[0].contains("frontmatter"));
    let r = report(
        "tag-format",
        "---\ntags: a, b\n---\n",
        json!({"placement": "inline"}),
    );
    assert!(r[0].contains("inline"));
    let r = report(
        "tag-format",
        "---\ntags: [MyTag]\n---\n",
        json!({"case": "kebab"}),
    );
    assert!(r[0].contains("my-tag") || r[0].contains("kebab"));
}

#[test]
fn folder_naming_table() {
    let run = |path: &str, o: Value| {
        let mut o = o;
        o["file-path"] = json!(path);
        report("folder-naming", "body\n", o)
    };
    assert!(run("Notes/x.md", json!({})).is_empty());
    assert!(run("my-notes/sub/x.md", json!({"case": "kebab"})).is_empty());
    let r = run("My Notes/sub/x.md", json!({"case": "kebab"}));
    assert_eq!(r.len(), 1);
    assert!(r[0].contains("my-notes"));
    let r = run("a/b/c/x.md", json!({"max-depth": 2}));
    assert!(r[0].contains("depth 3"));
    let r = run(
        "inbox/misc/x.md",
        json!({"allowed-names": ["inbox", "projects"]}),
    );
    assert_eq!(r.len(), 1);
    assert!(r[0].contains("misc"));
    assert!(run(".obsidian/Plugins/x.md", json!({"case": "kebab"})).is_empty());
    assert!(run(".Hidden/x.md", json!({"case": "kebab"})).is_empty());
    assert!(run("x.md", json!({"case": "kebab", "max-depth": 1})).is_empty());
    assert!(run("A\\B\\x.md", json!({"case": "lower"})).len() == 1);
    assert!(report("folder-naming", "b\n", json!({"case": "kebab"})).is_empty());
}

#[test]
fn filename_pattern_table() {
    let run = |path: &str, o: Value| {
        let mut o = o;
        o["file-path"] = json!(path);
        report("filename-pattern", "body\n", o)
    };
    assert!(run("a/Any Name.md", json!({})).is_empty());
    assert!(run("a/my-note.md", json!({"preset": "kebab"})).is_empty());
    assert_eq!(run("a/My Note.md", json!({"preset": "kebab"})).len(), 1);
    assert!(run("2026-12-01-standup.md", json!({"preset": "dated"})).is_empty());
    assert_eq!(
        run("2026-13-01-standup.md", json!({"preset": "dated"})).len(),
        1
    );
    assert_eq!(run("2026-12-01.md", json!({"preset": "dated"})).len(), 1);
    assert!(run("Meeting 1.md", json!({"pattern": "^Meeting \\d+$"})).is_empty());
    assert_eq!(
        run("Other.md", json!({"pattern": "^Meeting \\d+$"})).len(),
        1
    );
    assert_eq!(run("Other.md", json!({"pattern": "("})).len(), 1);
    assert_eq!(
        run("Bad Name.md", json!({"preset": "kebab", "pattern": "^x"})).len(),
        1
    );
}

#[test]
fn frontmatter_schema_table() {
    let schema = json!({
        "required": ["title", "status"],
        "types": {"priority": "number", "due": "date", "draft": "bool", "aliases": "list", "title": "string"},
        "enums": {"status": ["draft", "done"]}
    });
    let ok = "---\ntitle: T\nstatus: done\npriority: 2\ndue: 2026-12-01\ndraft: false\naliases: [a]\n---\nbody\n";
    assert!(report("frontmatter-schema", ok, schema.clone()).is_empty());
    let bad = "---\nstatus: wip\npriority: high\ndue: 2026-02-30\ndraft: yes please\naliases: a\ntitle: 5\n---\n";
    let r = report("frontmatter-schema", bad, schema.clone());
    let all = r.join("\n");
    for want in [
        "\"status\" expects one of draft, done, found \"wip\"",
        "\"priority\" expects number, found string",
        "\"due\" expects date",
        "\"draft\" expects bool",
        "\"aliases\" expects list",
        "\"title\" expects string, found number",
    ] {
        assert!(all.contains(want), "missing {want} in {all}");
    }
    let r = report(
        "frontmatter-schema",
        "---\nstatus: done\n---\n",
        schema.clone(),
    );
    assert!(r[0].contains("missing required key \"title\""));
    let r = report("frontmatter-schema", "no fm\n", schema.clone());
    assert!(r[0].contains("missing frontmatter"));
    let r = report("frontmatter-schema", "---\na: [\n---\n", schema.clone());
    assert!(r[0].contains("not valid YAML"));
    let r = report(
        "frontmatter-schema",
        "---\ntitle:\nstatus: done\n---\n",
        schema.clone(),
    );
    assert!(r[0].contains("\"title\""));
    let r = report(
        "frontmatter-schema",
        "---\ntags: [a, z]\n---\n",
        json!({"enums": {"tags": ["a", "b"]}}),
    );
    assert!(r[0].contains("found \"z\""));
    assert!(report("frontmatter-schema", "x\n", json!({})).is_empty());
}

const FRAGMENTS: &[&str] = &[
    "- [ ] task",
    "-  [X]  task",
    "* [/] half",
    "1. [-] skip",
    "- [?] odd",
    "- [x]done",
    "\u{1F4C5} 2026/12/1",
    "\u{1F4C5}2026-12-01",
    "\u{23F3} 20261201",
    "\u{1F6EB}2026.1.2",
    "\u{2705} 2026-12-02",
    "\u{1F4C5} 2026-02-30",
    "#MyTag",
    "#a/B/c",
    "#my_tag",
    "#123",
    "#x-Y",
    "# Heading",
    "`#Code` \u{1F4C5}2026/1/1",
    "[[n#Head]]",
    "https://x.y/#Frag",
    "$#A$",
    "text",
    "```",
    "---",
    "tags: [A, a]",
    "| #Tab |",
    "> - [X] quote",
    "<div>#Html</div>",
    "\r",
];

fn doc_strategy() -> impl Strategy<Value = String> {
    prop::collection::vec(
        (
            prop::collection::vec(prop::sample::select(FRAGMENTS), 1..4),
            prop::sample::select(&[" ", "", "\n"][..]),
        ),
        1..12,
    )
    .prop_map(|lines| {
        let mut out = String::new();
        for (parts, sep) in lines {
            out.push_str(&parts.join(sep));
            out.push('\n');
        }
        out
    })
}

fn variants(id: &str, n: usize) -> Value {
    match (id, n % 4) {
        ("task-format", 1) => json!({"normalize-spacing": false, "date-order": ["done", "due"]}),
        ("task-format", 2) => json!({"date-format": false, "lowercase-marker": false}),
        ("task-format", 3) => json!({"date-order": []}),
        ("tag-format", 0) => json!({"case": "kebab"}),
        ("tag-format", 1) => json!({"case": "snake", "max-depth": 2}),
        ("tag-format", 2) => json!({"case": "camel"}),
        ("tag-format", _) => json!({"case": "lower"}),
        _ => json!({}),
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(400))]

    /// Fixing a fixed text changes nothing, and no fixing edit remains.
    #[test]
    fn fixing_rules_are_idempotent(text in doc_strategy(), n in 0usize..4) {
        for id in ["task-format", "tag-format"] {
            let o = variants(id, n);
            let once = fix(id, &text, o.clone());
            let twice = fix(id, &once, o.clone());
            prop_assert_eq!(&once, &twice, "{} not idempotent", id);
            let rule = get_rule(id).expect("rule");
            let doc = Document::parse(&once);
            let left: Vec<_> = normalize(rule.check(&doc, &opts(id, o)))
                .into_iter()
                .filter(|e| once[e.start..e.end] != e.replacement)
                .collect();
            prop_assert!(left.is_empty(), "{} leaves edits after fix: {:?}", id, left);
        }
    }

    /// Fixing edits never overlap protected regions.
    #[test]
    fn fixing_edits_skip_protected_regions(text in doc_strategy(), n in 0usize..4) {
        let doc = Document::parse(&text);
        for id in ["task-format", "tag-format"] {
            let rule = get_rule(id).expect("rule");
            for e in rule.check(&doc, &opts(id, variants(id, n))) {
                if text[e.start..e.end] != e.replacement {
                    prop_assert!(!doc.is_protected(e.start, e.end), "{} edited protected bytes", id);
                }
            }
        }
    }

    /// Report-only rules never change the text.
    #[test]
    fn report_only_rules_never_change_text(text in doc_strategy()) {
        let cfg = |id: &str| match id {
            "frontmatter-schema" => json!({"required": ["a"], "types": {"b": "number"}}),
            _ => json!({"file-path": "My Dir/Bad Name.md", "case": "kebab", "preset": "kebab"}),
        };
        for id in ["folder-naming", "filename-pattern", "frontmatter-schema"] {
            let out = fix(id, &text, cfg(id));
            prop_assert_eq!(&out, &text);
        }
    }
}
