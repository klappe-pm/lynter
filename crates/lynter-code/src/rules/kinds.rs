//! One per-language table of tree-sitter node kinds shared by the
//! cross-language rules. Every kind named here is checked against its grammar
//! by `tests/structure_rules.rs`, so a grammar upgrade that renames one fails a
//! test instead of silently disabling a rule.

use crate::language::Language;

pub struct Kinds {
    /// Comment nodes. Adjacent line comments are separate nodes.
    pub comments: &'static [&'static str],
    /// Text prefixes that make a comment a doc comment.
    pub doc_prefixes: &'static [&'static str],
    /// Declarations that own a body. `max-function-lines` measures these. For
    /// SQL this is the statement.
    pub functions: &'static [&'static str],
    /// Function-like nodes that are not measured but whose body is not a
    /// nesting level (lambdas, closures, class bodies).
    pub body_owners: &'static [&'static str],
    /// Nodes that add one nesting level. For SQL these are subqueries and CASE.
    pub blocks: &'static [&'static str],
    /// Statement-level nodes: a re-parsed comment that yields one of these
    /// and no errors counts as commented-out code.
    pub statements: &'static [&'static str],
}

const RUST: Kinds = Kinds {
    comments: &["line_comment", "block_comment"],
    doc_prefixes: &["///", "//!", "/**", "/*!"],
    functions: &["function_item"],
    body_owners: &["closure_expression", "function_signature_item"],
    blocks: &["block"],
    statements: &[
        "expression_statement",
        "let_declaration",
        "function_item",
        "struct_item",
        "enum_item",
        "impl_item",
        "use_declaration",
        "const_item",
        "static_item",
        "mod_item",
        "trait_item",
        "type_item",
        "macro_invocation",
    ],
};

const PYTHON: Kinds = Kinds {
    comments: &["comment"],
    doc_prefixes: &[],
    functions: &["function_definition"],
    body_owners: &["class_definition", "lambda"],
    blocks: &["block"],
    statements: &[
        "expression_statement",
        "return_statement",
        "if_statement",
        "for_statement",
        "while_statement",
        "try_statement",
        "with_statement",
        "import_statement",
        "import_from_statement",
        "raise_statement",
        "assert_statement",
        "pass_statement",
        "function_definition",
        "class_definition",
        "delete_statement",
        "break_statement",
        "continue_statement",
    ],
};

const JS_STATEMENTS: &[&str] = &[
    "expression_statement",
    "lexical_declaration",
    "variable_declaration",
    "return_statement",
    "if_statement",
    "for_statement",
    "for_in_statement",
    "while_statement",
    "do_statement",
    "try_statement",
    "switch_statement",
    "throw_statement",
    "import_statement",
    "export_statement",
    "function_declaration",
    "class_declaration",
    "break_statement",
    "continue_statement",
];

const TYPESCRIPT: Kinds = Kinds {
    comments: &["comment"],
    doc_prefixes: &["/**"],
    functions: &[
        "function_declaration",
        "generator_function_declaration",
        "method_definition",
        "function_expression",
    ],
    body_owners: &["arrow_function", "class_body"],
    blocks: &["statement_block"],
    statements: JS_STATEMENTS,
};

const JAVASCRIPT: Kinds = Kinds {
    comments: &["comment"],
    doc_prefixes: &["/**"],
    functions: &[
        "function_declaration",
        "generator_function_declaration",
        "method_definition",
        "function_expression",
    ],
    body_owners: &["arrow_function", "class_body"],
    blocks: &["statement_block"],
    statements: JS_STATEMENTS,
};

const GO: Kinds = Kinds {
    comments: &["comment"],
    doc_prefixes: &[],
    functions: &["function_declaration", "method_declaration"],
    body_owners: &["func_literal"],
    blocks: &["block"],
    statements: &[
        "short_var_declaration",
        "assignment_statement",
        "expression_statement",
        "return_statement",
        "if_statement",
        "for_statement",
        "var_declaration",
        "const_declaration",
        "inc_statement",
        "dec_statement",
        "go_statement",
        "defer_statement",
        "expression_switch_statement",
        "function_declaration",
        "import_declaration",
        "type_declaration",
    ],
};

const JAVA: Kinds = Kinds {
    comments: &["line_comment", "block_comment"],
    doc_prefixes: &["/**"],
    functions: &["method_declaration", "constructor_declaration"],
    body_owners: &["lambda_expression", "static_initializer"],
    blocks: &["block", "constructor_body"],
    statements: &[
        "expression_statement",
        "local_variable_declaration",
        "return_statement",
        "if_statement",
        "for_statement",
        "enhanced_for_statement",
        "while_statement",
        "do_statement",
        "try_statement",
        "switch_expression",
        "throw_statement",
        "import_declaration",
        "method_declaration",
        "class_declaration",
        "break_statement",
        "continue_statement",
    ],
};

const CSHARP: Kinds = Kinds {
    comments: &["comment"],
    doc_prefixes: &["///", "/**"],
    functions: &[
        "method_declaration",
        "constructor_declaration",
        "local_function_statement",
        "accessor_declaration",
    ],
    body_owners: &["lambda_expression", "anonymous_method_expression"],
    blocks: &["block"],
    statements: &[
        "expression_statement",
        "local_declaration_statement",
        "return_statement",
        "if_statement",
        "for_statement",
        "foreach_statement",
        "while_statement",
        "do_statement",
        "try_statement",
        "switch_statement",
        "throw_statement",
        "using_directive",
        "method_declaration",
        "class_declaration",
        "break_statement",
        "continue_statement",
        "local_function_statement",
    ],
};

const SQL: Kinds = Kinds {
    comments: &["comment", "marginalia"],
    doc_prefixes: &[],
    functions: &["statement"],
    body_owners: &[],
    blocks: &["subquery", "case"],
    statements: &["statement"],
};

pub fn kinds(lang: Language) -> &'static Kinds {
    match lang {
        Language::Rust => &RUST,
        Language::Python => &PYTHON,
        Language::TypeScript | Language::Tsx => &TYPESCRIPT,
        Language::JavaScript => &JAVASCRIPT,
        Language::Go => &GO,
        Language::Java => &JAVA,
        Language::CSharp => &CSHARP,
        Language::Sql => &SQL,
    }
}

impl Kinds {
    pub fn is_comment(&self, kind: &str) -> bool {
        self.comments.contains(&kind)
    }
    pub fn is_function(&self, kind: &str) -> bool {
        self.functions.contains(&kind)
    }
    pub fn is_block(&self, kind: &str) -> bool {
        self.blocks.contains(&kind)
    }
    pub fn is_statement(&self, kind: &str) -> bool {
        self.statements.contains(&kind)
    }
    /// True when a block directly under a node of this kind is the body of a
    /// function, lambda or class, not a nesting level.
    pub fn owns_body(&self, kind: &str) -> bool {
        self.functions.contains(&kind) || self.body_owners.contains(&kind)
    }
    pub fn is_doc_comment(&self, text: &str) -> bool {
        self.doc_prefixes.iter().any(|p| match *p {
            "///" => text.starts_with("///") && !text.starts_with("////"),
            "/**" => text.starts_with("/**") && !text.starts_with("/***") && text != "/**/",
            p => text.starts_with(p),
        })
    }
}

/// One source line of a comment with its markers removed.
pub struct CommentLine {
    /// Byte offset, within the comment text, of the first byte of `text`.
    pub offset: usize,
    pub text: String,
}

/// True when the comment text is a `//`, `#` or `--` line comment.
pub fn is_line_comment(text: &str) -> bool {
    !text.starts_with("/*")
        && (text.starts_with("//") || text.starts_with('#') || text.starts_with("--"))
}

/// Split a comment into lines with comment markers (`//`, `///`, `#`, `--`,
/// `/*`, `*/`, leading `*`) and one following space removed.
pub fn comment_lines(text: &str) -> Vec<CommentLine> {
    let mut out = Vec::new();
    if is_line_comment(text) {
        let mark = if text.starts_with('#') {
            1
        } else {
            let slashes = text.bytes().take_while(|&b| b == b'/' || b == b'-').count();
            slashes.min(if text.starts_with("--") { 2 } else { 3 })
        };
        let mut start = mark;
        if text[start..].starts_with(['!', ' ']) {
            start += 1;
        }
        out.push(CommentLine {
            offset: start,
            text: text[start..].to_owned(),
        });
        return out;
    }
    let mut pos = 0;
    let n_lines = text.split('\n').count();
    for (i, raw) in text.split('\n').enumerate() {
        let line_start = pos;
        pos += raw.len() + 1;
        let mut skip = 0;
        let mut body = raw;
        if i == 0 {
            skip = if body.starts_with("/*") { 2 } else { 0 };
            skip += body[skip..]
                .bytes()
                .take_while(|&b| b == b'*' || b == b'!')
                .count();
            body = &body[skip..];
        } else {
            let ws = body.len() - body.trim_start().len();
            let t = &body[ws..];
            if t.starts_with('*') && !t.starts_with("*/") {
                skip = ws + 1;
                body = &body[skip..];
            }
        }
        if i == n_lines - 1 {
            if let Some(stripped) = body.trim_end().strip_suffix("*/") {
                body = stripped;
            }
        }
        if body.starts_with(' ') {
            skip += 1;
            body = &body[1..];
        }
        out.push(CommentLine {
            offset: line_start + skip,
            text: body.to_owned(),
        });
    }
    out
}
