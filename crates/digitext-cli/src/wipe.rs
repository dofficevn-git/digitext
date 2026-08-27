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
        "{prog}IOKAlCB4b8OhIHPhuqFjaCBk4buvIGxp4buHdSBEaWdpVGV4dCAoS0jDlE5HIGhvw6BuIHTDoWMgxJHGsOG7o2MpCgpDw6FjaCBkw7luZzoKICA={prog}IC0tbG9jYWwgIFstLXN0b3JhZ2Utcm9vdCA8xJHGsOG7nW5nLWThuqtuPl0gICBNw6F5IG5nxrDhu51pIGTDuW5nIC8gbcOheSBhZG1pbiAoYXBwIGRlc2t0b3ApCiAg{prog}IC0tc2VydmVyIFstLWFsbF0gICAgICAgICAgICAgICAgICAgICAgICBNw6F5IGNo4bunOiBQb3N0Z3JlcyArIGtobyBvYmplY3QgKyDEkcSpYQoKVHXhu7MgY2jhu41uOgogIC0tYXBwbHkgICAgICAgICAgICBUaGkgaMOgbmggdGjhuq10LiBLSMOUTkcgY8OzIGPhu50gbsOgeSA9IGNo4buJIElOIEvhur4gSE/huqBDSCAobeG6t2MgxJHhu4tuaCBhbiB0b8OgbikKICAtLWNvbmZpcm0gPGPDonU+ICAgIEPhuqVwIHPhurVuIGPDonUgeMOhYyBuaOG6rW4gKGNo4bqheSB0cm9uZyBzY3JpcHQpOyBraMO0bmcgY8OzIOKHkiBo4buPaSB0csOqbiBiw6BuIHBow61tCiAgLS1hbGwgICAgICAgICAgICAgIChjaOG7iSB24bubaSAtLXNlcnZlcikgeG/DoSBj4bqjIHTDoGkga2hv4bqjbiwgZOG7sSDDoW4sIGRhbmggbeG7pWMg4oCUIHNhdSDEkcOzIHBo4bqjaQogICAgICAgICAgICAgICAgICAgICBjaOG6oXkgbOG6oWkgYGRpZ2l0ZXh0IGFwaSBib290c3RyYXAtc3VwZXJhZG1pbmAKICAtLXN0b3JhZ2Utcm9vdCA8cD4gKGNo4buJIHbhu5tpIC0tbG9jYWwpIGNo4buJIMSR4buLbmgga2hvIHTDoGkgbGnhu4d1IHRoYXkgdsOsIMSR4buNYyB04burIHNldHRpbmdzCiAgLS1oZWxwIC8gLWggICAgICAgIEluIHRy4bujIGdpw7pwIG7DoHkKCkJp4bq/biBtw7RpIHRyxrDhu51uZyAodOG6p25nIC0tc2VydmVyKTogREFUQUJBU0VfVVJMIGhv4bq3YyBESUdJVEVYVF9EQVRBQkFTRV9VUkwgwrcgRElHSVRFWFRfREFUQV9ESVIgwrcKRElHSVRFWFRfUERGX1NUT1JFX0RJUiDCtyBESUdJVEVYVF9NSU5JT18qICjEkeG7pyA0IGtob8OhIG3hu5tpIGLhuq10KS4KClF1eSB0csOsbmg6IGThu7FuZyBr4bq/IGhv4bqhY2ggKMSR4bq/bSwga2jDtG5nIMSR4bulbmcgZOG7ryBsaeG7h3UpIOKGkiBi4bqhbiBkdXnhu4d0IOKGkiBnw7UgTkdVWcOKTiBWxIJOIGPDonUgeMOhYyBuaOG6rW4KKGPDonUgZ+G6r24gduG7m2kgdMOqbiBDU0RMIC8gxJHGsOG7nW5nIGThuqtuIGtobywgbsOqbiBjw6J1IGPhu6dhIG3DoXkgdGjhu60ga2jDtG5nIGNo4bqheSDEkcaw4bujYyB0csOqbiBtw6F5IHRo4bqtdCkuCg=="
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
                        .ok_or("LS1jb25maXJtIGPhuqduIG3hu5l0IGPDonUgxJFpIGvDqG0=")?
                        .clone(),
                );
            }
            "--storage-root" => {
                storage_root =
                    Some(PathBuf::from(r_1xb.next().ok_or(
                        "LS1zdG9yYWdlLXJvb3QgY+G6p24gxJHGsOG7nW5nIGThuqtuIMSRaSBrw6ht",
                    )?));
            }
            other => {
                return Err(format!("dGhhbSBz4buRIGtow7RuZyBo4bujcCBs4buHOiA={other}"));
            }
        }
    }
    let scope = match (r_1xc, r_1xd) {
        (true, false) => Scope::Local,
        (false, true) => Scope::Server,
        (true, true) => {
            return Err(
                "Y2jhu41uIE3hu5hUIHThuqduZyBt4buXaSBs4bqnbjogLS1sb2NhbCBob+G6t2MgLS1zZXJ2ZXIgKHhvw6EgaGFpIHThuqduZyBjw7luZyBsw7pjIHRow6wgbeG7mXQgY8OidSB4w6FjIG5o4bqtbiBz4bq9IHBo4bqjaSBnw6FuaCBj4bqjIGhhaSk="
                    .into(),
            );
        }
        (false, false) => {
            return Err(
                "dGhp4bq/dSB04bqnbmcgY+G6p24geG/DoTogLS1sb2NhbCBob+G6t2MgLS1zZXJ2ZXI=".into(),
            );
        }
    };
    if all && scope != Scope::Server {
        return Err("LS1hbGwgY2jhu4kgZMO5bmcgduG7m2kgLS1zZXJ2ZXI=".into());
    }
    if storage_root.is_some() && scope != Scope::Local {
        return Err("LS1zdG9yYWdlLXJvb3QgY2jhu4kgZMO5bmcgduG7m2kgLS1sb2NhbA==".into());
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
            "Y+G6p24gREFUQUJBU0VfVVJMIChob+G6t2MgRElHSVRFWFRfREFUQUJBU0VfVVJMKQ==".to_owned()
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
        "CsSQ4buDIHRoaSBow6BuaCwgZ8O1IE5HVVnDik4gVsSCTiBjw6J1IHNhdSBy4buTaSBFbnRlciAoQ3RybC1DIMSR4buDIGh14bu3KToKCiAgICA={expected}\n"
    );
    print!("> ");
    std::io::stdout().flush().ok();
    let mut r_1xh = String::new();
    let n = std::io::stdin()
        .lock()
        .read_line(&mut r_1xh)
        .map_err(|e| format!("xJHhu41jIHjDoWMgbmjhuq1uIGzhu5dpOiA={e}"))?;
    if n == 0 {
        return Err(
            "a2jDtG5nIMSR4buNYyDEkcaw4bujYyB4w6FjIG5o4bqtbiAoxJHhuqd1IHbDoG8gxJHDoyDEkcOzbmcpIOKAlCBkw7luZyAtLWNvbmZpcm0gbuG6v3UgY2jhuqF5IHRyb25nIHNjcmlwdA=="
                .into(),
        );
    }
    Ok(r_1xh)
}
fn f_114(plan: &Thd, r_1xi: &Tq2) -> Result<bool, String> {
    println!("{plan}");
    if plan.is_empty() {
        println!("Cktow7RuZyBjw7MgZ8OsIMSR4buDIHhvw6Eu");
        return Ok(false);
    }
    if !r_1xi.apply {
        println!(
            "CuKUgOKUgCBYRU0gVFLGr+G7mkMg4pSA4pSAIGNoxrBhIHhvw6EgZ8OsLiBUaMOqbSAtLWFwcGx5IMSR4buDIHRoaSBow6BuaCAoc+G6vSBo4buPaSBjw6J1IHjDoWMgbmjhuq1uKS4="
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
                "Y8OidSB4w6FjIG5o4bqtbiBraMO0bmcga2jhu5twIOKAlCBodeG7tywgS0jDlE5HIHhvw6EgZ8OsLgogIGPhuqduOiA={expected}"
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
            "Y8OzIG3hu6VjIGtow7RuZyB4b8OhIMSRxrDhu6NjIOKAlCB4ZW0gZGFuaCBzw6FjaCBs4buXaSDhu58gdHLDqm4="
                .into(),
        )
    }
}
async fn f_2ba(r_1xo: &Tq2) -> Result<(), String> {
    let r_1xs = f_17n()?;
    let r_1xx = digitext_db::connect(&r_1xs)
        .await
        .map_err(|e| format!("a+G6v3QgbuG7kWkgUG9zdGdyZXMgdGjhuqV0IGLhuqFpOiA={e}"))?;
    let r_1xw = minio_from_env()
        .map(|r_1xp| MinioObjectStore::new(&r_1xp).map(|s| (format!("MinIO {}", r_1xp.bucket), s)))
        .transpose()
        .map_err(|e| format!("ZOG7sW5nIE1pbklPIHRo4bqldCBi4bqhaTog{e}"))?;
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
            label: format!("dGjGsCBt4bulYyA={}", r_1xq.display()),
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
            "Y8OzIG3hu6VjIGtow7RuZyB4b8OhIMSRxrDhu6NjIOKAlCB4ZW0gZGFuaCBzw6FjaCBs4buXaSDhu58gdHLDqm4="
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
                    "a2jDtG5nIHThuqFvIMSRxrDhu6NjIHRva2lvIHJ1bnRpbWU6IA=={e}"
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
