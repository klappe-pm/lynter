//! Frontmatter rules: dedupe, escape, tag hashes, array style, insert,
//! remove, and moving body tags into the frontmatter.

use super::common::{bool_opt, doc_eol, list_opt, prot, scan, str_opt, Kind};
use super::yaml::{
    block_items, entries, frontmatter, mapping_like, parse_flow, unquote, Entry, FlowItem, Fm,
};
use crate::document::Document;
use crate::edit::Edit;
use crate::rule::{EditScope, Options, Rule};
use serde_json::{json, Value};
use std::collections::HashSet;

fn opts(v: Value) -> Options {
    v.as_object().cloned().unwrap_or_default()
}

fn clean_key(k: &str) -> String {
    k.trim().trim_end_matches(':').trim().to_owned()
}

// ---- dedupe-yaml-array-values ------------------------------------------------

pub struct DedupeYamlArrayValues;

impl Rule for DedupeYamlArrayValues {
    fn id(&self) -> &'static str {
        "dedupe-yaml-array-values"
    }

    fn description(&self) -> &'static str {
        "Remove duplicate values from YAML arrays, case sensitively."
    }

    fn default_options(&self) -> Options {
        opts(json!({
            "dedupe-aliases": true,
            "dedupe-tags": true,
            "dedupe-arrays": true,
            "keys-to-ignore": []
        }))
    }

    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit> {
        let Some(fm) = frontmatter(doc) else {
            return Vec::new();
        };
        let ignore: Vec<String> = list_opt(options.get("keys-to-ignore"))
            .iter()
            .map(|k| clean_key(k))
            .collect();
        let mut out = Vec::new();
        for e in entries(&fm) {
            let on = match e.key.as_str() {
                "aliases" => bool_opt(options.get("dedupe-aliases"), true),
                "tags" => bool_opt(options.get("dedupe-tags"), true),
                _ => bool_opt(options.get("dedupe-arrays"), true),
            };
            if !on || ignore.contains(&e.key) {
                continue;
            }
            let mut seen = HashSet::new();
            if let Some(items) = parse_flow(&e.value) {
                for k in 0..items.len() {
                    if !seen.insert(unquote(&items[k].text)) && k > 0 {
                        out.push(Edit::new(
                            e.value_start + items[k - 1].end,
                            e.value_start + items[k].end,
                            "",
                            "duplicate array value removed",
                        ));
                    }
                }
            } else if let Some(items) = block_items(&fm, &e) {
                if items.iter().any(|i| mapping_like(&i.text)) {
                    continue;
                }
                for it in &items {
                    if !seen.insert(unquote(&it.text)) {
                        let l = &fm.lines[it.line];
                        out.push(Edit::new(
                            l.start,
                            l.next,
                            "",
                            "duplicate array value removed",
                        ));
                    }
                }
            }
        }
        out
    }
}

// ---- escape-yaml-special-characters ------------------------------------------

pub struct EscapeYamlSpecialCharacters;

fn escape_scalar(v: &str, default: char) -> Option<String> {
    let first = v.chars().next()?;
    if matches!(first, '|' | '>' | '[' | '{') {
        return None;
    }
    let wrapped = v.len() >= 2
        && ((v.starts_with('"') && v.ends_with('"')) || (v.starts_with('\'') && v.ends_with('\'')));
    if wrapped {
        return None;
    }
    let (single, double) = (v.contains('\''), v.contains('"'));
    let wrap = match (single, double) {
        (true, true) => return None,
        (true, false) => '"',
        (false, true) => '\'',
        (false, false) if v.contains(": ") => default,
        _ => return None,
    };
    Some(format!("{wrap}{v}{wrap}"))
}

/// Edits (relative to `s`) for the items of a flow array, recursing into nested arrays.
fn escape_flow(s: &str, base: usize, default: char, out: &mut Vec<Edit>) {
    let Some(items) = parse_flow(s) else { return };
    for FlowItem { text, start, end } in items {
        if text.starts_with('[') {
            escape_flow(&s[start..end], base + start, default, out);
        } else if let Some(r) = escape_scalar(&text, default) {
            out.push(Edit::new(
                base + start,
                base + end,
                r,
                "special characters escaped",
            ));
        }
    }
}

impl Rule for EscapeYamlSpecialCharacters {
    fn id(&self) -> &'static str {
        "escape-yaml-special-characters"
    }

    fn description(&self) -> &'static str {
        "Quote frontmatter values that contain a colon and space, single quotes, or double quotes."
    }

    fn default_options(&self) -> Options {
        opts(json!({"default-escape-character": "\"", "escape-single-line-arrays": false}))
    }

    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit> {
        let Some(fm) = frontmatter(doc) else {
            return Vec::new();
        };
        let default = if str_opt(options.get("default-escape-character"), "\"") == "'" {
            '\''
        } else {
            '"'
        };
        let arrays = bool_opt(options.get("escape-single-line-arrays"), false);
        let mut out = Vec::new();
        for e in entries(&fm) {
            if !e.value.is_empty() {
                if e.value.starts_with('[') {
                    if arrays {
                        escape_flow(&e.value, e.value_start, default, &mut out);
                    }
                } else if let Some(r) = escape_scalar(&e.value, default) {
                    out.push(Edit::new(
                        e.value_start,
                        e.value_start + e.value.len(),
                        r,
                        "special characters escaped",
                    ));
                }
                continue;
            }
            for idx in e.first + 1..=e.last {
                let l = &fm.lines[idx];
                let indent = l.text.len() - l.text.trim_start_matches([' ', '\t']).len();
                let Some(rest) = l.text[indent..].strip_prefix('-') else {
                    continue;
                };
                let ws = rest.len() - rest.trim_start_matches([' ', '\t']).len();
                if ws == 0 {
                    continue;
                }
                let item = rest[ws..].trim_end();
                let deeper = fm.lines.get(idx + 1).is_some_and(|n| {
                    idx < e.last
                        && n.text.len() - n.text.trim_start_matches([' ', '\t']).len() > indent + 1
                });
                if item.is_empty() || deeper {
                    continue;
                }
                if let Some(r) = escape_scalar(item, default) {
                    let s = l.start + indent + 1 + ws;
                    out.push(Edit::new(
                        s,
                        s + item.len(),
                        r,
                        "special characters escaped",
                    ));
                }
            }
        }
        out
    }
}

// ---- format-tags-in-yaml -----------------------------------------------------

pub struct FormatTagsInYaml;

fn strip_hashes(text: &str, base: usize, out: &mut Vec<Edit>) {
    let mut prev: Option<char> = None;
    let mut it = text.char_indices().peekable();
    while let Some((i, c)) = it.next() {
        let boundary =
            prev.is_none_or(|p| p.is_whitespace() || matches!(p, ',' | '[' | '"' | '\''));
        if c == '#' && boundary {
            let mut j = i + 1;
            while text[j..].starts_with('#') {
                it.next();
                j += 1;
            }
            out.push(Edit::new(base + i, base + j, "", "tag hashtag removed"));
            prev = Some('#');
            continue;
        }
        prev = Some(c);
    }
}

impl Rule for FormatTagsInYaml {
    fn id(&self) -> &'static str {
        "format-tags-in-yaml"
    }

    fn description(&self) -> &'static str {
        "Remove leading hashtags from tags in frontmatter."
    }

    fn default_options(&self) -> Options {
        Options::new()
    }

    fn check(&self, doc: &Document, _options: &Options) -> Vec<Edit> {
        let Some(fm) = frontmatter(doc) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for e in entries(&fm) {
            if e.key != "tags" && e.key != "tag" {
                continue;
            }
            strip_hashes(&e.value, e.value_start, &mut out);
            if e.value.is_empty() {
                for l in fm.lines.iter().take(e.last + 1).skip(e.first + 1) {
                    let indent = l.text.len() - l.text.trim_start_matches([' ', '\t']).len();
                    if let Some(rest) = l.text[indent..].strip_prefix('-') {
                        let body = rest.trim_end();
                        strip_hashes(body, l.start + indent + 1, &mut out);
                    }
                }
            }
        }
        out
    }
}

// ---- format-yaml-arrays ------------------------------------------------------

pub struct FormatYamlArrays;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Style {
    Single,
    Multi,
    Comma,
    Space,
    ToSingle,
    ToMulti,
}

fn style_of(s: &str) -> Style {
    match s {
        "multi-line" => Style::Multi,
        "single string comma delimited" => Style::Comma,
        "single string space delimited" => Style::Space,
        "single string to single-line" => Style::ToSingle,
        "single string to multi-line" => Style::ToMulti,
        _ => Style::Single,
    }
}

fn render(key: &str, items: &[String], style: Style, eol: &str) -> String {
    let multi = |items: &[String]| {
        let kept: Vec<&String> = items.iter().filter(|i| !i.is_empty()).collect();
        let mut s = format!("{key}:{eol}");
        if kept.is_empty() {
            s.push_str(&format!("  -{eol}"));
        }
        for i in kept {
            s.push_str(&format!("  - {i}{eol}"));
        }
        s
    };
    let single = |items: &[String]| format!("{key}: [{}]{eol}", items.join(", "));
    let scalar = |items: &[String], sep: &str| {
        if items.is_empty() {
            format!("{key}:{eol}")
        } else {
            format!("{key}: {}{eol}", items.join(sep))
        }
    };
    match style {
        Style::Single => single(items),
        Style::Multi => multi(items),
        Style::Comma => scalar(items, ", "),
        Style::Space => scalar(items, " "),
        Style::ToSingle if items.len() <= 1 => scalar(items, ", "),
        Style::ToSingle => single(items),
        Style::ToMulti if items.len() <= 1 => scalar(items, ", "),
        Style::ToMulti => multi(items),
    }
}

fn scalar_items(value: &str, split_ws: bool) -> Vec<String> {
    let quoted = value.len() >= 2
        && ((value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\'')));
    if quoted {
        return vec![value.to_owned()];
    }
    let mut out = Vec::new();
    let mut depth = 0usize;
    let mut quote: Option<char> = None;
    let mut cur = String::new();
    let flush = |cur: &mut String, out: &mut Vec<String>| {
        let t = cur.trim();
        if !t.is_empty() {
            out.push(t.to_owned());
        }
        cur.clear();
    };
    for c in value.chars() {
        if let Some(q) = quote {
            cur.push(c);
            if c == q {
                quote = None;
            }
            continue;
        }
        match c {
            '"' | '\'' => {
                quote = Some(c);
                cur.push(c);
            }
            '[' | '{' => {
                depth += 1;
                cur.push(c);
            }
            ']' | '}' => {
                depth = depth.saturating_sub(1);
                cur.push(c);
            }
            ',' if depth == 0 => flush(&mut cur, &mut out),
            c if split_ws && depth == 0 && c.is_whitespace() => flush(&mut cur, &mut out),
            _ => cur.push(c),
        }
    }
    flush(&mut cur, &mut out);
    out
}

fn entry_items(
    fm: &Fm,
    e: &Entry,
    class_tags: bool,
    class_aliases: bool,
    forced: bool,
) -> Option<Vec<String>> {
    if e.value.is_empty() {
        let items = block_items(fm, e)?;
        if items.iter().any(|i| mapping_like(&i.text)) {
            return None;
        }
        return Some(items.into_iter().map(|i| i.text).collect());
    }
    if e.first != e.last {
        return None;
    }
    if e.value.starts_with('[') {
        return parse_flow(&e.value).map(|v| v.into_iter().map(|i| i.text).collect());
    }
    if matches!(
        e.value.chars().next(),
        Some('{' | '|' | '>' | '&' | '*' | '!')
    ) {
        return None;
    }
    if class_tags {
        Some(scalar_items(&e.value, true))
    } else if class_aliases {
        Some(scalar_items(&e.value, false))
    } else if forced {
        Some(vec![e.value.clone()])
    } else {
        None
    }
}

impl Rule for FormatYamlArrays {
    fn id(&self) -> &'static str {
        "format-yaml-arrays"
    }

    fn description(&self) -> &'static str {
        "Format YAML arrays, tags, and aliases as single-line arrays, multi-line lists, or strings."
    }

    fn default_options(&self) -> Options {
        opts(json!({
            "format-aliases": true,
            "aliases-style": "single-line",
            "format-tags": true,
            "tags-style": "single-line",
            "format-arrays": true,
            "default-array-style": "single-line",
            "force-single-line-keys": [],
            "force-multi-line-keys": []
        }))
    }

    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit> {
        let Some(fm) = frontmatter(doc) else {
            return Vec::new();
        };
        let force_single = list_opt(options.get("force-single-line-keys"));
        let force_multi = list_opt(options.get("force-multi-line-keys"));
        let text = doc.text();
        let mut out = Vec::new();
        for e in entries(&fm) {
            let (is_tags, is_aliases) = (e.key == "tags", e.key == "aliases");
            let fs = force_single.contains(&e.key);
            let fmu = force_multi.contains(&e.key);
            let on = fs
                || fmu
                || if is_tags {
                    bool_opt(options.get("format-tags"), true)
                } else if is_aliases {
                    bool_opt(options.get("format-aliases"), true)
                } else {
                    bool_opt(options.get("format-arrays"), true)
                };
            if !on {
                continue;
            }
            let style = if fs {
                Style::Single
            } else if fmu {
                Style::Multi
            } else if is_tags {
                style_of(str_opt(options.get("tags-style"), "single-line"))
            } else if is_aliases {
                style_of(str_opt(options.get("aliases-style"), "single-line"))
            } else {
                style_of(str_opt(options.get("default-array-style"), "single-line"))
            };
            let Some(items) = entry_items(&fm, &e, is_tags, is_aliases, fs || fmu) else {
                continue;
            };
            if items.is_empty() {
                continue;
            }
            let eol = fm.lines[e.first].eol;
            let new = render(&e.key_text, &items, style, eol);
            let (s, end) = (fm.lines[e.first].start, fm.lines[e.last].next);
            if text[s..end] != new {
                out.push(Edit::new(s, end, new, "array style normalized"));
            }
        }
        out
    }
}

// ---- insert-yaml-attributes --------------------------------------------------

pub struct InsertYamlAttributes;

impl Rule for InsertYamlAttributes {
    fn edit_scope(&self) -> EditScope {
        EditScope::Frontmatter
    }

    fn id(&self) -> &'static str {
        "insert-yaml-attributes"
    }

    fn description(&self) -> &'static str {
        "Add missing frontmatter attributes from a template."
    }

    fn default_options(&self) -> Options {
        opts(json!({"text-to-insert": "aliases: \ntags: "}))
    }

    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit> {
        let template = str_opt(options.get("text-to-insert"), "aliases: \ntags: ");
        let attrs: Vec<(&str, &str)> = template
            .split('\n')
            .map(|l| l.trim_end())
            .filter(|l| !l.trim().is_empty())
            .map(|l| (l.split(':').next().unwrap_or(l).trim(), l))
            .collect();
        if attrs.is_empty() {
            return Vec::new();
        }
        let text = doc.text();
        let message = "frontmatter attributes inserted";
        match frontmatter(doc) {
            Some(fm) => {
                let existing: HashSet<String> = entries(&fm).into_iter().map(|e| e.key).collect();
                let eol = fm.eol;
                let mut seen: HashSet<&str> = HashSet::new();
                let add: String = attrs
                    .iter()
                    .filter(|(k, _)| !existing.contains(*k) && seen.insert(*k))
                    .map(|(_, l)| format!("{l}{eol}"))
                    .collect();
                if add.is_empty() {
                    return Vec::new();
                }
                let at = fm.lines.first().map_or(fm.close_start, |l| l.start);
                vec![Edit::new(at, at, add, message)]
            }
            None => {
                let eol = doc_eol(text);
                let mut s = format!("---{eol}");
                let mut seen: HashSet<&str> = HashSet::new();
                for (k, l) in &attrs {
                    if seen.insert(*k) {
                        s.push_str(&format!("{l}{eol}"));
                    }
                }
                s.push_str(&format!("---{eol}"));
                vec![Edit::new(0, 0, s, message)]
            }
        }
    }
}

// ---- remove-yaml-keys --------------------------------------------------------

pub struct RemoveYamlKeys;

impl Rule for RemoveYamlKeys {
    fn id(&self) -> &'static str {
        "remove-yaml-keys"
    }

    fn description(&self) -> &'static str {
        "Delete listed keys, with their values, from the frontmatter."
    }

    fn default_options(&self) -> Options {
        opts(json!({"keys": []}))
    }

    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit> {
        let keys: Vec<String> = list_opt(options.get("keys"))
            .iter()
            .map(|k| clean_key(k))
            .collect();
        let Some(fm) = frontmatter(doc).filter(|_| !keys.is_empty()) else {
            return Vec::new();
        };
        entries(&fm)
            .into_iter()
            .filter(|e| keys.contains(&e.key))
            .map(|e| {
                Edit::new(
                    fm.lines[e.first].start,
                    fm.lines[e.last].next,
                    "",
                    "frontmatter key removed",
                )
            })
            .collect()
    }
}

// ---- move-tags-to-yaml -------------------------------------------------------

pub struct MoveTagsToYaml;

struct BodyTag {
    start: usize,
    end: usize,
    name: String,
}

fn body_tags(doc: &Document) -> Vec<BodyTag> {
    let sc = scan(doc, 2);
    let mut out = Vec::new();
    for l in &sc.lines {
        if l.kind != Kind::Normal {
            continue;
        }
        for (i, c) in l.text.char_indices() {
            if c != '#'
                || !l.text[..i]
                    .chars()
                    .next_back()
                    .is_none_or(char::is_whitespace)
            {
                continue;
            }
            let body = &l.text[i + 1..];
            let n: usize = body
                .chars()
                .take_while(|&c| c.is_alphanumeric() || matches!(c, '_' | '-' | '/'))
                .map(char::len_utf8)
                .sum();
            let name = &body[..n];
            if name.is_empty() || name.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }
            let (s, e) = (l.start + i, l.start + i + 1 + n);
            if prot(doc, s, e) {
                continue;
            }
            out.push(BodyTag {
                start: s,
                end: e,
                name: name.to_owned(),
            });
        }
    }
    out
}

impl Rule for MoveTagsToYaml {
    fn edit_scope(&self) -> EditScope {
        EditScope::Frontmatter
    }

    fn id(&self) -> &'static str {
        "move-tags-to-yaml"
    }

    fn description(&self) -> &'static str {
        "Move inline body tags into the frontmatter tags list."
    }

    fn default_options(&self) -> Options {
        opts(json!({"body-tag-operation": "nothing", "tags-to-ignore": []}))
    }

    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit> {
        let ignore: Vec<String> = list_opt(options.get("tags-to-ignore"))
            .iter()
            .map(|t| t.trim_start_matches('#').to_owned())
            .collect();
        let op = str_opt(options.get("body-tag-operation"), "nothing");
        let moved: Vec<BodyTag> = body_tags(doc)
            .into_iter()
            .filter(|t| !ignore.contains(&t.name))
            .collect();
        if moved.is_empty() {
            return Vec::new();
        }
        let text = doc.text();
        let mut tags: Vec<&str> = Vec::new();
        for t in &moved {
            if !tags.contains(&t.name.as_str()) {
                tags.push(&t.name);
            }
        }
        let message = "tags moved to frontmatter";
        let mut out = Vec::new();
        match frontmatter(doc) {
            None => {
                let eol = doc_eol(text);
                out.push(Edit::new(
                    0,
                    0,
                    format!("---{eol}tags: [{}]{eol}---{eol}", tags.join(", ")),
                    message,
                ));
            }
            Some(fm) => {
                let existing = entries(&fm).into_iter().find(|e| e.key == "tags");
                match existing {
                    None => out.push(Edit::new(
                        fm.close_start,
                        fm.close_start,
                        format!("tags: [{}]{}", tags.join(", "), fm.eol),
                        message,
                    )),
                    Some(e) => {
                        let have = |items: &[String], t: &str| {
                            items
                                .iter()
                                .any(|i| unquote(i).trim_start_matches('#') == t)
                        };
                        if let Some(items) = parse_flow(&e.value) {
                            let texts: Vec<String> = items.into_iter().map(|i| i.text).collect();
                            let new: Vec<&&str> =
                                tags.iter().filter(|t| !have(&texts, t)).collect();
                            if !new.is_empty() {
                                let at = e.value_start + e.value.len() - 1;
                                let joined = new.iter().map(|t| **t).collect::<Vec<_>>().join(", ");
                                let ins = if texts.is_empty() {
                                    joined
                                } else {
                                    format!(", {joined}")
                                };
                                out.push(Edit::new(at, at, ins, message));
                            }
                        } else if let Some(items) = block_items(&fm, &e) {
                            let texts: Vec<String> = items.iter().map(|i| i.text.clone()).collect();
                            let new: Vec<&&str> =
                                tags.iter().filter(|t| !have(&texts, t)).collect();
                            if let (false, Some(last)) = (new.is_empty(), items.last()) {
                                let l = &fm.lines[last.line];
                                let indent = &l.text[..last.indent];
                                let add: String = new
                                    .iter()
                                    .map(|t| format!("{indent}- {t}{}", l.eol))
                                    .collect();
                                out.push(Edit::new(l.next, l.next, add, message));
                            }
                        } else if e.value.is_empty() && e.first == e.last {
                            let l = &fm.lines[e.first];
                            let at = l.start + l.text.trim_end().len();
                            out.push(Edit::new(
                                at,
                                at,
                                format!(" [{}]", tags.join(", ")),
                                message,
                            ));
                        } else if !e.value.is_empty()
                            && e.first == e.last
                            && !e.value.starts_with(['[', '{', '|', '>'])
                        {
                            let texts = scalar_items(&e.value, true);
                            let new: Vec<&&str> =
                                tags.iter().filter(|t| !have(&texts, t)).collect();
                            if !new.is_empty() {
                                let all: Vec<String> = texts
                                    .iter()
                                    .cloned()
                                    .chain(new.iter().map(|t| (**t).to_owned()))
                                    .collect();
                                out.push(Edit::new(
                                    e.value_start,
                                    e.value_start + e.value.len(),
                                    format!("[{}]", all.join(", ")),
                                    message,
                                ));
                            }
                        }
                    }
                }
            }
        }
        match op {
            "remove-hashtag" => {
                for t in &moved {
                    out.push(Edit::new(t.start, t.start + 1, "", "tag hashtag removed"));
                }
            }
            "remove-whole-tag" => {
                for t in &moved {
                    let s = match text[..t.start].chars().next_back() {
                        Some(c @ (' ' | '\t')) => t.start - c.len_utf8(),
                        _ => t.start,
                    };
                    out.push(Edit::new(s, t.end, "", "tag removed"));
                }
            }
            _ => {}
        }
        out
    }
}
