// Copyright (c) 2026 DigiText. All rights reserved.
//! Webhook nhận dự án từ hệ thống ngoài (migration 0028, 9/11/2026).
//!
//! `POST /api/v1/hooks/du-an` — hệ thống quản lý hợp đồng bắn về tên dự án, loại dự án,
//! khối lượng công việc và ngày kết thúc hợp đồng; Hệ thống tự tạo dự án và ghi khối
//! lượng vào `project_contracts` để màn Báo cáo tiến độ tính ngay được % hoàn thành.
//!
//! Không dùng extractor `Ts` (đăng nhập người dùng) — bên gọi là máy, xác thực bằng
//! khoá trong bảng `api_clients`, kèm chữ ký HMAC-SHA256 nếu khoá đó có đặt bí mật ký.
//!
//! Tài liệu cho bên gửi: `docs/WEBHOOK-DU-AN.md`.

use crate::app::AppState;
use crate::client_ip::T2t;
use crate::error::ApiError;
use crate::extract::Ts;
use axum::body::Bytes;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::NaiveDate;
use digitext_db::progress::Contract;
use digitext_db::webhooks::{self as wh, ApiClient};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use uuid::Uuid;

/// Gói tin cũ hơn mốc này thì từ chối — chặn phát lại gói đã chặn được.
const LECH_GIAY_TOI_DA: i64 = 300;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/hooks/du-an", post(nhan_du_an))
        .route("/hooks/kiem-tra", get(kiem_tra).post(kiem_tra))
        .route("/hooks/nhat-ky", get(nhat_ky))
}

// ---------------------------------------------------------------- xác thực

fn loi_khoa(message: impl Into<String>) -> ApiError {
    ApiError {
        status: StatusCode::UNAUTHORIZED,
        code: "hook_unauthorized",
        message: message.into(),
    }
}

/// HMAC-SHA256 tự dựng trên sha2 — khỏi thêm phụ thuộc mới vào workspace.
fn hmac_sha256(key: &[u8], msg: &[u8]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut k = [0u8; 64];
    if key.len() > 64 {
        k[..32].copy_from_slice(&Sha256::digest(key));
    } else {
        k[..key.len()].copy_from_slice(key);
    }
    let mut trong = [0x36u8; 64];
    let mut ngoai = [0x5cu8; 64];
    for i in 0..64 {
        trong[i] ^= k[i];
        ngoai[i] ^= k[i];
    }
    let giua = Sha256::new()
        .chain_update(trong)
        .chain_update(msg)
        .finalize();
    let cuoi = Sha256::new()
        .chain_update(ngoai)
        .chain_update(giua)
        .finalize();
    let mut ra = [0u8; 32];
    ra.copy_from_slice(&cuoi);
    ra
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// So sánh không rò thời gian — tránh dò chữ ký từng ký tự.
fn bang_nhau(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |t, (x, y)| t | (x ^ y)) == 0
}

fn lay_header(h: &HeaderMap, ten: &str) -> Option<String> {
    h.get(ten)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

/// Đệm kết luận "bí mật này khớp chuỗi PHC này", sống 60 giây.
///
/// Mỗi lần xác thực là một lần Argon2id 19 MiB. Webhook gọi vài lần/ngày nên không ai
/// thấy, nhưng cổng nhà cung cấp bên DO365 gọi **mỗi lần có người mở màn hình**: hàng
/// chục lời gọi cùng lúc đủ chiếm hết luồng worker và kéo theo tải tệp, callback OCR,
/// ký số.
///
/// Chỉ đệm **kết luận của phép băm**, không đệm bản ghi `api_clients`: mỗi lời gọi vẫn
/// tra khoá trong CSDL, vẫn lọc IP, vẫn kiểm lệch giờ và chữ ký. Nhờ vậy `thu-hoi`
/// (đổi `is_active`) và `doi` (đổi `secret_hash` ⇒ đổi luôn vân tay) có hiệu lực ngay
/// lập tức, không phải chờ hết hạn đệm.
static DEM_KHOA: std::sync::OnceLock<
    std::sync::Mutex<std::collections::HashMap<[u8; 32], std::time::Instant>>,
> = std::sync::OnceLock::new();
const DEM_KHOA_TTL: std::time::Duration = std::time::Duration::from_secs(60);
const DEM_KHOA_TRAN: usize = 256;

/// Vân tay của bộ ba (khoá công khai, bí mật, chuỗi băm). Băm lại để **không giữ bí mật
/// thô trong bộ nhớ tiến trình**.
fn dem_khoa_van_tay(key_id: &str, bi_mat: &str, bam: &str) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    Sha256::new()
        .chain_update(key_id.as_bytes())
        .chain_update([0u8])
        .chain_update(bi_mat.as_bytes())
        .chain_update([0u8])
        .chain_update(bam.as_bytes())
        .finalize()
        .into()
}

fn dem_khoa_con_han(van_tay: &[u8; 32]) -> bool {
    let dem = DEM_KHOA.get_or_init(Default::default);
    let Ok(d) = dem.lock() else { return false };
    d.get(van_tay).is_some_and(|at| at.elapsed() < DEM_KHOA_TTL)
}

fn dem_khoa_ghi(van_tay: [u8; 32]) {
    let dem = DEM_KHOA.get_or_init(Default::default);
    if let Ok(mut d) = dem.lock() {
        d.retain(|_, at| at.elapsed() < DEM_KHOA_TTL);
        if d.len() >= DEM_KHOA_TRAN {
            return;
        }
        d.insert(van_tay, std::time::Instant::now());
    }
}

/// Kiểm khoá, IP và chữ ký. Trả về hệ thống gọi, hoặc 401/403.
pub(crate) async fn xac_thuc(
    st: &AppState,
    headers: &HeaderMap,
    ip: &T2t,
    body: &[u8],
) -> Result<ApiClient, ApiError> {
    let khoa = lay_header(headers, "x-digitext-key")
        .ok_or_else(|| loi_khoa("thiếu header X-DigiText-Key"))?;
    let (key_id, bi_mat) = khoa
        .split_once('.')
        .ok_or_else(|| loi_khoa("X-DigiText-Key phải có dạng <keyId>.<bí mật>"))?;

    let kh = wh::tim_client(&st.pool, key_id)
        .await?
        .filter(|c| c.is_active)
        .ok_or_else(|| loi_khoa("khoá không tồn tại hoặc đã bị thu hồi"))?;
    // Khoá đã xác minh trong 60 giây gần đây thì bỏ qua Argon2; bí mật sai luôn cho vân
    // tay khác nên không bao giờ trúng đệm, tức mọi lần dò đều phải trả giá băm đầy đủ.
    let van_tay = dem_khoa_van_tay(key_id, bi_mat, &kh.secret_hash);
    let da_dem = dem_khoa_con_han(&van_tay);
    if !da_dem {
        // Chống dò bí mật, chốt theo ĐỊA CHỈ người gọi — cố ý không chốt theo `key_id`:
        // phần công khai của khoá nằm ngay trong header, ai xem một gói tin cũng đọc
        // được, nên chốt theo nó thì người lạ chỉ cần gửi bí mật bậy vài lần là khoá
        // cửa của chính hệ thống thật. Chốt theo IP thì kẻ dò tự khoá mình.
        if let Some(dia_chi) = ip.0 {
            if let Err(giay) = st.limiter.check_ip(dia_chi) {
                return Err(ApiError::too_many_attempts(giay));
            }
        }
        // Bản async: đẩy phép băm sang spawn_blocking, trần 8 chỗ cùng lúc. Bản đồng bộ
        // `password::verify` chạy thẳng trên luồng worker Tokio và sẽ chặn cả runtime.
        if !crate::password::f_2pn(&kh.secret_hash, bi_mat).await {
            // Chỉ ghi lần sai, không xoá sổ khi đúng: bộ đếm IP dùng chung với đường
            // đăng nhập người dùng, xoá hộ ở đây là mở lối gỡ chốt cho đường kia.
            if let Some(dia_chi) = ip.0 {
                st.limiter.record_failure_ip(dia_chi);
            }
            return Err(loi_khoa("bí mật của khoá không đúng"));
        }
        dem_khoa_ghi(van_tay);
    }

    // Lọc IP (nếu khoá có đặt). Danh sách rỗng = không lọc.
    if let Some(cho_phep) = kh.ip_allow.as_ref().filter(|v| !v.is_empty()) {
        let hien = ip.0.map(|i| i.to_string()).unwrap_or_default();
        if !cho_phep.iter().any(|a| a.trim() == hien) {
            return Err(ApiError {
                status: StatusCode::FORBIDDEN,
                code: "hook_ip_forbidden",
                message: format!("IP {hien} không nằm trong danh sách cho phép của khoá"),
            });
        }
    }

    // Chữ ký chỉ bắt buộc khi khoá có bí mật ký.
    if let Some(bi_mat_ky) = kh.secret_hmac.as_ref().filter(|s| !s.is_empty()) {
        let ts = lay_header(headers, "x-digitext-timestamp")
            .ok_or_else(|| loi_khoa("thiếu header X-DigiText-Timestamp"))?;
        let ts_num: i64 = ts
            .parse()
            .map_err(|_| loi_khoa("X-DigiText-Timestamp phải là số giây Unix"))?;
        let lech = (chrono::Utc::now().timestamp() - ts_num).abs();
        if lech > LECH_GIAY_TOI_DA {
            return Err(loi_khoa(format!(
                "dấu thời gian lệch {lech} giây, quá mức cho phép {LECH_GIAY_TOI_DA} giây — kiểm lại đồng hồ máy gửi"
            )));
        }
        let ky = lay_header(headers, "x-digitext-signature")
            .ok_or_else(|| loi_khoa("thiếu header X-DigiText-Signature"))?;
        let ky = ky
            .strip_prefix("sha256=")
            .unwrap_or(&ky)
            .to_ascii_lowercase();
        let mut chuoi = Vec::with_capacity(ts.len() + 1 + body.len());
        chuoi.extend_from_slice(ts.as_bytes());
        chuoi.push(b'.');
        chuoi.extend_from_slice(body);
        let dung = hex(&hmac_sha256(bi_mat_ky.as_bytes(), &chuoi));
        if !bang_nhau(ky.as_bytes(), dung.as_bytes()) {
            return Err(loi_khoa(
                "chữ ký không khớp — ký chuỗi \"<timestamp>.<thân gói>\" bằng HMAC-SHA256",
            ));
        }
    }

    // Chỉ ghi khi vừa qua đường băm thật — tức nhiều nhất mỗi khoá một lần mỗi 60 giây.
    // Cổng nhà cung cấp gọi liên tục mà cứ mỗi lời gọi một UPDATE thì bảng hai dòng này
    // phình vì HOT update, autovacuum chạy theo không xuể.
    if !da_dem {
        let _ = wh::ghi_da_goi(&st.pool, kh.id).await;
    }
    Ok(kh)
}

// ---------------------------------------------------------------- gói tin

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoiTin {
    #[serde(default)]
    event: Option<String>,
    #[serde(default, alias = "maHopDong", alias = "maNgoaiHeThong")]
    ma_ngoai: Option<String>,
    #[serde(default, alias = "maDa")]
    ma_du_an: Option<String>,
    #[serde(default, alias = "tenDa", alias = "ten")]
    ten_du_an: Option<String>,
    #[serde(default, alias = "loai")]
    loai_du_an: Option<String>,
    /// Tên cơ quan chủ đầu tư (0029). Tuỳ chọn; để trống thì giữ nguyên giá trị cũ.
    #[serde(default, alias = "coQuan", alias = "chuDauTuTen")]
    chu_dau_tu: Option<String>,
    #[serde(default)]
    khoi_luong: Option<KhoiLuong>,
    #[serde(default)]
    ngay_ky_hop_dong: Option<String>,
    #[serde(default, alias = "ngayKetThuc")]
    ngay_ket_thuc_hop_dong: Option<String>,
    #[serde(default, alias = "hanChot")]
    han_hoan_thanh: Option<String>,
    #[serde(default, alias = "nguoiPhuTrach")]
    can_bo_phu_trach: Option<String>,
    #[serde(default)]
    ghi_chu: Option<String>,
    #[serde(default)]
    quan_tri: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KhoiLuong {
    #[serde(default, alias = "soTo")]
    to: Option<i64>,
    #[serde(default, alias = "soTrang")]
    trang: Option<i64>,
    #[serde(default, alias = "soHoSo")]
    ho_so: Option<i64>,
    #[serde(default, alias = "soHop")]
    hop: Option<i64>,
    #[serde(default, alias = "soTep")]
    tep: Option<i64>,
    /// Mét giá (mét tài liệu) — khoá DUY NHẤT nhận số lẻ, vì hợp đồng chỉnh lý khai "150.5 m".
    /// Bốn khoá kia là số đếm, số lẻ ở đó là dấu hiệu bên gửi tính sai chứ không phải đơn vị.
    #[serde(default, alias = "soMet", alias = "metGia")]
    met: Option<f64>,
}

impl KhoiLuong {
    fn dem(&self) -> [Option<i64>; 5] {
        [self.to, self.trang, self.ho_so, self.hop, self.tep]
    }
    fn co_gi(&self) -> bool {
        self.dem().iter().any(Option::is_some) || self.met.is_some()
    }
    fn am(&self) -> bool {
        self.dem().iter().any(|v| v.is_some_and(|n| n < 0))
            || self.met.is_some_and(|m| m < 0.0)
    }
    /// Mét không phải số thực hợp lệ (NaN/vô cực lọt qua được JSON của vài thư viện) hoặc lớn
    /// đến mức không còn là mét giá — `numeric(12,2)` chỉ chứa tới 10 chữ số phần nguyên.
    fn met_vo_ly(&self) -> bool {
        self.met.is_some_and(|m| !m.is_finite() || m >= 1e10)
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct KetQua {
    project_id: Uuid,
    ma_ngoai: String,
    ma_du_an: String,
    ten_du_an: String,
    loai_du_an: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    chu_dau_tu: Option<String>,
    /// Khối lượng hợp đồng **sau khi ghi**, chỉ có khi gói tin chạm tới hợp đồng.
    #[serde(skip_serializing_if = "Option::is_none")]
    khoi_luong: Option<Value>,
    tao_moi: bool,
    /// true = mã dự án do Hệ thống tự đặt vì gói tin không gửi mã.
    ma_tu_sinh: bool,
    quan_tri: Vec<String>,
    canh_bao: Vec<String>,
}

/// Loại việc cho phép, kèm mấy cách viết hay gặp để bên gửi khỏi phải tra mã.
const LOAI: &[(&str, &[&str])] = &[
    (
        "so_hoa",
        &["sohoa", "số hoá", "số hóa", "sohóa", "digitize"],
    ),
    ("chinh_ly", &["chinhly", "chỉnh lý", "chinh ly"]),
    (
        "chinh_ly_so_hoa",
        &[
            "chinhlysohoa",
            "chỉnh lý số hoá",
            "chỉnh lý kết hợp số hoá",
            "chinh ly so hoa",
        ],
    ),
    ("luu_tru", &["luutru", "lưu trữ", "luu tru", "nhap aip"]),
    ("khac", &["khác", "other"]),
];

fn chuan_loai(v: &str) -> Option<&'static str> {
    let g = v.trim().to_lowercase();
    if g.is_empty() {
        return None;
    }
    LOAI.iter()
        .find(|(ma, alias)| *ma == g || alias.iter().any(|a| *a == g))
        .map(|(ma, _)| *ma)
}

fn ds_loai() -> String {
    LOAI.iter().map(|(m, _)| *m).collect::<Vec<_>>().join(" | ")
}

/// Mã dự án: chữ HOA, số, `-`, `_`. Không nhận dấu cách hay dấu chấm — mã còn dùng
/// để gõ xác nhận khi xoá dữ liệu dự án và để đặt tên thư mục khi xuất gói.
fn chuan_ma(v: &str) -> Result<String, ApiError> {
    let m = v.trim().to_uppercase();
    if m.chars().count() < 2 || m.chars().count() > 20 {
        return Err(ApiError::bad_request("maDuAn phải dài 2–20 ký tự"));
    }
    if !m
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(ApiError::bad_request(
            "maDuAn chỉ nhận chữ không dấu, số, '-' và '_' (không dấu cách, không dấu chấm)",
        ));
    }
    Ok(m)
}

pub(crate) fn doc_ngay(v: &str, truong: &str) -> Result<NaiveDate, ApiError> {
    let v = v.trim();
    for mau in ["%Y-%m-%d", "%d/%m/%Y", "%d-%m-%Y"] {
        if let Ok(d) = NaiveDate::parse_from_str(v, mau) {
            // chrono nhận cả năm viết tắt ("27" ra năm 0027) — chặn luôn kẻo
            // hợp đồng nằm ở thế kỷ thứ nhất rồi báo cáo tiến độ tính sai.
            use chrono::Datelike;
            if !(2000..=2100).contains(&d.year()) {
                return Err(ApiError::bad_request(format!(
                    "{truong} = {v:?} có năm {} — ghi đủ bốn chữ số năm (YYYY-MM-DD)",
                    d.year()
                )));
            }
            return Ok(d);
        }
    }
    Err(ApiError::bad_request(format!(
        "{truong} = \"{v}\" không đọc được — dùng YYYY-MM-DD"
    )))
}

pub(crate) fn doc_ngay_opt(v: &Option<String>, truong: &str) -> Result<Option<NaiveDate>, ApiError> {
    match v.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(s) => doc_ngay(s, truong).map(Some),
        None => Ok(None),
    }
}

// ---------------------------------------------------------------- xử lý

async fn nhan_du_an(
    State(st): State<AppState>,
    ip: T2t,
    headers: HeaderMap,
    body: Bytes,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    let kh = xac_thuc(&st, &headers, &ip, &body).await?;
    let goi: Option<Value> = serde_json::from_slice(&body).ok();
    let doc_str = |ten: &str| {
        goi.as_ref()
            .and_then(|g| g.get(ten))
            .and_then(Value::as_str)
            .map(str::to_owned)
    };
    let event_goi = doc_str("event");
    let ma_ngoai_goi = doc_str("maNgoai");

    match xu_ly(&st, &kh, &body).await {
        Ok((status, kq)) => {
            let _ = wh::ghi_delivery(
                &st.pool,
                wh::Delivery {
                    client_id: Some(kh.id),
                    nguon: Some(&kh.nguon),
                    event: event_goi.as_deref(),
                    ma_ngoai: Some(&kq.ma_ngoai),
                    payload: goi,
                    status: status.as_u16() as i32,
                    ket_qua: serde_json::to_value(&kq).ok(),
                    loi: None,
                    project_id: Some(kq.project_id),
                    ip: ip.as_audit().as_deref(),
                },
            )
            .await;
            Ok((status, Json(json!(kq))))
        }
        Err(e) => {
            let _ = wh::ghi_delivery(
                &st.pool,
                wh::Delivery {
                    client_id: Some(kh.id),
                    nguon: Some(&kh.nguon),
                    event: event_goi.as_deref(),
                    ma_ngoai: ma_ngoai_goi.as_deref(),
                    payload: goi,
                    status: e.status.as_u16() as i32,
                    ket_qua: None,
                    loi: Some(&e.message),
                    project_id: None,
                    ip: ip.as_audit().as_deref(),
                },
            )
            .await;
            Err(e)
        }
    }
}

async fn xu_ly(
    st: &AppState,
    kh: &ApiClient,
    body: &[u8],
) -> Result<(StatusCode, KetQua), ApiError> {
    let g: GoiTin = serde_json::from_slice(body)
        .map_err(|e| ApiError::bad_request(format!("thân gói không phải JSON hợp lệ: {e}")))?;

    let event = g.event.as_deref().map(str::trim).unwrap_or("du_an.tao");
    if !matches!(
        event,
        "du_an.tao" | "du_an.capnhat" | "du_an.dong" | "du_an.mo"
    ) {
        return Err(ApiError::bad_request(format!(
            "event \"{event}\" không hiểu — dùng du_an.tao | du_an.capnhat | du_an.dong | du_an.mo"
        )));
    }

    let ma_ngoai = g
        .ma_ngoai
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            ApiError::bad_request(
                "thiếu maNgoai — mã dự án/hợp đồng ở hệ thống của bạn, dùng để không tạo trùng khi gửi lại",
            )
        })?;
    if ma_ngoai.chars().count() > 120 {
        return Err(ApiError::bad_request("maNgoai dài quá 120 ký tự"));
    }

    let ten = g
        .ten_du_an
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let loai = match g
        .loai_du_an
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        Some(v) => Some(chuan_loai(v).ok_or_else(|| {
            ApiError::bad_request(format!("loaiDuAn \"{v}\" không hiểu — nhận: {}", ds_loai()))
        })?),
        None => None,
    };
    if let Some(k) = &g.khoi_luong {
        if k.am() {
            return Err(ApiError::bad_request("khối lượng không được âm"));
        }
        if k.met_vo_ly() {
            return Err(ApiError::bad_request(
                "khoiLuong.met không phải số mét hợp lệ (phải hữu hạn và nhỏ hơn 10 tỉ)",
            ));
        }
    }
    let chu_dau_tu = g.chu_dau_tu.as_deref().map(str::trim).filter(|s| !s.is_empty());
    if chu_dau_tu.is_some_and(|v| v.chars().count() > 300) {
        return Err(ApiError::bad_request("chuDauTu dài quá 300 ký tự"));
    }
    let ky = doc_ngay_opt(&g.ngay_ky_hop_dong, "ngayKyHopDong")?;
    let ket_thuc = doc_ngay_opt(&g.ngay_ket_thuc_hop_dong, "ngayKetThucHopDong")?;
    let han = doc_ngay_opt(&g.han_hoan_thanh, "hanHoanThanh")?.or(ket_thuc);
    if let (Some(a), Some(b)) = (ky, ket_thuc) {
        if b < a {
            return Err(ApiError::bad_request(
                "ngayKetThucHopDong sớm hơn ngayKyHopDong",
            ));
        }
    }

    let mut canh_bao: Vec<String> = Vec::new();

    // 1. Dự án này đã về lần nào chưa?
    let da_co = wh::tim_theo_ma_ngoai(&st.pool, &kh.nguon, ma_ngoai).await?;

    let (du_an, tao_moi) = match da_co {
        Some(d) => {
            let bat = match event {
                "du_an.dong" => Some(false),
                "du_an.mo" => Some(true),
                _ => None,
            };
            if let Some(m) = g.ma_du_an.as_deref().filter(|m| !m.trim().is_empty()) {
                let m = chuan_ma(m)?;
                if m != d.code {
                    canh_bao.push(format!(
                        "gói tin gửi maDuAn \"{m}\" nhưng dự án này đang mang mã \"{}\" — Hệ thống giữ mã cũ, đổi mã phải làm trong app",
                        d.code
                    ));
                }
            }
            let d = wh::cap_nhat_du_an(&st.pool, d.id, ten, loai, bat, &kh.nguon, ma_ngoai, chu_dau_tu)
                .await?
                .ok_or_else(ApiError::not_found)?;
            (d, false)
        }
        None => {
            if matches!(event, "du_an.dong" | "du_an.mo") {
                return Err(ApiError {
                    status: StatusCode::NOT_FOUND,
                    code: "not_found",
                    message: format!(
                        "chưa có dự án nào mang maNgoai \"{ma_ngoai}\" — gửi du_an.tao trước"
                    ),
                });
            }
            let ten = ten.ok_or_else(|| {
                ApiError::bad_request("thiếu tenDuAn — bắt buộc khi tạo dự án mới")
            })?;
            if ten.chars().count() > 300 {
                return Err(ApiError::bad_request("tenDuAn dài quá 300 ký tự"));
            }

            // Mã dự án: lấy của bên gửi, không có thì Hệ thống tự đặt DA<yy><nnn>.
            match g.ma_du_an.as_deref().filter(|m| !m.trim().is_empty()) {
                Some(m) => {
                    let ma = chuan_ma(m)?;
                    match wh::tim_theo_ma(&st.pool, &ma).await? {
                        // Người ta đã tạo tay dự án này rồi — nhận về chứ không tạo bản thứ hai.
                        Some(cu) if cu.ma_ngoai.is_none() => {
                            canh_bao.push(format!(
                                "dự án mã {ma} đã có trong Hệ thống (tạo trong app) — đã gắn maNgoai \"{ma_ngoai}\" vào dự án đó thay vì tạo mới"
                            ));
                            let d = wh::cap_nhat_du_an(
                                &st.pool,
                                cu.id,
                                Some(ten),
                                loai,
                                None,
                                &kh.nguon,
                                ma_ngoai,
                                chu_dau_tu,
                            )
                            .await?
                            .ok_or_else(ApiError::not_found)?;
                            (d, false)
                        }
                        Some(cu) => {
                            return Err(ApiError::conflict(format!(
                                "mã dự án {ma} đang dùng cho maNgoai \"{}\" (nguồn {}) — đổi maDuAn khác",
                                cu.ma_ngoai.unwrap_or_default(),
                                cu.nguon.unwrap_or_default()
                            )));
                        }
                        None => (tao(st, kh, &ma, ten, loai, ma_ngoai, false, chu_dau_tu).await?, true),
                    }
                }
                None => {
                    let tien_to = format!("DA{}", chrono::Utc::now().format("%y"));
                    let mut ra = None;
                    // Hai dự án về cùng lúc có thể giành nhau một mã; thử lại vài lần.
                    for _ in 0..5 {
                        let ma = wh::ma_tu_sinh_ke_tiep(&st.pool, &tien_to).await?;
                        match tao(st, kh, &ma, ten, loai, ma_ngoai, true, chu_dau_tu).await {
                            Ok(d) => {
                                ra = Some(d);
                                break;
                            }
                            Err(e) if e.status == StatusCode::CONFLICT => continue,
                            Err(e) => return Err(e),
                        }
                    }
                    let d = ra.ok_or_else(|| {
                        ApiError::conflict("không đặt được mã dự án tự sinh — thử lại")
                    })?;
                    canh_bao.push(format!(
                        "gói tin không gửi maDuAn nên Hệ thống tự đặt mã {} — người quản trị nên đổi sang mã viết tắt dễ nhớ trước khi bắt đầu quét",
                        d.code
                    ));
                    (d, true)
                }
            }
        }
    };

    // 2. Khối lượng và mốc thời gian hợp đồng. Trường nào không gửi thì giữ nguyên
    //    giá trị cũ — không xoá cái người ta đã nhập tay trong app.
    let k = g.khoi_luong.unwrap_or_default();
    let co_hop_dong = k.co_gi()
        || ky.is_some()
        || ket_thuc.is_some()
        || g.can_bo_phu_trach.is_some()
        || g.ghi_chu.is_some();
    // Khối lượng ĐÃ LƯU, trả lại trong phản hồi để bên gửi tự đối chiếu (đề nghị DO365
    // 11/11/2026). Không tốn truy vấn nào: đây đúng là bản ghi vừa viết xuống, kèm cả những
    // trường gói tin không gửi mà hệ thống giữ lại từ lần trước — chính chỗ hay lệch.
    let mut khoi_luong_ra: Option<Value> = None;
    if co_hop_dong {
        let cu = wh::lay_hop_dong(&st.pool, du_an.id).await?;
        let hd = Contract {
            signed_on: ky.or(cu.as_ref().and_then(|c| c.signed_on)),
            end_on: ket_thuc.or(cu.as_ref().and_then(|c| c.end_on)),
            deadline_on: han.or(cu.as_ref().and_then(|c| c.deadline_on)),
            contract_pages: k.trang.or(cu.as_ref().and_then(|c| c.contract_pages)),
            contract_files: k.tep.or(cu.as_ref().and_then(|c| c.contract_files)),
            contract_boxes: k.hop.or(cu.as_ref().and_then(|c| c.contract_boxes)),
            contract_dossiers: k.ho_so.or(cu.as_ref().and_then(|c| c.contract_dossiers)),
            contract_sheets: k.to.or(cu.as_ref().and_then(|c| c.contract_sheets)),
            contract_meters: k.met.or(cu.as_ref().and_then(|c| c.contract_meters)),
            manager_name: g
                .can_bo_phu_trach
                .clone()
                .or_else(|| cu.as_ref().and_then(|c| c.manager_name.clone())),
            note: g
                .ghi_chu
                .clone()
                .or_else(|| cu.as_ref().and_then(|c| c.note.clone())),
        };
        digitext_db::progress::upsert_contract_opt(&st.pool, du_an.id, &hd, None).await?;
        let mut m = Map::new();
        // Đúng tên khoá của gói tin gửi lên, và bỏ hẳn khoá rỗng thay vì trả `null` —
        // để bên gửi so thẳng với cái mình vừa gửi.
        for (k, v) in [
            ("to", hd.contract_sheets),
            ("trang", hd.contract_pages),
            ("hoSo", hd.contract_dossiers),
            ("hop", hd.contract_boxes),
            ("tep", hd.contract_files),
        ] {
            if let Some(v) = v {
                m.insert(k.into(), json!(v));
            }
        }
        if let Some(v) = hd.contract_meters {
            m.insert("met".into(), json!(v));
        }
        if !m.is_empty() {
            khoi_luong_ra = Some(Value::Object(m));
        }
    }

    // 3. Quản trị dự án. Không có ai thì chỉ super admin nhìn thấy dự án — nói rõ ra.
    let mut xin = g.quan_tri.clone();
    xin.retain(|s| !s.trim().is_empty());
    let mut nguoi: Vec<(Uuid, String)> = if xin.is_empty() {
        Vec::new()
    } else {
        let tim = wh::tim_nguoi_de_gan(&st.pool, &xin).await?;
        for k in &xin {
            let co = tim.iter().any(|(_, u)| u.eq_ignore_ascii_case(k.trim()));
            if !co {
                canh_bao.push(format!(
                    "không tìm thấy tài khoản \"{}\" trong Hệ thống — chưa gán làm quản trị dự án",
                    k.trim()
                ));
            }
        }
        tim
    };
    if let Some(mac_dinh) = kh.default_admin_id {
        if !nguoi.iter().any(|(id, _)| *id == mac_dinh) {
            nguoi.push((mac_dinh, String::new()));
        }
    }
    let ids: Vec<Uuid> = nguoi.iter().map(|(id, _)| *id).collect();
    wh::gan_quan_tri(&st.pool, du_an.id, &ids).await?;
    // Quản trị mặc định của khoá chưa có tên — tra lại để phản hồi nói rõ đã gán ai.
    if nguoi.iter().any(|(_, u)| u.is_empty()) {
        let ten = wh::ten_nguoi(&st.pool, &ids).await?;
        for (id, u) in nguoi.iter_mut() {
            if u.is_empty() {
                if let Some((_, t)) = ten.iter().find(|(i, _)| i == id) {
                    *u = t.clone();
                }
            }
        }
    }
    if ids.is_empty() {
        canh_bao.push(
            "dự án chưa có người quản trị nên chỉ super admin nhìn thấy — gửi kèm quanTri, hoặc đặt quản trị mặc định cho khoá".into(),
        );
    }

    digitext_db::audit::append(
        &st.pool,
        digitext_db::audit::Tp {
            actor_id: None,
            device_id: None,
            action: if tao_moi {
                "project.create"
            } else {
                "project.update"
            },
            target: Some(format!("project:{}", du_an.id)),
            detail: Some(json!({
                "code": du_an.code, "nguon": kh.nguon, "maNgoai": ma_ngoai,
                "event": event, "qua": "webhook"
            })),
            ip: None,
        },
    )
    .await?;

    let kq = KetQua {
        project_id: du_an.id,
        ma_ngoai: ma_ngoai.to_string(),
        ma_du_an: du_an.code.clone(),
        ten_du_an: du_an.name.clone(),
        loai_du_an: du_an.loai.clone(),
        chu_dau_tu: du_an.chu_dau_tu.clone(),
        khoi_luong: khoi_luong_ra,
        tao_moi,
        ma_tu_sinh: du_an.ma_tu_sinh,
        quan_tri: nguoi
            .into_iter()
            .map(|(_, u)| u)
            .filter(|u| !u.is_empty())
            .collect(),
        canh_bao,
    };
    Ok((
        if tao_moi {
            StatusCode::CREATED
        } else {
            StatusCode::OK
        },
        kq,
    ))
}

async fn tao(
    st: &AppState,
    kh: &ApiClient,
    ma: &str,
    ten: &str,
    loai: Option<&str>,
    ma_ngoai: &str,
    ma_tu_sinh: bool,
    chu_dau_tu: Option<&str>,
) -> Result<wh::DuAn, ApiError> {
    wh::tao_du_an(
        &st.pool,
        wh::DuAnMoi {
            code: ma,
            name: ten,
            loai,
            nguon: &kh.nguon,
            ma_ngoai,
            ma_tu_sinh,
            chu_dau_tu,
        },
    )
    .await
    .map_err(|e| {
        if digitext_db::f_1jn(&e) {
            ApiError::conflict(format!("mã dự án {ma} đã tồn tại"))
        } else {
            e.into()
        }
    })
}

// ---------------------------------------------------------------- phụ trợ

/// Bên gửi thử khoá mà không tạo gì.
async fn kiem_tra(
    State(st): State<AppState>,
    ip: T2t,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Value>, ApiError> {
    let kh = xac_thuc(&st, &headers, &ip, &body).await?;
    Ok(Json(json!({
        "ok": true,
        "nguon": kh.nguon,
        "ten": kh.ten,
        "batBuocKyHmac": kh.secret_hmac.is_some_and(|s| !s.is_empty()),
        "gioMayChu": chrono::Utc::now().to_rfc3339(),
        "loaiDuAnChoPhep": LOAI.iter().map(|(m, _)| *m).collect::<Vec<_>>(),
        // Bên gửi dò được khoá khối lượng nào máy chủ nhận mà không phải mở tài liệu — thêm khoá
        // mới (0029 thêm "met") thì chỗ này tự nói, khỏi phải báo tay.
        "khoiLuongKhoa": ["to", "trang", "hoSo", "hop", "tep", "met"],
        "khoiLuongSoLe": ["met"],
        "coChuDauTu": true,
    })))
}

#[derive(Debug, Deserialize)]
struct LocNhatKy {
    #[serde(default)]
    nguon: Option<String>,
    #[serde(default)]
    limit: Option<i64>,
}

/// Nhật ký nhận — người quản trị xem trong app khi bên kia bảo "đã bắn rồi".
async fn nhat_ky(
    State(st): State<AppState>,
    ts: Ts,
    Query(q): Query<LocNhatKy>,
) -> Result<Json<Value>, ApiError> {
    ts.require_perm(digitext_core::Permission::ProjectUpdate)?;
    let rows = wh::danh_sach_delivery(&st.pool, q.nguon.as_deref(), q.limit.unwrap_or(100)).await?;
    Ok(Json(json!(rows
        .into_iter()
        .map(|r| json!({
            "id": r.id, "nguon": r.nguon, "event": r.event, "maNgoai": r.ma_ngoai,
            "status": r.status, "loi": r.loi, "projectId": r.project_id, "ip": r.ip,
            "nhanLuc": r.nhan_luc, "payload": r.payload, "ketQua": r.ket_qua,
        }))
        .collect::<Vec<_>>())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hmac_dung_theo_rfc_4231() {
        // RFC 4231 ca 2
        let ra = hmac_sha256(b"Jefe", b"what do ya want for nothing?");
        assert_eq!(
            hex(&ra),
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
    }

    #[test]
    fn hmac_khoa_dai_hon_mot_khoi() {
        // RFC 4231 ca 4 dùng khoá 131 byte 0xaa với dữ liệu "Test Using Larger Than Block-Size Key"
        let khoa = [0xaau8; 131];
        let ra = hmac_sha256(
            &khoa,
            b"Test Using Larger Than Block-Size Key - Hash Key First",
        );
        assert_eq!(
            hex(&ra),
            "60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54"
        );
    }

    #[test]
    fn so_sanh_khong_ro_thoi_gian() {
        assert!(bang_nhau(b"abc", b"abc"));
        assert!(!bang_nhau(b"abc", b"abd"));
        assert!(!bang_nhau(b"abc", b"abcd"));
    }

    /// "Chỉnh lý kết hợp số hoá" ĐÃ có mã riêng — bên gửi không phải nhồi nội dung công việc
    /// vào tenDuAn. Test này là chỗ bám khi ai đó định thêm giá trị thứ sáu.
    #[test]
    fn loai_du_an_co_ma_cho_ca_chinh_ly_lan_so_hoa() {
        assert_eq!(chuan_loai("chinh_ly_so_hoa"), Some("chinh_ly_so_hoa"));
        assert_eq!(chuan_loai("Chỉnh lý kết hợp số hoá"), Some("chinh_ly_so_hoa"));
        assert_eq!(chuan_loai("chỉnh lý số hoá"), Some("chinh_ly_so_hoa"));
        // Đừng để "chỉnh lý số hoá" rơi vào "chinh_ly" chỉ vì nó chứa chữ "chỉnh lý":
        // `chuan_loai` so BẰNG, không so CHỨA. Khớp sai thì dự án hai việc bị ghi thành một việc.
        assert_ne!(chuan_loai("chỉnh lý số hoá"), Some("chinh_ly"));
    }

    /// Mét là khoá khối lượng duy nhất nhận số lẻ, và là khoá duy nhất có thể mang NaN/vô cực
    /// lọt qua JSON. `numeric(12,2)` chỉ chứa 10 chữ số phần nguyên — quá thì CSDL nổ lúc ghi,
    /// phải chặn ở cửa chứ không để thành 500.
    #[test]
    fn met_chan_so_vo_ly() {
        let k = |m: f64| KhoiLuong { met: Some(m), ..Default::default() };
        assert!(!k(150.5).met_vo_ly());
        assert!(!k(0.0).met_vo_ly());
        assert!(k(f64::NAN).met_vo_ly());
        assert!(k(f64::INFINITY).met_vo_ly());
        assert!(k(1e10).met_vo_ly());
        assert!(k(-5.0).am(), "mét âm phải bị bắt bởi am()");
        assert!(k(150.5).co_gi(), "chỉ gửi mét cũng là có khối lượng");
        assert!(!KhoiLuong::default().co_gi());
    }

    /// Bốn khoá đếm vẫn là số nguyên: gửi 150.5 vào `soTrang` phải hỏng gói tin chứ không được
    /// làm tròn im lặng thành 150 hay 151.
    #[test]
    fn khoa_dem_khong_nhan_so_le() {
        let r: Result<KhoiLuong, _> = serde_json::from_str(r#"{"trang": 150.5}"#);
        assert!(r.is_err(), "soTrang phải là số nguyên");
        let ok: KhoiLuong = serde_json::from_str(r#"{"met": 150.5, "trang": 150}"#).unwrap();
        assert_eq!(ok.met, Some(150.5));
        assert_eq!(ok.trang, Some(150));
    }

    #[test]
    fn loai_du_an_nhan_nhieu_cach_viet() {
        assert_eq!(chuan_loai("chinh_ly"), Some("chinh_ly"));
        assert_eq!(chuan_loai("Chỉnh lý"), Some("chinh_ly"));
        assert_eq!(chuan_loai("SỐ HOÁ"), Some("so_hoa"));
        assert_eq!(chuan_loai("xyz"), None);
    }

    #[test]
    fn ma_du_an_chan_dau_cach_va_dau_cham() {
        assert_eq!(chuan_ma(" vyen ").unwrap(), "VYEN");
        assert!(chuan_ma("VY EN").is_err());
        assert!(chuan_ma("H05.13").is_err());
        assert!(chuan_ma("X").is_err());
    }

    #[test]
    fn ngay_nhan_hai_dinh_dang() {
        assert_eq!(
            doc_ngay("2027-03-31", "x").unwrap(),
            NaiveDate::from_ymd_opt(2027, 3, 31).unwrap()
        );
        assert_eq!(
            doc_ngay("31/03/2027", "x").unwrap(),
            NaiveDate::from_ymd_opt(2027, 3, 31).unwrap()
        );
        assert!(doc_ngay("31-3-27", "x").is_err());
    }
}
