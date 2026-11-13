// Copyright (c) 2026 DigiText. All rights reserved.
//! Hai truy vấn cho cửa **tự kiểm mục lục** của cổng nhà cung cấp
//! (`digitext-api/src/routes/ncc.rs`). Cả hai chỉ đọc, và chỉ trả số gộp — người gọi là người
//! ngoài, không được thấy tiêu đề hồ sơ hay tên người của dự án.

use sqlx::PgPool;
use uuid::Uuid;

/// Một dòng Thời hạn bảo quản trong Danh mục của dự án.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ThbqRow {
    pub ten: String,
    /// `cl_thbq.trang_thai = 1` — xem trigger ở migration 0015.
    pub vinh_vien: bool,
}

/// Danh mục THBQ dùng để khớp cột THBQ của tệp mục lục.
///
/// **Phải giống hệt `danh_muc::catalog`** — cùng điều kiện gộp (`project_id = $1 OR project_id
/// IS NULL`) và **cùng thứ tự** (`so_tt NULLS LAST, ten`). Máy trạm khớp THBQ bằng `cat.thbq`
/// lấy từ `GET /cl/catalog`, nên lệch điều kiện hay lệch thứ tự là bộ đọc ở máy chủ cho kết quả
/// khác bộ đọc ở máy trạm trên cùng một tệp. Khi có hai dòng cùng tên, `khop_thbq` lấy dòng
/// **đầu tiên** trong danh sách ⇒ thứ tự là một phần của hành vi, không phải chuyện thẩm mỹ.
///
/// Migration 0012 gieo sẵn 14 dòng dùng chung (`project_id IS NULL`), nên trên thực tế danh sách
/// này **không bao giờ rỗng** trừ khi ai đó đã xoá bộ gieo.
pub async fn thbq_cua_du_an(pool: &PgPool, project_id: Uuid) -> sqlx::Result<Vec<ThbqRow>> {
    sqlx::query_as(
        "SELECT ten, (trang_thai = 1) AS vinh_vien FROM cl_thbq \
         WHERE (project_id = $1 OR project_id IS NULL) AND deleted_at IS NULL \
         ORDER BY so_tt NULLS LAST, ten",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
}

/// Tên các phông đã mở trong dự án, để đối chiếu với dòng `Phông: …` ghi trong tệp mục lục.
///
/// Chỉ trả **tên** — cổng nhà cung cấp không cần `id`, và `id` phông là thứ dùng để gọi những
/// đường ghi dữ liệu mà nhà thầu không được phép đi tới.
///
/// Rỗng có hai nghĩa khác nhau mà người gọi phải phân biệt: dự án chưa mở phông nào (bình
/// thường, lúc mới khởi tạo), chứ không phải "không tìm thấy dự án" — dự án không tồn tại đã bị
/// chặn từ trước bằng `webhooks::tim_theo_ma_ngoai`.
pub async fn phong_cua_du_an(pool: &PgPool, project_id: Uuid) -> sqlx::Result<Vec<String>> {
    sqlx::query_scalar(
        "SELECT ten FROM cl_phong WHERE project_id = $1 AND deleted_at IS NULL ORDER BY ten",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
}

/// Số hồ sơ trong tệp bị trùng số tạm với hồ sơ ĐÃ có trên máy chủ, đếm theo từng phông.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TrungTheoPhong {
    pub phong_ten: String,
    pub so_trung: i64,
}

/// Đối chiếu `(vĩnh viễn, số tạm)` của tệp với `cl_ho_so` đang có trong dự án.
///
/// Khoá duy nhất thật của hệ thống là `ux_cl_ho_so_so_tam (phong_id, vinh_vien,
/// lower(btrim(so_tam)))` — **theo phông, không theo dự án**. Một dự án có thể nhiều phông, mà
/// cổng nhà cung cấp không biết sẽ nhập vào phông nào, nên đếm cho **từng phông** và để bên kia
/// bày ra; nhập vào phông nào thì nhìn đúng số của phông đó.
///
/// `so_tam` truyền vào phải đã `lower` + `trim` sẵn (giống biểu thức của chỉ mục).
pub async fn trung_so_tam_theo_phong(
    pool: &PgPool,
    project_id: Uuid,
    vinh_vien: &[bool],
    so_tam: &[String],
) -> sqlx::Result<Vec<TrungTheoPhong>> {
    if so_tam.is_empty() {
        return Ok(Vec::new());
    }
    sqlx::query_as(
        "SELECT p.ten AS phong_ten, count(DISTINCT (h.vinh_vien, lower(btrim(h.so_tam))))::bigint AS so_trung \
         FROM cl_phong p \
         JOIN cl_ho_so h ON h.phong_id = p.id AND h.deleted_at IS NULL \
         JOIN unnest($2::bool[], $3::text[]) AS k(vv, st) \
              ON k.vv = h.vinh_vien AND k.st = lower(btrim(h.so_tam)) \
         WHERE p.project_id = $1 AND p.deleted_at IS NULL \
         GROUP BY p.ten \
         ORDER BY so_trung DESC, p.ten",
    )
    .bind(project_id)
    .bind(vinh_vien)
    .bind(so_tam)
    .fetch_all(pool)
    .await
}
