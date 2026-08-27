// Copyright (c) 2026 DigiText. All rights reserved.
use digitext_pdf::{f_vz, Tgh, Tny};
fn span(r_46m: &str, r_46n: f32) -> Tny {
    Tny {
        text: r_46m.to_owned(),
        x: 100.0,
        y: r_46n,
        width: 600.0,
        height: 40.0,
    }
}
fn main() {
    let out = std::env::args()
        .nth(1)
        .expect("ZMO5bmc6IGR1bXBfcGRmIDzEkcaw4budbmctZOG6q24tcmEucGRmPg==");
    let r_46p = Tgh {
        jpeg: vec![0xFF, 0xD8, 0xFF, 0xD9],
        width_px: 1000,
        height_px: 1400,
        dpi: 200,
        spans: vec![
            span("4bumeSBiYW4gbmjDom4gZMOibiB04buJbmggVGjDoWkgTmd1ecOqbg==", 50.0),
            span("aOG7kyBzxqEgc+G7kSAxMjMvUcSQLVVCTkQg4oCUIMSQ4buZYyBs4bqtcCBU4buxIGRvIEjhuqFuaCBwaMO6Yw==",
            120.0),
        ],
    };
    let r_46o = f_vz(&[r_46p]).expect("build pdf");
    std::fs::write(&out, r_46o).expect("ghi file");
    eprintln!("xJHDoyBnaGkg{out}");
}
