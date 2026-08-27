// Copyright (c) 2026 DigiText. All rights reserved.
use crate::{Depth, PlanItem, Report, Scope, Thd};
use std::path::{Component, Path, PathBuf};
pub const C_5: &str = "vn.digitext.desktop";
pub const DB_FILE: &str = "digitext.db";
pub const STORAGE_DIR_NAME: &str = "DigiText";
pub const SETTING_STORAGE_ROOT: &str = "storage_root";
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    MacOs,
    Windows,
    Linux,
}
impl Os {
    pub fn current() -> Os {
        if cfg!(target_os = "macos") {
            Os::MacOs
        } else if cfg!(target_os = "windows") {
            Os::Windows
        } else {
            Os::Linux
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tb8 {
    pub os: Os,
    pub home: Option<PathBuf>,
    pub appdata: Option<PathBuf>,
    pub local_appdata: Option<PathBuf>,
    pub xdg_data_home: Option<PathBuf>,
    pub temp: PathBuf,
}
impl Tb8 {
    pub fn from_env() -> Tb8 {
        let r_5ox = |r_5ov: &str| {
            std::env::var_os(r_5ov)
                .map(PathBuf::from)
                .filter(|r_5ow| !r_5ow.as_os_str().is_empty())
        };
        Tb8 {
            os: Os::current(),
            home: r_5ox("HOME").or_else(|| r_5ox("USERPROFILE")),
            appdata: r_5ox("APPDATA"),
            local_appdata: r_5ox("LOCALAPPDATA"),
            xdg_data_home: r_5ox("XDG_DATA_HOME"),
            temp: std::env::temp_dir(),
        }
    }
    pub fn app_data_dir(&self) -> Option<PathBuf> {
        match self.os {
            Os::MacOs => Some(
                self.home
                    .as_ref()?
                    .join("Library/Application Support")
                    .join(C_5),
            ),
            Os::Windows => Some(self.appdata.as_ref()?.join(C_5)),
            Os::Linux => Some(match &self.xdg_data_home {
                Some(x) => x.join(C_5),
                None => self.home.as_ref()?.join(".local/share").join(C_5),
            }),
        }
    }
    pub fn log_dir(&self) -> Option<PathBuf> {
        match self.os {
            Os::MacOs => Some(self.home.as_ref()?.join("Library/Logs").join(C_5)),
            Os::Windows => Some(self.local_appdata.as_ref()?.join(C_5).join("logs")),
            Os::Linux => Some(self.app_data_dir()?.join("logs")),
        }
    }
    pub fn default_storage_root(&self) -> Option<PathBuf> {
        Some(self.home.as_ref()?.join("Documents").join(STORAGE_DIR_NAME))
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootSource {
    Override,
    Db,
    Default,
    DefaultDbUnreadable,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tba {
    pub app_data_dir: PathBuf,
    pub log_dir: Option<PathBuf>,
    pub storage_root: PathBuf,
    pub root_source: RootSource,
    pub page_cache: PathBuf,
    pub temp_dir: PathBuf,
}
const C_MI: &[&str] = &[
    "digitext-pages",
    "digitext-scan-",
    "digitext-wia",
    "digitext-twain",
    "digitext-export-",
    "digitext-prune-",
];
pub fn f_1iy(r_5p1: &Path, home: Option<&Path>) -> bool {
    if r_5p1.as_os_str().is_empty() {
        return true;
    }
    if let Some(r_5p0) = home {
        if r_5p1 == r_5p0 {
            return true;
        }
    }
    let r_5oz = r_5p1
        .components()
        .filter(|r_5oy| matches!(r_5oy, Component::Normal(_)))
        .count();
    r_5oz <= 1
}
pub fn f_25w(r_5p4: &Path) -> Option<PathBuf> {
    if !r_5p4.exists() {
        return None;
    }
    let r_5p7 = f_2n8()?;
    let r_5p3 = r_5p7.join(DB_FILE);
    let r_5p6 = (|| {
        std::fs::copy(r_5p4, &r_5p3).ok()?;
        let wal = f_2r5(r_5p4, "-wal");
        if wal.exists() {
            std::fs::copy(&wal, f_2r5(&r_5p3, "-wal")).ok()?;
        }
        let r_5p2 = rusqlite::Connection::open_with_flags(
            &r_5p3,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE,
        )
        .ok()?;
        let v: String = r_5p2
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                [SETTING_STORAGE_ROOT],
                |r_5p5| r_5p5.get(0),
            )
            .ok()?;
        let v = v.trim();
        (!v.is_empty()).then(|| PathBuf::from(v))
    })();
    let _ = std::fs::remove_dir_all(&r_5p7);
    r_5p6
}
fn f_2n8() -> Option<PathBuf> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let r_5p8 = std::env::temp_dir();
    for _ in 0..256 {
        let n = SEQ.fetch_add(1, Ordering::Relaxed);
        let r_5p9 = r_5p8.join(format!("digitext-wipe-doc-{}-{n}", std::process::id()));
        match std::fs::create_dir(&r_5p9) {
            Ok(()) => return Some(r_5p9),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(_) => return None,
        }
    }
    None
}
fn f_2r5(r_5pa: &Path, r_5pc: &str) -> PathBuf {
    let mut r_5pb = r_5pa.as_os_str().to_owned();
    r_5pb.push(r_5pc);
    PathBuf::from(r_5pb)
}
pub fn resolve(r_5pd: &Tb8, r_5pg: Option<PathBuf>) -> Result<Tba, String> {
    let app_data_dir = r_5pd
        .app_data_dir()
        .ok_or(
            "a2jDtG5nIHjDoWMgxJHhu4tuaCDEkcaw4bujYyB0aMawIG3hu6VjIGThu68gbGnhu4d1IGFwcCAodGhp4bq/dSBIT01FL0FQUERBVEEp",
        )?;
    let db = app_data_dir.join(DB_FILE);
    let (storage_root, root_source) = match r_5pg {
        Some(p) => (p, RootSource::Override),
        None => match f_25w(&db) {
            Some(p) => (p, RootSource::Db),
            None => {
                let r_5pe = r_5pd
                        .default_storage_root()
                        .ok_or(
                            "a2jDtG5nIHjDoWMgxJHhu4tuaCDEkcaw4bujYyBraG8gdMOgaSBsaeG7h3UgKHRoaeG6v3UgSE9NRSkg4oCUIGTDuW5nIC0tc3RvcmFnZS1yb290",
                        )?;
                let r_5pf = if db.exists() {
                    RootSource::DefaultDbUnreadable
                } else {
                    RootSource::Default
                };
                (r_5pe, r_5pf)
            }
        },
    };
    if f_1iy(&storage_root, r_5pd.home.as_deref()) {
        return Err(
            format!(
                "a2hvIHTDoGkgbGnhu4d1IA=={}IG7hurFtIOG7nyB24buLIHRyw60gcXXDoSBuZ3V5IGhp4buDbSDEkeG7gyB4b8OhIChn4buRYyDhu5UgxJHEqWEgLyB0aMawIG3hu6VjIG5ow6ApIOKAlCBraeG7g20gdHJhIGzhuqFpIHNldHRpbmdzLnN0b3JhZ2Vfcm9vdCBob+G6t2MgY2jhu4kgxJHhu4tuaCAtLXN0b3JhZ2Utcm9vdA==",
                storage_root.display()
            ),
        );
    }
    Ok(Tba {
        app_data_dir,
        log_dir: r_5pd.log_dir(),
        storage_root,
        root_source,
        page_cache: r_5pd.temp.join("digitext-pages"),
        temp_dir: r_5pd.temp.clone(),
    })
}
fn measure(r_5pj: &Path) -> (u64, u64) {
    let (mut n, mut r_5ph) = (0u64, 0u64);
    let mut r_5pm = vec![r_5pj.to_path_buf()];
    while let Some(r_5pi) = r_5pm.pop() {
        let Ok(r_5pl) = std::fs::read_dir(&r_5pi) else {
            continue;
        };
        for e in r_5pl.flatten() {
            match e.file_type() {
                Ok(r_5pn) if r_5pn.is_dir() => r_5pm.push(e.path()),
                Ok(_) => {
                    n += 1;
                    r_5ph += e.metadata().map(|r_5pk| r_5pk.len()).unwrap_or(0);
                }
                Err(_) => {}
            }
        }
    }
    (n, r_5ph)
}
fn f_2jk(temp: &Path) -> Vec<PathBuf> {
    let Ok(r_5pq) = std::fs::read_dir(temp) else {
        return Vec::new();
    };
    r_5pq
        .flatten()
        .filter(|e| {
            let r_5po = e.file_name().to_string_lossy().into_owned();
            r_5po != "digitext-pages" && C_MI.iter().any(|r_5pp| r_5po.starts_with(r_5pp))
        })
        .map(|e| e.path())
        .collect()
}
pub fn build_plan(r_5q6: &Tba) -> Thd {
    let mut r_5px = Vec::new();
    let (r_5q0, r_5ps) = measure(&r_5q6.app_data_dir);
    r_5px.push(
        PlanItem::new(
            "Q1NETCArIGThu68gbGnhu4d1IGFwcA==",
            r_5q6.app_data_dir.display().to_string(),
            r_5q0,
        )
        .with_bytes(r_5ps),
    );
    let (r_5q2, r_5pu) = measure(&r_5q6.storage_root);
    r_5px.push(
        PlanItem::new(
            "S2hvIHTDoGkgbGnhu4d1IChQREYv4bqjbmgvZ8OzaSk=",
            r_5q6.storage_root.display().to_string(),
            r_5q2,
        )
        .with_bytes(r_5pu),
    );
    let (r_5q1, r_5pt) = measure(&r_5q6.page_cache);
    r_5px.push(
        PlanItem::new(
            "Q2FjaGUg4bqjbmggcXXDqXQ=",
            r_5q6.page_cache.display().to_string(),
            r_5q1,
        )
        .with_bytes(r_5pt),
    );
    let temp = f_2jk(&r_5q6.temp_dir);
    let (mut r_5q4, mut r_5pw) = (0u64, 0u64);
    for r_5q5 in &temp {
        if r_5q5.is_dir() {
            let (n, r_5pr) = measure(r_5q5);
            r_5q4 += n;
            r_5pw += r_5pr;
        } else {
            r_5q4 += 1;
            r_5pw += r_5q5.metadata().map(|r_5pz| r_5pz.len()).unwrap_or(0);
        }
    }
    r_5px.push(
        PlanItem::new(
            "VOG6oW0gbcOheSBxdcOpdCAvIHh14bqldCBmaWxl",
            format!("{}/digitext-*", r_5q6.temp_dir.display()),
            r_5q4,
        )
        .with_bytes(r_5pw),
    );
    if let Some(r_5py) = &r_5q6.log_dir {
        let (r_5q3, r_5pv) = measure(r_5py);
        r_5px.push(
            PlanItem::new(
                "Tmjhuq10IGvDvSDhu6luZyBk4bulbmc=",
                r_5py.display().to_string(),
                r_5q3,
            )
            .with_bytes(r_5pv),
        );
    }
    let mut r_5q7 = Vec::new();
    if r_5q6.root_source == RootSource::DefaultDbUnreadable {
        r_5q7
            .push(
                format!(
                    "Q8OTIA=={DB_FILE}IG5oxrBuZyBLSMOUTkcgxJHhu41jIMSRxrDhu6NjIGBzZXR0aW5ncy5zdG9yYWdlX3Jvb3RgIHThu6sgxJHDsyDigJQgxJFhbmcgxJBPw4FOIGtobyBsw6Ag{}LiBO4bq/dSBuZ8aw4budaSBkw7luZyB04burbmcgY2jhu41uIGtobyDhu58g4buVIG5nb8OgaS/hu5UgbeG6oW5nIHRow6wgbOG7h25oIG7DoHkgbmjhuq9tIFNBSSBjaOG7lyB2w6AgaOG7kyBzxqEgdGjhuq10IHbhuqtuIGPDsm4gbmd1ecOqbjsga2nhu4NtIHRyYSBy4buTaSBjaOG7iSDEkeG7i25oIC0tc3RvcmFnZS1yb290Lg==",
                    r_5q6.storage_root.display()
                ),
            );
    }
    r_5q7
        .extend([
            "S2jDtG5nIGhvw6BuIHTDoWMgxJHGsOG7o2MuIEjhu5Mgc8ahIMSRw6MgcXXDqXQgbcOgIENIxq9BIG7hu5lwIGzDqm4gbcOheSBjaOG7pyBz4bq9IG3huqV0IHbEqW5oIHZp4buFbi4="
                .to_owned(),
            "R+G7k20gY+G6oyDEkcSDbmcgbmjhuq1wIG9mZmxpbmUsIGtob8OhIHRoaeG6v3QgYuG7iyBMQU4gdsOgIG3DoyBzaXRlIOKAlCBzYXUga2hpIHhvw6EgcGjhuqNpIMSRxINuZyBuaOG6rXAgbOG6oWkga2hpIGPDsyBt4bqhbmcgxJHhu4MgbOG6pXkgZ2nhuqV5IHBow6lwIG9mZmxpbmUgbeG7m2ku"
                .to_owned(),
            "RmlsZSDEkcOjICdMxrB1IHJh4oCmJyBi4bqxbmcgaOG7mXAgdGhv4bqhaSBo4buHIMSRaeG7gXUgaMOgbmggbuG6sW0gbmdvw6BpIGtobyBuw6puIEtIw5RORyBi4buLIHhvw6Eg4oCUIHThu7Ega2nhu4NtIHRyYSBu4bq/dSBj4bqnbi4="
                .to_owned(),
        ]);
    Thd {
        scope: Scope::Local,
        depth: Depth::All,
        target_id: r_5q6.storage_root.display().to_string(),
        items: r_5px,
        warnings: r_5q7,
    }
}
fn remove_contents(r_5q8: &Path, rep: &mut Report) {
    let Ok(r_5qc) = std::fs::read_dir(r_5q8) else {
        return;
    };
    for e in r_5qc.flatten() {
        let r_5qa = e.path();
        let r_5q9 = e.file_type().map(|r_5qd| r_5qd.is_dir()).unwrap_or(false);
        let (n, _) = if r_5q9 { measure(&r_5qa) } else { (1, 0) };
        let r_5qb = if r_5q9 {
            std::fs::remove_dir_all(&r_5qa)
        } else {
            std::fs::remove_file(&r_5qa)
        };
        match r_5qb {
            Ok(()) => rep.removed += n,
            Err(e) => rep.errors.push(format!("eG/DoSA={}: {e}", r_5qa.display())),
        }
    }
}
pub fn execute(r_5qh: &Tba) -> Report {
    let mut rep = Report::default();
    remove_contents(&r_5qh.app_data_dir, &mut rep);
    remove_contents(&r_5qh.storage_root, &mut rep);
    match std::fs::remove_dir_all(&r_5qh.page_cache) {
        Ok(()) => rep.removed += 1,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => rep
            .errors
            .push(format!("eG/DoSA={}: {e}", r_5qh.page_cache.display())),
    }
    for r_5qf in f_2jk(&r_5qh.temp_dir) {
        let r_5qg = if r_5qf.is_dir() {
            std::fs::remove_dir_all(&r_5qf)
        } else {
            std::fs::remove_file(&r_5qf)
        };
        match r_5qg {
            Ok(()) => rep.removed += 1,
            Err(e) => rep.errors.push(format!("eG/DoSA={}: {e}", r_5qf.display())),
        }
    }
    if let Some(r_5qe) = &r_5qh.log_dir {
        remove_contents(r_5qe, &mut rep);
    }
    rep
}
