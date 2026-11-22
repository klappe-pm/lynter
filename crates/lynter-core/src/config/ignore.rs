//! The `ignore` block: folders and files Lynter never touches. Every host
//! asks this before linting, so a path ignored in the command line is ignored
//! in the plugin and the browser too.

use super::Config;
use serde_json::Value;

fn list(config: &Config, key: &str) -> Vec<String> {
    config
        .raw
        .get("ignore")
        .and_then(|i| i.get(key))
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|s| s.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

/// True when `rel`, a path relative to the config's directory with `/`
/// separators, is under an ignored folder or is an ignored file.
pub fn is_ignored(config: &Config, rel: &str) -> bool {
    for folder in list(config, "folders") {
        let n = folder.trim_end_matches('/');
        if !n.is_empty() && (rel == n || rel.starts_with(&format!("{n}/"))) {
            return true;
        }
    }
    list(config, "files").iter().any(|f| f == rel)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{parse_config, LoadResult};

    #[test]
    fn folders_match_by_prefix_and_files_exactly() {
        let LoadResult::Ok { config, .. } = parse_config(
            "version: 1\nignore:\n  folders: [templates, archive/]\n  files: [inbox/scratch.md]\n",
        ) else {
            panic!("config must load");
        };
        assert!(is_ignored(&config, "templates/daily.md"));
        assert!(is_ignored(&config, "archive/2020/a.md"));
        assert!(is_ignored(&config, "inbox/scratch.md"));
        assert!(!is_ignored(&config, "templates-old/a.md"));
        assert!(!is_ignored(&config, "inbox/other.md"));
        assert!(!is_ignored(&config, "notes/a.md"));
    }
}
