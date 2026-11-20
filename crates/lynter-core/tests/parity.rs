//! Replays every case exported from the TypeScript core and asserts
//! byte-identical output and identical profile chains.

use lynter_core::config::{parse_config, LoadResult};
use lynter_core::document::Protection;
use lynter_core::rules::get_rule;
use lynter_core::runner::{fix_rounds, fix_with_protection};
use lynter_core::scope::{FileFacts, MatchContext, Resolver};
use serde_json::{json, Value};
use std::path::PathBuf;

fn fixture(name: &str) -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).expect("fixture json")
}

fn rule_parity(id: &str) {
    let fx = fixture(&format!("rules-{id}.json"));
    let rule = get_rule(id).expect("rule");
    let cases = fx["cases"].as_array().expect("cases");
    let mut failures = Vec::new();
    for case in cases {
        let input = case["input"].as_str().expect("input");
        let mut options = rule.default_options();
        for (k, v) in case["options"].as_object().expect("options") {
            options.insert(k.clone(), v.clone());
        }
        // One round is the TypeScript single application; the fixed point is the TypeScript rule iterated until stable.
        // strip-strong and contractions loop internally in TypeScript (to a fixed point and over three ordered passes).
        let rounds = if matches!(id, "strip-strong" | "contractions") {
            64
        } else {
            1
        };
        let got = fix_rounds(rule.as_ref(), input, &options, Protection::LineScan, rounds);
        let want = case["expected"].as_str().expect("expected");
        let got_fixed = fix_with_protection(rule.as_ref(), input, &options, Protection::LineScan);
        let want_fixed = case["expected_fixed"].as_str().expect("expected_fixed");
        if got_fixed != want_fixed {
            failures.push(format!("{} (fixed point)\n  input:    {:?}\n  options:  {}\n  expected: {:?}\n  got:      {:?}", case["name"], input, case["options"], want_fixed, got_fixed));
        }
        if got != want {
            failures.push(format!(
                "{}\n  input:    {:?}\n  options:  {}\n  expected: {:?}\n  got:      {:?}",
                case["name"], input, case["options"], want, got
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{id}: {} of {} cases differ\n{}",
        failures.len(),
        cases.len(),
        failures
            .iter()
            .take(8)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
    println!("{id}: {} cases identical", cases.len());
}

#[test]
fn strip_strong_parity() {
    rule_parity("strip-strong");
}

#[test]
fn replace_em_dash_parity() {
    rule_parity("replace-em-dash");
}

#[test]
fn header_case_parity() {
    rule_parity("header-case");
}

#[test]
fn contractions_parity() {
    rule_parity("contractions");
}

fn strings(v: &Value) -> Vec<String> {
    v.as_array()
        .map(|a| {
            a.iter()
                .filter_map(|s| s.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn scope_parity() {
    let fx = fixture("scope.json");
    let mut resolvers = std::collections::HashMap::new();
    for (name, yaml) in fx["configs"].as_object().expect("configs") {
        match parse_config(yaml.as_str().expect("yaml")) {
            LoadResult::Ok { config, .. } => {
                resolvers.insert(name.clone(), Resolver::new(&config));
            }
            LoadResult::Err(e) => panic!("config {name} rejected: {e:?}"),
        }
    }
    let cases = fx["cases"].as_array().expect("cases");
    let mut failures = Vec::new();
    for case in cases {
        let f = &case["facts"];
        let facts = FileFacts {
            path: f["path"].as_str().expect("path").to_owned(),
            frontmatter: f["frontmatter"].as_str().map(str::to_owned),
            context: MatchContext {
                tags: strings(&f["tags"]),
                today: f["today"].as_str().map(str::to_owned),
                backlinks: strings(&f["backlinks"]),
                aliases: strings(&f["aliases"]),
            },
        };
        let got = resolvers[case["config"].as_str().expect("config")].resolve(&facts);
        let got = json!({
            "chain": got.chain,
            "rules": got.rules,
            "ruleOrder": got.rule_order,
            "noteType": got.note_type,
        });
        if got != case["expected"] {
            failures.push(format!(
                "{}\n  expected: {}\n  got:      {}",
                case["name"], case["expected"], got
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "scope: {} of {} cases differ\n{}",
        failures.len(),
        cases.len(),
        failures
            .iter()
            .take(6)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
    println!("scope: {} cases identical", cases.len());
}

#[test]
fn config_parity() {
    let fx = fixture("config.json");
    let cases = fx["cases"].as_array().expect("cases");
    let mut failures = Vec::new();
    for case in cases {
        let name = case["name"].as_str().expect("name");
        let got = parse_config(case["yaml"].as_str().expect("yaml"));
        let ok = case["ok"].as_bool().expect("ok");
        match (&got, ok) {
            (LoadResult::Ok { warnings, .. }, true) => {
                if json!(warnings) != case["warnings"] {
                    failures.push(format!(
                        "{name}: warnings {:?} vs {}",
                        warnings, case["warnings"]
                    ));
                }
            }
            (LoadResult::Err(errors), false) => {
                // YAML syntax error text is parser specific; every other message must match.
                let want = case["errors"].as_array().expect("errors");
                let is_yaml_err = want.len() == 1
                    && want[0]["message"]
                        .as_str()
                        .is_some_and(|m| m.starts_with("yaml parse error"));
                if !is_yaml_err {
                    let got_json: Vec<Value> = errors
                        .iter()
                        .map(|e| json!({"path": e.path, "message": e.message}))
                        .collect();
                    if Value::Array(got_json.clone()) != Value::Array(want.clone()) {
                        failures.push(format!(
                            "{name}:\n  expected {}\n  got      {}",
                            Value::Array(want.clone()),
                            Value::Array(got_json)
                        ));
                    }
                }
            }
            _ => failures.push(format!("{name}: ok mismatch (expected ok={ok})")),
        }
    }
    assert!(
        failures.is_empty(),
        "config: {} of {} cases differ\n{}",
        failures.len(),
        cases.len(),
        failures.join("\n")
    );
    println!("config: {} cases identical", cases.len());
}

/// With parser-backed protection the output may differ from TypeScript only
/// where the parse protects a region the line scan does not (for example code
/// after a closed fence, or a mid-document `---` block).
#[test]
fn parsed_protection_only_widens() {
    use lynter_core::document::Document;
    use lynter_core::zones::in_zone;
    let mut total = 0;
    let mut differing = 0;
    for id in [
        "strip-strong",
        "replace-em-dash",
        "header-case",
        "contractions",
    ] {
        let fx = fixture(&format!("rules-{id}.json"));
        let rule = get_rule(id).expect("rule");
        for case in fx["cases"].as_array().expect("cases") {
            total += 1;
            let input = case["input"].as_str().expect("input");
            let mut options = rule.default_options();
            for (k, v) in case["options"].as_object().expect("options") {
                options.insert(k.clone(), v.clone());
            }
            let parsed = fix_with_protection(rule.as_ref(), input, &options, Protection::Parsed);
            if parsed == case["expected_fixed"].as_str().expect("expected_fixed") {
                continue;
            }
            differing += 1;
            // The baseline is the line scan with CRLF fences closing, the one
            // deliberate divergence from the TypeScript scan.
            let base = lynter_core::zones::scan_zones_with(
                input,
                &lynter_core::zones::line_spans(input),
                true,
            );
            let full = Document::parse(input);
            let ts_scan =
                lynter_core::zones::scan_zones(input, &lynter_core::zones::line_spans(input));
            let crlf_fence_closed = ts_scan != base;
            let widened = crlf_fence_closed
                || full.zones().iter().any(|z| {
                    !in_zone(&base, z.start, z.end) || {
                        // A parser zone not fully inside one scan zone adds protection.
                        !base.iter().any(|s| s.start <= z.start && s.end >= z.end)
                    }
                });
            assert!(
                widened,
                "{id}: {} differs without a widened protected region",
                case["name"]
            );
        }
    }
    println!("parsed protection differs from TypeScript in {differing} of {total} cases, all in parser-only regions");
}
