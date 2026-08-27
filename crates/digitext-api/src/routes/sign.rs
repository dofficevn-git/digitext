// Copyright (c) 2026 DigiText. All rights reserved.
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::{f_29a, Scope, Ts};
use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::Engine;
use digitext_core::{DocStatus, Permission};
use digitext_db::{documents, sign_sessions};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
const C_AE: i64 = 30 * 60;
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/sign/jobs", get(f_1np))
        .route("/sign/{id}/begin", post(begin))
        .route("/sign/{id}/complete", post(complete))
}
fn f_13e(r_q2: &str, field: &str) -> Result<Vec<u8>, ApiError> {
    base64::engine::general_purpose::STANDARD
        .decode(r_q2)
        .map_err(|_| {
            ApiError::bad_request(format!(
                "{field} không phải base64 hợp lệ"
            ))
        })
}
fn f_195(r_q3: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(r_q3)
}
fn storage_err(e: digitext_storage::StorageError) -> ApiError {
    tracing::error!(error = % e, "lỗi kho object (ký số)");
    ApiError::service_unavailable("kho object không sẵn sàng")
}
fn f_2f2(e: digitext_sign_verify::SignError) -> ApiError {
    tracing::error!(error = % e, "lỗi ký PAdES (server-side)");
    ApiError::internal()
}
fn store(r_q4: &AppState) -> Result<&std::sync::Arc<dyn digitext_storage::ObjectStore>, ApiError> {
    r_q4.store
        .as_ref()
        .ok_or_else(|| ApiError::service_unavailable("kho ký số chưa cấu hình"))
}
async fn authorize_project(r_q6: &AppState, r_q5: &Ts, project_id: Uuid) -> Result<(), ApiError> {
    if f_29a(&r_q6.pool, &r_q5.user)
        .await?
        .allows_project(project_id)
    {
        Ok(())
    } else {
        Err(ApiError::forbidden())
    }
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SignJob {
    id: Uuid,
    project_id: Uuid,
    filename: String,
    ho_so_ref: Option<String>,
    page_count: Option<i32>,
    status: String,
}
async fn f_1np(State(r_qb): State<AppState>, r_q7: Ts) -> Result<Json<Vec<SignJob>>, ApiError> {
    r_q7.require_perm(Permission::SignView)?;
    let r_q9 = match f_29a(&r_qb.pool, &r_q7.user).await? {
        Scope::All => None,
        Scope::Projects(r_q8) => Some(r_q8),
    };
    let r_qa =
        documents::f_1n8(&r_qb.pool, DocStatus::ToSign.as_str(), r_q9.as_deref(), 200).await?;
    Ok(Json(
        r_qa.into_iter()
            .map(|r| SignJob {
                id: r.id,
                project_id: r.project_id,
                filename: r.filename,
                ho_so_ref: r.ho_so_ref,
                page_count: r.page_count,
                status: r.status,
            })
            .collect(),
    ))
}
#[derive(Debug, Deserialize)]
struct T19 {
    certificate: String,
    #[serde(default)]
    intermediates: Vec<String>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BeginResp {
    session_id: Uuid,
    tbs_digest: String,
}
async fn begin(
    State(r_qn): State<AppState>,
    r_qc: Ts,
    Path(id): Path<Uuid>,
    Json(r_qj): Json<T19>,
) -> Result<Json<BeginResp>, ApiError> {
    r_qc.require_perm(Permission::SignExecute)?;
    let store = store(&r_qn)?;
    let r_qd = f_13e(&r_qj.certificate, "certificate")?;
    digitext_sign_verify::f_xy(&r_qd).map_err(|_| {
        ApiError::bad_request(
            "certificate không phải cert X.509 DER hợp lệ",
        )
    })?;
    let intermediates: Vec<Vec<u8>> = r_qj
        .intermediates
        .iter()
        .enumerate()
        .map(|(i, r_qk)| f_13e(r_qk, &format!("intermediates[{i}]")))
        .collect::<Result<_, _>>()?;
    let roots = r_qn
        .trust_roots
        .as_ref()
        .ok_or_else(|| {
            ApiError::service_unavailable(
                "chưa cấu hình DIGITEXT_SIGN_TRUST_ROOTS — không thể xác minh chuỗi chứng thư nên từ chối ký (đặt đường dẫn bundle PEM chứa root của CA PHÁT HÀNH TOKEN: repo có sẵn trust/ica-sign-trust-roots.pem cho token I-CA — VNRCA + I-CA SHA-256; rồi khởi động lại)",
            )
        })?;
    roots
        .verify_signer_chain_der(&r_qd, &intermediates, f_st() as i64)
        .map_err(|e| {
            ApiError::bad_request(format!(
                "chứng thư không được tin cậy: {e}"
            ))
        })?;
    let r_qe = documents::get(&r_qn.pool, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    authorize_project(&r_qn, &r_qc, r_qe.project_id).await?;
    let r_qp = digitext_storage::f_2qs(r_qe.project_id, id);
    let r_qg = store
        .get(&r_qp)
        .await
        .map_err(storage_err)?
        .ok_or_else(|| {
            ApiError::conflict(
                "PDF 2 lớp chưa sẵn trong kho (chưa OCR/ghép xong)",
            )
        })?;
    let r_qf = (!intermediates.is_empty()).then_some(intermediates.as_slice());
    let r_ql = sign_sessions::begin(&r_qn.pool, id, Some(r_qc.user.id), Some(&r_qd), r_qf, C_AE)
        .await?
        .ok_or_else(|| {
            ApiError::conflict("tài liệu không ở trạng thái chờ ký")
        })?;
    let r_qm = f_st();
    let r_qo = r_qn
        .sign_visible
        .then(|| digitext_sign_verify::pdf::f_2qb(&r_qd, r_qm, r_qn.sign_stamp))
        .flatten();
    if r_qn.sign_visible && r_qo.is_none() {
        tracing::warn!(
            "không tìm được font cho khối chữ ký hiển thị — ký VÔ HÌNH (lệch Điều 8.1.c)"
        );
    }
    let r_qh =
        digitext_sign_verify::pdf::f_23p(&r_qg, &r_qd, r_qm, r_qo.as_ref()).map_err(f_2f2)?;
    let r_qi = digitext_storage::f_23q(r_qe.project_id, id);
    store.put(&r_qi, &r_qh.pdf).await.map_err(storage_err)?;
    sign_sessions::f_2ea(
        &r_qn.pool,
        r_ql.id,
        &r_qh.tbs_digest,
        &r_qh.signed_attrs_der,
        &r_qi,
    )
    .await?;
    Ok(Json(BeginResp {
        session_id: r_ql.id,
        tbs_digest: f_195(&r_qh.tbs_digest),
    }))
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct T31 {
    session_id: Uuid,
    signature: String,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CompleteResp {
    status: String,
}
async fn complete(
    State(r_r1): State<AppState>,
    r_qq: Ts,
    Path(id): Path<Uuid>,
    Json(r_qx): Json<T31>,
) -> Result<Json<CompleteResp>, ApiError> {
    r_qq.require_perm(Permission::SignExecute)?;
    let store = store(&r_r1)?;
    let signature = f_13e(&r_qx.signature, "signature")?;
    let r_qr = documents::get(&r_r1.pool, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    authorize_project(&r_r1, &r_qq, r_qr.project_id).await?;
    let r_qy = sign_sessions::get(&r_r1.pool, r_qx.session_id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    if r_qy.document_id != id {
        return Err(ApiError::bad_request(
            "phiên không thuộc tài liệu này",
        ));
    }
    if r_qy.signer_user_id != Some(r_qq.user.id) {
        return Err(ApiError::forbidden());
    }
    if r_qy.status != "open" {
        return Err(ApiError::conflict("phiên ký không còn mở"));
    }
    let r_qv = r_qy.prepared_key.ok_or_else(|| {
        ApiError::conflict("phiên chưa chuẩn bị (gọi begin trước)")
    })?;
    let r_qz = r_qy
        .signed_attrs
        .ok_or_else(|| ApiError::conflict("phiên thiếu SignedAttributes"))?;
    let cert = r_qy
        .signer_cert
        .ok_or_else(|| ApiError::conflict("phiên thiếu cert người ký"))?;
    let intermediates = r_qy.signer_intermediates.unwrap_or_default();
    let r_qw = store
        .get(&r_qv)
        .await
        .map_err(storage_err)?
        .ok_or_else(|| {
            ApiError::conflict("PDF đã chuẩn bị không còn trong kho")
        })?;
    let timestamp = match r_r1.tsa_url.as_deref() {
        Some(url) => Some(fetch_timestamp(url, &signature).await?),
        None => None,
    };
    let signed_pdf = digitext_sign_verify::f_1cn(
        &r_qw,
        &r_qz,
        &signature,
        &cert,
        &intermediates,
        timestamp.as_deref(),
    )
    .map_err(f_2f2)?;
    let r_r2 = digitext_sign_verify::f_2q1(
        &signed_pdf,
        r_r1.trust_roots.as_deref(),
        r_r1.tsa_trust_roots.as_deref(),
    );
    let ok = match &r_r2 {
        Ok(r) if !r.is_valid() => false,
        Ok(r) if !r.signer_chain_ok => false,
        Ok(_) if r_r1.tsa_url.is_none() => true,
        Ok(r) if !r.timestamp_structural_ok => false,
        Ok(r) => r_r1.tsa_trust_roots.is_none() || r.timestamp_trusted,
        Err(_) => false,
    };
    if !ok {
        let why = match &r_r2 {
            Ok(r) => {
                format!("verify chữ ký thất bại (§S4.1 bước 4): {r:?}")
            }
            Err(e) => {
                format!("verify chữ ký thất bại (§S4.1 bước 4): {e}")
            }
        };
        tracing::warn!(
            session = % r_qy.id, document = % id, reason = % why,
            "từ chối phiên ký"
        );
        sign_sessions::f_26q(&r_r1.pool, r_qy.id, &why).await?;
        return Ok(Json(CompleteResp {
            status: "rejected".into(),
        }));
    }
    let (signed_pdf, r_qs) = if r_r1.pades_lt {
        match f_sm(&r_r1, &signed_pdf, &cert, &intermediates).await {
            LtOutcome::Attached(r_qt) => (r_qt, true),
            LtOutcome::NoMaterial => (signed_pdf, false),
            LtOutcome::Revoked => {
                let why = "cert người ký BỊ THU HỒI (OCSP/CRL) — từ chối lưu trữ chữ ký (PAdES-LT)";
                tracing::warn!(
                    session = % r_qy.id, document = % id, reason = why,
                    "từ chối phiên ký (LT)"
                );
                sign_sessions::f_26q(&r_r1.pool, r_qy.id, why).await?;
                return Ok(Json(CompleteResp {
                    status: "rejected".into(),
                }));
            }
        }
    } else {
        (signed_pdf, false)
    };
    let signed_pdf = if r_r1.pades_lta && r_qs {
        match r_r1.tsa_url.as_deref() {
            Some(url) => {
                let r_qu = f_sn(url, &signed_pdf).await?;
                let ok = matches!(
                    digitext_sign_verify::f_2q1(& r_qu, r_r1.trust_roots.as_deref(), r_r1
                    .tsa_trust_roots.as_deref(),), Ok(ref r) if r.is_lta_structural() &&
                    (r_r1.tsa_trust_roots.is_none() || r.is_lta())
                );
                if !ok {
                    let why = "archive-timestamp (PAdES-LTA) không tái xác minh được — TSA trả token không hợp lệ (không phải RSA?); từ chối để tránh lưu bản 'LTA' giả";
                    tracing::error!(
                        session = % r_qy.id, document = % id,
                        "từ chối phiên ký (LTA): {why}"
                    );
                    sign_sessions::f_26q(&r_r1.pool, r_qy.id, why).await?;
                    return Ok(Json(CompleteResp {
                        status: "rejected".into(),
                    }));
                }
                r_qu
            }
            None => {
                tracing::error!(
                    session = % r_qy.id, document = % id,
                    "PAdES-LTA bật nhưng thiếu TSA lúc ký — lưu LT (không archive-timestamp)"
                );
                signed_pdf
            }
        }
    } else {
        if r_r1.pades_lta && !r_qs {
            tracing::warn!(
                session = % r_qy.id, document = % id,
                "PAdES-LTA bật nhưng chữ ký KHÔNG có /DSS (LT không thu được OCSP/CRL) — lưu B/T"
            );
        }
        signed_pdf
    };
    let r_r0 = digitext_storage::f_2fc(r_qr.project_id, id);
    store.put(&r_r0, &signed_pdf).await.map_err(storage_err)?;
    sign_sessions::complete(&r_r1.pool, r_qy.id, &r_r0).await?;
    Ok(Json(CompleteResp {
        status: "signed".into(),
    }))
}
enum LtOutcome {
    Attached(Vec<u8>),
    NoMaterial,
    Revoked,
}
const C_EH: usize = 512 * 1024;
const C_2E: usize = 8 * 1024 * 1024;
const C_IA: std::time::Duration = std::time::Duration::from_secs(15);
async fn f_sm(r_re: &AppState, r_rc: &[u8], r_rd: &[u8], intermediates: &[Vec<u8>]) -> LtOutcome {
    use digitext_sign_verify::ocsp::{self, Revocation};
    let Some(roots) = r_re.trust_roots.as_deref() else {
        tracing::warn!(
            "PAdES-LT bật nhưng thiếu trust_roots — bỏ qua nhúng /DSS"
        );
        return LtOutcome::NoMaterial;
    };
    let Some(r_r7) = roots.resolve_issuer_der(r_rd, intermediates) else {
        tracing::warn!(
            "PAdES-LT: không tìm được cert issuer để lấy OCSP/CRL — lưu B/T"
        );
        return LtOutcome::NoMaterial;
    };
    let mut r_r9: Vec<Vec<u8>> = Vec::new();
    let mut r_r5: Vec<Vec<u8>> = Vec::new();
    if let Some(url) = ocsp::f_1vw(r_rd) {
        match ocsp::f_vl(&r_r7, r_rd) {
            Ok(r_ra) => match f_1c7(&url, r_ra).await {
                Ok(r_rb) => match ocsp::f_y7(&r_rb, &r_r7, r_rd) {
                    Ok(c) if c.signature_ok && c.status == Revocation::Revoked => {
                        return LtOutcome::Revoked;
                    }
                    Ok(c) if c.signature_ok && c.status == Revocation::Good => r_r9.push(r_rb),
                    Ok(_) => {
                        tracing::warn!(
                            % url,
                            "OCSP không tin được (chữ ký sai / unknown) — bỏ"
                        )
                    }
                    Err(e) => {
                        tracing::warn!(
                            % url, error =% e,
                            "OCSP không đọc được — bỏ"
                        )
                    }
                },
                Err(e) => {
                    tracing::warn!(
                        % url, error =% e, "tải OCSP lỗi — bỏ"
                    )
                }
            },
            Err(e) => {
                tracing::warn!(
                    error =% e, "dựng OCSP request lỗi — bỏ"
                )
            }
        }
    }
    if let Some(url) = ocsp::f_12o(r_rd) {
        match f_1c3(&url).await {
            Ok(r_r4) => match ocsp::f_y1(&r_r4, &r_r7, r_rd) {
                Ok(c) if c.signature_ok && c.status == Revocation::Revoked => {
                    return LtOutcome::Revoked;
                }
                Ok(c) if c.signature_ok && c.status == Revocation::Good => r_r5.push(r_r4),
                Ok(_) => {
                    tracing::warn!(
                        % url,
                        "CRL không tin được (chữ ký sai) — bỏ"
                    )
                }
                Err(e) => {
                    tracing::warn!(
                        % url, error =% e,
                        "CRL không đọc được — bỏ"
                    )
                }
            },
            Err(e) => {
                tracing::warn!(% url, error =% e, "tải CRL lỗi — bỏ")
            }
        }
    }
    if r_r9.is_empty() && r_r5.is_empty() {
        tracing::warn!(
            "PAdES-LT: không thu được OCSP/CRL hợp lệ — lưu B/T (không /DSS)"
        );
        return LtOutcome::NoMaterial;
    }
    let mut r_r3: Vec<Vec<u8>> = vec![r_rd.to_vec()];
    for r_r6 in intermediates.iter().chain(std::iter::once(&r_r7)) {
        if !r_r3.contains(r_r6) {
            r_r3.push(r_r6.clone());
        }
    }
    let r_rf = digitext_sign_verify::dss::ValidationMaterial {
        certs: r_r3,
        crls: r_r5,
        ocsps: r_r9,
    };
    match digitext_sign_verify::dss::embed_dss(r_rc, &r_rf) {
        Ok(r_r8) => LtOutcome::Attached(r_r8),
        Err(e) => {
            tracing::warn!(error =% e, "nhúng /DSS lỗi — lưu B/T");
            LtOutcome::NoMaterial
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LtaStartup {
    Enable,
    MissingLt,
    MissingTsa,
    Off,
}
pub fn f_1pw(r_ri: bool, r_rh: bool, r_rg: bool) -> LtaStartup {
    if !r_ri {
        LtaStartup::Off
    } else if !r_rh {
        LtaStartup::MissingLt
    } else if !r_rg {
        LtaStartup::MissingTsa
    } else {
        LtaStartup::Enable
    }
}
async fn f_sn(r_rl: &str, r_rj: &[u8]) -> Result<Vec<u8>, ApiError> {
    let r_rk = digitext_sign_verify::dss::f_23h(r_rj).map_err(f_2f2)?;
    let token = fetch_timestamp(r_rl, &r_rk.tbs).await?;
    digitext_sign_verify::dss::f_1cp(&r_rk, &token).map_err(f_2f2)
}
async fn f_1c7(r_ro: &str, r_rm: Vec<u8>) -> Result<Vec<u8>, String> {
    let client = reqwest::Client::builder()
        .timeout(C_IA)
        .build()
        .map_err(|e| e.to_string())?;
    let r_rn = client
        .post(r_ro)
        .header("Content-Type", "application/ocsp-request")
        .header("Accept", "application/ocsp-response")
        .body(r_rm)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !r_rn.status().is_success() {
        return Err(format!("HTTP {}", r_rn.status()));
    }
    f_25o(r_rn, C_EH).await
}
async fn f_1c3(r_rq: &str) -> Result<Vec<u8>, String> {
    let client = reqwest::Client::builder()
        .timeout(C_IA)
        .build()
        .map_err(|e| e.to_string())?;
    let r_rp = client.get(r_rq).send().await.map_err(|e| e.to_string())?;
    if !r_rp.status().is_success() {
        return Err(format!("HTTP {}", r_rp.status()));
    }
    f_25o(r_rp, C_2E).await
}
async fn f_25o(r_rs: reqwest::Response, cap: usize) -> Result<Vec<u8>, String> {
    if let Some(len) = r_rs.content_length() {
        if len as usize > cap {
            return Err(format!(
                "phản hồi quá lớn ({len} > {cap})"
            ));
        }
    }
    let r_rr = r_rs.bytes().await.map_err(|e| e.to_string())?;
    if r_rr.len() > cap {
        return Err(format!(
            "phản hồi quá lớn ({} > {cap})",
            r_rr.len()
        ));
    }
    Ok(r_rr.to_vec())
}
fn f_st() -> u64 {
    chrono::Utc::now().timestamp().max(0) as u64
}
fn f_2mn(ctx: &str, r_rt: impl std::fmt::Display) -> ApiError {
    tracing::error!(error = % r_rt, "TSA (PAdES-T): {ctx}");
    ApiError::service_unavailable(
        "dịch vụ đóng dấu thời gian (TSA) không sẵn sàng",
    )
}
const C_MV: usize = 256 * 1024;
const C_MW: std::time::Duration = std::time::Duration::from_secs(15);
async fn fetch_timestamp(r_ry: &str, signature: &[u8]) -> Result<Vec<u8>, ApiError> {
    if !r_ry.starts_with("https://") {
        return Err(
            ApiError::service_unavailable(
                "DIGITEXT_TSA_URL phải dùng https (tem thời gian trên http bị MITM sửa được)",
            ),
        );
    }
    let nonce: u64 = rand::random();
    let r_rv =
        digitext_sign_verify::tsa::build_request(signature, Some(nonce), true).map_err(f_2f2)?;
    let client = reqwest::Client::builder()
        .timeout(C_MW)
        .build()
        .map_err(|e| f_2mn("dựng HTTP client", e))?;
    let r_rw = client
        .post(r_ry)
        .header("Content-Type", "application/timestamp-query")
        .body(r_rv)
        .send()
        .await
        .map_err(|e| f_2mn("gửi request thất bại", e))?;
    if !r_rw.status().is_success() {
        return Err(f_2mn("HTTP lỗi", r_rw.status()));
    }
    if let Some(len) = r_rw.content_length() {
        if len as usize > C_MV {
            return Err(f_2mn("phản hồi quá lớn", len));
        }
    }
    let r_ru = r_rw
        .bytes()
        .await
        .map_err(|e| f_2mn("đọc phản hồi thất bại", e))?;
    if r_ru.len() > C_MV {
        return Err(f_2mn("phản hồi quá lớn", r_ru.len()));
    }
    let token = digitext_sign_verify::tsa::f_2le(&r_ru)
        .map_err(|e| f_2mn("phản hồi không hợp lệ", e))?;
    let r_rx = digitext_sign_verify::tsa::f_2py(&token, signature)
        .map_err(|e| f_2mn("token không hợp lệ", e))?;
    if !r_rx.check_nonce(nonce) {
        return Err(
            f_2mn(
                "nonce trong token không khớp nonce đã gửi (token phát lại?)",
                nonce,
            ),
        );
    }
    Ok(token)
}
