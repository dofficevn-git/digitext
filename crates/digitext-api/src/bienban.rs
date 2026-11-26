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
            "(Ký, ghi rõ họ tên)".to_string(),
        ],
        right: vec![
            r_z.to_string(),
            "(Ký, ghi rõ họ tên, đóng dấu)".to_string(),
        ],
    }
}
fn f_1ga(don_vi: &str, dia_diem: &str, ngay: &str) -> Vec<FormBlock> {
    vec![
        FormBlock::Paragraph(format!("Hôm nay, ngày {ngay}, tại {dia_diem}, {don_vi} tiến hành lập biên bản với nội dung sau:"))
    ]
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TepLoi {
    pub ten_tep: String,
    pub dia_chi: String,
    pub mo_ta_loi: Option<String>,
    /// 0032: mã hồ sơ của tệp và biện pháp đã khắc phục (Điều 29.4.đ, g).
    #[serde(default)]
    pub ma_ho_so: Option<String>,
    #[serde(default)]
    pub khac_phuc: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tab {
    pub don_vi: String,
    pub dia_diem: String,
    pub ngay: String,
    pub thanh_phan: Vec<String>,
    /// 0032 (Điều 29.2): kế hoạch kiểm tra, kỳ, phương pháp (toàn bộ / lấy mẫu), thời gian thực hiện.
    #[serde(default)]
    pub ke_hoach: Option<String>,
    #[serde(default)]
    pub ky: Option<String>,
    #[serde(default)]
    pub phuong_phap: Option<String>,
    #[serde(default)]
    pub thoi_gian: Option<String>,
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
    b.push(FormBlock::SectionHeading("I. THÀNH PHẦN".into()));
    b.push(FormBlock::Bullets(r_10.thanh_phan.clone()));
    b.push(FormBlock::SectionHeading(
        "II. PHẠM VI VÀ NỘI DUNG KIỂM TRA".into(),
    ));
    if let Some(x) = opt("Kế hoạch kiểm tra (Điều 29.2)", &r_10.ke_hoach) {
        b.push(x);
    }
    if let Some(x) = opt("Kỳ kiểm tra", &r_10.ky) {
        b.push(x);
    }
    b.push(field("Phạm vi kiểm tra", r_10.pham_vi.clone()));
    if let Some(x) = opt("Phương pháp", &r_10.phuong_phap) {
        b.push(x);
    }
    if let Some(x) = opt("Thời gian thực hiện", &r_10.thoi_gian) {
        b.push(x);
    }
    b.push(FormBlock::Paragraph("Nội dung kiểm tra theo khoản 3 Điều 29:".into()));
    b.push(FormBlock::Bullets(vec![
        "(a) Khả năng tiếp cận, sử dụng, cập nhật của thông tin về dữ liệu chủ: dữ liệu chủ (mục lục hồ sơ, văn bản) đọc được, tìm được, sửa được bằng phần mềm.".into(),
        "(b) Quá trình bảo quản và bảo đảm an toàn thông tin: tệp còn trong kho, mã băm khớp lúc nhập; sao lưu theo lịch; quyền truy cập đúng vai trò.".into(),
        "(c) Tính chính xác thông tin về dữ liệu chủ với tệp tin tài liệu lưu trữ kèm theo: số trang khớp mục lục, tệp mở được, đạt PDF/A.".into(),
    ]));
    b.push(FormBlock::SectionHeading(
        "III. KẾT QUẢ KIỂM TRA".into(),
    ));
    b.push(field(
        "Tổng số tệp kiểm tra",
        r_10.tong_so_tep.to_string(),
    ));
    b.push(field("Số tệp đạt", r_10.so_dat.to_string()));
    b.push(field("Số tệp lỗi", r_10.so_loi.to_string()));
    if !r_10.tep_loi.is_empty() {
        b.push(
            FormBlock::Paragraph(
                "Danh mục tệp kiểm tra bị lỗi và địa chỉ lưu trữ (Điều 29.4.đ):"
                    .into(),
            ),
        );
        b.push(FormBlock::Table {
            columns: vec!["STT".into(), "Tên tệp".into(), "Mã hồ sơ".into(), "Địa chỉ lưu trữ".into(), "Mô tả lỗi".into(), "Khắc phục".into()],
            rows: r_10
                .tep_loi
                .iter()
                .enumerate()
                .map(|(i, r_11)| {
                    vec![
                        (i + 1).to_string(),
                        r_11.ten_tep.clone(),
                        r_11.ma_ho_so.clone().unwrap_or_default(),
                        r_11.dia_chi.clone(),
                        r_11.mo_ta_loi.clone().unwrap_or_default(),
                        r_11.khac_phuc.clone().unwrap_or_default(),
                    ]
                })
                .collect(),
        });
    }
    if let Some(x) = opt(
        "Phương án khắc phục lỗi (Điều 29.4.g)",
        &r_10.phuong_an_khac_phuc,
    ) {
        b.push(x);
    }
    if let Some(x) = opt("Kết luận", &r_10.ket_luan) {
        b.push(x);
    }
    b.push(f_2f9(
        "NGƯỜI LẬP BIÊN BẢN",
        "THỦ TRƯỞNG ĐƠN VỊ",
    ));
    (
        "Biên bản kiểm tra tài liệu lưu trữ số".into(),
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
    /// 0032: dung lượng, số hiệu phương tiện, nơi bảo quản (Điều 30.3).
    #[serde(default)]
    pub dung_luong: Option<String>,
    #[serde(default)]
    pub so_hieu_phuong_tien: Option<String>,
    #[serde(default)]
    pub noi_bao_quan: Option<String>,
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
        field("Đơn vị", r_12.don_vi.clone()),
        field("Kỳ ghi nhật ký", r_12.ky.clone()),
        FormBlock::SectionHeading("CÁC LẦN SAO LƯU".into()),
    ];
    // 0032: bảng thật thay gạch đầu dòng nối " | " (có dung lượng, số hiệu phương tiện, nơi bảo quản khi biên bản mới ghi).
    let mo_rong = r_12.entries.iter().any(|e| e.dung_luong.is_some() || e.so_hieu_phuong_tien.is_some() || e.noi_bao_quan.is_some());
    let mut columns = vec!["Thời gian".to_string(), "Loại".into(), "Nguồn dữ liệu".into(), "Phương tiện".into()];
    if mo_rong {
        columns.extend(["Dung lượng".to_string(), "Số hiệu PT".into(), "Nơi bảo quản".into()]);
    }
    columns.extend(["Kết quả".to_string(), "Người thực hiện".into()]);
    b.push(FormBlock::Table {
        columns,
        rows: r_12
            .entries
            .iter()
            .map(|e| {
                let mut r = vec![e.thoi_gian.clone(), e.loai.clone(), e.nguon_du_lieu.clone(), e.phuong_tien.clone()];
                if mo_rong {
                    r.extend([e.dung_luong.clone().unwrap_or_default(), e.so_hieu_phuong_tien.clone().unwrap_or_default(), e.noi_bao_quan.clone().unwrap_or_default()]);
                }
                r.extend([e.ket_qua.clone(), e.nguoi_thuc_hien.clone()]);
                r
            })
            .collect(),
    });
    b.push(f_2f9(
        "NGƯỜI LẬP NHẬT KÝ",
        "PHỤ TRÁCH ĐƠN VỊ",
    ));
    (
        "Nhật ký sao lưu tài liệu lưu trữ số".into(),
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
    /// 0032: thời gian sao lưu, dung lượng, mã băm / cách đọc lại, hạn lưu bản sao (Điều 30.3).
    #[serde(default)]
    pub thoi_gian: Option<String>,
    #[serde(default)]
    pub dung_luong: Option<String>,
    #[serde(default)]
    pub ma_bam: Option<String>,
    #[serde(default)]
    pub han_luu: Option<String>,
}
fn f_27k(r_13: &Tkj) -> (String, Vec<FormBlock>) {
    let mut b = f_1ga(&r_13.don_vi, &r_13.dia_diem, &r_13.ngay);
    b.push(field(
        "Loại sao lưu (Điều 30.1)",
        r_13.loai_sao_luu.clone(),
    ));
    b.push(
        field(
            "Nguồn dữ liệu sao lưu (CSDL + mã nguồn phần mềm — Điều 30.3.c)",
            r_13.nguon_du_lieu.clone(),
        ),
    );
    b.push(field(
        "Phương tiện lưu trữ",
        r_13.phuong_tien.clone(),
    ));
    if let Some(r_14) = opt("Mô tả phương tiện", &r_13.mo_ta_phuong_tien) {
        b.push(r_14);
    }
    if let Some(x) = opt("Thời gian sao lưu", &r_13.thoi_gian) {
        b.push(x);
    }
    if let Some(x) = opt("Dung lượng bản sao", &r_13.dung_luong) {
        b.push(x);
    }
    if let Some(x) = opt("Mã băm / cách kiểm đọc lại", &r_13.ma_bam) {
        b.push(x);
    }
    b.push(field(
        "Kết quả kiểm tra sau sao lưu (Điều 30.3.d)",
        r_13.ket_qua_kiem_tra.clone(),
    ));
    if let Some(x) = opt("Hạn lưu bản sao", &r_13.han_luu) {
        b.push(x);
    }
    b.push(field(
        "Nơi bảo quản phương tiện (Điều 30.3.đ)",
        r_13.noi_bao_quan.clone(),
    ));
    b.push(f_2f9(
        "NGƯỜI THỰC HIỆN SAO LƯU",
        "NGƯỜI KIỂM TRA",
    ));
    (
        "Biên bản sao lưu tài liệu lưu trữ số".into(),
        b,
    )
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tm4 {
    pub don_vi: String,
    pub dia_diem: String,
    pub ngay: String,
    /// 0032 (Điều 31): mã sự cố, mức độ, phạm vi ảnh hưởng, dữ liệu mất, tệp kiểm sau phục hồi.
    #[serde(default)]
    pub ma_su_co: Option<String>,
    #[serde(default)]
    pub muc_do: Option<String>,
    #[serde(default)]
    pub pham_vi: Option<String>,
    #[serde(default)]
    pub du_lieu_mat: Option<String>,
    #[serde(default)]
    pub tep_kiem_sau_phuc_hoi: Option<String>,
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
    b.push(FormBlock::SectionHeading("I. SỰ CỐ".into()));
    if let Some(x) = opt("Mã sự cố", &r_15.ma_su_co) {
        b.push(x);
    }
    b.push(field(
        "Thời điểm xảy ra sự cố",
        r_15.thoi_diem_su_co.clone(),
    ));
    b.push(field("Mô tả sự cố", r_15.mo_ta_su_co.clone()));
    if let Some(x) = opt("Mức độ", &r_15.muc_do) {
        b.push(x);
    }
    if let Some(x) = opt("Phạm vi ảnh hưởng", &r_15.pham_vi) {
        b.push(x);
    }
    b.push(field(
        "Nguyên nhân (Điều 31.1)",
        r_15.nguyen_nhan.clone(),
    ));
    b.push(FormBlock::SectionHeading(
        "II. XỬ LÝ VÀ PHỤC HỒI".into(),
    ));
    b.push(
        FormBlock::Paragraph(
            "Các biện pháp cách ly, khắc phục đã thực hiện (Điều 31.2/31.3):"
                .into(),
        ),
    );
    b.push(FormBlock::Bullets(r_15.bien_phap.clone()));
    b.push(
        field(
            "Nguồn phục hồi (bản sao lưu gần nhất — Điều 31.4)",
            r_15.nguon_phuc_hoi.clone(),
        ),
    );
    b.push(field(
        "Phương pháp phục hồi",
        r_15.phuong_phap_phuc_hoi.clone(),
    ));
    b.push(field(
        "Kết quả kiểm tra dữ liệu sau phục hồi (Điều 31.5)",
        r_15.ket_qua_kiem_tra.clone(),
    ));
    if let Some(x) = opt("Dữ liệu mất (nếu có)", &r_15.du_lieu_mat) {
        b.push(x);
    }
    if let Some(x) = opt("Tệp đã kiểm sau phục hồi", &r_15.tep_kiem_sau_phuc_hoi) {
        b.push(x);
    }
    b.push(f_2f9(
        "NGƯỜI XỬ LÝ SỰ CỐ",
        "PHỤ TRÁCH ĐƠN VỊ",
    ));
    (
        "Biên bản xử lý sự cố và phục hồi cơ sở dữ liệu tài liệu lưu trữ"
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
            "Mã đăng ký yêu cầu thu nộp",
            r_16.ma_dang_ky.clone(),
        ),
        field(
            "Mã cơ quan, tổ chức nộp",
            r_16.ma_co_quan.clone(),
        ),
        field(
            "Tên cơ quan, tổ chức nộp",
            r_16.ten_co_quan.clone(),
        ),
        field("Số lần nộp", r_16.so_lan_nop.to_string()),
        field(
            "Tổng số hồ sơ, tài liệu nộp theo đăng ký",
            r_16.tong_dang_ky.to_string(),
        ),
        field(
            "Tổng số hồ sơ, tài liệu thu nộp theo đăng ký",
            r_16.tong_thu_nop.to_string(),
        ),
        field(
            "Tổng số được duyệt",
            r_16.tong_duoc_duyet.to_string(),
        ),
        field(
            "Tổng số trả lại",
            r_16.tong_tra_lai.to_string(),
        ),
        field(
            "Thời gian thu nộp",
            r_16.thoi_gian_thu_nop.clone(),
        ),
        field("Người thu nộp", r_16.nguoi_thu_nop.clone()),
    ];
    if let Some(r_17) = opt("Ghi chú", &r_16.ghi_chu) {
        b.push(r_17);
    }
    b.push(f_2f9(
        "ĐẠI DIỆN CƠ QUAN NỘP",
        "ĐẠI DIỆN LƯU TRỮ LỊCH SỬ",
    ));
    (
        "Biên bản thu nộp hồ sơ, tài liệu vào lưu trữ lịch sử"
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
        "Mã đăng ký yêu cầu thu nộp (nếu có)",
        &r_18.ma_dang_ky,
    ) {
        b.push(x);
    }
    b.push(field(
        "Mã cơ quan, tổ chức nộp",
        r_18.ma_co_quan.clone(),
    ));
    b.push(field(
        "Tên cơ quan, tổ chức nộp",
        r_18.ten_co_quan.clone(),
    ));
    b.push(field(
        "Số lần nộp",
        r_18.so_lan_nop.to_string(),
    ));
    b.push(field(
        "Tổng số hồ sơ, tài liệu nộp theo đăng ký",
        r_18.tong_dang_ky.to_string(),
    ));
    b.push(field(
        "Tổng số hồ sơ, tài liệu giao nhận",
        r_18.tong_giao_nhan.to_string(),
    ));
    b.push(field(
        "Tổng số dung lượng",
        r_18.tong_dung_luong.clone(),
    ));
    b.push(field(
        "Tổng số thiết bị lưu trữ",
        r_18.thiet_bi.len().to_string(),
    ));
    if !r_18.thiet_bi.is_empty() {
        b.push(
            FormBlock::Paragraph(
                "Tình trạng, mô tả chi tiết thiết bị (Điều 22.2.đ):"
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
        "Thời gian giao nhận",
        r_18.thoi_gian_giao_nhan.clone(),
    ));
    b.push(field("Người giao", r_18.nguoi_giao.clone()));
    b.push(field("Người nhận", r_18.nguoi_nhan.clone()));
    if let Some(x) = opt("Ghi chú khác", &r_18.ghi_chu) {
        b.push(x);
    }
    b.push(f_2f9("NGƯỜI GIAO", "NGƯỜI NHẬN"));
    (
        "Biên bản thu nộp hồ sơ, tài liệu (giao nhận trực tiếp)"
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
        field("Mã đăng ký yêu cầu thu nộp", r_1a.ma_dang_ky
        .clone()), field("Mã cơ quan, tổ chức nộp", r_1a.ma_co_quan
        .clone()), field("Tên cơ quan, tổ chức nộp", r_1a.ten_co_quan
        .clone()), field("Số lần nộp", r_1a.so_lan_nop.to_string()),
        field("Tổng số hồ sơ, tài liệu giao nhận vào lưu trữ lịch sử",
        r_1a.tong_giao_nhan.to_string(),),
    ];
    if let Some(x) = opt(
        "Tổng số dung lượng",
        &r_1a.tong_dung_luong,
    ) {
        b.push(x);
    }
    b.push(field(
        "Thời gian giao nhận",
        r_1a.thoi_gian.clone(),
    ));
    b.push(field(
        "Bên giao (cơ quan, tổ chức nộp)",
        r_1a.ben_giao.clone(),
    ));
    b.push(field(
        "Bên nhận (lưu trữ lịch sử)",
        r_1a.ben_nhan.clone(),
    ));
    if let Some(x) = opt("Ghi chú khác", &r_1a.ghi_chu) {
        b.push(x);
    }
    b.push(f_2f9(
        "ĐẠI DIỆN CƠ QUAN NỘP",
        "ĐẠI DIỆN LƯU TRỮ LỊCH SỬ",
    ));
    (
        "Biên bản giao nhận hồ sơ, tài liệu vào lưu trữ lịch sử"
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
            "Mã đăng ký yêu cầu thu nộp",
            r_1b.ma_dang_ky.clone(),
        ),
        field(
            "Mã cơ quan, tổ chức nộp",
            r_1b.ma_co_quan.clone(),
        ),
        field(
            "Tên cơ quan, tổ chức nộp",
            r_1b.ten_co_quan.clone(),
        ),
        field("Số lần nộp", r_1b.so_lan_nop.to_string()),
        field(
            "Tổng số hồ sơ, tài liệu trả lại",
            r_1b.tong_tra_lai.to_string(),
        ),
    ];
    if !r_1b.danh_sach.is_empty() {
        b.push(
            FormBlock::Paragraph(
                "Danh sách hồ sơ, tài liệu không được phê duyệt (Điều 25.1.a):"
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
        "Phương thức trả lại (Điều 25.4)",
        r_1b.phuong_thuc_tra.clone(),
    ));
    b.push(field(
        "Thời gian trả lại",
        r_1b.thoi_gian.clone(),
    ));
    b.push(field(
        "Bên giao (lưu trữ lịch sử)",
        r_1b.ben_giao.clone(),
    ));
    b.push(field(
        "Bên nhận (cơ quan, tổ chức nộp)",
        r_1b.ben_nhan.clone(),
    ));
    if let Some(r_1d) = opt("Ghi chú khác", &r_1b.ghi_chu) {
        b.push(r_1d);
    }
    b.push(f_2f9(
        "ĐẠI DIỆN LƯU TRỮ LỊCH SỬ",
        "ĐẠI DIỆN CƠ QUAN NỘP",
    ));
    (
        "Biên bản hồ sơ, tài liệu trả lại".into(),
        b,
    )
}
/// Biên bản "generic" (0024, 3/11/2026): máy trạm gửi sẵn tiêu đề + các khối, máy chủ chỉ dựng PDF. Mọi mẫu thi công
/// số hoá (giao nhận giấy, nhật ký, kiểm tra sản phẩm, nghiệm thu, bàn giao) đi đường này — thêm mẫu mới không cần sửa
/// máy chủ, chỉ cần kind có trong CHECK của bảng records và mẫu ở desktop/src/features/records/recordForms.ts.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "t", rename_all = "camelCase")]
pub enum GenericBlock {
    Heading { text: String },
    Paragraph { text: String },
    Field { label: String, value: String },
    Bullets { items: Vec<String> },
    /// Bảng có viền thật ở cả PDF (`FormBlock::Table`, 26/11/2026) và Word. Trước 26/11 PDF hạ thành gạch đầu dòng nối " | ".
    Table { columns: Vec<String>, rows: Vec<Vec<String>> },
    Signatures { left: Vec<String>, right: Vec<String> },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenericForm {
    pub title: String,
    pub blocks: Vec<GenericBlock>,
}
pub const GENERIC_KINDS: [&str; 9] = [
    "giao_nhan_giay", "tra_tai_lieu_giay", "nhat_ky_so_hoa", "kiem_tra_san_pham", "nghiem_thu", "ban_giao_lan",
    // lưu trữ hiện hành (Điều 19, 0031):
    "giao_nhan_nop_luu", "so_nop_luu", "danh_muc_ho_so",
];
pub fn is_generic(kind: &str) -> bool {
    GENERIC_KINDS.contains(&kind)
}
fn render_generic(f: &GenericForm) -> Result<(String, Vec<FormBlock>), String> {
    if f.title.trim().is_empty() {
        return Err("biên bản thiếu tiêu đề".into());
    }
    if f.blocks.is_empty() {
        return Err("biên bản không có nội dung".into());
    }
    let mut out = Vec::with_capacity(f.blocks.len() + 4);
    for b in &f.blocks {
        match b {
            GenericBlock::Heading { text } => out.push(FormBlock::SectionHeading(text.clone())),
            GenericBlock::Paragraph { text } => out.push(FormBlock::Paragraph(text.clone())),
            GenericBlock::Field { label, value } => out.push(field(label, value.clone())),
            GenericBlock::Bullets { items } => out.push(FormBlock::Bullets(items.clone())),
            GenericBlock::Table { columns, rows } => out.push(FormBlock::Table { columns: columns.clone(), rows: rows.clone() }),
            GenericBlock::Signatures { left, right } => out.push(FormBlock::Signatures { left: left.clone(), right: right.clone() }),
        }
    }
    Ok((f.title.trim().to_string(), out))
}
pub fn f_2i2(r_1e: &str) -> &'static str {
    match r_1e {
        "giao_nhan_giay" => "(Theo khoản 1 Điều 9 — Thông tư 05/2025/TT-BNV: khảo sát, bàn giao tài liệu)",
        "tra_tai_lieu_giay" => "(Theo điểm đ khoản 4 Điều 9 — Thông tư 05/2025/TT-BNV: trả tài liệu gốc về kho)",
        "nhat_ky_so_hoa" => "(Theo khoản 4 Điều 9 — Thông tư 05/2025/TT-BNV: nhật ký số hoá)",
        "kiem_tra_san_pham" => "(Theo khoản 5 Điều 9 — Thông tư 05/2025/TT-BNV: kiểm tra sản phẩm số hoá)",
        "nghiem_thu" => "(Theo khoản 8 Điều 9 — Thông tư 05/2025/TT-BNV: nghiệm thu)",
        "ban_giao_lan" => "(Theo khoản 8 Điều 9 — Thông tư 05/2025/TT-BNV: bàn giao sản phẩm)",
        "giao_nhan_nop_luu" => "(Theo Điều 19 — Thông tư 05/2025/TT-BNV: nộp hồ sơ vào lưu trữ hiện hành)",
        "so_nop_luu" => "(Theo Điều 19 — Thông tư 05/2025/TT-BNV: sổ nộp lưu hồ sơ vào lưu trữ hiện hành)",
        "danh_muc_ho_so" => "(Theo khoản 4 Điều 19 — Thông tư 05/2025/TT-BNV: danh mục hồ sơ)",
        "kiem_tra" => {
            "(Mẫu theo Mục 1 Phụ lục VII — Thông tư 05/2025/TT-BNV, Điều 29)"
        }
        "nhat_ky_sao_luu" => {
            "(Mẫu theo Mục 2 Phụ lục VII — Thông tư 05/2025/TT-BNV, Điều 30)"
        }
        "sao_luu" => {
            "(Mẫu theo Mục 3 Phụ lục VII — Thông tư 05/2025/TT-BNV, Điều 30)"
        }
        "su_co" => {
            "(Mẫu theo Mục 4 Phụ lục VII — Thông tư 05/2025/TT-BNV, Điều 31)"
        }
        "thu_nop" => {
            "(Theo khoản 2 Điều 25 — Thông tư 05/2025/TT-BNV)"
        }
        "thu_nop_truc_tiep" => {
            "(Theo điểm đ khoản 2 Điều 22 — Thông tư 05/2025/TT-BNV)"
        }
        "giao_nhan" => {
            "(Theo điểm a khoản 5 Điều 25 — Thông tư 05/2025/TT-BNV)"
        }
        "tra_lai" => {
            "(Theo điểm a khoản 5 Điều 25 — Thông tư 05/2025/TT-BNV)"
        }
        _ => "",
    }
}
pub fn render(r_1f: &str, payload: &serde_json::Value) -> Result<(String, Vec<FormBlock>), String> {
    fn de<T: serde::de::DeserializeOwned>(r_1g: &serde_json::Value) -> Result<T, String> {
        serde_json::from_value(r_1g.clone())
            .map_err(|e| format!("payload không hợp lệ: {e}"))
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
        k if is_generic(k) => render_generic(&de::<GenericForm>(payload)?),
        other => Err(format!(
            "loại biên bản không hợp lệ: {other}"
        )),
    }
}
/// Biên bản dạng Word (4/11/2026): mẫu generic dựng thẳng từ khối (bảng thật), mẫu cũ từ khối PDF.
pub fn render_docx(kind: &str, payload: &serde_json::Value, so: Option<&str>) -> Result<Vec<u8>, String> {
    let (title, blocks) = if is_generic(kind) {
        let f: GenericForm = serde_json::from_value(payload.clone()).map_err(|e| format!("payload không hợp lệ: {e}"))?;
        render_generic(&f)?;
        (f.title.trim().to_string(), crate::docx::from_generic(&f))
    } else {
        let (t, b) = render(kind, payload)?;
        (t, crate::docx::from_form(&b))
    };
    crate::docx::render(&title, Some(f_2i2(kind)), so, &blocks)
}
#[cfg(test)]
mod generic_tests {
    use super::*;
    #[test]
    fn generic_render_va_tu_choi_rong() {
        let v = serde_json::json!({
            "title": "Biên bản giao nhận tài liệu giấy",
            "blocks": [
                {"t": "paragraph", "text": "Hôm nay…"},
                {"t": "field", "label": "Phông", "value": "UBND xã A"},
                {"t": "table", "columns": ["Hộp", "Số HS"], "rows": [["01", "12"], ["02", "9"]]},
                {"t": "signatures", "left": ["BÊN GIAO"], "right": ["BÊN NHẬN"]}
            ],
            "fields": {"bo_qua": true}
        });
        let (title, blocks) = render("giao_nhan_giay", &v).expect("render");
        assert_eq!(title, "Biên bản giao nhận tài liệu giấy");
        // 4 khối vào → 4 khối ra: bảng giữ là một `FormBlock::Table` (trước 26/11 bị tách thành đoạn + gạch đầu dòng = 5).
        assert_eq!(blocks.len(), 4);
        assert!(matches!(&blocks[2], FormBlock::Table { columns, rows } if columns.len() == 2 && rows.len() == 2));
        assert!(render("giao_nhan_giay", &serde_json::json!({"title": "x", "blocks": []})).is_err());
        assert!(render("kiem_tra", &v).is_err());
        assert!(!f_2i2("nghiem_thu").is_empty());
    }
}
