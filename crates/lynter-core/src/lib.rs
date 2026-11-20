//! Lynter core: document model, rule runner, config loader, scope engine, and
//! the Markdown content rules.

#![forbid(unsafe_code)]

pub mod config;
pub mod document;
pub mod edit;
pub mod lint;
pub mod notetypes;
pub mod rule;
pub mod rules;
pub mod runner;
pub mod scope;
pub mod zones;

pub use document::Document;
pub use edit::Edit;
pub use rule::{EditScope, Options, Rule};
pub use runner::{run, Mode, RunOptions, RunResult, Violation};
