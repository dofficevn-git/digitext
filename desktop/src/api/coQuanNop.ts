// Copyright (c) 2026 DigiText. All rights reserved.
// Cơ quan nộp lưu vào lưu trữ lịch sử (0040, 27/11/2026 — đợt 6, Digifile D11 / D12): danh mục, tài khoản gắn cơ quan, khoá API.
// Đi qua Tauri bc_call (quyền theo đường: GET submission.view, còn lại submission.review).

import { invoke } from "@/api/ipc";

export interface CoQuanNop {
  id: string;
  projectId: string;
  ma: string;
  ten: string;
  diaChi: string | null;
  nguoiLienHe: string | null;
  dienThoai: string | null;
  email: string | null;
  ghiChu: string | null;
  coApiKey: boolean;
  apiKeyTaoAt: string | null;
  apiKeyGoiAt: string | null;
  soTaiKhoan: number;
  soDotNop: number;
  createdAt: string;
  updatedAt: string;
}

export interface CoQuanNopMoi {
  ma: string;
  ten: string;
  diaChi?: string | null;
  nguoiLienHe?: string | null;
  dienThoai?: string | null;
  email?: string | null;
  ghiChu?: string | null;
}

export interface TaiKhoanCoQuan {
  id: string;
  username: string;
  displayName: string;
  isActive: boolean;
}

const call = <T,>(method: string, path: string, body?: unknown) => invoke<T>("bc_call", { method, path, body: body ?? null });

export const coQuanNopList = (projectId: string) => call<CoQuanNop[]>("GET", `/co-quan-nop?projectId=${encodeURIComponent(projectId)}`);
export const coQuanNopTao = (projectId: string, m: CoQuanNopMoi) => call<CoQuanNop>("POST", "/co-quan-nop", { projectId, ...m });
export const coQuanNopSua = (id: string, m: CoQuanNopMoi) => call<CoQuanNop>("PUT", `/co-quan-nop/${id}`, m);
export const coQuanNopXoa = (id: string) => call<null>("DELETE", `/co-quan-nop/${id}`);
export const coQuanNopCapKey = (id: string) => call<{ id: string; apiKey: string; huongDan: string }>("POST", `/co-quan-nop/${id}/api-key`, {});
export const coQuanNopThuHoiKey = (id: string) => call<null>("DELETE", `/co-quan-nop/${id}/api-key`);
export const coQuanNopTaiKhoan = (id: string) => call<TaiKhoanCoQuan[]>("GET", `/co-quan-nop/${id}/tai-khoan`);
