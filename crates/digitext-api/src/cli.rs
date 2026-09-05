// Copyright (c) 2026 DigiText. All rights reserved.
use crate::app::{router, AppState};
use crate::config::Config;
use crate::jwt::JwtKeys;
use crate::password;
use anyhow::{bail, Context};
use digitext_core::Role;
use digitext_db::users;
use std::process::ExitCode;
use std::sync::Arc;
pub fn cli_main(r_1q: &[String]) -> ExitCode {
    let rt = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("không tạo được tokio runtime: {e}");
            return ExitCode::FAILURE;
        }
    };
    match rt.block_on(run(r_1q)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("Error: {e:?}");
            ExitCode::FAILURE
        }
    }
}
async fn run(r_1r: &[String]) -> anyhow::Result<()> {
    match r_1r.first().map(String::as_str) {
        None | Some("serve") => serve().await,
        Some("bootstrap-superadmin") => f_v2(&r_1r[1..]).await,
        Some("backfill-source-pdfs") => backfill_source_pdfs().await,
        Some(other) => {
            bail!(
                "lệnh không hợp lệ: {other} (dùng: serve | bootstrap-superadmin --username <tên> | backfill-source-pdfs)"
            )
        }
    }
}
async fn backfill_source_pdfs() -> anyhow::Result<()> {
    let r_1s = Config::from_env()?;
    let r_1t = r_1s
        .minio
        .clone()
        .context(
            "thiếu cấu hình MinIO (DIGITEXT_MINIO_*) — không thể backfill (bật source_store trước)",
        )?;
    let r_1u = digitext_db::connect(&r_1s.database_url)
        .await
        .context("kết nối Postgres thất bại")?;
    let store = digitext_storage::MinioObjectStore::new(&r_1t)
        .context("dựng MinIO object store thất bại")?;
    eprintln!(
        "→ backfill: DB ok, MinIO bucket={} endpoint={}, data_dir={}",
        r_1t.bucket, r_1t.endpoint, r_1s.data_dir
    );
    let r_1v = crate::backfill::backfill_source_pdfs(
        &r_1u,
        &store,
        std::path::Path::new(&r_1s.data_dir),
        true,
    )
    .await?;
    println!(
        "Backfill xong: tổng={} · đã đẩy={} · bỏ qua (đã có)={} · thiếu file={} · lỗi={}",
        r_1v.total, r_1v.uploaded, r_1v.skipped, r_1v.missing, r_1v.errors
    );
    if r_1v.errors > 0 {
        bail!(
            "có {} lỗi khi backfill — xem log",
            r_1v.errors
        );
    }
    Ok(())
}
async fn serve() -> anyhow::Result<()> {
    let mut r_26 = digitext_telemetry::init(&digitext_telemetry::TelemetryConfig::from_env(
        "digitext-api",
        env!("CARGO_PKG_VERSION"),
        "info,tower_http=debug",
    ));
    let r_1w = Config::from_env()?;
    let r_21 = digitext_db::f_11d(&r_1w.database_url)
        .await
        .context("kết nối/migrate Postgres thất bại")?;
    let r_1x = JwtKeys::load_or_generate(&r_1w.jwt_key_path)?;
    let r_1y = tokio::net::TcpListener::bind(&r_1w.bind)
        .await
        .with_context(|| format!("không bind được {}", r_1w.bind))?;
    tracing::info!(
        "digitext-api lắng nghe tại http://{}",
        r_1w.bind
    );
    let r_25 = r_21.clone();
    let r_22 = r_21.clone();
    let mut r_24 =
        AppState::new(r_21, Arc::new(r_1x), r_1w.data_dir).with_behind_proxy(r_1w.behind_proxy);
    let r_20 = digitext_storage::f_vr(
        r_1w.minio.as_ref(),
        r_1w.pdf_store_bucket.as_deref(),
        r_1w.pdf_store_dir.as_deref(),
    )
    .context("dựng kho PDF 2 lớp thất bại")?
    .map(|r_23| {
        Arc::new(digitext_storage::WormGuard::new(r_23)) as Arc<dyn digitext_storage::ObjectStore>
    });
    if r_1w.pdf_store_bucket.is_none() && r_1w.pdf_store_dir.is_some() {
        tracing::warn!(
            "kho PDF 2 lớp dùng ĐĨA CỤC BỘ — chỉ đúng khi worker OCR chạy CÙNG máy.              Worker ở máy khác thì đặt DIGITEXT_PDF_STORE_BUCKET (MinIO) cho cả hai."
        );
    }
    if let Some(store) = r_20 {
        r_24 = r_24.with_store(store);
        tracing::info!(
            minio = r_1w.pdf_store_bucket.is_some(),
            "luồng ký số + kiểm PDF/A BẬT (kho object cấu hình)"
        );
        f_2gp(r_25);
    }
    match r_1w.minio {
        Some(r_1z) => match digitext_storage::MinioObjectStore::new(&r_1z) {
            Ok(store) => {
                tracing::info!(
                    bucket = % r_1z.bucket, endpoint = % r_1z.endpoint,
                    "kho PDF nguồn MinIO BẬT (upload → OCR)"
                );
                r_24 = r_24.with_source_store(Arc::new(store));
            }
            Err(e) => {
                tracing::error!(
                    error = % e,
                    "dựng MinIO PDF nguồn lỗi — tài liệu upload sẽ KHÔNG OCR được"
                );
            }
        },
        None => {
            tracing::warn!(
                "MinIO PDF nguồn CHƯA cấu hình (DIGITEXT_MINIO_*) — tài liệu upload chỉ lưu đĩa API, worker OCR sẽ không fetch được"
            )
        }
    }
    match r_1w.sign_trust_roots.as_deref() {
        Some(path) => {
            let pem = std::fs::read_to_string(path)
                .unwrap_or_else(|e| {
                    panic!(
                        "không đọc được DIGITEXT_SIGN_TRUST_ROOTS='{path}': {e}"
                    )
                });
            let roots =
                digitext_sign_verify::chain::TrustStore::from_pem(&pem).unwrap_or_else(|e| {
                    panic!("DIGITEXT_SIGN_TRUST_ROOTS='{path}' không hợp lệ: {e}")
                });
            tracing::info!(
                path = % path, roots = roots.len(),
                "trust store chữ ký ĐÃ nạp"
            );
            r_24 = r_24.with_trust_roots(roots);
        }
        None => {
            tracing::warn!(
                "DIGITEXT_SIGN_TRUST_ROOTS chưa cấu hình — KHÔNG kiểm được chuỗi chứng thư nên mọi yêu cầu ký sẽ bị từ chối (đặt bundle PEM root của CA phát hành token để bật ký số; token I-CA dùng trust/ica-sign-trust-roots.pem trong repo)"
            )
        }
    }
    if let Some(r_27) = r_1w.tsa_url {
        tracing::info!(
            tsa = % r_27, "đóng dấu thời gian TSA BẬT (PAdES-T)"
        );
        r_24 = r_24.with_tsa_url(r_27);
    }
    match r_1w.tsa_trust_roots.as_deref() {
        Some(path) => {
            let pem = std::fs::read_to_string(path)
                .unwrap_or_else(|e| {
                    panic!(
                        "không đọc được DIGITEXT_TSA_TRUST_ROOTS='{path}': {e}"
                    )
                });
            let roots =
                digitext_sign_verify::chain::TrustStore::from_pem(&pem).unwrap_or_else(|e| {
                    panic!("DIGITEXT_TSA_TRUST_ROOTS='{path}' không hợp lệ: {e}")
                });
            tracing::info!(
                path = % path, roots = roots.len(),
                "trust store TSA ĐÃ nạp"
            );
            r_24 = r_24.with_tsa_trust_roots(roots);
        }
        None => {
            tracing::warn!(
                "DIGITEXT_TSA_TRUST_ROOTS chưa cấu hình — tem thời gian chỉ ở mức cấu trúc, KHÔNG tin được khi tái xác minh (đặt bundle PEM root TSA để bật kiểm chuỗi tem — H3)"
            )
        }
    }
    if r_1w.pades_lt {
        tracing::info!(
            "PAdES-LT BẬT — complete sẽ tải OCSP/CRL + nhúng /DSS (long-term validation)"
        );
        r_24 = r_24.with_pades_lt(true);
    }
    use crate::routes::sign::{f_1pw, LtaStartup};
    match f_1pw(r_1w.pades_lta, r_1w.pades_lt, r_24.tsa_url.is_some()) {
        LtaStartup::Enable => {
            tracing::info!(
                "PAdES-LTA BẬT — complete sẽ đóng DocTimeStamp phủ /DSS (archive-timestamp) sau LT"
            );
            r_24 = r_24.with_pades_lta(true);
        }
        LtaStartup::MissingLt => {
            tracing::error!(
                "DIGITEXT_PADES_LTA bật nhưng DIGITEXT_PADES_LT TẮT — LTA cần /DSS để phủ; BỎ QUA LTA (bật DIGITEXT_PADES_LT)"
            )
        }
        LtaStartup::MissingTsa => {
            tracing::error!(
                "DIGITEXT_PADES_LTA bật nhưng thiếu DIGITEXT_TSA_URL — LTA cần TSA (https, RSA) để đóng archive-timestamp; BỎ QUA LTA"
            )
        }
        LtaStartup::Off => {}
    }
    r_24 = r_24.with_sign_appearance(r_1w.sign_visible, r_1w.sign_stamp);
    if r_1w.sign_stamp {
        tracing::warn!(
            "DIGITEXT_SIGN_STAMP bật: ký kèm CON DẤU TRÒN — lệch Điều 8.1.c TT 05/2025 (Thông tư quy định khối text Times 10 và cấm hiển thị hình ảnh dấu)"
        );
    } else if !r_1w.sign_visible {
        tracing::warn!(
            "DIGITEXT_SIGN_VISIBLE tắt: ký VÔ HÌNH — lệch Điều 8.1.c TT 05/2025"
        );
    }
    crate::routes::documents::f_2id(r_24.data_dir.as_path()).await;
    f_2gr(r_24.data_dir.clone());
    f_2gs(r_22);
    axum::serve(
        r_1y,
        router(r_24).into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;
    r_26.shutdown();
    Ok(())
}
const C_K7: std::time::Duration = std::time::Duration::from_secs(60);
fn f_2gp(r_29: sqlx::PgPool) {
    tokio::spawn(async move {
        let mut r_2a = tokio::time::interval(C_K7);
        r_2a.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            r_2a.tick().await;
            match digitext_db::sign_sessions::f_272(&r_29).await {
                Ok(r_28) if r_28 > 0 => {
                    tracing::info!(
                        released = r_28,
                        "sweeper: release phiên ký hết hạn → to_sign"
                    )
                }
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!(
                        error = % e,
                        "sweeper phiên ký lỗi (sẽ thử lại)"
                    )
                }
            }
        }
    });
}
const C_FU: std::time::Duration = std::time::Duration::from_secs(6 * 3600);
fn f_2gr(data_dir: Arc<std::path::PathBuf>) {
    tokio::spawn(async move {
        let mut r_2b = tokio::time::interval(C_FU);
        r_2b.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            r_2b.tick().await;
            crate::routes::documents::f_2id(data_dir.as_path()).await;
        }
    });
}
const C_X: std::time::Duration = std::time::Duration::from_secs(3600);
const C_V: i32 = 48;
fn f_2gs(r_2d: sqlx::PgPool) {
    tokio::spawn(async move {
        let mut r_2e = tokio::time::interval(C_X);
        r_2e.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            r_2e.tick().await;
            match digitext_db::upload_batches::f_qf(&r_2d, C_V).await {
                Ok(r_2c) if r_2c > 0 => {
                    tracing::info!(
                        abandoned = r_2c,
                        hours = C_V,
                        "reaper: đóng đợt nộp bỏ dở → abandoned"
                    )
                }
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!(
                        error = % e,
                        "reaper đợt nộp lỗi (sẽ thử lại)"
                    )
                }
            }
        }
    });
}
fn f_1ye(r_2g: &[String]) -> anyhow::Result<(String, Option<String>)> {
    let mut username: Option<&str> = None;
    let mut r_2h: Option<&str> = None;
    let mut r_2i = r_2g.iter();
    while let Some(r_2f) = r_2i.next() {
        match r_2f.as_str() {
            "--username" => username = r_2i.next().map(String::as_str),
            "--display-name" => r_2h = r_2i.next().map(String::as_str),
            other => bail!("tham số không hợp lệ: {other}"),
        }
    }
    let username = username.context("thiếu --username <tên>")?;
    Ok((username.to_owned(), r_2h.map(str::to_owned)))
}
async fn f_v2(r_2j: &[String]) -> anyhow::Result<()> {
    let (username, r_2l) = f_1ye(r_2j)?;
    let r_2k = Config::from_env()?;
    let r_2m = digitext_db::f_11d(&r_2k.database_url).await?;
    if users::count(&r_2m).await? > 0 {
        bail!(
            "bảng users đã có dữ liệu — bootstrap-superadmin chỉ chạy trên hệ thống trống"
        );
    }
    let r_2n = rpassword::prompt_password("Mật khẩu super admin: ")?;
    let r_2o = rpassword::prompt_password("Nhập lại mật khẩu: ")?;
    if r_2n != r_2o {
        bail!("hai lần nhập mật khẩu không khớp");
    }
    if r_2n.chars().count() < 8 {
        bail!("mật khẩu phải tối thiểu 8 ký tự");
    }
    let r_2p = users::create(
        &r_2m,
        users::Te1 {
            username: &username,
            email: None,
            display_name: r_2l.as_deref().unwrap_or(&username),
            pwd_hash: &password::hash(&r_2n)?,
            role: Role::SuperAdmin,
            project_id: None,
            created_by: None,
        },
    )
    .await?;
    println!(
        "Đã tạo super admin '{}' (id {}).",
        r_2p.username, r_2p.id
    );
    Ok(())
}
