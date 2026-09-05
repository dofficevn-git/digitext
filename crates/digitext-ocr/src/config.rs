// Copyright (c) 2026 DigiText. All rights reserved.
use std::collections::HashMap;
use std::fmt::Display;
use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;
pub const C_5G: &str = "DIGITEXT_DATABASE_URL";
pub const C_6D: &str = "DIGITEXT_TRITON_ENDPOINT";
pub const C_6E: &str = "DIGITEXT_TRITON_HTTP_ENDPOINT";
pub const C_5T: &str = "DIGITEXT_METRICS_ADDR";
pub const C_6I: &str = "DIGITEXT_WORKER_CONCURRENCY";
pub const C_5I: &str = "DIGITEXT_INFER_BATCH_SIZE";
pub const C_6C: &str = "DIGITEXT_TILE_SIZE";
pub const C_6B: &str = "DIGITEXT_TILE_OVERLAP";
pub const C_5F: &str = "DIGITEXT_CONF_FALLBACK";
pub const C_6A: &str = "DIGITEXT_REVIEW_LOW_CONF_RATIO";
pub const C_5S: &str = "DIGITEXT_MAX_RETRIES";
pub const C_68: &str = "DIGITEXT_RETRY_BASE_DELAY_MS";
pub const C_69: &str = "DIGITEXT_RETRY_MAX_DELAY_MS";
pub const C_5J: &str = "DIGITEXT_LEASE_SECONDS";
pub const C_6J: &str = "DIGITEXT_WORKER_ENABLED";
pub const C_62: &str = "DIGITEXT_PAGE_SOURCE_DIR";
pub const C_6K: &str = "DIGITEXT_WORKER_ID";
pub const C_5L: &str = "DIGITEXT_LLM_CORRECTOR_ENABLED";
pub const C_5Q: &str = "DIGITEXT_LLM_CORRECTOR_TAU";
pub const C_5N: &str = "DIGITEXT_LLM_CORRECTOR_MAX_EDIT";
pub const C_5O: &str = "DIGITEXT_LLM_CORRECTOR_MIN_CONF";
pub const C_5M: &str = "DIGITEXT_LLM_CORRECTOR_ENDPOINT";
pub const C_5P: &str = "DIGITEXT_LLM_CORRECTOR_MODEL";
pub const C_5R: &str = "DIGITEXT_LLM_CORRECTOR_TIMEOUT_MS";
pub const C_5K: &str = "DIGITEXT_LEXICON_PATH";
pub const C_6F: &str = "DIGITEXT_VLM_FALLBACK_ENABLED";
pub const ENV_VLM_ENDPOINT: &str = "DIGITEXT_VLM_ENDPOINT";
pub const ENV_VLM_MODEL: &str = "DIGITEXT_VLM_MODEL";
pub const C_6H: &str = "DIGITEXT_VLM_TIMEOUT_MS";
pub const C_6G: &str = "DIGITEXT_VLM_MAX_IMAGE_SIDE";
pub const ENV_FIELDS_ENGINE: &str = "DIGITEXT_FIELDS_ENGINE";
pub const ENV_OCR_ENGINE: &str = "DIGITEXT_OCR_ENGINE";
pub const ENV_OCR_HTTP_ENDPOINT: &str = "DIGITEXT_OCR_HTTP_ENDPOINT";
pub const ENV_OCR_HTTP_API_KEY: &str = "DIGITEXT_OCR_HTTP_API_KEY";
pub const ENV_OCR_HTTP_TIMEOUT_MS: &str = "DIGITEXT_OCR_HTTP_TIMEOUT_MS";
pub const C_5H: &str = "DIGITEXT_FIELDS_V2_MAX_PAGES";
pub const C_64: &str = "DIGITEXT_PDF_DPI";
pub const C_65: &str = "DIGITEXT_PDF_RASTER_DPI";
pub const C_63: &str = "DIGITEXT_PDFTOPPM_BIN";
pub const C_67: &str = "DIGITEXT_PDF_STORE_DIR";
pub const C_66: &str = "DIGITEXT_PDF_STORE_BUCKET";
pub const C_5E: &str = "DIGITEXT_AUTO_MAKE_PDF2";
pub const C_5W: &str = "DIGITEXT_MINIO_ENDPOINT";
pub const C_5V: &str = "DIGITEXT_MINIO_BUCKET";
pub const C_5U: &str = "DIGITEXT_MINIO_ACCESS_KEY";
pub const C_5Y: &str = "DIGITEXT_MINIO_SECRET_KEY";
pub const C_5X: &str = "DIGITEXT_MINIO_REGION";
pub const C_61: &str = "DIGITEXT_PAGE_MODE";
const C_3Y: &str = "us-east-1";
const C_3X: &str = "0.0.0.0:9090";
const C_41: &str = "http://127.0.0.1:8000";
const C_40: &str = "/var/lib/digitext/pages";
const C_43: &str = "worker-1";
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ConfigError {
    #[error("thiếu biến môi trường bắt buộc: {0}")]
    Missing(&'static str),
    #[error("giá trị không hợp lệ cho {field}: {reason}")]
    Invalid { field: &'static str, reason: String },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageMode {
    Pdf,
    Images,
}
/// Engine NHẬN DẠNG CHỮ (khác FieldsEngine — cái đó là engine BÓC TRƯỜNG).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OcrEngine {
    /// Triton gRPC — mặc định, giữ tương thích ngược.
    Triton,
    /// Dịch vụ HTTP ở ocr/ (PaddleOCR toàn văn).
    Http,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldsEngine {
    V1,
    V2,
}
#[derive(Debug, Clone, PartialEq)]
pub struct WorkerConfig {
    pub database_url: String,
    pub ocr_engine: OcrEngine,
    /// Rỗng khi ocr_engine = Http — lúc đó Triton không được dùng tới.
    pub triton_endpoint: String,
    pub ocr_http_endpoint: String,
    pub ocr_http_api_key: String,
    pub ocr_http_timeout_ms: u64,
    pub triton_http_endpoint: String,
    pub metrics_addr: String,
    pub worker_concurrency: usize,
    pub infer_batch_size: usize,
    pub tile_size: u32,
    pub tile_overlap: u32,
    pub conf_fallback: f32,
    pub review_low_conf_ratio: f32,
    pub max_retries: u32,
    pub retry_base_delay: Duration,
    pub retry_max_delay: Duration,
    pub lease: Duration,
    pub worker_enabled: bool,
    pub page_source_dir: String,
    pub worker_id: String,
    pub llm_corrector_enabled: bool,
    pub corrector_tau: f32,
    pub corrector_max_edit: usize,
    pub corrector_min_conf: f32,
    pub vlm_fallback_enabled: bool,
    pub fields_engine: FieldsEngine,
    pub fields_v2_max_pages: u32,
    pub pdf_dpi: u32,
    pub pdf_raster_dpi: u32,
    pub pdftoppm_bin: String,
    pub pdf_store_dir: Option<String>,
    pub pdf_store_bucket: Option<String>,
    pub minio: Option<digitext_storage::Tde>,
    pub page_mode: PageMode,
    pub auto_make_pdf2: bool,
}
impl WorkerConfig {
    pub const ENV_KEYS: [&'static str; 40] = [
        C_5G,
        C_6D,
        C_6E,
        C_5T,
        C_6I,
        C_5I,
        C_6C,
        C_6B,
        C_5F,
        C_6A,
        C_5S,
        C_68,
        C_69,
        C_5J,
        C_6J,
        C_62,
        C_6K,
        C_5L,
        C_5Q,
        C_5N,
        C_5O,
        C_6F,
        ENV_FIELDS_ENGINE,
        ENV_OCR_ENGINE,
        ENV_OCR_HTTP_ENDPOINT,
        ENV_OCR_HTTP_API_KEY,
        ENV_OCR_HTTP_TIMEOUT_MS,
        C_5H,
        C_64,
        C_65,
        C_63,
        C_67,
        C_66,
        C_5E,
        C_5W,
        C_5V,
        C_5U,
        C_5Y,
        C_5X,
        C_61,
    ];
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut r_30q = HashMap::new();
        for r_30p in Self::ENV_KEYS {
            if let Ok(r_30r) = std::env::var(r_30p) {
                r_30q.insert(r_30p.to_owned(), r_30r);
            }
        }
        Self::from_map(&r_30q)
    }
    pub fn from_map(r_30y: &HashMap<String, String>) -> Result<Self, ConfigError> {
        let database_url = require(r_30y, C_5G)?.to_owned();
        if !(database_url.starts_with("postgres://") || database_url.starts_with("postgresql://")) {
            return Err(ConfigError::Invalid {
                field: C_5G,
                reason: "phải bắt đầu bằng postgres:// hoặc postgresql://"
                    .to_owned(),
            });
        }
        let ocr_engine = match r_30y
            .get(ENV_OCR_ENGINE)
            .map(|s| s.to_ascii_lowercase())
            .as_deref()
        {
            None | Some("triton") => OcrEngine::Triton,
            Some("http") => OcrEngine::Http,
            Some(other) => {
                return Err(ConfigError::Invalid {
                    field: ENV_OCR_ENGINE,
                    reason: format!(
                        "chỉ nhận 'triton' hoặc 'http', nhận {other:?}"
                    ),
                });
            }
        };
        // Chỉ bắt buộc khi thực sự dùng Triton. Bắt buộc vô điều kiện sẽ khiến
        // worker không khởi động nổi ở deployment không còn Triton.
        let triton_endpoint = match ocr_engine {
            OcrEngine::Triton => require(r_30y, C_6D)?.to_owned(),
            OcrEngine::Http => r_30y.get(C_6D).cloned().unwrap_or_default(),
        };
        if triton_endpoint.chars().any(char::is_whitespace) {
            return Err(ConfigError::Invalid {
                field: C_6D,
                reason: "không được chứa khoảng trắng".to_owned(),
            });
        }
        let ocr_http_endpoint = match ocr_engine {
            OcrEngine::Http => {
                let v = require(r_30y, ENV_OCR_HTTP_ENDPOINT)?.to_owned();
                if !(v.starts_with("http://") || v.starts_with("https://")) {
                    return Err(ConfigError::Invalid {
                        field: ENV_OCR_HTTP_ENDPOINT,
                        reason: "phải bắt đầu bằng http:// hoặc https://".to_owned(),
                    });
                }
                v
            }
            OcrEngine::Triton => r_30y
                .get(ENV_OCR_HTTP_ENDPOINT)
                .cloned()
                .unwrap_or_default(),
        };
        let ocr_http_api_key = r_30y
            .get(ENV_OCR_HTTP_API_KEY)
            .cloned()
            .unwrap_or_default();
        let ocr_http_timeout_ms =
            f_1eu::<u64>(r_30y, ENV_OCR_HTTP_TIMEOUT_MS, 120_000, 1_000, 600_000)?;
        let triton_http_endpoint = r_30y
            .get(C_6E)
            .map(String::as_str)
            .unwrap_or(C_41)
            .to_owned();
        if !(triton_http_endpoint.starts_with("http://")
            || triton_http_endpoint.starts_with("https://"))
        {
            return Err(ConfigError::Invalid {
                field: C_6E,
                reason: "phải bắt đầu bằng http:// hoặc https://"
                    .to_owned(),
            });
        }
        let metrics_addr = r_30y
            .get(C_5T)
            .map(String::as_str)
            .unwrap_or(C_3X)
            .to_owned();
        metrics_addr
            .parse::<SocketAddr>()
            .map_err(|_| ConfigError::Invalid {
                field: C_5T,
                reason: format!(
                    "không phải địa chỉ host:port hợp lệ: {metrics_addr:?}"
                ),
            })?;
        let worker_concurrency = f_1eu::<usize>(r_30y, C_6I, 4, 1, 64)?;
        let infer_batch_size = f_1eu::<usize>(r_30y, C_5I, 8, 1, 128)?;
        let tile_size = f_1eu::<u32>(r_30y, C_6C, 1024, 256, 4096)?;
        if tile_size % 32 != 0 {
            return Err(ConfigError::Invalid {
                field: C_6C,
                reason: format!("phải là bội số của 32, nhận {tile_size}"),
            });
        }
        let tile_overlap = f_1eu::<u32>(r_30y, C_6B, 64, 0, 4096)?;
        if tile_overlap >= tile_size {
            return Err(ConfigError::Invalid {
                field: C_6B,
                reason: format!(
                    "phải < tile_size ({tile_size}), nhận {tile_overlap}"
                ),
            });
        }
        let conf_fallback = f_1eu::<f32>(r_30y, C_5F, 0.80, 0.0, 1.0)?;
        let review_low_conf_ratio = f_1eu::<f32>(r_30y, C_6A, 0.20, 0.0, 1.0)?;
        let max_retries = f_1eu::<u32>(r_30y, C_5S, 5, 0, 20)?;
        let base_ms = f_1eu::<u64>(r_30y, C_68, 200, 1, 60_000)?;
        let max_ms = f_1eu::<u64>(r_30y, C_69, 30_000, 1, 600_000)?;
        if max_ms < base_ms {
            return Err(ConfigError::Invalid {
                field: C_69,
                reason: format!(
                    "phải >= retry_base_delay_ms ({base_ms}), nhận {max_ms}"
                ),
            });
        }
        let r_30x = f_1eu::<u64>(r_30y, C_5J, 120, 5, 3600)?;
        let worker_enabled = f_1ee(r_30y, C_6J, false)?;
        let page_source_dir = r_30y
            .get(C_62)
            .map(String::as_str)
            .unwrap_or(C_40)
            .to_owned();
        let worker_id = r_30y
            .get(C_6K)
            .map(String::as_str)
            .unwrap_or(C_43)
            .to_owned();
        if worker_id.is_empty() {
            return Err(ConfigError::Invalid {
                field: C_6K,
                reason: "không được rỗng".to_owned(),
            });
        }
        let llm_corrector_enabled = f_1ee(r_30y, C_5L, false)?;
        let corrector_tau = f_1eu::<f32>(r_30y, C_5Q, 0.75, 0.0, 1.0)?;
        let corrector_max_edit = f_1eu::<usize>(r_30y, C_5N, 2, 1, 32)?;
        let corrector_min_conf = f_1eu::<f32>(r_30y, C_5O, 0.90, 0.0, 1.0)?;
        let vlm_fallback_enabled = f_1ee(r_30y, C_6F, false)?;
        let fields_engine = match r_30y
            .get(ENV_FIELDS_ENGINE)
            .map(|s| s.to_ascii_lowercase())
            .as_deref()
        {
            None | Some("v1") => FieldsEngine::V1,
            Some("v2") => FieldsEngine::V2,
            Some(other) => {
                return Err(ConfigError::Invalid {
                    field: ENV_FIELDS_ENGINE,
                    reason: format!(
                        "chỉ nhận 'v1' hoặc 'v2', nhận {other:?}"
                    ),
                });
            }
        };
        let fields_v2_max_pages = f_1eu::<u32>(r_30y, C_5H, 1, 1, 10)?;
        let pdf_dpi = f_1eu::<u32>(r_30y, C_64, 300, 72, 1200)?;
        let pdf_raster_dpi = f_1eu::<u32>(r_30y, C_65, digitext_pdf::FM1_MIN_DPI, 72, 600)?;
        let pdftoppm_bin = r_30y
            .get(C_63)
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .unwrap_or("pdftoppm")
            .to_owned();
        let auto_make_pdf2 = f_1ee(r_30y, C_5E, false)?;
        let pdf_store_dir = r_30y.get(C_67).filter(|s| !s.is_empty()).cloned();
        let pdf_store_bucket = r_30y.get(C_66).filter(|s| !s.is_empty()).cloned();
        let r_30v = |r_30w: &'static str| {
            r_30y
                .get(r_30w)
                .map(String::as_str)
                .filter(|s| !s.is_empty())
        };
        let minio = match (r_30v(C_5W), r_30v(C_5V), r_30v(C_5U), r_30v(C_5Y)) {
            (None, None, None, None) => None,
            (Some(r_30u), Some(r_30t), Some(r_30s), Some(r_30z)) => {
                if !(r_30u.starts_with("http://") || r_30u.starts_with("https://")) {
                    return Err(ConfigError::Invalid {
                        field: C_5W,
                        reason:
                            "phải bắt đầu bằng http:// hoặc https://"
                                .to_owned(),
                    });
                }
                Some(digitext_storage::Tde {
                    endpoint: r_30u.to_owned(),
                    bucket: r_30t.to_owned(),
                    region: r_30v(C_5X).unwrap_or(C_3Y).to_owned(),
                    access_key: r_30s.to_owned(),
                    secret_key: r_30z.to_owned(),
                })
            }
            _ => {
                return Err(ConfigError::Invalid {
                    field: C_5W,
                    reason: "MinIO cần đủ 4 khoá ENDPOINT/BUCKET/ACCESS_KEY/SECRET_KEY (hoặc bỏ hết)"
                        .to_owned(),
                });
            }
        };
        let page_mode = match r_30v(C_61).map(str::to_ascii_lowercase).as_deref() {
            None | Some("pdf") => PageMode::Pdf,
            Some("images") => PageMode::Images,
            Some(_) => {
                return Err(ConfigError::Invalid {
                    field: C_61,
                    reason: "chỉ nhận 'pdf' hoặc 'images'".to_owned(),
                });
            }
        };
        Ok(WorkerConfig {
            database_url,
            triton_endpoint,
            triton_http_endpoint,
            metrics_addr,
            worker_concurrency,
            infer_batch_size,
            tile_size,
            tile_overlap,
            conf_fallback,
            review_low_conf_ratio,
            max_retries,
            retry_base_delay: Duration::from_millis(base_ms),
            retry_max_delay: Duration::from_millis(max_ms),
            lease: Duration::from_secs(r_30x),
            worker_enabled,
            page_source_dir,
            worker_id,
            llm_corrector_enabled,
            corrector_tau,
            corrector_max_edit,
            corrector_min_conf,
            vlm_fallback_enabled,
            ocr_engine,
            ocr_http_endpoint,
            ocr_http_api_key,
            ocr_http_timeout_ms,
            fields_engine,
            fields_v2_max_pages,
            pdf_dpi,
            pdf_raster_dpi,
            pdftoppm_bin,
            pdf_store_dir,
            pdf_store_bucket,
            minio,
            page_mode,
            auto_make_pdf2,
        })
    }
}
fn require<'a>(
    r_311: &'a HashMap<String, String>,
    r_310: &'static str,
) -> Result<&'a str, ConfigError> {
    match r_311.get(r_310) {
        Some(r_312) if !r_312.is_empty() => Ok(r_312),
        _ => Err(ConfigError::Missing(r_310)),
    }
}
fn f_1eu<T>(
    r_315: &HashMap<String, String>,
    r_314: &'static str,
    r_313: T,
    min: T,
    max: T,
) -> Result<T, ConfigError>
where
    T: FromStr + PartialOrd + Display + Copy,
{
    let value = match r_315.get(r_314) {
        None => return Ok(r_313),
        Some(s) => s.parse::<T>().map_err(|_| ConfigError::Invalid {
            field: r_314,
            reason: format!("không phải số hợp lệ: {s:?}"),
        })?,
    };
    if value < min || value > max {
        return Err(ConfigError::Invalid {
            field: r_314,
            reason: format!("phải trong [{min}, {max}], nhận {value}"),
        });
    }
    Ok(value)
}
fn f_1ee(
    r_318: &HashMap<String, String>,
    r_317: &'static str,
    r_316: bool,
) -> Result<bool, ConfigError> {
    match r_318.get(r_317).map(String::as_str) {
        None => Ok(r_316),
        Some("true") => Ok(true),
        Some("false") => Ok(false),
        Some(other) => Err(ConfigError::Invalid {
            field: r_317,
            reason: format!("phải 'true' hoặc 'false', nhận {other:?}"),
        }),
    }
}
