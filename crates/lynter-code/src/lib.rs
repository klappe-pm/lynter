//! Lynter code host: a tree-sitter language registry keyed by file extension,
//! the `CodeRule` contract, the JSON and YAML data rules, one built-in code
//! rule (`identifier-case`), and a runner that applies them with the options
//! of the profile the core scope engine resolves for the file.
//!
//! Scope note for core: profile matching has no `languages` matcher. Profiles
//! match by `extension` or `path` today. Until core adds one, a rule stanza
//! may carry a `languages: [rust, python]` option that this crate honours.

#![forbid(unsafe_code)]

#[cfg(not(target_arch = "wasm32"))]
pub mod adapters;
pub mod language;
pub mod rule;
pub mod rules;
pub mod runner;
pub mod text;

pub use language::Language;
pub use rule::{CodeRule, SourceFile};
pub use runner::{handles_extension, lint_source, registry};
