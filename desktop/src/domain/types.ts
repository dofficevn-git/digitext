// Copyright (c) 2026 DigiText. All rights reserved.

export type V3xs = "super_admin" | "manager" | "admin" | "user" | "customer";

export type V3xt =
  | "home"
  | "drive"
  | "tree"
  | "dossier"
  | "khogia"
  | "scan"
  | "editor"
  | "upload"
  | "sign"
  | "tiff"
  | "users"
  | "roles"
  | "settings"
  | "package"
  | "submission"
  | "records"
  | "access"
  | "disposal"
  | "info"
  | "bk"
  | "server"
  | "report"
  | "work"
  | "workManage";

export type V3xu = "danhmuccl" | "laphoso" | "hethonghoa" | "dongbo" | "sohoa" | "ocr";

export type V3xw = "khogia" | "khaithac" | "muontra";

export type V3xx = "sohoa" | "chinhly" | "nhanvien" | "phanmem" | "muontra";

export type V3xy = "root" | "folder" | "doc";

export type V3xz = "signed" | "uploading" | "editing" | "flagged" | "scanned";

export type V3y0 = "ok" | "info" | "warn" | "err" | "gray" | "run";

export interface V3y1 {
  id: string;
  type: V3xy;
  name: string;
  owner?: string;
  members?: string[];
  kids?: string[];
  pages?: number;
  status?: V3xz;

  path?: string;

  size?: number;

  archive?: V3y2;
}

export interface V3y2 {
  maDuAn: string;
  tenDuAn: string;
  hopSo: string;
  hoSoSo: string;
  tieuDe: string;
}

export type V3y3 = 300 | 600;
export type V3y4 = "rgb" | "gray";
export type V3y5 = "adf" | "glass";

export interface V3y6 {
  id: string;
  name: string;
  model: string;
  connected: boolean;
}

export interface V3y7 {
  kind: string;

  manual: boolean;

  message: string;
}

export interface V3y8 {
  deskew: boolean;

  punchHoles: boolean;

  despeckle: boolean;

  denoise: boolean;

  sharpen: boolean;
}

export const V3y9: V3y8 = {
  deskew: true,
  punchHoles: true,
  despeckle: true,
  denoise: true,
  sharpen: true,
};

export const V3ya: V3y8 = {
  deskew: false,
  punchHoles: false,
  despeckle: false,
  denoise: false,
  sharpen: false,
};

export interface V3yb {
  deviceId?: string;
  dpi: V3y3;
  color: V3y4;
  source: V3y5;

  duplex?: boolean;

  startIndex?: number;

  maxPages?: number;

  target?: string;

  enhance?: V3y8;
}

export interface V3yc {
  index: number;
  widthPx: number;
  heightPx: number;
  dpi: number;
  widthMm: number;
  heightMm: number;

  paper: string;
  paperKnown: boolean;

  dpiOk: boolean;

  thumbnail: string;

  preview: string;

  sourcePath: string;
}

export interface V3yd {
  x: number;
  y: number;
  w: number;
  h: number;
}

export interface V3ye {
  rotateQuarters: number;
  brightness: number;
  contrast: number;
  gamma: number;
  colorMode: "color" | "gray" | "bw";
  bwThreshold: number;

  flipH?: boolean;

  flipV?: boolean;

  deskewDeg?: number;

  crop?: V3yd | null;

  autoCrop?: boolean;

  autoDeskew?: boolean;

  align?: "center" | "left";

  denoise?: boolean;

  despeckle?: boolean;

  sharpen?: boolean;

  punchHoles?: boolean;

  autoTone?: boolean;

  sourceDpi?: number;

  targetDpi?: number;
}

export interface V3yf {
  preview: string;
  thumbnail: string;
  widthPx: number;
  heightPx: number;

  dpi?: number;

  canUndo?: boolean;
  canRedo?: boolean;
}

export interface V3yg {
  canUndo: boolean;
  canRedo: boolean;
}

export type V3yh =
  | { kind: "starting" }
  | { kind: "capturing"; device: string }
  | { kind: "disconnected"; atPage: number }
  | { kind: "completed"; pages: number }
  | { kind: "cancelled"; pages: number };

export interface V3yi {
  name: string;
  email: string;
  role: string;
  av: string;
  c: number;
  owner?: boolean;
}

export type V3yj = "DANG" | "NN";

export interface V3yk {
  maDuAn: string;
  tenDuAn: string;
  maKhoi: V3yj;
  phongSo?: string;
  mucLucSo?: string;

  isTest?: boolean;
}

export interface V3yl {
  maDv: string;
  maPhong: string;
  maDonVi: string;
  tenDonVi: string;
  soTt?: number;
}

export interface V3ym {
  mangCvid: number;
  maMang: string;
  tenMang: string;
  maDv: string;
  soTt?: number;
}

export type V3yn = 1 | 2 | 3;

export interface V3yo {
  thbqid: number;
  tenThbq: string;
  trangThai: V3yn;
  giaTri: number;
  sort: number;
}

export interface V3yp {
  maNgonNgu: string;
  tenNgonNgu: string;
}

export interface V3yq {
  maLoai: string;
  tenLoai: string;
}

export interface V3yr {
  id: number;
  hopSo: string;
  hoSoSo: string;
  soTam?: string;
  maHoSo?: string;
  tieuDe: string;
  thoiGian: string;
  ngayBatDau?: string;
  ngayKetThuc?: string;
  soTo: string;
  thbq: string;
  trungThua?: boolean;
  trangThai?: string;
  tongSoVanBan?: number;
  dossierId?: number;
  mang?: string;
  ghiChu?: string;
  nhom?: string;
  nam?: string;
  nguon?: string;

  pdfPath?: string;
  pageCount?: number;
  status?: "pending" | "done";
}

export interface V3ys {
  phongs: V3yk[];
  phongBans: V3yl[];
  mangs: V3ym[];
  thbqs: V3yo[];
  ngonNgus: V3yp[];
  loaiVanBans: V3yq[];
  hoSos: V3yr[];
}
