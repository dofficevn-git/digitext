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
                "{field}IGtow7RuZyBwaOG6o2kgYmFzZTY0IGjhu6NwIGzhu4c="
            ))
        })
}
fn f_195(r_q3: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(r_q3)
}
fn storage_err(e: digitext_storage::StorageError) -> ApiError {
    tracing::error!(error = % e, "bOG7l2kga2hvIG9iamVjdCAoa8O9IHPhu5Ep");
    ApiError::service_unavailable("a2hvIG9iamVjdCBraMO0bmcgc+G6tW4gc8Ogbmc=")
}
fn f_2f2(e: digitext_sign_verify::SignError) -> ApiError {
    tracing::error!(error = % e, "bOG7l2kga8O9IFBBZEVTIChzZXJ2ZXItc2lkZSk=");
    ApiError::internal()
}
fn store(r_q4: &AppState) -> Result<&std::sync::Arc<dyn digitext_storage::ObjectStore>, ApiError> {
    r_q4.store
        .as_ref()
        .ok_or_else(|| ApiError::service_unavailable("a2hvIGvDvSBz4buRIGNoxrBhIGPhuqV1IGjDrG5o"))
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
            "Y2VydGlmaWNhdGUga2jDtG5nIHBo4bqjaSBjZXJ0IFguNTA5IERFUiBo4bujcCBs4buH",
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
                "Y2jGsGEgY+G6pXUgaMOsbmggRElHSVRFWFRfU0lHTl9UUlVTVF9ST09UUyDigJQga2jDtG5nIHRo4buDIHjDoWMgbWluaCBjaHXhu5dpIGNo4bupbmcgdGjGsCBuw6puIHThu6sgY2jhu5FpIGvDvSAoxJHhurd0IMSRxrDhu51uZyBk4bqrbiBidW5kbGUgUEVNIGNo4bupYSByb290IGPhu6dhIENBIFBIw4FUIEjDgE5IIFRPS0VOOiByZXBvIGPDsyBz4bq1biB0cnVzdC9pY2Etc2lnbi10cnVzdC1yb290cy5wZW0gY2hvIHRva2VuIEktQ0Eg4oCUIFZOUkNBICsgSS1DQSBTSEEtMjU2OyBy4buTaSBraOG7n2kgxJHhu5luZyBs4bqhaSk=",
            )
        })?;
    roots
        .verify_signer_chain_der(&r_qd, &intermediates, f_st() as i64)
        .map_err(|e| {
            ApiError::bad_request(format!(
                "Y2jhu6luZyB0aMawIGtow7RuZyDEkcaw4bujYyB0aW4gY+G6rXk6IA=={e}"
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
                "UERGIDIgbOG7m3AgY2jGsGEgc+G6tW4gdHJvbmcga2hvIChjaMawYSBPQ1IvZ2jDqXAgeG9uZyk=",
            )
        })?;
    let r_qf = (!intermediates.is_empty()).then_some(intermediates.as_slice());
    let r_ql = sign_sessions::begin(&r_qn.pool, id, Some(r_qc.user.id), Some(&r_qd), r_qf, C_AE)
        .await?
        .ok_or_else(|| {
            ApiError::conflict("dMOgaSBsaeG7h3Uga2jDtG5nIOG7nyB0cuG6oW5nIHRow6FpIGNo4budIGvDvQ==")
        })?;
    let r_qm = f_st();
    let r_qo = r_qn
        .sign_visible
        .then(|| digitext_sign_verify::pdf::f_2qb(&r_qd, r_qm, r_qn.sign_stamp))
        .flatten();
    if r_qn.sign_visible && r_qo.is_none() {
        tracing::warn!(
            "a2jDtG5nIHTDrG0gxJHGsOG7o2MgZm9udCBjaG8ga2jhu5FpIGNo4buvIGvDvSBoaeG7g24gdGjhu4sg4oCUIGvDvSBWw5QgSMOMTkggKGzhu4djaCDEkGnhu4F1IDguMS5jKQ=="
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
            "cGhpw6puIGtow7RuZyB0aHXhu5ljIHTDoGkgbGnhu4d1IG7DoHk=",
        ));
    }
    if r_qy.signer_user_id != Some(r_qq.user.id) {
        return Err(ApiError::forbidden());
    }
    if r_qy.status != "open" {
        return Err(ApiError::conflict("cGhpw6puIGvDvSBraMO0bmcgY8OybiBt4buf"));
    }
    let r_qv = r_qy.prepared_key.ok_or_else(|| {
        ApiError::conflict("cGhpw6puIGNoxrBhIGNodeG6qW4gYuG7iyAoZ+G7jWkgYmVnaW4gdHLGsOG7m2Mp")
    })?;
    let r_qz = r_qy
        .signed_attrs
        .ok_or_else(|| ApiError::conflict("cGhpw6puIHRoaeG6v3UgU2lnbmVkQXR0cmlidXRlcw=="))?;
    let cert = r_qy
        .signer_cert
        .ok_or_else(|| ApiError::conflict("cGhpw6puIHRoaeG6v3UgY2VydCBuZ8aw4budaSBrw70="))?;
    let intermediates = r_qy.signer_intermediates.unwrap_or_default();
    let r_qw = store
        .get(&r_qv)
        .await
        .map_err(storage_err)?
        .ok_or_else(|| {
            ApiError::conflict("UERGIMSRw6MgY2h14bqpbiBi4buLIGtow7RuZyBjw7JuIHRyb25nIGtobw==")
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
                format!("dmVyaWZ5IGNo4buvIGvDvSB0aOG6pXQgYuG6oWkgKMKnUzQuMSBixrDhu5tjIDQpOiA={r:?}")
            }
            Err(e) => {
                format!("dmVyaWZ5IGNo4buvIGvDvSB0aOG6pXQgYuG6oWkgKMKnUzQuMSBixrDhu5tjIDQpOiA={e}")
            }
        };
        tracing::warn!(
            session = % r_qy.id, document = % id, reason = % why,
            "dOG7qyBjaOG7kWkgcGhpw6puIGvDvQ=="
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
                let why = "Y2VydCBuZ8aw4budaSBrw70gQuG7iiBUSFUgSOG7kkkgKE9DU1AvQ1JMKSDigJQgdOG7qyBjaOG7kWkgbMawdSB0cuG7ryBjaOG7ryBrw70gKFBBZEVTLUxUKQ==";
                tracing::warn!(
                    session = % r_qy.id, document = % id, reason = why,
                    "dOG7qyBjaOG7kWkgcGhpw6puIGvDvSAoTFQp"
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
                    let why = "YXJjaGl2ZS10aW1lc3RhbXAgKFBBZEVTLUxUQSkga2jDtG5nIHTDoWkgeMOhYyBtaW5oIMSRxrDhu6NjIOKAlCBUU0EgdHLhuqMgdG9rZW4ga2jDtG5nIGjhu6NwIGzhu4cgKGtow7RuZyBwaOG6o2kgUlNBPyk7IHThu6sgY2jhu5FpIMSR4buDIHRyw6FuaCBsxrB1IGLhuqNuICdMVEEnIGdp4bqj";
                    tracing::error!(
                        session = % r_qy.id, document = % id,
                        "dOG7qyBjaOG7kWkgcGhpw6puIGvDvSAoTFRBKTog{why}"
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
                    "UEFkRVMtTFRBIGLhuq10IG5oxrBuZyB0aGnhur91IFRTQSBsw7pjIGvDvSDigJQgbMawdSBMVCAoa2jDtG5nIGFyY2hpdmUtdGltZXN0YW1wKQ=="
                );
                signed_pdf
            }
        }
    } else {
        if r_r1.pades_lta && !r_qs {
            tracing::warn!(
                session = % r_qy.id, document = % id,
                "UEFkRVMtTFRBIGLhuq10IG5oxrBuZyBjaOG7ryBrw70gS0jDlE5HIGPDsyAvRFNTIChMVCBraMO0bmcgdGh1IMSRxrDhu6NjIE9DU1AvQ1JMKSDigJQgbMawdSBCL1Q="
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
            "UEFkRVMtTFQgYuG6rXQgbmjGsG5nIHRoaeG6v3UgdHJ1c3Rfcm9vdHMg4oCUIGLhu48gcXVhIG5ow7puZyAvRFNT"
        );
        return LtOutcome::NoMaterial;
    };
    let Some(r_r7) = roots.resolve_issuer_der(r_rd, intermediates) else {
        tracing::warn!(
            "UEFkRVMtTFQ6IGtow7RuZyB0w6xtIMSRxrDhu6NjIGNlcnQgaXNzdWVyIMSR4buDIGzhuqV5IE9DU1AvQ1JMIOKAlCBsxrB1IEIvVA=="
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
                            "T0NTUCBraMO0bmcgdGluIMSRxrDhu6NjIChjaOG7ryBrw70gc2FpIC8gdW5rbm93bikg4oCUIGLhu48="
                        )
                    }
                    Err(e) => {
                        tracing::warn!(
                            % url, error =% e,
                            "T0NTUCBraMO0bmcgxJHhu41jIMSRxrDhu6NjIOKAlCBi4buP"
                        )
                    }
                },
                Err(e) => {
                    tracing::warn!(
                        % url, error =% e, "dOG6o2kgT0NTUCBs4buXaSDigJQgYuG7jw=="
                    )
                }
            },
            Err(e) => {
                tracing::warn!(
                    error =% e, "ZOG7sW5nIE9DU1AgcmVxdWVzdCBs4buXaSDigJQgYuG7jw=="
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
                        "Q1JMIGtow7RuZyB0aW4gxJHGsOG7o2MgKGNo4buvIGvDvSBzYWkpIOKAlCBi4buP"
                    )
                }
                Err(e) => {
                    tracing::warn!(
                        % url, error =% e,
                        "Q1JMIGtow7RuZyDEkeG7jWMgxJHGsOG7o2Mg4oCUIGLhu48="
                    )
                }
            },
            Err(e) => {
                tracing::warn!(% url, error =% e, "dOG6o2kgQ1JMIGzhu5dpIOKAlCBi4buP")
            }
        }
    }
    if r_r9.is_empty() && r_r5.is_empty() {
        tracing::warn!(
            "UEFkRVMtTFQ6IGtow7RuZyB0aHUgxJHGsOG7o2MgT0NTUC9DUkwgaOG7o3AgbOG7hyDigJQgbMawdSBCL1QgKGtow7RuZyAvRFNTKQ=="
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
            tracing::warn!(error =% e, "bmjDum5nIC9EU1MgbOG7l2kg4oCUIGzGsHUgQi9U");
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
                "cGjhuqNuIGjhu5NpIHF1w6EgbOG7m24gKA=={len} > {cap})"
            ));
        }
    }
    let r_rr = r_rs.bytes().await.map_err(|e| e.to_string())?;
    if r_rr.len() > cap {
        return Err(format!(
            "cGjhuqNuIGjhu5NpIHF1w6EgbOG7m24gKA=={} > {cap})",
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
        "ZOG7i2NoIHbhu6UgxJHDs25nIGThuqV1IHRo4budaSBnaWFuIChUU0EpIGtow7RuZyBz4bq1biBzw6BuZw==",
    )
}
const C_MV: usize = 256 * 1024;
const C_MW: std::time::Duration = std::time::Duration::from_secs(15);
async fn fetch_timestamp(r_ry: &str, signature: &[u8]) -> Result<Vec<u8>, ApiError> {
    if !r_ry.starts_with("https://") {
        return Err(
            ApiError::service_unavailable(
                "RElHSVRFWFRfVFNBX1VSTCBwaOG6o2kgZMO5bmcgaHR0cHMgKHRlbSB0aOG7nWkgZ2lhbiB0csOqbiBodHRwIGLhu4sgTUlUTSBz4butYSDEkcaw4bujYyk=",
            ),
        );
    }
    let nonce: u64 = rand::random();
    let r_rv =
        digitext_sign_verify::tsa::build_request(signature, Some(nonce), true).map_err(f_2f2)?;
    let client = reqwest::Client::builder()
        .timeout(C_MW)
        .build()
        .map_err(|e| f_2mn("ZOG7sW5nIEhUVFAgY2xpZW50", e))?;
    let r_rw = client
        .post(r_ry)
        .header("Content-Type", "application/timestamp-query")
        .body(r_rv)
        .send()
        .await
        .map_err(|e| f_2mn("Z+G7rWkgcmVxdWVzdCB0aOG6pXQgYuG6oWk=", e))?;
    if !r_rw.status().is_success() {
        return Err(f_2mn("SFRUUCBs4buXaQ==", r_rw.status()));
    }
    if let Some(len) = r_rw.content_length() {
        if len as usize > C_MV {
            return Err(f_2mn("cGjhuqNuIGjhu5NpIHF1w6EgbOG7m24=", len));
        }
    }
    let r_ru = r_rw
        .bytes()
        .await
        .map_err(|e| f_2mn("xJHhu41jIHBo4bqjbiBo4buTaSB0aOG6pXQgYuG6oWk=", e))?;
    if r_ru.len() > C_MV {
        return Err(f_2mn("cGjhuqNuIGjhu5NpIHF1w6EgbOG7m24=", r_ru.len()));
    }
    let token = digitext_sign_verify::tsa::f_2le(&r_ru)
        .map_err(|e| f_2mn("cGjhuqNuIGjhu5NpIGtow7RuZyBo4bujcCBs4buH", e))?;
    let r_rx = digitext_sign_verify::tsa::f_2py(&token, signature)
        .map_err(|e| f_2mn("dG9rZW4ga2jDtG5nIGjhu6NwIGzhu4c=", e))?;
    if !r_rx.check_nonce(nonce) {
        return Err(
            f_2mn(
                "bm9uY2UgdHJvbmcgdG9rZW4ga2jDtG5nIGto4bubcCBub25jZSDEkcOjIGfhu61pICh0b2tlbiBwaMOhdCBs4bqhaT8p",
                nonce,
            ),
        );
    }
    Ok(token)
}
