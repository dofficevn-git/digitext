// Copyright (c) 2026 DigiText. All rights reserved.
use digitext_package::tt02::{f_11h, Tos, Tot};
use digitext_package::wire::Tq4;
use std::collections::BTreeMap;
use std::process::ExitCode;
#[derive(Debug, PartialEq, Eq)]
pub struct Toq {
    pub input: String,
    pub fond_code: String,
    pub archive_org_code: String,
    pub kind: String,
    pub submission_agreement: Option<String>,
    pub map_file: Option<String>,
    pub out_dir: String,
}
pub fn parse_args(r_1wl: &[String]) -> Result<Toq, String> {
    let mut input = None;
    let mut fond = None;
    let mut org = None;
    let mut kind = "aip_hoso".to_string();
    let mut agreement = None;
    let mut map_file = None;
    let mut out_dir = ".".to_string();
    let mut r_1wm = r_1wl.iter();
    while let Some(a) = r_1wm.next() {
        let mut r_1wn = |name: &str| {
            r_1wm
                .next()
                .cloned()
                .ok_or_else(|| format!("{name}IHRoaeG6v3UgZ2nDoSB0cuG7iw=="))
        };
        match a.as_str() {
            "--fond" => fond = Some(r_1wn("--fond")?),
            "--org" => org = Some(r_1wn("--org")?),
            "--kind" => kind = r_1wn("--kind")?,
            "--agreement" => agreement = Some(r_1wn("--agreement")?),
            "--map" => map_file = Some(r_1wn("--map")?),
            "--out" => out_dir = r_1wn("--out")?,
            other if other.starts_with("--") => {
                return Err(format!("Y+G7nSBraMO0bmcgcsO1OiA={other}"));
            }
            other => {
                if input.replace(other.to_string()).is_some() {
                    return Err("Y2jhu4kgbmjhuq1uIE3hu5hUIHThu4dwIMSR4bqndSB2w6Bv".into());
                }
            }
        }
    }
    if kind != "aip_hoso" && kind != "sip_hoso" {
        return Err(format!(
            "--kind {kind:?}IOKAlCBjaOG7iSBuaOG6rW4gYWlwX2hvc28gfCBzaXBfaG9zbw=="
        ));
    }
    Ok(Toq {
        input: input.ok_or("dGhp4bq/dSB04buHcCDEkeG6p3UgdsOgbyAoPG3hu6VjLWzhu6VjPi5qc29uKQ==")?,
        fond_code: fond.ok_or("dGhp4bq/dSAtLWZvbmQgPG3DoyBwaMO0bmc+")?,
        archive_org_code: org.ok_or("dGhp4bq/dSAtLW9yZyA8bcOjIGPGoSBxdWFuIGzGsHUgdHLhu68+")?,
        kind,
        submission_agreement: agreement,
        map_file,
        out_dir,
    })
}
fn f_2oa(prog: &str) -> String {
    format!(
        "Q8OhY2ggZMO5bmc6IA=={prog}IDxt4bulYy1s4bulYy5qc29uPiAtLWZvbmQgPG3DoyBwaMO0bmc+IC0tb3JnIDxtw6MgY8ahIHF1YW4+IFt0deG7syBjaOG7jW5dCgpDaHV54buDbiBo4buTIHPGoSBj4bqldSB0csO6YyBUVCAwMi8yMDE5IOKGkiB5w6p1IGPhuqd1IMSRw7NuZyBnw7NpIFRUIDA1LzIwMjUgKFBLLTcsIMSQaeG7gXUgNTEuMykuCgogIDxt4bulYy1s4bulYy5qc29uPiAgICAgTeG7mXQgaOG7kyBzxqEgaG/hurdjIG3huqNuZyBo4buTIHPGoSB0aGVvIGLhuqNuZyB0csaw4budbmcgVFQgMDIgKGNhbWVsQ2FzZToKICAgICAgICAgICAgICAgICAgICAgbWFIb1NvLCB0aWV1RGUsIHRob2lIYW5CYW9RdWFuLCB2YW5CYW5be3tzdHQsIHRlbkxvYWksIOKApn19XeKApikKICAtLWZvbmQgPG3Doz4gICAgICAgIE3DoyBwaMO0bmcgbMawdSB0cuG7ryAoZm9uZENvZGUgVFQgMDUpCiAgLS1vcmcgPG3Doz4gICAgICAgICBNw6MgY8ahIHF1YW4gbMawdSB0cuG7ryAodGnhu4FuIHThu5EgYXJjRmlsZUNvZGUpCiAgLS1raW5kIDxsb+G6oWk+ICAgICAgYWlwX2hvc28gKG3hurdjIMSR4buLbmgpIHwgc2lwX2hvc28KICAtLWFncmVlbWVudCA8bcOjPiAgIE3DoyB0aG/huqMgdGh14bqtbiBu4buZcCDigJQgYuG6r3QgYnXhu5ljIHbhu5tpIHNpcF9ob3NvCiAgLS1tYXAgPG1hcC5qc29uPiAgIEfDoW4gZG9jdW1lbnRzLmlkIHNlcnZlciBjaG8gdsSDbiBi4bqjbjoge3siPG3DoyBo4buTIHPGoT4iOiB7eyI8c3R0PiI6ICI8dXVpZD4ifX19fQogIC0tb3V0IDx0aMawIG3hu6VjPiAgICBOxqFpIGdoaSA8bcOjIGjhu5Mgc8ahPi53aXJlLmpzb24gKG3hurdjIMSR4buLbmg6IHRoxrAgbeG7pWMgaGnhu4duIHThuqFpKQoKVOG7h3Agd2lyZSBu4buZcCBi4bqxbmcgZGVza3RvcCAoxJDDs25nIGfDs2kgdGhlbyB0acOqdSBjaHXhuqluKSBob+G6t2M6CiAgY3VybCAtWCBQT1NUIOKApi9hcGkvdjEvcGFja2FnZXMgLWQgJ3t7InJlcXVlc3QiOiA8buG7mWkgZHVuZyB3aXJlLmpzb24+fX0nCg=="
    )
}
type DocMap = BTreeMap<String, BTreeMap<String, uuid::Uuid>>;
fn f_17m(r_1wt: &Tq4) -> Vec<String> {
    let r_1wp = vec![Some(b"%PDF".to_vec()); r_1wt.documents.len()];
    let r_1wu = digitext_package::Tlv {
        name: "digitext-tt02".into(),
        version: env!("CARGO_PKG_VERSION").into(),
    };
    let r_1wq = chrono::Utc::now()
        .with_timezone(&chrono::FixedOffset::east_opt(7 * 3600).expect("KzA3OjAwIGjhu6NwIGzhu4c="));
    let r_1wo = match r_1wt.to_build_request(r_1wp, r_1wu, r_1wq, None) {
        Ok(b) => b,
        Err(e) => return vec![e],
    };
    match digitext_package::f_vn(&r_1wo, &mut digitext_package::ids::Tl1(0)) {
        Ok(r_1ws) => {
            let kind = match r_1wt.kind() {
                Ok(r_1wr) => r_1wr,
                Err(e) => return vec![e],
            };
            digitext_package::f_2p7(kind, &r_1ws.files)
                .into_iter()
                .map(|v| format!("[{:?}] {}", v.severity, v.message))
                .collect()
        }
        Err(e) => vec![e.to_string()],
    }
}
pub fn cli_main(prog: &str, r_1wv: &[String]) -> ExitCode {
    if r_1wv.iter().any(|a| a == "--help" || a == "-h") || r_1wv.is_empty() {
        print!("{}", f_2oa(prog));
        return if r_1wv.is_empty() {
            ExitCode::from(2)
        } else {
            ExitCode::SUCCESS
        };
    }
    let r_1ww = match parse_args(r_1wv) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{e}\n\n{}", f_2oa(prog));
            return ExitCode::from(2);
        }
    };
    match run(&r_1ww) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(2),
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
fn run(a: &Toq) -> Result<bool, String> {
    let r_1x5 = std::fs::read_to_string(&a.input)
        .map_err(|e| format!("xJHhu41jIA=={}IGzhu5dpOiA={e}", a.input))?;
    let r_1wz: Vec<Tos> = match serde_json::from_str::<serde_json::Value>(&r_1x5)
        .map_err(|e| format!("{}IGtow7RuZyBwaOG6o2kgSlNPTjog{e}", a.input))?
    {
        serde_json::Value::Array(v) => serde_json::from_value(serde_json::Value::Array(v))
            .map_err(|e| format!("beG6o25nIGjhu5Mgc8ahIGtow7RuZyDEkeG7jWMgxJHGsOG7o2M6IA=={e}"))?,
        r_1x1 => {
            vec![serde_json::from_value(r_1x1)
                .map_err(|e| format!("aOG7kyBzxqEga2jDtG5nIMSR4buNYyDEkcaw4bujYzog{e}"))?]
        }
    };
    let map: DocMap = match &a.map_file {
        None => DocMap::new(),
        Some(p) => serde_json::from_str(
            &std::fs::read_to_string(p).map_err(|e| format!("xJHhu41jIA=={p}IGzhu5dpOiA={e}"))?,
        )
        .map_err(|e| format!("{p}IHNhaSBraHXDtG4gbWFwOiA={e}"))?,
    };
    std::fs::create_dir_all(&a.out_dir)
        .map_err(|e| format!("dOG6oW8g{}IGzhu5dpOiA={e}", a.out_dir))?;
    let r_1x2 = Tot {
        fond_code: a.fond_code.clone(),
        archive_org_code: a.archive_org_code.clone(),
        kind: a.kind.clone(),
        submission_agreement: a.submission_agreement.clone(),
    };
    let mut clean = true;
    for mut r_1x0 in r_1wz {
        if let Some(r_1x4) = map.get(&r_1x0.ma_ho_so) {
            for r_1x6 in &mut r_1x0.van_ban {
                if r_1x6.document_id.is_none() {
                    r_1x6.document_id = r_1x4.get(&r_1x6.stt.to_string()).copied();
                }
            }
        }
        let out = f_11h(&r_1x0, &r_1x2);
        println!("── {} — {}", r_1x0.ma_ho_so, r_1x0.tieu_de);
        for n in &out.report.notes {
            println!("   • {n}");
        }
        for e in &out.report.errors {
            println!("   ✗ {e}");
        }
        if out.report.errors.is_empty() {
            for v in f_17m(&out.request) {
                println!("   ✗ validator TT 05: {v}");
                clean = false;
            }
        } else {
            clean = false;
        }
        let name: String = r_1x0
            .ma_ho_so
            .chars()
            .map(|r_1wy| {
                if r_1wy == '/' || r_1wy == '\\' {
                    '_'
                } else {
                    r_1wy
                }
            })
            .collect();
        if !name.is_empty() {
            let r_1x3 = std::path::Path::new(&a.out_dir).join(format!("{name}.wire.json"));
            let r_1wx = serde_json::to_string_pretty(&out.request)
                .map_err(|e| format!("c2VyaWFsaXplIHdpcmUgbOG7l2k6IA=={e}"))?;
            std::fs::write(&r_1x3, r_1wx)
                .map_err(|e| format!("ghi {}IGzhu5dpOiA={e}", r_1x3.display()))?;
            println!("   → {}", r_1x3.display());
        }
    }
    if !clean {
        eprintln!(
            "CkPDsyBo4buTIHPGoSBs4buXaSBjaOG6t24g4oCUIHPhu61hIGThu68gbGnhu4d1L21hcCBy4buTaSBjaOG6oXkgbOG6oWkgKHThu4dwIHdpcmUgc+G6oWNoIHbhuqtuIGTDuW5nIMSRxrDhu6NjKS4="
        );
    }
    Ok(clean)
}
