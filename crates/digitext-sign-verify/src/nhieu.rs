// Copyright (c) 2026 DigiText. All rights reserved.
//! Kiểm **mọi** chữ ký trong một PDF, không chỉ một.
//!
//! [`crate::verify`] xác minh đúng một chữ ký — chữ ký mà `f_2f8` tìm thấy — rồi đòi
//! `/ByteRange` của nó phủ trọn tệp. Với văn bản ký **một lần** thì đúng. Với văn bản ký
//! **nhiều lần** thì sai theo hai hướng, cả hai đều đã dựng mẫu kiểm chứng:
//!
//! - Nếu chữ ký cũ đứng đầu `/AcroForm/Fields` — tình huống bình thường của công cụ ký tăng
//!   dần (VGCA SignTool: phê duyệt → số công văn → ngày → dấu tổ chức) — thì `/ByteRange` của
//!   nó hết ở biên bản sửa của chính nó, không tới cuối tệp, và `verify` kết luận *"PDF đã bị
//!   sửa sau khi ký"*. Đó là **vu oan**: văn bản hoàn toàn hợp lệ.
//! - Nếu chữ ký mới được chọn thì `verify` nói "hợp lệ" — nhưng đó là kết luận về **một** chữ
//!   ký, mấy chữ ký kia không ai xem.
//!
//! Module này làm việc đúng: liệt kê từng chữ ký theo thứ tự bản sửa, xác minh **từng cái trên
//! đúng vùng byte của nó**, rồi nói riêng hai chuyện vẫn bị trộn làm một:
//!
//! 1. **Chữ ký có toàn vẹn không** — CMS đúng và băm khớp trên vùng nó phủ. Chuyện của riêng
//!    từng chữ ký.
//! 2. **Sau chữ ký đó tệp có bị sửa trái phép không** — phần đuôi có khớp chuỗi bản sửa của
//!    PDF hay không. Chuyện của cả tệp.
//!
//! Một chữ ký không phủ tới cuối tệp **không** phải dấu hiệu bị sửa, chừng nào phần sau nó là
//! các bản sửa tăng dần hợp lệ. Lẫn hai chuyện này là đúng chỗ bản cũ sai.

use crate::cms;
use crate::pdf::{f_15c, f_1cu, f_1gu, f_1yg, f_ws, SignerInfo};
use x509_cert::Certificate;

/// Một chữ ký (hoặc dấu thời gian tài liệu) tìm thấy trong PDF.
#[derive(Debug, Clone)]
pub struct ChuKyPdf {
    /// 1, 2, 3… theo **thứ tự bản sửa**: chữ ký phủ ít byte nhất là chữ ký ký trước.
    pub thu_tu: usize,
    /// `Sig` (chữ ký) hay `DocTimeStamp` (dấu thời gian cho cả tài liệu).
    pub loai: String,
    /// `/SubFilter`: `ETSI.CAdES.detached` (PAdES), `adbe.pkcs7.detached`, `ETSI.RFC3161`…
    pub khuon_dang: Option<String>,
    /// `/Name` khai trong chính ô chữ ký — công cụ ký ghi, **không** phải lấy từ chứng thư.
    pub ten_khai: Option<String>,
    /// `/Reason` — lý do ký. VGCA dùng chỗ này để phân biệt phê duyệt / số / ngày / dấu.
    pub ly_do: Option<String>,
    /// `/Location`.
    pub noi: Option<String>,
    /// `/M` — thời điểm ký do máy người ký khai, **không có giá trị chứng minh** (không TSA).
    pub thoi_diem_khai: Option<String>,
    /// `/ByteRange` nguyên văn.
    pub vung: [usize; 4],
    /// Byte cuối mà chữ ký này phủ tới.
    pub phu_den: usize,
    /// Phủ tới hết tệp — tức đây là chữ ký sau cùng.
    pub phu_tron_tep: bool,
    /// `/ByteRange` có bao đúng hai bên ô `/Contents` hay không. `false` = ô chữ ký **tự khai
    /// sai** vùng nó phủ; mọi kết luận sau đó vô nghĩa, và đây là dấu hiệu bị can thiệp.
    pub vung_khop_o_chu_ky: bool,
    /// Chữ ký đúng + băm khớp **trên vùng nó phủ**.
    pub toan_ven: bool,
    /// Chứng thư có gắn vào chữ ký (ESS signing-certificate). Thiếu = chưa đạt PAdES cơ bản.
    pub gan_chung_thu: bool,
    /// Có dấu thời gian trong chữ ký.
    pub co_tsa: bool,
    /// Thời điểm theo dấu thời gian — thứ **chứng minh được**, khác `thoi_diem_khai`.
    pub thoi_diem_tsa_unix: Option<i64>,
    /// Thông tin chứng thư người ký.
    pub nguoi_ky: Option<SignerInfo>,
    /// Vì sao không xác minh được. `None` = xác minh xong (dù kết quả đạt hay không).
    pub loi: Option<String>,
}

/// Cắt nhãn ở ranh giới TỪ, không cắt giữa từ, và không cắt giữa ký tự nhiều byte.
fn cat_nhan(t: &str, toi_da: usize) -> String {
    let t = t.split_whitespace().collect::<Vec<_>>().join(" ");
    if t.chars().count() <= toi_da {
        return t;
    }
    let mut ra = String::new();
    for tu in t.split(' ') {
        if ra.chars().count() + tu.chars().count() + 1 > toi_da {
            break;
        }
        if !ra.is_empty() {
            ra.push(' ');
        }
        ra.push_str(tu);
    }
    if ra.is_empty() {
        ra = t.chars().take(toi_da).collect();
    }
    format!("{ra}…")
}

impl ChuKyPdf {
    /// Chữ ký này tự nó đạt: vùng khai đúng, chữ ký đúng, băm khớp.
    pub fn dat(&self) -> bool {
        self.vung_khop_o_chu_ky && self.toan_ven
    }
    /// Nhãn gọi tên chữ ký này trong câu báo cáo. Ưu tiên `/Reason` — VGCA SignTool ghi ở đó
    /// để phân biệt phê duyệt / số công văn / ngày / dấu tổ chức.
    pub fn nhan(&self) -> String {
        let dau = if self.loai == "DocTimeStamp" { "Dấu thời gian tài liệu" } else { "Chữ ký" };
        let mut s = format!("{dau} #{}", self.thu_tu);
        if let Some(t) = self.ly_do.as_deref().or(self.ten_khai.as_deref()).filter(|t| !t.is_empty()) {
            // Công cụ ký hay ghi CẢ MỘT CÂU vào `/Reason` ("UBND X da ky len van ban nay"),
            // nên phải cắt: nhãn này nằm giữa câu báo lỗi, dài quá thì không ai đọc nổi.
            s.push_str(&format!(" ({})", cat_nhan(t, 40)));
        }
        s
    }
}

/// Kết quả kiểm cả tệp.
#[derive(Debug, Clone)]
pub struct KetQuaNhieu {
    pub chu_ky: Vec<ChuKyPdf>,
    /// Biên các bản sửa, lấy từ chuỗi `%%EOF`. Bản sửa đầu hết ở `moc[0]`, v.v.
    pub moc_ban_sua: Vec<usize>,
    /// Số byte ở cuối tệp **không** thuộc bản sửa nào — tức có người nối thêm byte vào sau
    /// `%%EOF` cuối. Khác 0 là dấu hiệu bị can thiệp thật.
    pub byte_thua_cuoi: usize,
    /// Phần đuôi sau chữ ký sau cùng là **increment DSS/DocTimeStamp hợp lệ**.
    ///
    /// Có trường này vì nếu không thì tệp PAdES-LT bị kết luận sai: ở đó dữ liệu xác minh dài
    /// hạn được nối thành một bản sửa SAU chữ ký, nên không chữ ký nào phủ tới cuối tệp — mà
    /// đó lại là tệp ký **đúng chuẩn hơn**, không phải tệp bị sửa.
    pub duoi_la_dss: bool,
    /// Có dữ liệu xác minh dài hạn (LTV/DSS) — chứng thư, OCSP, CRL nhúng kèm.
    pub co_dss: bool,
    /// Có dấu thời gian cho **cả tài liệu** (DocTimeStamp), tức mức PAdES-LTA.
    pub co_dau_thoi_gian_tai_lieu: bool,
    /// Vì sao phần đuôi không được nhận là increment DSS — chỉ có khi `duoi_la_dss` là `false`
    /// và thật sự có đuôi.
    pub duoi_loi: Option<String>,
}

impl KetQuaNhieu {
    pub fn so_chu_ky(&self) -> usize {
        self.chu_ky.len()
    }
    /// Mọi chữ ký đều đạt, và không có byte lạ nối sau cùng.
    pub fn tat_ca_dat(&self) -> bool {
        !self.chu_ky.is_empty() && self.byte_thua_cuoi == 0 && self.chu_ky.iter().all(ChuKyPdf::dat)
    }
    /// Số chữ ký không đạt.
    pub fn so_khong_dat(&self) -> usize {
        self.chu_ky.iter().filter(|c| !c.dat()).count()
    }
}

fn tim(h: &[u8], kim: &[u8]) -> Option<usize> {
    if kim.is_empty() || h.len() < kim.len() {
        return None;
    }
    h.windows(kim.len()).position(|w| w == kim)
}

/// Biên các bản sửa: ngay sau mỗi `%%EOF` (bỏ qua khoảng trắng theo sau).
fn moc_ban_sua(pdf: &[u8]) -> Vec<usize> {
    let mut v = Vec::new();
    let mut i = 0;
    while let Some(k) = tim(&pdf[i..], b"%%EOF") {
        let mut het = i + k + 5;
        while pdf.get(het).is_some_and(|b| *b == b'\r' || *b == b'\n' || *b == b' ' || *b == b'\t') {
            het += 1;
        }
        v.push(het);
        i = i + k + 5;
    }
    v
}

/// Mọi khối `N G obj … endobj` có chứa `/ByteRange` — không đi qua `/AcroForm/Fields`.
///
/// Quét byte thay vì đi theo xref là có chủ ý: tệp nhiều bản sửa có **nhiều** từ điển
/// `/AcroForm`, mỗi bản sửa một cái, và cái "sống" chỉ liệt kê trường của bản sửa đó — đúng
/// chỗ làm bản cũ bỏ sót chữ ký. Quét byte thì không bỏ sót cái nào.
fn khoi_chu_ky(pdf: &[u8]) -> Vec<(usize, usize)> {
    let mut ra = Vec::new();
    let mut i = 0;
    while let Some(k) = tim(&pdf[i..], b"/ByteRange") {
        let at = i + k;
        i = at + 10;
        let dau = &pdf[..at];
        let Some(o) = dau.windows(4).rposition(|w| w == b" obj" || w == b"\nobj" || w == b"\robj") else {
            continue;
        };
        let bd = dau[..o].iter().rposition(|b| *b == b'\n' || *b == b'\r').map(|x| x + 1).unwrap_or(0);
        let Some(kt) = tim(&pdf[at..], b"endobj").map(|x| at + x) else {
            continue;
        };
        if !ra.iter().any(|(a, _): &(usize, usize)| *a == bd) {
            ra.push((bd, kt));
        }
    }
    ra
}

/// Giá trị của một khoá dạng tên: `/SubFilter /ETSI.CAdES.detached`.
fn gia_tri_ten(doan: &[u8], khoa: &[u8]) -> Option<String> {
    let at = tim(doan, khoa)? + khoa.len();
    let mut i = at;
    while doan.get(i).is_some_and(|b| b.is_ascii_whitespace()) {
        i += 1;
    }
    if doan.get(i) != Some(&b'/') {
        return None;
    }
    i += 1;
    let bd = i;
    while doan
        .get(i)
        .is_some_and(|b| !b.is_ascii_whitespace() && !b"/<>[]()".contains(b))
    {
        i += 1;
    }
    Some(String::from_utf8_lossy(&doan[bd..i]).into_owned()).filter(|s| !s.is_empty())
}

/// Giá trị của một khoá dạng chuỗi: `/Reason (…)` hoặc `/Reason <FEFF…>`.
fn gia_tri_chuoi(doan: &[u8], khoa: &[u8]) -> Option<String> {
    let at = tim(doan, khoa)? + khoa.len();
    let mut i = at;
    while doan.get(i).is_some_and(|b| b.is_ascii_whitespace()) {
        i += 1;
    }
    match doan.get(i) {
        Some(b'(') => {
            let mut sau = 1usize;
            let mut byte = Vec::new();
            let mut j = i + 1;
            while let Some(b) = doan.get(j) {
                match b {
                    b'\\' => {
                        if let Some(n) = doan.get(j + 1) {
                            byte.push(*n);
                            j += 2;
                            continue;
                        }
                        j += 1;
                    }
                    b'(' => {
                        sau += 1;
                        byte.push(*b);
                        j += 1;
                    }
                    b')' => {
                        sau -= 1;
                        if sau == 0 {
                            break;
                        }
                        byte.push(*b);
                        j += 1;
                    }
                    _ => {
                        byte.push(*b);
                        j += 1;
                    }
                }
            }
            Some(chuoi_pdf(&byte))
        }
        Some(b'<') => {
            let kt = i + 1 + tim(&doan[i + 1..], b">")?;
            let byte = f_1gu(&doan[i + 1..kt])?;
            Some(chuoi_pdf(&byte))
        }
        _ => None,
    }
    .filter(|s| !s.is_empty())
}

/// Chuỗi PDF → text. UTF-16BE khi có BOM `FE FF`, còn lại coi là UTF-8 rồi mới tới Latin-1
/// (PDFDocEncoding trùng Latin-1 ở khoảng hay dùng).
fn chuoi_pdf(b: &[u8]) -> String {
    if b.len() >= 2 && b[0] == 0xFE && b[1] == 0xFF {
        let u: Vec<u16> = b[2..].chunks_exact(2).map(|c| u16::from_be_bytes([c[0], c[1]])).collect();
        return String::from_utf16_lossy(&u).trim().to_string();
    }
    match core::str::from_utf8(b) {
        Ok(s) => s.trim().to_string(),
        Err(_) => b.iter().map(|c| *c as char).collect::<String>().trim().to_string(),
    }
}

fn thong_tin_nguoi_ky(der: &[u8]) -> Option<SignerInfo> {
    let sd = cms::f_1zy(der).ok()?;
    let cert = cms::signer_cert(&sd).ok()?;
    let issuer_cn = {
        let tmp = Certificate {
            tbs_certificate: x509_cert::TbsCertificate {
                subject: cert.tbs_certificate.issuer.clone(),
                ..cert.tbs_certificate.clone()
            },
            ..cert.clone()
        };
        crate::pdf::f_2fg(&tmp)
    };
    let v = &cert.tbs_certificate.validity;
    Some(SignerInfo {
        signer: crate::pdf::f_2fg(&cert),
        subject: cert.tbs_certificate.subject.to_string(),
        issuer: issuer_cn,
        not_before_unix: v.not_before.to_unix_duration().as_secs() as i64,
        not_after_unix: v.not_after.to_unix_duration().as_secs() as i64,
    })
}

/// `/ByteRange` có bao đúng hai bên ô `/Contents` không.
///
/// Phép kiểm này **phải** chạy: không có nó thì một ô chữ ký khai `/ByteRange` bất kỳ cũng
/// "băm khớp" — kẻ sửa tệp chỉ cần trỏ vùng phủ vào đoạn mình không sửa.
fn vung_bao_dung_o(vung: [usize; 4], o_dau: usize, o_cuoi: usize) -> bool {
    let Some(het_doan_dau) = vung[0].checked_add(vung[1]) else {
        return false;
    };
    // `o_dau` trỏ byte đầu TRONG dấu `<`, `o_cuoi` trỏ dấu `>`: vùng phải hở đúng ô đó.
    let kieu_a = o_dau.checked_sub(1).is_some_and(|t| het_doan_dau == t && vung[2] == o_cuoi + 1);
    let kieu_b = het_doan_dau == o_dau && vung[2] == o_cuoi;
    kieu_a || kieu_b
}

/// Kiểm mọi chữ ký trong PDF. Không lỗi toàn cục: tệp không có chữ ký nào thì trả danh sách
/// rỗng, chữ ký nào không đọc được thì chữ ký ấy mang `loi`, các chữ ký khác vẫn có kết quả.
pub fn kiem_tat_ca(pdf: &[u8]) -> KetQuaNhieu {
    let moc = moc_ban_sua(pdf);
    let byte_thua_cuoi = moc.last().map(|m| pdf.len().saturating_sub(*m)).unwrap_or(pdf.len());
    let mut ds: Vec<ChuKyPdf> = Vec::new();

    for (bd, kt) in khoi_chu_ky(pdf) {
        let doan = &pdf[bd..kt];
        let loai = gia_tri_ten(doan, b"/Type").unwrap_or_else(|| "?".into());
        let mut ck = ChuKyPdf {
            thu_tu: 0,
            loai,
            khuon_dang: gia_tri_ten(doan, b"/SubFilter"),
            ten_khai: gia_tri_chuoi(doan, b"/Name"),
            ly_do: gia_tri_chuoi(doan, b"/Reason"),
            noi: gia_tri_chuoi(doan, b"/Location"),
            thoi_diem_khai: gia_tri_chuoi(doan, b"/M"),
            vung: [0; 4],
            phu_den: 0,
            phu_tron_tep: false,
            vung_khop_o_chu_ky: false,
            toan_ven: false,
            gan_chung_thu: false,
            co_tsa: false,
            thoi_diem_tsa_unix: None,
            nguoi_ky: None,
            loi: None,
        };

        let Some(vung) = f_ws(pdf, bd, kt).and_then(|(a, b)| f_1yg(pdf, a, b)) else {
            ck.loi = Some("không đọc được /ByteRange".into());
            ds.push(ck);
            continue;
        };
        ck.vung = vung;
        ck.phu_den = vung[2].saturating_add(vung[3]);
        ck.phu_tron_tep = ck.phu_den == pdf.len();

        let Some((o_dau, o_cuoi)) = f_1cu(pdf, bd, kt) else {
            ck.loi = Some("không thấy ô /Contents".into());
            ds.push(ck);
            continue;
        };
        ck.vung_khop_o_chu_ky = vung_bao_dung_o(vung, o_dau, o_cuoi);

        let Some(der) = f_1gu(&pdf[o_dau..o_cuoi]) else {
            ck.loi = Some("/Contents không phải hex".into());
            ds.push(ck);
            continue;
        };
        // Băm ĐÚNG vùng chữ ký này khai — không đòi nó phủ tới cuối tệp. Đây là chỗ bản cũ sai.
        let Some(bam) = f_15c(pdf, vung) else {
            ck.loi = Some("/ByteRange trỏ ra ngoài tệp".into());
            ds.push(ck);
            continue;
        };
        match cms::f_2pq(&der, &bam) {
            Ok(t) => {
                ck.toan_ven = t.signature_ok && t.digest_ok;
                ck.gan_chung_thu = t.cert_binding_ok;
                ck.co_tsa = t.timestamp_present;
                ck.thoi_diem_tsa_unix = t.timestamp_time_unix;
            }
            Err(e) => ck.loi = Some(format!("{e}")),
        }
        ck.nguoi_ky = thong_tin_nguoi_ky(&der);
        ds.push(ck);
    }

    // Thứ tự bản sửa: phủ ít byte hơn là ký trước.
    ds.sort_by_key(|c| (c.phu_den, c.vung[1]));
    for (i, c) in ds.iter_mut().enumerate() {
        c.thu_tu = i + 1;
    }
    // Phần đuôi sau chữ ký SAU CÙNG: increment DSS/DocTimeStamp là hợp lệ và đáng mừng
    // (PAdES-LT/LTA), không phải dấu hiệu bị sửa. Dùng lại `dss::f_2p3` — nó kiểm cả việc
    // increment không đổi `/Root` và không sửa object đã ký, chứ không chỉ tìm chữ `/DSS`.
    let het = ds.iter().map(|c| c.phu_den).max().unwrap_or(0);
    let mut duoi_la_dss = false;
    let mut co_dss = false;
    let mut co_dau_thoi_gian_tai_lieu = false;
    let mut duoi_loi = None;
    if !ds.is_empty() && het < pdf.len() {
        match crate::dss::f_2p3(pdf, het, None) {
            Ok(t) => {
                duoi_la_dss = true;
                co_dss = t.has_dss;
                co_dau_thoi_gian_tai_lieu = t.archive_present && t.archive_structural_ok;
            }
            Err(e) => duoi_loi = Some(format!("{e}")),
        }
    }
    KetQuaNhieu {
        chu_ky: ds,
        moc_ban_sua: moc,
        byte_thua_cuoi,
        duoi_la_dss,
        co_dss,
        co_dau_thoi_gian_tai_lieu,
        duoi_loi,
    }
}

/// Có phải PDF có ít nhất một ô chữ ký — rẻ, chỉ tìm chuỗi, không phân tích gì.
pub fn co_chu_ky(pdf: &[u8]) -> bool {
    tim(pdf, b"/ByteRange").is_some()
}


/// Kết luận cho **một tệp**, kèm lý do. Tách `loi` / `canh_bao` theo đúng khuôn
/// `digitext_pkgcheck`: lỗi làm tệp không đạt, cảnh báo thì không.
#[derive(Debug, Clone, Default)]
pub struct KetLuanTep {
    /// `"Đạt"` · `"Đạt, có cảnh báo"` · `"Không đạt"` · `"Chưa ký"`.
    pub ket_luan: String,
    pub loi: Vec<String>,
    pub canh_bao: Vec<String>,
}

/// Mốc thời gian dùng để xét hiệu lực chứng thư, tính theo giây Unix.
///
/// Là **tham số** chứ không đọc đồng hồ bên trong: kết luận về chữ ký phải dựng lại được y
/// nguyên khi kiểm lại sau này, và test không chạy được nếu kết quả đổi theo ngày.
pub fn ket_luan_tep(kq: &KetQuaNhieu, bay_gio_unix: i64) -> KetLuanTep {
    let mut loi = Vec::new();
    let mut canh_bao = Vec::new();

    if kq.chu_ky.is_empty() {
        return KetLuanTep { ket_luan: "Chưa ký".into(), loi, canh_bao };
    }

    // Byte nối thêm sau `%%EOF` cuối: không bản sửa nào nhận, tức có người ghi thêm vào tệp.
    // Khác hẳn chuyện một chữ ký không phủ tới cuối tệp — xem ghi chú đầu module.
    if kq.byte_thua_cuoi > 0 {
        loi.push(format!(
            "có {} byte nối thêm sau dấu kết thúc tệp (%%EOF) — không thuộc bản sửa nào",
            kq.byte_thua_cuoi
        ));
    }

    for c in &kq.chu_ky {
        let ten = c.nhan();
        if let Some(e) = &c.loi {
            loi.push(format!("{ten}: không xác minh được — {e}"));
            continue;
        }
        if !c.vung_khop_o_chu_ky {
            loi.push(format!(
                "{ten}: vùng ký khai trong tệp không bao quanh ô chữ ký của chính nó — tệp đã bị can thiệp"
            ));
            continue;
        }
        if !c.toan_ven {
            loi.push(format!("{ten}: KHÔNG toàn vẹn — phần tệp mà chữ ký này phủ đã bị sửa sau khi ký"));
            continue;
        }
        // Từ đây: chữ ký đúng, băm khớp. Phần còn lại là chuyện chất lượng chữ ký.
        if !c.gan_chung_thu {
            canh_bao.push(format!(
                "{ten}: không gắn chứng thư người ký (thiếu thuộc tính ESS signing-certificate, chưa đạt PAdES cơ bản)"
            ));
        }
        match (c.co_tsa, c.thoi_diem_tsa_unix) {
            (true, Some(t)) => {
                if let Some(ng) = &c.nguoi_ky {
                    if t < ng.not_before_unix || t > ng.not_after_unix {
                        loi.push(format!("{ten}: ký ngoài thời hạn hiệu lực của chứng thư"));
                    }
                }
            }
            _ => {
                canh_bao.push(format!(
                    "{ten}: không có dấu thời gian (TSA) — không chứng minh được ký lúc nào"
                ));
                // Không TSA + chứng thư đã hết hạn = về lâu dài không còn chứng minh được. Đây
                // là lý do TT05 đòi PAdES-LTV cho tài liệu lưu trữ.
                if let Some(ng) = &c.nguoi_ky {
                    if bay_gio_unix > ng.not_after_unix {
                        canh_bao.push(format!(
                            "{ten}: chứng thư đã hết hạn mà chữ ký không có dấu thời gian — không còn chứng minh được chữ ký có từ lúc chứng thư còn hiệu lực"
                        ));
                    }
                }
            }
        }
    }

    // Phần đuôi sau chữ ký sau cùng. CHỈ là lỗi khi nó không phải increment DSS/DocTimeStamp:
    // tệp PAdES-LT nối dữ liệu xác minh dài hạn thành một bản sửa SAU chữ ký, nên ở đó không
    // chữ ký nào phủ tới cuối tệp mà tệp vẫn đúng — đúng chuẩn hơn tệp không có DSS.
    if !loi.iter().any(|l| l.contains("nối thêm")) && !kq.chu_ky.iter().any(|c| c.phu_tron_tep) && !kq.duoi_la_dss {
        let vi = kq.duoi_loi.as_deref().unwrap_or("không nhận ra là bản sửa hợp lệ");
        loi.push(format!(
            "phần cuối tệp không chữ ký nào bảo vệ, và không phải bản sửa dữ liệu xác minh (DSS): {vi}"
        ));
    }

    let ket_luan = if !loi.is_empty() {
        "Không đạt"
    } else if !canh_bao.is_empty() {
        "Đạt, có cảnh báo"
    } else {
        "Đạt"
    };
    KetLuanTep { ket_luan: ket_luan.into(), loi, canh_bao }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{finalize, prepare, verify};
    use der::Encode;
    use lopdf::{dictionary, Document, Object};
    use rsa::pkcs1v15::SigningKey;
    use rsa::pkcs8::EncodePublicKey;
    use rsa::signature::{SignatureEncoding, Signer};
    use sha2::Sha256;

    fn trang_trong() -> Vec<u8> {
        let mut doc = Document::with_version("1.7");
        let pages_id = doc.new_object_id();
        let page = doc.add_object(dictionary! {
            "Type" => "Page", "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        });
        doc.objects.insert(
            pages_id,
            Object::Dictionary(dictionary! { "Type" => "Pages", "Kids" => vec![page.into()], "Count" => 1 }),
        );
        let catalog = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
        doc.trailer.set("Root", catalog);
        let mut out = Vec::new();
        doc.save_to(&mut out).unwrap();
        out
    }

    fn khoa() -> (SigningKey<Sha256>, Vec<u8>) {
        use der::Decode;
        use x509_cert::builder::{Builder, CertificateBuilder, Profile};
        use x509_cert::name::Name;
        use x509_cert::serial_number::SerialNumber;
        use x509_cert::time::Validity;
        let mut rng = rand::thread_rng();
        let sk = rsa::RsaPrivateKey::new(&mut rng, 2048).unwrap();
        let signer = SigningKey::<Sha256>::new(sk.clone());
        let spki_der = sk.to_public_key().to_public_key_der().unwrap();
        let spki = spki::SubjectPublicKeyInfoOwned::from_der(spki_der.as_bytes()).unwrap();
        let ten: Name = "CN=Thu Nghiem,O=DigiText,C=VN".parse().unwrap();
        let cert = CertificateBuilder::new(
            Profile::Root,
            SerialNumber::from(1u32),
            Validity::from_now(core::time::Duration::from_secs(3600 * 24 * 365)).unwrap(),
            ten,
            spki,
            &signer,
        )
        .unwrap()
        .build()
        .unwrap();
        (signer, cert.to_der().unwrap())
    }

    fn ky(pdf: &[u8], signer: &SigningKey<Sha256>, cert: &[u8], luc: u64) -> Vec<u8> {
        let thg = prepare(pdf, cert, luc).expect("prepare");
        let sig = signer.sign(&thg.signed_attrs_der).to_bytes().to_vec();
        finalize(&thg, &sig, cert).expect("finalize")
    }

    /// PDF hai ô chữ ký, ô ĐẦU khai `/ByteRange` chỉ phủ phần đầu tệp — hình dáng công văn
    /// được ký tăng dần bởi công cụ khác, nơi chỉ chữ ký CUỐI phủ trọn tệp.
    fn pdf_cu_dung_truoc() -> Vec<u8> {
        let mut doc = Document::with_version("1.7");
        let pages_id = doc.new_object_id();
        let page = doc.add_object(dictionary! {
            "Type" => "Page", "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        });
        doc.objects.insert(
            pages_id,
            Object::Dictionary(dictionary! { "Type" => "Pages", "Kids" => vec![page.into()], "Count" => 1 }),
        );
        let cu = doc.add_object(dictionary! {
            "Type" => "Sig", "Name" => Object::string_literal("KY-CU"),
            "ByteRange" => vec![0.into(), 100.into(), 200.into(), 120.into()],
            "Contents" => Object::String(vec![0u8; 64], lopdf::StringFormat::Hexadecimal),
        });
        let moi = doc.add_object(dictionary! {
            "Type" => "Sig", "Name" => Object::string_literal("KY-MOI"),
            "ByteRange" => vec![0.into(), 100.into(), 200.into(), 900.into()],
            "Contents" => Object::String(vec![0u8; 64], lopdf::StringFormat::Hexadecimal),
        });
        let f_cu = doc.add_object(dictionary! { "FT" => "Sig", "T" => Object::string_literal("Sig1"), "V" => cu });
        let f_moi = doc.add_object(dictionary! { "FT" => "Sig", "T" => Object::string_literal("Sig2"), "V" => moi });
        let acro = doc.add_object(dictionary! { "Fields" => vec![f_cu.into(), f_moi.into()] });
        let catalog = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id, "AcroForm" => acro });
        doc.trailer.set("Root", catalog);
        let mut out = Vec::new();
        doc.save_to(&mut out).unwrap();
        out
    }

    /// Nền: ký một lần thì thấy đúng một chữ ký và nó đạt. Thiếu bài này thì các bài sau
    /// không chứng minh được gì — thất bại có thể chỉ do khoá test dựng sai.
    #[test]
    fn mot_chu_ky_thi_thay_mot_va_dat() {
        let (s, c) = khoa();
        let pdf = ky(&trang_trong(), &s, &c, 1_765_000_000);
        let kq = kiem_tat_ca(&pdf);
        assert_eq!(kq.so_chu_ky(), 1, "phải thấy đúng 1 chữ ký");
        assert!(kq.chu_ky[0].dat(), "chữ ký phải đạt: {:?}", kq.chu_ky[0]);
        assert!(kq.chu_ky[0].phu_tron_tep, "chữ ký duy nhất phải phủ trọn tệp");
        assert_eq!(kq.byte_thua_cuoi, 0, "không được có byte lạ sau %%EOF");
        assert!(kq.tat_ca_dat());
        assert_eq!(kq.chu_ky[0].nguoi_ky.as_ref().map(|n| n.signer.as_str()), Some("Thu Nghiem"));
    }

    /// **Chỗ bản cũ sai.** Chữ ký cũ đứng đầu `/AcroForm/Fields` thì `verify` kết luận "PDF đã
    /// bị sửa sau khi ký" — vu oan cho văn bản hợp lệ. Bộ mới phải thấy CẢ HAI chữ ký và
    /// không nói câu đó.
    #[test]
    fn chu_ky_cu_dung_truoc_khong_bi_vu_la_da_sua() {
        let pdf = pdf_cu_dung_truoc();
        // Ghim hành vi bản cũ, để biết chính xác bộ mới sửa cái gì.
        let loi_cu = verify(&pdf).expect_err("bản cũ phải báo lỗi — đó là lý do có module này");
        assert!(format!("{loi_cu}").contains("bị sửa sau khi ký"), "{loi_cu}");
        let kq = kiem_tat_ca(&pdf);
        assert_eq!(kq.so_chu_ky(), 2, "phải thấy CẢ HAI ô chữ ký");
        assert_eq!(kq.chu_ky[0].ten_khai.as_deref(), Some("KY-CU"), "chữ ký cũ phải xếp thứ 1");
        assert_eq!(kq.chu_ky[1].ten_khai.as_deref(), Some("KY-MOI"));
        assert!(!kq.chu_ky[0].phu_tron_tep, "chữ ký cũ không phủ trọn tệp — chuyện bình thường");
    }

    /// Ô chữ ký khai `/ByteRange` **không** bao quanh ô `/Contents` của nó: phải bị bắt.
    /// Không có phép kiểm này thì kẻ sửa tệp chỉ cần trỏ vùng phủ vào đoạn mình không sửa.
    #[test]
    fn vung_khai_khong_bao_o_chu_ky_thi_khong_dat() {
        let kq = kiem_tat_ca(&pdf_cu_dung_truoc());
        for c in &kq.chu_ky {
            assert!(!c.vung_khop_o_chu_ky, "vùng bịa phải bị bắt: {c:?}");
            assert!(!c.dat(), "không khớp ô chữ ký thì không được coi là đạt");
        }
    }

    /// Byte nối thêm sau `%%EOF` cuối phải bị đếm — đó là can thiệp thật, khác hẳn việc một
    /// chữ ký không phủ tới cuối tệp (chuyện thường của văn bản nhiều chữ ký).
    #[test]
    fn byte_noi_them_sau_eof_bi_dem() {
        let (s, c) = khoa();
        let mut pdf = ky(&trang_trong(), &s, &c, 1_765_000_000);
        assert_eq!(kiem_tat_ca(&pdf).byte_thua_cuoi, 0);
        pdf.extend_from_slice(b"RAC NOI THEM");
        let kq = kiem_tat_ca(&pdf);
        assert_eq!(kq.byte_thua_cuoi, 12, "phải đếm đúng 12 byte lạ");
        assert!(!kq.tat_ca_dat(), "có byte lạ nối sau cùng thì không được kết luận đạt");
    }

    /// Sửa một byte TRONG vùng chữ ký phủ: phải thành không toàn vẹn. Đây là phép kiểm chứng
    /// minh bộ mới vẫn bắt được sửa thật, chứ không phải nới tay cho mọi thứ qua.
    #[test]
    fn sua_byte_trong_vung_phu_thi_mat_toan_ven() {
        let (s, c) = khoa();
        let pdf = ky(&trang_trong(), &s, &c, 1_765_000_000);
        assert!(kiem_tat_ca(&pdf).tat_ca_dat());
        let vung = kiem_tat_ca(&pdf).chu_ky[0].vung;
        let mut hong = pdf.clone();
        // Đổi một byte trong đoạn đầu mà chữ ký phủ.
        let i = vung[0] + vung[1] / 2;
        hong[i] ^= 0xFF;
        let kq = kiem_tat_ca(&hong);
        assert_eq!(kq.so_chu_ky(), 1);
        assert!(!kq.chu_ky[0].toan_ven, "sửa byte trong vùng phủ phải làm mất toàn vẹn");
        assert!(!kq.tat_ca_dat());
    }

    /// Ký HAI lần bằng chính DigiText: in ra để thấy chữ ký thứ nhất còn toàn vẹn hay không.
    #[test]
    fn ky_hai_lan_bang_digitext_thi_chu_ky_dau_the_nao() {
        let (s, c) = khoa();
        let mot = ky(&trang_trong(), &s, &c, 1_765_000_000);
        let hai = ky(&mot, &s, &c, 1_765_000_900);
        let kq = kiem_tat_ca(&hai);
        println!("== thay {} chu ky", kq.so_chu_ky());
        for ck in &kq.chu_ky {
            println!(
                "   #{} phu_den={} tron={} khop_o={} toan_ven={} loi={:?}",
                ck.thu_tu, ck.phu_den, ck.phu_tron_tep, ck.vung_khop_o_chu_ky, ck.toan_ven, ck.loi
            );
        }
        assert_eq!(kq.so_chu_ky(), 2, "ky hai lan thi phai thay hai chu ky");
    }
}

#[cfg(test)]
mod ket_luan_tests {
    use super::*;

    fn ck(toan_ven: bool, khop: bool, tron: bool) -> ChuKyPdf {
        ChuKyPdf {
            thu_tu: 1,
            loai: "Sig".into(),
            khuon_dang: Some("ETSI.CAdES.detached".into()),
            ten_khai: None,
            ly_do: None,
            noi: None,
            thoi_diem_khai: None,
            vung: [0, 10, 20, 30],
            phu_den: 50,
            phu_tron_tep: tron,
            vung_khop_o_chu_ky: khop,
            toan_ven,
            gan_chung_thu: true,
            co_tsa: true,
            thoi_diem_tsa_unix: Some(1_765_000_000),
            nguoi_ky: Some(SignerInfo {
                signer: "Nguyen Van A".into(),
                subject: "CN=Nguyen Van A".into(),
                issuer: "VGCA".into(),
                not_before_unix: 1_700_000_000,
                not_after_unix: 1_800_000_000,
            }),
            loi: None,
        }
    }
    fn kq(ds: Vec<ChuKyPdf>, thua: usize) -> KetQuaNhieu {
        KetQuaNhieu {
            chu_ky: ds,
            moc_ban_sua: vec![50],
            byte_thua_cuoi: thua,
            duoi_la_dss: false,
            co_dss: false,
            co_dau_thoi_gian_tai_lieu: false,
            duoi_loi: None,
        }
    }

    #[test]
    fn khong_co_chu_ky_thi_la_chua_ky_chu_khong_phai_khong_dat() {
        let k = ket_luan_tep(&kq(Vec::new(), 0), 1_765_000_000);
        assert_eq!(k.ket_luan, "Chưa ký");
        assert!(k.loi.is_empty(), "chưa ký là một TRẠNG THÁI, không phải lỗi của tệp");
    }

    /// Chữ ký cũ trong văn bản nhiều chữ ký **không phủ trọn tệp** — và điều đó KHÔNG được
    /// thành lỗi. Đây là cả lý do module này tồn tại, nên phải có test ghim.
    #[test]
    fn chu_ky_cu_khong_phu_tron_tep_khong_bi_coi_la_loi() {
        let mut cu = ck(true, true, false);
        cu.thu_tu = 1;
        let mut moi = ck(true, true, true);
        moi.thu_tu = 2;
        let k = ket_luan_tep(&kq(vec![cu, moi], 0), 1_765_000_000);
        assert_eq!(k.ket_luan, "Đạt", "lỗi: {:?}", k.loi);
        assert!(k.loi.is_empty(), "{:?}", k.loi);
    }

    /// Đuôi không ai bảo vệ VÀ không phải increment DSS: lỗi.
    #[test]
    fn duoi_khong_ai_bao_ve_va_khong_phai_dss_thi_la_loi() {
        let mut q = kq(vec![ck(true, true, false)], 0);
        q.duoi_loi = Some("increment đổi /Root".into());
        let k = ket_luan_tep(&q, 1_765_000_000);
        assert_eq!(k.ket_luan, "Không đạt");
        assert!(k.loi.iter().any(|l| l.contains("không chữ ký nào bảo vệ")), "{:?}", k.loi);
        assert!(k.loi.iter().any(|l| l.contains("đổi /Root")), "phải nói rõ vì sao: {:?}", k.loi);
    }

    /// **Chỗ dễ kết luận sai nhất sau khi sửa.** Tệp PAdES-LT: dữ liệu xác minh dài hạn (DSS)
    /// nối thành một bản sửa SAU chữ ký, nên không chữ ký nào phủ tới cuối tệp — nhưng đó là
    /// tệp ký đúng chuẩn HƠN, không phải tệp bị sửa. Không được báo lỗi.
    #[test]
    fn duoi_la_dss_thi_khong_phai_loi_du_khong_chu_ky_nao_phu_het() {
        let mut q = kq(vec![ck(true, true, false)], 0);
        q.duoi_la_dss = true;
        q.co_dss = true;
        let k = ket_luan_tep(&q, 1_765_000_000);
        assert_eq!(k.ket_luan, "Đạt", "lỗi: {:?}", k.loi);
        assert!(k.loi.is_empty(), "{:?}", k.loi);
    }

    #[test]
    fn mat_toan_ven_va_vung_bia_deu_la_loi() {
        let k = ket_luan_tep(&kq(vec![ck(false, true, true)], 0), 1_765_000_000);
        assert_eq!(k.ket_luan, "Không đạt");
        assert!(k.loi.iter().any(|l| l.contains("KHÔNG toàn vẹn")), "{:?}", k.loi);
        let k = ket_luan_tep(&kq(vec![ck(true, false, true)], 0), 1_765_000_000);
        assert_eq!(k.ket_luan, "Không đạt");
        assert!(k.loi.iter().any(|l| l.contains("can thiệp")), "{:?}", k.loi);
    }

    /// Byte nối thêm sau `%%EOF` là lỗi, và khi đã có lỗi đó thì KHÔNG báo thêm câu "không chữ
    /// ký nào phủ hết tệp" — cùng một nguyên nhân, nói hai câu là làm người đọc tưởng hai lỗi.
    #[test]
    fn byte_noi_them_chi_bao_mot_cau() {
        let k = ket_luan_tep(&kq(vec![ck(true, true, false)], 12), 1_765_000_000);
        assert_eq!(k.ket_luan, "Không đạt");
        assert_eq!(k.loi.len(), 1, "{:?}", k.loi);
        assert!(k.loi[0].contains("nối thêm"));
    }

    #[test]
    fn ky_ngoai_thoi_han_chung_thu_la_loi() {
        let mut c = ck(true, true, true);
        c.thoi_diem_tsa_unix = Some(1_900_000_000); // sau not_after
        let k = ket_luan_tep(&kq(vec![c], 0), 1_765_000_000);
        assert_eq!(k.ket_luan, "Không đạt");
        assert!(k.loi.iter().any(|l| l.contains("ngoài thời hạn")), "{:?}", k.loi);
    }

    /// Không TSA: chỉ cảnh báo. Nhưng nếu chứng thư ĐÃ hết hạn thì nói thêm rằng chữ ký không
    /// còn chứng minh được — đó mới là điều quan trọng với tài liệu lưu trữ lâu dài.
    #[test]
    fn khong_tsa_la_canh_bao_va_noi_them_khi_chung_thu_het_han() {
        let mut c = ck(true, true, true);
        c.co_tsa = false;
        c.thoi_diem_tsa_unix = None;
        let k = ket_luan_tep(&kq(vec![c.clone()], 0), 1_765_000_000);
        assert_eq!(k.ket_luan, "Đạt, có cảnh báo");
        assert_eq!(k.canh_bao.len(), 1, "{:?}", k.canh_bao);
        // Cùng tệp đó, xét ở thời điểm sau khi chứng thư hết hạn.
        let k = ket_luan_tep(&kq(vec![c], 0), 1_900_000_000);
        assert_eq!(k.canh_bao.len(), 2, "{:?}", k.canh_bao);
        assert!(k.canh_bao.iter().any(|w| w.contains("không còn chứng minh được")), "{:?}", k.canh_bao);
    }

    /// Nhãn chữ ký phải lấy `/Reason` — VGCA ghi ở đó để phân biệt phê duyệt / số / ngày / dấu.
    #[test]
    fn nhan_chu_ky_lay_theo_ly_do_ky() {
        let mut c = ck(true, true, true);
        c.ly_do = Some("Dau to chuc".into());
        assert_eq!(c.nhan(), "Chữ ký #1 (Dau to chuc)");
        c.ly_do = None;
        c.ten_khai = Some("Pham Cong Thao".into());
        assert_eq!(c.nhan(), "Chữ ký #1 (Pham Cong Thao)");
        c.ten_khai = None;
        assert_eq!(c.nhan(), "Chữ ký #1");
        c.loai = "DocTimeStamp".into();
        assert_eq!(c.nhan(), "Dấu thời gian tài liệu #1");
    }
}

#[cfg(test)]
mod tep_that_tests {
    use super::*;

    /// Đối chiếu bộ MỚI với bộ CŨ trên PDF thật trong `test/`. Chạy tay:
    /// `cargo test -p digitext-sign-verify doi_chieu_tep_that -- --ignored --nocapture`
    ///
    /// Mục đích không phải tìm lỗi của tệp, mà chứng minh bộ mới **không làm hỏng** thứ bộ cũ
    /// đang kết luận đúng: PDF lưu trữ ký một lần phải cho cùng một câu trả lời.
    #[test]
    #[ignore = "cần thư mục test/ có PDF đã ký"]
    fn doi_chieu_tep_that() {
        let goc = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test");
        let mut ds = Vec::new();
        gom(&goc, &mut ds);
        assert!(!ds.is_empty(), "không thấy PDF nào dưới {}", goc.display());
        let mut lech = 0;
        for p in &ds {
            let Ok(b) = std::fs::read(p) else { continue };
            if !co_chu_ky(&b) {
                continue;
            }
            let moi = kiem_tat_ca(&b);
            let cu = crate::verify(&b);
            let ten = p.file_name().unwrap_or_default().to_string_lossy().to_string();
            let k = ket_luan_tep(&moi, 1_795_000_000);
            let cu_noi = match &cu {
                Ok(t) => format!("toan_ven={}", t.signature_ok && t.digest_ok),
                Err(e) => format!("LOI({e})"),
            };
            let moi_dat = moi.tat_ca_dat();
            let cu_dat = matches!(&cu, Ok(t) if t.signature_ok && t.digest_ok);
            if moi_dat != cu_dat {
                lech += 1;
            }
            println!(
                "{:<46} so_ck={} moi_dat={} | cu: {} | ket_luan={} {}",
                ten,
                moi.so_chu_ky(),
                moi_dat,
                cu_noi,
                k.ket_luan,
                if moi_dat != cu_dat { "<<< LECH" } else { "" }
            );
            for x in k.loi.iter().chain(k.canh_bao.iter()) {
                println!("        - {x}");
            }
        }
        println!("-- {} tep co chu ky, {} tep lech ket luan --", ds.len(), lech);
    }

    fn gom(d: &std::path::Path, ra: &mut Vec<std::path::PathBuf>) {
        let Ok(rd) = std::fs::read_dir(d) else { return };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                gom(&p, ra);
            } else if p.extension().is_some_and(|x| x.eq_ignore_ascii_case("pdf")) {
                ra.push(p);
            }
        }
    }
}

#[cfg(test)]
mod cat_nhan_tests {
    use super::cat_nhan;

    #[test]
    fn cat_o_ranh_gioi_tu_va_khong_lam_hong_chu_tieng_viet() {
        assert_eq!(cat_nhan("Dau to chuc", 40), "Dau to chuc", "đủ ngắn thì giữ nguyên, không thêm dấu …");
        let dai = "ỦY BAN NHÂN DÂN ĐẶC KHU CÔ TÔ đã ký lên văn bản này";
        let ra = cat_nhan(dai, 40);
        assert!(ra.ends_with('…'), "{ra}");
        assert!(ra.chars().count() <= 41, "{} ký tự: {ra}", ra.chars().count());
        assert!(dai.starts_with(ra.trim_end_matches('…')), "phải là đoạn đầu nguyên văn: {ra}");
        // Cắt theo KÝ TỰ, không theo byte: tiếng Việt 2–3 byte/chữ, cắt theo byte là ra ký tự hỏng.
        assert!(ra.chars().all(|c| c != '\u{FFFD}'), "{ra}");
        // Khoảng trắng thừa trong `/Reason` bị dồn lại — tệp thật có "NHÂN DÂN  XÃ" hai dấu cách.
        assert_eq!(cat_nhan("ỦY BAN  NHÂN   DÂN", 40), "ỦY BAN NHÂN DÂN");
        // Một từ dài hơn cả trần: vẫn phải cắt, không được trả chuỗi rỗng.
        let mot_tu = "a".repeat(100);
        assert_eq!(cat_nhan(&mot_tu, 10), format!("{}…", "a".repeat(10)));
    }
}
