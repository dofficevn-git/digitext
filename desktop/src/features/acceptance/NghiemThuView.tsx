// Copyright (c) 2026 DigiText. All rights reserved.
// Nghiệm thu, bàn giao (Bước 8 Điều 9 TT 05/2025, 3/11/2026): khối lượng theo phông = mục lục chỉnh lý (hộp, hồ sơ, tờ) ⨝
// kho máy chủ (tệp, trang, PDF/A, ký số) so với hợp đồng (hộp/hồ sơ/tờ/trang/tệp, nhập ở Báo cáo › Tiến độ dự án); gói AIP
// của dự án; sổ biên bản nghiệm thu / bàn giao / kiểm tra với số liệu điền sẵn; "Tạo bộ hồ sơ nghiệm thu" ghi vào một thư mục:
// Excel khối lượng, PDF (và Word) mọi biên bản của dự án, mục lục TT05 từng phông, gói AIP đã dựng kèm báo cáo kiểm tra gói
// (TT05 + E-ARK CSIP, 4/11/2026), rồi nén cả thư mục thành .zip. Phông ↔ thư mục máy chủ ghép theo tên (bỏ dấu), sửa tay
// được ở cột Thư mục máy chủ.

import { useCallback, useMemo, useState } from "react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { V41t } from "@/live/useLive";
import { isTauri } from "@/api/ipc";
import { V460 } from "@/state";
import { useProject } from "@/state/project";
import { clCatalogList, clHoSoList, clRows, str, type ClCatalog, type ClRow } from "@/api/chinhly";
import { docsReviewSummary } from "@/api/docs";
import { reportProjectProgress, type ProjectProgress } from "@/api/progressReport";
import { V4ru, V4rw } from "@/api/packaging";
import { recordsAll, V4t1 } from "@/api/records";
import { pkgcheckHeaderDaNho, pkgcheckReport, pkgcheckRun, type PkgResult } from "@/api/pkgcheck";
import { zipDir } from "@/api/bundle";
import { writeXlsx } from "@/features/chinhly/excel";
import { xuatTT05 } from "@/features/chinhly/excelXuat";
import { PackageList } from "@/features/package/PackFromDossiers";
import { SoBienBan, TEN_LOAI, type NguonDien, type NoiDungDien } from "@/features/records/RecordsView";
import type { V4ta } from "@/features/records/recordForms";
import { Badge, Button, Card, Checkbox, Dialog, EmptyState, Help, KpiRow, KpiTile, notifyError, PageHeader, Select, Table, toast, type Column } from "@/ui";

const n = (v: number | null | undefined) => (v == null ? "—" : v.toLocaleString("vi-VN"));
const pct = (a: number, b: number | null | undefined) => (b && b > 0 ? `${Math.min(999, Math.round((a / b) * 100))}%` : "");
const soTo = (s: string) => {
  const m = /\d+/.exec(s);
  return m ? Number(m[0]) : 0;
};
/** Bỏ dấu, thường hoá, gộp ký tự lạ thành khoảng trắng — để ghép tên phông với tên thư mục trên máy chủ. */
const chuan = (s: string) =>
  s
    .normalize("NFD")
    .replace(/[̀-ͯ]/g, "")
    .replace(/đ/gi, "d")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, " ")
    .trim();
const tenTep = (s: string) => s.replace(/[\\/:*?"<>|]+/g, "-").trim() || "tep";
const KIND_BO_HO_SO: V4ta[] = ["nghiem_thu", "ban_giao_lan", "kiem_tra_san_pham", "giao_nhan_giay", "tra_tai_lieu_giay", "nhat_ky_so_hoa"];

interface MayChu {
  files: number;
  pages: number;
  pdfa: number;
  signed: number;
  hop: Set<string>;
  hoSo: Set<string>;
}
interface Mlhs {
  hoSo: number;
  hop: number;
  to: number;
}
interface Dong {
  phong: ClRow;
  ma: string;
  ten: string;
  thuMuc: string;
  mlhs: Mlhs | null;
  mc: MayChu | null;
}

function ghepThuMuc(phong: ClRow, thuMucs: string[]): string {
  const ma = chuan(str(phong, "ma_du_an"));
  const ten = chuan(str(phong, "ten"));
  let best = "";
  let diem = 0;
  for (const t of thuMucs) {
    const ct = chuan(t);
    let d = 0;
    if (ten && ct === ten) d = 100;
    else if (ten && (ct.includes(ten) || ten.includes(ct)) && ct.length >= 6) d = 60 + Math.min(30, ct.length);
    else if (ma.length >= 3 && ct.split(" ").includes(ma)) d = 50;
    if (d > diem) {
      diem = d;
      best = t;
    }
  }
  return best;
}

export default function NghiemThuView() {
  const { session } = V460();
  const proj = useProject();
  const projectId = session?.projectId ?? proj.projectId ?? "";
  const dev = isTauri() || import.meta.env.DEV;
  const [ghiDe, setGhiDe] = useState<Record<string, string>>({});
  const [busy, setBusy] = useState<string | null>(null);
  const [moBo, setMoBo] = useState(() => import.meta.env.DEV && new URLSearchParams(location.search).has("bo"));
  const [tuyChon, setTuyChon] = useState({ goi: true, word: true, zip: true });

  const taiTienDo = useCallback(async (): Promise<ProjectProgress | null> => (projectId ? ((await reportProjectProgress({ projectId })).find((p) => p.projectId === projectId) ?? null) : null), [projectId]);
  const tienDo = V41t(taiTienDo, { topics: ["docs", "projects"], pollMs: 180_000, enabled: dev && !!projectId, errorText: () => "Không tải được hợp đồng / tiến độ." });

  const taiMayChu = useCallback(async () => {
    if (!projectId) return new Map<string, MayChu>();
    const m = new Map<string, MayChu>();
    for (const r of await docsReviewSummary({ projectId })) {
      const g = m.get(r.phong) ?? { files: 0, pages: 0, pdfa: 0, signed: 0, hop: new Set<string>(), hoSo: new Set<string>() };
      g.files += r.files;
      g.pages += r.pages;
      g.pdfa += r.pdfaDocs;
      g.signed += r.signedDocs;
      if (r.hop) g.hop.add(r.hop);
      if (r.hoSo) g.hoSo.add(`${r.hop}/${r.hoSo}`);
      m.set(r.phong, g);
    }
    return m;
  }, [projectId]);
  const mayChu = V41t(taiMayChu, { topics: ["docs"], pollMs: 180_000, enabled: dev && !!projectId, errorText: () => "Không tải được số liệu kho máy chủ." });

  const taiCatalog = useCallback(async () => clCatalogList(), []);
  const catalog = V41t(taiCatalog, { topics: ["chinhLy", "catalog"], enabled: dev, errorText: () => "Không tải được danh mục phông." });
  const phongList = useMemo<ClRow[]>(() => catalog.data?.phong ?? [], [catalog.data]);

  const taiMlhs = useCallback(async () => {
    const m = new Map<string, Mlhs>();
    for (const p of phongList) {
      const rows = (await clHoSoList({ phongId: p.id, limit: 200000 })).rows;
      const hop = new Set<string>();
      let to = 0;
      for (const h of rows) {
        const hs = str(h, "hop_so");
        if (hs) hop.add(`${str(h, "vinh_vien") === "true" ? "V" : "T"}${hs}`);
        to += soTo(str(h, "so_to"));
      }
      m.set(p.id, { hoSo: rows.length, hop: hop.size, to });
    }
    return m;
  }, [phongList]);
  const mlhs = V41t(taiMlhs, { topics: ["chinhLy", "hoso"], enabled: dev && phongList.length > 0, errorText: () => "Không đếm được mục lục hồ sơ." });

  const taiGoi = useCallback(async () => (await V4ru()).filter((p) => p.projectId === projectId), [projectId]);
  const goi = V41t(taiGoi, { topics: ["docs", "packages"], pollMs: 60_000, enabled: dev && !!projectId, errorText: () => "Không tải được danh sách gói." });

  const thuMucs = useMemo(() => [...(mayChu.data?.keys() ?? [])].filter(Boolean).sort(), [mayChu.data]);
  const rows = useMemo<Dong[]>(
    () =>
      phongList.map((p) => {
        const thuMuc = ghiDe[p.id] ?? ghepThuMuc(p, thuMucs);
        return { phong: p, ma: str(p, "ma_du_an"), ten: str(p, "ten"), thuMuc, mlhs: mlhs.data?.get(p.id) ?? null, mc: thuMuc ? (mayChu.data?.get(thuMuc) ?? null) : null };
      }),
    [phongList, thuMucs, ghiDe, mlhs.data, mayChu.data],
  );
  const tong = useMemo(() => {
    const t = { hoSo: 0, hop: 0, to: 0, files: 0, pages: 0, pdfa: 0, signed: 0 };
    for (const r of rows) {
      if (r.mlhs) {
        t.hoSo += r.mlhs.hoSo;
        t.hop += r.mlhs.hop;
        t.to += r.mlhs.to;
      }
      if (r.mc) {
        t.files += r.mc.files;
        t.pages += r.mc.pages;
        t.pdfa += r.mc.pdfa;
        t.signed += r.mc.signed;
      }
    }
    return t;
  }, [rows]);
  const hd = tienDo.data?.contract ?? null;
  const goiXong = useMemo(() => (goi.data ?? []).filter((p) => p.status === "built"), [goi.data]);
  const goiTaiLieu = useMemo(() => goiXong.reduce((s, p) => s + (p.documentCount || 0), 0), [goiXong]);
  const dangTai = (tienDo.loading && !tienDo.data) || (mayChu.loading && !mayChu.data) || (mlhs.loading && !mlhs.data);

  const cols: Column<Dong>[] = [
    { key: "ma", header: "Mã phông", width: 110, sortable: true },
    { key: "ten", header: "Tên phông", cell: (r) => <span className="line-clamp-2">{r.ten}</span> },
    {
      key: "thuMuc",
      header: "Thư mục máy chủ",
      width: 230,
      cell: (r) => (
        <Select size="sm" value={r.thuMuc} onChange={(v) => setGhiDe((g) => ({ ...g, [r.phong.id]: v }))} options={thuMucs.map((t) => ({ value: t, label: t }))} placeholder="— chưa ghép —" aria-label={`Thư mục máy chủ của phông ${r.ma}`} />
      ),
    },
    { key: "hop", header: "Hộp", align: "right", width: 70, cell: (r) => n(r.mlhs?.hop) },
    { key: "hoSo", header: "Hồ sơ", align: "right", width: 80, cell: (r) => n(r.mlhs?.hoSo) },
    { key: "to", header: "Tờ (MLHS)", align: "right", width: 90, cell: (r) => n(r.mlhs?.to) },
    { key: "files", header: "Tệp quét", align: "right", width: 90, cell: (r) => (r.mc ? n(r.mc.files) : "") },
    { key: "pages", header: "Trang quét", align: "right", width: 95, cell: (r) => (r.mc ? n(r.mc.pages) : "") },
    { key: "pdfa", header: "PDF/A", align: "right", width: 80, cell: (r) => (r.mc ? `${n(r.mc.pdfa)}` : "") },
    { key: "signed", header: "Ký số", align: "right", width: 80, cell: (r) => (r.mc ? `${n(r.mc.signed)}` : "") },
    {
      key: "tt",
      header: "Hoàn thành",
      width: 120,
      cell: (r) => {
        if (!r.mc || !r.mlhs) return <Badge tone="gray">Chưa đủ dữ liệu</Badge>;
        const kyXong = r.mc.files > 0 && r.mc.signed >= r.mc.files;
        const duHoSo = r.mc.hoSo.size >= r.mlhs.hoSo;
        return kyXong && duHoSo ? <Badge tone="ok">Đủ, đã ký</Badge> : <Badge tone="warn">{pct(r.mc.signed, r.mc.files) || "0%"} ký · {n(r.mc.hoSo.size)}/{n(r.mlhs.hoSo)} hồ sơ</Badge>;
      },
    },
  ];

  const sheetKhoiLuong = () => {
    const aoa: (string | number)[][] = [["Mã phông", "Tên phông", "Thư mục máy chủ", "Hộp (MLHS)", "Hồ sơ (MLHS)", "Tờ (MLHS)", "Tệp quét", "Trang quét", "Tệp PDF/A", "Tệp ký số", "Hồ sơ có tệp"]];
    for (const r of rows) aoa.push([r.ma, r.ten, r.thuMuc, r.mlhs?.hop ?? "", r.mlhs?.hoSo ?? "", r.mlhs?.to ?? "", r.mc?.files ?? "", r.mc?.pages ?? "", r.mc?.pdfa ?? "", r.mc?.signed ?? "", r.mc?.hoSo.size ?? ""]);
    aoa.push(["TỔNG", "", "", tong.hop, tong.hoSo, tong.to, tong.files, tong.pages, tong.pdfa, tong.signed, ""]);
    aoa.push(["HỢP ĐỒNG", "", "", hd?.contractBoxes ?? "", hd?.contractDossiers ?? "", hd?.contractSheets ?? "", hd?.contractFiles ?? "", hd?.contractPages ?? "", "", "", ""]);
    aoa.push(["% so hợp đồng", "", "", pct(tong.hop, hd?.contractBoxes), pct(tong.hoSo, hd?.contractDossiers), pct(tong.to, hd?.contractSheets), pct(tong.files, hd?.contractFiles), pct(tong.pages, hd?.contractPages), "", "", ""]);
    aoa.push([]);
    aoa.push(["Gói AIP đã dựng", goiXong.length, "Tài liệu trong gói", goiTaiLieu]);
    return { name: "Khối lượng", aoa, cols: [12, 40, 30, 10, 12, 12, 10, 12, 12, 12, 14].map((wch) => ({ wch })) };
  };
  const xuatExcel = () => void writeXlsx([sheetKhoiLuong()], `khoi-luong-nghiem-thu-${projectId.slice(0, 8)}.xlsx`).catch((e) => notifyError(e, "xuất Excel"));

  const nguonDien: NguonDien = useMemo(
    () => ({
      label: "Phạm vi",
      options: [{ value: "", label: "Toàn dự án" }, ...rows.map((r) => ({ value: r.phong.id, label: `${r.ma || "—"} · ${r.ten}` }))],
      dien: async (kind, value): Promise<NoiDungDien> => {
        const chon = value ? rows.filter((r) => r.phong.id === value) : rows;
        const t = { hoSo: 0, hop: 0, to: 0, files: 0, pages: 0, pdfa: 0, signed: 0 };
        for (const r of chon) {
          if (r.mlhs) {
            t.hoSo += r.mlhs.hoSo;
            t.hop += r.mlhs.hop;
            t.to += r.mlhs.to;
          }
          if (r.mc) {
            t.files += r.mc.files;
            t.pages += r.mc.pages;
            t.pdfa += r.mc.pdfa;
            t.signed += r.mc.signed;
          }
        }
        const khoiLuong = chon.map((r) => ({ maPhong: r.ma, tenPhong: r.ten, hop: String(r.mlhs?.hop ?? ""), hoSo: String(r.mlhs?.hoSo ?? ""), to: String(r.mlhs?.to ?? ""), trang: String(r.mc?.pages ?? ""), tep: String(r.mc?.files ?? ""), pdfa: String(r.mc?.pdfa ?? ""), ky: String(r.mc?.signed ?? "") }));
        if (kind === "ban_giao_lan") {
          const daCo = (await recordsAll({ kinds: ["ban_giao_lan"], projectId }).catch(() => [])).length;
          return {
            fields: { duAn: proj.projectLabel || projectId, lan: `Lần ${daCo + 1}`, soGoi: String(goiXong.length), soTaiLieu: String(goiTaiLieu), kemTheo: "Mục lục hồ sơ TT05 (Excel), biên bản kiểm tra sản phẩm, biên bản nghiệm thu" },
            lists: { goiTin: goiXong.slice(0, 200).map((p) => ({ nhan: p.label, taiLieu: String(p.documentCount), tep: p.zipName ?? "", sha: (p.zipSha256 ?? "").slice(0, 8), kichThuoc: p.zipSize != null ? `${(p.zipSize / 1048576).toFixed(1)} MB` : "" })) },
          };
        }
        const kiemTra = await recordsAll({ kinds: ["kiem_tra_san_pham"], projectId }).catch(() => []);
        const duDu = hd?.contractDossiers ? t.hoSo >= hd.contractDossiers : true;
        return {
          fields: {
            duAn: proj.projectLabel || projectId,
            phamVi: value ? `${chon[0]?.ma ?? ""} · ${chon[0]?.ten ?? ""}` : "Toàn dự án",
            hopHd: hd?.contractBoxes != null && !value ? String(hd.contractBoxes) : "",
            hoSoHd: hd?.contractDossiers != null && !value ? String(hd.contractDossiers) : "",
            toHd: hd?.contractSheets != null && !value ? String(hd.contractSheets) : "",
            trangHd: hd?.contractPages != null && !value ? String(hd.contractPages) : "",
            hopTt: String(t.hop),
            hoSoTt: String(t.hoSo),
            toTt: String(t.to),
            trangTt: String(t.pages),
            tepTt: String(t.files),
            tepPdfa: String(t.pdfa),
            tepKy: String(t.signed),
            goiAip: value ? "" : String(goiXong.length),
            bienBanKiemTra: kiemTra.map((r) => r.soBienBan).join(", "),
            ketLuan: t.files > 0 && t.signed >= t.files && duDu ? "Sản phẩm đạt yêu cầu Điều 8 TT 05/2025, đủ khối lượng, đồng ý nghiệm thu" : `Còn ${n(Math.max(0, t.files - t.signed))} tệp chưa ký số${duDu ? "" : ", chưa đủ hồ sơ so với hợp đồng"}; nghiệm thu phần đã hoàn thành`,
          },
          lists: { khoiLuong },
        };
      },
    }),
    [rows, hd, goiXong, goiTaiLieu, projectId, proj.projectLabel],
  );

  const goiCoTep = useMemo(() => goiXong.filter((p) => p.zipName), [goiXong]);
  const goiDungLuong = useMemo(() => goiCoTep.reduce((s, p) => s + (p.zipSize ?? 0), 0), [goiCoTep]);

  /** Bộ hồ sơ nghiệm thu, ghi vào `<thư mục chọn>\Ho-so-nghiem-thu-<ngày>`:
   *  01 Excel khối lượng (+ trang kết quả kiểm gói) · 02 biên bản PDF (+ Word) · 03 mục lục TT05 từng phông ·
   *  04 gói AIP đã dựng · 05 báo cáo kiểm tra gói PDF; tuỳ chọn nén cả thư mục thành .zip cạnh nó. */
  const taoBoHoSo = async () => {
    if (!isTauri()) {
      toast.warning("Chỉ tạo được trong ứng dụng máy tính.");
      return;
    }
    let goc: string;
    try {
      const r = await openDialog({ directory: true, multiple: false, title: "Chọn thư mục chứa bộ hồ sơ nghiệm thu" });
      if (typeof r !== "string" || !r) return;
      goc = r;
    } catch (e) {
      notifyError(e, "mở hộp chọn thư mục");
      return;
    }
    setMoBo(false);
    const opt = { ...tuyChon, goi: tuyChon.goi && goiCoTep.length > 0 };
    const ngay = new Date().toISOString().slice(0, 10);
    const dir = `${goc}\\Ho-so-nghiem-thu-${ngay}`;
    setBusy("Đang tạo bộ hồ sơ…");
    let tid = toast.info("Bộ hồ sơ nghiệm thu: đang đọc sổ biên bản…");
    const bao = (m: string) => {
      setBusy(m);
      toast.dismiss(tid);
      tid = toast.info(`Bộ hồ sơ nghiệm thu: ${m}`);
    };
    const loi: string[] = [];
    const msg = (e: unknown) => (e instanceof Error ? e.message : String(e));
    let soTep = 0;
    let kiemGoi: PkgResult[] = [];
    let zipPath = "";
    try {
      // 02 — biên bản
      const bbs = await recordsAll({ kinds: KIND_BO_HO_SO, projectId });
      for (const [i, r] of bbs.entries()) {
        bao(`biên bản ${i + 1}/${bbs.length}…`);
        const ten = `${tenTep(r.soBienBan)} - ${tenTep(TEN_LOAI[r.kind] ?? r.kind)}`;
        try {
          await V4t1(r.id, `${ten}.pdf`, `${dir}\\02-Bien-ban`);
          soTep += 1;
        } catch (e) {
          loi.push(`biên bản ${r.soBienBan}: ${msg(e)}`);
        }
        if (opt.word) {
          try {
            await V4t1(r.id, `${ten}.docx`, `${dir}\\02-Bien-ban\\Word`, "docx");
            soTep += 1;
          } catch (e) {
            loi.push(`Word ${r.soBienBan}: ${msg(e)}`);
          }
        }
      }
      // 03 — mục lục TT05 từng phông
      const cat: ClCatalog | undefined = catalog.data;
      const loai = new Map((cat?.loai_van_ban ?? []).map((l) => [str(l, "ma"), str(l, "ten")]));
      const layHet = async (table: "van_ban" | "van_ban_file", order: string): Promise<ClRow[]> => {
        const out: ClRow[] = [];
        const LO = 100000;
        for (let offset = 0; ; offset += LO) {
          const r = await clRows(table, { order, limit: LO, offset });
          out.push(...r.rows);
          if (r.rows.length < LO) return out;
        }
      };
      const coHoSo = rows.filter((r) => (r.mlhs?.hoSo ?? 0) > 0);
      if (coHoSo.length > 0) {
        bao("đang đọc văn bản của mục lục…");
        const vbAll = await layHet("van_ban", "so_thu_tu");
        const tepAll = await layHet("van_ban_file", "so_tt");
        const tepCua = new Map<string, string>();
        for (const f of tepAll) {
          const vb = str(f, "van_ban_id");
          const ten = str(f, "name");
          if (ten && !tepCua.has(vb)) tepCua.set(vb, ten);
        }
        for (const [i, r] of coHoSo.entries()) {
          bao(`mục lục TT05 phông ${r.ma || r.ten} (${i + 1}/${coHoSo.length})…`);
          try {
            const hoSos = (await clHoSoList({ phongId: r.phong.id, limit: 200000 })).rows;
            const ids = new Set(hoSos.map((h) => h.id));
            const vbs = new Map<string, ClRow[]>();
            for (const v of vbAll) {
              const k = str(v, "ho_so_id");
              if (!ids.has(k)) continue;
              v.ten_loai_vb = loai.get(str(v, "loai_vb")) ?? "";
              if (!vbs.has(k)) vbs.set(k, []);
              vbs.get(k)!.push(v);
            }
            await xuatTT05(hoSos, vbs, r.phong, tepCua, `${dir}\\03-Muc-luc-TT05`);
            soTep += 1;
          } catch (e) {
            loi.push(`mục lục phông ${r.ma}: ${msg(e)}`);
          }
        }
      }
      // 04 + 05 — gói AIP và báo cáo kiểm tra gói
      if (opt.goi) {
        const thuMucGoi = `${dir}\\04-Goi-AIP`;
        let daTai = 0;
        for (const [i, g] of goiCoTep.entries()) {
          bao(`tải gói ${i + 1}/${goiCoTep.length} (${g.label})…`);
          try {
            await V4rw(g.id, g.zipName!, thuMucGoi);
            daTai += 1;
            soTep += 1;
          } catch (e) {
            loi.push(`gói ${g.label}: ${msg(e)}`);
          }
        }
        if (daTai > 0) {
          bao(`kiểm tra ${daTai} gói theo TT05 + E-ARK CSIP…`);
          try {
            kiemGoi = await pkgcheckRun([thuMucGoi]);
            await pkgcheckReport(kiemGoi, pkgcheckHeaderDaNho(), "05-Bao-cao-kiem-tra-goi.pdf", dir);
            soTep += 1;
          } catch (e) {
            loi.push(`kiểm tra gói: ${msg(e)}`);
          }
        }
      }
      // 01 — Excel khối lượng, ghi sau cùng để kèm kết quả kiểm gói
      bao("ghi Excel khối lượng…");
      const sheets = [sheetKhoiLuong()];
      if (kiemGoi.length) {
        const aoa: (string | number)[][] = [["Gói", "Mã OBJID", "Hồ sơ", "Tài liệu", "Trang", "Tệp PDF", "Lỗi", "Cảnh báo", "Kết luận", "SHA-256"]];
        for (const k of kiemGoi) aoa.push([k.name, k.objid, k.fileCode || k.title, k.totalDoc ?? "", k.pages ?? "", k.pdfs.length, k.errors, k.warnings, k.verdict, k.sha256 ?? ""]);
        sheets.push({ name: "Kiểm tra gói", aoa, cols: [34, 30, 30, 10, 10, 10, 8, 10, 22, 66].map((wch) => ({ wch })) });
      }
      await writeXlsx(sheets, "01-Khoi-luong-nghiem-thu.xlsx", dir);
      soTep += 1;
      // nén
      if (opt.zip) {
        bao("nén thư mục thành .zip…");
        try {
          zipPath = await zipDir(dir);
        } catch (e) {
          loi.push(`nén zip: ${msg(e)}`);
        }
      }
      toast.dismiss(tid);
      const datGoi = kiemGoi.filter((k) => k.errors === 0).length;
      const tomTat = [`${n(soTep)} tệp`, kiemGoi.length ? `${datGoi}/${kiemGoi.length} gói không lỗi` : "", zipPath ? "đã nén .zip" : ""].filter(Boolean).join(" · ");
      if (loi.length) toast.warning(`Bộ hồ sơ đã ghi ${tomTat}, ${n(loi.length)} lỗi.`, { description: `${dir} — ${loi.slice(0, 3).join(" · ")}` });
      else toast.success(`Đã tạo bộ hồ sơ nghiệm thu: ${tomTat}.`, { description: zipPath || dir });
    } catch (e) {
      toast.dismiss(tid);
      notifyError(e, "tạo bộ hồ sơ nghiệm thu");
    } finally {
      setBusy(null);
    }
  };

  if (!projectId) {
    return (
      <section className="view active">
        <PageHeader title="Nghiệm thu, bàn giao" description="Bước 8 Điều 9 TT 05/2025: khối lượng so với hợp đồng, gói AIP, biên bản nghiệm thu và bàn giao." />
        <Card padding="md">
          <EmptyState icon="duAn" title="Chọn dự án đang làm ở thanh trên" description="Nghiệm thu tính theo dự án." />
        </Card>
      </section>
    );
  }
  return (
    <section className="view active">
      <PageHeader
        title="Nghiệm thu, bàn giao"
        description={`Bước 8 Điều 9 TT 05/2025: khối lượng theo phông (mục lục chỉnh lý và kho máy chủ) so với hợp đồng, gói AIP đã dựng, biên bản nghiệm thu / bàn giao. Dự án ${proj.projectLabel || ""}.`}
        actions={
          <>
            <Button icon="tepExcel" onClick={xuatExcel} disabled={rows.length === 0}>
              Xuất Excel
            </Button>
            <Button variant="primary" icon="thuMuc" loading={!!busy} onClick={() => setMoBo(true)} disabled={rows.length === 0 || !!busy}>
              {busy ?? "Tạo bộ hồ sơ nghiệm thu…"}
            </Button>
          </>
        }
      />
      <Help title="Số liệu lấy từ đâu" className="mb-3">
        <p className="text-sm">
          <b>Hộp, hồ sơ, tờ</b> đếm từ mục lục chỉnh lý của từng phông (Dữ liệu chủ). <b>Tệp, trang, PDF/A, ký số</b> gộp từ kho máy chủ theo thư mục phông; thư mục ghép tự động theo tên, sửa được ở cột Thư mục máy chủ. <b>Hợp đồng</b> (hộp, hồ sơ, tờ, trang, tệp) nhập ở Báo cáo › Tiến độ dự án › Hợp đồng. <b>Bộ hồ sơ nghiệm thu</b> gồm Excel khối lượng, PDF và Word mọi biên bản của dự án, mục lục TT05 từng phông, gói AIP đã dựng kèm báo cáo kiểm tra gói, ghi vào thư mục anh chọn và nén thành .zip.
        </p>
      </Help>
      <KpiRow className="mb-3">
        <KpiTile label="Hộp" value={tong.hop} sub={hd?.contractBoxes != null ? `${pct(tong.hop, hd.contractBoxes)} của ${n(hd.contractBoxes)} hợp đồng` : "hợp đồng chưa nhập"} loading={dangTai} />
        <KpiTile label="Hồ sơ" value={tong.hoSo} sub={hd?.contractDossiers != null ? `${pct(tong.hoSo, hd.contractDossiers)} của ${n(hd.contractDossiers)} hợp đồng` : "hợp đồng chưa nhập"} loading={dangTai} />
        <KpiTile label="Tờ (mục lục)" value={tong.to} sub={hd?.contractSheets != null ? `${pct(tong.to, hd.contractSheets)} của ${n(hd.contractSheets)} hợp đồng` : "hợp đồng chưa nhập"} loading={dangTai} />
        <KpiTile label="Trang quét" value={tong.pages} sub={hd?.contractPages != null ? `${pct(tong.pages, hd.contractPages)} của ${n(hd.contractPages)} hợp đồng` : `${n(tong.files)} tệp`} loading={dangTai} />
        <KpiTile label="Tệp ký số" value={tong.signed} sub={tong.files ? `${pct(tong.signed, tong.files)} số tệp · PDF/A ${n(tong.pdfa)}` : undefined} tone={tong.files > 0 && tong.signed >= tong.files ? "ok" : tong.files > 0 ? "warn" : undefined} loading={dangTai} />
        <KpiTile label="Gói AIP đã dựng" value={goiXong.length} sub={goiXong.length ? `${n(goiTaiLieu)} tài liệu` : undefined} loading={goi.loading && !goi.data} />
      </KpiRow>
      <Card title="Khối lượng theo phông" padding="none">
        <Table columns={cols} rows={rows} getRowId={(r) => r.phong.id} dense maxHeight={380} state={{ loading: dangTai && rows.length === 0, error: catalog.data ? mayChu.error || mlhs.error || undefined : catalog.error, onRetry: () => { catalog.reload(); mayChu.reload(); mlhs.reload(); tienDo.reload(); }, empty: { title: "Dự án chưa có phông trong Dữ liệu chủ", description: "Nhập mục lục hồ sơ (Dữ liệu chủ › Lập hồ sơ) để có khối lượng." } }} />
      </Card>
      <div className="mt-3">
        <SoBienBan kinds={["nghiem_thu", "ban_giao_lan", "kiem_tra_san_pham"]} projectId={projectId} title="Biên bản nghiệm thu, bàn giao, kiểm tra" description="Số liệu điền sẵn từ bảng khối lượng và danh sách gói; chọn phạm vi toàn dự án hoặc một phông." nguonDien={nguonDien} nutLap="Lập biên bản…" embedded />
      </div>
      <h2 className="mb-2 mt-4 text-lg font-semibold text-fg-strong">Gói AIP của mọi dự án</h2>
      <PackageList />
      <Dialog
        open={moBo}
        onOpenChange={setMoBo}
        title="Tạo bộ hồ sơ nghiệm thu"
        description="Ghi vào thư mục Ho-so-nghiem-thu-<ngày> bên trong thư mục anh chọn."
        size="md"
        footer={
          <>
            <Button onClick={() => setMoBo(false)}>Huỷ</Button>
            <Button variant="primary" icon="thuMuc" onClick={() => void taoBoHoSo()}>
              Chọn thư mục và tạo
            </Button>
          </>
        }
      >
        <ol className="mb-3 list-decimal space-y-0.5 pl-5 text-sm text-fg">
          <li>Excel khối lượng theo phông so với hợp đồng</li>
          <li>PDF mọi biên bản của dự án ({KIND_BO_HO_SO.map((k) => TEN_LOAI[k] ?? k).join(", ").toLowerCase()})</li>
          <li>Mục lục hồ sơ TT05 của {n(rows.filter((r) => (r.mlhs?.hoSo ?? 0) > 0).length)} phông</li>
        </ol>
        <div className="flex flex-col gap-2">
          <Checkbox checked={tuyChon.word} onChange={(v) => setTuyChon((t) => ({ ...t, word: v }))} text="Kèm bản Word (.docx) của từng biên bản để chỉnh sửa, in ký tay" />
          <Checkbox
            checked={tuyChon.goi && goiCoTep.length > 0}
            disabled={goiCoTep.length === 0}
            onChange={(v) => setTuyChon((t) => ({ ...t, goi: v }))}
            text={goiCoTep.length ? `Tải ${n(goiCoTep.length)} gói AIP đã dựng (${(goiDungLuong / 1073741824).toFixed(2)} GB) và kiểm tra theo TT05 + E-ARK CSIP` : "Dự án chưa có gói AIP đã dựng để tải và kiểm tra"}
          />
          <Checkbox checked={tuyChon.zip} onChange={(v) => setTuyChon((t) => ({ ...t, zip: v }))} text="Nén cả thư mục thành tệp .zip để bàn giao" />
        </div>
        <p className="mt-3 text-xs text-fg-muted">Báo cáo kiểm tra gói dùng phần đầu (cơ quan, người kiểm tra, người duyệt) đã nhập lần gần nhất ở Tiện ích › Kiểm tra gói tin.</p>
      </Dialog>
    </section>
  );
}
