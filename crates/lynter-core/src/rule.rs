//! The rule contract: a pure function from a document and options to edits.

use crate::document::Document;
use crate::edit::Edit;
use serde_json::{Map, Value};

pub type Options = Map<String, Value>;

/// Where a rule is allowed to edit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditScope {
    /// Edits stay outside every protected zone.
    Body,
    /// Edits stay inside the frontmatter zone.
    Frontmatter,
}

pub trait Rule: Send + Sync {
    /// Stable kebab-case identifier.
    fn id(&self) -> &'static str;
    /// One-line description.
    fn description(&self) -> &'static str;
    /// Options applied when a profile does not override them.
    fn default_options(&self) -> Options;
    /// Edits for one round. `check` and `fix` both call this.
    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit>;
    /// Declared edit scope; `Body` unless the rule rewrites frontmatter.
    fn edit_scope(&self) -> EditScope {
        EditScope::Body
    }
}
