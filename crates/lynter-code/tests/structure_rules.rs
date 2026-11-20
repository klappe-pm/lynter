//! The five cross-language rules in every compiled-in grammar: one firing and
//! one clean fixture each, plus option, doc-comment and node-kind checks.
mod common;
use common::run;
use lynter_code::rules::kinds::kinds;
use lynter_code::Language;

struct Fx {
    path: &'static str,
    /// Prefix every comment-only fixture needs to be a valid file.
    pre: &'static str,
    /// Line comment marker.
    marker: &'static str,
    bad_syntax: &'static str,
    ok_syntax: &'static str,
    /// Code that re-parses, put in a comment.
    code: &'static str,
    /// Source with `n` levels of nesting.
    nest: fn(usize) -> String,
    /// Source with one function or statement of `n` body lines.
    body: fn(usize) -> String,
}

fn rep(s: &str, n: usize) -> String {
    s.repeat(n)
}

fn braces(head: &str, open: &str, stmt: &str, n: usize, tail: &str) -> String {
    format!("{head}{}{stmt}\n{}{tail}", rep(open, n), rep("}\n", n))
}

fn lines(head: &str, stmt: &str, n: usize, tail: &str) -> String {
    format!("{head}{}{tail}", rep(&format!("{stmt}\n"), n))
}

fn fixtures() -> Vec<Fx> {
    let mut v: Vec<Fx> = Vec::new();
    #[cfg(feature = "lang-rust")]
    v.push(Fx {
        path: "a.rs",
        pre: "",
        marker: "//",
        bad_syntax: "fn main( {\n",
        ok_syntax: "fn main() {}\n",
        code: "let total = add(1, 2);",
        nest: |n| braces("fn a() {\n", "if x {\n", "let y = 1;", n, "}\n"),
        body: |n| lines("fn a() {\n", "let y = 1;", n, "}\n"),
    });
    #[cfg(feature = "lang-python")]
    v.push(Fx {
        path: "a.py",
        pre: "",
        marker: "#",
        bad_syntax: "def f(:\n    pass\n",
        ok_syntax: "def f():\n    pass\n",
        code: "total = add(1, 2)",
        nest: |n| {
            let mut s = String::from("def a():\n");
            for i in 1..=n {
                s += &format!("{}if x:\n", "    ".repeat(i));
            }
            s + &format!("{}pass\n", "    ".repeat(n + 1))
        },
        body: |n| lines("def a():\n", "    y = 1", n, ""),
    });
    #[cfg(feature = "lang-typescript")]
    v.push(Fx {
        path: "a.ts",
        pre: "",
        marker: "//",
        bad_syntax: "function ( {\n",
        ok_syntax: "function f(): void {}\n",
        code: "const total = add(1, 2);",
        nest: |n| braces("function a() {\n", "if (x) {\n", "y();", n, "}\n"),
        body: |n| lines("function a() {\n", "  y();", n, "}\n"),
    });
    #[cfg(feature = "lang-typescript")]
    v.push(Fx {
        path: "a.tsx",
        pre: "",
        marker: "//",
        bad_syntax: "function ( {\n",
        ok_syntax: "function f() { return <div/>; }\n",
        code: "const total = add(1, 2);",
        nest: |n| braces("function a() {\n", "if (x) {\n", "y();", n, "}\n"),
        body: |n| lines("function a() {\n", "  y();", n, "}\n"),
    });
    #[cfg(feature = "lang-javascript")]
    v.push(Fx {
        path: "a.js",
        pre: "",
        marker: "//",
        bad_syntax: "function ( {\n",
        ok_syntax: "function f() {}\n",
        code: "const total = add(1, 2);",
        nest: |n| braces("function a() {\n", "if (x) {\n", "y();", n, "}\n"),
        body: |n| lines("function a() {\n", "  y();", n, "}\n"),
    });
    #[cfg(feature = "lang-go")]
    v.push(Fx {
        path: "a.go",
        pre: "package main\n",
        marker: "//",
        bad_syntax: "package main\nfunc (\n",
        ok_syntax: "package main\nfunc f() {}\n",
        code: "total := add(1, 2)",
        nest: |n| braces("package main\nfunc a() {\n", "if x {\n", "y()", n, "}\n"),
        body: |n| lines("package main\nfunc a() {\n", "\ty()", n, "}\n"),
    });
    #[cfg(feature = "lang-java")]
    v.push(Fx {
        path: "A.java",
        pre: "",
        marker: "//",
        bad_syntax: "class A { void f( }\n",
        ok_syntax: "class A { void f() {} }\n",
        code: "int total = add(1, 2);",
        nest: |n| braces("class A {\nvoid a() {\n", "if (x) {\n", "y();", n, "}\n}\n"),
        body: |n| lines("class A {\nvoid a() {\n", "  y();", n, "}\n}\n"),
    });
    #[cfg(feature = "lang-csharp")]
    v.push(Fx {
        path: "A.cs",
        pre: "",
        marker: "//",
        bad_syntax: "class A { void F( }\n",
        ok_syntax: "class A { void F() {} }\n",
        code: "int total = Add(1, 2);",
        nest: |n| {
            braces(
                "class A {\nvoid A1() {\n",
                "if (x) {\n",
                "Y();",
                n,
                "}\n}\n",
            )
        },
        body: |n| lines("class A {\nvoid A1() {\n", "  Y();", n, "}\n}\n"),
    });
    #[cfg(feature = "lang-sql")]
    v.push(Fx {
        path: "a.sql",
        pre: "",
        marker: "--",
        bad_syntax: "SELECT * FROM (;\n",
        ok_syntax: "SELECT 1;\n",
        code: "SELECT a, b FROM t WHERE a = 1;",
        nest: |n| {
            let mut q = String::from("SELECT 1");
            for i in 0..n {
                q = format!("SELECT * FROM ({q}) t{i}");
            }
            q + ";\n"
        },
        body: |n| {
            format!(
                "SELECT\n{}  1\nFROM t;\n",
                rep("  a,\n", n.saturating_sub(2))
            )
        },
    });
    v
}

fn cfg(rule: &str, opts: &str) -> String {
    format!("version: 1\ndefaults:\n  rules:\n    {rule}: {{enabled: true{opts}}}\n")
}

fn count(rule: &str, opts: &str, path: &str, src: &str) -> usize {
    match run(&cfg(rule, opts), path, src) {
        None => 0,
        Some(r) => {
            assert!(r.violations.iter().all(|v| v.rule == rule && !v.fixed));
            assert_eq!(r.text, src, "{path}: report only");
            r.violations.len()
        }
    }
}

#[test]
fn syntax_error_fires_with_position_and_clean_passes() {
    for f in fixtures() {
        let r = run(&cfg("syntax-error", ""), f.path, f.bad_syntax)
            .unwrap_or_else(|| panic!("{}: no result", f.path));
        assert!(!r.violations.is_empty(), "{}: expected a finding", f.path);
        assert!(
            r.violations[0].message.contains("line ")
                && r.violations[0].message.contains("column "),
            "{}: {}",
            f.path,
            r.violations[0].message
        );
        assert_eq!(
            count("syntax-error", "", f.path, f.ok_syntax),
            0,
            "{}",
            f.path
        );
    }
}

#[test]
fn syntax_error_reports_missing_nodes() {
    #[cfg(feature = "lang-rust")]
    {
        let r = run(&cfg("syntax-error", ""), "a.rs", "fn a() { let x = 1 }\n").unwrap();
        assert!(
            r.violations
                .iter()
                .any(|v| v.message.starts_with("missing ")),
            "{:?}",
            r.violations
        );
    }
}

#[test]
fn comment_tag_fires_and_clean() {
    for f in fixtures() {
        let (p, m, pre) = (f.path, f.marker, f.pre);
        let bad = format!("{pre}{m} TODO fix this later\n");
        assert_eq!(count("comment-tag", "", p, &bad), 1, "{p}");
        let ok = format!(
            "{pre}{m} TODO(kevin): fix this later\n{m} FIXME #12: later\n{m} HACK(a-b): x\n{m} XXX ABC-9: y\n{m} TODOS are fine\n"
        );
        assert_eq!(count("comment-tag", "", p, &ok), 0, "{p}");
    }
}

#[test]
fn comment_tag_options_and_block_comments() {
    #[cfg(feature = "lang-rust")]
    {
        let src = "/* line one\n * TODO later\n * FIXME(a): ok\n */\n// NOTE x\n// FIXME y\n";
        assert_eq!(count("comment-tag", "", "a.rs", src), 2);
        assert_eq!(count("comment-tag", ", tags: [NOTE]", "a.rs", src), 1);
        assert_eq!(
            count(
                "comment-tag",
                ", pattern: '{tag} @{owner}:'",
                "a.rs",
                "// TODO @kev: x\n"
            ),
            0
        );
        assert_eq!(
            count(
                "comment-tag",
                ", pattern: '{tag} @{owner}:'",
                "a.rs",
                "// TODO(kev): x\n"
            ),
            1
        );
        let r = run(&cfg("comment-tag", ""), "a.rs", "fn a() {}\n// TODO x\n").unwrap();
        assert_eq!(r.violations[0].line, 2);
        assert_eq!(
            &"fn a() {}\n// TODO x\n"[r.violations[0].start..r.violations[0].end],
            "TODO"
        );
    }
}

#[test]
fn commented_out_code_fires_and_clean() {
    for f in fixtures() {
        let (p, m, pre) = (f.path, f.marker, f.pre);
        let bad = format!("{pre}{m} {}\n", f.code);
        assert_eq!(count("commented-out-code", "", p, &bad), 1, "{p}: {bad}");
        let ok = format!("{pre}{m} this is just a plain prose comment about the module\n{m} ok\n");
        assert_eq!(count("commented-out-code", "", p, &ok), 0, "{p}");
        // Short comments are ignored.
        let short = format!("{pre}{m} a = 1;\n");
        assert_eq!(count("commented-out-code", "", p, &short), 0, "{p}: short");
    }
}

#[test]
fn commented_out_code_runs_and_doc_comments() {
    #[cfg(feature = "lang-rust")]
    {
        let run3 = "// let a = f(1);\n// let b = g(a);\n// h(b);\nfn x() {}\n";
        assert_eq!(count("commented-out-code", "", "a.rs", run3), 1);
        // Each line alone is not code, the run is.
        let split = "// fn f() {\n//     g();\n// }\n";
        assert_eq!(count("commented-out-code", "", "a.rs", split), 1);
        // Doc comments are ignored.
        assert_eq!(
            count(
                "commented-out-code",
                "",
                "a.rs",
                "/// let total = add(1, 2);\nfn a() {}\n"
            ),
            0
        );
        assert_eq!(
            count(
                "commented-out-code",
                "",
                "a.rs",
                "/** let total = add(1, 2); */\nfn a() {}\n"
            ),
            0
        );
        // A trailing comment is not merged with the next line.
        assert_eq!(
            count(
                "commented-out-code",
                "",
                "a.rs",
                "let a = 1; // let b = f(1);\n// ok\n"
            ),
            1
        );
        assert_eq!(
            count("commented-out-code", ", min-length: 40", "a.rs", run3),
            0
        );
    }
    #[cfg(feature = "lang-python")]
    {
        let src = "# if ready:\n#     launch(rocket)\nx = 1\n";
        assert_eq!(count("commented-out-code", "", "a.py", src), 1);
    }
    #[cfg(feature = "lang-typescript")]
    assert_eq!(
        count(
            "commented-out-code",
            "",
            "a.ts",
            "/** const a = f(1); */\n/* const b = f(2); */\n"
        ),
        1
    );
}

#[test]
fn max_nesting_depth_fires_and_clean() {
    for f in fixtures() {
        assert_eq!(
            count("max-nesting-depth", "", f.path, &(f.nest)(5)),
            1,
            "{}",
            f.path
        );
        assert_eq!(
            count("max-nesting-depth", "", f.path, &(f.nest)(4)),
            0,
            "{}",
            f.path
        );
        assert_eq!(
            count("max-nesting-depth", ", max: 2", f.path, &(f.nest)(3)),
            1,
            "{}",
            f.path
        );
    }
}

#[test]
fn max_nesting_depth_counts_case_in_sql() {
    #[cfg(feature = "lang-sql")]
    {
        let case = |n: usize| {
            let mut e = String::from("1");
            for _ in 0..n {
                e = format!("CASE WHEN a > 1 THEN {e} ELSE 0 END");
            }
            format!("SELECT {e} FROM t;\n")
        };
        assert_eq!(count("max-nesting-depth", "", "a.sql", &case(5)), 1);
        assert_eq!(count("max-nesting-depth", "", "a.sql", &case(4)), 0);
        let mixed = "SELECT CASE WHEN a THEN (SELECT CASE WHEN b THEN (SELECT 1) END) END FROM (SELECT 1) t;\n";
        assert_eq!(count("max-nesting-depth", ", max: 3", "a.sql", mixed), 1);
    }
}

#[test]
fn max_function_lines_fires_and_clean() {
    for f in fixtures() {
        assert_eq!(
            count("max-function-lines", ", max: 5", f.path, &(f.body)(10)),
            1,
            "{}",
            f.path
        );
        assert_eq!(
            count("max-function-lines", ", max: 5", f.path, &(f.body)(1)),
            0,
            "{}",
            f.path
        );
    }
}

#[test]
fn max_function_lines_default_is_60() {
    #[cfg(feature = "lang-rust")]
    {
        let body = |n: usize| lines("fn a() {\n", "let y = 1;", n, "}\n");
        assert_eq!(count("max-function-lines", "", "a.rs", &body(58)), 0);
        assert_eq!(count("max-function-lines", "", "a.rs", &body(59)), 1);
    }
}

#[test]
fn rules_respect_the_languages_option() {
    #[cfg(feature = "lang-rust")]
    assert!(run(
        &cfg("syntax-error", ", languages: [python]"),
        "a.rs",
        "fn ((\n"
    )
    .is_none());
}

#[test]
fn every_kind_in_the_table_exists_in_its_grammar() {
    for lang in lynter_code::language::available() {
        let grammar = lang.grammar().unwrap();
        let k = kinds(lang);
        let all = [
            k.comments,
            k.functions,
            k.body_owners,
            k.blocks,
            k.statements,
        ];
        for list in all {
            for kind in list {
                assert_ne!(
                    grammar.id_for_node_kind(kind, true),
                    0,
                    "{lang:?}: {kind} is not a named node kind"
                );
            }
        }
    }
}

#[test]
fn comment_kinds_match_real_parses() {
    let samples: Vec<(Language, &str, usize)> = vec![
        #[cfg(feature = "lang-rust")]
        (Language::Rust, "// a\n/* b */\n/// c\nfn f() {}\n", 3),
        #[cfg(feature = "lang-python")]
        (Language::Python, "# a\nx = 1\n", 1),
        #[cfg(feature = "lang-typescript")]
        (Language::TypeScript, "// a\n/* b */\nlet x = 1;\n", 2),
        #[cfg(feature = "lang-javascript")]
        (Language::JavaScript, "// a\n/* b */\nlet x = 1;\n", 2),
        #[cfg(feature = "lang-go")]
        (Language::Go, "package p\n// a\n/* b */\n", 2),
        #[cfg(feature = "lang-java")]
        (Language::Java, "// a\n/* b */\nclass A {}\n", 2),
        #[cfg(feature = "lang-csharp")]
        (Language::CSharp, "// a\n/* b */\nclass A {}\n", 2),
        #[cfg(feature = "lang-sql")]
        (Language::Sql, "-- a\n/* b */\nSELECT 1;\n", 2),
    ];
    for (lang, src, expected) in samples {
        let tree = lang.parse(src).unwrap();
        let k = kinds(lang);
        let mut n = 0;
        let mut stack = vec![tree.root_node()];
        while let Some(node) = stack.pop() {
            if k.is_comment(node.kind()) {
                n += 1;
            }
            let mut c = node.walk();
            stack.extend(node.children(&mut c));
        }
        assert_eq!(n, expected, "{lang:?}");
    }
}
