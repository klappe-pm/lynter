//! Built-in rules: JSON and YAML data rules, `identifier-case`, and the
//! cross-language rules `syntax-error`, `comment-tag`, `commented-out-code`,
//! `max-nesting-depth` and `max-function-lines`.

pub mod comment_tag;
pub mod commented_out_code;
pub mod identifier_case;
pub mod json;
pub mod json_parse;
pub mod kinds;
pub mod structure;
pub mod syntax_error;
pub mod yaml;

use crate::rule::SourceFile;
use lynter_core::Options;
use serde_json::Value;

pub(crate) fn ext_in(f: &SourceFile, exts: &[&str]) -> bool {
    f.extension().is_some_and(|e| exts.contains(&e.as_str()))
}

pub(crate) fn opt_strings(o: &Options, key: &str) -> Vec<String> {
    o.get(key)
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

pub(crate) fn opt_str<'a>(o: &'a Options, key: &str, default: &'a str) -> &'a str {
    o.get(key).and_then(Value::as_str).unwrap_or(default)
}

pub(crate) fn opt_bool(o: &Options, key: &str, default: bool) -> bool {
    o.get(key).and_then(Value::as_bool).unwrap_or(default)
}

/// For each output slot, the index of the key that belongs there. Priority
/// keys come first in list order, then the rest alphabetically (or in their
/// original order when `alpha` is false).
pub(crate) fn key_permutation(keys: &[String], priority: &[String], alpha: bool) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..keys.len()).collect();
    idx.sort_by(|&a, &b| {
        let rank = |i: usize| priority.iter().position(|p| *p == keys[i]);
        match (rank(a), rank(b)) {
            (Some(x), Some(y)) => x.cmp(&y).then(a.cmp(&b)),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) if alpha => keys[a].cmp(&keys[b]).then(a.cmp(&b)),
            (None, None) => a.cmp(&b),
        }
    });
    idx
}
