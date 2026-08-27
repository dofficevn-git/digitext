// Copyright (c) 2026 DigiText. All rights reserved.
use digitext_pdf::FormBlock;
use serde::{Deserialize, Serialize};
fn field(r_u: &str, r_v: impl Into<String>) -> FormBlock {
    FormBlock::Field {
        label: r_u.to_string(),
        value: r_v.into(),
    }
}
fn opt(r_w: &str, r_x: &Option<String>) -> Option<FormBlock> {
    r_x.as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(|v| field(r_w, v))
}
fn f_2f9(r_y: &str, r_z: &str) -> FormBlock {
    FormBlock::Signatures {
        left: vec![
            r_y.to_string(),
            "KEvDvSwgZ2hpIHLDtSBo4buNIHTDqm4p".to_string(),
        ],
        right: vec![
            r_z.to_string(),
            "KEvDvSwgZ2hpIHLDtSBo4buNIHTDqm4sIMSRw7NuZyBk4bqldSk=".to_string(),
        ],
    }
}
fn f_1ga(don_vi: &str, dia_diem: &str, ngay: &str) -> Vec<FormBlock> {
    vec![
        FormBlock::Paragraph(format!("SMO0bSBuYXksIG5nw6B5IA=={ngay}LCB04bqhaSA={dia_diem}, {don_vi}IHRp4bq/biBow6BuaCBs4bqtcCBiacOqbiBi4bqjbiB24bubaSBu4buZaSBkdW5nIHNhdTo="))
    ]
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TepLoi {
    pub ten_tep: String,
    pub dia_chi: String,
    pub mo_ta_loi: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tab {
    pub don_vi: String,
    pub dia_diem: String,
    pub ngay: String,
    pub thanh_phan: Vec<String>,
    pub pham_vi: String,
    pub tong_so_tep: u64,
    pub so_dat: u64,
    pub so_loi: u64,
    #[serde(default)]
    pub tep_loi: Vec<TepLoi>,
    pub phuong_an_khac_phuc: Option<String>,
    pub ket_luan: Option<String>,
    pub nguoi_lap: String,
    pub thu_truong: String,
}
fn f_27h(r_10: &Tab) -> (String, Vec<FormBlock>) {
    let mut b = f_1ga(&r_10.don_vi, &r_10.dia_diem, &r_10.ngay);
    b.push(FormBlock::SectionHeading("SS4gVEjDgE5IIFBI4bqmTg==".into()));
    b.push(FormBlock::Bullets(r_10.thanh_phan.clone()));
    b.push(FormBlock::SectionHeading(
        "SUkuIFBI4bqgTSBWSSBWw4AgTuG7mEkgRFVORyBLSeG7gk0gVFJB".into(),
    ));
    b.push(field("UGjhuqFtIHZpIGtp4buDbSB0cmE=", r_10.pham_vi.clone()));
    b.push(
        FormBlock::Paragraph(
            "TuG7mWkgZHVuZyBraeG7g20gdHJhIHRoZW8ga2hv4bqjbiAzIMSQaeG7gXUgMjk6IChhKSBraOG6oyBuxINuZyB0aeG6v3AgY+G6rW4sIHPhu60gZOG7pW5nLCBj4bqtcCBuaOG6rXQgY+G7p2EgdGjDtG5nIHRpbiB24buBIGThu68gbGnhu4d1IGNo4bunOyAoYikgcXXDoSB0csOsbmggYuG6o28gcXXhuqNuIHbDoCBi4bqjbyDEkeG6o20gYW4gdG/DoG4gdGjDtG5nIHRpbjsgKGMpIHTDrW5oIGNow61uaCB4w6FjIHRow7RuZyB0aW4gduG7gSBk4buvIGxp4buHdSBjaOG7pyB24bubaSB04buHcCB0aW4gdMOgaSBsaeG7h3UgbMawdSB0cuG7ryBrw6htIHRoZW8u"
                .into(),
        ),
    );
    b.push(FormBlock::SectionHeading(
        "SUlJLiBL4bq+VCBRVeG6oiBLSeG7gk0gVFJB".into(),
    ));
    b.push(field(
        "VOG7lW5nIHPhu5EgdOG7h3Aga2nhu4NtIHRyYQ==",
        r_10.tong_so_tep.to_string(),
    ));
    b.push(field("U+G7kSB04buHcCDEkeG6oXQ=", r_10.so_dat.to_string()));
    b.push(field("U+G7kSB04buHcCBs4buXaQ==", r_10.so_loi.to_string()));
    if !r_10.tep_loi.is_empty() {
        b.push(
            FormBlock::Paragraph(
                "RGFuaCBt4bulYyB04buHcCBraeG7g20gdHJhIGLhu4sgbOG7l2kgdsOgIMSR4buLYSBjaOG7iSBsxrB1IHRy4buvICjEkGnhu4F1IDI5LjQuxJEpOg=="
                    .into(),
            ),
        );
        b.push(FormBlock::Bullets(
            r_10.tep_loi
                .iter()
                .map(|r_11| match &r_11.mo_ta_loi {
                    Some(m) => format!("{} — {} ({m})", r_11.ten_tep, r_11.dia_chi),
                    None => format!("{} — {}", r_11.ten_tep, r_11.dia_chi),
                })
                .collect(),
        ));
    }
    if let Some(x) = opt(
        "UGjGsMahbmcgw6FuIGto4bqvYyBwaOG7pWMgbOG7l2kgKMSQaeG7gXUgMjkuNC5nKQ==",
        &r_10.phuong_an_khac_phuc,
    ) {
        b.push(x);
    }
    if let Some(x) = opt("S+G6v3QgbHXhuq1u", &r_10.ket_luan) {
        b.push(x);
    }
    b.push(f_2f9(
        "TkfGr+G7nEkgTOG6rFAgQknDik4gQuG6ok4=",
        "VEjhu6YgVFLGr+G7nk5HIMSQxqBOIFbhu4o=",
    ));
    (
        "QmnDqm4gYuG6o24ga2nhu4NtIHRyYSB0w6BpIGxp4buHdSBsxrB1IHRy4buvIHPhu5E=".into(),
        b,
    )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Te6 {
    pub thoi_gian: String,
    pub loai: String,
    pub nguon_du_lieu: String,
    pub phuong_tien: String,
    pub ket_qua: String,
    pub nguoi_thuc_hien: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Te7 {
    pub don_vi: String,
    pub ky: String,
    pub entries: Vec<Te6>,
    pub nguoi_lap: String,
}
fn f_27j(r_12: &Te7) -> (String, Vec<FormBlock>) {
    let mut b = vec![
        field("xJDGoW4gduG7iw==", r_12.don_vi.clone()),
        field("S+G7syBnaGkgbmjhuq10IGvDvQ==", r_12.ky.clone()),
        FormBlock::SectionHeading("Q8OBQyBM4bqmTiBTQU8gTMavVQ==".into()),
    ];
    b.push(
        FormBlock::Bullets(
            r_12
                .entries
                .iter()
                .map(|e| {
                    format!(
                        "{} | {}IHwgbmd14buTbjog{}IHwgcGjGsMahbmcgdGnhu4duOiA={}IHwga+G6v3QgcXXhuqM6IA=={}IHwgbmfGsOG7nWkgdGjhu7FjIGhp4buHbjog{}",
                        e.thoi_gian, e.loai, e.nguon_du_lieu, e.phuong_tien, e.ket_qua, e
                        .nguoi_thuc_hien
                    )
                })
                .collect(),
        ),
    );
    b.push(f_2f9(
        "TkfGr+G7nEkgTOG6rFAgTkjhuqxUIEvDnQ==",
        "UEjhu6QgVFLDgUNIIMSQxqBOIFbhu4o=",
    ));
    (
        "Tmjhuq10IGvDvSBzYW8gbMawdSB0w6BpIGxp4buHdSBsxrB1IHRy4buvIHPhu5E=".into(),
        b,
    )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tkj {
    pub don_vi: String,
    pub dia_diem: String,
    pub ngay: String,
    pub loai_sao_luu: String,
    pub nguon_du_lieu: String,
    pub phuong_tien: String,
    pub mo_ta_phuong_tien: Option<String>,
    pub ket_qua_kiem_tra: String,
    pub noi_bao_quan: String,
    pub nguoi_thuc_hien: String,
    pub nguoi_kiem_tra: String,
}
fn f_27k(r_13: &Tkj) -> (String, Vec<FormBlock>) {
    let mut b = f_1ga(&r_13.don_vi, &r_13.dia_diem, &r_13.ngay);
    b.push(field(
        "TG/huqFpIHNhbyBsxrB1ICjEkGnhu4F1IDMwLjEp",
        r_13.loai_sao_luu.clone(),
    ));
    b.push(
        field(
            "Tmd14buTbiBk4buvIGxp4buHdSBzYW8gbMawdSAoQ1NETCArIG3DoyBuZ3Xhu5NuIHBo4bqnbiBt4buBbSDigJQgxJBp4buBdSAzMC4zLmMp",
            r_13.nguon_du_lieu.clone(),
        ),
    );
    b.push(field(
        "UGjGsMahbmcgdGnhu4duIGzGsHUgdHLhu68=",
        r_13.phuong_tien.clone(),
    ));
    if let Some(r_14) = opt("TcO0IHThuqMgcGjGsMahbmcgdGnhu4du", &r_13.mo_ta_phuong_tien) {
        b.push(r_14);
    }
    b.push(field(
        "S+G6v3QgcXXhuqMga2nhu4NtIHRyYSBzYXUgc2FvIGzGsHUgKMSQaeG7gXUgMzAuMy5kKQ==",
        r_13.ket_qua_kiem_tra.clone(),
    ));
    b.push(field(
        "TsahaSBi4bqjbyBxdeG6o24gcGjGsMahbmcgdGnhu4duICjEkGnhu4F1IDMwLjMuxJEp",
        r_13.noi_bao_quan.clone(),
    ));
    b.push(f_2f9(
        "TkfGr+G7nEkgVEjhu7BDIEhJ4buGTiBTQU8gTMavVQ==",
        "TkfGr+G7nEkgS0nhu4JNIFRSQQ==",
    ));
    (
        "QmnDqm4gYuG6o24gc2FvIGzGsHUgdMOgaSBsaeG7h3UgbMawdSB0cuG7ryBz4buR".into(),
        b,
    )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tm4 {
    pub don_vi: String,
    pub dia_diem: String,
    pub ngay: String,
    pub thoi_diem_su_co: String,
    pub mo_ta_su_co: String,
    pub nguyen_nhan: String,
    pub bien_phap: Vec<String>,
    pub nguon_phuc_hoi: String,
    pub phuong_phap_phuc_hoi: String,
    pub ket_qua_kiem_tra: String,
    pub nguoi_xu_ly: String,
    pub phu_trach: String,
}
fn f_27l(r_15: &Tm4) -> (String, Vec<FormBlock>) {
    let mut b = f_1ga(&r_15.don_vi, &r_15.dia_diem, &r_15.ngay);
    b.push(FormBlock::SectionHeading("SS4gU+G7sCBD4buQ".into()));
    b.push(field(
        "VGjhu51pIMSRaeG7g20geOG6o3kgcmEgc+G7sSBj4buR",
        r_15.thoi_diem_su_co.clone(),
    ));
    b.push(field("TcO0IHThuqMgc+G7sSBj4buR", r_15.mo_ta_su_co.clone()));
    b.push(field(
        "Tmd1ecOqbiBuaMOibiAoxJBp4buBdSAzMS4xKQ==",
        r_15.nguyen_nhan.clone(),
    ));
    b.push(FormBlock::SectionHeading(
        "SUkuIFjhu6wgTMOdIFbDgCBQSOG7pEMgSOG7kkk=".into(),
    ));
    b.push(
        FormBlock::Paragraph(
            "Q8OhYyBiaeG7h24gcGjDoXAgY8OhY2ggbHksIGto4bqvYyBwaOG7pWMgxJHDoyB0aOG7sWMgaGnhu4duICjEkGnhu4F1IDMxLjIvMzEuMyk6"
                .into(),
        ),
    );
    b.push(FormBlock::Bullets(r_15.bien_phap.clone()));
    b.push(
        field(
            "Tmd14buTbiBwaOG7pWMgaOG7k2kgKGLhuqNuIHNhbyBsxrB1IGfhuqduIG5o4bqldCDigJQgxJBp4buBdSAzMS40KQ==",
            r_15.nguon_phuc_hoi.clone(),
        ),
    );
    b.push(field(
        "UGjGsMahbmcgcGjDoXAgcGjhu6VjIGjhu5Np",
        r_15.phuong_phap_phuc_hoi.clone(),
    ));
    b.push(field(
        "S+G6v3QgcXXhuqMga2nhu4NtIHRyYSBk4buvIGxp4buHdSBzYXUgcGjhu6VjIGjhu5NpICjEkGnhu4F1IDMxLjUp",
        r_15.ket_qua_kiem_tra.clone(),
    ));
    b.push(f_2f9(
        "TkfGr+G7nEkgWOG7rCBMw50gU+G7sCBD4buQ",
        "UEjhu6QgVFLDgUNIIMSQxqBOIFbhu4o=",
    ));
    (
        "QmnDqm4gYuG6o24geOG7rSBsw70gc+G7sSBj4buRIHbDoCBwaOG7pWMgaOG7k2kgY8ahIHPhu58gZOG7ryBsaeG7h3UgdMOgaSBsaeG7h3UgbMawdSB0cuG7rw=="
            .into(),
        b,
    )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct To3 {
    pub ma_dang_ky: String,
    pub ma_co_quan: String,
    pub ten_co_quan: String,
    pub so_lan_nop: i32,
    pub tong_dang_ky: i64,
    pub tong_thu_nop: i64,
    pub tong_duoc_duyet: i64,
    pub tong_tra_lai: i64,
    pub thoi_gian_thu_nop: String,
    pub nguoi_thu_nop: String,
    pub ghi_chu: Option<String>,
}
fn f_27n(r_16: &To3) -> (String, Vec<FormBlock>) {
    let mut b = vec![
        field(
            "TcOjIMSRxINuZyBrw70gecOqdSBj4bqndSB0aHUgbuG7mXA=",
            r_16.ma_dang_ky.clone(),
        ),
        field(
            "TcOjIGPGoSBxdWFuLCB04buVIGNo4bupYyBu4buZcA==",
            r_16.ma_co_quan.clone(),
        ),
        field(
            "VMOqbiBjxqEgcXVhbiwgdOG7lSBjaOG7qWMgbuG7mXA=",
            r_16.ten_co_quan.clone(),
        ),
        field("U+G7kSBs4bqnbiBu4buZcA==", r_16.so_lan_nop.to_string()),
        field(
            "VOG7lW5nIHPhu5EgaOG7kyBzxqEsIHTDoGkgbGnhu4d1IG7hu5lwIHRoZW8gxJHEg25nIGvDvQ==",
            r_16.tong_dang_ky.to_string(),
        ),
        field(
            "VOG7lW5nIHPhu5EgaOG7kyBzxqEsIHTDoGkgbGnhu4d1IHRodSBu4buZcCB0aGVvIMSRxINuZyBrw70=",
            r_16.tong_thu_nop.to_string(),
        ),
        field(
            "VOG7lW5nIHPhu5EgxJHGsOG7o2MgZHV54buHdA==",
            r_16.tong_duoc_duyet.to_string(),
        ),
        field(
            "VOG7lW5nIHPhu5EgdHLhuqMgbOG6oWk=",
            r_16.tong_tra_lai.to_string(),
        ),
        field(
            "VGjhu51pIGdpYW4gdGh1IG7hu5lw",
            r_16.thoi_gian_thu_nop.clone(),
        ),
        field("TmfGsOG7nWkgdGh1IG7hu5lw", r_16.nguoi_thu_nop.clone()),
    ];
    if let Some(r_17) = opt("R2hpIGNow7o=", &r_16.ghi_chu) {
        b.push(r_17);
    }
    b.push(f_2f9(
        "xJDhuqBJIERJ4buGTiBDxqAgUVVBTiBO4buYUA==",
        "xJDhuqBJIERJ4buGTiBMxq9VIFRS4buuIEzhu4pDSCBT4bus",
    ));
    (
        "QmnDqm4gYuG6o24gdGh1IG7hu5lwIGjhu5Mgc8ahLCB0w6BpIGxp4buHdSB2w6BvIGzGsHUgdHLhu68gbOG7i2NoIHPhu60="
            .into(),
        b,
    )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct To2 {
    pub ten: String,
    pub tinh_trang: String,
    pub mo_ta: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct To5 {
    pub ma_dang_ky: Option<String>,
    pub ma_co_quan: String,
    pub ten_co_quan: String,
    pub so_lan_nop: i32,
    pub tong_dang_ky: i64,
    pub tong_giao_nhan: i64,
    pub tong_dung_luong: String,
    #[serde(default)]
    pub thiet_bi: Vec<To2>,
    pub thoi_gian_giao_nhan: String,
    pub nguoi_giao: String,
    pub nguoi_nhan: String,
    pub ghi_chu: Option<String>,
}
fn f_27p(r_18: &To5) -> (String, Vec<FormBlock>) {
    let mut b = Vec::new();
    if let Some(x) = opt(
        "TcOjIMSRxINuZyBrw70gecOqdSBj4bqndSB0aHUgbuG7mXAgKG7hur91IGPDsyk=",
        &r_18.ma_dang_ky,
    ) {
        b.push(x);
    }
    b.push(field(
        "TcOjIGPGoSBxdWFuLCB04buVIGNo4bupYyBu4buZcA==",
        r_18.ma_co_quan.clone(),
    ));
    b.push(field(
        "VMOqbiBjxqEgcXVhbiwgdOG7lSBjaOG7qWMgbuG7mXA=",
        r_18.ten_co_quan.clone(),
    ));
    b.push(field(
        "U+G7kSBs4bqnbiBu4buZcA==",
        r_18.so_lan_nop.to_string(),
    ));
    b.push(field(
        "VOG7lW5nIHPhu5EgaOG7kyBzxqEsIHTDoGkgbGnhu4d1IG7hu5lwIHRoZW8gxJHEg25nIGvDvQ==",
        r_18.tong_dang_ky.to_string(),
    ));
    b.push(field(
        "VOG7lW5nIHPhu5EgaOG7kyBzxqEsIHTDoGkgbGnhu4d1IGdpYW8gbmjhuq1u",
        r_18.tong_giao_nhan.to_string(),
    ));
    b.push(field(
        "VOG7lW5nIHPhu5EgZHVuZyBsxrDhu6NuZw==",
        r_18.tong_dung_luong.clone(),
    ));
    b.push(field(
        "VOG7lW5nIHPhu5EgdGhp4bq/dCBi4buLIGzGsHUgdHLhu68=",
        r_18.thiet_bi.len().to_string(),
    ));
    if !r_18.thiet_bi.is_empty() {
        b.push(
            FormBlock::Paragraph(
                "VMOsbmggdHLhuqFuZywgbcO0IHThuqMgY2hpIHRp4bq/dCB0aGnhur90IGLhu4sgKMSQaeG7gXUgMjIuMi7EkSk6"
                    .into(),
            ),
        );
        b.push(FormBlock::Bullets(
            r_18.thiet_bi
                .iter()
                .map(|r_19| match &r_19.mo_ta {
                    Some(m) => format!("{} — {} ({m})", r_19.ten, r_19.tinh_trang),
                    None => format!("{} — {}", r_19.ten, r_19.tinh_trang),
                })
                .collect(),
        ));
    }
    b.push(field(
        "VGjhu51pIGdpYW4gZ2lhbyBuaOG6rW4=",
        r_18.thoi_gian_giao_nhan.clone(),
    ));
    b.push(field("TmfGsOG7nWkgZ2lhbw==", r_18.nguoi_giao.clone()));
    b.push(field("TmfGsOG7nWkgbmjhuq1u", r_18.nguoi_nhan.clone()));
    if let Some(x) = opt("R2hpIGNow7oga2jDoWM=", &r_18.ghi_chu) {
        b.push(x);
    }
    b.push(f_2f9("TkfGr+G7nEkgR0lBTw==", "TkfGr+G7nEkgTkjhuqxO"));
    (
        "QmnDqm4gYuG6o24gdGh1IG7hu5lwIGjhu5Mgc8ahLCB0w6BpIGxp4buHdSAoZ2lhbyBuaOG6rW4gdHLhu7FjIHRp4bq/cCk="
            .into(),
        b,
    )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T8q {
    pub ma_dang_ky: String,
    pub ma_co_quan: String,
    pub ten_co_quan: String,
    pub so_lan_nop: i32,
    pub tong_giao_nhan: i64,
    pub tong_dung_luong: Option<String>,
    pub thoi_gian: String,
    pub ben_giao: String,
    pub ben_nhan: String,
    pub ghi_chu: Option<String>,
}
fn f_27f(r_1a: &T8q) -> (String, Vec<FormBlock>) {
    let mut b = vec![
        field("TcOjIMSRxINuZyBrw70gecOqdSBj4bqndSB0aHUgbuG7mXA=", r_1a.ma_dang_ky
        .clone()), field("TcOjIGPGoSBxdWFuLCB04buVIGNo4bupYyBu4buZcA==", r_1a.ma_co_quan
        .clone()), field("VMOqbiBjxqEgcXVhbiwgdOG7lSBjaOG7qWMgbuG7mXA=", r_1a.ten_co_quan
        .clone()), field("U+G7kSBs4bqnbiBu4buZcA==", r_1a.so_lan_nop.to_string()),
        field("VOG7lW5nIHPhu5EgaOG7kyBzxqEsIHTDoGkgbGnhu4d1IGdpYW8gbmjhuq1uIHbDoG8gbMawdSB0cuG7ryBs4buLY2ggc+G7rQ==",
        r_1a.tong_giao_nhan.to_string(),),
    ];
    if let Some(x) = opt(
        "VOG7lW5nIHPhu5EgZHVuZyBsxrDhu6NuZw==",
        &r_1a.tong_dung_luong,
    ) {
        b.push(x);
    }
    b.push(field(
        "VGjhu51pIGdpYW4gZ2lhbyBuaOG6rW4=",
        r_1a.thoi_gian.clone(),
    ));
    b.push(field(
        "QsOqbiBnaWFvIChjxqEgcXVhbiwgdOG7lSBjaOG7qWMgbuG7mXAp",
        r_1a.ben_giao.clone(),
    ));
    b.push(field(
        "QsOqbiBuaOG6rW4gKGzGsHUgdHLhu68gbOG7i2NoIHPhu60p",
        r_1a.ben_nhan.clone(),
    ));
    if let Some(x) = opt("R2hpIGNow7oga2jDoWM=", &r_1a.ghi_chu) {
        b.push(x);
    }
    b.push(f_2f9(
        "xJDhuqBJIERJ4buGTiBDxqAgUVVBTiBO4buYUA==",
        "xJDhuqBJIERJ4buGTiBMxq9VIFRS4buuIEzhu4pDSCBT4bus",
    ));
    (
        "QmnDqm4gYuG6o24gZ2lhbyBuaOG6rW4gaOG7kyBzxqEsIHTDoGkgbGnhu4d1IHbDoG8gbMawdSB0cuG7ryBs4buLY2ggc+G7rQ=="
            .into(),
        b,
    )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Toc {
    pub ma_ho_so: String,
    pub tieu_de: String,
    pub ly_do: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tob {
    pub ma_dang_ky: String,
    pub ma_co_quan: String,
    pub ten_co_quan: String,
    pub so_lan_nop: i32,
    pub tong_tra_lai: i64,
    #[serde(default)]
    pub danh_sach: Vec<Toc>,
    pub phuong_thuc_tra: String,
    pub thoi_gian: String,
    pub ben_giao: String,
    pub ben_nhan: String,
    pub ghi_chu: Option<String>,
}
fn f_27r(r_1b: &Tob) -> (String, Vec<FormBlock>) {
    let mut b = vec![
        field(
            "TcOjIMSRxINuZyBrw70gecOqdSBj4bqndSB0aHUgbuG7mXA=",
            r_1b.ma_dang_ky.clone(),
        ),
        field(
            "TcOjIGPGoSBxdWFuLCB04buVIGNo4bupYyBu4buZcA==",
            r_1b.ma_co_quan.clone(),
        ),
        field(
            "VMOqbiBjxqEgcXVhbiwgdOG7lSBjaOG7qWMgbuG7mXA=",
            r_1b.ten_co_quan.clone(),
        ),
        field("U+G7kSBs4bqnbiBu4buZcA==", r_1b.so_lan_nop.to_string()),
        field(
            "VOG7lW5nIHPhu5EgaOG7kyBzxqEsIHTDoGkgbGnhu4d1IHRy4bqjIGzhuqFp",
            r_1b.tong_tra_lai.to_string(),
        ),
    ];
    if !r_1b.danh_sach.is_empty() {
        b.push(
            FormBlock::Paragraph(
                "RGFuaCBzw6FjaCBo4buTIHPGoSwgdMOgaSBsaeG7h3Uga2jDtG5nIMSRxrDhu6NjIHBow6ogZHV54buHdCAoxJBp4buBdSAyNS4xLmEpOg=="
                    .into(),
            ),
        );
        b.push(FormBlock::Bullets(
            r_1b.danh_sach
                .iter()
                .map(|r_1c| match &r_1c.ly_do {
                    Some(l) => {
                        format!("{} — {} (lý do: {l})", r_1c.ma_ho_so, r_1c.tieu_de)
                    }
                    None => format!("{} — {}", r_1c.ma_ho_so, r_1c.tieu_de),
                })
                .collect(),
        ));
    }
    b.push(field(
        "UGjGsMahbmcgdGjhu6ljIHRy4bqjIGzhuqFpICjEkGnhu4F1IDI1LjQp",
        r_1b.phuong_thuc_tra.clone(),
    ));
    b.push(field(
        "VGjhu51pIGdpYW4gdHLhuqMgbOG6oWk=",
        r_1b.thoi_gian.clone(),
    ));
    b.push(field(
        "QsOqbiBnaWFvIChsxrB1IHRy4buvIGzhu4tjaCBz4butKQ==",
        r_1b.ben_giao.clone(),
    ));
    b.push(field(
        "QsOqbiBuaOG6rW4gKGPGoSBxdWFuLCB04buVIGNo4bupYyBu4buZcCk=",
        r_1b.ben_nhan.clone(),
    ));
    if let Some(r_1d) = opt("R2hpIGNow7oga2jDoWM=", &r_1b.ghi_chu) {
        b.push(r_1d);
    }
    b.push(f_2f9(
        "xJDhuqBJIERJ4buGTiBMxq9VIFRS4buuIEzhu4pDSCBT4bus",
        "xJDhuqBJIERJ4buGTiBDxqAgUVVBTiBO4buYUA==",
    ));
    (
        "QmnDqm4gYuG6o24gaOG7kyBzxqEsIHTDoGkgbGnhu4d1IHRy4bqjIGzhuqFp".into(),
        b,
    )
}
pub fn f_2i2(r_1e: &str) -> &'static str {
    match r_1e {
        "kiem_tra" => {
            "KE3huqt1IHRoZW8gTeG7pWMgMSBQaOG7pSBs4bulYyBWSUkg4oCUIFRow7RuZyB0xrAgMDUvMjAyNS9UVC1CTlYsIMSQaeG7gXUgMjkp"
        }
        "nhat_ky_sao_luu" => {
            "KE3huqt1IHRoZW8gTeG7pWMgMiBQaOG7pSBs4bulYyBWSUkg4oCUIFRow7RuZyB0xrAgMDUvMjAyNS9UVC1CTlYsIMSQaeG7gXUgMzAp"
        }
        "sao_luu" => {
            "KE3huqt1IHRoZW8gTeG7pWMgMyBQaOG7pSBs4bulYyBWSUkg4oCUIFRow7RuZyB0xrAgMDUvMjAyNS9UVC1CTlYsIMSQaeG7gXUgMzAp"
        }
        "su_co" => {
            "KE3huqt1IHRoZW8gTeG7pWMgNCBQaOG7pSBs4bulYyBWSUkg4oCUIFRow7RuZyB0xrAgMDUvMjAyNS9UVC1CTlYsIMSQaeG7gXUgMzEp"
        }
        "thu_nop" => {
            "KFRoZW8ga2hv4bqjbiAyIMSQaeG7gXUgMjUg4oCUIFRow7RuZyB0xrAgMDUvMjAyNS9UVC1CTlYp"
        }
        "thu_nop_truc_tiep" => {
            "KFRoZW8gxJFp4buDbSDEkSBraG/huqNuIDIgxJBp4buBdSAyMiDigJQgVGjDtG5nIHTGsCAwNS8yMDI1L1RULUJOVik="
        }
        "giao_nhan" => {
            "KFRoZW8gxJFp4buDbSBhIGtob+G6o24gNSDEkGnhu4F1IDI1IOKAlCBUaMO0bmcgdMawIDA1LzIwMjUvVFQtQk5WKQ=="
        }
        "tra_lai" => {
            "KFRoZW8gxJFp4buDbSBhIGtob+G6o24gNSDEkGnhu4F1IDI1IOKAlCBUaMO0bmcgdMawIDA1LzIwMjUvVFQtQk5WKQ=="
        }
        _ => "",
    }
}
pub fn render(r_1f: &str, payload: &serde_json::Value) -> Result<(String, Vec<FormBlock>), String> {
    fn de<T: serde::de::DeserializeOwned>(r_1g: &serde_json::Value) -> Result<T, String> {
        serde_json::from_value(r_1g.clone())
            .map_err(|e| format!("cGF5bG9hZCBraMO0bmcgaOG7o3AgbOG7hzog{e}"))
    }
    match r_1f {
        "kiem_tra" => Ok(f_27h(&de::<Tab>(payload)?)),
        "nhat_ky_sao_luu" => Ok(f_27j(&de::<Te7>(payload)?)),
        "sao_luu" => Ok(f_27k(&de::<Tkj>(payload)?)),
        "su_co" => Ok(f_27l(&de::<Tm4>(payload)?)),
        "thu_nop" => Ok(f_27n(&de::<To3>(payload)?)),
        "thu_nop_truc_tiep" => Ok(f_27p(&de::<To5>(payload)?)),
        "giao_nhan" => Ok(f_27f(&de::<T8q>(payload)?)),
        "tra_lai" => Ok(f_27r(&de::<Tob>(payload)?)),
        other => Err(format!(
            "bG/huqFpIGJpw6puIGLhuqNuIGtow7RuZyBo4bujcCBs4buHOiA={other}"
        )),
    }
}
