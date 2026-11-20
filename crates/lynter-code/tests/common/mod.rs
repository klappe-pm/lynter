#![allow(dead_code)]
use lynter_code::lint_source;
use lynter_core::config::{parse_config, LoadResult};
use lynter_core::lint::LintResult;
use lynter_core::scope::Resolver;

pub fn run(config: &str, path: &str, text: &str) -> Option<LintResult> {
    let LoadResult::Ok { config, .. } = parse_config(config) else {
        panic!("config rejected");
    };
    lint_source(text, path, &Resolver::new(&config), Some("2026-01-01"))
}

pub fn messages(r: &LintResult) -> Vec<String> {
    r.violations
        .iter()
        .map(|v| format!("{}:{}:{}", v.rule, v.line, v.message))
        .collect()
}
