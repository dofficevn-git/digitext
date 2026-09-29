// Copyright (c) 2026 DigiText. All rights reserved.
//! Lịch sử sửa hồ sơ / văn bản theo từng trường (cl_lich_su). Máy chủ tính diff — cả khi ghi
//! online lẫn khi áp op đồng bộ — nên số "trường thêm mới / trường sửa" (nguồn thống kê nhân viên)
//! luôn tính từ dữ liệu thật.

use super::merge::{norm, Row};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{PgConnection, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Diff {
    pub f: String,
    pub o: Value,
    pub n: Value,
}

/// So `before` (None = thêm mới) với `after` trên `fields`; trả (diffs, n_new, n_edited).
pub fn diff_rows(before: Option<&Row>, after: &Row, fields: &[&str]) -> (Vec<Diff>, i32, i32) {
    let mut diffs = Vec::new();
    let (mut n_new, mut n_edited) = (0, 0);
    for f in fields {
        let o = norm(before.and_then(|b| b.get(*f)));
        let n = norm(after.get(*f));
        if o == n {
            continue;
        }
        if o.is_null() {
            n_new += 1;
        } else {
            n_edited += 1;
        }
        diffs.push(Diff { f: (*f).to_owned(), o, n });
    }
    (diffs, n_new, n_edited)
}

#[derive(Debug, Clone)]
pub struct NewEntry<'a> {
    pub project_id: Uuid,
    pub entity: &'a str,
    pub entity_id: Uuid,
    pub ho_so_id: Uuid,
    pub phong_id: Option<Uuid>,
    pub action: &'a str,
    pub actor_id: Option<Uuid>,
    pub actor_name: Option<&'a str>,
    pub at_client: Option<DateTime<Utc>>,
    pub source: &'a str,
    pub title: Option<&'a str>,
    pub diffs: Vec<Diff>,
    pub n_new: i32,
    pub n_edited: i32,
}

pub async fn insert(conn: &mut PgConnection, e: NewEntry<'_>) -> sqlx::Result<()> {
    sqlx::query(
        "INSERT INTO cl_lich_su (id, project_id, entity, entity_id, ho_so_id, phong_id, action, actor_id, actor_name, at_client, source, title, diffs, n_new, n_edited) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)",
    )
    .bind(Uuid::now_v7())
    .bind(e.project_id)
    .bind(e.entity)
    .bind(e.entity_id)
    .bind(e.ho_so_id)
    .bind(e.phong_id)
    .bind(e.action)
    .bind(e.actor_id)
    .bind(e.actor_name)
    .bind(e.at_client)
    .bind(e.source)
    .bind(e.title)
    .bind(serde_json::to_value(&e.diffs).unwrap_or(Value::Array(vec![])))
    .bind(e.n_new)
    .bind(e.n_edited)
    .execute(&mut *conn)
    .await?;
    Ok(())
}

#[derive(Debug, Default, Clone)]
pub struct Filter {
    pub project_id: Uuid,
    pub entity: Option<String>,
    pub ho_so_id: Option<Uuid>,
    pub phong_id: Option<Uuid>,
    pub actor_id: Option<Uuid>,
    pub action: Option<String>,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub q: Option<String>,
    pub limit: i64,
    pub offset: i64,
}

/// Dòng lịch sử kèm số tạm / tiêu đề hồ sơ hiện tại.
pub async fn list(pool: &PgPool, f: &Filter) -> sqlx::Result<(Vec<Value>, i64)> {
    let where_sql = "l.project_id = $1 \
        AND ($2::text IS NULL OR l.entity = $2) AND ($3::uuid IS NULL OR l.ho_so_id = $3) \
        AND ($4::uuid IS NULL OR l.phong_id = $4) AND ($5::uuid IS NULL OR l.actor_id = $5) \
        AND ($6::text IS NULL OR l.action = $6) AND ($7::timestamptz IS NULL OR l.at >= $7) \
        AND ($8::timestamptz IS NULL OR l.at < $8) \
        AND ($9::text IS NULL OR l.title ILIKE '%' || $9 || '%' OR h.so_tam ILIKE '%' || $9 || '%' OR l.actor_name ILIKE '%' || $9 || '%')";
    let sql = format!(
        "SELECT to_jsonb(l) || jsonb_build_object('so_tam', h.so_tam, 'tieu_de_hien_tai', h.tieu_de, 'ho_so_so', h.ho_so_so) AS row \
         FROM cl_lich_su l LEFT JOIN cl_ho_so h ON h.id = l.ho_so_id WHERE {where_sql} ORDER BY l.at DESC, l.ver DESC LIMIT $10 OFFSET $11"
    );
    let count_sql = format!("SELECT count(*) FROM cl_lich_su l LEFT JOIN cl_ho_so h ON h.id = l.ho_so_id WHERE {where_sql}");
    fn bind<'q>(q: sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments>, f: &'q Filter) -> sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments> {
        q.bind(f.project_id)
            .bind(f.entity.as_deref())
            .bind(f.ho_so_id)
            .bind(f.phong_id)
            .bind(f.actor_id)
            .bind(f.action.as_deref())
            .bind(f.from)
            .bind(f.to)
            .bind(f.q.as_deref().map(str::trim).filter(|s| !s.is_empty()))
    }
    use sqlx::Row as _;
    let rows = bind(sqlx::query(&sql), f).bind(f.limit.clamp(1, 100_000)).bind(f.offset.max(0)).fetch_all(pool).await?;
    let total: i64 = bind(sqlx::query(&count_sql), f).fetch_one(pool).await?.try_get(0)?;
    Ok((rows.into_iter().filter_map(|r| r.try_get::<Value, _>("row").ok()).collect(), total))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn dem_truong_moi_va_sua() {
        let before = json!({"tieu_de": "A", "so_to": null, "ghi_chu": "x"}).as_object().unwrap().clone();
        let after = json!({"tieu_de": "B", "so_to": "5", "ghi_chu": " x "}).as_object().unwrap().clone();
        let (d, n_new, n_edited) = diff_rows(Some(&before), &after, &["tieu_de", "so_to", "ghi_chu"]);
        assert_eq!((n_new, n_edited), (1, 1));
        assert_eq!(d.len(), 2);
        assert_eq!(d[0], Diff { f: "tieu_de".into(), o: json!("A"), n: json!("B") });
        let (d2, n2, e2) = diff_rows(None, &after, &["tieu_de", "so_to", "ghi_chu", "tu_khoa"]);
        assert_eq!((d2.len(), n2, e2), (3, 3, 0));
    }
}
