// Copyright (c) 2026 DigiText. All rights reserved.
// Đóng gói › Từ hồ sơ (26/10/2026): phông / hộp / hồ sơ lấy từ TÀI LIỆU ĐÃ LÊN MÁY CHỦ của DỰ ÁN ĐANG CHỌN
// (tầng 1 của đường dẫn = phông, đúng quy ước "mỗi thư mục đẩy lên là một phông"). Trước đây tab này dùng danh mục
// hồ sơ lập trên máy quản trị (bảng cục bộ, không phân dự án) nên hiện lẫn phông của dự án khác và rỗng với dự án
// đẩy theo thư mục. Mỗi hồ sơ chọn → một gói trên máy chủ (gói mức tài liệu: mỗi tài liệu một gói).
// Nhập mục lục Excel (26/10/2026): tệp "Mục lục hồ sơ" (Hộp số · Hồ sơ số · Tiêu đề · Thời gian · Số tờ · THBQ, cùng
// mẫu màn Chỉnh lý) ghép với hồ sơ trên máy chủ theo số hộp + số hồ sơ trong phông đã chọn → gói mang tiêu đề, thời
// gian, số tờ, THBQ thật thay vì tên thư mục. Mục lục đã ghép được nhớ theo dự án + phông (localStorage).
// Mục lục hồ sơ do MÁY CHỦ đọc từ 10/11/2026 (`/cl/muc-luc/doc`) — cùng bộ luật với màn Nhập mục lục.
// Hai mẫu Excel: (1) Mục lục hồ sơ → ghép theo số hộp + số hồ sơ; (2) Metadata phẳng (Metadata-<PHÔNG>.xlsx, mỗi dòng
// một văn bản, "Mã lưu trữ tài liệu" = tên tệp) → ghép theo TÊN TỆP trên máy chủ, lấy luôn mã hồ sơ / mã phông.

import { useEffect, useMemo, useRef, useState } from "react";
import { docMucLucMayChu, type DongHoSo } from "@/features/chinhly/mucLucMayChu";
import { docLuoiExcel, docMetadataPhang, laMetadataPhang } from "./metadataPhang";
import { V41t } from "@/live/useLive";
import { isTauri } from "@/api/ipc";
import { docsForGroups, docsReviewSummary, type ReviewSummaryRow, type V4ej } from "@/api/docs";
import { reviewKey, reviewKeyOf } from "@/features/dossier/reviewScope";
import { V4rv } from "@/api/packaging";
import { writeXlsx, type Sheet } from "@/features/chinhly/excel";
import { useProject } from "@/state/project";
import { V4sk, V4sn, V4su, type V4sj, type V4sm } from "./packageWire";
import { Badge, Button, Card, Dialog, EmptyState, ErrorBanner, Field, FormGrid, IconButton, Input, notifyError, Select, Table, Tip, toast, type Column, errorText } from "@/ui";

const n = (v: number) => v.toLocaleString("vi-VN");
const so = (s: string | null | undefined): string => {
  const m = /(\d+)/.exec(s ?? "");
  return m ? m[1] : (s ?? "").trim();
};
/** Khoá ghép hộp + hồ sơ: theo SỐ ("HOP 01" = "01" = "1"), không có số thì theo chữ thường. */
const khoaSo = (s: string | null | undefined): string => {
  const m = /(\d+)/.exec(s ?? "");
  return m ? String(Number(m[1])) : (s ?? "").trim().toLowerCase();
};
const khoaHoSo = (hop: string | null | undefined, hoSo: string | null | undefined) => `${khoaSo(hop)}|${khoaSo(hoSo)}`;
/** Mã hồ sơ suy từ tên tệp theo mẫu <mã hồ sơ>.<tt văn bản>.<tt tệp>.pdf (H05.13.24.01.1998.31.01.0000033.pdf →
 *  H05.13.24.01.1998.31); chấp nhận dấu phẩy gõ nhầm thay dấu chấm. Dùng khi tên tệp trên máy chủ không có trong Excel
 *  (số thứ tự tệp bị đánh lại) nhưng Excel vẫn có hồ sơ đó. */
function maTuTenTep(t: string): string | null {
  const m = /^(.+?)[.,]\d{2}[.,]\d{6,8}\.pdf$/i.exec(t.replace(/,/g, "."));
  return m ? m[1].toUpperCase() : null;
}
/** Tên THBQ → mã thời hạn bảo quản TT02: 01 vĩnh viễn, 02 70 năm, 03 50, 04 30, 05 20, 06 10, 07 khác. */
function maintenanceOf(thbq: string): string {
  const t = thbq.toLowerCase();
  if (t.includes("vĩnh viễn") || t.includes("vinh vien")) return "01";
  for (const [nam, ma] of [["70", "02"], ["50", "03"], ["30", "04"], ["20", "05"], ["10", "06"]] as const) if (new RegExp(`\\b${nam}\\b`).test(t)) return ma;
  return "07";
}
/** dd/mm/yyyy → yyyy-mm-dd (TT02 dùng ngày ISO); chỉ có năm → đầu/cuối năm; không đúng dạng thì giữ nguyên. */
function isoNgay(s: string, cuoi = false): string {
  const t = (s ?? "").trim();
  let m = /^(\d{1,2})\/(\d{1,2})\/(\d{4})$/.exec(t);
  if (m) return `${m[3]}-${m[2].padStart(2, "0")}-${m[1].padStart(2, "0")}`;
  m = /^(\d{1,2})\/(\d{4})$/.exec(t);
  if (m) return `${m[2]}-${m[1].padStart(2, "0")}-${cuoi ? "28" : "01"}`;
  if (/^\d{4}$/.test(t)) return cuoi ? `${t}-12-31` : `${t}-01-01`;
  return t;
}

/** Một dòng mục lục đã rút gọn để ghép và lưu lại (localStorage). */
export interface DongMucLuc {
  hop: string;
  hoSo: string;
  tieuDe: string;
  batDau: string;
  ketThuc: string;
  soTo: string;
  thbq: string;
  dong: number;
  /** Metadata phẳng: mã hồ sơ (H05.13.24.01.2005.01) → dùng làm Fondcode.hồ sơ trong gói. */
  maHoSo?: string;
  maGocGiay?: string;
  /** Metadata phẳng: số tệp trên máy chủ của hồ sơ này và số tệp mang đúng mã hồ sơ này trong Excel. */
  soTep?: number;
  soTepKhop?: number;
  /** Metadata phẳng: tệp trong cùng thư mục nhưng Excel ghi mã hồ sơ khác (thường là dòng gõ lệch). */
  maKhac?: { ma: string; soTep: number }[];
  /** Metadata phẳng: số tệp khớp nhờ mã suy từ tên tệp (tên tệp trên máy chủ không có trong Excel). */
  soTepSuyMa?: number;
  /** Hồ sơ Excel không thành gói riêng: lý do (nằm trong thư mục nào). */
  lyDo?: string;
}
export interface MucLucDaNhap {
  ten: string;
  luc: string;
  loai?: "mucluc" | "phang";
  dongs: DongMucLuc[];
  /** Khoá hộp|hồ sơ bị trùng trong tệp (lấy dòng đầu). */
  trung: string[];
  /** Metadata phẳng: hồ sơ trong Excel không có tệp nào trên máy chủ. */
  khongTep?: DongMucLuc[];
  fondCode?: string;
  fondName?: string;
}
function rutGon(r: DongHoSo): DongMucLuc {
  return { hop: r.hopSo, hoSo: r.hoSoSo, tieuDe: r.tieuDe, batDau: r.tg.batDau, ketThuc: r.tg.ketThuc, soTo: r.soTo, thbq: (r.thbq ?? r.thbqGoc).trim(), dong: r.dong };
}
function gopMucLuc(cu: MucLucDaNhap | null, ten: string, moi: DongMucLuc[]): MucLucDaNhap {
  const dongs = [...(cu?.dongs ?? [])];
  const co = new Set(dongs.map((d) => khoaHoSo(d.hop, d.hoSo)));
  const trung = new Set(cu?.trung ?? []);
  for (const d of moi) {
    const k = khoaHoSo(d.hop, d.hoSo);
    if (co.has(k)) trung.add(k);
    else {
      co.add(k);
      dongs.push(d);
    }
  }
  // Giữ mã / tên phông và loại của tệp đầu: thêm tệp thứ hai mà mất mã phông thì hộp thoại
  // đóng gói lại quay về đoán theo tên thư mục (5/11/2026).
  return { ten: cu && cu.ten !== ten ? `${cu.ten} + ${ten}` : ten, luc: new Date().toISOString(), dongs, trung: [...trung], loai: cu?.loai, khongTep: cu?.khongTep, fondCode: cu?.fondCode, fondName: cu?.fondName };
}
const KHOA_MUC_LUC = (projectId: string, phong: string) => `pack.mucluc.${projectId}.${phong}`;
function docMucLucDaLuu(projectId: string, phong: string): MucLucDaNhap | null {
  try {
    const raw = localStorage.getItem(KHOA_MUC_LUC(projectId, phong));
    return raw ? (JSON.parse(raw) as MucLucDaNhap) : null;
  } catch {
    return null;
  }
}
function luuMucLuc(projectId: string, phong: string, ml: MucLucDaNhap | null) {
  try {
    if (ml) localStorage.setItem(KHOA_MUC_LUC(projectId, phong), JSON.stringify(ml));
    else localStorage.removeItem(KHOA_MUC_LUC(projectId, phong));
  } catch {
    /* mục lục lớn quá hạn mức: lần sau nhập lại */
  }
}
const CHUA_PHAN_LOAI = "Chưa phân loại";
const KINDS: V4sj[] = ["aip_hoso", "aip_tailieu", "sip_hoso", "sip_tailieu", "dip"];
const THBQ: { value: string; label: string }[] = [
  { value: "01", label: "Vĩnh viễn" },
  { value: "02", label: "70 năm" },
  { value: "03", label: "50 năm" },
  { value: "04", label: "30 năm" },
  { value: "05", label: "20 năm" },
  { value: "06", label: "10 năm" },
  { value: "07", label: "Khác" },
];
function nho(key: string): string {
  try {
    return localStorage.getItem(key) ?? "";
  } catch {
    return "";
  }
}
function luu(key: string, v: string) {
  try {
    localStorage.setItem(key, v);
  } catch {
    /* tiện ích nhớ */
  }
}
/** Mã phông gợi ý từ tên thư mục: bỏ dấu, giữ chữ số và chữ cái, nối bằng dấu chấm — người dùng sửa được. */
function goiYMaPhong(ten: string): string {
  return ten
    .normalize("NFD")
    .replace(/[̀-ͯ]/g, "")
    .replace(/đ/g, "d")
    .replace(/Đ/g, "D")
    .toUpperCase()
    .replace(/[^A-Z0-9]+/g, ".")
    .replace(/^\.+|\.+$/g, "")
    .slice(0, 40);
}

export default function PackFromServer() {
  const proj = useProject();
  const projectId = proj.projectId ?? "";
  const { data, error, loading, refreshing, reload } = V41t(() => docsReviewSummary({ projectId }), { topics: ["docs", "dossier", "packages"], pollMs: 60_000, enabled: (isTauri() || import.meta.env.DEV) && !!projectId });
  const rows = useMemo(() => (data ?? []).filter((r) => r.projectId === projectId), [data, projectId]);
  const [phong, setPhong] = useState("");
  const [hop, setHop] = useState("");
  const [q, setQ] = useState("");
  const [sel, setSel] = useState<Set<string>>(new Set());
  const [dialog, setDialog] = useState(false);
  const [mucLuc, setMucLuc] = useState<MucLucDaNhap | null>(null);
  const [docMl, setDocMl] = useState(false);
  const [xemThieu, setXemThieu] = useState(false);
  const fileRef = useRef<HTMLInputElement>(null);
  const collator = useMemo(() => new Intl.Collator("vi", { numeric: true }), []);

  // Đổi dự án → bỏ bộ lọc và lựa chọn cũ (hồ sơ của dự án khác không được lẫn vào gói).
  useEffect(() => {
    setPhong("");
    setHop("");
    setSel(new Set());
  }, [projectId]);

  const dsPhong = useMemo(() => {
    const m = new Map<string, { hoSo: number; tep: number }>();
    for (const r of rows) {
      const k = r.phong || "";
      const v = m.get(k) ?? { hoSo: 0, tep: 0 };
      v.hoSo += 1;
      v.tep += r.files;
      m.set(k, v);
    }
    return [...m.entries()].sort((a, b) => collator.compare(a[0], b[0]));
  }, [rows, collator]);
  useEffect(() => {
    if (dsPhong.length === 1 && !phong) setPhong(dsPhong[0][0] || CHUA_PHAN_LOAI);
  }, [dsPhong, phong]);
  const phongKey = phong === CHUA_PHAN_LOAI ? "" : phong;
  const trongPhong = useMemo(() => (phong ? rows.filter((r) => (r.phong || "") === phongKey) : []), [rows, phong, phongKey]);
  // Mục lục đã ghép nhớ theo dự án + phông.
  useEffect(() => {
    setMucLuc(phong ? docMucLucDaLuu(projectId, phong) : null);
  }, [projectId, phong]);
  const mlTheoKhoa = useMemo(() => {
    const m = new Map<string, DongMucLuc>();
    for (const d of mucLuc?.dongs ?? []) m.set(khoaHoSo(d.hop, d.hoSo), d);
    return m;
  }, [mucLuc]);
  const mlCua = (r: ReviewSummaryRow) => mlTheoKhoa.get(khoaHoSo(r.hop, r.hoSo));
  const soKhop = useMemo(() => trongPhong.filter((r) => mlTheoKhoa.has(khoaHoSo(r.hop, r.hoSo))).length, [trongPhong, mlTheoKhoa]);
  const dongThieuTep = useMemo(() => {
    if (!mucLuc) return [];
    if (mucLuc.loai === "phang") return mucLuc.khongTep ?? [];
    const co = new Set(trongPhong.map((r) => khoaHoSo(r.hop, r.hoSo)));
    return mucLuc.dongs.filter((d) => !co.has(khoaHoSo(d.hop, d.hoSo)));
  }, [mucLuc, trongPhong]);
  // Tên tệp từng hồ sơ trên máy chủ (nạp một lần mỗi phông khi ghép metadata phẳng).
  const tepNhomRef = useRef<{ phong: string; map: Map<string, string[]> } | null>(null);
  const napTepNhom = async (): Promise<Map<string, string[]>> => {
    if (tepNhomRef.current?.phong === `${projectId}|${phong}`) return tepNhomRef.current.map;
    const all = await docsForGroups(trongPhong.map((r) => ({ projectId: r.projectId, phong: r.phong, hop: r.hop, hoSo: r.hoSo })));
    const map = new Map<string, string[]>();
    for (const d of all) {
      if (d.status === "disposed") continue;
      const k = reviewKeyOf(d);
      map.set(k, [...(map.get(k) ?? []), d.filename.toLowerCase()]);
    }
    tepNhomRef.current = { phong: `${projectId}|${phong}`, map };
    return map;
  };
  /** Metadata phẳng → một dòng cho mỗi hồ sơ trên máy chủ: mã hồ sơ chiếm đa số tệp. */
  const ghepPhang = async (ten: string, rows: unknown[][], dongTieuDe: number): Promise<MucLucDaNhap> => {
    const md = docMetadataPhang(rows, dongTieuDe);
    const tepNhom = await napTepNhom();
    const theoMa = new Map(md.hoSos.map((h) => [h.maHoSo, h]));
    const maHoa = new Map(md.hoSos.map((h) => [h.maHoSo.toUpperCase(), h.maHoSo]));
    const daDung = new Set<string>();
    const lyDoKhac = new Map<string, string>();
    const dongs: DongMucLuc[] = [];
    for (const r of trongPhong) {
      const tep = tepNhom.get(idOf(r)) ?? [];
      const dem = new Map<string, number>();
      let suyMa = 0;
      for (const t of tep) {
        let ma = md.tepToiHoSo.get(t);
        if (!ma) {
          // Tên tệp không có trong Excel: thử mã hồ sơ suy từ tên tệp.
          const suy = maTuTenTep(t);
          const ma2 = suy ? maHoa.get(suy) : undefined;
          if (ma2) {
            ma = ma2;
            suyMa += 1;
          }
        }
        if (ma) dem.set(ma, (dem.get(ma) ?? 0) + 1);
      }
      if (dem.size === 0) continue;
      // Mã chiếm nhiều tệp nhất; hoà thì ưu tiên mã đầy đủ (có năm 4 chữ số, không có ".."), rồi mã xuất hiện trước trong Excel.
      const duDay = (ma: string) => (/\.\d{4}\./.test(ma) && !ma.includes("..") ? 1 : 0);
      const xep = [...dem.entries()].sort((a, b) => b[1] - a[1] || duDay(b[0]) - duDay(a[0]) || (theoMa.get(a[0])?.dong ?? 0) - (theoMa.get(b[0])?.dong ?? 0));
      const [ma, khop] = xep[0];
      const h = theoMa.get(ma)!;
      daDung.add(ma);
      const maKhac = xep.slice(1).map(([m, k]) => ({ ma: m, soTep: k }));
      for (const { ma: m } of maKhac) lyDoKhac.set(m, `${r.hop || "—"} / ${r.hoSo || "—"} (mã chính ${ma})`);
      dongs.push({ hop: r.hop, hoSo: r.hoSo, tieuDe: h.tieuDe, batDau: h.batDau, ketThuc: h.ketThuc, soTo: h.soTo, thbq: h.thbq, dong: h.dong, maHoSo: h.maHoSo, maGocGiay: h.maGocGiay, soTep: tep.length, soTepKhop: khop, maKhac: maKhac.length ? maKhac : undefined, soTepSuyMa: suyMa || undefined });
    }
    const khongTep = md.hoSos
      .filter((h) => !daDung.has(h.maHoSo))
      .map((h) => ({ hop: "", hoSo: "", tieuDe: h.tieuDe, batDau: h.batDau, ketThuc: h.ketThuc, soTo: h.soTo, thbq: h.thbq, dong: h.dong, maHoSo: h.maHoSo, soTep: h.tep.length, lyDo: lyDoKhac.get(h.maHoSo) ? `tệp nằm trong ${lyDoKhac.get(h.maHoSo)}` : "không tệp nào trùng tên trên máy chủ" }));
    return { ten, luc: new Date().toISOString(), loai: "phang", dongs, trung: [], khongTep, fondCode: md.fondCode, fondName: md.fondName };
  };
  const nhapMucLuc = async (files: File[]) => {
    if (!phong) {
      toast.error("Chọn phông trước rồi mới nhập mục lục.");
      return;
    }
    if (files.length === 0) {
      toast.error("Chưa chọn tệp Excel nào.");
      return;
    }
    setDocMl(true);
    try {
      let ml = mucLuc?.loai === "phang" ? null : mucLuc;
      for (const f of files) {
        const { sheet, rows: luoi } = await docLuoiExcel(await f.arrayBuffer());
        const iPhang = laMetadataPhang(luoi);
        if (iPhang >= 0) {
          // Metadata phẳng: thay hẳn mục lục cũ của phông (ghép theo tên tệp, không gộp với mẫu mục lục).
          const kq = await ghepPhang(f.name, luoi, iPhang);
          ml = kq;
          const soTepKhop = kq.dongs.reduce((a, d) => a + (d.soTepKhop ?? 0), 0);
          const tong = kq.dongs.length + (kq.khongTep?.length ?? 0);
          toast.success(`Metadata ${n(tong)} hồ sơ: khớp ${n(kq.dongs.length)}/${n(trongPhong.length)} hồ sơ trên máy chủ (${n(soTepKhop)} tệp cùng tên).`, {
            description: kq.khongTep?.length ? `${n(kq.khongTep.length)} hồ sơ trong Excel chưa có tệp nào trên máy chủ.` : undefined,
          });
          continue;
        }
        // 10/11/2026: đọc bằng MÁY CHỦ (`/cl/muc-luc/doc`) thay vì bộ TS trên máy — cùng bộ luật với
        // màn Nhập mục lục và cổng nhà cung cấp. Danh mục THBQ cũng là danh mục THẬT của dự án thay
        // cho `THBQ_DANH_MUC` viết cứng ở đây (bộ cứng đó thiếu/thừa so với Danh mục từng dự án).
        // Truyền `sheet` mà `docLuoiExcel` đã chọn để giữ đúng sheet như trước.
        const { pt } = await docMucLucMayChu(projectId, await f.arrayBuffer(), f.name, { sheet });
        const dongs = pt.rows.filter((r) => r.hoSoSo).map(rutGon);
        if (dongs.length === 0) {
          toast.error(`${f.name}: không thấy dòng hồ sơ nào`, { detail: pt.vanDe[0]?.noiDung ?? "Cần dòng tiêu đề cột Hộp số · Hồ sơ số · Tiêu đề hồ sơ · Thời gian BĐ - KT trong 15 dòng đầu." });
          continue;
        }
        ml = gopMucLuc(ml?.loai === "phang" ? null : ml, f.name, dongs);
        const co = new Set(trongPhong.map((r) => khoaHoSo(r.hop, r.hoSo)));
        const khop = ml.dongs.filter((d) => co.has(khoaHoSo(d.hop, d.hoSo))).length;
        toast.success(`Mục lục ${n(ml.dongs.length)} hồ sơ, khớp ${n(khop)}/${n(trongPhong.length)} hồ sơ trên máy chủ.`, { description: ml.trung.length ? `${n(ml.trung.length)} cặp hộp/hồ sơ trùng trong tệp, lấy dòng đầu.` : undefined });
      }
      if (ml) {
        setMucLuc(ml);
        luuMucLuc(projectId, phong, ml);
      }
    } catch (e) {
      notifyError(e, "đọc mục lục Excel");
    } finally {
      setDocMl(false);
    }
  };
  const boMucLuc = () => {
    setMucLuc(null);
    luuMucLuc(projectId, phong, null);
  };
  const dsHop = useMemo(() => [...new Set(trongPhong.map((r) => r.hop || ""))].sort(collator.compare), [trongPhong, collator]);
  const loc = useMemo(() => {
    const k = q.trim().toLowerCase();
    return trongPhong
      .filter((r) => (!hop || (r.hop || "") === hop) && (!k || `${r.hop} ${r.hoSo}`.toLowerCase().includes(k)))
      .sort((a, b) => collator.compare(a.hop, b.hop) || collator.compare(a.hoSo, b.hoSo));
  }, [trongPhong, hop, q, collator]);
  const idOf = (r: ReviewSummaryRow) => reviewKey(r.projectId, r.phong, r.hop, r.hoSo);
  const daChon = useMemo(() => rows.filter((r) => sel.has(idOf(r))), [rows, sel]);
  // "Khớp" = có dòng mục lục VÀ không lệch tệp nào. Hồ sơ lệch bị bỏ qua, đóng gói sau khi
  // bên lập mục lục sửa xong — quyết định của anh Hiếu 5/11/2026.
  const dongChuan = (d: DongMucLuc | undefined) => !!d && !d.maKhac?.length && !(d.soTep != null && d.soTepKhop != null && d.soTepKhop < d.soTep);
  const khopHienThi = useMemo(() => loc.filter((r) => dongChuan(mlTheoKhoa.get(khoaHoSo(r.hop, r.hoSo)))), [loc, mlTheoKhoa]);
  const soKhopChuan = useMemo(() => trongPhong.filter((r) => dongChuan(mlTheoKhoa.get(khoaHoSo(r.hop, r.hoSo)))).length, [trongPhong, mlTheoKhoa]);
  // Hồ sơ trên máy chủ không có dòng nào trong Excel (cột Tiêu đề ghi "Chưa có trong mục lục").
  const thieuMucLuc = useMemo(() => trongPhong.filter((r) => !mlTheoKhoa.has(khoaHoSo(r.hop, r.hoSo))), [trongPhong, mlTheoKhoa]);
  // Dòng Excel ghép được nhưng tệp trong thư mục mang mã hồ sơ khác, hoặc tên tệp không có trong Excel.
  const lechTep = useMemo(
    () => (mucLuc?.dongs ?? []).filter((d) => d.maKhac?.length || (d.soTep != null && d.soTepKhop != null && d.soTepKhop < d.soTep)),
    [mucLuc],
  );
  const soLech = dongThieuTep.length + thieuMucLuc.length + (mucLuc?.trung.length ?? 0) + lechTep.length;
  const [dangXuat, setDangXuat] = useState(false);
  const xuatLech = async () => {
    if (!mucLuc) return;
    setDangXuat(true);
    try {
      const ngay = new Date().toLocaleString("vi-VN");
      const tong: Sheet["aoa"] = [
        ["Đối chiếu mục lục với tài liệu trên máy chủ"],
        [],
        ["Phông", phong],
        ["Dự án", trongPhong[0]?.projectCode || projectId],
        ["Tệp mục lục", mucLuc.ten],
        ["Mã phông trong tệp", mucLuc.fondCode ?? ""],
        ["Xuất lúc", ngay],
        [],
        ["Hồ sơ trong Excel", mucLuc.dongs.length + (mucLuc.khongTep?.length ?? 0)],
        ["Hồ sơ trên máy chủ", trongPhong.length],
        ["Khớp hộp/hồ sơ hai bên", soKhop],
        ["Đóng gói được (khớp và không lệch tệp)", soKhopChuan],
        ["Chưa đóng gói, chờ sửa mục lục", trongPhong.length - soKhopChuan],
        [],
        ["CHỖ LỆCH", "Số dòng", "Ở trang"],
        [mucLuc.loai === "phang" ? "Mã hồ sơ Excel không thành hồ sơ riêng" : "Dòng mục lục không có tệp trên máy chủ", dongThieuTep.length, "Excel thua"],
        ["Hồ sơ trên máy chủ chưa có trong Excel", thieuMucLuc.length, "May chu thua"],
        ["Cặp hộp/hồ sơ trùng trong Excel", mucLuc.trung.length, "Trung trong Excel"],
        ["Hồ sơ khớp nhưng tệp lệch mã", lechTep.length, "Lech ma tep"],
        [],
        ["Hướng dẫn", "Sửa trong tệp mục lục gốc rồi nhập lại; hồ sơ khớp vẫn đóng gói bình thường."],
      ];
      const sheets: Sheet[] = [
        { name: "Tong hop", aoa: tong, cols: [{ wch: 52 }, { wch: 46 }, { wch: 20 }] },
        {
          name: "Excel thua",
          aoa: [
            ["Dòng", "Mã hồ sơ", "Hộp", "Hồ sơ", "Tiêu đề", "Số tệp trong Excel", "Vì sao không ghép được"],
            ...dongThieuTep.map((d) => [d.dong, d.maHoSo ?? "", d.hop, d.hoSo, d.tieuDe, d.soTep ?? "", d.lyDo ?? ""] as Sheet["aoa"][number]),
          ],
          cols: [{ wch: 7 }, { wch: 26 }, { wch: 10 }, { wch: 10 }, { wch: 60 }, { wch: 18 }, { wch: 60 }],
        },
        {
          name: "May chu thua",
          aoa: [
            ["Hộp", "Hồ sơ", "Số tệp", "Số trang", "Đã có PDF/A", "Đã duyệt"],
            ...thieuMucLuc.map((r) => [r.hop, r.hoSo, r.files, r.pages, r.pdfaDocs, r.approved] as Sheet["aoa"][number]),
          ],
          cols: [{ wch: 12 }, { wch: 12 }, { wch: 10 }, { wch: 10 }, { wch: 14 }, { wch: 12 }],
        },
        {
          name: "Trung trong Excel",
          aoa: [
            ["Hộp", "Hồ sơ", "Tiêu đề dòng được lấy", "Dòng được lấy"],
            ...mucLuc.trung.map((k) => {
              const d = mlTheoKhoa.get(k);
              return [k.split("|")[0] ?? "", k.split("|")[1] ?? "", d?.tieuDe ?? "", d?.dong ?? ""] as Sheet["aoa"][number];
            }),
          ],
          cols: [{ wch: 12 }, { wch: 12 }, { wch: 60 }, { wch: 14 }],
        },
        {
          name: "Lech ma tep",
          aoa: [
            ["Dòng", "Mã hồ sơ", "Hộp", "Hồ sơ", "Tệp trong thư mục", "Tệp đúng mã", "Mã khác Excel ghi cho tệp cùng thư mục"],
            ...lechTep.map(
              (d) =>
                [d.dong, d.maHoSo ?? "", d.hop, d.hoSo, d.soTep ?? "", d.soTepKhop ?? "", (d.maKhac ?? []).map((k) => `${k.ma} (${k.soTep} tệp)`).join(", ")] as Sheet["aoa"][number],
            ),
          ],
          cols: [{ wch: 7 }, { wch: 26 }, { wch: 10 }, { wch: 10 }, { wch: 18 }, { wch: 14 }, { wch: 60 }],
        },
      ];
      const ten = `doi-chieu-muc-luc-${(mucLuc.fondCode || phong).replace(/[^\w.-]+/g, "-")}.xlsx`;
      await writeXlsx(sheets, ten);
      toast.success(`Đã xuất ${n(soLech)} chỗ lệch ra Excel.`, { description: ten });
    } catch (e) {
      notifyError(e, "xuất Excel đối chiếu");
    } finally {
      setDangXuat(false);
    }
  };

  const tiLe = (a: number, b: number, tone: "ok" | "warn") => (b === 0 ? "—" : <Badge tone={a >= b ? "ok" : a === 0 ? "gray" : tone}>{`${n(a)}/${n(b)}`}</Badge>);
  const cols: Column<ReviewSummaryRow>[] = [
    { key: "hop", header: "Hộp", width: 110, sortable: true, sortValue: (r) => r.hop, cell: (r) => r.hop || "—" },
    { key: "hoSo", header: "Hồ sơ", width: 120, sortable: true, sortValue: (r) => r.hoSo, cell: (r) => r.hoSo || "—" },
    ...(mucLuc
      ? ([
          {
            key: "tieuDe",
            header: "Tiêu đề (mục lục)",
            className: "whitespace-normal",
            cell: (r: ReviewSummaryRow) => {
              const d = mlCua(r);
              if (!d) return <span className="text-warning-700">Chưa có trong mục lục</span>;
              return (
                <span title={`Dòng ${d.dong} · ${mucLuc.ten}`}>
                  {d.maHoSo && <span className="mr-1.5 font-mono text-xs text-fg-subtle">{d.maHoSo}</span>}
                  {d.tieuDe}
                  {d.maKhac?.length ? (
                    <Tip text={`Excel ghi mã khác cho ${n(d.soTep! - d.soTepKhop!)} tệp trong thư mục này: ${d.maKhac.map((k) => `${k.ma} (${n(k.soTep)} tệp)`).join(", ")}. Gói vẫn lấy đủ ${n(d.soTep!)} tệp theo mã chính; sửa mã trong Excel nếu đó là dòng gõ lệch.`}>
                      <Badge tone="warn" className="ml-1.5">{`${n(d.soTep! - d.soTepKhop!)} tệp Excel ghi mã khác`}</Badge>
                    </Tip>
                  ) : d.soTep != null && d.soTepKhop != null && d.soTepKhop < d.soTep ? (
                    <Tip text="Tên tệp này không có trong Excel và cũng không suy được mã hồ sơ từ tên. Gói vẫn lấy đủ tệp trong thư mục.">
                      <Badge tone="warn" className="ml-1.5">{`${n(d.soTep - d.soTepKhop)} tệp không có trong Excel`}</Badge>
                    </Tip>
                  ) : null}
                  {d.soTepSuyMa ? (
                    <Tip text="Tên tệp trên máy chủ khác Excel (thường do số thứ tự tệp đánh lại) nhưng mã hồ sơ trong tên tệp có trong Excel nên vẫn ghép được.">
                      <Badge tone="info" className="ml-1.5">{`${n(d.soTepSuyMa)} tệp ghép theo mã`}</Badge>
                    </Tip>
                  ) : null}
                </span>
              );
            },
          },
          { key: "tg", header: "Thời gian", width: 190, cell: (r: ReviewSummaryRow) => { const d = mlCua(r); return d ? [d.batDau, d.ketThuc].filter(Boolean).join(" – ") || "—" : ""; } },
          { key: "thbq", header: "THBQ", width: 100, cell: (r: ReviewSummaryRow) => mlCua(r)?.thbq ?? "" },
        ] as Column<ReviewSummaryRow>[])
      : []),
    { key: "files", header: "Tệp", align: "right", width: 80, sortable: true, cell: (r) => n(r.files) },
    { key: "pages", header: "Trang", align: "right", width: 90, sortable: true, cell: (r) => n(r.pages) },
    { key: "pdfa", header: "PDF/A", width: 110, cell: (r) => tiLe(r.pdfaDocs, r.files, "warn") },
    { key: "signed", header: "Ký số", width: 110, cell: (r) => tiLe(r.signedDocs, r.files, "warn") },
    { key: "approved", header: "Đã duyệt", width: 110, cell: (r) => tiLe(r.approved, r.files, "warn") },
  ];

  if (!projectId) {
    return (
      <Card padding="md">
        <EmptyState icon="duAn" title="Chọn dự án đang làm ở thanh trên" description="Phông, hộp, hồ sơ để đóng gói lấy từ tài liệu đã lên máy chủ của dự án đó." />
      </Card>
    );
  }
  return (
    <Card padding="none">
      <div className="flex flex-wrap items-end gap-3 border-b border-border-subtle p-3">
        <Field label={`Phông · dự án ${proj.projectLabel || projectId}`}>
          <Select
            size="sm"
            value={phong}
            onChange={(v) => {
              setPhong(v);
              setHop("");
            }}
            placeholder={dsPhong.length ? "— Chọn phông —" : loading ? "Đang tải…" : "Dự án chưa có tài liệu"}
            options={dsPhong.map(([p, v]) => ({ value: p || CHUA_PHAN_LOAI, label: `${p || CHUA_PHAN_LOAI} · ${n(v.hoSo)} hồ sơ · ${n(v.tep)} tệp` }))}
            className="w-96"
          />
        </Field>
        <Field label="Hộp">
          <Select size="sm" value={hop} disabled={!phong} onChange={setHop} placeholder="Tất cả" options={dsHop.map((h) => ({ value: h, label: h || "—" }))} className="w-40" />
        </Field>
        <Field label="Tìm">
          <Input size="sm" prefix="tim" value={q} onChange={(e) => setQ(e.target.value)} placeholder="Số hộp, số hồ sơ" className="w-56" />
        </Field>
        <span className="flex-1" />
        {phong && (
          <>
            <input ref={fileRef} type="file" accept=".xlsx,.xls,.xlsm" multiple hidden aria-hidden onChange={(e) => { const fs = Array.from(e.target.files ?? []); e.target.value = ""; void nhapMucLuc(fs); }} />
            <Tip text="Tệp Mục lục hồ sơ (Hộp số · Hồ sơ số · Tiêu đề · Thời gian · Số tờ · THBQ). Ghép theo số hộp + số hồ sơ; chọn thêm tệp thứ hai (vĩnh viễn / có thời hạn) thì gộp.">
              <Button size="sm" icon="nhapTep" loading={docMl} onClick={() => fileRef.current?.click()}>
                {mucLuc ? "Thêm mục lục" : "Nhập mục lục Excel"}
              </Button>
            </Tip>
          </>
        )}
        <span className="text-sm text-fg-subtle">
          {phong ? `${n(loc.length)} hồ sơ` : `${n(dsPhong.length)} phông · ${n(rows.length)} hồ sơ`}
          {sel.size ? ` · đã chọn ${n(sel.size)}` : ""}
        </span>
        <IconButton label="Tải lại" icon="lamMoi" size="sm" variant="secondary" loading={refreshing} onClick={reload} />
        <Button variant="primary" icon="goi" disabled={sel.size === 0} onClick={() => setDialog(true)}>
          Đóng gói ({n(sel.size)})
        </Button>
      </div>
      {mucLuc && phong && (
        <div className="flex flex-wrap items-center gap-2 border-b border-border-subtle bg-surface-sunken px-3 py-1.5 text-sm">
          <Badge tone="info">{mucLuc.loai === "phang" ? "Metadata" : "Mục lục"}: {mucLuc.ten}</Badge>
          {mucLuc.fondCode && <Badge tone="gray">Mã phông {mucLuc.fondCode}</Badge>}
          <span>
            {n(mucLuc.dongs.length + (mucLuc.khongTep?.length ?? 0))} hồ sơ trong Excel · đóng gói được <b>{n(soKhopChuan)}</b>/{n(trongPhong.length)} hồ sơ trên máy chủ
            {dongThieuTep.length ? <> · <button type="button" className="text-fg-link hover:underline" onClick={() => setXemThieu(true)}>{mucLuc.loai === "phang" ? `${n(dongThieuTep.length)} mã hồ sơ Excel không thành gói riêng` : `${n(dongThieuTep.length)} dòng mục lục không có tệp`}</button></> : null}
            {mucLuc.trung.length ? ` · ${n(mucLuc.trung.length)} cặp hộp/hồ sơ trùng trong tệp` : ""}
          </span>
          <span className="flex-1" />
          <Tip text="Chọn sẵn hồ sơ khớp mục lục rồi mở hộp thoại đóng gói. Hồ sơ lệch không được lấy vào; bấm Huỷ ở hộp thoại thì phần đã chọn vẫn giữ để sửa tay.">
            <span>
              <Button size="sm" variant="primary" icon="goi" disabled={khopHienThi.length === 0} onClick={() => { setSel(new Set(khopHienThi.map(idOf))); setDialog(true); }}>
                Đóng gói {n(khopHienThi.length)} hồ sơ khớp
              </Button>
            </span>
          </Tip>
          <Tip text="Xuất Excel mọi chỗ lệch giữa mục lục và tài liệu trên máy chủ (Excel thừa, máy chủ thừa, trùng, tệp lệch mã) để gửi lại bên lập mục lục kiểm tra.">
            <span>
              <Button size="sm" variant="secondary" icon="xuatTep" loading={dangXuat} disabled={soLech === 0} onClick={() => void xuatLech()}>
                Xuất Excel {n(soLech)} chỗ lệch
              </Button>
            </span>
          </Tip>
          <Button size="sm" variant="ghost" icon="xoa" onClick={boMucLuc}>
            Bỏ mục lục
          </Button>
        </div>
      )}
      {mucLuc && phong && soLech > 0 && (
        <div className="border-b border-border-subtle bg-warning-bg px-3 py-1.5 text-xs text-warning-700">
          Hồ sơ lệch mục lục được bỏ qua, chưa đóng gói. Xuất Excel gửi bên lập mục lục sửa, nhập lại tệp đã sửa rồi đóng gói nốt phần đó.
        </div>
      )}
      {xemThieu && (
        <Dialog open onOpenChange={(o) => !o && setXemThieu(false)} title={mucLuc?.loai === "phang" ? `${n(dongThieuTep.length)} mã hồ sơ trong Excel không thành gói riêng` : `${n(dongThieuTep.length)} hồ sơ trong Excel không có tệp trên máy chủ`} description={mucLuc?.loai === "phang" ? `Phông ${phong}: mã hồ sơ này hoặc không có tệp trùng tên trên máy chủ, hoặc tệp của nó nằm chung thư mục với một mã khác (thường là dòng Excel gõ lệch mã).` : `Phông ${phong}: hồ sơ ghi trong mục lục nhưng chưa thấy thư mục hộp/hồ sơ tương ứng đã đẩy lên.`} size="xl">
          <Table
            columns={[
              { key: "dong", header: "Dòng", width: 70, align: "right", cell: (d: DongMucLuc) => d.dong },
              { key: "ma", header: mucLuc?.loai === "phang" ? "Mã hồ sơ" : "Hộp · Hồ sơ", width: 200, cell: (d: DongMucLuc) => d.maHoSo || `${d.hop} · ${d.hoSo}` },
              { key: "tieuDe", header: "Tiêu đề", className: "whitespace-normal", cell: (d: DongMucLuc) => d.tieuDe },
              { key: "soTep", header: "Tệp trong Excel", width: 110, align: "right", cell: (d: DongMucLuc) => (d.soTep != null ? n(d.soTep) : "") },
              ...(mucLuc?.loai === "phang" ? [{ key: "lyDo", header: "Vì sao không thành gói riêng", className: "whitespace-normal text-sm", cell: (d: DongMucLuc) => d.lyDo ?? "" } as Column<DongMucLuc>] : []),
            ]}
            rows={dongThieuTep.slice(0, 500)}
            getRowId={(d) => String(d.dong)}
            dense
            maxHeight={420}
          />
        </Dialog>
      )}
      <Table
        columns={cols}
        rows={loc}
        getRowId={idOf}
        dense
        selection={{ selected: sel, onChange: setSel }}
        maxHeight="calc(100dvh - 360px)"
        state={{
          loading: loading && !data,
          error: data ? undefined : error,
          onRetry: reload,
          empty: !phong ? { title: "Chọn phông để xem hồ sơ", description: "Mỗi thư mục đã đẩy lên máy chủ là một phông; bên trong là hộp, hồ sơ." } : { title: "Không có hồ sơ khớp bộ lọc" },
        }}
      />
      {dialog && <PackServerDialog rows={daChon} mucLuc={mlTheoKhoa} excelFond={mucLuc ? { code: mucLuc.fondCode ?? "", name: mucLuc.fondName ?? "", ten: mucLuc.ten } : undefined} projectId={projectId} projectLabel={proj.projectLabel || projectId} onClose={() => setDialog(false)} />}
    </Card>
  );
}

interface KetQua {
  key: string;
  ok: number;
  loi: string[];
}

function PackServerDialog({ rows, mucLuc, excelFond, projectId, projectLabel, onClose }: { rows: ReviewSummaryRow[]; mucLuc: Map<string, DongMucLuc>; excelFond?: { code: string; name: string; ten: string }; projectId: string; projectLabel: string; onClose: () => void }) {
  const mlCua = (r: ReviewSummaryRow) => mucLuc.get(khoaHoSo(r.hop, r.hoSo));
  const soCoMl = rows.filter((r) => mlCua(r)).length;
  const [chiCoMl, setChiCoMl] = useState(soCoMl > 0 && soCoMl < rows.length);
  const dsPhong = useMemo(() => [...new Set(rows.map((r) => r.phong || CHUA_PHAN_LOAI))], [rows]);
  const motPhong = dsPhong.length === 1 ? dsPhong[0] : "";
  const [kind, setKind] = useState<V4sj>("aip_hoso");
  // Mã / tên phông: có tệp metadata thì LẤY THẲNG trong đó, khoá ô không cho gõ — mã phông gõ tay
  // lệch với mã hồ sơ trong Excel là hỏng mã lưu trữ cả phông (5/11/2026). Không có metadata mới
  // cho gõ, mặc định theo lần nhớ trước rồi mới đoán từ tên thư mục.
  const maTuMetadata = !!excelFond?.code;
  const tenTuMetadata = !!excelFond?.name;
  const tuMetadata = maTuMetadata;
  const [fondCode, setFondCode] = useState(() => excelFond?.code || (motPhong ? nho(`pack.fondCode.${motPhong}`) || goiYMaPhong(motPhong) : ""));
  const [fondName, setFondName] = useState(() => excelFond?.name || (motPhong ? nho(`pack.fondName.${motPhong}`) || motPhong : ""));
  const [orgCode, setOrgCode] = useState(() => nho("pack.archiveOrgCode"));
  const [orgName, setOrgName] = useState(() => nho("pack.archiveOrgName"));
  const [maintenance, setMaintenance] = useState("01");
  const [agreement, setAgreement] = useState("");
  const [dip, setDip] = useState<Partial<V4sm>>({ requestDate: new Date().toISOString().slice(0, 10) });
  const [busy, setBusy] = useState(false);
  const [tienDo, setTienDo] = useState("");
  const [ketQua, setKetQua] = useState<Map<string, KetQua> | null>(null);
  const idOf = (r: ReviewSummaryRow) => reviewKey(r.projectId, r.phong, r.hop, r.hoSo);

  const chay = async () => {
    setBusy(true);
    luu("pack.archiveOrgCode", orgCode.trim());
    luu("pack.archiveOrgName", orgName.trim());
    if (motPhong && !tuMetadata) {
      luu(`pack.fondCode.${motPhong}`, fondCode.trim());
      luu(`pack.fondName.${motPhong}`, fondName.trim());
    }
    const out = new Map<string, KetQua>();
    const tepTheoNhom = new Map<string, V4ej[]>();
    try {
      const all = await docsForGroups(
        rows.map((r) => ({ projectId: r.projectId, phong: r.phong, hop: r.hop, hoSo: r.hoSo })),
        (d, t) => setTienDo(`Đang nạp tài liệu ${d}/${t} lô…`),
      );
      for (const d of all) {
        if (d.status === "disposed") continue;
        const k = reviewKeyOf(d);
        tepTheoNhom.set(k, [...(tepTheoNhom.get(k) ?? []), d]);
      }
    } catch (e) {
      notifyError(e, "nạp tài liệu trên máy chủ");
      setBusy(false);
      return;
    }
    let i = 0;
    for (const r of rows) {
      i += 1;
      setTienDo(`Đang tạo gói ${i}/${rows.length}…`);
      const key = idOf(r);
      const kq: KetQua = { key, ok: 0, loi: [] };
      out.set(key, kq);
      const ml = mlCua(r);
      if (chiCoMl && !ml) {
        kq.loi.push("bỏ qua: chưa có dòng mục lục");
        continue;
      }
      const ds = (tepTheoNhom.get(key) ?? []).sort((a, b) => a.filename.localeCompare(b.filename, "vi", { numeric: true }));
      if (ds.length === 0) {
        kq.loi.push("không có tài liệu nào trên máy chủ");
        continue;
      }
      const hopSo = so(r.hop);
      const hsSo = so(r.hoSo);
      const tieuDe = ml?.tieuDe?.trim() || `${r.phong || CHUA_PHAN_LOAI} – ${r.hop || "hộp —"} – ${r.hoSo || "hồ sơ —"}`;
      const base: V4sm = {
        ...V4sn,
        ...(kind === "dip" ? dip : {}),
        kind,
        fondCode: fondCode.trim(),
        archiveOrgCode: orgCode.trim(),
        fondName: fondName.trim() || r.phong,
        archiveOrgName: orgName.trim(),
        // Mã hồ sơ: theo Excel metadata nếu có (H05.13.24.01.2005.01), không thì <mã phông>.<hộp>.<hồ sơ>.
        fileCode: ml?.maHoSo?.trim() || `${fondCode.trim()}.${hopSo}.${hsSo}`,
        title: tieuDe,
        // THBQ: theo dòng mục lục nếu có, không thì theo lựa chọn chung; SIP luôn vĩnh viễn.
        maintenance: kind.startsWith("sip") ? "01" : ml?.thbq ? maintenanceOf(ml.thbq) : maintenance,
        numberOfPaper: ml?.soTo ? String(Number(ml.soTo) || ml.soTo) : "",
        numberOfPage: String(ds.reduce((a, d) => a + (d.pageCount ?? 0), 0) || ""),
        paperFileCode: ml?.maGocGiay?.trim() || ml?.maHoSo?.trim() || `${hopSo}.${hsSo}`,
        startDate: ml ? isoNgay(ml.batDau) : "",
        endDate: ml ? isoNgay(ml.ketThuc, true) : "",
        submissionAgreement: agreement,
        certTrichYeu: dip.certTrichYeu || tieuDe,
      };
      const nhom = kind === "aip_tailieu" || kind === "sip_tailieu" ? ds.map((d) => [d]) : [ds];
      for (const [j, g] of nhom.entries()) {
        const form: V4sm = nhom.length > 1 ? { ...base, fileCode: `${base.fileCode}.${String(j + 1).padStart(3, "0")}`, title: g[0].filename } : base;
        const { request, errors } = V4su(form, g.map((d) => ({ documentId: d.id, filename: d.filename })));
        if (!request) {
          kq.loi.push(...errors);
          break;
        }
        try {
          await V4rv({ projectId, hoSoRef: g[0].hoSoRef, request });
          kq.ok += 1;
        } catch (e) {
          kq.loi.push(errorText(e));
        }
      }
      setKetQua(new Map(out));
    }
    setKetQua(out);
    setBusy(false);
    setTienDo("");
    const tong = [...out.values()].reduce((a, k) => a + k.ok, 0);
    const loi = [...out.values()].filter((k) => k.loi.length > 0).length;
    if (tong > 0) toast.success(`Đã tạo ${n(tong)} gói. Máy chủ đang dựng, theo dõi ở tab Gói tin.`, { description: loi ? `${n(loi)} hồ sơ có lỗi, xem cột Kết quả.` : undefined });
    else toast.error("Không tạo được gói nào. Xem cột Kết quả.");
  };

  const cols: Column<ReviewSummaryRow>[] = [
    ...(dsPhong.length > 1 ? [{ key: "phong", header: "Phông", className: "whitespace-normal", cell: (r: ReviewSummaryRow) => r.phong || CHUA_PHAN_LOAI } as Column<ReviewSummaryRow>] : []),
    { key: "hop", header: "Hộp", width: 100, cell: (r) => r.hop || "—" },
    { key: "hoSo", header: "Hồ sơ", width: 110, cell: (r) => r.hoSo || "—" },
    ...(mucLuc.size
      ? ([
          { key: "tieuDe", header: "Tiêu đề (mục lục)", className: "whitespace-normal", cell: (r: ReviewSummaryRow) => mlCua(r)?.tieuDe ?? <span className="text-warning-700">chưa có</span> },
          { key: "thbq", header: "THBQ", width: 90, cell: (r: ReviewSummaryRow) => mlCua(r)?.thbq ?? "" },
        ] as Column<ReviewSummaryRow>[])
      : []),
    { key: "files", header: "Tệp", align: "right", width: 70, cell: (r) => n(r.files) },
    { key: "signed", header: "Đã ký", align: "right", width: 80, cell: (r) => n(r.signedDocs) },
    ...(ketQua
      ? [
          {
            key: "kq",
            header: "Kết quả",
            className: "whitespace-normal text-sm",
            cell: (r: ReviewSummaryRow) => {
              const kq = ketQua.get(idOf(r));
              if (!kq) return "…";
              return (
                <>
                  {kq.ok > 0 && <span className="text-success-700">Đã tạo {kq.ok} gói. </span>}
                  {kq.loi.length > 0 && <span className="text-danger-700">{kq.loi.join(" · ")}</span>}
                </>
              );
            },
          } as Column<ReviewSummaryRow>,
        ]
      : []),
  ];
  const chuaKy = rows.filter((r) => r.signedDocs < r.files).length;

  return (
    <Dialog
      open
      onOpenChange={(o) => !o && !busy && onClose()}
      closable={!busy}
      title={`Đóng gói ${n(rows.length)} hồ sơ`}
      description={`Dự án ${projectLabel}${motPhong ? ` · phông ${motPhong}` : ` · ${dsPhong.length} phông`}. Mỗi hồ sơ một gói; gói lấy bản ký số, không có thì lấy PDF/A.`}
      size="xl"
      footer={
        <>
          <span className="self-center text-sm text-fg-subtle">{tienDo || (chuaKy ? `${n(chuaKy)} hồ sơ còn tài liệu chưa ký số.` : "")}</span>
          <span className="flex-1" />
          <Button onClick={onClose} disabled={busy}>
            {ketQua ? "Đóng" : "Huỷ"}
          </Button>
          <Button variant="primary" icon="goi" loading={busy} disabled={!!ketQua && !busy} onClick={() => void chay()}>
            Đóng gói {V4sk[kind]}
          </Button>
        </>
      }
    >
      {!motPhong && <ErrorBanner error="Đã chọn hồ sơ của nhiều phông: mã phông và tên cơ quan bên dưới áp cho tất cả. Nên đóng gói từng phông một." className="mb-3" />}
      <FormGrid cols={3}>
        <Field label="Loại gói">
          <Select value={kind} onChange={(v) => setKind(v as V4sj)} options={KINDS.map((k) => ({ value: k, label: V4sk[k] }))} />
        </Field>
        <Field label="Mã phông" hint={maTuMetadata ? `Lấy trong ${excelFond?.ten ?? "tệp metadata"}, cùng mã với mã hồ sơ` : "Fondcode trong gói; gợi ý từ tên thư mục"}>
          <Input value={fondCode} readOnly={maTuMetadata} onChange={(e) => setFondCode(e.target.value)} className={maTuMetadata ? "bg-surface-sunken text-fg-subtle" : undefined} />
        </Field>
        <Field label="Tên cơ quan hình thành phông" hint={tenTuMetadata ? "Lấy trong tệp metadata" : maTuMetadata ? "Tệp metadata không ghi tên phông — gõ cho đúng tên trên quyết định" : undefined}>
          <Input value={fondName} readOnly={tenTuMetadata} onChange={(e) => setFondName(e.target.value)} placeholder={motPhong || "UBND xã …"} className={tenTuMetadata ? "bg-surface-sunken text-fg-subtle" : undefined} />
        </Field>
        <Field label="Mã cơ quan lưu trữ">
          <Input value={orgCode} onChange={(e) => setOrgCode(e.target.value)} />
        </Field>
        <Field label="Tên cơ quan lưu trữ">
          <Input value={orgName} onChange={(e) => setOrgName(e.target.value)} placeholder="Lưu trữ lịch sử tỉnh …" />
        </Field>
        {kind.startsWith("sip") ? (
          <Field label="Mã yêu cầu nộp" hint="Thoả thuận nộp lưu, bắt buộc với SIP">
            <Input value={agreement} onChange={(e) => setAgreement(e.target.value)} />
          </Field>
        ) : (
          <Field label="Thời hạn bảo quản" hint={mucLuc.size ? "Chỉ dùng cho hồ sơ không có dòng mục lục; có thì lấy THBQ trong mục lục" : "Áp cho mọi hồ sơ trong lần này"}>
            <Select value={maintenance} onChange={setMaintenance} options={THBQ} />
          </Field>
        )}
        {mucLuc.size > 0 && (
          <Field label="Hồ sơ không có dòng mục lục" hint={`${n(soCoMl)}/${n(rows.length)} hồ sơ đã chọn có dòng mục lục`}>
            <Select value={chiCoMl ? "bo" : "goi"} onChange={(v) => setChiCoMl(v === "bo")} options={[{ value: "bo", label: "Bỏ qua, không đóng gói" }, { value: "goi", label: "Vẫn đóng gói, tiêu đề theo tên thư mục" }]} />
          </Field>
        )}
      </FormGrid>
      {kind === "dip" && (
        <Card className="mt-3" title="Thông tin cấp bản sao (DIP)" padding="sm">
          <FormGrid cols={3}>
            {(
              [
                ["requestId", "Mã yêu cầu khai thác"],
                ["requestDate", "Ngày yêu cầu"],
                ["certMaXacThuc", "Mã xác thực lưu trữ"],
                ["certTenLuuTru", "Tên lưu trữ lịch sử"],
                ["certNguoiNhanHoTen", "Họ tên người nhận"],
                ["certNguoiNhanDinhDanh", "Mã định danh người nhận"],
                ["certTenLoai", "Tên loại tài liệu"],
                ["certMucDich", "Mục đích sử dụng"],
              ] as const
            ).map(([k, label]) => (
              <Field key={k} label={label}>
                <Input type={k === "requestDate" ? "date" : "text"} value={(dip[k] as string | undefined) ?? ""} onChange={(e) => setDip((d) => ({ ...d, [k]: e.target.value }))} />
              </Field>
            ))}
          </FormGrid>
        </Card>
      )}
      <div className="mt-3">
        <Table columns={cols} rows={rows} getRowId={idOf} maxHeight={300} dense />
      </div>
      <p className="mt-2 text-xs text-fg-subtle">
        {excelFond?.code ? <>Mã hồ sơ trong gói lấy đúng <b>Mã hồ sơ</b> trong Excel metadata; tiêu đề, thời gian, số tờ, THBQ cũng từ đó.</> : mucLuc.size ? <>Mã hồ sơ trong gói ghi theo <b>mã phông.số hộp.số hồ sơ</b>; tiêu đề, thời gian, số tờ, THBQ lấy từ dòng mục lục đã ghép.</> : <>Chưa nhập Excel: mã hồ sơ ghi theo <b>mã phông.số hộp.số hồ sơ</b>, tiêu đề theo tên thư mục, không có thời gian và số tờ.</>}
      </p>
    </Dialog>
  );
}
