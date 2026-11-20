//! Byte-range edits and their application.

/// Replace `[start, end)` of the source with `replacement`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edit {
    pub start: usize,
    pub end: usize,
    pub replacement: String,
    pub message: String,
}

impl Edit {
    pub fn new(
        start: usize,
        end: usize,
        replacement: impl Into<String>,
        message: impl Into<String>,
    ) -> Edit {
        Edit {
            start,
            end,
            replacement: replacement.into(),
            message: message.into(),
        }
    }
}

/// Sort edits by position and drop any that overlap an earlier one.
pub fn normalize(mut edits: Vec<Edit>) -> Vec<Edit> {
    edits.sort_by_key(|e| (e.start, e.end));
    let mut out: Vec<Edit> = Vec::with_capacity(edits.len());
    for e in edits {
        if let Some(prev) = out.last() {
            if e.start < prev.end || (e.start == prev.start && e.end == prev.end) {
                continue;
            }
        }
        out.push(e);
    }
    out
}

/// Apply non-overlapping, sorted edits as splices on the original source.
pub fn apply(text: &str, edits: &[Edit]) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last = 0;
    for e in edits {
        out.push_str(&text[last..e.start]);
        out.push_str(&e.replacement);
        last = e.end;
    }
    out.push_str(&text[last..]);
    out
}
