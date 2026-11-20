//! The one place the CLI composes config, scope resolution, and the rule
//! hosts: Markdown and text go to `lynter_core::lint_file`, everything else
//! to `lynter_code`.

use lynter_core::config::Config;
use lynter_core::lint::{lint_file, ScopeContext};
use lynter_core::notetypes::frontmatter::split_document;
use lynter_core::scope::{inline_tags, FileFacts, MatchContext, Resolver};
use lynter_core::Violation;

pub struct Linted {
    pub text: String,
    pub changed: bool,
    pub violations: Vec<Violation>,
    pub chain: Vec<String>,
    /// True for Markdown and text, whose JSON entries keep the version 1
    /// shape (no line, description message, `fixed: true`).
    pub markdown: bool,
}

fn is_prose(rel_path: &str) -> bool {
    let lower = rel_path.to_lowercase();
    lower.ends_with(".md") || lower.ends_with(".txt")
}

pub fn facts_for(text: &str, rel_path: &str, today: &str) -> FileFacts {
    let fm = split_document(text);
    FileFacts {
        path: rel_path.to_owned(),
        frontmatter: fm.has.then(|| fm.yaml_lines.join("\n")),
        context: MatchContext {
            tags: inline_tags(text),
            today: Some(today.to_owned()),
            ..MatchContext::default()
        },
    }
}

/// Lint one file. `None` means no rule applies to a non-Markdown file, so the
/// caller skips it entirely.
pub fn lint_any(
    text: &str,
    rel_path: &str,
    resolver: &Resolver,
    config: &Config,
    today: &str,
) -> Option<Linted> {
    if is_prose(rel_path) {
        let r = lint_file(
            text,
            rel_path,
            config,
            Some(today),
            &ScopeContext::default(),
        );
        return Some(Linted {
            text: r.text,
            changed: r.changed,
            violations: r.violations,
            chain: r.profile_chain,
            markdown: true,
        });
    }
    lynter_code::lint_source(text, rel_path, resolver, Some(today)).map(|r| Linted {
        text: r.text,
        changed: r.changed,
        violations: r.violations,
        chain: r.profile_chain,
        markdown: false,
    })
}
