//! Language registry keyed by file extension. Each grammar sits behind a
//! cargo feature so a build only links what it needs.

use tree_sitter::Language as TsLanguage;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Language {
    Rust,
    Python,
    TypeScript,
    Tsx,
    JavaScript,
    Sql,
    Go,
    Java,
    CSharp,
}

pub const ALL: [Language; 9] = [
    Language::Rust,
    Language::Python,
    Language::TypeScript,
    Language::Tsx,
    Language::JavaScript,
    Language::Sql,
    Language::Go,
    Language::Java,
    Language::CSharp,
];

/// Lowercase extension of the last path segment.
pub fn extension_of(path: &str) -> Option<String> {
    let base = &path[path.rfind('/').map_or(0, |i| i + 1)..];
    base.rfind('.')
        .filter(|&d| d > 0)
        .map(|d| base[d + 1..].to_lowercase())
}

impl Language {
    /// Language for a lowercase extension, whether or not its grammar is
    /// compiled in. Use [`Language::grammar`] to test availability.
    pub fn from_extension(ext: &str) -> Option<Language> {
        Some(match ext {
            "rs" => Language::Rust,
            "py" | "pyi" => Language::Python,
            "ts" | "mts" | "cts" => Language::TypeScript,
            "tsx" => Language::Tsx,
            "js" | "mjs" | "cjs" | "jsx" => Language::JavaScript,
            "sql" => Language::Sql,
            "go" => Language::Go,
            "java" => Language::Java,
            "cs" => Language::CSharp,
            _ => return None,
        })
    }

    pub fn from_path(path: &str) -> Option<Language> {
        Language::from_extension(&extension_of(path)?)
    }

    /// Name used in rule options and scope lists. TSX shares `typescript`.
    pub fn name(self) -> &'static str {
        match self {
            Language::Rust => "rust",
            Language::Python => "python",
            Language::TypeScript | Language::Tsx => "typescript",
            Language::JavaScript => "javascript",
            Language::Sql => "sql",
            Language::Go => "go",
            Language::Java => "java",
            Language::CSharp => "csharp",
        }
    }

    /// The compiled-in grammar, or `None` when its cargo feature is off.
    pub fn grammar(self) -> Option<TsLanguage> {
        match self {
            #[cfg(feature = "lang-rust")]
            Language::Rust => Some(tree_sitter_rust::LANGUAGE.into()),
            #[cfg(feature = "lang-python")]
            Language::Python => Some(tree_sitter_python::LANGUAGE.into()),
            #[cfg(feature = "lang-typescript")]
            Language::TypeScript => Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
            #[cfg(feature = "lang-typescript")]
            Language::Tsx => Some(tree_sitter_typescript::LANGUAGE_TSX.into()),
            #[cfg(feature = "lang-javascript")]
            Language::JavaScript => Some(tree_sitter_javascript::LANGUAGE.into()),
            #[cfg(feature = "lang-sql")]
            Language::Sql => Some(tree_sitter_sequel::LANGUAGE.into()),
            #[cfg(feature = "lang-go")]
            Language::Go => Some(tree_sitter_go::LANGUAGE.into()),
            #[cfg(feature = "lang-java")]
            Language::Java => Some(tree_sitter_java::LANGUAGE.into()),
            #[cfg(feature = "lang-csharp")]
            Language::CSharp => Some(tree_sitter_c_sharp::LANGUAGE.into()),
            #[allow(unreachable_patterns)]
            _ => None,
        }
    }

    pub fn is_available(self) -> bool {
        self.grammar().is_some()
    }

    /// Parse `text`, or `None` when the grammar is not compiled in.
    pub fn parse(self, text: &str) -> Option<tree_sitter::Tree> {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&self.grammar()?).ok()?;
        parser.parse(text, None)
    }
}

/// Languages whose grammar is compiled into this build.
pub fn available() -> Vec<Language> {
    ALL.iter().copied().filter(|l| l.is_available()).collect()
}
