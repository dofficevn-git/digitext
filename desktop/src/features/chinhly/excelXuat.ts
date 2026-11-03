// Copyright (c) 2026 DigiText. All rights reserved.
// Xuất Excel chỉnh lý theo mẫu bản cũ: mục lục hồ sơ hoàn thiện (6 kiểu có dòng nhóm), mục lục văn bản,
// metadata TT 02/2019 (32 cột) và TT 05/2025 (51 cột), phụ lục, lịch sử.

import { str, num, type ClRow } from "@/api/chinhly";
import { writeXlsx, type Sheet } from "./excel";
import { collator, khoangNgay, namTuDen, sapXepHoSo } from "./print";

type Cell = string | number | null;
const COT7 = ["Hộp số", "Hồ sơ số", "Tiêu đề hồ sơ", "Ngày tháng BĐ-Ngày tháng KT", "Số tờ", "Thời hạn bảo quản", "Ghi chú"];
const W7 = [8, 10, 46, 30, 8, 35, 35].map((wch) => ({ wch }));

function dong7(r: ClRow, boTap: boolean): Cell[] {
  return [boTap ? (num(r, "bo_so") ?? "") : str(r, "hop_so"), boTap ? (num(r, "tap_so") ?? "") : str(r, "ho_so_so") || str(r, "so_tam"), str(r, "tieu_de"), khoangNgay(r), str(r, "so_to") === "0" ? "" : str(r, "so_to"), str(r, "ten_thbq"), str(r, "ghi_chu")];
}

export type KieuMLHS = 1 | 2 | 3 | 4 | 5 | 6;
export const KIEU_MLHS: [KieuMLHS, string][] = [
  [1, "Mặt hoạt động - Thời gian"],
  [2, "Thời gian - Mặt hoạt động"],
  [3, "THBQ tăng - Mặt hoạt động - Thời gian"],
  [4, "THBQ giảm - Mặt hoạt động - Thời gian"],
  [5, "THBQ tăng - Thời gian - Mặt hoạt động"],
  [6, "THBQ giảm - Thời gian - Mặt hoạt động"],
];

/** Mục lục hồ sơ hoàn thiện (6 kiểu): dòng nhóm gộp 7 cột theo Mảng / Năm / THBQ. `kieu 0` = theo số hồ sơ (kho). */
export async function xuatMucLucHoanThien(rows: ClRow[], kieu: KieuMLHS | 0, boTap: boolean, tenPhong: string): Promise<void> {
  const mang = (a: ClRow, b: ClRow) => (num(a, "mang_so_tt") ?? 1e9) - (num(b, "mang_so_tt") ?? 1e9) || collator.compare(str(a, "ten_mang"), str(b, "ten_mang"));
  const nam = (a: ClRow, b: ClRow) => (num(a, "nam_ket_thuc") ?? 9999) - (num(b, "nam_ket_thuc") ?? 9999);
  const thbq = (a: ClRow, b: ClRow) => (num(a, "thbq_so_tt") ?? 1e9) - (num(b, "thbq_so_tt") ?? 1e9);
  const hs = (a: ClRow, b: ClRow) => collator.compare(str(a, "ho_so_so"), str(b, "ho_so_so")) || collator.compare(str(a, "so_tam"), str(b, "so_tam"));
  const rs = [...rows];
  const tieuDe = ["MẪU XUẤT HOÀN THIỆN THEO SỐ HỒ SƠ", "MẶT HOẠT ĐỘNG-THỜI GIAN", "MẪU XUẤT HOÀN THIỆN THEO THỜI GIAN - MẶT HOẠT ĐỘNG", "MẪU XUẤT HOÀN THIỆN THEO THBQ-MẶT HOẠT ĐỘNG-THỜI GIAN", "MẪU XUẤT HOÀN THIỆN THEO THBQ-MẶT HOẠT ĐỘNG-THỜI GIAN", "MẪU XUẤT HOÀN THIỆN THEO THBQ-THỜI GIAN-MẶT HOẠT ĐỘNG", "MẪU XUẤT HOÀN THIỆN THEO THBQ-THỜI GIAN-MẶT HOẠT ĐỘNG"][kieu];
  type Cmp = (a: ClRow, b: ClRow) => number;
  type Nhom = (r: ClRow) => string;
  const thbqDesc: Cmp = (a, b) => -thbq(a, b);
  const CMP: Record<number, Cmp[]> = { 0: [hs], 1: [mang, nam, hs], 2: [nam, mang, hs], 3: [thbq, mang, nam, hs], 4: [thbqDesc, mang, nam, hs], 5: [thbq, nam, mang, hs], 6: [thbqDesc, nam, mang, hs] };
  const cmp = CMP[kieu];
  rs.sort((a, b) => cmp.reduce((o, f) => o || f(a, b), 0));
  const gMang: Nhom = (r) => str(r, "ten_mang");
  const gMang2: Nhom = (r) => `Mảng: ${str(r, "ten_mang")}`;
  const gNam: Nhom = (r) => `Năm: ${num(r, "nam_ket_thuc") ?? "…"}`;
  const gThbq: Nhom = (r) => `Thời hạn bảo quản: ${str(r, "ten_thbq")}`;
  const NHOM: Record<number, Nhom[]> = { 0: [], 1: [gMang, gNam], 2: [gNam, gMang], 3: [gThbq, gMang2, gNam], 4: [gThbq, gMang2, gNam], 5: [gThbq, gNam, gMang2], 6: [gThbq, gNam, gMang2] };
  const nhom = NHOM[kieu];
  const boTapThat = boTap && kieu !== 5 && kieu !== 6;
  const aoa: Cell[][] = [[tieuDe], [], boTapThat ? ["Bó số", "Tập số", ...COT7.slice(2)] : [...COT7], COT7.map((_, i) => i + 1)];
  const merges = ["A1:G1"];
  const last: string[] = nhom.map(() => "\u0000");
  for (const r of rs) {
    for (let i = 0; i < nhom.length; i++) {
      const label = nhom[i](r);
      if (label !== last[i]) {
        last[i] = label;
        for (let j = i + 1; j < nhom.length; j++) last[j] = "\u0000";
        aoa.push([label]);
        merges.push(`A${aoa.length}:G${aoa.length}`);
      }
    }
    aoa.push(dong7(r, boTapThat));
  }
  const ten = `MucLucHoSo_${kieu}_${tenPhong.replace(/[^\p{L}\p{N}]+/gu, "_")}_${new Date().toISOString().slice(0, 10)}.xlsx`;
  await writeXlsx([{ name: kieu === 0 ? "Project Report" : `Kieu ${kieu}`, aoa, cols: W7, merges }], ten);
}

/** Mục lục văn bản: mỗi hồ sơ một sheet (thay cho zip nhiều tệp của bản cũ). */
export async function xuatMucLucVanBan(hoSos: ClRow[], vanBans: Map<string, ClRow[]>): Promise<void> {
  const sheets: Sheet[] = [];
  const used = new Set<string>();
  for (const h of sapXepHoSo(hoSos)) {
    const vbs = [...(vanBans.get(h.id) ?? [])].sort((a, b) => (num(a, "so_thu_tu") ?? 0) - (num(b, "so_thu_tu") ?? 0));
    const aoa: Cell[][] = [
      ["MỤC LỤC VĂN BẢN"],
      [`Hộp số: ${str(h, "hop_so")}`],
      [`Hồ sơ số: ${str(h, "ho_so_so") || str(h, "so_tam")}`],
      [`Tiêu đề hồ sơ: ${str(h, "tieu_de")}`],
      ["STT", "Số ký hiệu VB", "Ngày tháng VB", "Tác giả", "Trích yếu nội dung", "Tờ số", "Ghi chú"],
      [1, 2, 3, 4, 5, 6, 7],
      ...vbs.map((v) => [num(v, "so_thu_tu") ?? "", [str(v, "so_van_ban"), str(v, "so_ky_hieu")].filter(Boolean).join("/"), str(v, "ngay_thang"), str(v, "co_quan_ban_hanh"), str(v, "trich_yeu"), str(v, "to_so"), str(v, "ghi_chu")] as Cell[]),
    ];
    let name = `${str(h, "hop_so") || "0"}-${str(h, "ho_so_so") || str(h, "so_tam")}`.replace(/[:\\/?*[\]]/g, " ").slice(0, 28) || "HS";
    let k = 1;
    const base = name;
    while (used.has(name.toLowerCase())) name = `${base} ${++k}`.slice(0, 31);
    used.add(name.toLowerCase());
    sheets.push({ name, aoa, merges: ["A1:G1"], cols: [6, 16, 12, 24, 50, 8, 24].map((wch) => ({ wch })) });
  }
  await writeXlsx(sheets.length ? sheets : [{ name: "MLVB", aoa: [["Không có văn bản"]] }], `MucLucVanBan_${new Date().toISOString().slice(0, 10)}.xlsx`);
}

/** Phụ lục hướng dẫn tra tìm (Excel). */
export async function xuatPhuLuc(rows: ClRow[], maPhong: string): Promise<void> {
  const rs = rows.filter((r) => str(r, "hop_so"));
  // Hộp / hồ sơ số đánh riêng cho vĩnh viễn và có thời hạn (0015): tách khoảng theo loại khi phông có cả hai.
  const laVv = (r: ClRow) => r.vinh_vien === true || r.vinh_vien === 1;
  const haiLoai = new Set(rs.map(laVv)).size > 1;
  type G = { pb: string; vv: boolean; mang: string; hs: string[]; hop: string[] };
  const gs = new Map<string, G>();
  for (const r of rs) {
    const k = `${str(r, "ten_phong_ban")}|${laVv(r)}|${str(r, "ten_mang")}`;
    const mang = haiLoai ? `${str(r, "ten_mang")} (${laVv(r) ? "vĩnh viễn" : "có thời hạn"})` : str(r, "ten_mang");
    if (!gs.has(k)) gs.set(k, { pb: str(r, "ten_phong_ban"), vv: laVv(r), mang, hs: [], hop: [] });
    const g = gs.get(k)!;
    if (str(r, "ho_so_so")) g.hs.push(str(r, "ho_so_so"));
    g.hop.push(str(r, "hop_so"));
  }
  const aoa: Cell[][] = [["HƯỚNG DẪN TRA TÌM MỤC LỤC"], [], ["STT", "Nội dung", "Từ hồ sơ", "Đến hồ sơ", "Từ hộp số", "Đến hộp số", "Trang mục lục", "Ghi chú"], [1, 2, 3, 4, 5, 6, 7, 8]];
  const merges = ["A1:H1"];
  let pb = "\u0000";
  let stt = 0;
  for (const g of [...gs.values()].sort((a, b) => collator.compare(a.pb, b.pb) || Number(b.vv) - Number(a.vv) || collator.compare(a.mang, b.mang))) {
    if (g.pb !== pb) {
      pb = g.pb;
      aoa.push([pb || "(Chưa xếp phòng ban)"]);
      merges.push(`A${aoa.length}:H${aoa.length}`);
    }
    const hs = g.hs.sort(collator.compare);
    const hop = g.hop.sort(collator.compare);
    aoa.push([++stt, g.mang, hs[0] ?? "", hs[hs.length - 1] ?? "", hop[0] ?? "", hop[hop.length - 1] ?? "", "", ""]);
  }
  await writeXlsx([{ name: maPhong.slice(0, 31) || "PhuLuc", aoa, merges }], `PhuLucDuAn_${maPhong}_${new Date().toISOString().slice(0, 10)}.xlsx`);
}

// ------------------------------------------------------------------ metadata TT02 / TT05

const pad7 = (n: number | null) => String(n ?? 0).padStart(7, "0");

/** TT 02/2019/TT-BNV: 32 cột, mỗi dòng một văn bản (hồ sơ không có VB vẫn ra 1 dòng). */
export async function xuatTT02(hoSos: ClRow[], vanBans: Map<string, ClRow[]>, tenPhong: string): Promise<void> {
  const head = ["Mã Cơ Quan", "Mã Phông", "Tên Phông", "Mã Hồ Sơ", "Hộp Số", "Hồ sơ số", "Tiêu Đề HS", "Thời gian bắt đầu", "Thời gian kết thúc", "Thời hạn bảo quản", "Ghi chú", "Số lượng tờ", "Tổng số VB", "Số lượng trang", "Chế độ sử dụng", "Ngôn ngữ", "Tình trạng vật lý", "Mã VB", "Số TT VB trong HS", "Tên loại VB", "Số ký hiệu VB", "Ngày/Tháng/Năm VB", "Cơ quan ban hành VB", "Trích yếu nội dung VB", "Tờ số", "Số lượng trang VB", "Ghi chú", "Ngôn ngữ", "Chế độ SD", "Mức độ tin cậy", "Bút tích", "Tình trạng vật lý"];
  const aoa: Cell[][] = [["MÔ TẢ PHÔNG", "", "", "MÔ TẢ HỒ SƠ", ...Array(13).fill(""), "MÔ TẢ VĂN BẢN"], head, head.map((_, i) => i + 1)];
  for (const h of sapXepHoSo(hoSos)) {
    const maHs = ["000", str(h, "ma_don_vi"), str(h, "ma_du_an"), String(num(h, "nam_ket_thuc") ?? ""), str(h, "ho_so_so"), str(h, "ma_mang")].join(".");
    const vbs = [...(vanBans.get(h.id) ?? [])].sort((a, b) => (num(a, "so_thu_tu") ?? 0) - (num(b, "so_thu_tu") ?? 0));
    const hsPart: Cell[] = [str(h, "ma_du_an"), str(h, "ma_don_vi"), str(h, "ten_phong_ban") || tenPhong, maHs, str(h, "hop_so"), str(h, "ho_so_so"), str(h, "tieu_de"), str(h, "ngay_bat_dau"), str(h, "ngay_ket_thuc"), str(h, "ten_thbq"), str(h, "ghi_chu"), str(h, "so_to"), vbs.length, num(h, "so_luong_trang") ?? "", str(h, "che_do_su_dung"), str(h, "ngon_ngu"), str(h, "tinh_trang_vat_ly")];
    if (vbs.length === 0) aoa.push([...hsPart, ...Array(15).fill("")]);
    for (const v of vbs) {
      const stt = num(v, "so_thu_tu");
      aoa.push([...hsPart, `${maHs}.${stt ?? ""}`, stt ?? "", str(v, "ten_loai_vb") || str(v, "loai_vb"), [str(v, "so_van_ban"), str(v, "so_ky_hieu")].filter(Boolean).join("/"), str(v, "ngay_thang"), str(v, "co_quan_ban_hanh"), str(v, "trich_yeu"), str(v, "to_so"), num(v, "so_trang") ?? "", str(v, "ghi_chu"), str(v, "ngon_ngu"), str(v, "che_do_su_dung"), str(v, "do_tin_cay"), str(v, "but_tich"), str(v, "tinh_trang_vat_ly")]);
    }
  }
  await writeXlsx([{ name: "XUẤT HỒ SƠ SỐ HÓA", aoa, merges: ["A1:C1", "D1:Q1", "R1:AF1"] }], `TrichXuat_TT02_${new Date().toISOString().slice(0, 10)}.xlsx`);
}

/** Trạng thái duyệt của một văn bản (cột 52 ngoài chuẩn TT05, 2/11/2026): máy tạo từ OCR đã / chưa có người duyệt, hay nhập tay. */
export function trangThaiDuyet(v: ClRow): string {
  if (str(v, "nguon") !== "ocr") return "Nhập tay";
  return str(v, "duyet_luc") ? "Đã duyệt" : "Chưa duyệt (OCR)";
}

/**
 * TT 05/2025/TT-BNV: 51 cột theo "Mẫu metadata chuẩn 05.xlsx" (9 phông · 19 hồ sơ · 23 văn bản), mã THBQ 01–07.
 * `tepCua`: văn bản id → tên tệp quét thật (cl_van_ban_file, 2/11/2026) → "Mã định danh / Mã lưu trữ tài liệu" = tên tệp bỏ
 * .pdf, "Tệp tin tài liệu" = đường dẫn tới tệp thật; không có thì ghép như cũ. Cột 52 "Duyệt" thêm ngoài 51 cột chuẩn.
 */
export async function xuatTT05(hoSos: ClRow[], vanBans: Map<string, ClRow[]>, phong: ClRow | undefined, tepCua: Map<string, string> = new Map(), luuVao?: string): Promise<void> {
  const head = [
    "Mã Phông", "Tên Phông", "Lịch sử đơn vị hình thành phông", "Thời gian tài liệu", "Tổng số tài liệu", "Phương án phân loại", "Ngôn ngữ", "Chế độ dự phòng", "Ghi chú",
    "Mã hồ sơ", "Tiêu đề hồ sơ", "Thời hạn lưu trữ", "Mức độ tiếp cận", "Ngôn ngữ", "Thời gian bắt đầu", "Thời gian kết thúc", "Từ khóa", "Tổng số tài liệu trong hồ sơ", "Số lượng tờ", "Số lượng trang", "Tình trạng vật lý", "Ký hiệu thông tin", "Mức độ tin cậy", "Mã hồ sơ gốc giấy", "Chế độ lập tài liệu dự phòng", "Tình trạng lập tài liệu dự phòng", "Tệp tin hồ sơ", "Ghi chú",
    "Mã định danh tài liệu", "Mã lưu trữ tài liệu", "Thời hạn lưu trữ", "Tên loại tài liệu", "Số của tài liệu", "Ký hiệu của tài liệu", "Ngày tháng năm", "Cơ quan ban hành", "Trích yếu", "Ngôn ngữ", "Số lượng trang", "Số lượng tờ", "Ký hiệu thông tin", "Từ khóa", "Mức độ tiếp cận", "Mức độ tin cậy", "Bút tích", "Tình trạng vật lý", "Quy trình xử lý", "Chế độ dự phòng", "Tình trạng dự phòng", "Tệp tin tài liệu", "Ghi chú",
    "Duyệt",
  ];
  const rs = sapXepHoSo(hoSos);
  const y = namTuDen(rs);
  const ma = str(phong, "ma_du_an");
  const aoa: Cell[][] = [
    ["FILE METADATA NHẬP VÀO PHẦN MỀM (CÁC TRƯỜNG THÔNG TIN ĐẦY ĐỦ THEO THÔNG TƯ 05/2025/TT-BNV) — cột 52 \"Duyệt\" ngoài chuẩn: trạng thái duyệt văn bản máy tạo từ OCR"],
    ["MÔ TẢ PHÔNG", ...Array(8).fill(""), "MÔ TẢ HỒ SƠ", ...Array(18).fill(""), "MÔ TẢ VĂN BẢN"],
    head,
    head.map((_, i) => i + 1),
  ];
  const phongPart: Cell[] = [ma, str(phong, "ten"), "", y.tu ? `${y.tu}-${y.den}` : "", "", "", "01", "1", str(phong, "ghi_chu")];
  for (const h of rs) {
    const maHs = str(h, "ma_ho_so");
    const vbs = [...(vanBans.get(h.id) ?? [])].sort((a, b) => (num(a, "so_thu_tu") ?? 0) - (num(b, "so_thu_tu") ?? 0));
    const tepHs = [ma, str(h, "ma_mang"), str(h, "hop_so"), str(h, "ho_so_so")].filter(Boolean).join("\\");
    const hsPart: Cell[] = [
      maHs, str(h, "tieu_de"), str(h, "thbq_ma_tt05") || "07", str(h, "che_do_su_dung") || "01", str(h, "ngon_ngu") || "01", str(h, "ngay_bat_dau"), str(h, "ngay_ket_thuc"), str(h, "tu_khoa"), vbs.length, str(h, "so_to"), num(h, "so_luong_trang") ?? "",
      str(h, "tinh_trang_vat_ly") || "02", str(h, "ky_hieu_thong_tin"), str(h, "muc_do_tin_cay") || "03", str(h, "ma_ho_so_giay") || [ma, str(h, "hop_so"), str(h, "ho_so_so")].filter(Boolean).join("."), "1", "01", tepHs, str(h, "ghi_chu"),
    ];
    if (vbs.length === 0) aoa.push([...phongPart, ...hsPart, ...Array(24).fill("")]);
    for (const v of vbs) {
      const stt = num(v, "so_thu_tu");
      const tenTep = tepCua.get(v.id);
      const maVb = tenTep ? tenTep.replace(/\.pdf$/i, "") : maHs ? `${maHs}.${pad7(stt)}` : "";
      const tepVb = tenTep ? (tepHs ? `${tepHs}\\${tenTep}` : tenTep) : tepHs ? `${tepHs}\\${pad7(stt)}.pdf` : "";
      aoa.push([
        ...phongPart, ...hsPart,
        maVb, maVb, str(h, "thbq_ma_tt05") || "07", str(v, "loai_vb") || "32", str(v, "so_van_ban"), str(v, "so_ky_hieu"), str(v, "ngay_thang"), str(v, "co_quan_ban_hanh"), str(v, "trich_yeu"), str(v, "ngon_ngu") || "01", num(v, "so_trang") ?? "", num(v, "to_so") ?? "",
        str(v, "ky_hieu_thong_tin"), str(v, "tu_khoa"), str(v, "che_do_su_dung") || "01", str(v, "do_tin_cay") || "03", str(v, "but_tich"), str(v, "tinh_trang_vat_ly") || "02", "1", "1", "01", tepVb, str(v, "ghi_chu"),
        trangThaiDuyet(v),
      ]);
    }
  }
  await writeXlsx([{ name: "HỒ SƠ DỮ LIỆU (TT05)", aoa, merges: ["A1:AZ1", "A2:I2", "J2:AB2", "AC2:AZ2"], cols: head.map((c) => ({ wch: Math.min(28, Math.max(10, c.length + 2)) })) }], luuVao ? `MLHS-TT05-${(str(phong, "ma_du_an") || "phong").replace(/[\\/:*?"<>|]+/g, "-")}.xlsx` : `TrichXuatDuLieu_TT05_${new Date().toISOString().slice(0, 10)}.xlsx`, luuVao);
}

/** Lịch sử hồ sơ / văn bản (Excel): STT · Thao tác · Ngày · Giờ · Người · Tóm tắt · Nội dung · Số trường mới · Số trường sửa. */
export async function xuatLichSu(rows: ClRow[], ten: string): Promise<void> {
  const HD: Record<string, string> = { add: "Thêm mới", edit: "Sửa", delete: "Xoá", restore: "Khôi phục", import: "Nhập Excel", systematize: "Đánh số", boxing: "Gán hộp/bó/tập", assign_gia: "Xếp giá", ocr: "Máy tạo từ OCR" };
  const aoa: Cell[][] = [["STT", "Thao tác", "Ngày thực hiện", "Giờ thực hiện", "Người thực hiện", "Đối tượng", "Tóm tắt", "Nội dung", "Số trường mới", "Số trường sửa"]];
  let tong = 0;
  rows.forEach((r, i) => {
    const at = new Date(str(r, "at"));
    const diffs = (r.diffs as { f: string; o: unknown; n: unknown }[] | undefined) ?? [];
    const nd = diffs.map((d) => `${d.f}: ${d.o ?? ""} → ${d.n ?? ""}`).join(" - ");
    const nm = num(r, "n_new") ?? 0;
    const ns = num(r, "n_edited") ?? 0;
    tong += nm + ns;
    aoa.push([i + 1, HD[str(r, "action")] ?? str(r, "action"), Number.isNaN(at.getTime()) ? "" : at.toLocaleDateString("vi-VN"), Number.isNaN(at.getTime()) ? "" : at.toLocaleTimeString("vi-VN", { hour: "2-digit", minute: "2-digit" }), str(r, "actor_name"), str(r, "entity") === "van_ban" ? "Văn bản" : "Hồ sơ", `${str(r, "so_tam") ? `HS ${str(r, "so_tam")} · ` : ""}${str(r, "title")}`, nd, nm, ns]);
  });
  aoa.push(["", "Tổng số trường thông tin đã cập nhật", "", "", "", "", "", "", tong, ""]);
  await writeXlsx([{ name: ten.slice(0, 31) || "LichSu", aoa, cols: [5, 14, 12, 8, 20, 10, 40, 60, 10, 10].map((wch) => ({ wch })) }], `lichsu_${new Date().toISOString().slice(0, 10)}.xlsx`);
}
