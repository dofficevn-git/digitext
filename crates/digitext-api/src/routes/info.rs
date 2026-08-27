// Copyright (c) 2026 DigiText. All rights reserved.
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::{f_29a, Scope, Ts};
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use chrono::{DateTime, Utc};
use digitext_core::Permission;
use digitext_db::info::{self, T9v, Tdu};
use digitext_db::{notifications, projects};
use digitext_pdf::FormBlock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/info", get(list_requests).post(create_request))
        .route("/info/{id}", get(get_request))
        .route("/info/{id}/approve", post(approve_request))
        .route("/info/{id}/refuse", post(refuse_request))
        .route("/info/{id}/pay", post(confirm_payment))
        .route("/info/{id}/generate", post(generate))
        .route("/info/{id}/fulfill", post(fulfill))
        .route("/info/{id}/download", get(download))
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T3a {
    pub project_id: Option<Uuid>,
    pub hinh_thuc: String,
    pub chu_de: String,
    pub muc_dich: String,
    pub phuong_thuc: Option<String>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InfoDto {
    pub id: Uuid,
    pub project_id: Uuid,
    pub requester: Option<Uuid>,
    pub hinh_thuc: String,
    pub chu_de: String,
    pub muc_dich: String,
    pub phuong_thuc: Option<String>,
    pub status: String,
    pub review_note: Option<String>,
    pub phi: Option<i64>,
    pub paid_at: Option<DateTime<Utc>>,
    pub co_ket_qua: bool,
    pub bien_tap: Option<String>,
    pub approved_at: Option<DateTime<Utc>>,
    pub fulfilled_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
impl From<T9v> for InfoDto {
    fn from(r_i8: T9v) -> Self {
        InfoDto {
            id: r_i8.id,
            project_id: r_i8.project_id,
            requester: r_i8.requester,
            hinh_thuc: r_i8.hinh_thuc,
            chu_de: r_i8.chu_de,
            muc_dich: r_i8.muc_dich,
            phuong_thuc: r_i8.phuong_thuc,
            status: r_i8.status,
            review_note: r_i8.review_note,
            phi: r_i8.phi,
            paid_at: r_i8.paid_at,
            co_ket_qua: r_i8.ket_qua_key.is_some(),
            bien_tap: r_i8.bien_tap,
            approved_at: r_i8.approved_at,
            fulfilled_at: r_i8.fulfilled_at,
            created_at: r_i8.created_at,
            updated_at: r_i8.updated_at,
        }
    }
}
fn can_touch_project(r_i9: &Ts, r_ib: &Scope, r_ia: Uuid) -> bool {
    r_i9.can_touch_project(r_ib, r_ia)
}
async fn load_in_scope(r_if: &AppState, r_ic: &Ts, id: Uuid) -> Result<T9v, ApiError> {
    let r_id = info::get(&r_if.pool, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let r_ie = f_29a(&r_if.pool, &r_ic.user).await?;
    if !can_touch_project(r_ic, &r_ie, r_id.project_id) {
        return Err(ApiError::forbidden());
    }
    Ok(r_id)
}
fn require_owner_or_reviewer(r_ig: &Ts, r_ih: &T9v) -> Result<(), ApiError> {
    if r_ih.requester == Some(r_ig.user.id) || r_ig.has_perm(Permission::AccessReview) {
        Ok(())
    } else {
        Err(ApiError::forbidden())
    }
}
fn note_of(r_ij: &Option<String>) -> Option<&str> {
    r_ij.as_deref()
        .map(str::trim)
        .filter(|r_ii| !r_ii.is_empty())
}
async fn notify_requester(r_im: &AppState, r_il: &T9v, r_ik: &str) {
    if let Some(r_in) = r_il.requester {
        if let Err(e) = notifications::f_1v8(&r_im.pool, r_in, None, "info_status", r_ik).await {
            tracing::warn!(
                error = % e,
                "a2jDtG5nIGdoaSDEkcaw4bujYyB0aMO0bmcgYsOhbyBjdW5nIGPhuqVwIHRow7RuZyB0aW4="
            );
        }
    }
}
async fn create_request(
    State(r_is): State<AppState>,
    r_io: Ts,
    Json(r_ip): Json<T3a>,
) -> Result<(StatusCode, Json<InfoDto>), ApiError> {
    r_io.require_perm(Permission::AccessRequest)?;
    let project_id = r_ip.project_id.or(r_io.user.project_id).ok_or_else(|| {
        ApiError::bad_request(
            "dGhp4bq/dSBwcm9qZWN0SWQgKG5nxrDhu51pIGfhu41pIGtow7RuZyB0aHXhu5ljIGThu7Egw6FuIG7DoG8p",
        )
    })?;
    let r_ir = f_29a(&r_is.pool, &r_io.user).await?;
    if !can_touch_project(&r_io, &r_ir, project_id) {
        return Err(ApiError::forbidden());
    }
    if !matches!(
        r_ip.hinh_thuc.as_str(),
        "danh_muc" | "trich_xuat" | "tong_hop"
    ) {
        return Err(
            ApiError::bad_request(
                "aGluaFRodWMgbmjhuq1uICdkYW5oX211YycgfCAndHJpY2hfeHVhdCcgfCAndG9uZ19ob3AnICjEkGnhu4F1IDQ0KQ==",
            ),
        );
    }
    if r_ip.chu_de.trim().is_empty() {
        return Err(ApiError::bad_request(
            "dGhp4bq/dSBjaOG7pyDEkeG7gSBj4bulIHRo4buDICjEkGnhu4F1IDQ0LjEuYSk=",
        ));
    }
    if r_ip.muc_dich.trim().is_empty() {
        return Err(ApiError::bad_request(
            "cGjhuqNpIGtoYWkgbeG7pWMgxJHDrWNoIHPhu60gZOG7pW5nICjEkGnhu4F1IDQ1LjMp",
        ));
    }
    let r_iq = info::create(
        &r_is.pool,
        Tdu {
            project_id,
            requester: Some(r_io.user.id),
            hinh_thuc: &r_ip.hinh_thuc,
            chu_de: r_ip.chu_de.trim(),
            muc_dich: r_ip.muc_dich.trim(),
            phuong_thuc: note_of(&r_ip.phuong_thuc),
        },
    )
    .await?;
    Ok((StatusCode::CREATED, Json(r_iq.into())))
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
async fn list_requests(
    State(r_ix): State<AppState>,
    r_it: Ts,
    Query(r_iu): Query<ListQuery>,
) -> Result<Json<Vec<InfoDto>>, ApiError> {
    r_it.require_perm(Permission::AccessView)?;
    let limit = r_iu.limit.unwrap_or(50).clamp(1, 200);
    let offset = r_iu.offset.unwrap_or(0).max(0);
    let r_iw = f_29a(&r_ix.pool, &r_it.user).await?;
    let projects = r_it.list_projects(r_iw);
    let rows = info::list(&r_ix.pool, projects.as_deref(), limit, offset).await?;
    let rows: Vec<T9v> = if r_it.has_perm(Permission::AccessReview) {
        rows
    } else {
        rows.into_iter()
            .filter(|r_iv| r_iv.requester == Some(r_it.user.id))
            .collect()
    };
    Ok(Json(rows.into_iter().map(InfoDto::from).collect()))
}
async fn get_request(
    State(r_j0): State<AppState>,
    r_iy: Ts,
    Path(id): Path<Uuid>,
) -> Result<Json<InfoDto>, ApiError> {
    r_iy.require_perm(Permission::AccessView)?;
    let r_iz = load_in_scope(&r_j0, &r_iy, id).await?;
    require_owner_or_reviewer(&r_iy, &r_iz)?;
    Ok(Json(r_iz.into()))
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteBody {
    pub note: Option<String>,
}
async fn approve_request(
    State(r_j4): State<AppState>,
    r_j1: Ts,
    Path(id): Path<Uuid>,
    Json(r_j2): Json<NoteBody>,
) -> Result<Json<InfoDto>, ApiError> {
    r_j1.require_perm(Permission::AccessReview)?;
    let r_j3 = load_in_scope(&r_j4, &r_j1, id).await?;
    let Some(approved) = info::approve(&r_j4.pool, r_j3.id, note_of(&r_j2.note)).await? else {
        return Err(ApiError::conflict(format!(
            "Y2jhu4kgZHV54buHdCDEkcaw4bujYyB5w6p1IGPhuqd1IGNo4budIGR1eeG7h3QgKMSRYW5nIGzDoCAn{}')",
            r_j3.status
        )));
    };
    let phi = approved.phi.unwrap_or(0);
    notify_requester(
            &r_j4,
            &approved,
            &format!(
                "WcOqdSBj4bqndSBjdW5nIGPhuqVwIHRow7RuZyB0aW4gxJHDoyBEVVnhu4ZUIOKAlCBwaMOtIA=={phi}IMSR4buTbmcgKMSQaeG7gXUgNDYuMS5jKQ=="
            ),
        )
        .await;
    Ok(Json(approved.into()))
}
async fn refuse_request(
    State(r_j9): State<AppState>,
    r_j5: Ts,
    Path(id): Path<Uuid>,
    Json(r_j6): Json<NoteBody>,
) -> Result<Json<InfoDto>, ApiError> {
    r_j5.require_perm(Permission::AccessReview)?;
    let r_j8 = load_in_scope(&r_j9, &r_j5, id).await?;
    let note = note_of(&r_j6.note).ok_or_else(|| {
        ApiError::bad_request(
            "dOG7qyBjaOG7kWkgcGjhuqNpIG7DqnUgcsO1IGzDvSBkbyAoxJBp4buBdSA0Ni4xLmMp",
        )
    })?;
    if !info::refuse(&r_j9.pool, r_j8.id, note).await? {
        return Err(
            ApiError::conflict(
                format!(
                    "Y2jhu4kgdOG7qyBjaOG7kWkgxJHGsOG7o2MgecOqdSBj4bqndSBjaOG7nSBkdXnhu4d0ICjEkWFuZyBsw6AgJw=={}')",
                    r_j8.status
                ),
            ),
        );
    }
    let r_j7 = info::get(&r_j9.pool, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    notify_requester(
        &r_j9,
        &r_j7,
        &format!("WcOqdSBj4bqndSBjdW5nIGPhuqVwIHRow7RuZyB0aW4gYuG7iyBU4buqIENI4buQSTog{note}"),
    )
    .await;
    Ok(Json(r_j7.into()))
}
async fn confirm_payment(
    State(r_jd): State<AppState>,
    r_ja: Ts,
    Path(id): Path<Uuid>,
) -> Result<Json<InfoDto>, ApiError> {
    r_ja.require_perm(Permission::AccessReview)?;
    let r_jc = load_in_scope(&r_jd, &r_ja, id).await?;
    if !info::confirm_payment(&r_jd.pool, r_jc.id).await? {
        return Err(
            ApiError::conflict(
                "Y2jhu4kgeMOhYyBuaOG6rW4gdGhhbmggdG/DoW4gY2hvIHnDqnUgY+G6p3UgxJHDoyBkdXnhu4d0IHbDoCBjaMawYSB0aGFuaCB0b8Ohbg=="
                    .to_string(),
            ),
        );
    }
    let r_jb = info::get(&r_jd.pool, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    notify_requester(
        &r_jd,
        &r_jb,
        "xJDDoyB4w6FjIG5o4bqtbiB0aGFuaCB0b8OhbiBwaMOtICjEkGnhu4F1IDQ2LjQp",
    )
    .await;
    Ok(Json(r_jb.into()))
}
fn f_2pl(r_je: &str, r_jh: &str, r_jg: usize) -> String {
    let r_jf = r_je.to_lowercase();
    let r_jj = r_jf.find(&r_jh.to_lowercase()).unwrap_or(0);
    let r_jl = r_je[..r_jj].chars().count().saturating_sub(r_jg / 4);
    let r_jk: String = r_je.chars().skip(r_jl).take(r_jg).collect();
    let mut r_ji = String::new();
    if r_jl > 0 {
        r_ji.push('…');
    }
    r_ji.push_str(r_jk.trim());
    if r_je.chars().count() > r_jl + r_jg {
        r_ji.push('…');
    }
    r_ji
}
fn f_1kc(project_id: Uuid, id: Uuid) -> String {
    format!("{project_id}-info/{id}.pdf")
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T8k {
    pub bien_tap: Option<String>,
}
async fn generate(
    State(r_jv): State<AppState>,
    r_jm: Ts,
    Path(id): Path<Uuid>,
    Json(r_jo): Json<T8k>,
) -> Result<Json<InfoDto>, ApiError> {
    r_jm.require_perm(Permission::AccessReview)?;
    let r_ju = load_in_scope(&r_jv, &r_jm, id).await?;
    if r_ju.status != "approved" {
        return Err(
            ApiError::conflict(
                format!(
                    "Y2jhu4kga+G6v3QgeHXhuqV0IHNhdSBraGkgZHV54buHdCB2w6AgdHLGsOG7m2Mga2hpIHRy4bqjICjEkWFuZyBsw6AgJw=={}')",
                    r_ju.status
                ),
            ),
        );
    }
    let r_jw = r_jv
        .store
        .as_ref()
        .ok_or_else(|| ApiError::service_unavailable("a2hvIG9iamVjdCBjaMawYSBj4bqldSBow6xuaA=="))?;
    let (quota, _) = info::f_25g(&r_jv.pool, r_ju.project_id).await?;
    let org = projects::get(&r_jv.pool, r_ju.project_id)
        .await?
        .map(|r_js| r_js.name)
        .unwrap_or_else(|| "TMawdSB0cuG7ryBs4buLY2ggc+G7rQ==".to_string());
    let mut r_jn: Vec<FormBlock> = vec![
        FormBlock::Field {
            label: "Q2jhu6cgxJHhu4E=".into(),
            value: r_ju.chu_de.clone(),
        },
        FormBlock::Field {
            label: "TeG7pWMgxJHDrWNoIHPhu60gZOG7pW5n".into(),
            value: r_ju.muc_dich.clone(),
        },
    ];
    let r_jy;
    let r_jx = match r_ju.hinh_thuc.as_str() {
        "danh_muc" => {
            let hits = info::f_x8(&r_jv.pool, r_ju.project_id, &r_ju.chu_de, quota + 1).await?;
            r_jy = hits.len() as i64 > quota;
            let hits = &hits[..hits.len().min(quota as usize)];
            r_jn.push(FormBlock::SectionHeading(format!(
                "REFOSCBN4bukQyBUw4BJIExJ4buGVSAo{}IG3hu6VjKQ==",
                hits.len()
            )));
            r_jn.push(FormBlock::Bullets(
                hits.iter()
                    .map(|h| {
                        format!(
                            "{} | {} | {} trang | {}",
                            h.ho_so_ref.as_deref().unwrap_or("—"),
                            h.filename,
                            h.page_count.unwrap_or(0),
                            h.uploaded_at.format("%d/%m/%Y"),
                        )
                    })
                    .collect(),
            ));
            "REFOSCBN4bukQyBUw4BJIExJ4buGVSBMxq9VIFRS4buuIFRIRU8gQ0jhu6YgxJDhu4A="
        }
        "trich_xuat" => {
            let hits = info::f_1b9(&r_jv.pool, r_ju.project_id, &r_ju.chu_de, quota + 1).await?;
            r_jy = hits.len() as i64 > quota;
            let hits = &hits[..hits.len().min(quota as usize)];
            r_jn.push(FormBlock::SectionHeading(format!(
                "TuG7mEkgRFVORyBUUsONQ0ggWFXhuqRUIE5HVVnDik4gVsSCTiAo{}IMSRb+G6oW4p",
                hits.len()
            )));
            for h in hits {
                r_jn.push(FormBlock::Field {
                    label: "xJDhu4thIGNo4buJIGzGsHUgdHLhu68=".into(),
                    value: format!(
                        "{} — {} — trang {}IChsacOqbiBr4bq/dCBn4buRYzog{})",
                        h.ho_so_ref.as_deref().unwrap_or("—"),
                        h.filename,
                        h.page_no,
                        h.document_id
                    ),
                });
                r_jn.push(FormBlock::Paragraph(format!(
                    "“{}”",
                    f_2pl(&h.content, &r_ju.chu_de, 400)
                )));
            }
            "VEjDlE5HIFRJTiBUUsONQ0ggWFXhuqRUIFThu6ogVMOASSBMSeG7hlUgTMavVSBUUuG7riBT4buQ"
        }
        _ => {
            let rows_ = info::f_2i4(&r_jv.pool, r_ju.project_id, &r_ju.chu_de, quota + 1).await?;
            r_jy = rows_.len() as i64 > quota;
            let rows_ = &rows_[..rows_.len().min(quota as usize)];
            let tong_tl: i64 = rows_.iter().map(|r| r.so_tai_lieu).sum();
            let tong_trang: i64 = rows_.iter().map(|r| r.tong_trang).sum();
            r_jn.push(FormBlock::SectionHeading(
                "QsOBTyBDw4FPIFThu5RORyBI4buiUCBU4buwIMSQ4buYTkc=".into(),
            ));
            r_jn.push(FormBlock::Field {
                label: "VOG7lW5nIHPhu5EgaOG7kyBzxqEgbGnDqm4gcXVhbg==".into(),
                value: rows_.len().to_string(),
            });
            r_jn.push(FormBlock::Field {
                label: "VOG7lW5nIHPhu5EgdMOgaSBsaeG7h3UgLyB04buVbmcgc+G7kSB0cmFuZw==".into(),
                value: format!("{tong_tl} / {tong_trang}"),
            });
            r_jn.push(
                FormBlock::Bullets(
                    rows_
                        .iter()
                        .map(|r| {
                            format!(
                                "SOG7kyBzxqEg{}: {}IHTDoGkgbGnhu4d1LCA={} trang, {}IHRyYW5nIGto4bubcCBu4buZaSBkdW5n",
                                r.ho_so_ref.as_deref().unwrap_or("—"), r.so_tai_lieu, r
                                .tong_trang, r.so_trang_khop,
                            )
                        })
                        .collect(),
                ),
            );
            "VEjDlE5HIFRJTiBU4buUTkcgSOG7olAgVOG7qiBDU0RMIFTDgEkgTEnhu4ZVIEzGr1UgVFLhu64="
        }
    };
    if r_jy {
        r_jn.push(
            FormBlock::Paragraph(
                format!(
                    "KEvhur90IHF14bqjIHbGsOG7o3QgZ2nhu5tpIGjhuqFuIA=={quota}IG3hu6VjIGNobyBt4buZdCBs4bqnbiB5w6p1IGPhuqd1IOKAlCDEkGnhu4F1IDQ0LjQ7IHRodSBo4bq5cCBjaOG7pyDEkeG7gSBob+G6t2MgdMOhY2ggecOqdSBj4bqndS4p"
                ),
            ),
        );
    }
    if let Some(r_jp) = note_of(&r_jo.bien_tap) {
        r_jn.push(FormBlock::SectionHeading(
            "UsOAIFNPw4FULCBCScOKTiBU4bqsUCAoxJBp4buBdSA0Ny4yKQ==".into(),
        ));
        r_jn.push(FormBlock::Paragraph(r_jp.to_string()));
    }
    r_jn.push(FormBlock::SectionHeading("WMOBQyBUSOG7sEM=".into()));
    r_jn.push(
        FormBlock::Paragraph(
            format!(
                "S+G6v3QgcXXhuqMgxJHGsOG7o2Mga+G6v3QgeHXhuqV0IHThu6sgSOG7hyB0aOG7kW5nIHF14bqjbiBsw70gdMOgaSBsaeG7h3UgbMawdSB0cuG7ryBz4buRIHbDoCDEkcaw4bujYyB4w6FjIHRo4buxYyBi4bufaSA={org}LiBUaOG7nWkgxJFp4buDbSBr4bq/dCB4deG6pXQ6IA=={} (+07:00).",
                Utc::now().with_timezone(& chrono::FixedOffset::east_opt(7 * 3600)
                .expect("+07:00")).format("%d/%m/%Y %H:%M"),
            ),
        ),
    );
    let r_jq = digitext_pdf::f_1cq().ok_or_else(|| {
        ApiError::service_unavailable(
            "a2jDtG5nIHTDrG0gdGjhuqV5IGZvbnQgVFRGIOKAlCDEkeG6t3QgRElHSVRFWFRfQ0VSVF9GT05U",
        )
    })?;
    let font = std::fs::read(&r_jq).map_err(|e| {
        tracing::error!(
            path = % r_jq.display(), error = % e, "xJHhu41jIGZvbnQgbOG7l2k="
        );
        ApiError::internal()
    })?;
    let pdf = digitext_pdf::f_ve(
            r_jx,
            Some(
                "KEN1bmcgY+G6pXAgdGjDtG5nIHRpbiB0aGVvIE3hu6VjIDQgQ2jGsMahbmcgVkkg4oCUIFRow7RuZyB0xrAgMDUvMjAyNS9UVC1CTlYp",
            ),
            &r_jn,
            &font,
        )
        .map_err(|e| ApiError::bad_request(
            format!("ZOG7sW5nIFBERiBr4bq/dCBxdeG6oyBs4buXaTog{e}"),
        ))?;
    let r_jr = f_1kc(r_ju.project_id, r_ju.id);
    r_jw.put(&r_jr, &pdf).await.map_err(|e| {
        tracing::error!(
            error = % e,
            "UFVUIGvhur90IHF14bqjIGN1bmcgY+G6pXAgdGjDtG5nIHRpbiBs4buXaQ=="
        );
        ApiError::internal()
    })?;
    info::f_2ee(&r_jv.pool, r_ju.id, &r_jr, note_of(&r_jo.bien_tap)).await?;
    let r_jt = info::get(&r_jv.pool, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    Ok(Json(r_jt.into()))
}
async fn fulfill(
    State(r_k2): State<AppState>,
    r_jz: Ts,
    Path(id): Path<Uuid>,
) -> Result<Json<InfoDto>, ApiError> {
    r_jz.require_perm(Permission::AccessReview)?;
    let r_k1 = load_in_scope(&r_k2, &r_jz, id).await?;
    if !info::fulfill(&r_k2.pool, r_k1.id).await? {
        return Err(
            ApiError::conflict(
                "Y2jhu4kgdHLhuqMga+G6v3QgcXXhuqMga2hpIMSRw6MgZHV54buHdCB2w6AgxJDDgyBr4bq/dCB4deG6pXQgKMSQaeG7gXUgNDcuMyk="
                    .to_string(),
            ),
        );
    }
    let r_k0 = info::get(&r_k2.pool, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    notify_requester(
            &r_k2,
            &r_k0,
            "S+G6v3QgcXXhuqMgY3VuZyBj4bqlcCB0aMO0bmcgdGluIMSRw6Mgc+G6tW4gc8Ogbmcg4oCUIHThuqNpIHThu6sgdMOgaSBraG/huqNuICjEkGnhu4F1IDQ3LjQp",
        )
        .await;
    Ok(Json(r_k0.into()))
}
async fn download(
    State(r_k8): State<AppState>,
    r_k3: Ts,
    Path(id): Path<Uuid>,
) -> Result<Response, ApiError> {
    r_k3.require_perm(Permission::AccessView)?;
    let r_k7 = load_in_scope(&r_k8, &r_k3, id).await?;
    require_owner_or_reviewer(&r_k3, &r_k7)?;
    let r_k5 = r_k3.has_perm(Permission::AccessReview);
    if !r_k5 {
        if r_k7.status != "fulfilled" {
            return Err(ApiError::conflict(
                "a+G6v3QgcXXhuqMgY2jGsGEgxJHGsOG7o2MgdHLhuqMgKMSQaeG7gXUgNDcuNCk=".to_string(),
            ));
        }
        if !r_k7.fee_settled() {
            return Err(ApiError::conflict(
                "Y2jGsGEgdGhhbmggdG/DoW4gcGjDrSAoxJBp4buBdSA0Ni4zLmIvNDYuNCk=".to_string(),
            ));
        }
    }
    let r_k6 = r_k7.ket_qua_key.as_deref().ok_or_else(|| {
        ApiError::conflict(
            "Y2jGsGEga+G6v3QgeHXhuqV0IGvhur90IHF14bqjICjEkGnhu4F1IDQ3LjEp".to_string(),
        )
    })?;
    let r_k9 = r_k8
        .store
        .as_ref()
        .ok_or_else(|| ApiError::service_unavailable("a2hvIG9iamVjdCBjaMawYSBj4bqldSBow6xuaA=="))?;
    let r_k4 = r_k9
        .get(r_k6)
        .await
        .map_err(|e| {
            tracing::error!(error = % e, "xJHhu41jIGvhur90IHF14bqjIGzhu5dp");
            ApiError::internal()
        })?
        .ok_or_else(ApiError::not_found)?;
    Ok((
        [
            (header::CONTENT_TYPE, "application/pdf".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"cung-cap-thong-tin-{id}.pdf\""),
            ),
        ],
        Body::from(r_k4),
    )
        .into_response())
}
