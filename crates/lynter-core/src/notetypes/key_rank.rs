use super::frontmatter::YamlEntry;
use std::cmp::Ordering;

pub const GLOBAL_KEY_ORDER_HEAD: [&str; 8] = [
    "preset",
    "domain",
    "category",
    "sub-category",
    "types",
    "date-created",
    "date-revised",
    "links",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Sub {
    Index(usize),
    Name(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyRank {
    pub group: u8,
    pub sub: Sub,
}

pub fn rank_key(key: &str, key_order: &[String]) -> KeyRank {
    let r = |group, sub| KeyRank { group, sub };
    if key == "relo-file" {
        return r(4, Sub::Index(0));
    }
    if key == "relo-folder" {
        return r(4, Sub::Index(1));
    }
    if let Some(i) = key_order.iter().position(|k| k == key) {
        return r(0, Sub::Index(i));
    }
    if let Some(i) = GLOBAL_KEY_ORDER_HEAD.iter().position(|k| *k == key) {
        return r(1, Sub::Index(i));
    }
    match key {
        "aliases" => r(3, Sub::Index(0)),
        "tags" => r(3, Sub::Index(1)),
        _ => r(2, Sub::Name(key.to_lowercase())),
    }
}

/// Compare as JavaScript does: numbers numerically, strings by UTF-16 code unit.
pub fn compare_rank(a: &KeyRank, b: &KeyRank) -> Ordering {
    a.group.cmp(&b.group).then_with(|| match (&a.sub, &b.sub) {
        (Sub::Index(x), Sub::Index(y)) => x.cmp(y),
        (Sub::Name(x), Sub::Name(y)) => x.encode_utf16().cmp(y.encode_utf16()),
        (Sub::Index(x), Sub::Name(y)) => x.to_string().encode_utf16().cmp(y.encode_utf16()),
        (Sub::Name(x), Sub::Index(y)) => x.encode_utf16().cmp(y.to_string().encode_utf16()),
    })
}

/// Insert a new entry at its ranked position among existing keyed entries.
/// Loose entries stay put; ties append after equals.
pub fn insert_ranked(
    entries: &[YamlEntry],
    entry: YamlEntry,
    key_order: &[String],
) -> Vec<YamlEntry> {
    let mut out = entries.to_vec();
    let Some(key) = &entry.key else {
        out.push(entry);
        return out;
    };
    let new_rank = rank_key(key, key_order);
    let index = entries.iter().position(|e| {
        e.key
            .as_deref()
            .is_some_and(|k| compare_rank(&rank_key(k, key_order), &new_rank) == Ordering::Greater)
    });
    out.insert(index.unwrap_or(out.len()), entry);
    out
}
