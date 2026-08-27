// Copyright (c) 2026 DigiText. All rights reserved.

export type V4ta = "kiem_tra" | "nhat_ky_sao_luu" | "sao_luu" | "su_co" | "thu_nop_truc_tiep";

export interface V4tb {
  key: string;
  label: string;
  kind: "text" | "number";
  optional?: boolean;
  multiline?: boolean;
}

export interface V4tc {
  key: string;
  label: string;
  optional?: boolean;
}

export interface V4td {
  key: string;
  label: string;

  simple?: boolean;
  columns: V4tc[];
  minRows: number;
}

export interface V4te {
  kind: V4ta;
  label: string;

  legal: string;
  fields: V4tb[];
  lists: V4td[];
}

const t = (V2u0: string, V2u1: string, V2u2 = false): V4tb => ({
  key: V2u0,
  label: V2u1,
  kind: "text",
  optional: V2u2,
});
const n = (V2u3: string, V2u4: string): V4tb => ({ key: V2u3, label: V2u4, kind: "number" });

export const V4tf: V4te[] = [
  {
    kind: "kiem_tra",
    label: "Biên bản kiểm tra tài liệu lưu trữ số",
    legal: "Mục 1 Phụ lục VII — Điều 29",
    fields: [
      t("donVi", "Đơn vị kiểm tra"),
      t("diaDiem", "Địa điểm"),
      t("ngay", "Ngày lập (DD/MM/YYYY)"),
      t("phamVi", "Phạm vi kiểm tra (phông / chọn mẫu)"),
      n("tongSoTep", "Tổng số tệp kiểm tra"),
      n("soDat", "Số tệp đạt"),
      n("soLoi", "Số tệp lỗi"),
      t("phuongAnKhacPhuc", "Phương án khắc phục (Điều 29.4.g)", true),
      t("ketLuan", "Kết luận", true),
      t("nguoiLap", "Người lập biên bản"),
      t("thuTruong", "Thủ trưởng đơn vị"),
    ],
    lists: [
      {
        key: "thanhPhan",
        label: "Thành phần tham gia",
        simple: true,
        columns: [],
        minRows: 1,
      },
      {
        key: "tepLoi",
        label: "Danh mục tệp lỗi (Điều 29.4.đ)",
        columns: [
          { key: "tenTep", label: "Tên tệp" },
          { key: "diaChi", label: "Địa chỉ lưu trữ" },
          { key: "moTaLoi", label: "Mô tả lỗi", optional: true },
        ],
        minRows: 0,
      },
    ],
  },
  {
    kind: "nhat_ky_sao_luu",
    label: "Nhật ký sao lưu",
    legal: "Mục 2 Phụ lục VII — Điều 30",
    fields: [
      t("donVi", "Đơn vị"),
      t("ky", "Kỳ ghi nhật ký"),
      t("nguoiLap", "Người lập"),
    ],
    lists: [
      {
        key: "entries",
        label: "Các lần sao lưu",
        columns: [
          { key: "thoiGian", label: "Thời gian" },
          { key: "loai", label: "Loại (gia tăng/đầy đủ)" },
          { key: "nguonDuLieu", label: "Nguồn dữ liệu" },
          { key: "phuongTien", label: "Phương tiện" },
          { key: "ketQua", label: "Kết quả" },
          { key: "nguoiThucHien", label: "Người thực hiện" },
        ],
        minRows: 1,
      },
    ],
  },
  {
    kind: "sao_luu",
    label: "Biên bản sao lưu",
    legal: "Mục 3 Phụ lục VII — Điều 30",
    fields: [
      t("donVi", "Đơn vị"),
      t("diaDiem", "Địa điểm"),
      t("ngay", "Ngày lập (DD/MM/YYYY)"),
      t("loaiSaoLuu", "Loại sao lưu (Điều 30.1)"),
      t("nguonDuLieu", "Nguồn dữ liệu (CSDL + mã nguồn — 30.3.c)"),
      t("phuongTien", "Phương tiện lưu trữ"),
      t("moTaPhuongTien", "Mô tả phương tiện", true),
      t("ketQuaKiemTra", "Kết quả kiểm tra sau sao lưu (30.3.d)"),
      t("noiBaoQuan", "Nơi bảo quản phương tiện (30.3.đ)"),
      t("nguoiThucHien", "Người thực hiện"),
      t("nguoiKiemTra", "Người kiểm tra"),
    ],
    lists: [],
  },
  {
    kind: "su_co",
    label: "Biên bản xử lý sự cố và phục hồi",
    legal: "Mục 4 Phụ lục VII — Điều 31",
    fields: [
      t("donVi", "Đơn vị"),
      t("diaDiem", "Địa điểm"),
      t("ngay", "Ngày lập (DD/MM/YYYY)"),
      t("thoiDiemSuCo", "Thời điểm xảy ra sự cố"),
      t("moTaSuCo", "Mô tả sự cố"),
      t("nguyenNhan", "Nguyên nhân (phần cứng / phần mềm — 31.1)"),
      t(
        "nguonPhucHoi",
        "Nguồn phục hồi (bản sao lưu gần nhất — 31.4)",
      ),
      t("phuongPhapPhucHoi", "Phương pháp phục hồi"),
      t("ketQuaKiemTra", "Kết quả kiểm tra sau phục hồi (31.5)"),
      t("nguoiXuLy", "Người xử lý sự cố"),
      t("phuTrach", "Phụ trách đơn vị"),
    ],
    lists: [
      {
        key: "bienPhap",
        label: "Biện pháp cách ly / khắc phục (31.2, 31.3)",
        simple: true,
        columns: [],
        minRows: 1,
      },
    ],
  },
  {
    kind: "thu_nop_truc_tiep",
    label: "Biên bản thu nộp (giao nhận trực tiếp)",
    legal: "Điều 22.2.đ",
    fields: [
      t("maDangKy", "Mã đăng ký yêu cầu (nếu có)", true),
      t("maCoQuan", "Mã cơ quan, tổ chức nộp"),
      t("tenCoQuan", "Tên cơ quan, tổ chức nộp"),
      n("soLanNop", "Số lần nộp"),
      n(
        "tongDangKy",
        "Tổng số hồ sơ, tài liệu nộp theo đăng ký",
      ),
      n("tongGiaoNhan", "Tổng số hồ sơ, tài liệu giao nhận"),
      t("tongDungLuong", "Tổng số dung lượng (vd 1,2 GB)"),
      t("thoiGianGiaoNhan", "Thời gian giao nhận (DD/MM/YYYY)"),
      t("nguoiGiao", "Người giao"),
      t("nguoiNhan", "Người nhận"),
      t("ghiChu", "Ghi chú khác", true),
    ],
    lists: [
      {
        key: "thietBi",
        label:
          "Thiết bị lưu trữ bàn giao (tình trạng, mô tả — 22.2.đ)",
        columns: [
          { key: "ten", label: "Thiết bị" },
          { key: "tinhTrang", label: "Tình trạng" },
          { key: "moTa", label: "Mô tả chi tiết", optional: true },
        ],
        minRows: 1,
      },
    ],
  },
];

export function V4tg(V2u5: V4ta): V4te {
  const t = V4tf.find((x) => x.kind === V2u5);
  if (!t) throw new Error(`không có mẫu: ${V2u5}`);
  return t;
}

export interface V4th {
  payload: Record<string, unknown> | null;
  errors: string[];
}

export function V4ti(
  template: V4te,
  V2u6: Record<string, string>,
  V2u7: Record<string, Record<string, string>[]>,
): V4th {
  const V2u8: string[] = [];
  const V2u9: Record<string, unknown> = {};

  for (const f of template.fields) {
    const V2ua = (V2u6[f.key] ?? "").trim();
    if (!V2ua) {
      if (!f.optional) V2u8.push(`Thiếu "${f.label}".`);
      continue;
    }
    if (f.kind === "number") {
      const V2ub = Number(V2ua);
      if (!Number.isFinite(V2ub) || V2ub < 0) {
        V2u8.push(`"${f.label}" phải là số không âm.`);
        continue;
      }
      V2u9[f.key] = V2ub;
    } else {
      V2u9[f.key] = V2ua;
    }
  }

  for (const l of template.lists) {
    const V2uc = (V2u7[l.key] ?? []).filter((r) => Object.values(r).some((v) => (v ?? "").trim()));
    if (V2uc.length < l.minRows) {
      V2u8.push(`"${l.label}" cần tối thiểu ${l.minRows} dòng.`);
      continue;
    }
    if (l.simple) {
      V2u9[l.key] = V2uc.map((r) => (r.value ?? "").trim()).filter(Boolean);
      if ((V2u9[l.key] as string[]).length < l.minRows) {
        V2u8.push(`"${l.label}" cần tối thiểu ${l.minRows} dòng.`);
      }
      continue;
    }
    const V2ud: Record<string, unknown>[] = [];
    for (const [i, row] of V2uc.entries()) {
      const V2ue: Record<string, unknown> = {};
      for (const c of l.columns) {
        const v = (row[c.key] ?? "").trim();
        if (!v) {
          if (!c.optional) V2u8.push(`"${l.label}" dòng ${i + 1}: thiếu "${c.label}".`);
          continue;
        }
        V2ue[c.key] = v;
      }
      V2ud.push(V2ue);
    }
    V2u9[l.key] = V2ud;
  }

  return V2u8.length > 0 ? { payload: null, errors: V2u8 } : { payload: V2u9, errors: [] };
}
