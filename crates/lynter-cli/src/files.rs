//! Target expansion, ignore handling, and `--changed`.

use ignore::WalkBuilder;
use lynter_core::config::Config;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn is_lintable(p: &Path) -> bool {
    p.extension()
        .and_then(|e| e.to_str())
        .map(str::to_lowercase)
        .is_some_and(|e| matches!(e.as_str(), "md" | "txt") || lynter_code::handles_extension(&e))
}

pub fn expand(paths: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    let mut out: Vec<PathBuf> = Vec::new();
    for p in paths {
        let abs = std::path::absolute(p).map_err(|e| e.to_string())?;
        let meta = std::fs::metadata(&abs)
            .map_err(|_| format!("no such file or directory: {}", p.display()))?;
        if meta.is_dir() {
            let walk = WalkBuilder::new(&abs)
                .require_git(false)
                .sort_by_file_name(|a, b| a.cmp(b))
                .filter_entry(|e| e.file_name() != "node_modules")
                .build();
            for entry in walk.flatten() {
                if entry.file_type().is_some_and(|t| t.is_file()) && is_lintable(entry.path()) {
                    out.push(entry.into_path());
                }
            }
        } else if is_lintable(&abs) {
            out.push(abs);
        }
    }
    let mut seen = std::collections::HashSet::new();
    out.retain(|p| seen.insert(p.clone()));
    Ok(out)
}

/// Config-dir-relative path with forward slashes; absolute outside it.
pub fn vault_path(config_dir: &Path, file: &Path) -> String {
    let rel = file.strip_prefix(config_dir).unwrap_or(file);
    rel.components()
        .map(|c| c.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
        .replace("//", "/")
}

pub fn is_ignored(config: &Config, rel: &str) -> bool {
    lynter_core::config::is_ignored(config, rel)
}

fn git(args: &[&str], cwd: &Path) -> Result<String, String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| e.to_string())?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_owned());
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Markdown files changed against HEAD plus staged, that still exist.
pub fn changed(cwd: &Path) -> Result<Vec<PathBuf>, String> {
    let root = git(&["rev-parse", "--show-toplevel"], cwd)
        .map_err(|e| format!("--changed requires a git repository: {e}"))?;
    let root = PathBuf::from(root.trim());
    let mut names = std::collections::BTreeSet::new();
    if let Ok(o) = git(&["diff", "--name-only", "-z", "HEAD"], cwd) {
        names.extend(o.split('\0').filter(|s| !s.is_empty()).map(str::to_owned));
    }
    let staged = git(&["diff", "--name-only", "-z", "--cached"], cwd)
        .map_err(|e| format!("git diff failed under --changed: {e}"))?;
    names.extend(
        staged
            .split('\0')
            .filter(|s| !s.is_empty())
            .map(str::to_owned),
    );
    Ok(names
        .into_iter()
        .map(|n| root.join(n))
        .filter(|p| is_lintable(p) && p.exists())
        .collect())
}
