// Copyright (c) 2026 DigiText. All rights reserved.
// Mẫu in chỉnh lý (HTML + window.print, theo đúng bố cục bản cũ): Mục lục hồ sơ, Mục lục văn bản,
// Nhãn hộp A4 2×2, Sản phẩm mục lục có bìa, Phụ lục hướng dẫn tra tìm.

import { isTauri } from "@/api/ipc";
import { V4e0 } from "@/api/exportFile";
import { str, num, type ClRow } from "@/api/chinhly";

export const collator = new Intl.Collator("vi", { numeric: true, sensitivity: "base" });
export const esc = (s: unknown) => String(s ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c] ?? c);

export const CSS = `
@page { size: A4; margin: 15mm 12mm; }
body { font-family: "Times New Roman", serif; font-size: 12pt; color: #000; margin: 0; }
h2, h3 { text-align: center; margin: 4pt 0; }
table.bang { width: 100%; border-collapse: collapse; margin-top: 6pt; }
table.bang th, table.bang td { border: 1px solid #000; padding: 3pt 4pt; vertical-align: top; font-size: 11pt; }
table.bang th { text-align: center; font-weight: bold; }
table.bang tr.stt td { text-align: center; font-style: italic; }
table.bang tr.nhom td { font-weight: bold; background: #f2f2f2; }
table.bang tr.nam td { font-style: italic; }
.num { text-align: center; }
.ky { margin-top: 18pt; width: 45%; margin-left: auto; text-align: center; }
.ky .ngay { font-style: italic; }
.ky .chuc { font-weight: bold; margin-top: 4pt; }
.ky .ghi { font-style: italic; font-size: 10pt; }
.ky .trong { height: 60pt; }
.dau { display: flex; justify-content: space-between; font-size: 11pt; }
.dau .co { text-transform: uppercase; font-weight: bold; }
.trang { page-break-after: always; }
.bia { page-break-after: always; height: 257mm; border: 4px double #d97706; padding: 14mm; box-sizing: border-box; text-align: center; display: flex; flex-direction: column; justify-content: space-between; }
.bia .ten { text-transform: uppercase; font-weight: bold; font-size: 16pt; }
.bia .tieude { color: #b91c1c; font-size: 38pt; font-weight: bold; letter-spacing: 2pt; }
.bia .dong { text-transform: uppercase; font-size: 14pt; line-height: 1.8; }
.bia .dien { text-align: left; display: inline-block; font-size: 14pt; line-height: 2.2; }
.bia .nam { font-weight: bold; font-size: 14pt; }
.nhan-trang { display: grid; grid-template-columns: 1fr 1fr; grid-template-rows: 1fr 1fr; gap: 6mm; height: 262mm; page-break-after: always; }
.nhan { border: 1px solid #000; outline: 4px double #000; outline-offset: -6px; padding: 10mm 8mm; box-sizing: border-box; display: flex; flex-direction: column; justify-content: space-between; text-align: center; }
.nhan .co { text-transform: uppercase; font-size: 12pt; }
.nhan .pb { text-transform: uppercase; font-weight: bold; font-size: 13pt; }
.nhan .hop { font-size: 40pt; font-weight: bold; }
.nhan .so { font-size: 56pt; font-weight: bold; line-height: 1; }
.nhan .chan { text-align: left; font-size: 13pt; line-height: 1.7; }
.nhan .chan b { float: right; font-size: 15pt; }
`;

export function inHtml(title: string, body: string): void {
  const html = `<!doctype html><html><head><meta charset="utf-8"><title>${esc(title)}</title><style>${CSS}</style></head><body>${body}<script>window.onload=function(){setTimeout(function(){window.print()},400)}</script></body></html>`;
  if (isTauri()) {
    void V4e0(html);
    return;
  }
  const w = window.open("", "_blank");
  if (w) {
    w.document.write(html);
    w.document.close();
  }
}

export function khoiKy(nguoi = "Người lập"): string {
  return `<div class="ky"><div class="ngay">.............., ngày ..... tháng ..... năm 20.....</div><div class="chuc">${esc(nguoi)}</div><div class="ghi">(Ký và ghi rõ họ tên, chức vụ/chức danh)</div><div class="trong"></div></div>`;
}

export function khoangNgay(r: ClRow): string {
  const a = str(r, "ngay_bat_dau");
  const b = str(r, "ngay_ket_thuc");
  if (a && b) return `${a} - ${b}`;
  return a || b || "N/A";
}

export function sapXepHoSo(rows: ClRow[]): ClRow[] {
  return [...rows].sort((a, b) => (num(a, "hop_so_int") ?? 1e9) - (num(b, "hop_so_int") ?? 1e9) || collator.compare(str(a, "hop_so"), str(b, "hop_so")) || collator.compare(str(a, "ho_so_so"), str(b, "ho_so_so")) || collator.compare(str(a, "so_tam"), str(b, "so_tam")));
}

export function namTuDen(rows: ClRow[]): { tu: number; den: number } {
  const ys: number[] = [];
  for (const r of rows) for (const k of ["ngay_bat_dau", "ngay_ket_thuc"]) {
    const m = /(\d{4})/.exec(str(r, k));
    if (m) ys.push(Number(m[1]));
  }
  return ys.length ? { tu: Math.min(...ys), den: Math.max(...ys) } : { tu: 0, den: 0 };
}

const COT_MLHS = ["Hộp số", "Hồ sơ số", "Tiêu đề hồ sơ", "Ngày tháng bắt đầu - kết thúc", "Số tờ", "Thời hạn bảo quản", "Ghi chú"];

function dongHoSo(r: ClRow): string {
  return `<tr><td class="num">${esc(str(r, "hop_so"))}</td><td class="num">${esc(str(r, "ho_so_so") || str(r, "so_tam"))}</td><td>${esc(str(r, "tieu_de"))}</td><td class="num">${esc(khoangNgay(r))}</td><td class="num">${esc(str(r, "so_to") === "0" ? "" : str(r, "so_to"))}</td><td class="num">${esc(str(r, "ten_thbq"))}</td><td>${esc(str(r, "ghi_chu"))}</td></tr>`;
}
const dongStt = (n: number) => `<tr class="stt">${Array.from({ length: n }, (_, i) => `<td>(${i + 1})</td>`).join("")}</tr>`;

export function dau(coQuan: string, tenPhong: string): string {
  return `<div class="dau"><div><div class="co">${esc(coQuan)}</div><div class="co">${esc(tenPhong)}</div></div><div></div></div>`;
}

/** Mục lục hồ sơ (bản in kho): 7 cột, dòng (1)…(7), khối ký. */
export function inMucLucHoSo(rows: ClRow[], tenPhong: string, tenPhongBan: string): string {
  const rs = sapXepHoSo(rows);
  const y = namTuDen(rs);
  return `${dau(tenPhong, tenPhongBan)}<h2>MỤC LỤC HỒ SƠ</h2><h3>${y.tu ? `TỪ NĂM ${y.tu} ĐẾN NĂM ${y.den}` : ""}</h3><table class="bang"><thead><tr>${COT_MLHS.map((c) => `<th>${c}</th>`).join("")}</tr>${dongStt(7)}</thead><tbody>${rs.map(dongHoSo).join("")}</tbody></table>${khoiKy()}`;
}

/** Sản phẩm mục lục có bìa: bìa 1, bìa 2, danh sách nhóm theo mảng rồi năm, chân "Viết bằng chữ". */
export function inSanPham(rows: ClRow[], tenPhong: string, tenPhongBan: string, phongSo: string, mucLucSo: string, coThoiHan: boolean): string {
  const rs = [...rows].sort((a, b) => (num(a, "mang_so_tt") ?? 1e9) - (num(b, "mang_so_tt") ?? 1e9) || collator.compare(str(a, "ten_mang"), str(b, "ten_mang")) || (num(a, "nam_ket_thuc") ?? 0) - (num(b, "nam_ket_thuc") ?? 0) || (num(a, "hop_so_int") ?? 1e9) - (num(b, "hop_so_int") ?? 1e9) || collator.compare(str(a, "ho_so_so"), str(b, "ho_so_so")));
  const y = namTuDen(rs);
  const nay = new Date().getFullYear();
  const bia = (them: string) =>
    `<div class="bia"><div><div class="ten">${esc(tenPhong)}</div><div class="ten">${esc(tenPhongBan)}</div><div>*****</div></div><div><div class="tieude">MỤC LỤC HỒ SƠ</div><div class="dong">Phông lưu trữ<br>${esc(tenPhongBan || tenPhong)}<br>${y.tu ? `Từ năm ${y.tu} đến năm ${y.den}` : ""}</div><div class="dong">(${coThoiHan ? "CÓ THỜI HẠN" : "VĨNH VIỄN"})</div>${them}</div><div class="nam">NĂM ${nay}</div></div>`;
  const dien = `<div class="dien">PHÔNG SỐ: ${esc(phongSo) || "........"}<br>MỤC LỤC SỐ: ${esc(mucLucSo) || "........"}<br>QUYỂN SỐ: ........<br>SỐ TRANG: ........<br>SỐ HỒ SƠ TỪ: ........ ĐẾN: ........<br>SỐ TRANG TỪ: ........ ĐẾN: ........</div>`;
  let body = "";
  let mang = "\u0000";
  let nam = -1;
  for (const r of rs) {
    const m = str(r, "ten_mang");
    if (m !== mang) {
      mang = m;
      nam = -1;
      body += `<tr class="nhom"><td colspan="7">${esc(m || "(Chưa xếp mảng)")}</td></tr>`;
    }
    const n = num(r, "nam_ket_thuc") ?? 0;
    if (n !== nam) {
      nam = n;
      body += `<tr class="nam"><td colspan="7">Năm ${n || "…"}</td></tr>`;
    }
    body += dongHoSo(r);
  }
  return `${bia("")}${bia(dien)}${dau(tenPhong, tenPhongBan)}<h2>MỤC LỤC HỒ SƠ</h2><table class="bang"><thead><tr>${COT_MLHS.map((c) => `<th>${c}</th>`).join("")}</tr>${dongStt(7)}</thead><tbody>${body}</tbody></table><p><i>Mục lục này bao gồm: ${rs.length} hồ sơ (đơn vị bảo quản).</i><br><i>Viết bằng chữ: ${esc(hoaDau(soThanhChu(rs.length)))} hồ sơ (đơn vị bảo quản).</i></p>${khoiKy()}`;
}

/** Mục lục văn bản: mỗi hồ sơ một khối, ngắt trang. */
export function inMucLucVanBan(hoSos: ClRow[], vanBans: Map<string, ClRow[]>, tenPhong: string, tenPhongBan: string): string {
  const cot = ["STT", "Số ký hiệu VB", "Ngày tháng VB", "Tác giả", "Trích yếu nội dung", "Tờ số", "Ghi chú"];
  return sapXepHoSo(hoSos)
    .map((h) => {
      const vbs = [...(vanBans.get(h.id) ?? [])].sort((a, b) => (num(a, "so_thu_tu") ?? 0) - (num(b, "so_thu_tu") ?? 0));
      return `<div class="trang">${dau(tenPhong, tenPhongBan)}<h3>MỤC LỤC VĂN BẢN</h3><div>Hộp số: <b>${esc(str(h, "hop_so"))}</b> &nbsp;&nbsp; Hồ sơ số: <b>${esc(str(h, "ho_so_so") || str(h, "so_tam"))}</b><br>Tiêu đề hồ sơ: <b>${esc(str(h, "tieu_de"))}</b></div><table class="bang"><thead><tr>${cot.map((c) => `<th>${c}</th>`).join("")}</tr>${dongStt(7)}</thead><tbody>${vbs
        .map((v) => `<tr><td class="num">${esc(str(v, "so_thu_tu"))}</td><td class="num">${esc([str(v, "so_van_ban"), str(v, "so_ky_hieu")].filter(Boolean).join("/"))}</td><td class="num">${esc(str(v, "ngay_thang"))}</td><td>${esc(str(v, "co_quan_ban_hanh"))}</td><td>${esc(str(v, "trich_yeu"))}</td><td class="num">${esc(str(v, "to_so"))}</td><td>${esc(str(v, "ghi_chu"))}</td></tr>`)
        .join("")}</tbody></table>${khoiKy()}</div>`;
    })
    .join("");
}

/** Nhãn hộp A4 2×2: gom hồ sơ liên tiếp cùng số hộp (theo phòng ban). */
export function inNhanHop(rows: ClRow[], tenPhong: string): string {
  // Hộp 01 vĩnh viễn khác hộp 01 có thời hạn (0015): gom theo loại trước (sort ổn định giữ thứ tự hộp trong loại).
  const vv = (r: ClRow) => r.vinh_vien === true || r.vinh_vien === 1;
  const rs = sapXepHoSo(rows)
    .filter((r) => str(r, "hop_so"))
    .sort((a, b) => Number(vv(b)) - Number(vv(a)));
  type Nhan = { pb: string; vv: boolean; hop: string; nam: string; tu: string; den: string; thbq: string; n: number };
  const nhans: Nhan[] = [];
  for (const r of rs) {
    const key = `${str(r, "phong_ban_id")}|${vv(r)}|${str(r, "hop_so")}`;
    const last = nhans[nhans.length - 1];
    if (last && `${last.pb}|${last.vv}|${last.hop}` === key) {
      last.den = str(r, "ho_so_so") || str(r, "so_tam");
      last.n += 1;
    } else {
      nhans.push({ pb: str(r, "phong_ban_id"), vv: vv(r), hop: str(r, "hop_so"), nam: String(num(r, "nam_ket_thuc") ?? ""), tu: str(r, "ho_so_so") || str(r, "so_tam"), den: str(r, "ho_so_so") || str(r, "so_tam"), thbq: str(r, "ten_thbq"), n: 1 });
    }
  }
  const ten = new Map(rs.map((r) => [str(r, "phong_ban_id"), str(r, "ten_phong_ban")]));
  const pages: string[] = [];
  for (let i = 0; i < nhans.length; i += 4) {
    const page = nhans.slice(i, i + 4);
    while (page.length < 4) page.push({ pb: "", vv: false, hop: "", nam: "", tu: "", den: "", thbq: "", n: 0 });
    pages.push(
      `<div class="nhan-trang">${page
        .map((x) =>
          x.hop === "" && x.n === 0
            ? `<div class="nhan"></div>`
            : `<div class="nhan"><div><div class="co">${esc(tenPhong)}</div><div class="pb">${esc(ten.get(x.pb) ?? "")}</div><div>*****</div></div><div><div class="hop">HỘP SỐ</div><div class="so">${esc(x.hop)}</div></div><div class="chan">Năm: <b>${esc(x.nam)}</b><br>Từ hồ sơ số: <b>${esc(x.tu)}</b><br>Đến hồ sơ số: <b>${esc(x.den)}</b><br>Thời hạn bảo quản: <b>${esc(x.thbq)}</b></div></div>`,
        )
        .join("")}</div>`,
    );
  }
  return pages.join("");
}

/** Phụ lục "Hướng dẫn tra tìm mục lục": nhóm phòng ban → mỗi mảng một dòng (từ/đến hồ sơ, từ/đến hộp). */
export function inPhuLuc(rows: ClRow[], tenPhong: string): string {
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
  const cot = ["STT", "Nội dung", "Từ hồ sơ", "Đến hồ sơ", "Từ hộp số", "Đến hộp số", "Trang mục lục"];
  let body = "";
  let pb = "\u0000";
  let stt = 0;
  for (const g of [...gs.values()].sort((a, b) => collator.compare(a.pb, b.pb) || Number(b.vv) - Number(a.vv) || collator.compare(a.mang, b.mang))) {
    if (g.pb !== pb) {
      pb = g.pb;
      body += `<tr class="nhom"><td colspan="7">${esc(pb || "(Chưa xếp phòng ban)")}</td></tr>`;
    }
    const hs = g.hs.sort(collator.compare);
    const hop = g.hop.sort(collator.compare);
    body += `<tr><td class="num">${++stt}</td><td>${esc(g.mang)}</td><td class="num">${esc(hs[0] ?? "")}</td><td class="num">${esc(hs[hs.length - 1] ?? "")}</td><td class="num">${esc(hop[0] ?? "")}</td><td class="num">${esc(hop[hop.length - 1] ?? "")}</td><td></td></tr>`;
  }
  return `${dau(tenPhong, "")}<h2>HƯỚNG DẪN TRA TÌM MỤC LỤC</h2><table class="bang"><thead><tr>${cot.map((c) => `<th>${c}</th>`).join("")}</tr>${dongStt(7)}</thead><tbody>${body}</tbody></table>${khoiKy()}`;
}

// ------------------------------------------------------------------ số → chữ (giọng Bắc, như digitext-core)

const CHU = ["không", "một", "hai", "ba", "bốn", "năm", "sáu", "bảy", "tám", "chín"];
function nhom3(v: number, full: boolean, out: string[]) {
  const h = Math.floor(v / 100);
  const t = Math.floor(v / 10) % 10;
  const u = v % 10;
  if (h > 0 || full) out.push(`${CHU[h]} trăm`);
  if (t === 0) {
    if (u > 0) {
      if (h > 0 || full) out.push("linh");
      out.push(CHU[u]);
    }
  } else if (t === 1) {
    out.push("mười");
    if (u === 5) out.push("lăm");
    else if (u > 0) out.push(CHU[u]);
  } else {
    out.push(`${CHU[t]} mươi`);
    if (u === 1) out.push("mốt");
    else if (u === 4) out.push("tư");
    else if (u === 5) out.push("lăm");
    else if (u > 0) out.push(CHU[u]);
  }
}
export function soThanhChu(n: number): string {
  if (!Number.isFinite(n) || n < 0) return "";
  if (n === 0) return "không";
  const groups: number[] = [];
  let v = Math.floor(n);
  while (v > 0) {
    groups.push(v % 1000);
    v = Math.floor(v / 1000);
  }
  const units = ["", "nghìn", "triệu", "tỷ", "nghìn tỷ"];
  const out: string[] = [];
  const top = groups.length - 1;
  for (let i = top; i >= 0; i--) {
    if (groups[i] === 0) continue;
    nhom3(groups[i], i !== top, out);
    if (i > 0) out.push(units[Math.min(i, units.length - 1)]);
  }
  return out.join(" ");
}
export const hoaDau = (s: string) => (s ? s[0].toUpperCase() + s.slice(1) : s);
