//! A small strict JSON parser that keeps byte spans and duplicate keys, which
//! `serde_json::Value` cannot.

pub struct Member {
    pub key: String,
    pub key_start: usize,
    pub key_end: usize,
    /// End of the member's value; the member spans `key_start..end`.
    pub end: usize,
    pub value: Node,
}

pub enum Node {
    Object { members: Vec<Member> },
    Array { items: Vec<Node> },
    Scalar,
}

pub struct ParseError {
    pub offset: usize,
    pub message: String,
}

struct P<'a> {
    s: &'a [u8],
    text: &'a str,
    i: usize,
}

const MAX_DEPTH: usize = 512;

pub fn parse(text: &str) -> Result<Node, ParseError> {
    let mut p = P {
        s: text.as_bytes(),
        text,
        i: 0,
    };
    p.ws();
    let n = p.value(0)?;
    p.ws();
    if p.i < p.s.len() {
        return Err(p.err("unexpected content after the top-level value"));
    }
    Ok(n)
}

impl P<'_> {
    fn err(&self, m: &str) -> ParseError {
        ParseError {
            offset: self.i.min(self.s.len()),
            message: m.to_owned(),
        }
    }

    fn ws(&mut self) {
        while matches!(self.s.get(self.i), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.i += 1;
        }
    }

    fn value(&mut self, depth: usize) -> Result<Node, ParseError> {
        if depth > MAX_DEPTH {
            return Err(self.err("nesting is too deep"));
        }
        match self.s.get(self.i) {
            None => Err(self.err("unexpected end of input, expected a value")),
            Some(b'{') => self.object(depth),
            Some(b'[') => self.array(depth),
            Some(b'"') => self.string().map(|_| Node::Scalar),
            Some(b't') => self.lit("true"),
            Some(b'f') => self.lit("false"),
            Some(b'n') => self.lit("null"),
            Some(b'-' | b'0'..=b'9') => self.number(),
            Some(_) => Err(self.err("unexpected character, expected a value")),
        }
    }

    fn lit(&mut self, w: &str) -> Result<Node, ParseError> {
        if self.s[self.i..].starts_with(w.as_bytes()) {
            self.i += w.len();
            Ok(Node::Scalar)
        } else {
            Err(self.err("invalid literal"))
        }
    }

    fn digits(&mut self) -> usize {
        let st = self.i;
        while matches!(self.s.get(self.i), Some(b'0'..=b'9')) {
            self.i += 1;
        }
        self.i - st
    }

    fn number(&mut self) -> Result<Node, ParseError> {
        if self.s[self.i] == b'-' {
            self.i += 1;
        }
        if self.s.get(self.i) == Some(&b'0') {
            self.i += 1;
        } else if self.digits() == 0 {
            return Err(self.err("invalid number"));
        }
        if self.s.get(self.i) == Some(&b'.') {
            self.i += 1;
            if self.digits() == 0 {
                return Err(self.err("digits expected after the decimal point"));
            }
        }
        if matches!(self.s.get(self.i), Some(b'e' | b'E')) {
            self.i += 1;
            if matches!(self.s.get(self.i), Some(b'+' | b'-')) {
                self.i += 1;
            }
            if self.digits() == 0 {
                return Err(self.err("digits expected in the exponent"));
            }
        }
        Ok(Node::Scalar)
    }

    fn string(&mut self) -> Result<String, ParseError> {
        self.i += 1;
        let mut out = String::new();
        loop {
            let Some(&b) = self.s.get(self.i) else {
                return Err(self.err("unterminated string"));
            };
            match b {
                b'"' => {
                    self.i += 1;
                    return Ok(out);
                }
                b'\\' => {
                    self.i += 1;
                    let Some(&e) = self.s.get(self.i) else {
                        return Err(self.err("unterminated escape"));
                    };
                    self.i += 1;
                    match e {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{8}'),
                        b'f' => out.push('\u{c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let hex = self
                                .text
                                .get(self.i..self.i + 4)
                                .filter(|h| h.bytes().all(|c| c.is_ascii_hexdigit()));
                            let Some(h) = hex else {
                                return Err(self.err("invalid \\u escape"));
                            };
                            let cp = u32::from_str_radix(h, 16).unwrap_or(0xFFFD);
                            self.i += 4;
                            out.push(char::from_u32(cp).unwrap_or('\u{FFFD}'));
                        }
                        _ => {
                            self.i -= 1;
                            return Err(self.err("invalid escape"));
                        }
                    }
                }
                0..=0x1f => return Err(self.err("control character in string")),
                _ => {
                    let ch = self.text[self.i..].chars().next().unwrap_or('\u{FFFD}');
                    out.push(ch);
                    self.i += ch.len_utf8();
                }
            }
        }
    }

    fn array(&mut self, depth: usize) -> Result<Node, ParseError> {
        self.i += 1;
        let mut items = Vec::new();
        self.ws();
        if self.s.get(self.i) == Some(&b']') {
            self.i += 1;
            return Ok(Node::Array { items });
        }
        loop {
            self.ws();
            items.push(self.value(depth + 1)?);
            self.ws();
            match self.s.get(self.i) {
                Some(b',') => self.i += 1,
                Some(b']') => {
                    self.i += 1;
                    return Ok(Node::Array { items });
                }
                _ => return Err(self.err("expected ',' or ']'")),
            }
        }
    }

    fn object(&mut self, depth: usize) -> Result<Node, ParseError> {
        self.i += 1;
        let mut members = Vec::new();
        self.ws();
        if self.s.get(self.i) == Some(&b'}') {
            self.i += 1;
            return Ok(Node::Object { members });
        }
        loop {
            self.ws();
            if self.s.get(self.i) != Some(&b'"') {
                return Err(self.err("expected a string key"));
            }
            let key_start = self.i;
            let key = self.string()?;
            let key_end = self.i;
            self.ws();
            if self.s.get(self.i) != Some(&b':') {
                return Err(self.err("expected ':' after the key"));
            }
            self.i += 1;
            self.ws();
            let value = self.value(depth + 1)?;
            members.push(Member {
                key,
                key_start,
                key_end,
                end: self.i,
                value,
            });
            self.ws();
            match self.s.get(self.i) {
                Some(b',') => self.i += 1,
                Some(b'}') => {
                    self.i += 1;
                    return Ok(Node::Object { members });
                }
                _ => return Err(self.err("expected ',' or '}'")),
            }
        }
    }
}
