// Copyright (c) 2026 DigiText. All rights reserved.
//! BƯỚC ĐÓNG GÓI: mỗi hồ sơ (thư mục) đã qua bước kiểm → một gói TT05 (SIP/AIP hồ sơ).
//!
//! Dùng lại nguyên bộ dựng gói của DigiText: dòng Excel → hồ sơ cấu trúc TT02
//! (`digitext_package::tt02`) → bảng mã TT05 → `f_vn` dựng METS/EAD/PREMIS/ZIP → `f_2p7`
//! tự kiểm gói. Không cần Postgres/MinIO: byte tệp đọc thẳng từ đĩa.
//!
//! Tệp trong gói được ĐỔI TÊN thành `<mã hồ sơ>.<STT 7 số>.pdf` (quy tắc mã tài liệu của
//! TT05); tên tệp gốc ghi vào phần mô tả của tài liệu. Nội dung tệp giữ nguyên từng byte —
//! chữ ký số trong PDF không bị ảnh hưởng.
use crate::check::{inspect_pdf, norm_date, parse_doc_name, rel_to_path, seq_of, Dossier, Plan};
use digitext_package::tt02::{f_11h, Tos, Tot, Tou};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub struct PackOptions {
    /// `sip_hoso` | `aip_hoso`
    pub kind: String,
    /// Mã cơ quan lưu trữ; `None` = đoạn đầu của mã phông (H49.132.02 → H49).
    pub org_code: Option<String>,
    /// Mã thoả thuận nộp lưu — bắt buộc với SIP.
    pub agreement: Option<String>,
    /// Tên cơ quan lưu trữ (ghi vào agent ARCHIVIST và PREMIS); None = dùng mã.
    pub org_name: Option<String>,
    /// Tên cơ quan hình thành phông (agent CREATOR); None = dùng mã phông.
    pub fond_name: Option<String>,
    /// Đơn vị đóng gói (agent CREATOR thứ hai ở METS gốc), ví dụ công ty số hoá.
    pub packager: Option<digitext_package::OrgIdent>,
    pub out_dir: PathBuf,
    /// Dựng lại cả hồ sơ đã có gói.
    pub force: bool,
    /// Vẫn đóng gói các hồ sơ sạch khi kho còn hồ sơ lỗi. Mặc định `false`: còn LỖI ở bất
    /// kỳ đâu thì KHÔNG bắt đầu đóng gói (khớp hết rồi mới đóng).
    pub allow_partial: bool,
    /// CHỈ ĐỂ THỬ: vẫn đóng gói khi tệp chưa đạt Điều 8.2.a TT05 (không PDF/A, thiếu lớp
    /// chữ, dpi/màu thấp, chưa ký). Mặc định `false` — gói TT05 phải chuẩn.
    pub allow_non_tt05: bool,
    /// Trước khi dựng, xác minh ISO 19005 từng tệp bằng veraPDF theo đúng loại tự khai (cần Docker).
    pub verapdf: bool,
    pub threads: usize,
    /// Chỉ đóng các hồ sơ này (theo thư mục HOẶC mã hồ sơ); rỗng = tất cả.
    pub only: Vec<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackResult {
    pub dir: String,
    pub hs_code: String,
    /// built | skipped_existing | skipped_errors | failed
    pub status: String,
    pub zip_name: Option<String>,
    pub zip_sha256: Option<String>,
    pub zip_size: Option<u64>,
    pub objid: Option<String>,
    pub docs: usize,
    pub not_pdfa: u32,
    pub unsigned: u32,
    pub notes: Vec<String>,
    pub error: Option<String>,
}

const STATE_FILE: &str = "trang-thai-dong-goi.json";

pub fn load_state(out_dir: &Path) -> BTreeMap<String, PackResult> {
    std::fs::read(out_dir.join(STATE_FILE))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}
fn save_state(out_dir: &Path, st: &BTreeMap<String, PackResult>) {
    if let Ok(b) = serde_json::to_vec_pretty(st) {
        let tmp = out_dir.join(format!("{STATE_FILE}.tmp"));
        if std::fs::write(&tmp, b).is_ok() {
            let _ = std::fs::rename(tmp, out_dir.join(STATE_FILE));
        }
    }
}

fn num(s: &Option<String>) -> Option<u32> {
    s.as_deref().and_then(crate::check::lenient_u32).map(|x| x.0)
}
fn date(s: &Option<String>) -> Option<String> {
    s.as_deref().map(|x| norm_date(x).0)
}

/// Hồ sơ Excel → hồ sơ cấu trúc TT02 mà bộ chuyển `f_11h` hiểu. Trả kèm tên tệp gốc theo STT.
pub fn to_tt02(d: &Dossier) -> (Tos, Vec<String>) {
    let f = d.first();
    let names: Vec<String> = d
        .rows
        .iter()
        .map(|r| r.doc_path.as_deref().unwrap_or("").rsplit(['\\', '/']).next().unwrap_or("").to_string())
        .collect();
    // STT lấy từ đuôi tên tệp nếu đủ và không trùng; không thì đánh theo thứ tự dòng.
    let seqs: Vec<Option<u32>> = names.iter().map(|n| seq_of(n)).collect();
    let uniq: HashSet<u32> = seqs.iter().flatten().copied().collect();
    let use_seq = seqs.iter().all(Option::is_some) && uniq.len() == seqs.len();
    let lang = |s: &Option<String>| s.clone().map(|x| vec![x]).unwrap_or_default();
    let van_ban = d
        .rows
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let stt = if use_seq { seqs[i].unwrap() } else { i as u32 + 1 };
            let origin = format!("Tệp gốc: {}", names[i]);
            Tou {
                stt,
                muc_luc: if use_seq { parse_doc_name(&names[i]).muc_luc } else { None },
                ten_loai: r.doc_type.clone(),
                so_van_ban: r.doc_number.clone(),
                ky_hieu: r.doc_notation.clone(),
                ngay_thang: date(&r.doc_date),
                co_quan_ban_hanh: r.doc_organ.clone(),
                trich_yeu: r.doc_subject.clone(),
                ngon_ngu: lang(&r.doc_lang),
                so_trang: num(&r.doc_pages),
                ghi_chu: Some(match &r.doc_note {
                    Some(n) => format!("{n}. {origin}"),
                    None => origin,
                }),
                ky_hieu_thong_tin: r.doc_infor_sign.clone(),
                tu_khoa: r.doc_keyword.clone(),
                che_do_su_dung: r.doc_mode.clone(),
                but_tich: r.doc_autograph.clone(),
                tinh_trang_vat_ly: r.doc_physical.clone(),
                // Bộ chuyển đòi documentId (đường qua máy chủ). Đường ngoại tuyến cấp byte
                // trực tiếp nên chỉ cần một định danh giữ chỗ.
                document_id: Some(uuid::Uuid::new_v4()),
                tep_tin: Some(names[i].clone()),
            }
        })
        .collect::<Vec<_>>();
    let sum_pages: u32 = d.rows.iter().filter_map(|r| num(&r.doc_pages)).sum();
    let hs = Tos {
        ma_ho_so: f.hs_code.clone().unwrap_or_default(),
        tieu_de: f.hs_title.clone().unwrap_or_default(),
        thoi_han_bao_quan: f.hs_thbq.clone().unwrap_or_default(),
        che_do_su_dung: f.hs_mode.clone(),
        ngon_ngu: if f.hs_lang.is_some() { lang(&f.hs_lang) } else { vec!["Tiếng Việt".into()] },
        thoi_gian_bat_dau: date(&f.hs_start),
        thoi_gian_ket_thuc: date(&f.hs_end),
        tong_so_van_ban: Some(d.rows.len() as u32),
        chu_giai: f.hs_note.clone(),
        ky_hieu_thong_tin: f.hs_infor_sign.clone(),
        tu_khoa: f.hs_keyword.clone(),
        so_luong_to: num(&f.hs_papers),
        so_luong_trang: num(&f.hs_pages).or(Some(sum_pages)),
        tinh_trang_vat_ly: f.hs_physical.clone(),
        ma_ho_so_giay: f.hs_paper_code.clone(),
        van_ban,
    };
    (hs, names)
}

fn build_one(d: &Dossier, root: &Path, o: &PackOptions) -> PackResult {
    let mut res = PackResult {
        dir: d.dir.clone(), hs_code: d.hs_code(), status: "failed".into(), zip_name: None, zip_sha256: None,
        zip_size: None, objid: None, docs: d.rows.len(), not_pdfa: 0, unsigned: 0, notes: Vec::new(), error: None,
    };
    let fond = d.first().fond_code.clone().unwrap_or_default();
    let org = o.org_code.clone().unwrap_or_else(|| fond.split('.').next().unwrap_or("").to_string());
    let (hs, _names) = to_tt02(d);
    let conv = f_11h(&hs, &Tot {
        fond_code: fond, archive_org_code: org, kind: o.kind.clone(), submission_agreement: o.agreement.clone(),
        fond_name: o.fond_name.clone(), archive_org_name: o.org_name.clone(),
    });
    let mut notes: Vec<String> = conv.report.notes;
    notes.sort();
    notes.dedup();
    res.notes = notes;
    if !conv.report.errors.is_empty() {
        res.error = Some(conv.report.errors.join("; "));
        return res;
    }
    // ---- đọc byte theo đúng thứ tự văn bản
    let mut bytes = Vec::with_capacity(d.rows.len());
    let mut to_verify: Vec<(std::path::PathBuf, Option<String>)> = Vec::new();
    for r in &d.rows {
        let rel = r.doc_path.as_deref().unwrap_or("");
        match std::fs::read(rel_to_path(root, rel)) {
            Ok(b) => {
                let f = inspect_pdf(&b);
                if o.verapdf && !o.allow_non_tt05 {
                    to_verify.push((rel_to_path(root, rel), f.declared.clone()));
                }
                if let Some(why) = f.encrypted_or_broken {
                    res.error = Some(format!("{rel}: {why}"));
                    return res;
                }
                if let Some(p) = num(&r.doc_pages) {
                    if p != f.pages {
                        res.error = Some(format!("{rel}: Excel ghi {p} trang, PDF thật có {} trang — không khớp, không đóng gói", f.pages));
                        return res;
                    }
                }
                res.not_pdfa += (!f.pdfa) as u32;
                res.unsigned += (!f.signed) as u32;
                if !o.allow_non_tt05 {
                    let is_map = crate::check::classify_map(r.doc_type.as_deref(), r.doc_subject.as_deref(), f.max_page_cm).is_map();
                    if let Some((code, msg)) = f.tt05_failures(is_map).into_iter().next() {
                        res.error = Some(format!("{rel}: [{code}] {msg} — chưa đạt TT05, không đóng gói"));
                        return res;
                    }
                }
                bytes.push(Some(b));
            }
            Err(e) => {
                res.error = Some(format!("không đọc được {rel}: {e}"));
                return res;
            }
        }
    }
    if !to_verify.is_empty() {
        let vera = crate::pdfa::verify_files(&to_verify);
        for (p, declared) in &to_verify {
            match vera.get(p) {
                Some(v) if v.compliant => {}
                Some(v) => {
                    res.error = Some(format!("{}: veraPDF KHÔNG ĐẠT {} — {} — không đóng gói", p.display(), declared.clone().unwrap_or_default(), v.rules.first().cloned().unwrap_or_default()));
                    return res;
                }
                None => {
                    res.error = Some(format!("{}: veraPDF không chạy được (Docker chưa bật?) — bỏ tuỳ chọn veraPDF hoặc bật Docker", p.display()));
                    return res;
                }
            }
        }
    }
    let software = digitext_package::Tlv { name: "DigiText (nhập kho ngoài)".into(), version: env!("CARGO_PKG_VERSION").into() };
    // Đồng hồ và múi giờ của máy đang đóng gói: chỉnh ngày máy là chỉnh được ngày tạo gói.
    let now = chrono::Local::now().fixed_offset();
    let mut wire = conv.request.clone();
    if let Some(pk) = &o.packager {
        wire.packager_name = Some(pk.name.clone());
        wire.packager_code = Some(pk.code.clone());
    }
    wire.allow_non_pdfa = o.allow_non_tt05;
    let req = match wire.to_build_request(bytes, software, now, None) {
        Ok(mut r) => {
            r.user_manual_pdf = crate::user_manual_pdf();
            r
        }
        Err(e) => {
            res.error = Some(e);
            return res;
        }
    };
    let built = match digitext_package::f_vn(&req, &mut digitext_package::ids::Tic) {
        Ok(b) => b,
        Err(e) => {
            res.error = Some(e.to_string());
            return res;
        }
    };
    drop(req);
    let bad: Vec<String> = digitext_package::f_2p7(built.kind, &built.files)
        .into_iter()
        .filter(|v| matches!(v.severity, digitext_package::Severity::Error))
        .map(|v| format!("[{}] {}", v.code, v.message))
        .collect();
    if !bad.is_empty() {
        res.error = Some(format!("gói dựng xong nhưng tự kiểm không đạt: {}", bad.join("; ")));
        return res;
    }
    let target = o.out_dir.join(&built.zip_name);
    let tmp = o.out_dir.join(format!("{}.part", built.zip_name));
    if let Err(e) = std::fs::write(&tmp, &built.zip_bytes).and_then(|_| std::fs::rename(&tmp, &target)) {
        let _ = std::fs::remove_file(&tmp);
        res.error = Some(format!("không ghi được {}: {e}", target.display()));
        return res;
    }
    res.status = "built".into();
    res.zip_size = Some(built.zip_bytes.len() as u64);
    res.zip_sha256 = Some(built.zip_sha256.clone());
    res.objid = Some(built.objid.clone());
    res.zip_name = Some(built.zip_name);
    res
}

/// Đóng gói các hồ sơ trong `packable` (thư mục hồ sơ KHÔNG có lỗi ở bước kiểm).
/// Chạy lại được: hồ sơ đã có gói (và tệp zip còn đó, đúng cỡ) thì bỏ qua trừ khi `force`.
pub fn run(
    plan: &Plan,
    packable: &HashSet<String>,
    o: &PackOptions,
    progress: &(dyn Fn(usize, usize, &PackResult) + Sync),
) -> Result<Vec<PackResult>, String> {
    std::fs::create_dir_all(&o.out_dir).map_err(|e| format!("không tạo được {}: {e}", o.out_dir.display()))?;
    let state = Mutex::new(load_state(&o.out_dir));
    let only: HashSet<String> = o.only.iter().map(|s| s.trim().to_lowercase()).collect();
    let mut todo: Vec<&Dossier> = plan
        .dossiers
        .iter()
        .filter(|d| only.is_empty() || only.contains(&d.dir.to_lowercase()) || only.contains(&d.hs_code().to_lowercase()))
        .collect();
    if let Some(n) = o.limit {
        todo.truncate(n);
    }
    let total = todo.len();
    let next = AtomicUsize::new(0);
    let done = AtomicUsize::new(0);
    let out: Mutex<Vec<(usize, PackResult)>> = Mutex::new(Vec::with_capacity(total));
    std::thread::scope(|s| {
        for _ in 0..o.threads.max(1) {
            s.spawn(|| loop {
                let i = next.fetch_add(1, Ordering::Relaxed);
                if i >= total {
                    break;
                }
                let d = todo[i];
                let prev = state.lock().unwrap().get(&d.dir).cloned();
                let res = if !packable.contains(&d.dir) {
                    PackResult {
                        dir: d.dir.clone(), hs_code: d.hs_code(), status: "skipped_errors".into(), zip_name: None,
                        zip_sha256: None, zip_size: None, objid: None, docs: d.rows.len(), not_pdfa: 0, unsigned: 0,
                        notes: Vec::new(), error: Some("hồ sơ còn LỖI ở bước kiểm — sửa Excel/thư mục rồi chạy lại".into()),
                    }
                } else if let Some(p) = prev.filter(|p| {
                    !o.force && p.status == "built"
                        && p.zip_name.as_ref().map_or(false, |z| {
                            std::fs::metadata(o.out_dir.join(z)).map(|m| Some(m.len()) == p.zip_size).unwrap_or(false)
                        })
                }) {
                    PackResult { status: "skipped_existing".into(), ..p }
                } else {
                    build_one(d, &plan.root, o)
                };
                if res.status == "built" {
                    let mut st = state.lock().unwrap();
                    // Dựng lại (force): xoá gói cũ của hồ sơ này để không lẫn hai bản.
                    if let Some(old) = st.get(&d.dir).and_then(|p| p.zip_name.clone()) {
                        if Some(&old) != res.zip_name.as_ref() {
                            let _ = std::fs::remove_file(o.out_dir.join(old));
                        }
                    }
                    st.insert(d.dir.clone(), res.clone());
                    save_state(&o.out_dir, &st);
                }
                progress(done.fetch_add(1, Ordering::Relaxed) + 1, total, &res);
                out.lock().unwrap().push((i, res));
            });
        }
    });
    let mut v = out.into_inner().unwrap();
    v.sort_by_key(|x| x.0);
    Ok(v.into_iter().map(|x| x.1).collect())
}
