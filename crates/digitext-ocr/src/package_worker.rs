// Copyright (c) 2026 DigiText. All rights reserved.
use chrono::FixedOffset;
use digitext_core::DocStatus;
use digitext_db::packages::{self, T1e, Tgc};
use digitext_package::wire::Tq4;
use digitext_package::{f_vn, ids::Tic, PackageKind, Tlv};
use sqlx::PgPool;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
#[derive(Debug, PartialEq, Eq)]
pub enum ScanVerdict {
    Clean,
    Infected(String),
}
type Tkn<'a> =
    std::pin::Pin<Box<dyn std::future::Future<Output = Result<ScanVerdict, String>> + Send + 'a>>;
pub trait Tpu: Send + Sync {
    fn name(&self) -> &str;
    fn scan<'a>(&'a self, bytes: &'a [u8]) -> Tkn<'a>;
}
pub struct ClamdScanner {
    addr: String,
}
impl ClamdScanner {
    pub fn new(addr: impl Into<String>) -> Self {
        ClamdScanner { addr: addr.into() }
    }
    async fn instream(&self, r_3bw: &[u8]) -> Result<ScanVerdict, String> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let mut r_3by = tokio::net::TcpStream::connect(&self.addr)
            .await
            .map_err(|e| format!("kết nối clamd {} lỗi: {e}", self.addr))?;
        r_3by
            .write_all(b"zINSTREAM\0")
            .await
            .map_err(|e| format!("gửi INSTREAM lỗi: {e}"))?;
        for r_3bx in r_3bw.chunks(64 * 1024) {
            r_3by
                .write_all(&(r_3bx.len() as u32).to_be_bytes())
                .await
                .map_err(|e| format!("gửi khúc dữ liệu lỗi: {e}"))?;
            r_3by
                .write_all(r_3bx)
                .await
                .map_err(|e| format!("gửi khúc dữ liệu lỗi: {e}"))?;
        }
        r_3by
            .write_all(&0u32.to_be_bytes())
            .await
            .map_err(|e| format!("kết thúc stream lỗi: {e}"))?;
        let mut r_3bz = Vec::new();
        r_3by
            .read_to_end(&mut r_3bz)
            .await
            .map_err(|e| format!("đọc phản hồi clamd lỗi: {e}"))?;
        let text = String::from_utf8_lossy(&r_3bz);
        let text = text.trim_end_matches(['\0', '\n', ' ']);
        if text.ends_with("OK") {
            Ok(ScanVerdict::Clean)
        } else if let Some(sig) = text
            .strip_suffix("FOUND")
            .and_then(|r_3c0| r_3c0.rsplit(':').next())
        {
            Ok(ScanVerdict::Infected(sig.trim().to_string()))
        } else {
            Err(format!(
                "phản hồi clamd không hiểu được: {text:?}"
            ))
        }
    }
}
impl Tpu for ClamdScanner {
    fn name(&self) -> &str {
        "clamd"
    }
    fn scan<'a>(&'a self, r_3c1: &'a [u8]) -> Tkn<'a> {
        Box::pin(self.instream(r_3c1))
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum PackageRunOutcome {
    Idle,
    Built(uuid::Uuid),
    Failed(uuid::Uuid),
}
pub struct PackageWorker<'a> {
    pool: &'a PgPool,
    store: &'a dyn digitext_storage::ObjectStore,
    worker_id: &'a str,
    lease_secs: i64,
    software: Tlv,
    scanner: Option<&'a dyn Tpu>,
}
fn f_2qd() -> FixedOffset {
    FixedOffset::east_opt(7 * 3600).expect("+07:00 hợp lệ")
}
impl<'a> PackageWorker<'a> {
    pub fn new(
        pool: &'a PgPool,
        store: &'a dyn digitext_storage::ObjectStore,
        worker_id: &'a str,
        lease_secs: i64,
    ) -> Self {
        PackageWorker {
            pool,
            store,
            worker_id,
            lease_secs,
            software: Tlv {
                name: "DigiText".into(),
                version: env!("CARGO_PKG_VERSION").into(),
            },
            scanner: None,
        }
    }
    pub fn with_scanner(mut self, scanner: &'a dyn Tpu) -> Self {
        self.scanner = Some(scanner);
        self
    }
    pub async fn run_once(&self) -> sqlx::Result<PackageRunOutcome> {
        let Some(r_3c6) = packages::claim_next(self.pool, self.worker_id, self.lease_secs).await?
        else {
            return Ok(PackageRunOutcome::Idle);
        };
        match self.build(&r_3c6).await {
            Ok(r_3c2) => {
                let r_3c4 = packages::f_1r5(
                    self.pool,
                    r_3c6.id,
                    self.worker_id,
                    T1e {
                        objid: &r_3c2.objid,
                        zip_key: &r_3c2.zip_key,
                        zip_name: &r_3c2.zip_name,
                        zip_sha256: &r_3c2.zip_sha256,
                        zip_size: r_3c2.zip_size,
                        document_count: r_3c2.document_count,
                    },
                )
                .await?;
                if !r_3c4 {
                    tracing::warn!(
                        package = % r_3c6.id,
                        "mất lease trước khi chốt built — bỏ"
                    );
                    return Ok(PackageRunOutcome::Failed(r_3c6.id));
                }
                if matches!(
                    r_3c6.kind.parse::<PackageKind>(),
                    Ok(PackageKind::AipHoso | PackageKind::AipTailieu)
                ) {
                    for doc_id in &r_3c2.document_ids {
                        let _ = digitext_db::documents::transition(
                            self.pool,
                            *doc_id,
                            DocStatus::Archived,
                        )
                        .await?;
                    }
                }
                if let Some(r_3c7) = r_3c6.submission_id {
                    match digitext_db::submissions::reconcile(self.pool, r_3c7).await? {
                        Some(r_3c5) => {
                            tracing::info!(
                                submission = % r_3c7, registered = r_3c5.registered,
                                submitted = r_3c5.submitted, enough = r_3c5.enough,
                                "đối chiếu Mục lục lần nộp"
                            )
                        }
                        None => {
                            tracing::debug!(
                                submission = % r_3c7,
                                "lần nộp không ở trạng thái nhận gói — bỏ qua đối chiếu"
                            )
                        }
                    }
                }
                tracing::info!(
                    package = % r_3c6.id, objid = % r_3c2.objid,
                    "gói tin dựng xong"
                );
                Ok(PackageRunOutcome::Built(r_3c6.id))
            }
            Err(BuildStepError::Db(e)) => Err(e),
            Err(BuildStepError::Data(r_3c3)) => {
                tracing::warn!(
                    package = % r_3c6.id, error = % r_3c3,
                    "dựng gói thất bại (lỗi dữ liệu)"
                );
                packages::f_1rd(self.pool, r_3c6.id, self.worker_id, &r_3c3).await?;
                Ok(PackageRunOutcome::Failed(r_3c6.id))
            }
        }
    }
    async fn build(&self, r_3cg: &Tgc) -> Result<T1f, BuildStepError> {
        let r_3cj: Tq4 = serde_json::from_value(r_3cg.request_json.clone()).map_err(|e| {
            BuildStepError::Data(format!(
                "request_json không đọc được: {e}"
            ))
        })?;
        let r_3ci = r_3cj.shape_errors();
        if !r_3ci.is_empty() {
            return Err(BuildStepError::Data(format!(
                "yêu cầu sai hình dạng: {}",
                r_3ci.join("; ")
            )));
        }
        let mut r_3cf: Vec<Option<Vec<u8>>> = Vec::with_capacity(r_3cj.documents.len());
        let mut document_ids = Vec::new();
        for d in &r_3cj.documents {
            match d.document_id {
                None => r_3cf.push(None),
                Some(doc_id) => {
                    let r_3cd = digitext_db::documents::get(self.pool, doc_id)
                        .await
                        .map_err(BuildStepError::Db)?
                        .ok_or_else(|| {
                            BuildStepError::Data(format!(
                                "tài liệu {doc_id} không tồn tại"
                            ))
                        })?;
                    if r_3cd.project_id != r_3cg.project_id {
                        return Err(
                            BuildStepError::Data(
                                format!(
                                    "tài liệu {doc_id} không thuộc dự án của gói"
                                ),
                            ),
                        );
                    }
                    let signed = digitext_storage::f_2fc(r_3cg.project_id, doc_id);
                    let warm = digitext_storage::f_2qs(r_3cg.project_id, doc_id);
                    let r_3c9 = match self.store.get(&signed).await {
                        Ok(Some(b)) => b,
                        Ok(None) => match self.store.get(&warm).await {
                            Ok(Some(b)) => {
                                tracing::warn!(
                                    package = % r_3cg.id, document = % doc_id,
                                    "chưa có PDF đã ký — dùng bản warm chưa ký"
                                );
                                b
                            }
                            Ok(None) => {
                                return Err(
                                        BuildStepError::Data(
                                            format!(
                                                "thiếu PDF trong kho cho tài liệu {doc_id} (đã thử {signed} và {warm})"
                                            ),
                                        ),
                                    );
                            }
                            Err(e) => {
                                return Err(BuildStepError::Data(format!(
                                    "đọc kho object lỗi ({warm}): {e}"
                                )));
                            }
                        },
                        Err(e) => {
                            return Err(BuildStepError::Data(format!(
                                "đọc kho object lỗi ({signed}): {e}"
                            )));
                        }
                    };
                    let r_3cb = digitext_pdf::f_2p9(&r_3c9, r_3cd.page_count);
                    if !r_3cb.ok {
                        return Err(
                            BuildStepError::Data(
                                format!(
                                    "tài liệu {doc_id} không đạt kiểm PDF lưu trữ: {}",
                                    r_3cb.reason.unwrap_or_else(|| "không đạt"
                                    .into())
                                ),
                            ),
                        );
                    }
                    r_3cf.push(Some(r_3c9));
                    document_ids.push(doc_id);
                }
            }
        }
        let r_3ca = if r_3cj.dip_cert.is_some() {
            let r_3ce = digitext_pdf::f_1cq()
                .ok_or_else(|| {
                    BuildStepError::Data(
                        "không tìm thấy font TTF cho văn bản xác thực DIP — đặt DIGITEXT_CERT_FONT"
                            .to_string(),
                    )
                })?;
            Some(std::fs::read(&r_3ce).map_err(|e| {
                BuildStepError::Data(format!(
                    "đọc font {} lỗi: {e}",
                    r_3ce.display()
                ))
            })?)
        } else {
            None
        };
        // Ngày tạo gói = đồng hồ máy chủ đang đóng gói (múi giờ máy).
        let r_3cc = chrono::Local::now().fixed_offset();
        let mut r_3c8 = r_3cj
            .to_build_request(r_3cf, self.software.clone(), r_3cc, r_3ca)
            .map_err(BuildStepError::Data)?;
        // documentation/UserManual.pdf: đọc từ DIGITEXT_PACKAGE_MANUAL nếu máy chủ có đặt.
        r_3c8.user_manual_pdf = std::env::var_os("DIGITEXT_PACKAGE_MANUAL")
            .and_then(|p| std::fs::read(p).ok())
            .filter(|b| b.starts_with(b"%PDF-"));
        let built = f_vn(&r_3c8, &mut Tic).map_err(|e| BuildStepError::Data(e.to_string()))?;
        let r_3ch = match self.scanner {
            None => "not_configured".to_string(),
            Some(scanner) => match scanner.scan(&built.zip_bytes).await {
                Ok(ScanVerdict::Clean) => format!("clean ({})", scanner.name()),
                Ok(ScanVerdict::Infected(sig)) => {
                    let note = format!("infected: {sig}");
                    let _ = packages::f_2ek(self.pool, r_3cg.id, &note).await;
                    return Err(
                            BuildStepError::Data(
                                format!(
                                    "quét virus phát hiện mã độc: {sig} — gói không được đưa vào kho"
                                ),
                            ),
                        );
                }
                Err(e) => {
                    let note = format!("scan_error: {e}");
                    let _ = packages::f_2ek(self.pool, r_3cg.id, &note).await;
                    return Err(BuildStepError::Data(format!(
                        "quét virus lỗi: {e}"
                    )));
                }
            },
        };
        packages::f_2ek(self.pool, r_3cg.id, &r_3ch)
            .await
            .map_err(BuildStepError::Db)?;
        let zip_key = digitext_storage::f_1x5(r_3cg.project_id, r_3cg.id);
        self.store
            .put(&zip_key, &built.zip_bytes)
            .await
            .map_err(|e| {
                BuildStepError::Data(format!("PUT ZIP vào kho lỗi ({zip_key}): {e}"))
            })?;
        Ok(T1f {
            objid: built.objid,
            zip_key,
            zip_name: built.zip_name,
            zip_sha256: built.zip_sha256,
            zip_size: built.zip_bytes.len() as i64,
            document_count: r_3cj.documents.len() as i32,
            document_ids,
        })
    }
    pub async fn run_loop(
        &self,
        r_3ck: Duration,
        r_3cl: i32,
        r_3cp: &CancellationToken,
    ) -> sqlx::Result<u64> {
        let mut r_3co = 0u64;
        while !r_3cp.is_cancelled() {
            let r_3cn = tokio::select! {
                biased; _ = r_3cp.cancelled() => break, r = self.run_once() => r ?,
            };
            if r_3cn == PackageRunOutcome::Idle {
                let r_3cm = packages::requeue_expired(self.pool, r_3cl).await?;
                if r_3cm > 0 {
                    tracing::warn!(
                        requeued = r_3cm,
                        "requeue yêu cầu đóng gói hết hạn lease"
                    );
                }
                tokio::select! {
                    _ = r_3cp.cancelled() => break, _ = tokio::time::sleep(r_3ck) => {}
                }
            } else {
                r_3co += 1;
            }
        }
        tracing::info!(
            worker_id = self.worker_id,
            r_3co,
            "vòng lặp đóng gói dừng"
        );
        Ok(r_3co)
    }
}
struct T1f {
    objid: String,
    zip_key: String,
    zip_name: String,
    zip_sha256: String,
    zip_size: i64,
    document_count: i32,
    document_ids: Vec<uuid::Uuid>,
}
enum BuildStepError {
    Db(sqlx::Error),
    Data(String),
}
pub async fn f_2bd(pool: &PgPool, r_3cq: Duration, r_3cr: &CancellationToken) -> sqlx::Result<()> {
    use chrono::Datelike;
    while !r_3cr.is_cancelled() {
        tokio::select! {
            _ = r_3cr.cancelled() => break, _ = tokio::time::sleep(r_3cq) => { let year =
            chrono::Utc::now().with_timezone(& f_2qd()).year(); let holidays =
            digitext_db::workdays::f_1hg(& std::env::var("DIGITEXT_HOLIDAYS")
            .unwrap_or_default(), year - 1..= year,); let n =
            digitext_db::notifications::f_2fx(pool, & holidays). await ?; if n > 0 {
            tracing::info!(created = n, "nhắc SLA thu nộp (WF-3)"); } }
        }
    }
    Ok(())
}
pub async fn f_2am(
    pool: &PgPool,
    store: &dyn digitext_storage::ObjectStore,
    r_3cs: Duration,
    r_3ct: &CancellationToken,
) -> sqlx::Result<()> {
    while !r_3ct.is_cancelled() {
        tokio::select! {
            _ = r_3ct.cancelled() => break, _ = tokio::time::sleep(r_3cs) => { let copies
            = digitext_db::access::f_1ag(pool). await ?; let mut deleted = Vec::new();
            for (item_id, key) in & copies { match store.delete(key). await { Ok(()) =>
            deleted.push(* item_id), Err(e) => { tracing::warn!(key = % key, error = % e,
            "xóa bản người dùng hết hạn lỗi — thử lại lượt sau");
            } } } if ! deleted.is_empty() { digitext_db::access::f_1r9(pool, & deleted).
            await ?; } let expired = digitext_db::access::f_1rc(pool). await ?; if !
            deleted.is_empty() || expired > 0 { tracing::info!(copies = deleted.len(),
            requests = expired,
            "khai thác: đã xóa bản người dùng hết hạn (Điều 37.5/40.6)");
            } }
        }
    }
    Ok(())
}
pub async fn f_2an(pool: &PgPool, r_3cu: Duration, r_3cv: &CancellationToken) -> sqlx::Result<()> {
    while !r_3cv.is_cancelled() {
        tokio::select! {
            _ = r_3cv.cancelled() => break, _ = tokio::time::sleep(r_3cu) => { let dup =
            digitext_db::disposal::f_18b(pool). await ?; let exp =
            digitext_db::disposal::f_1ah(pool). await ?; if dup.is_empty() && exp
            .is_empty() { continue; } tracing::info!(trung_lap = dup.len(), het_han = exp
            .len(),
            "danh mục hủy: phát hiện mới (Điều 48.2/48.3)");
            let mut per_project : std::collections::BTreeMap < uuid::Uuid, (usize, usize)
            > = std::collections::BTreeMap::new(); for (p, _) in & dup { per_project
            .entry(* p).or_default().0 += 1; } for (p, _) in & exp { per_project.entry(*
            p).or_default().1 += 1; } for (project, (n_dup, n_exp)) in per_project { let
            admins = digitext_db::projects::f_re(pool, project). await ?; let msg =
            format!("Danh mục hủy có phát hiện mới: {n_dup} trùng lặp (Điều 48.2), {n_exp} hết thời hạn lưu trữ (Điều 48.3) — cần xem xét");
            for admin in admins { let _ = digitext_db::notifications::f_1v8(pool, admin,
            None, "disposal_review", & msg,). await ?; } } }
        }
    }
    Ok(())
}
async fn check_document(
    store: &dyn digitext_storage::ObjectStore,
    r_3cz: uuid::Uuid,
    doc_id: uuid::Uuid,
    r_3cy: Option<i32>,
) -> Result<(), String> {
    let signed = digitext_storage::f_2fc(r_3cz, doc_id);
    let warm = digitext_storage::f_2qs(r_3cz, doc_id);
    let (key, r_3cw) = match store.get(&signed).await {
        Ok(Some(b)) => (signed, b),
        Ok(None) => match store.get(&warm).await {
            Ok(Some(b)) => (warm.clone(), b),
            Ok(None) => {
                return Err(format!(
                    "thiếu object trong kho ({signed} và {warm})"
                ));
            }
            Err(e) => return Err(format!("kho object lỗi: {e}")),
        },
        Err(e) => return Err(format!("kho object lỗi: {e}")),
    };
    let r_3cx = digitext_pdf::f_2p9(&r_3cw, r_3cy);
    if !r_3cx.ok {
        return Err(format!(
            "{key}: {}",
            r_3cx
                .reason
                .unwrap_or_else(|| "không đạt".into())
        ));
    }
    Ok(())
}
pub async fn f_2b5(
    pool: &PgPool,
    store: &dyn digitext_storage::ObjectStore,
    r_3d1: Duration,
    r_3d0: i64,
    r_3d2: &CancellationToken,
) -> sqlx::Result<()> {
    while !r_3d2.is_cancelled() {
        tokio::select! {
            _ = r_3d2.cancelled() => break, _ = tokio::time::sleep(r_3d1) => { let due =
            digitext_db::ops::f_17z(pool, r_3d0). await ?; let mut per_project :
            std::collections::BTreeMap < uuid::Uuid, usize > =
            std::collections::BTreeMap::new(); for (doc_id, project_id, _filename,
            page_count) in due { match check_document(store, project_id, doc_id,
            page_count). await { Ok(()) => { digitext_db::ops::f_26a(pool, project_id,
            doc_id, "dat", None). await ?; } Err(detail) => {
            digitext_db::ops::f_26a(pool, project_id, doc_id, "loi", Some(& detail),).
            await ?; * per_project.entry(project_id).or_default() += 1; } } } for
            (project, n_loi) in per_project { let admins =
            digitext_db::projects::f_re(pool, project). await ?; let msg =
            format!("Kiểm tra tài liệu (Điều 29): phát hiện {n_loi} tệp LỖI — xem danh mục tệp lỗi và lập biên bản kiểm tra (Mục 1 Phụ lục VII)");
            for admin in admins { let _ = digitext_db::notifications::f_1v8(pool, admin,
            None, "integrity_failed", & msg,). await ?; } } let last_ok =
            digitext_db::ops::f_1me(pool). await ?; let overdue = match last_ok { Some(t)
            => chrono::Utc::now() - t > chrono::Duration::hours(26), None => true, }; if
            overdue && ! digitext_db::ops::f_ti(pool, 20). await ? { let msg = match
            last_ok { Some(t) =>
            format!("SAO LƯU TRỄ: lần sao lưu đạt gần nhất lúc {} — Điều 30.1.a yêu cầu sao lưu gia tăng HẰNG NGÀY",
            t.with_timezone(& f_2qd()).format("%d/%m/%Y %H:%M")), None =>
            "CHƯA CÓ SAO LƯU nào được ghi nhận — Điều 30.1.a yêu cầu sao lưu gia tăng hằng ngày (script vận hành báo qua POST /ops/backups)"
            .to_string(), }; for admin in digitext_db::ops::f_rk(pool). await ? { let _ =
            digitext_db::notifications::f_1v8(pool, admin, None, "backup_overdue", &
            msg,). await ?; } } }
        }
    }
    Ok(())
}
