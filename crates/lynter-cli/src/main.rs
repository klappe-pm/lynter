//! The `lynter` command line.

mod adapt;
mod cfg;
mod files;
mod init;
mod lint;

use clap::{Parser, Subcommand};
use lynter_core::rules::get_rule;
use lynter_core::Mode;
use serde_json::json;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const OUTPUT_VERSION: u32 = 1;

#[derive(Parser)]
#[command(name = "lynter", version, about = "Lint and fix Markdown vaults")]
struct Cli {
    /// Use this config file instead of discovering one.
    #[arg(long, global = true, value_name = "PATH")]
    config: Option<PathBuf>,
    /// One JSON line per file (output-version 1).
    #[arg(long, global = true)]
    json: bool,
    /// Also target git-changed .md/.txt files (HEAD diff plus staged).
    #[arg(long, global = true)]
    changed: bool,
    /// ISO date (yyyy-MM-dd) used for age matching.
    #[arg(long, global = true, value_name = "DATE", value_parser = parse_date)]
    today: Option<String>,
    /// Skip the native-tool adapters (ruff, Biome) even when enabled in config.
    #[arg(long, global = true)]
    no_adapters: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Report violations, exit 1 if any.
    Check { paths: Vec<PathBuf> },
    /// Apply fixes and print changed files.
    Fix {
        paths: Vec<PathBuf>,
        /// Filter mode: read stdin, write the fixed text to stdout.
        #[arg(long)]
        stdin: bool,
        /// Config-dir-relative path used for scope resolution with --stdin.
        #[arg(long, value_name = "PATH")]
        stdin_path: Option<String>,
    },
    /// Show the governing config, profile chain, and rules for a file.
    Explain { path: PathBuf },
    /// Write a commented starter lynter.yaml to the current directory.
    Init,
}

fn parse_date(s: &str) -> Result<String, String> {
    let b = s.as_bytes();
    let ok = b.len() == 10
        && b.iter().enumerate().all(|(i, c)| {
            if i == 4 || i == 7 {
                *c == b'-'
            } else {
                c.is_ascii_digit()
            }
        });
    if ok {
        Ok(s.to_owned())
    } else {
        Err("must be an ISO date (yyyy-MM-dd)".to_owned())
    }
}

fn today_utc() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    let z = (secs / 86_400) as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}")
}

fn usage(msg: &str) -> ExitCode {
    eprintln!("lynter: {msg}");
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let today = cli.today.clone().unwrap_or_else(today_utc);
    match &cli.command {
        Command::Check { paths } => lint_paths(&cli, paths, Mode::Check, &today),
        Command::Fix {
            paths,
            stdin,
            stdin_path,
        } => {
            if *stdin {
                if !paths.is_empty() || cli.changed {
                    return usage("fix --stdin takes no paths and no --changed");
                }
                stdin_fix(&cli, stdin_path.as_deref(), &today)
            } else {
                lint_paths(&cli, paths, Mode::Fix, &today)
            }
        }
        Command::Explain { path } => explain(&cli, path, &today),
        Command::Init => init_cmd(),
    }
}

fn init_cmd() -> ExitCode {
    let target = Path::new("lynter.yaml");
    if target.exists() {
        return usage("lynter.yaml already exists; refusing to overwrite it");
    }
    match std::fs::write(target, init::STARTER) {
        Ok(()) => {
            println!("wrote lynter.yaml");
            ExitCode::SUCCESS
        }
        Err(e) => usage(&format!("cannot write lynter.yaml: {e}")),
    }
}

fn report_errors(errors: &[String]) {
    for e in errors {
        eprintln!("lynter: {e}");
    }
}

fn lint_paths(cli: &Cli, paths: &[PathBuf], mode: Mode, today: &str) -> ExitCode {
    if paths.is_empty() && !cli.changed {
        let cmd = if mode == Mode::Check { "check" } else { "fix" };
        return usage(&format!("{cmd} requires at least one path or --changed"));
    }
    let mut targets = match files::expand(paths) {
        Ok(f) => f,
        Err(e) => return usage(&e),
    };
    if cli.changed {
        let cwd = std::env::current_dir().unwrap_or_default();
        match files::changed(&cwd) {
            Ok(c) => {
                for f in c {
                    if !targets.contains(&f) {
                        targets.push(f);
                    }
                }
            }
            Err(e) => return usage(&e),
        }
    }
    let mut cache = cfg::Cache::new(cli.config.as_deref());
    let mut out = std::io::stdout().lock();
    let mut violations = 0usize;
    let mut adapter_failed = false;
    let mut notified = std::collections::HashSet::new();
    for file in &targets {
        let dir = file.parent().unwrap_or(Path::new("."));
        let loaded = match cache.for_dir(dir) {
            Ok(l) => l,
            Err(e) => {
                report_errors(&e);
                return ExitCode::from(2);
            }
        };
        let rel = files::vault_path(&loaded.dir, file);
        if files::is_ignored(&loaded.config, &rel) {
            continue;
        }
        let text = match std::fs::read(file) {
            Ok(b) => String::from_utf8_lossy(&b).into_owned(),
            Err(e) => return usage(&format!("cannot read {}: {e}", file.display())),
        };
        let r = lint::lint_any(&text, &rel, &loaded.resolver, &loaded.config, today);
        let fix = mode == Mode::Fix;
        if let Some(r) = r.as_ref().filter(|r| fix && r.changed) {
            // Adapters rewrite the file in place, so Lynter's own fixes land
            // first.
            if let Err(e) = std::fs::write(file, &r.text) {
                return usage(&format!("cannot write {}: {e}", file.display()));
            }
        }
        let mut adapter = adapt::FileRun::default();
        if !cli.no_adapters {
            let chain = match &r {
                Some(r) => r.chain.clone(),
                None => {
                    loaded
                        .resolver
                        .resolve(&lint::facts_for(&text, &rel, today))
                        .chain
                }
            };
            adapter = adapt::run_file(file, &rel, &loaded, &chain, fix, &mut notified);
        }
        if r.is_none() && !adapter.ran {
            continue;
        }
        let markdown = r.as_ref().is_some_and(|r| r.markdown);
        let own = r.as_ref().map_or(&[][..], |r| &r.violations[..]);
        violations += own.len() + adapter.violations.len();
        let mut changed = r.as_ref().is_some_and(|r| r.changed);
        if fix && adapter.ran {
            let now = std::fs::read(file).unwrap_or_default();
            let before = r.as_ref().map_or(&text, |r| &r.text);
            changed |= now != before.as_bytes();
        }
        if let Some(e) = &adapter.error {
            eprintln!("lynter: {rel}: {e}");
            adapter_failed = true;
        }
        if cli.json {
            // Output-version 1 reports one entry per rule that fires, with the
            // rule description as the message, no line, and fixed=true.
            let mut seen: Vec<&str> = Vec::new();
            let mut vs = Vec::new();
            for v in own {
                if !markdown {
                    vs.push(json!({"rule": v.rule, "line": v.line, "message": v.message, "fixed": v.fixed}));
                    continue;
                }
                if seen.contains(&v.rule.as_str()) {
                    continue;
                }
                seen.push(&v.rule);
                let message =
                    get_rule(&v.rule).map_or(v.message.clone(), |r| r.description().to_owned());
                vs.push(json!({"rule": v.rule, "line": null, "message": message, "fixed": true}));
            }
            for v in &adapter.violations {
                vs.push(json!({
                    "rule": v.rule,
                    "line": v.line,
                    "column": v.column,
                    "message": v.message,
                    "fixable": v.fixable,
                    "fixed": false,
                }));
            }
            if let Some(e) = &adapter.error {
                vs.push(
                    json!({"rule": "adapter-error", "line": null, "message": e, "fixed": false}),
                );
            }
            let chain = r.as_ref().map_or_else(
                || {
                    loaded
                        .resolver
                        .resolve(&lint::facts_for(&text, &rel, today))
                        .chain
                },
                |r| r.chain.clone(),
            );
            let line = json!({
                "path": rel,
                "profile": chain.last().cloned().unwrap_or_default(),
                "violations": vs,
                "renamed_to": null,
                "relocated_to": null,
                "output-version": OUTPUT_VERSION,
            });
            let _ = writeln!(out, "{line}");
        } else if mode == Mode::Check {
            for v in own {
                let _ = writeln!(out, "{rel}: {}: {}", v.rule, v.message);
            }
            for v in &adapter.violations {
                let _ = writeln!(
                    out,
                    "{rel}:{}:{}: {}: {}",
                    v.line, v.column, v.rule, v.message
                );
            }
            if let Some(e) = &adapter.error {
                let _ = writeln!(out, "{rel}: adapter-error: {e}");
            }
        } else if changed {
            let _ = writeln!(out, "{rel}");
        }
    }
    if adapter_failed {
        ExitCode::from(2)
    } else if mode == Mode::Check && violations > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn stdin_fix(cli: &Cli, stdin_path: Option<&str>, today: &str) -> ExitCode {
    let mut bytes = Vec::new();
    let _ = std::io::stdin().read_to_end(&mut bytes);
    let input = String::from_utf8_lossy(&bytes).into_owned();
    let echo = |s: &str| {
        let mut o = std::io::stdout().lock();
        let _ = o.write_all(s.as_bytes());
    };
    let mut cache = cfg::Cache::new(cli.config.as_deref());
    let cwd = std::env::current_dir().unwrap_or_default();
    let loaded = match cache.for_dir(&cwd) {
        Ok(l) => l,
        Err(e) => {
            report_errors(&e);
            echo(&input);
            return ExitCode::from(2);
        }
    };
    let rel = stdin_path.unwrap_or("stdin.md");
    if files::is_ignored(&loaded.config, rel) {
        echo(&input);
        return ExitCode::SUCCESS;
    }
    match lint::lint_any(&input, rel, &loaded.resolver, &loaded.config, today) {
        Some(r) => echo(&r.text),
        None => echo(&input),
    }
    ExitCode::SUCCESS
}

fn explain(cli: &Cli, path: &Path, today: &str) -> ExitCode {
    let abs = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
    if !abs.is_file() {
        return usage(&format!("no such file: {}", path.display()));
    }
    let mut cache = cfg::Cache::new(cli.config.as_deref());
    let loaded = match cache.for_dir(abs.parent().unwrap_or(Path::new("."))) {
        Ok(l) => l,
        Err(e) => {
            report_errors(&e);
            return ExitCode::from(2);
        }
    };
    let rel = files::vault_path(&loaded.dir, &abs);
    let text = String::from_utf8_lossy(&std::fs::read(&abs).unwrap_or_default()).into_owned();
    let resolved = loaded
        .resolver
        .resolve(&lint::facts_for(&text, &rel, today));
    println!("path: {rel}");
    println!("config: {}", loaded.path.display());
    println!("ignored: {}", files::is_ignored(&loaded.config, &rel));
    println!("profile chain: {}", resolved.chain.join(" -> "));
    println!(
        "note type: {}",
        resolved.note_type.as_deref().unwrap_or("(none)")
    );
    println!("rules:");
    if resolved.rules.is_empty() {
        println!("  (none configured)");
    }
    for (id, stanza) in &resolved.rules {
        let on = stanza.get("enabled").and_then(|v| v.as_bool()) == Some(true);
        let state = if on { "enabled" } else { "disabled" };
        match get_rule(id) {
            Some(r) => println!("  {id}: {state}  {}", r.description()),
            None => println!("  {id}: {state}"),
        }
    }
    ExitCode::SUCCESS
}
