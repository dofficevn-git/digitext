// Copyright (c) 2026 DigiText. All rights reserved.
// Đóng gói › Từ hồ sơ (26/10/2026): phông / hộp / hồ sơ lấy từ TÀI LIỆU ĐÃ LÊN MÁY CHỦ của DỰ ÁN ĐANG CHỌN
// (tầng 1 của đường dẫn = phông, đúng quy ước "mỗi thư mục đẩy lên là một phông"). Trước đây tab này dùng danh mục
// hồ sơ lập trên máy quản trị (bảng cục bộ, không phân dự án) nên hiện lẫn phông của dự án khác và rỗng với dự án
// đẩy theo thư mục. Mỗi hồ sơ chọn → một gói trên máy chủ (gói mức tài liệu: mỗi tài liệu một gói).
// Nhập mục lục Excel (26/10/2026): tệp "Mục lục hồ sơ" (Hộp số · Hồ sơ số · Tiêu đề · Thời gian · Số tờ · THBQ, cùng
// mẫu màn Chỉnh lý) ghép với hồ sơ trên máy chủ theo số hộp + số hồ sơ trong phông đã chọn → gói mang tiêu đề, thời
// gian, số tờ, THBQ thật thay vì tên thư mục. Mục lục đã ghép được nhớ theo dự án + phông (localStorage).

import { useEffect, useMemo, useRef, useState } from "react";
import { docMucLucTuBuffer, type DongHoSo } from "@/features/chinhly/mucLucImport";
import { V41t } from "@/live/useLive";
import { isTauri } from "@/api/ipc";
import { docsForGroups, docsReviewSummary, type ReviewSummaryRow, type V4ej } from "@/api/docs";
import { reviewKey, reviewKeyOf } from "@/features/dossier/reviewScope";
import { V4rv } from "@/api/packaging";
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
/** Tên THBQ → mã thời hạn bảo quản TT02: 01 vĩnh viễn, 02 70 năm, 03 50, 04 30, 05 20, 06 10, 07 khác. */
function maintenanceOf(thbq: string): string {
  const t = thbq.toLowerCase();
  if (t.includes("vĩnh viễn") || t.includes("vinh vien")) return "01";
  for (const [nam, ma] of [["70", "02"], ["50", "03"], ["30", "04"], ["20", "05"], ["10", "06"]] as const) if (new RegExp(`\\b${nam}\\b`).test(t)) return ma;
  return "07";
}
/** dd/mm/yyyy → yyyy-mm-dd (TT02 dùng ngày ISO); không đúng dạng thì giữ nguyên. */
function isoNgay(s: string): string {
  const m = /^\s*(\d{1,2})\/(\d{1,2})\/(\d{4})\s*$/.exec(s ?? "");
  return m ? `${m[3]}-${m[2].padStart(2, "0")}-${m[1].padStart(2, "0")}` : (s ?? "").trim();
}
const THBQ_DANH_MUC = ["Vĩnh viễn", "70 năm", "50 năm", "30 năm", "20 năm", "10 năm", "5 năm"];

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
}
export interface MucLucDaNhap {
  ten: string;
  luc: string;
  dongs: DongMucLuc[];
  /** Khoá hộp|hồ sơ bị trùng trong tệp (lấy dòng đầu). */
  trung: string[];
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
  return { ten: cu && cu.ten !== ten ? `${cu.ten} + ${ten}` : ten, luc: new Date().toISOString(), dongs, trung: [...trung] };
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
    const co = new Set(trongPhong.map((r) => khoaHoSo(r.hop, r.hoSo)));
    return mucLuc.dongs.filter((d) => !co.has(khoaHoSo(d.hop, d.hoSo)));
  }, [mucLuc, trongPhong]);
  const nhapMucLuc = async (files: FileList | null) => {
    if (!files?.length || !phong) return;
    setDocMl(true);
    try {
      let ml = mucLuc;
      for (const f of Array.from(files)) {
        const pt = await docMucLucTuBuffer(await f.arrayBuffer(), f.name, THBQ_DANH_MUC, ["Vĩnh viễn"]);
        const dongs = pt.rows.filter((r) => r.hoSoSo).map(rutGon);
        if (dongs.length === 0) {
          toast.error(`${f.name}: không thấy dòng hồ sơ nào`, { detail: pt.vanDe[0]?.noiDung });
          continue;
        }
        ml = gopMucLuc(ml, f.name, dongs);
      }
      if (ml) {
        setMucLuc(ml);
        luuMucLuc(projectId, phong, ml);
        const co = new Set(trongPhong.map((r) => khoaHoSo(r.hop, r.hoSo)));
        const khop = ml.dongs.filter((d) => co.has(khoaHoSo(d.hop, d.hoSo))).length;
        toast.success(`Mục lục ${n(ml.dongs.length)} hồ sơ, khớp ${n(khop)} hồ sơ trên máy chủ.`, { description: ml.trung.length ? `${n(ml.trung.length)} cặp hộp/hồ sơ trùng trong tệp, lấy dòng đầu.` : undefined });
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
              return d ? <span title={`Dòng ${d.dong} · ${mucLuc.ten}`}>{d.tieuDe}</span> : <span className="text-warning-700">Chưa có trong mục lục</span>;
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
            <input ref={fileRef} type="file" accept=".xlsx,.xls,.xlsm" multiple hidden aria-hidden onChange={(e) => { const fs = e.target.files; e.target.value = ""; void nhapMucLuc(fs); }} />
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
          <Badge tone="info">Mục lục: {mucLuc.ten}</Badge>
          <span>
            {n(mucLuc.dongs.length)} dòng · khớp <b>{n(soKhop)}</b>/{n(trongPhong.length)} hồ sơ trên máy chủ
            {dongThieuTep.length ? <> · <button type="button" className="text-fg-link hover:underline" onClick={() => setXemThieu(true)}>{n(dongThieuTep.length)} dòng mục lục không có tệp</button></> : null}
            {mucLuc.trung.length ? ` · ${n(mucLuc.trung.length)} cặp hộp/hồ sơ trùng trong tệp` : ""}
          </span>
          <span className="flex-1" />
          <Button size="sm" variant="ghost" onClick={() => setSel(new Set(loc.filter((r) => mlTheoKhoa.has(khoaHoSo(r.hop, r.hoSo))).map(idOf)))}>
            Chọn hồ sơ có mục lục
          </Button>
          <Button size="sm" variant="ghost" icon="xoa" onClick={boMucLuc}>
            Bỏ mục lục
          </Button>
        </div>
      )}
      {xemThieu && (
        <Dialog open onOpenChange={(o) => !o && setXemThieu(false)} title={`${n(dongThieuTep.length)} dòng mục lục không có tệp trên máy chủ`} description={`Phông ${phong}: hồ sơ ghi trong mục lục nhưng chưa thấy thư mục hộp/hồ sơ tương ứng đã đẩy lên.`} size="lg">
          <Table
            columns={[
              { key: "dong", header: "Dòng", width: 70, align: "right", cell: (d: DongMucLuc) => d.dong },
              { key: "hop", header: "Hộp", width: 80, cell: (d: DongMucLuc) => d.hop },
              { key: "hoSo", header: "Hồ sơ", width: 90, cell: (d: DongMucLuc) => d.hoSo },
              { key: "tieuDe", header: "Tiêu đề", className: "whitespace-normal", cell: (d: DongMucLuc) => d.tieuDe },
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
      {dialog && <PackServerDialog rows={daChon} mucLuc={mlTheoKhoa} projectId={projectId} projectLabel={proj.projectLabel || projectId} onClose={() => setDialog(false)} />}
    </Card>
  );
}

interface KetQua {
  key: string;
  ok: number;
  loi: string[];
}

function PackServerDialog({ rows, mucLuc, projectId, projectLabel, onClose }: { rows: ReviewSummaryRow[]; mucLuc: Map<string, DongMucLuc>; projectId: string; projectLabel: string; onClose: () => void }) {
  const mlCua = (r: ReviewSummaryRow) => mucLuc.get(khoaHoSo(r.hop, r.hoSo));
  const soCoMl = rows.filter((r) => mlCua(r)).length;
  const [chiCoMl, setChiCoMl] = useState(soCoMl > 0 && soCoMl < rows.length);
  const dsPhong = useMemo(() => [...new Set(rows.map((r) => r.phong || CHUA_PHAN_LOAI))], [rows]);
  const motPhong = dsPhong.length === 1 ? dsPhong[0] : "";
  const [kind, setKind] = useState<V4sj>("aip_hoso");
  const [fondCode, setFondCode] = useState(() => (motPhong ? nho(`pack.fondCode.${motPhong}`) || goiYMaPhong(motPhong) : ""));
  const [fondName, setFondName] = useState(() => (motPhong ? nho(`pack.fondName.${motPhong}`) || motPhong : ""));
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
    if (motPhong) {
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
        fileCode: `${fondCode.trim()}.${hopSo}.${hsSo}`,
        title: tieuDe,
        // THBQ: theo dòng mục lục nếu có, không thì theo lựa chọn chung; SIP luôn vĩnh viễn.
        maintenance: kind.startsWith("sip") ? "01" : ml?.thbq ? maintenanceOf(ml.thbq) : maintenance,
        numberOfPaper: ml?.soTo ? String(Number(ml.soTo) || ml.soTo) : "",
        numberOfPage: String(ds.reduce((a, d) => a + (d.pageCount ?? 0), 0) || ""),
        paperFileCode: `${hopSo}.${hsSo}`,
        startDate: ml ? isoNgay(ml.batDau) : "",
        endDate: ml ? isoNgay(ml.ketThuc) : "",
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
        <Field label="Mã phông" hint="Fondcode trong gói; gợi ý từ tên thư mục">
          <Input value={fondCode} onChange={(e) => setFondCode(e.target.value)} />
        </Field>
        <Field label="Tên cơ quan hình thành phông">
          <Input value={fondName} onChange={(e) => setFondName(e.target.value)} placeholder={motPhong || "UBND xã …"} />
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
        Mã hồ sơ trong gói ghi theo <b>mã phông.số hộp.số hồ sơ</b>.{" "}
        {mucLuc.size ? "Tiêu đề, thời gian, số tờ, THBQ lấy từ dòng mục lục đã ghép." : "Chưa nhập mục lục Excel: tiêu đề hồ sơ lấy theo tên thư mục, không có thời gian và số tờ."}
      </p>
    </Dialog>
  );
}
