//! Integration tests against temp vaults.

use assert_cmd::Command;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

const CONFIG: &str = "version: 1
defaults:
  rules:
    strip-strong: {enabled: true}
    replace-em-dash: {enabled: true}
    header-case: {enabled: true, h1: \"First letter\"}
    contractions: {enabled: true}
profiles:
  drafts:
    match: {path: [\"drafts/**\"]}
    rules:
      header-case: {enabled: false}
ignore:
  files: [notes/skip.md]
";

fn write(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

fn vault(config_name: &str) -> TempDir {
    let d = TempDir::new().unwrap();
    let r = d.path();
    write(r, config_name, CONFIG);
    write(
        r,
        "notes/a.md",
        "---\ntitle: a\n---\n# the quick brown fox\n\nThis is **bold** text \u{2014} with a dash. It is fine, do not worry.\n",
    );
    write(r, "notes/clean.md", "# Clean\n\nAll good here.\n");
    write(r, "notes/skip.md", "# Ignore me\n\n**x** it is\n");
    write(
        r,
        "drafts/d.md",
        "# some draft title\n\nWe are **here** \u{2014} now.\n",
    );
    write(r, "notes/ignored-by-git.md", "**x**\n");
    write(r, ".gitignore", "ignored-by-git.md\n");
    d
}

fn lynter() -> Command {
    Command::cargo_bin("lynter").unwrap()
}

#[test]
fn check_reports_and_exits_one() {
    let d = vault("lynter.yaml");
    let out = lynter()
        .current_dir(d.path())
        .args(["check", "."])
        .assert()
        .code(1);
    let s = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(s.contains("notes/a.md: strip-strong:"), "{s}");
    assert!(s.contains("drafts/d.md: replace-em-dash:"), "{s}");
    assert!(!s.contains("skip.md"), "ignore.files respected: {s}");
    assert!(!s.contains("ignored-by-git"), ".gitignore respected: {s}");
    assert!(!s.contains("clean.md"), "{s}");
}

#[test]
fn check_clean_file_exits_zero() {
    let d = vault("lynter.yaml");
    lynter()
        .current_dir(d.path())
        .args(["check", "notes/clean.md"])
        .assert()
        .code(0);
}

#[test]
fn fix_rewrites_and_lists_changed_files() {
    let d = vault("lynter.yaml");
    let out = lynter()
        .current_dir(d.path())
        .args(["fix", "notes"])
        .assert()
        .code(0);
    let s = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(s.trim(), "notes/a.md");
    let text = fs::read_to_string(d.path().join("notes/a.md")).unwrap();
    assert!(text.contains("# The quick brown fox"), "{text}");
    assert!(
        text.contains("bold text, with a dash. It's fine, don't worry."),
        "{text}"
    );
    lynter()
        .current_dir(d.path())
        .args(["check", "notes/a.md"])
        .assert()
        .code(0);
}

#[test]
fn fix_stdin_filters_and_echoes_on_config_error() {
    let d = vault("lynter.yaml");
    lynter()
        .current_dir(d.path())
        .args(["fix", "--stdin", "--stdin-path", "drafts/x.md"])
        .write_stdin("# keep case\n\n**b** it is\n")
        .assert()
        .code(0)
        .stdout("# keep case\n\nb it's\n");
    let empty = TempDir::new().unwrap();
    lynter()
        .current_dir(empty.path())
        .args(["fix", "--stdin"])
        .write_stdin("**keep**\n")
        .assert()
        .code(2)
        .stdout("**keep**\n");
}

#[test]
fn missing_config_and_bad_config_exit_two() {
    let empty = TempDir::new().unwrap();
    write(empty.path(), "a.md", "x\n");
    lynter()
        .current_dir(empty.path())
        .args(["check", "a.md"])
        .assert()
        .code(2);
    write(empty.path(), "lynter.yaml", "version: 7\n");
    lynter()
        .current_dir(empty.path())
        .args(["check", "a.md"])
        .assert()
        .code(2);
    lynter().args(["check"]).assert().code(2);
    lynter().args(["bogus"]).assert().code(2);
}

#[test]
fn explicit_config_overrides_discovery() {
    let d = vault("lynter.yaml");
    let other = TempDir::new().unwrap();
    write(
        other.path(),
        "alt.yaml",
        "version: 1\ndefaults:\n  rules:\n    strip-strong: {enabled: false}\n",
    );
    lynter()
        .current_dir(d.path())
        .args(["check", "notes/a.md", "--config"])
        .arg(other.path().join("alt.yaml"))
        .assert()
        .code(0);
}

#[test]
fn deprecated_names_are_discovered_with_notice() {
    let d = vault("linter.yaml");
    let out = lynter()
        .current_dir(d.path())
        .args(["check", "notes/a.md"])
        .assert()
        .code(1);
    let e = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(e.contains("deprecated"), "{e}");
}

#[test]
fn explain_shows_config_chain_and_rules() {
    let d = vault("lynter.yaml");
    let out = lynter()
        .current_dir(d.path())
        .args(["explain", "drafts/d.md"])
        .assert()
        .code(0);
    let s = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(s.contains("path: drafts/d.md"), "{s}");
    assert!(s.contains("lynter.yaml"), "{s}");
    assert!(s.contains("profile chain: defaults -> drafts"), "{s}");
    assert!(s.contains("header-case: disabled"), "{s}");
    assert!(s.contains("strip-strong: enabled"), "{s}");
}

#[test]
fn init_writes_a_valid_config_and_refuses_to_overwrite() {
    let d = TempDir::new().unwrap();
    lynter().current_dir(d.path()).arg("init").assert().code(0);
    assert!(d.path().join("lynter.yaml").is_file());
    write(d.path(), "a.md", "# A\n\nplain\n");
    lynter()
        .current_dir(d.path())
        .args(["check", "a.md"])
        .assert()
        .code(0);
    lynter().current_dir(d.path()).arg("init").assert().code(2);
}

#[test]
fn json_lines_have_stable_keys() {
    let d = vault("lynter.yaml");
    let out = lynter()
        .current_dir(d.path())
        .args(["check", "notes/clean.md", "--json", "--today", "2026-01-01"])
        .assert()
        .code(0);
    let s = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(
        s.trim(),
        r#"{"path":"notes/clean.md","profile":"defaults","violations":[],"renamed_to":null,"relocated_to":null,"output-version":1}"#
    );
    lynter()
        .args(["check", ".", "--today", "bad"])
        .assert()
        .code(2);
}

#[test]
fn version_flag_works() {
    lynter().arg("--version").assert().code(0);
}

#[test]
fn changed_targets_git_modified_markdown() {
    let d = vault("lynter.yaml");
    let git = |args: &[&str]| {
        let ok = std::process::Command::new("git")
            .args(["-c", "user.name=t", "-c", "user.email=t@example.com"])
            .args(args)
            .current_dir(d.path())
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        ok
    };
    if !git(&["init", "-q"]) {
        return;
    }
    assert!(git(&["add", "-A"]));
    assert!(git(&["commit", "-qm", "init"]));
    write(d.path(), "notes/clean.md", "# Clean\n\nIt is **new**.\n");
    let out = lynter()
        .current_dir(d.path())
        .args(["check", "--changed"])
        .assert()
        .code(1);
    let s = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(s.contains("notes/clean.md"), "{s}");
    assert!(!s.contains("notes/a.md"), "{s}");
}

const CODE_CONFIG: &str = "version: 1
defaults:
  rules:
    json-key-order: {enabled: true, priority: [name]}
    json-syntax: {enabled: true}
    yaml-duplicate-keys: {enabled: true}
    identifier-case: {enabled: true}
";

#[test]
fn routes_json_yaml_and_code_files_to_the_code_host() {
    let d = TempDir::new().unwrap();
    let r = d.path();
    write(r, "lynter.yaml", CODE_CONFIG);
    write(r, "pkg/a.json", "{\"b\": 1, \"name\": \"x\"}\n");
    write(r, "pkg/bad.json", "{\"a\": }\n");
    write(r, "pkg/d.yml", "a: 1\na: 2\n");
    write(r, "src/lib.rs", "fn BadName() {}\n");
    write(r, "src/ok.rs", "fn good_name() {}\n");
    write(r, "notes/n.md", "# Fine\n");
    let out = lynter()
        .current_dir(r)
        .args(["check", ".", "--json", "--today", "2026-01-01"])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    let text = String::from_utf8(out.stdout).unwrap();
    let lines: Vec<serde_json::Value> = text
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    let find = |p: &str| {
        lines
            .iter()
            .find(|l| l["path"] == p)
            .unwrap_or_else(|| panic!("{p} missing in {text}"))
    };
    assert_eq!(
        find("pkg/a.json")["violations"][0]["rule"],
        "json-key-order"
    );
    assert_eq!(find("pkg/a.json")["violations"][0]["fixed"], true);
    assert_eq!(find("pkg/bad.json")["violations"][0]["rule"], "json-syntax");
    assert_eq!(find("pkg/bad.json")["violations"][0]["line"], 1);
    assert_eq!(
        find("pkg/d.yml")["violations"][0]["rule"],
        "yaml-duplicate-keys"
    );
    assert_eq!(
        find("src/lib.rs")["violations"][0]["rule"],
        "identifier-case"
    );
    assert_eq!(find("src/ok.rs")["violations"].as_array().unwrap().len(), 0);
    assert!(lines.iter().all(|l| l["output-version"] == 1));
    // The config file itself has no yaml rule enabled that fires, and the
    // Markdown note is still linted.
    assert_eq!(
        find("notes/n.md")["violations"].as_array().unwrap().len(),
        0
    );

    let fixed = lynter()
        .current_dir(r)
        .args(["fix", "pkg/a.json"])
        .output()
        .unwrap();
    assert!(fixed.status.success());
    assert_eq!(
        fs::read_to_string(r.join("pkg/a.json")).unwrap(),
        "{\"name\": \"x\", \"b\": 1}\n"
    );
}

#[test]
fn code_files_without_enabled_rules_are_skipped() {
    let d = vault("lynter.yaml");
    write(d.path(), "notes/data.json", "{\"a\": 1, \"a\": 2}");
    let out = lynter()
        .current_dir(d.path())
        .args(["check", "notes", "--json", "--today", "2026-01-01"])
        .output()
        .unwrap();
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(!text.contains("data.json"), "{text}");
}
