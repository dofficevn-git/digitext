// Copyright (c) 2026 DigiText. All rights reserved.
// Vá tệp .xlsx mà ExcelJS ghi lại từ tệp gốc của người dùng (17/11/2026).
//
// ExcelJS 4.4 không hiểu một số loại định dạng có điều kiện (Duplicate Values, Unique Values…): đọc vào thì mất quy tắc,
// ghi ra thì để lại thẻ rỗng `<conditionalFormatting sqref="C72"/>`. Chuẩn OOXML đòi mỗi thẻ có ít nhất một `<cfRule>`
// → Excel báo tệp hỏng, tự "Repair" và bỏ luôn phần ghi chú (`legacyDrawing` đứng ngay sau) — ô tô màu nhưng di chuột
// không đọc được lỗi (UBNDDONGSONVV_DADIEN.xlsx, xã Tiền Phong, 47 quy tắc tô tiêu đề trùng ở cột C).
//
// Cách vá: thẻ rỗng nào cũng thay bằng ĐÚNG khối gốc cùng `sqref` lấy từ tệp gốc (giữ định dạng của người lập mục lục).
// Quy tắc trỏ `dxfId` vào bảng dxf trong styles.xml — chỉ khôi phục khi bảng dxf hai bên giống hệt; khác thì bỏ thẻ rỗng
// (mất tô trùng của khách nhưng tệp mở được, ghi chú còn).

import JSZip from "jszip";

const giaiThucThe = (s: string) =>
  s.replace(/&quot;/g, '"').replace(/&apos;/g, "'").replace(/&lt;/g, "<").replace(/&gt;/g, ">").replace(/&amp;/g, "&");

/** Tên sheet → đường dẫn tệp XML của sheet trong gói. */
async function duongDanSheet(z: JSZip): Promise<Map<string, string>> {
  const out = new Map<string, string>();
  const wb = await z.file("xl/workbook.xml")?.async("string");
  const rels = await z.file("xl/_rels/workbook.xml.rels")?.async("string");
  if (!wb || !rels) return out;
  const dich = new Map<string, string>();
  for (const m of rels.matchAll(/<Relationship\b[^>]*>/g)) {
    const id = /\bId="([^"]+)"/.exec(m[0])?.[1];
    const t = /\bTarget="([^"]+)"/.exec(m[0])?.[1];
    if (id && t) dich.set(id, t.startsWith("/") ? t.slice(1) : `xl/${t}`);
  }
  for (const m of wb.matchAll(/<(?:\w+:)?sheet\b[^>]*>/g)) {
    const ten = /\bname="([^"]*)"/.exec(m[0])?.[1];
    const id = /\br:id="([^"]+)"/.exec(m[0])?.[1];
    const p = id ? dich.get(id) : undefined;
    if (ten !== undefined && p) out.set(giaiThucThe(ten), p);
  }
  return out;
}

async function bangDxf(z: JSZip): Promise<string> {
  const st = (await z.file("xl/styles.xml")?.async("string")) ?? "";
  return /<(?:\w+:)?dxfs\b[\s\S]*?<\/(?:\w+:)?dxfs>/.exec(st)?.[0].replace(/<(?:\w+:)?dxfs\b[^>]*>/, "") ?? "";
}

/** Khối `<conditionalFormatting>` của sheet gốc theo `sqref` (bỏ tiền tố namespace nếu có, vd `x:`). */
function khoiGoc(xml: string): Map<string, string> {
  const out = new Map<string, string>();
  for (const m of xml.matchAll(/<(\w+:)?conditionalFormatting\b[^>]*>[\s\S]*?<\/\1?conditionalFormatting>/g)) {
    let k = m[0];
    if (m[1]) k = k.replace(new RegExp(`<(/?)${m[1]}`, "g"), "<$1");
    const sq = /\bsqref="([^"]*)"/.exec(k)?.[1];
    if (sq && /<cfRule\b/.test(k) && !out.has(sq)) out.set(sq, k);
  }
  return out;
}

export interface KetQuaVa {
  bytes: Uint8Array;
  /** Số thẻ định dạng có điều kiện rỗng đã khôi phục từ tệp gốc / đã phải bỏ. */
  khoiPhuc: number;
  bo: number;
}

/** Thay các thẻ `<conditionalFormatting …/>` rỗng trong tệp ExcelJS ghi ra bằng khối gốc (hoặc bỏ). */
export async function vaDinhDangDieuKien(goc: ArrayBuffer | Uint8Array, ra: Uint8Array): Promise<KetQuaVa> {
  const zr = await JSZip.loadAsync(ra);
  const zg = await JSZip.loadAsync(goc);
  const [sheetRa, sheetGoc] = await Promise.all([duongDanSheet(zr), duongDanSheet(zg)]);
  const dxfGiong = (await bangDxf(zr)) === (await bangDxf(zg));
  let khoiPhuc = 0;
  let bo = 0;
  for (const [ten, pRa] of sheetRa) {
    const f = zr.file(pRa);
    if (!f) continue;
    const xml = await f.async("string");
    if (!/<conditionalFormatting\b[^>]*\/>/.test(xml)) continue;
    const pGoc = sheetGoc.get(ten);
    const goc = dxfGiong && pGoc ? khoiGoc((await zg.file(pGoc)?.async("string")) ?? "") : new Map<string, string>();
    const moi = xml.replace(/<conditionalFormatting\b[^>]*\/>/g, (the) => {
      const sq = /\bsqref="([^"]*)"/.exec(the)?.[1];
      const k = sq ? goc.get(sq) : undefined;
      if (k) {
        khoiPhuc++;
        return k;
      }
      bo++;
      return "";
    });
    zr.file(pRa, moi);
  }
  if (khoiPhuc === 0 && bo === 0) return { bytes: ra, khoiPhuc, bo };
  const bytes = await zr.generateAsync({ type: "uint8array", compression: "DEFLATE" });
  return { bytes, khoiPhuc, bo };
}
