//! Obsidian notes rules: tasks, tags, folder and file names, frontmatter.
//!
//! These rules have no TypeScript original; their behavior is covered by
//! `tests/notes.rs`. Report-only checks emit a no-op edit (the replacement is
//! the original text) so they surface as violations without changing bytes.

mod casing;
mod filename_pattern;
mod folder_naming;
mod frontmatter;
mod frontmatter_schema;
mod tag_format;
mod task_format;

pub use casing::CaseStyle;
pub use filename_pattern::FilenamePattern;
pub use folder_naming::FolderNaming;
pub use frontmatter_schema::FrontmatterSchema;
pub use tag_format::TagFormat;
pub use task_format::TaskFormat;

use crate::edit::Edit;
use crate::rule::{Options, Rule};

/// Options key under which a host passes the vault-relative file path to the
/// path-based rules (`folder-naming`, `filename-pattern`).
pub const FILE_PATH_OPTION: &str = "file-path";

/// The notes rules, all off by default.
pub fn notes_rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(TaskFormat),
        Box::new(TagFormat),
        Box::new(FolderNaming),
        Box::new(FilenamePattern),
        Box::new(FrontmatterSchema),
    ]
}

pub(crate) fn opt_bool(o: &Options, key: &str, default: bool) -> bool {
    o.get(key).and_then(|v| v.as_bool()).unwrap_or(default)
}

pub(crate) fn opt_str<'a>(o: &'a Options, key: &str) -> Option<&'a str> {
    o.get(key).and_then(|v| v.as_str())
}

pub(crate) fn opt_usize(o: &Options, key: &str) -> usize {
    o.get(key)
        .and_then(|v| v.as_u64())
        .map_or(0, |n| n as usize)
}

pub(crate) fn opt_list(o: &Options, key: &str) -> Vec<String> {
    match o.get(key) {
        Some(serde_json::Value::Array(a)) => a
            .iter()
            .filter_map(|v| v.as_str().map(str::to_owned))
            .collect(),
        _ => Vec::new(),
    }
}

/// True for a real calendar date.
pub(crate) fn valid_ymd(y: u32, m: u32, d: u32) -> bool {
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let days = match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    y > 0 && (1..=days).contains(&d)
}

/// Collects report-only findings and turns them into no-op edits. Findings
/// on the same range merge into one message, since the runner drops edits
/// with identical ranges.
#[derive(Default)]
pub(crate) struct Reports(Vec<(usize, usize, String)>);

impl Reports {
    pub(crate) fn push(&mut self, start: usize, end: usize, message: impl Into<String>) {
        let message = message.into();
        if let Some(r) = self.0.iter_mut().find(|r| r.0 == start && r.1 == end) {
            r.2.push_str("; ");
            r.2.push_str(&message);
        } else {
            self.0.push((start, end, message));
        }
    }

    pub(crate) fn into_edits(self, text: &str) -> Vec<Edit> {
        self.0
            .into_iter()
            .map(|(s, e, m)| Edit::new(s, e, &text[s..e], m))
            .collect()
    }
}
