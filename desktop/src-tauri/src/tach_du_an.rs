// Copyright (c) 2026 DigiText. All rights reserved.
//! Tách các bảng SQLite đời cũ theo dự án (23/11/2026).
//!
//! Bảng đời cũ của máy trạm (`phong`, `ho_so`, `trang`, `dossier`, `van_ban`, `phong_ban`, `mang_cong_viec`, `thoi_han_bq`,
//! `kho`, `gia`, `ho_so_history`) không có cột dự án và khoá của chúng không gồm dự án (`phong.ma_du_an`; `ho_so` theo
//! phông + hộp + số; `dossier` theo phông + THBQ + số tạm). Tài khoản quản trị đổi "Dự án đang làm" thì phông, hồ sơ của
//! mọi dự án từng đồng bộ hiện lẫn vào nhau, và hai dự án trùng mã phông đè dữ liệu của nhau.
//!
//! Cách sửa không đụng tới hàng trăm câu SQL đang dùng các bảng này: tại một thời điểm bảng cũ CHỈ chứa dữ liệu của dự án
//! đang làm. Dữ liệu của các dự án khác nằm trong bảng kho `cat_<bảng>` (thêm cột `du_an`). Đổi dự án = cất dự án cũ vào
//! kho, nạp dự án mới ra (giữ nguyên id). `""` là ngăn "chưa chọn dự án".
//!
//! Lần đầu chạy (`tach_lan_dau`): sao lưu tệp SQLite rồi chia dữ liệu đang lẫn về từng dự án — theo `cl_id` →
//! `cl_*.project_id`; phông, hồ sơ cây thư mục theo mã phông có trong `cl_phong` của dự án (mã dùng ở nhiều dự án thì mỗi
//! dự án một bản); dòng không xác định được dự án thuộc về dự án đang làm lúc tách.

use rusqlite::{params, Connection, OptionalExtension};

/// Dự án đang nằm trong bảng cũ (`""` = chưa chọn).
pub const KEY_SONG: &str = "bang_cu_du_an";
const KEY_DA_TACH: &str = "bang_cu_da_tach";
const KEY_DA_CHIEU: &str = "bang_cu_da_chieu:";

/// Thứ tự NẠP: cha trước con (khoá ngoại bật). CẤT xoá theo thứ tự ngược để ON DELETE SET NULL / CASCADE không chạm dòng nào.
const BANG: &[&str] = &["phong", "thoi_han_bq", "kho", "gia", "phong_ban", "mang_cong_viec", "dossier", "van_ban", "ho_so_history", "ho_so", "trang"];

fn co_bang(conn: &Connection, t: &str) -> rusqlite::Result<bool> {
    Ok(conn.query_row("SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = ?1", params![t], |r| r.get::<_, i64>(0))? > 0)
}

fn cot(conn: &Connection, t: &str) -> rusqlite::Result<Vec<String>> {
    let mut st = conn.prepare(&format!("PRAGMA table_info(\"{t}\")"))?;
    let v = st.query_map([], |r| r.get::<_, String>(1))?.collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(v)
}

fn co_cot(conn: &Connection, t: &str, c: &str) -> rusqlite::Result<bool> {
    Ok(cot(conn, t)?.iter().any(|x| x == c))
}

/// `"a", "b"` hoặc `x."a", x."b"`.
fn ds(cols: &[String], bi_danh: Option<&str>) -> String {
    cols.iter().map(|c| bi_danh.map_or_else(|| format!("\"{c}\""), |a| format!("{a}.\"{c}\""))).collect::<Vec<_>>().join(", ")
}

/// Bảng kho `cat_<t>` có đủ cột của bảng gốc (bảng gốc được thêm cột qua các lần nâng cấp). Trả danh sách cột bảng gốc.
fn dam_bao_kho(conn: &Connection, t: &str) -> rusqlite::Result<Vec<String>> {
    let cols = cot(conn, t)?;
    let kho = format!("cat_{t}");
    if !co_bang(conn, &kho)? {
        conn.execute_batch(&format!("CREATE TABLE \"{kho}\" AS SELECT '' AS du_an, * FROM \"{t}\" WHERE 0; CREATE INDEX IF NOT EXISTS ix_{kho}_du_an ON \"{kho}\" (du_an);"))?;
    } else {
        let da_co = cot(conn, &kho)?;
        for c in &cols {
            if !da_co.contains(c) {
                conn.execute_batch(&format!("ALTER TABLE \"{kho}\" ADD COLUMN \"{c}\";"))?;
            }
        }
    }
    Ok(cols)
}

fn bang_co(conn: &Connection) -> rusqlite::Result<Vec<&'static str>> {
    let mut v = Vec::new();
    for t in BANG {
        if co_bang(conn, t)? {
            v.push(*t);
        }
    }
    Ok(v)
}

fn xoa_bang_song(conn: &Connection, bang: &[&str]) -> rusqlite::Result<()> {
    for t in bang.iter().rev() {
        conn.execute(&format!("DELETE FROM \"{t}\""), [])?;
    }
    Ok(())
}

/// Cất toàn bộ bảng cũ vào ngăn `du_an`.
fn cat(conn: &Connection, du_an: &str) -> rusqlite::Result<()> {
    let bang = bang_co(conn)?;
    for t in &bang {
        let cols = dam_bao_kho(conn, t)?;
        conn.execute(&format!("INSERT INTO \"cat_{t}\" (du_an, {c}) SELECT ?1, {c} FROM \"{t}\"", c = ds(&cols, None)), params![du_an])?;
    }
    xoa_bang_song(conn, &bang)
}

/// Nạp ngăn `du_an` ra bảng cũ (bảng cũ phải đang rỗng). Trả số dòng đã nạp.
fn nap(conn: &Connection, du_an: &str) -> rusqlite::Result<usize> {
    let mut n = 0;
    for t in bang_co(conn)? {
        let cols = dam_bao_kho(conn, t)?;
        n += conn.execute(&format!("INSERT OR IGNORE INTO \"{t}\" ({c}) SELECT {c} FROM \"cat_{t}\" WHERE du_an = ?1", c = ds(&cols, None)), params![du_an])?;
        conn.execute(&format!("DELETE FROM \"cat_{t}\" WHERE du_an = ?1"), params![du_an])?;
    }
    Ok(n)
}

fn dat(conn: &Connection, key: &str, v: &str) -> rusqlite::Result<()> {
    crate::db::f_2ei(conn, key, Some(v))
}

fn lay(conn: &Connection, key: &str) -> rusqlite::Result<String> {
    Ok(crate::db::f_1ew(conn, key)?.unwrap_or_default())
}

/// Chạy `f` trong một SAVEPOINT (dùng được cả khi đang ở trong giao dịch khác); lỗi thì hoàn tác.
fn voi_savepoint<T>(conn: &Connection, ten: &str, f: impl FnOnce() -> rusqlite::Result<T>) -> rusqlite::Result<T> {
    conn.execute_batch(&format!("SAVEPOINT {ten}"))?;
    match f() {
        Ok(v) => {
            conn.execute_batch(&format!("RELEASE {ten}"))?;
            Ok(v)
        }
        Err(e) => {
            let _ = conn.execute_batch(&format!("ROLLBACK TO {ten}; RELEASE {ten}"));
            Err(e)
        }
    }
}

/// Sao lưu tệp SQLite trước lần tách đầu (`<tệp>.truoc-tach-du-an`). CSDL trong bộ nhớ (test) thì bỏ qua.
fn sao_luu(conn: &Connection) -> rusqlite::Result<()> {
    let Some(path) = conn.path().filter(|p| !p.is_empty() && *p != ":memory:") else { return Ok(()) };
    let dich = format!("{path}.truoc-tach-du-an");
    if std::path::Path::new(&dich).exists() {
        return Ok(());
    }
    conn.execute("VACUUM INTO ?1", params![dich])?;
    Ok(())
}

const MA_PHONG_CL: &str = "(SELECT DISTINCT project_id, lower(trim(ma_du_an)) AS ma FROM cl_phong WHERE deleted_at IS NULL AND project_id IS NOT NULL AND ma_du_an IS NOT NULL)";

/// Chia dữ liệu đang lẫn trong bảng cũ về các ngăn dự án. Sau bước này bảng cũ rỗng, `KEY_SONG` = "".
fn tach_lan_dau(conn: &Connection, hien_tai: &str) -> rusqlite::Result<()> {
    let bang = bang_co(conn)?;
    let mut co_du_lieu = false;
    for t in &bang {
        // thoi_han_bq luôn có dòng gieo sẵn — không tính là dữ liệu người dùng.
        if *t != "thoi_han_bq" && conn.query_row(&format!("SELECT count(*) FROM \"{t}\""), [], |r| r.get::<_, i64>(0))? > 0 {
            co_du_lieu = true;
        }
    }
    if co_du_lieu {
        sao_luu(conn)?;
    }
    voi_savepoint(conn, "tach_lan_dau", || {
        let mut c: std::collections::HashMap<&str, Vec<String>> = std::collections::HashMap::new();
        for t in &bang {
            c.insert(*t, dam_bao_kho(conn, t)?);
        }
        let co = |t: &str| bang.contains(&t);
        // Chèn vào kho: `du_an` là biểu thức theo bí danh `x` của bảng gốc; `?1` = dự án đang làm.
        let chen = |t: &str, du_an: &str, tu: &str| -> rusqlite::Result<usize> {
            let cols = &c[t];
            let sql = format!("INSERT INTO \"cat_{t}\" (du_an, {a}) SELECT {du_an}, {b} FROM {tu}", a = ds(cols, None), b = ds(cols, Some("x")));
            if sql.contains("?1") {
                conn.execute(&sql, params![hien_tai])
            } else {
                conn.execute(&sql, [])
            }
        };
        let theo_cl = |t: &str, cl: &str| -> rusqlite::Result<String> {
            Ok(if co_cot(conn, t, "cl_id")? && co_bang(conn, cl)? { format!("(SELECT project_id FROM {cl} WHERE id = x.cl_id)") } else { "NULL".to_string() })
        };

        if co("dossier") {
            chen("dossier", &format!("COALESCE({}, ?1)", theo_cl("dossier", "cl_ho_so")?), "dossier x")?;
        }
        if co("van_ban") && co("dossier") {
            chen("van_ban", "k.du_an", "van_ban x JOIN cat_dossier k ON k.id = x.dossier_id")?;
        }
        if co("ho_so_history") {
            let tu_ho_so = if co("dossier") { "(SELECT k.du_an FROM cat_dossier k WHERE k.id = x.dossier_id LIMIT 1)" } else { "NULL" };
            chen("ho_so_history", &format!("COALESCE({tu_ho_so}, ?1)"), "ho_so_history x")?;
        }
        if co("phong_ban") {
            chen("phong_ban", &format!("COALESCE({}, ?1)", theo_cl("phong_ban", "cl_phong_ban")?), "phong_ban x")?;
        }
        if co("mang_cong_viec") {
            let tu_pb = if co("phong_ban") { "(SELECT k.du_an FROM cat_phong_ban k WHERE k.ma_dv = x.ma_dv LIMIT 1)" } else { "NULL" };
            chen("mang_cong_viec", &format!("COALESCE({}, {tu_pb}, ?1)", theo_cl("mang_cong_viec", "cl_mang")?), "mang_cong_viec x")?;
        }
        if co("kho") {
            chen("kho", &format!("COALESCE({}, ?1)", theo_cl("kho", "cl_kho")?), "kho x")?;
        }
        if co("gia") {
            let tu_kho = if co("kho") { "(SELECT k.du_an FROM cat_kho k WHERE k.ma_kho = x.ma_kho LIMIT 1)" } else { "NULL" };
            chen("gia", &format!("COALESCE({}, {tu_kho}, ?1)", theo_cl("gia", "cl_gia")?), "gia x")?;
        }
        // Phông và hồ sơ cây thư mục: theo mã phông có trong cl_phong của từng dự án (mỗi dự án một bản); không thuộc dự án
        // nào → dự án đang làm.
        if co("phong") {
            chen("phong", "c.project_id", &format!("phong x JOIN {MA_PHONG_CL} c ON c.ma = lower(trim(x.ma_du_an))"))?;
            chen("phong", "?1", &format!("phong x WHERE lower(trim(x.ma_du_an)) NOT IN (SELECT ma FROM {MA_PHONG_CL})"))?;
        }
        if co("ho_so") {
            chen("ho_so", "c.project_id", &format!("ho_so x JOIN {MA_PHONG_CL} c ON c.ma = lower(trim(x.phong_id))"))?;
            chen("ho_so", "?1", &format!("ho_so x WHERE x.phong_id IS NULL OR lower(trim(x.phong_id)) NOT IN (SELECT ma FROM {MA_PHONG_CL})"))?;
        }
        if co("trang") && co("ho_so") {
            chen("trang", "k.du_an", "trang x JOIN cat_ho_so k ON k.id = x.ho_so_id")?;
        }
        // THBQ: dòng gắn cl_thbq của một dự án → dự án đó; dòng gieo sẵn / dùng chung → mọi ngăn.
        if co("thoi_han_bq") {
            let cua = theo_cl("thoi_han_bq", "cl_thbq")?;
            let mut ngan = vec!["SELECT ?1 AS du_an".to_string(), "SELECT ''".to_string()];
            for t in ["dossier", "phong", "ho_so", "phong_ban"] {
                if co(t) {
                    ngan.push(format!("SELECT du_an FROM cat_{t}"));
                }
            }
            chen("thoi_han_bq", &cua, &format!("thoi_han_bq x WHERE {cua} IS NOT NULL"))?;
            chen("thoi_han_bq", "b.du_an", &format!("thoi_han_bq x JOIN ({}) b WHERE {cua} IS NULL", ngan.join(" UNION ")))?;
        }

        // Bổ sung dòng cha còn thiếu trong từng ngăn (khoá ngoại + tên hiển thị): mảng → phòng ban → phông; giá → kho; THBQ.
        let bu = |t: &str, khoa: &str, tham_chieu: &[(&str, &str)]| -> rusqlite::Result<()> {
            let nguon: Vec<String> = tham_chieu.iter().filter(|(b, col)| co(b) && co_cot(conn, b, col).unwrap_or(false)).map(|(b, col)| format!("SELECT du_an, \"{col}\" AS k FROM cat_{b} WHERE \"{col}\" IS NOT NULL")).collect();
            if !co(t) || nguon.is_empty() {
                return Ok(());
            }
            let cols = &c[t];
            conn.execute(
                &format!(
                    "INSERT INTO \"cat_{t}\" (du_an, {a}) SELECT r.du_an, {b} FROM (SELECT DISTINCT du_an, k FROM ({n})) r JOIN \"{t}\" x ON x.\"{khoa}\" = r.k \
                     WHERE NOT EXISTS (SELECT 1 FROM \"cat_{t}\" z WHERE z.du_an = r.du_an AND z.\"{khoa}\" = r.k)",
                    a = ds(cols, None),
                    b = ds(cols, Some("x")),
                    n = nguon.join(" UNION ")
                ),
                [],
            )?;
            Ok(())
        };
        bu("mang_cong_viec", "mang_cvid", &[("dossier", "mang_cvid")])?;
        bu("phong_ban", "ma_dv", &[("mang_cong_viec", "ma_dv"), ("dossier", "ma_dv")])?;
        bu("phong", "ma_du_an", &[("dossier", "ma_da"), ("ho_so", "phong_id"), ("phong_ban", "ma_phong")])?;
        bu("kho", "ma_kho", &[("gia", "ma_kho")])?;
        bu("thoi_han_bq", "thbqid", &[("dossier", "thbq")])?;

        xoa_bang_song(conn, &bang)?;
        dat(conn, KEY_SONG, "")?;
        dat(conn, KEY_DA_TACH, "1")
    })
}

/// Bảo đảm bảng cũ đang chứa đúng dự án `du_an` (`""` = chưa chọn). Trả `true` khi vừa đổi ngăn.
pub fn chuyen(conn: &Connection, du_an: &str) -> rusqlite::Result<bool> {
    let moi = du_an.trim();
    if lay(conn, KEY_DA_TACH)? != "1" {
        // Lần tách đầu có sao lưu (VACUUM) nên phải chạy ngoài giao dịch; đang trong giao dịch thì để lần gọi sau.
        if !conn.is_autocommit() {
            return Ok(false);
        }
        tach_lan_dau(conn, moi)?;
    }
    if lay(conn, KEY_SONG)? == moi {
        return Ok(false);
    }
    voi_savepoint(conn, "doi_du_an", || {
        let cu = lay(conn, KEY_SONG)?;
        cat(conn, &cu)?;
        nap(conn, moi)?;
        dat(conn, KEY_SONG, moi)
    })?;
    // Lần đầu mở một dự án sau khi tách: chiếu lại từ bản sao chỉnh lý để vá các dòng từng bị dự án khác đè / chặn
    // (trùng mã phông, trùng số tạm). Lỗi không làm hỏng việc đổi dự án.
    if !moi.is_empty() {
        let khoa = format!("{KEY_DA_CHIEU}{moi}");
        if lay(conn, &khoa)? != "1" {
            match voi_savepoint(conn, "chieu_lai", || crate::chinhly::legacy::project_all(conn, moi)) {
                Ok(_) => dat(conn, &khoa, "1")?,
                Err(e) => tracing::warn!(error = %e, du_an = moi, "tách dự án: không chiếu lại được bảng cũ"),
            }
        }
    }
    Ok(true)
}

/// Như `chuyen` nhưng nuốt lỗi (ghi log): dùng ở các điểm móc không được làm hỏng lệnh đang chạy.
pub fn dam_bao(conn: &Connection, du_an: Option<&str>) {
    if let Err(e) = chuyen(conn, du_an.unwrap_or("")) {
        tracing::warn!(error = %e, "tách dự án: không đổi được ngăn bảng cũ");
    }
}

/// Dự án đang nằm trong bảng cũ, nếu đã tách.
#[allow(dead_code)]
pub fn dang_song(conn: &Connection) -> Option<String> {
    conn.query_row("SELECT value FROM settings WHERE key = ?1", params![KEY_SONG], |r| r.get::<_, String>(0)).optional().ok().flatten()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open() -> (Connection, std::path::PathBuf) {
        let f = std::env::temp_dir().join(format!("digitext-tach-{}-{}.db", std::process::id(), uuid::Uuid::now_v7()));
        let _ = std::fs::remove_file(&f);
        (crate::db::open(&f).unwrap(), f)
    }

    fn dem(c: &Connection, sql: &str) -> i64 {
        c.query_row(sql, [], |r| r.get(0)).unwrap()
    }

    fn cl(c: &Connection, bang: &str, id: &str, du_an: &str, json: &str) {
        c.execute(&format!("INSERT INTO {bang} (id, project_id, row_json) VALUES (?1, ?2, ?3)"), params![id, du_an, json]).unwrap();
    }

    /// Chạy tay trên BẢN SAO CSDL thật: `DIGITEXT_THU_DB=<đường dẫn> cargo test thu_tren_ban_sao -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn thu_tren_ban_sao() {
        let Ok(path) = std::env::var("DIGITEXT_THU_DB") else { return };
        let c = crate::db::open(std::path::Path::new(&path)).unwrap();
        let bang = ["phong", "ho_so", "dossier", "van_ban", "phong_ban", "mang_cong_viec", "thoi_han_bq"];
        let in_dem = |nhan: &str| {
            let s: Vec<String> = bang.iter().map(|t| format!("{t}={}", dem(&c, &format!("SELECT count(*) FROM {t}")))).collect();
            println!("{nhan}: {}", s.join(" "));
        };
        in_dem("truoc khi tach");
        let du_an: Vec<String> = {
            let mut st = c.prepare("SELECT DISTINCT project_id FROM cl_phong WHERE project_id IS NOT NULL ORDER BY 1").unwrap();
            let v = st.query_map([], |r| r.get(0)).unwrap().collect::<rusqlite::Result<Vec<_>>>().unwrap();
            v
        };
        let dang_chon = lay(&c, "cl_project_id").unwrap();
        let t0 = std::time::Instant::now();
        chuyen(&c, &dang_chon).unwrap();
        println!("tach lan dau {} ms", t0.elapsed().as_millis());
        in_dem(&format!("du an dang chon {}", &dang_chon.chars().take(8).collect::<String>()));
        for p in &du_an {
            let t = std::time::Instant::now();
            chuyen(&c, p).unwrap();
            let ten: Vec<String> = {
                let mut st = c.prepare("SELECT ma_du_an || ' ' || ten_du_an FROM phong ORDER BY 1").unwrap();
                let v = st.query_map([], |r| r.get(0)).unwrap().collect::<rusqlite::Result<Vec<_>>>().unwrap();
                v
            };
            in_dem(&format!("{} ({} ms)", &p.chars().take(8).collect::<String>(), t.elapsed().as_millis()));
            println!("   phong: {}", ten.join(" ; "));
            assert_eq!(dem(&c, "SELECT count(*) FROM pragma_foreign_key_check"), 0, "khoá ngoại");
        }
        chuyen(&c, "").unwrap();
        in_dem("chua chon du an");
    }

    #[test]
    fn hai_du_an_trung_ma_phong_khong_con_lan() {
        let (c, f) = open();
        // Hai dự án A, B cùng có phông mã "01"; bảng cũ đang lẫn: tên phông bị B đè, hồ sơ của cả hai nằm chung.
        cl(&c, "cl_phong", "pa", "A", r#"{"id":"pa","ma_du_an":"01","ten":"UBND SONG KHE"}"#);
        cl(&c, "cl_phong", "pb", "B", r#"{"id":"pb","ma_du_an":"01","ten":"LUU TRU LICH SU"}"#);
        cl(&c, "cl_ho_so", "ha", "A", r#"{"id":"ha","phong_id":"pa","so_tam":"1","tieu_de":"Ho so cua A"}"#);
        cl(&c, "cl_ho_so", "hb", "B", r#"{"id":"hb","phong_id":"pb","so_tam":"2","tieu_de":"Ho so cua B"}"#);
        crate::db::f_2o6(&c, "01", "LUU TRU LICH SU", None, None, None).unwrap();
        crate::db::f_2o6(&c, "MAY", "Chi co tren may", None, None, None).unwrap();
        c.execute("INSERT INTO dossier (ma_da, so_tam, tieu_de, cl_id) VALUES ('01', '1', 'Ho so cua A', 'ha')", []).unwrap();
        c.execute("INSERT INTO dossier (ma_da, so_tam, tieu_de, cl_id) VALUES ('01', '2', 'Ho so cua B', 'hb')", []).unwrap();
        c.execute("INSERT INTO dossier (ma_da, so_tam, tieu_de) VALUES ('MAY', '9', 'Chua dong bo')", []).unwrap();
        let id_a: i64 = c.query_row("SELECT id FROM dossier WHERE cl_id = 'ha'", [], |r| r.get(0)).unwrap();
        c.execute("INSERT INTO van_ban (dossier_id, thu_tu, trich_yeu) VALUES (?1, 1, 'VB cua A')", params![id_a]).unwrap();
        c.execute("INSERT INTO ho_so (phong_id, hop_so, ho_so_so, tieu_de) VALUES ('01', '1', '1', 'Cay thu muc')", []).unwrap();

        // Lần đầu: đang làm dự án A.
        assert!(chuyen(&c, "A").unwrap());
        assert_eq!(dem(&c, "SELECT count(*) FROM dossier WHERE cl_id = 'hb'"), 0, "hồ sơ của B không còn ở A");
        assert_eq!(dem(&c, "SELECT count(*) FROM dossier WHERE cl_id = 'ha'"), 1);
        assert_eq!(dem(&c, "SELECT count(*) FROM dossier WHERE tieu_de = 'Chua dong bo'"), 1, "dòng chưa rõ dự án về dự án đang làm");
        assert_eq!(dem(&c, "SELECT count(*) FROM van_ban"), 1);
        assert_eq!(dem(&c, "SELECT count(*) FROM phong WHERE ma_du_an = '01'"), 1);
        assert_eq!(c.query_row::<i64, _, _>("SELECT id FROM dossier WHERE cl_id = 'ha'", [], |r| r.get(0)).unwrap(), id_a, "giữ nguyên id");
        assert!(std::path::Path::new(&format!("{}.truoc-tach-du-an", f.display())).exists(), "có sao lưu trước khi tách");
        assert!(!chuyen(&c, "A").unwrap(), "gọi lại không làm gì");

        // Sang B: chỉ thấy của B; phông "01" có mặt ở cả hai ngăn.
        assert!(chuyen(&c, "B").unwrap());
        assert_eq!(dem(&c, "SELECT count(*) FROM dossier WHERE cl_id = 'ha'"), 0);
        assert_eq!(dem(&c, "SELECT count(*) FROM dossier WHERE cl_id = 'hb'"), 1);
        assert_eq!(dem(&c, "SELECT count(*) FROM dossier WHERE tieu_de = 'Chua dong bo'"), 0);
        assert_eq!(dem(&c, "SELECT count(*) FROM van_ban"), 0);
        assert_eq!(dem(&c, "SELECT count(*) FROM phong WHERE ma_du_an = '01'"), 1);
        assert_eq!(dem(&c, "SELECT count(*) FROM phong WHERE ma_du_an = 'MAY'"), 0);
        assert_eq!(dem(&c, "SELECT count(*) FROM ho_so"), 1, "hồ sơ cây thư mục của mã phông dùng chung có ở cả hai dự án");
        // B thêm hồ sơ trùng số tạm với A — trước đây vướng UNIQUE (ma_da, thbq, so_tam).
        c.execute("INSERT INTO dossier (ma_da, so_tam, tieu_de) VALUES ('01', '1', 'So tam 1 cua B')", []).unwrap();

        // Về A: dữ liệu A nguyên vẹn, không thấy dòng B vừa thêm.
        assert!(chuyen(&c, "A").unwrap());
        assert_eq!(dem(&c, "SELECT count(*) FROM dossier WHERE tieu_de = 'So tam 1 cua B'"), 0);
        assert_eq!(dem(&c, "SELECT count(*) FROM dossier WHERE cl_id = 'ha'"), 1);
        assert_eq!(dem(&c, "SELECT count(*) FROM van_ban WHERE trich_yeu = 'VB cua A'"), 1);
        // Chưa chọn dự án: không thấy gì.
        assert!(chuyen(&c, "").unwrap());
        assert_eq!(dem(&c, "SELECT count(*) FROM dossier"), 0);
        assert_eq!(dem(&c, "SELECT count(*) FROM phong"), 0);
        // Lại B: dòng B thêm vẫn còn.
        assert!(chuyen(&c, "B").unwrap());
        assert_eq!(dem(&c, "SELECT count(*) FROM dossier WHERE tieu_de = 'So tam 1 cua B'"), 1);
        drop(c);
        let _ = std::fs::remove_file(&f);
        let _ = std::fs::remove_file(format!("{}.truoc-tach-du-an", f.display()));
    }
}
