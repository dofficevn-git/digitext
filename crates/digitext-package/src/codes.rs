// Copyright (c) 2026 DigiText. All rights reserved.
pub const C_BD: u8 = 7;
pub const C_BE: &str = "01";
pub const C_CE: u8 = 3;
pub const C_AD: u8 = 11;
pub const C_FY: u8 = 3;
pub const C_29: u8 = 3;
pub const C_27: &str = "01";
pub const C_2A: &str = "02";
pub const C_28: &str = "03";
pub const C_IB: u8 = 2;
pub const C_NU: u8 = 32;
pub const C_NV: u8 = 2;
pub const C_NT: u8 = 2;
pub const C_26: u8 = 2;
pub const C_G8: u8 = 3;
pub const C_79: u8 = 10;
pub fn f_1jd(r_3wi: &str, r_3wj: u8) -> bool {
    if r_3wi.len() != 2 || !r_3wi.bytes().all(|r_3wh| r_3wh.is_ascii_digit()) {
        return false;
    }
    let r_3wk: u8 = r_3wi.parse().unwrap_or(0);
    (1..=r_3wj).contains(&r_3wk)
}
pub fn f_1ix(r_3wl: &str) -> bool {
    r_3wl == "0" || r_3wl == "1"
}
