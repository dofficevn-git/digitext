// Copyright (c) 2026 DigiText. All rights reserved.

import type { V4ej, ReviewSummaryRow } from "@/api/docs";

export const V4ve = "(Chưa rõ phông)";

export const V4vf = "(Chưa phân loại)";

export const V4vg = "(Chưa rõ dự án)";

export const V4vh = 0;
export const V4vi = 1;
export const V4vj = 2;
export const V4vk = 3;

export interface V4vl {
  name: string;

  path: string;

  depth: number;
  folders: V4vl[];
  docs: V4ej[];

  docCount: number;

  pageCount: number;

  sizeBytes: number;

  boxCount: number | null;

  dossierCount: number | null;

  /** Số tệp ĐÃ KÝ SỐ trong nhánh. */
  signedCount: number;
  /** Số tệp mới có PDF/A (chưa ký). */
  pdfaOnlyCount: number;
}

/** Đã ký số: đã qua bước ký (signed) hoặc đã lưu kho (archived). */
export function isSignedDoc(d: V4ej): boolean {
  return d.status === "signed" || d.status === "archived";
}

/** Mới chỉ có PDF/A (lớp chữ), chưa ký số. */
export function isPdfaOnlyDoc(d: V4ej): boolean {
  return !isSignedDoc(d) && (d.pdfaPages ?? 0) > 0;
}

export function V4vm(d: V4ej): string {
  const V328 = (d.projectCode ?? "").trim();
  const name = (d.projectName ?? "").trim();
  if (V328 && name) return `${V328} · ${name}`;
  return V328 || name || V4vg;
}

function V4vn(s: string): string {
  return s.replace(/[/\\]/g, "-").trim();
}

export function V4vo(d: V4ej): string {
  if (d.relPath && d.relPath.trim() !== "") return d.relPath;
  if (d.hoSoRef && d.hoSoRef.trim() !== "") return `${V4ve}/${d.hoSoRef}`;
  return V4vf;
}

export function V4vp(d: V4ej): string {
  return `${V4vn(V4vm(d))}/${V4vo(d)}`;
}

/** Nhóm hồ sơ (dự án/phông/hộp/hồ sơ) ứng với một thư mục lá — để nạp tệp khi mở. */
export interface FolderGroup {
  projectId: string;
  phong: string;
  hop: string;
  hoSo: string;
}
export function groupKeyOf(g: FolderGroup): string {
  return JSON.stringify([g.projectId, g.phong, g.hop, g.hoSo]);
}
/** Đường dẫn thư mục của một hồ sơ trong cây Kho máy chủ — cùng quy ước với V4vp/V4vo. */
function folderSegments(r: ReviewSummaryRow): string[] {
  const code = r.projectCode.trim();
  const name = r.projectName.trim();
  const duAn = (code && name ? `${code} · ${name}` : code || name || V4vg).replace(/[/\\]/g, "-").trim();
  if (r.phong) return [duAn, r.phong, r.hop, r.hoSo].filter((x) => x.trim() !== "");
  if (r.hop || r.hoSo) return [duAn, V4ve, r.hop, r.hoSo].filter((x) => x.trim() !== "");
  return [duAn, V4vf];
}
/** Cây Kho máy chủ dựng từ số liệu gộp sẵn của máy chủ (một dòng mỗi hồ sơ, không có tệp).
 *  Tệp của thư mục lá nạp khi người dùng mở (`docsByPath`). Trả thêm bảng khoá nhóm → đường dẫn. */
export function treeFromSummary(rows: ReviewSummaryRow[]): { root: V4vl; groupOf: Map<string, FolderGroup>; pathOfKey: Map<string, string> } {
  const root: V4vl = { name: "", path: "", depth: -1, folders: [], docs: [], docCount: 0, pageCount: 0, sizeBytes: 0, boxCount: null, dossierCount: null, signedCount: 0, pdfaOnlyCount: 0 };
  const byPath = new Map<string, V4vl>([["", root]]);
  const groupOf = new Map<string, FolderGroup>();
  const pathOfKey = new Map<string, string>();
  const ensure = (segs: string[]): V4vl => {
    let cur = root;
    let path = "";
    for (const seg of segs) {
      path = path === "" ? seg : `${path}/${seg}`;
      let f = byPath.get(path);
      if (!f) {
        f = { name: seg, path, depth: cur.depth + 1, folders: [], docs: [], docCount: 0, pageCount: 0, sizeBytes: 0, boxCount: null, dossierCount: null, signedCount: 0, pdfaOnlyCount: 0 };
        byPath.set(path, f);
        cur.folders.push(f);
      }
      cur = f;
    }
    return cur;
  };
  for (const r of rows) {
    const leaf = ensure(folderSegments(r));
    leaf.docCount += r.files;
    leaf.pageCount += r.pages;
    leaf.sizeBytes += r.sizeBytes;
    leaf.signedCount += r.signedDocs;
    leaf.pdfaOnlyCount += r.pdfaOnlyDocs;
    const g: FolderGroup = { projectId: r.projectId, phong: r.phong, hop: r.hop, hoSo: r.hoSo };
    groupOf.set(leaf.path, g);
    pathOfKey.set(groupKeyOf(g), leaf.path);
  }
  const col = new Intl.Collator("vi", { numeric: true });
  const fin = (f: V4vl) => {
    f.folders.sort((a, b) => col.compare(a.name, b.name));
    for (const c of f.folders) fin(c);
    if (f.folders.length > 0) {
      f.docCount += f.folders.reduce((n, c) => n + c.docCount, 0);
      f.pageCount += f.folders.reduce((n, c) => n + c.pageCount, 0);
      f.sizeBytes += f.folders.reduce((n, c) => n + c.sizeBytes, 0);
      f.signedCount += f.folders.reduce((n, c) => n + c.signedCount, 0);
      f.pdfaOnlyCount += f.folders.reduce((n, c) => n + c.pdfaOnlyCount, 0);
    }
    f.boxCount = f.depth === V4vi ? f.folders.length : f.depth < V4vi ? f.folders.reduce((n, c) => n + (c.boxCount ?? 0), 0) : null;
    f.dossierCount = f.depth === V4vj ? f.folders.length : f.depth < V4vj ? f.folders.reduce((n, c) => n + (c.dossierCount ?? 0), 0) : null;
  };
  fin(root);
  return { root, groupOf, pathOfKey };
}
/** Các thư mục lá (có nhóm hồ sơ) dưới một thư mục. */
export function leavesUnder(f: V4vl, groupOf: ReadonlyMap<string, FolderGroup>): V4vl[] {
  const out: V4vl[] = [];
  const walk = (n: V4vl) => {
    if (groupOf.has(n.path)) out.push(n);
    for (const c of n.folders) walk(c);
  };
  walk(f);
  return out;
}
/** Tệp đã nạp dưới một thư mục (chỉ những lá đã nạp). */
export function filesUnder(f: V4vl, docsByPath: ReadonlyMap<string, V4ej[]>): V4vu[] {
  const out: V4vu[] = [];
  const walk = (n: V4vl) => {
    for (const c of n.folders) walk(c);
    for (const d of docsByPath.get(n.path) ?? []) out.push({ doc: d, folderPath: n.path });
  };
  walk(f);
  return out;
}
/** Thư mục có đường dẫn kết thúc bằng tầng cuối của relPath (dùng cho "đi tới tệp"). */
export function findFolderByRelPath(root: V4vl, relPath: string): string | null {
  const segs = V4vz(relPath);
  const last = segs[segs.length - 1];
  let hit: string | null = null;
  const walk = (f: V4vl) => {
    if (hit !== null) return;
    if (f.path === last || f.path.endsWith(`/${last}`)) {
      hit = f.path;
      return;
    }
    for (const c of f.folders) walk(c);
  };
  walk(root);
  return hit;
}

export function V4vq(docs: V4ej[]): V4vl {
  const root: V4vl = {
    name: "",
    path: "",
    depth: -1,
    folders: [],
    docs: [],
    docCount: 0,
    pageCount: 0,
    sizeBytes: 0,
    boxCount: null,
    dossierCount: null,
    signedCount: 0,
    pdfaOnlyCount: 0,
  };
  const V329 = new Map<string, V4vl>([["", root]]);

  const V32a = (V32b: string[]): V4vl => {
    let V32c = root;
    let V32d = "";
    for (const V32e of V32b) {
      V32d = V32d === "" ? V32e : `${V32d}/${V32e}`;
      let V32f = V329.get(V32d);
      if (!V32f) {
        V32f = {
          name: V32e,
          path: V32d,
          depth: V32c.depth + 1,
          folders: [],
          docs: [],
          docCount: 0,
          pageCount: 0,
          sizeBytes: 0,
          boxCount: null,
          dossierCount: null,
          signedCount: 0,
          pdfaOnlyCount: 0,
        };
        V329.set(V32d, V32f);
        V32c.folders.push(V32f);
      }
      V32c = V32f;
    }
    return V32c;
  };

  for (const d of docs) {
    const V32g = V4vp(d)
      .split("/")
      .map((s) => s.trim())
      .filter((s) => s !== "");
    V32a(V32g).docs.push(d);
  }

  const V32h = new Intl.Collator("vi", { numeric: true });
  const V32i = (f: V4vl) => {
    f.folders.sort((a, b) => V32h.compare(a.name, b.name));
    f.docs.sort((a, b) => V32h.compare(a.filename, b.filename));
    for (const c of f.folders) V32i(c);

    f.docCount = f.docs.length + f.folders.reduce((n, c) => n + c.docCount, 0);
    f.pageCount =
      f.docs.reduce((n, d) => n + (d.pageCount ?? 0), 0) +
      f.folders.reduce((n, c) => n + c.pageCount, 0);
    f.sizeBytes =
      f.docs.reduce((n, d) => n + d.sizeBytes, 0) + f.folders.reduce((n, c) => n + c.sizeBytes, 0);
    f.signedCount =
      f.docs.filter(isSignedDoc).length + f.folders.reduce((n, c) => n + c.signedCount, 0);
    f.pdfaOnlyCount =
      f.docs.filter(isPdfaOnlyDoc).length + f.folders.reduce((n, c) => n + c.pdfaOnlyCount, 0);

    f.boxCount =
      f.depth === V4vi
        ? f.folders.length
        : f.depth < V4vi
          ? f.folders.reduce((n, c) => n + (c.boxCount ?? 0), 0)
          : null;
    f.dossierCount =
      f.depth === V4vj
        ? f.folders.length
        : f.depth < V4vj
          ? f.folders.reduce((n, c) => n + (c.dossierCount ?? 0), 0)
          : null;
  };
  V32i(root);
  return root;
}

export function V4vr(root: V4vl): string[] {
  const V32j: string[] = [];
  const V32k = (f: V4vl) => {
    for (const c of f.folders) {
      V32j.push(c.path);
      V32k(c);
    }
  };
  V32k(root);
  return V32j;
}

export const V4vs = 30;

export function V4vt(root: V4vl): Set<string> {
  const V32l = V4vr(root);
  if (V32l.length <= V4vs) return new Set(V32l);
  return new Set(root.folders.map((f) => f.path));
}

export interface V4vu {
  doc: V4ej;
  folderPath: string;
}

export function V4vv(f: V4vl): V4vu[] {
  const V32m: V4vu[] = [];
  const V32n = (n: V4vl) => {
    for (const c of n.folders) V32n(c);
    for (const d of n.docs) V32m.push({ doc: d, folderPath: n.path });
  };
  V32n(f);
  return V32m;
}

export function V4vw(root: V4vl): Map<string, string[]> {
  const m = new Map<string, string[]>();
  const V32o = (f: V4vl): string[] => {
    const V32p: string[] = [];
    for (const c of f.folders) for (const V32q of V32o(c)) V32p.push(V32q);
    for (const d of f.docs) V32p.push(d.id);
    m.set(f.path, V32p);
    return V32p;
  };
  V32o(root);
  return m;
}

export type V4vx = "none" | "some" | "all";

export function V4vy(V32r: readonly string[], V32s: ReadonlySet<string>): V4vx {
  if (V32r.length === 0) return "none";
  let n = 0;
  for (const V32t of V32r) if (V32s.has(V32t)) n++;
  if (n === 0) return "none";
  return n === V32r.length ? "all" : "some";
}

export function V4vz(path: string | null | undefined): string[] {
  const V32u = (path && path.trim() !== "" ? path : V4vf)
    .split("/")
    .map((s) => s.trim())
    .filter((s) => s !== "");
  const V32v: string[] = [];
  let V32w = "";
  for (const V32x of V32u) {
    V32w = V32w === "" ? V32x : `${V32w}/${V32x}`;
    V32v.push(V32w);
  }
  return V32v;
}

export interface V4w0 {
  relPath: string;
  filename: string;
}

export function V4w1(root: V4vl, V32y: V4w0): string | null {
  const V32z = V4vz(V32y.relPath);
  const V330 = V32z[V32z.length - 1];
  let V331: string | null = null;
  const V332 = (f: V4vl) => {
    if (V331 !== null) return;
    if (
      (f.path === V330 || f.path.endsWith(`/${V330}`)) &&
      f.docs.some((d) => d.filename === V32y.filename)
    ) {
      V331 = f.path;
      return;
    }
    for (const c of f.folders) V332(c);
  };
  V332(root);
  return V331;
}

export function V4w2(
  V333: string,
  V334: string,
  V335: { folderPath: string; filename: string } | null,
): boolean {
  return V335 !== null && V335.filename === V334 && V335.folderPath === V333;
}
