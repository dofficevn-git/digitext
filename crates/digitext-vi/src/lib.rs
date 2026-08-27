// Copyright (c) 2026 DigiText. All rights reserved.
#![forbid(unsafe_code)]
use std::collections::{HashMap, HashSet};
use unicode_normalization::char::is_combining_mark;
use unicode_normalization::UnicodeNormalization;
const C_MO: [char; 5] = ['\u{0300}', '\u{0301}', '\u{0303}', '\u{0309}', '\u{0323}'];
fn f_1jm(r_5n9: char) -> bool {
    C_MO.contains(&r_5n9)
}
pub fn f_2hu(r_5nb: &str) -> String {
    r_5nb
        .nfd()
        .filter(|c| !is_combining_mark(*c))
        .map(|c| match c {
            'đ' => 'd',
            'Đ' => 'D',
            r_5na => r_5na,
        })
        .collect()
}
fn f_x5(r_5nd: &[char]) -> Vec<char> {
    let mut r_5nh: Vec<char> = Vec::with_capacity(r_5nd.len());
    let mut r_5ne = 0;
    while r_5ne < r_5nd.len() {
        if r_5ne + 2 < r_5nd.len() && f_1jm(r_5nd[r_5ne + 2]) {
            let (r_5nk, r_5nl) = (r_5nd[r_5ne], r_5nd[r_5ne + 1]);
            let r_5ng = r_5nd
                .get(r_5ne + 3)
                .is_none_or(|r_5nc| !r_5nc.is_alphabetic());
            let r_5nf = matches!(r_5nk, 'o' | 'O') && matches!(r_5nl, 'a' | 'e' | 'A' | 'E');
            let r_5nj = matches!(r_5nk, 'u' | 'U') && matches!(r_5nl, 'y' | 'Y');
            let r_5ni = r_5nj && matches!(r_5nh.last(), Some('q' | 'Q'));
            if r_5ng && (r_5nf || r_5nj) && !r_5ni {
                r_5nh.push(r_5nk);
                r_5nh.push(r_5nd[r_5ne + 2]);
                r_5nh.push(r_5nl);
                r_5ne += 3;
                continue;
            }
        }
        r_5nh.push(r_5nd[r_5ne]);
        r_5ne += 1;
    }
    r_5nh
}
pub fn normalize(r_5no: &str) -> String {
    let r_5nm: Vec<char> = r_5no.nfc().collect::<String>().nfd().collect();
    let r_5nn = f_x5(&r_5nm);
    r_5nn.into_iter().collect::<String>().nfc().collect()
}
const C_48: f32 = 0.85;
#[derive(Debug, Default, Clone)]
pub struct Lexicon {
    canonical: HashSet<String>,
    by_stripped: HashMap<String, Vec<String>>,
}
impl Lexicon {
    pub fn from_words<I, S>(r_5ns: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut r_5nq = Lexicon::default();
        for w in r_5ns {
            let r_5nr = normalize(w.as_ref());
            if r_5nr.is_empty() {
                continue;
            }
            let r_5np = f_2hu(&r_5nr).to_lowercase();
            r_5nq
                .by_stripped
                .entry(r_5np)
                .or_default()
                .push(r_5nr.clone());
            r_5nq.canonical.insert(r_5nr);
        }
        r_5nq
    }
    pub fn contains(&self, r_5nw: &str) -> bool {
        let r_5nv = normalize(r_5nw);
        if self.canonical.contains(&r_5nv) {
            return true;
        }
        let r_5nu = r_5nv.to_lowercase();
        self.canonical
            .iter()
            .any(|r_5nt| r_5nt.to_lowercase() == r_5nu)
    }
    pub fn correct_token(&self, r_5o3: &str, r_5o0: f32) -> (String, bool) {
        let r_5o2 = normalize(r_5o3);
        if r_5o0 >= C_48 || self.contains(&r_5o2) {
            return (r_5o2, false);
        }
        let r_5o1 = f_2hu(&r_5o2).to_lowercase();
        if let Some(r_5nz) = self.by_stripped.get(&r_5o1) {
            if let Some(r_5nx) = r_5nz.iter().min_by_key(|r_5ny| f_2lf(&r_5o2, r_5ny)) {
                if r_5nx.as_str() != r_5o2.as_str() {
                    return (r_5nx.clone(), true);
                }
            }
        }
        (r_5o2, false)
    }
}
fn f_2lf(a: &str, r_5o4: &str) -> usize {
    a.chars()
        .zip(r_5o4.chars())
        .filter(|(r_5o5, y)| r_5o5.to_lowercase().ne(y.to_lowercase()))
        .count()
        + a.chars().count().abs_diff(r_5o4.chars().count())
}
