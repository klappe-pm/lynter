//! Declarative code checks over fenced code blocks. Checks are data: a regex
//! run line by line inside fences, never code. Messages never carry matched
//! text.

use super::util::js_trim;
use crate::document::Document;
use crate::edit::Edit;
use crate::rule::{Options, Rule};
use crate::scope::{compile_glob, CompiledGlob};
use fancy_regex::{Captures, Regex};
use serde_json::{json, Map, Value};
use std::sync::LazyLock;

pub const MAX_LINE_LENGTH: usize = 2000;

static FENCE_OPEN: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"^ {0,3}(`{3,}|~{3,})").expect("regex"));

fn builtin_checks() -> Map<String, Value> {
    let v = json!({
        "no-token-shaped-strings": {
            "enabled": false,
            "description": "Flag token-shaped literals (cloud keys, bearer tokens, long hex) inside code blocks.",
            "pattern": "AKIA[0-9A-Z]{16}|gh[pousr]_[A-Za-z0-9]{36,}|xox[baprs]-[A-Za-z0-9-]{10,}|eyJ[A-Za-z0-9_-]{20,}\\.[A-Za-z0-9_-]{10,}|\\b[A-Fa-f0-9]{40,}\\b",
            "message": "token-shaped literal in a code block; move it to a secret reference"
        },
        "no-gnu-only-flags-in-sh": {
            "enabled": false,
            "description": "Flag GNU-only shell idioms that break on macOS BSD userland.",
            "languages": ["sh", "bash", "zsh", "shell"],
            "pattern": "sed\\s+-i(?!\\s+'')(?=\\s|$)|\\breadlink\\s+-f\\b|\\bflock\\b|(?<![.\\w-])timeout\\s+\\d",
            "message": "GNU-only shell idiom; use sed -i '' and python3 equivalents for portability"
        },
        "no-trailing-whitespace-in-code": {
            "enabled": false,
            "description": "Strip trailing whitespace on lines inside code blocks.",
            "pattern": "[ \\t]+$",
            "message": "trailing whitespace in a code block",
            "fix": {"replacement": ""}
        }
    });
    v.as_object().cloned().unwrap_or_default()
}

struct Compiled {
    regex: Regex,
    languages: Option<Vec<String>>,
    paths: Option<Vec<CompiledGlob>>,
    replacement: Option<String>,
}

fn string_list(v: Option<&Value>) -> Option<Vec<String>> {
    match v {
        Some(Value::Array(a)) => Some(
            a.iter()
                .filter_map(|x| x.as_str().map(str::to_owned))
                .collect(),
        ),
        _ => None,
    }
}

/// Merge built-ins under the configured checks (configured wins, field by
/// field) and compile the enabled ones of the requested kind.
fn compile(configured: Option<&Value>, want_fixing: bool) -> Vec<Compiled> {
    let mut merged = builtin_checks();
    if let Some(Value::Object(cfg)) = configured {
        for (id, check) in cfg {
            let Value::Object(fields) = check else {
                continue;
            };
            let entry = merged
                .entry(id.clone())
                .or_insert_with(|| Value::Object(Map::new()));
            if let Value::Object(t) = entry {
                for (k, v) in fields {
                    t.insert(k.clone(), v.clone());
                }
            }
        }
    }
    let mut out = Vec::new();
    for check in merged.values() {
        let Some(pattern) = check.get("pattern").and_then(Value::as_str) else {
            continue;
        };
        if check.get("enabled") != Some(&Value::Bool(true)) {
            continue;
        }
        let replacement = check
            .get("fix")
            .and_then(|f| f.get("replacement"))
            .and_then(Value::as_str)
            .map(str::to_owned);
        if want_fixing != replacement.is_some() {
            continue;
        }
        let flags: String = check
            .get("flags")
            .and_then(Value::as_str)
            .unwrap_or("")
            .chars()
            .filter(|c| matches!(c, 'i' | 'm' | 's'))
            .collect();
        let source = if flags.is_empty() {
            pattern.to_owned()
        } else {
            format!("(?{flags}){pattern}")
        };
        let Ok(regex) = Regex::new(&source) else {
            continue;
        };
        out.push(Compiled {
            regex,
            languages: string_list(check.get("languages"))
                .map(|l| l.iter().map(|s| s.to_lowercase()).collect()),
            paths: string_list(check.get("paths"))
                .map(|p| p.iter().map(|g| compile_glob(g)).collect()),
            replacement,
        });
    }
    out
}

struct Block {
    language: String,
    start_line: usize,
    lines: usize,
}

fn fenced_blocks(lines: &[&str]) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut open: Option<(char, String, usize, usize)> = None;
    for (i, line) in lines.iter().enumerate() {
        let m = FENCE_OPEN.captures(line);
        match &mut open {
            None => {
                if let Some(m) = m {
                    let whole = m.get(0).map_or(0, |w| w.end());
                    let language = js_trim(&line[whole..])
                        .split(super::util::is_js_ws)
                        .next()
                        .unwrap_or("")
                        .to_lowercase();
                    let marker = m[1].chars().next().unwrap_or('`');
                    open = Some((marker, language, i + 1, 0));
                }
            }
            Some((marker, language, start, count)) => {
                let closes = m.as_ref().is_some_and(|m| {
                    m[1].starts_with(*marker)
                        && js_trim(&line[m.get(0).map_or(0, |w| w.end())..]).is_empty()
                });
                if closes {
                    blocks.push(Block {
                        language: language.clone(),
                        start_line: *start,
                        lines: *count,
                    });
                    open = None;
                } else {
                    *count += 1;
                }
            }
        }
    }
    if let Some((_, language, start, count)) = open {
        blocks.push(Block {
            language,
            start_line: start,
            lines: count,
        });
    }
    blocks
}

/// Line range (0-based, end exclusive) of the block's content that the zone
/// scanner also treats as fenced code. The line scanner here pairs fences on its
/// own, so it can disagree with the zones (for example after a `---` pair that
/// the zone scanner reads as frontmatter, or inside math, a table, or indented
/// code). Lynter never edits outside a code-fence zone, so a block whose opener
/// is not the start of a CodeFence zone yields `None`, and content is clamped to
/// that zone.
fn content_range(doc: &Document, b: &Block) -> Option<std::ops::Range<usize>> {
    let text = doc.text();
    let lines = doc.lines();
    let open = lines.get(b.start_line.checked_sub(1)?)?;
    let zone = doc
        .zones()
        .iter()
        .find(|z| z.kind == crate::zones::ZoneKind::CodeFence && z.start == open.start)?;
    let (marker, n) = crate::zones::fence_open(open.text(text))?;
    let last = lines.iter().position(|l| l.end == zone.end)?;
    let closed = last >= b.start_line
        && crate::zones::is_fence_close(lines[last].text(text), marker, n, true);
    let limit = if closed { last } else { last + 1 };
    Some(b.start_line..(b.start_line + b.lines).min(limit))
}

fn applies(c: &Compiled, language: &str, path: Option<&str>) -> bool {
    if let Some(l) = &c.languages {
        if !l.iter().any(|x| x == language) {
            return false;
        }
    }
    if let Some(globs) = &c.paths {
        let Some(p) = path else { return false };
        if !globs.iter().any(|g| g.is_match(p)) {
            return false;
        }
    }
    true
}

fn utf16_len(s: &str) -> usize {
    s.encode_utf16().count()
}

/// Expand a JavaScript replacement template.
fn expand(tpl: &str, caps: &Captures<'_>, hay: &str) -> String {
    let whole = caps.get(0).expect("group 0");
    let b: Vec<char> = tpl.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] != '$' || i + 1 >= b.len() {
            out.push(b[i]);
            i += 1;
            continue;
        }
        match b[i + 1] {
            '$' => {
                out.push('$');
                i += 2;
            }
            '&' => {
                out.push_str(whole.as_str());
                i += 2;
            }
            '`' => {
                out.push_str(&hay[..whole.start()]);
                i += 2;
            }
            '\'' => {
                out.push_str(&hay[whole.end()..]);
                i += 2;
            }
            d if d.is_ascii_digit() => {
                let one = d.to_digit(10).unwrap_or(0) as usize;
                let two = b
                    .get(i + 2)
                    .filter(|c| c.is_ascii_digit())
                    .map(|c| one * 10 + c.to_digit(10).unwrap_or(0) as usize);
                let count = caps.len();
                if let Some(n) = two.filter(|&n| n >= 1 && n < count) {
                    out.push_str(caps.get(n).map_or("", |m| m.as_str()));
                    i += 3;
                } else if one >= 1 && one < count {
                    out.push_str(caps.get(one).map_or("", |m| m.as_str()));
                    i += 2;
                } else {
                    out.push('$');
                    i += 1;
                }
            }
            _ => {
                out.push('$');
                i += 1;
            }
        }
    }
    out
}

fn replace_all(re: &Regex, s: &str, tpl: &str) -> String {
    let mut out = String::new();
    let mut last = 0;
    for caps in re.captures_iter(s).flatten() {
        let m = caps.get(0).expect("group 0");
        out.push_str(&s[last..m.start()]);
        out.push_str(&expand(tpl, &caps, s));
        last = m.end();
    }
    out.push_str(&s[last..]);
    out
}

fn path_of(options: &Options) -> Option<&str> {
    options
        .get("file-path")
        .and_then(Value::as_str)
        .filter(|p| !p.is_empty())
}

fn has_violation(doc: &Document, options: &Options) -> bool {
    let text = doc.text();
    let checks = compile(options.get("checks"), false);
    if checks.is_empty() {
        return false;
    }
    let lines: Vec<&str> = text.split('\n').collect();
    let path = path_of(options);
    for block in fenced_blocks(&lines) {
        let Some(range) = content_range(doc, &block) else {
            continue;
        };
        for c in &checks {
            if !applies(c, &block.language, path) {
                continue;
            }
            for idx in range.clone() {
                let line = lines[idx];
                if utf16_len(line) > MAX_LINE_LENGTH {
                    continue;
                }
                if c.regex.is_match(line).unwrap_or(false) {
                    return true;
                }
            }
        }
    }
    false
}

fn apply_fixing(doc: &Document, options: &Options) -> String {
    let text = doc.text();
    let checks = compile(options.get("checks"), true);
    if checks.is_empty() {
        return text.to_owned();
    }
    let mut lines: Vec<String> = text.split('\n').map(str::to_owned).collect();
    let path = path_of(options);
    let borrowed: Vec<&str> = text.split('\n').collect();
    for block in fenced_blocks(&borrowed) {
        let Some(range) = content_range(doc, &block) else {
            continue;
        };
        for c in &checks {
            if !applies(c, &block.language, path) {
                continue;
            }
            let tpl = c.replacement.as_deref().unwrap_or("");
            for idx in range.clone() {
                if utf16_len(&lines[idx]) > MAX_LINE_LENGTH {
                    continue;
                }
                lines[idx] = replace_all(&c.regex, &lines[idx], tpl);
            }
        }
    }
    lines.join("\n")
}

/// Reporting rule: flags matches, never mutates. The edit appends one space,
/// which only signals "would change" to the runner.
pub struct CodeChecks;
/// Fixing rule: applies checks that define `fix.replacement`.
pub struct CodeChecksFix;

impl Rule for CodeChecks {
    fn id(&self) -> &'static str {
        "code-checks"
    }
    fn description(&self) -> &'static str {
        "Declarative code checks over fenced code blocks (patterns from lynter.yaml code-checks; never executes code)."
    }
    fn default_options(&self) -> Options {
        Options::new()
    }
    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit> {
        if has_violation(doc, options) {
            let end = doc.text().len();
            vec![Edit::new(end, end, "\0", "code check matched")]
        } else {
            Vec::new()
        }
    }
}

impl Rule for CodeChecksFix {
    fn id(&self) -> &'static str {
        "code-checks-fix"
    }
    fn description(&self) -> &'static str {
        "Apply fixing code checks (those with fix.replacement) inside fenced code blocks."
    }
    fn default_options(&self) -> Options {
        Options::new()
    }
    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit> {
        super::util::diff_edit(doc.text(), &apply_fixing(doc, options), "code check fixed")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixing_never_edits_math_after_a_zone_paired_fence() {
        let text = "---\n~~~\n---\n~~~\n~~~\n$$\necho hi   ";
        let doc = Document::parse(text);
        let options = json!({"checks": {"mine": {"enabled": true, "pattern": "[ \\t]+$", "fix": {"replacement": ""}}}})
            .as_object()
            .cloned()
            .unwrap_or_default();
        assert_eq!(apply_fixing(&doc, &options), text);
    }

    #[test]
    fn fixing_still_edits_a_real_fence() {
        let text = "```\necho hi   \n```\n";
        let doc = Document::parse(text);
        let options = json!({"checks": {"mine": {"enabled": true, "pattern": "[ \\t]+$", "fix": {"replacement": ""}}}})
            .as_object()
            .cloned()
            .unwrap_or_default();
        assert_eq!(apply_fixing(&doc, &options), "```\necho hi\n```\n");
    }
}
