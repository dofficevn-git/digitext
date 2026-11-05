// Copyright (c) 2026 DigiText. All rights reserved.
//! NHẬP GÓI AIP/SIP VÀO HỆ THỐNG (bản Lưu trữ GĐ3, 4/11/2026 — docs/KE-HOACH-BAN-LUU-TRU.md; TT 05/2025/TT-BNV Điều 9 khoản 7).
//!
//! Một gói → kiểm bằng pkgcheck (cùng bộ kiểm của Tiện ích › Kiểm tra gói tin) → zip gốc vào kho WORM (`{p}-packages/`,
//! `packages.nguon = 'nhap'`) → phông theo mã phông trong OBJID (tạo nếu thiếu) → hồ sơ từ EAD hồ sơ → từng văn bản từ EAD
//! văn bản + tệp PDF (documents `archived`, bản đã ký ở `{p}-warm/{doc}.signed.pdf`, chưa ký ở `{p}-warm/{doc}.pdf`) →
//! lớp chữ PDF vào `ocr_results` (tìm toàn văn) → `cl_ho_so_goi` (dấu đã nhập xong). Mọi ghi mục lục đi qua `apply_op`
//! (nguồn `import`, có lịch sử).
//!
//! Nhập lại được: dừng giữa chừng thì chạy lại sẽ dùng lại gói / hồ sơ / văn bản / tài liệu đã ghi; gói đã có dòng
//! `cl_ho_so_goi` thì báo "đã có". Gói có lỗi kiểm bị từ chối, trừ khi `cho_phep_loi`; chữ ký số KHÔNG toàn vẹn luôn bị
//! từ chối. Tệp chưa ký vẫn nhận, ghi `documents.ky_so = 'chua_ky'` và đếm trong báo cáo (chốt với anh Hiếu 4/11/2026).
use anyhow::Context;
use digitext_core::Permission;
use digitext_db::chinh_ly::{self as cl, OpCtx, OpKind, OpResult, Row, SyncOp};
use digitext_pkgcheck::{Extract, Level, PackageResult};
use digitext_storage::ObjectStore;
use serde::Serialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::path::Path;
use uuid::Uuid;

/// Dòng `engine_versions` gắn lớp chữ lấy từ gói (migration 0025, active = false).
pub const ENGINE_LOP_CHU: Uuid = Uuid::from_u128(0x0000_0000_0000_4000_8000_0000_000e_0001);

pub struct NhapCtx<'a> {
    pub pool: &'a PgPool,
    pub store: &'a dyn ObjectStore,
    pub project_id: Uuid,
    pub user_id: Uuid,
    pub user_name: &'a str,
    pub device_id: Option<Uuid>,
    pub has_perm: &'a (dyn Fn(Permission) -> bool + Sync),
    pub is_super: bool,
    pub batch_id: Uuid,
    /// Nhận cả gói kiểm KHÔNG ĐẠT (trừ chữ ký hỏng) — gói đời cũ, chủ đầu tư chấp nhận.
    pub cho_phep_loi: bool,
}

impl NhapCtx<'_> {
    fn op_ctx(&self) -> OpCtx<'_> {
        OpCtx {
            project_id: self.project_id,
            user_id: self.user_id,
            user_name: self.user_name,
            device_id: self.device_id,
            source: "import",
            has_perm: self.has_perm,
            is_super: self.is_super,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KetQuaGoi {
    pub ten_tep: String,
    pub objid: String,
    /// nhap | da_co | tu_choi | loi
    pub ket_qua: &'static str,
    pub ket_luan_kiem: String,
    pub so_loi: usize,
    pub so_canh_bao: usize,
    pub package_id: Option<Uuid>,
    pub ho_so_id: Option<Uuid>,
    pub ma_ho_so: String,
    pub tieu_de: String,
    pub so_van_ban: usize,
    pub so_chua_ky: usize,
    pub thong_bao: Option<String>,
}

impl KetQuaGoi {
    fn moi(ten_tep: String, res: &PackageResult) -> Self {
        KetQuaGoi {
            ten_tep,
            objid: res.objid.clone(),
            ket_qua: "loi",
            ket_luan_kiem: res.verdict.clone(),
            so_loi: res.errors,
            so_canh_bao: res.warnings,
            package_id: None,
            ho_so_id: None,
            ma_ho_so: res.file_code.clone(),
            tieu_de: res.title.clone(),
            so_van_ban: 0,
            so_chua_ky: 0,
            thong_bao: None,
        }
    }
    fn ket(mut self, ket_qua: &'static str, thong_bao: impl Into<String>) -> Self {
        self.ket_qua = ket_qua;
        let t = thong_bao.into();
        self.thong_bao = (!t.is_empty()).then_some(t);
        self
    }
}

pub async fn tao_lo(pool: &PgPool, project_id: Uuid, user_id: Uuid, nguon: &str, ghi_chu: Option<&str>) -> sqlx::Result<Uuid> {
    let id = Uuid::now_v7();
    sqlx::query("INSERT INTO ingest_batches (id, project_id, created_by, nguon, ghi_chu) VALUES ($1, $2, $3, $4, $5)")
        .bind(id)
        .bind(project_id)
        .bind(user_id)
        .bind(nguon)
        .bind(ghi_chu)
        .execute(pool)
        .await?;
    Ok(id)
}

pub async fn ket_thuc_lo(pool: &PgPool, batch_id: Uuid) -> sqlx::Result<()> {
    sqlx::query("UPDATE ingest_batches SET finished_at = now() WHERE id = $1").bind(batch_id).execute(pool).await?;
    Ok(())
}

async fn ghi_ket_qua(ctx: &NhapCtx<'_>, kq: &KetQuaGoi, res: &PackageResult) -> sqlx::Result<()> {
    let chi_tiet: Vec<&digitext_pkgcheck::Finding> = res.findings.iter().filter(|f| f.level != Level::Ok).take(300).collect();
    sqlx::query(
        "INSERT INTO ingest_items (id, batch_id, project_id, ten_tep, objid, loai_goi, sha256, size_bytes, ket_qua, ket_luan_kiem, \
         so_loi, so_canh_bao, package_id, ho_so_id, so_van_ban, so_chua_ky, thong_bao, chi_tiet) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18)",
    )
    .bind(Uuid::now_v7())
    .bind(ctx.batch_id)
    .bind(ctx.project_id)
    .bind(&kq.ten_tep)
    .bind((!kq.objid.is_empty()).then_some(&kq.objid))
    .bind(&res.kind)
    .bind(res.sha256.as_deref())
    .bind(res.size as i64)
    .bind(kq.ket_qua)
    .bind(&kq.ket_luan_kiem)
    .bind(kq.so_loi as i32)
    .bind(kq.so_canh_bao as i32)
    .bind(kq.package_id)
    .bind(kq.ho_so_id)
    .bind(kq.so_van_ban as i32)
    .bind(kq.so_chua_ky as i32)
    .bind(kq.thong_bao.as_deref())
    .bind(json!(chi_tiet))
    .execute(ctx.pool)
    .await?;
    Ok(())
}

/// Mã phông: `urn:<mã phông>:uuid:…` (OBJID gói DigiText / TT05); không có thì phần mã hồ sơ trước năm (4 chữ số).
pub fn ma_phong(objid: &str, ma_ho_so: &str) -> Option<String> {
    if let Some(rest) = objid.strip_prefix("urn:") {
        if let Some((m, _)) = rest.split_once(":uuid:") {
            let m = m.trim();
            if !m.is_empty() {
                return Some(m.to_string());
            }
        }
    }
    let parts: Vec<&str> = ma_ho_so.split('.').collect();
    let nam = parts.iter().position(|p| p.len() == 4 && p.bytes().all(|b| b.is_ascii_digit()))?;
    (nam > 0).then(|| parts[..nam].join("."))
}

/// Số tạm của hồ sơ trong phông: mã hồ sơ bỏ tiền tố mã phông ("H49.132.02.2017.01.04" → "2017.01.04").
pub fn so_tam_tu_ma(ma_ho_so: &str, phong: &str) -> String {
    ma_ho_so.strip_prefix(phong).and_then(|r| r.strip_prefix('.')).filter(|r| !r.is_empty()).unwrap_or(ma_ho_so).to_string()
}

/// Số thứ tự văn bản = 7 chữ số cuối của mã văn bản (`<mã hồ sơ>.<STT 7 số>`).
pub fn stt_tu_ma(ma_vb: &str) -> Option<i32> {
    let last = ma_vb.rsplit('.').next()?;
    (last.len() == 7 && last.bytes().all(|b| b.is_ascii_digit())).then(|| last.parse().ok()).flatten().filter(|n: &i32| *n > 0)
}

fn op_loi(r: &OpResult) -> Option<String> {
    match r {
        OpResult::Rejected { message, errors, .. } => {
            Some(match errors.first() {
                Some(e) => format!("{message} ({})", e.field),
                None => message.clone(),
            })
        }
        OpResult::Conflict { .. } => Some("dữ liệu đã bị người khác sửa".into()),
        _ => None,
    }
}

async fn ghi(ctx: &NhapCtx<'_>, table: &str, id: Uuid, new: Row) -> anyhow::Result<Result<(), String>> {
    let op = SyncOp { op_id: None, table: table.into(), id, op: OpKind::Upsert, base: None, new, if_ver: None, edited_at: None };
    let r = cl::apply_op(ctx.pool, &ctx.op_ctx(), &op).await?;
    Ok(match op_loi(&r) {
        Some(e) => Err(e),
        None => Ok(()),
    })
}

fn chen(row: &mut Row, k: &str, v: Option<&String>) {
    if let Some(v) = v.map(|v| v.trim()).filter(|v| !v.is_empty()) {
        row.insert(k.into(), json!(v));
    }
}

fn chen_so(row: &mut Row, k: &str, v: Option<&String>) {
    if let Some(n) = v.and_then(|v| v.trim().parse::<i64>().ok()) {
        row.insert(k.into(), json!(n));
    }
}

/// Lớp chữ từng trang (lopdf). Trang không đọc được / rỗng thì bỏ.
fn lop_chu(pdf: &[u8]) -> Vec<(i32, String)> {
    let Ok(doc) = lopdf::Document::load_mem(pdf) else { return Vec::new() };
    let pages: Vec<u32> = doc.get_pages().keys().copied().collect();
    pages
        .into_iter()
        .filter_map(|n| {
            let t = doc.extract_text(&[n]).ok()?;
            let t = t.split_whitespace().collect::<Vec<_>>().join(" ");
            (!t.is_empty()).then_some((n as i32, t))
        })
        .collect()
}

/// Zip của gói: tệp .zip giữ nguyên bản giao; thư mục đã giải nén → nén lại dưới thư mục gốc `<OBJID ':'→'_'>`.
async fn zip_goc(path: &Path, ex: &Extract, objid: &str) -> anyhow::Result<Vec<u8>> {
    if path.is_file() {
        return tokio::fs::read(path).await.with_context(|| format!("đọc {}", path.display()));
    }
    let root = objid.replace(':', "_");
    let mut w = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let opt = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated).large_file(true);
    for (p, b) in &ex.files {
        w.start_file(format!("{root}/{p}"), opt)?;
        std::io::Write::write_all(&mut w, b)?;
    }
    Ok(w.finish()?.into_inner())
}

/// Nhập một gói (tệp .zip hoặc thư mục). Lỗi hệ thống (CSDL, kho) trả `Err`; mọi kết luận về gói nằm trong `KetQuaGoi`
/// và đã ghi vào `ingest_items`.
pub async fn nhap_goi(ctx: &NhapCtx<'_>, path: &Path) -> anyhow::Result<KetQuaGoi> {
    let ten_tep = path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let p = path.to_path_buf();
    let (res, ex) = tokio::task::spawn_blocking(move || digitext_pkgcheck::check_and_extract(&p)).await.context("kiểm gói bị dừng")?;
    let kq = KetQuaGoi::moi(ten_tep, &res);
    let kq = match ex {
        None => kq.ket("tu_choi", "Không đọc được gói (thiếu METS.xml hoặc ZIP hỏng)"),
        Some(ex) => match nhap_noi_dung(ctx, path, &res, &ex, kq.clone()).await {
            Ok(k) => k,
            Err(e) => {
                tracing::error!(error = %e, goi = %kq.ten_tep, "nhập gói lỗi");
                kq.ket("loi", format!("Lỗi khi ghi: {e:#}"))
            }
        },
    };
    ghi_ket_qua(ctx, &kq, &res).await?;
    let _ = digitext_db::audit::append(
        ctx.pool,
        digitext_db::audit::Tp {
            actor_id: Some(ctx.user_id),
            device_id: ctx.device_id,
            action: "ingest.package",
            target: Some(kq.objid.clone()),
            detail: Some(json!({ "tenTep": kq.ten_tep, "ketQua": kq.ket_qua, "hoSoId": kq.ho_so_id, "soVanBan": kq.so_van_ban, "soChuaKy": kq.so_chua_ky })),
            ip: None,
        },
    )
    .await;
    Ok(kq)
}

async fn nhap_noi_dung(ctx: &NhapCtx<'_>, path: &Path, res: &PackageResult, ex: &Extract, mut kq: KetQuaGoi) -> anyhow::Result<KetQuaGoi> {
    let pool = ctx.pool;
    let pid = ctx.project_id;
    let kind = res.kind.as_str();
    if kind != "AIP" && kind != "SIP" {
        return Ok(kq.ket("tu_choi", format!("Chỉ nhận gói AIP hoặc SIP (gói này: {kind})")));
    }
    if res.objid.is_empty() {
        return Ok(kq.ket("tu_choi", "Gói không có OBJID"));
    }
    if res.pdfs.iter().any(|r| r.sig_valid == Some(false)) {
        return Ok(kq.ket("tu_choi", "Có tệp chữ ký số KHÔNG toàn vẹn (bị sửa sau khi ký) — không nhận"));
    }
    if res.errors > 0 && !ctx.cho_phep_loi {
        return Ok(kq.ket("tu_choi", format!("Kiểm gói KHÔNG ĐẠT ({} lỗi) — xem chi tiết; nhận cả gói lỗi thì bật tuỳ chọn tương ứng", res.errors)));
    }
    let ma_ho_so = ex.ho_so.get("arcFileCode").or_else(|| ex.ho_so.get("fileCode")).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).unwrap_or_else(|| res.file_code.clone());
    if ma_ho_so.is_empty() {
        return Ok(kq.ket("tu_choi", "EAD hồ sơ không có mã hồ sơ (arcFileCode / fileCode)"));
    }
    kq.ma_ho_so = ma_ho_so.clone();
    let da_co: Option<Uuid> = sqlx::query_scalar("SELECT ho_so_id FROM cl_ho_so_goi WHERE project_id = $1 AND objid = $2").bind(pid).bind(&res.objid).fetch_optional(pool).await?;
    if let Some(h) = da_co {
        kq.ho_so_id = Some(h);
        return Ok(kq.ket("da_co", "Gói này đã nhập trước đó"));
    }
    let khac: Option<String> = sqlx::query_scalar("SELECT objid FROM cl_ho_so_goi WHERE project_id = $1 AND lower(ma_ho_so_goc) = lower($2) LIMIT 1").bind(pid).bind(&ma_ho_so).fetch_optional(pool).await?;
    if let Some(o) = khac {
        return Ok(kq.ket("tu_choi", format!("Hồ sơ {ma_ho_so} đã nhập từ gói khác ({o}) — xoá hồ sơ đó trước nếu muốn thay bằng gói này")));
    }
    let Some(phong_ma) = ma_phong(&res.objid, &ma_ho_so) else {
        return Ok(kq.ket("tu_choi", "Không xác định được mã phông (OBJID không dạng urn:<mã phông>:uuid:…)"));
    };

    // 1. Zip gốc vào kho WORM + dòng packages (dùng lại nếu lần trước dừng giữa chừng).
    let zip = zip_goc(path, ex, &res.objid).await?;
    let sha = format!("{:x}", Sha256::digest(&zip));
    let tai_lieu = ex.files.contains_key("metadata/descriptive/EAD_TL.xml");
    let pkg_kind = match (kind, tai_lieu) {
        ("AIP", false) => "aip_hoso",
        ("AIP", true) => "aip_tailieu",
        (_, false) => "sip_hoso",
        (_, true) => "sip_tailieu",
    };
    let pkg_cu: Option<Uuid> = sqlx::query_scalar("SELECT id FROM packages WHERE project_id = $1 AND objid = $2 AND nguon = 'nhap' LIMIT 1").bind(pid).bind(&res.objid).fetch_optional(pool).await?;
    let pkg_id = pkg_cu.unwrap_or_else(Uuid::now_v7);
    let zip_key = digitext_storage::f_1x5(pid, pkg_id);
    ctx.store.put(&zip_key, &zip).await.with_context(|| format!("ghi zip gốc vào kho ({zip_key})"))?;
    if pkg_cu.is_none() {
        let zip_name = if path.is_file() { kq.ten_tep.clone() } else { format!("{}.zip", res.objid.replace(':', "_")) };
        sqlx::query(
            "INSERT INTO packages (id, project_id, requested_by, kind, status, ho_so_ref, label, request_json, objid, zip_key, zip_name, \
             zip_sha256, zip_size, document_count, nguon) VALUES ($1,$2,$3,$4,'built',$5,$6,$7,$8,$9,$10,$11,$12,$13,'nhap')",
        )
        .bind(pkg_id)
        .bind(pid)
        .bind(ctx.user_id)
        .bind(pkg_kind)
        .bind(&ma_ho_so)
        .bind(if res.title.is_empty() { ma_ho_so.clone() } else { res.title.clone() })
        .bind(json!({ "nguon": "nhap", "tenTep": kq.ten_tep, "batchId": ctx.batch_id, "ketLuanKiem": res.verdict, "nguoiTao": res.creator_org, "phanMem": res.creator_software }))
        .bind(&res.objid)
        .bind(&zip_key)
        .bind(&zip_name)
        .bind(&sha)
        .bind(zip.len() as i64)
        .bind(ex.docs.len() as i32)
        .execute(pool)
        .await?;
    }
    drop(zip);
    kq.package_id = Some(pkg_id);

    // 2. Phông.
    let phong_id: Option<Uuid> = sqlx::query_scalar("SELECT id FROM cl_phong WHERE project_id = $1 AND lower(ma_du_an) = lower($2) AND deleted_at IS NULL")
        .bind(pid)
        .bind(&phong_ma)
        .fetch_optional(pool)
        .await?;
    let phong_id = match phong_id {
        Some(id) => id,
        None => {
            let id = Uuid::now_v7();
            let ten = res.creator_org.rsplit_once(" (").map(|x| x.0).unwrap_or(&res.creator_org).trim().to_string();
            let mut row = Row::new();
            row.insert("ma_du_an".into(), json!(phong_ma));
            row.insert("ten".into(), json!(if ten.is_empty() { phong_ma.clone() } else { ten }));
            if let Err(e) = ghi(ctx, "cl_phong", id, row).await? {
                return Ok(kq.ket("loi", format!("Không tạo được phông {phong_ma}: {e}")));
            }
            id
        }
    };

    // 3. Hồ sơ.
    let h = &ex.ho_so;
    let so_tam = so_tam_tu_ma(&ma_ho_so, &phong_ma);
    let mut ghi_chu: Vec<String> = Vec::new();
    let maint = h.get("maintenance").map(|s| s.trim().to_string()).unwrap_or_default();
    let thbq: Option<Uuid> = if maint.is_empty() || maint == "07" {
        None
    } else {
        sqlx::query_scalar("SELECT id FROM cl_thbq WHERE (project_id IS NULL OR project_id = $1) AND ma_tt05 = $2 AND trang_thai IN (1, 2) AND deleted_at IS NULL ORDER BY so_tt NULLS LAST LIMIT 1")
            .bind(pid)
            .bind(&maint)
            .fetch_optional(pool)
            .await?
    };
    if thbq.is_none() && !maint.is_empty() {
        ghi_chu.push(format!("Thời hạn bảo quản theo gói: mã {maint}"));
    }
    let mut row = Row::new();
    row.insert("phong_id".into(), json!(phong_id));
    row.insert("so_tam".into(), json!(so_tam));
    if let Some(hs) = ma_ho_so.rsplit('.').next().filter(|x| !x.is_empty() && x.len() <= 8) {
        row.insert("ho_so_so".into(), json!(hs));
    }
    if let Some(t) = thbq {
        row.insert("thbq_id".into(), json!(t));
    }
    row.insert("tieu_de".into(), json!(h.get("title").map(|s| s.trim()).unwrap_or("")));
    chen(&mut row, "ngay_bat_dau", h.get("startDate"));
    chen(&mut row, "ngay_ket_thuc", h.get("endDate"));
    chen(&mut row, "tu_khoa", h.get("keyword"));
    chen_so(&mut row, "tong_so_vb", h.get("totalDoc"));
    chen(&mut row, "so_to", h.get("numberOfPaper"));
    chen_so(&mut row, "so_luong_trang", h.get("numberOfPage"));
    chen(&mut row, "che_do_su_dung", h.get("mode"));
    chen(&mut row, "ngon_ngu", h.get("language"));
    chen(&mut row, "ky_hieu_thong_tin", h.get("inforSign"));
    chen(&mut row, "muc_do_tin_cay", h.get("confidenceLevel"));
    chen(&mut row, "ma_ho_so_giay", h.get("paperFileCode"));
    if let Some(d) = h.get("description").map(|s| s.trim()).filter(|s| !s.is_empty()) {
        ghi_chu.insert(0, d.to_string());
    }
    ghi_chu.push(format!("Nhập từ gói {}", res.objid));
    row.insert("ghi_chu".into(), json!(ghi_chu.join("; ")));
    let hs_cu: Option<Uuid> = sqlx::query_scalar("SELECT id FROM cl_ho_so WHERE phong_id = $1 AND lower(btrim(so_tam)) = lower(btrim($2)) AND deleted_at IS NULL LIMIT 1")
        .bind(phong_id)
        .bind(&so_tam)
        .fetch_optional(pool)
        .await?;
    let ho_so_id = hs_cu.unwrap_or_else(Uuid::now_v7);
    if let Err(e) = ghi(ctx, "cl_ho_so", ho_so_id, row).await? {
        return Ok(kq.ket("loi", format!("Không ghi được hồ sơ {ma_ho_so}: {e}")));
    }
    kq.ho_so_id = Some(ho_so_id);

    // 4. Văn bản + tài liệu.
    let mut loi_vb: Vec<String> = Vec::new();
    let mut chua_ky = 0usize;
    let mut so_vb = 0usize;
    for (i, d) in ex.docs.iter().enumerate() {
        let f = &d.fields;
        let ma_vb = f.get("arcDocCode").or_else(|| f.get("docCode")).cloned().unwrap_or_default();
        let stt = stt_tu_ma(&ma_vb).unwrap_or(i as i32 + 1);
        let mut row = Row::new();
        row.insert("ho_so_id".into(), json!(ho_so_id));
        row.insert("so_thu_tu".into(), json!(stt));
        chen(&mut row, "loai_vb", f.get("typeName"));
        chen(&mut row, "so_van_ban", f.get("codeNumber"));
        chen(&mut row, "so_ky_hieu", f.get("codeNotation"));
        chen(&mut row, "ngay_thang", f.get("issuedDate"));
        chen(&mut row, "co_quan_ban_hanh", f.get("organName"));
        chen(&mut row, "trich_yeu", f.get("subject").or_else(|| f.get("title")));
        chen(&mut row, "ngon_ngu", f.get("language"));
        chen_so(&mut row, "so_trang", f.get("numberOfPage"));
        chen(&mut row, "ky_hieu_thong_tin", f.get("inforSign"));
        chen(&mut row, "tu_khoa", f.get("keyword"));
        chen(&mut row, "che_do_su_dung", f.get("mode"));
        chen(&mut row, "do_tin_cay", f.get("confidenceLevel"));
        chen(&mut row, "but_tich", f.get("autograph"));
        chen(&mut row, "ghi_chu", f.get("description"));
        let vb_cu: Option<Uuid> = sqlx::query_scalar("SELECT id FROM cl_van_ban WHERE ho_so_id = $1 AND so_thu_tu = $2 AND deleted_at IS NULL")
            .bind(ho_so_id)
            .bind(stt)
            .fetch_optional(pool)
            .await?;
        let vb_id = vb_cu.unwrap_or_else(Uuid::now_v7);
        if let Err(e) = ghi(ctx, "cl_van_ban", vb_id, row).await? {
            loi_vb.push(format!("{ma_vb}: {e}"));
            continue;
        }
        so_vb += 1;
        let Some(dp) = d.data_path.as_deref() else {
            loi_vb.push(format!("{ma_vb}: không có tệp dữ liệu"));
            continue;
        };
        let Some(bytes) = ex.files.get(dp) else { continue };
        let ten = dp.rsplit('/').next().unwrap_or(dp).to_string();
        let pdf_row = res.pdfs.iter().find(|r| r.file == ten);
        let ky_so = match pdf_row.map(|r| (r.signed, r.sig_valid)) {
            Some((true, Some(true))) => Some("hop_le"),
            Some((true, _)) => Some("khong_hop_le"),
            Some((false, _)) => Some("chua_ky"),
            None => None,
        };
        if ky_so == Some("chua_ky") {
            chua_ky += 1;
        }
        let doc_id = luu_tai_lieu(ctx, pkg_id, &phong_ma, &so_tam, &ten, bytes, pdf_row, ky_so).await?;
        let co_tep: Option<Uuid> = sqlx::query_scalar("SELECT id FROM cl_van_ban_file WHERE van_ban_id = $1 AND document_id = $2 AND deleted_at IS NULL")
            .bind(vb_id)
            .bind(doc_id)
            .fetch_optional(pool)
            .await?;
        if co_tep.is_none() {
            let mut fr = Row::new();
            fr.insert("van_ban_id".into(), json!(vb_id));
            fr.insert("document_id".into(), json!(doc_id));
            fr.insert("name".into(), json!(ten));
            fr.insert("path".into(), json!(dp));
            fr.insert("size".into(), json!(bytes.len()));
            fr.insert("format".into(), json!(ten.rsplit_once('.').map(|x| x.1.to_lowercase()).unwrap_or_default()));
            if let Some(n) = pdf_row.map(|r| r.pages).filter(|n| *n > 0) {
                fr.insert("so_trang".into(), json!(n));
            }
            fr.insert("so_tt".into(), json!(1));
            if let Err(e) = ghi(ctx, "cl_van_ban_file", Uuid::now_v7(), fr).await? {
                loi_vb.push(format!("{ten}: nối tệp: {e}"));
            }
        }
    }
    kq.so_van_ban = so_vb;
    kq.so_chua_ky = chua_ky;
    if !loi_vb.is_empty() {
        let n = loi_vb.len();
        let mau: Vec<String> = loi_vb.into_iter().take(5).collect();
        return Ok(kq.ket("loi", format!("{n} văn bản ghi lỗi (chạy lại để nhập tiếp): {}", mau.join(" · "))));
    }

    // 5. Dấu đã nhập xong.
    sqlx::query("INSERT INTO cl_ho_so_goi (ho_so_id, project_id, package_id, objid, ma_ho_so_goc, so_tai_lieu, so_chua_ky) VALUES ($1,$2,$3,$4,$5,$6,$7) ON CONFLICT (ho_so_id) DO NOTHING")
        .bind(ho_so_id)
        .bind(pid)
        .bind(pkg_id)
        .bind(&res.objid)
        .bind(&ma_ho_so)
        .bind(so_vb as i32)
        .bind(chua_ky as i32)
        .execute(pool)
        .await?;
    let mut tb = Vec::new();
    if hs_cu.is_some() {
        tb.push("ghép vào hồ sơ đã có cùng số".to_string());
    }
    if chua_ky > 0 {
        tb.push(format!("{chua_ky} tệp chưa ký số"));
    }
    if res.errors > 0 {
        tb.push(format!("nhận dù kiểm gói có {} lỗi", res.errors));
    }
    Ok(kq.ket("nhap", tb.join("; ")))
}

#[allow(clippy::too_many_arguments)]
async fn luu_tai_lieu(
    ctx: &NhapCtx<'_>,
    pkg_id: Uuid,
    phong_ma: &str,
    so_tam: &str,
    ten: &str,
    bytes: &[u8],
    pdf_row: Option<&digitext_pkgcheck::PdfRow>,
    ky_so: Option<&str>,
) -> anyhow::Result<Uuid> {
    let pool = ctx.pool;
    let pid = ctx.project_id;
    let b3 = blake3::hash(bytes).to_hex().to_string();
    let rel_path = format!("{phong_ma}/{so_tam}/{ten}");
    let cu: Option<Uuid> = sqlx::query_scalar("SELECT id FROM documents WHERE project_id = $1 AND blake3 = $2 AND filename = $3 AND COALESCE(rel_path, '') = $4")
        .bind(pid)
        .bind(&b3)
        .bind(ten)
        .bind(&rel_path)
        .fetch_optional(pool)
        .await?;
    let doc_id = cu.unwrap_or_else(Uuid::now_v7);
    let key = if ky_so.is_some_and(|k| k != "chua_ky") { digitext_storage::f_2fc(pid, doc_id) } else { digitext_storage::f_2qs(pid, doc_id) };
    ctx.store.put(&key, bytes).await.with_context(|| format!("ghi {ten} vào kho"))?;
    if cu.is_none() {
        let pages = pdf_row.map(|r| r.pages as i32).filter(|n| *n > 0);
        let pdfa_2b = pdf_row.and_then(|r| r.pdfa.as_deref()).map(|d| d.to_uppercase().contains("2B"));
        sqlx::query(
            "INSERT INTO documents (id, project_id, uploaded_by, filename, ho_so_ref, page_count, size_bytes, blake3, storage_path, rel_path, \
             status, pdfa_key, pdfa_pages, pdfa_2b, pdfa_at, goi_id, ky_so) \
             VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,'archived',$11,$12,$13,CASE WHEN $11::text IS NULL THEN NULL ELSE now() END,$14,$15)",
        )
        .bind(doc_id)
        .bind(pid)
        .bind(ctx.user_id)
        .bind(ten)
        .bind(so_tam)
        .bind(pages)
        .bind(bytes.len() as i64)
        .bind(&b3)
        .bind(&key)
        .bind(&rel_path)
        .bind(pdf_row.filter(|r| r.pdfa.is_some()).map(|_| key.clone()))
        .bind(pages)
        .bind(pdfa_2b)
        .bind(pkg_id)
        .bind(ky_so)
        .execute(pool)
        .await?;
    }
    if pdf_row.is_some() {
        let owned = bytes.to_vec();
        let trang = tokio::task::spawn_blocking(move || lop_chu(&owned)).await.unwrap_or_default();
        for (n, t) in trang {
            sqlx::query(
                "INSERT INTO ocr_results (id, document_id, page_no, engine_version_id, text_source, content) VALUES ($1,$2,$3,$4,'ocr_raw',$5) \
                 ON CONFLICT (document_id, page_no, engine_version_id, text_source) DO NOTHING",
            )
            .bind(Uuid::now_v7())
            .bind(doc_id)
            .bind(n)
            .bind(ENGINE_LOP_CHU)
            .bind(t)
            .execute(pool)
            .await?;
        }
    }
    Ok(doc_id)
}

/// Mô tả một tài liệu NHẬP TỪ GÓI để cấp DIP (bản Lưu trữ GĐ4): gói nhập không có `request_json` dựng gói, nên EAD văn bản
/// dựng lại từ mục lục đã nhập (cl_van_ban + hồ sơ + THBQ). Bản số worker lấy theo `documentId` như tài liệu thường.
pub struct DipTuGoi {
    pub doc: digitext_package::wire::Tq3,
    pub fond_code: String,
    pub archive_org_code: String,
    pub ma: String,
    pub trich_yeu: Option<String>,
}

/// "Tên cơ quan (MÃ)" → "MÃ".
fn ma_trong_ngoac(s: &str) -> Option<String> {
    let (_, sau) = s.trim().rsplit_once('(')?;
    let ma = sau.trim_end_matches(')').trim();
    (!ma.is_empty()).then(|| ma.to_string())
}

/// Chế độ sử dụng về mã TT05 (01 công khai · 02 hạn chế · 03 mật).
fn ma_che_do(s: Option<&str>) -> String {
    let s = s.map(str::trim).unwrap_or("");
    if matches!(s, "01" | "02" | "03") {
        return s.to_string();
    }
    let t = s.to_lowercase();
    if t.starts_with("hạn chế") || t.starts_with("han che") {
        "02".into()
    } else if t.starts_with("mật") {
        "03".into()
    } else {
        "01".into()
    }
}

#[derive(sqlx::FromRow)]
struct VanBanGoi {
    so_thu_tu: i32,
    loai_vb: Option<String>,
    so_van_ban: Option<String>,
    so_ky_hieu: Option<String>,
    ngay_thang: Option<String>,
    co_quan_ban_hanh: Option<String>,
    trich_yeu: Option<String>,
    ngon_ngu: Option<String>,
    so_trang: Option<i32>,
    ky_hieu_thong_tin: Option<String>,
    tu_khoa: Option<String>,
    che_do: Option<String>,
    do_tin_cay: Option<String>,
    but_tich: Option<String>,
    ghi_chu: Option<String>,
    thbq: Option<String>,
    ma_ho_so_goc: String,
    ma_phong: String,
    nguoi_tao: Option<String>,
    ten_tep: String,
}

pub async fn dip_tu_goi(pool: &PgPool, document_id: Uuid) -> sqlx::Result<Option<DipTuGoi>> {
    let r: Option<VanBanGoi> = sqlx::query_as(
        "SELECT v.so_thu_tu, v.loai_vb, v.so_van_ban, v.so_ky_hieu, v.ngay_thang, v.co_quan_ban_hanh, v.trich_yeu, v.ngon_ngu, \
                v.so_trang, v.ky_hieu_thong_tin, v.tu_khoa, COALESCE(NULLIF(btrim(v.che_do_su_dung), ''), h.che_do_su_dung) AS che_do, \
                v.do_tin_cay, v.but_tich, v.ghi_chu, t.ma_tt05 AS thbq, g.ma_ho_so_goc, p.ma_du_an AS ma_phong, \
                pk.request_json->>'nguoiTao' AS nguoi_tao, f.name AS ten_tep \
         FROM documents d \
         JOIN cl_van_ban_file f ON f.document_id = d.id AND f.deleted_at IS NULL \
         JOIN cl_van_ban v ON v.id = f.van_ban_id AND v.deleted_at IS NULL \
         JOIN cl_ho_so h ON h.id = v.ho_so_id \
         JOIN cl_phong p ON p.id = h.phong_id \
         JOIN cl_ho_so_goi g ON g.ho_so_id = h.id \
         LEFT JOIN packages pk ON pk.id = g.package_id \
         LEFT JOIN cl_thbq t ON t.id = h.thbq_id \
         WHERE d.id = $1 AND d.goi_id IS NOT NULL LIMIT 1",
    )
    .bind(document_id)
    .fetch_optional(pool)
    .await?;
    let Some(v) = r else { return Ok(None) };
    let ma = digitext_core::chinh_ly::ma_van_ban_tt05(&v.ma_ho_so_goc, v.so_thu_tu as i64);
    let meta = digitext_package::ead::T4y {
        doc_code: ma.clone(),
        maintenance: v.thbq.unwrap_or_else(|| "01".into()),
        type_name: v.loai_vb.unwrap_or_default(),
        code_number: v.so_van_ban,
        code_notation: v.so_ky_hieu,
        issued_date: v.ngay_thang,
        organ_name: v.co_quan_ban_hanh,
        subject: v.trich_yeu.clone(),
        language: v.ngon_ngu.map(|s| s.split([',', ';']).map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect()).unwrap_or_default(),
        number_of_page: v.so_trang.and_then(|n| u32::try_from(n).ok()),
        infor_sign: v.ky_hieu_thong_tin,
        keyword: v.tu_khoa,
        mode: ma_che_do(v.che_do.as_deref()),
        confidence_level: v.do_tin_cay,
        autograph: v.but_tich,
        description: v.ghi_chu,
        ..Default::default()
    };
    Ok(Some(DipTuGoi {
        doc: digitext_package::wire::Tq3 {
            document_id: Some(document_id),
            file_name: v.ten_tep,
            mimetype: Some("application/pdf".into()),
            metadata: digitext_package::ead::DocEad::Doc(meta),
            fetch: None,
        },
        archive_org_code: v.nguoi_tao.as_deref().and_then(ma_trong_ngoac).unwrap_or_else(|| v.ma_phong.clone()),
        fond_code: v.ma_phong,
        ma,
        trich_yeu: v.trich_yeu,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ma_phong_va_so_tam() {
        assert_eq!(ma_phong("urn:H49.132.02:uuid:0A6AC212-672F", "H49.132.02.2017.01.04").as_deref(), Some("H49.132.02"));
        assert_eq!(ma_phong("uuid-123", "H49.132.02.2017.01.04").as_deref(), Some("H49.132.02"));
        assert_eq!(ma_phong("uuid-123", "2017.01").as_deref(), None);
        assert_eq!(so_tam_tu_ma("H49.132.02.2017.01.04", "H49.132.02"), "2017.01.04");
        assert_eq!(so_tam_tu_ma("ABC.1", "H49"), "ABC.1");
        assert_eq!(stt_tu_ma("H49.132.02.2017.01.04.0000012"), Some(12));
        assert_eq!(stt_tu_ma("H49.132.02.2017.01.04.12"), None);
        assert_eq!(ma_trong_ngoac("UBND huyện Diễn Châu (H40.25)").as_deref(), Some("H40.25"));
        assert_eq!(ma_trong_ngoac("UBND huyện"), None);
        assert_eq!(ma_che_do(Some("Hạn chế")), "02");
        assert_eq!(ma_che_do(None), "01");
    }

    /// Lớp chữ của PDF trong gói thật:
    /// `DIGITEXT_PKG_SAMPLE=<gói.zip> cargo test -p digitext-api --lib thu_lop_chu -- --ignored --nocapture`
    #[test]
    #[ignore]
    fn thu_lop_chu() {
        let p = std::env::var("DIGITEXT_PKG_SAMPLE").unwrap();
        let (_, ex) = digitext_pkgcheck::check_and_extract(Path::new(&p));
        let ex = ex.unwrap();
        for d in ex.docs.iter().take(3) {
            let b = &ex.files[d.data_path.as_deref().unwrap()];
            let t = lop_chu(b);
            println!("{} trang có chữ: {:?}", t.len(), t.first().map(|(n, s)| (n, s.chars().take(160).collect::<String>())));
        }
    }
}
