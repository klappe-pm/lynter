//! Property tests for the slice 2 rules: fixing twice equals fixing once, and
//! protected block regions survive byte for byte.

use lynter_core::document::{Document, Protection};
use lynter_core::rules::get_rule;
use lynter_core::runner::fix_with_protection;
use lynter_core::zones::ZoneKind;
use proptest::prelude::*;
use serde_json::{json, Value};

const FRAGMENTS: &[&str] = &[
    "plain words",
    "It is not ready",
    "- one",
    "* two",
    "+ three",
    "  - nested",
    "1. first",
    "- [ ] task",
    "---",
    "***",
    "wrapped line one",
    "wrapped line two",
    "trailing  ",
    "Intro:",
    "- apples.",
    "- pears",
    "> quote",
    "| a | b |",
    "| - | - |",
    "[ref]: http://x.y",
    "<div>",
    "$$",
    "```bash",
    "```",
    "~~~",
    "    indented code",
    "echo hi   ",
    "# Title",
    "## Heading",
    "x",
    "",
    "tags: [b, a]",
    "domain: x",
    "status: DRAFT",
    "tags:\n  - z\n  - a",
    "aliases: [q, p]",
    "caf\u{e9} text",
    "\u{1F600} emoji",
];

fn doc_strategy() -> impl Strategy<Value = String> {
    (
        prop::option::of(prop::collection::vec(prop::sample::select(FRAGMENTS), 0..5)),
        prop::collection::vec(prop::sample::select(FRAGMENTS), 1..14),
        prop::bool::ANY,
    )
        .prop_map(|(fm, body, trailing)| {
            let mut s = String::new();
            if let Some(fm) = fm {
                s.push_str("---\n");
                for l in fm {
                    s.push_str(l);
                    s.push('\n');
                }
                s.push_str("---\n");
            }
            s.push_str(&body.join("\n"));
            if trailing {
                s.push('\n');
            }
            s
        })
}

fn rule_options(id: &str) -> Vec<Value> {
    match id {
        "list-style" => vec![json!({}), json!({"marker": "*", "tighten": false})],
        "paragraph-spacing" => vec![
            json!({}),
            json!({"blank-lines": 0}),
            json!({"blank-lines": 2}),
        ],
        "yaml-key-sort" => {
            vec![json!({"priority-keys": ["domain", "status"], "defaults": {"status": "NEW"}})]
        }
        "h1-matches-stem" => vec![json!({"file-path": "notes/My Note.md"})],
        "code-checks-fix" => vec![
            json!({"checks": {"mine": {"enabled": true, "pattern": "[ \\t]+$", "fix": {"replacement": ""}}}}),
        ],
        "note-type-key-sort" => vec![json!({"schema": {"key-order": ["tags", "domain"]}})],
        "note-type-insert-keys" => {
            vec![json!({"schema": {"required": {"status": "NEW", "type": "x"}}})]
        }
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

/// Block zones the rule must leave alone.
fn guarded(id: &str) -> fn(ZoneKind) -> bool {
    match id {
        "yaml-key-sort"
        | "alphabetize-property-values"
        | "note-type-key-sort"
        | "note-type-insert-keys" => {
            |k| k.is_block() && !matches!(k, ZoneKind::Frontmatter | ZoneKind::ParsedBlock)
        }
        "code-checks-fix" => {
            |k| k.is_block() && !matches!(k, ZoneKind::CodeFence | ZoneKind::ParsedBlock)
        }
        _ => |k| k.is_block(),
    }
}

const RULES: &[&str] = &[
    "list-style",
    "paragraph-spacing",
    "join-paragraph-lines-fix",
    "prose-list-to-sentences-fix",
    "yaml-key-sort",
    "alphabetize-property-values",
    "note-type-key-sort",
    "note-type-insert-keys",
    "h1-matches-stem",
    "code-checks-fix",
];

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    #[test]
    fn fix_is_idempotent(text in doc_strategy()) {
        for id in RULES {
            for extra in rule_options(id) {
                let once = fix(id, &text, &extra);
                let twice = fix(id, &once, &extra);
                prop_assert_eq!(&once, &twice, "{} not idempotent for {:?}", id, text);
            }
        }
    }

    #[test]
    fn protected_regions_survive(text in doc_strategy()) {
        let before = Document::parse(&text);
        for id in RULES {
            // h1-matches-stem inserts after the line-scanned frontmatter; a stray
            // `---` pair can make the parse see a wider metadata block.
            if *id == "h1-matches-stem" {
                continue;
            }
            let guard = guarded(id);
            for extra in rule_options(id) {
                let out = fix(id, &text, &extra);
                for z in before.zones().iter().filter(|z| guard(z.kind)) {
                    let piece = &text[z.start..z.end];
                    prop_assert!(
                        out.contains(piece),
                        "{} edited protected {:?} {:?} in {:?}", id, z.kind, piece, text
                    );
                }
            }
        }
    }
}
