use super::casing::CaseStyle;
use super::{opt_bool, opt_list, opt_str, opt_usize, Reports, FILE_PATH_OPTION};
use crate::document::Document;
use crate::edit::Edit;
use crate::rule::{Options, Rule};
use serde_json::json;

/// Report folder path segments that break a case style, depth, or name list.
pub struct FolderNaming;

impl Rule for FolderNaming {
    fn id(&self) -> &'static str {
        "folder-naming"
    }

    fn description(&self) -> &'static str {
        "Report folder names in the file path that break a case style, the maximum depth, or the allowed names. Report only; needs the file-path option."
    }

    fn default_options(&self) -> Options {
        json!({"case": "none", "max-depth": 0, "allowed-names": [], "ignore-hidden": true})
            .as_object()
            .cloned()
            .unwrap_or_default()
    }

    fn check(&self, doc: &Document, options: &Options) -> Vec<Edit> {
        let Some(path) = opt_str(options, FILE_PATH_OPTION) else {
            return Vec::new();
        };
        let case = opt_str(options, "case").and_then(CaseStyle::parse);
        let max_depth = opt_usize(options, "max-depth");
        let allowed = opt_list(options, "allowed-names");
        let ignore_hidden = opt_bool(options, "ignore-hidden", true);
        let norm = path.replace('\\', "/");
        let mut segs: Vec<&str> = norm
            .split('/')
            .filter(|s| !s.is_empty() && *s != ".")
            .collect();
        segs.pop();
        if ignore_hidden {
            if let Some(i) = segs.iter().position(|s| s.starts_with('.')) {
                segs.truncate(i);
            }
        }
        let mut reports = Reports::default();
        if max_depth > 0 && segs.len() > max_depth {
            reports.push(
                0,
                0,
                format!(
                    "folder depth {} exceeds the maximum of {max_depth}",
                    segs.len()
                ),
            );
        }
        for s in &segs {
            if !allowed.is_empty() {
                if !allowed.iter().any(|a| a == s) {
                    reports.push(0, 0, format!("folder \"{s}\" is not in the allowed names"));
                }
                continue;
            }
            if let Some(style) = case {
                if !style.conforms(s) {
                    reports.push(
                        0,
                        0,
                        format!(
                            "folder \"{s}\" is not {} case, expected \"{}\"",
                            style.name(),
                            style.convert(s)
                        ),
                    );
                }
            }
        }
        reports.into_edits(doc.text())
    }
}
