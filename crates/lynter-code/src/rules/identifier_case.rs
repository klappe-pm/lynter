//! `identifier-case`: function and variable names follow a case style per
//! language. The rule reports only: renaming a declaration without its
//! references would break the code, so it never rewrites.

use crate::language::Language;
use crate::rule::{report, CodeRule, SourceFile};
use lynter_core::edit::Edit;
use lynter_core::Options;
use serde_json::{json, Value};
use tree_sitter::Node;

pub struct IdentifierCase;

const LANGS: [&str; 8] = [
    "rust",
    "python",
    "typescript",
    "javascript",
    "sql",
    "go",
    "java",
    "csharp",
];

fn default_function_style(lang: &str) -> &'static str {
    match lang {
        "rust" | "python" | "sql" => "snake",
        "go" => "mixed",
        "csharp" => "pascal",
        _ => "camel",
    }
}

fn default_variable_style(lang: &str) -> &'static str {
    match lang {
        "rust" | "python" | "sql" => "snake",
        "go" => "mixed",
        _ => "camel",
    }
}

fn matches_style(name: &str, style: &str) -> bool {
    let core = name.trim_matches('_');
    let Some(first) = core.chars().next() else {
        return true;
    };
    if !core.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return false;
    }
    match style {
        "snake" => !core.chars().any(char::is_uppercase),
        "camel" => !core.contains('_') && !first.is_uppercase(),
        "pascal" => !core.contains('_') && first.is_uppercase(),
        "mixed" => !core.contains('_'),
        _ => true,
    }
}

fn is_constant(name: &str) -> bool {
    let core = name.trim_matches('_');
    core.chars().any(char::is_alphabetic) && !core.chars().any(char::is_lowercase)
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Function,
    Variable,
}

/// Name nodes declared by `node`, with whether each is a function or variable.
fn declared<'t>(lang: Language, node: Node<'t>, out: &mut Vec<(Node<'t>, Kind)>) {
    let field = |n: Node<'t>, f: &str| n.child_by_field_name(f);
    let ident = |n: Node<'t>| (n.kind() == "identifier").then_some(n);
    match (lang, node.kind()) {
        (Language::Rust, "function_item") => {
            out.extend(field(node, "name").map(|n| (n, Kind::Function)))
        }
        (Language::Rust, "let_declaration") => out.extend(
            field(node, "pattern")
                .and_then(ident)
                .map(|n| (n, Kind::Variable)),
        ),
        (Language::Python, "function_definition") => {
            out.extend(field(node, "name").map(|n| (n, Kind::Function)));
        }
        (Language::Python, "assignment") => out.extend(
            field(node, "left")
                .and_then(ident)
                .map(|n| (n, Kind::Variable)),
        ),
        (
            Language::TypeScript | Language::Tsx | Language::JavaScript,
            "function_declaration" | "method_definition",
        ) => out.extend(field(node, "name").map(|n| (n, Kind::Function))),
        (Language::TypeScript | Language::Tsx | Language::JavaScript, "variable_declarator") => {
            out.extend(
                field(node, "name")
                    .and_then(ident)
                    .map(|n| (n, Kind::Variable)),
            );
        }
        (Language::Go, "function_declaration" | "method_declaration") => {
            out.extend(field(node, "name").map(|n| (n, Kind::Function)));
        }
        (Language::Go, "short_var_declaration") => {
            if let Some(left) = field(node, "left") {
                let mut c = left.walk();
                out.extend(
                    left.children(&mut c)
                        .filter(|n| n.kind() == "identifier")
                        .map(|n| (n, Kind::Variable)),
                );
            }
        }
        (Language::Go, "var_spec") => {
            let mut c = node.walk();
            out.extend(
                node.children_by_field_name("name", &mut c)
                    .filter(|n| n.kind() == "identifier")
                    .map(|n| (n, Kind::Variable)),
            );
        }
        (Language::Java | Language::CSharp, "method_declaration") => {
            out.extend(field(node, "name").map(|n| (n, Kind::Function)));
        }
        (Language::Java | Language::CSharp, "variable_declarator") => {
            let name = field(node, "name").or_else(|| {
                let mut c = node.walk();
                let found = node.children(&mut c).find(|n| n.kind() == "identifier");
                found
            });
            out.extend(name.and_then(ident).map(|n| (n, Kind::Variable)));
        }
        (Language::Sql, "create_table" | "create_function") => {
            let name = field(node, "name").or_else(|| {
                let mut c = node.walk();
                let found = node
                    .children(&mut c)
                    .find(|n| n.kind() == "object_reference");
                found
            });
            out.extend(name.map(|n| (n, Kind::Function)));
        }
        _ => {}
    }
}

fn style_for(o: &Options, group: &str, lang: &str, default: &str) -> String {
    o.get(group)
        .and_then(|g| g.get(lang))
        .and_then(Value::as_str)
        .unwrap_or(default)
        .to_owned()
}

impl CodeRule for IdentifierCase {
    fn id(&self) -> &'static str {
        "identifier-case"
    }
    fn description(&self) -> &'static str {
        "Function and variable names follow the configured case style for the language."
    }
    fn default_options(&self) -> Options {
        let mut functions = serde_json::Map::new();
        let mut variables = serde_json::Map::new();
        for l in LANGS {
            functions.insert(l.into(), json!(default_function_style(l)));
            variables.insert(l.into(), json!(default_variable_style(l)));
        }
        let Value::Object(m) = json!({
            "functions": functions,
            "variables": variables,
            "allow-constants": true,
            "ignore": [],
        }) else {
            unreachable!()
        };
        m
    }
    fn applies_to(&self, f: &SourceFile) -> bool {
        f.language.is_some()
    }
    fn check(&self, f: &SourceFile, o: &Options) -> Vec<Edit> {
        let (Some(lang), Some(tree)) = (f.language, f.tree) else {
            return Vec::new();
        };
        let lname = lang.name();
        let fn_style = style_for(o, "functions", lname, default_function_style(lname));
        let var_style = style_for(o, "variables", lname, default_variable_style(lname));
        let constants = o
            .get("allow-constants")
            .and_then(Value::as_bool)
            .unwrap_or(true);
        let ignore = super::opt_strings(o, "ignore");
        let mut found = Vec::new();
        let mut stack = vec![tree.root_node()];
        while let Some(n) = stack.pop() {
            declared(lang, n, &mut found);
            let mut c = n.walk();
            stack.extend(n.children(&mut c));
        }
        found.sort_by_key(|(n, _)| n.start_byte());
        found.dedup_by_key(|(n, _)| n.start_byte());
        let mut edits = Vec::new();
        for (n, kind) in found {
            let name = &f.text[n.byte_range()];
            if ignore.iter().any(|i| i == name) {
                continue;
            }
            let (style, what) = match kind {
                Kind::Function => (&fn_style, "function"),
                Kind::Variable => (&var_style, "variable"),
            };
            if matches_style(name, style)
                || (kind == Kind::Variable && constants && is_constant(name))
            {
                continue;
            }
            edits.push(report(
                f.text,
                n.start_byte(),
                n.end_byte(),
                format!("{what} {name:?} should be {style} case"),
            ));
        }
        edits
    }
}
