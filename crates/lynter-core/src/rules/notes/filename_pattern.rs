use super::{opt_str, valid_ymd, Reports, FILE_PATH_OPTION};
use crate::document::Document;
use crate::edit::Edit;
use crate::rule::{Options, Rule};
use fancy_regex::Regex;
use serde_json::json;
use std::sync::LazyLock;

static KEBAB: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$").expect("regex"));
static DATED: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\d{4})-(\d{2})-(\d{2})-.+$").expect("regex"));

fn stem(path: &str) -> Option<String> {
    let norm = path.replace('\\', "/");
    let name = norm.rsplit('/').next().filter(|n| !n.is_empty())?;
    Some(match name.rfind('.') {
        Some(i) if i > 0 => name[..i].to_owned(),
        _ => name.to_owned(),
    })
}

/// Report file names that do not match a regex or a preset.
pub struct FilenamePattern;

impl Rule for FilenamePattern {
    fn id(&self) -> &'static str {
        "filename-pattern"
    }

    fn description(&self) -> &'static str {
        "Report file names (without extension) that do not match a regex or a preset (kebab, dated). Report only; needs the file-path option."
    }

    fn default_options(&self) -> Options {
        json!({"preset": "none", "pattern": ""})
            .as_object()
            .cloned()
            .unwrap_or_default()
    }

    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit> {
        let Some(path) = opt_str(options, FILE_PATH_OPTION) else {
            return Vec::new();
        };
        let Some(stem) = stem(path) else {
            return Vec::new();
        };
        let mut reports = Reports::default();
        match opt_str(options, "preset").unwrap_or("none") {
            "kebab" if !KEBAB.is_match(&stem).unwrap_or(true) => {
                reports.push(0, 0, format!("file name \"{stem}\" is not kebab-case"));
            }
            "dated" => {
                let ok = DATED.captures(&stem).ok().flatten().is_some_and(|c| {
                    let n = |i| c.get(i).and_then(|m| m.as_str().parse::<u32>().ok());
                    matches!((n(1), n(2), n(3)), (Some(y), Some(m), Some(d)) if valid_ymd(y, m, d))
                });
                if !ok {
                    reports.push(
                        0,
                        0,
                        format!(
                            "file name \"{stem}\" does not start with a yyyy-mm-dd- date prefix"
                        ),
                    );
                }
            }
            _ => {}
        }
        let pattern = opt_str(options, "pattern").unwrap_or("");
        if !pattern.is_empty() {
            match Regex::new(pattern) {
                Ok(re) => {
                    if !re.is_match(&stem).unwrap_or(true) {
                        reports.push(
                            0,
                            0,
                            format!("file name \"{stem}\" does not match the pattern {pattern}"),
                        );
                    }
                }
                Err(_) => reports.push(0, 0, format!("the pattern {pattern} is not a valid regex")),
            }
        }
        reports.into_edits(doc.text())
    }
}
