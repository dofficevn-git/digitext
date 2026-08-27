// Copyright (c) 2026 DigiText. All rights reserved.
use crate::{
    cli,
    config::{
        FieldsEngine, PageMode, WorkerConfig, C_5K, C_5M, C_5P, C_5R, C_6G, C_6H,
        ENV_FIELDS_ENGINE, ENV_VLM_ENDPOINT, ENV_VLM_MODEL,
    },
    corrector::Lexicon,
    grpc::{GrpcTritonInfer, GrpcTritonProbe},
    health,
    metrics::Metrics,
    package_worker,
    readiness::Readiness,
    render::{
        ImagePageSource, LocalDirPageBytes, MinioPageBytes, PageBytesSource, PdfMinioPageBytes,
    },
    serve::{self, AppState},
    vllm::VllmCorrector,
    vlm::{VlmFieldExtractor, VlmFieldExtractorV2},
    worker::{self, Worker},
};
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
fn help_text(prog: &str) -> String {
    format!(
        "{prog} — worker OCR (DigiText, §G/§P)\n\nCách dùng:\n  {prog} [run]      Chạy worker (mặc định; vòng lặp job đầy đủ ở P-M2)\n  {prog} --health   In báo cáo sức khoẻ JSON rồi thoát (0=Ok, 1=Down)\n  {prog} --version  In phiên bản\n  {prog} --help     In trợ giúp\n\nCấu hình qua biến môi trường DIGITEXT_* (bắt buộc: DIGITEXT_DATABASE_URL,\nDIGITEXT_TRITON_ENDPOINT). Xem module `config` để biết đầy đủ khoá + mặc định.\n"
    )
}
pub fn cli_main(prog: &str, r_33h: &[String]) -> ExitCode {
    match cli::parse(std::iter::once(prog).chain(r_33h.iter().map(String::as_str))) {
        cli::Command::Help => {
            print!("{}", help_text(prog));
            ExitCode::SUCCESS
        }
        cli::Command::Version => {
            println!("{prog} {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        cli::Command::Health => f_2ap(),
        cli::Command::Run => f_2bh(),
        cli::Command::Unknown(arg) => {
            eprintln!("tham số không rõ: {arg}\n");
            print!("{}", help_text(prog));
            ExitCode::from(2)
        }
    }
}
fn f_2ap() -> ExitCode {
    let r_33i = match WorkerConfig::from_env() {
        Ok(_) => health::Check::ok("config", true),
        Err(e) => health::Check::fail("config", true, e.to_string()),
    };
    let r_33j = health::HealthReport::aggregate(vec![r_33i]);
    println!("{}", r_33j.to_json());
    if r_33j.is_ok() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
fn f_2bh() -> ExitCode {
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(r_33n) => r_33n,
        Err(e) => {
            eprintln!("không tạo được tokio runtime: {e}");
            return ExitCode::FAILURE;
        }
    };
    runtime
        .block_on(async move {
            let _telemetry = digitext_telemetry::init(
                &digitext_telemetry::TelemetryConfig::from_env(
                    "digitext-ocr",
                    env!("CARGO_PKG_VERSION"),
                    "info",
                ),
            );
            let cfg = match WorkerConfig::from_env() {
                Ok(cfg) => cfg,
                Err(e) => {
                    tracing::error!(
                        error = % e,
                        "cấu hình sai — dừng (fail-fast §P1)"
                    );
                    return ExitCode::FAILURE;
                }
            };
            let r_33k: std::net::SocketAddr = cfg
                .metrics_addr
                .parse()
                .expect("metrics_addr đã validate");
            let shutdown = CancellationToken::new();
            f_2gq(shutdown.clone());
            let r_33q = Arc::new(AtomicBool::new(false));
            let r_33r = if cfg.worker_enabled {
                match f_2gu(&cfg, shutdown.clone(), r_33q.clone()).await {
                    Ok(handles) => Some(handles),
                    Err(e) => {
                        tracing::error!(
                            error = % e,
                            "không khởi động được worker loop"
                        );
                        return ExitCode::FAILURE;
                    }
                }
            } else {
                tracing::info!(
                    "worker loop TẮT (DIGITEXT_WORKER_ENABLED=false) — chỉ phục vụ HTTP"
                );
                None
            };
            let r_33m = match GrpcTritonProbe::connect_lazy(&cfg.triton_endpoint) {
                Ok(r_33l) => r_33l,
                Err(e) => {
                    tracing::error!(
                        error = % e,
                        "endpoint gRPC Triton không hợp lệ"
                    );
                    return ExitCode::FAILURE;
                }
            };
            let r_33p = AppState {
                metrics: Arc::new(Metrics::new()),
                readiness: Arc::new(
                    Readiness::new(r_33m, Readiness::<GrpcTritonProbe>::default_models()),
                ),
            };
            tracing::info!(
                % r_33k, triton = % cfg.triton_endpoint, worker = cfg.worker_enabled,
                "phục vụ /healthz /readyz (metric đẩy OTLP về SigNoz)"
            );
            let r_33o = tokio::select! {
                r = serve::serve(r_33k, r_33p) => r, _ = shutdown.cancelled() => Ok(()),
            };
            shutdown.cancel();
            if let Some(handles) = r_33r {
                for h in handles {
                    let _ = h.await;
                }
            }
            if r_33q.load(Ordering::SeqCst) {
                tracing::error!(
                    "worker loop lỗi nghiêm trọng — thoát FAILURE để systemd restart"
                );
                return ExitCode::FAILURE;
            }
            match r_33o {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => {
                    tracing::error!(error = % e, "server HTTP dừng do lỗi");
                    ExitCode::FAILURE
                }
            }
        })
}
type Tq6 = Vec<tokio::task::JoinHandle<()>>;
fn f_2re(r_33s: bool, r_33t: bool) -> bool {
    r_33s && !r_33t
}
async fn f_2gu(
    cfg: &WorkerConfig,
    shutdown: CancellationToken,
    r_34x: Arc<AtomicBool>,
) -> Result<Tq6, String> {
    let r_34m = digitext_db::connect(&cfg.database_url)
        .await
        .map_err(|e| format!("kết nối Postgres: {e}"))?;
    let engine = digitext_db::engine_versions::f_ql(&r_34m, "recognition")
        .await
        .map_err(|e| format!("đọc engine_versions: {e}"))?
        .ok_or_else(|| {
            "chưa đăng ký engine 'recognition' trong bảng engine_versions"
                .to_owned()
        })?;
    let r_33z = engine.id;
    let r_343: Option<(VlmFieldExtractorV2, uuid::Uuid)> = if cfg.fields_engine == FieldsEngine::V2
    {
        let ep = std::env::var(ENV_VLM_ENDPOINT).unwrap_or_default();
        let model = std::env::var(ENV_VLM_MODEL).unwrap_or_default();
        if ep.is_empty() || model.is_empty() {
            return Err(
                format!(
                    "{ENV_FIELDS_ENGINE}=v2 đòi {ENV_VLM_ENDPOINT}IHbDoCA={ENV_VLM_MODEL} (engine chính không được no-op âm thầm)"
                ),
            );
        }
        let timeout = std::env::var(C_6H)
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .map(Duration::from_millis)
            .unwrap_or_else(|| Duration::from_secs(120));
        let v2_engine = digitext_db::engine_versions::ensure(
            &r_34m,
            "qwen2.5-vl-7b-awq-kie",
            "fallback",
            "none",
        )
        .await
        .map_err(|e| format!("đăng ký engine v2: {e}"))?;
        let extractor = VlmFieldExtractorV2::new(ep, model, timeout)
            .map_err(|e| format!("dựng VLM extractor v2: {e}"))?;
        tracing::info!(
            engine = % v2_engine.id,
            "bóc 16 trường v2 (VLM-primary) BẬT"
        );
        Some((extractor, v2_engine.id))
    } else {
        None
    };
    let r_34u = r_34m.clone();
    let r_34v = shutdown.clone();
    let r_34t = cfg.max_retries as i32;
    let r_34s = tokio::spawn(async move {
        let _ = worker::f_2b8(&r_34u, Duration::from_secs(30), r_34t, &r_34v).await;
    });
    let r_34p = r_34m.clone();
    let r_34q = shutdown.clone();
    let r_34o = tokio::spawn(async move {
        if let Err(e) = package_worker::f_2bd(&r_34p, Duration::from_secs(3600), &r_34q).await {
            tracing::error!(
                error = % e,
                "sweeper nhắc SLA lỗi DB — dừng để tầng trên restart"
            );
        }
    });
    let r_34f = digitext_storage::f_vr(
        cfg.minio.as_ref(),
        cfg.pdf_store_bucket.as_deref(),
        cfg.pdf_store_dir.as_deref(),
    )
    .map_err(|e| format!("kho PDF 2 lớp: {e}"))?
    .map(|s| {
        std::sync::Arc::new(digitext_storage::WormGuard::new(s))
            as std::sync::Arc<dyn digitext_storage::ObjectStore>
    });
    match r_34f.as_ref() {
        Some(_) if cfg.pdf_store_bucket.is_some() => {
            tracing::info!(
                bucket = cfg.pdf_store_bucket.as_deref().unwrap_or(""),
                "kho PDF 2 lớp: MinIO (API máy khác đọc được)"
            )
        }
        Some(_) => {
            tracing::warn!(
                dir = cfg.pdf_store_dir.as_deref().unwrap_or(""),
                "kho PDF 2 lớp: ĐĨA CỤC BỘ — API trên máy khác KHÔNG đọc được (kiểm PDF/A + ký số sẽ 503).              Đặt DIGITEXT_PDF_STORE_BUCKET để dùng MinIO."
            )
        }
        None => {
            tracing::warn!(
                "kho PDF 2 lớp CHƯA cấu hình — make_pdf2 sẽ không PUT"
            )
        }
    }
    let r_34b = r_34f.clone().map(|store| {
        let r_34c = r_34m.clone();
        let r_34d = shutdown.clone();
        tokio::spawn(async move {
            if let Err(e) =
                package_worker::f_2b5(&r_34c, &*store, Duration::from_secs(24 * 3600), 200, &r_34d)
                    .await
            {
                tracing::error!(
                    error = % e,
                    "sweeper bảo quản lỗi DB — dừng để restart"
                );
            }
        })
    });
    let r_33x = r_34m.clone();
    let r_33y = shutdown.clone();
    let r_33w = tokio::spawn(async move {
        if let Err(e) = package_worker::f_2an(&r_33x, Duration::from_secs(24 * 3600), &r_33y).await
        {
            tracing::error!(
                error = % e,
                "sweeper danh mục hủy lỗi DB — dừng để restart"
            );
        }
    });
    let r_342 = r_34f
        .clone()
        .map(|store| {
            let r_340 = r_34m.clone();
            let r_341 = shutdown.clone();
            tokio::spawn(async move {
                if let Err(e) = package_worker::f_2am(
                        &r_340,
                        &*store,
                        Duration::from_secs(3600),
                        &r_341,
                    )
                    .await
                {
                    tracing::error!(
                        error = % e,
                        "sweeper hết hạn khai thác lỗi DB — dừng để restart"
                    );
                }
            })
        });
    let r_34g = match r_34f.clone() {
        Some(store) => {
            let r_34j = r_34m.clone();
            let r_34k = shutdown.clone();
            let r_34h = cfg.lease.as_secs() as i64;
            let r_34i = cfg.max_retries as i32;
            let r_34l = format!("{}-pkg", cfg.worker_id);
            let clamd = std::env::var("DIGITEXT_CLAMD_ADDR")
                .ok()
                .filter(|r_33u| !r_33u.is_empty())
                .map(package_worker::ClamdScanner::new);
            Some(tokio::spawn(async move {
                let w = package_worker::PackageWorker::new(&r_34j, &*store, &r_34l, r_34h);
                let w = match &clamd {
                    Some(r_34n) => {
                        tracing::info!("quét virus gói: clamd bật");
                        w.with_scanner(r_34n)
                    }
                    None => w,
                };
                if let Err(e) = w.run_loop(Duration::from_secs(5), r_34i, &r_34k).await {
                    tracing::error!(
                        error = % e,
                        "vòng lặp đóng gói lỗi DB — dừng để tầng trên restart"
                    );
                }
            }))
        }
        None => {
            tracing::info!(
                "vòng lặp đóng gói TẮT (chưa cấu hình kho PDF) — yêu cầu packages sẽ chờ"
            );
            None
        }
    };
    let cfg = cfg.clone();
    let r_347 = tokio::spawn(async move {
        let infer = match GrpcTritonInfer::connect_lazy(&cfg.triton_endpoint, "ocr_pipeline") {
            Ok(r_345) => r_345,
            Err(e) => {
                tracing::error!(error = % e, "dựng gRPC infer client");
                return;
            }
        };
        let r_34e = match &cfg.minio {
            Some(r_349) => match digitext_storage::MinioObjectStore::new(r_349) {
                Ok(store) => {
                    let store = Arc::new(store);
                    match cfg.page_mode {
                        PageMode::Pdf => {
                            tracing::info!(
                                bucket = % r_349.bucket, raster_dpi = cfg.pdf_raster_dpi,
                                "nguồn ảnh trang: MinIO (PDF→tách trang; trang không có JPEG nhúng thì raster hoá)"
                            );
                            PageBytesSource::Pdf(PdfMinioPageBytes::new(
                                store,
                                r_34m.clone(),
                                crate::render::Tid {
                                    dpi: cfg.pdf_raster_dpi,
                                    pdftoppm_bin: cfg.pdftoppm_bin.clone(),
                                },
                            ))
                        }
                        PageMode::Images => {
                            tracing::info!(
                                bucket = % r_349.bucket,
                                "nguồn ảnh trang: MinIO (ảnh trang rời)"
                            );
                            PageBytesSource::Minio(MinioPageBytes::new(store))
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(
                        error = % e, "dựng MinIO object store"
                    );
                    return;
                }
            },
            None => {
                tracing::info!(
                    dir = % cfg.page_source_dir,
                    "nguồn ảnh trang: thư mục cục bộ"
                );
                PageBytesSource::Local(LocalDirPageBytes::new(&cfg.page_source_dir))
            }
        };
        let r_34r = ImagePageSource::new(r_34e, cfg.tile_size, cfg.tile_overlap);
        let store = r_34f.clone();
        let r_34w = if cfg.vlm_fallback_enabled {
            match (
                std::env::var(ENV_VLM_ENDPOINT),
                std::env::var(ENV_VLM_MODEL),
            ) {
                (Ok(ep), Ok(model)) if !ep.is_empty() && !model.is_empty() => {
                    let timeout = std::env::var(C_6H)
                        .ok()
                        .and_then(|s| s.parse::<u64>().ok())
                        .map(std::time::Duration::from_millis)
                        .unwrap_or_else(|| std::time::Duration::from_secs(60));
                    let r_348 = std::env::var(C_6G)
                        .ok()
                        .and_then(|s| s.parse::<u32>().ok())
                        .unwrap_or(crate::vlm::DEFAULT_MAX_IMAGE_SIDE);
                    match VlmFieldExtractor::new(ep, model, timeout)
                        .map(|v| v.with_max_image_side(r_348))
                    {
                        Ok(v) => Some(v),
                        Err(e) => {
                            tracing::error!(
                                error = % e,
                                "VLM Mode 2: dựng extractor lỗi — chạy KHÔNG VLM"
                            );
                            None
                        }
                    }
                }
                _ => {
                    tracing::warn!(
                        "VLM Mode 2 bật nhưng thiếu {}/{} — chạy KHÔNG VLM",
                        ENV_VLM_ENDPOINT, ENV_VLM_MODEL
                    );
                    None
                }
            }
        } else {
            None
        };
        let r_34a = Metrics::new();
        let lexicon = f_1os();
        let corrector = f_v9(cfg.llm_corrector_enabled);
        tracing::info!(
            worker_id = % cfg.worker_id, engine = % r_33z, r_34f = store.is_some(), r_34w
            = r_34w.is_some(), corrector = corrector.is_some(), fields_v2 = r_343
            .is_some(), "vòng lặp worker bắt đầu"
        );
        let r_346 = Duration::from_millis(500);
        let run = match &corrector {
            Some(r_33v) => {
                let mut w = Worker::with_corrector(
                    &r_34m,
                    &r_34r,
                    &infer,
                    &cfg,
                    &cfg.worker_id,
                    r_33z,
                    r_33v,
                    &lexicon,
                );
                if let Some(s) = &store {
                    w = w.with_store(&**s);
                }
                if let Some(v) = &r_34w {
                    w = w.with_vlm(v);
                }
                if let Some((v2, v2_engine)) = &r_343 {
                    w = w.with_fields_v2(v2, *v2_engine);
                }
                w.with_metrics(&r_34a).run_loop(r_346, &shutdown).await
            }
            None => {
                let mut w = Worker::new(&r_34m, &r_34r, &infer, &cfg, &cfg.worker_id, r_33z);
                if let Some(s) = &store {
                    w = w.with_store(&**s);
                }
                if let Some(v) = &r_34w {
                    w = w.with_vlm(v);
                }
                if let Some((v2, v2_engine)) = &r_343 {
                    w = w.with_fields_v2(v2, *v2_engine);
                }
                w.with_metrics(&r_34a).run_loop(r_346, &shutdown).await
            }
        };
        if let Err(e) = run {
            if f_2re(true, shutdown.is_cancelled()) {
                tracing::error!(
                    error = % e,
                    "vòng lặp worker lỗi DB — thoát để systemd restart"
                );
                r_34x.store(true, Ordering::SeqCst);
                shutdown.cancel();
            } else {
                tracing::info!(
                    error = % e,
                    "vòng lặp worker dừng theo shutdown chủ động"
                );
            }
        }
    });
    let mut r_344 = vec![r_347, r_34s, r_34o, r_33w];
    if let Some(h) = r_34b {
        r_344.push(h);
    }
    if let Some(h) = r_342 {
        r_344.push(h);
    }
    if let Some(h) = r_34g {
        r_344.push(h);
    }
    Ok(r_344)
}
fn f_1os() -> Lexicon {
    match std::env::var(C_5K) {
        Ok(r_34z) if !r_34z.is_empty() => match std::fs::read_to_string(&r_34z) {
            Ok(r_350) => {
                let r_351: Vec<&str> = r_350
                    .lines()
                    .map(str::trim)
                    .filter(|r_34y| !r_34y.is_empty())
                    .collect();
                tracing::info!(
                    r_34z = % r_34z, r_351 = r_351.len(),
                    "nạp lexicon corrector (§G7)"
                );
                Lexicon::from_words(r_351)
            }
            Err(e) => {
                tracing::warn!(
                    r_34z = % r_34z, error = % e,
                    "đọc lexicon lỗi — dùng lexicon rỗng"
                );
                Lexicon::from_words(Vec::<String>::new())
            }
        },
        _ => Lexicon::from_words(Vec::<String>::new()),
    }
}
fn f_v9(r_353: bool) -> Option<VllmCorrector> {
    if !r_353 {
        return None;
    }
    match (std::env::var(C_5M), std::env::var(C_5P)) {
        (Ok(r_354), Ok(r_355)) if !r_354.is_empty() && !r_355.is_empty() => {
            let r_357 = std::env::var(C_5R)
                .ok()
                .and_then(|r_356| r_356.parse::<u64>().ok())
                .map(Duration::from_millis)
                .unwrap_or_else(|| Duration::from_secs(30));
            match VllmCorrector::new(r_354, r_355, r_357) {
                Ok(r_352) => Some(r_352),
                Err(e) => {
                    tracing::error!(
                        error = % e,
                        "corrector §G6: dựng vLLM client lỗi — chạy KHÔNG corrector"
                    );
                    None
                }
            }
        }
        _ => {
            tracing::warn!(
                "corrector §G6 BẬT nhưng thiếu {}/{} — chạy KHÔNG corrector (cờ không còn no-op âm thầm)",
                C_5M, C_5P
            );
            None
        }
    }
}
fn f_2gq(shutdown: CancellationToken) {
    tokio::spawn(async move {
        f_2qn().await;
        tracing::info!("nhận tín hiệu dừng — graceful shutdown");
        shutdown.cancel();
    });
}
async fn f_2qn() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let mut r_359 = signal(SignalKind::terminate()).expect("đăng ký SIGTERM");
        let mut r_358 = signal(SignalKind::interrupt()).expect("đăng ký SIGINT");
        tokio::select! {
            _ = r_359.recv() => {} _ = r_358.recv() => {}
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}
