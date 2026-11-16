// Copyright (c) 2026 DigiText. All rights reserved.
//! KIỂM KHO THEO BẢN KÊ — chạy đúng bước kiểm của tab «Đóng gói nhanh», nhưng trên máy chủ và
//! KHÔNG có đĩa.
//!
//! Vì sao làm được: `nhanh` ⇒ `deep = false`, mà nhánh mở PDF trong [`crate::check`] nằm trọn
//! sau `if deep`. Bước kiểm nông chỉ đọc **tên tệp + cỡ tệp + tệp Excel** — ba thứ trình duyệt
//! lấy được từ ô chọn thư mục mà không đọc một byte nội dung nào. Nên toàn bộ mã lỗi của
//! «Đóng gói nhanh» chạy nguyên vẹn ở đây, không phải bỏ phép kiểm nào (khác cửa kiểm gói tin,
//! nơi mã băm và PDF/A buộc phải bỏ vì thiếu byte).
//!
//! Việc riêng của bản kê là **căn lề đường dẫn**: Excel ghi `UBND\DCXD\HO SO 153\x.pdf`, còn ô
//! chọn thư mục của trình duyệt luôn gắn tên thư mục người dùng chọn vào đầu — có khi thừa một
//! lớp (`KHO\UBND\DCXD\…`), có khi thiếu (`DCXD\…` vì họ trỏ thẳng vào thư mục con). Bản desktop
//! xử lý bằng cách dò đĩa; ở đây không có đĩa nên phải suy từ chính hai danh sách.
use crate::check::{base_of, gom, norm_rel, CheckReport, Sink};
use crate::excel::Sheet;
use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;

/// Một tệp trong bản kê cổng gửi lên: đường dẫn tương đối + cỡ byte. Không có nội dung.
#[derive(Debug, Clone)]
pub struct TepKe {
    pub duong_dan: String,
    pub co: u64,
}

/// Việc căn lề đã làm với đường dẫn của bản kê, để nói lại cho người dùng.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanLe {
    /// Tiền tố bị CẮT khỏi mọi đường dẫn trong bản kê (rỗng = không cắt gì).
    pub bo: String,
    /// Tiền tố được THÊM vào đầu (rỗng = không thêm gì).
    pub them: String,
    /// Số mục khớp một đường dẫn Excel sau khi căn.
    pub khop: usize,
}

#[derive(Debug)]
pub struct KetQuaKho {
    pub bao_cao: CheckReport,
    /// `None` = bản kê vốn đã khớp Excel, không phải chỉnh gì.
    pub can_le: Option<CanLe>,
    /// Mục không phải `.pdf` — bỏ qua, y như bản desktop chỉ quét PDF.
    pub bo_qua_khong_pdf: usize,
    /// Mục nằm ngoài các thư mục cấp đầu mà Excel nhắc tới — bỏ qua, cùng lý do.
    pub ngoai_pham_vi: usize,
    /// Số mục còn lại thực sự đem đi đối chiếu.
    pub so_tep: usize,
}

fn la_pdf(rel: &str) -> bool {
    rel.rsplit_once('.').is_some_and(|(_, e)| e.eq_ignore_ascii_case("pdf"))
}

fn doan(rel: &str) -> Vec<&str> {
    rel.split('\\').filter(|s| !s.is_empty()).collect()
}

/// Số đoạn trùng tính từ CUỐI hai đường dẫn (không phân biệt hoa thường).
fn duoi_chung(a: &[&str], b: &[&str]) -> usize {
    let mut k = 0;
    while k < a.len() && k < b.len() {
        let (x, y) = (a[a.len() - 1 - k], b[b.len() - 1 - k]);
        if x.to_lowercase() != y.to_lowercase() {
            break;
        }
        k += 1;
    }
    k
}

/// Suy cặp (cắt, thêm) đưa đường dẫn bản kê về đúng hệ quy chiếu của Excel.
///
/// Cách làm: với mỗi đường dẫn Excel, tìm mục bản kê **trùng tên tệp**, đo phần đuôi chung của
/// hai đường dẫn rồi suy ra hai đầu thừa/thiếu. Cặp nào được nhiều đường dẫn Excel bầu nhất thì
/// thắng — một vài tên tệp trùng nhau ngẫu nhiên không lái được kết quả.
fn ung_vien(excel: &[String], ke: &[String]) -> Option<(String, String)> {
    let mut theo_ten: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, m) in ke.iter().enumerate() {
        theo_ten.entry(base_of(m).to_lowercase()).or_default().push(i);
    }
    let mut phieu: HashMap<(String, String), usize> = HashMap::new();
    for e in excel.iter().take(200) {
        let Some(ds) = theo_ten.get(&base_of(e).to_lowercase()) else { continue };
        // Tên tệp trùng ở nhiều chỗ thì mỗi chỗ được bỏ một phiếu; đa số quyết định.
        for &i in ds.iter().take(8) {
            let (ed, md) = (doan(e), doan(&ke[i]));
            let k = duoi_chung(&ed, &md);
            if k == 0 {
                continue;
            }
            let bo = md[..md.len() - k].join("\\");
            let them = ed[..ed.len() - k].join("\\");
            if bo.is_empty() && them.is_empty() {
                continue; // đã khớp sẵn, không phải căn
            }
            *phieu.entry((bo, them)).or_default() += 1;
        }
    }
    // Hoà phiếu thì lấy cặp cắt ÍT nhất — chỉnh tay nhẹ hơn luôn là giả thiết an toàn hơn.
    phieu
        .into_iter()
        .max_by(|a, b| {
            let (bo_a, bo_b) = (a.0 .0.len(), b.0 .0.len());
            a.1.cmp(&b.1).then_with(|| bo_b.cmp(&bo_a))
        })
        .map(|(k, _)| k)
}

fn ap(rel: &str, bo: &str, them: &str) -> Option<String> {
    let con = if bo.is_empty() {
        rel
    } else if rel.len() > bo.len()
        && rel.as_bytes()[bo.len()] == b'\\'
        && rel[..bo.len()].to_lowercase() == bo.to_lowercase()
    {
        &rel[bo.len() + 1..]
    } else {
        return None;
    };
    Some(if them.is_empty() { con.to_string() } else { format!("{them}\\{con}") })
}

/// Chạy bước kiểm «Đóng gói nhanh» trên bản kê thay cho đĩa.
///
/// `ten_excel` chỉ để ghi lại trong báo cáo — KHÔNG được là đường dẫn trên máy chủ, vì báo cáo
/// này đi thẳng ra ngoài cho nhà thầu.
pub fn kiem_ban_ke(sheet: &Sheet, ten_excel: &str, tep: &[TepKe]) -> KetQuaKho {
    let plan = gom(sheet, PathBuf::new());
    let mut sink = Sink::new();

    // ---- phạm vi: đúng các thư mục cấp đầu mà Excel nhắc tới, y như bản đĩa chỉ quét `tops`.
    // Không lọc thì mọi thứ khác trong thư mục người dùng chọn đều thành "thừa tệp" — bản
    // desktop không báo như thế, và một kho lẫn thư mục khác sẽ ngập lỗi giả.
    let tops: BTreeSet<String> = plan
        .dossiers
        .iter()
        .filter_map(|d| d.dir.split('\\').next())
        .filter(|t| !t.is_empty() && !t.starts_with('?'))
        .map(|t| t.to_lowercase())
        .collect();

    let excel: Vec<String> = sheet.rows.iter().filter_map(|r| r.doc_path.as_deref()).map(norm_rel).collect();
    let mut bo_qua_khong_pdf = 0usize;
    let mut ke: Vec<(String, u64)> = Vec::with_capacity(tep.len());
    for t in tep {
        let rel = norm_rel(&t.duong_dan);
        if rel.is_empty() {
            continue;
        }
        if !la_pdf(&rel) {
            bo_qua_khong_pdf += 1;
            continue;
        }
        ke.push((rel, t.co));
    }

    // ---- căn lề: chỉ nhận khi nó làm SỐ MỤC KHỚP TĂNG LÊN. Suy sai mà vẫn áp thì mọi tệp đều
    // hoá "thiếu", tệ hơn hẳn việc để nguyên.
    let trong_excel: BTreeSet<String> = excel.iter().map(|e| e.to_lowercase()).collect();
    let dem = |v: &[(String, u64)]| v.iter().filter(|(r, _)| trong_excel.contains(&r.to_lowercase())).count();
    let khop_thang = dem(&ke);
    let mut can_le = None;
    let chi_duong_dan: Vec<String> = ke.iter().map(|(r, _)| r.clone()).collect();
    if let Some((bo, them)) = ung_vien(&excel, &chi_duong_dan) {
        let thu: Vec<(String, u64)> =
            ke.iter().map(|(r, c)| (ap(r, &bo, &them).unwrap_or_else(|| r.clone()), *c)).collect();
        let khop = dem(&thu);
        if khop > khop_thang {
            sink.info(
                "KHO-CAN-LE",
                "",
                "",
                None,
                None,
                match (bo.is_empty(), them.is_empty()) {
                    (false, true) => format!(
                        "bản kê có thêm lớp thư mục «{bo}» ở đầu — đã bỏ để khớp đường dẫn Excel ({khop} tệp)"
                    ),
                    (true, false) => format!(
                        "bản kê thiếu đoạn đầu «{them}» mà Excel ghi — đã thêm vào để đối chiếu ({khop} tệp)"
                    ),
                    _ => format!("đổi đoạn đầu đường dẫn «{bo}» → «{them}» để khớp Excel ({khop} tệp)"),
                },
            );
            can_le = Some(CanLe { bo, them, khop });
            ke = thu;
        }
    }

    // ---- thư mục cấp đầu Excel nhắc tới mà bản kê không có mục nào
    for t in &tops {
        let co = ke.iter().any(|(r, _)| r.split('\\').next().is_some_and(|x| x.to_lowercase() == *t));
        if !co {
            sink.error(
                "THU-MUC-GOC-KHONG-CO",
                "",
                "",
                None,
                None,
                format!(
                    "bản kê không có tệp nào trong «{t}» — Excel ghi đường dẫn từ «{t}\\…», hãy chọn đúng thư mục CHỨA «{t}»"
                ),
            );
        }
    }

    let truoc = ke.len();
    ke.retain(|(r, _)| r.split('\\').next().is_some_and(|t| tops.contains(&t.to_lowercase())));
    let ngoai_pham_vi = truoc - ke.len();
    let so_tep = ke.len();

    let mut disk: HashMap<String, (String, u64)> = HashMap::with_capacity(so_tep);
    for (rel, co) in ke {
        disk.insert(rel.to_lowercase(), (rel, co));
    }

    let bao_cao = crate::check::chay(sheet, ten_excel, &plan, disk, sink, false, false, false, 1, &|_, _| {});
    KetQuaKho { bao_cao, can_le, bo_qua_khong_pdf, ngoai_pham_vi, so_tep }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ds(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn bo_lop_thu_muc_trinh_duyet_gan_them() {
        // Excel ghi từ «UBND», trình duyệt kê từ thư mục người dùng chọn là «KHO».
        let e = ds(&[r"UBND\DCXD\HS 153\a.pdf", r"UBND\DCXD\HS 153\b.pdf"]);
        let m = ds(&[r"KHO\UBND\DCXD\HS 153\a.pdf", r"KHO\UBND\DCXD\HS 153\b.pdf"]);
        assert_eq!(ung_vien(&e, &m), Some(("KHO".into(), String::new())));
        assert_eq!(ap(&m[0], "KHO", ""), Some(r"UBND\DCXD\HS 153\a.pdf".into()));
    }

    #[test]
    fn them_doan_dau_khi_nguoi_dung_tro_vao_thu_muc_con() {
        let e = ds(&[r"UBND\DCXD\HS 153\a.pdf"]);
        let m = ds(&[r"DCXD\HS 153\a.pdf"]);
        assert_eq!(ung_vien(&e, &m), Some((String::new(), "UBND".into())));
        assert_eq!(ap(&m[0], "", "UBND"), Some(r"UBND\DCXD\HS 153\a.pdf".into()));
    }

    #[test]
    fn khop_san_thi_khong_can_le() {
        let e = ds(&[r"UBND\HS 1\a.pdf", r"UBND\HS 1\b.pdf"]);
        assert_eq!(ung_vien(&e, &e), None);
    }

    #[test]
    fn da_so_quyet_dinh_khi_mot_ten_tep_trung_ngau_nhien() {
        // «bia.pdf» còn nằm ở một chỗ lạc; hai tệp kia mới là căn cứ thật.
        let e = ds(&[r"UBND\HS 1\bia.pdf", r"UBND\HS 1\x.pdf", r"UBND\HS 2\y.pdf"]);
        let m = ds(&[
            r"KHO\LAC\bia.pdf",
            r"KHO\UBND\HS 1\x.pdf",
            r"KHO\UBND\HS 2\y.pdf",
            r"KHO\UBND\HS 1\bia.pdf",
        ]);
        assert_eq!(ung_vien(&e, &m), Some(("KHO".into(), String::new())));
    }

    #[test]
    fn cat_tien_to_khong_phan_biet_hoa_thuong_va_khong_cat_nham_tien_to_dai() {
        assert_eq!(ap(r"kho\a.pdf", "KHO", ""), Some("a.pdf".into()));
        // «KHO2» KHÔNG được coi là nằm trong «KHO».
        assert_eq!(ap(r"KHO2\a.pdf", "KHO", ""), None);
        // Đúng bằng tiền tố, không còn phần con → không cắt được.
        assert_eq!(ap("KHO", "KHO", ""), None);
    }

    #[test]
    fn chi_nhan_pdf() {
        assert!(la_pdf("a.PDF"));
        assert!(la_pdf(r"x\y.pdf"));
        assert!(!la_pdf("a.pdfx"));
        assert!(!la_pdf("Thumbs.db"));
        assert!(!la_pdf("khongduoi"));
    }

    #[test]
    fn duoi_chung_dem_theo_doan_chu_khong_theo_ky_tu() {
        assert_eq!(duoi_chung(&doan(r"a\HS 1\x.pdf"), &doan(r"b\c\HS 1\x.pdf")), 2);
        // «HS 11» khác «HS 1» — không được tính là chung dù trùng tiền tố chuỗi.
        assert_eq!(duoi_chung(&doan(r"a\HS 1\x.pdf"), &doan(r"a\HS 11\x.pdf")), 1);
    }

    fn o(v: &str) -> Option<String> {
        Some(v.to_string())
    }

    /// Một kho nhỏ nhưng có đủ loại lệch: thiếu tệp, thừa tệp, STT nhảy, trùng mã.
    fn to_du_lieu() -> (crate::excel::Sheet, Vec<(String, u64)>) {
        use crate::excel::Row;
        let mut rows = Vec::new();
        let mut them = |line: u32, dir: &str, hs: &str, stt: &str, trang: &str| {
            let tep = format!("{hs}.{stt}.pdf");
            rows.push(Row {
                line,
                fond_code: o("H49.101"),
                hs_code: o(hs),
                hs_title: o("Hồ sơ thử"),
                hs_thbq: o("Vĩnh viễn"),
                hs_papers: o("10"),
                doc_file_code: o(&tep),
                doc_type: o("Quyết định"),
                doc_subject: o("Trích yếu thử"),
                doc_date: o("09/07/2012"),
                doc_pages: o(trang),
                doc_path: o(&format!("{dir}\\{tep}")),
                ..Row::default()
            });
        };
        them(2, r"UBND\HOP01\HS 001", "H49.101.01.1993.001", "0000001", "3");
        them(3, r"UBND\HOP01\HS 001", "H49.101.01.1993.001", "0000002", "4");
        // STT nhảy từ 2 sang 4 → cảnh báo TL-STT-KHONG-LIEN-TUC
        them(4, r"UBND\HOP01\HS 001", "H49.101.01.1993.001", "0000004", "2");
        them(5, r"UBND\HOP01\HS 002", "H49.101.01.1993.002", "0000001", "5");
        // Excel khai nhưng đĩa không có → TEP-KHONG-CO-TREN-DIA
        them(6, r"UBND\HOP01\HS 002", "H49.101.01.1993.002", "0000002", "1");

        let sheet = crate::excel::Sheet {
            sheet_name: "Sheet1".into(),
            header_line: 1,
            rows,
            missing_columns: Vec::new(),
            mapping: Vec::new(),
        };
        // Trên đĩa: thiếu tệp của dòng 6, lại thừa một tệp Excel không khai.
        let dia = vec![
            (r"UBND\HOP01\HS 001\H49.101.01.1993.001.0000001.pdf".to_string(), 1111u64),
            (r"UBND\HOP01\HS 001\H49.101.01.1993.001.0000002.pdf".to_string(), 2222),
            (r"UBND\HOP01\HS 001\H49.101.01.1993.001.0000004.pdf".to_string(), 3333),
            (r"UBND\HOP01\HS 002\H49.101.01.1993.002.0000001.pdf".to_string(), 4444),
            (r"UBND\HOP01\HS 002\H49.101.01.1993.002.0000009.pdf".to_string(), 5555),
        ];
        (sheet, dia)
    }

    fn dau_van_de(bc: &CheckReport) -> Vec<(String, String, u32)> {
        let mut v: Vec<(String, String, u32)> = bc
            .issues
            .iter()
            .filter(|i| i.code != "KHO-CAN-LE")
            .map(|i| (i.code.to_string(), i.dossier.clone(), i.count))
            .collect();
        v.sort();
        v
    }

    /// Cái phải chứng minh: cùng dữ liệu, bản kê cho KẾT QUẢ Y HỆT lần quét đĩa thật.
    /// Không có bài này thì "cửa máy chủ chạy đúng bước kiểm của desktop" chỉ là lời nói.
    #[test]
    fn ban_ke_cho_ket_qua_y_het_quet_dia_that() {
        let (sheet, dia) = to_du_lieu();
        let goc = std::env::temp_dir().join(format!("dgt-kho-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&goc);
        for (rel, co) in &dia {
            let p = crate::check::rel_to_path(&goc, rel);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(&p, vec![b'x'; *co as usize]).unwrap();
        }
        let plan = crate::check::plan(&sheet, &goc);
        let tren_dia = crate::check::run(
            &sheet,
            std::path::Path::new("Metadata.xlsx"),
            &plan,
            false,
            false,
            false,
            1,
            &|_, _| {},
        );

        // Bản kê đúng như ô chọn thư mục của trình duyệt trả: có lớp bọc tên thư mục đã chọn,
        // lẫn cả tệp rác không phải PDF.
        let mut tep: Vec<TepKe> = dia
            .iter()
            .map(|(rel, co)| TepKe { duong_dan: format!("KHO 2026/{}", rel.replace('\\', "/")), co: *co })
            .collect();
        tep.push(TepKe { duong_dan: "KHO 2026/Thumbs.db".into(), co: 7 });
        let kq = kiem_ban_ke(&sheet, "Metadata.xlsx", &tep);

        // khop = 4 chứ không phải 5: tệp ...0000009.pdf có trong kho nhưng Excel không khai,
        // nên nó không tính là "khớp" — và đó chính là lỗi thừa tệp kiểm ở dưới.
        assert_eq!(kq.can_le, Some(CanLe { bo: "KHO 2026".into(), them: String::new(), khop: 4 }));
        assert_eq!(kq.bo_qua_khong_pdf, 1, "tệp không phải PDF phải bị bỏ, không hoá lỗi thừa tệp");
        assert_eq!(kq.so_tep, 5);
        assert_eq!(dau_van_de(&kq.bao_cao), dau_van_de(&tren_dia), "bản kê lệch kết quả quét đĩa");
        assert_eq!(kq.bao_cao.errors(), tren_dia.errors());
        assert_eq!(kq.bao_cao.warnings(), tren_dia.warnings());
        assert_eq!(kq.bao_cao.files_on_disk, tren_dia.files_on_disk);
        assert_eq!(kq.bao_cao.dossiers_ok(), tren_dia.dossiers_ok());

        // Và đúng những lỗi mong đợi, kẻo hai bên cùng trả rỗng mà vẫn "khớp".
        let ma: BTreeSet<&str> = kq.bao_cao.issues.iter().map(|i| i.code).collect();
        assert!(ma.contains("TEP-KHONG-CO-TREN-DIA"), "phải bắt tệp Excel khai mà kho không có: {ma:?}");
        assert!(ma.contains("TEP-KHONG-CO-TRONG-EXCEL"), "phải bắt tệp thừa trong kho: {ma:?}");
        assert!(kq.bao_cao.errors() >= 2);

        let _ = std::fs::remove_dir_all(&goc);
    }

    /// Không suy được cách căn thì phải ĐỂ NGUYÊN, và nói rõ chọn nhầm thư mục.
    #[test]
    fn chon_nham_han_mot_thu_muc_khac_thi_bao_chon_sai_chu_khong_can_bua() {
        let (sheet, _) = to_du_lieu();
        let tep = vec![TepKe { duong_dan: "ANH/khong-lien-quan.pdf".into(), co: 9 }];
        let kq = kiem_ban_ke(&sheet, "Metadata.xlsx", &tep);
        assert_eq!(kq.can_le, None, "không có căn cứ thì không được tự ý căn lề");
        assert_eq!(kq.so_tep, 0, "tệp ngoài phạm vi Excel không được đem đối chiếu");
        assert_eq!(kq.ngoai_pham_vi, 1);
        assert!(kq.bao_cao.issues.iter().any(|i| i.code == "THU-MUC-GOC-KHONG-CO"));
    }

    /// Bản kê rỗng: mọi dòng Excel thành "thiếu tệp", không được hoảng hay trả rỗng êm ru.
    #[test]
    fn ban_ke_rong_thi_moi_dong_excel_deu_thieu_tep() {
        let (sheet, _) = to_du_lieu();
        let kq = kiem_ban_ke(&sheet, "Metadata.xlsx", &[]);
        assert_eq!(kq.so_tep, 0);
        assert!(kq.bao_cao.errors() >= 5, "5 dòng Excel phải ra ít nhất 5 lỗi thiếu tệp");
    }
}
