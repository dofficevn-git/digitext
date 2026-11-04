// Copyright (c) 2026 DigiText. All rights reserved.

import type { V3xs, V3xt } from "./types";

export type V3yt = "read" | "operate" | "admin";

const V3yu: Record<V3xs, number> = {
  customer: 0,
  user: 1,
  admin: 2,
  manager: 2,
  super_admin: 3,
};

const V3yv: Record<V3yt, number> = { read: 0, operate: 1, admin: 2 };

export function V3yw(Ve: V3xs, Vf: V3yt): boolean {
  return V3yu[Ve] >= V3yv[Vf];
}

export type V3yx =
  | "dossier.read"
  | "dossier.create"
  | "dossier.update"
  | "dossier.delete"
  | "dossier.export"
  | "catalog.read"
  | "catalog.create"
  | "catalog.update"
  | "catalog.delete"
  | "catalog.import"
  | "catalog.export"
  | "scan.view"
  | "scan.execute"
  | "editor.view"
  | "editor.edit"
  | "upload.execute"
  | "sign.view"
  | "sign.execute"
  | "drive.view"
  | "drive.download"
  | "document.list"
  | "document.download"
  | "document.review"
  | "package.view"
  | "package.create"
  | "package.download"
  | "submission.view"
  | "submission.create"
  | "submission.review"
  | "submission.approve"
  | "record.view"
  | "record.create"
  | "access.view"
  | "access.request"
  | "access.review"
  | "disposal.view"
  | "disposal.review"
  | "disposal.decide"
  | "bk.view"
  | "bk.manage"
  | "tree.view"
  | "tree.assign"
  | "stats.view"
  | "stats.view_project"
  | "user.view"
  | "user.create"
  | "user.update"
  | "user.revoke"
  | "user.reset_password"
  | "user.grant_permissions"
  | "project.view"
  | "project.create"
  | "project.update"
  | "project.assign_admin"
  | "settings.view"
  | "settings.update"
  | "device.view"
  | "device.revoke"
  | "work.execute"
  | "work.manage"
  | "dossier.systematize"
  | "khogia.manage"
  | "loan.view"
  | "loan.manage";

export interface V3yy {
  perm: V3yx;
  label: string;
}

export interface V3yz {
  key: string;
  label: string;
  perms: V3yy[];
}

export const V3z0: V3yz[] = [
  {
    key: "dossier",
    label: "Hồ sơ",
    perms: [
      { perm: "dossier.read", label: "Xem hồ sơ" },
      { perm: "dossier.create", label: "Tạo hồ sơ" },
      { perm: "dossier.update", label: "Sửa hồ sơ" },
      { perm: "dossier.delete", label: "Xoá hồ sơ" },
      { perm: "dossier.export", label: "Xuất hồ sơ" },
    ],
  },
  {
    key: "catalog",
    label: "Danh mục",
    perms: [
      { perm: "catalog.read", label: "Xem danh mục" },
      { perm: "catalog.create", label: "Thêm mục" },
      { perm: "catalog.update", label: "Sửa mục" },
      { perm: "catalog.delete", label: "Xoá mục" },
      { perm: "catalog.import", label: "Nhập từ Excel" },
      { perm: "catalog.export", label: "Xuất danh mục" },
    ],
  },
  {
    key: "scan",
    label: "Quét",
    perms: [
      { perm: "scan.view", label: "Mở màn quét" },
      { perm: "scan.execute", label: "Thực hiện quét" },
    ],
  },
  {
    key: "editor",
    label: "Chỉnh ảnh",
    perms: [
      { perm: "editor.view", label: "Mở trình chỉnh ảnh" },
      { perm: "editor.edit", label: "Chỉnh sửa ảnh" },
    ],
  },
  {
    key: "upload",
    label: "Tải lên",
    perms: [{ perm: "upload.execute", label: "Tải tài liệu lên máy chủ" }],
  },
  {
    key: "sign",
    label: "Ký số",
    perms: [
      { perm: "sign.view", label: "Xem ký số" },
      { perm: "sign.execute", label: "Thực hiện ký số" },
    ],
  },
  {
    key: "drive",
    label: "Kho bàn giao",
    perms: [
      { perm: "drive.view", label: "Xem kho" },
      { perm: "drive.download", label: "Tải từ kho" },
    ],
  },
  {
    key: "document",
    label: "Tài liệu đã nộp (máy chủ)",
    perms: [
      { perm: "document.list", label: "Xem danh sách tài liệu" },
      { perm: "document.download", label: "Tải tài liệu về" },
      {
        perm: "document.review",
        label: "Duyệt / từ chối kết quả bóc tách",
      },
    ],
  },
  {
    key: "work",
    label: "Gói việc xác nhận trường (cộng tác viên)",
    perms: [
      { perm: "work.execute", label: "Làm gói việc được giao (xác nhận / kiểm tra chéo)" },
      { perm: "work.manage", label: "Tạo, giao và duyệt gói việc" },
    ],
  },
  {
    key: "chinhly",
    label: "Chỉnh lý (máy chủ)",
    perms: [
      { perm: "dossier.systematize", label: "Hệ thống hoá: đánh số hồ sơ, gán hộp/bó/tập, xuất/nhập mục lục tạm" },
      { perm: "khogia.manage", label: "Quản lý kho / giá lưu trữ, xếp hồ sơ lên giá" },
      { perm: "loan.view", label: "Xem phiếu mượn / trả hồ sơ giấy" },
      { perm: "loan.manage", label: "Lập phiếu mượn, ghi trả hồ sơ giấy" },
    ],
  },
  {
    key: "package",
    label: "Đóng gói SIP/AIP/DIP",
    perms: [
      { perm: "package.view", label: "Xem gói tin" },
      { perm: "package.create", label: "Tạo gói tin" },
      { perm: "package.download", label: "Tải gói tin" },
    ],
  },
  {
    key: "submission",
    label: "Thu nộp lưu trữ lịch sử",
    perms: [
      { perm: "submission.view", label: "Xem lần nộp" },
      { perm: "submission.create", label: "Đăng ký nộp" },
      {
        perm: "submission.review",
        label: "Xác nhận / tiếp nhận / chấm hồ sơ",
      },
      { perm: "submission.approve", label: "Phê duyệt / hoàn thành" },
    ],
  },
  {
    key: "record",
    label: "Biên bản (Phụ lục VII)",
    perms: [
      { perm: "record.view", label: "Xem / tải biên bản" },
      { perm: "record.create", label: "Lập biên bản" },
    ],
  },
  {
    key: "access",
    label: "Khai thác lưu trữ số",
    perms: [
      {
        perm: "access.view",
        label: "Xem yêu cầu / đọc bản người dùng",
      },
      { perm: "access.request", label: "Đăng ký yêu cầu khai thác" },
      { perm: "access.review", label: "Xét duyệt / phí / cấp DIP" },
    ],
  },
  {
    key: "disposal",
    label: "Hủy tài liệu hết giá trị",
    perms: [
      {
        perm: "disposal.view",
        label: "Xem danh mục / đợt hủy (thành viên Hội đồng)",
      },
      {
        perm: "disposal.review",
        label: "Rà soát / lập đợt / thẩm định",
      },
      { perm: "disposal.decide", label: "Quyết định + thao tác hủy" },
    ],
  },
  {
    key: "bk",
    label: "Bảo quản & sao lưu",
    perms: [
      { perm: "bk.view", label: "Xem kiểm tra / sổ sao lưu" },
      { perm: "bk.manage", label: "Chạy kiểm tra / ghi nhận sao lưu" },
    ],
  },
  {
    key: "tree",
    label: "Cây thư mục",
    perms: [
      { perm: "tree.view", label: "Xem cây thư mục" },
      { perm: "tree.assign", label: "Phát cây thư mục" },
    ],
  },
  {
    key: "stats",
    label: "Thống kê",
    perms: [
      { perm: "stats.view", label: "Báo cáo / xem số liệu" },
      { perm: "stats.view_project", label: "Xem số liệu toàn dự án" },
    ],
  },
  {
    key: "user",
    label: "Quản lý người dùng",
    perms: [
      { perm: "user.view", label: "Xem danh sách" },
      { perm: "user.create", label: "Tạo tài khoản" },
      { perm: "user.update", label: "Sửa tài khoản" },
      { perm: "user.revoke", label: "Vô hiệu hoá tài khoản" },
      { perm: "user.reset_password", label: "Đặt lại mật khẩu" },
      { perm: "user.grant_permissions", label: "Phân quyền cho người khác" },
    ],
  },
  {
    key: "project",
    label: "Quản lý dự án",
    perms: [
      { perm: "project.view", label: "Xem dự án" },
      { perm: "project.create", label: "Tạo dự án" },
      { perm: "project.update", label: "Sửa dự án" },
      { perm: "project.assign_admin", label: "Gán admin phụ trách" },
    ],
  },
  {
    key: "device",
    label: "Thiết bị",
    perms: [
      { perm: "device.view", label: "Xem thiết bị" },
      { perm: "device.revoke", label: "Thu hồi thiết bị" },
    ],
  },
  {
    key: "settings",
    label: "Cài đặt",
    perms: [
      { perm: "settings.view", label: "Xem cài đặt" },
      { perm: "settings.update", label: "Đổi cài đặt" },
    ],
  },
];

export const V3z1: V3yx[] = V3z0.flatMap((g) => g.perms.map((p) => p.perm));

const V3z2: Record<V3yx, string> = Object.fromEntries(
  V3z0.flatMap((g) => g.perms.map((p) => [p.perm, p.label])),
) as Record<V3yx, string>;

export function V3z3(Vg: V3yx): string {
  return V3z2[Vg] ?? Vg;
}

const V3z4: Record<V3yx, number> = {
  "dossier.read": 0,
  "catalog.read": 0,
  "drive.view": 0,
  "drive.download": 0,
  "tree.view": 0,
  "settings.view": 0,

  "access.view": 0,
  "access.request": 0,
  "dossier.create": 1,
  "dossier.update": 1,
  "dossier.export": 1,
  "catalog.create": 1,
  "catalog.update": 1,
  "catalog.import": 1,
  "catalog.export": 1,
  "scan.view": 1,
  "scan.execute": 1,
  "editor.view": 1,
  "editor.edit": 1,
  "upload.execute": 1,
  "sign.view": 1,
  "sign.execute": 1,
  "stats.view": 1,
  "settings.update": 1,
  "package.view": 1,
  "package.create": 1,
  "package.download": 1,
  "submission.view": 1,
  "submission.create": 1,
  "submission.review": 2,
  "submission.approve": 2,
  "access.review": 2,

  "disposal.view": 1,
  "disposal.review": 2,
  "disposal.decide": 2,
  "bk.view": 2,
  "bk.manage": 2,
  "record.view": 1,
  "record.create": 1,
  "dossier.delete": 2,
  "catalog.delete": 2,
  "document.list": 2,
  "document.download": 2,
  "document.review": 2,
  "tree.assign": 2,
  "stats.view_project": 2,
  "user.view": 2,
  "user.create": 2,
  "user.update": 2,
  "user.revoke": 2,
  "user.reset_password": 2,
  "user.grant_permissions": 2,
  "project.view": 2,
  "project.create": 2,
  "project.update": 2,
  "device.view": 2,
  "device.revoke": 2,
  "project.assign_admin": 3,
  "work.execute": 1,
  "work.manage": 2,
  "dossier.systematize": 2,
  "khogia.manage": 2,
  "loan.view": 1,
  "loan.manage": 1,
};

// Quản lý (15/10/2026, khớp digitext-core permission.rs): như quản trị nhưng KHÔNG xoá; được gán quản trị dự án.
const QUAN_LY_CHAN: V3yx[] = ["dossier.delete", "catalog.delete", "disposal.decide"];

export function V3z5(Vh: V3xs, Vi: V3yx): boolean {
  if (Vh === "manager") {
    if (QUAN_LY_CHAN.includes(Vi)) return false;
    if (Vi === "project.assign_admin") return true;
  }
  return V3yu[Vh] >= V3z4[Vi];
}

export function V3z6(Vj: V3xs): V3yx[] {
  return V3z1.filter((p) => V3z5(Vj, p));
}

const V3z7: Record<V3xt, V3yx> = {
  home: "drive.view",
  drive: "drive.view",
  tree: "tree.view",
  dossier: "dossier.read",
  khogia: "dossier.read",
  scan: "scan.view",
  editor: "editor.view",
  upload: "upload.execute",
  sign: "sign.view",
  tiff: "sign.view",
  users: "user.view",
  roles: "user.view",
  settings: "settings.view",

  package: "package.view",

  submission: "submission.view",

  records: "record.view",

  access: "access.view",

  disposal: "disposal.view",

  info: "access.view",

  bk: "bk.view",

  server: "document.list",

  report: "stats.view_project",

  work: "work.execute",

  workManage: "work.manage",
  // Hub Tiện ích mở cho mọi người đã đăng nhập; từng tab tự kiểm quyền thật (package.view, document.review, dossier.read).
  tienich: "drive.view",
  // Thi công Điều 9 TT05 (3/11/2026): giao nhận giấy = sổ biên bản; nhật ký = số liệu dashboard của dự án.
  handover: "record.view",
  nhatky: "drive.view",
  qc: "document.list",
  acceptance: "stats.view_project",
};

export function V3z8(Vk: V3yx[], view: V3xt): boolean {
  return Vk.includes(V3z7[view]);
}

export function V3z9(Vl: V3yx[] | undefined, Vm: V3yx): boolean {
  return !!Vl?.includes(Vm);
}

export function V3za(Vn: V3xs): string {
  switch (Vn) {
    case "super_admin":
      return "Quản trị hệ thống";
    case "manager":
      return "Quản lý";
    case "admin":
      return "Quản trị dự án";
    case "user":
      return "Nhân viên dự án";
    case "customer":
      return "Khách hàng";
  }
}
