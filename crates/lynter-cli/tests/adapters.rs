//! Native-tool adapters (ruff, Biome) through the CLI. Everything here runs
//! with the real tools absent: fake executables stand in for them, and one
//! test exercises real ruff when it is installed.

use assert_cmd::Command;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

const PY: &str = "import os\nx = 1\n";

fn lynter() -> Command {
    Command::cargo_bin("lynter").unwrap()
}

fn vault(config_body: &str) -> TempDir {
    let d = TempDir::new().unwrap();
    fs::write(
        d.path().join("lynter.yaml"),
        format!("version: 1\n{config_body}"),
    )
    .unwrap();
    fs::write(d.path().join("a.py"), PY).unwrap();
    fs::write(d.path().join("b.js"), "var a = 1;\n").unwrap();
    d
}

fn adapters_config(ruff_path: &Path, extra: &str) -> String {
    format!(
        "defaults:\n  adapters:\n    ruff: {{enabled: true, path: \"{}\"{extra}}}\n",
        ruff_path.display()
    )
}

fn json_lines(stdout: &[u8]) -> Vec<Value> {
    String::from_utf8_lossy(stdout)
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

#[test]
fn missing_binary_is_a_notice_not_a_failure() {
    let d = vault(&adapters_config(
        Path::new("/nonexistent/lynter-test-ruff"),
        "",
    ));
    let out = lynter()
        .current_dir(d.path())
        .args(["check", "--json", "."])
        .assert()
        .code(0)
        .get_output()
        .clone();
    let err = String::from_utf8_lossy(&out.stderr);
    assert_eq!(err.matches("notice: ruff not found").count(), 1, "{err}");
    assert!(out.stdout.is_empty());
}

#[test]
fn missing_binary_notice_is_printed_once_for_many_files() {
    let d = vault(&adapters_config(
        Path::new("/nonexistent/lynter-test-ruff"),
        "",
    ));
    fs::write(d.path().join("c.py"), PY).unwrap();
    let out = lynter()
        .current_dir(d.path())
        .args(["check", "."])
        .assert()
        .code(0)
        .get_output()
        .clone();
    let err = String::from_utf8_lossy(&out.stderr);
    assert_eq!(err.matches("notice: ruff not found").count(), 1, "{err}");
}

#[test]
fn adapters_are_off_by_default() {
    let d = vault("");
    lynter()
        .current_dir(d.path())
        .args(["check", "."])
        .assert()
        .code(0)
        .stderr("");
}

#[test]
fn unknown_adapter_keys_are_a_config_error() {
    let d = vault("defaults:\n  adapters:\n    ruff: {enabled: true, bogus: 1}\n");
    let out = lynter()
        .current_dir(d.path())
        .args(["check", "."])
        .assert()
        .code(2)
        .get_output()
        .clone();
    assert!(String::from_utf8_lossy(&out.stderr).contains("defaults.adapters.ruff.bogus"));
}

#[test]
fn explain_and_check_accept_a_clean_adapters_block() {
    let d = vault(
        "defaults:\n  adapters:\n    biome: {enabled: false}\nprofiles:\n  x:\n    match: {path: [\"x/**\"]}\n    adapters:\n      ruff: {enabled: true, select: [F]}\n",
    );
    lynter()
        .current_dir(d.path())
        .args(["check", "."])
        .assert()
        .code(0)
        .stderr("");
}

#[cfg(unix)]
mod fake {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    /// An executable that prints `body` to stdout and exits with `code`.
    fn script(dir: &Path, name: &str, body: &str, code: i32) -> PathBuf {
        let p = dir.join(name);
        fs::write(
            &p,
            format!("#!/bin/sh\ncat <<'EOF'\n{body}\nEOF\nexit {code}\n"),
        )
        .unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o755)).unwrap();
        p
    }

    const RUFF_JSON: &str = r#"[{"cell":null,"code":"F401","end_location":{"column":10,"row":1},"filename":"/x/a.py","fix":{"applicability":"safe","edits":[],"message":"Remove"},"location":{"column":8,"row":1},"message":"`os` imported but unused","name":"unused-import","noqa_row":1,"severity":"error","url":null}]"#;

    #[test]
    fn json_lines_carry_prefixed_ids_and_version_one() {
        let d = vault("");
        let bin = script(d.path(), "fake-ruff", RUFF_JSON, 1);
        fs::write(
            d.path().join("lynter.yaml"),
            format!("version: 1\n{}", adapters_config(&bin, "")),
        )
        .unwrap();
        let out = lynter()
            .current_dir(d.path())
            .args(["check", "--json", "a.py"])
            .assert()
            .code(1)
            .get_output()
            .clone();
        let lines = json_lines(&out.stdout);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0]["output-version"], 1);
        assert_eq!(lines[0]["path"], "a.py");
        let v = &lines[0]["violations"][0];
        assert_eq!(v["rule"], "ruff/F401");
        assert_eq!(v["line"], 1);
        assert_eq!(v["column"], 8);
        assert_eq!(v["fixable"], true);
    }

    #[test]
    fn no_adapters_skips_them() {
        let d = vault("");
        let bin = script(d.path(), "fake-ruff", RUFF_JSON, 1);
        fs::write(
            d.path().join("lynter.yaml"),
            format!("version: 1\n{}", adapters_config(&bin, "")),
        )
        .unwrap();
        lynter()
            .current_dir(d.path())
            .args(["check", "--no-adapters", "--json", "a.py"])
            .assert()
            .code(0)
            .stdout("")
            .stderr("");
        lynter()
            .current_dir(d.path())
            .args(["fix", "--no-adapters", "a.py"])
            .assert()
            .code(0)
            .stdout("");
    }

    #[test]
    fn a_crash_is_one_adapter_error_with_exit_two() {
        let d = vault("");
        let bin = script(d.path(), "fake-ruff", "", 2);
        fs::write(
            d.path().join("lynter.yaml"),
            format!("version: 1\n{}", adapters_config(&bin, "")),
        )
        .unwrap();
        let out = lynter()
            .current_dir(d.path())
            .args(["check", "--json", "a.py"])
            .assert()
            .code(2)
            .get_output()
            .clone();
        let lines = json_lines(&out.stdout);
        let vs = lines[0]["violations"].as_array().unwrap();
        assert_eq!(vs.len(), 1);
        assert_eq!(vs[0]["rule"], "adapter-error");
        assert_eq!(lines[0]["output-version"], 1);
    }

    #[test]
    fn unparseable_output_is_an_adapter_error() {
        let d = vault("");
        let bin = script(d.path(), "fake-ruff", "this is not json", 0);
        fs::write(
            d.path().join("lynter.yaml"),
            format!("version: 1\n{}", adapters_config(&bin, "")),
        )
        .unwrap();
        let out = lynter()
            .current_dir(d.path())
            .args(["check", "a.py"])
            .assert()
            .code(2)
            .get_output()
            .clone();
        assert!(String::from_utf8_lossy(&out.stdout).contains("adapter-error"));
    }

    #[test]
    fn a_profile_can_switch_an_adapter_off() {
        let d = vault("");
        let bin = script(d.path(), "fake-ruff", RUFF_JSON, 1);
        fs::write(
            d.path().join("lynter.yaml"),
            format!(
                "version: 1\n{}profiles:\n  quiet:\n    match: {{path: [\"a.py\"]}}\n    adapters:\n      ruff: {{enabled: false}}\n",
                adapters_config(&bin, "")
            ),
        )
        .unwrap();
        lynter()
            .current_dir(d.path())
            .args(["check", "a.py"])
            .assert()
            .code(0)
            .stdout("");
    }
}

fn find_tool(name: &str) -> Option<PathBuf> {
    let mut dirs: Vec<PathBuf> = std::env::split_paths(&std::env::var_os("PATH")?).collect();
    dirs.push(PathBuf::from("/root/.local/bin"));
    dirs.into_iter().map(|d| d.join(name)).find(|p| p.is_file())
}

#[test]
fn real_ruff_when_installed() {
    let Some(ruff) = find_tool("ruff") else {
        eprintln!("skipping real_ruff_when_installed: ruff is not installed");
        return;
    };
    let d = vault("");
    fs::write(
        d.path().join("lynter.yaml"),
        format!("version: 1\n{}", adapters_config(&ruff, ", select: [F401]")),
    )
    .unwrap();
    let out = lynter()
        .current_dir(d.path())
        .args(["check", "--json", "a.py"])
        .assert()
        .code(1)
        .get_output()
        .clone();
    let lines = json_lines(&out.stdout);
    assert!(!lines.is_empty());
    for l in &lines {
        assert_eq!(l["output-version"], 1);
        for v in l["violations"].as_array().unwrap() {
            assert!(v["rule"].as_str().unwrap().starts_with("ruff/"), "{v}");
        }
    }
    assert_eq!(lines[0]["violations"][0]["rule"], "ruff/F401");
    lynter()
        .current_dir(d.path())
        .args(["fix", "a.py"])
        .assert()
        .code(0)
        .stdout("a.py\n");
    assert!(!fs::read_to_string(d.path().join("a.py"))
        .unwrap()
        .contains("import os"));
}

#[test]
fn real_biome_when_installed() {
    let Some(biome) = find_tool("biome").or_else(|| {
        let p = PathBuf::from("/tmp/biome/node_modules/.bin/biome");
        p.is_file().then_some(p)
    }) else {
        eprintln!("skipping real_biome_when_installed: biome is not installed");
        return;
    };
    let d = vault("");
    fs::write(d.path().join("b.js"), "debugger;\n").unwrap();
    fs::write(
        d.path().join("lynter.yaml"),
        format!(
            "version: 1\ndefaults:\n  adapters:\n    biome: {{enabled: true, path: \"{}\"}}\n",
            biome.display()
        ),
    )
    .unwrap();
    let out = lynter()
        .current_dir(d.path())
        .args(["check", "--json", "b.js"])
        .assert()
        .code(1)
        .get_output()
        .clone();
    let lines = json_lines(&out.stdout);
    assert_eq!(
        lines[0]["violations"][0]["rule"],
        "biome/lint/suspicious/noDebugger"
    );
    assert_eq!(lines[0]["output-version"], 1);
}
