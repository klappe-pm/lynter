//! Whole-line content rules ported from the TypeScript core: list style,
//! paragraph spacing, paragraph joining, prose lists, and code checks.

mod code_checks;
mod join_paragraph_lines;
mod list_style;
mod paragraph_spacing;
mod prose_list;
pub mod util;

pub use code_checks::{CodeChecks, CodeChecksFix};
pub use join_paragraph_lines::{
    is_joinable_prose, JoinParagraphLines, JoinParagraphLinesFix, INTENTIONAL_BREAK,
};
pub use list_style::ListStyle;
pub use paragraph_spacing::ParagraphSpacing;
pub use prose_list::{ProseListToSentences, ProseListToSentencesFix};

use crate::rule::Rule;

/// The rules in this family, in registry order.
pub fn rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(JoinParagraphLines),
        Box::new(JoinParagraphLinesFix),
        Box::new(ProseListToSentences),
        Box::new(ProseListToSentencesFix),
        Box::new(ParagraphSpacing),
        Box::new(ListStyle),
        Box::new(CodeChecks),
        Box::new(CodeChecksFix),
    ]
}
