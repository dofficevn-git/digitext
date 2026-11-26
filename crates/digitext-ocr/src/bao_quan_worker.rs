// Copyright (c) 2026 DigiText. All rights reserved.
//! Việc nền bảo quản Điều 29–31 (0032, 26/11/2026), chạy ở CẢ HAI chế độ worker (thi công và chỉ-lưu-trữ):
//! - `thuc_hien_ke_hoach`: mỗi giờ lấy kế hoạch đã duyệt / đang thực hiện, kiểm một lô tệp (băm lại + PDF/A; zip gói sha256),
//!   ghi `integrity_checks(nguon = 'ke_hoach')`, mở dòng `bk_loi`; hết tệp chờ → báo quản trị dự án lập biên bản Mẫu 01.
//! - `nhac_lich_sao_luu`: thay cảnh báo 26 giờ cứng của `package_worker::f_2b5` — từng lịch (`bk_lich_sao_luu`) có chu kỳ và độ
//!   trễ cho phép riêng; nhắc thêm "bản cũ cần xoá" khi số bản đạt vượt số bản phải giữ (chỉ nhắc, xoá thật do script / người).
//! - `nhac_phuong_tien`: phương tiện lưu trữ sắp hết hạn độ bền (≤ 1 năm, Điều 30.4) — mỗi tháng một lần tới khi thay.
//! Lượt đầu chạy NGAY khi khởi động (khác khuôn cũ ngủ trước), các lượt sau theo chu kỳ. `DIGITEXT_BQ_GIAY` rút ngắn chu kỳ khi thử.
use digitext_core::bao_quan as bq;
use digitext_db::{bao_quan as db, notifications, ops, projects};
use sqlx::PgPool;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

async fn bao_quan_tri_du_an(pool: &PgPool, project: uuid::Uuid, kind: &str, msg: &str, ref_: &str) -> sqlx::Result<()> {
    for a in projects::f_re(pool, project).await? {
        let _ = notifications::f_1v8_ref(pool, a, kind, msg, ref_).await?;
    }
    Ok(())
}

async fn bao_moi_quan_tri(pool: &PgPool, kind: &str, msg: &str, ref_: &str) -> sqlx::Result<()> {
    for a in ops::f_rk(pool).await? {
        let _ = notifications::f_1v8_ref(pool, a, kind, msg, ref_).await?;
    }
    Ok(())
}

/// Kiểm `lo` tệp / gói còn chờ của một kế hoạch; trả (đã kiểm, lỗi, kế hoạch vừa hoàn thành). Dùng chung hàm băm với sweeper
/// bảo quản (`package_worker::check_document` / `check_package`).
pub async fn kiem_lo(pool: &PgPool, store: &dyn digitext_storage::ObjectStore, k: &db::KhRow, lo: i64) -> sqlx::Result<(usize, usize, bool)> {
    let (mut n, mut loi, mut xong) = (0usize, 0usize, false);
    for t in db::kh_tep_cho(pool, k.id, lo).await? {
        let kq = match (t.document_id, t.package_id, t.zip_key.as_deref(), t.zip_sha256.as_deref()) {
            (Some(doc), _, _, _) => crate::package_worker::check_document(store, k.project_id, doc, t.page_count, t.blake3.as_deref()).await,
            (None, Some(_), Some(key), Some(sha)) => crate::package_worker::check_package(store, key, sha).await,
            _ => Err("gói không còn zip / sha256".to_string()),
        };
        n += 1;
        let x = match kq {
            Ok(()) => db::kh_ghi_ket_qua(pool, k, &t, "dat", None).await?,
            Err(e) => {
                loi += 1;
                db::kh_ghi_ket_qua(pool, k, &t, "loi", Some(&e)).await?
            }
        };
        xong |= x;
    }
    Ok((n, loi, xong))
}

pub async fn thuc_hien_ke_hoach(pool: &PgPool, store: &dyn digitext_storage::ObjectStore, chu_ky: Duration, lo: i64, tok: &CancellationToken) -> sqlx::Result<()> {
    while !tok.is_cancelled() {
        for k in db::kh_dang_chay(pool).await? {
            if tok.is_cancelled() {
                break;
            }
            let (n, loi, xong) = kiem_lo(pool, store, &k, lo).await?;
            tracing::info!(ke_hoach = %k.ma, da_kiem = n, loi, hoan_thanh = xong, "kế hoạch kiểm tra (Điều 29): kiểm một lô");
            if xong {
                bao_quan_tri_du_an(pool, k.project_id, "plan_done", &format!("Kế hoạch kiểm tra {} đã kiểm hết {} tệp ({} lỗi): xem kết quả và lập biên bản Mẫu 01", k.ma, k.so_tep, k.so_loi + i32::try_from(loi).unwrap_or(0)), &format!("kh:{}:xong", k.id)).await?;
            }
        }
        tokio::select! {
            _ = tok.cancelled() => break,
            _ = tokio::time::sleep(chu_ky) => {}
        }
    }
    Ok(())
}

pub async fn nhac_lich_sao_luu(pool: &PgPool, chu_ky: Duration, tok: &CancellationToken) -> sqlx::Result<()> {
    while !tok.is_cancelled() {
        let bay_gio = chrono::Utc::now();
        let hom_nay = digitext_db::nop_luu::hom_nay_vn();
        for l in db::lich_list(pool).await? {
            if !l.bat {
                continue;
            }
            match bq::tinh_trang_lich(l.lan_dat, i64::from(l.chu_ky_gio), i64::from(l.do_tre_cho_phep_gio), bay_gio) {
                bq::TinhTrangLich::Tre { han_ke_tiep, gio } => {
                    let msg = format!(
                        "SAO LƯU TRỄ {gio} giờ: lịch «{}» ({}) phải có bản mới trước {} — Điều 30.1",
                        l.ten,
                        l.nguon.as_deref().unwrap_or(""),
                        han_ke_tiep.with_timezone(&chrono::FixedOffset::east_opt(7 * 3600).expect("+07:00")).format("%d/%m/%Y %H:%M")
                    );
                    // Mỗi ngày một lần cho tới khi có bản đạt.
                    bao_moi_quan_tri(pool, "backup_overdue", &msg, &format!("lich:{}:tre:{}", l.id, hom_nay)).await?;
                }
                bq::TinhTrangLich::ChuaCo => {
                    // Chưa có bản nào: lịch ngắn (≤ 2 ngày) nhắc hằng ngày, lịch dài nhắc hằng tháng.
                    let khoa = if l.chu_ky_gio <= 48 { hom_nay.format("%Y-%m-%d").to_string() } else { hom_nay.format("%Y-%m").to_string() };
                    let msg = format!("CHƯA CÓ bản sao lưu nào theo lịch «{}» — Điều 30.1 (script vận hành ghi nhận qua POST /bao-quan/sao-luu)", l.ten);
                    bao_moi_quan_tri(pool, "backup_overdue", &msg, &format!("lich:{}:chuaco:{khoa}", l.id)).await?;
                }
                bq::TinhTrangLich::DungHan { .. } => {}
            }
            let can_xoa = bq::so_ban_can_xoa(l.so_ban, i64::from(l.giu_so_ban));
            if can_xoa > 0 {
                let msg = format!("Lịch «{}» đang giữ {} bản, quá số phải giữ ({}): xoá {can_xoa} bản cũ nhất rồi đánh dấu ở Bảo quản › Sao lưu", l.ten, l.so_ban, l.giu_so_ban);
                bao_moi_quan_tri(pool, "backup_prune", &msg, &format!("lich:{}:prune:{}", l.id, hom_nay.format("%Y-%m-%d"))).await?;
            }
        }
        tokio::select! {
            _ = tok.cancelled() => break,
            _ = tokio::time::sleep(chu_ky) => {}
        }
    }
    Ok(())
}

pub async fn nhac_phuong_tien(pool: &PgPool, chu_ky: Duration, tok: &CancellationToken) -> sqlx::Result<()> {
    while !tok.is_cancelled() {
        let hom_nay = digitext_db::nop_luu::hom_nay_vn();
        for (id, ma, han) in db::pt_sap_het(pool, hom_nay, bq::NHAC_PHUONG_TIEN_TRUOC_NGAY).await? {
            let con = (han - hom_nay).num_days();
            let msg = if con < 0 {
                format!("Phương tiện lưu trữ {ma} ĐÃ QUÁ hạn độ bền {} ngày ({}) — chuyển dữ liệu sang phương tiện mới (Điều 30.4)", -con, han.format("%d/%m/%Y"))
            } else {
                format!("Phương tiện lưu trữ {ma} còn {con} ngày tới hạn độ bền ({}) — lên kế hoạch chuyển đổi trước 1 năm (Điều 30.4)", han.format("%d/%m/%Y"))
            };
            let moi = bao_moi_quan_tri_dem(pool, "media_expiring", &msg, &bq::khoa_nhac_thang(&ma, hom_nay)).await?;
            if moi {
                db::pt_danh_dau_nhac(pool, id).await?;
            }
        }
        tokio::select! {
            _ = tok.cancelled() => break,
            _ = tokio::time::sleep(chu_ky) => {}
        }
    }
    Ok(())
}

/// Như `bao_moi_quan_tri` nhưng cho biết có gửi được ít nhất một thông báo mới không.
async fn bao_moi_quan_tri_dem(pool: &PgPool, kind: &str, msg: &str, ref_: &str) -> sqlx::Result<bool> {
    let mut moi = false;
    for a in ops::f_rk(pool).await? {
        moi |= notifications::f_1v8_ref(pool, a, kind, msg, ref_).await?;
    }
    Ok(moi)
}
