// Copyright (c) 2026 DigiText. All rights reserved.
//! BÁO CÁO THỐNG KÊ TT 05/2025/TT-BNV Điều 44.3b (27/11/2026, đợt 3 đối chiếu Digifile TT05 — nhóm 3A). Số liệu ở
//! `digitext_db::bao_cao`; ở đây là DANH MỤC báo cáo (mã, tên, căn cứ, nhóm, bản áp dụng, bộ lọc, cột) mà desktop dựng màn từ đó,
//! và ba dạng xuất: JSON (xem trên màn), Excel (`xlsx::sheet_bang`), PDF (`digitext_pdf::manual`, A4 ngang khi nhiều cột, đầu văn
//! bản hành chính + khối ký). Không lưu bản xuất vào sổ biên bản (anh chưa chốt câu 15 — xuất là tệp tải về; cần lưu thì làm sau).
//! - `GET /bao-cao` — danh mục (lọc theo bản: thi công / lưu trữ / cả hai); 20 báo cáo kể cả sổ chuyển đổi (0037)
//! - `GET /bao-cao/{ma}?projectId&phongId&namTu&namDen&from&to&format=json|xlsx|pdf`
//! Quyền: `stats.view` hoặc `stats.view_project` + thuộc dự án. Báo cáo sao lưu là toàn máy chủ (vẫn kiểm theo dự án đang chọn).
use super::chinh_ly::{pham_vi_phong, phong_hieu_dung, project_access};
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::Ts;
use crate::xlsx::{self, CotBang};
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use digitext_core::Permission;
use digitext_db::bao_cao::{self as db, BoLoc};
use digitext_pdf::manual::ManualBlock;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new().route("/bao-cao", get(danh_muc)).route("/bao-cao/{ma}", get(chay))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kieu {
    Chu,
    So,
    Ma,
    Ngay,
    Bool,
}
impl Kieu {
    fn as_str(self) -> &'static str {
        match self {
            Kieu::Chu => "chu",
            Kieu::So => "so",
            Kieu::Ma => "ma",
            Kieu::Ngay => "ngay",
            Kieu::Bool => "bool",
        }
    }
}
struct Cot {
    k: &'static str,
    ten: &'static str,
    kieu: Kieu,
    rong: f64,
}
const fn chu(k: &'static str, ten: &'static str, rong: f64) -> Cot {
    Cot { k, ten, kieu: Kieu::Chu, rong }
}
const fn so(k: &'static str, ten: &'static str) -> Cot {
    Cot { k, ten, kieu: Kieu::So, rong: 11.0 }
}
const fn ma(k: &'static str, ten: &'static str, rong: f64) -> Cot {
    Cot { k, ten, kieu: Kieu::Ma, rong }
}
const fn ngay(k: &'static str, ten: &'static str) -> Cot {
    Cot { k, ten, kieu: Kieu::Ngay, rong: 12.0 }
}
const fn bo(k: &'static str, ten: &'static str) -> Cot {
    Cot { k, ten, kieu: Kieu::Bool, rong: 9.0 }
}

/// Bản áp dụng: `ca_hai` | `thi_cong` | `luu_tru`. Bộ lọc: `phong` (phông), `nam` (năm kết thúc hồ sơ / năm đợt), `thoi_gian` (từ – đến).
struct Def {
    ma: &'static str,
    ten: &'static str,
    can_cu: &'static str,
    nhom: &'static str,
    ban: &'static str,
    bo_loc: &'static [&'static str],
    cot: &'static [Cot],
    /// Có dòng tổng (cộng các cột số).
    tong: bool,
}

const NHOM: [(&str, &str); 5] = [
    ("muc_luc", "Mục lục hồ sơ"),
    ("tai_lieu", "Tài liệu số và gói tin"),
    ("bao_quan", "Bảo quản, tiêu huỷ"),
    ("khai_thac", "Khai thác, mượn trả"),
    ("kho", "Kho lưu trữ"),
];

const DEFS: &[Def] = &[
    Def {
        ma: "phong", ten: "Tổng hợp theo phông", can_cu: "Điều 44.3b TT 05/2025/TT-BNV", nhom: "muc_luc", ban: "ca_hai", bo_loc: &["phong", "nam"], tong: true,
        cot: &[ma("ma_du_an", "Mã phông", 12.0), chu("ten_phong", "Tên phông", 40.0), so("so_phong_ban", "Phòng ban"), so("so_hop", "Hộp"), so("so_ho_so", "Hồ sơ"), so("hs_vv", "Vĩnh viễn"), so("hs_co_thoi_han", "Có thời hạn"), so("hs_het", "Hết THBQ"), so("so_van_ban", "Văn bản"), so("so_to", "Tờ"), so("so_trang", "Trang"), so("hs_tu_goi", "Từ gói AIP"), so("hs_da_nop_luu", "Đã nộp lưu")],
    },
    Def {
        ma: "ho_so_nam", ten: "Hồ sơ theo năm", can_cu: "Điều 44.3b TT 05/2025/TT-BNV", nhom: "muc_luc", ban: "ca_hai", bo_loc: &["phong", "nam"], tong: true,
        cot: &[ma("nam", "Năm", 10.0), so("so_ho_so", "Hồ sơ"), so("hs_vv", "Vĩnh viễn"), so("hs_co_thoi_han", "Có thời hạn"), so("hs_het", "Hết THBQ"), so("so_hop", "Hộp"), so("so_van_ban", "Văn bản"), so("so_trang", "Trang")],
    },
    Def {
        ma: "phong_ban", ten: "Theo đơn vị hình thành (phòng ban)", can_cu: "Điều 44.3b TT 05/2025/TT-BNV", nhom: "muc_luc", ban: "ca_hai", bo_loc: &["phong"], tong: true,
        cot: &[chu("ten_phong", "Phông", 28.0), chu("ten_phong_ban", "Phòng ban", 30.0), so("hop_vv", "Hộp VV"), so("hs_vv", "Hồ sơ VV"), so("hop_co_thoi_han", "Hộp CTH"), so("hs_co_thoi_han", "Hồ sơ CTH"), so("bo", "Bó loại"), so("tap", "Tập loại"), so("hs_het", "Hết THBQ"), so("hs_trung_thua", "Trùng thừa"), so("hs_chua_danh_so", "Chưa đánh số"), so("hs_tong", "Tổng hồ sơ"), so("vb_tong", "Văn bản")],
    },
    Def {
        ma: "thbq", ten: "Theo thời hạn bảo quản", can_cu: "Điều 44.3b; Điều 15 Luật Lưu trữ 2024", nhom: "muc_luc", ban: "ca_hai", bo_loc: &["phong", "nam"], tong: true,
        cot: &[chu("ten_thbq", "Thời hạn bảo quản", 26.0), ma("ma_tt05", "Mã TT05", 9.0), chu("loai", "Loại", 16.0), so("so_ho_so", "Hồ sơ"), so("so_van_ban", "Văn bản"), so("so_trang", "Trang"), so("hs_het", "Đã hết hạn"), so("hs_het_5_nam", "Hết trong 5 năm")],
    },
    Def {
        ma: "che_do_su_dung", ten: "Theo chế độ sử dụng", can_cu: "Điều 44.3b TT 05/2025/TT-BNV", nhom: "muc_luc", ban: "ca_hai", bo_loc: &["phong", "nam"], tong: true,
        cot: &[chu("gia_tri", "Chế độ sử dụng", 30.0), so("so_ho_so", "Hồ sơ"), so("so_van_ban", "Văn bản"), so("so_trang", "Trang")],
    },
    Def {
        ma: "do_tin_cay", ten: "Theo mức độ tin cậy", can_cu: "Điều 44.3b TT 05/2025/TT-BNV", nhom: "muc_luc", ban: "ca_hai", bo_loc: &["phong", "nam"], tong: true,
        cot: &[chu("gia_tri", "Mức độ tin cậy", 30.0), so("so_ho_so", "Hồ sơ"), so("so_van_ban", "Văn bản"), so("so_trang", "Trang")],
    },
    Def {
        ma: "loai_van_ban", ten: "Theo loại văn bản", can_cu: "Điều 44.3b; Phụ lục I TT 05/2025/TT-BNV", nhom: "muc_luc", ban: "ca_hai", bo_loc: &["phong", "nam"], tong: true,
        cot: &[ma("ma", "Mã", 8.0), chu("ten_loai", "Loại văn bản", 34.0), so("so_van_ban", "Văn bản"), so("so_ho_so", "Hồ sơ"), so("so_trang", "Trang"), so("vb_co_tai_lieu_so", "Có tài liệu số")],
    },
    Def {
        ma: "tai_lieu_so", ten: "Tài liệu số: PDF/A, chữ ký số, virus", can_cu: "Điều 7, 8 TT 05/2025/TT-BNV", nhom: "tai_lieu", ban: "ca_hai", bo_loc: &["phong", "thoi_gian"], tong: true,
        cot: &[ma("ma_du_an", "Mã phông", 12.0), chu("ten_phong", "Phông", 34.0), so("so_tai_lieu", "Tài liệu"), so("dung_luong_mb", "MB"), so("so_trang", "Trang"), so("pdfa_dat", "PDF/A đạt"), so("pdfa_khong_dat", "PDF/A lỗi"), so("pdfa_chua_kiem", "Chưa kiểm PDF/A"), so("ky_hop_le", "Ký hợp lệ"), so("ky_khong_hop_le", "Ký không hợp lệ"), so("chua_ky", "Chưa ký"), so("virus_sach", "Virus: sạch"), so("virus_nhiem", "Virus: nhiễm"), so("virus_chua_quet", "Chưa quét"), so("da_luu_tru", "Đã lưu trữ")],
    },
    Def {
        ma: "goi_tin", ten: "Gói tin AIP / SIP / DIP", can_cu: "Điều 9, 10 TT 05/2025/TT-BNV", nhom: "tai_lieu", ban: "ca_hai", bo_loc: &["thoi_gian"], tong: true,
        cot: &[ma("loai", "Loại gói", 12.0), chu("nguon", "Nguồn", 16.0), so("so_goi", "Gói"), so("da_dung", "Đã dựng"), so("loi", "Lỗi"), so("dang_dung", "Đang dựng"), so("so_tai_lieu", "Tài liệu"), so("dung_luong_mb", "MB"), so("da_huy", "Đã huỷ"), so("virus_nhiem", "Nhiễm virus")],
    },
    Def {
        ma: "thu_nop", ten: "Thu nộp vào lưu trữ lịch sử", can_cu: "Điều 20–25 TT 05/2025/TT-BNV", nhom: "tai_lieu", ban: "thi_cong", bo_loc: &["thoi_gian"], tong: true,
        cot: &[ma("thang", "Tháng", 10.0), so("so_dang_ky", "Đăng ký"), so("tu_choi", "Từ chối"), so("cho_nhan", "Chờ nhận"), so("da_nhan", "Đã nhận"), so("da_duyet", "Đã duyệt"), so("hoan_tat", "Hoàn tất"), so("so_trang", "Trang"), so("dung_luong_mb", "MB"), so("truc_tiep", "Nộp trực tiếp")],
    },
    Def {
        ma: "nhap_he_thong", ten: "Nhập Hệ thống (gói AIP)", can_cu: "Điều 9 khoản 7 TT 05/2025/TT-BNV", nhom: "tai_lieu", ban: "ca_hai", bo_loc: &["thoi_gian"], tong: true,
        cot: &[ma("thang", "Tháng", 10.0), so("so_dot", "Đợt"), so("so_goi", "Gói"), so("da_nhap", "Đã nhập"), so("da_co", "Đã có"), so("tu_choi", "Từ chối"), so("loi", "Lỗi"), so("so_van_ban", "Văn bản"), so("so_chua_ky", "Chưa ký"), so("dung_luong_mb", "MB")],
    },
    Def {
        ma: "kiem_tra", ten: "Kiểm tra toàn vẹn tài liệu", can_cu: "Điều 29 TT 05/2025/TT-BNV", nhom: "bao_quan", ban: "ca_hai", bo_loc: &["thoi_gian"], tong: true,
        cot: &[ma("thang", "Tháng", 10.0), chu("nguon", "Nguồn", 22.0), so("so_lan", "Lượt kiểm"), so("dat", "Đạt"), so("loi", "Lỗi"), so("so_tai_lieu", "Tài liệu"), so("so_goi", "Gói"), so("so_ke_hoach", "Kế hoạch")],
    },
    Def {
        ma: "sao_luu", ten: "Sao lưu dữ liệu", can_cu: "Điều 30 TT 05/2025/TT-BNV", nhom: "bao_quan", ban: "ca_hai", bo_loc: &["thoi_gian"], tong: true,
        cot: &[ma("thang", "Tháng", 10.0), chu("loai", "Loại", 16.0), so("so_lan", "Lượt"), so("dat", "Đạt"), so("khong_dat", "Không đạt"), so("dung_luong_mb", "MB"), so("da_xoa", "Đã xoá"), so("so_phuong_tien", "Phương tiện")],
    },
    Def {
        ma: "tieu_huy", ten: "Tiêu huỷ tài liệu hết giá trị", can_cu: "Điều 48–50 TT 05/2025/TT-BNV", nhom: "bao_quan", ban: "ca_hai", bo_loc: &["nam"], tong: true,
        cot: &[ma("nam", "Năm", 8.0), chu("trang_thai", "Trạng thái", 18.0), chu("quyet_dinh_so", "Số quyết định", 16.0), chu("hinh_thuc", "Hình thức", 12.0), so("so_tai_lieu", "Tài liệu"), so("het_han", "Hết hạn"), so("trung_lap", "Trùng lặp"), so("da_huy", "Đã huỷ"), so("giu_lai", "Giữ lại"), ngay("created_at", "Lập"), ngay("decided_at", "Quyết định"), ngay("executed_at", "Thực hiện")],
    },
    Def {
        ma: "khai_thac_yeu_cau", ten: "Yêu cầu khai thác, sử dụng", can_cu: "Điều 35–40 TT 05/2025/TT-BNV", nhom: "khai_thac", ban: "ca_hai", bo_loc: &["thoi_gian"], tong: true,
        cot: &[chu("hinh_thuc", "Hình thức", 22.0), so("so_yeu_cau", "Yêu cầu"), so("cho_duyet", "Chờ duyệt"), so("da_duyet", "Đã duyệt"), so("tu_choi", "Từ chối"), so("het_han", "Hết hạn"), so("so_tai_lieu", "Tài liệu"), so("tl_duyet", "TL duyệt"), so("tl_tu_choi", "TL từ chối"), so("phi", "Phí (đ)"), so("phi_da_thu", "Đã thu (đ)"), so("trong_han", "Đúng hạn"), so("qua_han", "Quá hạn"), so("pct_dung_han", "% đúng hạn")],
    },
    Def {
        ma: "luot_khai_thac", ten: "Lượt khai thác (xem, tải, in, chuyển đổi)", can_cu: "Điều 44.3b; Điều 13–16 TT 05/2025/TT-BNV", nhom: "khai_thac", ban: "ca_hai", bo_loc: &["thoi_gian"], tong: true,
        cot: &[ma("thang", "Tháng", 10.0), chu("viec", "Việc", 20.0), so("so_luot", "Lượt"), so("so_tai_lieu", "Tài liệu"), so("so_nguoi", "Người")],
    },
    Def {
        ma: "cung_cap_thong_tin", ten: "Cung cấp thông tin tài liệu lưu trữ", can_cu: "Điều 41–43 TT 05/2025/TT-BNV", nhom: "khai_thac", ban: "ca_hai", bo_loc: &["thoi_gian"], tong: true,
        cot: &[chu("hinh_thuc", "Hình thức", 18.0), so("so_yeu_cau", "Yêu cầu"), so("dang_xu_ly", "Đang xử lý"), so("da_duyet", "Đã duyệt"), so("da_cung_cap", "Đã cung cấp"), so("tu_choi", "Từ chối"), so("phi", "Phí (đ)"), so("phi_da_thu", "Đã thu (đ)"), so("ngay_xu_ly_tb", "Ngày xử lý TB")],
    },
    Def {
        ma: "muon_tra", ten: "Sổ mượn – trả hồ sơ giấy", can_cu: "Điều 32 TT 05/2025/TT-BNV", nhom: "khai_thac", ban: "ca_hai", bo_loc: &["phong", "thoi_gian"], tong: false,
        cot: &[ma("so_phieu", "Số phiếu", 14.0), ngay("ngay_muon", "Ngày mượn"), chu("nguoi_muon", "Người mượn", 22.0), chu("don_vi", "Đơn vị", 22.0), ma("ma_ho_so", "Mã hồ sơ", 18.0), chu("tieu_de", "Tiêu đề hồ sơ", 44.0), ma("hop_so", "Hộp", 8.0), ngay("ngay_du_kien_tra", "Hạn trả"), so("so_lan_gia_han", "Gia hạn"), ngay("ngay_thuc_te_tra", "Ngày trả"), chu("trang_thai", "Trạng thái", 16.0), chu("nguoi_lap", "Người lập", 18.0)],
    },
    Def {
        ma: "chuyen_doi", ten: "Sổ chuyển đổi tài liệu số ra giấy", can_cu: "Điều 13–16, Phụ lục VI TT 05/2025/TT-BNV", nhom: "khai_thac", ban: "ca_hai", bo_loc: &["phong", "thoi_gian"], tong: false,
        cot: &[ma("so_chuyen_doi", "Số chuyển đổi", 16.0), ngay("created_at", "Thời điểm"), ma("ma_tai_lieu", "Mã tài liệu gốc", 24.0), chu("ten_tai_lieu", "Tài liệu", 40.0), ma("ma_ho_so", "Mã hồ sơ", 18.0), chu("ten_phong", "Phông", 24.0), so("so_trang", "Trang"), chu("muc_dich", "Mục đích", 22.0), chu("nguoi_nhan", "Người nhận", 18.0), chu("ky_so_ket_luan", "Chữ ký số", 16.0), chu("nguoi_thuc_hien", "Người thực hiện", 18.0)],
    },
    Def {
        ma: "kho_gia", ten: "Kho – giá – hộp (mức đầy)", can_cu: "Điều 44.3b TT 05/2025/TT-BNV", nhom: "kho", ban: "ca_hai", bo_loc: &["phong"], tong: true,
        cot: &[chu("ten_kho", "Kho", 20.0), chu("ten", "Giá", 20.0), chu("ten_phong", "Phông", 26.0), chu("ten_phong_ban", "Phòng ban", 24.0), so("tu_hop", "Từ hộp"), so("den_hop", "Đến hộp"), so("suc_chua", "Sức chứa"), so("so_hop", "Hộp đang chứa"), so("so_ho_so", "Hồ sơ"), so("pct_day", "% đầy"), bo("gan_day", "Gần đầy")],
    },
];

fn def_of(ma: &str) -> Option<&'static Def> {
    DEFS.iter().find(|d| d.ma == ma)
}

fn ban_hop(d: &Def) -> bool {
    match d.ban {
        "thi_cong" => !crate::config::edition_luu_tru(),
        "luu_tru" => crate::config::edition_luu_tru(),
        _ => true,
    }
}

fn def_json(d: &Def) -> Value {
    json!({
        "ma": d.ma, "ten": d.ten, "canCu": d.can_cu, "nhom": d.nhom, "ban": d.ban, "boLoc": d.bo_loc, "tong": d.tong,
        "cot": d.cot.iter().map(|c| json!({ "k": c.k, "ten": c.ten, "kieu": c.kieu.as_str(), "rong": c.rong })).collect::<Vec<_>>(),
    })
}

async fn quyen(st: &AppState, ts: &Ts, project: Uuid) -> Result<(), ApiError> {
    if !(ts.has_perm(Permission::StatsView) || ts.has_perm(Permission::StatsViewProject)) {
        return Err(ApiError::forbidden());
    }
    project_access(st, ts, project).await
}

async fn danh_muc(ts: Ts) -> Result<Json<Value>, ApiError> {
    if !(ts.has_perm(Permission::StatsView) || ts.has_perm(Permission::StatsViewProject)) {
        return Err(ApiError::forbidden());
    }
    Ok(Json(json!({
        "nhom": NHOM.iter().map(|(k, t)| json!({ "ma": k, "ten": t })).collect::<Vec<_>>(),
        "baoCao": DEFS.iter().filter(|d| ban_hop(d)).map(def_json).collect::<Vec<_>>(),
    })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Q {
    project_id: Uuid,
    #[serde(default)]
    phong_id: Option<Uuid>,
    #[serde(default)]
    nam_tu: Option<i32>,
    #[serde(default)]
    nam_den: Option<i32>,
    #[serde(default)]
    from: Option<String>,
    #[serde(default)]
    to: Option<String>,
    #[serde(default)]
    format: Option<String>,
}

/// Ngày VN → mốc UTC (đầu ngày +07:00); `to` là ngày kết thúc bao gồm → +1 ngày.
fn vn(s: Option<&str>, end: bool) -> Result<Option<DateTime<Utc>>, ApiError> {
    let Some(s) = s.map(str::trim).filter(|x| !x.is_empty()) else { return Ok(None) };
    let d = NaiveDate::parse_from_str(s, "%d/%m/%Y").or_else(|_| NaiveDate::parse_from_str(s, "%Y-%m-%d")).map_err(|_| ApiError::bad_request(format!("Ngày {s:?} không hợp lệ (dd/MM/yyyy)")))?;
    let d = if end { d.succ_opt().unwrap_or(d) } else { d };
    Ok(Some(d.and_hms_opt(0, 0, 0).unwrap().and_utc() - chrono::Duration::hours(7)))
}

fn ngay_vn(t: DateTime<Utc>) -> String {
    t.with_timezone(&chrono::FixedOffset::east_opt(7 * 3600).expect("+07:00")).format("%d/%m/%Y").to_string()
}

async fn so_lieu(st: &AppState, d: &Def, b: &BoLoc) -> Result<Vec<Value>, ApiError> {
    let pool = &st.pool;
    let rows = match d.ma {
        "phong" => db::phong(pool, b).await?,
        "ho_so_nam" => db::ho_so_nam(pool, b).await?,
        "phong_ban" => digitext_db::chinh_ly::thong_ke::chinh_ly(pool, b.project_id, b.phong_id, b.phong_ids.as_deref()).await?,
        "thbq" => db::thbq(pool, b, digitext_db::nop_luu::hom_nay_vn().year()).await?,
        "che_do_su_dung" => db::che_do_su_dung(pool, b).await?,
        "do_tin_cay" => db::do_tin_cay(pool, b).await?,
        "loai_van_ban" => db::loai_van_ban(pool, b).await?,
        "tai_lieu_so" => db::tai_lieu_so(pool, b).await?,
        "goi_tin" => db::goi_tin(pool, b).await?,
        "thu_nop" => db::thu_nop(pool, b).await?,
        "nhap_he_thong" => db::nhap_he_thong(pool, b).await?,
        "kiem_tra" => db::kiem_tra(pool, b).await?,
        "sao_luu" => db::sao_luu(pool, b).await?,
        "tieu_huy" => db::tieu_huy(pool, b).await?,
        "khai_thac_yeu_cau" => db::khai_thac_yeu_cau(pool, b).await?,
        "luot_khai_thac" => db::luot_khai_thac(pool, b).await?,
        "cung_cap_thong_tin" => db::cung_cap_thong_tin(pool, b).await?,
        "muon_tra" => db::muon_tra(pool, b).await?,
        "kho_gia" => db::kho_gia(pool, b).await?,
        "chuyen_doi" => db::chuyen_doi(pool, b).await?,
        _ => return Err(ApiError::not_found()),
    };
    Ok(rows)
}

fn so_f64(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.trim().parse().ok(),
        _ => None,
    }
}
fn fmt_so(x: f64) -> String {
    if x.fract().abs() < 1e-9 {
        format!("{}", x.round() as i64)
    } else {
        format!("{x:.1}")
    }
}
/// Giá trị ô dạng chữ để xuất tệp (số giữ dạng số thô cho Excel cộng được; ngày về dd/MM/yyyy).
fn o_chu(c: &Cot, v: &Value) -> String {
    match c.kieu {
        Kieu::So => so_f64(v).map(fmt_so).unwrap_or_default(),
        Kieu::Ngay => match v.as_str() {
            Some(s) if s.len() >= 10 => {
                let d = &s[..10];
                NaiveDate::parse_from_str(d, "%Y-%m-%d").map(|d| d.format("%d/%m/%Y").to_string()).unwrap_or_else(|_| s.to_string())
            }
            Some(s) => s.to_string(),
            None => String::new(),
        },
        Kieu::Bool => match v.as_bool() {
            Some(true) => "Có".into(),
            Some(false) => "".into(),
            None => String::new(),
        },
        _ => match v {
            Value::String(s) => s.clone(),
            Value::Null => String::new(),
            Value::Number(n) => n.to_string(),
            other => other.to_string(),
        },
    }
}

/// Dòng tổng: cộng cột số (trừ cột `pct_day`, `ngay_xu_ly_tb` là trung bình → để trống), cột chữ đầu ghi "Tổng cộng".
fn dong_tong(d: &Def, rows: &[Value]) -> Option<Vec<String>> {
    if !d.tong || rows.is_empty() {
        return None;
    }
    let mut out: Vec<String> = Vec::with_capacity(d.cot.len());
    let mut da_ghi_nhan = false;
    for c in d.cot {
        match c.kieu {
            Kieu::So if !matches!(c.k, "pct_day" | "ngay_xu_ly_tb" | "tu_hop" | "den_hop") => {
                let s: f64 = rows.iter().filter_map(|r| so_f64(&r[c.k])).sum();
                out.push(fmt_so(s));
            }
            Kieu::Chu | Kieu::Ma if !da_ghi_nhan => {
                out.push("Tổng cộng".into());
                da_ghi_nhan = true;
            }
            _ => out.push(String::new()),
        }
    }
    Some(out)
}

fn mo_ta_bo_loc(d: &Def, q: &Q, ten_phong: Option<&str>) -> Vec<(String, String)> {
    let mut v = Vec::new();
    if d.bo_loc.contains(&"phong") {
        v.push(("Phông".into(), ten_phong.map(str::to_owned).unwrap_or_else(|| "Tất cả phông".into())));
    }
    if d.bo_loc.contains(&"nam") {
        let s = match (q.nam_tu, q.nam_den) {
            (Some(a), Some(b)) if a == b => format!("Năm {a}"),
            (Some(a), Some(b)) => format!("Từ năm {a} đến năm {b}"),
            (Some(a), None) => format!("Từ năm {a}"),
            (None, Some(b)) => format!("Đến năm {b}"),
            _ => "Mọi năm".into(),
        };
        v.push(("Năm".into(), s));
    }
    if d.bo_loc.contains(&"thoi_gian") {
        let f = q.from.clone().filter(|s| !s.trim().is_empty());
        let t = q.to.clone().filter(|s| !s.trim().is_empty());
        let s = match (f, t) {
            (Some(a), Some(b)) => format!("Từ {} đến {}", ngay_dep(&a), ngay_dep(&b)),
            (Some(a), None) => format!("Từ {}", ngay_dep(&a)),
            (None, Some(b)) => format!("Đến {}", ngay_dep(&b)),
            _ => "Toàn bộ thời gian".into(),
        };
        v.push(("Thời gian".into(), s));
    }
    v
}
fn ngay_dep(s: &str) -> String {
    NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d").map(|d| d.format("%d/%m/%Y").to_string()).unwrap_or_else(|_| s.trim().to_string())
}

async fn chay(State(st): State<AppState>, ts: Ts, Path(ma): Path<String>, Query(q): Query<Q>) -> Result<Response, ApiError> {
    let d = def_of(&ma).filter(|d| ban_hop(d)).ok_or_else(ApiError::not_found)?;
    quyen(&st, &ts, q.project_id).await?;
    let scope = pham_vi_phong(&st, &ts, q.project_id).await?;
    let phong_id = phong_hieu_dung(scope.as_deref(), q.phong_id)?;
    let b = BoLoc { project_id: q.project_id, phong_id, nam_tu: q.nam_tu, nam_den: q.nam_den, from: vn(q.from.as_deref(), false)?, to: vn(q.to.as_deref(), true)?, phong_ids: scope };
    if let (Some(a), Some(c)) = (b.nam_tu, b.nam_den) {
        if a > c {
            return Err(ApiError::bad_request("Năm bắt đầu phải không lớn hơn năm kết thúc"));
        }
    }
    let rows = so_lieu(&st, d, &b).await?;
    let tong = dong_tong(d, &rows);
    let format = q.format.as_deref().unwrap_or("json").trim().to_lowercase();
    if format == "json" {
        return Ok(Json(json!({ "def": def_json(d), "rows": rows, "tong": tong })).into_response());
    }
    let cq = digitext_db::co_quan::hieu_dung(&st.pool, q.project_id).await?;
    let ten_cq = cq.ten.clone();
    let ten_phong = match q.phong_id {
        Some(p) => sqlx::query_scalar::<_, String>("SELECT ma_du_an || ' — ' || ten FROM cl_phong WHERE id = $1").bind(p).fetch_optional(&st.pool).await?,
        None => None,
    };
    let bo_loc = mo_ta_bo_loc(d, &q, ten_phong.as_deref());
    let bang: Vec<Vec<String>> = rows.iter().map(|r| d.cot.iter().map(|c| o_chu(c, &r[c.k])).collect()).collect();
    let hom_nay = ngay_vn(Utc::now());
    let ten_tep = format!("bao-cao-{}-{}", d.ma.replace('_', "-"), Utc::now().with_timezone(&chrono::FixedOffset::east_opt(7 * 3600).expect("+07:00")).format("%Y%m%d"));
    match format.as_str() {
        "xlsx" => {
            let cot: Vec<CotBang> = d
                .cot
                .iter()
                .map(|c| match c.kieu {
                    Kieu::So => CotBang::so(c.ten, c.rong),
                    Kieu::Ma | Kieu::Ngay | Kieu::Bool => CotBang::ma(c.ten, c.rong),
                    Kieu::Chu => CotBang::chu(c.ten, c.rong),
                })
                .collect();
            let mut dau = vec![ten_cq.clone(), format!("BÁO CÁO {}", d.ten.to_uppercase()), format!("Căn cứ: {}", d.can_cu)];
            for (k, v) in &bo_loc {
                dau.push(format!("{k}: {v}"));
            }
            dau.push(format!("Lập ngày {hom_nay} — {}", ts.user.display_name));
            let sheet = xlsx::sheet_bang(d.ten, &dau, &cot, &bang, tong.as_deref());
            let bytes = xlsx::dong_goi(&[sheet]).map_err(|e| {
                tracing::error!(error = %e, "đóng gói Excel báo cáo lỗi");
                ApiError::internal()
            })?;
            Ok(tep(bytes, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet", &format!("{ten_tep}.xlsx")))
        }
        "pdf" => {
            let font_path = digitext_pdf::f_1cq().ok_or_else(|| ApiError::service_unavailable("không tìm thấy font TTF cho báo cáo — đặt DIGITEXT_CERT_FONT"))?;
            let font = std::fs::read(&font_path).map_err(|e| {
                tracing::error!(path = %font_path.display(), error = %e, "đọc font báo cáo lỗi");
                ApiError::internal()
            })?;
            let now_vn = Utc::now().with_timezone(&chrono::FixedOffset::east_opt(7 * 3600).expect("+07:00"));
            let mut blocks = vec![
                ManualBlock::OfficialHeader {
                    left: cq.ten_cap_tren.iter().map(|t| t.to_uppercase()).chain(std::iter::once(format!("*{}", ten_cq.to_uppercase()))).collect(),
                    right: vec!["*CỘNG HOÀ XÃ HỘI CHỦ NGHĨA VIỆT NAM".into(), "*Độc lập - Tự do - Hạnh phúc".into()],
                    dated: format!("Ngày {} tháng {} năm {}", now_vn.day(), now_vn.month(), now_vn.year()),
                },
                ManualBlock::Title(format!("BÁO CÁO {}", d.ten.to_uppercase())),
                ManualBlock::Subtitle(format!("Căn cứ: {}", d.can_cu)),
                ManualBlock::Gap(6.0),
            ];
            if !bo_loc.is_empty() {
                blocks.push(ManualBlock::KeyValue(bo_loc.clone()));
                blocks.push(ManualBlock::Gap(8.0));
            }
            let mut rows_pdf: Vec<Vec<String>> = bang.iter().enumerate().map(|(i, r)| std::iter::once((i + 1).to_string()).chain(r.iter().cloned()).collect()).collect();
            if let Some(t) = &tong {
                rows_pdf.push(std::iter::once(String::new()).chain(t.iter().cloned()).collect());
            }
            let mut headers = vec!["TT".to_string()];
            headers.extend(d.cot.iter().map(|c| c.ten.to_string()));
            // Bề rộng Excel (ký tự) dùng làm tỷ lệ; cột ngày / mã cần tối thiểu để "27/11/2026" không gãy dòng ở 9,8 pt.
            let mut widths = vec![5.0f32];
            widths.extend(d.cot.iter().map(|c| match c.kieu {
                Kieu::Ngay => c.rong.max(15.0) as f32,
                Kieu::Ma => c.rong.max(11.0) as f32,
                Kieu::So => c.rong.max(10.0) as f32,
                _ => c.rong as f32,
            }));
            blocks.push(ManualBlock::Table { headers, widths, rows: rows_pdf });
            blocks.push(ManualBlock::Gap(16.0));
            blocks.push(ManualBlock::Signatures {
                left: ("NGƯỜI LẬP BÁO CÁO".into(), "(Ký, ghi rõ họ tên)".into(), ts.user.display_name.clone()),
                right: (cq.chuc_vu_nguoi_ky.to_uppercase(), "(Ký, ghi rõ họ tên, đóng dấu)".into(), cq.nguoi_ky.clone().unwrap_or_default()),
            });
            blocks.push(ManualBlock::Footer(format!("DigiText — số liệu lấy lúc {} · {} dòng", now_vn.format("%d/%m/%Y %H:%M"), rows.len())));
            let ngang = d.cot.len() > 7;
            let bytes = if ngang { digitext_pdf::manual::render_manual_ngang(&blocks, &font) } else { digitext_pdf::manual::render_manual(&blocks, &font) }
                .map_err(|e| ApiError::bad_request(format!("dựng PDF báo cáo lỗi: {e}")))?;
            Ok(tep(bytes, "application/pdf", &format!("{ten_tep}.pdf")))
        }
        other => Err(ApiError::bad_request(format!("format {other:?} không hỗ trợ (json | xlsx | pdf)"))),
    }
}

pub(crate) fn tep(bytes: Vec<u8>, kieu: &str, ten: &str) -> Response {
    ([(header::CONTENT_TYPE, kieu.to_string()), (header::CONTENT_DISPOSITION, format!("attachment; filename=\"{ten}\""))], Body::from(bytes)).into_response()
}
