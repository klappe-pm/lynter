//! The code rule contract. Rules return core [`Edit`]s: an edit whose
//! replacement equals the text it covers is a report-only finding, exactly as
//! in the core runner.

use crate::language::Language;
use lynter_core::edit::Edit;
use lynter_core::Options;
use tree_sitter::Tree;

/// One file as a code rule sees it.
pub struct SourceFile<'a> {
    pub text: &'a str,
    pub path: &'a str,
    /// Set when the extension maps to a compiled-in grammar.
    pub language: Option<Language>,
    /// The parse tree, present when `language` is set.
    pub tree: Option<&'a Tree>,
}

impl SourceFile<'_> {
    pub fn extension(&self) -> Option<String> {
        crate::language::extension_of(self.path)
    }
}

pub trait CodeRule: Send + Sync {
    /// Stable kebab-case identifier.
    fn id(&self) -> &'static str;
    /// One-line description.
    fn description(&self) -> &'static str;
    /// Options applied when a profile does not override them.
    fn default_options(&self) -> Options;
    /// True when the rule wants this file (extension or language).
    fn applies_to(&self, file: &SourceFile) -> bool;
    /// Edits for one round.
    fn check(&self, file: &SourceFile, options: &Options) -> Vec<Edit>;
}

/// A no-op edit over `[start, end)`: a finding without a fix.
pub fn report(text: &str, start: usize, end: usize, message: impl Into<String>) -> Edit {
    Edit::new(start, end, text[start..end].to_owned(), message)
}
