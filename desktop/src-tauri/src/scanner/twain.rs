// Copyright (c) 2026 DigiText. All rights reserved.
#![allow(
    non_snake_case,
    non_camel_case_types,
    dead_code,
    clippy::upper_case_acronyms
)]
use super::{
    f_199, f_19b, f_2bw, f_vo, BatchState, CancelToken, ColorMode, PaperSource, Tko, Tkr, Tkv, Tkw,
    C_G1,
};
use libloading::{Library, Symbol};
use std::ffi::c_void;
use std::time::{Duration, Instant};
type T8x = *mut c_void;
type T8w = *mut c_void;
type Tq0 = usize;
type Tal = isize;
type Tam = isize;
type Tq5 = unsafe extern "system" fn(T8x, u32, Tq0, Tal) -> Tam;
const C_8Y: isize = -3;
const C_23: u32 = 0x2;
const C_8E: u32 = 0x0042;
const C_FZ: u32 = 0x0001;
#[repr(C)]
#[derive(Clone, Copy)]
struct Tg0 {
    x: i32,
    y: i32,
}
#[repr(C)]
#[derive(Clone, Copy)]
struct Tcf {
    hwnd: T8x,
    message: u32,
    wParam: Tq0,
    lParam: Tal,
    time: u32,
    pt: Tg0,
}
#[repr(C)]
struct Tpz {
    cbSize: u32,
    style: u32,
    lpfnWndProc: Option<Tq5>,
    cbClsExtra: i32,
    cbWndExtra: i32,
    hInstance: T8w,
    hIcon: *mut c_void,
    hCursor: *mut c_void,
    hbrBackground: *mut c_void,
    lpszMenuName: *const u16,
    lpszClassName: *const u16,
    hIconSm: *mut c_void,
}
#[link(name = "user32")]
extern "system" {
    fn RegisterClassExW(lpwcx: *const Tpz) -> u16;
    fn UnregisterClassW(lpClassName: *const u16, hInstance: T8w) -> i32;
    #[allow(clippy::too_many_arguments)]
    fn CreateWindowExW(
        dwExStyle: u32,
        lpClassName: *const u16,
        lpWindowName: *const u16,
        dwStyle: u32,
        x: i32,
        y: i32,
        nWidth: i32,
        nHeight: i32,
        hWndParent: T8x,
        hMenu: *mut c_void,
        hInstance: T8w,
        lpParam: *mut c_void,
    ) -> T8x;
    fn DestroyWindow(hWnd: T8x) -> i32;
    fn DefWindowProcW(hWnd: T8x, msg: u32, wParam: Tq0, lParam: Tal) -> Tam;
    fn PeekMessageW(lpMsg: *mut Tcf, hWnd: T8x, min: u32, max: u32, remove: u32) -> i32;
    fn TranslateMessage(lpMsg: *const Tcf) -> i32;
    fn DispatchMessageW(lpMsg: *const Tcf) -> Tam;
}
#[link(name = "kernel32")]
extern "system" {
    fn GetModuleHandleW(lpModuleName: *const u16) -> T8w;
    fn GlobalAlloc(uFlags: u32, dwBytes: usize) -> *mut c_void;
    fn GlobalLock(hMem: *mut c_void) -> *mut c_void;
    fn GlobalUnlock(hMem: *mut c_void) -> i32;
    fn GlobalFree(hMem: *mut c_void) -> *mut c_void;
}
#[link(name = "ole32")]
extern "system" {
    fn CoInitializeEx(pvReserved: *mut c_void, dwCoInit: u32) -> i32;
    fn CoUninitialize();
}
fn f_2qz(r_2kv: &str) -> Vec<u16> {
    r_2kv.encode_utf16().chain(std::iter::once(0)).collect()
}
fn f_2n2(msg: &str) {
    use std::io::Write;
    let r_2kx = std::env::temp_dir().join("digitext-twain.log");
    if let Ok(mut r_2kw) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(r_2kx)
    {
        let _ = writeln!(r_2kw, "{msg}");
    }
}
type Tnp = u16;
type Tnq = u32;
type Tni = i16;
type Tnc = u16;
type Tnf = *mut c_void;
type Tnk = *mut c_void;
type Tno = [u8; 34];
const C_46: Tnq = 0x0001;
const C_47: Tnq = 0x0002;
const C_3N: Tnp = 0x0001;
const C_3O: Tnp = 0x0002;
const C_3P: Tnp = 0x0003;
const C_3Q: Tnp = 0x0004;
const C_3R: Tnp = 0x0005;
const DAT_SETUPMEMXFER: Tnp = 0x0006;
const C_3S: Tnp = 0x0009;
const DAT_IMAGEMEMFILEXFER: Tnp = 0x010c;
const C_CJ: Tnp = 0x0001;
const MSG_GETDEFAULT: Tnp = 0x0003;
const C_CK: Tnp = 0x0004;
const C_CL: Tnp = 0x0005;
const MSG_SET: Tnp = 0x0006;
const C_CN: Tnp = 0x0007;
const MSG_XFERREADY: Tnp = 0x0101;
const MSG_CLOSEDSREQ: Tnp = 0x0102;
const MSG_OPENDSM: Tnp = 0x0301;
const C_CH: Tnp = 0x0302;
const MSG_OPENDS: Tnp = 0x0401;
const C_CG: Tnp = 0x0402;
const MSG_ENABLEDS: Tnp = 0x0500;
const C_CI: Tnp = 0x0501;
const C_CM: Tnp = 0x0601;
const MSG_ENDXFER: Tnp = 0x0701;
const C_1Q: Tnp = 0x0001;
const C_1P: Tnp = 0x1002;
const C_1O: Tnp = 0x1013;
const C_9E: Tnp = 0x0101;
const C_9F: Tnp = 0x0103;
const C_9D: Tnp = 0x110c;
const C_9G: Tnp = 0x1118;
const C_9H: Tnp = 0x1119;
const C_9B: Tnp = 0x1150;
const C_9C: Tnp = 0x1151;
const C_N1: Tnp = 1;
const C_N2: Tnp = 2;
const C_N7: Tnp = 4;
const C_MX: Tnp = 0;
const C_N0: Tnp = 5;
const C_NA: Tnp = 0x0001;
const C_NB: Tnp = 0x0004;
const C_N8: Tnp = 0x0006;
const C_N9: Tnp = 0x0007;
const C_MY: Tnq = 0x0001;
const C_MZ: Tnq = 0x0008;
const C_N5: Tnp = 0;
const C_N3: Tnp = 4;
const C_N6: Tnp = 6;
const C_N4: Tnp = 7;
#[repr(C, packed(2))]
#[derive(Clone, Copy)]
struct Tns {
    MajorNum: Tnp,
    MinorNum: Tnp,
    Language: Tnp,
    Country: Tnp,
    Info: Tno,
}
#[repr(C, packed(2))]
#[derive(Clone, Copy)]
struct Tng {
    Id: Tnq,
    Version: Tns,
    ProtocolMajor: Tnp,
    ProtocolMinor: Tnp,
    SupportedGroups: Tnq,
    Manufacturer: Tno,
    ProductFamily: Tno,
    ProductName: Tno,
}
#[repr(C, packed(2))]
struct Tnr {
    ShowUI: Tnc,
    ModalUI: Tnc,
    hParent: Tnf,
}
#[repr(C, packed(2))]
struct Tnd {
    Cap: Tnp,
    ConType: Tnp,
    hContainer: Tnf,
}
#[repr(C, packed(2))]
struct Tnl {
    ItemType: Tnp,
    Item: Tnq,
}
#[repr(C, packed(2))]
struct Tne {
    pEvent: Tnk,
    TWMessage: Tnp,
}
#[repr(C, packed(2))]
struct Tnm {
    Count: Tnp,
    Reserved: Tnq,
}
#[repr(C, packed(2))]
struct Tnn {
    MinBufSize: Tnq,
    MaxBufSize: Tnq,
    Preferred: Tnq,
}
#[repr(C, packed(2))]
struct Tnj {
    Flags: Tnq,
    Length: Tnq,
    TheMem: Tnk,
}
#[repr(C, packed(2))]
struct Tnh {
    Compression: Tnp,
    BytesPerRow: Tnq,
    Columns: Tnq,
    Rows: Tnq,
    XOffset: Tnq,
    YOffset: Tnq,
    BytesWritten: Tnq,
    Memory: Tnj,
}
type T59 = unsafe extern "system" fn(
    pOrigin: *mut Tng,
    pDest: *mut Tng,
    DG: Tnq,
    DAT: Tnp,
    MSG: Tnp,
    pData: Tnk,
) -> Tnp;
fn f_2hq(r_2l1: &str) -> Tno {
    let mut r_2l0 = [0u8; 34];
    let r_2ky = r_2l1.as_bytes();
    let r_2kz = r_2ky.len().min(33);
    r_2l0[..r_2kz].copy_from_slice(&r_2ky[..r_2kz]);
    r_2l0
}
fn f_2hr(r_2l4: Tno) -> String {
    let r_2l3 = r_2l4
        .iter()
        .position(|&r_2l2| r_2l2 == 0)
        .unwrap_or(r_2l4.len());
    String::from_utf8_lossy(&r_2l4[..r_2l3]).trim().to_string()
}
fn f_1d4(r_2l5: u32) -> u32 {
    r_2l5 & 0xFFFF
}
fn f_rx() -> Tng {
    Tng {
        Id: 0,
        Version: Tns {
            MajorNum: 1,
            MinorNum: 0,
            Language: 0,
            Country: 0,
            Info: f_2hq("DigiText"),
        },
        ProtocolMajor: 2,
        ProtocolMinor: 4,
        SupportedGroups: C_46 | C_47,
        Manufacturer: f_2hq("DigiText"),
        ProductFamily: f_2hq("DigiText"),
        ProductName: f_2hq("DigiText Desktop"),
    }
}
struct ComGuard;
impl ComGuard {
    fn new() -> Self {
        unsafe { CoInitializeEx(std::ptr::null_mut(), C_23) };
        ComGuard
    }
}
impl Drop for ComGuard {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}
struct MsgWindow {
    hwnd: T8x,
    class: Vec<u16>,
    hinst: T8w,
}
impl MsgWindow {
    fn new() -> Result<Self, String> {
        unsafe {
            let hinst = GetModuleHandleW(std::ptr::null());
            let class = f_2qz("DigiTextTwainWnd");
            let r_2l6 = Tpz {
                cbSize: std::mem::size_of::<Tpz>() as u32,
                style: 0,
                lpfnWndProc: Some(DefWindowProcW),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: hinst,
                hIcon: std::ptr::null_mut(),
                hCursor: std::ptr::null_mut(),
                hbrBackground: std::ptr::null_mut(),
                lpszMenuName: std::ptr::null(),
                lpszClassName: class.as_ptr(),
                hIconSm: std::ptr::null_mut(),
            };
            RegisterClassExW(&r_2l6);
            let name = f_2qz("DigiText TWAIN");
            let hwnd = CreateWindowExW(
                0,
                class.as_ptr(),
                name.as_ptr(),
                0,
                0,
                0,
                0,
                0,
                C_8Y as *mut c_void,
                std::ptr::null_mut(),
                hinst,
                std::ptr::null_mut(),
            );
            if hwnd.is_null() {
                return Err(
                    "không tạo được cửa sổ message cho TWAIN".into(),
                );
            }
            Ok(MsgWindow { hwnd, class, hinst })
        }
    }
}
impl Drop for MsgWindow {
    fn drop(&mut self) {
        unsafe {
            if !self.hwnd.is_null() {
                DestroyWindow(self.hwnd);
            }
            UnregisterClassW(self.class.as_ptr(), self.hinst);
        }
    }
}
struct Dsm {
    _lib: Library,
    entry: T59,
}
impl Dsm {
    fn load() -> Result<Self, String> {
        unsafe {
            let r_2l7 = Library::new("twaindsm.dll")
                .map_err(|e| {
                    format!(
                        "không nạp được twaindsm.dll (đã cài TWAIN DSM?): {e}"
                    )
                })?;
            let r_2l8: Symbol<T59> = r_2l7
                .get(b"DSM_Entry\0")
                .map_err(|e| format!("không tìm thấy DSM_Entry: {e}"))?;
            let entry = *r_2l8;
            Ok(Dsm { _lib: r_2l7, entry })
        }
    }
}
struct Twain {
    dsm: Dsm,
    app: Tng,
    ds: Tng,
    ds_open: bool,
    dsm_open: bool,
    hwnd: T8x,
    _com: ComGuard,
    _win: MsgWindow,
}
impl Twain {
    fn open() -> Result<Self, String> {
        f_2n2("open: ComGuard…");
        let r_2l9 = ComGuard::new();
        f_2n2("open: MsgWindow…");
        let r_2la = MsgWindow::new()?;
        f_2n2("open: Dsm::load…");
        let dsm = Dsm::load()?;
        let mut app = f_rx();
        let mut hwnd = r_2la.hwnd;
        f_2n2("open: OPENDSM…");
        let rc = unsafe {
            (dsm.entry)(
                &mut app,
                std::ptr::null_mut(),
                C_46,
                C_3Q,
                MSG_OPENDSM,
                &mut hwnd as *mut T8x as Tnk,
            )
        };
        f_2n2(&format!("open: OPENDSM rc={rc}"));
        if rc != C_N5 {
            return Err(format!("MSG_OPENDSM rc={rc}"));
        }
        Ok(Twain {
            dsm,
            app,
            ds: unsafe { std::mem::zeroed() },
            ds_open: false,
            dsm_open: true,
            hwnd,
            _com: r_2l9,
            _win: r_2la,
        })
    }
    fn dsm_call(&mut self, r_2ld: Tnq, r_2lb: Tnp, msg: Tnp, r_2lc: Tnk) -> Tnp {
        unsafe {
            (self.dsm.entry)(
                &mut self.app,
                std::ptr::null_mut(),
                r_2ld,
                r_2lb,
                msg,
                r_2lc,
            )
        }
    }
    fn ds_call(&mut self, r_2lg: Tnq, r_2le: Tnp, msg: Tnp, r_2lf: Tnk) -> Tnp {
        unsafe { (self.dsm.entry)(&mut self.app, &mut self.ds, r_2lg, r_2le, msg, r_2lf) }
    }
    fn enumerate(&mut self) -> Vec<Tng> {
        let mut r_2lj = Vec::new();
        let mut r_2lh: Tng = unsafe { std::mem::zeroed() };
        f_2n2("enumerate: GETFIRST…");
        let rc = self.dsm_call(C_46, C_3P, C_CK, &mut r_2lh as *mut _ as Tnk);
        f_2n2(&format!(
            "enumerate: GETFIRST rc={rc} name='{}' mfr='{}'",
            f_2hr(r_2lh.ProductName),
            f_2hr(r_2lh.Manufacturer)
        ));
        if rc != C_N5 {
            return r_2lj;
        }
        r_2lj.push(r_2lh);
        let mut guard = 0;
        loop {
            guard += 1;
            if guard > 64 {
                f_2n2("enumerate: guard 64 → dừng");
                break;
            }
            let mut r_2li: Tng = unsafe { std::mem::zeroed() };
            let rc = self.dsm_call(C_46, C_3P, C_CL, &mut r_2li as *mut _ as Tnk);
            f_2n2(&format!(
                "enumerate: GETNEXT rc={rc} name='{}' mfr='{}'",
                f_2hr(r_2li.ProductName),
                f_2hr(r_2li.Manufacturer)
            ));
            if rc == C_N4 {
                break;
            }
            if rc != C_N5 {
                continue;
            }
            r_2lj.push(r_2li);
        }
        r_2lj
    }
    fn open_ds(&mut self, product_name: &str) -> Result<(), String> {
        let mut ds: Tng = if product_name.is_empty() {
            let mut r_2lk: Tng = unsafe { std::mem::zeroed() };
            let rc = self.dsm_call(C_46, C_3P, MSG_GETDEFAULT, &mut r_2lk as *mut _ as Tnk);
            if rc != C_N5 {
                return Err(format!("MSG_GETDEFAULT rc={rc}"));
            }
            r_2lk
        } else {
            self.enumerate()
                .into_iter()
                .find(|r_2ll| f_2hr(r_2ll.ProductName) == product_name)
                .ok_or_else(|| {
                    format!("không tìm thấy máy quét '{product_name}'")
                })?
        };
        let rc = self.dsm_call(C_46, C_3P, MSG_OPENDS, &mut ds as *mut _ as Tnk);
        if rc != C_N5 {
            return Err(format!("MSG_OPENDS rc={rc}"));
        }
        self.ds = ds;
        self.ds_open = true;
        Ok(())
    }
    fn set_cap(&mut self, cap: Tnp, r_2lp: Tnp, r_2lo: Tnq) -> Result<(), String> {
        unsafe {
            let r_2ln = GlobalAlloc(C_8E, std::mem::size_of::<Tnl>());
            if r_2ln.is_null() {
                return Err("GlobalAlloc cap thất bại".into());
            }
            let r_2lq = GlobalLock(r_2ln) as *mut Tnl;
            std::ptr::write_unaligned(
                r_2lq,
                Tnl {
                    ItemType: r_2lp,
                    Item: r_2lo,
                },
            );
            GlobalUnlock(r_2ln);
            let mut r_2lm = Tnd {
                Cap: cap,
                ConType: C_N0,
                hContainer: r_2ln,
            };
            let rc = self.ds_call(C_46, C_3N, MSG_SET, &mut r_2lm as *mut _ as Tnk);
            GlobalFree(r_2ln);
            if rc != C_N5 {
                return Err(format!("MSG_SET cap={cap:#06x} rc={rc}"));
            }
            Ok(())
        }
    }
    fn set_caps(&mut self, r_2lv: &Tko) -> Result<(), String> {
        self.set_cap(C_9F, C_NB, C_N7 as u32).map_err(|e| {
            format!("máy quét không nhận truyền MEMFILE ({e})")
        })?;
        self.set_cap(C_9D, C_NB, C_MX as u32)?;
        let r_2lw = match r_2lv.color {
            ColorMode::Rgb => C_N2,
            ColorMode::Gray => C_N1,
        };
        self.set_cap(C_9E, C_NB, r_2lw as u32)?;
        self.set_cap(C_9G, C_N9, f_1d4(r_2lv.dpi))?;
        self.set_cap(C_9H, C_N9, f_1d4(r_2lv.dpi))?;
        let r_2lr = matches!(r_2lv.source, PaperSource::Adf);
        let _ = self.set_cap(C_1P, C_N8, u32::from(r_2lr));
        let r_2lt = u32::from(r_2lr && r_2lv.duplex);
        let _ = self.set_cap(C_1O, C_N8, r_2lt);
        if r_2lr {
            let _ = self.set_cap(C_9B, C_N8, 1);
            let _ = self.set_cap(C_9C, C_N8, 1);
        }
        let r_2ls: i16 = match r_2lv.max_pages {
            Some(r_2lu) => r_2lu as i16,
            None => -1,
        };
        let _ = self.set_cap(C_1Q, C_NA, (r_2ls as u16) as u32);
        Ok(())
    }
    fn enable(&mut self) -> Result<(), String> {
        let mut r_2lx = Tnr {
            ShowUI: 0,
            ModalUI: 0,
            hParent: self.hwnd,
        };
        let rc = self.ds_call(C_46, C_3S, MSG_ENABLEDS, &mut r_2lx as *mut _ as Tnk);
        if rc != C_N5 {
            return Err(format!("MSG_ENABLEDS rc={rc}"));
        }
        Ok(())
    }
    fn pump_until_xferready(
        &mut self,
        r_2ly: &CancelToken,
        r_2lz: Instant,
    ) -> Result<bool, String> {
        loop {
            if r_2ly.is_cancelled() {
                return Err("đã huỷ".into());
            }
            if Instant::now() > r_2lz {
                return Err(
                    "hết thời gian chờ máy quét sẵn sàng (XFERREADY)"
                        .into(),
                );
            }
            let mut msg: Tcf = unsafe { std::mem::zeroed() };
            let r_2m1 = unsafe { PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, C_FZ) };
            if r_2m1 == 0 {
                std::thread::sleep(Duration::from_millis(10));
                continue;
            }
            let mut r_2m0 = Tne {
                pEvent: &mut msg as *mut Tcf as Tnk,
                TWMessage: 0,
            };
            let rc = self.ds_call(C_46, C_3O, C_CM, &mut r_2m0 as *mut _ as Tnk);
            let r_2m2 = r_2m0.TWMessage;
            if rc == C_N3 {
                match r_2m2 {
                    MSG_XFERREADY => return Ok(true),
                    MSG_CLOSEDSREQ => return Ok(false),
                    _ => {}
                }
            } else {
                unsafe {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        }
    }
    fn transfer_image(&mut self) -> Result<Vec<u8>, String> {
        let mut r_2mb: Tnn = unsafe { std::mem::zeroed() };
        let rc = self.ds_call(C_46, DAT_SETUPMEMXFER, C_CJ, &mut r_2mb as *mut _ as Tnk);
        if rc != C_N5 {
            return Err(format!("DAT_SETUPMEMXFER rc={rc}"));
        }
        let r_2m9 = r_2mb.Preferred;
        let r_2m6 = r_2mb.MinBufSize;
        let r_2m3 = r_2m9.max(r_2m6).max(64 * 1024) as usize;
        unsafe {
            let r_2m5 = GlobalAlloc(C_8E, r_2m3);
            if r_2m5.is_null() {
                return Err("GlobalAlloc buffer thất bại".into());
            }
            let r_2m8 = GlobalLock(r_2m5);
            let mut r_2m4 = Vec::new();
            let r_2ma = loop {
                let mut r_2mc: Tnh = std::mem::zeroed();
                r_2mc.Memory = Tnj {
                    Flags: C_MY | C_MZ,
                    Length: r_2m3 as u32,
                    TheMem: r_2m8,
                };
                let rc = self.ds_call(
                    C_47,
                    DAT_IMAGEMEMFILEXFER,
                    C_CJ,
                    &mut r_2mc as *mut _ as Tnk,
                );
                if rc == C_N5 || rc == C_N6 {
                    let r_2m7 = r_2mc.BytesWritten as usize;
                    if r_2m7 > 0 {
                        r_2m4.extend_from_slice(std::slice::from_raw_parts(
                            r_2m8 as *const u8,
                            r_2m7.min(r_2m3),
                        ));
                    }
                    if rc == C_N6 {
                        break Ok(r_2m4);
                    }
                } else {
                    break Err(format!("DAT_IMAGEMEMFILEXFER rc={rc}"));
                }
            };
            GlobalUnlock(r_2m5);
            GlobalFree(r_2m5);
            r_2ma
        }
    }
    fn end_xfer(&mut self) -> Result<u16, String> {
        let mut r_2md: Tnm = unsafe { std::mem::zeroed() };
        let rc = self.ds_call(C_46, C_3R, MSG_ENDXFER, &mut r_2md as *mut _ as Tnk);
        if rc != C_N5 {
            return Err(format!("MSG_ENDXFER rc={rc}"));
        }
        Ok(r_2md.Count)
    }
    fn reset_pending(&mut self) {
        let mut r_2me: Tnm = unsafe { std::mem::zeroed() };
        let _ = self.ds_call(C_46, C_3R, C_CN, &mut r_2me as *mut _ as Tnk);
    }
}
impl Drop for Twain {
    fn drop(&mut self) {
        if self.ds_open {
            let mut r_2mf = Tnr {
                ShowUI: 0,
                ModalUI: 0,
                hParent: std::ptr::null_mut(),
            };
            let _ = self.ds_call(C_46, C_3S, C_CI, &mut r_2mf as *mut _ as Tnk);
            let mut ds = self.ds;
            let _ = self.dsm_call(C_46, C_3P, C_CG, &mut ds as *mut _ as Tnk);
        }
        if self.dsm_open {
            let mut hwnd = self.hwnd;
            let _ = self.dsm_call(C_46, C_3Q, C_CH, &mut hwnd as *mut T8x as Tnk);
        }
    }
}
pub struct Tov;
impl Tov {
    pub fn new() -> Self {
        Tov
    }
}
impl Tkv for Tov {
    fn list_devices(&self) -> Result<Vec<Tkw>, String> {
        f_2n2("== list_devices: start ==");
        let mut twain = Twain::open()?;
        f_2n2("list_devices: open ok → enumerate");
        let r_2mj = twain.enumerate();
        f_2n2(&format!(
            "list_devices: enumerate → {} thiết bị",
            r_2mj.len()
        ));
        let r_2mh = r_2mj
            .into_iter()
            .map(|r_2mi| {
                let name = f_2hr(r_2mi.ProductName);
                let r_2mk = f_2hr(r_2mi.ProductFamily);
                Tkw {
                    id: name.clone(),
                    name: if name.is_empty() {
                        "Máy quét TWAIN".into()
                    } else {
                        name
                    },
                    model: r_2mk,
                    connected: true,
                }
            })
            .filter(|r_2mg| !r_2mg.id.is_empty())
            .collect();
        Ok(r_2mh)
    }
    fn scan_batch(&self, r_2mm: &str, r_2mt: &Tko, r_2my: &dyn Tkr, r_2mo: &CancelToken) {
        r_2my.state(r_2mm, BatchState::Starting);
        let mut twain = match Twain::open() {
            Ok(r_2mz) => r_2mz,
            Err(e) => return r_2my.error(r_2mm, &e),
        };
        let r_2n3 = r_2mt.device_id.clone().unwrap_or_default();
        if let Err(e) = twain.open_ds(&r_2n3) {
            return r_2my.error(r_2mm, &e);
        }
        let r_2mq = f_2hr(twain.ds.ProductName);
        r_2my.state(r_2mm, BatchState::Capturing { device: r_2mq });
        if let Err(e) = twain.set_caps(r_2mt) {
            return r_2my.error(r_2mm, &e);
        }
        if let Err(e) = twain.enable() {
            return r_2my.error(r_2mm, &e);
        }
        let r_2mp = Instant::now() + Duration::from_secs(60);
        match twain.pump_until_xferready(r_2mo, r_2mp) {
            Ok(true) => {}
            Ok(false) => {
                r_2my.state(r_2mm, BatchState::Cancelled { pages: 0 });
                return;
            }
            Err(e) => {
                if r_2mo.is_cancelled() {
                    r_2my.state(r_2mm, BatchState::Cancelled { pages: 0 });
                } else {
                    r_2my.error(r_2mm, &e);
                    r_2my.state(
                        r_2mm,
                        BatchState::Disconnected {
                            at_page: r_2mt.start_index,
                        },
                    );
                }
                return;
            }
        }
        let r_2mx = matches!(r_2mt.source, PaperSource::Glass);
        let mut index = r_2mt.start_index;
        loop {
            if r_2mo.is_cancelled() {
                twain.reset_pending();
                r_2my.state(
                    r_2mm,
                    BatchState::Cancelled {
                        pages: index.saturating_sub(1),
                    },
                );
                return;
            }
            let r_2n1 = match twain.transfer_image() {
                Ok(r_2ml) => r_2ml,
                Err(e) => {
                    r_2my.error(r_2mm, &e);
                    r_2my.state(r_2mm, BatchState::Disconnected { at_page: index });
                    return;
                }
            };
            let r_2mw = match twain.end_xfer() {
                Ok(r_2mn) => r_2mn,
                Err(e) => {
                    r_2my.error(r_2mm, &e);
                    r_2my.state(r_2mm, BatchState::Disconnected { at_page: index });
                    return;
                }
            };
            match image::load_from_memory(&r_2n1) {
                Ok(r_2ms) => {
                    let (r_2n2, r_2mr) = (r_2ms.width(), r_2ms.height());
                    let r_2n0 = f_19b(&r_2ms.thumbnail(96, 136).to_rgb8());
                    let r_2mv = f_199(&r_2ms.thumbnail(C_G1, C_G1).to_rgb8());
                    let mut r_2mu = f_vo(index, r_2n2, r_2mr, r_2mt.dpi, r_2n0, r_2mv);
                    r_2mu.source_path = f_2bw(&r_2ms, r_2mm, index);
                    r_2my.page(r_2mm, r_2mu);
                    index += 1;
                }
                Err(e) => {
                    return r_2my.error(r_2mm, &format!("giải mã trang {index}: {e}"));
                }
            }
            if r_2mw == 0 || r_2mx {
                if r_2mx && r_2mw > 0 {
                    twain.reset_pending();
                }
                r_2my.state(
                    r_2mm,
                    BatchState::Completed {
                        pages: index.saturating_sub(1),
                    },
                );
                return;
            }
        }
    }
}
