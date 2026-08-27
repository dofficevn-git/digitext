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
            eprintln!("a2jDtG5nIHThuqFvIMSRxrDhu6NjIHRva2lvIHJ1bnRpbWU6IA=={e}");
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
                "bOG7h25oIGtow7RuZyBo4bujcCBs4buHOiA={other}IChkw7luZzogc2VydmUgfCBib290c3RyYXAtc3VwZXJhZG1pbiAtLXVzZXJuYW1lIDx0w6puPiB8IGJhY2tmaWxsLXNvdXJjZS1wZGZzKQ=="
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
            "dGhp4bq/dSBj4bqldSBow6xuaCBNaW5JTyAoRElHSVRFWFRfTUlOSU9fKikg4oCUIGtow7RuZyB0aOG7gyBiYWNrZmlsbCAoYuG6rXQgc291cmNlX3N0b3JlIHRyxrDhu5tjKQ==",
        )?;
    let r_1u = digitext_db::connect(&r_1s.database_url)
        .await
        .context("a+G6v3QgbuG7kWkgUG9zdGdyZXMgdGjhuqV0IGLhuqFp")?;
    let store = digitext_storage::MinioObjectStore::new(&r_1t)
        .context("ZOG7sW5nIE1pbklPIG9iamVjdCBzdG9yZSB0aOG6pXQgYuG6oWk=")?;
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
        "QmFja2ZpbGwgeG9uZzogdOG7lW5nPQ=={}IMK3IMSRw6MgxJHhuql5PQ=={}IMK3IGLhu48gcXVhICjEkcOjIGPDsyk9{}IMK3IHRoaeG6v3UgZmlsZT0={}IMK3IGzhu5dpPQ=={}",
        r_1v.total, r_1v.uploaded, r_1v.skipped, r_1v.missing, r_1v.errors
    );
    if r_1v.errors > 0 {
        bail!(
            "Y8OzIA=={}IGzhu5dpIGtoaSBiYWNrZmlsbCDigJQgeGVtIGxvZw==",
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
        .context("a+G6v3QgbuG7kWkvbWlncmF0ZSBQb3N0Z3JlcyB0aOG6pXQgYuG6oWk=")?;
    let r_1x = JwtKeys::load_or_generate(&r_1w.jwt_key_path)?;
    let r_1y = tokio::net::TcpListener::bind(&r_1w.bind)
        .await
        .with_context(|| format!("a2jDtG5nIGJpbmQgxJHGsOG7o2Mg{}", r_1w.bind))?;
    tracing::info!(
        "ZGlnaXRleHQtYXBpIGzhuq9uZyBuZ2hlIHThuqFpIGh0dHA6Ly8={}",
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
    .context("ZOG7sW5nIGtobyBQREYgMiBs4bubcCB0aOG6pXQgYuG6oWk=")?
    .map(|r_23| {
        Arc::new(digitext_storage::WormGuard::new(r_23)) as Arc<dyn digitext_storage::ObjectStore>
    });
    if r_1w.pdf_store_bucket.is_none() && r_1w.pdf_store_dir.is_some() {
        tracing::warn!(
            "a2hvIFBERiAyIGzhu5twIGTDuW5nIMSQxKhBIEPhu6RDIELhu5gg4oCUIGNo4buJIMSRw7puZyBraGkgd29ya2VyIE9DUiBjaOG6oXkgQ8OZTkcgbcOheS4gICAgICAgICAgICAgIFdvcmtlciDhu58gbcOheSBraMOhYyB0aMOsIMSR4bq3dCBESUdJVEVYVF9QREZfU1RPUkVfQlVDS0VUIChNaW5JTykgY2hvIGPhuqMgaGFpLg=="
        );
    }
    if let Some(store) = r_20 {
        r_24 = r_24.with_store(store);
        tracing::info!(
            minio = r_1w.pdf_store_bucket.is_some(),
            "bHXhu5NuZyBrw70gc+G7kSArIGtp4buDbSBQREYvQSBC4bqsVCAoa2hvIG9iamVjdCBj4bqldSBow6xuaCk="
        );
        f_2gp(r_25);
    }
    match r_1w.minio {
        Some(r_1z) => match digitext_storage::MinioObjectStore::new(&r_1z) {
            Ok(store) => {
                tracing::info!(
                    bucket = % r_1z.bucket, endpoint = % r_1z.endpoint,
                    "a2hvIFBERiBuZ3Xhu5NuIE1pbklPIELhuqxUICh1cGxvYWQg4oaSIE9DUik="
                );
                r_24 = r_24.with_source_store(Arc::new(store));
            }
            Err(e) => {
                tracing::error!(
                    error = % e,
                    "ZOG7sW5nIE1pbklPIFBERiBuZ3Xhu5NuIGzhu5dpIOKAlCB0w6BpIGxp4buHdSB1cGxvYWQgc+G6vSBLSMOUTkcgT0NSIMSRxrDhu6Nj"
                );
            }
        },
        None => {
            tracing::warn!(
                "TWluSU8gUERGIG5ndeG7k24gQ0jGr0EgY+G6pXUgaMOsbmggKERJR0lURVhUX01JTklPXyopIOKAlCB0w6BpIGxp4buHdSB1cGxvYWQgY2jhu4kgbMawdSDEkcSpYSBBUEksIHdvcmtlciBPQ1Igc+G6vSBraMO0bmcgZmV0Y2ggxJHGsOG7o2M="
            )
        }
    }
    match r_1w.sign_trust_roots.as_deref() {
        Some(path) => {
            let pem = std::fs::read_to_string(path)
                .unwrap_or_else(|e| {
                    panic!(
                        "a2jDtG5nIMSR4buNYyDEkcaw4bujYyBESUdJVEVYVF9TSUdOX1RSVVNUX1JPT1RTPSc={path}': {e}"
                    )
                });
            let roots =
                digitext_sign_verify::chain::TrustStore::from_pem(&pem).unwrap_or_else(|e| {
                    panic!("DIGITEXT_SIGN_TRUST_ROOTS='{path}JyBraMO0bmcgaOG7o3AgbOG7hzog{e}")
                });
            tracing::info!(
                path = % path, roots = roots.len(),
                "dHJ1c3Qgc3RvcmUgY2jhu68ga8O9IMSQw4MgbuG6oXA="
            );
            r_24 = r_24.with_trust_roots(roots);
        }
        None => {
            tracing::warn!(
                "RElHSVRFWFRfU0lHTl9UUlVTVF9ST09UUyBjaMawYSBj4bqldSBow6xuaCDigJQgS0jDlE5HIGtp4buDbSDEkcaw4bujYyBjaHXhu5dpIGNo4bupbmcgdGjGsCBuw6puIG3hu41pIHnDqnUgY+G6p3Uga8O9IHPhur0gYuG7iyB04burIGNo4buRaSAoxJHhurd0IGJ1bmRsZSBQRU0gcm9vdCBj4bunYSBDQSBwaMOhdCBow6BuaCB0b2tlbiDEkeG7gyBi4bqtdCBrw70gc+G7kTsgdG9rZW4gSS1DQSBkw7luZyB0cnVzdC9pY2Etc2lnbi10cnVzdC1yb290cy5wZW0gdHJvbmcgcmVwbyk="
            )
        }
    }
    if let Some(r_27) = r_1w.tsa_url {
        tracing::info!(
            tsa = % r_27, "xJHDs25nIGThuqV1IHRo4budaSBnaWFuIFRTQSBC4bqsVCAoUEFkRVMtVCk="
        );
        r_24 = r_24.with_tsa_url(r_27);
    }
    match r_1w.tsa_trust_roots.as_deref() {
        Some(path) => {
            let pem = std::fs::read_to_string(path)
                .unwrap_or_else(|e| {
                    panic!(
                        "a2jDtG5nIMSR4buNYyDEkcaw4bujYyBESUdJVEVYVF9UU0FfVFJVU1RfUk9PVFM9Jw=={path}': {e}"
                    )
                });
            let roots =
                digitext_sign_verify::chain::TrustStore::from_pem(&pem).unwrap_or_else(|e| {
                    panic!("DIGITEXT_TSA_TRUST_ROOTS='{path}JyBraMO0bmcgaOG7o3AgbOG7hzog{e}")
                });
            tracing::info!(
                path = % path, roots = roots.len(),
                "dHJ1c3Qgc3RvcmUgVFNBIMSQw4MgbuG6oXA="
            );
            r_24 = r_24.with_tsa_trust_roots(roots);
        }
        None => {
            tracing::warn!(
                "RElHSVRFWFRfVFNBX1RSVVNUX1JPT1RTIGNoxrBhIGPhuqV1IGjDrG5oIOKAlCB0ZW0gdGjhu51pIGdpYW4gY2jhu4kg4bufIG3hu6ljIGPhuqV1IHRyw7pjLCBLSMOUTkcgdGluIMSRxrDhu6NjIGtoaSB0w6FpIHjDoWMgbWluaCAoxJHhurd0IGJ1bmRsZSBQRU0gcm9vdCBUU0EgxJHhu4MgYuG6rXQga2nhu4NtIGNodeG7l2kgdGVtIOKAlCBIMyk="
            )
        }
    }
    if r_1w.pades_lt {
        tracing::info!(
            "UEFkRVMtTFQgQuG6rFQg4oCUIGNvbXBsZXRlIHPhur0gdOG6o2kgT0NTUC9DUkwgKyBuaMO6bmcgL0RTUyAobG9uZy10ZXJtIHZhbGlkYXRpb24p"
        );
        r_24 = r_24.with_pades_lt(true);
    }
    use crate::routes::sign::{f_1pw, LtaStartup};
    match f_1pw(r_1w.pades_lta, r_1w.pades_lt, r_24.tsa_url.is_some()) {
        LtaStartup::Enable => {
            tracing::info!(
                "UEFkRVMtTFRBIELhuqxUIOKAlCBjb21wbGV0ZSBz4bq9IMSRw7NuZyBEb2NUaW1lU3RhbXAgcGjhu6cgL0RTUyAoYXJjaGl2ZS10aW1lc3RhbXApIHNhdSBMVA=="
            );
            r_24 = r_24.with_pades_lta(true);
        }
        LtaStartup::MissingLt => {
            tracing::error!(
                "RElHSVRFWFRfUEFERVNfTFRBIGLhuq10IG5oxrBuZyBESUdJVEVYVF9QQURFU19MVCBU4bquVCDigJQgTFRBIGPhuqduIC9EU1MgxJHhu4MgcGjhu6c7IELhu44gUVVBIExUQSAoYuG6rXQgRElHSVRFWFRfUEFERVNfTFQp"
            )
        }
        LtaStartup::MissingTsa => {
            tracing::error!(
                "RElHSVRFWFRfUEFERVNfTFRBIGLhuq10IG5oxrBuZyB0aGnhur91IERJR0lURVhUX1RTQV9VUkwg4oCUIExUQSBj4bqnbiBUU0EgKGh0dHBzLCBSU0EpIMSR4buDIMSRw7NuZyBhcmNoaXZlLXRpbWVzdGFtcDsgQuG7jiBRVUEgTFRB"
            )
        }
        LtaStartup::Off => {}
    }
    r_24 = r_24.with_sign_appearance(r_1w.sign_visible, r_1w.sign_stamp);
    if r_1w.sign_stamp {
        tracing::warn!(
            "RElHSVRFWFRfU0lHTl9TVEFNUCBi4bqtdDoga8O9IGvDqG0gQ09OIEThuqRVIFRSw5JOIOKAlCBs4buHY2ggxJBp4buBdSA4LjEuYyBUVCAwNS8yMDI1IChUaMO0bmcgdMawIHF1eSDEkeG7i25oIGto4buRaSB0ZXh0IFRpbWVzIDEwIHbDoCBj4bqlbSBoaeG7g24gdGjhu4sgaMOsbmgg4bqjbmggZOG6pXUp"
        );
    } else if !r_1w.sign_visible {
        tracing::warn!(
            "RElHSVRFWFRfU0lHTl9WSVNJQkxFIHThuq90OiBrw70gVsOUIEjDjE5IIOKAlCBs4buHY2ggxJBp4buBdSA4LjEuYyBUVCAwNS8yMDI1"
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
                        "c3dlZXBlcjogcmVsZWFzZSBwaGnDqm4ga8O9IGjhur90IGjhuqFuIOKGkiB0b19zaWdu"
                    )
                }
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!(
                        error = % e,
                        "c3dlZXBlciBwaGnDqm4ga8O9IGzhu5dpIChz4bq9IHRo4butIGzhuqFpKQ=="
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
                        "cmVhcGVyOiDEkcOzbmcgxJHhu6N0IG7hu5lwIGLhu48gZOG7nyDihpIgYWJhbmRvbmVk"
                    )
                }
                Ok(_) => {}
                Err(e) => {
                    tracing::warn!(
                        error = % e,
                        "cmVhcGVyIMSR4bujdCBu4buZcCBs4buXaSAoc+G6vSB0aOG7rSBs4bqhaSk="
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
            other => bail!("dGhhbSBz4buRIGtow7RuZyBo4bujcCBs4buHOiA={other}"),
        }
    }
    let username = username.context("dGhp4bq/dSAtLXVzZXJuYW1lIDx0w6puPg==")?;
    Ok((username.to_owned(), r_2h.map(str::to_owned)))
}
async fn f_v2(r_2j: &[String]) -> anyhow::Result<()> {
    let (username, r_2l) = f_1ye(r_2j)?;
    let r_2k = Config::from_env()?;
    let r_2m = digitext_db::f_11d(&r_2k.database_url).await?;
    if users::count(&r_2m).await? > 0 {
        bail!(
            "YuG6o25nIHVzZXJzIMSRw6MgY8OzIGThu68gbGnhu4d1IOKAlCBib290c3RyYXAtc3VwZXJhZG1pbiBjaOG7iSBjaOG6oXkgdHLDqm4gaOG7hyB0aOG7kW5nIHRy4buRbmc="
        );
    }
    let r_2n = rpassword::prompt_password("TeG6rXQga2jhuql1IHN1cGVyIGFkbWluOiA=")?;
    let r_2o = rpassword::prompt_password("Tmjhuq1wIGzhuqFpIG3huq10IGto4bqpdTog")?;
    if r_2n != r_2o {
        bail!("aGFpIGzhuqduIG5o4bqtcCBt4bqtdCBraOG6qXUga2jDtG5nIGto4bubcA==");
    }
    if r_2n.chars().count() < 8 {
        bail!("beG6rXQga2jhuql1IHBo4bqjaSB04buRaSB0aGnhu4N1IDgga8O9IHThu7E=");
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
        "xJDDoyB04bqhbyBzdXBlciBhZG1pbiAn{}' (id {}).",
        r_2p.username, r_2p.id
    );
    Ok(())
}
