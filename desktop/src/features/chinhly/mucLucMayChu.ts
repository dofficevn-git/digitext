// Copyright (c) 2026 DigiText. All rights reserved.
// Đọc mục lục hồ sơ (MLHS) BẰNG MÁY CHỦ — `POST /cl/muc-luc/doc`, bộ đọc là crate `digitext-mucluc`.
//
// Vì sao không đọc trên máy nữa (chốt 10/11/2026): từ khi cổng nhà cung cấp của hệ thống ngoài cũng
// cần đọc mục lục, bộ đọc đã có bản Rust ở máy chủ. Hai bản cùng chạy là đúng căn bệnh "luật hai
// nơi" mà dự án này đã chịu một lần (bộ quyền ở `permission.rs` và `permissions.ts`): luật MLHS đã
// sửa 4 lần trong 5 tuần, bản nào cũ đi là bản đó báo "nhập được" cho tệp sắp bị từ chối.
//
// Không mất khả năng nào: màn «Nhập mục lục» VỐN ĐÃ bắt buộc có mạng (nút nhập tắt khi `offline`).
//
// Bản TS cũ (`mucLucImport.ts` + `thoiGianHoSo.ts`) KHÔNG còn chạy trong app — nó ở lại làm **bản
// đối chứng** cho bản Rust, chỉ `scripts/kiem-muc-luc.ts` dùng. Xem đầu hai tệp đó.
//
// Kiểu dữ liệu dưới đây là HÌNH DÂY (wire) của phản hồi máy chủ, bám đúng tên trường cũ để màn hình
// đổi ít nhất. Hai chỗ khác bản TS cũ: dòng hồ sơ mang `pbIdx` / `mangId` (chỉ số + khoá) thay vì
// object `phongBan` / `mang` lồng — tra bằng `pbCua` / `mangCua`.

import { clRemote } from "@/api/chinhly";

export type KetLuanThoiGian = "dung" | "da_dien" | "canh_bao" | "loi";

export interface KetQuaThoiGian {
  /** dd/mm/yyyy hoặc "" khi không điền được. */
  batDau: string;
  ketThuc: string;
  ketLuan: KetLuanThoiGian;
  /** Ghi chú kiểm tra (mỗi phần tử một ý), rỗng khi đúng. */
  ghiChu: string[];
}

export interface NhomMang {
  /** Khoá bền trong một lần phân tích — dòng hồ sơ trỏ tới bằng `mangId`. */
  id: number;
  dong: number;
  stt: number;
  ma: string;
  ten: string;
  soHoSo: number;
  /** Mảng "Vấn đề chung" do máy chủ tự tạo cho phòng ban không có dòng mảng phía trên hồ sơ. */
  tuDong: boolean;
}

export interface NhomPhongBan {
  dong: number;
  stt: number;
  ma: string;
  ten: string;
  mangs: NhomMang[];
  soHoSo: number;
  /** Phòng ban "Chung" do máy chủ tự tạo vì tệp không chia phòng ban. */
  tuDong: boolean;
}

export interface VanDe {
  dong: number;
  muc: "loi" | "canh_bao";
  noiDung: string;
}

/** Chỉ số cột 0-based. Cột không có trong tệp thì **không có khoá** (không phải `null`). */
export type CotMap = Partial<
  Record<"hopSo" | "hoSoSo" | "tieuDe" | "thoiGian" | "ngayBatDau" | "ngayKetThuc" | "soTo" | "thbq" | "ghiChu", number>
>;

export interface DongHoSo {
  dong: number;
  hopSo: string;
  hoSoSo: string;
  tieuDe: string;
  thoiGianGoc: string;
  soTo: string;
  thbqGoc: string;
  /** Tên THBQ đúng như Danh mục (null = không khớp). */
  thbq: string | null;
  /** true vĩnh viễn · false có thời hạn · null chưa biết (THBQ không khớp). */
  vinhVien: boolean | null;
  ghiChu: string;
  /** Chỉ số trong `PhanTichMucLuc.phongBans` — tra bằng `pbCua`. */
  pbIdx: number | null;
  /** `NhomMang.id` — tra bằng `mangCua`. */
  mangId: number | null;
  tg: KetQuaThoiGian;
  /** Lỗi chặn nhập của riêng dòng này. */
  loi: string[];
  /** Cảnh báo của dòng (vẫn nhập). */
  canhBao: string[];
}

export interface PhanTichMucLuc {
  tenTep: string;
  sheet: string;
  sheets: string[];
  khongChiaPhongBan: boolean;
  thbqMacDinh: string;
  tenPhongTrongTep: string;
  dongTieuDeCot: number;
  cot: CotMap;
  phongBans: NhomPhongBan[];
  rows: DongHoSo[];
  /** Lỗi + cảnh báo cấu trúc, không gồm ghi chú thời gian từng dòng. */
  vanDe: VanDe[];
  thbqTep: string;
  soHop: number;
  soLoi: number;
  soCanhBao: number;
  soDaDien: number;
  /** Nội dung tệp gốc. Máy chủ KHÔNG trả — máy trạm tự giữ để đọc lại sheet khác và để đánh dấu tệp. */
  buf?: ArrayBuffer;
}

export interface KetQuaDocMucLuc {
  pt: PhanTichMucLuc;
  /** Dòng gửi thẳng `POST /cl/ho-so/import`, **xếp đúng thứ tự `pt.rows`**. */
  rowsNhap: Record<string, unknown>[];
  coLoiChan: boolean;
}

export interface TuyChonDocMucLuc {
  /** Sheet muốn đọc; không có → máy chủ chọn sheet đầu tiên có dòng tiêu đề cột. */
  sheet?: string;
  /** Tên THBQ dùng cho ô THBQ trống; không có → ô trống là lỗi dòng. */
  thbqMacDinh?: string;
}

/** ArrayBuffer → base64, cắt khúc 32 KiB vì `String.fromCharCode(...)` tràn ngăn xếp với tệp lớn. */
function sangBase64(buf: ArrayBuffer): string {
  const b = new Uint8Array(buf);
  const KHUC = 0x8000;
  let s = "";
  for (let i = 0; i < b.length; i += KHUC) s += String.fromCharCode(...b.subarray(i, i + KHUC));
  return btoa(s);
}

/**
 * Gửi tệp mục lục lên máy chủ và nhận kết quả phân tích.
 *
 * Danh mục THBQ do **máy chủ** lấy từ chính dự án (cùng truy vấn với `GET /cl/catalog`), không
 * truyền từ máy trạm — truyền đi là mở đường cho hai bên lệch danh mục.
 */
export async function docMucLucMayChu(
  projectId: string,
  buf: ArrayBuffer,
  tenTep: string,
  tuyChon?: TuyChonDocMucLuc,
): Promise<KetQuaDocMucLuc> {
  // Máy chủ (routes/chinh_ly/muc_luc.rs) trả khoá `phanTich`, không phải `pt` — đọc nhầm `pt` làm màn Nhập mục lục
  // sập "Cannot read properties of undefined (reading 'map')" ở bản 0.24.10 (17/11/2026).
  const r = await clRemote<{ phanTich?: PhanTichMucLuc; rowsNhap?: Record<string, unknown>[]; coLoiChan?: boolean }>(
    "POST",
    "/cl/muc-luc/doc",
    {
      projectId,
      tenTep,
      noiDung: sangBase64(buf),
      sheet: tuyChon?.sheet,
      thbqMacDinh: tuyChon?.thbqMacDinh,
    },
  );
  if (!r?.phanTich || !Array.isArray(r.phanTich.rows)) {
    throw new Error("Máy chủ trả kết quả đọc mục lục không đúng dạng — cập nhật máy chủ cùng phiên bản với ứng dụng.");
  }
  return { pt: { ...r.phanTich, buf }, rowsNhap: r.rowsNhap ?? [], coLoiChan: !!r.coLoiChan };
}

/** Đọc tệp người dùng chọn. Giữ lại `ArrayBuffer` để đổi sheet mà không phải chọn tệp lần hai. */
export async function docMucLucTuFile(
  projectId: string,
  f: File,
  tuyChon?: TuyChonDocMucLuc,
): Promise<KetQuaDocMucLuc> {
  return docMucLucMayChu(projectId, await f.arrayBuffer(), f.name, tuyChon);
}

// ------------------------------------------------------------------ tra cứu + nhãn

export function pbCua(pt: PhanTichMucLuc, r: DongHoSo): NhomPhongBan | null {
  return r.pbIdx === null || r.pbIdx === undefined ? null : (pt.phongBans[r.pbIdx] ?? null);
}

export function mangCua(pt: PhanTichMucLuc, r: DongHoSo): NhomMang | null {
  const pb = pbCua(pt, r);
  if (!pb || r.mangId === null || r.mangId === undefined) return null;
  return pb.mangs.find((m) => m.id === r.mangId) ?? null;
}

/** Số tạm của từng dòng = Hồ sơ số trong tệp. Chỉ là phép chiếu để hiện, không phải luật. */
export function soTamCua(pt: PhanTichMucLuc): Map<number, string> {
  return new Map(pt.rows.map((r) => [r.dong, r.hoSoSo]));
}

export const TEN_LOAI = (vv: boolean | null) => (vv === null ? "chưa rõ loại" : vv ? "vĩnh viễn" : "có thời hạn");

export const KET_LUAN_LABEL: Record<KetLuanThoiGian, string> = {
  dung: "Đúng",
  da_dien: "Đã điền",
  canh_bao: "Cảnh báo",
  loi: "Lỗi",
};

/** Số chuẩn để xếp/hiện: "001" → "1"; "12A" giữ nguyên. Chỉ dùng cho trình bày. */
export function soChuan(s: string): string {
  const t = (s ?? "").trim();
  return /^\d+$/.test(t) ? String(+t) : t;
}
