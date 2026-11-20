use super::{parse_config, ConfigError, LoadResult};
use std::path::{Path, PathBuf};

pub const CANONICAL_CONFIG_FILENAME: &str = "lynter.yaml";

/// Accepted filenames in discovery order; the first two are deprecated.
pub const CONFIG_FILENAMES: [&str; 3] = [
    CANONICAL_CONFIG_FILENAME,
    "linter.yaml",
    "lappe-linter.yaml",
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Discovered {
    pub path: PathBuf,
    /// One deprecation notice when a deprecated name was found.
    pub notice: Option<String>,
}

/// Look for a config in `dir`: `lynter.yaml`, then `linter.yaml`, then
/// `lappe-linter.yaml`. The first one found wins, with one notice when it is
/// a deprecated name.
pub fn discover_config(dir: &Path) -> Option<Discovered> {
    for name in CONFIG_FILENAMES {
        let path = dir.join(name);
        if path.is_file() {
            let notice = (name != CANONICAL_CONFIG_FILENAME)
                .then(|| format!("{name} is deprecated; rename it to {CANONICAL_CONFIG_FILENAME}"));
            return Some(Discovered { path, notice });
        }
    }
    None
}

/// Read and parse a config file. An unreadable file fails closed.
pub fn load_config_file(path: &Path) -> LoadResult {
    match std::fs::read_to_string(path) {
        Ok(text) => parse_config(&text),
        Err(e) => LoadResult::Err(vec![ConfigError {
            path: String::new(),
            message: format!("cannot read {}: {e}", path.display()),
        }]),
    }
}
