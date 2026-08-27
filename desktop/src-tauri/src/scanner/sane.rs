// Copyright (c) 2026 DigiText. All rights reserved.
use super::{
    f_199, f_19b, f_2bw, f_vo, BatchState, CancelToken, ColorMode, PaperSource, Tko, Tkr, Tkv, Tkw,
    C_G1,
};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{mpsc, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};
const C_BT: u32 = 1000;
const LIST_TIMEOUT: Duration = Duration::from_secs(15);
const C_8V: Duration = Duration::from_secs(10);
fn f_2cf() -> Result<&'static str, String> {
    static BIN: OnceLock<Option<String>> = OnceLock::new();
    let r_2hr = BIN.get_or_init(|| {
        const C_1L: &[&str] = &[
            "/opt/homebrew/bin/scanimage",
            "/usr/local/bin/scanimage",
            "/opt/local/bin/scanimage",
            "/usr/bin/scanimage",
        ];
        for r_2hq in C_1L {
            if std::path::Path::new(r_2hq).exists() {
                return Some((*r_2hq).to_string());
            }
        }
        if Command::new("scanimage").arg("-V").output().is_ok() {
            return Some("scanimage".to_string());
        }
        None
    });
    r_2hr
        .as_deref()
        .ok_or_else(|| {
            "Không tìm thấy 'scanimage'. Cài SANE rồi thử lại: `brew install sane-backends` (macOS) hoặc gói sane-utils (Linux)."
                .to_string()
        })
}
pub struct Tki;
impl Tki {
    pub fn new() -> Self {
        Tki
    }
}
impl Tkv for Tki {
    fn list_devices(&self) -> Result<Vec<Tkw>, String> {
        let r_2hs = f_259()?;
        if !r_2hs.is_empty() {
            return Ok(r_2hs);
        }
        thread::sleep(Duration::from_millis(400));
        f_259()
    }
    fn scan_batch(&self, r_2hu: &str, r_2i2: &Tko, r_2i5: &dyn Tkr, r_2hv: &CancelToken) {
        r_2i5.state(r_2hu, BatchState::Starting);
        let device = match r_2i2.device_id.clone() {
            Some(d) if !d.is_empty() => d,
            _ => match self.list_devices() {
                Ok(r_2i0) => match r_2i0.into_iter().next() {
                    Some(d) => d.id,
                    None => {
                        return r_2i5
                                    .error(
                                        r_2hu,
                                        "không tìm thấy máy quét (scanimage -L trống)",
                                    );
                    }
                },
                Err(e) => return r_2i5.error(r_2hu, &e),
            },
        };
        let bin = match f_2cf() {
            Ok(r_2ht) => r_2ht,
            Err(e) => return r_2i5.error(r_2hu, &e),
        };
        r_2i5.state(
            r_2hu,
            BatchState::Capturing {
                device: device.clone(),
            },
        );
        let (source, mode) = f_154(&device, r_2i2.source, r_2i2.color, r_2i2.duplex);
        if r_2i2.duplex && matches!(r_2i2.source, PaperSource::Adf) {
            f_2c1(
                r_2hu,
                bin,
                &device,
                source.as_deref(),
                &mode,
                r_2i2,
                r_2i5,
                r_2hv,
            );
            return;
        }
        let r_2i4 = matches!(r_2i2.source, PaperSource::Glass);
        let r_2i6 = if matches!(r_2i2.source, PaperSource::Adf) {
            f_t5(&device)
        } else {
            Vec::new()
        };
        let mut index = r_2i2.start_index;
        loop {
            if r_2hv.is_cancelled() {
                r_2i5.state(
                    r_2hu,
                    BatchState::Cancelled {
                        pages: index.saturating_sub(1),
                    },
                );
                return;
            }
            let mut r_2hw = Command::new(bin);
            r_2hw
                .arg("-d")
                .arg(&device)
                .arg("--format=tiff")
                .arg(format!("--resolution={}", r_2i2.dpi))
                .arg("--mode")
                .arg(&mode);
            if let Some(r_2i7) = &source {
                r_2hw.arg("--source").arg(r_2i7);
            }
            r_2hw.args(&r_2i6);
            let (stdout, stderr, r_2i1) = match f_2b6(r_2hw, r_2hv) {
                Ok(Some(r_2i8)) => r_2i8,
                Ok(None) => {
                    r_2i5.state(
                        r_2hu,
                        BatchState::Cancelled {
                            pages: index.saturating_sub(1),
                        },
                    );
                    return;
                }
                Err(e) => return r_2i5.error(r_2hu, &format!("scanimage: {e}")),
            };
            if !r_2i1 {
                let r_2hy = stderr.to_lowercase();
                let r_2hx = r_2hy.is_empty()
                    || r_2hy.contains("out of documents")
                    || r_2hy.contains("no documents")
                    || r_2hy.contains("document feeder out of");
                if r_2hx {
                    r_2i5.state(
                        r_2hu,
                        BatchState::Completed {
                            pages: index.saturating_sub(1),
                        },
                    );
                } else {
                    r_2i5.error(r_2hu, stderr.trim());
                    r_2i5.state(r_2hu, BatchState::Disconnected { at_page: index });
                }
                return;
            }
            match image::load_from_memory(&stdout) {
                Ok(r_2hz) => {
                    let (r_2ia, h) = (r_2hz.width(), r_2hz.height());
                    let r_2i9 = f_19b(&r_2hz.thumbnail(96, 136).to_rgb8());
                    let r_2i3 = f_199(&r_2hz.thumbnail(C_G1, C_G1).to_rgb8());
                    let mut page = f_vo(index, r_2ia, h, r_2i2.dpi, r_2i9, r_2i3);
                    page.source_path = f_2bw(&r_2hz, r_2hu, index);
                    r_2i5.page(r_2hu, page);
                    index += 1;
                }
                Err(e) => {
                    return r_2i5.error(r_2hu, &format!("giải mã trang {index}: {e}"));
                }
            }
            if r_2i4 || index.saturating_sub(r_2i2.start_index) >= C_BT {
                r_2i5.state(
                    r_2hu,
                    BatchState::Completed {
                        pages: index.saturating_sub(1),
                    },
                );
                return;
            }
        }
    }
}
#[allow(clippy::too_many_arguments)]
fn f_2c1(
    r_2ib: &str,
    bin: &str,
    device: &str,
    source: Option<&str>,
    mode: &str,
    r_2is: &Tko,
    r_2j0: &dyn Tkr,
    r_2ie: &CancelToken,
) {
    let r_2ij =
        std::env::temp_dir().join(format!("digitext-scan-{}-{}", std::process::id(), r_2ib));
    if let Err(e) = fs::create_dir_all(&r_2ij) {
        return r_2j0
            .error(
                r_2ib,
                &format!(
                    "không tạo được thư mục tạm để quét 2 mặt: {e}"
                ),
            );
    }
    let r_2iv = r_2ij.join("p%05d.tif");
    let mut r_2ih = Command::new(bin);
    r_2ih
        .arg("-d")
        .arg(device)
        .arg("--format=tiff")
        .arg(format!("--batch={}", r_2iv.display()))
        .arg(format!("--resolution={}", r_2is.dpi))
        .arg("--mode")
        .arg(mode);
    if let Some(r_2j1) = source {
        r_2ih.arg("--source").arg(r_2j1);
    }
    r_2ih.args(f_t5(device));
    let mut r_2ig = match r_2ih.stdout(Stdio::null()).stderr(Stdio::piped()).spawn() {
        Ok(r_2id) => r_2id,
        Err(e) => {
            let _ = fs::remove_dir_all(&r_2ij);
            return r_2j0.error(
                r_2ib,
                &format!("không chạy được scanimage ({bin}): {e}"),
            );
        }
    };
    let mut r_2in = r_2ig.stderr.take().expect("piped stderr");
    let (r_2j5, r_2iy) = mpsc::channel();
    thread::spawn(move || {
        let mut s = String::new();
        let _ = r_2in.read_to_string(&mut s);
        let _ = r_2j5.send(s);
    });
    let mut r_2j3: Option<std::process::ExitStatus> = None;
    let mut r_2ii = 0u32;
    let mut r_2it = 0u32;
    let mut r_2if = false;
    loop {
        if r_2ie.is_cancelled() {
            let _ = r_2ig.kill();
            let _ = r_2ig.wait();
            r_2if = true;
            break;
        }
        if let Ok(Some(r_2j2)) = r_2ig.try_wait() {
            r_2j3 = Some(r_2j2);
        }
        let r_2io = r_2j3.is_some();
        let r_2iw = f_11w(&r_2ij);
        let r_2iz = if r_2io {
            r_2iw
        } else {
            r_2iw.saturating_sub(1)
        };
        while r_2ii < r_2iz {
            let r_2iu = r_2ij.join(format!("p{:05}.tif", r_2ii + 1));
            match fs::read(&r_2iu) {
                Ok(r_2ic) if !r_2ic.is_empty() => match image::load_from_memory(&r_2ic) {
                    Ok(r_2iq) => {
                        let (r_2j6, h) = (r_2iq.width(), r_2iq.height());
                        let r_2j4 = f_19b(&r_2iq.thumbnail(96, 136).to_rgb8());
                        let r_2ix = f_199(&r_2iq.thumbnail(C_G1, C_G1).to_rgb8());
                        let r_2ip = r_2is.start_index + r_2it;
                        let mut page = f_vo(r_2ip, r_2j6, h, r_2is.dpi, r_2j4, r_2ix);
                        page.source_path = f_2bw(&r_2iq, r_2ib, r_2ip);
                        r_2j0.page(r_2ib, page);
                        r_2it += 1;
                    }
                    Err(e) => r_2j0.error(
                        r_2ib,
                        &format!("giải mã trang {}: {e}", r_2is.start_index + r_2it),
                    ),
                },
                _ => {}
            }
            r_2ii += 1;
        }
        if r_2io {
            break;
        }
        thread::sleep(Duration::from_millis(80));
    }
    let stderr = r_2iy.recv().unwrap_or_default();
    let _ = fs::remove_dir_all(&r_2ij);
    let r_2il = r_2is.start_index.saturating_sub(1) + r_2it;
    if r_2if {
        r_2j0.state(r_2ib, BatchState::Cancelled { pages: r_2il });
        return;
    }
    let r_2ir = r_2j3.map(|s| s.success()).unwrap_or(false);
    let r_2im = stderr.to_lowercase();
    let r_2ik = r_2it > 0
        || r_2im.is_empty()
        || r_2im.contains("out of documents")
        || r_2im.contains("no documents")
        || r_2im.contains("document feeder out of");
    if r_2ir || r_2ik {
        r_2j0.state(r_2ib, BatchState::Completed { pages: r_2il });
    } else {
        r_2j0.error(r_2ib, stderr.trim());
        r_2j0.state(
            r_2ib,
            BatchState::Disconnected {
                at_page: r_2is.start_index + r_2it,
            },
        );
    }
}
fn f_11w(r_2j7: &Path) -> u32 {
    let mut r_2j8 = 0u32;
    while r_2j7.join(format!("p{:05}.tif", r_2j8 + 1)).exists() {
        r_2j8 += 1;
    }
    r_2j8
}
fn f_2b6(
    mut r_2jc: Command,
    r_2ja: &CancelToken,
) -> Result<Option<(Vec<u8>, String, bool)>, String> {
    let mut r_2jb = r_2jc
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;
    let mut r_2je = r_2jb.stdout.take().expect("piped stdout");
    let mut r_2jd = r_2jb.stderr.take().expect("piped stderr");
    let (r_2jk, r_2jg) = mpsc::channel();
    let (r_2jj, r_2jf) = mpsc::channel();
    thread::spawn(move || {
        let mut r_2j9 = Vec::new();
        let _ = r_2je.read_to_end(&mut r_2j9);
        let _ = r_2jk.send(r_2j9);
    });
    thread::spawn(move || {
        let mut s = String::new();
        let _ = r_2jd.read_to_string(&mut s);
        let _ = r_2jj.send(s);
    });
    let r_2ji = loop {
        if r_2ja.is_cancelled() {
            let _ = r_2jb.kill();
            let _ = r_2jb.wait();
            return Ok(None);
        }
        match r_2jb.try_wait().map_err(|e| e.to_string())? {
            Some(r_2jh) => break r_2jh,
            None => thread::sleep(Duration::from_millis(50)),
        }
    };
    let stdout = r_2jg.recv().unwrap_or_default();
    let stderr = r_2jf.recv().unwrap_or_default();
    Ok(Some((stdout, stderr, r_2ji.success())))
}
fn f_259() -> Result<Vec<Tkw>, String> {
    let (stdout, stderr, r_2jp) = f_2cg(&["-L"], LIST_TIMEOUT)?;
    let r_2jr = String::from_utf8_lossy(&stdout);
    let mut r_2jl = Vec::new();
    for r_2jo in r_2jr.lines() {
        let Some(r_2jq) = r_2jo.trim().strip_prefix("device ") else {
            continue;
        };
        let Some((r_2jn, desc)) = r_2jq.split_once(" is a ") else {
            continue;
        };
        let r_2jm = r_2jn.trim().trim_matches(['`', '\'']).to_string();
        let desc = desc.trim().to_string();
        r_2jl.push(Tkw {
            id: r_2jm,
            name: desc.clone(),
            model: desc,
            connected: true,
        });
    }
    if r_2jl.is_empty() && !r_2jp {
        let msg = stderr.trim();
        if !msg.is_empty() {
            return Err(format!("scanimage -L lỗi: {msg}"));
        }
    }
    Ok(r_2jl)
}
fn f_2cg(r_2js: &[&str], r_2k2: Duration) -> Result<(Vec<u8>, String, bool), String> {
    let bin = f_2cf()?;
    let mut r_2ju = Command::new(bin)
        .args(r_2js)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("không chạy được scanimage ({bin}): {e}"))?;
    let mut r_2jx = r_2ju.stdout.take().expect("piped stdout");
    let mut r_2jw = r_2ju.stderr.take().expect("piped stderr");
    let (r_2k4, r_2jz) = mpsc::channel();
    let (r_2k3, r_2jy) = mpsc::channel();
    thread::spawn(move || {
        let mut r_2jt = Vec::new();
        let _ = r_2jx.read_to_end(&mut r_2jt);
        let _ = r_2k4.send(r_2jt);
    });
    thread::spawn(move || {
        let mut s = String::new();
        let _ = r_2jw.read_to_string(&mut s);
        let _ = r_2k3.send(s);
    });
    let r_2jv = Instant::now() + r_2k2;
    let r_2k1 = loop {
        match r_2ju.try_wait().map_err(|e| e.to_string())? {
            Some(r_2k0) => break r_2k0,
            None => {
                if Instant::now() >= r_2jv {
                    let _ = r_2ju.kill();
                    let _ = r_2ju.wait();
                    return Err(format!(
                        "scanimage quá {}s khi dò máy quét",
                        r_2k2.as_secs()
                    ));
                }
                thread::sleep(Duration::from_millis(40));
            }
        }
    };
    let stdout = r_2jz.recv().unwrap_or_default();
    let stderr = r_2jy.recv().unwrap_or_default();
    Ok((stdout, stderr, r_2k1.success()))
}
fn f_1wj(device: &str, opt: &str) -> Vec<String> {
    let Ok((stdout, _stderr, _ok)) = f_2cg(&["--help", "-d", device], C_8V) else {
        return Vec::new();
    };
    let r_2k9 = String::from_utf8_lossy(&stdout);
    let r_2k7 = format!("--{opt} ");
    for r_2k5 in r_2k9.lines() {
        let Some(r_2k8) = r_2k5.trim_start().strip_prefix(&r_2k7) else {
            continue;
        };
        let r_2k6 = r_2k8.split('[').next().unwrap_or("").trim();
        return r_2k6
            .split('|')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }
    Vec::new()
}
type T2i = (Option<String>, String);
fn f_154(device: &str, source: PaperSource, color: ColorMode, duplex: bool) -> T2i {
    static CACHE: OnceLock<Mutex<HashMap<String, T2i>>> = OnceLock::new();
    let r_2ka = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let r_2kc = format!("{device}|{source:?}|{color:?}|dup={duplex}");
    if let Some(r_2kd) = r_2ka.lock().unwrap().get(&r_2kc) {
        return r_2kd.clone();
    }
    let r_2kb = (f_22w(device, source, duplex), f_22u(device, color));
    r_2ka.lock().unwrap().insert(r_2kc, r_2kb.clone());
    r_2kb
}
fn f_22w(device: &str, r_2kg: PaperSource, duplex: bool) -> Option<String> {
    let r_2kf = f_1wj(device, "source");
    if r_2kf.is_empty() {
        return None;
    }
    let r_2ke = match r_2kg {
        PaperSource::Adf if duplex => r_2kf
            .iter()
            .find(|v| v.to_lowercase().contains("duplex"))
            .or_else(|| {
                r_2kf.iter().find(|v| {
                    let l = v.to_lowercase();
                    l.contains("adf") && !l.contains("back")
                })
            }),
        PaperSource::Adf => r_2kf.iter().find(|v| {
            let l = v.to_lowercase();
            l.contains("adf") && !l.contains("duplex") && !l.contains("back")
        }),
        PaperSource::Glass => r_2kf.iter().find(|v| {
            let l = v.to_lowercase();
            l.contains("flatbed") || l.contains("platen") || l.contains("glass")
        }),
    };
    r_2ke.or_else(|| r_2kf.first()).cloned()
}
fn f_22u(device: &str, r_2kj: ColorMode) -> String {
    let r_2kh = match r_2kj {
        ColorMode::Rgb => "color",
        ColorMode::Gray => "gray",
    };
    let r_2ki = f_1wj(device, "mode");
    r_2ki
        .iter()
        .find(|v| v.to_lowercase() == r_2kh)
        .or_else(|| r_2ki.iter().find(|v| v.to_lowercase().contains(r_2kh)))
        .cloned()
        .unwrap_or_else(|| match r_2kj {
            ColorMode::Rgb => "Color".into(),
            ColorMode::Gray => "Gray".into(),
        })
}
fn f_t5(device: &str) -> Vec<String> {
    static CACHE: OnceLock<Mutex<HashMap<String, Vec<String>>>> = OnceLock::new();
    let r_2kl = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(r_2ko) = r_2kl.lock().unwrap().get(device) {
        return r_2ko.clone();
    }
    let help = match f_2cg(&["--help", "-d", device], C_8V) {
        Ok((stdout, _, _)) => String::from_utf8_lossy(&stdout).into_owned(),
        Err(_) => String::new(),
    };
    let mut r_2kk = Vec::new();
    if help.contains("--swcrop") {
        r_2kk.push("--swcrop=yes".to_string());
    }
    if help.contains("--swdeskew") {
        r_2kk.push("--swdeskew=yes".to_string());
    }
    if help.contains("--ald") {
        r_2kk.push("--ald=yes".to_string());
        let r_2km = 380.0_f32;
        let h = f_1zl(&help, "page-height").map_or(r_2km, |r_2kn| r_2kn.min(r_2km));
        r_2kk.push(format!("--page-height={h}"));
    }
    r_2kl
        .lock()
        .unwrap()
        .insert(device.to_string(), r_2kk.clone());
    r_2kk
}
fn f_1zl(help: &str, opt: &str) -> Option<f32> {
    let r_2ks = format!("--{opt} ");
    for r_2kr in help.lines() {
        let Some(r_2ku) = r_2kr.trim_start().strip_prefix(&r_2ks) else {
            continue;
        };
        let r_2kp = r_2ku.split("..").nth(1)?;
        let r_2kt: String = r_2kp
            .chars()
            .take_while(|r_2kq| r_2kq.is_ascii_digit() || *r_2kq == '.')
            .collect();
        return r_2kt.parse().ok();
    }
    None
}
