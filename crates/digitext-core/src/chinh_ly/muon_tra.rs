// Copyright (c) 2026 DigiText. All rights reserved.
//! Mượn – trả hồ sơ giấy CÓ DUYỆT (27/11/2026, đối chiếu Digifile TT05 L19/F14; migration 0034). Logic thuần dùng chung máy chủ
//! (`digitext_db::chinh_ly::muon_tra`, bộ chặn trong `sync::apply_inner`) và desktop (`muonTraFlow.ts` chép lại bảng chuyển).
//!
//! cho_duyet ──duyệt──► da_duyet ──giao──► dang_muon ──trả──► da_tra
//!     │ từ chối → tu_choi      │ huỷ → huy          │ gia hạn (tối đa GIA_HAN_TOI_DA lần, giữ `han_tra_goc`)
//!     │ huỷ → huy
//! Lập phiếu trực tiếp `dang_muon` (lập và giao luôn) chỉ khi người lập có `loan.approve`. Bản sao offline KHÔNG đổi trạng thái.
use crate::Permission;

pub const CHO_DUYET: &str = "cho_duyet";
pub const DA_DUYET: &str = "da_duyet";
pub const TU_CHOI: &str = "tu_choi";
pub const DANG_MUON: &str = "dang_muon";
pub const DA_TRA: &str = "da_tra";
pub const HUY: &str = "huy";

pub const TRANG_THAI: [&str; 6] = [CHO_DUYET, DA_DUYET, TU_CHOI, DANG_MUON, DA_TRA, HUY];

/// Số lần gia hạn tối đa cho một phiếu (Digifile không nêu; chọn 2 để lần thứ ba buộc trả rồi lập phiếu mới).
pub const GIA_HAN_TOI_DA: i32 = 2;
/// Nhắc "sắp đến hạn" trước bao nhiêu ngày; quá hạn nhắc lại mỗi `NHAC_QUA_HAN_MOI_NGAY` ngày.
pub const NHAC_TRUOC_NGAY: i64 = 3;
pub const NHAC_QUA_HAN_MOI_NGAY: i64 = 7;

pub fn hop_le(tt: &str) -> bool {
    TRANG_THAI.contains(&tt)
}

/// Phiếu còn "mở" (chiếm hồ sơ về mặt nghiệp vụ): chờ duyệt, đã duyệt chờ giao, đang mượn.
pub fn mo(tt: &str) -> bool {
    matches!(tt, CHO_DUYET | DA_DUYET | DANG_MUON)
}

/// Bảng chuyển trạng thái. `tu == den` không phải chuyển.
pub fn chuyen_duoc(tu: &str, den: &str) -> bool {
    matches!((tu, den), (CHO_DUYET, DA_DUYET | TU_CHOI | HUY) | (DA_DUYET, DANG_MUON | HUY) | (DANG_MUON, DA_TRA))
}

/// Quyền cần có để đưa phiếu VÀO trạng thái `den` (duyệt / từ chối: `loan.approve`; còn lại `loan.manage`).
pub fn quyen_chuyen(den: &str) -> Permission {
    match den {
        DA_DUYET | TU_CHOI => Permission::LoanApprove,
        _ => Permission::LoanManage,
    }
}

/// Trạng thái được phép khi LẬP phiếu: `cho_duyet` với `loan.manage`; `dang_muon` (lập và giao luôn) cần `loan.approve`.
pub fn tao_duoc(tt: &str, co_approve: bool) -> bool {
    tt == CHO_DUYET || (tt == DANG_MUON && co_approve)
}

/// Số phiếu: `MT-{năm}-{stt:04}`.
pub fn so_phieu(nam: i32, stt: i64) -> String {
    format!("MT-{nam}-{stt:04}")
}

pub fn ten_trang_thai(tt: &str) -> &'static str {
    match tt {
        CHO_DUYET => "Chờ duyệt",
        DA_DUYET => "Đã duyệt, chờ giao",
        TU_CHOI => "Từ chối",
        DANG_MUON => "Đang mượn",
        DA_TRA => "Đã trả",
        HUY => "Đã huỷ",
        _ => "Không rõ",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bang_chuyen() {
        assert!(chuyen_duoc(CHO_DUYET, DA_DUYET) && chuyen_duoc(CHO_DUYET, TU_CHOI) && chuyen_duoc(CHO_DUYET, HUY));
        assert!(chuyen_duoc(DA_DUYET, DANG_MUON) && chuyen_duoc(DA_DUYET, HUY));
        assert!(chuyen_duoc(DANG_MUON, DA_TRA));
        assert!(!chuyen_duoc(CHO_DUYET, DANG_MUON), "chưa duyệt không được giao");
        assert!(!chuyen_duoc(DANG_MUON, HUY), "đang mượn chỉ còn đường trả");
        assert!(!chuyen_duoc(DA_TRA, DANG_MUON) && !chuyen_duoc(TU_CHOI, DA_DUYET) && !chuyen_duoc(HUY, CHO_DUYET));
        assert!(!chuyen_duoc(DANG_MUON, DANG_MUON));
        for t in TRANG_THAI {
            assert!(hop_le(t));
        }
        assert!(!hop_le("dang_cho"));
    }

    #[test]
    fn quyen_va_lap() {
        assert_eq!(quyen_chuyen(DA_DUYET), Permission::LoanApprove);
        assert_eq!(quyen_chuyen(TU_CHOI), Permission::LoanApprove);
        assert_eq!(quyen_chuyen(DANG_MUON), Permission::LoanManage);
        assert_eq!(quyen_chuyen(DA_TRA), Permission::LoanManage);
        assert!(tao_duoc(CHO_DUYET, false) && !tao_duoc(DANG_MUON, false) && tao_duoc(DANG_MUON, true) && !tao_duoc(DA_TRA, true));
        assert_eq!(so_phieu(2026, 7), "MT-2026-0007");
        assert!(mo(CHO_DUYET) && mo(DA_DUYET) && mo(DANG_MUON) && !mo(DA_TRA) && !mo(HUY) && !mo(TU_CHOI));
    }
}
