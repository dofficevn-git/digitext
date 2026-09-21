// Copyright (c) 2026 DigiText. All rights reserved.
//! Nhập kho tài liệu TỪ BÊN NGOÀI rồi đóng gói TT05 — mặt tiền Tauri của `digitext-extimport`.
//!
//! Hai lệnh, đều chạy ở luồng nền và phát tiến độ qua sự kiện `extimport://progress`:
//!   `extimport_check` — đối chiếu thư mục ↔ Excel (tuỳ chọn soi từng PDF theo Điều 8 TT05);
//!   `extimport_pack`  — kiểm lại, CHỈ KHI không còn lỗi mới đóng gói (mỗi hồ sơ một gói).
//! Không gọi máy chủ: dữ liệu đọc thẳng từ đĩa, gói ghi ra thư mục người dùng chọn.
use crate::auth::AuthState;
use crate::error::AppError;
use digitext_core::Permission;
use digitext_extimport::{check, excel, pack, report};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, State};

const EV_PROGRESS: &str = "extimport://progress";
/// Một lần chỉ cho chạy một việc (kiểm hoặc đóng gói) — cả hai đều đọc đĩa rất nặng.
static BUSY: AtomicBool = AtomicBool::new(false);

struct BusyGuard;
impl BusyGuard {
    fn take() -> Result<Self, AppError> {
        if BUSY.swap(true, Ordering::SeqCst) {
            return Err(AppError::validation("Đang có một lượt kiểm tra/đóng gói kho ngoài chạy dở — chờ xong rồi thử lại."));
        }
        Ok(BusyGuard)
    }
}
impl Drop for BusyGuard {
    fn drop(&mut self) {
        BUSY.store(false, Ordering::SeqCst);
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Progress<'a> {
    phase: &'a str,
    done: usize,
    total: usize,
    note: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckArgs {
    pub root: String,
    pub excel: String,
    #[serde(default)]
    pub deep: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeSummary {
    pub severity: check::Severity,
    pub code: String,
    pub count: u32,
    pub dossiers: usize,
    pub example: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CheckSummary {
    pub root: String,
    pub sheet: String,
    pub deep: bool,
    pub rows: usize,
    pub files_on_disk: usize,
    pub dossiers: usize,
    pub dossiers_ok: usize,
    pub bytes: u64,
    pub errors: u32,
    pub warnings: u32,
    pub pdfa: Option<u32>,
    pub signed: Option<u32>,
    pub pages: Option<u32>,
    pub by_code: Vec<CodeSummary>,
    /// Tối đa 300 dòng đầu (lỗi trước) — bản đầy đủ nằm trong tệp CSV.
    pub issues: Vec<check::Issue>,
    pub issues_total: usize,
    pub column_mapping: Vec<(String, String)>,
    pub report_dir: String,
    pub report_files: Vec<String>,
}

fn summarize(rep: &check::CheckReport, report_dir: &std::path::Path, files: Vec<PathBuf>) -> CheckSummary {
    let mut by: BTreeMap<(check::Severity, &str), (u32, std::collections::BTreeSet<&str>, &str)> = BTreeMap::new();
    for i in &rep.issues {
        let e = by.entry((i.severity, i.code)).or_insert((0, Default::default(), i.message.as_str()));
        e.0 += i.count;
        e.1.insert(i.dossier.as_str());
    }
    let sum = |f: fn(&check::DossierSummary) -> Option<u32>| -> Option<u32> {
        rep.deep.then(|| rep.dossiers.iter().filter_map(f).sum())
    };
    CheckSummary {
        root: rep.root.clone(),
        sheet: rep.sheet.clone(),
        deep: rep.deep,
        rows: rep.rows,
        files_on_disk: rep.files_on_disk,
        dossiers: rep.dossiers.len(),
        dossiers_ok: rep.dossiers_ok(),
        bytes: rep.dossiers.iter().map(|d| d.bytes).sum(),
        errors: rep.errors(),
        warnings: rep.warnings(),
        pdfa: sum(|d| d.pdfa),
        signed: sum(|d| d.signed),
        pages: sum(|d| d.pages),
        by_code: by
            .into_iter()
            .map(|((severity, code), (count, ds, ex))| CodeSummary {
                severity, code: code.to_string(), count, dossiers: ds.len(), example: ex.to_string(),
            })
            .collect(),
        issues: rep.issues.iter().take(300).cloned().collect(),
        issues_total: rep.issues.len(),
        column_mapping: rep.column_mapping.clone(),
        report_dir: report_dir.display().to_string(),
        report_files: files.iter().map(|p| p.display().to_string()).collect(),
    }
}

fn load(excel_path: &str, root: &str) -> Result<(excel::Sheet, check::Plan), AppError> {
    let sheet = excel::read(std::path::Path::new(excel_path)).map_err(AppError::validation)?;
    if !sheet.missing_columns.is_empty() {
        return Err(AppError::validation(format!(
            "Tệp Excel thiếu cột bắt buộc: {}",
            sheet.missing_columns.join(", ")
        )));
    }
    if sheet.rows.is_empty() {
        return Err(AppError::validation("Tệp Excel không có dòng tài liệu nào."));
    }
    let plan = check::plan(&sheet, std::path::Path::new(root));
    Ok((sheet, plan))
}

fn report_dir_for(excel_path: &str) -> PathBuf {
    std::path::Path::new(excel_path)
        .parent()
        .map(|p| p.join("_digitext_kiem_tra"))
        .unwrap_or_else(|| PathBuf::from("_digitext_kiem_tra"))
}

fn run_check(app: &AppHandle, a: &CheckArgs, phase: &'static str) -> Result<(check::Plan, check::CheckReport), AppError> {
    let _ = app.emit(EV_PROGRESS, Progress { phase: "excel", done: 0, total: 0, note: None });
    let (sheet, plan) = load(&a.excel, &a.root)?;
    let app2 = app.clone();
    let rep = check::run(&sheet, std::path::Path::new(&a.excel), &plan, a.deep, true, 4, &move |done, total| {
        if done == total || done % 5 == 0 {
            let _ = app2.emit(EV_PROGRESS, Progress { phase, done, total, note: None });
        }
    });
    Ok((plan, rep))
}

#[tauri::command]
pub async fn extimport_check(app: AppHandle, auth: State<'_, AuthState>, args: CheckArgs) -> Result<CheckSummary, AppError> {
    auth.require_perm(Permission::PackageCreate)?;
    let guard = BusyGuard::take()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = guard;
        let (_plan, rep) = run_check(&app, &args, "check")?;
        let dir = report_dir_for(&args.excel);
        let files = report::write_check(&rep, &dir).map_err(AppError::internal)?;
        tracing::info!(root = % rep.root, dossiers = rep.dossiers.len(), errors = rep.errors(), deep = rep.deep, "extimport_check");
        Ok(summarize(&rep, &dir, files))
    })
    .await
    .map_err(|e| AppError::internal(format!("luồng kiểm tra dừng bất thường: {e}")))?
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackArgs {
    pub root: String,
    pub excel: String,
    pub out_dir: String,
    /// `aip_hoso` | `sip_hoso`
    pub kind: String,
    #[serde(default)]
    pub agreement: Option<String>,
    #[serde(default)]
    pub org_code: Option<String>,
    /// Kho còn hồ sơ lỗi vẫn đóng các hồ sơ sạch. Mặc định false: khớp hết mới đóng.
    #[serde(default)]
    pub allow_partial: bool,
    #[serde(default)]
    pub force: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackSummary {
    pub check: CheckSummary,
    /// false = bị chặn ở bước kiểm, KHÔNG có gói nào được dựng.
    pub started: bool,
    pub built: usize,
    pub existing: usize,
    pub held: usize,
    pub failed: usize,
    pub bytes: u64,
    pub failures: Vec<pack::PackResult>,
    pub out_dir: String,
    pub result_file: Option<String>,
}

#[tauri::command]
pub async fn extimport_pack(app: AppHandle, auth: State<'_, AuthState>, args: PackArgs) -> Result<PackSummary, AppError> {
    auth.require_perm(Permission::PackageCreate)?;
    if args.kind != "aip_hoso" && args.kind != "sip_hoso" {
        return Err(AppError::validation("Loại gói chỉ nhận aip_hoso hoặc sip_hoso."));
    }
    let agreement = args.agreement.clone().filter(|s| !s.trim().is_empty());
    if args.kind == "sip_hoso" && agreement.is_none() {
        return Err(AppError::validation("Gói SIP bắt buộc mã thoả thuận nộp lưu."));
    }
    if args.out_dir.trim().is_empty() {
        return Err(AppError::validation("Chưa chọn thư mục ghi gói tin."));
    }
    let guard = BusyGuard::take()?;
    tauri::async_runtime::spawn_blocking(move || {
        let _guard = guard;
        let out_dir = PathBuf::from(&args.out_dir);
        // Bước kiểm NHANH trên toàn kho (mã phông, mã hồ sơ, đường dẫn, tên tệp, thiếu/thừa tệp…).
        // Từng PDF được soi lại ngay trước khi dựng gói (số trang khớp Excel + Điều 8.2.a TT05).
        let (plan, rep) = run_check(&app, &CheckArgs { root: args.root.clone(), excel: args.excel.clone(), deep: false }, "precheck")?;
        let files = report::write_check(&rep, &out_dir).map_err(AppError::internal)?;
        let check_sum = summarize(&rep, &out_dir, files);
        let mut out = PackSummary {
            check: check_sum, started: false, built: 0, existing: 0, held: 0, failed: 0, bytes: 0,
            failures: Vec::new(), out_dir: out_dir.display().to_string(), result_file: None,
        };
        if rep.errors() > 0 && !args.allow_partial {
            return Ok(out);
        }
        out.started = true;
        let opts = pack::PackOptions {
            kind: args.kind.clone(), org_code: args.org_code.clone().filter(|s| !s.trim().is_empty()), agreement,
            out_dir: out_dir.clone(), force: args.force, allow_partial: args.allow_partial, allow_non_tt05: false,
            threads: 2, only: Vec::new(), limit: None,
        };
        let app2 = app.clone();
        let results = pack::run(&plan, &rep.packable(), &opts, &move |done, total, r| {
            let note = (r.status == "failed").then(|| format!("{}: {}", r.hs_code, r.error.clone().unwrap_or_default()));
            let _ = app2.emit(EV_PROGRESS, Progress { phase: "pack", done, total, note });
        })
        .map_err(AppError::internal)?;
        for r in &results {
            match r.status.as_str() {
                "built" => {
                    out.built += 1;
                    out.bytes += r.zip_size.unwrap_or(0);
                }
                "skipped_existing" => out.existing += 1,
                "skipped_errors" => out.held += 1,
                _ => out.failed += 1,
            }
        }
        out.failures = results.iter().filter(|r| r.status == "failed").take(200).cloned().collect();
        out.result_file = report::write_pack(&results, &out_dir).ok().map(|p| p.display().to_string());
        tracing::info!(built = out.built, failed = out.failed, held = out.held, out = % out.out_dir, "extimport_pack");
        Ok(out)
    })
    .await
    .map_err(|e| AppError::internal(format!("luồng đóng gói dừng bất thường: {e}")))?
}
