//! ruff adapter for Python: `ruff check --output-format json`.

use super::{Adapter, AdapterError, AdapterSettings, AdapterViolation, Output};
use serde_json::Value;
use std::path::PathBuf;

pub struct Ruff;

impl Adapter for Ruff {
    fn id(&self) -> &'static str {
        "ruff"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["py", "pyi"]
    }

    fn binary_name(&self) -> &'static str {
        "ruff"
    }

    fn command_args(&self, settings: &AdapterSettings, fix: bool) -> Vec<String> {
        let mut a: Vec<String> = ["check", "--output-format", "json", "--no-cache"]
            .into_iter()
            .map(str::to_owned)
            .collect();
        if !settings.select.is_empty() {
            a.push("--select".to_owned());
            a.push(settings.select.join(","));
        }
        if fix {
            a.push("--fix".to_owned());
        }
        a.extend(settings.args.iter().cloned());
        a
    }

    /// ruff exits 0 clean and 1 with findings; 2 is an abnormal exit.
    fn is_crash(&self, out: &Output) -> bool {
        !matches!(out.code, Some(0 | 1))
    }

    fn parse(
        &self,
        stdout: &str,
        files: &[PathBuf],
    ) -> Result<Vec<AdapterViolation>, AdapterError> {
        parse(stdout, files)
    }
}

fn pos(v: Option<&Value>, key: &str) -> usize {
    v.and_then(|l| l.get(key))
        .and_then(Value::as_u64)
        .map_or(0, |n| n as usize)
}

/// Parse ruff's JSON array. Findings are attributed to the file ruff names,
/// falling back to the only file when ruff names none.
pub fn parse(stdout: &str, files: &[PathBuf]) -> Result<Vec<AdapterViolation>, AdapterError> {
    let bad = |why: &str| AdapterError(format!("ruff: unparseable output: {why}"));
    let value: Value = serde_json::from_str(stdout.trim()).map_err(|e| bad(&e.to_string()))?;
    let items = value
        .as_array()
        .ok_or_else(|| bad("expected a JSON array"))?;
    let mut out = Vec::new();
    for item in items {
        let Some(obj) = item.as_object() else {
            return Err(bad("array entry is not an object"));
        };
        let code = obj
            .get("code")
            .and_then(Value::as_str)
            .or_else(|| obj.get("name").and_then(Value::as_str))
            .unwrap_or("unknown");
        let message = obj
            .get("message")
            .and_then(Value::as_str)
            .ok_or_else(|| bad("entry has no message"))?;
        let file = obj
            .get("filename")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .or_else(|| files.first().map(|f| f.display().to_string()))
            .unwrap_or_default();
        let loc = obj.get("location");
        out.push(AdapterViolation {
            rule: format!("ruff/{code}"),
            file,
            line: pos(loc, "row"),
            column: pos(loc, "column"),
            message: message.to_owned(),
            fixable: obj.get("fix").is_some_and(|f| f.is_object()),
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = include_str!("fixtures/ruff.json");
    const SYNTAX: &str = include_str!("fixtures/ruff-syntax.json");

    #[test]
    fn parses_recorded_output() {
        let v = parse(SAMPLE, &[]).unwrap();
        assert_eq!(v.len(), 2);
        assert_eq!(v[1].rule, "ruff/E711");
        let v = &v[..1];
        assert_eq!(v[0].rule, "ruff/F401");
        assert_eq!(v[0].file, "/work/a.py");
        assert_eq!((v[0].line, v[0].column), (1, 8));
        assert_eq!(v[0].message, "`os` imported but unused");
        assert!(v[0].fixable);
    }

    #[test]
    fn syntax_errors_are_not_fixable() {
        let v = parse(SYNTAX, &[]).unwrap();
        assert_eq!(v.len(), 2);
        assert!(v
            .iter()
            .all(|x| x.rule == "ruff/invalid-syntax" && !x.fixable));
    }

    #[test]
    fn empty_array_is_clean() {
        assert!(parse("[]\n", &[]).unwrap().is_empty());
    }

    #[test]
    fn garbage_is_an_error() {
        assert!(parse("not json", &[]).is_err());
        assert!(parse("{\"a\":1}", &[]).is_err());
        assert!(parse("[1]", &[]).is_err());
        assert!(parse("[{\"code\":\"F1\"}]", &[]).is_err());
    }

    #[test]
    fn builds_the_command() {
        let s = AdapterSettings {
            enabled: true,
            select: vec!["F".into(), "E".into()],
            args: vec!["--preview".into()],
            ..AdapterSettings::default()
        };
        let a = Ruff.command_args(&s, true);
        assert_eq!(&a[..4], ["check", "--output-format", "json", "--no-cache"]);
        assert!(a.windows(2).any(|w| w == ["--select", "F,E"]));
        assert!(a.contains(&"--fix".to_owned()));
        assert_eq!(a.last().unwrap(), "--preview");
        assert!(!Ruff.command_args(&s, false).contains(&"--fix".to_owned()));
    }
}
