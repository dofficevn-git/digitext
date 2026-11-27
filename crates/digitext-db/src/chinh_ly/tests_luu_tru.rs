// Copyright (c) 2026 DigiText. All rights reserved.
//! Test CSDL cho lưu trữ hiện hành (0031): khoá sửa hồ sơ đã nộp lưu, phạm vi phòng ban, luồng phiếu nộp lưu, danh mục hồ sơ.
//! Chạy khi có DIGITEXT_TEST_DATABASE_URL (đã áp migrations tới 0031). Mỗi test tạo dự án riêng rồi xoá (cascade).

use super::merge::Row;
use super::sync::{self, OpCtx, OpKind, OpResult, SyncOp};
use super::ho_so::{self, PhamVi};
use crate::{danh_muc_ho_so as dm, nop_luu};
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
    user2: Uuid,
    phong: Uuid,
    pb_a: Uuid,
    pb_b: Uuid,
}

async fn fixture(pool: &PgPool) -> Fx {
    let project = Uuid::now_v7();
    sqlx::query("INSERT INTO projects (id, code, name) VALUES ($1, $2, 'test luu tru')")
        .bind(project)
        .bind(format!("LT{}", &project.simple().to_string()[24..]))
        .execute(pool)
        .await
        .unwrap();
    let phong = Uuid::now_v7();
    sqlx::query("INSERT INTO cl_phong (id, project_id, ma_du_an, ten, phong_so) VALUES ($1, $2, 'UBND', 'Phông thử', '01')").bind(phong).bind(project).execute(pool).await.unwrap();
    let (pb_a, pb_b) = (Uuid::now_v7(), Uuid::now_v7());
    for (id, ma, ten) in [(pb_a, "VP", "Văn phòng"), (pb_b, "NV", "Phòng Nội vụ")] {
        sqlx::query("INSERT INTO cl_phong_ban (id, project_id, phong_id, ma_don_vi, ten) VALUES ($1, $2, $3, $4, $5)").bind(id).bind(project).bind(phong).bind(ma).bind(ten).execute(pool).await.unwrap();
    }
    let (user, user2) = (Uuid::now_v7(), Uuid::now_v7());
    for (id, pb) in [(user, pb_a), (user2, pb_b)] {
        sqlx::query("INSERT INTO users (id, username, display_name, pwd_hash, role, project_id, phong_ban_id) VALUES ($1, $2, 'Người thử', 'x', 'user', $3, $4)")
            .bind(id)
            .bind(format!("lt{}", &id.simple().to_string()[20..]))
            .bind(project)
            .bind(pb)
            .execute(pool)
            .await
            .unwrap();
    }
    Fx { project, user, user2, phong, pb_a, pb_b }
}

async fn cleanup(pool: &PgPool, fx: &Fx) {
    sqlx::query("DELETE FROM users WHERE id = ANY($1)").bind(vec![fx.user, fx.user2]).execute(pool).await.unwrap();
    sqlx::query("DELETE FROM projects WHERE id = $1").bind(fx.project).execute(pool).await.unwrap();
}

fn all(_: digitext_core::Permission) -> bool {
    true
}

fn ctx<'a>(fx: &Fx, source: &'a str) -> OpCtx<'a> {
    OpCtx { project_id: fx.project, user_id: fx.user, user_name: "Người thử", device_id: None, source, has_perm: &all, is_super: false }
}

fn row(v: Value) -> Row {
    v.as_object().unwrap().clone()
}

fn upsert(table: &str, id: Uuid, new: Value) -> SyncOp {
    SyncOp { op_id: None, table: table.into(), id, op: OpKind::Upsert, base: None, new: row(new), if_ver: None, edited_at: None }
}

fn delete(table: &str, id: Uuid) -> SyncOp {
    SyncOp { op_id: None, table: table.into(), id, op: OpKind::Delete, base: None, new: Row::new(), if_ver: None, edited_at: None }
}

fn is_locked(r: &OpResult) -> bool {
    matches!(r, OpResult::Rejected { reason, .. } if reason == "locked")
}

async fn tao_ho_so(pool: &PgPool, fx: &Fx, pb: Uuid, so_tam: &str, by: Uuid) -> Uuid {
    let id = Uuid::now_v7();
    let c = OpCtx { project_id: fx.project, user_id: by, user_name: "Người thử", device_id: None, source: "online", has_perm: &all, is_super: false };
    let r = sync::apply_op(pool, &c, &upsert("cl_ho_so", id, json!({"phong_id": fx.phong, "phong_ban_id": pb, "so_tam": so_tam, "tieu_de": format!("Hồ sơ {so_tam}"), "ngay_ket_thuc": "2025"})))
        .await
        .unwrap();
    assert!(r.is_ok(), "{r:?}");
    id
}

/// Nộp → hồ sơ khoá (sửa mô tả / thêm văn bản / xoá bị từ chối, cột hệ thống hoá vẫn ghi) → rút → sửa lại được →
/// nộp lại → tiếp nhận (một nhận, một trả) → hồ sơ nhận là da_nop có khoa_luc, hồ sơ trả là tra_lai sửa được.
#[tokio::test]
async fn nop_luu_khoa_ho_so_va_mo_khi_rut() {
    let Some(pool) = pool().await else { return };
    let fx = fixture(&pool).await;
    let c = ctx(&fx, "online");
    let hs1 = tao_ho_so(&pool, &fx, fx.pb_a, "01", fx.user).await;
    let hs2 = tao_ho_so(&pool, &fx, fx.pb_a, "02", fx.user).await;
    let vb = Uuid::now_v7();
    assert!(sync::apply_op(&pool, &c, &upsert("cl_van_ban", vb, json!({"ho_so_id": hs1, "so_thu_tu": 1, "trich_yeu": "Công văn A"}))).await.unwrap().is_ok());

    let phieu = nop_luu::lap(&pool, nop_luu::LapPhieu { project_id: fx.project, phong_ban_id: Some(fx.pb_a), nguoi_nop: fx.user, nguoi_nop_ten: "Người thử", ghi_chu: None, ho_so_ids: &[hs1, hs2] })
        .await
        .unwrap()
        .expect("lập phiếu");
    assert_eq!(phieu.trang_thai, "cho_tiep_nhan");
    assert!(phieu.ma_phieu.ends_with("/NL-VP.2026") || phieu.ma_phieu.contains("/NL-VP."), "{}", phieu.ma_phieu);
    assert_eq!(phieu.so_ho_so, 2);
    assert_eq!(phieu.so_van_ban, Some(1));
    // Nộp lần hai cùng hồ sơ → từ chối.
    let lai = nop_luu::lap(&pool, nop_luu::LapPhieu { project_id: fx.project, phong_ban_id: Some(fx.pb_a), nguoi_nop: fx.user, nguoi_nop_ten: "x", ghi_chu: None, ho_so_ids: &[hs1] }).await.unwrap();
    assert!(lai.is_err(), "{lai:?}");

    // Khoá: mô tả, văn bản mới, văn bản cũ, tệp, xoá — kể cả push offline.
    assert!(is_locked(&sync::apply_op(&pool, &c, &upsert("cl_ho_so", hs1, json!({"tieu_de": "Sửa"}))).await.unwrap()));
    assert!(is_locked(&sync::apply_op(&pool, &c, &upsert("cl_van_ban", Uuid::now_v7(), json!({"ho_so_id": hs1, "so_thu_tu": 2, "trich_yeu": "B"}))).await.unwrap()));
    assert!(is_locked(&sync::apply_op(&pool, &c, &upsert("cl_van_ban", vb, json!({"trich_yeu": "Sửa VB"}))).await.unwrap()));
    assert!(is_locked(&sync::apply_op(&pool, &c, &upsert("cl_van_ban_file", Uuid::now_v7(), json!({"van_ban_id": vb, "name": "a.pdf"}))).await.unwrap()));
    assert!(is_locked(&sync::apply_op(&pool, &c, &delete("cl_ho_so", hs1)).await.unwrap()));
    assert!(is_locked(&sync::apply_op(&pool, &ctx(&fx, "sync"), &upsert("cl_ho_so", hs1, json!({"tieu_de": "Sửa offline"}))).await.unwrap()));
    // Cột hệ thống hoá (online_only) vẫn ghi được; dòng danh mục thì không.
    let r = sync::apply_op(&pool, &c, &upsert("cl_ho_so", hs1, json!({"ho_so_so": "15"}))).await.unwrap();
    assert!(r.is_ok(), "{r:?}");
    assert!(is_locked(&sync::apply_op(&pool, &c, &upsert("cl_ho_so", hs1, json!({"dm_dong_id": Uuid::now_v7()}))).await.unwrap()));
    // Không đổi gì → NoChange, không phải locked.
    assert!(matches!(sync::apply_op(&pool, &c, &upsert("cl_ho_so", hs1, json!({"tieu_de": "Hồ sơ 01"}))).await.unwrap(), OpResult::NoChange { .. }));

    // Rút → mở.
    let p2 = nop_luu::rut(&pool, phieu.id, fx.user, "Người thử").await.unwrap().expect("rút");
    assert_eq!(p2.trang_thai, "rut");
    assert!(sync::apply_op(&pool, &c, &upsert("cl_ho_so", hs1, json!({"tieu_de": "Sửa sau rút"}))).await.unwrap().is_ok());
    let tt: Option<String> = sqlx::query_scalar("SELECT nop_luu_trang_thai FROM cl_ho_so WHERE id = $1").bind(hs1).fetch_one(&pool).await.unwrap();
    assert_eq!(tt, None);

    // Nộp lại, chấm một trả, tiếp nhận.
    let p3 = nop_luu::lap(&pool, nop_luu::LapPhieu { project_id: fx.project, phong_ban_id: Some(fx.pb_a), nguoi_nop: fx.user, nguoi_nop_ten: "Người thử", ghi_chu: None, ho_so_ids: &[hs1, hs2] }).await.unwrap().expect("lập lại");
    assert!(nop_luu::cham(&pool, p3.id, hs2, "tra_lai", None).await.unwrap().is_err(), "trả lại phải có lý do");
    nop_luu::cham(&pool, p3.id, hs2, "tra_lai", Some("Thiếu chứng từ kết thúc")).await.unwrap().expect("chấm");
    let p4 = nop_luu::tiep_nhan(&pool, p3.id, fx.user2, "Cán bộ LT", None).await.unwrap().expect("tiếp nhận");
    assert_eq!(p4.trang_thai, "da_tiep_nhan");
    let (tt1, khoa1): (Option<String>, Option<chrono::DateTime<chrono::Utc>>) = sqlx::query_as("SELECT nop_luu_trang_thai, khoa_luc FROM cl_ho_so WHERE id = $1").bind(hs1).fetch_one(&pool).await.unwrap();
    assert_eq!((tt1.as_deref(), khoa1.is_some()), (Some("da_nop"), true));
    let tt2: Option<String> = sqlx::query_scalar("SELECT nop_luu_trang_thai FROM cl_ho_so WHERE id = $1").bind(hs2).fetch_one(&pool).await.unwrap();
    assert_eq!(tt2.as_deref(), Some("tra_lai"));
    assert!(is_locked(&sync::apply_op(&pool, &c, &upsert("cl_ho_so", hs1, json!({"tieu_de": "X"}))).await.unwrap()));
    assert!(sync::apply_op(&pool, &c, &upsert("cl_ho_so", hs2, json!({"tieu_de": "Sửa hồ sơ bị trả"}))).await.unwrap().is_ok());
    // Lịch sử có mốc nộp lưu.
    let acts: Vec<String> = sqlx::query_scalar("SELECT action FROM cl_lich_su WHERE ho_so_id = $1 ORDER BY at").bind(hs1).fetch_all(&pool).await.unwrap();
    assert!(acts.iter().any(|a| a == "nop_luu") && acts.iter().any(|a| a == "rut_nop_luu") && acts.iter().any(|a| a == "tiep_nhan"), "{acts:?}");
    // Sổ nộp lưu năm nay có phiếu, hồ sơ trong phiếu đủ 2 dòng, mở khoá ngoại lệ mở được.
    let nam = nop_luu::hom_nay_vn().format("%Y").to_string().parse::<i32>().unwrap();
    assert_eq!(nop_luu::so_nop_luu(&pool, fx.project, nam, None).await.unwrap().len(), 1);
    assert_eq!(nop_luu::ho_so_cua(&pool, p3.id).await.unwrap().len(), 2);
    assert!(nop_luu::mo_khoa(&pool, hs1, fx.user2).await.unwrap());
    assert!(sync::apply_op(&pool, &c, &upsert("cl_ho_so", hs1, json!({"tieu_de": "Sau mở khoá"}))).await.unwrap().is_ok());
    cleanup(&pool, &fx).await;
}

/// Phạm vi phòng ban: người phòng A thấy hồ sơ phòng A và hồ sơ mình lập ở phòng khác; không có phạm vi thì thấy hết;
/// hồ sơ phòng A đã nộp lưu vẫn thấy (còn trong 360 ngày).
#[tokio::test]
async fn pham_vi_phong_ban_loc_ho_so() {
    let Some(pool) = pool().await else { return };
    let fx = fixture(&pool).await;
    let a1 = tao_ho_so(&pool, &fx, fx.pb_a, "A1", fx.user).await;
    let _b1 = tao_ho_so(&pool, &fx, fx.pb_b, "B1", fx.user2).await;
    let b2 = tao_ho_so(&pool, &fx, fx.pb_b, "B2", fx.user).await; // người A lập hộ ở phòng B
    let f = |pv: Option<PhamVi>, nop_luu: Option<&str>| ho_so::Filter { project_id: fx.project, pham_vi: pv, nop_luu: nop_luu.map(str::to_owned), limit: 100, ..Default::default() };
    let ids = |rows: Vec<Value>| -> Vec<Uuid> { rows.iter().filter_map(|r| r.get("id").and_then(Value::as_str)).filter_map(|s| s.parse().ok()).collect() };
    let (rows, n) = ho_so::list(&pool, &f(None, None)).await.unwrap();
    assert_eq!((rows.len(), n), (3, 3));
    let (rows, n) = ho_so::list(&pool, &f(Some(PhamVi { phong_ban_id: fx.pb_a, user_id: fx.user }), None)).await.unwrap();
    let got = ids(rows);
    assert_eq!(n, 2);
    assert!(got.contains(&a1) && got.contains(&b2), "{got:?}");
    let (rows, _) = ho_so::list(&pool, &f(Some(PhamVi { phong_ban_id: fx.pb_b, user_id: fx.user2 }), None)).await.unwrap();
    assert_eq!(rows.len(), 2, "phòng B thấy B1, B2");
    // Pull theo phạm vi: người phòng A kéo về A1 + B2 (mình lập) và văn bản của chúng, không có B1.
    let vb_b1 = Uuid::now_v7();
    assert!(sync::apply_op(&pool, &OpCtx { project_id: fx.project, user_id: fx.user2, user_name: "x", device_id: None, source: "online", has_perm: &all, is_super: false }, &upsert("cl_van_ban", vb_b1, json!({"ho_so_id": _b1, "so_thu_tu": 1, "trich_yeu": "VB của B1"}))).await.unwrap().is_ok());
    let p = sync::pull(&pool, fx.project, 0, 1000, Some(PhamVi { phong_ban_id: fx.pb_a, user_id: fx.user }), None).await.unwrap();
    let hs_keo: Vec<String> = p.rows.iter().filter(|r| r.table == "cl_ho_so").filter_map(|r| r.row.get("so_tam").and_then(Value::as_str).map(str::to_owned)).collect();
    assert_eq!(hs_keo.len(), 2, "{hs_keo:?}");
    assert!(hs_keo.contains(&"A1".to_string()) && hs_keo.contains(&"B2".to_string()));
    assert!(!p.rows.iter().any(|r| r.table == "cl_van_ban" && r.row.get("id").and_then(Value::as_str) == Some(&vb_b1.to_string())), "văn bản của B1 không được kéo về");
    assert!(p.rows.iter().any(|r| r.table == "cl_phong_ban"), "danh mục vẫn kéo đủ");
    let p_full = sync::pull(&pool, fx.project, 0, 1000, None, None).await.unwrap();
    assert_eq!(p_full.rows.iter().filter(|r| r.table == "cl_ho_so").count(), 3);
    // Nộp lưu A1 → vẫn thấy (trong 360 ngày), lọc nop_luu hoạt động.
    let p = nop_luu::lap(&pool, nop_luu::LapPhieu { project_id: fx.project, phong_ban_id: Some(fx.pb_a), nguoi_nop: fx.user, nguoi_nop_ten: "x", ghi_chu: None, ho_so_ids: &[a1] }).await.unwrap().unwrap();
    nop_luu::tiep_nhan(&pool, p.id, fx.user2, "LT", None).await.unwrap().unwrap();
    let (rows, _) = ho_so::list(&pool, &f(Some(PhamVi { phong_ban_id: fx.pb_a, user_id: fx.user }), None)).await.unwrap();
    assert_eq!(rows.len(), 2);
    let (rows, _) = ho_so::list(&pool, &f(None, Some("da_nop"))).await.unwrap();
    assert_eq!(ids(rows), vec![a1]);
    let (rows, _) = ho_so::list(&pool, &f(None, Some("chua_nop"))).await.unwrap();
    assert_eq!(rows.len(), 2);
    // Hết 360 ngày (giả bằng cách lùi khoa_luc) → phòng A không còn thấy A1, người lập cũng không (created_by là user nhưng…
    // created_by vẫn thấy: quy tắc là "hồ sơ mình lập" luôn thấy) → kiểm bằng người khác cùng phòng A.
    sqlx::query("UPDATE cl_ho_so SET khoa_luc = now() - interval '400 days' WHERE id = $1").bind(a1).execute(&pool).await.unwrap();
    let (rows, _) = ho_so::list(&pool, &f(Some(PhamVi { phong_ban_id: fx.pb_a, user_id: Uuid::now_v7() }), None)).await.unwrap();
    assert!(rows.is_empty(), "người khác phòng A hết quyền xem A1 sau 360 ngày: {rows:?}");
    // Hồ sơ cần nộp + tổng quan + nhắc hạn (hồ sơ kết thúc 2025 → hạn 01/03/2026 → quá hạn → có thông báo).
    let (rows, _) = nop_luu::ho_so_can_nop(&pool, fx.project, Some(fx.pb_b), Some("qua_han"), 100, 0).await.unwrap();
    assert_eq!(rows.len(), 2, "{rows:?}");
    assert!(rows[0].get("han_nop").is_some() && rows[0].get("con_ngay").is_some());
    let tq = nop_luu::tong_quan(&pool, fx.project, None).await.unwrap();
    assert_eq!((tq.qua_han, tq.chua_nop), (2, 2));
    let hom_nay = chrono::NaiveDate::from_ymd_opt(2026, 11, 26).unwrap();
    // quet_nhac quét TOÀN máy chủ (test khác chạy song song cũng tạo hồ sơ) → chỉ đếm thông báo của user trong test này.
    let dem = |pool: PgPool| async move {
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM notifications WHERE user_id = ANY($1)").bind(vec![fx.user, fx.user2]).fetch_one(&pool).await.unwrap()
    };
    nop_luu::quet_nhac(&pool, hom_nay, 2025).await.unwrap();
    let c1 = dem(pool.clone()).await;
    assert!(c1 >= 2, "ít nhất người lập của B1 và B2 được nhắc quá hạn: {c1}");
    nop_luu::quet_nhac(&pool, hom_nay, 2025).await.unwrap();
    assert_eq!(dem(pool.clone()).await, c1, "chạy lại cùng ngày không nhắc lặp");
    let kinds: Vec<String> = sqlx::query_scalar("SELECT DISTINCT kind FROM notifications WHERE user_id = ANY($1) ORDER BY 1").bind(vec![fx.user, fx.user2]).fetch_all(&pool).await.unwrap();
    assert!(kinds.iter().any(|k| k == "nop_luu_qua_han"), "{kinds:?}");
    sqlx::query("DELETE FROM notifications WHERE user_id = ANY($1)").bind(vec![fx.user, fx.user2]).execute(&pool).await.unwrap();
    cleanup(&pool, &fx).await;
}

/// Danh mục: tạo → dòng (phòng ban chỉ sửa dòng mình) → trình → duyệt → ban hành → dòng hiệu lực; phiên bản mới chép dòng;
/// ban hành bản mới làm bản cũ hết hiệu lực; đối chiếu đếm hồ sơ gắn dòng.
#[tokio::test]
async fn danh_muc_ho_so_tron_luong() {
    let Some(pool) = pool().await else { return };
    let fx = fixture(&pool).await;
    let d = dm::tao(&pool, fx.project, fx.phong, 2026, fx.user, None).await.unwrap().expect("tạo");
    assert_eq!((d.phien_ban, d.trang_thai.as_str()), (1, "du_thao"));
    assert!(dm::tao(&pool, fx.project, fx.phong, 2026, fx.user, None).await.unwrap().is_err(), "đang có bản chưa ban hành");
    let dong = |pb: Uuid, ma: &str, td: &str| dm::DongMoi { phong_ban_id: pb, mang_id: None, ma_dong: ma.into(), tieu_de: td.into(), thbq_id: None, nguoi_lap: None, ghi_chu: None, so_tt: None };
    assert!(dm::trinh(&pool, d.id, fx.user).await.unwrap().is_err(), "chưa có dòng không trình được");
    let d1 = dm::them_dong(&pool, d.id, fx.project, fx.user, Some(fx.pb_a), &dong(fx.pb_a, "01", "Tập lưu quyết định")).await.unwrap().expect("thêm dòng A");
    assert!(dm::them_dong(&pool, d.id, fx.project, fx.user, Some(fx.pb_a), &dong(fx.pb_b, "02", "Dòng phòng khác")).await.unwrap().is_err(), "phòng A không thêm dòng phòng B");
    assert!(dm::them_dong(&pool, d.id, fx.project, fx.user, None, &dong(fx.pb_a, "01", "Trùng mã")).await.unwrap().is_err(), "trùng ma_dong");
    let (them, sua, loi) = dm::nhap_dong(&pool, d.id, fx.project, fx.user2, None, &[dong(fx.pb_b, "02", "Hồ sơ hội nghị"), dong(fx.pb_b, "", "thiếu mã"), dong(fx.pb_b, "02", "Hồ sơ hội nghị (sửa)")]).await.unwrap().expect("nhập");
    assert_eq!((them, sua, loi.len()), (1, 1, 1), "{loi:?}");
    assert_eq!(dm::dong_cua(&pool, d.id, None).await.unwrap().len(), 2);
    assert_eq!(dm::dong_cua(&pool, d.id, Some(fx.pb_b)).await.unwrap().len(), 1);
    dm::trinh(&pool, d.id, fx.user).await.unwrap().expect("trình");
    assert!(dm::them_dong(&pool, d.id, fx.project, fx.user, None, &dong(fx.pb_a, "03", "x")).await.unwrap().is_err(), "đã trình không sửa dòng");
    assert!(dm::ban_hanh(&pool, d.id, fx.user, "12/QĐ", chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap()).await.unwrap().is_err(), "chưa duyệt không ban hành");
    let tl = dm::tra_lai(&pool, d.id, fx.user2, "Thiếu phòng Tài chính").await.unwrap().expect("trả lại");
    assert_eq!((tl.trang_thai.as_str(), tl.ly_do_tra.as_deref()), ("du_thao", Some("Thiếu phòng Tài chính")));
    dm::trinh(&pool, d.id, fx.user).await.unwrap().unwrap();
    dm::duyet(&pool, d.id, fx.user2).await.unwrap().expect("duyệt");
    let bh = dm::ban_hanh(&pool, d.id, fx.user2, "12/QĐ-UBND", chrono::NaiveDate::from_ymd_opt(2026, 1, 15).unwrap()).await.unwrap().expect("ban hành");
    assert_eq!(bh.trang_thai, "ban_hanh");
    assert_eq!(dm::dong_hieu_luc(&pool, fx.phong, 2026, Some(fx.pb_a)).await.unwrap().len(), 1);
    // Gắn hồ sơ vào dòng, đối chiếu.
    let hs = tao_ho_so(&pool, &fx, fx.pb_a, "H1", fx.user).await;
    let c = ctx(&fx, "online");
    sqlx::query("UPDATE cl_ho_so SET ngay_ket_thuc = '2026' WHERE id = $1").bind(hs).execute(&pool).await.unwrap();
    assert!(sync::apply_op(&pool, &c, &upsert("cl_ho_so", hs, json!({"dm_dong_id": d1}))).await.unwrap().is_ok());
    assert!(!sync::apply_op(&pool, &ctx(&fx, "sync"), &upsert("cl_ho_so", hs, json!({"dm_dong_id": Value::Null}))).await.unwrap().is_ok(), "đổi dòng danh mục offline bị từ chối (online_only)");
    let dc = dm::doi_chieu(&pool, d.id).await.unwrap().unwrap();
    let theo = dc["theoDanhMuc"].as_array().unwrap();
    assert_eq!(theo.iter().find(|r| r["maDong"] == "01").unwrap()["soHoSo"], json!(1));
    // Phiên bản mới → v2 chép 2 dòng; ban hành v2 → v1 hết hiệu lực; đối chiếu v2 vẫn đếm hồ sơ qua ma_dong.
    let v2 = dm::phien_ban_moi(&pool, d.id, fx.user).await.unwrap().expect("v2");
    assert_eq!((v2.phien_ban, dm::dong_cua(&pool, v2.id, None).await.unwrap().len()), (2, 2));
    dm::trinh(&pool, v2.id, fx.user).await.unwrap().unwrap();
    dm::duyet(&pool, v2.id, fx.user2).await.unwrap().unwrap();
    dm::ban_hanh(&pool, v2.id, fx.user2, "30/QĐ-UBND", chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap()).await.unwrap().unwrap();
    let tt1: String = sqlx::query_scalar("SELECT trang_thai FROM lt_danh_muc WHERE id = $1").bind(d.id).fetch_one(&pool).await.unwrap();
    assert_eq!(tt1, "het_hieu_luc");
    let dc2 = dm::doi_chieu(&pool, v2.id).await.unwrap().unwrap();
    assert_eq!(dc2["theoDanhMuc"].as_array().unwrap().iter().find(|r| r["maDong"] == "01").unwrap()["soHoSo"], json!(1));
    assert_eq!(dm::list(&pool, fx.project, Some(fx.phong), Some(2026)).await.unwrap().len(), 2);
    cleanup(&pool, &fx).await;
}
