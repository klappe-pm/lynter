//! Filename rules: kebab-case stems and an H1 that matches the stem.

use crate::document::Document;
use crate::edit::Edit;
use crate::rule::{Options, Rule};
use crate::rules::content::util::{diff_edit, is_js_ws};
use regex::Regex;
use std::sync::LazyLock;
use unicode_normalization::UnicodeNormalization;

const RESERVED_STEMS: [&str; 9] = [
    "README",
    "CLAUDE",
    "SKILL",
    "PASSOFF",
    "GIT-STRATEGY",
    "PR-STANDARDS",
    "MEMORY",
    "AGENTS",
    "CODEX",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenameStrategy {
    Replace,
    Prepend,
    Append,
}

impl RenameStrategy {
    pub fn parse(s: Option<&str>) -> RenameStrategy {
        match s {
            Some("prepend") => RenameStrategy::Prepend,
            Some("append") => RenameStrategy::Append,
            _ => RenameStrategy::Replace,
        }
    }
}

pub fn is_reserved_stem(stem: &str) -> bool {
    RESERVED_STEMS.contains(&stem)
}

fn re(p: &str) -> Regex {
    Regex::new(p).expect("regex")
}

static ACRONYM: LazyLock<Regex> = LazyLock::new(|| re(r"(\p{Lu}+)(\p{Lu}\p{Ll})"));
static CAMEL: LazyLock<Regex> = LazyLock::new(|| re(r"([\p{Ll}\p{Nd}])(\p{Lu})"));
static MARKS: LazyLock<Regex> = LazyLock::new(|| re(r"\p{M}+"));
static SPACES: LazyLock<Regex> =
    LazyLock::new(|| re(&crate::rules::content::util::js_pattern(r"[\s_]+")));
static STRIP: LazyLock<Regex> = LazyLock::new(|| re(r"[^\p{L}\p{N}-]+"));
static HYPHENS: LazyLock<Regex> = LazyLock::new(|| re(r"-{2,}"));

/// Unicode-aware kebab-case slug of a filename stem.
pub fn kebab_case_name(stem: &str) -> String {
    if is_reserved_stem(stem) {
        return stem.to_owned();
    }
    let s = ACRONYM.replace_all(stem, "${1}-${2}");
    let s = CAMEL.replace_all(&s, "${1}-${2}");
    let s: String = s.nfkd().collect();
    let s = MARKS.replace_all(&s, "").to_lowercase();
    let s = SPACES.replace_all(&s, "-");
    let s = STRIP.replace_all(&s, "");
    let s = HYPHENS.replace_all(&s, "-");
    s.trim_matches('-').to_owned()
}

struct Token {
    token: String,
    remainder: String,
}

fn extract_structural_token(stem: &str) -> Option<Token> {
    static PATTERNS: LazyLock<Vec<(Regex, bool)>> = LazyLock::new(|| {
        let sep = crate::rules::content::util::js_pattern(r"[\s_-]");
        vec![
            (
                re(&format!(r"^([0-9]{{4}}-[0-9]{{2}}-[0-9]{{2}})(?:{sep}|$)")),
                true,
            ),
            (
                re(&format!(r"(?:^|{sep})([0-9]{{4}}-[0-9]{{2}}-[0-9]{{2}})$")),
                false,
            ),
            (re(&format!(r"^([0-9]+)(?:{sep}|$)")), true),
            (re(&format!(r"(?:^|{sep})([0-9]+)$")), false),
        ]
    });
    for (pattern, leading) in PATTERNS.iter() {
        let Some(caps) = pattern.captures(stem) else {
            continue;
        };
        let g = caps.get(1)?;
        let (start, end) = (g.start(), g.end());
        let before = &stem[..start];
        let after = &stem[end..];
        let strip = |s: &str, from_start: bool| -> String {
            let mut chars = s.chars();
            if from_start {
                match chars.next() {
                    Some(c) if is_js_ws(c) || c == '_' || c == '-' => chars.as_str().to_owned(),
                    _ => s.to_owned(),
                }
            } else {
                match chars.next_back() {
                    Some(c) if is_js_ws(c) || c == '_' || c == '-' => chars.as_str().to_owned(),
                    _ => s.to_owned(),
                }
            }
        };
        let remainder = if *leading {
            strip(after, true)
        } else {
            strip(before, false)
        };
        return Some(Token {
            token: g.as_str().to_owned(),
            remainder,
        });
    }
    None
}

pub fn derive_compliant_stem(stem: &str, strategy: RenameStrategy) -> String {
    let slug = kebab_case_name(stem);
    if strategy == RenameStrategy::Replace || is_reserved_stem(stem) {
        return slug;
    }
    let Some(t) = extract_structural_token(stem) else {
        return slug;
    };
    let remainder = kebab_case_name(&t.remainder);
    let joined = if remainder.is_empty() {
        t.token
    } else if strategy == RenameStrategy::Prepend {
        format!("{}-{remainder}", t.token)
    } else {
        format!("{remainder}-{}", t.token)
    };
    kebab_case_name(&joined)
}

/// Append `-2`, `-3`, ... until the proposed stem is free.
pub fn resolve_collision(proposed: &str, taken: &std::collections::HashSet<String>) -> String {
    if !taken.contains(proposed) {
        return proposed.to_owned();
    }
    let mut n = 2;
    while taken.contains(&format!("{proposed}-{n}")) {
        n += 1;
    }
    format!("{proposed}-{n}")
}

/// Basename of a path minus a trailing `.md` extension.
pub fn path_stem(path: &str) -> String {
    let base = path.rsplit(['/', '\\']).next().unwrap_or(path);
    if base.len() >= 3
        && base.is_char_boundary(base.len() - 3)
        && base[base.len() - 3..].eq_ignore_ascii_case(".md")
    {
        base[..base.len() - 3].to_owned()
    } else {
        base.to_owned()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenameProposal {
    pub stem: String,
    pub proposed: String,
}

pub fn propose_rename(path: Option<&str>, strategy: RenameStrategy) -> Option<RenameProposal> {
    let path = path.filter(|p| !p.is_empty())?;
    let stem = path_stem(path);
    let proposed = derive_compliant_stem(&stem, strategy);
    (!proposed.is_empty() && proposed != stem).then_some(RenameProposal { stem, proposed })
}

fn path_option(options: &Options) -> Option<&str> {
    options
        .get("file-path")
        .and_then(serde_json::Value::as_str)
        .filter(|p| !p.is_empty())
}

/// Report-only: flags a stem that is not kebab-case.
pub struct KebabCaseFilename;

impl Rule for KebabCaseFilename {
    fn id(&self) -> &'static str {
        "kebab-case-filename"
    }
    fn description(&self) -> &'static str {
        "Flag filenames that are not kebab-case and propose the compliant name; the rename runs only where rename mode allows it."
    }
    fn default_options(&self) -> Options {
        Options::new()
    }
    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit> {
        let strategy = RenameStrategy::parse(options.get("strategy").and_then(|v| v.as_str()));
        let Some(p) = propose_rename(path_option(options), strategy) else {
            return Vec::new();
        };
        let text = doc.text();
        let sep = if text.ends_with('\n') || text.is_empty() {
            ""
        } else {
            "\n"
        };
        let end = text.len();
        vec![Edit::new(
            end,
            end,
            format!(
                "{sep}<!-- lynter kebab-case-filename: rename \"{}\" to \"{}\" -->\n",
                p.stem, p.proposed
            ),
            "filename is not kebab-case",
        )]
    }
}

static FENCE_OPEN: LazyLock<Regex> = LazyLock::new(|| re(r"^ {0,3}(`{3,}|~{3,})"));
static H1_LINE: LazyLock<Regex> = LazyLock::new(|| re(r"^#($|[ \t])"));

fn frontmatter_end(lines: &[&str]) -> usize {
    let strip = |l: &str| l.strip_suffix('\r').unwrap_or(l).to_owned();
    if strip(lines[0]) != "---" {
        return 0;
    }
    (1..lines.len())
        .find(|&i| strip(lines[i]) == "---")
        .map_or(0, |i| i + 1)
}

/// Keep the first H1 equal to the filename stem.
pub struct H1MatchesStem;

impl Rule for H1MatchesStem {
    fn id(&self) -> &'static str {
        "h1-matches-stem"
    }
    fn description(&self) -> &'static str {
        "Keep the first H1 equal to the filename stem, rewriting it in place or inserting it after the frontmatter."
    }
    fn default_options(&self) -> Options {
        Options::new()
    }
    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit> {
        let Some(path) = path_option(options) else {
            return Vec::new();
        };
        let text = doc.text();
        let heading = format!("# {}", path_stem(path));
        let mut lines: Vec<&str> = text.split('\n').collect();
        let fm_end = frontmatter_end(&lines);
        let mut open: Option<(char, usize)> = None;
        for i in fm_end..lines.len() {
            let line = lines[i].strip_suffix('\r').unwrap_or(lines[i]);
            let was_in_fence = open.is_some();
            let mut is_delim = false;
            if let Some(m) = FENCE_OPEN.captures(line) {
                let run = &m[1];
                let c = run.chars().next().unwrap_or('`');
                match open {
                    Some((oc, on)) => {
                        if oc == c && run.len() >= on {
                            open = None;
                            is_delim = true;
                        }
                    }
                    None => {
                        open = Some((c, run.len()));
                        is_delim = true;
                    }
                }
            }
            if was_in_fence || is_delim {
                continue;
            }
            if H1_LINE.is_match(line) {
                if line == heading {
                    return Vec::new();
                }
                lines[i] = &heading;
                return diff_edit(text, &lines.join("\n"), "h1 rewritten to match filename");
            }
        }
        let mut out: Vec<&str> = Vec::new();
        out.extend(&lines[..fm_end]);
        if fm_end > 0 {
            out.push("");
        }
        out.push(&heading);
        let body = &lines[fm_end..];
        if !body.is_empty() {
            if !body[0].strip_suffix('\r').unwrap_or(body[0]).is_empty() {
                out.push("");
            }
            out.extend(body);
        }
        diff_edit(text, &out.join("\n"), "h1 inserted to match filename")
    }
}
