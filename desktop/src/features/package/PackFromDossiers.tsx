// Copyright (c) 2026 DigiText. All rights reserved.
// Đóng gói từ bảng hồ sơ: chọn loại gói (SIP / AIP hồ sơ hay tài liệu, DIP) → ghép hồ sơ với tài liệu đã lên máy chủ
// theo phông / hộp / hồ sơ → tạo gói trên máy chủ (mỗi hồ sơ một gói; gói mức tài liệu: mỗi tài liệu một gói).

import { useEffect, useMemo, useState } from "react";
import { V41t } from "@/live/useLive";
import { isTauri } from "@/api/ipc";
import { V4ek, type V4ej } from "@/api/docs";
import { V4ru, V4rv, V4rw, type V4rt } from "@/api/packaging";
import type { V4bm } from "@/api/dossier";
import type { V3yo } from "@/domain/types";
import { V4sk, V4sn, V4su, type V4sj, type V4sm } from "./packageWire";
import { Badge, Button, Card, Dialog, ErrorBanner, Field, FormGrid, IconButton, Input, notifyError, Select, Table, Tip, toast, type Column, errorText } from "@/ui";

const so = (s: string | null | undefined): number | null => {
  const m = /(\d+)/.exec(s ?? "");
  return m ? Number(m[1]) : null;
};
const phongOf = (d: V4ej) => {
  const p = (d.relPath ?? "").split("/").filter((x) => x.trim() !== "");
  return p.length >= 2 ? p[0].trim() : "";
};
function hopHoSo(d: V4ej): [number | null, number | null] {
  const ref = d.hoSoRef ?? "";
  const i = ref.indexOf("/");
  if (i > 0) return [so(ref.slice(0, i)), so(ref.slice(i + 1))];
  const p = (d.relPath ?? "").split("/").filter((x) => x.trim() !== "");
  return p.length >= 3 ? [so(p[1]), so(p[2])] : [null, null];
}
/** Tên THBQ → mã thời hạn bảo quản (TT02): 01 vĩnh viễn, 02 70 năm … 07 khác. */
function maintenanceOf(tenThbq: string): string {
  const t = tenThbq.toLowerCase();
  if (t.includes("vĩnh viễn")) return "01";
  for (const [nam, ma] of [["70", "02"], ["50", "03"], ["30", "04"], ["20", "05"], ["10", "06"]] as const) if (t.includes(nam)) return ma;
  return "07";
}
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
const KINDS: V4sj[] = ["sip_hoso", "sip_tailieu", "aip_hoso", "aip_tailieu", "dip"];
interface KetQua {
  hoSo: string;
  ok: number;
  loi: string[];
}

export function PackDialog({ dossiers, thbqs, onClose }: { dossiers: V4bm[]; thbqs: V3yo[]; onClose: () => void }) {
  const [docs, setDocs] = useState<V4ej[] | null>(null);
  const [docErr, setDocErr] = useState<unknown>(null);
  const [kind, setKind] = useState<V4sj>("aip_hoso");
  const [projectId, setProjectId] = useState(() => nho("pack.projectId"));
  const [phong, setPhong] = useState("");
  const [orgCode, setOrgCode] = useState(() => nho("pack.archiveOrgCode"));
  const [orgName, setOrgName] = useState(() => nho("pack.archiveOrgName"));
  const [fondCode, setFondCode] = useState(() => dossiers[0]?.maDa ?? "");
  const [fondName, setFondName] = useState(() => nho("pack.fondName"));
  const [agreement, setAgreement] = useState("");
  const [dip, setDip] = useState<Partial<V4sm>>({ requestDate: new Date().toISOString().slice(0, 10) });
  const [busy, setBusy] = useState(false);
  const [ketQua, setKetQua] = useState<KetQua[] | null>(null);

  useEffect(() => {
    if (!isTauri()) return;
    V4ek().then(setDocs).catch(setDocErr);
  }, []);
  const duAn = useMemo(() => {
    const m = new Map<string, string>();
    for (const d of docs ?? []) if (d.projectId) m.set(d.projectId, `${d.projectCode ?? ""} · ${d.projectName ?? ""}`);
    return [...m.entries()];
  }, [docs]);
  useEffect(() => {
    if (duAn.length && !duAn.some(([id]) => id === projectId)) setProjectId(duAn[0][0]);
  }, [duAn, projectId]);
  const dsPhong = useMemo(() => {
    const s = new Set<string>();
    for (const d of docs ?? []) if (d.projectId === projectId && phongOf(d)) s.add(phongOf(d));
    return [...s].sort(new Intl.Collator("vi", { numeric: true }).compare);
  }, [docs, projectId]);
  useEffect(() => {
    if (!dsPhong.includes(phong)) {
      const ma = (dossiers[0]?.maDa ?? "").toUpperCase();
      setPhong(dsPhong.find((p) => ma && p.toUpperCase().includes(ma)) ?? dsPhong[0] ?? "");
    }
  }, [dsPhong, phong, dossiers]);
  const ghep = useMemo(() => {
    const trongPhong = (docs ?? []).filter((d) => d.projectId === projectId && (dsPhong.length === 0 || phongOf(d) === phong));
    return dossiers.map((r) => {
      const hop = so(r.hopSo);
      const hs = so(r.soTam);
      const ds = trongPhong
        .filter((d) => {
          const [h, s] = hopHoSo(d);
          return h != null && s != null && h === hop && s === hs;
        })
        .sort((a, b) => a.filename.localeCompare(b.filename, "vi", { numeric: true }));
      return { r, ds };
    });
  }, [docs, dossiers, projectId, phong, dsPhong.length]);
  const tenThbq = (id?: number) => thbqs.find((t) => t.thbqid === id)?.tenThbq ?? "";

  const chay = async () => {
    setBusy(true);
    luu("pack.projectId", projectId);
    luu("pack.archiveOrgCode", orgCode.trim());
    luu("pack.archiveOrgName", orgName.trim());
    luu("pack.fondName", fondName.trim());
    const out: KetQua[] = [];
    for (const { r, ds } of ghep) {
      const kq: KetQua = { hoSo: `Hộp ${r.hopSo ?? "—"} · HS ${r.soTam}`, ok: 0, loi: [] };
      out.push(kq);
      if (ds.length === 0) {
        kq.loi.push("không có tài liệu nào trên máy chủ khớp phông, hộp, hồ sơ này");
        continue;
      }
      const base: V4sm = {
        ...V4sn,
        ...(kind === "dip" ? dip : {}),
        kind,
        fondCode: fondCode.trim(),
        archiveOrgCode: orgCode.trim(),
        fondName: fondName.trim(),
        archiveOrgName: orgName.trim(),
        fileCode: (r.maHoSo ?? "").trim() || `${fondCode.trim()}.${r.hopSo ?? ""}.${r.soTam}`,
        title: r.tieuDe,
        maintenance: kind.startsWith("sip") ? "01" : maintenanceOf(tenThbq(r.thbq)),
        numberOfPaper: r.soTo ?? "",
        numberOfPage: String(ds.reduce((a, d) => a + (d.pageCount ?? 0), 0) || ""),
        paperFileCode: (r.maHoSo ?? "").trim() || `${r.hopSo ?? ""}.${r.soTam}`,
        startDate: r.ngayBatDau ?? "",
        endDate: r.ngayKetThuc ?? "",
        submissionAgreement: agreement,
        certTrichYeu: dip.certTrichYeu || r.tieuDe,
        certMaLuuTruGoc: dip.certMaLuuTruGoc || (r.maHoSo ?? ""),
      };
      const nhom = kind === "aip_tailieu" || kind === "sip_tailieu" ? ds.map((d) => [d]) : [ds];
      for (const [i, g] of nhom.entries()) {
        const form: V4sm = nhom.length > 1 ? { ...base, fileCode: `${base.fileCode}.${String(i + 1).padStart(3, "0")}`, title: g[0].filename } : base;
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
      setKetQua([...out]);
    }
    setKetQua(out);
    setBusy(false);
    const tong = out.reduce((a, k) => a + k.ok, 0);
    const loi = out.filter((k) => k.loi.length > 0).length;
    if (tong > 0) toast.success(`Đã tạo ${tong} gói. Máy chủ đang dựng, theo dõi ở tab Gói tin.`, { description: loi ? `${loi} hồ sơ có lỗi, xem cột Kết quả.` : undefined });
    else toast.error("Không tạo được gói nào. Xem cột Kết quả.");
  };

  const khop = ghep.filter((g) => g.ds.length > 0).length;
  const cols: Column<{ r: V4bm; ds: V4ej[] }>[] = [
    { key: "hopSo", header: "Hộp", align: "right", width: 70, cell: (g) => g.r.hopSo },
    { key: "soTam", header: "HS số", align: "right", width: 80, cell: (g) => g.r.soTam },
    { key: "tieuDe", header: "Tiêu đề", className: "whitespace-normal", cell: (g) => g.r.tieuDe },
    { key: "ds", header: "Tài liệu trên máy chủ", width: 170, cell: (g) => (docs == null ? "…" : g.ds.length ? <Badge tone="ok">{g.ds.length} tài liệu</Badge> : <Badge tone="err">Không khớp</Badge>) },
    ...(ketQua
      ? [
          {
            key: "kq",
            header: "Kết quả",
            className: "whitespace-normal text-sm",
            cell: (g: { r: V4bm; ds: V4ej[] }) => {
              const kq = ketQua[ghep.indexOf(g)];
              if (!kq) return "…";
              return (
                <>
                  {kq.ok > 0 && <span className="text-success-700">Đã tạo {kq.ok} gói. </span>}
                  {kq.loi.length > 0 && <span className="text-danger-700">{kq.loi.join(" · ")}</span>}
                </>
              );
            },
          } as Column<{ r: V4bm; ds: V4ej[] }>,
        ]
      : []),
  ];

  return (
    <Dialog
      open
      onOpenChange={(o) => !o && !busy && onClose()}
      closable={!busy}
      title={`Đóng gói ${dossiers.length} hồ sơ`}
      size="xl"
      footer={
        <>
          <span className="self-center text-sm text-fg-subtle">{docs ? `${khop}/${dossiers.length} hồ sơ khớp tài liệu trên máy chủ.` : "Đang tải danh sách tài liệu máy chủ…"}</span>
          <span className="flex-1" />
          <Button onClick={onClose} disabled={busy}>
            {ketQua ? "Đóng" : "Huỷ"}
          </Button>
          <Button variant="primary" icon="goi" loading={busy} disabled={khop === 0 || !docs} onClick={() => void chay()}>
            Đóng gói {V4sk[kind]}
          </Button>
        </>
      }
    >
      {docErr !== null && <ErrorBanner error={docErr} className="mb-3" />}
      <FormGrid cols={3}>
        <Field label="Loại gói">
          <Select value={kind} onChange={(v) => setKind(v as V4sj)} options={KINDS.map((k) => ({ value: k, label: V4sk[k] }))} />
        </Field>
        <Field label="Dự án trên máy chủ">
          <Select value={projectId} onChange={setProjectId} options={duAn.map(([id, ten]) => ({ value: id, label: ten }))} />
        </Field>
        {dsPhong.length > 0 && (
          <Field label="Phông trên máy chủ">
            <Select value={phong} onChange={setPhong} options={dsPhong.map((p) => ({ value: p, label: p }))} />
          </Field>
        )}
        <Field label="Mã phông">
          <Input value={fondCode} onChange={(e) => setFondCode(e.target.value)} />
        </Field>
        <Field label="Mã cơ quan lưu trữ">
          <Input value={orgCode} onChange={(e) => setOrgCode(e.target.value)} />
        </Field>
        <Field label="Tên cơ quan hình thành phông" hint="Ghi vào phần cơ quan tạo lập của gói">
          <Input value={fondName} onChange={(e) => setFondName(e.target.value)} placeholder="UBND xã …" />
        </Field>
        <Field label="Tên cơ quan lưu trữ" hint="Ghi vào phần cơ quan lưu trữ của gói">
          <Input value={orgName} onChange={(e) => setOrgName(e.target.value)} placeholder="Lưu trữ lịch sử tỉnh …" />
        </Field>
        {kind.startsWith("sip") && (
          <Field label="Mã yêu cầu nộp" hint="Thoả thuận nộp lưu, nếu có">
            <Input value={agreement} onChange={(e) => setAgreement(e.target.value)} />
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
        <Table columns={cols} rows={ghep} getRowId={(g) => String(g.r.id)} maxHeight={300} dense state={{ loading: docs == null && !docErr }} />
      </div>
    </Dialog>
  );
}

const TRANG_THAI: Record<string, { label: string; tone: "gray" | "info" | "ok" | "err" }> = {
  queued: { label: "Chờ dựng", tone: "gray" },
  building: { label: "Đang dựng", tone: "info" },
  built: { label: "Xong", tone: "ok" },
  failed: { label: "Lỗi", tone: "err" },
};

/** Danh sách gói tin trên máy chủ + tải về. */
export function PackageList() {
  const { data, error, loading, refreshing, reload } = V41t(() => V4ru(), { topics: ["docs", "packages"], pollMs: 10_000, enabled: isTauri() });
  const [q, setQ] = useState("");
  const [statusF, setStatusF] = useState("");
  const list = useMemo(() => {
    const k = q.trim().toLowerCase();
    return (data ?? []).filter((p) => (!statusF || p.status === statusF) && (!k || p.label.toLowerCase().includes(k) || (p.zipName ?? "").toLowerCase().includes(k)));
  }, [data, q, statusF]);
  const tai = async (p: V4rt) => {
    try {
      const path = await V4rw(p.id, p.zipName ?? `${p.label}.zip`);
      toast.success("Đã tải gói về.", { description: path });
    } catch (e) {
      notifyError(e, "tải gói");
    }
  };
  const cols: Column<V4rt>[] = [
    { key: "label", header: "Nhãn", className: "whitespace-normal", sortable: true, cell: (p) => <Tip text={p.objid ?? ""}><span>{p.label}</span></Tip> },
    { key: "kind", header: "Loại", width: 150, sortable: true, cell: (p) => V4sk[p.kind as V4sj] ?? p.kind },
    { key: "status", header: "Trạng thái", width: 120, cell: (p) => { const t = TRANG_THAI[p.status] ?? { label: p.status, tone: "gray" as const }; return <Tip text={p.error ?? ""}><span><Badge tone={t.tone}>{t.label}</Badge></span></Tip>; } },
    { key: "documentCount", header: "Tài liệu", align: "right", width: 90, sortable: true },
    { key: "zipSize", header: "Kích thước", align: "right", width: 100, sortable: true, cell: (p) => (p.zipSize != null ? `${(p.zipSize / 1048576).toFixed(1)} MB` : "—") },
    { key: "createdAt", header: "Tạo lúc", width: 150, sortable: true, cell: (p) => new Date(p.createdAt).toLocaleString("vi-VN") },
  ];
  return (
    <Card padding="none">
      <div className="flex flex-wrap items-center gap-2 border-b border-border-subtle p-3">
        <Input size="sm" prefix="tim" placeholder="Nhãn, tên tệp" value={q} onChange={(e) => setQ(e.target.value)} className="w-60" aria-label="Tìm gói" />
        <Select size="sm" value={statusF} onChange={setStatusF} placeholder="Mọi trạng thái" options={Object.entries(TRANG_THAI).map(([k, v]) => ({ value: k, label: v.label }))} className="w-40" aria-label="Lọc trạng thái" />
        <span className="flex-1" />
        <span className="text-sm text-fg-subtle">{list.length.toLocaleString("vi-VN")} gói</span>
        <IconButton label="Tải lại" icon="lamMoi" size="sm" variant="secondary" loading={refreshing} onClick={reload} />
      </div>
      <Table
        columns={cols}
        rows={list}
        getRowId={(p) => p.id}
        state={{ loading: loading && !data, error: data ? undefined : error, onRetry: reload, empty: { title: "Chưa có gói tin", description: "Chọn hồ sơ ở tab Từ hồ sơ rồi bấm Đóng gói." } }}
        rowActions={(p) => (p.status === "built" && p.zipName ? <Button size="sm" icon="taiVe" onClick={() => void tai(p)}>Tải về</Button> : p.status === "failed" && p.error ? <Tip text={p.error}><span className="text-xs text-danger-700">Lỗi</span></Tip> : null)}
        maxHeight="calc(100dvh - 340px)"
      />
    </Card>
  );
}
