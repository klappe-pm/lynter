//! Runs the native-tool adapters (ruff, Biome) for one file and turns their
//! outcome into what the CLI reports.

use crate::cfg::Loaded;
use lynter_code::adapters::{adapter_for_extension, resolve_settings, Adapter, AdapterViolation};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// What the adapters did for one file.
#[derive(Default)]
pub struct FileRun {
    /// An adapter was enabled, found, and run, so the file is reported even
    /// when the code host had no rule for it.
    pub ran: bool,
    pub violations: Vec<AdapterViolation>,
    /// Crash or unparseable output, reported as one `adapter-error`.
    pub error: Option<String>,
}

fn language_of(adapter: &dyn Adapter) -> &'static str {
    match adapter.id() {
        "ruff" => "Python",
        _ => "TypeScript and JavaScript",
    }
}

/// Run the adapter for `file` when one handles it and config enables it.
/// `chain` is the resolved profile chain for the file. A missing binary prints
/// one notice per adapter per run to stderr and skips the file.
pub fn run_file(
    file: &Path,
    rel: &str,
    loaded: &Loaded,
    chain: &[String],
    fix: bool,
    notified: &mut HashSet<&'static str>,
) -> FileRun {
    let Some(ext) = file
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_lowercase)
    else {
        return FileRun::default();
    };
    let Some(adapter) = adapter_for_extension(&ext) else {
        return FileRun::default();
    };
    let mut settings = match resolve_settings(&loaded.config.raw, chain).remove(adapter.id()) {
        Some(s) if s.enabled => s,
        _ => return FileRun::default(),
    };
    if let Some(p) = &settings.path {
        let p = PathBuf::from(p);
        if p.is_relative() {
            settings.path = Some(loaded.dir.join(p).to_string_lossy().into_owned());
        }
    }
    let Some(binary) = adapter.detect(&settings) else {
        if notified.insert(adapter.id()) {
            eprintln!(
                "lynter: notice: {} not found, skipping {} adapter checks (install it, or set adapters.{}.path)",
                adapter.binary_name(),
                language_of(adapter.as_ref()),
                adapter.id()
            );
        }
        return FileRun::default();
    };
    match adapter.run(&binary, &[file.to_path_buf()], &settings, fix) {
        Ok(mut violations) => {
            for v in &mut violations {
                v.file = rel.to_owned();
            }
            FileRun {
                ran: true,
                violations,
                error: None,
            }
        }
        Err(e) => FileRun {
            ran: true,
            violations: Vec::new(),
            error: Some(e.0),
        },
    }
}
