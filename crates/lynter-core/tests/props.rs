//! Property tests: fix is idempotent and never edits protected regions.

use lynter_core::document::{Document, Protection};
use lynter_core::edit::normalize;
use lynter_core::rule::{EditScope, Options};
use lynter_core::rules::builtin_rules;
use lynter_core::runner::{fix_with_protection, run, Mode, RunOptions};
use lynter_core::zones::ZoneKind;
use proptest::prelude::*;
use serde_json::{json, Value};

const FRAGMENTS: &[&str] = &[
    "plain words",
    "It is not ready",
    "we are here",
    "do not go",
    "I am sure",
    "CANNOT stop",
    "it is, but they are not",
    "**bold** text",
    "__under__ score",
    "***both*** ok",
    "a \u{2014} b",
    "a\u{2014}b",
    "10\u{2013}12",
    "x \u{2013} y",
    "`code **x** it is not`",
    "$a**b$",
    "$$it is not$$",
    "[[wiki **link** it is]]",
    "https://ex.com/a**b**c",
    "[t](http://x.y/**z**)",
    "# the quick BROWN fox",
    "## Some Section_here",
    "### camelCaseHeading",
    "#hashtag",
    "# ",
    "#",
    "| a | b |",
    "| --- | --- |",
    "| it is | **x** |",
    "- list **item** it is",
    "> quote it is not",
    "<div>it is not **x**</div>",
    "    indented **x** it is not",
    "- [ ] task it is",
    "\u{1F600} it is **ok**",
    "caf\u{e9} it is",
    "```",
    "~~~",
    "---",
    "$$",
    "**",
    "__",
    "\u{2014}",
    "it  is",
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

fn options_for(id: &str, variant: usize) -> Options {
    let v: Value = match (id, variant % 3) {
        ("header-case", 0) => json!({"h1": "kebab-case", "h2": "camelCase", "h3": "Title Case"}),
        ("header-case", 1) => json!({"h1": "First letter", "h2": "underscore_formatted"}),
        ("header-case", _) => {
            json!({"h1": "Title Case", "h2": "Title Case", "h3": "Title Case", "h4": "kebab-case"})
        }
        ("replace-em-dash", 1) => json!({"word-joining": true}),
        ("replace-em-dash", 2) => json!({"replacement": ": "}),
        ("strip-strong", 1) => json!({"keep-heading-strong": true}),
        ("contractions", 1) => json!({"it is": false, "do not": false}),
        _ => json!({}),
    };
    let mut options = builtin_rules()
        .into_iter()
        .find(|r| r.id() == id)
        .map(|r| r.default_options())
        .unwrap_or_default();
    for (k, val) in v.as_object().cloned().unwrap_or_default() {
        options.insert(k, val);
    }
    options
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(400))]

    /// Fixing a fixed text changes nothing, and check then finds no violation.
    #[test]
    fn fix_is_idempotent(text in doc_strategy(), variant in 0usize..3) {
        for rule in builtin_rules() {
            let options = options_for(rule.id(), variant);
            let once = fix_with_protection(rule.as_ref(), &text, &options, Protection::Parsed);
            let twice = fix_with_protection(rule.as_ref(), &once, &options, Protection::Parsed);
            prop_assert_eq!(&once, &twice, "rule {} not idempotent", rule.id());
            let doc = Document::parse(&once);
            prop_assert!(normalize(rule.check(&doc, &options)).is_empty(), "rule {} leaves edits after fix", rule.id());
        }
    }

    /// No edit overlaps a protected region. header-case rewrites whole heading
    /// lines, so for it only whole-line (block) regions are protected.
    #[test]
    fn edits_never_touch_protected_regions(text in doc_strategy(), variant in 0usize..3) {
        let doc = Document::parse(&text);
        for rule in builtin_rules() {
            let options = options_for(rule.id(), variant);
            for e in rule.check(&doc, &options) {
                if rule.id() == "header-case" {
                    let line = doc.lines().iter().find(|l| l.start == e.start).expect("edit starts at a line");
                    prop_assert!(!doc.is_line_block_masked(line), "header-case edited a protected line");
                } else if rule.edit_scope() == EditScope::Frontmatter {
                    // A frontmatter rule edits only inside the frontmatter zone, and
                    // never inside any other protected zone.
                    // With no frontmatter yet, the rule may create it by inserting at offset 0.
                    let fm = doc.zones().iter().find(|z| z.kind == ZoneKind::Frontmatter).map_or(
                        lynter_core::zones::Zone { start: 0, end: 0, kind: ZoneKind::Frontmatter },
                        |z| *z,
                    );
                    prop_assert!(
                        e.start >= fm.start && e.end <= fm.end,
                        "{} edited outside frontmatter {}..{}", rule.id(), e.start, e.end
                    );
                    // Parse-level metadata zones that merely cover the frontmatter do not count.
                    let (lo, hi) = (e.start, e.end);
                    let empty = e.start == e.end;
                    let others = doc.zones().iter().any(|z| {
                        z.kind != ZoneKind::Frontmatter
                            && !(z.start >= fm.start && z.end <= fm.end)
                            && !(z.kind == ZoneKind::ParsedBlock && z.start <= fm.start && z.end >= fm.end)
                            && if empty { z.start < lo && z.end > lo } else { z.start < hi && z.end > lo }
                    });
                    prop_assert!(!others, "{} edited another protected zone {}..{}", rule.id(), e.start, e.end);
                } else {
                    prop_assert!(!doc.is_protected(e.start, e.end), "{} edited protected bytes {}..{}", rule.id(), e.start, e.end);
                }
            }
        }
    }

    /// The full pipeline leaves every protected block region byte-identical.
    #[test]
    fn protected_blocks_survive_a_full_fix(text in doc_strategy()) {
        let rules = json!({
            "strip-strong": {"enabled": true}, "replace-em-dash": {"enabled": true},
            "contractions": {"enabled": true},
            "header-case": {"enabled": true, "h1": "kebab-case", "h2": "camelCase"},
        });
        let opts = RunOptions { rules: rules.as_object().cloned().unwrap_or_default(), ..Default::default() };
        let out = run(&text, &opts, Mode::Fix);
        let before = Document::parse(&text);
        let after = Document::parse(&out.text);
        let blocks = |d: &Document| -> Vec<String> {
            d.zones().iter().filter(|z| z.kind.is_block()).map(|z| d.text()[z.start..z.end].to_owned()).collect()
        };
        // Every block region of the input appears unchanged in the output.
        let out_blocks = blocks(&after);
        for b in blocks(&before) {
            prop_assert!(out_blocks.contains(&b) || out.text.contains(&b), "block changed: {:?}", b);
        }
        // And the whole pipeline is idempotent.
        prop_assert_eq!(run(&out.text, &opts, Mode::Fix).text, out.text);
    }
}
