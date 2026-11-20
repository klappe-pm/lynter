//! Case styles shared by the tag and folder rules.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaseStyle {
    Kebab,
    Snake,
    Camel,
    Lower,
}

impl CaseStyle {
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "kebab" | "kebab-case" => Self::Kebab,
            "snake" | "snake_case" => Self::Snake,
            "camel" | "camelCase" => Self::Camel,
            "lower" | "lowercase" => Self::Lower,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Kebab => "kebab",
            Self::Snake => "snake",
            Self::Camel => "camel",
            Self::Lower => "lower",
        }
    }

    /// Convert one name (no `/`) to this style.
    pub fn convert(self, s: &str) -> String {
        if self == Self::Lower {
            return s.to_lowercase();
        }
        let words = words(s);
        if words.is_empty() {
            return s.to_owned();
        }
        match self {
            Self::Kebab => words.join("-"),
            Self::Snake => words.join("_"),
            _ => words
                .iter()
                .enumerate()
                .map(|(i, w)| if i == 0 { w.clone() } else { capitalize(w) })
                .collect(),
        }
    }

    pub fn conforms(self, s: &str) -> bool {
        self.convert(s) == s
    }
}

fn capitalize(w: &str) -> String {
    let mut c = w.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// Lowercased words, split on separators and lower-to-upper boundaries.
fn words(s: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut prev: Option<char> = None;
    for c in s.chars() {
        if c.is_whitespace() || c == '_' || c == '-' {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur).to_lowercase());
            }
            prev = None;
            continue;
        }
        if c.is_uppercase() && prev.is_some_and(|p| p.is_lowercase() || p.is_numeric()) {
            out.push(std::mem::take(&mut cur).to_lowercase());
        }
        cur.push(c);
        prev = Some(c);
    }
    if !cur.is_empty() {
        out.push(cur.to_lowercase());
    }
    out
}
