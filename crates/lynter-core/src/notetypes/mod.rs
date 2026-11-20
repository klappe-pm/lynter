//! Frontmatter, note-type, and filename rules.

pub mod filename;
pub mod format;
pub mod frontmatter;
pub mod key_rank;
pub mod locale;
pub mod schema;
mod validate;
mod yaml_rules;

pub use filename::{
    derive_compliant_stem, is_reserved_stem, kebab_case_name, path_stem, propose_rename,
    resolve_collision, H1MatchesStem, KebabCaseFilename, RenameProposal, RenameStrategy,
};
pub use validate::{collect_note_type_violations, NoteTypeValidate};
pub use yaml_rules::{
    apply_date_keys, AlphabetizePropertyValues, NoteTypeDateKeys, NoteTypeInsertKeys,
    NoteTypeKeySort, YamlKeySort, YamlTimestamp,
};

use crate::rule::Rule;

/// The rules in this family, in registry order.
pub fn rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(YamlKeySort),
        Box::new(AlphabetizePropertyValues),
        Box::new(YamlTimestamp),
        Box::new(NoteTypeInsertKeys),
        Box::new(NoteTypeKeySort),
        Box::new(NoteTypeDateKeys),
        Box::new(NoteTypeValidate),
        Box::new(H1MatchesStem),
        Box::new(KebabCaseFilename),
    ]
}
