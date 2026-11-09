// Copyright (c) 2026 DigiText. All rights reserved.
//! `digitext-api migrate` — áp `migrations/*.sql` theo thứ tự tên (bản Lưu trữ GĐ2, 9/11/2026; dùng cho cả máy chủ thi công
//! thay việc áp tay bằng psql). Đã áp ghi ở bảng `_digitext_migrations` (tên, sha256, lúc áp).
//!
//! Mỗi tệp được tách thành từng câu lệnh và chạy lần lượt trên MỘT kết nối ở chế độ autocommit: 0006 / 0017 / 0021 có
//! `CREATE INDEX CONCURRENTLY` (không chạy được trong giao dịch), còn tệp nào tự viết `BEGIN; … COMMIT;` thì giao dịch đó
//! vẫn giữ nguyên vì cùng kết nối. Câu lệnh lỗi giữa giao dịch → `ROLLBACK`, dừng, tệp không được ghi là đã áp.
//!
//! Máy chủ đã áp tay từ trước (OCR PC): chạy một lần `migrate --da-ap-den 0026` để đánh dấu 0001–0026 là đã áp mà không
//! chạy lại. CSDL đã có bảng mà chưa có `_digitext_migrations` thì lệnh từ chối, tránh chạy lại 0001 lên dữ liệu thật.
use anyhow::{bail, Context};
use sha2::{Digest, Sha256};
use sqlx::{Connection, Executor, PgConnection};
use std::path::{Path, PathBuf};

/// Tách SQL thành câu lệnh theo `;` ngoài chuỗi `'…'`, định danh `"…"`, dollar-quote `$tag$…$tag$` và comment.
pub fn tach_cau_lenh(sql: &str) -> Vec<String> {
    let b = sql.as_bytes();
    let mut out = Vec::new();
    let mut start = 0;
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'-' if b.get(i + 1) == Some(&b'-') => {
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if b.get(i + 1) == Some(&b'*') => {
                i += 2;
                while i + 1 < b.len() && !(b[i] == b'*' && b[i + 1] == b'/') {
                    i += 1;
                }
                i += 2;
                continue;
            }
            q @ (b'\'' | b'"') => {
                i += 1;
                while i < b.len() {
                    if b[i] == q {
                        if b.get(i + 1) == Some(&q) {
                            i += 2;
                            continue;
                        }
                        break;
                    }
                    i += 1;
                }
            }
            b'$' => {
                // $tag$ với tag rỗng hoặc chữ/số/_ (không bắt đầu bằng số — $1 là tham số).
                let mut j = i + 1;
                while j < b.len() && (b[j].is_ascii_alphanumeric() || b[j] == b'_') {
                    j += 1;
                }
                let la_tag = j < b.len() && b[j] == b'$' && !b.get(i + 1).is_some_and(|c| c.is_ascii_digit());
                if la_tag {
                    let tag = &sql[i..=j];
                    match sql[j + 1..].find(tag) {
                        Some(k) => i = j + 1 + k + tag.len() - 1,
                        None => i = b.len(),
                    }
                }
            }
            b';' => {
                let s = sql[start..i].trim();
                if co_lenh(s) {
                    out.push(s.to_string());
                }
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    let s = sql[start.min(sql.len())..].trim();
    if co_lenh(s) {
        out.push(s.to_string());
    }
    out
}

/// Đoạn còn gì ngoài comment và khoảng trắng.
fn co_lenh(s: &str) -> bool {
    s.lines().map(|l| l.split("--").next().unwrap_or("").trim()).any(|l| !l.is_empty())
}

/// Thư mục migrations: `--thu-muc`, rồi `DIGITEXT_MIGRATIONS_DIR`, rồi `<thư mục exe>/migrations`, rồi `./migrations`.
pub fn tim_thu_muc(chi_dinh: Option<&str>) -> anyhow::Result<PathBuf> {
    if let Some(d) = chi_dinh.map(PathBuf::from).or_else(|| std::env::var_os("DIGITEXT_MIGRATIONS_DIR").map(PathBuf::from)) {
        if d.is_dir() {
            return Ok(d);
        }
        bail!("không có thư mục migrations {}", d.display());
    }
    let canh_exe = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.join("migrations")));
    for d in canh_exe.into_iter().chain([PathBuf::from("migrations")]) {
        if d.is_dir() {
            return Ok(d);
        }
    }
    bail!("không tìm thấy thư mục migrations (đặt cạnh digitext-api.exe, hoặc --thu-muc / DIGITEXT_MIGRATIONS_DIR)")
}

/// (tên tệp, nội dung) theo thứ tự tên; chỉ tệp dạng `NNNN_*.sql`.
pub fn doc_tep(dir: &Path) -> anyhow::Result<Vec<(String, String)>> {
    let mut v = Vec::new();
    for e in std::fs::read_dir(dir).with_context(|| format!("đọc {}", dir.display()))? {
        let p = e?.path();
        let ten = p.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        if ten.ends_with(".sql") && ten.len() > 5 && ten[..4].bytes().all(|c| c.is_ascii_digit()) {
            v.push((ten, std::fs::read_to_string(&p).with_context(|| format!("đọc {}", p.display()))?));
        }
    }
    v.sort();
    Ok(v)
}

fn sha(s: &str) -> String {
    format!("{:x}", Sha256::digest(s.as_bytes()))
}

pub struct Tuychon {
    pub thu_muc: Option<String>,
    /// Đánh dấu mọi tệp có số ≤ giá trị này là đã áp (không chạy) — máy chủ áp tay từ trước.
    pub da_ap_den: Option<u32>,
    /// Chỉ liệt kê, không áp.
    pub chi_xem: bool,
}

pub async fn chay(database_url: &str, tc: Tuychon) -> anyhow::Result<()> {
    let dir = tim_thu_muc(tc.thu_muc.as_deref())?;
    let tep = doc_tep(&dir)?;
    if tep.is_empty() {
        bail!("{} không có tệp NNNN_*.sql", dir.display());
    }
    let mut conn = PgConnection::connect(database_url).await.context("kết nối Postgres thất bại")?;
    let co_bang_theo_doi: bool = sqlx::query_scalar("SELECT to_regclass('public._digitext_migrations') IS NOT NULL").fetch_one(&mut conn).await?;
    if !co_bang_theo_doi {
        let co_du_lieu: bool = sqlx::query_scalar("SELECT to_regclass('public.projects') IS NOT NULL").fetch_one(&mut conn).await?;
        if co_du_lieu && tc.da_ap_den.is_none() {
            bail!(
                "CSDL đã có bảng nhưng chưa có _digitext_migrations (migration từng áp tay). Chạy một lần \
                 `digitext-api migrate --da-ap-den <số migration cuối đã áp>` để đánh dấu, rồi chạy lại `migrate`."
            );
        }
        if tc.chi_xem {
            println!("Chưa có bảng _digitext_migrations — mọi tệp đều chờ áp.");
        } else {
            conn.execute("CREATE TABLE IF NOT EXISTS _digitext_migrations (ten text PRIMARY KEY, sha256 text NOT NULL, applied_at timestamptz NOT NULL DEFAULT now(), danh_dau boolean NOT NULL DEFAULT false)")
                .await?;
        }
    }
    let da_ap: Vec<(String, String)> = if co_bang_theo_doi {
        sqlx::query_as("SELECT ten, sha256 FROM _digitext_migrations").fetch_all(&mut conn).await?
    } else {
        Vec::new()
    };
    let da_ap: std::collections::BTreeMap<String, String> = da_ap.into_iter().collect();

    let mut so_ap = 0;
    for (ten, noi_dung) in &tep {
        let h = sha(noi_dung);
        if let Some(cu) = da_ap.get(ten) {
            if *cu != h {
                println!("  ! {ten}: nội dung khác lúc áp (sha256 đổi) — không áp lại, kiểm tra tay nếu cần");
            }
            continue;
        }
        let so: u32 = ten[..4].parse().unwrap_or(u32::MAX);
        if tc.da_ap_den.is_some_and(|n| so <= n) {
            if tc.chi_xem {
                println!("  = {ten} (sẽ đánh dấu đã áp)");
            } else {
                sqlx::query("INSERT INTO _digitext_migrations (ten, sha256, danh_dau) VALUES ($1, $2, true) ON CONFLICT (ten) DO NOTHING")
                    .bind(ten)
                    .bind(&h)
                    .execute(&mut conn)
                    .await?;
                println!("  = {ten} đánh dấu đã áp");
            }
            continue;
        }
        if tc.chi_xem {
            println!("  + {ten} chờ áp");
            continue;
        }
        let bat_dau = std::time::Instant::now();
        for (k, cau) in tach_cau_lenh(noi_dung).iter().enumerate() {
            if let Err(e) = conn.execute(cau.as_str()).await {
                let _ = conn.execute("ROLLBACK").await;
                let dau: String = cau.chars().take(160).collect();
                bail!("{ten}: câu lệnh {} lỗi: {e}\n  {dau}", k + 1);
            }
        }
        sqlx::query("INSERT INTO _digitext_migrations (ten, sha256) VALUES ($1, $2)").bind(ten).bind(&h).execute(&mut conn).await?;
        println!("  + {ten} đã áp ({:.1} s)", bat_dau.elapsed().as_secs_f32());
        so_ap += 1;
    }
    if !tc.chi_xem {
        println!("Xong: áp {so_ap} tệp mới; tổng {} tệp trong {}", tep.len(), dir.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tach_giu_chuoi_ham_va_comment() {
        let sql = "-- đầu; không tách\nBEGIN;\nCREATE FUNCTION f() RETURNS int LANGUAGE sql AS $$ SELECT 1; $$;\n\
                   INSERT INTO t VALUES ('a;b', 'it''s;');\n/* khối; comment */ CREATE TABLE \"x;y\" (a int);\n\
                   DO $body$ BEGIN PERFORM 1; END $body$;\nSELECT $1::int;\nCOMMIT;\n-- cuối";
        let v = tach_cau_lenh(sql);
        assert_eq!(v.len(), 7, "{v:#?}");
        assert_eq!(v[0], "-- đầu; không tách\nBEGIN");
        assert!(v[1].ends_with("$$ SELECT 1; $$"));
        assert!(v[2].contains("'it''s;'"));
        assert!(v[3].contains("\"x;y\""));
        assert!(v[4].ends_with("END $body$"));
        assert_eq!(v[5], "SELECT $1::int");
        assert_eq!(v[6], "COMMIT");
    }

    #[test]
    fn tach_moi_migration_that() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations");
        if !dir.is_dir() {
            return;
        }
        for (ten, sql) in doc_tep(&dir).unwrap() {
            let v = tach_cau_lenh(&sql);
            assert!(!v.is_empty(), "{ten}");
            // Không câu nào kết thúc giữa thân hàm: số dấu $$ trong mỗi câu phải chẵn.
            for c in &v {
                assert_eq!(c.matches("$$").count() % 2, 0, "{ten}: {}", c.chars().take(120).collect::<String>());
            }
        }
    }
}
