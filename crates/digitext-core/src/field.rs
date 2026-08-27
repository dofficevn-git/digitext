// Copyright (c) 2026 DigiText. All rights reserved.
use std::fmt;
use std::str::FromStr;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VanBanField {
    ThuTu,
    LoaiVanBan,
    SoKyHieu,
    NgayThang,
    CoQuanBanHanh,
    TrichYeu,
    ToSo,
    SoTrang,
    TrangSo,
    NgonNgu,
    CheDoSuDung,
    DoTinCay,
    ButTich,
    TinhTrangVatLy,
    TaiLieuSo,
    GhiChu,
}
impl VanBanField {
    pub const ALL: [VanBanField; 16] = [
        VanBanField::ThuTu,
        VanBanField::LoaiVanBan,
        VanBanField::SoKyHieu,
        VanBanField::NgayThang,
        VanBanField::CoQuanBanHanh,
        VanBanField::TrichYeu,
        VanBanField::ToSo,
        VanBanField::SoTrang,
        VanBanField::TrangSo,
        VanBanField::NgonNgu,
        VanBanField::CheDoSuDung,
        VanBanField::DoTinCay,
        VanBanField::ButTich,
        VanBanField::TinhTrangVatLy,
        VanBanField::TaiLieuSo,
        VanBanField::GhiChu,
    ];
    pub fn as_str(self) -> &'static str {
        match self {
            VanBanField::ThuTu => "thuTu",
            VanBanField::LoaiVanBan => "loaiVanBan",
            VanBanField::SoKyHieu => "soKyHieu",
            VanBanField::NgayThang => "ngayThang",
            VanBanField::CoQuanBanHanh => "coQuanBanHanh",
            VanBanField::TrichYeu => "trichYeu",
            VanBanField::ToSo => "toSo",
            VanBanField::SoTrang => "soTrang",
            VanBanField::TrangSo => "trangSo",
            VanBanField::NgonNgu => "ngonNgu",
            VanBanField::CheDoSuDung => "cheDoSuDung",
            VanBanField::DoTinCay => "doTinCay",
            VanBanField::ButTich => "butTich",
            VanBanField::TinhTrangVatLy => "tinhTrangVatLy",
            VanBanField::TaiLieuSo => "taiLieuSo",
            VanBanField::GhiChu => "ghiChu",
        }
    }
}
impl fmt::Display for VanBanField {
    fn fmt(&self, r_1yi: &mut fmt::Formatter<'_>) -> fmt::Result {
        r_1yi.write_str(self.as_str())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("trường văn bản không hợp lệ: {0:?}")]
pub struct Tgr(pub String);
impl FromStr for VanBanField {
    type Err = Tgr;
    fn from_str(r_1yk: &str) -> Result<Self, Self::Err> {
        VanBanField::ALL
            .into_iter()
            .find(|r_1yj| r_1yj.as_str() == r_1yk)
            .ok_or_else(|| Tgr(r_1yk.to_owned()))
    }
}
impl TryFrom<String> for VanBanField {
    type Error = Tgr;
    fn try_from(r_1yl: String) -> Result<Self, Self::Error> {
        r_1yl.parse()
    }
}
