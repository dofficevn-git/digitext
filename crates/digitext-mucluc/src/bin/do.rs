// Copyright (c) 2026 DigiText. All rights reserved.
//! Chạy bộ đọc mục lục trên một thư mục Excel thật và in ra **đúng khuôn** harness TS
//! (`desktop/scripts/kiem-muc-luc.ts`) để đối chứng từng tệp:
//!
//! ```text
//! npm run kiem:mucluc -- D:/DigiText/test/MLHS        # bản TS (chuẩn)
//! cargo run -p digitext-mucluc --bin mucluc-do -- D:/DigiText/test/MLHS
//! ```
//!
//! Hai bên phải ra **y hệt**: sheet, số hồ sơ, số phòng ban, bộ cột, số lỗi, số cảnh báo, các ô
//! trống, kết luận CHẶN / nhập được, và 6 thông báo hay gặp nhất. Lệch chỗ nào thì bản Rust chưa
//! dùng được — xem ba bẫy tầng ô ở đầu `lib.rs`.

use digitext_mucluc::phan_tich::{co_loi_chan, phan_tich_muc_luc, Muc, TuyChon};
use digitext_mucluc::{doc_so, SoMucLuc};
use std::path::Path;

/// Bảng THBQ mặc định (`digitext-core` `chinh_ly::codes::THBQ_SEED`) — chạy ngoài app không có
/// Danh mục. **Phải giống hệt danh sách trong harness TS**, kẻo lệch số lỗi THBQ.
const THBQ: &[&str] = &[
    "Vĩnh viễn",
    "5 năm",
    "10 năm",
    "15 năm",
    "20 năm",
    "50 năm",
    "70 năm",
    "05 năm đánh giá",
    "10 năm đánh giá",
    "15 năm đánh giá",
    "20 năm đánh giá",
    "50 năm đánh giá",
    "70 năm đánh giá",
    "Theo tuổi thọ công trình",
];

/// Nén thông báo để gom: bỏ nội dung trong ngoặc kép, số → `N`. Khuôn của harness TS.
fn nen(s: &str) -> String {
    let mut ra = String::with_capacity(s.len());
    let mut trong_nhay = false;
    let mut vua_so = false;
    for c in s.chars() {
        if c == '"' {
            if trong_nhay {
                ra.push_str("…\"");
                trong_nhay = false;
            } else {
                ra.push('"');
                trong_nhay = true;
            }
            vua_so = false;
            continue;
        }
        if trong_nhay {
            continue;
        }
        if c.is_ascii_digit() {
            if !vua_so {
                ra.push('N');
                vua_so = true;
            }
            continue;
        }
        vua_so = false;
        ra.push(c);
    }
    ra
}

/// Chuỗi JSON tối giản — chỉ cần đủ để đối chứng, không phải bộ mã hoá đầy đủ.
fn nhay(s: &str) -> String {
    // Debug cua Rust thoat dung dau nhay va gach cheo, giu nguyen chu Viet in duoc.
    // Ky tu dieu khien doi thanh dau cach truoc cho JSON hop le.
    let sach: String = s.chars().map(|c| if c.is_control() { ' ' } else { c }).collect();
    format!("{sach:?}")
}

fn ten_ket_luan(k: digitext_mucluc::thoi_gian::KetLuan) -> &'static str {
    use digitext_mucluc::thoi_gian::KetLuan as K;
    match k {
        K::Dung => "dung",
        K::DaDien => "da_dien",
        K::CanhBao => "canh_bao",
        K::Loi => "loi",
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let Some(dir) = args.get(1) else {
        eprintln!("Cách dùng: mucluc-do <thư mục Excel> [THBQ mặc định]");
        std::process::exit(2);
    };
    let thbq_mac_dinh = args.get(2).map(String::as_str);
    let dir = Path::new(dir);
    let danh_muc: Vec<String> = THBQ.iter().map(|s| s.to_string()).collect();
    let vv = vec!["Vĩnh viễn".to_owned()];

    let mut ten: Vec<String> = match std::fs::read_dir(dir) {
        Ok(d) => d
            .filter_map(Result::ok)
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .filter(|n| {
                let l = n.to_ascii_lowercase();
                l.ends_with(".xlsx") || l.ends_with(".xls") || l.ends_with(".xlsm")
            })
            .collect(),
        Err(e) => {
            eprintln!("không đọc được thư mục: {e}");
            std::process::exit(2);
        }
    };
    ten.sort();

    let (mut t_tep, mut t_ho_so, mut t_chan, mut t_loi, mut t_canh_bao) = (0usize, 0usize, 0usize, 0usize, 0usize);
    for n in &ten {
        let SoMucLuc { sheets, sheet, rows, .. } = match doc_so(&dir.join(n), None) {
            Ok(s) => s,
            Err(e) => {
                println!("## {n}  !! {e}");
                continue;
            }
        };
        let pt = phan_tich_muc_luc(
            &rows,
            &TuyChon {
                ten_tep: n,
                sheet: &sheet,
                sheets: &sheets,
                thbq_danh_muc: &danh_muc,
                thbq_vinh_vien: &vv,
                thbq_mac_dinh,
            },
        );

        // Gom thông báo theo thứ tự gặp lần đầu, rồi xếp theo số lần giảm dần (sort ổn định).
        let mut khoa: Vec<String> = Vec::new();
        let mut dem: Vec<usize> = Vec::new();
        for r in &pt.rows {
            for s in r.loi.iter().map(|x| format!("LỖI {x}")).chain(r.canh_bao.iter().map(|x| format!("cb {x}"))) {
                let k = nen(&s);
                match khoa.iter().position(|x| *x == k) {
                    Some(i) => dem[i] += 1,
                    None => {
                        khoa.push(k);
                        dem.push(1);
                    }
                }
            }
        }
        let mut thu_tu: Vec<usize> = (0..khoa.len()).collect();
        thu_tu.sort_by_key(|i| std::cmp::Reverse(dem[*i]));

        let trong = |f: &dyn Fn(&digitext_mucluc::phan_tich::DongHoSo) -> bool| pt.rows.iter().filter(|r| f(r)).count();
        let chan = co_loi_chan(&pt);
        let cat: String = n.chars().take(60).collect();
        println!(
            "## {}  [sheet '{}' /{}] → {} hồ sơ, PB {}{}, cột {} | lỗi {} cảnh báo {} | hộp trống {} THBQ trống {} TG trống {} | {}",
            cat,
            pt.sheet,
            pt.sheets.len(),
            pt.rows.len(),
            pt.phong_bans.len(),
            if pt.khong_chia_phong_ban { " (Chung)" } else { "" },
            pt.cot.khoa().join(","),
            pt.so_loi,
            pt.so_canh_bao,
            trong(&|r| r.hop_so.is_empty()),
            trong(&|r| r.thbq.is_none()),
            trong(&|r| r.tg.bat_dau.is_empty()),
            if chan { "CHẶN" } else { "nhập được" }
        );
        for v in &pt.van_de {
            println!("   [{}] dòng {}: {}", if v.muc == Muc::Loi { "loi" } else { "canh_bao" }, v.dong, v.noi_dung);
        }
        for i in thu_tu.into_iter().take(6) {
            println!("   {}× {}", dem[i], khoa[i]);
        }

        // MUCLUC_XUAT_JSON=<thư mục>: kết xuất từng dòng để đối chứng với harness TS
        // (`KIEM_XUAT_JSON` bên kia). Chỉ dùng khi dò lệch, không phải đường chạy thật.
        if let Ok(ra) = std::env::var("MUCLUC_XUAT_JSON") {
            let _ = std::fs::create_dir_all(&ra);
            let mut j = String::from("[");
            for (k, r) in pt.rows.iter().enumerate() {
                if k > 0 {
                    j.push(',');
                }
                j.push_str(&format!(
                    "{{\"dong\":{},\"ho_so_so\":{},\"bd\":{},\"kt\":{},\"ket_luan\":\"{}\",\"so_loi\":{},\"so_cb\":{},\"tg_goc\":{}}}",
                    r.dong,
                    nhay(&r.ho_so_so),
                    nhay(&r.tg.bat_dau),
                    nhay(&r.tg.ket_thuc),
                    ten_ket_luan(r.tg.ket_luan),
                    r.loi.len(),
                    r.canh_bao.len(),
                    nhay(&r.thoi_gian_goc),
                ));
            }
            j.push(']');
            let ten_ra = Path::new(n).with_extension("json");
            let _ = std::fs::write(Path::new(&ra).join(ten_ra.file_name().unwrap()), j);
        }

        t_tep += 1;
        t_ho_so += pt.rows.len();
        t_chan += usize::from(chan);
        t_loi += pt.so_loi;
        t_canh_bao += pt.so_canh_bao;
    }
    println!("TỔNG: {t_tep} tệp, {t_ho_so} hồ sơ, {t_chan} tệp bị chặn, {t_loi} lỗi, {t_canh_bao} cảnh báo");
}
