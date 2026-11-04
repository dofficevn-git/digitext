// Copyright (c) 2026 DigiText. All rights reserved.
// Kiểm bộ nhập mục lục hồ sơ (features/chinhly/mucLucImport.ts) trên một thư mục Excel thật, không cần mở app:
//   npm run kiem:mucluc -- D:/DigiText/test/MLHS ["THBQ mặc định"]
// In mỗi tệp: sheet được chọn, số hồ sơ, phòng ban, cột nhận được, lỗi chặn / cảnh báo, ô trống, 6 lỗi hay gặp.
// Chạy qua scripts/kiem-muc-luc.mjs (esbuild bundle + stub hai module Tauri). Bộ 23 mục lục Bắc Giang (2/11/2026):
// mục tiêu không tệp nào bị chặn ngoài dòng trùng hồ sơ số / thiếu thời gian thật.

import * as fs from "node:fs";
import * as path from "node:path";
import { coLoiChan, docMucLucTuBuffer } from "@/features/chinhly/mucLucImport";

/** Bảng THBQ mặc định (digitext-core chinh_ly::codes::THBQ_SEED) — khi chạy ngoài app không có Danh mục. */
const THBQ = ["Vĩnh viễn", "5 năm", "10 năm", "15 năm", "20 năm", "50 năm", "70 năm", "05 năm đánh giá", "10 năm đánh giá", "15 năm đánh giá", "20 năm đánh giá", "50 năm đánh giá", "70 năm đánh giá", "Theo tuổi thọ công trình"];

const dir = process.argv[2];
if (!dir) {
  console.error("Cách dùng: npm run kiem:mucluc -- <thư mục Excel> [THBQ mặc định]");
  process.exit(2);
}
const thbqMacDinh = process.argv[3] || undefined;
const tong = { tep: 0, hoSo: 0, chan: 0, loi: 0, canhBao: 0 };
for (const f of fs.readdirSync(dir).sort()) {
  if (!/\.xlsx?$|\.xlsm$/i.test(f)) continue;
  const buf = fs.readFileSync(path.join(dir, f));
  const ab = buf.buffer.slice(buf.byteOffset, buf.byteOffset + buf.byteLength) as ArrayBuffer;
  const pt = await docMucLucTuBuffer(ab, f, THBQ, ["Vĩnh viễn"], { thbqMacDinh });
  const dem = new Map<string, number>();
  for (const r of pt.rows) {
    for (const s of [...r.loi.map((x) => `LỖI ${x}`), ...r.canhBao.map((x) => `cb ${x}`)]) {
      const k = s.replace(/"[^"]*"/g, '"…"').replace(/\d+/g, "N");
      dem.set(k, (dem.get(k) ?? 0) + 1);
    }
  }
  const trong = (f2: (r: (typeof pt.rows)[number]) => boolean) => pt.rows.filter(f2).length;
  const chan = coLoiChan(pt);
  // KIEM_XUAT_JSON=<thư mục>: ghi hồ sơ đã đọc (hộp, hồ sơ số, số tờ, vĩnh viễn) để chạy thử đối soát với kho máy chủ.
  if (process.env.KIEM_XUAT_JSON) {
    fs.mkdirSync(process.env.KIEM_XUAT_JSON, { recursive: true });
    const hs = pt.rows.map((r) => ({ hop_so: r.hopSo || null, ho_so_so: r.hoSoSo || null, so_tam: r.hoSoSo || String(r.dong), vinh_vien: r.vinhVien === true, so_to: r.soTo || null, tieu_de: r.tieuDe }));
    fs.writeFileSync(path.join(process.env.KIEM_XUAT_JSON, f.replace(/\.[^.]+$/, ".json")), JSON.stringify(hs));
  }
  console.log(
    `## ${f.slice(0, 60)}  [sheet '${pt.sheet}' /${pt.sheets.length}] → ${pt.rows.length} hồ sơ, PB ${pt.phongBans.length}${pt.khongChiaPhongBan ? " (Chung)" : ""}, cột ${Object.keys(pt.cot).join(",")} | lỗi ${pt.soLoi} cảnh báo ${pt.soCanhBao} | hộp trống ${trong((r) => !r.hopSo)} THBQ trống ${trong((r) => !r.thbq)} TG trống ${trong((r) => !r.tg.batDau)} | ${chan ? "CHẶN" : "nhập được"}`,
  );
  for (const v of pt.vanDe) console.log(`   [${v.muc}] dòng ${v.dong}: ${v.noiDung}`);
  for (const [k, c] of [...dem.entries()].sort((a, b) => b[1] - a[1]).slice(0, 6)) console.log(`   ${c}× ${k}`);
  tong.tep++;
  tong.hoSo += pt.rows.length;
  tong.chan += chan ? 1 : 0;
  tong.loi += pt.soLoi;
  tong.canhBao += pt.soCanhBao;
}
console.log(`TỔNG: ${tong.tep} tệp, ${tong.hoSo} hồ sơ, ${tong.chan} tệp bị chặn, ${tong.loi} lỗi, ${tong.canhBao} cảnh báo`);
