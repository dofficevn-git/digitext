// Copyright (c) 2026 DigiText. All rights reserved.
//! Test CSDL cho chỉnh lý: chạy khi có DIGITEXT_TEST_DATABASE_URL (đã áp migrations/0012). Mỗi test
//! tạo dự án + người dùng riêng rồi xoá (cascade) ở cuối.

use super::merge::Row;
use super::sync::{self, OpCtx, OpKind, OpResult, SyncOp};
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

async fn pool() -> Option<PgPool> {
    let url = std::env::var("DIGITEXT_TEST_DATABASE_URL").ok()?;
    Some(crate::connect(&url).await.expect("kết nối CSDL thử"))
}

struct Fx {
    project: Uuid,
    user: Uuid,
    phong: Uuid,
}

async fn fixture(pool: &PgPool) -> Fx {
    let project = Uuid::now_v7();
    let user = Uuid::now_v7();
    sqlx::query("INSERT INTO projects (id, code, name) VALUES ($1, $2, 'test chinh_ly')")
        .bind(project)
        .bind(format!("CL{}", &project.simple().to_string()[24..])) // đuôi ngẫu nhiên (đầu uuid v7 là thời gian, test chạy song song sẽ trùng)
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO users (id, username, display_name, pwd_hash, role, project_id) VALUES ($1, $2, 'Người thử', 'x', 'admin', $3)")
        .bind(user)
        .bind(format!("u{}", &user.simple().to_string()[20..]))
        .bind(project)
        .execute(pool)
        .await
        .unwrap();
    let phong = Uuid::now_v7();
    sqlx::query("INSERT INTO cl_phong (id, project_id, ma_du_an, ten, phong_so) VALUES ($1, $2, 'UBND', 'Phông thử', '01')")
        .bind(phong)
        .bind(project)
        .execute(pool)
        .await
        .unwrap();
    Fx { project, user, phong }
}

async fn cleanup(pool: &PgPool, fx: &Fx) {
    sqlx::query("DELETE FROM users WHERE id = $1").bind(fx.user).execute(pool).await.unwrap();
    sqlx::query("DELETE FROM projects WHERE id = $1").bind(fx.project).execute(pool).await.unwrap();
}

fn all(_: digitext_core::Permission) -> bool {
    true
}
fn none(_: digitext_core::Permission) -> bool {
    false
}

fn ctx<'a>(fx: &Fx, source: &'a str, perm: &'a (dyn Fn(digitext_core::Permission) -> bool + Sync)) -> OpCtx<'a> {
    OpCtx { project_id: fx.project, user_id: fx.user, user_name: "Người thử", device_id: None, source, has_perm: perm, is_super: false }
}

fn row(v: Value) -> Row {
    v.as_object().unwrap().clone()
}

fn upsert(table: &str, id: Uuid, new: Value) -> SyncOp {
    SyncOp { op_id: None, table: table.into(), id, op: OpKind::Upsert, base: None, new: row(new), if_ver: None, edited_at: None }
}

fn row_of(r: &OpResult) -> Row {
    match r {
        OpResult::Applied { row: Some(r), .. } | OpResult::Merged { row: r, .. } | OpResult::NoChange { row: r, .. } => r.clone(),
        other => panic!("không có dòng: {other:?}"),
    }
}

#[tokio::test]
async fn so_tam_trung_bi_tu_choi_va_lich_su_dem_truong() {
    let Some(pool) = pool().await else { return };
    let fx = fixture(&pool).await;
    let c = ctx(&fx, "online", &all);
    let id = Uuid::now_v7();
    let r = sync::apply_op(&pool, &c, &upsert("cl_ho_so", id, json!({"phong_id": fx.phong, "so_tam": "12", "tieu_de": "Hồ sơ A", "ngay_ket_thuc": "3/2021"}))).await.unwrap();
    let first = row_of(&r);
    assert_eq!(first["ngay_ket_thuc"], json!("03/2021"));
    assert_eq!(first["nam_ket_thuc"], json!(2021));
    // trùng số tạm khác hoa thường
    let dup = sync::apply_op(&pool, &c, &upsert("cl_ho_so", Uuid::now_v7(), json!({"phong_id": fx.phong, "so_tam": " 12", "tieu_de": "B"}))).await.unwrap();
    assert!(matches!(&dup, OpResult::Rejected { reason, .. } if reason == "unique:ux_cl_ho_so_so_tam"), "{dup:?}");
    // sửa 2 trường: 1 mới (so_to), 1 sửa (tieu_de)
    let r2 = sync::apply_op(&pool, &c, &upsert("cl_ho_so", id, json!({"tieu_de": "Hồ sơ A2", "so_to": "9"}))).await.unwrap();
    assert!(matches!(r2, OpResult::Applied { .. }));
    let hist: Vec<(String, i32, i32)> = sqlx::query_as("SELECT action, n_new, n_edited FROM cl_lich_su WHERE ho_so_id = $1 ORDER BY at").bind(id).fetch_all(&pool).await.unwrap();
    assert_eq!(hist.len(), 2);
    assert_eq!(hist[0].0, "add");
    assert!(hist[0].1 >= 4);
    assert_eq!((hist[1].0.as_str(), hist[1].1, hist[1].2), ("edit", 1, 1));
    // không đổi → NoChange, không thêm lịch sử
    let r3 = sync::apply_op(&pool, &c, &upsert("cl_ho_so", id, json!({"tieu_de": " Hồ sơ A2 "}))).await.unwrap();
    assert!(matches!(r3, OpResult::NoChange { .. }));
    // không có quyền → forbidden
    let cn = ctx(&fx, "online", &none);
    let r4 = sync::apply_op(&pool, &cn, &upsert("cl_ho_so", id, json!({"tieu_de": "X"}))).await.unwrap();
    assert!(matches!(r4, OpResult::Rejected { reason, .. } if reason == "forbidden"));
    cleanup(&pool, &fx).await;
}

#[tokio::test]
async fn push_gop_xung_dot_va_idempotent() {
    let Some(pool) = pool().await else { return };
    let fx = fixture(&pool).await;
    let c = ctx(&fx, "online", &all);
    let id = Uuid::now_v7();
    let base = row_of(&sync::apply_op(&pool, &c, &upsert("cl_ho_so", id, json!({"phong_id": fx.phong, "so_tam": "1", "tieu_de": "A", "so_to": "1"}))).await.unwrap());
    // B sửa online tiêu đề
    sync::apply_op(&pool, &c, &upsert("cl_ho_so", id, json!({"tieu_de": "B"}))).await.unwrap();
    // A push offline đổi so_to (khác trường) → merged
    let cs = ctx(&fx, "sync", &all);
    let op_id = Uuid::now_v7();
    let op = SyncOp { op_id: Some(op_id), table: "cl_ho_so".into(), id, op: OpKind::Upsert, base: Some(base.clone()), new: row(json!({"so_to": "7"})), if_ver: None, edited_at: None };
    let r = sync::apply_op(&pool, &cs, &op).await.unwrap();
    match &r {
        OpResult::Merged { row, merged_fields, .. } => {
            assert_eq!(merged_fields, &vec!["so_to".to_string()]);
            assert_eq!(row["tieu_de"], json!("B"));
            assert_eq!(row["so_to"], json!("7"));
        }
        other => panic!("{other:?}"),
    }
    // gửi lại cùng op_id → duplicate
    let again = sync::apply_op(&pool, &cs, &op).await.unwrap();
    assert!(matches!(again, OpResult::Duplicate { .. }));
    // A push đổi tiêu đề (cùng trường B đã đổi) → conflict kèm dòng máy chủ
    let op2 = SyncOp { op_id: None, table: "cl_ho_so".into(), id, op: OpKind::Upsert, base: Some(base.clone()), new: row(json!({"tieu_de": "C"})), if_ver: None, edited_at: None };
    match sync::apply_op(&pool, &cs, &op2).await.unwrap() {
        OpResult::Conflict { row: Some(row), conflict_fields } => {
            assert_eq!(conflict_fields, vec!["tieu_de".to_string()]);
            assert_eq!(row["tieu_de"], json!("B"));
        }
        other => panic!("{other:?}"),
    }
    // cột chỉ online bị từ chối khi push
    let op3 = SyncOp { op_id: None, table: "cl_ho_so".into(), id, op: OpKind::Upsert, base: Some(base.clone()), new: row(json!({"ho_so_so": "5"})), if_ver: None, edited_at: None };
    assert!(matches!(sync::apply_op(&pool, &cs, &op3).await.unwrap(), OpResult::Rejected { reason, .. } if reason == "online_only_field"));
    // bảng chỉ online bị từ chối khi push
    let op4 = upsert("cl_kho", Uuid::now_v7(), json!({"ten": "Kho 1"}));
    assert!(matches!(sync::apply_op(&pool, &cs, &op4).await.unwrap(), OpResult::Rejected { reason, .. } if reason == "online_only_table"));
    // if_ver lệch ở đường online → conflict
    let op5 = SyncOp { op_id: None, table: "cl_ho_so".into(), id, op: OpKind::Upsert, base: None, new: row(json!({"tieu_de": "D"})), if_ver: Some(1), edited_at: None };
    assert!(matches!(sync::apply_op(&pool, &c, &op5).await.unwrap(), OpResult::Conflict { .. }));
    cleanup(&pool, &fx).await;
}

#[tokio::test]
async fn pull_theo_ver_tombstone_va_xoa_khoi_phuc() {
    let Some(pool) = pool().await else { return };
    let fx = fixture(&pool).await;
    let c = ctx(&fx, "online", &all);
    let hs = Uuid::now_v7();
    sync::apply_op(&pool, &c, &upsert("cl_ho_so", hs, json!({"phong_id": fx.phong, "so_tam": "5", "tieu_de": "A"}))).await.unwrap();
    let vb = Uuid::now_v7();
    sync::apply_op(&pool, &c, &upsert("cl_van_ban", vb, json!({"ho_so_id": hs, "so_thu_tu": 1, "trich_yeu": "VB 1", "ngay_thang": "5/3/2020"}))).await.unwrap();
    // STT trùng
    let d = sync::apply_op(&pool, &c, &upsert("cl_van_ban", Uuid::now_v7(), json!({"ho_so_id": hs, "so_thu_tu": 1, "trich_yeu": "VB 1b"}))).await.unwrap();
    assert!(matches!(d, OpResult::Rejected { reason, .. } if reason == "unique:ux_cl_van_ban_stt"));
    let p0 = sync::pull(&pool, fx.project, 0, 1000).await.unwrap();
    let tables: Vec<&str> = p0.rows.iter().map(|r| r.table.as_str()).collect();
    assert!(tables.contains(&"cl_phong") && tables.contains(&"cl_ho_so") && tables.contains(&"cl_van_ban") && tables.contains(&"cl_lich_su"));
    assert!(tables.contains(&"cl_thbq"), "bảng mã dùng chung phải được kéo về");
    assert!(!p0.has_more && !p0.full);
    let cursor = p0.next;
    // phân trang
    let p1 = sync::pull(&pool, fx.project, 0, 2).await.unwrap();
    assert_eq!(p1.rows.len(), 2);
    assert!(p1.has_more);
    // xoá hồ sơ → cascade văn bản (mềm), tombstone xuất hiện khi since > 0
    let del = SyncOp { op_id: None, table: "cl_ho_so".into(), id: hs, op: OpKind::Delete, base: None, new: Row::new(), if_ver: None, edited_at: None };
    assert!(matches!(sync::apply_op(&pool, &c, &del).await.unwrap(), OpResult::Applied { .. }));
    let p2 = sync::pull(&pool, fx.project, cursor, 1000).await.unwrap();
    let dead: Vec<&sync::PullRow> = p2.rows.iter().filter(|r| r.table != "cl_lich_su" && !r.row["deleted_at"].is_null()).collect();
    assert_eq!(dead.len(), 2, "{:?}", p2.rows);
    // sửa dòng đã xoá → deleted_on_server
    let e = sync::apply_op(&pool, &c, &upsert("cl_ho_so", hs, json!({"tieu_de": "Z"}))).await.unwrap();
    assert!(matches!(e, OpResult::Rejected { reason, .. } if reason == "deleted_on_server"));
    // khôi phục cả cây
    let mut conn = pool.acquire().await.unwrap();
    let n = super::rows::restore(&mut conn, super::spec::spec("cl_ho_so").unwrap(), hs, Some(fx.user)).await.unwrap();
    assert_eq!(n, 2);
    let (rows, total) = super::ho_so::list(&pool, &super::ho_so::Filter { project_id: fx.project, limit: 10, ..Default::default() }).await.unwrap();
    assert_eq!(total, 1);
    assert_eq!(rows[0]["so_van_ban"], json!(1));
    // dọn tombstone: lùi deleted_at 100 ngày rồi prune → pull với since cũ báo full
    sync::apply_op(&pool, &c, &del).await.unwrap();
    sqlx::query("UPDATE cl_van_ban SET deleted_at = now() - interval '100 days' WHERE id = $1").bind(vb).execute(&pool).await.unwrap();
    sqlx::query("UPDATE cl_ho_so SET deleted_at = now() - interval '100 days' WHERE id = $1").bind(hs).execute(&pool).await.unwrap();
    let pruned = sync::prune(&pool, 90).await.unwrap();
    assert!(pruned >= 2);
    let p3 = sync::pull(&pool, fx.project, 1, 10).await.unwrap();
    assert!(p3.full);
    cleanup(&pool, &fx).await;
}

#[tokio::test]
async fn he_thong_hoa_danh_so_va_gan_hop() {
    use super::he_thong_hoa as hth;
    use digitext_core::chinh_ly::SortOrder;
    let Some(pool) = pool().await else { return };
    let fx = fixture(&pool).await;
    let c = ctx(&fx, "online", &all);
    let pb = Uuid::now_v7();
    sync::apply_op(&pool, &c, &upsert("cl_phong_ban", pb, json!({"phong_id": fx.phong, "ma_don_vi": "VP", "ten": "Văn phòng"}))).await.unwrap();
    let thbq_vv: Uuid = sqlx::query_scalar("SELECT id FROM cl_thbq WHERE project_id IS NULL AND trang_thai = 1 LIMIT 1").fetch_one(&pool).await.unwrap();
    let ids: Vec<Uuid> = (0..5).map(|_| Uuid::now_v7()).collect();
    for (k, id) in ids.iter().enumerate() {
        // số tạm 5,4,3,2,1 với ngày KT tăng dần → thứ tự theo ngày KT khác thứ tự số tạm
        sync::apply_op(&pool, &c, &upsert("cl_ho_so", *id, json!({"phong_id": fx.phong, "phong_ban_id": pb, "thbq_id": thbq_vv, "so_tam": (5 - k).to_string(), "tieu_de": format!("HS {k}"), "ngay_ket_thuc": (2000 + k as i32).to_string()})))
            .await
            .unwrap();
    }
    let p = hth::preview(&pool, fx.project, fx.phong, &hth::Filter::default(), SortOrder::ThoiGianMang, 1, 1).await.unwrap();
    assert_eq!(p.rows.len(), 5);
    assert_eq!(p.rows[0].so_tam, "5"); // ngày KT 2000 nhỏ nhất
    assert_eq!(p.rows[0].stt, 1);
    assert_eq!(p.rows[4].stt, 5);
    assert_eq!(p.rows[0].ma_ho_so_moi.as_deref(), Some("UBND.01.2000.1"));
    // snapshot cũ → Stale
    let stale = hth::apply(&pool, &c, fx.phong, &hth::Filter::default(), SortOrder::ThoiGianMang, 1, 1, p.snapshot_ver - 1).await.unwrap();
    assert!(matches!(stale, Err(hth::ApplyErr::Stale)));
    let ok = hth::apply(&pool, &c, fx.phong, &hth::Filter::default(), SortOrder::ThoiGianMang, 1, 1, p.snapshot_ver).await.unwrap().ok().unwrap();
    assert_eq!(ok.rows.iter().map(|r| r.ho_so_so_cu.clone().unwrap()).collect::<Vec<_>>(), vec!["1", "2", "3", "4", "5"]);
    let ls: i64 = sqlx::query_scalar("SELECT count(*) FROM cl_lich_su WHERE phong_id = $1 AND action = 'systematize'").bind(fx.phong).fetch_one(&pool).await.unwrap();
    assert_eq!(ls, 5);
    // gán hộp: 3 hồ sơ đầu hộp "1", 2 hồ sơ sau hộp "2"; 1 dòng lỗi (hồ sơ lạ)
    let rows: Vec<hth::BoxRow> = ok.rows.iter().enumerate().map(|(k, r)| hth::BoxRow { ho_so_id: r.ho_so_id, hop_so: Some(if k < 3 { "1" } else { "2" }.into()), bo_so: None, tap_so: None, clear: false }).collect();
    let mut rows = rows;
    rows.push(hth::BoxRow { ho_so_id: Uuid::now_v7(), hop_so: Some("9".into()), bo_so: None, tap_so: None, clear: false });
    let b = hth::boxing(&pool, &c, 1, &rows).await.unwrap();
    assert_eq!((b.applied, b.hops_created, b.errors.len()), (5, 2, 1));
    let (tu, den, vv): (Option<String>, Option<String>, bool) = sqlx::query_as("SELECT tu_ho_so, den_ho_so, vinh_vien FROM cl_hop WHERE phong_ban_id = $1 AND hop_so = '1'").bind(pb).fetch_one(&pool).await.unwrap();
    assert_eq!((tu.as_deref(), den.as_deref(), vv), (Some("1"), Some("3"), true));
    // giá suy ra từ dải hộp
    let kho = Uuid::now_v7();
    sync::apply_op(&pool, &c, &upsert("cl_kho", kho, json!({"ten": "Kho A"}))).await.unwrap();
    let gia = Uuid::now_v7();
    sync::apply_op(&pool, &c, &upsert("cl_gia", gia, json!({"kho_id": kho, "phong_ban_id": pb, "ten": "Giá 1", "tu_hop": 1, "den_hop": 1}))).await.unwrap();
    let (rows, _) = super::ho_so::list(&pool, &super::ho_so::Filter { project_id: fx.project, gia_id: Some(gia), limit: 10, ..Default::default() }).await.unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0]["ten_kho"], json!("Kho A"));
    // bó / tập cho hồ sơ hết hạn
    let b2 = hth::boxing(&pool, &c, 2, &[hth::BoxRow { ho_so_id: ids[0], hop_so: None, bo_so: Some(7), tap_so: Some(2), clear: false }]).await.unwrap();
    assert_eq!((b2.applied, b2.bo_tap_created), (1, 2));
    cleanup(&pool, &fx).await;
}

#[tokio::test]
async fn danh_muc_khong_xoa_khi_con_dung() {
    let Some(pool) = pool().await else { return };
    let fx = fixture(&pool).await;
    let c = ctx(&fx, "online", &all);
    let pb = Uuid::now_v7();
    assert!(matches!(sync::apply_op(&pool, &c, &upsert("cl_phong_ban", pb, json!({"phong_id": fx.phong, "ma_don_vi": "VP", "ten": "Văn phòng"}))).await.unwrap(), OpResult::Applied { .. }));
    let mang = Uuid::now_v7();
    let m = sync::apply_op(&pool, &c, &upsert("cl_mang", mang, json!({"phong_ban_id": pb, "ten": "Tổng hợp"}))).await.unwrap();
    assert_eq!(row_of(&m)["ma_mang"], json!("00"));
    let hs = Uuid::now_v7();
    sync::apply_op(&pool, &c, &upsert("cl_ho_so", hs, json!({"phong_id": fx.phong, "phong_ban_id": pb, "mang_id": mang, "so_tam": "1", "tieu_de": "A", "thbq": "bỏ qua"}))).await.map(|r| assert!(matches!(r, OpResult::Rejected { reason, .. } if reason == "unknown_field"))).unwrap();
    sync::apply_op(&pool, &c, &upsert("cl_ho_so", hs, json!({"phong_id": fx.phong, "phong_ban_id": pb, "mang_id": mang, "so_tam": "1", "tieu_de": "A"}))).await.unwrap();
    let del = SyncOp { op_id: None, table: "cl_mang".into(), id: mang, op: OpKind::Delete, base: None, new: Row::new(), if_ver: None, edited_at: None };
    assert!(matches!(sync::apply_op(&pool, &c, &del).await.unwrap(), OpResult::Rejected { reason, .. } if reason == "in_use"));
    // cha thuộc dự án khác → forbidden
    let other = fixture(&pool).await;
    let bad = sync::apply_op(&pool, &c, &upsert("cl_phong_ban", Uuid::now_v7(), json!({"phong_id": other.phong, "ten": "X"}))).await.unwrap();
    assert!(matches!(bad, OpResult::Rejected { reason, .. } if reason == "forbidden"));
    let cat = super::danh_muc::catalog(&pool, fx.project).await.unwrap();
    assert_eq!(cat["phong_ban"].as_array().unwrap().len(), 1);
    assert_eq!(cat["thbq"].as_array().unwrap().len(), 14);
    cleanup(&pool, &other).await;
    cleanup(&pool, &fx).await;
}
