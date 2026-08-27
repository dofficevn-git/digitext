// Copyright (c) 2026 DigiText. All rights reserved.
use super::{
    f_199, f_19b, f_2bw, f_vo, BatchState, CancelToken, ColorMode, PaperSource, Tko, Tkr, Tkv, Tkw,
    C_G1,
};
use std::io::{BufRead, BufReader, Read};
use std::process::{Command, Stdio};
use std::sync::{mpsc, OnceLock};
use std::thread;
use std::time::{Duration, Instant};
const LIST_TIMEOUT: Duration = Duration::from_secs(25);
fn f_2qy(msg: &str) {
    use std::io::Write;
    let r_2n4 = std::env::temp_dir().join("digitext-wia.log");
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(r_2n4)
    {
        let _ = writeln!(f, "{msg}");
    }
}
fn f_23e() -> &'static str {
    static BIN: OnceLock<String> = OnceLock::new();
    BIN.get_or_init(|| {
        if let Ok(root) = std::env::var("SystemRoot") {
            let p = format!("{root}\\System32\\WindowsPowerShell\\v1.0\\powershell.exe");
            if std::path::Path::new(&p).exists() {
                return p;
            }
        }
        "powershell.exe".to_string()
    })
}
fn f_1gz(r_2n5: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        r_2n5.creation_flags(0x0800_0000);
    }
    #[cfg(not(windows))]
    {
        let _ = r_2n5;
    }
}
fn f_2b7(script: &str, r_2ng: Duration) -> Result<(Vec<u8>, String, bool), String> {
    let mut r_2n8 = Command::new(f_23e());
    r_2n8
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    f_1gz(&mut r_2n8);
    let mut r_2n7 = r_2n8
        .spawn()
        .map_err(|e| format!("không chạy được PowerShell: {e}"))?;
    let mut r_2nb = r_2n7.stdout.take().expect("piped stdout");
    let mut r_2na = r_2n7.stderr.take().expect("piped stderr");
    let (r_2ni, r_2nd) = mpsc::channel();
    let (r_2nh, r_2nc) = mpsc::channel();
    thread::spawn(move || {
        let mut r_2n6 = Vec::new();
        let _ = r_2nb.read_to_end(&mut r_2n6);
        let _ = r_2ni.send(r_2n6);
    });
    thread::spawn(move || {
        let mut s = String::new();
        let _ = r_2na.read_to_string(&mut s);
        let _ = r_2nh.send(s);
    });
    let r_2n9 = Instant::now() + r_2ng;
    let r_2nf = loop {
        match r_2n7.try_wait().map_err(|e| e.to_string())? {
            Some(r_2ne) => break r_2ne,
            None => {
                if Instant::now() >= r_2n9 {
                    let _ = r_2n7.kill();
                    let _ = r_2n7.wait();
                    return Err(format!("PowerShell quá {}s", r_2ng.as_secs()));
                }
                thread::sleep(Duration::from_millis(40));
            }
        }
    };
    let stdout = r_2nd.recv().unwrap_or_default();
    let stderr = r_2nc.recv().unwrap_or_default();
    Ok((stdout, stderr, r_2nf.success()))
}
const C_AH: &str = r#"
$ErrorActionPreference='SilentlyContinue'
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$dm = New-Object -ComObject WIA.DeviceManager
foreach($di in $dm.DeviceInfos){
  if($di.Type -eq 1){
    $name=''
    try { $name = $di.Properties.Item('Name').Value } catch {}
    Write-Output ("{0}`t{1}" -f $di.DeviceID, $name)
  }
}
"#;
const C_JH: &str = "\nparam([string]$DeviceID,[int]$Dpi,[int]$DataType,[string]$Adf,[int]$DocHandling,[string]$OutDir,[int]$StartIndex)\n$ErrorActionPreference='Stop'\n# UTF-8 stdout: nếu không, đường dẫn in ra dùng OEM codepage; %TEMP% chứa tên user tiếng\n# Việt → Rust đọc hỏng → mọi trang fail đọc file → lô \"hoàn thành\" 0 trang.\n[Console]::OutputEncoding = [System.Text.Encoding]::UTF8\n$FMT='{B96B3CAB-0728-11D3-9D7B-0000F81EF32E}'   # wiaFormatBMP\n$dm = New-Object -ComObject WIA.DeviceManager\n$info = $dm.DeviceInfos | Where-Object { $_.DeviceID -eq $DeviceID } | Select-Object -First 1\nif(-not $info){ [Console]::Error.WriteLine('device-not-found'); exit 2 }\n$dev = $info.Connect()\n# DOCUMENT_HANDLING_SELECT (3088): 1=FEEDER, 5=FEEDER|DUPLEX (1|4); 0 = mặc định (kính phẳng).\nif($DocHandling -gt 0){ try { $dev.Properties.Item('3088').Value = $DocHandling } catch {} }\n$item = $dev.Items.Item(1)\nfunction SetP($id,$v){ try { $item.Properties.Item(\"$id\").Value = $v } catch {} }\nSetP 6147 $Dpi        # WIA_IPS_XRES\nSetP 6148 $Dpi        # WIA_IPS_YRES\nSetP 4103 $DataType   # WIA_IPA_DATATYPE (2=gray,3=color)\n$i = $StartIndex\nwhile($true){\n  try { $img = $item.Transfer($FMT) }\n  catch { [Console]::Error.WriteLine('end:'+$_.Exception.Message); break }\n  $p = Join-Path $OutDir ('page-{0:00000}.bmp' -f $i)\n  if(Test-Path $p){ Remove-Item -Force $p }\n  $img.SaveFile($p)\n  [Console]::Out.WriteLine($p)\n  try { [System.Runtime.InteropServices.Marshal]::ReleaseComObject($img) | Out-Null } catch {}\n  $i++\n  if($Adf -ne '1'){ break }   # kính phẳng: 1 trang\n}\n";
pub struct Tq1;
impl Tq1 {
    pub fn new() -> Self {
        Tq1
    }
}
impl Tkv for Tq1 {
    fn list_devices(&self) -> Result<Vec<Tkw>, String> {
        f_2qy("== list_devices: WIA enumerate (PowerShell) ==");
        let (stdout, stderr, ok) = f_2b7(C_AH, LIST_TIMEOUT)?;
        let r_2nn = String::from_utf8_lossy(&stdout);
        let mut r_2nl = Vec::new();
        for line in r_2nn.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let (id, name) = match line.split_once('\t') {
                Some((r_2nj, r_2nk)) => (r_2nj.trim().to_string(), r_2nk.trim().to_string()),
                None => (line.to_string(), line.to_string()),
            };
            f_2qy(&format!("list_devices: id='{id}' name='{name}'"));
            let r_2nm = if name.is_empty() {
                "Máy quét WIA".to_string()
            } else {
                name.clone()
            };
            r_2nl.push(Tkw {
                id,
                name: r_2nm,
                model: name,
                connected: true,
            });
        }
        f_2qy(&format!(
            "list_devices: {} thiết bị (ok={ok})",
            r_2nl.len()
        ));
        if r_2nl.is_empty() && !ok {
            let e = stderr.trim();
            if !e.is_empty() {
                return Err(format!("WIA lỗi: {e}"));
            }
        }
        Ok(r_2nl)
    }
    fn scan_batch(&self, batch_id: &str, r_2ny: &Tko, r_2o6: &dyn Tkr, r_2nr: &CancelToken) {
        r_2o6.state(batch_id, BatchState::Starting);
        let device = match r_2ny.device_id.clone() {
            Some(r_2nu) if !r_2nu.is_empty() => r_2nu,
            _ => {
                r_2o6.error(batch_id, "Chưa chọn máy quét.");
                r_2o6.state(batch_id, BatchState::Cancelled { pages: 0 });
                return;
            }
        };
        let r_2o9 = std::env::temp_dir();
        let r_2o5 = r_2o9.join("digitext-wia-scan.ps1");
        if let Err(e) = std::fs::write(&r_2o5, C_JH) {
            return r_2o6.error(
                batch_id,
                &format!("không ghi được script WIA: {e}"),
            );
        }
        let r_2nz = r_2o9.join(format!("digitext-wia-{batch_id}"));
        let _ = std::fs::create_dir_all(&r_2nz);
        let datatype = match r_2ny.color {
            ColorMode::Rgb => "3",
            ColorMode::Gray => "2",
        };
        let r_2no = matches!(r_2ny.source, PaperSource::Adf);
        let adf = if r_2no { "1" } else { "0" };
        let doc_handling = if r_2no {
            if r_2ny.duplex {
                "5"
            } else {
                "1"
            }
        } else {
            "0"
        };
        f_2qy(
            &format!(
                "== scan: dev='{device}' dpi={} dt={datatype} adf={adf} duplex={} doc_handling={doc_handling} start={} ==",
                r_2ny.dpi, r_2ny.duplex, r_2ny.start_index
            ),
        );
        r_2o6.state(
            batch_id,
            BatchState::Capturing {
                device: device.clone(),
            },
        );
        let mut r_2nt = Command::new(f_23e());
        r_2nt
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-ExecutionPolicy",
                "Bypass",
                "-File",
                &r_2o5.to_string_lossy(),
                "-DeviceID",
                &device,
                "-Dpi",
                &r_2ny.dpi.to_string(),
                "-DataType",
                datatype,
                "-Adf",
                adf,
                "-DocHandling",
                doc_handling,
                "-OutDir",
                &r_2nz.to_string_lossy(),
                "-StartIndex",
                &r_2ny.start_index.to_string(),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        f_1gz(&mut r_2nt);
        let mut r_2ns = match r_2nt.spawn() {
            Ok(r_2nq) => r_2nq,
            Err(e) => {
                return r_2o6.error(
                    batch_id,
                    &format!("không chạy được PowerShell: {e}"),
                );
            }
        };
        let mut r_2nv = r_2ns.stderr.take().expect("piped stderr");
        let (r_2oa, r_2o3) = mpsc::channel();
        thread::spawn(move || {
            let mut s = String::new();
            let _ = r_2nv.read_to_string(&mut s);
            let _ = r_2oa.send(s);
        });
        let stdout = r_2ns.stdout.take().expect("piped stdout");
        let (r_2ob, r_2o4) = mpsc::channel::<String>();
        let r_2nx = thread::spawn(move || {
            let r_2o2 = BufReader::new(stdout);
            for r_2nw in r_2o2.lines() {
                match r_2nw {
                    Ok(l) => {
                        if r_2ob.send(l).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });
        let mut index = r_2ny.start_index;
        let mut produced = 0u32;
        let mut failed = 0u32;
        loop {
            if r_2nr.is_cancelled() {
                let _ = r_2ns.kill();
                let _ = r_2ns.wait();
                let _ = r_2nx.join();
                f_2qy("scan: huỷ");
                r_2o6.state(batch_id, BatchState::Cancelled { pages: produced });
                let _ = std::fs::remove_dir_all(&r_2nz);
                return;
            }
            let r_2o0 = match r_2o4.recv_timeout(Duration::from_millis(200)) {
                Ok(l) => l.trim().to_string(),
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            };
            if !r_2o0.to_ascii_lowercase().ends_with(".bmp") {
                continue;
            }
            match std::fs::read(&r_2o0).map(|r_2np| image::load_from_memory(&r_2np)) {
                Ok(Ok(img)) => {
                    let (w, h) = (img.width(), img.height());
                    let r_2o8 = f_19b(&img.thumbnail(96, 136).to_rgb8());
                    let r_2o1 = f_199(&img.thumbnail(C_G1, C_G1).to_rgb8());
                    let mut page = f_vo(index, w, h, r_2ny.dpi, r_2o8, r_2o1);
                    page.source_path = f_2bw(&img, batch_id, index);
                    r_2o6.page(batch_id, page);
                    f_2qy(&format!("scan: trang {index} {w}x{h}"));
                    index += 1;
                    produced += 1;
                }
                Ok(Err(e)) => {
                    failed += 1;
                    f_2qy(&format!(
                        "scan: giải mã trang {index} lỗi {e}"
                    ));
                    r_2o6.error(
                        batch_id,
                        &format!(
                            "Trang {index} lỗi giải mã ảnh — quét lại."
                        ),
                    );
                }
                Err(e) => {
                    failed += 1;
                    f_2qy(&format!(
                        "scan: đọc trang {index} lỗi {e}"
                    ));
                    r_2o6
                        .error(
                            batch_id,
                            &format!(
                                "Trang {index} không đọc được file ảnh — quét lại."
                            ),
                        );
                }
            }
            let _ = std::fs::remove_file(&r_2o0);
        }
        let _ = r_2nx.join();
        let r_2o7 = r_2ns.wait();
        let stderr = r_2o3.recv().unwrap_or_default();
        let err = stderr.trim();
        f_2qy(&format!(
            "scan: xong produced={produced} failed={failed} code={:?} stderr={err}",
            r_2o7.ok().and_then(|s| s.code())
        ));
        let _ = std::fs::remove_dir_all(&r_2nz);
        if produced == 0 && !err.is_empty() && !err.starts_with("end:") {
            r_2o6.error(batch_id, &format!("WIA scan: {err}"));
            r_2o6.state(batch_id, BatchState::Disconnected { at_page: index });
            return;
        }
        r_2o6.state(batch_id, BatchState::Completed { pages: produced });
    }
}
