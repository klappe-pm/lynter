//! Offset and position helpers shared by the rules.

/// 1-based line and 1-based column (in characters) of a byte offset.
pub fn line_col(text: &str, offset: usize) -> (usize, usize) {
    let offset = offset.min(text.len());
    let head = &text[..offset];
    let line = head.bytes().filter(|&b| b == b'\n').count() + 1;
    let start = head.rfind('\n').map_or(0, |i| i + 1);
    (line, head[start..].chars().count() + 1)
}

/// Byte offset of a 1-based line and 0-based character column.
pub fn offset_of(text: &str, line: usize, col0: usize) -> usize {
    let mut start = 0;
    for _ in 1..line.max(1) {
        match text[start..].find('\n') {
            Some(i) => start += i + 1,
            None => return text.len(),
        }
    }
    let rest = &text[start..];
    let end = rest.find('\n').unwrap_or(rest.len());
    let line_text = &rest[..end];
    start
        + line_text
            .char_indices()
            .nth(col0)
            .map_or(line_text.len(), |(i, _)| i)
}
