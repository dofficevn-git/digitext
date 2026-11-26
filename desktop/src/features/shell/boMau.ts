// Copyright (c) 2026 DigiText. All rights reserved.
// Bộ màu giao diện (26/11/2026) — theo DO365 (components/ChonBoMau.tsx): DO365 cam · Microsoft Teams tím (mặc định) · Zalo xanh.
// Ghi nhớ theo MÁY (localStorage). Giá trị màu thật nằm ở styles/bo-mau.css; `mau` dưới đây chỉ là 3 ô xem trước.

export type BoMau = "teams" | "do365" | "zalo";

const KHOA = "digitext.boMau";

export const BO_MAU: { k: BoMau; ten: string; phu: string; mau: [string, string, string] }[] = [
  { k: "do365", ten: "DO365", phu: "cam logo DOFFICE", mau: ["#C2410C", "#FDEFE6", "#F6F6F8"] },
  { k: "teams", ten: "Microsoft Teams", phu: "tím mặc định", mau: ["#5A58A6", "#EEEDF8", "#F5F5F5"] },
  { k: "zalo", ten: "Zalo", phu: "xanh quen thuộc", mau: ["#0068FF", "#E5F0FF", "#EBEDF0"] },
];

/** Bộ màu đã chọn trên máy này; chưa chọn / hỏng → "teams" (y như trước khi có tính năng). */
export function boMauDaChon(): BoMau {
  try {
    const v = localStorage.getItem(KHOA);
    return v === "do365" || v === "zalo" || v === "teams" ? v : "teams";
  } catch {
    return "teams";
  }
}

/** Gắn lớp lên <html> (không phải <body> — xem bo-mau.css). Bộ "teams" = không lớp nào. */
export function apBoMau(b: BoMau): void {
  const c = document.documentElement.classList;
  c.toggle("theme-do365", b === "do365");
  c.toggle("theme-zalo", b === "zalo");
}

export function datBoMau(b: BoMau): void {
  apBoMau(b);
  try {
    localStorage.setItem(KHOA, b);
  } catch {}
}

// Nền tối — độc lập với bộ màu (3 bộ × 2 nền), cũng ghi nhớ theo máy. Màu ở styles/nen-toi.css.
const KHOA_TOI = "digitext.nenToi";

export function nenToiDaChon(): boolean {
  try {
    return localStorage.getItem(KHOA_TOI) === "1";
  } catch {
    return false;
  }
}

export function apNenToi(toi: boolean): void {
  document.documentElement.classList.toggle("dark", toi);
}

export function datNenToi(toi: boolean): void {
  apNenToi(toi);
  try {
    localStorage.setItem(KHOA_TOI, toi ? "1" : "0");
  } catch {}
}
