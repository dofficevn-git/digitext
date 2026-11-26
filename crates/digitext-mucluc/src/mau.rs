// Copyright (c) 2026 DigiText. All rights reserved.
//! **Tệp mẫu mục lục hồ sơ** + **hướng dẫn điền**, dựng từ chính bảng nhận cột của bộ đọc.
//!
//! Vì sao ở trong crate đọc chứ không ở chỗ phát tệp: nhà thầu điền sai mẫu thì bộ đọc từ chối,
//! nên mẫu và bộ đọc phải là một thứ. Mọi tên cột ở đây lấy từ [`crate::MO_TA_COT`], mà test
//! `moi_cach_viet_deu_vao_dung_cot` bắt từng cách viết trong bảng đó phải được
//! [`crate::tim_tieu_de_cot`] xếp đúng chỗ ⇒ sửa bí danh mà quên mẫu là test đỏ, không phải để
//! nhà thầu phát hiện hộ.
//!
//! Crate này **không** đóng gói `.xlsx` (không phụ thuộc `zip`): nó trả *nội dung* các sheet,
//! phần nén thành tệp nằm ở `digitext-api/src/xlsx.rs`.

use crate::MO_TA_COT;

/// Kiểu trình bày của một ô.
///
/// Tách khỏi nội dung vì phần nén `.xlsx` nằm ở crate khác: crate này nói "ô này là tiêu đề
/// cột", crate kia quyết nó trông ra sao. Thêm kiểu mới thì phải thêm một ô trong bảng kiểu của
/// `digitext-api/src/xlsx.rs`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Kieu {
    #[default]
    Thuong,
    /// Dòng tên phông ở đầu sheet — đậm, cỡ lớn hơn.
    Tieu,
    /// Câu dặn — nghiêng, chữ xám.
    Dan,
    /// Dòng tiêu đề cột — nền đậm, chữ trắng, căn giữa, có viền.
    TieuDeCot,
    /// Ô dữ liệu, có viền, xuống dòng trong ô.
    O,
    /// Ô dữ liệu, có viền, căn giữa (ngày tháng, số hộp).
    OGiua,
    /// Dòng phòng ban `I. …` / mảng `1. …` — nền nhạt, đậm, có viền suốt dòng.
    Nhom,
    /// Đậm thường (tiêu đề mục trong sheet Hướng dẫn).
    Dam,
}

/// Một ô của tệp mẫu.
#[derive(Debug, Clone, Default)]
pub struct O {
    pub v: String,
    pub kieu: Kieu,
    /// Ghi vào `.xlsx` dạng số, không phải chuỗi.
    pub so: bool,
}

pub fn t(v: impl Into<String>) -> O {
    O { v: v.into(), ..Default::default() }
}
pub fn d(v: impl Into<String>) -> O {
    O { v: v.into(), kieu: Kieu::Dam, so: false }
}
pub fn s(v: impl Into<String>) -> O {
    O { v: v.into(), kieu: Kieu::Thuong, so: true }
}
/// Ô kiểu bất kỳ.
pub fn k(v: impl Into<String>, kieu: Kieu) -> O {
    O { v: v.into(), kieu, so: false }
}
/// Ô kiểu bất kỳ, ghi dạng số.
pub fn ks(v: impl Into<String>, kieu: Kieu) -> O {
    O { v: v.into(), kieu, so: true }
}

/// Tên cột Excel: 0 → `A`, 25 → `Z`, 26 → `AA`. Ở đây vì cả tầng nội dung (tính vùng gộp ô) lẫn
/// tầng nén đều cần, và hai bản tính lệch nhau thì tệp hỏng theo kiểu khó thấy.
pub fn cot_ten(mut i: usize) -> String {
    let mut r = Vec::new();
    loop {
        r.push(b'A' + (i % 26) as u8);
        if i < 26 {
            break;
        }
        i = i / 26 - 1;
    }
    r.reverse();
    String::from_utf8(r).expect("chỉ gồm A-Z")
}

#[derive(Debug, Clone, Default)]
pub struct SheetMau {
    pub ten: String,
    /// Bề rộng từng cột (đơn vị "ký tự" của Excel). Thiếu thì cột dùng bề rộng mặc định.
    pub rong: Vec<f64>,
    pub rows: Vec<Vec<O>>,
    /// Chiều cao từng dòng, theo chỉ số của `rows`. `None` hoặc thiếu = để Excel tự tính.
    /// **Dòng có xuống dòng trong ô thì phải đặt chiều cao**: Excel chỉ tự co giãn khi người ta
    /// sửa ô, nên tệp mới sinh sẽ bị cắt mất dòng thứ hai.
    pub cao: Vec<Option<f64>>,
    /// Vùng gộp ô, ví dụ `"A1:H1"`.
    pub gop: Vec<String>,
    /// Đóng băng mọi dòng TRÊN dòng này (1-based). `Some(4)` = khoá ba dòng đầu khi cuộn.
    pub dong_bang: Option<u32>,
    /// Vùng Excel **đừng** cảnh báo "number stored as text". Cột Hồ sơ số cố ý là chữ ("01",
    /// "12A") nên không có chỗ này thì mỗi ô một tam giác xanh, nhìn như tệp hỏng.
    pub bo_qua_loi_so: Vec<String>,
    /// Hiện lưới ô nền. Bảng đã có viền thì tắt đi cho sạch.
    pub luoi: bool,
}

/// Một mục của bảng hướng dẫn — trả cả trong JSON (để cổng nhà cung cấp bày) lẫn trong sheet
/// "Hướng dẫn" của tệp mẫu (để người tải tệp về đọc offline). Một nguồn, hai chỗ hiện.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MucHuongDan {
    pub tieu_de: String,
    pub y: Vec<String>,
}

/// Trần do cửa `POST /ncc/tu-kiem/muc-luc` đặt — nhắc trong hướng dẫn để nhà thầu khỏi gửi tệp
/// 300 MB rồi nhận lỗi. Giữ khớp với `MUC_LUC_BYTE_TOI_DA` / `MUC_LUC_DONG_TOI_DA` bên
/// `digitext-api/src/routes/ncc.rs` (test bên đó canh).
pub const MIB_TOI_DA: usize = 8;
pub const DONG_TOI_DA: usize = 5000;

/// Bảng hướng dẫn điền. Nội dung các mục "lỗi chặn" lấy đúng câu bộ đọc sẽ trả về, xem
/// `phan_tich::phan_tich_muc_luc`.
pub fn huong_dan() -> Vec<MucHuongDan> {
    let cach_viet = MO_TA_COT
        .iter()
        .filter(|m| !m.cung_nhan.is_empty())
        .map(|m| format!("{} — cũng nhận: {}", m.nhan, m.cung_nhan.join(" · ")))
        .collect::<Vec<_>>();

    vec![
        MucHuongDan {
            tieu_de: "Dòng tiêu đề cột".to_string(),
            y: vec![
                "Máy chủ tự tìm dòng tiêu đề cột trong 15 dòng đầu của sheet — không bắt buộc phải ở dòng 1.".to_string(),
                format!(
                    "Dòng đó phải có ít nhất hai cột: {}. Thiếu một trong hai thì cả tệp bị từ chối.",
                    MO_TA_COT.iter().filter(|m| m.bat_buoc).map(|m| m.nhan).collect::<Vec<_>>().join(" và ")
                ),
                "Tên cột không phân biệt hoa thường, dấu, hay khoảng trắng thừa.".to_string(),
                "Cột \"STT\" / \"TT\" không bao giờ bị nhận nhầm thành cột khác — cứ để nguyên.".to_string(),
            ],
        },
        MucHuongDan {
            tieu_de: "Các cách viết tên cột được nhận".to_string(),
            y: cach_viet,
        },
        MucHuongDan {
            tieu_de: "Phòng ban và mảng công việc".to_string(),
            y: vec![
                "Không có cột riêng cho phòng ban và mảng — chúng là những DÒNG nằm trong cột Tiêu đề hồ sơ.".to_string(),
                "Dòng mở đầu bằng số La Mã rồi dấu chấm là PHÒNG BAN: \"I. VĂN PHÒNG\", \"II. PHÒNG NỘI VỤ\".".to_string(),
                "Dòng mở đầu bằng số rồi dấu chấm là MẢNG CÔNG VIỆC: \"1. Công tác tổ chức cán bộ\".".to_string(),
                "Hồ sơ phải nằm DƯỚI một dòng phòng ban. Hồ sơ nằm trên dòng phòng ban đầu tiên là lỗi chặn.".to_string(),
                "Hồ sơ nằm ngay dưới phòng ban mà chưa có dòng mảng nào thì máy chủ tự xếp vào mảng \"Vấn đề chung\".".to_string(),
                "Tệp không có dòng phòng ban nào thì KHÔNG nhập được: thêm dòng \"I. <tên phòng ban>\" phía trên các hồ sơ.".to_string(),
            ],
        },
        MucHuongDan {
            tieu_de: "Tên phông".to_string(),
            y: vec![
                "Ghi \"Phông: <tên phông>\" ở một ô bất kỳ PHÍA TRÊN dòng tiêu đề cột.".to_string(),
                "Máy chủ dùng tên này để đối chiếu với phông đã mở trong hệ thống, và để đếm hồ sơ trùng cho đúng phông.".to_string(),
                "Không ghi cũng nhập được, nhưng kết quả trùng với kho sẽ phải liệt kê cho mọi phông của dự án.".to_string(),
                "Chỉ ghi MỘT dòng như vậy. Có nhiều dòng chứa \"Phông:\" thì máy chủ lấy dòng cuối cùng.".to_string(),
            ],
        },
        MucHuongDan {
            tieu_de: "Những lỗi làm dòng hồ sơ bị chặn".to_string(),
            y: vec![
                "Thiếu tiêu đề hồ sơ.".to_string(),
                "Chưa có dòng phòng ban (I. …) phía trên.".to_string(),
                "Thiếu thời gian bắt đầu - kết thúc: ô thời gian trống VÀ tiêu đề không ghi năm nào.".to_string(),
                "Thời gian ghi kiểu máy chủ không đọc được — dạng chắc ăn là dd/mm/yyyy - dd/mm/yyyy.".to_string(),
                "Thiếu thời hạn bảo quản, khi không chọn THBQ mặc định cho cả tệp.".to_string(),
                "Thời hạn bảo quản không có trong Danh mục — xem sheet \"Danh mục THBQ\", phải viết đúng tên ở đó.".to_string(),
                "Hồ sơ số trùng với một dòng khác trong cùng tệp, cùng loại bảo quản.".to_string(),
                "Tài liệu bảo quản vĩnh viễn hoặc từ 20 năm trở lên mà không có số tờ (ô trống hoặc ghi 0).".to_string(),
            ],
        },
        MucHuongDan {
            tieu_de: "Những thứ chỉ là cảnh báo (vẫn nhập được)".to_string(),
            y: vec![
                "Số tờ ghi kèm đơn vị lạ (\"18 quyển\"): bỏ qua số tờ, ghi lại nguyên văn vào Ghi chú.".to_string(),
                "Thời gian suy ra từ năm trong tiêu đề vì ô thời gian để trống.".to_string(),
            ],
        },
        MucHuongDan {
            tieu_de: "Giới hạn một lần kiểm".to_string(),
            y: vec![
                format!("Tệp tối đa {MIB_TOI_DA} MiB, tối đa {DONG_TOI_DA} dòng hồ sơ."),
                "Vượt thì tách tệp theo phông hoặc theo hộp.".to_string(),
                "Mỗi lần chỉ đọc MỘT sheet. Tệp nhiều sheet thì chọn sheet sau khi kiểm lần đầu.".to_string(),
            ],
        },
    ]
}

/// Nội dung tệp mẫu: 3 sheet — "Mục lục hồ sơ", "Danh mục THBQ", "Hướng dẫn".
///
/// `thbq` là Danh mục thời hạn bảo quản **của chính dự án** (tên, có phải vĩnh viễn không), lấy
/// từ máy chủ. Hai dòng hồ sơ ví dụ dùng luôn tên thật trong danh mục đó, nên mẫu phát cho dự án
/// nào là đúng danh mục dự án ấy — không có chuyện mẫu ghi "Vĩnh viễn" mà dự án lại gọi khác.
pub fn tep_mau(ten_phong: Option<&str>, thbq: &[(String, bool)]) -> Vec<SheetMau> {
    let vv = thbq.iter().find(|(_, v)| *v).map(|(t, _)| t.clone()).unwrap_or_else(|| "Vĩnh viễn".into());
    let ct = thbq.iter().find(|(_, v)| !*v).map(|(t, _)| t.clone()).unwrap_or_else(|| "10 năm".into());

    // Cột của sheet mẫu: bỏ "Thời gian" vì đã có cặp Ngày bắt đầu - Ngày kết thúc (có cả ba cột
    // vẫn chạy, nhưng mẫu không nên dạy người ta điền thừa).
    let cot: Vec<&crate::MoTaCot> = MO_TA_COT.iter().filter(|m| m.khoa != "thoiGian").collect();
    let c = |khoa: &str| cot.iter().position(|m| m.khoa == khoa).expect("cột phải có trong mẫu");
    let n = cot.len();
    // Ô trống của dòng dữ liệu vẫn phải có VIỀN, nên nền là `Kieu::O` chứ không phải ô rỗng —
    // ô rỗng kiểu thường bị bỏ hẳn khỏi XML và bảng sẽ thủng một mảng.
    let dong = |cac: &[(usize, O)], nen: Kieu| -> Vec<O> {
        let mut r: Vec<O> = (0..n).map(|_| k("", nen)).collect();
        for (i, o) in cac {
            r[*i] = o.clone();
        }
        r
    };

    let i_hop = c("hopSo");
    let i_hs = c("hoSoSo");
    let i_td = c("tieuDe");
    let i_bd = c("ngayBatDau");
    let i_kt = c("ngayKetThuc");
    let i_st = c("soTo");
    let i_tq = c("thbq");
    let i_gc = c("ghiChu");
    let cot_cuoi = cot_ten(n - 1);

    // Dòng 1-2 nằm TRÊN dòng tiêu đề cột nên bộ đọc chỉ soi chúng để tìm "Phông:" — chỗ an toàn
    // duy nhất để đặt một câu dặn. Đừng chuyển câu đó xuống dưới bảng: ô ở cột Hộp số của một
    // dòng cuối tệp sẽ thành một hộp thứ ba, ô ở cột Tiêu đề thành hồ sơ thiếu mọi thứ.
    let rows_ml: Vec<Vec<O>> = vec![
        vec![k(format!("Phông: {}", ten_phong.unwrap_or("………………")), Kieu::Tieu)],
        // KHÔNG được viết chữ "Phông:" trong câu này: bộ đọc quét mọi ô phía trên dòng tiêu đề
        // tìm "Phông: …" và lấy lần khớp CUỐI ⇒ câu dặn sẽ đè mất tên phông thật (test bắt).
        vec![k(
            "Xoá 5 dòng ví dụ bên dưới trước khi điền. Giữ nguyên dòng tên phông ở trên và dòng tiêu đề cột.",
            Kieu::Dan,
        )],
        cot.iter().map(|m| k(m.nhan, Kieu::TieuDeCot)).collect(),
        dong(&[(i_td, k("I. VĂN PHÒNG", Kieu::Nhom))], Kieu::Nhom),
        dong(&[(i_td, k("1. Công tác tổ chức cán bộ", Kieu::Nhom))], Kieu::Nhom),
        dong(
            &[
                (i_hop, ks("1", Kieu::OGiua)),
                (i_hs, k("01", Kieu::OGiua)),
                (i_td, k("Hồ sơ về việc kiện toàn tổ chức bộ máy năm 2020", Kieu::O)),
                (i_bd, k("05/01/2020", Kieu::OGiua)),
                (i_kt, k("28/12/2020", Kieu::OGiua)),
                (i_st, ks("42", Kieu::OGiua)),
                (i_tq, k(vv.clone(), Kieu::O)),
            ],
            Kieu::O,
        ),
        dong(
            &[
                (i_hs, k("02", Kieu::OGiua)),
                (i_td, k("Hồ sơ về kiện toàn các ban chỉ đạo năm 2020", Kieu::O)),
                (i_bd, k("12/03/2020", Kieu::OGiua)),
                (i_kt, k("30/11/2020", Kieu::OGiua)),
                (i_st, ks("18", Kieu::OGiua)),
                (i_tq, k(vv, Kieu::O)),
                (i_gc, k("Hộp số để trống = vẫn thuộc hộp của dòng trên", Kieu::O)),
            ],
            Kieu::O,
        ),
        dong(&[(i_td, k("2. Công tác thi đua khen thưởng", Kieu::Nhom))], Kieu::Nhom),
        // Hộp 2, không phải hộp 1: một hộp chỉ chứa MỘT loại bảo quản. Trộn vĩnh viễn với có
        // thời hạn trong cùng số hộp thì hệ thống cảnh báo và tách thành hai hộp cùng số — mẫu
        // không được dạy cái mà bộ kiểm sẽ kêu.
        dong(
            &[
                (i_hop, ks("2", Kieu::OGiua)),
                (i_hs, k("01", Kieu::OGiua)),
                (i_td, k("Hồ sơ xét tặng danh hiệu thi đua năm 2021", Kieu::O)),
                (i_bd, k("04/01/2021", Kieu::OGiua)),
                (i_kt, k("25/12/2021", Kieu::OGiua)),
                (i_st, ks("67", Kieu::OGiua)),
                (i_tq, k(ct, Kieu::O)),
            ],
            Kieu::O,
        ),
    ];

    // Vùng "đừng kêu number stored as text": đúng các ô Hồ sơ số có chữ số, không hơn.
    let co_hs = |r: &Vec<O>| r.get(i_hs).map(|o| !o.v.is_empty()).unwrap_or(false);
    let sq_hs = match (rows_ml.iter().position(co_hs), rows_ml.iter().rposition(co_hs)) {
        (Some(a), Some(b)) => vec![format!("{c}{}:{c}{}", a + 1, b + 1, c = cot_ten(i_hs))],
        _ => Vec::new(),
    };
    let mut cao_ml: Vec<Option<f64>> = vec![Some(20.0), Some(18.0), Some(28.0)];
    // 30 chứ không phải 22: ô Ghi chú xuống hai dòng, mà Excel KHÔNG tự co giãn dòng của tệp
    // mới sinh ⇒ để 22 là cắt mất dòng dưới (thấy khi xem bản in, không thấy khi chạy test).
    cao_ml.resize(rows_ml.len(), Some(30.0));

    let muc_luc = SheetMau {
        ten: "Mục lục hồ sơ".to_string(),
        rong: cot
            .iter()
            .map(|m| match m.khoa {
                "tieuDe" => 52.0,
                "ghiChu" => 38.0,
                "thbq" => 16.0,
                "ngayBatDau" | "ngayKetThuc" => 13.0,
                _ => 10.0,
            })
            .collect(),
        cao: cao_ml,
        gop: vec![format!("A1:{cot_cuoi}1"), format!("A2:{cot_cuoi}2")],
        dong_bang: Some(4),
        bo_qua_loi_so: sq_hs,
        luoi: false,
        rows: rows_ml,
    };

    let mut rows_dm: Vec<Vec<O>> = vec![
        vec![k("Danh mục thời hạn bảo quản của dự án", Kieu::Tieu)],
        vec![k(
            "Ô THBQ trong sheet \"Mục lục hồ sơ\" phải viết ĐÚNG một trong các tên ở cột bên trái.",
            Kieu::Dan,
        )],
        vec![k("Tên thời hạn bảo quản", Kieu::TieuDeCot), k("Loại", Kieu::TieuDeCot)],
    ];
    for (ten, v) in thbq {
        rows_dm.push(vec![
            k(ten.clone(), Kieu::O),
            k(if *v { "Vĩnh viễn" } else { "Có thời hạn" }, Kieu::OGiua),
        ]);
    }
    let mut cao_dm: Vec<Option<f64>> = vec![Some(20.0), Some(18.0), Some(26.0)];
    cao_dm.resize(rows_dm.len(), Some(20.0));

    let danh_muc = SheetMau {
        ten: "Danh mục THBQ".to_string(),
        rong: vec![44.0, 16.0],
        cao: cao_dm,
        // KHÔNG gộp hai dòng đầu: ô gộp **cắt** phần chữ thừa, còn ô thường thì cho chữ tràn
        // sang các ô trống bên phải. Câu dặn ở đây dài hơn hai cột nên gộp là mất chữ.
        gop: Vec::new(),
        dong_bang: Some(4),
        bo_qua_loi_so: Vec::new(),
        luoi: false,
        rows: rows_dm,
    };

    let mut rows_hd: Vec<Vec<O>> = vec![
        vec![k("Hướng dẫn điền mục lục hồ sơ", Kieu::Tieu)],
        vec![k(
            "Tệp này do máy chủ DigiText sinh ra từ chính bộ đọc đang chạy — tên cột ở đây là tên cột bộ đọc nhận.",
            Kieu::Dan,
        )],
        vec![],
        vec![d("Bảng cột")],
        vec![
            k("Tên cột", Kieu::TieuDeCot),
            k("Bắt buộc", Kieu::TieuDeCot),
            k("Kiểu", Kieu::TieuDeCot),
            k("Cũng nhận các tên", Kieu::TieuDeCot),
            k("Giải thích", Kieu::TieuDeCot),
        ],
    ];
    let mut cao_hd: Vec<Option<f64>> = vec![Some(22.0), Some(18.0), Some(8.0), Some(22.0), Some(24.0)];
    for m in MO_TA_COT {
        rows_hd.push(vec![
            k(m.nhan, Kieu::O),
            k(if m.bat_buoc { "Bắt buộc" } else { "" }, Kieu::OGiua),
            k(m.kieu, Kieu::O),
            k(m.cung_nhan.join(" · "), Kieu::O),
            k(m.giai_thich, Kieu::O),
        ]);
        // Hai dòng chữ trong ô Giải thích là đủ cho mọi mục hiện có; đặt cứng vì Excel không tự
        // co giãn dòng của tệp mới sinh.
        cao_hd.push(Some(32.0));
    }
    for muc in huong_dan() {
        rows_hd.push(vec![]);
        cao_hd.push(Some(8.0));
        rows_hd.push(vec![d(muc.tieu_de)]);
        cao_hd.push(Some(22.0));
        for y in muc.y {
            rows_hd.push(vec![t(""), t(y)]);
            cao_hd.push(None);
        }
    }

    vec![
        muc_luc,
        danh_muc,
        SheetMau {
            ten: "Hướng dẫn".to_string(),
            rong: vec![20.0, 11.0, 17.0, 32.0, 72.0],
            cao: cao_hd,
            gop: vec!["A1:E1".to_string(), "A2:E2".to_string()],
            dong_bang: None,
            bo_qua_loi_so: Vec::new(),
            luoi: false,
            rows: rows_hd,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{tim_tieu_de_cot, MO_TA_COT};
    use calamine::Data;

    fn thbq_thu() -> Vec<(String, bool)> {
        vec![("Vĩnh viễn".into(), true), ("20 năm".into(), false), ("10 năm".into(), false)]
    }

    /// Chính bộ đọc phải đọc được tệp mẫu. Không có test này thì mẫu là một tệp Excel đẹp mà bộ
    /// đọc từ chối, và nhà thầu là người đầu tiên biết.
    #[test]
    fn bo_doc_tim_duoc_dong_tieu_de_cua_mau() {
        let sheets = tep_mau(Some("Sở Nông nghiệp"), &thbq_thu());
        let rows: Vec<Vec<Data>> = sheets[0]
            .rows
            .iter()
            .map(|r| r.iter().map(|o| Data::String(o.v.clone())).collect())
            .collect();
        let (dong, cot) = tim_tieu_de_cot(&rows).expect("bộ đọc phải nhận ra dòng tiêu đề của tệp mẫu");
        assert_eq!(dong, 3, "dòng tiêu đề của mẫu là dòng 3 (dòng 1 Phông, dòng 2 trống)");
        assert!(cot.ho_so_so.is_some() && cot.tieu_de.is_some());
        // Mẫu cố ý KHÔNG có cột Thời gian — dùng cặp Ngày bắt đầu / Ngày kết thúc.
        assert!(cot.ngay_bat_dau.is_some() && cot.ngay_ket_thuc.is_some());
        assert!(cot.thoi_gian.is_none());
    }

    /// Phân tích trọn tệp mẫu: 3 hồ sơ ví dụ, không lỗi chặn nào. Đây là phép thử "mẫu điền sẵn
    /// nộp lên có qua không" — qua thì hướng dẫn mới đáng tin.
    #[test]
    fn mau_dien_san_khong_co_loi_chan() {
        let dm = thbq_thu();
        let sheets = tep_mau(Some("Sở Nông nghiệp"), &dm);
        let rows: Vec<Vec<Data>> = sheets[0]
            .rows
            .iter()
            .map(|r| r.iter().map(|o| Data::String(o.v.clone())).collect())
            .collect();
        let ten: Vec<String> = dm.iter().map(|(t, _)| t.clone()).collect();
        let vv: Vec<String> = dm.iter().filter(|(_, v)| *v).map(|(t, _)| t.clone()).collect();
        let pt = crate::phan_tich::phan_tich_muc_luc(
            &rows,
            &crate::phan_tich::TuyChon {
                ten_tep: "mau.xlsx",
                sheet: "Mục lục hồ sơ",
                sheets: &["Mục lục hồ sơ".to_string()],
                thbq_danh_muc: &ten,
                thbq_vinh_vien: &vv,
                thbq_mac_dinh: None,
            },
        );
        assert_eq!(pt.ten_phong_trong_tep, "Sở Nông nghiệp");
        assert_eq!(pt.rows.len(), 3, "mẫu có 3 hồ sơ ví dụ, lấy được {:?}", pt.rows.iter().map(|r| &r.tieu_de).collect::<Vec<_>>());
        assert_eq!(pt.phong_bans.len(), 1);
        assert_eq!(pt.phong_bans[0].mangs.len(), 2, "mảng: {:?}", pt.phong_bans[0].mangs.iter().map(|m| (m.dong, m.ten.clone(), m.tu_dong)).collect::<Vec<_>>());
        assert!(
            !crate::phan_tich::co_loi_chan(&pt),
            "mẫu phải nhập được: {:?}",
            pt.rows.iter().flat_map(|r| r.loi.clone()).chain(pt.van_de.iter().filter(|v| v.muc == crate::phan_tich::Muc::Loi).map(|v| v.noi_dung.clone())).collect::<Vec<_>>()
        );
        // Hộp để trống ở dòng 2 phải tính vào hộp của dòng trên.
        assert_eq!(pt.rows[1].hop_so, "1");
        assert_eq!(pt.so_hop, 2);
        // Mẫu cũng không được sinh CẢNH BÁO nào: nhà thầu mở mẫu ra kiểm thử mà thấy cảnh báo
        // thì không biết là lỗi của họ hay của mẫu.
        assert_eq!(
            pt.so_canh_bao,
            0,
            "mẫu còn cảnh báo: {:?}",
            pt.rows
                .iter()
                .flat_map(|r| r.canh_bao.clone())
                .chain(pt.van_de.iter().filter(|v| v.muc != crate::phan_tich::Muc::Loi).map(|v| v.noi_dung.clone()))
                .collect::<Vec<_>>()
        );
    }

    /// THBQ trong hai dòng ví dụ phải là tên có thật trong danh mục truyền vào — nếu không thì
    /// mẫu tự nó đã sai ngay dòng đầu tiên nhà thầu nhìn thấy.
    #[test]
    fn thbq_vi_du_lay_tu_danh_muc_cua_du_an() {
        let dm = vec![("Bảo quản vĩnh viễn".to_string(), true), ("Bảy mươi năm".to_string(), false)];
        let sheets = tep_mau(None, &dm);
        let phang: Vec<String> = sheets[0].rows.iter().flat_map(|r| r.iter().map(|o| o.v.clone())).collect();
        assert!(phang.iter().any(|v| v == "Bảo quản vĩnh viễn"));
        assert!(phang.iter().any(|v| v == "Bảy mươi năm"));
        assert!(!phang.iter().any(|v| v == "Vĩnh viễn"), "không được tự bịa tên THBQ");
    }

    /// Danh mục rỗng là lỗi cấu hình máy chủ chứ không phải cớ để sập.
    #[test]
    fn danh_muc_rong_van_dung_duoc_mau() {
        let sheets = tep_mau(None, &[]);
        assert_eq!(sheets.len(), 3);
        assert!(sheets[1].rows.len() >= 1);
    }

    #[test]
    fn huong_dan_nhac_du_moi_loi_chan() {
        let hd = huong_dan();
        let phang = hd.iter().flat_map(|m| m.y.clone()).collect::<Vec<_>>().join(" | ");
        for can in ["Thiếu tiêu đề hồ sơ", "dòng phòng ban", "Danh mục", "trùng", "thời gian"] {
            assert!(phang.to_lowercase().contains(&can.to_lowercase()), "hướng dẫn thiếu ý \"{can}\"");
        }
        assert!(hd.iter().any(|m| m.tieu_de.contains("Giới hạn")));
    }

    #[test]
    fn bang_cot_trong_huong_dan_phu_het_mo_ta_cot() {
        let sheets = tep_mau(None, &thbq_thu());
        let hd = &sheets[2];
        for m in MO_TA_COT {
            assert!(
                hd.rows.iter().any(|r| r.first().map(|o| o.v == m.nhan).unwrap_or(false)),
                "sheet Hướng dẫn thiếu cột {}",
                m.nhan
            );
        }
    }
}

// ------------------------------------------------------------------ báo cáo «Kiểm tra»

/// Bảng **Kiểm tra** của một lần soát mục lục: mỗi hồ sơ một dòng, kèm kết luận và lý do.
///
/// Dựng ở đây chứ không ở cổng vì nó phải nói **đúng** những gì bộ đọc kết luận — cổng dựng lại
/// thì mỗi lần đổi một câu thông báo là bảng nói khác máy. Cùng bộ cột với sheet "Kiểm tra" mà
/// bản desktop đã xuất từ 2/10/2026, bỏ cột "Số tạm" (số tạm chỉ có nghĩa sau khi đã nhập vào
/// kho, nhà thầu chưa nhập thì chưa có).
///
/// Dòng **lỗi cấu trúc** (`van_de`) xếp xen vào đúng vị trí dòng của nó, không dồn xuống cuối:
/// người sửa tệp đọc từ trên xuống, không ai đọc hai bảng.
/// Chiều cao cần cho một dòng bảng, ước theo cột **dài nhất** so với bề rộng của chính cột đó.
///
/// Excel chỉ tự co giãn chiều cao khi người ta sửa ô, nên tệp mới sinh phải tự đặt. Bản đầu
/// ngày 14/11/2026 chỉ nhìn cột Ghi chú ⇒ dòng có tiêu đề hồ sơ dài mà ghi chú trống bị cắt
/// mất nửa chữ, và dòng ghi chú 200 ký tự thì cụt đuôi. Chỉ ảnh dựng ra mới thấy, không test
/// nào bắt được, nên để nguyên công thức ở đây cho dễ sửa.
fn cao_can(o: &[O], rong: &[f64]) -> f64 {
    const CAO_DONG: f64 = 14.0;
    const DEM: f64 = 8.0;
    const TOI_DA: f64 = 108.0;
    let so_dong = o
        .iter()
        .zip(rong)
        .map(|(c, w)| {
            // Ký tự xuống dòng sẵn trong ô cũng tính, mỗi đoạn tự xuống dòng theo bề rộng.
            c.v.split('\n')
                .map(|d| {
                    let n = d.chars().count() as f64;
                    (n / w.max(4.0)).ceil().max(1.0)
                })
                .sum::<f64>()
        })
        .fold(1.0, f64::max);
    (CAO_DONG * so_dong + DEM).min(TOI_DA)
}

pub fn bao_cao_kiem_tra(pt: &crate::phan_tich::PhanTich) -> Vec<SheetMau> {
    use crate::phan_tich::Muc;
    use crate::thoi_gian::KetLuan;

    let cot = [
        ("Dòng Excel", 10.0),
        ("Loại", 12.0),
        ("Hộp số", 8.0),
        ("Hồ sơ số", 10.0),
        ("Phòng ban", 22.0),
        ("Mảng", 22.0),
        ("Tiêu đề hồ sơ", 50.0),
        ("Thời gian gốc", 20.0),
        ("Bắt đầu", 11.0),
        ("Kết thúc", 11.0),
        ("THBQ", 12.0),
        ("Số tờ", 8.0),
        ("Kết luận", 12.0),
        ("Ghi chú kiểm tra", 60.0),
    ];

    let rong_cot: Vec<f64> = cot.iter().map(|(_, w)| *w).collect();
    let mut rows: Vec<Vec<O>> = Vec::new();
    let mut cao: Vec<Option<f64>> = Vec::new();

    let tieu = format!(
        "Kiểm mục lục hồ sơ — {}{}",
        pt.ten_tep,
        if pt.sheet.is_empty() { String::new() } else { format!(" · sheet {}", pt.sheet) }
    );
    rows.push(vec![k(tieu, Kieu::Tieu)]);
    cao.push(Some(22.0));
    let ket = if crate::phan_tich::co_loi_chan(pt) {
        format!("CHƯA NHẬP ĐƯỢC — còn {} lỗi chặn, nộp lúc này sẽ bị từ chối cả tệp.", pt.so_loi)
    } else {
        format!("Nhập được — {} hồ sơ, {} cảnh báo vẫn nhập được.", pt.rows.len(), pt.so_canh_bao)
    };
    rows.push(vec![k(ket, Kieu::Dan)]);
    cao.push(Some(18.0));
    rows.push(vec![k(
        format!(
            "{} hồ sơ · {} hộp · {} lỗi · {} cảnh báo · {} dòng máy tự suy thời gian",
            pt.rows.len(),
            pt.so_hop,
            pt.so_loi,
            pt.so_canh_bao,
            pt.so_da_dien
        ),
        Kieu::Dan,
    )]);
    cao.push(Some(18.0));
    rows.push(Vec::new());
    cao.push(None);

    rows.push(cot.iter().map(|(t, _)| k(*t, Kieu::TieuDeCot)).collect());
    cao.push(Some(28.0));
    let dong_bang = Some(rows.len() as u32 + 1);

    // Lỗi cấu trúc gắn theo dòng, để chèn xen vào đúng chỗ.
    let mut xen: std::collections::BTreeMap<usize, Vec<&crate::phan_tich::VanDe>> = Default::default();
    let mut chung: Vec<&crate::phan_tich::VanDe> = Vec::new();
    for v in &pt.van_de {
        if v.dong > 0 {
            xen.entry(v.dong).or_default().push(v);
        } else {
            chung.push(v);
        }
    }

    let them_van_de = |rows: &mut Vec<Vec<O>>, cao: &mut Vec<Option<f64>>, v: &crate::phan_tich::VanDe| {
        let mut o = vec![O::default(); cot.len()];
        o[0] = ks(if v.dong > 0 { v.dong.to_string() } else { String::new() }, Kieu::OGiua);
        o[12] = k(if v.muc == Muc::Loi { "Lỗi" } else { "Cảnh báo" }, Kieu::OGiua);
        o[13] = k(&v.noi_dung, Kieu::O);
        cao.push(Some(cao_can(&o, &rong_cot)));
        rows.push(o);
    };

    for v in &chung {
        them_van_de(&mut rows, &mut cao, v);
    }

    for r in &pt.rows {
        for v in xen.remove(&r.dong).unwrap_or_default() {
            them_van_de(&mut rows, &mut cao, v);
        }
        let pb = r.pb_idx.and_then(|i| pt.phong_bans.get(i));
        let mang = pb.and_then(|p| r.mang_id.and_then(|id| p.mangs.iter().find(|m| m.id == id)));
        let ket_luan = if !r.loi.is_empty() {
            "Lỗi"
        } else if !r.canh_bao.is_empty() || matches!(r.tg.ket_luan, KetLuan::CanhBao | KetLuan::Loi) {
            "Cảnh báo"
        } else {
            "Đạt"
        };
        let ghi: Vec<String> = r
            .loi
            .iter()
            .chain(r.canh_bao.iter())
            .chain(r.tg.ghi_chu.iter())
            .cloned()
            .collect();
        rows.push(vec![
            ks(r.dong.to_string(), Kieu::OGiua),
            k(crate::phan_tich::ten_loai(r.vinh_vien), Kieu::OGiua),
            k(&r.hop_so, Kieu::OGiua),
            k(&r.ho_so_so, Kieu::OGiua),
            k(pb.map(|p| format!("{} · {}", p.ma, p.ten)).unwrap_or_default(), Kieu::O),
            k(mang.map(|m| format!("{} · {}", m.ma, m.ten)).unwrap_or_default(), Kieu::O),
            k(&r.tieu_de, Kieu::O),
            k(r.thoi_gian_goc.replace(['\n', '\r'], " ").trim(), Kieu::O),
            k(&r.tg.bat_dau, Kieu::OGiua),
            k(&r.tg.ket_thuc, Kieu::OGiua),
            k(r.thbq.as_deref().unwrap_or(&r.thbq_goc), Kieu::O),
            k(&r.so_to, Kieu::OGiua),
            k(ket_luan, Kieu::OGiua),
            k(ghi.join("; "), Kieu::O),
        ]);
        cao.push(Some(cao_can(rows.last().expect("vừa đẩy"), &rong_cot)));
    }
    // Dòng lỗi cấu trúc trỏ tới dòng không phải hồ sơ (tiêu đề nhóm…) thì vẫn phải hiện.
    for (_, vs) in std::mem::take(&mut xen) {
        for v in vs {
            them_van_de(&mut rows, &mut cao, v);
        }
    }

    let n = rows.len();
    vec![SheetMau {
        ten: "Kiểm tra".into(),
        rong: rong_cot,
        rows,
        cao,
        gop: Vec::new(),
        dong_bang,
        // Cột Dòng Excel / Hộp số / Hồ sơ số / Số tờ cố ý là chữ.
        bo_qua_loi_so: vec![format!("A1:D{n}"), format!("L1:L{n}")],
        luoi: false,
    }]
}
