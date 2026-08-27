// Copyright (c) 2026 DigiText. All rights reserved.
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine as _;
use rand::Rng;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};
const C_1Z: usize = 5;
const C_4: &[u8] = b"2346789ACDEFHKLMNPRTUX";
const TTL: Duration = Duration::from_secs(180);
const MAX_ENTRIES: usize = 256;
const C_8G: usize = 5;
const C_8F: usize = 7;
fn glyph(c: u8) -> Option<[&'static str; C_8F]> {
    Some(match c {
        b'2' => [
            ".###.", "#...#", "....#", "...#.", "..#..", ".#...", "#####",
        ],
        b'3' => [
            "#####", "....#", "...#.", "..##.", "....#", "#...#", ".###.",
        ],
        b'4' => [
            "...#.", "..##.", ".#.#.", "#..#.", "#####", "...#.", "...#.",
        ],
        b'6' => [
            "..##.", ".#...", "#....", "####.", "#...#", "#...#", ".###.",
        ],
        b'7' => [
            "#####", "....#", "...#.", "..#..", ".#...", ".#...", ".#...",
        ],
        b'8' => [
            ".###.", "#...#", "#...#", ".###.", "#...#", "#...#", ".###.",
        ],
        b'9' => [
            ".###.", "#...#", "#...#", ".####", "....#", "...#.", ".##..",
        ],
        b'A' => [
            "..#..", ".#.#.", "#...#", "#...#", "#####", "#...#", "#...#",
        ],
        b'C' => [
            ".###.", "#...#", "#....", "#....", "#....", "#...#", ".###.",
        ],
        b'D' => [
            "####.", "#...#", "#...#", "#...#", "#...#", "#...#", "####.",
        ],
        b'E' => [
            "#####", "#....", "#....", "####.", "#....", "#....", "#####",
        ],
        b'F' => [
            "#####", "#....", "#....", "####.", "#....", "#....", "#....",
        ],
        b'H' => [
            "#...#", "#...#", "#...#", "#####", "#...#", "#...#", "#...#",
        ],
        b'K' => [
            "#...#", "#..#.", "#.#..", "##...", "#.#..", "#..#.", "#...#",
        ],
        b'L' => [
            "#....", "#....", "#....", "#....", "#....", "#....", "#####",
        ],
        b'M' => [
            "#...#", "##.##", "#.#.#", "#.#.#", "#...#", "#...#", "#...#",
        ],
        b'N' => [
            "#...#", "##..#", "#.#.#", "#.#.#", "#..##", "#...#", "#...#",
        ],
        b'P' => [
            "####.", "#...#", "#...#", "####.", "#....", "#....", "#....",
        ],
        b'R' => [
            "####.", "#...#", "#...#", "####.", "#.#..", "#..#.", "#...#",
        ],
        b'T' => [
            "#####", "..#..", "..#..", "..#..", "..#..", "..#..", "..#..",
        ],
        b'U' => [
            "#...#", "#...#", "#...#", "#...#", "#...#", "#...#", ".###.",
        ],
        b'X' => [
            "#...#", "#...#", ".#.#.", "..#..", ".#.#.", "#...#", "#...#",
        ],
        _ => return None,
    })
}
const C_G9: f32 = 7.0;
const ADVANCE: f32 = 44.0;
const C_BJ: f32 = 28.0;
const IMG_H: f32 = 84.0;
const C_BZ: f32 = 13.0;
const C_C4: f32 = 7.0;
fn f_1hz() -> f32 {
    C_BJ * 2.0 + ADVANCE * (C_1Z as f32 - 1.0) + C_8G as f32 * C_G9
}
fn f_27m(r_d3: &str, r_d8: &mut impl Rng) -> String {
    let w = f_1hz();
    let mut r_da = String::with_capacity(8 * 1024);
    r_da.push_str(
        &format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {IMG_H}" width="{w}" height="{IMG_H}" role="img">"#
        ),
    );
    r_da.push_str(&format!(
        r#"<rect width="{w}" height="{IMG_H}" fill="hsl(255 25% 96%)"/>"#
    ));
    for _ in 0..4 {
        let (x1, y1) = (r_d8.gen_range(0.0..w * 0.3), r_d8.gen_range(0.0..IMG_H));
        let (x2, y2) = (r_d8.gen_range(w * 0.7..w), r_d8.gen_range(0.0..IMG_H));
        let (cx, cy) = (r_d8.gen_range(0.0..w), r_d8.gen_range(-20.0..IMG_H + 20.0));
        let hue = r_d8.gen_range(200..300);
        r_da.push_str(
            &format!(
                r#"<path d="M{x1:.1} {y1:.1} Q{cx:.1} {cy:.1} {x2:.1} {y2:.1}" fill="none" stroke="hsl({hue} 40% 62%)" stroke-width="{:.1}"/>"#,
                r_d8.gen_range(0.8..1.8)
            ),
        );
    }
    for (i, c) in r_d3.bytes().enumerate() {
        let Some(g) = glyph(c) else { continue };
        let r_d6 = C_BJ + ADVANCE * i as f32;
        let r_d7 = (IMG_H - C_8F as f32 * C_G9) / 2.0 + r_d8.gen_range(-3.0..3.0);
        let cx = r_d6 + C_8G as f32 * C_G9 / 2.0;
        let cy = r_d7 + C_8F as f32 * C_G9 / 2.0;
        let rot = r_d8.gen_range(-C_BZ..C_BZ);
        let skew = r_d8.gen_range(-C_C4..C_C4);
        let hue = r_d8.gen_range(230..280);
        let light = r_d8.gen_range(22..38);
        r_da.push_str(
            &format!(
                r#"<g transform="rotate({rot:.1} {cx:.1} {cy:.1}) translate({cx:.1} {cy:.1}) skewX({skew:.1}) translate({:.1} {:.1})" fill="hsl({hue} 55% {light}%)">"#,
                - cx, - cy
            ),
        );
        for (r_d9, r_d5) in g.iter().enumerate() {
            for (r_d4, r_d2) in r_d5.bytes().enumerate() {
                if r_d2 != b'#' {
                    continue;
                }
                let x = r_d6 + r_d4 as f32 * C_G9 + r_d8.gen_range(-0.9..0.9);
                let y = r_d7 + r_d9 as f32 * C_G9 + r_d8.gen_range(-0.9..0.9);
                let size = C_G9 + r_d8.gen_range(0.0..1.2);
                r_da.push_str(&format!(
                    r#"<rect x="{x:.1}" y="{y:.1}" width="{size:.1}" height="{size:.1}"/>"#
                ));
            }
        }
        r_da.push_str("</g>");
    }
    for _ in 0..70 {
        let x = r_d8.gen_range(0.0..w);
        let y = r_d8.gen_range(0.0..IMG_H);
        let r = r_d8.gen_range(0.4..1.6);
        let hue = r_d8.gen_range(200..300);
        r_da.push_str(&format!(
            r#"<circle cx="{x:.1}" cy="{y:.1}" r="{r:.1}" fill="hsl({hue} 45% 55%)"/>"#
        ));
    }
    r_da.push_str("</svg>");
    r_da
}
fn f_131(svg: &str) -> String {
    format!("data:image/svg+xml;base64,{}", B64.encode(svg.as_bytes()))
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct T2j {
    pub id: String,
    pub image: String,
}
struct T6v {
    answer: String,
    issued: Instant,
}
#[derive(Default)]
pub struct CaptchaStore(Mutex<HashMap<String, T6v>>);
fn normalize(r_db: &str) -> String {
    r_db.chars()
        .filter(|c| !c.is_whitespace())
        .flat_map(|c| c.to_uppercase())
        .collect()
}
impl CaptchaStore {
    pub fn issue(&self) -> T2j {
        self.issue_at(Instant::now())
    }
    pub fn issue_at(&self, r_de: Instant) -> T2j {
        let mut r_dg = rand::thread_rng();
        let r_dc: String = (0..C_1Z)
            .map(|_| C_4[r_dg.gen_range(0..C_4.len())] as char)
            .collect();
        let id: String = (0..16)
            .map(|_| format!("{:02x}", r_dg.gen::<u8>()))
            .collect();
        let image = f_131(&f_27m(&r_dc, &mut r_dg));
        if let Ok(mut r_dd) = self.0.lock() {
            r_dd.retain(|_, e| r_de.duration_since(e.issued) < TTL);
            while r_dd.len() >= MAX_ENTRIES {
                let r_df = r_dd
                    .iter()
                    .min_by_key(|(_, e)| e.issued)
                    .map(|(k, _)| k.clone());
                match r_df {
                    Some(k) => {
                        r_dd.remove(&k);
                    }
                    None => break,
                }
            }
            r_dd.insert(
                id.clone(),
                T6v {
                    answer: r_dc,
                    issued: r_de,
                },
            );
        }
        T2j { id, image }
    }
    pub fn verify(&self, id: &str, r_dh: &str) -> bool {
        self.verify_at(id, r_dh, Instant::now())
    }
    pub fn verify_at(&self, id: &str, r_dj: &str, r_dl: Instant) -> bool {
        let Ok(mut r_dk) = self.0.lock() else {
            return false;
        };
        let Some(r_di) = r_dk.remove(id) else {
            return false;
        };
        if r_dl.duration_since(r_di.issued) >= TTL {
            return false;
        }
        !r_dj.is_empty() && normalize(r_dj) == r_di.answer
    }
}
