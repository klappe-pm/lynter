//! Property tests for the compat rules: fixing twice equals fixing once, no
//! edit lands in a protected region (checked against the zones each rule is
//! allowed to leave alone), the frontmatter rules never touch the body, and a
//! check run never changes text.

use lynter_core::document::{Document, Protection};
use lynter_core::rules::get_rule;
use lynter_core::runner::{fix_with_protection, run, Mode, RunOptions};
use lynter_core::zones::ZoneKind;
use proptest::prelude::*;
use serde_json::{json, Value};

const IDS: &[&str] = &[
    "consecutive-blank-lines",
    "dedupe-yaml-array-values",
    "default-language-for-code-fences",
    "emphasis-style",
    "empty-line-around-blockquotes",
    "empty-line-around-code-fences",
    "empty-line-around-horizontal-rules",
    "empty-line-around-math-block",
    "empty-line-around-tables",
    "escape-yaml-special-characters",
    "format-tags-in-yaml",
    "format-yaml-arrays",
    "header-increment",
    "heading-blank-lines",
    "headings-start-line",
    "insert-yaml-attributes",
    "line-break-at-document-end",
    "move-tags-to-yaml",
    "no-bare-urls",
    "ordered-list-style",
    "proper-ellipsis",
    "quote-style",
    "remove-empty-list-markers",
    "remove-multiple-spaces",
    "remove-trailing-punctuation-in-heading",
    "remove-yaml-keys",
    "space-after-list-markers",
    "strong-style",
    "trailing-spaces",
    "unordered-list-style",
];

/// Rules that edit frontmatter by design.
const YAML: &[&str] = &[
    "dedupe-yaml-array-values",
    "escape-yaml-special-characters",
    "format-tags-in-yaml",
    "format-yaml-arrays",
    "insert-yaml-attributes",
    "remove-yaml-keys",
    "move-tags-to-yaml",
];

const BODY: &[&str] = &[
    "plain words  here",
    "Wait... what",
    "a..b and c....d",
    "*em* and _em2_ and **strong** and __strong2__",
    "***both*** ok",
    "snake_case_name here",
    "\"double\" and 'single' it's",
    "\u{201c}smart\u{201d} \u{2018}x\u{2019}",
    "- bullet",
    "* star",
    "+ plus",
    "  - nested",
    "-   spaced bullet",
    "- ",
    "-",
    "- [ ] task",
    "- [x]",
    "1. first",
    "3.  third",
    "2) paren",
    "1.",
    "> quote",
    "> > nested quote",
    "> [!note]",
    ">",
    "# Heading.",
    "## Heading!",
    "   ## indented heading",
    "### Deep heading &amp;",
    "###### six",
    "#tag and #other/tag and #123",
    "text #inline",
    "---",
    "***",
    "* * *",
    "___",
    "Setext title",
    "| a | b |",
    "|---|---|",
    "| 1 | 2 |",
    "a | b",
    "```",
    "```js",
    "~~~",
    "````",
    "$$",
    "$$x$$",
    "$ money $5",
    "    indented code",
    "<div>",
    "</div>",
    "https://example.com/a.",
    "see https://example.com/a, ok",
    "[l](https://example.com)",
    "[https://example.com](https://example.com)",
    "<https://example.com>",
    "`code   with...` text",
    "[[wiki...link]]",
    "<% tp.date.now(\"x\") %>",
    "trailing  ",
    "tab\t",
    "",
    "",
    "x",
    "caf\u{e9}  text",
    "\u{1F600} emoji",
    "\r",
];

const FRONT: &[&str] = &[
    "tags: [b, a, b]",
    "tags: #x #y",
    "tags:\n  - z\n  - a\n  - z",
    "aliases: One, Two",
    "aliases: [q, p]",
    "key: value: with colon",
    "other: it's here",
    "quoted: 'ok: fine'",
    "arr: [a, a, b]",
    "list:\n- x\n- x",
    "title: plain",
    "gists:\n  - id: 1\n    url: u",
    "empty:",
    "date: 2020-01-01",
    "# comment",
];

fn doc_strategy() -> impl Strategy<Value = String> {
    (
        prop::option::of(prop::collection::vec(prop::sample::select(FRONT), 0..5)),
        prop::collection::vec(
            (
                prop::collection::vec(prop::sample::select(BODY), 1..3),
                prop::sample::select(&[" ", "", "\n"][..]),
            ),
            1..14,
        ),
        prop::bool::ANY,
        prop::bool::ANY,
    )
        .prop_map(|(fm, body, trailing, crlf)| {
            let mut s = String::new();
            if let Some(fm) = fm {
                s.push_str("---\n");
                for l in fm {
                    s.push_str(l);
                    s.push('\n');
                }
                s.push_str("---\n");
            }
            let lines: Vec<String> = body.iter().map(|(p, sep)| p.join(sep)).collect();
            s.push_str(&lines.join("\n"));
            if trailing {
                s.push('\n');
            }
            if crlf {
                s = s.replace("\r\n", "\n").replace('\n', "\r\n");
            }
            s
        })
}

fn variants(id: &str) -> Vec<Value> {
    match id {
        "dedupe-yaml-array-values" => vec![
            json!({}),
            json!({"dedupe-tags": false, "keys-to-ignore": ["arr"]}),
        ],
        "default-language-for-code-fences" => vec![json!({"language": "text"})],
        "emphasis-style" | "strong-style" => vec![
            json!({}),
            json!({"style": "asterisk"}),
            json!({"style": "underscore"}),
        ],
        "empty-line-around-math-block" => vec![json!({}), json!({"number-of-dollar-signs": 1})],
        "escape-yaml-special-characters" => vec![
            json!({}),
            json!({"default-escape-character": "'", "escape-single-line-arrays": true}),
        ],
        "format-yaml-arrays" => vec![
            json!({}),
            json!({"default-array-style": "multi-line", "tags-style": "multi-line", "aliases-style": "multi-line"}),
            json!({"tags-style": "single string space delimited", "aliases-style": "single string comma delimited"}),
            json!({"tags-style": "single string to single-line", "aliases-style": "single string to multi-line", "force-single-line-keys": ["title"], "force-multi-line-keys": ["empty"]}),
        ],
        "header-increment" => vec![json!({}), json!({"start-at-level-2": true})],
        "heading-blank-lines" => vec![
            json!({}),
            json!({"bottom": false}),
            json!({"empty-line-between-yaml-and-header": false}),
        ],
        "insert-yaml-attributes" => vec![
            json!({}),
            json!({"text-to-insert": "tags: x\nfoo: bar\ntags: dup"}),
        ],
        "move-tags-to-yaml" => vec![
            json!({}),
            json!({"body-tag-operation": "remove-hashtag", "tags-to-ignore": ["other/tag"]}),
            json!({"body-tag-operation": "remove-whole-tag"}),
        ],
        "ordered-list-style" => vec![json!({}), json!({"number-style": "lazy", "end-style": ")"})],
        "quote-style" => vec![
            json!({}),
            json!({"single-quote-style": "smart", "double-quote-style": "smart"}),
            json!({"single-quote-enabled": false, "double-quote-style": "smart"}),
        ],
        "remove-trailing-punctuation-in-heading" => {
            vec![json!({}), json!({"trailing-punctuation": "?;"})]
        }
        "remove-yaml-keys" => vec![json!({"keys": ["tags", "key:", "list"]})],
        "trailing-spaces" => vec![json!({}), json!({"two-space-linebreak": true})],
        "unordered-list-style" => vec![
            json!({}),
            json!({"style": "-"}),
            json!({"style": "*"}),
            json!({"style": "+"}),
        ],
        _ => vec![json!({})],
    }
}

fn options_of(id: &str, extra: &Value) -> lynter_core::rule::Options {
    let mut o = get_rule(id).expect("rule").default_options();
    for (k, v) in extra.as_object().cloned().unwrap_or_default() {
        o.insert(k, v);
    }
    o
}

fn fix(id: &str, text: &str, extra: &Value) -> String {
    let rule = get_rule(id).expect("rule");
    fix_with_protection(
        rule.as_ref(),
        text,
        &options_of(id, extra),
        Protection::Parsed,
    )
}

/// Zone kinds whose text must survive a rule, per rule. Frontmatter rules
/// edit the frontmatter; `no-bare-urls` wraps URLs; the spacing rules decide
/// block structure with their own scan, so zones the Markdown parse and the
/// line scan disagree about (tables, CRLF fences, math) are not compared.
fn guarded(id: &str, kind: ZoneKind, start: usize, text: &str) -> bool {
    if matches!(kind, ZoneKind::Url | ZoneKind::Wikilink) {
        return false;
    }
    // A metadata block the parse pairs with a stray `---` can stop being one once the
    // frontmatter changes between rounds; real frontmatter is covered by its own zone.
    if kind == ZoneKind::ParsedBlock && text[start..].starts_with("---") {
        return false;
    }
    if YAML.contains(&id)
        && (kind == ZoneKind::Frontmatter || (kind == ZoneKind::ParsedBlock && start == 0))
    {
        return false;
    }
    if id == "default-language-for-code-fences"
        && matches!(kind, ZoneKind::CodeFence | ZoneKind::ParsedBlock)
    {
        return false;
    }
    if id.starts_with("empty-line-around") || id == "heading-blank-lines" {
        if matches!(
            kind,
            ZoneKind::Table | ZoneKind::ParsedBlock | ZoneKind::MathBlock
        ) {
            return false;
        }
        if kind == ZoneKind::CodeFence && text.contains('\r') {
            return false;
        }
    }
    if id == "line-break-at-document-end" || id == "consecutive-blank-lines" {
        // Both only touch lines outside protected blocks, but a block may run to the end.
        return !matches!(kind, ZoneKind::ParsedBlock | ZoneKind::Table);
    }
    true
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    #[test]
    fn fix_is_idempotent(text in doc_strategy()) {
        for id in IDS {
            for extra in variants(id) {
                let once = fix(id, &text, &extra);
                let twice = fix(id, &once, &extra);
                prop_assert_eq!(&once, &twice, "{} {} not idempotent for {:?}", id, extra, text);
            }
        }
    }

    #[test]
    fn protected_regions_survive(text in doc_strategy()) {
        let before = Document::parse(&text);
        for id in IDS {
            for extra in variants(id) {
                let out = fix(id, &text, &extra);
                for z in before.zones().iter().filter(|z| {
                    (z.kind.is_block() || matches!(z.kind, ZoneKind::InlineCode | ZoneKind::InlineMath | ZoneKind::Html | ZoneKind::Url))
                        && guarded(id, z.kind, z.start, &text)
                }) {
                    let piece = &text[z.start..z.end];
                    // A span that crosses lines is a parser artifact: an edit elsewhere (a list
                    // item that can now interrupt the paragraph) can end it before the next round.
                    if matches!(z.kind, ZoneKind::InlineCode | ZoneKind::InlineMath) && piece.contains('\n') {
                        continue;
                    }
                    // A block that runs to the end of the text may lose or gain a final line break.
                    let piece = if z.end == text.len() { piece.trim_end_matches(['\r', '\n']) } else { piece };
                    prop_assert!(
                        out.contains(piece),
                        "{} {} edited protected {:?} {:?} in {:?} -> {:?}", id, extra, z.kind, piece, text, out
                    );
                }
            }
        }
    }

    #[test]
    fn frontmatter_rules_never_touch_the_body(text in doc_strategy()) {
        let before = Document::parse(&text);
        let Some(fm) = before.zones().iter().find(|z| z.kind == ZoneKind::Frontmatter && z.start == 0) else {
            return Ok(());
        };
        let body = &text[fm.end..];
        for id in YAML.iter().filter(|i| **i != "move-tags-to-yaml") {
            for extra in variants(id) {
                let out = fix(id, &text, &extra);
                prop_assert!(out.ends_with(body), "{} edited the body of {:?} -> {:?}", id, text, out);
            }
        }
    }

    #[test]
    fn check_mode_never_changes_text(text in doc_strategy()) {
        for id in IDS {
            for extra in variants(id) {
                let mut stanza = extra.as_object().cloned().unwrap_or_default();
                stanza.insert("enabled".into(), json!(true));
                let mut rules = serde_json::Map::new();
                rules.insert((*id).to_owned(), Value::Object(stanza));
                let opts = RunOptions { rules, ..Default::default() };
                let out = run(&text, &opts, Mode::Check);
                prop_assert_eq!(&out.text, &text, "{} changed text in check mode", id);
                prop_assert!(!out.changed);
            }
        }
    }
}

#[test]
fn rules_are_off_unless_enabled() {
    let text = "a  b...\n\n\n* x\n";
    let out = run(text, &RunOptions::default(), Mode::Fix);
    assert_eq!(out.text, text);
    for id in IDS {
        let mut rules = serde_json::Map::new();
        rules.insert((*id).to_owned(), json!({"enabled": false}));
        let out = run(
            text,
            &RunOptions {
                rules,
                ..Default::default()
            },
            Mode::Fix,
        );
        assert_eq!(out.text, text, "{id} ran while disabled");
    }
}

/// Insertions around a table and a bare URL land on zone boundaries, never inside a zone.
#[test]
fn table_and_url_edits_stay_outside_zones() {
    use lynter_core::document::Document;
    use lynter_core::rules::get_rule;
    let text = "Intro\n| a | b |\n|---|---|\n| 1 | 2 |\nMore text.\nSee https://example.com/b.\n";
    let doc = Document::parse(text);
    for id in ["empty-line-around-tables", "no-bare-urls"] {
        let rule = get_rule(id).expect("rule");
        let edits = rule.check(&doc, &rule.default_options());
        assert!(!edits.is_empty(), "{id} made no edits");
        for e in edits {
            assert!(
                !doc.is_protected(e.start, e.end),
                "{id} edited protected bytes {}..{}",
                e.start,
                e.end
            );
        }
    }
}
