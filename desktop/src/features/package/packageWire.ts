// Copyright (c) 2026 DigiText. All rights reserved.

export type V4sj = "aip_hoso" | "aip_tailieu" | "sip_hoso" | "sip_tailieu" | "dip";

export const V4sk: Record<V4sj, string> = {
  aip_hoso: "QUlQIGjhu5Mgc8ahIChi4bqjbyBxdeG6o24p",
  aip_tailieu: "QUlQIHTDoGkgbGnhu4d1IChi4bqjbyBxdeG6o24p",
  sip_hoso: "U0lQIGjhu5Mgc8ahIChu4buZcCBsxrB1KQ==",
  sip_tailieu: "U0lQIHTDoGkgbGnhu4d1IChu4buZcCBsxrB1KQ==",
  dip: "RElQIChraGFpIHRow6FjIGPDsyB4w6FjIHRo4buxYyk=",
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
        "U1RUIE3hu6VjIGzhu6VjIChQSy01KToga2hhaSDEkeG7pyBj4bq3cCB04burL8SR4bq/biwgc+G7kSBuZ3V5w6puLg==",
    };
  if (from < 1 || from > to)
    return {
      range: null,
      error: "U1RUIE3hu6VjIGzhu6VjIChQSy01KTogY+G6p24gMSDiiaQgdOG7qyDiiaQgxJHhur9uLg==",
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

  if (!V2oi.fondCode.trim()) V2oj.push("VGhp4bq/dSBtw6MgcGjDtG5nIChGb25kY29kZSku");
  if (!V2oi.archiveOrgCode.trim()) V2oj.push("VGhp4bq/dSBtw6MgY8ahIHF1YW4gbMawdSB0cuG7ry4=");
  if (docs.length === 0) V2oj.push("Q2jGsGEgY2jhu41uIHTDoGkgbGnhu4d1IG7DoG8u");
  if (V2om && docs.length > 1)
    V2oj.push(
      "R8OzaSBt4bupYyB0w6BpIGxp4buHdSBjaOG7iSBjaOG7qWEgxJHDum5nIDAxIHTDoGkgbGnhu4d1IGNow61uaC4=",
    );
  if (V2on && !V2oi.submissionAgreement.trim())
    V2oj.push(
      "R8OzaSBTSVAgYuG6r3QgYnXhu5ljIG3DoyB5w6p1IGPhuqd1IG7hu5lwICh0aG/huqMgdGh14bqtbiBu4buZcCku",
    );
  if (V2on && V2oi.maintenance !== "01")
    V2oj.push(
      "Tmd14buTbiBu4buZcCBsxrB1IGNo4buJIG5o4bqtbiB0aOG7nWkgaOG6oW4gJzAxOiB2xKluaCB2aeG7hW4nLg==",
    );

  const V2oo = V2oi.confidenceLevel === "02" || V2oi.confidenceLevel === "03";
  if (V2ok === "dip") {
    if (!V2oi.requestId.trim())
      V2oj.push("VGhp4bq/dSBtw6MgecOqdSBj4bqndSBraGFpIHRow6FjIChESVApLg==");
    if (!V2oi.requestDate.trim())
      V2oj.push("VGhp4bq/dSBuZ8OgeSB5w6p1IGPhuqd1IGtoYWkgdGjDoWMgKERJUCku");

    if (!V2oi.certMaXacThuc.trim())
      V2oj.push("VGhp4bq/dSBtw6MgeMOhYyB0aOG7sWMgbMawdSB0cuG7ryAoxJBp4buBdSA0Mi4yLmEpLg==");
    if (!V2oi.certTenLuuTru.trim())
      V2oj.push("VGhp4bq/dSB0w6puIGzGsHUgdHLhu68gbOG7i2NoIHPhu60gKMSQaeG7gXUgNDIuMi5iKS4=");
    if (!V2oi.certNguoiNhanHoTen.trim() || !V2oi.certNguoiNhanDinhDanh.trim())
      V2oj.push(
        "VGhp4bq/dSBo4buNIHTDqm4gLyBtw6MgxJHhu4tuaCBkYW5oIG5nxrDhu51pIG5o4bqtbiAoxJBp4buBdSA0Mi4yLmMpLg==",
      );
    if (!V2oi.certMaLuuTruGoc.trim())
      V2oj.push("VGhp4bq/dSBtw6MgbMawdSB0cuG7ryB0w6BpIGxp4buHdSBn4buRYyAoxJBp4buBdSA0Mi4yLmQpLg==");
    if (!V2oi.certTenLoai.trim())
      V2oj.push("VGhp4bq/dSB0w6puIGxv4bqhaSB0w6BpIGxp4buHdSAoxJBp4buBdSA0Mi4yLmUpLg==");
    if (!V2oi.certTrichYeu.trim())
      V2oj.push("VGhp4bq/dSB0csOtY2ggeeG6v3UgLyB0acOqdSDEkeG7gSAoxJBp4buBdSA0Mi4yLmcpLg==");
    if (!V2oi.certMucDich.trim())
      V2oj.push("VGhp4bq/dSBt4bulYyDEkcOtY2ggc+G7rSBk4bulbmcgKMSQaeG7gXUgNDIuMi5oKS4=");
    const V2op = V4sq(V2oi.certSoLuongBan);
    if (V2op === null || V2op < 1)
      V2oj.push("U+G7kSBsxrDhu6NuZyBi4bqjbiB04buRaSB0aGnhu4N1IDEgKMSQaeG7gXUgNDIuMi5sKS4=");
  } else if (!V2oi.fileCode.trim()) {
    V2oj.push("VGhp4bq/dSBtw6MgaOG7kyBzxqEgLyBtw6MgZ8OzaSB0aW4u");
  }
  if (V2ok !== "dip" && !V2oi.title.trim()) V2oj.push("VGhp4bq/dSB0acOqdSDEkeG7gS4=");
  if (V2ol && V2oo) {
    if (V4sq(V2oi.numberOfPaper) === null)
      V2oj.push("SOG7kyBzxqEgc+G7kSBow7NhIGLhuq90IGJ14buZYyBz4buRIGzGsOG7o25nIHThu50u");
    if (!V2oi.paperFileCode.trim())
      V2oj.push("SOG7kyBzxqEgc+G7kSBow7NhIGLhuq90IGJ14buZYyBtw6MgaOG7kyBzxqEgZ+G7kWMgZ2nhuqV5Lg==");
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
