//! `lynter.yaml` loading: discovery, parsing, and fail-closed validation.

mod defaults;
mod discovery;
mod ignore;
mod migrate;
mod scaffold;
mod types;
mod unknown;
mod validate;

pub use defaults::{
    base_note_type, default_config, default_rules, DEFAULT_PRIORITY_KEYS, OPT_IN_RULES,
};
pub use discovery::{
    discover_config, load_config_file, Discovered, CANONICAL_CONFIG_FILENAME, CONFIG_FILENAMES,
};
pub use ignore::is_ignored;
pub use migrate::migrate_data_json;
pub use scaffold::scaffold;
pub use types::{
    Config, ConfigError, DateRange, LoadResult, NoteType, Profile, ProfileMatch, RulesConfig,
};
pub use unknown::{known_rule_ids, unknown_rules, UnknownRule, CODE_RULE_IDS};

use serde_json::{Map, Value};

/// Parse and validate a config text. Fail closed: any structural error yields
/// `LoadResult::Err` with every error found, never a partial config. Unknown
/// keys are warnings.
pub fn parse_config(yaml_text: &str) -> LoadResult {
    let raw = match yaml_to_json(yaml_text) {
        Ok(v) => v,
        Err(message) => {
            return LoadResult::Err(vec![ConfigError {
                path: String::new(),
                message: format!("yaml parse error: {message}"),
            }])
        }
    };
    validate::validate(raw)
}

/// Parse YAML text into a JSON value tree (mapping keys become strings).
pub fn yaml_to_json(text: &str) -> Result<Value, String> {
    if text.trim().is_empty() {
        return Ok(Value::Null);
    }
    let v: serde_yaml_ng::Value = serde_yaml_ng::from_str(text).map_err(|e| e.to_string())?;
    convert(v)
}

fn convert(v: serde_yaml_ng::Value) -> Result<Value, String> {
    use serde_yaml_ng::Value as Y;
    Ok(match v {
        Y::Null => Value::Null,
        Y::Bool(b) => Value::Bool(b),
        Y::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::from(i)
            } else if let Some(u) = n.as_u64() {
                Value::from(u)
            } else {
                n.as_f64()
                    .and_then(serde_json::Number::from_f64)
                    .map_or(Value::Null, Value::Number)
            }
        }
        Y::String(s) => Value::String(s),
        Y::Sequence(seq) => Value::Array(seq.into_iter().map(convert).collect::<Result<_, _>>()?),
        Y::Mapping(map) => {
            let mut out = Map::new();
            for (k, val) in map {
                let key = match k {
                    Y::String(s) => s,
                    Y::Number(n) => n.to_string(),
                    Y::Bool(b) => b.to_string(),
                    Y::Null => "null".to_owned(),
                    _ => return Err("unsupported mapping key".to_owned()),
                };
                out.insert(key, convert(val)?);
            }
            Value::Object(out)
        }
        Y::Tagged(t) => convert(t.value)?,
    })
}
