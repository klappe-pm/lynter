//! An approximation of `String.prototype.localeCompare` under the root
//! collation, for case-folded list values: punctuation, then digits, then
//! letters; accents break ties after base letters.

use std::cmp::Ordering;

const PUNCT_ORDER: &str = "_-,;:!?.'\"()[]{}@*/\\&#%`^+<=>|~$";

fn fold_accent(c: char) -> (char, u8) {
    let base = match c {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' => 'a',
        'ç' | 'ć' | 'č' => 'c',
        'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ę' | 'ě' => 'e',
        'ì' | 'í' | 'î' | 'ï' | 'ī' => 'i',
        'ñ' | 'ń' | 'ň' => 'n',
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' => 'o',
        'ù' | 'ú' | 'û' | 'ü' | 'ū' | 'ů' => 'u',
        'ý' | 'ÿ' => 'y',
        'š' | 'ś' => 's',
        'ž' | 'ź' | 'ż' => 'z',
        other => return (other, 0),
    };
    (base, 1)
}

fn primary(c: char) -> (u8, u32) {
    if c.is_whitespace() {
        return (0, 0);
    }
    if let Some(i) = PUNCT_ORDER.chars().position(|p| p == c) {
        return (1, i as u32);
    }
    if c.is_ascii_digit() {
        return (3, c as u32);
    }
    if c.is_alphabetic() {
        return (4, c as u32);
    }
    (2, c as u32)
}

pub fn locale_compare(a: &str, b: &str) -> Ordering {
    let fa: Vec<(char, u8)> = a.chars().map(fold_accent).collect();
    let fb: Vec<(char, u8)> = b.chars().map(fold_accent).collect();
    let pa: Vec<(u8, u32)> = fa.iter().map(|(c, _)| primary(*c)).collect();
    let pb: Vec<(u8, u32)> = fb.iter().map(|(c, _)| primary(*c)).collect();
    pa.cmp(&pb)
        .then_with(|| {
            let sa: Vec<u8> = fa.iter().map(|x| x.1).collect();
            let sb: Vec<u8> = fb.iter().map(|x| x.1).collect();
            sa.cmp(&sb)
        })
        .then_with(|| a.cmp(b))
}
