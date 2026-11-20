//! Fail-closed validation, ported rule by rule from the TypeScript loader.

use super::types::{
    Config, ConfigError, DateRange, LoadResult, NoteType, Profile, ProfileMatch, RulesConfig,
};
use serde_json::{Map, Value};

type Obj = Map<String, Value>;

const TOP_LEVEL_KEYS: &[&str] = &[
    "version",
    "defaults",
    "rule-order",
    "profiles",
    "note-types",
    "templates",
    "automations",
    "rename",
    "ignore",
    "providers",
    "code-checks",
];
const MATCH_KEYS: &[&str] = &[
    "path",
    "extension",
    "frontmatter",
    "tag",
    "age",
    "date-created",
    "date-revised",
    "backlink",
    "alias",
];
const PROFILE_KEYS: &[&str] = &["match", "rules", "rule-order"];
const NOTE_TYPE_KEYS: &[&str] = &["required", "key-order", "values", "date-keys", "match"];
const GLOBAL_TEMPLATE_KEYS: &[&str] = &[
    "pinned-keys",
    "key-order",
    "frontmatter",
    "body",
    "age-line",
];
const SCOPED_TEMPLATE_KEYS: &[&str] = &[
    "name",
    "extends",
    "match",
    "toggles",
    "pinned-keys",
    "key-order",
    "frontmatter",
    "body",
    "age-line",
];
const TEMPLATES_KEYS: &[&str] = &["global", "by-scope"];
const AUTOMATION_KEYS: &[&str] = &[
    "name", "trigger", "action", "failure", "log", "report", "schedule", "scope",
];
const AUTOMATION_TRIGGERS: &[&str] = &[
    "on-write",
    "on-create",
    "on-rename",
    "pre-commit",
    "ci",
    "schedule",
    "manual",
];
const CODE_CHECK_KEYS: &[&str] = &[
    "enabled",
    "description",
    "paths",
    "languages",
    "pattern",
    "flags",
    "message",
    "fix",
];

fn is_scalar(v: &Value) -> bool {
    matches!(v, Value::String(_) | Value::Number(_) | Value::Bool(_))
}

fn is_scalar_array(v: &Value) -> bool {
    v.as_array().is_some_and(|a| a.iter().all(is_scalar))
}

fn is_string_array(v: &Value) -> bool {
    v.as_array().is_some_and(|a| a.iter().all(Value::is_string))
}

fn is_valid_iso_date(v: &Value) -> bool {
    let Some(s) = v.as_str() else { return false };
    let b = s.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return false;
    }
    if !b
        .iter()
        .enumerate()
        .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
    {
        return false;
    }
    let (y, m, d): (i32, u32, u32) = (
        s[0..4].parse().unwrap_or(0),
        s[5..7].parse().unwrap_or(0),
        s[8..10].parse().unwrap_or(0),
    );
    crate::scope::valid_ymd(y, m, d)
}

#[derive(Default)]
struct V {
    errors: Vec<ConfigError>,
    warnings: Vec<String>,
}

impl V {
    fn error(&mut self, path: impl Into<String>, message: impl Into<String>) {
        self.errors.push(ConfigError {
            path: path.into(),
            message: message.into(),
        });
    }
    fn warn(&mut self, message: String) {
        self.warnings.push(message);
    }
    fn unknown_keys(&mut self, obj: &Obj, allowed: &[&str], path: &str) {
        for k in obj.keys() {
            if !allowed.contains(&k.as_str()) {
                self.warn(format!("unknown key \"{path}.{k}\" ignored"));
            }
        }
    }
}

fn validate_rules(v: &mut V, value: &Value, path: &str) {
    let Some(map) = value.as_object() else {
        v.error(path, "must be a mapping of rule id to rule config");
        return;
    };
    for (rule_id, cfg) in map {
        let rule_path = format!("{path}.{rule_id}");
        let Some(cfg) = cfg.as_object() else {
            v.error(rule_path, "rule config must be a mapping");
            continue;
        };
        if let Some(en) = cfg.get("enabled") {
            if !en.is_boolean() {
                v.error(format!("{rule_path}.enabled"), "must be a boolean");
            }
        }
    }
}

fn validate_match(v: &mut V, value: &Value, path: &str) {
    let Some(m) = value.as_object() else {
        v.error(path, "must be a mapping");
        return;
    };
    v.unknown_keys(m, MATCH_KEYS, path);
    for key in ["path", "extension", "tag", "age", "backlink", "alias"] {
        if let Some(x) = m.get(key) {
            if !is_string_array(x) {
                v.error(format!("{path}.{key}"), "must be a list of strings");
            }
        }
    }
    for key in ["date-created", "date-revised"] {
        if let Some(range) = m.get(key) {
            match range.as_object() {
                None => v.error(
                    format!("{path}.{key}"),
                    "must be a mapping with after and/or before",
                ),
                Some(r) => {
                    for bound in ["after", "before"] {
                        if let Some(b) = r.get(bound) {
                            if !is_valid_iso_date(b) {
                                v.error(
                                    format!("{path}.{key}.{bound}"),
                                    "must be an ISO yyyy-MM-dd string",
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    if let Some(fm) = m.get("frontmatter") {
        match fm.as_object() {
            None => v.error(
                format!("{path}.frontmatter"),
                "must be a mapping of key to value predicate",
            ),
            Some(fm) => {
                for (k, val) in fm {
                    if !is_scalar(val) && !is_scalar_array(val) {
                        v.error(
                            format!("{path}.frontmatter.{k}"),
                            "must be a scalar or a list of scalars",
                        );
                    }
                }
            }
        }
    }
}

fn validate_profiles(v: &mut V, value: &Value, path: &str) {
    let Some(map) = value.as_object() else {
        v.error(path, "must be a mapping of profile name to profile config");
        return;
    };
    for (name, profile) in map {
        let pp = format!("{path}.{name}");
        let Some(p) = profile.as_object() else {
            v.error(pp, "profile must be a mapping");
            continue;
        };
        v.unknown_keys(p, PROFILE_KEYS, &pp);
        if let Some(m) = p.get("match") {
            validate_match(v, m, &format!("{pp}.match"));
        }
        if let Some(r) = p.get("rules") {
            validate_rules(v, r, &format!("{pp}.rules"));
        }
        if let Some(o) = p.get("rule-order") {
            if !is_string_array(o) {
                v.error(format!("{pp}.rule-order"), "must be a list of rule ids");
            }
        }
    }
}

fn validate_note_types(v: &mut V, value: &Value, path: &str) {
    let Some(map) = value.as_object() else {
        v.error(path, "must be a mapping of note type name to schema");
        return;
    };
    for (name, schema) in map {
        let sp = format!("{path}.{name}");
        let Some(s) = schema.as_object() else {
            v.error(sp, "note type schema must be a mapping");
            continue;
        };
        v.unknown_keys(s, NOTE_TYPE_KEYS, &sp);
        if let Some(req) = s.get("required") {
            match req.as_object() {
                None => v.error(
                    format!("{sp}.required"),
                    "must be a mapping of key to default value",
                ),
                Some(req) => {
                    for (k, val) in req {
                        if !val.is_null() && !is_scalar(val) && !is_scalar_array(val) {
                            v.error(
                                format!("{sp}.required.{k}"),
                                "must be a scalar, a list of scalars, or null",
                            );
                        }
                    }
                }
            }
        }
        if let Some(ko) = s.get("key-order") {
            if !is_string_array(ko) {
                v.error(format!("{sp}.key-order"), "must be a list of strings");
            }
        }
        if let Some(values) = s.get("values") {
            match values.as_object() {
                None => v.error(
                    format!("{sp}.values"),
                    "must be a mapping of key to allowed values",
                ),
                Some(values) => {
                    for (k, allowed) in values {
                        if !is_scalar_array(allowed) {
                            v.error(format!("{sp}.values.{k}"), "must be a list of scalars");
                        }
                    }
                }
            }
        }
        if let Some(dk) = s.get("date-keys") {
            match dk.as_object() {
                None => v.error(
                    format!("{sp}.date-keys"),
                    "must be a mapping with optional created/revised keys",
                ),
                Some(dk) => {
                    for (k, val) in dk {
                        if k != "created" && k != "revised" {
                            v.warn(format!("unknown key \"{sp}.date-keys.{k}\" ignored"));
                        } else if !val.is_string() {
                            v.error(format!("{sp}.date-keys.{k}"), "must be a string");
                        }
                    }
                }
            }
        }
        if let Some(m) = s.get("match") {
            validate_match(v, m, &format!("{sp}.match"));
        }
    }
}

fn validate_template_frontmatter(v: &mut V, value: &Value, path: &str) {
    let Some(map) = value.as_object() else {
        v.error(path, "must be a mapping of key to seed value");
        return;
    };
    for (k, seed) in map {
        if !seed.is_null() && !is_scalar(seed) && !is_scalar_array(seed) {
            v.error(
                format!("{path}.{k}"),
                "must be a scalar, a list of scalars, or null",
            );
        }
    }
}

fn validate_template_fields(v: &mut V, obj: &Obj, path: &str, allowed: &[&str]) {
    v.unknown_keys(obj, allowed, path);
    for key in ["pinned-keys", "key-order"] {
        if let Some(x) = obj.get(key) {
            if !is_string_array(x) {
                v.error(format!("{path}.{key}"), "must be a list of strings");
            }
        }
    }
    if let Some(fm) = obj.get("frontmatter") {
        validate_template_frontmatter(v, fm, &format!("{path}.frontmatter"));
    }
    if let Some(b) = obj.get("body") {
        if !b.is_string() {
            v.error(format!("{path}.body"), "must be a string");
        }
    }
    if let Some(a) = obj.get("age-line") {
        if !a.is_boolean() {
            v.error(format!("{path}.age-line"), "must be a boolean");
        }
    }
}

fn non_empty_string(v: Option<&Value>) -> Option<&str> {
    v.and_then(Value::as_str).filter(|s| !s.trim().is_empty())
}

fn validate_templates(v: &mut V, value: &Value, path: &str) {
    let Some(t) = value.as_object() else {
        v.error(path, "must be a mapping with optional global and by-scope");
        return;
    };
    v.unknown_keys(t, TEMPLATES_KEYS, path);
    if let Some(g) = t.get("global") {
        match g.as_object() {
            None => v.error(format!("{path}.global"), "must be a mapping"),
            Some(g) => {
                validate_template_fields(v, g, &format!("{path}.global"), GLOBAL_TEMPLATE_KEYS)
            }
        }
    }
    if let Some(scoped) = t.get("by-scope") {
        let Some(list) = scoped.as_array() else {
            v.error(
                format!("{path}.by-scope"),
                "must be a list of scoped templates",
            );
            return;
        };
        let mut seen: Vec<String> = Vec::new();
        for (i, entry) in list.iter().enumerate() {
            let ep = format!("{path}.by-scope[{i}]");
            let Some(e) = entry.as_object() else {
                v.error(ep, "scoped template must be a mapping");
                continue;
            };
            match non_empty_string(e.get("name")) {
                None => v.error(format!("{ep}.name"), "must be a non-empty string"),
                Some(n) if seen.iter().any(|s| s == n) => v.error(
                    format!("{ep}.name"),
                    format!("duplicate template name \"{n}\""),
                ),
                Some(n) => seen.push(n.to_owned()),
            }
            if let Some(x) = e.get("extends") {
                if x.as_str() != Some("global") {
                    v.error(format!("{ep}.extends"), "only \"global\" is supported");
                }
            }
            if let Some(m) = e.get("match") {
                validate_match(v, m, &format!("{ep}.match"));
            }
            if let Some(toggles) = e.get("toggles") {
                match toggles.as_object() {
                    None => v.error(
                        format!("{ep}.toggles"),
                        "must be a mapping of key to on/off",
                    ),
                    Some(tg) => {
                        for (k, val) in tg {
                            let ok = val.is_boolean() || matches!(val.as_str(), Some("on" | "off"));
                            if !ok {
                                v.error(
                                    format!("{ep}.toggles.{k}"),
                                    "must be a boolean or \"on\"/\"off\"",
                                );
                            }
                        }
                    }
                }
            }
            validate_template_fields(v, e, &ep, SCOPED_TEMPLATE_KEYS);
        }
    }
}

fn validate_automations(v: &mut V, value: &Value, path: &str) {
    let Some(list) = value.as_array() else {
        v.error(path, "must be a list of automations");
        return;
    };
    let mut seen: Vec<String> = Vec::new();
    for (i, entry) in list.iter().enumerate() {
        let ep = format!("{path}[{i}]");
        let Some(e) = entry.as_object() else {
            v.error(ep, "automation must be a mapping");
            continue;
        };
        v.unknown_keys(e, AUTOMATION_KEYS, &ep);
        match non_empty_string(e.get("name")) {
            None => v.error(format!("{ep}.name"), "must be a non-empty string"),
            Some(n) if seen.iter().any(|s| s == n) => v.error(
                format!("{ep}.name"),
                format!("duplicate automation name \"{n}\""),
            ),
            Some(n) => seen.push(n.to_owned()),
        }
        let trigger = e.get("trigger").and_then(Value::as_str);
        if !trigger.is_some_and(|t| AUTOMATION_TRIGGERS.contains(&t)) {
            v.error(
                format!("{ep}.trigger"),
                "must be one of on-write, on-create, on-rename, pre-commit, ci, schedule, manual",
            );
        }
        let enums: [(&str, &[&str]); 4] = [
            ("action", &["check", "fix", "apply-template"]),
            ("failure", &["open", "closed"]),
            ("log", &["spool", "none"]),
            ("report", &["md", "json", "none"]),
        ];
        for (key, allowed) in enums {
            if let Some(x) = e.get(key) {
                if !x.as_str().is_some_and(|s| allowed.contains(&s)) {
                    v.error(
                        format!("{ep}.{key}"),
                        format!("must be one of {}", allowed.join(", ")),
                    );
                }
            }
        }
        if let Some(s) = e.get("schedule") {
            if !s.is_string() {
                v.error(format!("{ep}.schedule"), "must be a cron string");
            }
        }
        if trigger == Some("schedule") && !e.get("schedule").is_some_and(Value::is_string) {
            v.error(
                format!("{ep}.schedule"),
                "a schedule trigger requires a cron string",
            );
        }
        if let Some(sc) = e.get("scope") {
            validate_match(v, sc, &format!("{ep}.scope"));
        }
    }
}

fn validate_rename(v: &mut V, value: &Value, path: &str) {
    let Some(r) = value.as_object() else {
        v.error(path, "must be a mapping with a mode key");
        return;
    };
    v.unknown_keys(r, &["mode", "strategy"], path);
    if !r
        .get("mode")
        .and_then(Value::as_str)
        .is_some_and(|m| matches!(m, "off" | "flag" | "rename"))
    {
        v.error(
            format!("{path}.mode"),
            "must be one of \"off\", \"flag\", \"rename\"",
        );
    }
    if let Some(s) = r.get("strategy") {
        if !s
            .as_str()
            .is_some_and(|m| matches!(m, "replace" | "prepend" | "append"))
        {
            v.error(
                format!("{path}.strategy"),
                "must be one of \"replace\", \"prepend\", \"append\"",
            );
        }
    }
}

fn validate_ignore(v: &mut V, value: &Value, path: &str) {
    let Some(i) = value.as_object() else {
        v.error(path, "must be a mapping with optional folders/files lists");
        return;
    };
    v.unknown_keys(i, &["folders", "files"], path);
    for key in ["folders", "files"] {
        if let Some(x) = i.get(key) {
            if !is_string_array(x) {
                v.error(format!("{path}.{key}"), "must be a list of strings");
            }
        }
    }
}

fn validate_providers(v: &mut V, value: &Value, path: &str) {
    let Some(map) = value.as_object() else {
        v.error(path, "must be a mapping of provider namespace to config");
        return;
    };
    for (ns, cfg) in map {
        let pp = format!("{path}.{ns}");
        let Some(c) = cfg.as_object() else {
            v.error(pp, "provider config must be a mapping");
            continue;
        };
        v.unknown_keys(c, &["rules"], &pp);
        if let Some(r) = c.get("rules") {
            validate_rules(v, r, &format!("{pp}.rules"));
        }
    }
}

/// Code checks are additive: a malformed check is a warning, not an error.
fn validate_code_checks(v: &mut V, value: &Value, path: &str) {
    let Some(map) = value.as_object() else {
        v.error(path, "must be a mapping of check id to check definition");
        return;
    };
    for (id, check) in map {
        let cp = format!("{path}.{id}");
        let Some(c) = check.as_object() else {
            v.warn(format!("\"{cp}\" skipped: must be a mapping"));
            continue;
        };
        v.unknown_keys(c, CODE_CHECK_KEYS, &cp);
        if c.get("pattern").is_some_and(|p| !p.is_string()) {
            v.warn(format!("\"{cp}\" skipped: pattern must be a string"));
        }
        if c.get("languages").is_some_and(|p| !is_string_array(p)) {
            v.warn(format!(
                "\"{cp}\" skipped: languages must be a list of strings"
            ));
        }
        if c.get("paths").is_some_and(|p| !is_string_array(p)) {
            v.warn(format!("\"{cp}\" skipped: paths must be a list of strings"));
        }
    }
}

/// The base note-type's required keys bound every scoped schema's required keys.
fn validate_governance(raw: &Obj) -> Vec<ConfigError> {
    let mut errors = Vec::new();
    let Some(types) = raw.get("note-types").and_then(Value::as_object) else {
        return errors;
    };
    let Some(base_req) = types
        .get("base")
        .and_then(Value::as_object)
        .and_then(|b| b.get("required"))
        .and_then(Value::as_object)
    else {
        return errors;
    };
    for (name, schema) in types {
        if name == "base" {
            continue;
        }
        let Some(req) = schema
            .as_object()
            .and_then(|s| s.get("required"))
            .and_then(Value::as_object)
        else {
            continue;
        };
        for key in req.keys() {
            if !base_req.contains_key(key) {
                errors.push(ConfigError {
                    path: format!("note-types.{name}.required.{key}"),
                    message: "required key not present in the base note-type's required set; a scoped schema may only drop or reorder base-required keys, never introduce new ones".to_owned(),
                });
            }
        }
    }
    errors
}

pub fn validate(raw: Value) -> LoadResult {
    let mut v = V::default();
    let Some(obj) = raw.as_object() else {
        v.error("", "config must be a yaml mapping");
        return LoadResult::Err(v.errors);
    };
    v.unknown_keys_top(obj);
    if obj.get("version").and_then(Value::as_f64) != Some(1.0) {
        v.error("version", "must be 1");
    }
    if let Some(d) = obj.get("defaults") {
        match d.as_object() {
            None => v.error("defaults", "must be a mapping"),
            Some(d) => {
                for k in d.keys() {
                    if k != "rules" {
                        v.warn(format!("unknown key \"defaults.{k}\" ignored"));
                    }
                }
                if let Some(r) = d.get("rules") {
                    validate_rules(&mut v, r, "defaults.rules");
                }
            }
        }
    }
    if let Some(ro) = obj.get("rule-order") {
        if !is_string_array(ro) {
            v.error("rule-order", "must be a list of rule ids");
        }
    }
    if let Some(p) = obj.get("profiles") {
        validate_profiles(&mut v, p, "profiles");
    }
    if let Some(nt) = obj.get("note-types") {
        validate_note_types(&mut v, nt, "note-types");
        for e in validate_governance(obj) {
            v.error(e.path, e.message);
        }
    }
    if let Some(t) = obj.get("templates") {
        validate_templates(&mut v, t, "templates");
    }
    if let Some(a) = obj.get("automations") {
        validate_automations(&mut v, a, "automations");
    }
    if let Some(r) = obj.get("rename") {
        validate_rename(&mut v, r, "rename");
    }
    if let Some(i) = obj.get("ignore") {
        validate_ignore(&mut v, i, "ignore");
    }
    if let Some(p) = obj.get("providers") {
        validate_providers(&mut v, p, "providers");
    }
    if let Some(c) = obj.get("code-checks") {
        validate_code_checks(&mut v, c, "code-checks");
    }
    if !v.errors.is_empty() {
        return LoadResult::Err(v.errors);
    }
    LoadResult::Ok {
        config: Box::new(build(&raw)),
        warnings: v.warnings,
    }
}

impl V {
    fn unknown_keys_top(&mut self, obj: &Obj) {
        for k in obj.keys() {
            if !TOP_LEVEL_KEYS.contains(&k.as_str()) {
                self.warn(format!("unknown key \"{k}\" ignored"));
            }
        }
    }
}

fn strings(v: Option<&Value>) -> Vec<String> {
    v.and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|s| s.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

fn range(v: Option<&Value>) -> Option<DateRange> {
    let o = v?.as_object()?;
    Some(DateRange {
        after: o.get("after").and_then(Value::as_str).map(str::to_owned),
        before: o.get("before").and_then(Value::as_str).map(str::to_owned),
    })
}

fn build_match(v: &Value) -> ProfileMatch {
    let Some(m) = v.as_object() else {
        return ProfileMatch::default();
    };
    ProfileMatch {
        path: strings(m.get("path")),
        extension: strings(m.get("extension")),
        frontmatter: m
            .get("frontmatter")
            .and_then(Value::as_object)
            .map(|o| o.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default(),
        tag: strings(m.get("tag")),
        age: strings(m.get("age")),
        date_created: range(m.get("date-created")),
        date_revised: range(m.get("date-revised")),
        backlink: strings(m.get("backlink")),
        alias: strings(m.get("alias")),
    }
}

fn rules_of(v: Option<&Value>) -> Option<RulesConfig> {
    v.and_then(Value::as_object).cloned()
}

fn build(raw: &Value) -> Config {
    let obj = raw.as_object().cloned().unwrap_or_default();
    let profiles = obj
        .get("profiles")
        .and_then(Value::as_object)
        .map(|m| {
            m.iter()
                .map(|(name, p)| Profile {
                    name: name.clone(),
                    match_: p.get("match").map(build_match),
                    rules: rules_of(p.get("rules")),
                    rule_order: p.get("rule-order").map(|_| strings(p.get("rule-order"))),
                })
                .collect()
        })
        .unwrap_or_default();
    let note_types = obj
        .get("note-types")
        .and_then(Value::as_object)
        .map(|m| {
            m.iter()
                .map(|(name, s)| NoteType {
                    name: name.clone(),
                    match_: s.get("match").map(build_match),
                })
                .collect()
        })
        .unwrap_or_default();
    Config {
        defaults_rules: rules_of(obj.get("defaults").and_then(|d| d.get("rules"))),
        rule_order: obj
            .get("rule-order")
            .map(|_| strings(obj.get("rule-order"))),
        profiles,
        note_types,
        raw: raw.clone(),
    }
}
