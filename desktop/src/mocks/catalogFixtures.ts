// Copyright (c) 2026 DigiText. All rights reserved.

import type { V3ys, V3yj } from "@/domain/types";

export const V48e: Record<V3yj, string> = {
  DANG: "Khối Đảng",
  NN: "Khối Nhà nước",
};

export const V48f: Record<1 | 2 | 3, string> = {
  1: "Vĩnh viễn",
  2: "Có thời hạn",
  3: "Giữ lại",
};

export const V48g: V3ys = {
  phongs: [
    {
      maDuAn: "VPTU",
      tenDuAn: "Văn phòng Tỉnh ủy (mẫu)",
      maKhoi: "DANG",
      phongSo: "01",
      mucLucSo: "01",
    },
    {
      maDuAn: "BTGTU",
      tenDuAn: "Ban Tuyên giáo Tỉnh ủy (mẫu)",
      maKhoi: "DANG",
    },
    { maDuAn: "QUMAU", tenDuAn: "Quận ủy (mẫu)", maKhoi: "DANG" },
    { maDuAn: "STC", tenDuAn: "Sở Tài chính (mẫu)", maKhoi: "NN", phongSo: "12" },
    {
      maDuAn: "STNMT",
      tenDuAn: "Sở Tài nguyên & Môi trường (mẫu)",
      maKhoi: "NN",
    },
    { maDuAn: "UBNDX", tenDuAn: "UBND Xã (mẫu)", maKhoi: "NN" },
    { maDuAn: "TEST17", tenDuAn: "TEST17", maKhoi: "NN", isTest: true },
    { maDuAn: "3110", tenDuAn: "3110", maKhoi: "NN", isTest: true },
  ],
  phongBans: [
    { maDv: "VPTU-VP", maPhong: "VPTU", maDonVi: "00", tenDonVi: "Văn phòng", soTt: 1 },
    {
      maDv: "VPTU-TH",
      maPhong: "VPTU",
      maDonVi: "01",
      tenDonVi: "Phòng Tổng hợp",
      soTt: 2,
    },
    {
      maDv: "BTGTU-NV",
      maPhong: "BTGTU",
      maDonVi: "00",
      tenDonVi: "Phòng Nghiệp vụ",
      soTt: 1,
    },
    {
      maDv: "STC-VP",
      maPhong: "STC",
      maDonVi: "00",
      tenDonVi: "Văn phòng Sở",
      soTt: 1,
    },
    {
      maDv: "STC-NS",
      maPhong: "STC",
      maDonVi: "01",
      tenDonVi: "Phòng Quản lý Ngân sách",
      soTt: 2,
    },
    { maDv: "STC-TTr", maPhong: "STC", maDonVi: "02", tenDonVi: "Thanh tra Sở", soTt: 3 },
    {
      maDv: "STNMT-VP",
      maPhong: "STNMT",
      maDonVi: "00",
      tenDonVi: "Văn phòng Sở",
      soTt: 1,
    },
    {
      maDv: "STNMT-DD",
      maPhong: "STNMT",
      maDonVi: "01",
      tenDonVi: "Phòng Đo đạc & Bản đồ",
      soTt: 2,
    },
    {
      maDv: "UBNDX-VP",
      maPhong: "UBNDX",
      maDonVi: "00",
      tenDonVi: "Văn phòng - Thống kê",
      soTt: 1,
    },
  ],
  mangs: [
    {
      mangCvid: 1,
      maMang: "01",
      tenMang: "Hành chính - Quản trị",
      maDv: "VPTU-VP",
      soTt: 1,
    },
    {
      mangCvid: 2,
      maMang: "02",
      tenMang: "Tài chính Đảng",
      maDv: "VPTU-VP",
      soTt: 2,
    },
    {
      mangCvid: 3,
      maMang: "01",
      tenMang: "Tổng hợp - Báo cáo",
      maDv: "VPTU-TH",
      soTt: 1,
    },
    { mangCvid: 4, maMang: "01", tenMang: "Tuyên truyền", maDv: "BTGTU-NV", soTt: 1 },
    {
      mangCvid: 5,
      maMang: "01",
      tenMang: "Văn thư - Lưu trữ",
      maDv: "STC-VP",
      soTt: 1,
    },
    {
      mangCvid: 6,
      maMang: "01",
      tenMang: "Quản lý chi thường xuyên",
      maDv: "STC-NS",
      soTt: 1,
    },
    {
      mangCvid: 7,
      maMang: "02",
      tenMang: "Quản lý chi đầu tư",
      maDv: "STC-NS",
      soTt: 2,
    },
    {
      mangCvid: 8,
      maMang: "03",
      tenMang: "Quyết toán ngân sách",
      maDv: "STC-NS",
      soTt: 3,
    },
    {
      mangCvid: 9,
      maMang: "01",
      tenMang: "Giải quyết khiếu nại",
      maDv: "STC-TTr",
      soTt: 1,
    },
    {
      mangCvid: 10,
      maMang: "01",
      tenMang: "Hồ sơ địa chính",
      maDv: "STNMT-DD",
      soTt: 1,
    },
    {
      mangCvid: 11,
      maMang: "01",
      tenMang: "Hộ tịch - Chứng thực",
      maDv: "UBNDX-VP",
      soTt: 1,
    },
  ],
  thbqs: [
    { thbqid: 1, tenThbq: "Vĩnh viễn", trangThai: 1, giaTri: 0, sort: 1 },
    { thbqid: 2, tenThbq: "5 năm", trangThai: 2, giaTri: 5, sort: 2 },
    { thbqid: 3, tenThbq: "10 năm", trangThai: 2, giaTri: 10, sort: 3 },
    { thbqid: 4, tenThbq: "20 năm", trangThai: 2, giaTri: 20, sort: 4 },
    { thbqid: 5, tenThbq: "50 năm", trangThai: 2, giaTri: 50, sort: 5 },
    { thbqid: 6, tenThbq: "70 năm", trangThai: 2, giaTri: 70, sort: 6 },
    {
      thbqid: 7,
      tenThbq: "Giữ lại để xét hủy",
      trangThai: 3,
      giaTri: 0,
      sort: 7,
    },
  ],
  ngonNgus: [
    { maNgonNgu: "01", tenNgonNgu: "Tiếng Việt" },
    { maNgonNgu: "02", tenNgonNgu: "English" },
  ],
  loaiVanBans: [
    { maLoai: "01", tenLoai: "Nghị quyết" },
    { maLoai: "02", tenLoai: "Quyết định" },
    { maLoai: "03", tenLoai: "Chỉ thị" },
    { maLoai: "04", tenLoai: "Công văn" },
  ],
  hoSos: [],
};

export const V48h: V3ys = {
  phongs: [],
  phongBans: [],
  mangs: [],
  thbqs: [],
  ngonNgus: [],
  loaiVanBans: [],
  hoSos: [],
};
