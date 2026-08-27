// Copyright (c) 2026 DigiText. All rights reserved.
use unicode_normalization::UnicodeNormalization;
fn f_1j3(c: char) -> bool {
    matches!(
        c,
        '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{FEFF}' | '\u{00AD}'
    )
}
pub fn f_x2(s: &str) -> String {
    let mut r_1zz = String::with_capacity(s.len());
    let mut r_1zy = s.nfc().peekable();
    while let Some(c) = r_1zy.next() {
        match c {
            c if f_1j3(c) => {}
            '\r' => {
                if r_1zy.peek() == Some(&'\n') {
                    r_1zy.next();
                }
                r_1zz.push('\n');
            }
            '\n' | '\u{2028}' | '\u{2029}' => r_1zz.push('\n'),
            c if c.is_whitespace() => r_1zz.push(' '),
            c => r_1zz.push(c),
        }
    }
    let r_200: Vec<String> = r_1zz
        .split('\n')
        .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect();
    let mut r_201: Vec<String> = Vec::with_capacity(r_200.len());
    for l in r_200 {
        if l.is_empty() && r_201.last().is_some_and(|r_202| r_202.is_empty()) {
            continue;
        }
        r_201.push(l);
    }
    while r_201.first().is_some_and(|l| l.is_empty()) {
        r_201.remove(0);
    }
    while r_201.last().is_some_and(|l| l.is_empty()) {
        r_201.pop();
    }
    r_201.join("\n")
}
pub const C_Y: f64 = 2.0;
pub fn f_ue(a: Option<&[f64]>, b: Option<&[f64]>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(r_205), Some(r_206)) => {
            r_205.len() == 4
                && r_206.len() == 4
                && r_205
                    .iter()
                    .zip(r_206.iter())
                    .all(|(r_203, r_204)| (r_203 - r_204).abs() <= C_Y)
        }
        _ => false,
    }
}
