//! Native-tool adapters: ruff for Python and Biome for TypeScript and
//! JavaScript. They shell out to the installed tool, so they exist only in
//! native builds (the CLI and CI), never in WASM, and every adapter is
//! optional: a missing binary is a notice, not a failure.

mod biome;
mod config;
mod ruff;

pub use biome::Biome;
pub use config::{resolve_settings, validate_config, AdapterSettings};
pub use ruff::Ruff;

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Rule id of the single violation reported when an adapter crashes or emits
/// output Lynter cannot parse.
pub const ERROR_RULE: &str = "adapter-error";

/// One finding from a native tool, already in Lynter's shape.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdapterViolation {
    /// Prefixed rule id, `ruff/F401` or `biome/lint/suspicious/noDebugger`.
    pub rule: String,
    pub file: String,
    /// 1-based line, 0 when the tool gave none.
    pub line: usize,
    /// 1-based column, 0 when the tool gave none.
    pub column: usize,
    pub message: String,
    pub fixable: bool,
}

/// The tool crashed, was killed, or printed output that is not its documented
/// JSON.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdapterError(pub String);

impl std::fmt::Display for AdapterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// The raw result of one tool process.
pub struct Output {
    pub stdout: String,
    pub stderr: String,
    pub code: Option<i32>,
}

pub trait Adapter {
    /// Short id: `ruff` or `biome`. Also the rule id prefix and the key under
    /// `adapters:` in lynter.yaml.
    fn id(&self) -> &'static str;

    /// Lowercase file extensions this adapter lints.
    fn extensions(&self) -> &'static [&'static str];

    /// Executable name looked up on PATH.
    fn binary_name(&self) -> &'static str;

    /// Whether `fix` mode is available.
    fn supports_fix(&self) -> bool {
        true
    }

    /// Arguments placed before the file list.
    fn command_args(&self, settings: &AdapterSettings, fix: bool) -> Vec<String>;

    /// Parse the tool's stdout into Lynter violations. `files` are the paths
    /// the tool was run on, used to attribute findings and to map the tool's
    /// own file names back.
    fn parse(&self, stdout: &str, files: &[PathBuf])
        -> Result<Vec<AdapterViolation>, AdapterError>;

    /// Whether the process outcome is a crash rather than "found problems".
    fn is_crash(&self, out: &Output) -> bool;

    fn handles(&self, ext: &str) -> bool {
        self.extensions().contains(&ext)
    }

    /// The binary to run: an explicit `path` from config when it exists, else
    /// the first match on PATH. `None` means the tool is not installed.
    fn detect(&self, settings: &AdapterSettings) -> Option<PathBuf> {
        match &settings.path {
            Some(p) => {
                let p = PathBuf::from(p);
                p.is_file().then_some(p)
            }
            None => which(self.binary_name()),
        }
    }

    /// Run the tool on `files` and parse its output. In fix mode the tool
    /// rewrites the files; the result lists what remains.
    fn run(
        &self,
        binary: &Path,
        files: &[PathBuf],
        settings: &AdapterSettings,
        fix: bool,
    ) -> Result<Vec<AdapterViolation>, AdapterError> {
        let mut cmd = Command::new(binary);
        cmd.args(self.command_args(settings, fix && self.supports_fix()))
            .args(files)
            .stdin(Stdio::null());
        let o = cmd.output().map_err(|e| {
            AdapterError(format!(
                "{}: cannot run {}: {e}",
                self.id(),
                binary.display()
            ))
        })?;
        let out = Output {
            stdout: String::from_utf8_lossy(&o.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&o.stderr).into_owned(),
            code: o.status.code(),
        };
        if self.is_crash(&out) {
            return Err(AdapterError(format!(
                "{} failed ({}): {}",
                self.id(),
                out.code
                    .map_or("killed by signal".to_owned(), |c| format!("exit {c}")),
                first_line(&out.stderr, &out.stdout)
            )));
        }
        self.parse(&out.stdout, files)
    }
}

fn first_line(a: &str, b: &str) -> String {
    let pick = if a.trim().is_empty() { b } else { a };
    pick.lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("no output")
        .chars()
        .take(300)
        .collect()
}

/// First executable named `name` on PATH.
pub fn which(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    let names: Vec<String> = if cfg!(windows) {
        vec![
            format!("{name}.exe"),
            format!("{name}.cmd"),
            name.to_owned(),
        ]
    } else {
        vec![name.to_owned()]
    };
    std::env::split_paths(&path)
        .flat_map(|d| names.iter().map(move |n| d.join(n)))
        .find(|p| p.is_file())
}

/// Every built-in adapter.
pub fn registry() -> Vec<Box<dyn Adapter>> {
    vec![Box::new(Ruff), Box::new(Biome)]
}

/// The adapter that lints files with this lowercase extension.
pub fn adapter_for_extension(ext: &str) -> Option<Box<dyn Adapter>> {
    registry().into_iter().find(|a| a.handles(ext))
}

/// 1-based line and column of byte offset `at` in `text`.
pub(crate) fn line_col_at(text: &str, at: usize) -> (usize, usize) {
    let at = at.min(text.len());
    let before = &text.as_bytes()[..at];
    let line = before.iter().filter(|&&b| b == b'\n').count() + 1;
    let col = at
        - before
            .iter()
            .rposition(|&b| b == b'\n')
            .map_or(0, |i| i + 1)
        + 1;
    (line, col)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_routes_extensions() {
        assert_eq!(adapter_for_extension("py").unwrap().id(), "ruff");
        assert_eq!(adapter_for_extension("tsx").unwrap().id(), "biome");
        assert_eq!(adapter_for_extension("mjs").unwrap().id(), "biome");
        assert!(adapter_for_extension("rs").is_none());
    }

    #[test]
    fn missing_explicit_path_is_not_detected() {
        let s = AdapterSettings {
            enabled: true,
            path: Some("/nonexistent/ruff".to_owned()),
            ..AdapterSettings::default()
        };
        assert!(Ruff.detect(&s).is_none());
    }

    #[test]
    fn line_col_counts_from_one() {
        assert_eq!(line_col_at("ab\ncd", 4), (2, 2));
        assert_eq!(line_col_at("ab", 0), (1, 1));
    }
}
