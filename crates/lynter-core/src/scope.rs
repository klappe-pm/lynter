//! Scope engine: which profiles apply to a file, and the merged rule config.
//!
//! Precedence, low to high: defaults, extension matches, path glob matches
//! (deeper glob wins), frontmatter and tag matches, then an explicit
//! `linter-profile: <name>` frontmatter key applied last. Ties follow
//! declaration order, later declarations winning key by key.

use crate::config::{Config, DateRange, ProfileMatch, RulesConfig};
use globset::{GlobBuilder, GlobMatcher};
use serde_json::{Map, Value};

pub const RANK_NONE: u8 = 0;
pub const RANK_EXTENSION: u8 = 1;
pub const RANK_PATH: u8 = 2;
pub const RANK_FRONTMATTER: u8 = 3;

const OVERRIDE_KEY: &str = "linter-profile";

/// External per-file facts the pure core cannot derive from text alone.
#[derive(Clone, Debug, Default)]
pub struct MatchContext {
    pub tags: Vec<String>,
    /// ISO `yyyy-MM-dd` "today", for age.
    pub today: Option<String>,
    pub backlinks: Vec<String>,
    pub aliases: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct FileFacts {
    pub path: String,
    /// Raw frontmatter YAML text without the fences.
    pub frontmatter: Option<String>,
    pub context: MatchContext,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedProfile {
    pub chain: Vec<String>,
    pub rules: RulesConfig,
    pub rule_order: Option<Vec<String>>,
    pub note_type: Option<String>,
}

// ---- dates ----------------------------------------------------------------

fn days_in_month(y: i32, m: u32) -> u32 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ => {
            if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 {
                29
            } else {
                28
            }
        }
    }
}

/// True for a real calendar date with a four digit year of 100 or later.
pub fn valid_ymd(y: i32, m: u32, d: u32) -> bool {
    y >= 100 && (1..=12).contains(&m) && d >= 1 && d <= days_in_month(y, m)
}

fn day_number(y: i32, m: u32, d: u32) -> i64 {
    let y = i64::from(y) - i64::from(m <= 2);
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let mp = (i64::from(m) + 9) % 12;
    let doy = (153 * mp + 2) / 5 + i64::from(d) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// UTC day number of an ISO date (a time suffix is ignored), or `None`.
pub fn iso_to_day_number(value: &str) -> Option<i64> {
    let b = value.trim().as_bytes();
    if b.len() < 10 || b[4] != b'-' || b[7] != b'-' {
        return None;
    }
    if !b[..10]
        .iter()
        .enumerate()
        .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
    {
        return None;
    }
    let s = std::str::from_utf8(&b[..10]).ok()?;
    let (y, m, d): (i32, u32, u32) = (
        s[0..4].parse().ok()?,
        s[5..7].parse().ok()?,
        s[8..10].parse().ok()?,
    );
    valid_ymd(y, m, d).then(|| day_number(y, m, d))
}

/// The 5-day bucket label for an age in days; ages below 1 land in "1-5".
pub fn age_bucket_for_days(days: i64) -> String {
    let d = days.max(1);
    let lower = (d - 1) / 5 * 5 + 1;
    format!("{}-{}", lower, lower + 4)
}

pub fn age_bucket(created: Option<&Value>, today: Option<&str>) -> Option<String> {
    let c = iso_to_day_number(created?.as_str()?)?;
    let t = iso_to_day_number(today?)?;
    Some(age_bucket_for_days(t - c))
}

/// True when `date` falls within `[after, before]` inclusive.
pub fn date_in_range(date: Option<&Value>, range: &DateRange) -> bool {
    let Some(day) = date.and_then(Value::as_str).and_then(iso_to_day_number) else {
        return false;
    };
    let after = range.after.as_deref().map(iso_to_day_number);
    let before = range.before.as_deref().map(iso_to_day_number);
    if matches!(after, Some(None)) || matches!(before, Some(None)) {
        return false;
    }
    let (after, before) = (after.flatten(), before.flatten());
    if let (Some(a), Some(b)) = (after, before) {
        if a > b {
            return false;
        }
    }
    after.is_none_or(|a| day >= a) && before.is_none_or(|b| day <= b)
}

fn list_includes_any(wanted: &[String], actual: &[String]) -> bool {
    let have: Vec<String> = actual.iter().map(|v| v.to_lowercase()).collect();
    wanted.iter().any(|w| have.contains(&w.to_lowercase()))
}

// ---- frontmatter ----------------------------------------------------------

/// Parse raw frontmatter YAML into a flat mapping. `None` when absent, blank,
/// malformed, or not a mapping; never an error.
pub fn parse_frontmatter(raw: Option<&str>) -> Option<Map<String, Value>> {
    let raw = raw?;
    if raw.trim().is_empty() {
        return None;
    }
    match crate::config::yaml_to_json(raw).ok()? {
        Value::Object(m) => Some(m),
        _ => None,
    }
}

// ---- globs ----------------------------------------------------------------

pub struct CompiledGlob {
    /// Count of `/`-separated segments in the glob; deeper outranks shallower.
    pub depth: usize,
    matchers: Vec<GlobMatcher>,
}

fn sanitize_segment(seg: &str) -> String {
    let mut out = String::new();
    let mut prev_star = false;
    for c in seg.chars() {
        if c == '*' {
            if !prev_star {
                out.push('*');
            }
            prev_star = true;
            continue;
        }
        prev_star = false;
        match c {
            '\\' => out.push_str("\\\\"),
            '[' => out.push_str("[[]"),
            ']' => out.push_str("[]]"),
            '{' => out.push_str("[{]"),
            '}' => out.push_str("[}]"),
            _ => out.push(c),
        }
    }
    out
}

/// Compile a vault-relative glob. Supports `**` (zero or more whole segments,
/// so `a/**` also matches `a`), `*`, and `?` within a segment; brackets and
/// braces are literal.
pub fn compile_glob(glob: &str) -> CompiledGlob {
    let mut parts: Vec<String> = Vec::new();
    for seg in glob.split('/') {
        let part = if seg == "**" {
            "**".to_owned()
        } else {
            sanitize_segment(seg)
        };
        if part == "**" && parts.last().is_some_and(|p| p == "**") {
            continue;
        }
        parts.push(part);
    }
    let mut patterns = vec![parts.join("/")];
    if parts.len() > 1 && parts.last().is_some_and(|p| p == "**") {
        patterns.push(parts[..parts.len() - 1].join("/"));
    }
    let matchers = patterns
        .iter()
        .filter_map(|p| GlobBuilder::new(p).literal_separator(true).build().ok())
        .map(|g| g.compile_matcher())
        .collect();
    CompiledGlob {
        depth: glob.split('/').count(),
        matchers,
    }
}

impl CompiledGlob {
    pub fn is_match(&self, path: &str) -> bool {
        self.matchers.iter().any(|m| m.is_match(path))
    }
}

// ---- matchers -------------------------------------------------------------

fn js_number_string(n: &serde_json::Number) -> String {
    match n.as_f64() {
        Some(f) if f.fract() == 0.0 && f.abs() < 1e15 => format!("{}", f as i64),
        _ => n.to_string(),
    }
}

fn scalar_string(v: &Value) -> Option<String> {
    match v {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(js_number_string(n)),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

fn normalize_tag(tag: &str) -> String {
    tag.strip_prefix('#').unwrap_or(tag).to_owned()
}

fn tags_of(fm: &Map<String, Value>) -> Vec<String> {
    let raw = match fm.get("tags") {
        Some(v) if !v.is_null() => Some(v),
        _ => fm.get("tag").filter(|v| !v.is_null()),
    };
    match raw {
        Some(Value::Array(a)) => a
            .iter()
            .filter_map(scalar_string)
            .map(|t| normalize_tag(&t))
            .collect(),
        Some(Value::String(s)) => s
            .split(|c: char| c == ',' || c.is_whitespace())
            .filter(|p| !p.is_empty())
            .map(normalize_tag)
            .collect(),
        Some(v @ (Value::Number(_) | Value::Bool(_))) => scalar_string(v)
            .map(|t| vec![normalize_tag(&t)])
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

fn scalar_eq(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Number(x), Value::Number(y)) => x.as_f64() == y.as_f64(),
        _ => a == b && a.is_string() || a == b && a.is_boolean(),
    }
}

fn frontmatter_value_matches(expected: &Value, actual: Option<&Value>) -> bool {
    let Some(actual) = actual else { return false };
    match (expected, actual) {
        (Value::Array(exp), Value::Array(act)) => {
            exp.iter().all(|e| act.iter().any(|a| scalar_eq(a, e)))
        }
        (Value::Array(_), _) => false,
        (e, Value::Array(act)) => act.iter().any(|a| scalar_eq(a, e)),
        (e, a) => scalar_eq(a, e),
    }
}

fn extension_of(path: &str) -> Option<String> {
    let base = &path[path.rfind('/').map_or(0, |i| i + 1)..];
    base.rfind('.')
        .filter(|&d| d > 0)
        .map(|d| base[d + 1..].to_lowercase())
}

pub struct CompiledMatch {
    pub rank: u8,
    extensions: Option<Vec<String>>,
    globs: Option<Vec<CompiledGlob>>,
    fm_entries: Option<Vec<(String, Value)>>,
    tags: Option<Vec<String>>,
    ages: Option<Vec<String>>,
    created: Option<DateRange>,
    revised: Option<DateRange>,
    backlinks: Option<Vec<String>>,
    aliases: Option<Vec<String>>,
}

/// Result of evaluating a matcher: `Some(path_depth)` on a match.
pub type Evaluation = Option<usize>;

fn non_empty<T: Clone>(v: &[T]) -> Option<Vec<T>> {
    (!v.is_empty()).then(|| v.to_vec())
}

/// Compile a match. Every kind present must match (AND across kinds); within
/// one kind any alternative suffices (OR). An empty match has rank 0 and never
/// matches, so such a profile applies only through the explicit override key.
pub fn compile_match(m: &ProfileMatch) -> CompiledMatch {
    let extensions =
        non_empty(&m.extension).map(|v| v.iter().map(|e| e.to_lowercase()).collect::<Vec<_>>());
    let globs = non_empty(&m.path).map(|v| v.iter().map(|g| compile_glob(g)).collect::<Vec<_>>());
    let fm_entries = non_empty(&m.frontmatter);
    let tags = non_empty(&m.tag).map(|v| v.iter().map(|t| normalize_tag(t)).collect::<Vec<_>>());
    let ages = non_empty(&m.age);
    let backlinks = non_empty(&m.backlink);
    let aliases = non_empty(&m.alias);
    let has_fm_kind = fm_entries.is_some()
        || tags.is_some()
        || ages.is_some()
        || m.date_created.is_some()
        || m.date_revised.is_some()
        || backlinks.is_some()
        || aliases.is_some();
    let rank = if has_fm_kind {
        RANK_FRONTMATTER
    } else if globs.is_some() {
        RANK_PATH
    } else if extensions.is_some() {
        RANK_EXTENSION
    } else {
        RANK_NONE
    };
    CompiledMatch {
        rank,
        extensions,
        globs,
        fm_entries,
        tags,
        ages,
        created: m.date_created.clone(),
        revised: m.date_revised.clone(),
        backlinks,
        aliases,
    }
}

impl CompiledMatch {
    pub fn evaluate(
        &self,
        path: &str,
        fm: Option<&Map<String, Value>>,
        ctx: &MatchContext,
    ) -> Evaluation {
        if self.rank == RANK_NONE {
            return None;
        }
        if let Some(exts) = &self.extensions {
            if !extension_of(path).is_some_and(|e| exts.contains(&e)) {
                return None;
            }
        }
        let mut depth = 0;
        if let Some(globs) = &self.globs {
            for g in globs {
                if g.is_match(path) && g.depth > depth {
                    depth = g.depth;
                }
            }
            if depth == 0 {
                return None;
            }
        }
        if let Some(entries) = &self.fm_entries {
            let fm = fm?;
            if !entries
                .iter()
                .all(|(k, e)| frontmatter_value_matches(e, fm.get(k)))
            {
                return None;
            }
        }
        if let Some(wanted) = &self.tags {
            let mut present = fm.map(tags_of).unwrap_or_default();
            present.extend(ctx.tags.iter().map(|t| normalize_tag(t)));
            if !wanted.iter().any(|t| present.contains(t)) {
                return None;
            }
        }
        if let Some(ages) = &self.ages {
            let bucket = age_bucket(fm.and_then(|f| f.get("date-created")), ctx.today.as_deref())?;
            if !ages.contains(&bucket) {
                return None;
            }
        }
        if let Some(r) = &self.created {
            if !date_in_range(fm.and_then(|f| f.get("date-created")), r) {
                return None;
            }
        }
        if let Some(r) = &self.revised {
            if !date_in_range(fm.and_then(|f| f.get("date-revised")), r) {
                return None;
            }
        }
        if let Some(b) = &self.backlinks {
            if !list_includes_any(b, &ctx.backlinks) {
                return None;
            }
        }
        if let Some(a) = &self.aliases {
            if !list_includes_any(a, &ctx.aliases) {
                return None;
            }
        }
        Some(depth)
    }
}

// ---- resolver -------------------------------------------------------------

struct CompiledProfile {
    name: String,
    match_: CompiledMatch,
    rules: Option<RulesConfig>,
    rule_order: Option<Vec<String>>,
}

/// A config compiled once for repeated resolution.
pub struct Resolver {
    profiles: Vec<CompiledProfile>,
    note_types: Vec<(String, CompiledMatch)>,
    defaults: Option<RulesConfig>,
    rule_order: Option<Vec<String>>,
}

fn merge_rules(target: &mut RulesConfig, layer: Option<&RulesConfig>) {
    let Some(layer) = layer else { return };
    for (id, cfg) in layer {
        let entry = target
            .entry(id.clone())
            .or_insert_with(|| Value::Object(Map::new()));
        if let (Value::Object(t), Value::Object(l)) = (entry, cfg) {
            for (k, v) in l {
                t.insert(k.clone(), v.clone());
            }
        }
    }
}

impl Resolver {
    pub fn new(config: &Config) -> Resolver {
        let empty = ProfileMatch::default();
        Resolver {
            profiles: config
                .profiles
                .iter()
                .map(|p| CompiledProfile {
                    name: p.name.clone(),
                    match_: compile_match(p.match_.as_ref().unwrap_or(&empty)),
                    rules: p.rules.clone(),
                    rule_order: p.rule_order.clone(),
                })
                .collect(),
            note_types: config
                .note_types
                .iter()
                .map(|n| {
                    (
                        n.name.clone(),
                        compile_match(n.match_.as_ref().unwrap_or(&empty)),
                    )
                })
                .collect(),
            defaults: config.defaults_rules.clone(),
            rule_order: config.rule_order.clone(),
        }
    }

    pub fn resolve(&self, facts: &FileFacts) -> ResolvedProfile {
        let fm = parse_frontmatter(facts.frontmatter.as_deref());
        let ctx = &facts.context;
        let mut matched: Vec<(usize, u8, usize)> = Vec::new();
        for (index, p) in self.profiles.iter().enumerate() {
            if let Some(depth) = p.match_.evaluate(&facts.path, fm.as_ref(), ctx) {
                matched.push((index, p.match_.rank, depth));
            }
        }
        matched.sort_by_key(|&(index, rank, depth)| (rank, depth, index));
        let mut chain: Vec<usize> = matched.iter().map(|m| m.0).collect();
        if let Some(Value::String(name)) = fm.as_ref().and_then(|f| f.get(OVERRIDE_KEY)) {
            if let Some(i) = self.profiles.iter().position(|p| &p.name == name) {
                chain.retain(|&c| c != i);
                chain.push(i);
            }
        }
        let mut rules = RulesConfig::new();
        merge_rules(&mut rules, self.defaults.as_ref());
        for &i in &chain {
            merge_rules(&mut rules, self.profiles[i].rules.as_ref());
        }
        let note_type = self
            .note_types
            .iter()
            .find(|(_, m)| m.evaluate(&facts.path, fm.as_ref(), ctx).is_some())
            .map(|(n, _)| n.clone());
        let mut rule_order = self.rule_order.clone();
        for &i in &chain {
            if let Some(o) = &self.profiles[i].rule_order {
                if !o.is_empty() {
                    rule_order = Some(o.clone());
                }
            }
        }
        let mut names = vec!["defaults".to_owned()];
        names.extend(chain.iter().map(|&i| self.profiles[i].name.clone()));
        ResolvedProfile {
            chain: names,
            rules,
            rule_order,
            note_type,
        }
    }
}

/// Resolve one file against a config.
pub fn resolve_profile(facts: &FileFacts, config: &Config) -> ResolvedProfile {
    Resolver::new(config).resolve(facts)
}

/// Obsidian-style inline tags outside code, metadata, math, links, and URLs.
pub fn inline_tags(text: &str) -> Vec<String> {
    use std::sync::LazyLock;
    static TAG: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"(?:^|[\s(])#([\p{L}\p{N}_/-]+)").expect("regex"));
    static HAS_LETTER: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"[\p{L}_/-]").expect("regex"));
    if !text.contains('#') {
        return Vec::new();
    }
    let doc = crate::document::Document::parse(text);
    let mut tags: Vec<String> = Vec::new();
    for caps in TAG.captures_iter(text) {
        let whole = caps.get(0).expect("group 0");
        let body = caps.get(1).expect("group 1");
        let start = body.start() - 1;
        if HAS_LETTER.is_match(body.as_str())
            && !doc.is_protected(start, whole.end())
            && !tags.iter().any(|t| t == body.as_str())
        {
            tags.push(body.as_str().to_owned());
        }
    }
    tags
}
