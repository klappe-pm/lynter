//! Table tests for the compat rules: every example in
//! `tests/fixtures/compat/<id>.json` (input, options, expected output) is run
//! through its rule, plus the never-edited guarantees as direct tests.

use lynter_core::document::Protection;
use lynter_core::rule::Options;
use lynter_core::rules::get_rule;
use lynter_core::runner::fix_with_protection;
use serde_json::{json, Value};

struct Example {
    n: usize,
    input: String,
    options: Value,
    expected: Option<String>,
}

/// Loads `tests/fixtures/compat/<id>.json`: `{id, examples: [{n, input, options, expected}]}`.
/// An example with a null `expected` has no specified output and is skipped.
fn load_examples(id: &str) -> Vec<Example> {
    let path = format!(
        "{}/tests/fixtures/compat/{id}.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let src = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let v: Value = serde_json::from_str(&src).unwrap_or_else(|e| panic!("{path}: {e}"));
    assert_eq!(v["id"], id, "{path}: id mismatch");
    v["examples"]
        .as_array()
        .expect("examples array")
        .iter()
        .map(|e| Example {
            n: e["n"].as_u64().expect("n") as usize,
            input: e["input"].as_str().expect("input").to_owned(),
            options: e["options"].clone(),
            expected: e["expected"].as_str().map(str::to_owned),
        })
        .collect()
}

fn options_of(id: &str, extra: &Value) -> Options {
    let mut o = get_rule(id).expect("rule is registered").default_options();
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

fn run_spec(id: &str) {
    let examples = load_examples(id);
    assert!(
        examples.len() >= 6,
        "{id}: parsed {} examples",
        examples.len()
    );
    let mut failures = Vec::new();
    for ex in &examples {
        let Some(expected) = &ex.expected else {
            continue;
        };
        let extra = &ex.options;
        let got = fix(id, &ex.input, extra);
        if &got != expected {
            failures.push(format!(
                "example {} options {extra}\n  input:    {:?}\n  expected: {:?}\n  got:      {:?}",
                ex.n, ex.input, expected, got
            ));
        }
        let again = fix(id, &got, extra);
        if again != got {
            failures.push(format!(
                "example {} not idempotent: {:?} -> {:?}",
                ex.n, got, again
            ));
        }
    }
    assert!(failures.is_empty(), "{id}:\n{}", failures.join("\n"));
}

macro_rules! spec_tests {
    ($($name:ident => $id:literal),* $(,)?) => {
        $( #[test] fn $name() { run_spec($id); } )*
    };
}

spec_tests! {
    spec_consecutive_blank_lines => "consecutive-blank-lines",
    spec_dedupe_yaml_array_values => "dedupe-yaml-array-values",
    spec_default_language_for_code_fences => "default-language-for-code-fences",
    spec_emphasis_style => "emphasis-style",
    spec_empty_line_around_blockquotes => "empty-line-around-blockquotes",
    spec_empty_line_around_code_fences => "empty-line-around-code-fences",
    spec_empty_line_around_horizontal_rules => "empty-line-around-horizontal-rules",
    spec_empty_line_around_math_block => "empty-line-around-math-block",
    spec_empty_line_around_tables => "empty-line-around-tables",
    spec_escape_yaml_special_characters => "escape-yaml-special-characters",
    spec_format_tags_in_yaml => "format-tags-in-yaml",
    spec_format_yaml_arrays => "format-yaml-arrays",
    spec_header_increment => "header-increment",
    spec_heading_blank_lines => "heading-blank-lines",
    spec_headings_start_line => "headings-start-line",
    spec_insert_yaml_attributes => "insert-yaml-attributes",
    spec_line_break_at_document_end => "line-break-at-document-end",
    spec_move_tags_to_yaml => "move-tags-to-yaml",
    spec_no_bare_urls => "no-bare-urls",
    spec_ordered_list_style => "ordered-list-style",
    spec_proper_ellipsis => "proper-ellipsis",
    spec_quote_style => "quote-style",
    spec_remove_empty_list_markers => "remove-empty-list-markers",
    spec_remove_multiple_spaces => "remove-multiple-spaces",
    spec_remove_trailing_punctuation_in_heading => "remove-trailing-punctuation-in-heading",
    spec_remove_yaml_keys => "remove-yaml-keys",
    spec_space_after_list_markers => "space-after-list-markers",
    spec_strong_style => "strong-style",
    spec_trailing_spaces => "trailing-spaces",
    spec_unordered_list_style => "unordered-list-style",
}

#[test]
fn registry_has_all_thirty_ids_off_by_default() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/compat");
    let mut ids: Vec<String> = std::fs::read_dir(&dir)
        .expect("compat fixtures")
        .filter_map(|e| {
            let p = e.expect("entry").path();
            (p.extension()? == "json")
                .then(|| p.file_stem().expect("stem").to_string_lossy().into_owned())
        })
        .collect();
    ids.sort();
    let mut count = 0;
    for id in &ids {
        let rule = get_rule(id).unwrap_or_else(|| panic!("{id} not registered"));
        assert_eq!(rule.id(), id);
        // Off by default: nothing enables a rule without an explicit `enabled: true`.
        let opts = lynter_core::runner::RunOptions::default();
        let out =
            lynter_core::runner::run("a  b\n\n\n- x\n", &opts, lynter_core::runner::Mode::Fix);
        assert!(!out.changed, "{id}");
        count += 1;
    }
    assert_eq!(count, 30);
}

#[test]
fn header_increment_open_question_example_follows_the_stack_procedure() {
    // The spec leaves this input unspecified; the stack procedure is the decision of record.
    let out = fix(
        "header-increment",
        "# H1\n### H3\n###### H6\n##### H5\n## H2\n###### H6\n",
        &json!({}),
    );
    assert_eq!(out, "# H1\n## H3\n### H6\n### H5\n## H2\n### H6\n");
}

// ---- never-edited guarantees ------------------------------------------------

const PROTECTED_BODY: &str = "\n```\n  trailing   \n\n\n# not heading.\n- \n1.   x\n*a* **b** \"q\" ... https://a.b/c\n```\n\n    indented   code  \n\n`inline   code...` $$x  ...  y$$\n\n<div>html   ... \"x\"</div>\n";

fn assert_protected_untouched(id: &str, extra: &Value) {
    let rule = get_rule(id).expect("rule");
    let text = format!("---\ntitle: a  b\n---\n{PROTECTED_BODY}");
    let out = fix_with_protection(
        rule.as_ref(),
        &text,
        &options_of(id, extra),
        Protection::Parsed,
    );
    for piece in [
        "  trailing   \n\n\n# not heading.\n- \n1.   x\n*a* **b** \"q\" ... https://a.b/c\n",
        "    indented   code  ",
        "`inline   code...`",
        "$$x  ...  y$$",
        "<div>html   ... \"x\"</div>",
        "title: a  b",
    ] {
        assert!(
            out.contains(piece),
            "{id} changed protected {piece:?}\n{out:?}"
        );
    }
}

#[test]
fn body_rules_never_edit_code_math_html_or_frontmatter() {
    let cases: &[(&str, Value)] = &[
        ("consecutive-blank-lines", json!({})),
        ("emphasis-style", json!({"style": "underscore"})),
        ("strong-style", json!({"style": "underscore"})),
        ("empty-line-around-blockquotes", json!({})),
        ("empty-line-around-code-fences", json!({})),
        ("empty-line-around-horizontal-rules", json!({})),
        ("empty-line-around-math-block", json!({})),
        ("empty-line-around-tables", json!({})),
        ("header-increment", json!({})),
        ("heading-blank-lines", json!({})),
        ("headings-start-line", json!({})),
        ("no-bare-urls", json!({})),
        (
            "ordered-list-style",
            json!({"number-style": "lazy", "end-style": ")"}),
        ),
        ("proper-ellipsis", json!({})),
        (
            "quote-style",
            json!({"single-quote-style": "smart", "double-quote-style": "smart"}),
        ),
        ("remove-empty-list-markers", json!({})),
        ("remove-multiple-spaces", json!({})),
        ("remove-trailing-punctuation-in-heading", json!({})),
        ("space-after-list-markers", json!({})),
        ("trailing-spaces", json!({})),
        ("unordered-list-style", json!({"style": "*"})),
    ];
    for (id, extra) in cases {
        assert_protected_untouched(id, extra);
    }
}

#[test]
fn yaml_rules_never_edit_the_body() {
    let body = "# T\n- a\n- a\ntags: [a, a]\nkey: value: x\n#tag-in-body\n";
    let cases: &[(&str, Value)] = &[
        ("dedupe-yaml-array-values", json!({})),
        (
            "escape-yaml-special-characters",
            json!({"escape-single-line-arrays": true}),
        ),
        ("format-tags-in-yaml", json!({})),
        (
            "format-yaml-arrays",
            json!({"default-array-style": "multi-line"}),
        ),
        ("insert-yaml-attributes", json!({})),
        ("remove-yaml-keys", json!({"keys": ["tags", "key"]})),
    ];
    for (id, extra) in cases {
        let text = format!("---\ntags: [#x, #x]\nkey: a: b\n---\n{body}");
        let out = fix(id, &text, extra);
        assert!(out.ends_with(body), "{id} edited the body: {out:?}");
    }
}

#[test]
fn trailing_and_report_only_cases() {
    // A rule with nothing to do never changes text.
    for id in [
        "default-language-for-code-fences",
        "remove-yaml-keys",
        "insert-yaml-attributes",
    ] {
        let extra = if id == "insert-yaml-attributes" {
            json!({"text-to-insert": ""})
        } else {
            json!({})
        };
        let text = "---\na: 1\n---\n```\nx\n```\n";
        assert_eq!(fix(id, text, &extra), text, "{id}");
    }
}

#[test]
fn quote_style_odd_count_reports_a_diagnostic_and_converts_the_other_kind() {
    let rule = get_rule("quote-style").expect("rule");
    let opts = options_of(
        "quote-style",
        &json!({"single-quote-style": "smart", "double-quote-style": "smart"}),
    );
    let text = "\"one\" and \"two and 'x' y\n";
    let doc = lynter_core::Document::parse(text);
    let edits = rule.check(&doc, &opts);
    assert!(
        edits.iter().any(|e| e.message.contains("odd number")),
        "{edits:?}"
    );
    let out = fix_with_protection(rule.as_ref(), text, &opts, Protection::Parsed);
    assert_eq!(out, "\"one\" and \"two and \u{2018}x\u{2019} y\n");
}

#[test]
fn spec_conflicts_follow_numbered_behavior() {
    // heading-blank-lines: behavior 5 says no blank is added before a heading right after
    // frontmatter, but example 4 and "exactly one blank line" require one. The numbered
    // statement about exactly one blank line wins.
    let out = fix(
        "heading-blank-lines",
        "---\nk: v\n---\n# H\ntext\n",
        &json!({}),
    );
    assert_eq!(out, "---\nk: v\n---\n\n# H\n\ntext\n");
}
