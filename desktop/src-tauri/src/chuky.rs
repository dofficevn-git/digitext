// Copyright (c) 2026 DigiText. All rights reserved.
//! Tiện ích › Kiểm chữ ký số: quét thư mục trên ổ cứng, xác minh **mọi** chữ ký trong từng
//! tệp PDF, rồi trả bảng cho giao diện xuất Excel.
//!
//! Dùng `digitext_sign_verify::kiem_tat_ca`, **không** dùng `verify`: `verify` chỉ xác minh
//! một chữ ký và đòi nó phủ trọn tệp, nên với văn bản ký nhiều lần (VGCA SignTool: phê duyệt →
//! số công văn → ngày → dấu tổ chức) nó kết luận sai. Xem ghi chú đầu `sign-verify/nhieu.rs`.
//!
//! Việc đi thư mục nằm ở đây chứ không nằm trong crate là có chủ ý: crate giữ phần thuần
//! (kiểm và kết luận, test được), lớp này lo đĩa và báo tiến độ.
use crate::auth::AuthState;
use crate::error::AppError;
use digitext_core::Permission;
use digitext_sign_verify::{ket_luan_tep, kiem_tat_ca, ChuKyPdf};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, State};

static RUNNING: AtomicBool = AtomicBool::new(false);
static CANCEL: AtomicBool = AtomicBool::new(false);

/// Trần cỡ một tệp đọc vào bộ nhớ. Bản quét đọc trọn từng tệp để băm, nên tệp quá lớn thì bỏ
/// và nói rõ, chứ không để ứng dụng chết vì hết bộ nhớ.
const CO_TEP_TOI_DA: u64 = 512 * 1024 * 1024;
/// Trần số tệp một lượt quét, để một cú chọn nhầm ổ C: không treo ứng dụng hàng giờ.
const SO_TEP_TOI_DA: usize = 50_000;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub done: usize,
    pub total: usize,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChuKyRa {
    pub thu_tu: usize,
    /// Nhãn gọi tên chữ ký trong báo cáo, vd `Chữ ký #2 (Dấu tổ chức)`.
    pub nhan: String,
    /// `Sig` hoặc `DocTimeStamp`.
    pub loai: String,
    pub khuon_dang: Option<String>,
    pub ten_khai: Option<String>,
    pub ly_do: Option<String>,
    pub noi: Option<String>,
    /// CN của chứng thư — người/tổ chức ký thật, khác `tenKhai` do công cụ ký ghi.
    pub nguoi_ky: Option<String>,
    /// O= trong chủ thể chứng thư.
    pub to_chuc: Option<String>,
    pub nha_cap: Option<String>,
    pub hieu_luc_tu: Option<String>,
    pub hieu_luc_den: Option<String>,
    /// `/M` — thời điểm máy người ký khai. Không chứng minh được.
    pub thoi_diem_khai: Option<String>,
    /// Thời điểm theo dấu thời gian — thứ chứng minh được.
    pub thoi_diem_tsa: Option<String>,
    pub co_tsa: bool,
    pub gan_chung_thu: bool,
    pub toan_ven: bool,
    pub phu_tron_tep: bool,
    pub dat: bool,
    pub loi: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TepDaKy {
    pub duong_dan: String,
    pub ten: String,
    pub thu_muc: String,
    pub co: u64,
    /// `"Đạt"` · `"Đạt, có cảnh báo"` · `"Không đạt"` · `"Chưa ký"` · `"Bỏ qua"`.
    pub ket_luan: String,
    pub so_chu_ky: usize,
    pub byte_thua_cuoi: usize,
    /// Có dữ liệu xác minh dài hạn (LTV/DSS) nhúng kèm — mức PAdES-LT.
    pub co_dss: bool,
    /// Có dấu thời gian cho cả tài liệu — mức PAdES-LTA, mức cao nhất.
    pub co_dau_thoi_gian_tai_lieu: bool,
    pub loi: Vec<String>,
    pub canh_bao: Vec<String>,
    pub chu_ky: Vec<ChuKyRa>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KetQuaQuet {
    pub tep: Vec<TepDaKy>,
    /// Số tệp PDF tìm thấy.
    pub so_pdf: usize,
    /// Số mục không phải `.pdf` — đã bỏ. Hiện ra để người dùng khỏi hoang mang vì
    /// "tôi chọn 5000 tệp mà nó chỉ kiểm 300".
    pub bo_qua_khong_pdf: usize,
    /// Số tệp vượt trần cỡ, không đọc.
    pub bo_qua_qua_lon: usize,
    /// Quét bị cắt vì vượt trần số tệp.
    pub bi_cat: bool,
    pub so_dat: usize,
    pub so_canh_bao: usize,
    pub so_khong_dat: usize,
    pub so_chua_ky: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuetArgs {
    pub paths: Vec<String>,
}

fn gio_vn(unix: i64) -> String {
    digitext_sign_verify::thoi_gian_ky(unix.max(0) as u64)
}

/// Lấy giá trị một thuộc tính trong chủ thể RFC 4514, vd `O=` trong
/// `CN=Nguyen Van A,O=UBND tinh,C=VN`.
fn thuoc_tinh(subject: &str, khoa: &str) -> Option<String> {
    let tien_to = format!("{khoa}=");
    subject
        .split(',')
        .map(str::trim)
        .find_map(|p| p.strip_prefix(&tien_to))
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn ra(c: &ChuKyPdf) -> ChuKyRa {
    let ng = c.nguoi_ky.as_ref();
    ChuKyRa {
        thu_tu: c.thu_tu,
        nhan: c.nhan(),
        loai: c.loai.clone(),
        khuon_dang: c.khuon_dang.clone(),
        ten_khai: c.ten_khai.clone(),
        ly_do: c.ly_do.clone(),
        noi: c.noi.clone(),
        nguoi_ky: ng.map(|n| n.signer.clone()).filter(|s| !s.is_empty()),
        to_chuc: ng.and_then(|n| thuoc_tinh(&n.subject, "O")),
        nha_cap: ng.map(|n| n.issuer.clone()).filter(|s| !s.is_empty()),
        hieu_luc_tu: ng.map(|n| gio_vn(n.not_before_unix)),
        hieu_luc_den: ng.map(|n| gio_vn(n.not_after_unix)),
        thoi_diem_khai: c.thoi_diem_khai.clone(),
        thoi_diem_tsa: c.thoi_diem_tsa_unix.map(gio_vn),
        co_tsa: c.co_tsa,
        gan_chung_thu: c.gan_chung_thu,
        toan_ven: c.toan_ven,
        phu_tron_tep: c.phu_tron_tep,
        dat: c.dat(),
        loi: c.loi.clone(),
    }
}

/// Gom mọi `.pdf` dưới các đường dẫn đã chọn. Đường dẫn là tệp thì nhận chính nó.
fn gom(paths: &[String]) -> (Vec<PathBuf>, usize, bool) {
    let mut ra = Vec::new();
    let mut bo_qua = 0usize;
    let mut bi_cat = false;
    fn la_pdf(p: &Path) -> bool {
        p.extension().is_some_and(|x| x.eq_ignore_ascii_case("pdf"))
    }
    fn di(d: &Path, ra: &mut Vec<PathBuf>, bo_qua: &mut usize, bi_cat: &mut bool) {
        if *bi_cat {
            return;
        }
        let Ok(rd) = std::fs::read_dir(d) else { return };
        // Xếp theo tên để hai lần quét cùng một thư mục cho cùng một thứ tự dòng trong báo cáo.
        let mut muc: Vec<PathBuf> = rd.flatten().map(|e| e.path()).collect();
        muc.sort();
        for p in muc {
            if *bi_cat {
                return;
            }
            if p.is_dir() {
                di(&p, ra, bo_qua, bi_cat);
            } else if la_pdf(&p) {
                if ra.len() >= SO_TEP_TOI_DA {
                    *bi_cat = true;
                    return;
                }
                ra.push(p);
            } else {
                *bo_qua += 1;
            }
        }
    }
    for s in paths {
        let p = PathBuf::from(s);
        if p.is_dir() {
            di(&p, &mut ra, &mut bo_qua, &mut bi_cat);
        } else if la_pdf(&p) {
            ra.push(p);
        } else {
            bo_qua += 1;
        }
    }
    ra.sort();
    ra.dedup();
    (ra, bo_qua, bi_cat)
}

/// Quét và xác minh. Mỗi tệp đọc trọn vào bộ nhớ rồi thả ra ngay — không giữ cả thư mục.
#[tauri::command]
pub async fn chuky_quet(app: AppHandle, auth: State<'_, AuthState>, args: QuetArgs) -> Result<KetQuaQuet, AppError> {
    auth.require_perm(Permission::SignView)?;
    if RUNNING.swap(true, Ordering::SeqCst) {
        return Err(AppError::validation("đang có một đợt kiểm chữ ký chạy"));
    }
    CANCEL.store(false, Ordering::SeqCst);
    let res = tauri::async_runtime::spawn_blocking(move || {
        let (ds, bo_qua_khong_pdf, bi_cat) = gom(&args.paths);
        let total = ds.len();
        // Một mốc thời gian cho CẢ đợt: hai tệp trong cùng báo cáo không được xét hiệu lực
        // chứng thư ở hai thời điểm khác nhau.
        let bay_gio = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let mut tep = Vec::with_capacity(total);
        let mut bo_qua_qua_lon = 0usize;
        for (i, p) in ds.iter().enumerate() {
            if CANCEL.load(Ordering::SeqCst) {
                break;
            }
            let ten = p.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
            let _ = app.emit("chuky://progress", Progress { done: i, total, name: ten.clone() });
            let thu_muc = p.parent().map(|d| d.to_string_lossy().into_owned()).unwrap_or_default();
            let co = std::fs::metadata(p).map(|m| m.len()).unwrap_or(0);
            let mut row = TepDaKy {
                duong_dan: p.to_string_lossy().into_owned(),
                ten: ten.clone(),
                thu_muc,
                co,
                ket_luan: String::new(),
                so_chu_ky: 0,
                byte_thua_cuoi: 0,
                co_dss: false,
                co_dau_thoi_gian_tai_lieu: false,
                loi: Vec::new(),
                canh_bao: Vec::new(),
                chu_ky: Vec::new(),
            };
            if co > CO_TEP_TOI_DA {
                bo_qua_qua_lon += 1;
                row.ket_luan = "Bỏ qua".into();
                row.loi.push(format!("tệp {} MiB, vượt trần {} MiB của một lượt kiểm", co / 1048576, CO_TEP_TOI_DA / 1048576));
                tep.push(row);
                continue;
            }
            match std::fs::read(p) {
                Ok(b) => {
                    let kq = kiem_tat_ca(&b);
                    let k = ket_luan_tep(&kq, bay_gio);
                    row.so_chu_ky = kq.so_chu_ky();
                    row.byte_thua_cuoi = kq.byte_thua_cuoi;
                    row.co_dss = kq.co_dss;
                    row.co_dau_thoi_gian_tai_lieu = kq.co_dau_thoi_gian_tai_lieu;
                    row.chu_ky = kq.chu_ky.iter().map(ra).collect();
                    row.ket_luan = k.ket_luan;
                    row.loi = k.loi;
                    row.canh_bao = k.canh_bao;
                }
                Err(e) => {
                    row.ket_luan = "Bỏ qua".into();
                    row.loi.push(format!("không đọc được tệp: {e}"));
                }
            }
            tep.push(row);
            let _ = app.emit("chuky://progress", Progress { done: i + 1, total, name: ten });
        }
        let so_dat = tep.iter().filter(|t| t.ket_luan == "Đạt").count();
        let so_canh_bao = tep.iter().filter(|t| t.ket_luan == "Đạt, có cảnh báo").count();
        let so_khong_dat = tep.iter().filter(|t| t.ket_luan == "Không đạt").count();
        let so_chua_ky = tep.iter().filter(|t| t.ket_luan == "Chưa ký").count();
        tracing::info!(
            so_pdf = tep.len(),
            so_dat,
            so_canh_bao,
            so_khong_dat,
            so_chua_ky,
            "kiểm chữ ký số trên ổ cứng"
        );
        KetQuaQuet {
            so_pdf: tep.len(),
            tep,
            bo_qua_khong_pdf,
            bo_qua_qua_lon,
            bi_cat,
            so_dat,
            so_canh_bao,
            so_khong_dat,
            so_chua_ky,
        }
    })
    .await
    .map_err(|e| AppError::internal(format!("kiểm chữ ký lỗi: {e}")));
    RUNNING.store(false, Ordering::SeqCst);
    let res = res?;
    if res.tep.is_empty() {
        return Err(AppError::validation("không tìm thấy tệp PDF nào trong thư mục đã chọn"));
    }
    Ok(res)
}

#[tauri::command]
pub async fn chuky_huy(auth: State<'_, AuthState>) -> Result<(), AppError> {
    auth.require_perm(Permission::SignView)?;
    CANCEL.store(true, Ordering::SeqCst);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{gom, thuoc_tinh};

    #[test]
    fn boc_duoc_to_chuc_trong_chu_the() {
        let s = "CN=Nguyen Van A,O=UBND tinh Quang Ninh,OU=Van thu,C=VN";
        assert_eq!(thuoc_tinh(s, "O").as_deref(), Some("UBND tinh Quang Ninh"));
        assert_eq!(thuoc_tinh(s, "CN").as_deref(), Some("Nguyen Van A"));
        assert_eq!(thuoc_tinh(s, "L"), None);
        // Không được nhầm `OU=` thành `O=`: cùng tiền tố chữ O.
        assert_eq!(thuoc_tinh("OU=Van thu,C=VN", "O"), None);
        // Chủ thể rỗng hoặc thiếu khoá thì trả None, không trả chuỗi rỗng.
        assert_eq!(thuoc_tinh("", "O"), None);
        assert_eq!(thuoc_tinh("O=,C=VN", "O"), None);
    }

    /// Đi thư mục: lấy `.pdf` ở mọi cấp, đếm mục không phải PDF, thứ tự ổn định.
    ///
    /// Bài này canh ba lỗi dễ mắc mà biên dịch không bắt được: quét không xuống thư mục con;
    /// đếm thư mục vào số "bỏ qua vì không phải PDF" (rồi báo cáo hiện con số vô nghĩa); và
    /// thứ tự dòng đổi mỗi lần quét vì `read_dir` không hứa thứ tự.
    #[test]
    fn gom_di_het_thu_muc_con_va_dem_dung_phan_bo_qua() {
        let goc = std::env::temp_dir().join(format!("digitext-chuky-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&goc);
        let sau = goc.join("hop01").join("hoso001");
        std::fs::create_dir_all(&sau).expect("dựng thư mục thử");
        for (d, f) in [
            (goc.as_path(), "b.pdf"),
            (goc.as_path(), "a.PDF"),
            (goc.as_path(), "Thumbs.db"),
            (sau.as_path(), "c.pdf"),
            (sau.as_path(), "ghichu.txt"),
        ] {
            std::fs::write(d.join(f), b"x").expect("ghi tệp thử");
        }
        let (ds, bo_qua, bi_cat) = gom(&[goc.to_string_lossy().into_owned()]);

        assert_eq!(ds.len(), 3, "phải lấy cả tệp ở thư mục con, và nhận cả đuôi .PDF hoa: {ds:?}");
        assert_eq!(bo_qua, 2, "chỉ Thumbs.db và ghichu.txt được tính, THƯ MỤC thì không");
        assert!(!bi_cat);
        // Thứ tự ổn định: hai lần quét cùng thư mục cho cùng thứ tự dòng trong báo cáo.
        let (lan_hai, _, _) = gom(&[goc.to_string_lossy().into_owned()]);
        assert_eq!(ds, lan_hai);

        // Chọn thẳng một tệp: nhận chính nó, không coi là thư mục.
        let (mot, bq, _) = gom(&[goc.join("b.pdf").to_string_lossy().into_owned()]);
        assert_eq!(mot.len(), 1);
        assert_eq!(bq, 0);
        // Chọn thẳng tệp KHÔNG phải PDF: không nhận, và được đếm.
        let (khong, bq, _) = gom(&[goc.join("Thumbs.db").to_string_lossy().into_owned()]);
        assert!(khong.is_empty());
        assert_eq!(bq, 1);
        // Chọn trùng (thư mục + tệp bên trong) không được ra hai dòng cho một tệp.
        let (trung, _, _) = gom(&[goc.to_string_lossy().into_owned(), goc.join("b.pdf").to_string_lossy().into_owned()]);
        assert_eq!(trung.len(), 3, "trùng đường dẫn phải gộp lại: {trung:?}");

        let _ = std::fs::remove_dir_all(&goc);
    }

}
