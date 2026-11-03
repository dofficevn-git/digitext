// Copyright (c) 2026 DigiText. All rights reserved.
// Mẫu biên bản theo Phụ lục VII Thông tư 05/2025/TT-BNV. Nhãn ô là tên thường; căn cứ pháp lý để riêng ở `legal`.
// 3/11/2026: thêm mẫu THI CÔNG SỐ HOÁ theo Điều 9 (giao nhận / trả tài liệu giấy, nhật ký số hoá…). Các mẫu này là
// "generic": máy trạm dựng sẵn các khối (đoạn, ô, bảng, chữ ký) trong payload, máy chủ chỉ in PDF — thêm mẫu mới chỉ cần
// thêm object ở đây (kind phải có trong CHECK của bảng records, migration 0024).

export type V4ta = "kiem_tra" | "nhat_ky_sao_luu" | "sao_luu" | "su_co" | "thu_nop_truc_tiep" | "giao_nhan_giay" | "tra_tai_lieu_giay" | "nhat_ky_so_hoa";

export interface V4tb {
  key: string;
  label: string;
  kind: "text" | "number";
  optional?: boolean;
  multiline?: boolean;
  /** Gợi ý cách điền, hiện dưới ô. */
  hint?: string;
  /** Căn cứ pháp lý của ô, hiện ở tooltip. */
  legal?: string;
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
  legal?: string;
}

export interface V4te {
  kind: V4ta;
  label: string;
  /** Mô tả một câu dùng ở thẻ chọn mẫu. */
  description: string;
  legal: string;
  fields: V4tb[];
  lists: V4td[];
  /** Mẫu generic (Điều 9): máy trạm dựng khối PDF; `refKey` = ô làm khoá phụ `ref` (mã phông, ngày…). */
  generic?: { refKey?: string; signLeft: string; signRight: string; tieuDe: string };
}

const t = (key: string, label: string, optional = false, extra: Partial<V4tb> = {}): V4tb => ({ key, label, kind: "text", optional, ...extra });
const n = (key: string, label: string): V4tb => ({ key, label, kind: "number" });
const NGAY = "Ghi dạng DD/MM/YYYY";

export const V4tf: V4te[] = [
  {
    kind: "kiem_tra",
    label: "Biên bản kiểm tra tài liệu lưu trữ số",
    description: "Kết quả kiểm tra định kỳ tính toàn vẹn tài liệu, kèm danh mục tệp lỗi.",
    legal: "Mục 1 Phụ lục VII, Điều 29",
    fields: [
      t("donVi", "Đơn vị kiểm tra"),
      t("diaDiem", "Địa điểm"),
      t("ngay", "Ngày lập", false, { hint: NGAY }),
      t("phamVi", "Phạm vi kiểm tra", false, { hint: "Phông hoặc cách chọn mẫu" }),
      n("tongSoTep", "Tổng số tệp kiểm tra"),
      n("soDat", "Số tệp đạt"),
      n("soLoi", "Số tệp lỗi"),
      t("phuongAnKhacPhuc", "Phương án khắc phục", true, { legal: "Điều 29.4.g" }),
      t("ketLuan", "Kết luận", true),
      t("nguoiLap", "Người lập biên bản"),
      t("thuTruong", "Thủ trưởng đơn vị"),
    ],
    lists: [
      { key: "thanhPhan", label: "Thành phần tham gia", simple: true, columns: [], minRows: 1 },
      {
        key: "tepLoi",
        label: "Danh mục tệp lỗi",
        legal: "Điều 29.4.đ",
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
    description: "Các lần sao lưu trong một kỳ: thời gian, loại, nguồn, phương tiện, kết quả.",
    legal: "Mục 2 Phụ lục VII, Điều 30",
    fields: [t("donVi", "Đơn vị"), t("ky", "Kỳ ghi nhật ký"), t("nguoiLap", "Người lập")],
    lists: [
      {
        key: "entries",
        label: "Các lần sao lưu",
        columns: [
          { key: "thoiGian", label: "Thời gian" },
          { key: "loai", label: "Loại" },
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
    description: "Một lần sao lưu đầy đủ: nguồn, phương tiện, kết quả kiểm tra, nơi bảo quản.",
    legal: "Mục 3 Phụ lục VII, Điều 30",
    fields: [
      t("donVi", "Đơn vị"),
      t("diaDiem", "Địa điểm"),
      t("ngay", "Ngày lập", false, { hint: NGAY }),
      t("loaiSaoLuu", "Loại sao lưu", false, { legal: "Điều 30.1" }),
      t("nguonDuLieu", "Nguồn dữ liệu", false, { hint: "Cơ sở dữ liệu và mã nguồn", legal: "Điều 30.3.c" }),
      t("phuongTien", "Phương tiện lưu trữ"),
      t("moTaPhuongTien", "Mô tả phương tiện", true),
      t("ketQuaKiemTra", "Kết quả kiểm tra sau sao lưu", false, { legal: "Điều 30.3.d" }),
      t("noiBaoQuan", "Nơi bảo quản phương tiện", false, { legal: "Điều 30.3.đ" }),
      t("nguoiThucHien", "Người thực hiện"),
      t("nguoiKiemTra", "Người kiểm tra"),
    ],
    lists: [],
  },
  {
    kind: "su_co",
    label: "Biên bản xử lý sự cố và phục hồi",
    description: "Sự cố đã xảy ra, biện pháp cách ly, nguồn và kết quả phục hồi.",
    legal: "Mục 4 Phụ lục VII, Điều 31",
    fields: [
      t("donVi", "Đơn vị"),
      t("diaDiem", "Địa điểm"),
      t("ngay", "Ngày lập", false, { hint: NGAY }),
      t("thoiDiemSuCo", "Thời điểm xảy ra sự cố"),
      t("moTaSuCo", "Mô tả sự cố"),
      t("nguyenNhan", "Nguyên nhân", false, { hint: "Phần cứng hay phần mềm", legal: "Điều 31.1" }),
      t("nguonPhucHoi", "Nguồn phục hồi", false, { hint: "Bản sao lưu gần nhất", legal: "Điều 31.4" }),
      t("phuongPhapPhucHoi", "Phương pháp phục hồi"),
      t("ketQuaKiemTra", "Kết quả kiểm tra sau phục hồi", false, { legal: "Điều 31.5" }),
      t("nguoiXuLy", "Người xử lý sự cố"),
      t("phuTrach", "Phụ trách đơn vị"),
    ],
    lists: [{ key: "bienPhap", label: "Biện pháp cách ly, khắc phục", legal: "Điều 31.2, 31.3", simple: true, columns: [], minRows: 1 }],
  },
  {
    kind: "thu_nop_truc_tiep",
    label: "Biên bản thu nộp giao nhận trực tiếp",
    description: "Giao nhận hồ sơ, tài liệu và thiết bị lưu trữ trực tiếp giữa hai bên.",
    legal: "Điều 22.2.đ",
    fields: [
      t("maDangKy", "Mã đăng ký yêu cầu", true),
      t("maCoQuan", "Mã cơ quan, tổ chức nộp"),
      t("tenCoQuan", "Tên cơ quan, tổ chức nộp"),
      n("soLanNop", "Số lần nộp"),
      n("tongDangKy", "Tổng số hồ sơ, tài liệu theo đăng ký"),
      n("tongGiaoNhan", "Tổng số hồ sơ, tài liệu giao nhận"),
      t("tongDungLuong", "Tổng dung lượng", false, { hint: "Ví dụ 1,2 GB" }),
      t("thoiGianGiaoNhan", "Thời gian giao nhận", false, { hint: NGAY }),
      t("nguoiGiao", "Người giao"),
      t("nguoiNhan", "Người nhận"),
      t("ghiChu", "Ghi chú khác", true),
    ],
    lists: [
      {
        key: "thietBi",
        label: "Thiết bị lưu trữ bàn giao",
        legal: "Điều 22.2.đ",
        columns: [
          { key: "ten", label: "Thiết bị" },
          { key: "tinhTrang", label: "Tình trạng" },
          { key: "moTa", label: "Mô tả chi tiết", optional: true },
        ],
        minRows: 1,
      },
    ],
  },
  {
    kind: "giao_nhan_giay",
    label: "Biên bản giao nhận tài liệu giấy",
    description: "Đơn vị bàn giao tài liệu giấy cho nhà thầu số hoá: kiểm đếm hộp, hồ sơ, tờ và tình trạng.",
    legal: "Khoản 1 Điều 9",
    fields: [
      t("benGiao", "Bên giao (cơ quan, tổ chức)"),
      t("benNhan", "Bên nhận (đơn vị số hoá)"),
      t("phong", "Phông tài liệu"),
      t("maPhong", "Mã phông", true, { hint: "Dùng để lọc sổ biên bản theo phông" }),
      t("diaDiem", "Địa điểm"),
      t("ngay", "Ngày giao nhận", false, { hint: NGAY }),
      n("tongHop", "Tổng số hộp"),
      n("tongHoSo", "Tổng số hồ sơ"),
      t("tongTo", "Tổng số tờ", true),
      t("tinhTrang", "Tình trạng vật lý chung", false, { hint: "Ví dụ: tốt; 3 hộp có tài liệu rách, mờ chữ" }),
      t("nguoiGiao", "Người giao"),
      t("nguoiNhan", "Người nhận"),
      t("ghiChu", "Ghi chú", true, { multiline: true }),
    ],
    lists: [
      {
        key: "danhMucHop",
        label: "Danh mục hộp bàn giao",
        legal: "Điều 9.1: kiểm đếm hộp, hồ sơ, tờ",
        columns: [
          { key: "hopSo", label: "Hộp số" },
          { key: "tuHoSo", label: "Từ hồ sơ", optional: true },
          { key: "denHoSo", label: "Đến hồ sơ", optional: true },
          { key: "soHoSo", label: "Số hồ sơ" },
          { key: "soTo", label: "Số tờ", optional: true },
          { key: "tinhTrang", label: "Tình trạng", optional: true },
          { key: "ghiChu", label: "Ghi chú", optional: true },
        ],
        minRows: 0,
      },
    ],
    generic: { refKey: "maPhong", signLeft: "ĐẠI DIỆN BÊN GIAO", signRight: "ĐẠI DIỆN BÊN NHẬN", tieuDe: "Biên bản giao nhận tài liệu giấy để số hoá" },
  },
  {
    kind: "tra_tai_lieu_giay",
    label: "Biên bản trả tài liệu gốc",
    description: "Nhà thầu trả tài liệu giấy về kho sau khi số hoá xong: kiểm đếm lại hộp, hồ sơ, tờ.",
    legal: "Điểm đ khoản 4 Điều 9",
    fields: [
      t("benTra", "Bên trả (đơn vị số hoá)"),
      t("benNhan", "Bên nhận lại (cơ quan, tổ chức)"),
      t("phong", "Phông tài liệu"),
      t("maPhong", "Mã phông", true, { hint: "Dùng để lọc sổ biên bản theo phông" }),
      t("diaDiem", "Địa điểm"),
      t("ngay", "Ngày trả", false, { hint: NGAY }),
      n("tongHop", "Tổng số hộp"),
      n("tongHoSo", "Tổng số hồ sơ"),
      t("tongTo", "Tổng số tờ", true),
      t("tinhTrang", "Tình trạng sau số hoá", false, { hint: "Đối chiếu với biên bản giao nhận" }),
      t("soBienBanGiaoNhan", "Số biên bản giao nhận đối ứng", true),
      t("nguoiTra", "Người trả"),
      t("nguoiNhan", "Người nhận"),
      t("ghiChu", "Ghi chú", true, { multiline: true }),
    ],
    lists: [
      {
        key: "danhMucHop",
        label: "Danh mục hộp trả lại",
        columns: [
          { key: "hopSo", label: "Hộp số" },
          { key: "soHoSo", label: "Số hồ sơ" },
          { key: "soTo", label: "Số tờ", optional: true },
          { key: "tinhTrang", label: "Tình trạng", optional: true },
          { key: "ghiChu", label: "Ghi chú", optional: true },
        ],
        minRows: 0,
      },
    ],
    generic: { refKey: "maPhong", signLeft: "ĐẠI DIỆN BÊN TRẢ", signRight: "ĐẠI DIỆN BÊN NHẬN LẠI", tieuDe: "Biên bản trả tài liệu gốc sau số hoá" },
  },
  {
    kind: "nhat_ky_so_hoa",
    label: "Nhật ký số hoá",
    description: "Chốt một ngày thi công: sản lượng từng người (quét, đưa lên, OCR, PDF/A, ký, duyệt) và sự việc trong ngày.",
    legal: "Khoản 4 Điều 9",
    fields: [
      t("donVi", "Đơn vị số hoá"),
      t("duAn", "Dự án"),
      t("ngay", "Ngày", false, { hint: "Ghi dạng YYYY-MM-DD để lọc theo ngày" }),
      t("nguoiLap", "Người lập"),
      t("phuTrach", "Phụ trách thi công", true),
      t("suViec", "Sự việc, ghi chú trong ngày", true, { multiline: true }),
    ],
    lists: [
      {
        key: "sanLuong",
        label: "Sản lượng theo người",
        columns: [
          { key: "nguoi", label: "Người" },
          { key: "trangQuet", label: "Trang quét" },
          { key: "tepDuaLen", label: "Tệp đưa lên", optional: true },
          { key: "trangOcr", label: "Trang OCR", optional: true },
          { key: "tepPdfa", label: "PDF/A", optional: true },
          { key: "tepKy", label: "Ký số", optional: true },
          { key: "tepDuyet", label: "Duyệt", optional: true },
          { key: "ghiChu", label: "Ghi chú", optional: true },
        ],
        minRows: 0,
      },
    ],
    generic: { refKey: "ngay", signLeft: "NGƯỜI LẬP NHẬT KÝ", signRight: "PHỤ TRÁCH THI CÔNG", tieuDe: "Nhật ký số hoá tài liệu" },
  },
];

/** Khối PDF của biên bản generic (khớp `GenericBlock` ở crates/digitext-api/src/bienban.rs). */
export type KhoiGeneric =
  | { t: "heading"; text: string }
  | { t: "paragraph"; text: string }
  | { t: "field"; label: string; value: string }
  | { t: "bullets"; items: string[] }
  | { t: "table"; columns: string[]; rows: string[][] }
  | { t: "signatures"; left: string[]; right: string[] };

/** Dựng payload cho mẫu generic từ ô + bảng đã kiểm: giữ cả `fields`/`lists` thô để xem lại, nhân bản, cộng dồn. */
export function taoPayloadGeneric(template: V4te, fields: Record<string, string>, lists: Record<string, Record<string, string>[]>): { payload: Record<string, unknown>; ref: string | null } {
  const g = template.generic;
  if (!g) throw new Error(`mẫu ${template.kind} không phải generic`);
  const gia = (k: string) => (fields[k] ?? "").trim();
  const blocks: KhoiGeneric[] = [];
  const diaDiem = gia("diaDiem");
  const ngay = gia("ngay");
  if (ngay || diaDiem) blocks.push({ t: "paragraph", text: `Hôm nay, ngày ${ngay || "…"}${diaDiem ? `, tại ${diaDiem}` : ""}, các bên tiến hành lập biên bản với nội dung sau:` });
  blocks.push({ t: "heading", text: "I. NỘI DUNG" });
  for (const f of template.fields) {
    if (f.key === "diaDiem" || f.key === "ngay") continue;
    const v = gia(f.key);
    if (v) blocks.push({ t: "field", label: f.label, value: v });
  }
  for (const l of template.lists) {
    const rows = (lists[l.key] ?? []).filter((r) => Object.values(r).some((v) => (v ?? "").trim()));
    if (rows.length === 0) continue;
    blocks.push({ t: "heading", text: `II. ${l.label.toUpperCase()}` });
    if (l.simple) blocks.push({ t: "bullets", items: rows.map((r) => (r.value ?? "").trim()) });
    else blocks.push({ t: "table", columns: l.columns.map((c) => c.label), rows: rows.map((r) => l.columns.map((c) => (r[c.key] ?? "").trim())) });
  }
  blocks.push({ t: "signatures", left: [g.signLeft, "(Ký, ghi rõ họ tên)"], right: [g.signRight, "(Ký, ghi rõ họ tên, đóng dấu)"] });
  const ref = g.refKey ? gia(g.refKey) || null : null;
  return { payload: { title: g.tieuDe, blocks, fields, lists }, ref };
}

export function V4tg(kind: V4ta): V4te {
  const t = V4tf.find((x) => x.kind === kind);
  if (!t) throw new Error(`không có mẫu: ${kind}`);
  return t;
}

export interface V4th {
  payload: Record<string, unknown> | null;
  errors: string[];
  /** Lỗi theo ô (key ô → thông báo) để hiện ngay dưới ô. */
  fieldErrors: Record<string, string>;
}

export function V4ti(template: V4te, fields: Record<string, string>, lists: Record<string, Record<string, string>[]>): V4th {
  const errors: string[] = [];
  const fieldErrors: Record<string, string> = {};
  const payload: Record<string, unknown> = {};

  for (const f of template.fields) {
    const v = (fields[f.key] ?? "").trim();
    if (!v) {
      if (!f.optional) {
        errors.push(`Thiếu "${f.label}".`);
        fieldErrors[f.key] = "Chưa điền";
      }
      continue;
    }
    if (f.kind === "number") {
      const num = Number(v);
      if (!Number.isFinite(num) || num < 0) {
        errors.push(`"${f.label}" phải là số không âm.`);
        fieldErrors[f.key] = "Phải là số không âm";
        continue;
      }
      payload[f.key] = num;
    } else {
      payload[f.key] = v;
    }
  }

  for (const l of template.lists) {
    const rows = (lists[l.key] ?? []).filter((r) => Object.values(r).some((v) => (v ?? "").trim()));
    if (rows.length < l.minRows) {
      errors.push(`"${l.label}" cần tối thiểu ${l.minRows} dòng.`);
      continue;
    }
    if (l.simple) {
      payload[l.key] = rows.map((r) => (r.value ?? "").trim()).filter(Boolean);
      if ((payload[l.key] as string[]).length < l.minRows) errors.push(`"${l.label}" cần tối thiểu ${l.minRows} dòng.`);
      continue;
    }
    const out: Record<string, unknown>[] = [];
    for (const [i, row] of rows.entries()) {
      const o: Record<string, unknown> = {};
      for (const c of l.columns) {
        const v = (row[c.key] ?? "").trim();
        if (!v) {
          if (!c.optional) errors.push(`"${l.label}" dòng ${i + 1}: thiếu "${c.label}".`);
          continue;
        }
        o[c.key] = v;
      }
      out.push(o);
    }
    payload[l.key] = out;
  }

  return errors.length > 0 ? { payload: null, errors, fieldErrors } : { payload, errors: [], fieldErrors: {} };
}
