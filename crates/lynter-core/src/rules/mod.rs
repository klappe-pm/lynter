//! Content rules ported from the TypeScript core.

pub mod compat;
pub mod content;
mod contractions;
mod header_case;
mod notes;
mod replace_em_dash;
mod strip_strong;

pub use contractions::{Contractions, CONTRACTIONS};
pub use header_case::{format_heading_text, HeaderCase, HeaderCaseStyle};
pub use notes::{
    notes_rules, CaseStyle, FilenamePattern, FolderNaming, FrontmatterSchema, TagFormat,
    TaskFormat, FILE_PATH_OPTION,
};
pub use replace_em_dash::ReplaceEmDash;
pub use strip_strong::StripStrong;

use crate::rule::Rule;

/// Every built-in rule, in registry order.
pub fn builtin_rules() -> Vec<Box<dyn Rule>> {
    let mut rules: Vec<Box<dyn Rule>> = vec![
        Box::new(StripStrong),
        Box::new(ReplaceEmDash),
        Box::new(HeaderCase),
        Box::new(Contractions),
    ];
    rules.extend(content::rules());
    rules.extend(compat::rules());
    rules.extend(crate::notetypes::rules());
    rules.extend(notes_rules());
    rules
}

pub fn get_rule(id: &str) -> Option<Box<dyn Rule>> {
    builtin_rules().into_iter().find(|r| r.id() == id)
}

/// Next char boundary after `i`.
pub(crate) fn next_char(s: &str, i: usize) -> usize {
    i + s[i..].chars().next().map_or(1, char::len_utf8)
}
