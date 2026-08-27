// Copyright (c) 2026 DigiText. All rights reserved.
use digitext_storage::{FsObjectStore, MinioObjectStore, Tde};
use digitext_wipe::server::Tdp;
use digitext_wipe::{local, server, Depth, Scope, Thd};
use std::path::PathBuf;
use std::process::ExitCode;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tq2 {
    pub scope: Scope,
    pub depth: Depth,
    pub apply: bool,
    pub confirm: Option<String>,
    pub storage_root: Option<PathBuf>,
}
pub fn help_text(prog: &str) -> String {
    format!(
        "{prog} — xoá sạch dữ liệu DigiText (KHÔNG hoàn tác được)\n\nCách dùng:\n  {prog} --local  [--storage-root <đường-dẫn>]   Máy người dùng / máy admin (app desktop)\n  {prog} --server [--all]                        Máy chủ: Postgres + kho object + đĩa\n\nTuỳ chọn:\n  --apply            Thi hành thật. KHÔNG có cờ này = chỉ IN KẾ HOẠCH (mặc định an toàn)\n  --confirm <câu>    Cấp sẵn câu xác nhận (chạy trong script); không có ⇒ hỏi trên bàn phím\n  --all              (chỉ với --server) xoá cả tài khoản, dự án, danh mục — sau đó phải\n                     chạy lại `digitext api bootstrap-superadmin`\n  --storage-root <p> (chỉ với --local) chỉ định kho tài liệu thay vì đọc từ settings\n  --help / -h        In trợ giúp này\n\nBiến môi trường (tầng --server): DATABASE_URL hoặc DIGITEXT_DATABASE_URL · DIGITEXT_DATA_DIR ·\nDIGITEXT_PDF_STORE_DIR · DIGITEXT_MINIO_* (đủ 4 khoá mới bật).\n\nQuy trình: dựng kế hoạch (đếm, không đụng dữ liệu) → bạn duyệt → gõ NGUYÊN VĂN câu xác nhận\n(câu gắn với tên CSDL / đường dẫn kho, nên câu của máy thử không chạy được trên máy thật).\n"
    )
}
pub fn parse_args(r_1xa: &[String]) -> Result<Tq2, String> {
    let (mut r_1xc, mut r_1xd, mut apply, mut all) = (false, false, false, false);
    let mut confirm = None;
    let mut storage_root = None;
    let mut r_1xb = r_1xa.iter();
    while let Some(r_1x9) = r_1xb.next() {
        match r_1x9.as_str() {
            "--local" => r_1xc = true,
            "--server" => r_1xd = true,
            "--apply" => apply = true,
            "--all" => all = true,
            "--confirm" => {
                confirm = Some(
                    r_1xb
                        .next()
                        .ok_or("--confirm cần một câu đi kèm")?
                        .clone(),
                );
            }
            "--storage-root" => {
                storage_root =
                    Some(PathBuf::from(r_1xb.next().ok_or(
                        "--storage-root cần đường dẫn đi kèm",
                    )?));
            }
            other => {
                return Err(format!("tham số không hợp lệ: {other}"));
            }
        }
    }
    let scope = match (r_1xc, r_1xd) {
        (true, false) => Scope::Local,
        (false, true) => Scope::Server,
        (true, true) => {
            return Err(
                "chọn MỘT tầng mỗi lần: --local hoặc --server (xoá hai tầng cùng lúc thì một câu xác nhận sẽ phải gánh cả hai)"
                    .into(),
            );
        }
        (false, false) => {
            return Err(
                "thiếu tầng cần xoá: --local hoặc --server".into(),
            );
        }
    };
    if all && scope != Scope::Server {
        return Err("--all chỉ dùng với --server".into());
    }
    if storage_root.is_some() && scope != Scope::Local {
        return Err("--storage-root chỉ dùng với --local".into());
    }
    Ok(Tq2 {
        scope,
        depth: if all { Depth::All } else { Depth::Data },
        apply,
        confirm,
        storage_root,
    })
}
fn f_17n() -> Result<String, String> {
    std::env::var("DATABASE_URL")
        .or_else(|_| std::env::var("DIGITEXT_DATABASE_URL"))
        .map_err(|_| {
            "cần DATABASE_URL (hoặc DIGITEXT_DATABASE_URL)".to_owned()
        })
}
fn minio_from_env() -> Option<Tde> {
    let r_1xf = |r_1xe: &str| std::env::var(r_1xe).ok().filter(|r_1xg| !r_1xg.is_empty());
    Some(Tde {
        endpoint: r_1xf("DIGITEXT_MINIO_ENDPOINT")?,
        bucket: r_1xf("DIGITEXT_MINIO_BUCKET")?,
        access_key: r_1xf("DIGITEXT_MINIO_ACCESS_KEY")?,
        secret_key: r_1xf("DIGITEXT_MINIO_SECRET_KEY")?,
        region: r_1xf("DIGITEXT_MINIO_REGION").unwrap_or_else(|| "us-east-1".into()),
    })
}
fn f_sc(expected: &str) -> Result<String, String> {
    use std::io::{BufRead, Write};
    println!(
        "\nĐể thi hành, gõ NGUYÊN VĂN câu sau rồi Enter (Ctrl-C để huỷ):\n\n    {expected}\n"
    );
    print!("> ");
    std::io::stdout().flush().ok();
    let mut r_1xh = String::new();
    let n = std::io::stdin()
        .lock()
        .read_line(&mut r_1xh)
        .map_err(|e| format!("đọc xác nhận lỗi: {e}"))?;
    if n == 0 {
        return Err(
            "không đọc được xác nhận (đầu vào đã đóng) — dùng --confirm nếu chạy trong script"
                .into(),
        );
    }
    Ok(r_1xh)
}
fn f_114(plan: &Thd, r_1xi: &Tq2) -> Result<bool, String> {
    println!("{plan}");
    if plan.is_empty() {
        println!(r#"
Không có gì để xoá."#);
        return Ok(false);
    }
    if !r_1xi.apply {
        println!(
            r#"
── XEM TRƯỚC ── chưa xoá gì. Thêm --apply để thi hành (sẽ hỏi câu xác nhận)."#
        );
        return Ok(false);
    }
    let expected = plan.expected_phrase();
    let r_1xk = match &r_1xi.confirm {
        Some(r_1xj) => r_1xj.clone(),
        None => f_sc(&expected)?,
    };
    if !plan.phrase_matches(&r_1xk) {
        return Err(
            format!(
                "câu xác nhận không khớp — huỷ, KHÔNG xoá gì.\n  cần: {expected}"
            ),
        );
    }
    Ok(true)
}
fn f_2az(r_1xl: &Tq2) -> Result<(), String> {
    let r_1xm = local::Tb8::from_env();
    let r_1xn = local::resolve(&r_1xm, r_1xl.storage_root.clone())?;
    let plan = local::build_plan(&r_1xn);
    if !f_114(&plan, r_1xl)? {
        return Ok(());
    }
    let rep = local::execute(&r_1xn);
    println!("\n{rep}");
    if rep.ok() {
        Ok(())
    } else {
        Err(
            "có mục không xoá được — xem danh sách lỗi ở trên"
                .into(),
        )
    }
}
async fn f_2ba(r_1xo: &Tq2) -> Result<(), String> {
    let r_1xs = f_17n()?;
    let r_1xx = digitext_db::connect(&r_1xs)
        .await
        .map_err(|e| format!("kết nối Postgres thất bại: {e}"))?;
    let r_1xw = minio_from_env()
        .map(|r_1xp| MinioObjectStore::new(&r_1xp).map(|s| (format!("MinIO {}", r_1xp.bucket), s)))
        .transpose()
        .map_err(|e| format!("dựng MinIO thất bại: {e}"))?;
    let r_1xt = std::env::var("DIGITEXT_PDF_STORE_DIR")
        .ok()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from);
    let r_1xu = r_1xt.as_ref().map(FsObjectStore::new);
    let mut r_1xy: Vec<Tdp> = Vec::new();
    if let Some((r_1xv, s)) = &r_1xw {
        r_1xy.push(Tdp {
            label: r_1xv.clone(),
            store: s,
        });
    }
    if let (Some(r_1xq), Some(s)) = (&r_1xt, &r_1xu) {
        r_1xy.push(Tdp {
            label: format!("thư mục {}", r_1xq.display()),
            store: s,
        });
    }
    let r_1xr = std::env::var("DIGITEXT_DATA_DIR")
        .ok()
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .or_else(|| Some(PathBuf::from("data")))
        .filter(|p| p.exists());
    let r_1xz = server::Tl5 {
        dsn: r_1xs,
        pool: &r_1xx,
        stores: r_1xy,
        data_dir: r_1xr,
    };
    let plan = server::build_plan(&r_1xz, r_1xo.depth).await;
    if !f_114(&plan, r_1xo)? {
        return Ok(());
    }
    let rep = server::execute(&r_1xz, r_1xo.depth).await;
    println!("\n{rep}");
    if rep.ok() {
        Ok(())
    } else {
        Err(
            "có mục không xoá được — xem danh sách lỗi ở trên"
                .into(),
        )
    }
}
pub fn cli_main(prog: &str, r_1y1: &[String]) -> ExitCode {
    if r_1y1.iter().any(|r_1y0| r_1y0 == "--help" || r_1y0 == "-h") {
        print!("{}", help_text(prog));
        return ExitCode::SUCCESS;
    }
    let r_1y2 = match parse_args(r_1y1) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{e}\n");
            eprint!("{}", help_text(prog));
            return ExitCode::from(2);
        }
    };
    let r_1y3 = match r_1y2.scope {
        Scope::Local => f_2az(&r_1y2),
        Scope::Server => {
            match tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
            {
                Ok(r_1y4) => r_1y4.block_on(f_2ba(&r_1y2)),
                Err(e) => Err(format!(
                    "không tạo được tokio runtime: {e}"
                )),
            }
        }
    };
    match r_1y3 {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("\n❌ {e}");
            ExitCode::FAILURE
        }
    }
}
