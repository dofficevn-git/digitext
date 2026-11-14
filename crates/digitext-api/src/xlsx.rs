// Copyright (c) 2026 DigiText. All rights reserved.
//! Đóng gói `.xlsx` tối thiểu — đủ để phát **tệp mẫu mục lục hồ sơ** cho nhà thầu.
//!
//! Vì sao tự viết thay vì thêm một crate ghi Excel: workspace đã có `zip`, mà `.xlsx` chỉ là một
//! zip chứa vài tệp XML. Thêm phụ thuộc mới để ghi 3 sheet không đáng, và bản này không phải
//! đọc tệp của ai — nó chỉ sinh ra tệp của chính mình.
//!
//! Chỗ cần biết nếu sau này mở rộng:
//! - Chuỗi ghi **inline** (`t="inlineStr"`), không dùng `sharedStrings.xml`. Tốn chỗ hơn nhưng
//!   bỏ được cả một tệp và một bảng tra; tệp mẫu chỉ vài chục dòng.
//! - `styles.xml` có đúng **hai** kiểu ô: `s="0"` thường, `s="1"` đậm. Thêm kiểu thì phải thêm
//!   vào cả `fonts` lẫn `cellXfs` và `count` phải khớp, nếu không Excel báo tệp hỏng.
//! - Ô số ghi không có `t=` (mặc định là số). Ô số mà kẹp vào `inlineStr` thì Excel hiện tam
//!   giác xanh "number stored as text".
//! - Bỏ qua `docProps/*`: Excel và LibreOffice đều mở được tệp không có chúng.

use digitext_mucluc::mau::{cot_ten, Kieu, SheetMau};
use std::io::Write;

/// Thoát ký tự cho nội dung XML, và bỏ các ký tự điều khiển XML 1.0 không chấp nhận (ô Excel
/// không bao giờ nên có chúng, nhưng tên THBQ là dữ liệu người dùng nhập).
fn thoat(s: &str) -> String {
    let mut r = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '&' => r.push_str("&amp;"),
            '<' => r.push_str("&lt;"),
            '>' => r.push_str("&gt;"),
            '"' => r.push_str("&quot;"),
            '\'' => r.push_str("&apos;"),
            '\t' | '\n' | '\r' => r.push(c),
            c if (c as u32) < 0x20 => r.push(' '),
            c => r.push(c),
        }
    }
    r
}

/// Tên sheet hợp lệ: Excel cấm `: \ / ? * [ ]` và giới hạn 31 ký tự.
fn ten_sheet(s: &str) -> String {
    let sach: String = s.chars().map(|c| if matches!(c, ':' | '\\' | '/' | '?' | '*' | '[' | ']') { ' ' } else { c }).collect();
    let sach = sach.trim().to_string();
    if sach.chars().count() <= 31 {
        return sach;
    }
    sach.chars().take(31).collect()
}

/// Chỉ số kiểu ô trong `cellXfs` của [`STYLES`]. Thêm kiểu ở `digitext_mucluc::mau::Kieu` mà
/// quên ở đây thì ô rơi về kiểu thường — `match` không có nhánh `_` chính là để trình biên dịch
/// bắt chuyện đó.
fn kieu_so(k: Kieu) -> u32 {
    match k {
        Kieu::Thuong => 0,
        Kieu::Dam => 1,
        Kieu::Tieu => 2,
        Kieu::Dan => 3,
        Kieu::TieuDeCot => 4,
        Kieu::O => 5,
        Kieu::OGiua => 6,
        Kieu::Nhom => 7,
    }
}

fn xml_sheet(s: &SheetMau) -> String {
    let mut x = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <worksheet xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\">",
    );

    // sheetViews phải đứng TRƯỚC cols và sheetData — lược đồ OOXML ép thứ tự, sai là Excel báo
    // tệp hỏng chứ không bỏ qua.
    x.push_str("<sheetViews><sheetView workbookViewId=\"0\"");
    if !s.luoi {
        x.push_str(" showGridLines=\"0\"");
    }
    x.push('>');
    if let Some(n) = s.dong_bang.filter(|n| *n > 1) {
        x.push_str(&format!(
            "<pane ySplit=\"{}\" topLeftCell=\"A{n}\" activePane=\"bottomLeft\" state=\"frozen\"/>",
            n - 1
        ));
    }
    x.push_str("</sheetView></sheetViews>");

    if !s.rong.is_empty() {
        x.push_str("<cols>");
        for (i, w) in s.rong.iter().enumerate() {
            x.push_str(&format!(
                "<col min=\"{n}\" max=\"{n}\" width=\"{w}\" customWidth=\"1\"/>",
                n = i + 1
            ));
        }
        x.push_str("</cols>");
    }

    x.push_str("<sheetData>");
    for (ri, hang) in s.rows.iter().enumerate() {
        // Dòng chỉ bỏ hẳn khi KHÔNG có gì để vẽ: không chữ và cũng không kiểu nào cần nền/viền.
        let co_gi = hang.iter().any(|o| !o.v.is_empty() || o.kieu != Kieu::Thuong);
        let cao = s.cao.get(ri).copied().flatten();
        if !co_gi && cao.is_none() {
            continue;
        }
        match cao {
            Some(h) => x.push_str(&format!("<row r=\"{}\" ht=\"{h}\" customHeight=\"1\">", ri + 1)),
            None => x.push_str(&format!("<row r=\"{}\">", ri + 1)),
        }
        for (ci, o) in hang.iter().enumerate() {
            // Ô rỗng kiểu thường không cần tồn tại; ô rỗng có viền thì cần, nếu không bảng thủng.
            if o.v.is_empty() && o.kieu == Kieu::Thuong {
                continue;
            }
            let dc = format!("{}{}", cot_ten(ci), ri + 1);
            let st = kieu_so(o.kieu);
            // Chỉ ghi dạng số khi thực sự phân tích được, kẻo sinh ra XML Excel từ chối mở.
            match o.so.then(|| o.v.parse::<f64>().ok()).flatten() {
                Some(_) => x.push_str(&format!("<c r=\"{dc}\" s=\"{st}\"><v>{}</v></c>", thoat(&o.v))),
                None if o.v.is_empty() => x.push_str(&format!("<c r=\"{dc}\" s=\"{st}\"/>")),
                None => x.push_str(&format!(
                    "<c r=\"{dc}\" s=\"{st}\" t=\"inlineStr\"><is><t xml:space=\"preserve\">{}</t></is></c>",
                    thoat(&o.v)
                )),
            }
        }
        x.push_str("</row>");
    }
    x.push_str("</sheetData>");

    // Thứ tự bắt buộc của lược đồ: sheetData → mergeCells → … → ignoredErrors.
    if !s.gop.is_empty() {
        x.push_str(&format!("<mergeCells count=\"{}\">", s.gop.len()));
        for g in &s.gop {
            x.push_str(&format!("<mergeCell ref=\"{}\"/>", thoat(g)));
        }
        x.push_str("</mergeCells>");
    }
    if !s.bo_qua_loi_so.is_empty() {
        x.push_str("<ignoredErrors>");
        for v in &s.bo_qua_loi_so {
            x.push_str(&format!(
                "<ignoredError sqref=\"{}\" numberStoredAsText=\"1\"/>",
                thoat(v)
            ));
        }
        x.push_str("</ignoredErrors>");
    }

    x.push_str("</worksheet>");
    x
}

/// Bảng kiểu. **Thứ tự `cellXfs` phải khớp [`kieu_so`]**, và mọi `count` phải đúng số phần tử —
/// sai một con số là Excel báo "tệp cần sửa" rồi vứt hết định dạng.
///
/// Màu lấy theo nhận diện 0.19: tím `5A58A6` cho dòng tiêu đề, `EEEEF6` cho dòng phòng ban/mảng,
/// viền `D5D5DF` đủ thấy mà không ồn.
const STYLES: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
<styleSheet xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\">\
<fonts count=\"5\">\
<font><sz val=\"11\"/><name val=\"Calibri\"/></font>\
<font><b/><sz val=\"11\"/><name val=\"Calibri\"/></font>\
<font><b/><sz val=\"11\"/><color rgb=\"FFFFFFFF\"/><name val=\"Calibri\"/></font>\
<font><i/><sz val=\"10\"/><color rgb=\"FF6B6B78\"/><name val=\"Calibri\"/></font>\
<font><b/><sz val=\"13\"/><color rgb=\"FF2A2A3A\"/><name val=\"Calibri\"/></font>\
</fonts>\
<fills count=\"4\">\
<fill><patternFill patternType=\"none\"/></fill>\
<fill><patternFill patternType=\"gray125\"/></fill>\
<fill><patternFill patternType=\"solid\"><fgColor rgb=\"FF5A58A6\"/><bgColor indexed=\"64\"/></patternFill></fill>\
<fill><patternFill patternType=\"solid\"><fgColor rgb=\"FFEEEEF6\"/><bgColor indexed=\"64\"/></patternFill></fill>\
</fills>\
<borders count=\"2\">\
<border><left/><right/><top/><bottom/><diagonal/></border>\
<border>\
<left style=\"thin\"><color rgb=\"FFD5D5DF\"/></left>\
<right style=\"thin\"><color rgb=\"FFD5D5DF\"/></right>\
<top style=\"thin\"><color rgb=\"FFD5D5DF\"/></top>\
<bottom style=\"thin\"><color rgb=\"FFD5D5DF\"/></bottom>\
<diagonal/></border>\
</borders>\
<cellStyleXfs count=\"1\"><xf numFmtId=\"0\" fontId=\"0\" fillId=\"0\" borderId=\"0\"/></cellStyleXfs>\
<cellXfs count=\"8\">\
<xf numFmtId=\"0\" fontId=\"0\" fillId=\"0\" borderId=\"0\" xfId=\"0\"/>\
<xf numFmtId=\"0\" fontId=\"1\" fillId=\"0\" borderId=\"0\" xfId=\"0\" applyFont=\"1\"/>\
<xf numFmtId=\"0\" fontId=\"4\" fillId=\"0\" borderId=\"0\" xfId=\"0\" applyFont=\"1\" applyAlignment=\"1\"><alignment vertical=\"center\"/></xf>\
<xf numFmtId=\"0\" fontId=\"3\" fillId=\"0\" borderId=\"0\" xfId=\"0\" applyFont=\"1\" applyAlignment=\"1\"><alignment vertical=\"center\"/></xf>\
<xf numFmtId=\"0\" fontId=\"2\" fillId=\"2\" borderId=\"1\" xfId=\"0\" applyFont=\"1\" applyFill=\"1\" applyBorder=\"1\" applyAlignment=\"1\"><alignment horizontal=\"center\" vertical=\"center\" wrapText=\"1\"/></xf>\
<xf numFmtId=\"0\" fontId=\"0\" fillId=\"0\" borderId=\"1\" xfId=\"0\" applyBorder=\"1\" applyAlignment=\"1\"><alignment vertical=\"center\" wrapText=\"1\"/></xf>\
<xf numFmtId=\"0\" fontId=\"0\" fillId=\"0\" borderId=\"1\" xfId=\"0\" applyBorder=\"1\" applyAlignment=\"1\"><alignment horizontal=\"center\" vertical=\"center\"/></xf>\
<xf numFmtId=\"0\" fontId=\"1\" fillId=\"3\" borderId=\"1\" xfId=\"0\" applyFont=\"1\" applyFill=\"1\" applyBorder=\"1\" applyAlignment=\"1\"><alignment vertical=\"center\"/></xf>\
</cellXfs>\
<cellStyles count=\"1\"><cellStyle name=\"Normal\" xfId=\"0\" builtinId=\"0\"/></cellStyles>\
</styleSheet>";

/// Nén danh sách sheet thành tệp `.xlsx`.
pub fn dong_goi(sheets: &[SheetMau]) -> std::io::Result<Vec<u8>> {
    let mut w = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let opt = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    let mut ct = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\">\
         <Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/>\
         <Default Extension=\"xml\" ContentType=\"application/xml\"/>\
         <Override PartName=\"/xl/workbook.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml\"/>\
         <Override PartName=\"/xl/styles.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml\"/>",
    );
    let mut wb = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <workbook xmlns=\"http://schemas.openxmlformats.org/spreadsheetml/2006/main\" \
         xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\"><sheets>",
    );
    let mut rels = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">",
    );
    for (i, s) in sheets.iter().enumerate() {
        let n = i + 1;
        ct.push_str(&format!(
            "<Override PartName=\"/xl/worksheets/sheet{n}.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml\"/>"
        ));
        wb.push_str(&format!("<sheet name=\"{}\" sheetId=\"{n}\" r:id=\"rId{n}\"/>", thoat(&ten_sheet(&s.ten))));
        rels.push_str(&format!(
            "<Relationship Id=\"rId{n}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet\" Target=\"worksheets/sheet{n}.xml\"/>"
        ));
    }
    ct.push_str("</Types>");
    wb.push_str("</sheets></workbook>");
    rels.push_str(&format!(
        "<Relationship Id=\"rId{}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles\" Target=\"styles.xml\"/></Relationships>",
        sheets.len() + 1
    ));

    let mut ghi = |ten: &str, noi: &str| -> std::io::Result<()> {
        w.start_file(ten, opt)?;
        w.write_all(noi.as_bytes())
    };
    ghi("[Content_Types].xml", &ct)?;
    ghi(
        "_rels/.rels",
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
         <Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\">\
         <Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"xl/workbook.xml\"/>\
         </Relationships>",
    )?;
    ghi("xl/workbook.xml", &wb)?;
    ghi("xl/_rels/workbook.xml.rels", &rels)?;
    ghi("xl/styles.xml", STYLES)?;
    for (i, s) in sheets.iter().enumerate() {
        ghi(&format!("xl/worksheets/sheet{}.xml", i + 1), &xml_sheet(s))?;
    }
    Ok(w.finish()?.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;
    use digitext_mucluc::mau;

    /// Dùng lại hàm của `digitext-mucluc` — hai bản tính tên cột lệch nhau thì vùng gộp ô trỏ
    /// sai chỗ, mà Excel không kêu.
    #[test]
    fn ten_cot_excel() {
        assert_eq!(cot_ten(0), "A");
        assert_eq!(cot_ten(25), "Z");
        assert_eq!(cot_ten(26), "AA");
        assert_eq!(cot_ten(27), "AB");
        assert_eq!(cot_ten(51), "AZ");
        assert_eq!(cot_ten(52), "BA");
    }

    #[test]
    fn thoat_ky_tu_xml() {
        assert_eq!(thoat("a & b < c"), "a &amp; b &lt; c");
        assert_eq!(thoat("x\u{1}y"), "x y", "ký tự điều khiển phải thành dấu cách");
    }

    #[test]
    fn ten_sheet_bi_cat_va_bo_ky_tu_cam() {
        assert_eq!(ten_sheet("Mục lục hồ sơ"), "Mục lục hồ sơ");
        assert_eq!(ten_sheet("a/b:c"), "a b c");
        assert_eq!(ten_sheet(&"x".repeat(40)).chars().count(), 31);
    }

    /// Phép thử thật: đóng gói tệp mẫu rồi **đọc lại bằng chính bộ đọc MLHS**. Qua được nghĩa là
    /// tệp phát cho nhà thầu nộp ngược lên cửa tự kiểm sẽ chạy.
    #[test]
    fn mau_dong_goi_xong_doc_lai_duoc() {
        let dm = vec![("Vĩnh viễn".to_string(), true), ("20 năm".to_string(), false)];
        let tep = dong_goi(&mau::tep_mau(Some("Sở Nông nghiệp"), &dm)).expect("đóng gói được");
        assert!(tep.len() > 1000, "tệp quá nhỏ: {} byte", tep.len());
        assert_eq!(&tep[..2], b"PK", "phải là zip");

        let so = digitext_mucluc::doc_so_tu_bytes(&tep, None).expect("bộ đọc phải mở được tệp mẫu");
        assert_eq!(so.sheets.len(), 3);
        assert_eq!(so.sheet, "Mục lục hồ sơ");
        let ten: Vec<String> = dm.iter().map(|(t, _)| t.clone()).collect();
        let vv: Vec<String> = dm.iter().filter(|(_, v)| *v).map(|(t, _)| t.clone()).collect();
        let pt = digitext_mucluc::phan_tich::phan_tich_muc_luc(
            &so.rows,
            &digitext_mucluc::phan_tich::TuyChon {
                ten_tep: "mau.xlsx",
                sheet: &so.sheet,
                sheets: &so.sheets,
                thbq_danh_muc: &ten,
                thbq_vinh_vien: &vv,
                thbq_mac_dinh: None,
            },
        );
        assert_eq!(pt.ten_phong_trong_tep, "Sở Nông nghiệp");
        assert_eq!(pt.rows.len(), 3);
        assert_eq!(pt.so_loi, 0, "{:?}", pt.van_de);
        assert_eq!(pt.so_canh_bao, 0);
        // Số tờ ghi dạng số trong XML vẫn phải đọc ra "42", không phải "42.0".
        assert_eq!(pt.rows[0].so_to, "42");
        assert_eq!(pt.rows[0].hop_so, "1");
    }

    /// Xuất tệp mẫu ra đĩa để NHÌN bằng mắt. Không chạy trong bộ test thường.
    ///   cargo test -p digitext-api xuat_mau_de_xem -- --ignored --nocapture
    /// Xuất bảng «Kiểm tra» ra tệp để NHÌN. Lỗi trình bày (chữ bị cắt, cột hẹp) không test nào
    /// bắt được, phải dựng thành ảnh mà xem.
    ///   $env:BAO_CAO_RA="D:\tmp\bao-cao.xlsx"
    ///   cargo test -p digitext-api xuat_bao_cao_de_xem -- --ignored --nocapture
    #[test]
    #[ignore]
    fn xuat_bao_cao_de_xem() {
        use calamine::Data;
        let o = |v: &str| Data::String(v.to_string());
        let mut rows: Vec<Vec<Data>> = vec![
            vec![o("Hộp số"), o("Hồ sơ số"), o("Tiêu đề hồ sơ"), o("Thời gian"), o("Số tờ"), o("THBQ")],
            vec![o(""), o(""), o("I. VĂN PHÒNG HĐND VÀ UBND HUYỆN"), o(""), o(""), o("")],
            vec![o(""), o(""), o("1. Tài liệu tổng hợp"), o(""), o(""), o("")],
        ];
        for (i, (td, tg, so_to, thbq)) in [
            ("Hồ sơ Hội nghị tổng kết công tác năm 2020 của Ủy ban nhân dân huyện", "05/01/2020 - 28/12/2020", "124", "Vĩnh viễn"),
            ("Tập lưu quyết định của UBND huyện năm 2019", "", "86", "Vĩnh viễn"),
            ("Hồ sơ về việc giải quyết khiếu nại của công dân", "2020", "", "Không có thật"),
            ("Hồ sơ xây dựng cơ bản công trình đường liên xã năm 1700", "01/01/1700 - 31/12/1700", "40", "Vĩnh viễn"),
        ]
        .into_iter()
        .enumerate()
        {
            rows.push(vec![o("1"), o(&format!("{:02}", i + 1)), o(td), o(tg), o(so_to), o(thbq)]);
        }
        let dm = vec!["Vĩnh viễn".to_string()];
        let pt = digitext_mucluc::phan_tich::phan_tich_muc_luc(
            &rows,
            &digitext_mucluc::phan_tich::TuyChon {
                ten_tep: "muc-luc-vp-huyen.xlsx",
                sheet: "Sheet1",
                sheets: &["Sheet1".to_string()],
                thbq_danh_muc: &dm,
                thbq_vinh_vien: &dm,
                thbq_mac_dinh: None,
            },
        );
        let tep = dong_goi(&mau::bao_cao_kiem_tra(&pt)).unwrap();
        let ra = std::env::var("BAO_CAO_RA").unwrap_or_else(|_| "bao-cao-xem.xlsx".into());
        std::fs::write(&ra, &tep).unwrap();
        println!("đã ghi {ra} ({} byte)", tep.len());
    }

    #[test]
    #[ignore]
    fn xuat_mau_de_xem() {
        let dm = vec![
            ("Vĩnh viễn".to_string(), true),
            ("70 năm".to_string(), false),
            ("20 năm".to_string(), false),
            ("10 năm".to_string(), false),
            ("5 năm".to_string(), false),
        ];
        let tep = dong_goi(&mau::tep_mau(Some("Sở Nông nghiệp và PTNT Bắc Giang"), &dm)).unwrap();
        let ra = std::env::var("MAU_RA").unwrap_or_else(|_| "mau-xem.xlsx".into());
        std::fs::write(&ra, &tep).unwrap();
        println!("da ghi {ra} ({} byte)", tep.len());
    }

    /// Sheet "Danh mục THBQ" KHÔNG được nhận nhầm là mục lục khi nhà thầu chọn sheet đó.
    #[test]
    fn sheet_danh_muc_khong_phai_muc_luc() {
        let dm = vec![("Vĩnh viễn".to_string(), true)];
        let tep = dong_goi(&mau::tep_mau(None, &dm)).unwrap();
        let so = digitext_mucluc::doc_so_tu_bytes(&tep, Some("Danh mục THBQ")).unwrap();
        assert_eq!(so.sheet, "Danh mục THBQ", "ten sheet trong tep: {:?}", so.sheets);
        assert!(
            digitext_mucluc::tim_tieu_de_cot(&so.rows).is_none(),
            "sheet danh mục không được có dòng tiêu đề cột kiểu mục lục"
        );
    }
}
