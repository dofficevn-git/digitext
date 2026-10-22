// Copyright (c) 2026 DigiText. All rights reserved.
// Cây menu 2 cấp theo vòng đời nghiệp vụ: nhóm (icon) → mục. Trục quy trình đi đúng luồng chuẩn (22/10/2026):
// Chỉnh lý → Số hoá → Xử lý → Lưu trữ số — hồ sơ được chỉnh lý xong mới phát cây thư mục để quét. Mỗi nút có `gate` = màn dùng để kiểm quyền
// (domain/permissions.ts V3z7). `badge` là số đếm nhỏ cạnh mục, lấy từ useNotifications.

import type { V3xu, V3xw, V3xx, V3xt } from "@/domain/types";
import type { V41v } from "@/components/Icon";

export type V46r =
  | { kind: "view"; view: V3xt }
  | { kind: "hoso"; pane: V3xu }
  | { kind: "khogia"; tab: V3xw }
  | { kind: "report"; tab: V3xx };

export type NavBadge = "clConflicts" | "myWork" | "notifications";

export interface V46s {
  id: string;
  label: string;
  icon?: V41v;
  gate: V3xt;
  leaf?: V46r;
  children?: V46s[];
  badge?: NavBadge;
  /** Từ khoá thêm cho tìm nhanh (không dấu). */
  keywords?: string;
  /** Nhóm thuộc trục quy trình: "Bước n · việc chính", hiện dưới tên nhóm. */
  step?: string;
}

export const NAV_TREE: V46s[] = [
  { id: "home", label: "Tổng quan", icon: "trangChu", gate: "home", leaf: { kind: "view", view: "home" }, keywords: "trang chu dashboard" },
  {
    id: "chinhly",
    label: "Chỉnh lý",
    icon: "chinhly",
    step: "Bước 1 · lập hồ sơ",
    gate: "dossier",
    children: [
      { id: "danhmuccl", label: "Danh mục", gate: "dossier", leaf: { kind: "hoso", pane: "danhmuccl" }, keywords: "phong phong ban mang thbq" },
      { id: "laphoso", label: "Lập hồ sơ", gate: "dossier", leaf: { kind: "hoso", pane: "laphoso" }, keywords: "ho so van ban" },
      { id: "hethonghoa", label: "Hệ thống hoá", gate: "dossier", leaf: { kind: "hoso", pane: "hethonghoa" }, keywords: "danh so hop" },
      { id: "khogia", label: "Kho và giá", gate: "khogia", leaf: { kind: "khogia", tab: "khogia" }, keywords: "ke" },
      { id: "muontra", label: "Mượn – trả", gate: "khogia", leaf: { kind: "khogia", tab: "muontra" }, keywords: "phieu muon" },
      { id: "khaithac", label: "Tra cứu và kết xuất", gate: "khogia", leaf: { kind: "khogia", tab: "khaithac" }, keywords: "in muc luc nhan hop tt05 excel" },
      { id: "dongbo", label: "Đồng bộ", gate: "dossier", leaf: { kind: "hoso", pane: "dongbo" }, badge: "clConflicts", keywords: "xung dot offline" },
    ],
  },
  {
    id: "sohoa",
    label: "Số hoá",
    icon: "sohoa",
    step: "Bước 2 · quét, tải lên",
    gate: "drive",
    children: [
      { id: "tree", label: "Cây thư mục", gate: "tree", leaf: { kind: "view", view: "tree" }, keywords: "phat cay ho so muc luc" },
      { id: "scan", label: "Quét", gate: "scan", leaf: { kind: "view", view: "scan" }, keywords: "may quet scan" },
      { id: "drive", label: "Kho tài liệu", gate: "drive", leaf: { kind: "view", view: "drive" }, keywords: "thu muc tep" },
      { id: "taolap", label: "Tài liệu đã quét", gate: "dossier", leaf: { kind: "hoso", pane: "sohoa" }, keywords: "tao lap co so du lieu pdf" },
      { id: "upload", label: "Đưa lên máy chủ", gate: "upload", leaf: { kind: "view", view: "upload" }, keywords: "dong bo tai len gom lan" },
      { id: "server", label: "Kho máy chủ", gate: "server", leaf: { kind: "view", view: "server" }, keywords: "may chu tai ve pdf/a" },
    ],
  },
  {
    id: "xuly",
    label: "Xử lý",
    icon: "xuLy",
    step: "Bước 3 · OCR, ký số",
    gate: "dossier",
    children: [
      { id: "ocr", label: "OCR và bóc tách", gate: "dossier", leaf: { kind: "hoso", pane: "ocr" }, keywords: "nhan dang 16 truong" },
      { id: "giaoviec", label: "Giao việc", gate: "workManage", leaf: { kind: "view", view: "workManage" }, keywords: "goi viec xac nhan" },
      { id: "viec", label: "Việc của tôi", gate: "work", leaf: { kind: "view", view: "work" }, badge: "myWork" },
      { id: "kyso", label: "Ký số", gate: "sign", leaf: { kind: "view", view: "sign" }, keywords: "token chu ky" },
      { id: "tachtiff", label: "Tách TIFF", gate: "tiff", leaf: { kind: "view", view: "tiff" }, keywords: "tiff dang tach trang anh" },
      { id: "donggoi", label: "Đóng gói", gate: "package", leaf: { kind: "view", view: "package" }, keywords: "sip aip dip goi tin" },
    ],
  },
  {
    id: "luutruso",
    label: "Lưu trữ số",
    icon: "luuTruSo",
    step: "Bước 4 · nộp lưu",
    gate: "submission",
    children: [
      { id: "thunop", label: "Thu nộp", gate: "submission", leaf: { kind: "view", view: "submission" }, badge: "notifications", keywords: "nop luu lich su" },
      { id: "khaithacso", label: "Khai thác", gate: "access", leaf: { kind: "view", view: "access" }, keywords: "yeu cau ban sao dip" },
      { id: "cungcap", label: "Cung cấp thông tin", gate: "info", leaf: { kind: "view", view: "info" } },
      { id: "bienban", label: "Biên bản", gate: "records", leaf: { kind: "view", view: "records" }, keywords: "phu luc vii" },
      { id: "baoquan", label: "Bảo quản và sao lưu", gate: "bk", leaf: { kind: "view", view: "bk" }, keywords: "toan ven backup" },
      { id: "huy", label: "Tiêu huỷ", gate: "disposal", leaf: { kind: "view", view: "disposal" }, keywords: "het gia tri" },
    ],
  },
  {
    id: "baocao",
    label: "Báo cáo",
    icon: "baoCao",
    gate: "report",
    children: [
      { id: "report:sohoa", label: "Tiến độ dự án", gate: "report", leaf: { kind: "report", tab: "sohoa" }, keywords: "so hoa" },
      { id: "report:nhanvien", label: "Nhân viên", gate: "report", leaf: { kind: "report", tab: "nhanvien" } },
      { id: "report:chinhly", label: "Chỉnh lý", gate: "report", leaf: { kind: "report", tab: "chinhly" } },
      { id: "report:phanmem", label: "Phần mềm", gate: "report", leaf: { kind: "report", tab: "phanmem" } },
      { id: "report:muontra", label: "Mượn trả", gate: "report", leaf: { kind: "report", tab: "muontra" } },
    ],
  },
  {
    id: "quantri",
    label: "Quản trị",
    icon: "quanTri",
    gate: "users",
    children: [
      { id: "users", label: "Người dùng và dự án", gate: "users", leaf: { kind: "view", view: "users" }, keywords: "phan quyen" },
      { id: "settings", label: "Cài đặt", gate: "settings", leaf: { kind: "view", view: "settings" }, keywords: "may chu may quet luu tru" },
    ],
  },
];

/** Mọi lá theo thứ tự hiển thị. */
export function V46t(tree: V46s[] = NAV_TREE): V46r[] {
  const out: V46r[] = [];
  const walk = (nodes: V46s[]) => {
    for (const n of nodes) {
      if (n.leaf) out.push(n.leaf);
      if (n.children) walk(n.children);
    }
  };
  walk(tree);
  return out;
}

/** Trạng thái điều hướng tối thiểu để so khớp lá. */
export interface NavLocation {
  view: V3xt;
  hoSoPane: V3xu;
  khoGiaTab: V3xw;
  reportTab: V3xx;
}

export function leafActive(leaf: V46r, loc: NavLocation): boolean {
  switch (leaf.kind) {
    case "view":
      return loc.view === leaf.view;
    case "hoso":
      return loc.view === "dossier" && loc.hoSoPane === leaf.pane;
    case "khogia":
      return loc.view === "khogia" && loc.khoGiaTab === leaf.tab;
    case "report":
      return loc.view === "report" && loc.reportTab === leaf.tab;
  }
}

export interface NavActions {
  goto: (v: V3xt) => void;
  gotoHoSo: (p: V3xu) => void;
  gotoKhoGia: (t: V3xw) => void;
  gotoReport: (t: V3xx) => void;
}

export function gotoLeaf(leaf: V46r, nav: NavActions): void {
  switch (leaf.kind) {
    case "view":
      return nav.goto(leaf.view);
    case "hoso":
      return nav.gotoHoSo(leaf.pane);
    case "khogia":
      return nav.gotoKhoGia(leaf.tab);
    case "report":
      return nav.gotoReport(leaf.tab);
  }
}

/** Đường dẫn [nhóm, mục] của vị trí hiện tại (để hiện breadcrumb). */
export function navPath(loc: NavLocation, tree: V46s[] = NAV_TREE): V46s[] {
  for (const g of tree) {
    if (g.leaf && leafActive(g.leaf, loc)) return [g];
    for (const c of g.children ?? []) if (c.leaf && leafActive(c.leaf, loc)) return [g, c];
  }
  if (loc.view === "editor") return [{ id: "sohoa", label: "Số hoá", gate: "drive" }, { id: "editor", label: "Sửa ảnh", gate: "editor" }];
  return [];
}

/** Bỏ dấu tiếng Việt để tìm nhanh. */
export function khongDau(s: string): string {
  return s
    .normalize("NFD")
    .replace(/[̀-ͯ]/g, "")
    .replace(/đ/g, "d")
    .replace(/Đ/g, "D")
    .toLowerCase();
}
