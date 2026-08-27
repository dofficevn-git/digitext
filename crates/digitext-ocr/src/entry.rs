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
        "{prog}IOKAlCB3b3JrZXIgT0NSIChEaWdpVGV4dCwgwqdHL8KnUCkKCkPDoWNoIGTDuW5nOgogIA=={prog}IFtydW5dICAgICAgQ2jhuqF5IHdvcmtlciAobeG6t2MgxJHhu4tuaDsgdsOybmcgbOG6t3Agam9iIMSR4bqneSDEkeG7pyDhu58gUC1NMikKICA={prog}IC0taGVhbHRoICAgSW4gYsOhbyBjw6FvIHPhu6ljIGtob+G6uyBKU09OIHLhu5NpIHRob8OhdCAoMD1PaywgMT1Eb3duKQogIA=={prog}IC0tdmVyc2lvbiAgSW4gcGhpw6puIGLhuqNuCiAg{prog}IC0taGVscCAgICAgSW4gdHLhu6MgZ2nDunAKCkPhuqV1IGjDrG5oIHF1YSBiaeG6v24gbcO0aSB0csaw4budbmcgRElHSVRFWFRfKiAoYuG6r3QgYnXhu5ljOiBESUdJVEVYVF9EQVRBQkFTRV9VUkwsCkRJR0lURVhUX1RSSVRPTl9FTkRQT0lOVCkuIFhlbSBtb2R1bGUgYGNvbmZpZ2AgxJHhu4MgYmnhur90IMSR4bqneSDEkeG7pyBraG/DoSArIG3hurdjIMSR4buLbmguCg=="
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
            eprintln!("dGhhbSBz4buRIGtow7RuZyByw7U6IA=={arg}\n");
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
            eprintln!("a2jDtG5nIHThuqFvIMSRxrDhu6NjIHRva2lvIHJ1bnRpbWU6IA=={e}");
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
                        "Y+G6pXUgaMOsbmggc2FpIOKAlCBk4burbmcgKGZhaWwtZmFzdCDCp1AxKQ=="
                    );
                    return ExitCode::FAILURE;
                }
            };
            let r_33k: std::net::SocketAddr = cfg
                .metrics_addr
                .parse()
                .expect("bWV0cmljc19hZGRyIMSRw6MgdmFsaWRhdGU=");
            let shutdown = CancellationToken::new();
            f_2gq(shutdown.clone());
            let r_33q = Arc::new(AtomicBool::new(false));
            let r_33r = if cfg.worker_enabled {
                match f_2gu(&cfg, shutdown.clone(), r_33q.clone()).await {
                    Ok(handles) => Some(handles),
                    Err(e) => {
                        tracing::error!(
                            error = % e,
                            "a2jDtG5nIGto4bufaSDEkeG7mW5nIMSRxrDhu6NjIHdvcmtlciBsb29w"
                        );
                        return ExitCode::FAILURE;
                    }
                }
            } else {
                tracing::info!(
                    "d29ya2VyIGxvb3AgVOG6rlQgKERJR0lURVhUX1dPUktFUl9FTkFCTEVEPWZhbHNlKSDigJQgY2jhu4kgcGjhu6VjIHbhu6UgSFRUUA=="
                );
                None
            };
            let r_33m = match GrpcTritonProbe::connect_lazy(&cfg.triton_endpoint) {
                Ok(r_33l) => r_33l,
                Err(e) => {
                    tracing::error!(
                        error = % e,
                        "ZW5kcG9pbnQgZ1JQQyBUcml0b24ga2jDtG5nIGjhu6NwIGzhu4c="
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
                "cGjhu6VjIHbhu6UgL2hlYWx0aHogL3JlYWR5eiAobWV0cmljIMSR4bqpeSBPVExQIHbhu4EgU2lnTm96KQ=="
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
                    "d29ya2VyIGxvb3AgbOG7l2kgbmdoacOqbSB0cuG7jW5nIOKAlCB0aG/DoXQgRkFJTFVSRSDEkeG7gyBzeXN0ZW1kIHJlc3RhcnQ="
                );
                return ExitCode::FAILURE;
            }
            match r_33o {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => {
                    tracing::error!(error = % e, "c2VydmVyIEhUVFAgZOG7q25nIGRvIGzhu5dp");
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
        .map_err(|e| format!("a+G6v3QgbuG7kWkgUG9zdGdyZXM6IA=={e}"))?;
    let engine = digitext_db::engine_versions::f_ql(&r_34m, "recognition")
        .await
        .map_err(|e| format!("xJHhu41jIGVuZ2luZV92ZXJzaW9uczog{e}"))?
        .ok_or_else(|| {
            "Y2jGsGEgxJHEg25nIGvDvSBlbmdpbmUgJ3JlY29nbml0aW9uJyB0cm9uZyBi4bqjbmcgZW5naW5lX3ZlcnNpb25z"
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
                    "{ENV_FIELDS_ENGINE}PXYyIMSRw7JpIA=={ENV_VLM_ENDPOINT}IHbDoCA={ENV_VLM_MODEL}IChlbmdpbmUgY2jDrW5oIGtow7RuZyDEkcaw4bujYyBuby1vcCDDom0gdGjhuqdtKQ=="
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
        .map_err(|e| format!("xJHEg25nIGvDvSBlbmdpbmUgdjI6IA=={e}"))?;
        let extractor = VlmFieldExtractorV2::new(ep, model, timeout)
            .map_err(|e| format!("ZOG7sW5nIFZMTSBleHRyYWN0b3IgdjI6IA=={e}"))?;
        tracing::info!(
            engine = % v2_engine.id,
            "YsOzYyAxNiB0csaw4budbmcgdjIgKFZMTS1wcmltYXJ5KSBC4bqsVA=="
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
                "c3dlZXBlciBuaOG6r2MgU0xBIGzhu5dpIERCIOKAlCBk4burbmcgxJHhu4MgdOG6p25nIHRyw6puIHJlc3RhcnQ="
            );
        }
    });
    let r_34f = digitext_storage::f_vr(
        cfg.minio.as_ref(),
        cfg.pdf_store_bucket.as_deref(),
        cfg.pdf_store_dir.as_deref(),
    )
    .map_err(|e| format!("a2hvIFBERiAyIGzhu5twOiA={e}"))?
    .map(|s| {
        std::sync::Arc::new(digitext_storage::WormGuard::new(s))
            as std::sync::Arc<dyn digitext_storage::ObjectStore>
    });
    match r_34f.as_ref() {
        Some(_) if cfg.pdf_store_bucket.is_some() => {
            tracing::info!(
                bucket = cfg.pdf_store_bucket.as_deref().unwrap_or(""),
                "a2hvIFBERiAyIGzhu5twOiBNaW5JTyAoQVBJIG3DoXkga2jDoWMgxJHhu41jIMSRxrDhu6NjKQ=="
            )
        }
        Some(_) => {
            tracing::warn!(
                dir = cfg.pdf_store_dir.as_deref().unwrap_or(""),
                "a2hvIFBERiAyIGzhu5twOiDEkMSoQSBD4bukQyBC4buYIOKAlCBBUEkgdHLDqm4gbcOheSBraMOhYyBLSMOUTkcgxJHhu41jIMSRxrDhu6NjIChraeG7g20gUERGL0EgKyBrw70gc+G7kSBz4bq9IDUwMykuICAgICAgICAgICAgICDEkOG6t3QgRElHSVRFWFRfUERGX1NUT1JFX0JVQ0tFVCDEkeG7gyBkw7luZyBNaW5JTy4="
            )
        }
        None => {
            tracing::warn!(
                "a2hvIFBERiAyIGzhu5twIENIxq9BIGPhuqV1IGjDrG5oIOKAlCBtYWtlX3BkZjIgc+G6vSBraMO0bmcgUFVU"
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
                    "c3dlZXBlciBi4bqjbyBxdeG6o24gbOG7l2kgREIg4oCUIGThu6tuZyDEkeG7gyByZXN0YXJ0"
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
                "c3dlZXBlciBkYW5oIG3hu6VjIGjhu6d5IGzhu5dpIERCIOKAlCBk4burbmcgxJHhu4MgcmVzdGFydA=="
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
                        "c3dlZXBlciBo4bq/dCBo4bqhbiBraGFpIHRow6FjIGzhu5dpIERCIOKAlCBk4burbmcgxJHhu4MgcmVzdGFydA=="
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
                        tracing::info!("cXXDqXQgdmlydXMgZ8OzaTogY2xhbWQgYuG6rXQ=");
                        w.with_scanner(r_34n)
                    }
                    None => w,
                };
                if let Err(e) = w.run_loop(Duration::from_secs(5), r_34i, &r_34k).await {
                    tracing::error!(
                        error = % e,
                        "dsOybmcgbOG6t3AgxJHDs25nIGfDs2kgbOG7l2kgREIg4oCUIGThu6tuZyDEkeG7gyB04bqnbmcgdHLDqm4gcmVzdGFydA=="
                    );
                }
            }))
        }
        None => {
            tracing::info!(
                "dsOybmcgbOG6t3AgxJHDs25nIGfDs2kgVOG6rlQgKGNoxrBhIGPhuqV1IGjDrG5oIGtobyBQREYpIOKAlCB5w6p1IGPhuqd1IHBhY2thZ2VzIHPhur0gY2jhu50="
            );
            None
        }
    };
    let cfg = cfg.clone();
    let r_347 = tokio::spawn(async move {
        let infer = match GrpcTritonInfer::connect_lazy(&cfg.triton_endpoint, "ocr_pipeline") {
            Ok(r_345) => r_345,
            Err(e) => {
                tracing::error!(error = % e, "ZOG7sW5nIGdSUEMgaW5mZXIgY2xpZW50");
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
                                "bmd14buTbiDhuqNuaCB0cmFuZzogTWluSU8gKFBERuKGknTDoWNoIHRyYW5nOyB0cmFuZyBraMO0bmcgY8OzIEpQRUcgbmjDum5nIHRow6wgcmFzdGVyIGhvw6Ep"
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
                                "bmd14buTbiDhuqNuaCB0cmFuZzogTWluSU8gKOG6o25oIHRyYW5nIHLhu51pKQ=="
                            );
                            PageBytesSource::Minio(MinioPageBytes::new(store))
                        }
                    }
                }
                Err(e) => {
                    tracing::error!(
                        error = % e, "ZOG7sW5nIE1pbklPIG9iamVjdCBzdG9yZQ=="
                    );
                    return;
                }
            },
            None => {
                tracing::info!(
                    dir = % cfg.page_source_dir,
                    "bmd14buTbiDhuqNuaCB0cmFuZzogdGjGsCBt4bulYyBj4bulYyBi4buZ"
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
                                "VkxNIE1vZGUgMjogZOG7sW5nIGV4dHJhY3RvciBs4buXaSDigJQgY2jhuqF5IEtIw5RORyBWTE0="
                            );
                            None
                        }
                    }
                }
                _ => {
                    tracing::warn!(
                        "VkxNIE1vZGUgMiBi4bqtdCBuaMawbmcgdGhp4bq/dSA={}/{}IOKAlCBjaOG6oXkgS0jDlE5HIFZMTQ==",
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
            .is_some(), "dsOybmcgbOG6t3Agd29ya2VyIGLhuq90IMSR4bqndQ=="
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
                    "dsOybmcgbOG6t3Agd29ya2VyIGzhu5dpIERCIOKAlCB0aG/DoXQgxJHhu4Mgc3lzdGVtZCByZXN0YXJ0"
                );
                r_34x.store(true, Ordering::SeqCst);
                shutdown.cancel();
            } else {
                tracing::info!(
                    error = % e,
                    "dsOybmcgbOG6t3Agd29ya2VyIGThu6tuZyB0aGVvIHNodXRkb3duIGNo4bunIMSR4buZbmc="
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
                    "buG6oXAgbGV4aWNvbiBjb3JyZWN0b3IgKMKnRzcp"
                );
                Lexicon::from_words(r_351)
            }
            Err(e) => {
                tracing::warn!(
                    r_34z = % r_34z, error = % e,
                    "xJHhu41jIGxleGljb24gbOG7l2kg4oCUIGTDuW5nIGxleGljb24gcuG7l25n"
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
                        "Y29ycmVjdG9yIMKnRzY6IGThu7FuZyB2TExNIGNsaWVudCBs4buXaSDigJQgY2jhuqF5IEtIw5RORyBjb3JyZWN0b3I="
                    );
                    None
                }
            }
        }
        _ => {
            tracing::warn!(
                "Y29ycmVjdG9yIMKnRzYgQuG6rFQgbmjGsG5nIHRoaeG6v3Ug{}/{}IOKAlCBjaOG6oXkgS0jDlE5HIGNvcnJlY3RvciAoY+G7nSBraMO0bmcgY8OybiBuby1vcCDDom0gdGjhuqdtKQ==",
                C_5M, C_5P
            );
            None
        }
    }
}
fn f_2gq(shutdown: CancellationToken) {
    tokio::spawn(async move {
        f_2qn().await;
        tracing::info!("bmjhuq1uIHTDrW4gaGnhu4d1IGThu6tuZyDigJQgZ3JhY2VmdWwgc2h1dGRvd24=");
        shutdown.cancel();
    });
}
async fn f_2qn() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let mut r_359 = signal(SignalKind::terminate()).expect("xJHEg25nIGvDvSBTSUdURVJN");
        let mut r_358 = signal(SignalKind::interrupt()).expect("xJHEg25nIGvDvSBTSUdJTlQ=");
        tokio::select! {
            _ = r_359.recv() => {} _ = r_358.recv() => {}
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}
