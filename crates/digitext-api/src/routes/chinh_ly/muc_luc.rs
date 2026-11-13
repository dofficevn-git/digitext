// Copyright (c) 2026 DigiText. All rights reserved.
//! `POST /cl/muc-luc/doc` — đọc tệp mục lục hồ sơ (MLHS) **ở máy chủ** cho máy trạm.
//!
//! Vì sao có cửa này: bộ đọc MLHS trước đây chỉ có bản TypeScript trong desktop
//! (`desktop/src/features/chinhly/mucLucImport.ts`). Từ 10/11/2026 cổng nhà cung cấp của hệ
//! thống ngoài cũng cần đọc mục lục, nên bộ đọc đã chuyển sang Rust (`digitext-mucluc`). Hai bản
//! cùng tồn tại là **đúng căn bệnh "luật hai nơi"** mà dự án này đã chịu một lần (bộ quyền ở
//! `permission.rs` và `permissions.ts`): luật MLHS đã sửa 4 lần trong 5 tuần, bản nào cũ đi là
//! bản đó báo "đạt" cho tệp sắp bị từ chối.
//!
//! Cửa này là **chặng 1** của việc dẹp bản sao: máy trạm gọi sang đây thay vì tự đọc, rồi bản TS
//! bị xoá. Màn «Nhập mục lục» vốn **đã bắt buộc có mạng** (`NhapMucLucDialog.tsx` tắt nút khi
//! `offline`) nên chuyển lên máy chủ không mất khả năng nào.
//!
//! Khác cửa `/ncc/tu-kiem/muc-luc` (cổng nhà cung cấp) ở hai điểm:
//! * Xác thực bằng **phiên người dùng** (`Ts`) + quyền dự án, không bằng khoá webhook.
//! * Trả **đủ dữ liệu** để máy trạm vẽ lại cả màn và dựng lệnh nhập — gồm tiêu đề hồ sơ. Cửa
//!   nhà cung cấp cố tình KHÔNG trả tiêu đề; đây là người trong nhà nên trả.

use super::project_access;
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::Ts;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use digitext_core::Permission;
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

/// Cùng trần với cửa nhà cung cấp. Mục lục lớn nhất trong 23 tệp thật là 504 KiB.
const BYTE_TOI_DA: usize = 8 * 1024 * 1024;

pub fn router() -> Router<AppState> {
    Router::new().route(
        "/cl/muc-luc/doc",
        post(doc_muc_luc).layer(axum::extract::DefaultBodyLimit::max(12 * 1024 * 1024)),
    )
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DocReq {
    project_id: Uuid,
    #[serde(default)]
    ten_tep: Option<String>,
    /// Nội dung tệp `.xlsx` / `.xls` / `.xlsm`, mã hoá base64.
    noi_dung: String,
    /// Sheet muốn đọc; không có → sheet đầu tiên tìm được dòng tiêu đề cột.
    #[serde(default)]
    sheet: Option<String>,
    /// Tên THBQ dùng cho ô THBQ trống; không có → ô trống là lỗi dòng.
    #[serde(default)]
    thbq_mac_dinh: Option<String>,
}

async fn doc_muc_luc(State(st): State<AppState>, ts: Ts, Json(b): Json<DocReq>) -> Result<Json<Value>, ApiError> {
    project_access(&st, &ts, b.project_id).await?;
    // `project_access` cho super admin đi qua với BẤT KỲ uuid, kể cả dự án không tồn tại — đo
    // được 10/11/2026. Không kiểm thì gõ sai một ký tự là đọc xong cả tệp rồi mới vỡ ở lệnh nhập.
    let co: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM projects WHERE id = $1)")
        .bind(b.project_id)
        .fetch_one(&st.pool)
        .await?;
    if !co {
        return Err(ApiError::not_found());
    }
    // Đọc tệp là bước chuẩn bị của lệnh nhập, nên đòi đúng quyền của lệnh nhập.
    ts.require_perm(Permission::DossierCreate)?;

    let tep: Vec<u8> = {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD
            .decode(b.noi_dung.trim())
            .map_err(|_| ApiError::bad_request("noiDung không phải base64 hợp lệ"))?
    };
    if tep.is_empty() {
        return Err(ApiError::bad_request("tệp rỗng"));
    }
    if tep.len() > BYTE_TOI_DA {
        return Err(ApiError::bad_request(format!(
            "tệp {} MiB vượt trần {} MiB",
            tep.len() / 1024 / 1024,
            BYTE_TOI_DA / 1024 / 1024
        )));
    }

    // Danh mục THBQ của chính dự án — bộ đọc cần nó để khớp cột THBQ và biết loại bảo quản.
    let dm = digitext_db::chinh_ly::ncc::thbq_cua_du_an(&st.pool, b.project_id).await?;
    // Xem chú thích cùng nhánh ở `routes::ncc`: 0012 gieo 14 dòng dùng chung nên đây là chốt
    // an toàn, không phải hướng dẫn cho người dùng.
    if dm.is_empty() {
        return Err(ApiError::bad_request(
            "máy chủ không có Danh mục thời hạn bảo quản nào — bộ gieo danh mục (migration 0012) đã bị xoá",
        ));
    }
    let ten_dm: Vec<String> = dm.iter().map(|t| t.ten.clone()).collect();
    let ten_vv: Vec<String> = dm.iter().filter(|t| t.vinh_vien).map(|t| t.ten.clone()).collect();

    let ten_tep = b.ten_tep.as_deref().unwrap_or("muc-luc.xlsx").trim().to_owned();
    let sheet = b.sheet.as_deref().map(str::trim).filter(|s| !s.is_empty()).map(str::to_owned);
    let thbq_md = b.thbq_mac_dinh.clone();
    // Tệp 2.800 dòng là việc nặng CPU ⇒ luồng chặn, đừng chiếm worker Tokio.
    let pt = tokio::task::spawn_blocking(move || {
        let so = digitext_mucluc::doc_so_tu_bytes(&tep, sheet.as_deref())?;
        if let Some(xin) = sheet.as_deref() {
            if so.sheet != xin {
                return Err(format!("tệp không có sheet {xin:?} — các sheet có trong tệp: {}", so.sheets.join(", ")));
            }
        }
        Ok::<_, String>(digitext_mucluc::phan_tich::phan_tich_muc_luc(
            &so.rows,
            &digitext_mucluc::phan_tich::TuyChon {
                ten_tep: &ten_tep,
                sheet: &so.sheet,
                sheets: &so.sheets,
                thbq_danh_muc: &ten_dm,
                thbq_vinh_vien: &ten_vv,
                thbq_mac_dinh: thbq_md.as_deref(),
            },
        ))
    })
    .await
    .map_err(|_| ApiError::internal())?
    .map_err(ApiError::bad_request)?;

    // `rowsNhap` xếp đúng thứ tự `rows` — máy trạm gửi thẳng sang `/cl/ho-so/import`, và dùng
    // `rows[i].dong` để dịch số dòng lỗi mà lệnh nhập trả về sang số dòng trong tệp Excel.
    let rows_nhap: Vec<Value> = pt
        .rows
        .iter()
        .map(|r| Value::Object(digitext_mucluc::phan_tich::dong_nhap(&pt, r)))
        .collect();

    Ok(Json(json!({
        "phanTich": pt,
        "rowsNhap": rows_nhap,
        "coLoiChan": digitext_mucluc::phan_tich::co_loi_chan(&pt),
    })))
}
