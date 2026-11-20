//! Frontmatter rules: key sort, value alphabetizing, and the note-type
//! insert, sort, and date rules.

use super::frontmatter::{
    entries_to_lines, format_entry_lines, join_document, parse_frontmatter_data, split_document,
    split_entries, YamlEntry,
};
use super::key_rank::{compare_rank, insert_ranked, rank_key};
use super::locale::locale_compare;
use super::schema::{key_order_from, schema_from};
use crate::document::Document;
use crate::edit::Edit;
use crate::rule::{Options, Rule};
use crate::rules::content::util::{diff_edit, js_pattern};
use regex::Regex;
use serde_json::{json, Value};
use std::sync::LazyLock;

fn opts(v: Value) -> Options {
    v.as_object().cloned().unwrap_or_default()
}

fn string_list(v: Option<&Value>) -> Vec<String> {
    match v {
        Some(Value::Array(a)) => a
            .iter()
            .filter_map(|x| x.as_str().map(str::to_owned))
            .collect(),
        _ => Vec::new(),
    }
}

fn sort_entries(entries: &[YamlEntry], order: &[String]) -> Vec<YamlEntry> {
    let mut keyed: Vec<(usize, &YamlEntry)> = entries
        .iter()
        .enumerate()
        .filter(|(_, e)| e.key.is_some())
        .collect();
    keyed.sort_by(|(ia, a), (ib, b)| {
        compare_rank(
            &rank_key(a.key.as_deref().unwrap_or(""), order),
            &rank_key(b.key.as_deref().unwrap_or(""), order),
        )
        .then(ia.cmp(ib))
    });
    let mut out: Vec<YamlEntry> = entries
        .iter()
        .filter(|e| e.key.is_none())
        .cloned()
        .collect();
    out.extend(keyed.into_iter().map(|(_, e)| e.clone()));
    out
}

fn text_edit(doc: &Document, new: String, message: &str) -> Vec<Edit> {
    diff_edit(doc.text(), &new, message)
}

// ---- yaml-key-sort ----------------------------------------------------------

pub struct YamlKeySort;

impl Rule for YamlKeySort {
    fn id(&self) -> &'static str {
        "yaml-key-sort"
    }
    fn description(&self) -> &'static str {
        "Sort frontmatter keys into the configured order (priority-keys first, remaining alphabetical, aliases and tags last), inserting configured default keys when absent."
    }
    fn default_options(&self) -> Options {
        opts(json!({"priority-keys": [], "defaults": {}}))
    }
    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit> {
        let priority = string_list(options.get("priority-keys"));
        let defaults = options
            .get("defaults")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default();
        let fm = split_document(doc.text());
        if !fm.has {
            return Vec::new();
        }
        let Some(data) = parse_frontmatter_data(&fm.yaml_lines) else {
            return Vec::new();
        };
        let entries = split_entries(&fm.yaml_lines);
        let mut sorted = sort_entries(&entries, &priority);
        let loose: Vec<YamlEntry> = sorted.iter().filter(|e| e.key.is_none()).cloned().collect();
        sorted.retain(|e| e.key.is_some());
        for (key, value) in &defaults {
            if data.contains_key(key) {
                continue;
            }
            sorted = insert_ranked(
                &sorted,
                YamlEntry {
                    key: Some(key.clone()),
                    lines: format_entry_lines(key, value),
                },
                &priority,
            );
        }
        let mut all = loose;
        all.extend(sorted);
        let out = join_document(&super::frontmatter::FrontmatterDoc {
            has: true,
            yaml_lines: entries_to_lines(&all),
            body_lines: fm.body_lines,
        });
        text_edit(doc, out, "frontmatter keys sorted")
    }
}

// ---- alphabetize-property-values ----------------------------------------------

static BLOCK_ITEM: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&js_pattern(r"^(\s*)- (DOT*)$")).expect("regex"));
static FLOW_ARRAY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(&js_pattern(r"^([^:#]+:\s*)\[(DOT*)\]\s*$")).expect("regex"));

fn compare_values(a: &str, b: &str) -> std::cmp::Ordering {
    locale_compare(&a.to_lowercase(), &b.to_lowercase())
}

fn split_flow_items(body: &str) -> Option<Vec<String>> {
    if body.contains(['[', ']', '"', '\'']) {
        return None;
    }
    if crate::rules::content::util::js_trim(body).is_empty() {
        return Some(Vec::new());
    }
    Some(
        body.split(',')
            .map(|s| crate::rules::content::util::js_trim(s).to_owned())
            .collect(),
    )
}

pub struct AlphabetizePropertyValues;

impl Rule for AlphabetizePropertyValues {
    fn id(&self) -> &'static str {
        "alphabetize-property-values"
    }
    fn description(&self) -> &'static str {
        "Alphabetize the values of list properties (aliases, tags, links); keys and non-list values are untouched."
    }
    fn default_options(&self) -> Options {
        Options::new()
    }
    fn check(&self, doc: &Document, _options: &Options) -> Vec<Edit> {
        let fm = split_document(doc.text());
        if !fm.has {
            return Vec::new();
        }
        let entries: Vec<YamlEntry> = split_entries(&fm.yaml_lines)
            .into_iter()
            .map(|entry| {
                let Some(key) = entry.key.clone() else {
                    return entry;
                };
                if entry.lines.len() > 1 {
                    let items = &entry.lines[1..];
                    let caps: Vec<_> = items.iter().map(|l| BLOCK_ITEM.captures(l)).collect();
                    let all = caps.iter().all(Option::is_some);
                    let indents: std::collections::HashSet<&str> = caps
                        .iter()
                        .map(|c| {
                            c.as_ref()
                                .map_or("", |c| c.get(1).map_or("", |m| m.as_str()))
                        })
                        .collect();
                    if all && indents.len() == 1 {
                        let mut sorted: Vec<(&String, String)> = items
                            .iter()
                            .zip(&caps)
                            .map(|(l, c)| {
                                (l, c.as_ref().map_or(String::new(), |c| c[2].to_owned()))
                            })
                            .collect();
                        sorted.sort_by(|a, b| compare_values(&a.1, &b.1));
                        let mut lines = vec![entry.lines[0].clone()];
                        lines.extend(sorted.into_iter().map(|(l, _)| l.clone()));
                        return YamlEntry {
                            key: Some(key),
                            lines,
                        };
                    }
                    return entry;
                }
                if let Some(flow) = FLOW_ARRAY.captures(&entry.lines[0]) {
                    if let Some(mut items) = split_flow_items(&flow[2]) {
                        if items.len() > 1 {
                            items.sort_by(|a, b| compare_values(a, b));
                            return YamlEntry {
                                key: Some(key),
                                lines: vec![format!("{}[{}]", &flow[1], items.join(", "))],
                            };
                        }
                    }
                }
                entry
            })
            .collect();
        let out = join_document(&super::frontmatter::FrontmatterDoc {
            has: true,
            yaml_lines: entries_to_lines(&entries),
            body_lines: fm.body_lines,
        });
        text_edit(doc, out, "property values alphabetized")
    }
}

// ---- note-type-insert-keys ----------------------------------------------------

pub struct NoteTypeInsertKeys;

impl Rule for NoteTypeInsertKeys {
    fn id(&self) -> &'static str {
        "note-type-insert-keys"
    }
    fn description(&self) -> &'static str {
        "Insert required note-type frontmatter keys with their schema defaults when absent; existing values are never overwritten."
    }
    fn default_options(&self) -> Options {
        Options::new()
    }
    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit> {
        let Some(schema) = schema_from(options) else {
            return Vec::new();
        };
        let Some(required) = schema.get("required").and_then(Value::as_object) else {
            return Vec::new();
        };
        let fm = split_document(doc.text());
        let Some(data) = parse_frontmatter_data(&fm.yaml_lines) else {
            return Vec::new();
        };
        let inserts: Vec<_> = required
            .iter()
            .filter(|(k, d)| !d.is_null() && !data.contains_key(*k))
            .collect();
        if inserts.is_empty() {
            return Vec::new();
        }
        let order = key_order_from(schema);
        let mut entries = split_entries(&fm.yaml_lines);
        for (key, default) in inserts {
            entries = insert_ranked(
                &entries,
                YamlEntry {
                    key: Some(key.clone()),
                    lines: format_entry_lines(key, default),
                },
                &order,
            );
        }
        let out = join_document(&super::frontmatter::FrontmatterDoc {
            has: true,
            yaml_lines: entries_to_lines(&entries),
            body_lines: fm.body_lines,
        });
        text_edit(doc, out, "required keys inserted")
    }
}

// ---- note-type-key-sort ---------------------------------------------------------

pub struct NoteTypeKeySort;

impl Rule for NoteTypeKeySort {
    fn id(&self) -> &'static str {
        "note-type-key-sort"
    }
    fn description(&self) -> &'static str {
        "Order frontmatter keys by the note-type key-order, then the global default order (domain, category, sub-category, date-created, date-revised, remaining alphabetical, aliases and tags last)."
    }
    fn default_options(&self) -> Options {
        Options::new()
    }
    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit> {
        let Some(schema) = schema_from(options) else {
            return Vec::new();
        };
        let fm = split_document(doc.text());
        if !fm.has || parse_frontmatter_data(&fm.yaml_lines).is_none() {
            return Vec::new();
        }
        let order = key_order_from(schema);
        let sorted = sort_entries(&split_entries(&fm.yaml_lines), &order);
        let out = join_document(&super::frontmatter::FrontmatterDoc {
            has: true,
            yaml_lines: entries_to_lines(&sorted),
            body_lines: fm.body_lines,
        });
        text_edit(doc, out, "frontmatter keys sorted")
    }
}

// ---- note-type-date-keys / yaml-timestamp ------------------------------------------

fn normalize_out_key(text: &str, key: &str) -> String {
    let fm = split_document(text);
    if !fm.has {
        return text.to_owned();
    }
    let entries: Vec<YamlEntry> = split_entries(&fm.yaml_lines)
        .into_iter()
        .filter(|e| e.key.as_deref() != Some(key))
        .collect();
    join_document(&super::frontmatter::FrontmatterDoc {
        has: true,
        yaml_lines: entries_to_lines(&entries),
        body_lines: fm.body_lines,
    })
}

fn truthy_str(v: Option<&Value>) -> Option<&str> {
    v.and_then(Value::as_str).filter(|s| !s.is_empty())
}

/// The shared date-key engine. `originalText` and `today` come from options.
pub fn apply_date_keys(text: &str, options: &Options) -> String {
    let Some(schema) = schema_from(options) else {
        return text.to_owned();
    };
    let Some(date_keys) = schema.get("date-keys").and_then(Value::as_object) else {
        return text.to_owned();
    };
    let Some(today) = truthy_str(options.get("today")) else {
        return text.to_owned();
    };
    let original = options
        .get("originalText")
        .and_then(Value::as_str)
        .unwrap_or(text);
    let order = key_order_from(schema);
    let mut out = text.to_owned();

    if let Some(created) = truthy_str(date_keys.get("created")) {
        let fm = split_document(&out);
        if let Some(data) = parse_frontmatter_data(&fm.yaml_lines) {
            if !data.contains_key(created) {
                let entries = insert_ranked(
                    &split_entries(&fm.yaml_lines),
                    YamlEntry {
                        key: Some(created.to_owned()),
                        lines: vec![format!("{created}: {today}")],
                    },
                    &order,
                );
                out = join_document(&super::frontmatter::FrontmatterDoc {
                    has: true,
                    yaml_lines: entries_to_lines(&entries),
                    body_lines: fm.body_lines,
                });
            }
        }
    }

    if let Some(revised) = truthy_str(date_keys.get("revised")) {
        if normalize_out_key(original, revised) != normalize_out_key(&out, revised) {
            let fm = split_document(&out);
            if parse_frontmatter_data(&fm.yaml_lines).is_some() {
                let line = format!("{revised}: {today}");
                let mut entries = split_entries(&fm.yaml_lines);
                let entry = YamlEntry {
                    key: Some(revised.to_owned()),
                    lines: vec![line],
                };
                if let Some(i) = entries
                    .iter()
                    .position(|e| e.key.as_deref() == Some(revised))
                {
                    entries[i] = entry;
                } else {
                    entries = insert_ranked(&entries, entry, &order);
                }
                out = join_document(&super::frontmatter::FrontmatterDoc {
                    has: true,
                    yaml_lines: entries_to_lines(&entries),
                    body_lines: fm.body_lines,
                });
            }
        }
    }
    out
}

pub struct NoteTypeDateKeys;

impl Rule for NoteTypeDateKeys {
    fn id(&self) -> &'static str {
        "note-type-date-keys"
    }
    fn description(&self) -> &'static str {
        "Set the created date key on first lint when absent and bump the revised date key only when the run changed other content."
    }
    fn default_options(&self) -> Options {
        Options::new()
    }
    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit> {
        text_edit(
            doc,
            apply_date_keys(doc.text(), options),
            "date keys updated",
        )
    }
}

pub struct YamlTimestamp;

impl Rule for YamlTimestamp {
    fn id(&self) -> &'static str {
        "yaml-timestamp"
    }
    fn description(&self) -> &'static str {
        "Keep date-created and date-revised current (yyyy-MM-dd): created set on first lint, revised bumped only when other content changed."
    }
    fn default_options(&self) -> Options {
        opts(
            json!({"schema": {"date-keys": {"created": "date-created", "revised": "date-revised"}}}),
        )
    }
    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit> {
        text_edit(
            doc,
            apply_date_keys(doc.text(), options),
            "date keys updated",
        )
    }
}
