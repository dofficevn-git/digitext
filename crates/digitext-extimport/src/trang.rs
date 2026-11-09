// Copyright (c) 2026 DigiText. All rights reserved.
//! Đo chất lượng từng TRANG của một PDF đã quét: trang trắng, trang lệch, trang quét ngang.
//!
//! Chỉ đọc ảnh JPEG nhúng sẵn trong PDF (`digitext_pdf::page_jpeg`) — không dựng hình lại, không
//! cần Docker hay poppler, nên chạy được ngay trên máy người dùng cùng lúc với kiểm PDF/A.
//! Trang nào không phải ảnh JPEG (PDF chữ thật, ảnh CCITT/JBIG2) thì bỏ qua và đếm vào `khong_do`.
//!
//! Cách đo, cả hai đều làm trên bản xám đã rút nhỏ còn ~900 px bề ngang:
//!
//! * **Mực**: dựng biểu đồ xám, lấy mức sáng ở phân vị 90 làm NỀN GIẤY rồi đặt ngưỡng dưới nền
//!   `CHENH_NEN` mức. Làm vậy để giấy ố vàng hay bản quét xám vẫn đo đúng, thay vì so với màu
//!   trắng tuyệt đối. Tỉ lệ điểm tối hơn ngưỡng chính là lượng mực. Bỏ 3 % mép vì gáy sách, bóng
//!   nắp máy quét và lỗ ghim đều nằm ở đó.
//! * **Góc nghiêng**: lấy toạ độ các điểm mực rồi chiếu lên trục dọc ở từng góc thử. Dòng chữ nằm
//!   ngang nên khi quay đúng góc, các điểm dồn vào ít hàng — tổng bình phương biểu đồ chiếu đạt
//!   đỉnh. Quét thô ±6° bước 0,5° rồi dò lại quanh đỉnh bước 0,1°.
//!
//! Thêm một phép so: chiếu theo trục ngang (giả thuyết dòng chữ dựng đứng). Nếu điểm cao hơn hẳn
//! thì trang bị quét ngang 90°, không phải lệch vài độ.

use serde::{Deserialize, Serialize};

/// Dưới mức này coi là trang trắng (tỉ lệ điểm có mực).
///
/// Đo trên 332 trang quét thật của Bắc Giang (9/11/2026): trang ÍT mực nhất mà vẫn có nội dung
/// là 1,14 % (bảng 7 dòng ở đầu trang, phần còn lại để trống). Ngưỡng 0,15 % nằm dưới mức đó
/// khoảng 7 lần nên không bắt nhầm trang thưa chữ.
pub const MUC_TRANG_TRANG: f32 = 0.0015;
/// Từ `MUC_TRANG_TRANG` đến mức này thì chưa dám kết luận — báo "gần như trắng" để người soát xem lại.
pub const MUC_GAN_TRANG: f32 = 0.006;
/// Từ mức này coi là trang lệch (độ).
pub const GOC_LECH_TOI_THIEU: f32 = 1.0;
/// Bề ngang ảnh rút nhỏ để đo.
const DO_RONG_DO: u32 = 900;
/// Bỏ bao nhiêu phần mép mỗi bên.
const VIEN: f32 = 0.03;
/// Ngưỡng mực = nền giấy trừ đi chừng này mức xám.
const CHENH_NEN: u16 = 60;
/// Ít hơn chừng này điểm mực thì không đủ căn cứ đoán góc.
const DIEM_TOI_THIEU: usize = 400;
/// Lấy nhiều nhất chừng này điểm mực để đo góc (trang đặc chữ không làm chậm cả đợt).
const DIEM_TOI_DA: usize = 40_000;
/// Chiếu trục ngang hơn trục dọc chừng này lần thì coi là trang quét ngang.
const NGANG_HON: f64 = 1.35;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TrangDo {
    /// Số trang, đếm từ 1.
    pub trang: u32,
    /// Tỉ lệ điểm có mực, 0..1.
    pub muc: f32,
    /// Góc nghiêng ước lượng (độ, dương = nghiêng theo chiều kim đồng hồ). `None` = không đoán được.
    pub goc: Option<f32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct KetQuaTrang {
    /// Số trang đo được.
    pub da_do: u32,
    /// Số trang không lấy được ảnh JPEG nên không đo.
    pub khong_do: u32,
    pub trang_trang: Vec<TrangDo>,
    /// Rất ít mực nhưng chưa chắc trắng — nên mở ra xem.
    pub trang_gan_trang: Vec<TrangDo>,
    pub trang_lech: Vec<TrangDo>,
    /// Trang quét ngang 90° (chữ dựng đứng).
    pub trang_ngang: Vec<u32>,
    /// Góc lệch lớn nhất gặp được (độ).
    pub goc_lon_nhat: f32,
}

impl KetQuaTrang {
    pub fn co_van_de(&self) -> bool {
        !self.trang_trang.is_empty()
            || !self.trang_gan_trang.is_empty()
            || !self.trang_lech.is_empty()
            || !self.trang_ngang.is_empty()
    }
    /// Câu mô tả ngắn để đưa vào danh sách vấn đề của tệp.
    pub fn mo_ta(&self) -> Vec<String> {
        let ds = |v: &[TrangDo]| -> String {
            let dau = v.iter().take(12).map(|t| t.trang.to_string()).collect::<Vec<_>>().join(", ");
            if v.len() > 12 {
                format!("{dau} … và {} trang nữa", v.len() - 12)
            } else {
                dau
            }
        };
        let mut out = Vec::new();
        if !self.trang_trang.is_empty() {
            out.push(format!("{} trang trắng (trang {})", self.trang_trang.len(), ds(&self.trang_trang)));
        }
        if !self.trang_gan_trang.is_empty() {
            out.push(format!(
                "{} trang gần như trắng, nên xem lại (trang {})",
                self.trang_gan_trang.len(),
                ds(&self.trang_gan_trang)
            ));
        }
        if !self.trang_lech.is_empty() {
            out.push(format!(
                "{} trang lệch quá {:.1}° (lớn nhất {:.1}°, trang {})",
                self.trang_lech.len(),
                GOC_LECH_TOI_THIEU,
                self.goc_lon_nhat,
                ds(&self.trang_lech)
            ));
        }
        if !self.trang_ngang.is_empty() {
            let s = self.trang_ngang.iter().take(12).map(u32::to_string).collect::<Vec<_>>().join(", ");
            out.push(format!("{} trang quét ngang 90° (trang {s})", self.trang_ngang.len()));
        }
        out
    }
}

/// Ảnh xám đã rút nhỏ: (điểm ảnh, bề ngang, bề cao).
fn xam_nho(jpeg: &[u8]) -> Option<(Vec<u8>, u32, u32)> {
    let anh = image::load_from_memory_with_format(jpeg, image::ImageFormat::Jpeg).ok()?;
    let xam = anh.to_luma8();
    let (w, h) = (xam.width(), xam.height());
    if w == 0 || h == 0 {
        return None;
    }
    if w <= DO_RONG_DO {
        return Some((xam.into_raw(), w, h));
    }
    // Lấy mẫu thưa đều — nhanh hơn nội suy và đủ cho việc đếm mực / đoán góc.
    let buoc = (w as f32 / DO_RONG_DO as f32).max(1.0);
    let nw = (w as f32 / buoc).floor().max(1.0) as u32;
    let nh = (h as f32 / buoc).floor().max(1.0) as u32;
    let goc = xam.into_raw();
    let mut ra = Vec::with_capacity((nw * nh) as usize);
    for y in 0..nh {
        let sy = (y as f32 * buoc) as u32;
        let hang = (sy as usize) * (w as usize);
        for x in 0..nw {
            let sx = (x as f32 * buoc) as u32;
            ra.push(goc[hang + sx as usize]);
        }
    }
    Some((ra, nw, nh))
}

/// Ngưỡng mực: nền giấy (phân vị 90) trừ `CHENH_NEN`.
fn nguong_muc(diem: &[u8]) -> u8 {
    let mut bd = [0u32; 256];
    for p in diem {
        bd[*p as usize] += 1;
    }
    let moc = (diem.len() as f64 * 0.90) as u32;
    let mut cong = 0u32;
    let mut nen = 255u16;
    for (v, n) in bd.iter().enumerate() {
        cong += n;
        if cong >= moc {
            nen = v as u16;
            break;
        }
    }
    nen.saturating_sub(CHENH_NEN).clamp(40, 220) as u8
}

/// Toạ độ các điểm có mực bên trong vùng đã bỏ mép, kèm tỉ lệ mực.
fn diem_muc(diem: &[u8], w: u32, h: u32, nguong: u8) -> (Vec<(f32, f32)>, f32) {
    let x0 = (w as f32 * VIEN) as u32;
    let y0 = (h as f32 * VIEN) as u32;
    let x1 = w.saturating_sub(x0).max(x0 + 1);
    let y1 = h.saturating_sub(y0).max(y0 + 1);
    let tong = ((x1 - x0) as usize) * ((y1 - y0) as usize);
    if tong == 0 {
        return (Vec::new(), 0.0);
    }
    let mut ds = Vec::new();
    let mut dem = 0usize;
    for y in y0..y1 {
        let hang = (y as usize) * (w as usize);
        for x in x0..x1 {
            if diem[hang + x as usize] < nguong {
                dem += 1;
                ds.push((x as f32, y as f32));
            }
        }
    }
    // Thưa bớt cho đều nếu quá nhiều.
    if ds.len() > DIEM_TOI_DA {
        let buoc = ds.len() / DIEM_TOI_DA + 1;
        ds = ds.into_iter().step_by(buoc).collect();
    }
    (ds, dem as f32 / tong as f32)
}

/// Tổng bình phương biểu đồ chiếu ở một góc — càng cao nghĩa là dòng chữ càng dồn vào ít hàng.
fn diem_chieu(ds: &[(f32, f32)], goc_rad: f32, cao: f32, doc: bool) -> f64 {
    let (s, c) = goc_rad.sin_cos();
    let n = (cao.ceil() as usize) + 2;
    let mut bd = vec![0u32; n * 2];
    for (x, y) in ds {
        let p = if doc { y * c - x * s } else { x * c + y * s };
        let i = (p + cao) as isize;
        if i >= 0 && (i as usize) < bd.len() {
            bd[i as usize] += 1;
        }
    }
    bd.iter().map(|v| (*v as f64) * (*v as f64)).sum()
}

/// Góc nghiêng (độ) và có phải trang quét ngang không.
fn doan_goc(ds: &[(f32, f32)], w: u32, h: u32) -> (Option<f32>, bool) {
    if ds.len() < DIEM_TOI_THIEU {
        return (None, false);
    }
    let cao = h.max(w) as f32;
    let quet = |tho: f32, min: f32, max: f32, buoc: f32, doc: bool| -> (f32, f64) {
        let mut tot = (tho, f64::MIN);
        let mut a = min;
        while a <= max + 1e-6 {
            let d = diem_chieu(ds, a.to_radians(), cao, doc);
            if d > tot.1 {
                tot = (a, d);
            }
            a += buoc;
        }
        tot
    };
    let (tho, _) = quet(0.0, -6.0, 6.0, 0.5, true);
    let (goc, diem_doc) = quet(tho, tho - 0.5, tho + 0.5, 0.1, true);
    // Giả thuyết chữ dựng đứng: chiếu theo trục còn lại.
    let (_, diem_ngang) = quet(0.0, -6.0, 6.0, 0.5, false);
    let ngang = diem_ngang > diem_doc * NGANG_HON;
    (Some(goc), ngang)
}

/// Đo MỘT trang từ ảnh JPEG của nó: (lượng mực, góc nghiêng, có phải quét ngang).
/// Trả `None` nếu không giải nén được ảnh.
pub fn do_mot_trang(jpeg: &[u8]) -> Option<(f32, Option<f32>, bool)> {
    let (diem, w, h) = xam_nho(jpeg)?;
    let nguong = nguong_muc(&diem);
    let (ds, muc) = diem_muc(&diem, w, h, nguong);
    if muc < MUC_TRANG_TRANG {
        return Some((muc, None, false));
    }
    let (goc, ngang) = doan_goc(&ds, w, h);
    Some((muc, goc, ngang))
}

/// Đo mọi trang của một PDF. Trả `None` nếu không mở được tệp.
pub fn do_pdf(bytes: &[u8], so_trang: u32) -> Option<KetQuaTrang> {
    let parsed = digitext_pdf::ParsedPdf::parse(bytes).ok()?;
    let mut kq = KetQuaTrang::default();
    for trang in 1..=so_trang {
        let Ok((jpeg, _dpi)) = digitext_pdf::page_jpeg(&parsed, trang as i32) else {
            kq.khong_do += 1;
            continue;
        };
        let Some((muc, goc, ngang)) = do_mot_trang(&jpeg) else {
            kq.khong_do += 1;
            continue;
        };
        kq.da_do += 1;
        if muc < MUC_TRANG_TRANG {
            kq.trang_trang.push(TrangDo { trang, muc, goc: None });
            continue;
        }
        if muc < MUC_GAN_TRANG {
            kq.trang_gan_trang.push(TrangDo { trang, muc, goc });
        }
        if ngang {
            kq.trang_ngang.push(trang);
        }
        if let Some(g) = goc {
            if g.abs() > kq.goc_lon_nhat.abs() {
                kq.goc_lon_nhat = g;
            }
            if g.abs() >= GOC_LECH_TOI_THIEU && !ngang {
                kq.trang_lech.push(TrangDo { trang, muc, goc: Some(g) });
            }
        }
    }
    Some(kq)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Dựng ảnh xám giả: nền trắng, vẽ `so_dong` vạch đen nghiêng `goc` độ.
    fn anh_gia(w: u32, h: u32, so_dong: u32, goc: f32) -> Vec<u8> {
        let mut v = vec![255u8; (w * h) as usize];
        let t = goc.to_radians().tan();
        for i in 0..so_dong {
            let y0 = (h as f32) * (i as f32 + 1.0) / (so_dong as f32 + 1.0);
            for x in (w / 10)..(w * 9 / 10) {
                let y = y0 + t * (x as f32 - w as f32 / 2.0);
                for d in 0..3 {
                    let yy = y as i32 + d;
                    if yy >= 0 && (yy as u32) < h {
                        v[(yy as u32 * w + x) as usize] = 0;
                    }
                }
            }
        }
        v
    }

    #[test]
    fn trang_trang_thi_muc_gan_bang_khong() {
        let v = vec![250u8; 400 * 500];
        let ng = nguong_muc(&v);
        let (_, muc) = diem_muc(&v, 400, 500, ng);
        assert!(muc < MUC_TRANG_TRANG, "muc = {muc}");
    }

    #[test]
    fn giay_o_vang_van_dem_dung_muc() {
        // Nền xám 200 (giấy cũ) + vài vạch đen: vẫn phải thấy mực, không coi là trang trắng.
        let mut v = vec![200u8; 400 * 500];
        for y in 100..110 {
            for x in 40..360 {
                v[y * 400 + x] = 20;
            }
        }
        let ng = nguong_muc(&v);
        let (_, muc) = diem_muc(&v, 400, 500, ng);
        assert!(muc > MUC_TRANG_TRANG, "muc = {muc}");
    }

    #[test]
    fn doan_dung_goc_nghieng() {
        for that in [-3.0f32, -1.5, 0.0, 2.0, 4.0] {
            let v = anh_gia(600, 800, 20, that);
            let ng = nguong_muc(&v);
            let (ds, _) = diem_muc(&v, 600, 800, ng);
            let (goc, ngang) = doan_goc(&ds, 600, 800);
            let g = goc.expect("phải đoán được góc");
            assert!((g - that).abs() < 0.6, "thật {that}, đoán {g}");
            assert!(!ngang, "không phải trang ngang");
        }
    }

    #[test]
    fn it_muc_qua_thi_khong_doan_bua() {
        let ds: Vec<(f32, f32)> = (0..50).map(|i| (i as f32, i as f32)).collect();
        assert_eq!(doan_goc(&ds, 600, 800).0, None);
    }
}
