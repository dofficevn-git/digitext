// Copyright (c) 2026 DigiText. All rights reserved.

export type V4sj = "aip_hoso" | "aip_tailieu" | "sip_hoso" | "sip_tailieu" | "dip";

export const V4sk: Record<V4sj, string> = {
  aip_hoso: "AIP hồ sơ (bảo quản)",
  aip_tailieu: "AIP tài liệu (bảo quản)",
  sip_hoso: "SIP hồ sơ (nộp lưu)",
  sip_tailieu: "SIP tài liệu (nộp lưu)",
  dip: "DIP (khai thác có xác thực)",
};

export interface V4sl {
  documentId: string;
  filename: string;
}

export interface V4sm {
  kind: V4sj;
  fondCode: string;
  archiveOrgCode: string;

  fileCode: string;
  title: string;
  maintenance: string;
  mode: string;
  language: string;
  confidenceLevel: string;
  numberOfPaper: string;
  numberOfPage: string;
  paperFileCode: string;
  startDate: string;
  endDate: string;

  source: string;

  submissionAgreement: string;

  zipOrdinal: string;

  requestId: string;
  requestDate: string;
  purpose: string;
  includePremis: boolean;

  certMaXacThuc: string;
  certTenLuuTru: string;
  certNguoiNhanHoTen: string;
  certNguoiNhanDinhDanh: string;
  certMaLuuTruGoc: string;
  certSoKyHieuGoc: string;
  certTenLoai: string;
  certTrichYeu: string;
  certMucDich: string;
  certThoiHan: string;
  certSoLuongBan: string;
}

export const V4sn: V4sm = {
  kind: "aip_hoso",
  fondCode: "",
  archiveOrgCode: "",
  fileCode: "",
  title: "",
  maintenance: "01",
  mode: "01",
  language: "01",
  confidenceLevel: "02",
  numberOfPaper: "",
  numberOfPage: "",
  paperFileCode: "",
  startDate: "",
  endDate: "",
  source: "0",
  submissionAgreement: "",
  zipOrdinal: "1",
  requestId: "",
  requestDate: "",
  purpose: "01",
  includePremis: false,
  certMaXacThuc: "",
  certTenLuuTru: "",
  certNguoiNhanHoTen: "",
  certNguoiNhanDinhDanh: "",
  certMaLuuTruGoc: "",
  certSoKyHieuGoc: "",
  certTenLoai: "",
  certTrichYeu: "",
  certMucDich: "",
  certThoiHan: "",
  certSoLuongBan: "1",
};

export function V4so(V2ob: number): string {
  return String(V2ob + 1).padStart(7, "0");
}

export function V4sp(V2oc: string, V2od: number, V2oe: string): string {
  const dot = V2oe.lastIndexOf(".");
  const V2of = dot >= 0 ? V2oe.slice(dot) : ".pdf";
  return `${V2oc}.${V4so(V2od)}${V2of}`;
}

const V4sq = (s: string): number | null => {
  const t = s.trim();
  if (!t) return null;
  const n = Number(t);
  return Number.isFinite(n) && n >= 0 ? n : null;
};

export interface V4sr {
  range: [number, number] | null;
  error: string | null;
}

export function V4ss(V2og: string, V2oh: string): V4sr {
  const f = V2og.trim();
  const t = V2oh.trim();
  if (!f && !t) return { range: null, error: null };
  const from = Number(f);
  const to = Number(t);
  if (!f || !t || !Number.isInteger(from) || !Number.isInteger(to))
    return {
      range: null,
      error:
        "STT Mục lục (PK-5): khai đủ cặp từ/đến, số nguyên.",
    };
  if (from < 1 || from > to)
    return {
      range: null,
      error: "STT Mục lục (PK-5): cần 1 ≤ từ ≤ đến.",
    };
  return { range: [from, to], error: null };
}

export interface V4st {
  request: Record<string, unknown> | null;
  errors: string[];
}

export function V4su(V2oi: V4sm, docs: V4sl[]): V4st {
  const V2oj: string[] = [];
  const V2ok = V2oi.kind;
  const V2ol = V2ok === "aip_hoso" || V2ok === "sip_hoso";
  const V2om = V2ok === "aip_tailieu" || V2ok === "sip_tailieu";
  const V2on = V2ok === "sip_hoso" || V2ok === "sip_tailieu";

  if (!V2oi.fondCode.trim()) V2oj.push("Thiếu mã phông (Fondcode).");
  if (!V2oi.archiveOrgCode.trim()) V2oj.push("Thiếu mã cơ quan lưu trữ.");
  if (docs.length === 0) V2oj.push("Chưa chọn tài liệu nào.");
  if (V2om && docs.length > 1)
    V2oj.push(
      "Gói mức tài liệu chỉ chứa đúng 01 tài liệu chính.",
    );
  if (V2on && !V2oi.submissionAgreement.trim())
    V2oj.push(
      "Gói SIP bắt buộc mã yêu cầu nộp (thoả thuận nộp).",
    );
  if (V2on && V2oi.maintenance !== "01")
    V2oj.push(
      "Nguồn nộp lưu chỉ nhận thời hạn '01: vĩnh viễn'.",
    );

  const V2oo = V2oi.confidenceLevel === "02" || V2oi.confidenceLevel === "03";
  if (V2ok === "dip") {
    if (!V2oi.requestId.trim())
      V2oj.push("Thiếu mã yêu cầu khai thác (DIP).");
    if (!V2oi.requestDate.trim())
      V2oj.push("Thiếu ngày yêu cầu khai thác (DIP).");

    if (!V2oi.certMaXacThuc.trim())
      V2oj.push("Thiếu mã xác thực lưu trữ (Điều 42.2.a).");
    if (!V2oi.certTenLuuTru.trim())
      V2oj.push("Thiếu tên lưu trữ lịch sử (Điều 42.2.b).");
    if (!V2oi.certNguoiNhanHoTen.trim() || !V2oi.certNguoiNhanDinhDanh.trim())
      V2oj.push(
        "Thiếu họ tên / mã định danh người nhận (Điều 42.2.c).",
      );
    if (!V2oi.certMaLuuTruGoc.trim())
      V2oj.push("Thiếu mã lưu trữ tài liệu gốc (Điều 42.2.d).");
    if (!V2oi.certTenLoai.trim())
      V2oj.push("Thiếu tên loại tài liệu (Điều 42.2.e).");
    if (!V2oi.certTrichYeu.trim())
      V2oj.push("Thiếu trích yếu / tiêu đề (Điều 42.2.g).");
    if (!V2oi.certMucDich.trim())
      V2oj.push("Thiếu mục đích sử dụng (Điều 42.2.h).");
    const V2op = V4sq(V2oi.certSoLuongBan);
    if (V2op === null || V2op < 1)
      V2oj.push("Số lượng bản tối thiểu 1 (Điều 42.2.l).");
  } else if (!V2oi.fileCode.trim()) {
    V2oj.push("Thiếu mã hồ sơ / mã gói tin.");
  }
  if (V2ok !== "dip" && !V2oi.title.trim()) V2oj.push("Thiếu tiêu đề.");
  if (V2ol && V2oo) {
    if (V4sq(V2oi.numberOfPaper) === null)
      V2oj.push("Hồ sơ số hóa bắt buộc số lượng tờ.");
    if (!V2oi.paperFileCode.trim())
      V2oj.push("Hồ sơ số hóa bắt buộc mã hồ sơ gốc giấy.");
  }
  if (V2oj.length > 0) return { request: null, errors: V2oj };

  const V2oq = V2oi.fileCode.trim();
  let V2or: Record<string, unknown>;
  if (V2ol) {
    V2or = {
      type: "hoso",
      fileCode: V2oq,
      title: V2oi.title.trim(),
      maintenance: V2oi.maintenance,
      mode: V2oi.mode,
      language: [V2oi.language],
      startDate: V2oi.startDate.trim() || null,
      endDate: V2oi.endDate.trim() || null,
      totalDoc: docs.length,
      numberOfPaper: V4sq(V2oi.numberOfPaper),
      numberOfPage: V4sq(V2oi.numberOfPage),
      confidenceLevel: V2oi.confidenceLevel,
      paperFileCode: V2oi.paperFileCode.trim() || null,
      riskRecovery: false,
    };
  } else if (V2om) {
    V2or = {
      type: "goi_tin",
      fileCode: V2oq,
      title: V2oi.title.trim(),
      source: V2oi.source,
      totalDoc: V2ok === "sip_tailieu" ? docs.length : null,
      description: null,
    };
  } else {
    V2or = {
      type: "dip",
      requestId: V2oi.requestId.trim(),
      requestDate: V2oi.requestDate.trim(),
      purpose: V2oi.purpose,
      purposeContent: null,
      feeObjectType: null,
      researchTopic: null,
      description: null,
    };
  }

  const V2os = docs.map((d, i) => ({
    documentId: d.documentId,
    fileName: V4sp(V2oq || V2oi.requestId.trim() || "GOI", i, d.filename),
    metadata: {
      type: "doc",
      docCode: `${V2oq || V2oi.requestId.trim()}.${V4so(i)}`,
      maintenance: V2oi.maintenance,

      typeName: "32",
      mode: V2oi.mode,
      language: [V2oi.language],
      confidenceLevel: V2oi.confidenceLevel,
      subject: d.filename,
      riskRecovery: false,
    },
  }));

  const V2ot: Record<string, unknown> = {
    kind: V2ok,
    fondCode: V2oi.fondCode.trim(),
    archiveOrgCode: V2oi.archiveOrgCode.trim(),
    meta: V2or,
    documents: V2os,
    includePremis: V2oi.kind === "dip" ? V2oi.includePremis : false,
  };
  if (V2ok === "dip") {
    V2ot.dipCert = {
      maXacThuc: V2oi.certMaXacThuc.trim(),
      tenLuuTruLichSu: V2oi.certTenLuuTru.trim(),
      nguoiNhanHoTen: V2oi.certNguoiNhanHoTen.trim(),
      nguoiNhanDinhDanh: V2oi.certNguoiNhanDinhDanh.trim(),
      maLuuTruGoc: V2oi.certMaLuuTruGoc.trim(),
      soKyHieuGoc: V2oi.certSoKyHieuGoc.trim() || null,
      tenLoai: V2oi.certTenLoai.trim(),
      trichYeu: V2oi.certTrichYeu.trim(),
      mucDich: V2oi.certMucDich.trim(),
      ngayCap: V2oi.requestDate.trim(),
      thoiHanSuDung: V2oi.certThoiHan.trim() || null,
      soLuongBan: V4sq(V2oi.certSoLuongBan) ?? 1,
    };
  }
  if (V2on) {
    V2ot.submissionAgreement = V2oi.submissionAgreement.trim();
    const V2ou = V4sq(V2oi.zipOrdinal);
    if (V2ou !== null && V2ou > 0) V2ot.zipOrdinal = V2ou;
  }
  return { request: V2ot, errors: [] };
}
