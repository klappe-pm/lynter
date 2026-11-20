//! Compat rules: behavior-spec rules for spacing, lists, headings, inline
//! text, and frontmatter. Every rule here is off until a profile enables it.

mod common;
mod emphasis;
mod headings;
mod lists;
mod spacing;
mod text;
mod yaml;
mod yaml_rules;

pub use emphasis::{EmphasisStyle, StrongStyle};
pub use headings::{HeaderIncrement, HeadingsStartLine, RemoveTrailingPunctuationInHeading};
pub use lists::{
    OrderedListStyle, RemoveEmptyListMarkers, SpaceAfterListMarkers, UnorderedListStyle,
};
pub use spacing::{
    ConsecutiveBlankLines, EmptyLineAroundBlockquotes, EmptyLineAroundCodeFences,
    EmptyLineAroundHorizontalRules, EmptyLineAroundMathBlock, EmptyLineAroundTables,
    HeadingBlankLines, LineBreakAtDocumentEnd,
};
pub use text::{
    DefaultLanguageForCodeFences, NoBareUrls, ProperEllipsis, QuoteStyle, RemoveMultipleSpaces,
    TrailingSpaces,
};
pub use yaml_rules::{
    DedupeYamlArrayValues, EscapeYamlSpecialCharacters, FormatTagsInYaml, FormatYamlArrays,
    InsertYamlAttributes, MoveTagsToYaml, RemoveYamlKeys,
};

use crate::rule::Rule;

/// The rules in this family, in registry order.
pub fn rules() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(ConsecutiveBlankLines),
        Box::new(DedupeYamlArrayValues),
        Box::new(DefaultLanguageForCodeFences),
        Box::new(EmphasisStyle),
        Box::new(EmptyLineAroundBlockquotes),
        Box::new(EmptyLineAroundCodeFences),
        Box::new(EmptyLineAroundHorizontalRules),
        Box::new(EmptyLineAroundMathBlock),
        Box::new(EmptyLineAroundTables),
        Box::new(EscapeYamlSpecialCharacters),
        Box::new(FormatTagsInYaml),
        Box::new(FormatYamlArrays),
        Box::new(HeaderIncrement),
        Box::new(HeadingBlankLines),
        Box::new(HeadingsStartLine),
        Box::new(InsertYamlAttributes),
        Box::new(LineBreakAtDocumentEnd),
        Box::new(MoveTagsToYaml),
        Box::new(NoBareUrls),
        Box::new(OrderedListStyle),
        Box::new(ProperEllipsis),
        Box::new(QuoteStyle),
        Box::new(RemoveEmptyListMarkers),
        Box::new(RemoveMultipleSpaces),
        Box::new(RemoveTrailingPunctuationInHeading),
        Box::new(RemoveYamlKeys),
        Box::new(SpaceAfterListMarkers),
        Box::new(StrongStyle),
        Box::new(TrailingSpaces),
        Box::new(UnorderedListStyle),
    ]
}
