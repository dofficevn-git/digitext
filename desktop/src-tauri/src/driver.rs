// Copyright (c) 2026 DigiText. All rights reserved.
use crate::error::AppError;
use serde::Serialize;
use std::ffi::OsStr;
use std::path::Path;
use std::process::Command;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetOs {
    Windows,
    MacOs,
    Linux,
}
impl TargetOs {
    pub fn current() -> Self {
        #[cfg(target_os = "windows")]
        return TargetOs::Windows;
        #[cfg(target_os = "macos")]
        return TargetOs::MacOs;
        #[cfg(all(unix, not(target_os = "macos")))]
        return TargetOs::Linux;
    }
    fn label(self) -> &'static str {
        match self {
            TargetOs::Windows => "Windows",
            TargetOs::MacOs => "macOS",
            TargetOs::Linux => "Linux",
        }
    }
    pub fn installer_extensions(self) -> &'static [&'static str] {
        match self {
            TargetOs::Windows => &["exe", "msi"],
            TargetOs::MacOs => &["pkg", "dmg"],
            TargetOs::Linux => &["deb", "rpm"],
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallerKind {
    WindowsExe,
    WindowsMsi,
    MacPkg,
    MacDmg,
    LinuxDeb,
    LinuxRpm,
}
impl InstallerKind {
    fn target_os(self) -> TargetOs {
        match self {
            InstallerKind::WindowsExe | InstallerKind::WindowsMsi => TargetOs::Windows,
            InstallerKind::MacPkg | InstallerKind::MacDmg => TargetOs::MacOs,
            InstallerKind::LinuxDeb | InstallerKind::LinuxRpm => TargetOs::Linux,
        }
    }
    fn from_extension(ext: &str) -> Option<InstallerKind> {
        Some(match ext {
            "exe" => InstallerKind::WindowsExe,
            "msi" => InstallerKind::WindowsMsi,
            "pkg" => InstallerKind::MacPkg,
            "dmg" => InstallerKind::MacDmg,
            "deb" => InstallerKind::LinuxDeb,
            "rpm" => InstallerKind::LinuxRpm,
            _ => return None,
        })
    }
    fn is_manual(self) -> bool {
        matches!(self, InstallerKind::MacDmg)
    }
    fn label(self) -> &'static str {
        match self {
            InstallerKind::WindowsExe => "windows-exe",
            InstallerKind::WindowsMsi => "windows-msi",
            InstallerKind::MacPkg => "macos-pkg",
            InstallerKind::MacDmg => "macos-dmg",
            InstallerKind::LinuxDeb => "linux-deb",
            InstallerKind::LinuxRpm => "linux-rpm",
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reject {
    EmptyPath,
    UnsafePath,
    UnknownExtension(String),
    WrongOs { wanted: TargetOs, current: TargetOs },
}
pub fn f_zn(r_1up: &Path, current: TargetOs) -> Result<InstallerKind, Reject> {
    let r_1uq = r_1up.to_string_lossy();
    if r_1uq.trim().is_empty() {
        return Err(Reject::EmptyPath);
    }
    if r_1uq.chars().any(|r_1uo| r_1uo.is_control()) {
        return Err(Reject::UnsafePath);
    }
    let ext = r_1up
        .extension()
        .and_then(OsStr::to_str)
        .map(|e| e.to_ascii_lowercase())
        .unwrap_or_default();
    let kind = InstallerKind::from_extension(&ext).ok_or(Reject::UnknownExtension(ext))?;
    let wanted = kind.target_os();
    if wanted != current {
        return Err(Reject::WrongOs { wanted, current });
    }
    Ok(kind)
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct T9y {
    pub program: String,
    pub args: Vec<String>,
}
fn f_24l(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}
pub fn plan(kind: InstallerKind, r_1ur: &Path) -> T9y {
    let p = r_1ur.to_string_lossy().into_owned();
    match kind {
        InstallerKind::WindowsExe => T9y {
            program: "powershell".into(),
            args: vec![
                "-NoProfile".into(),
                "-NonInteractive".into(),
                "-Command".into(),
                format!("Start-Process -FilePath {} -Verb RunAs", f_24l(&p)),
            ],
        },
        InstallerKind::WindowsMsi => T9y {
            program: "powershell".into(),
            args: vec![
                "-NoProfile".into(),
                "-NonInteractive".into(),
                "-Command".into(),
                format!(
                    "Start-Process -FilePath 'msiexec' -ArgumentList '/i',{} -Verb RunAs",
                    f_24l(&p)
                ),
            ],
        },
        InstallerKind::MacPkg | InstallerKind::MacDmg => T9y {
            program: "open".into(),
            args: vec![p],
        },
        InstallerKind::LinuxDeb => T9y {
            program: "pkexec".into(),
            args: vec!["apt-get".into(), "install".into(), "-y".into(), p],
        },
        InstallerKind::LinuxRpm => T9y {
            program: "pkexec".into(),
            args: vec!["dnf".into(), "install".into(), "-y".into(), p],
        },
    }
}
pub trait T9z {
    fn launch(&self, plan: &T9y) -> std::io::Result<()>;
}
pub struct Tfm;
impl T9z for Tfm {
    fn launch(&self, plan: &T9y) -> std::io::Result<()> {
        let mut r_1us = Command::new(&plan.program);
        r_1us.args(&plan.args);
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const C_2D: u32 = 0x0800_0000;
            r_1us.creation_flags(C_2D);
        }
        r_1us.spawn().map(|_child| ())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct T9x {
    pub kind: String,
    pub manual: bool,
    pub message: String,
}
pub fn f_2aq(
    r_1uu: &Path,
    current: TargetOs,
    r_1ut: &dyn Fn(&Path) -> bool,
    r_1uv: &dyn T9z,
) -> Result<T9x, AppError> {
    let kind = f_zn(r_1uu, current).map_err(f_26t)?;
    if !r_1ut(r_1uu) {
        return Err(AppError::not_found(format!(
            "S2jDtG5nIHTDrG0gdGjhuqV5IGZpbGUgYuG7mSBjw6BpOiA={}",
            r_1uu.display()
        )));
    }
    let plan = plan(kind, r_1uu);
    r_1uv.launch(&plan).map_err(|e| f_1mh(kind, &e))?;
    Ok(T9x {
        kind: kind.label().into(),
        manual: kind.is_manual(),
        message: f_10t(kind, current),
    })
}
fn f_26t(r_1uw: Reject) -> AppError {
    match r_1uw {
        Reject::EmptyPath => {
            AppError::validation("Q2jGsGEgY2jhu41uIGZpbGUgYuG7mSBjw6BpLg==")
        }
        Reject::UnsafePath => {
            AppError::validation("xJDGsOG7nW5nIGThuqtuIGZpbGUga2jDtG5nIGjhu6NwIGzhu4cu")
        }
        Reject::UnknownExtension(ext) if ext.is_empty() => {
            AppError::validation(
                "RmlsZSBraMO0bmcgY8OzIHBo4bqnbiBt4bufIHLhu5luZyDigJQga2jDtG5nIHLDtSBsb+G6oWkgYuG7mSBjw6BpLg==",
            )
        }
        Reject::UnknownExtension(ext) => {
            AppError::validation(
                format!(
                    "RGlnaVRleHQga2jDtG5nIG5o4bqtbiBkaeG7h24gxJHGsOG7o2MgbG/huqFpIGLhu5kgY8OgaSAnLg=={ext}'."
                ),
            )
        }
        Reject::WrongOs { wanted, current } => {
            AppError::validation(
                format!(
                    "QuG7mSBjw6BpIG7DoHkgZMOgbmggY2hvIA=={}LCBraMO0bmcgY2jhuqF5IMSRxrDhu6NjIHRyw6puIA=={}.",
                    wanted.label(), current.label()
                ),
            )
        }
    }
}
fn f_1mh(kind: InstallerKind, e: &std::io::Error) -> AppError {
    if e.kind() == std::io::ErrorKind::NotFound {
        let tool = match kind.target_os() {
            TargetOs::Windows => "PowerShell",
            TargetOs::MacOs => "bOG7h25oICdvcGVuJw==",
            TargetOs::Linux => "cGtleGVjIChnw7NpIHBvbGljeWtpdC0xKQ==",
        };
        AppError::internal(
            format!(
                "S2jDtG5nIHTDrG0gdGjhuqV5IA=={tool}IMSR4buDIGNo4bqheSBi4buZIGPDoGkuIFZ1aSBsw7JuZyBjaOG6oXkgYuG7mSBjw6BpIHRo4bunIGPDtG5nLg=="
            ),
        )
    } else {
        AppError::internal(format!(
            "S2jDtG5nIGto4bufaSBjaOG6oXkgxJHGsOG7o2MgYuG7mSBjw6BpOiA={e}"
        ))
    }
}
fn f_10t(kind: InstallerKind, current: TargetOs) -> String {
    if kind.is_manual() {
        return "xJDDoyBt4bufIOG6o25oIMSRxKlhLiBIw6N5IGNo4bqheSBi4buZIGPDoGkgYsOqbiB0cm9uZywgc2F1IMSRw7MgYuG6pW0gIlTDrG0gbOG6oWkiLg=="
            .into();
    }
    let mut r_1ux = String::from(
        "xJDDoyBraOG7n2kgY2jhuqF5IGLhu5kgY8OgaS4gSG/DoG4gdOG6pXQgY8OhYyBixrDhu5tjIGPDoGksIHNhdSDEkcOzIGLhuqVtICJUw6xtIGzhuqFpIiDEkeG7gyBuaOG6rW4gbcOheS4=",
    );
    if matches!(current, TargetOs::MacOs | TargetOs::Linux) {
        r_1ux
            .push_str(
                "IEzGsHUgw706IHRyw6puIGjhu4cgbsOgeSBtw6F5IHF1w6l0IGTDuW5nIFNBTkUg4oCUIG7hur91IGRyaXZlciBj4bunYSBow6NuZyBraMO0bmcgcGjhuqNpIFNBTkUgYmFja2VuZCB0aMOsIG3DoXkgY8OzIHRo4buDIHbhuqtuIGtow7RuZyB4deG6pXQgaGnhu4duLg==",
            );
    }
    r_1ux
}
#[tauri::command]
pub fn scanner_driver_extensions(
    auth: tauri::State<'_, crate::auth::AuthState>,
) -> Result<Vec<String>, AppError> {
    auth.require(digitext_core::Perm::Read)?;
    Ok(TargetOs::current()
        .installer_extensions()
        .iter()
        .map(|s| s.to_string())
        .collect())
}
#[tauri::command]
pub async fn install_scanner_driver(
    auth: tauri::State<'_, crate::auth::AuthState>,
    path: String,
) -> Result<T9x, AppError> {
    auth.require_perm(digitext_core::Permission::ScanExecute)?;
    tauri::async_runtime::spawn_blocking(move || {
        let p = std::path::PathBuf::from(&path);
        let r_1v0 = f_2aq(&p, TargetOs::current(), &|r_1uz| r_1uz.is_file(), &Tfm);
        match &r_1v0 {
            Ok(r_1uy) => {
                tracing::info!(
                    kind = % r_1uy.kind,
                    "aW5zdGFsbF9zY2FubmVyX2RyaXZlcjogxJHDoyBraOG7n2kgY2jhuqF5"
                )
            }
            Err(e) => {
                tracing::warn!(
                    code = e.code(),
                    "aW5zdGFsbF9zY2FubmVyX2RyaXZlciBs4buXaTog{e}"
                )
            }
        }
        r_1v0
    })
    .await
    .map_err(|e| AppError::internal(format!("bHXhu5NuZyBjw6BpIGRyaXZlciBs4buXaTog{e}")))?
}
