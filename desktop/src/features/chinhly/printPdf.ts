// Copyright (c) 2026 DigiText. All rights reserved.
// Biểu mẫu chỉnh lý ra PDF TẠI MÁY TRẠM (27/11/2026, đợt 4 nhóm 3C): cùng dữ liệu và cách sắp xếp với print.ts (HTML), nhưng dựng
// bằng bộ dựng PDF chung (`pdf_manual`) → tệp PDF thật, phông chữ nhúng, không phụ thuộc trình duyệt. HTML `window.print` giữ làm
// "In nhanh" một vài bản phát hành rồi bỏ (đề xuất câu 18). Nhãn hộp 2×2 ra PDF từ đợt 5 (27/11/2026).

import { str, num, type ClRow } from "@/api/chinhly";
import { kb, pdfManual, type KhoiIn } from "@/api/inPdf";
import { collator, hoaDau, khoangNgay, namTuDen, nhanHopData, sapXepHoSo, soThanhChu } from "./print";

const COT_MLHS = ["Hộp số", "Hồ sơ số", "Tiêu đề hồ sơ", "Ngày tháng bắt đầu – kết thúc", "Số tờ", "Thời hạn bảo quản", "Ghi chú"];
const RONG_MLHS = [7, 8, 46, 18, 7, 13, 12];
const ngayKy = () => "…………, ngày …… tháng …… năm 20……";

function dongHoSo(r: ClRow): string[] {
  return [str(r, "hop_so"), str(r, "ho_so_so") || str(r, "so_tam"), str(r, "tieu_de"), khoangNgay(r), str(r, "so_to") === "0" ? "" : str(r, "so_to"), str(r, "ten_thbq"), str(r, "ghi_chu")];
}
function dau(tenPhong: string, tenPhongBan: string): KhoiIn {
  return kb.header([`*${tenPhong.toUpperCase()}`, ...(tenPhongBan ? [`*${tenPhongBan.toUpperCase()}`] : [])], [], "");
}
function khoiKy(nguoi = "NGƯỜI LẬP"): KhoiIn[] {
  return [kb.gap(10), kb.center(ngayKy(), 10, false), kb.gap(4), kb.sign(["", "", ""], [nguoi, "(Ký và ghi rõ họ tên, chức vụ)", ""])];
}

/** Mục lục hồ sơ (bản in kho): 7 cột, khối ký. */
export function pdfMucLucHoSo(rows: ClRow[], tenPhong: string, tenPhongBan: string, ten = "muc-luc-ho-so.pdf") {
  const rs = sapXepHoSo(rows);
  const y = namTuDen(rs);
  const blocks: KhoiIn[] = [dau(tenPhong, tenPhongBan), kb.title("MỤC LỤC HỒ SƠ")];
  if (y.tu) blocks.push(kb.subtitle(`TỪ NĂM ${y.tu} ĐẾN NĂM ${y.den}`));
  blocks.push(kb.gap(6), kb.table(COT_MLHS, RONG_MLHS, rs.map(dongHoSo)), kb.para(`Mục lục này gồm ${rs.length} hồ sơ (đơn vị bảo quản). Viết bằng chữ: ${hoaDau(soThanhChu(rs.length))} hồ sơ.`), ...khoiKy());
  return pdfManual(blocks, ten, true);
}

/** Sản phẩm mục lục có bìa: bìa 1, bìa 2 (ô điền), bảng gom theo mảng rồi năm. */
export function pdfSanPham(rows: ClRow[], tenPhong: string, tenPhongBan: string, phongSo: string, mucLucSo: string, coThoiHan: boolean, ten = "muc-luc-ho-so-san-pham.pdf") {
  const rs = [...rows].sort((a, b) => (num(a, "mang_so_tt") ?? 1e9) - (num(b, "mang_so_tt") ?? 1e9) || collator.compare(str(a, "ten_mang"), str(b, "ten_mang")) || (num(a, "nam_ket_thuc") ?? 0) - (num(b, "nam_ket_thuc") ?? 0) || (num(a, "hop_so_int") ?? 1e9) - (num(b, "hop_so_int") ?? 1e9) || collator.compare(str(a, "ho_so_so"), str(b, "ho_so_so")));
  const y = namTuDen(rs);
  const nay = new Date().getFullYear();
  const bia = (them: KhoiIn[]): KhoiIn[] => [
    kb.gap(40),
    kb.center(tenPhong.toUpperCase(), 14, true),
    ...(tenPhongBan ? [kb.center(tenPhongBan.toUpperCase(), 14, true)] : []),
    kb.center("*****", 12, false),
    kb.gap(90),
    kb.center("MỤC LỤC HỒ SƠ", 34, true),
    kb.gap(20),
    kb.center("Phông lưu trữ", 13, false),
    kb.center((tenPhongBan || tenPhong).toUpperCase(), 13, false),
    ...(y.tu ? [kb.center(`Từ năm ${y.tu} đến năm ${y.den}`, 13, false)] : []),
    kb.center(coThoiHan ? "(CÓ THỜI HẠN)" : "(VĨNH VIỄN)", 13, false),
    ...them,
    kb.gap(120),
    kb.center(`NĂM ${nay}`, 14, true),
  ];
  const dien: KhoiIn[] = [
    kb.gap(24),
    kb.kv([
      ["Phông số", phongSo || "………"],
      ["Mục lục số", mucLucSo || "………"],
      ["Quyển số", "………"],
      ["Số trang", "………"],
      ["Số hồ sơ từ … đến", "……… — ………"],
      ["Số trang từ … đến", "……… — ………"],
    ]),
  ];
  const bang: string[][] = [];
  let mang = "\u0000";
  let nam = -1;
  for (const r of rs) {
    const m = str(r, "ten_mang");
    if (m !== mang) {
      mang = m;
      nam = -1;
      bang.push(["", "", (m || "(Chưa xếp mảng)").toUpperCase(), "", "", "", ""]);
    }
    const n = num(r, "nam_ket_thuc") ?? 0;
    if (n !== nam) {
      nam = n;
      bang.push(["", "", `Năm ${n || "…"}`, "", "", "", ""]);
    }
    bang.push(dongHoSo(r));
  }
  const blocks: KhoiIn[] = [
    ...bia([]),
    kb.pageBreak(),
    ...bia(dien),
    kb.pageBreak(),
    dau(tenPhong, tenPhongBan),
    kb.title("MỤC LỤC HỒ SƠ"),
    kb.gap(6),
    kb.table(COT_MLHS, RONG_MLHS, bang),
    kb.para(`Mục lục này bao gồm: ${rs.length} hồ sơ (đơn vị bảo quản). Viết bằng chữ: ${hoaDau(soThanhChu(rs.length))} hồ sơ (đơn vị bảo quản).`),
    ...khoiKy(),
  ];
  return pdfManual(blocks, ten, false);
}

/** Mục lục văn bản: mỗi hồ sơ một khối, sang trang. */
export function pdfMucLucVanBan(hoSos: ClRow[], vanBans: Map<string, ClRow[]>, tenPhong: string, tenPhongBan: string, ten = "muc-luc-van-ban.pdf") {
  const cot = ["STT", "Số ký hiệu VB", "Ngày tháng VB", "Tác giả", "Trích yếu nội dung", "Tờ số", "Ghi chú"];
  const rong = [5, 12, 11, 18, 40, 6, 10];
  const blocks: KhoiIn[] = [];
  const hs = sapXepHoSo(hoSos);
  hs.forEach((h, i) => {
    const vbs = [...(vanBans.get(h.id) ?? [])].sort((a, b) => (num(a, "so_thu_tu") ?? 0) - (num(b, "so_thu_tu") ?? 0));
    if (i > 0) blocks.push(kb.pageBreak());
    blocks.push(
      dau(tenPhong, tenPhongBan),
      kb.title("MỤC LỤC VĂN BẢN"),
      kb.kv([
        ["Hộp số / Hồ sơ số", `${str(h, "hop_so")} / ${str(h, "ho_so_so") || str(h, "so_tam")}`],
        ["Tiêu đề hồ sơ", str(h, "tieu_de")],
      ]),
      kb.gap(6),
      kb.table(
        cot,
        rong,
        vbs.map((v) => [str(v, "so_thu_tu"), [str(v, "so_van_ban"), str(v, "so_ky_hieu")].filter(Boolean).join("/"), str(v, "ngay_thang"), str(v, "co_quan_ban_hanh"), str(v, "trich_yeu"), str(v, "to_so"), str(v, "ghi_chu")]),
      ),
      ...khoiKy(),
    );
  });
  return pdfManual(blocks, ten, true);
}

/** Phụ lục "Hướng dẫn tra tìm mục lục": nhóm phòng ban → mỗi mảng một dòng. */
export function pdfPhuLuc(rows: ClRow[], tenPhong: string, ten = "huong-dan-tra-tim.pdf") {
  const rs = rows.filter((r) => str(r, "hop_so"));
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
  const bang: string[][] = [];
  let pb = "\u0000";
  let stt = 0;
  for (const g of [...gs.values()].sort((a, b) => collator.compare(a.pb, b.pb) || Number(b.vv) - Number(a.vv) || collator.compare(a.mang, b.mang))) {
    if (g.pb !== pb) {
      pb = g.pb;
      bang.push(["", (pb || "(Chưa xếp phòng ban)").toUpperCase(), "", "", "", "", ""]);
    }
    const hs = g.hs.sort(collator.compare);
    const hop = g.hop.sort(collator.compare);
    bang.push([String(++stt), g.mang, hs[0] ?? "", hs[hs.length - 1] ?? "", hop[0] ?? "", hop[hop.length - 1] ?? "", ""]);
  }
  const blocks: KhoiIn[] = [dau(tenPhong, ""), kb.title("HƯỚNG DẪN TRA TÌM MỤC LỤC"), kb.gap(6), kb.table(["STT", "Nội dung", "Từ hồ sơ", "Đến hồ sơ", "Từ hộp số", "Đến hộp số", "Trang mục lục"], [5, 40, 9, 9, 9, 9, 10], bang), ...khoiKy()];
  return pdfManual(blocks, ten, false);
}

/** Nhãn hộp 2×2 (A4 dọc, 4 nhãn một trang, có viền cắt): cơ quan / phông, phòng ban, HỘP SỐ, năm, từ – đến hồ sơ, THBQ. */
export function pdfNhanHop(rows: ClRow[], tenPhong: string, ten = "nhan-hop.pdf") {
  const nhans = nhanHopData(rows).map((x) => ({
    top: [tenPhong.toUpperCase(), x.tenPb, "*****"].filter(Boolean),
    big: x.hop,
    bottom: [
      ["Năm", x.nam],
      ["Từ hồ sơ số", x.tu],
      ["Đến hồ sơ số", x.den],
      ["Thời hạn bảo quản", x.thbq],
    ] as [string, string][],
  }));
  return pdfManual([kb.labels(nhans)], ten, false);
}
