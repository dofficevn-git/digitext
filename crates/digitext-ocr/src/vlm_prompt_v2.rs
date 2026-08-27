// Copyright (c) 2026 DigiText. All rights reserved.
use crate::vlm_fields::VlmError;
use digitext_core::VanBanField;
use serde_json::{json, Value};
pub const FIELD_SPECS_V2: [(VanBanField, &str); 16] = [
    (
        VanBanField::ThuTu,
        "U+G7kSB0aOG7qSB04buxIGPhu6dhIHbEg24gYuG6o24gdHJvbmcgaOG7kyBzxqEgKGNo4buJIGNo4buvIHPhu5Ep",
    ),
    (
        VanBanField::LoaiVanBan,
        "TG/huqFpIHbEg24gYuG6o24sIHbDrSBk4bulOiBRdXnhur90IMSR4buLbmgsIEPDtG5nIHbEg24sIFThu50gdHLDrG5oLCBCw6FvIGPDoW8gdMOgaSBjaMOtbmg=",
    ),
    (
        VanBanField::SoKyHieu,
        "U+G7kSB2w6Aga8O9IGhp4buHdSB2xINuIGLhuqNuLCB2w60gZOG7pTogOTk5OS9RxJAtVUJORA==",
    ),
    (
        VanBanField::NgayThang,
        "TmfDoHkgdGjDoW5nIGJhbiBow6BuaCwgxJHhu4tuaCBk4bqhbmcgZGQvbW0veXl5eQ==",
    ),
    (VanBanField::CoQuanBanHanh, "VMOqbiBjxqEgcXVhbiBiYW4gaMOgbmggdsSDbiBi4bqjbg=="),
    (
        VanBanField::TrichYeu,
        "VHLDrWNoIHnhur91IG7hu5lpIGR1bmcgdsSDbiBi4bqjbiAoY8OzIHRo4buDIG5oaeG7gXUgZMOybmcp",
    ),
    (VanBanField::ToSo, "VOG7nSBz4buRIChjaOG7iSBjaOG7ryBz4buRKQ=="),
    (VanBanField::SoTrang, "U+G7kSB0cmFuZyAoY2jhu4kgY2jhu68gc+G7kSk="),
    (VanBanField::TrangSo, "VHJhbmcgc+G7kSAoY2jhu4kgY2jhu68gc+G7kSk="),
    (
        VanBanField::NgonNgu,
        "TmfDtG4gbmfhu68gY+G7p2EgdsSDbiBi4bqjbiwgdsOtIGThu6U6IFRp4bq/bmcgVmnhu4d0",
    ),
    (VanBanField::CheDoSuDung, "Q2jhur8gxJHhu5kgc+G7rSBk4bulbmcgdMOgaSBsaeG7h3U="),
    (VanBanField::DoTinCay, "xJDhu5kgdGluIGPhuq15IGPhu6dhIHTDoGkgbGnhu4d1"),
    (VanBanField::ButTich, "QsO6dCB0w61jaCB0csOqbiB2xINuIGLhuqNu"),
    (
        VanBanField::TinhTrangVatLy,
        "VMOsbmggdHLhuqFuZyB24bqtdCBsw70gY+G7p2EgdMOgaSBsaeG7h3U=",
    ),
    (VanBanField::TaiLieuSo, "VGjDtG5nIHRpbiB0w6BpIGxp4buHdSBz4buR"),
    (VanBanField::GhiChu, "R2hpIGNow7o="),
];
pub fn f_24a() -> String {
    let mut r_3k2 = String::from(
        "QuG6oW4gbMOgIGjhu4cgdGjhu5FuZyBiw7NjIHTDoWNoIHRow7RuZyB0aW4gdsSDbiBi4bqjbiBow6BuaCBjaMOtbmggVmnhu4d0IE5hbS4KSMOjeSDEkeG7jWMg4bqjbmggdHJhbmcgdsSDbiBi4bqjbiB2w6AgdHLDrWNoIHh14bqldCAxNiB0csaw4budbmcgc2F1OgoK",
    );
    let r_3k1: Vec<String> = FIELD_SPECS_V2
        .iter()
        .map(|(k, r_3k0)| format!("- {}: {}", k.as_str(), r_3k0))
        .collect();
    r_3k2.push_str(&r_3k1.join("\n"));
    r_3k2
        .push_str(
            "CgpRVVkgVOG6rkMgQuG6rlQgQlXhu5hDOgoxLiBDSOG7iCB0csOtY2ggeHXhuqV0IGNo4buvIFRI4buwQyBT4buwIG5ow6xuIHRo4bqleSB0csOqbiB0cmFuZy4gVFVZ4buGVCDEkOG7kEkgS0jDlE5HIHN1eSDEkW/DoW4sIGtow7RuZyBi4buLYS4KMi4gVHLGsOG7nW5nIG7DoG8ga2jDtG5nIHh14bqldCBoaeG7h24gdHLDqm4gdHJhbmcgdGjDrCB0cuG6oyB24buBIGNodeG7l2kgcuG7l25nICIiLiBLaMO0bmcgdOG7sSBzdXkgZGnhu4VuLgozLiBHaeG7ryBOR1VZw4pOIFbEgk4gdGnhur9uZyBWaeG7h3QgY8OzIGThuqV1IMSRw7puZyBuaMawIHRyw6puIHRyYW5nLgo0LiBW4bubaSBz4buRLCBrw70gaGnhu4d1LCBuZ8OgeSB0aMOhbmc6IGNow6lwIGNow61uaCB4w6FjIHThu6tuZyBrw70gdOG7sSwga2jDtG5nIGNodeG6qW4gaG/DoSBs4bqhaS4KNS4gQuG7jiBRVUEgaG/DoG4gdG/DoG4gZOG6pXUgY2jhu68ga8O9IHPhu5EgLyBjb24gZOG6pXUg4bufIE3DiVAgdHJhbmcgKGTDsm5nIGLhuq90IMSR4bqndSBi4bqxbmcgIkvDvSBi4bufaToiLCAiVGjhu51pIGdpYW46IiwgdMOqbiBjxqEgcXVhbiBxdcOpdC9sxrB1IHRy4buvKS4gxJDDsyBLSMOUTkcgcGjhuqNpIG7hu5lpIGR1bmcgdsSDbiBi4bqjbi4KClFVWSBU4bquQyBSScOKTkcgVOG7qk5HIFRSxq/hu5xORyAoxJHhu41jIGvhu7kg4oCUIGhheSBuaOG6p20gbmjhuqV0KToKLSBjb1F1YW5CYW5IYW5oOiBM4bqkWSBjxqEgcXVhbiDhu58gR8OTQyBUUsOKTiBCw4pOIFRSw4FJICh0acOqdSDEkeG7gSB2xINuIGLhuqNuKS4gVFVZ4buGVCDEkOG7kEkgS0jDlE5HIGzhuqV5IGPGoSBxdWFuIHNhdSAiS8OtbmggZ+G7rWk6IiAoxJHDsyBsw6AgTsagSSBOSOG6rE4pLCBLSMOUTkcgbOG6pXkgIkPhu5hORyBIT8OAIFjDgyBI4buYSSBDSOG7piBOR0jEqEEgVknhu4ZUIE5BTSIgKMSRw7MgbMOgIHF14buRYyBoaeG7h3UpLCBLSMOUTkcgbOG6pXkgdMOqbiBjxqEgcXVhbiB0cm9uZyBk4bqldSBjaOG7ryBrw70gc+G7kSDhu58gbcOpcCwgS0jDlE5HIGzhuqV5IGPGoSBxdWFuIGJhbiBow6BuaCBN4bqqVSBCSeG7glUgKHZkICJj4bunYSBC4buZIFTDoGkgY2jDrW5oIiB0cm9uZyDDtCBN4bqrdSBz4buRKS4gxJDGoW4vaOG7o3AgxJHhu5NuZyBjw6EgbmjDom4ga2jDtG5nIGPDsyBjxqEgcXVhbiBiYW4gaMOgbmggdGjDrCDEkeG7gyBy4buXbmcuCi0gc29LeUhpZXU6IEzhuqRZIHPhu5Eg4bufIMSR4bqndSB2xINuIGLhuqNuLCBuZ2F5IGTGsOG7m2kgY8ahIHF1YW4gYmFuIGjDoG5oICh2ZCAiU+G7kTogMTIzL1HEkC1VQk5EIikuIEtIw5RORyBs4bqleSBz4buRIGPhu6dhIHbEg24gYuG6o24gS0jDgUMgxJHGsOG7o2Mgbmjhuq9jIHRyb25nIHRow6JuIGLDoGkgKCJDxINuIGPhu6kgQ8O0bmcgdsSDbiBz4buRLi4uIiwgIlRo4buxYyBoaeG7h24gdsSDbiBi4bqjbiBz4buRLi4uIikuCi0gbmdheVRoYW5nOiBM4bqkWSBuZ8OgeSB0csOqbiBkw7JuZyAiLi4uLiwgbmfDoHkgLi4uIHRow6FuZyAuLi4gbsSDbSAuLi4iIOG7nyDEkOG6plUgdsSDbiBi4bqjbiAobmfDoHkgYmFuIGjDoG5oKS4gS0jDlE5HIGzhuqV5IG5nw6B5IHRyb25nIGThuqV1ICJDViDEkOG6vk4gLyBDw5RORyBWxIJOIMSQ4bq+TiIgKG5nw6B5IG5o4bqtbiksIEtIw5RORyBs4bqleSBuZ8OgeSB0cm9uZyB0aMOibiBiw6BpLCBLSMOUTkcgbOG6pXkgbmfDoHkgc2luaCwgbmfDoHkgY+G6pXAgQ01ORCwgbmfDoHkga+G6v3QgdGjDumMgbsSDbSB0w6BpIGNow61uaCwgaGF5IHRo4budaSBnaWFuIGNo4buvIGvDvSBz4buRIOG7nyBtw6lwLgotIGxvYWlWYW5CYW46IEzhuqRZIHThu6sgRMOSTkcgVEnDilUgxJDhu4AgaW4gxJHhuq1tIGdp4buvYSB0cmFuZyAoUVVZ4bq+VCDEkOG7ik5ILCBU4bucIFRSw4xOSCwgQsOBTyBDw4FPLCBUSMOUTkcgQsOBTywgS+G6viBIT+G6oENILCBHSeG6pFkgUEjDiVAsIEdJ4bqkWSBDSOG7qE5HIE5I4bqsTiwgSOG7olAgxJDhu5JORywgQknDik4gQuG6ok4sIEPDlE5HIFbEgk4uLi4pLiBO4bq/dSBraMO0bmcgY8OzIGTDsm5nIHRpw6p1IMSR4buBLCBzdXkgdOG7qyBrw70gaGnhu4d1IChDVj1Dw7RuZyB2xINuLCBRxJA9UXV54bq/dCDEkeG7i25oLCBUVHI9VOG7nSB0csOsbmgsIEJDPULDoW8gY8OhbywgVEI9VGjDtG5nIGLDoW8sIEtIPUvhur8gaG/huqFjaCwgR1A9R2nhuqV5IHBow6lwKS4KLSB0cmljaFlldTogTOG6pFkgZMOybmcgIlYvdiAuLi4iIGhv4bq3YyAiVuG7gSB2aeG7h2MgLi4uIiBuZ2F5IGTGsOG7m2kgc+G7kS9rw70gaGnhu4d1IChob+G6t2MgZMOybmcgcGjhu6UgxJHhu4EgbmdheSBkxrDhu5tpIHRpw6p1IMSR4buBKS4gS0jDlE5HIGzhuqV5IGPDonUgxJHhuqd1IFRIw4JOIELDgEkgKCJDxINuIGPhu6kuLi4iLCAiVGjhu7FjIGhp4buHbi4uLiIsICJLw61uaCBn4butaS4uLiIsIHTDqm4gY8ahIHF1YW4vY8O0bmcgdHkpLiBO4bq/dSB2xINuIGLhuqNuIGtow7RuZyBjw7MgZMOybmcgdHLDrWNoIHnhur91IHRow6wgxJHhu4MgcuG7l25nLgoKQ2jhu4kgdHLhuqMgduG7gSBkdXkgbmjhuqV0IG3hu5l0IMSR4buRaSB0xrDhu6NuZyBKU09OLg==",
        );
    r_3k2
}
pub const QWEN_PATCH: u32 = 28;
pub const MODEL_MAX_LEN_V2: u32 = 4096;
pub const C_G2: u32 = 1122;
pub const C_JE: u32 = 64;
pub const MAX_IMAGE_TOKENS_V2: u32 = 1850;
pub fn image_tokens(width: u32, height: u32) -> u32 {
    ((width as f64 / QWEN_PATCH as f64) * (height as f64 / QWEN_PATCH as f64)) as u32
}
pub fn fit_max_side(width: u32, height: u32, budget: u32) -> Result<Option<u32>, VlmError> {
    if width == 0 || height == 0 {
        return Err(VlmError::Backend(format!(
            "a8OtY2ggdGjGsOG7m2Mg4bqjbmggdsO0IGzDvTog{width}x{height}"
        )));
    }
    if image_tokens(width, height) <= budget {
        return Ok(None);
    }
    let r_3k3 = (0.99 * budget as f64 * (QWEN_PATCH as f64).powi(2)
        / (width as f64 * height as f64))
        .sqrt();
    Ok(Some(((r_3k3 * width.max(height) as f64) as u32).max(1)))
}
pub fn output_budget(img_tokens: u32) -> Result<u32, VlmError> {
    let budget = MODEL_MAX_LEN_V2 as i64 - img_tokens as i64 - C_G2 as i64 - C_JE as i64;
    if budget <= 0 {
        return Err(
            VlmError::Backend(
                format!(
                    "4bqjbmgg{img_tokens}IHRva2VuIGtow7RuZyBjw7JuIGNo4buXIGNobyDEkeG6p3UgcmEgKG1heF9sZW49{MODEL_MAX_LEN_V2}KTsgcGjhuqNpIHRodSBuaOG7jyDhuqNuaCB0csaw4bubYw=="
                ),
            ),
        );
    }
    Ok(budget as u32)
}
pub fn guided_schema_v2_json() -> String {
    use std::fmt::Write;
    let mut props = String::new();
    let mut required = String::new();
    for (i, (k, r_3k4)) in FIELD_SPECS_V2.iter().enumerate() {
        if i > 0 {
            props.push(',');
            required.push(',');
        }
        let key =
            serde_json::to_string(k.as_str()).expect("a2hvw6EgbMOgIGNodeG7l2kgaOG7o3AgbOG7hw==");
        let desc =
            serde_json::to_string(r_3k4).expect("bcO0IHThuqMgbMOgIGNodeG7l2kgaOG7o3AgbOG7hw==");
        write!(
            props,
            "{key}:{{\"type\":\"string\",\"description\":{desc}}}"
        )
        .expect("ghi String");
        required.push_str(&key);
    }
    format!(
        "{{\"type\":\"object\",\"properties\":{{{props}}},\
         \"required\":[{required}],\"additionalProperties\":false}}"
    )
}
#[derive(serde::Serialize)]
pub struct Tiz {
    model: String,
    messages: Value,
    temperature: f64,
    max_tokens: u32,
    guided_json: Box<serde_json::value::RawValue>,
}
pub fn build_request_v2(model: &str, r_3k5: &str, max_tokens: u32) -> Tiz {
    Tiz {
        model: model.to_owned(),
        messages: json!(
            [{ "role" : "user", "content" : [{ "type" : "image_url", "image_url" : {
            "url" : r_3k5 } }, { "type" : "text", "text" : f_24a() },] },]
        ),
        temperature: 0.0,
        max_tokens,
        guided_json: serde_json::value::RawValue::from_string(guided_schema_v2_json())
            .expect("Z3VpZGVkX3NjaGVtYV92Ml9qc29uIGzDoCBKU09OIGjhu6NwIGzhu4c="),
    }
}
pub fn parse_fields_v2(r_3ka: &Value) -> Vec<(VanBanField, String)> {
    let Some(content) = r_3ka
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|r_3k7| r_3k7.get("content"))
        .and_then(Value::as_str)
    else {
        return Vec::new();
    };
    let trimmed = content.trim();
    let trimmed = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .unwrap_or(trimmed)
        .trim_end_matches("```")
        .trim();
    let Ok(r_3k8) = serde_json::from_str::<Value>(trimmed) else {
        return Vec::new();
    };
    let mut r_3k9 = Vec::new();
    for r_3k6 in VanBanField::ALL {
        let Some(value) = r_3k8.get(r_3k6.as_str()).and_then(Value::as_str) else {
            continue;
        };
        let value = value.trim();
        if value.is_empty() {
            continue;
        }
        r_3k9.push((r_3k6, digitext_vi::normalize(value)));
    }
    r_3k9
}
