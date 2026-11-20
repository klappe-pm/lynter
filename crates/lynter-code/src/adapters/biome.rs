//! Biome adapter for TypeScript and JavaScript: `biome lint --reporter=json`.
//!
//! The JSON reporter is experimental, so parsing is defensive: the document is
//! located by its first `{` (a banner may precede it), line and column come
//! from `location.start` when present and from the byte `span` otherwise, and
//! entries missing a category fall back to `biome/unknown`.

use super::{line_col_at, Adapter, AdapterError, AdapterSettings, AdapterViolation, Output};
use serde_json::Value;
use std::path::PathBuf;

pub struct Biome;

impl Adapter for Biome {
    fn id(&self) -> &'static str {
        "biome"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["ts", "tsx", "mts", "cts", "js", "jsx", "mjs", "cjs"]
    }

    fn binary_name(&self) -> &'static str {
        "biome"
    }

    fn command_args(&self, settings: &AdapterSettings, fix: bool) -> Vec<String> {
        let mut a: Vec<String> = [
            "lint",
            "--reporter=json",
            "--colors=off",
            "--no-errors-on-unmatched",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect();
        if fix {
            a.push("--write".to_owned());
        }
        a.extend(settings.args.iter().cloned());
        a
    }

    /// Biome exits non-zero when it reports errors, with the JSON still on
    /// stdout, so only a missing document marks a crash (checked in `parse`).
    fn is_crash(&self, out: &Output) -> bool {
        out.code.is_none() || (out.code != Some(0) && !out.stdout.contains('{'))
    }

    fn parse(
        &self,
        stdout: &str,
        files: &[PathBuf],
    ) -> Result<Vec<AdapterViolation>, AdapterError> {
        parse(stdout, files)
    }
}

fn num(v: Option<&Value>, key: &str) -> Option<usize> {
    v.and_then(|l| l.get(key))
        .and_then(Value::as_u64)
        .map(|n| n as usize)
}

fn is_fixable(d: &Value) -> bool {
    let tagged = d
        .get("tags")
        .and_then(Value::as_array)
        .is_some_and(|t| t.iter().any(|x| x.as_str() == Some("fixable")));
    let advised = d.get("advices").and_then(Value::as_array).is_some_and(|a| {
        a.iter().any(|x| {
            x.get("text")
                .and_then(Value::as_str)
                .is_some_and(|t| t.to_lowercase().contains("fix:"))
        })
    });
    tagged || advised
}

/// Message text: a plain string (Biome 2) or a list of `{content}` pieces
/// (Biome 1).
fn message_of(d: &Value) -> Option<String> {
    match d.get("message")? {
        Value::String(s) => Some(s.clone()),
        Value::Array(parts) => Some(
            parts
                .iter()
                .filter_map(|p| p.get("content").and_then(Value::as_str))
                .collect(),
        ),
        _ => None,
    }
}

pub fn parse(stdout: &str, files: &[PathBuf]) -> Result<Vec<AdapterViolation>, AdapterError> {
    let bad = |why: &str| AdapterError(format!("biome: unparseable output: {why}"));
    let start = stdout.find('{').ok_or_else(|| bad("no JSON document"))?;
    let mut de = serde_json::Deserializer::from_str(&stdout[start..]).into_iter::<Value>();
    let doc = de
        .next()
        .ok_or_else(|| bad("no JSON document"))?
        .map_err(|e| bad(&e.to_string()))?;
    let diagnostics = doc
        .get("diagnostics")
        .and_then(Value::as_array)
        .ok_or_else(|| bad("no diagnostics array"))?;
    let mut out = Vec::new();
    for d in diagnostics {
        let loc = d.get("location");
        let path = loc
            .and_then(|l| l.get("path"))
            .and_then(|p| p.as_str().or_else(|| p.get("file").and_then(Value::as_str)));
        let file = files
            .first()
            .map(|f| f.display().to_string())
            .or_else(|| path.map(str::to_owned))
            .unwrap_or_default();
        let (mut line, mut column) = (0, 0);
        if let Some(s) = loc.and_then(|l| l.get("start")) {
            line = num(Some(s), "line").unwrap_or(0);
            column = num(Some(s), "column").unwrap_or(0);
        } else if let Some(at) = loc
            .and_then(|l| l.get("span"))
            .and_then(Value::as_array)
            .and_then(|s| s.first())
            .and_then(Value::as_u64)
        {
            let src = loc
                .and_then(|l| l.get("sourceCode"))
                .and_then(Value::as_str)
                .map(str::to_owned)
                .or_else(|| files.first().and_then(|f| std::fs::read_to_string(f).ok()));
            if let Some(src) = src {
                (line, column) = line_col_at(&src, at as usize);
            }
        }
        out.push(AdapterViolation {
            rule: format!(
                "biome/{}",
                d.get("category")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown")
            ),
            file,
            line,
            column,
            message: message_of(d).unwrap_or_else(|| "(no message)".to_owned()),
            fixable: is_fixable(d),
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = include_str!("fixtures/biome.json");
    const SYNTAX: &str = include_str!("fixtures/biome-syntax.json");

    #[test]
    fn parses_recorded_output() {
        let v = parse(SAMPLE, &[]).unwrap();
        assert_eq!(v.len(), 3);
        assert_eq!(v[0].rule, "biome/lint/complexity/useLiteralKeys");
        assert_eq!((v[0].line, v[0].column), (2, 15));
        assert!(!v[0].fixable);
        assert_eq!(v[1].rule, "biome/lint/style/useConst");
        assert!(v[1].fixable);
        assert_eq!(
            v[1].message,
            "This let declares a variable that is only assigned once."
        );
    }

    #[test]
    fn parse_errors_use_the_parse_category() {
        let v = parse(SYNTAX, &[]).unwrap();
        assert_eq!(v.len(), 2);
        assert!(v.iter().all(|x| x.rule == "biome/parse"));
    }

    #[test]
    fn tolerates_a_banner_and_the_older_shape() {
        let old = r#"warning text
{"diagnostics":[{"category":"lint/suspicious/noDoubleEquals","severity":"error",
"message":[{"elements":[],"content":"Use "},{"elements":[],"content":"==="}],
"tags":["fixable"],
"location":{"path":{"file":"x.js"},"span":[4,6],"sourceCode":"a\nb == c"}}]}"#;
        let v = parse(old, &[]).unwrap();
        assert_eq!(v[0].message, "Use ===");
        assert!(v[0].fixable);
        assert_eq!((v[0].line, v[0].column), (2, 3));
        assert_eq!(v[0].file, "x.js");
    }

    #[test]
    fn garbage_is_an_error() {
        assert!(parse("", &[]).is_err());
        assert!(parse("{broken", &[]).is_err());
        assert!(parse("{\"summary\":{}}", &[]).is_err());
    }

    #[test]
    fn builds_the_command() {
        let s = AdapterSettings {
            enabled: true,
            args: vec!["--max-diagnostics=5".into()],
            ..AdapterSettings::default()
        };
        let a = Biome.command_args(&s, true);
        assert_eq!(&a[..2], ["lint", "--reporter=json"]);
        assert!(a.contains(&"--write".to_owned()));
        assert!(!Biome
            .command_args(&s, false)
            .contains(&"--write".to_owned()));
    }
}
