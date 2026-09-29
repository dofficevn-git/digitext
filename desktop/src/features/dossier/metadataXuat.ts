// Copyright (c) 2026 DigiText. All rights reserved.
// Xuất / in METADATA đã bóc tách & xác nhận (16–18 trường) theo tài liệu — theo mẫu "Export batch to Excel" của
// doc_extract_pla: mỗi dòng một tài liệu, cột STT · Tên file · Đường dẫn · các trường · Trạng thái, cuối bảng
// dòng tổng kết (tổng số, theo trạng thái). Nguồn trường: máy chủ (`ocr_fields`, ưu tiên bản đã sửa / mới nhất).

import { V4fg } from "@/api/ocr";
import { V4fm, V4fn, V4fv } from "../upload/ocrLogic";
import { writeXlsx, type Sheet } from "../chinhly/excel";
import { esc, inHtml } from "../chinhly/print";

/** Một dòng metadata = một tài liệu. `fields` theo khoá 18 trường (`V4fm`), `daXacNhan` = có trường verified. */
export interface MetaRow {
  id: string;
  filename: string;
  hoSoRef: string;
  relPath: string;
  fields: Record<string, string>;
  daXacNhan: boolean;
  trangThai: string;
  nguoiLam?: string;
  kiemTra?: string;
  ghiChu?: string;
}

/** Lấy trường của nhiều tài liệu (mỗi trường lấy giá trị ở trang đầu tiên có, ưu tiên bản đã sửa / mới nhất). */
export async function layTruongNhieuTaiLieu(
  ids: string[],
  onProgress?: (done: number, total: number) => void,
  concurrency = 6,
): Promise<Map<string, { fields: Record<string, string>; daXacNhan: boolean }>> {
  const out = new Map<string, { fields: Record<string, string>; daXacNhan: boolean }>();
  let next = 0;
  let done = 0;
  const worker = async () => {
    while (next < ids.length) {
      const id = ids[next++];
      const fields: Record<string, string> = {};
      let daXacNhan = false;
      try {
        for (const f of V4fv(await V4fg(id))) {
          if (f.verified) daXacNhan = true;
          const v = f.value.trim();
          if (v && !fields[f.fieldKey]) fields[f.fieldKey] = v;
        }
      } catch {
        // tài liệu chưa có trường / lỗi mạng: để trống
      }
      out.set(id, { fields, daXacNhan });
      done += 1;
      onProgress?.(done, ids.length);
    }
  };
  await Promise.all(Array.from({ length: Math.min(concurrency, Math.max(1, ids.length)) }, worker));
  return out;
}

const COT_TRUOC: [string, number][] = [
  ["STT", 6],
  ["Tên file", 36],
  ["Hồ sơ", 14],
  ["Đường dẫn", 40],
];
const COT_SAU: [string, number][] = [
  ["Đã xác nhận", 12],
  ["Trạng thái", 18],
  ["Người làm", 18],
  ["Kiểm tra", 18],
  ["Ghi chú duyệt", 30],
];
const RONG_TRUONG: Record<string, number> = {
  thuTu: 8,
  loaiVanBan: 16,
  soKyHieu: 16,
  ngayThang: 12,
  coQuanChuQuan: 24,
  coQuanBanHanh: 28,
  trichYeu: 50,
  nguoiKy: 20,
  toSo: 8,
  soTrang: 8,
  trangSo: 8,
  ngonNgu: 10,
  cheDoSuDung: 12,
  doTinCay: 10,
  butTich: 12,
  tinhTrangVatLy: 12,
  taiLieuSo: 10,
  ghiChu: 24,
};

/** Excel: một sheet, tiêu đề cột dòng 1, dữ liệu từ dòng 2, dòng trống rồi tổng kết. */
export async function xuatMetadataExcel(rows: MetaRow[], tenTep: string, sheet = "Metadata"): Promise<void> {
  const header = [...COT_TRUOC.map((c) => c[0]), ...V4fn.map((k) => V4fm[k] ?? k), ...COT_SAU.map((c) => c[0])];
  const aoa: (string | number | null)[][] = [header];
  rows.forEach((r, i) => {
    aoa.push([
      i + 1,
      r.filename,
      r.hoSoRef,
      r.relPath,
      ...V4fn.map((k) => r.fields[k] ?? ""),
      r.daXacNhan ? "Có" : "",
      r.trangThai,
      r.nguoiLam ?? "",
      r.kiemTra ?? "",
      r.ghiChu ?? "",
    ]);
  });
  aoa.push([]);
  aoa.push(["Tổng số:", `${rows.length} tài liệu`]);
  aoa.push(["Đã xác nhận:", `${rows.filter((r) => r.daXacNhan).length} tài liệu`]);
  const theoTrangThai = new Map<string, number>();
  for (const r of rows) theoTrangThai.set(r.trangThai, (theoTrangThai.get(r.trangThai) ?? 0) + 1);
  for (const [tt, n] of theoTrangThai) aoa.push([`${tt}:`, `${n} tài liệu`]);
  const s: Sheet = {
    name: sheet,
    aoa,
    cols: [...COT_TRUOC.map((c) => ({ wch: c[1] })), ...V4fn.map((k) => ({ wch: RONG_TRUONG[k] ?? 14 })), ...COT_SAU.map((c) => ({ wch: c[1] }))],
  };
  await writeXlsx([s], tenTep);
}

const COT_IN: [string, string][] = [
  ["loaiVanBan", "Loại VB"],
  ["soKyHieu", "Số, KH"],
  ["ngayThang", "Ngày tháng"],
  ["coQuanBanHanh", "CQ ban hành"],
  ["nguoiKy", "Người ký"],
  ["trichYeu", "Trích yếu"],
  ["toSo", "Tờ số"],
  ["soTrang", "Số trang"],
];

/** Bản in A4 ngang: các cột chính của metadata + trạng thái; chân trang có dòng tổng kết và khối ký. */
export function inMetadata(rows: MetaRow[], tieuDe: string, phu: string): void {
  const th = ["STT", "Tên file", "Hồ sơ", ...COT_IN.map((c) => c[1]), "Xác nhận", "Trạng thái"];
  const body = rows
    .map(
      (r, i) =>
        `<tr><td class="num">${i + 1}</td><td>${esc(r.filename)}</td><td>${esc(r.hoSoRef)}</td>` +
        COT_IN.map(([k]) => `<td>${esc(r.fields[k] ?? "")}</td>`).join("") +
        `<td class="num">${r.daXacNhan ? "Có" : ""}</td><td>${esc(r.trangThai)}</td></tr>`,
    )
    .join("");
  const html =
    `<style>@page{size:A4 landscape;margin:12mm 10mm} table.bang th, table.bang td{font-size:9.5pt}</style>` +
    `<h2>${esc(tieuDe)}</h2>` +
    (phu ? `<h3 style="font-weight:normal;font-style:italic">${esc(phu)}</h3>` : "") +
    `<table class="bang"><thead><tr>${th.map((h) => `<th>${esc(h)}</th>`).join("")}</tr></thead><tbody>${body}</tbody></table>` +
    `<p style="margin-top:8pt">Tổng số: <b>${rows.length}</b> tài liệu · đã xác nhận: <b>${rows.filter((r) => r.daXacNhan).length}</b>.</p>` +
    `<div class="ky"><div class="ngay">…, ngày … tháng … năm 20…</div><div class="chuc">Người lập</div><div class="ghi">(Ký và ghi rõ họ tên)</div><div class="trong"></div></div>`;
  inHtml(tieuDe, html);
}

export const tenTepAnToan = (s: string) => s.replace(/[\\/:*?"<>|]+/g, "_").trim() || "metadata";
