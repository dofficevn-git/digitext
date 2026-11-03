// Copyright (c) 2026 DigiText. All rights reserved.
// Cây menu 2 cấp theo công đoạn thi công số hoá của Điều 9 Thông tư 05/2025/TT-BNV (3/11/2026): Dữ liệu chủ (bước 3) →
// Số hoá (bước 4: quét, đối chiếu, OCR, kết nối dữ liệu chủ) → Ký số, đóng gói (bước 6–7) → Nghiệm thu, bàn giao (bước 8);
// Giao nhận tài liệu (bước 1–2) đứng đầu trục; nhóm Kiểm tra sản phẩm (bước 5) thêm khi màn tương ứng ra đời. Dưới đường chia:
// Lưu trữ số (nghiệp vụ của lưu trữ lịch sử, sẽ tách bản riêng), Báo cáo, Tiện ích (mọi công cụ ngoài trục), Quản trị.
// Id mục con GIỮ NGUYÊN qua các lần sắp xếp để sidebarPrefs / tìm nhanh / deep link không gãy. Mỗi nút có `gate` = màn
// dùng để kiểm quyền (domain/permissions.ts V3z7). `badge` là số đếm nhỏ cạnh mục, lấy từ useNotifications.

import type { V3xu, V3xw, V3xx, V3xt, V3xTienIch } from "@/domain/types";
import type { V41v } from "@/components/Icon";

export type V46r =
  | { kind: "view"; view: V3xt }
  | { kind: "hoso"; pane: V3xu }
  | { kind: "khogia"; tab: V3xw }
  | { kind: "report"; tab: V3xx }
  | { kind: "tienich"; tab: V3xTienIch };

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
    id: "giaonhan",
    label: "Giao nhận tài liệu",
    icon: "nhapKho",
    step: "Bước 1–2 · khảo sát, kiểm đếm",
    gate: "handover",
    children: [{ id: "giaonhangiay", label: "Biên bản giao nhận", gate: "handover", leaf: { kind: "view", view: "handover" }, keywords: "ban giao tai lieu giay tra tai lieu goc hop ho so to" }],
  },
  {
    id: "dulieuchu",
    label: "Dữ liệu chủ",
    icon: "chinhly",
    step: "Bước 3 · mục lục, dữ liệu chủ",
    gate: "dossier",
    children: [
      { id: "danhmuccl", label: "Danh mục", gate: "dossier", leaf: { kind: "hoso", pane: "danhmuccl" }, keywords: "phong phong ban mang thbq chinh ly" },
      { id: "laphoso", label: "Lập hồ sơ", gate: "dossier", leaf: { kind: "hoso", pane: "laphoso" }, keywords: "ho so van ban nhap muc luc mlhs chinh ly" },
      { id: "hethonghoa", label: "Hệ thống hoá", gate: "dossier", leaf: { kind: "hoso", pane: "hethonghoa" }, keywords: "danh so hop" },
      { id: "khaithac", label: "Mục lục và kết xuất", gate: "khogia", leaf: { kind: "khogia", tab: "khaithac" }, keywords: "tra cuu in muc luc nhan hop excel" },
      { id: "dongbo", label: "Đồng bộ", gate: "dossier", leaf: { kind: "hoso", pane: "dongbo" }, badge: "clConflicts", keywords: "xung dot offline" },
    ],
  },
  {
    id: "sohoa",
    label: "Số hoá",
    icon: "sohoa",
    step: "Bước 4 · quét, OCR, đối chiếu",
    gate: "drive",
    children: [
      { id: "tree", label: "Cây thư mục", gate: "tree", leaf: { kind: "view", view: "tree" }, keywords: "phat cay ho so muc luc" },
      { id: "scan", label: "Quét", gate: "scan", leaf: { kind: "view", view: "scan" }, keywords: "may quet scan" },
      { id: "drive", label: "Kho tài liệu", gate: "drive", leaf: { kind: "view", view: "drive" }, keywords: "thu muc tep" },
      { id: "taolap", label: "Tài liệu đã quét", gate: "dossier", leaf: { kind: "hoso", pane: "sohoa" }, keywords: "tao lap co so du lieu pdf" },
      { id: "upload", label: "Đưa lên máy chủ", gate: "upload", leaf: { kind: "view", view: "upload" }, keywords: "dong bo tai len gom lan doi soat" },
      { id: "server", label: "Kho máy chủ", gate: "server", leaf: { kind: "view", view: "server" }, keywords: "may chu tai ve pdf/a" },
      { id: "ocr", label: "OCR và bóc tách", gate: "dossier", leaf: { kind: "hoso", pane: "ocr" }, keywords: "nhan dang 16 truong 18 truong soat duyet" },
      { id: "giaoviec", label: "Giao việc", gate: "workManage", leaf: { kind: "view", view: "workManage" }, keywords: "goi viec xac nhan" },
      { id: "viec", label: "Việc của tôi", gate: "work", leaf: { kind: "view", view: "work" }, badge: "myWork" },
      { id: "ketnoi", label: "Kết nối dữ liệu chủ", gate: "khogia", leaf: { kind: "khogia", tab: "ketnoi" }, keywords: "tao van ban tu ocr metadata tt05 tt02 du lieu chu" },
      { id: "nhatky", label: "Nhật ký số hoá", gate: "nhatky", leaf: { kind: "view", view: "nhatky" }, keywords: "san luong theo ngay nguoi chot nhat ky" },
    ],
  },
  {
    id: "kiemtra",
    label: "Kiểm tra sản phẩm",
    icon: "kiemTra",
    step: "Bước 5 · Điều 8, đối soát",
    gate: "qc",
    children: [{ id: "doisoat", label: "Kiểm tra theo phông", gate: "qc", leaf: { kind: "view", view: "qc" }, keywords: "doi soat so to trang quet pdf/a quet lai bien ban kiem tra qc" }],
  },
  {
    id: "kyso",
    label: "Ký số, đóng gói",
    icon: "kySo",
    step: "Bước 6–7 · ký số, gói AIP",
    gate: "sign",
    children: [
      { id: "kyso", label: "Ký số", gate: "sign", leaf: { kind: "view", view: "sign" }, keywords: "token chu ky" },
      { id: "donggoi", label: "Đóng gói", gate: "package", leaf: { kind: "view", view: "package" }, keywords: "sip aip dip goi tin" },
      { id: "thunop", label: "Thu nộp", gate: "submission", leaf: { kind: "view", view: "submission" }, badge: "notifications", keywords: "nop luu lich su nhap he thong" },
    ],
  },
  {
    id: "nghiemthu",
    label: "Nghiệm thu",
    icon: "bienBan",
    step: "Bước 8 · khối lượng, bàn giao",
    gate: "records",
    children: [
      { id: "nghiemthukl", label: "Nghiệm thu, bàn giao", gate: "acceptance", leaf: { kind: "view", view: "acceptance" }, keywords: "khoi luong hop dong ban giao goi aip bo ho so nghiem thu" },
      { id: "bienban", label: "Sổ biên bản", gate: "records", leaf: { kind: "view", view: "records" }, keywords: "phu luc vii bien ban" },
    ],
  },
  {
    id: "luutruso",
    label: "Lưu trữ số",
    icon: "luuTruSo",
    gate: "access",
    children: [
      { id: "khaithacso", label: "Khai thác", gate: "access", leaf: { kind: "view", view: "access" }, keywords: "yeu cau ban sao dip" },
      { id: "cungcap", label: "Cung cấp thông tin", gate: "info", leaf: { kind: "view", view: "info" } },
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
    id: "tienich",
    label: "Tiện ích",
    icon: "ketXuat",
    gate: "tienich",
    children: [
      { id: "tachtiff", label: "Tách TIFF", gate: "tiff", leaf: { kind: "view", view: "tiff" }, keywords: "tiff dang tach trang anh" },
      { id: "ti:pdfa", label: "Kiểm PDF/A", gate: "package", leaf: { kind: "tienich", tab: "pdfa" }, keywords: "verapdf iso 19005" },
      { id: "ti:kiemgoi", label: "Kiểm tra gói tin", gate: "package", leaf: { kind: "tienich", tab: "kiemgoi" }, keywords: "pkgcheck mets premis tt05" },
      { id: "ti:khongoai", label: "Đóng gói từ kho ngoài", gate: "package", leaf: { kind: "tienich", tab: "khongoai" }, keywords: "excel metadata thu muc pdf" },
      { id: "ti:nhanh", label: "Đóng gói nhanh", gate: "package", leaf: { kind: "tienich", tab: "nhanh" }, keywords: "chi doi chieu excel" },
      { id: "ti:vidu", label: "Ví dụ dạy máy", gate: "dossier", leaf: { kind: "tienich", tab: "vidu" }, keywords: "kie boc truong hoc may" },
      { id: "khogia", label: "Kho và giá", gate: "khogia", leaf: { kind: "khogia", tab: "khogia" }, keywords: "ke" },
      { id: "muontra", label: "Mượn – trả", gate: "khogia", leaf: { kind: "khogia", tab: "muontra" }, keywords: "phieu muon" },
      { id: "ti:chuyendulieu", label: "Chuyển dữ liệu cũ", gate: "dossier", leaf: { kind: "tienich", tab: "chuyendulieu" }, keywords: "sqlite ban cu" },
    ],
  },
  {
    id: "quantri",
    label: "Quản trị",
    icon: "quanTri",
    gate: "users",
    children: [
      { id: "users", label: "Người dùng và dự án", gate: "users", leaf: { kind: "view", view: "users" }, keywords: "phan quyen tai khoan" },
      { id: "roles", label: "Vai trò và quyền", gate: "roles", leaf: { kind: "view", view: "roles" }, keywords: "role phan quyen bo quyen mac dinh" },
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
  tienIchTab: V3xTienIch;
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
    case "tienich":
      return loc.view === "tienich" && loc.tienIchTab === leaf.tab;
  }
}

export interface NavActions {
  goto: (v: V3xt) => void;
  gotoHoSo: (p: V3xu) => void;
  gotoKhoGia: (t: V3xw) => void;
  gotoReport: (t: V3xx) => void;
  gotoTienIch: (t: V3xTienIch) => void;
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
    case "tienich":
      return nav.gotoTienIch(leaf.tab);
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
