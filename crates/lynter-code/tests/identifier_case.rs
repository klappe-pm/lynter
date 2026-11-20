mod common;
use common::run;

const CFG: &str = "version: 1\ndefaults:\n  rules:\n    identifier-case: {enabled: true}\n";

/// (path, source, flagged names). Each fixture has good and bad names for a
/// function and a variable, so every grammar and both name classes are hit.
fn fixtures() -> Vec<(&'static str, &'static str, Vec<&'static str>)> {
    let mut v: Vec<(&str, &str, Vec<&str>)> = Vec::new();
    #[cfg(feature = "lang-rust")]
    v.push((
        "a.rs",
        "fn good_name() { let ok_var = 1; let BadVar = 2; const MAX: u8 = 3; }\nfn BadName() {}\n",
        vec!["BadVar", "BadName"],
    ));
    #[cfg(feature = "lang-python")]
    v.push((
        "a.py",
        "def good_name():\n    ok_var = 1\n    badVar = 2\n    LIMIT = 3\n\ndef BadName():\n    pass\n",
        vec!["badVar", "BadName"],
    ));
    #[cfg(feature = "lang-typescript")]
    {
        v.push((
            "a.ts",
            "function goodName(x: number): number { const okVar = x; const bad_var = 1; return okVar; }\nfunction bad_name(): void {}\n",
            vec!["bad_var", "bad_name"],
        ));
        v.push((
            "a.tsx",
            "function goodName() { const bad_var = <div/>; return bad_var; }\n",
            vec!["bad_var"],
        ));
    }
    #[cfg(feature = "lang-javascript")]
    v.push((
        "a.js",
        "function goodName() { let okVar = 1; var Bad_var = 2; }\nfunction Bad_name() {}\n",
        vec!["Bad_var", "Bad_name"],
    ));
    #[cfg(feature = "lang-go")]
    v.push((
        "a.go",
        "package main\n\nfunc goodName() {\n\tokVar := 1\n\tbad_var := 2\n\t_, _ = okVar, bad_var\n}\n\nfunc bad_name() {}\n",
        vec!["bad_var", "bad_name"],
    ));
    #[cfg(feature = "lang-java")]
    v.push((
        "A.java",
        "class A {\n  void goodName() { int okVar = 1; int Bad_var = 2; }\n  void Bad_name() {}\n}\n",
        vec!["Bad_var", "Bad_name"],
    ));
    #[cfg(feature = "lang-csharp")]
    v.push((
        "A.cs",
        "class A {\n  void GoodName() { int okVar = 1; int Bad_var = 2; }\n  void bad_name() {}\n}\n",
        vec!["Bad_var", "bad_name"],
    ));
    #[cfg(feature = "lang-sql")]
    v.push((
        "a.sql",
        "CREATE TABLE good_name (id INT);\nCREATE TABLE BadName (id INT);\n",
        vec!["BadName"],
    ));
    v
}

#[test]
fn flags_bad_names_in_every_enabled_language() {
    let fx = fixtures();
    for (path, src, bad) in fx {
        let r = run(CFG, path, src).unwrap_or_else(|| panic!("{path}: no result"));
        let got: Vec<String> = r
            .violations
            .iter()
            .map(|v| src[v.start..v.end].to_owned())
            .collect();
        assert_eq!(got, bad, "{path}: {:?}", r.violations);
        assert!(r.violations.iter().all(|v| !v.fixed), "{path}: report only");
        assert_eq!(r.text, src, "{path}: text unchanged");
    }
}

#[test]
fn style_options_and_language_scope() {
    #[cfg(feature = "lang-rust")]
    {
        let c = "version: 1\ndefaults:\n  rules:\n    identifier-case: {enabled: true, functions: {rust: camel}, ignore: [skip_me]}\n";
        let r = run(
            c,
            "a.rs",
            "fn goodName() {}\nfn bad_name() {}\nfn skip_me() {}\nfn skipMe() {}\n",
        )
        .unwrap();
        let names: Vec<_> = r.violations.iter().map(|v| v.message.clone()).collect();
        assert_eq!(names, vec!["function \"bad_name\" should be camel case"]);
        // `languages` restricts a stanza to the listed languages.
        let only_py = "version: 1\ndefaults:\n  rules:\n    identifier-case: {enabled: true, languages: [python]}\n";
        assert!(run(only_py, "a.rs", "fn BadName() {}\n").is_none());
    }
}

#[test]
fn parse_errors_do_not_panic() {
    #[cfg(feature = "lang-rust")]
    assert!(run(CFG, "a.rs", "fn (((( {{{").is_some());
}
