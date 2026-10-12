// Copyright (c) 2026 DigiText. All rights reserved.
//! Đọc một gói tin vào bộ nhớ: tệp .zip (kể cả zip bọc thêm một lớp zip — gói TKT 9/10/2026 giao kiểu
//! `AIP.zip/AIP/uuid_….zip`) hoặc thư mục đã giải nén. Đường dẫn tệp tính từ THƯ MỤC GỐC của gói.
use crate::{Finding, Level};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::Read;
use std::path::Path;

pub struct Loaded {
    /// Tên thư mục gốc của gói (rỗng nếu gói không có đúng một thư mục gốc).
    pub root: String,
    /// Đường dẫn tương đối từ thư mục gốc (dấu `/`) → nội dung.
    pub files: BTreeMap<String, Vec<u8>>,
    pub notes: Vec<Finding>,
    pub sha256: Option<String>,
    pub size: u64,
}

fn read_zip(bytes: &[u8], label: &str) -> Result<Vec<(String, Vec<u8>)>, String> {
    let mut z = zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|e| format!("{label}: không mở được ZIP ({e})"))?;
    let mut out = Vec::new();
    for i in 0..z.len() {
        let mut f = z.by_index(i).map_err(|e| format!("{label}: mục {i} hỏng ({e})"))?;
        if f.is_dir() {
            continue;
        }
        let name = f.name().replace('\\', "/");
        let mut b = Vec::with_capacity(f.size() as usize);
        // Đọc hết mục → zip tự đối chiếu CRC; sai CRC thì báo lỗi.
        f.read_to_end(&mut b).map_err(|e| format!("{label}: mục \"{name}\" hỏng / sai CRC ({e})"))?;
        out.push((name, b));
    }
    Ok(out)
}

fn split_root(entries: Vec<(String, Vec<u8>)>, notes: &mut Vec<Finding>) -> (String, BTreeMap<String, Vec<u8>>) {
    let tops: std::collections::BTreeSet<String> = entries
        .iter()
        .filter_map(|(n, _)| n.split_once('/').map(|(t, _)| t.to_string()))
        .collect();
    let loose: Vec<&String> = entries.iter().filter(|(n, _)| !n.contains('/')).map(|(n, _)| n).collect();
    if tops.len() == 1 && loose.is_empty() {
        let root = tops.into_iter().next().unwrap_or_default();
        let files = entries.into_iter().map(|(n, b)| (n[root.len() + 1..].to_string(), b)).collect();
        notes.push(Finding::new(Level::Ok, "Cấu trúc", format!("Gói có đúng một thư mục gốc \"{root}\"")));
        return (root, files);
    }
    // Không có thư mục gốc nhưng METS.xml nằm ngay ở gốc zip: vẫn kiểm tiếp, báo lỗi cấu trúc.
    if entries.iter().any(|(n, _)| n == "METS.xml") {
        notes.push(Finding::new(Level::Err, "Cấu trúc", "Gói không có thư mục gốc — các tệp nằm thẳng ở gốc ZIP (E-ARK CSIP yêu cầu một thư mục gốc trùng tên gói)".into()));
        return (String::new(), entries.into_iter().collect());
    }
    if !loose.is_empty() && tops.len() == 1 {
        notes.push(Finding::new(Level::Err, "Cấu trúc", format!("Có tệp nằm ngoài thư mục gốc: {}", loose.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "))));
        let root = tops.into_iter().next().unwrap_or_default();
        let files = entries.into_iter().filter(|(n, _)| n.contains('/')).map(|(n, b)| (n[root.len() + 1..].to_string(), b)).collect();
        return (root, files);
    }
    notes.push(Finding::new(Level::Err, "Cấu trúc", format!("Gói có {} thư mục gốc ({}) — phải đúng một", tops.len(), tops.iter().take(5).cloned().collect::<Vec<_>>().join(", "))));
    (String::new(), entries.into_iter().collect())
}

pub fn load(path: &Path) -> Result<Loaded, String> {
    let mut notes = Vec::new();
    if path.is_dir() {
        let root = path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        let mut files = BTreeMap::new();
        let mut size = 0u64;
        let mut st = vec![path.to_path_buf()];
        while let Some(d) = st.pop() {
            for e in std::fs::read_dir(&d).map_err(|e| format!("đọc thư mục {}: {e}", d.display()))?.flatten() {
                let p = e.path();
                if p.is_dir() {
                    st.push(p);
                } else {
                    let rel = p.strip_prefix(path).unwrap_or(&p).to_string_lossy().replace('\\', "/");
                    let b = std::fs::read(&p).map_err(|e| format!("đọc {}: {e}", p.display()))?;
                    size += b.len() as u64;
                    files.insert(rel, b);
                }
            }
        }
        notes.push(Finding::new(Level::Ok, "Cấu trúc", format!("Kiểm thư mục đã giải nén \"{root}\"")));
        return Ok(Loaded { root, files, notes, sha256: None, size });
    }
    let bytes = std::fs::read(path).map_err(|e| format!("đọc {}: {e}", path.display()))?;
    let size = bytes.len() as u64;
    let sha = format!("{:x}", Sha256::digest(&bytes));
    let label = path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let mut entries = read_zip(&bytes, &label)?;
    notes.push(Finding::new(Level::Ok, "Cấu trúc", "ZIP mở được, CRC mọi mục đúng".into()));
    // Zip chỉ chứa đúng một tệp .zip (có thể nằm trong một thư mục) → gói thật nằm bên trong (một lớp).
    if entries.len() == 1 && entries[0].0.to_lowercase().ends_with(".zip") {
        let (inner_name, inner) = entries.remove(0);
        notes.push(Finding::new(
            Level::Warn,
            "Cấu trúc",
            format!("Gói bị nén thêm một lớp: \"{label}\" chỉ chứa \"{inner_name}\" — nên giao trực tiếp tệp gói, không bọc ZIP"),
        ));
        entries = read_zip(&inner, &inner_name)?;
        // Tên tệp ZIP bên trong nên trùng tên thư mục gốc.
        let stem = inner_name.rsplit('/').next().unwrap_or(&inner_name).trim_end_matches(".zip").trim_end_matches(".ZIP").to_string();
        let (root, files) = split_root(std::mem::take(&mut entries), &mut notes);
        if !root.is_empty() && stem != root {
            notes.push(Finding::new(Level::Warn, "Cấu trúc", format!("Tên tệp ZIP \"{stem}\" khác tên thư mục gốc \"{root}\"")));
        }
        return Ok(Loaded { root, files, notes, sha256: Some(sha), size });
    }
    let (root, files) = split_root(entries, &mut notes);
    let stem = label.trim_end_matches(".zip").trim_end_matches(".ZIP");
    if !root.is_empty() && stem != root {
        notes.push(Finding::new(Level::Warn, "Cấu trúc", format!("Tên tệp ZIP \"{stem}\" khác tên thư mục gốc \"{root}\"")));
    }
    Ok(Loaded { root, files, notes, sha256: Some(sha), size })
}

/// Tìm các gói trong danh sách đường dẫn: tệp .zip, thư mục gói (có METS.xml), hoặc thư mục chứa nhiều gói.
pub fn discover(paths: &[String]) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let mut st: Vec<(std::path::PathBuf, usize)> = paths.iter().map(|p| (std::path::PathBuf::from(p), 0)).collect();
    while let Some((p, depth)) = st.pop() {
        if p.is_file() {
            if p.extension().is_some_and(|e| e.eq_ignore_ascii_case("zip")) {
                out.push(p);
            }
        } else if p.is_dir() {
            if p.join("METS.xml").is_file() {
                out.push(p);
            } else if depth < 6 {
                if let Ok(rd) = std::fs::read_dir(&p) {
                    for e in rd.flatten() {
                        st.push((e.path(), depth + 1));
                    }
                }
            }
        }
    }
    out.sort();
    out.dedup();
    out
}
