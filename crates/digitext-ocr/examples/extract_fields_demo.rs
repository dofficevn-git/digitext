// Copyright (c) 2026 DigiText. All rights reserved.
use digitext_ocr::fields::extract_fields;
use digitext_ocr::infer::Tgi;
use serde_json::Value;
fn main() {
    let r_2zt = std::env::args()
        .nth(1)
        .expect("Y+G6p24gxJHGsOG7nW5nIGThuqtuIEpTT04gZMOybmcgT0NS");
    let r_2zu = std::fs::read_to_string(&r_2zt).expect("xJHhu41jIGZpbGUgbOG7l2k=");
    let r_2zq: Vec<Value> =
        serde_json::from_str(&r_2zu).expect("SlNPTiBkw7JuZyBraMO0bmcgaOG7o3AgbOG7hw==");
    let mut r_2zl = Vec::new();
    let mut r_2zv = Vec::new();
    let mut r_2zm = Vec::new();
    for r_2zp in &r_2zq {
        let r_2zk = r_2zp["box"].as_array().expect("dGhp4bq/dSBib3g=");
        r_2zl.push([
            r_2zk[0].as_f64().unwrap() as f32,
            r_2zk[1].as_f64().unwrap() as f32,
            r_2zk[2].as_f64().unwrap() as f32,
            r_2zk[3].as_f64().unwrap() as f32,
        ]);
        r_2zv.push(r_2zp["text"].as_str().unwrap_or("").to_string());
        r_2zm.push(r_2zp["conf"].as_f64().unwrap_or(0.9) as f32);
    }
    let r_2zs = Tgi {
        boxes: r_2zl,
        texts: r_2zv,
        confidences: r_2zm,
    };
    let r_2zo = extract_fields(&r_2zs);
    let r_2zr: Vec<Value> = r_2zo
        .iter()
        .map(|r_2zn| {
            serde_json::json!(
                { "field" : r_2zn.field.as_str(), "value" : r_2zn.value, "bbox" : r_2zn
                .bbox, "confidence" : r_2zn.confidence, }
            )
        })
        .collect();
    println!("{}", serde_json::to_string(&r_2zr).unwrap());
}
