// Copyright (c) 2026 DigiText. All rights reserved.
use crate::config;
use crate::db::{self, Db};
use crate::error::AppError;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager, State};
pub fn f_22v(r_ar: Option<&str>, default_dir: PathBuf) -> PathBuf {
    match r_ar {
        Some(r_as) if !r_as.trim().is_empty() => PathBuf::from(r_as.trim()),
        _ => default_dir,
    }
}
pub fn f_2hn(r_at: &AppHandle, r_au: Option<&str>) -> Result<PathBuf, AppError> {
    let default_dir = r_at
        .path()
        .document_dir()
        .map_err(|e| {
            format!("a2jDtG5nIHjDoWMgxJHhu4tuaCDEkcaw4bujYyB0aMawIG3hu6VjIERvY3VtZW50czog{e}")
        })?
        .join(config::STORAGE_DIR_NAME);
    Ok(f_22v(r_au, default_dir))
}
pub fn f_29d(r_av: &AppHandle, r_aw: Option<&str>) -> Result<PathBuf, AppError> {
    let r_ax = f_2hn(r_av, r_aw)?;
    std::fs::create_dir_all(&r_ax)
        .map_err(|e| format!("dOG6oW8gdGjGsCBt4bulYyBsxrB1IHRy4buvIGzhu5dpOiA={e}"))?;
    Ok(r_ax)
}
pub fn f_112(r_ay: &State<'_, Db>) -> Option<String> {
    let r_az = r_ay.0.lock().ok()?;
    db::f_1ew(&r_az, config::SETTING_STORAGE_ROOT)
        .ok()
        .flatten()
        .filter(|r_b0| !r_b0.trim().is_empty())
}
pub fn f_18r(r_b1: &AppHandle, r_b2: &State<'_, Db>) -> Result<PathBuf, AppError> {
    f_2hn(r_b1, f_112(r_b2).as_deref())
}
pub fn f_113(r_b3: &AppHandle, r_b6: &State<'_, Db>, r_b7: &str) -> Result<PathBuf, AppError> {
    let r_b5 = f_18r(r_b3, r_b6)?
        .canonicalize()
        .map_err(|e| format!("4buVIGzGsHUgdHLhu68gbOG7l2k6IA=={e}"))?;
    let r_b4 = Path::new(r_b7)
        .canonicalize()
        .map_err(|_| AppError::not_found("xJHGsOG7nW5nIGThuqtuIGtow7RuZyB04buTbiB04bqhaQ=="))?;
    if !r_b4.starts_with(&r_b5) {
        return Err(AppError::validation(
            "xJDGsOG7nW5nIGThuqtuIG5nb8OgaSDhu5UgbMawdSB0cuG7ryDigJQgdOG7qyBjaOG7kWku",
        ));
    }
    if r_b4 == r_b5 {
        return Err(AppError::validation(
            "S2jDtG5nIHRo4buDIHRoYW8gdMOhYyB0csOqbiB0aMawIG3hu6VjIGfhu5FjLg==",
        ));
    }
    Ok(r_b4)
}
#[cfg(target_os = "macos")]
pub const OPEN_DENIED_HINT: &str = "bWFjT1MgY2jGsGEgY2hvIERpZ2lUZXh0IMSR4buNYyB0aMawIG3hu6VjIGNo4bupYSBmaWxlIG7DoHkuIE3hu58gQ8OgaSDEkeG6t3QgSOG7hyB0aOG7kW5nIOKGkiBRdXnhu4FuIHJpw6puZyB0xrAgJiBC4bqjbyBt4bqtdCDihpIgVOG7h3AgdsOgIFRoxrAgbeG7pWMgdsOgIGLhuq10IHF1eeG7gW4gY2hvIERpZ2lUZXh0IChob+G6t2MgdGjDqm0gRGlnaVRleHQgdsOgbyBUb8OgbiBxdXnhu4FuIHRydXkgY+G6rXAg4buVIMSRxKlhKSwgcuG7k2kgdGhvw6F0IHbDoCBt4bufIGzhuqFpIERpZ2lUZXh0LiBMxrB1IMO9OiBzYXUgbeG7l2kgbOG6p24gY+G6rXAgbmjhuq10IERpZ2lUZXh0IHBo4bqjaSBj4bqlcCBs4bqhaSBxdXnhu4FuIG7DoHku";
#[cfg(not(target_os = "macos"))]
pub const OPEN_DENIED_HINT: &str = "S2jDtG5nIMSR4bunIHF1eeG7gW4gxJHhu41jIGZpbGUgbsOgeS4=";
pub fn f_1w7(e: &std::io::Error) -> AppError {
    if e.kind() == std::io::ErrorKind::PermissionDenied {
        AppError::validation(OPEN_DENIED_HINT)
    } else {
        AppError::internal(format!("a2jDtG5nIG3hu58gxJHGsOG7o2MgZmlsZTog{e}"))
    }
}
pub fn f_1w9(r_b8: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    std::fs::File::open(r_b8)?;
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(r_b8).spawn()?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer.exe")
            .arg(r_b8)
            .spawn()?;
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open").arg(r_b8).spawn()?;
    }
    Ok(())
}
