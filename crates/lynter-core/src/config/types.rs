use serde_json::{Map, Value};

/// `{rule-id: {enabled?, ...options}}`.
pub type RulesConfig = Map<String, Value>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigError {
    pub path: String,
    pub message: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DateRange {
    pub after: Option<String>,
    pub before: Option<String>,
}

/// A `match:` block. Empty lists count as absent, as in the TypeScript core.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProfileMatch {
    pub path: Vec<String>,
    pub extension: Vec<String>,
    pub frontmatter: Vec<(String, Value)>,
    pub tag: Vec<String>,
    pub age: Vec<String>,
    pub date_created: Option<DateRange>,
    pub date_revised: Option<DateRange>,
    pub backlink: Vec<String>,
    pub alias: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Profile {
    pub name: String,
    pub match_: Option<ProfileMatch>,
    pub rules: Option<RulesConfig>,
    pub rule_order: Option<Vec<String>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NoteType {
    pub name: String,
    pub match_: Option<ProfileMatch>,
}

/// A validated config. `raw` keeps the full document; the typed fields cover
/// what the scope engine and runner read.
#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    pub raw: Value,
    pub defaults_rules: Option<RulesConfig>,
    pub rule_order: Option<Vec<String>>,
    pub profiles: Vec<Profile>,
    pub note_types: Vec<NoteType>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LoadResult {
    Ok {
        config: Box<Config>,
        warnings: Vec<String>,
    },
    Err(Vec<ConfigError>),
}

impl LoadResult {
    pub fn is_ok(&self) -> bool {
        matches!(self, LoadResult::Ok { .. })
    }
}
