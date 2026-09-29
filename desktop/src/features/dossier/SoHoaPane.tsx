// Copyright (c) 2026 DigiText. All rights reserved.
// Tài liệu đã quét (hồ sơ trên máy này + PDF quét + OCR / nhập văn bản). mode "donggoi": cùng bảng nhưng chỉ để
// tích chọn và đóng gói (dùng ở màn Đóng gói).

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { V41p } from "@/live/useLive";
import { isTauri } from "@/api/ipc";
import type { V40v } from "@/api/archive";
import { V48j } from "@/api/catalog";
import type { V3yl, V3ym, V3yo } from "@/domain/types";
import { V4br, V4cg, type V4bm } from "@/api/dossier";
import { V460 } from "@/state";
import { chuaChonPhong } from "./phongScope";
import V4dy from "./VanBanPanel";
import V4h8 from "./TaiLieuPanel";
import { V4e3, V4e8 } from "./excel";
import { type V4dk, V4cl, V4cm, V4co, V4ct, V4d0, V4di, V4dp, V4dr, V4dt, V4du } from "./logic";
import { PackDialog } from "@/features/package/PackFromDossiers";
import { Badge, Button, Card, Checkbox, Dialog, Field, IconButton, Input, Menu, notifyError, PageHeader, Pagination, Select, Table, Tip, toast, Toolbar, type Column, type SortState } from "@/ui";

const n = (v: number) => v.toLocaleString("vi-VN");
const THBQ_TONE: Record<string, "info" | "ok" | "err" | "gray"> = { "badge-vv": "info", "badge-gl": "ok", "badge-het": "err" };

export default function SoHoaPane({ phongs, mode = "taolap" }: { phongs: V40v[]; mode?: "taolap" | "donggoi" }) {
  const dg = mode === "donggoi";
  const { setScanTarget, goto } = V460();
  const [phongBans, setPhongBans] = useState<V3yl[]>([]);
  const [mangs, setMangs] = useState<V3ym[]>([]);
  const [thbqs, setThbqs] = useState<V3yo[]>([]);
  const [maDa, setMaDa] = useState("");
  const [maDv, setMaDv] = useState("");
  const [mangId, setMangId] = useState("");
  const [thbqSel, setThbqSel] = useState<Set<number>>(new Set());
  const [q, setQ] = useState("");
  const [hetThbq, setHetThbq] = useState(false);
  const [giuLai, setGiuLai] = useState(false);
  const [rows, setRows] = useState<V4bm[]>([]);
  const [loaded, setLoaded] = useState(false);
  const [sel, setSel] = useState<Set<string>>(new Set());
  const [openVb, setOpenVb] = useState<V4bm | null>(null);
  const [openTl, setOpenTl] = useState<V4bm | null>(null);
  const [cols, setCols] = useState({ soTo: false, nbd: false, nkt: false });
  const [pageSize, setPageSize] = useState(25);
  const [page, setPage] = useState(1);
  const [sort, setSort] = useState<SortState | null>({ key: "id", dir: "asc" });
  const [dialog, setDialog] = useState<"import" | "inpack" | null>(null);
  const [importBusy, setImportBusy] = useState(false);
  const fileRef = useRef<HTMLInputElement>(null);

  const loadCat = useCallback(() => {
    V48j()
      .then((c) => {
        setPhongBans(c.phongBans);
        setMangs(c.mangs);
        setThbqs(c.thbqs);
      })
      .catch(() => {});
  }, []);
  V41p(loadCat, { topics: ["catalog"], enabled: isTauri() });

  const load = useCallback(() => {
    if (!isTauri()) return;
    if (chuaChonPhong(maDa)) {
      setRows([]);
      setLoaded(true);
      return;
    }
    V4br({ maDa, maDv: maDv || undefined, mangCvid: mangId ? Number(mangId) : undefined })
      .then((rs) => {
        setRows(rs);
        setLoaded(true);
      })
      .catch((e) => {
        setLoaded(true);
        notifyError(e, "nạp hồ sơ");
      });
  }, [maDa, maDv, mangId]);
  V41p(load, { topics: ["dossier", "khogia"] });
  useEffect(() => {
    if (!maDa && phongs.length === 1) setMaDa(phongs[0].maDuAn);
  }, [phongs, maDa]);

  const shown = useMemo(() => {
    let r = rows;
    const k = q.trim().toLowerCase();
    if (k) r = r.filter((x) => V4ct(x, k));
    if (thbqSel.size) r = r.filter((x) => x.thbq != null && thbqSel.has(x.thbq));
    if (giuLai) r = r.filter((x) => V4du(x, thbqs).cls === "badge-gl");
    if (hetThbq) r = r.filter((x) => V4du(x, thbqs).cls === "badge-het");
    return sort ? V4dp(r, sort.key as V4dk, sort.dir, thbqs, mangs) : r;
  }, [rows, q, thbqSel, giuLai, hetThbq, sort, thbqs, mangs]);
  useEffect(() => setPage(1), [shown.length, sort]);
  const pages = Math.max(1, Math.ceil(shown.length / pageSize));
  const cur = Math.min(page, pages);
  const pageRows = useMemo(() => shown.slice((cur - 1) * pageSize, cur * pageSize), [shown, cur, pageSize]);

  const pbOfPhong = V4cl(phongBans, maDa);
  const mangOfPb = V4cm(mangs, maDv);
  const selected = useMemo(() => rows.filter((r) => sel.has(String(r.id))), [rows, sel]);

  const quet = (r: V4bm) => {
    const target = V4d0(r);
    if (!target) {
      toast.error("Hồ sơ chưa gắn phông nên chưa tạo được thư mục quét. Chọn phông cho hồ sơ trước.");
      return;
    }
    setScanTarget(target);
    goto("scan");
  };
  const nhapExcel = async (file: File | undefined) => {
    if (!file) return;
    setImportBusy(true);
    try {
      const { items, skipped, issues } = await V4e8(file, rows);
      if (items.length === 0) {
        const chiTiet = V4di(issues);
        toast.error("Không đọc được văn bản hợp lệ nào trong tệp.", { detail: chiTiet || "Cần xác định được hồ sơ (cột Hồ sơ id hoặc Mã / Hồ sơ số) và có trích yếu, số ký hiệu hoặc tiêu đề." });
        return;
      }
      const k = await V4cg(items);
      toast.success(`Đã nhập ${n(k)} văn bản.`, { description: skipped ? `Bỏ qua ${n(skipped)} dòng thiếu dữ liệu.` : undefined });
      setDialog(null);
      load();
    } catch (e) {
      notifyError(e, "nhập Excel");
    } finally {
      setImportBusy(false);
    }
  };
  const xuatExcel = async () => {
    if (shown.length === 0) {
      toast.error("Không có hồ sơ nào để xuất.");
      return;
    }
    const data = shown.map((r) => ({
      "Mã hồ sơ": r.maHoSo ?? "",
      "Hộp số": r.hopSo ?? "",
      "HS số": r.soTam,
      "Tiêu đề": r.tieuDe,
      THBQ: V4co(thbqs, r.thbq),
      "Số tờ": V4dt(r.soTo),
      "Ngày bắt đầu": V4dr(r.ngayBatDau),
      "Ngày kết thúc": V4dr(r.ngayKetThuc),
      "Trạng thái": V4du(r, thbqs).label,
      "Số văn bản": r.soVanBan,
      "File đính kèm": r.soFileScan,
    }));
    try {
      await V4e3(data, "SoHoa", "xuat-csdl-so-hoa.xlsx");
      toast.success(`Đã xuất ${n(shown.length)} hồ sơ.`);
    } catch (e) {
      notifyError(e, "xuất Excel");
    }
  };

  if (openVb) {
    return (
      <V4dy
        dossier={openVb}
        onBack={() => {
          setOpenVb(null);
          load();
        }}
      />
    );
  }
  if (openTl) return <V4h8 dossier={openTl} onBack={() => setOpenTl(null)} />;

  const columns: Column<V4bm>[] = [
    ...(dg ? [] : [{ key: "maHoSo", header: "Mã hồ sơ", sortable: true, width: 150, cell: (r: V4bm) => <span className="text-xs text-fg-subtle">{r.maHoSo}</span> } as Column<V4bm>]),
    { key: "hopSo", header: "Hộp", sortable: true, align: "right", width: 70 },
    { key: "soTam", header: "HS số", sortable: true, align: "right", width: 80 },
    { key: "tieuDe", header: "Tiêu đề", sortable: true, className: "whitespace-normal min-w-64", cell: (r) => <button type="button" className="text-left hover:text-fg-link" onClick={() => setOpenVb(r)}>{r.tieuDe}</button> },
    ...(cols.soTo ? [{ key: "soTo", header: "Số tờ", sortable: true, align: "right", width: 70, cell: (r: V4bm) => V4dt(r.soTo) } as Column<V4bm>] : []),
    ...(cols.nbd ? [{ key: "ngayBatDau", header: "Bắt đầu", sortable: true, width: 100, cell: (r: V4bm) => V4dr(r.ngayBatDau) } as Column<V4bm>] : []),
    ...(cols.nkt ? [{ key: "ngayKetThuc", header: "Kết thúc", sortable: true, width: 100, cell: (r: V4bm) => V4dr(r.ngayKetThuc) } as Column<V4bm>] : []),
    {
      key: "trangThai",
      header: "THBQ",
      sortable: true,
      width: 120,
      cell: (r) => {
        const s = V4du(r, thbqs);
        return (
          <Tip text={V4co(thbqs, r.thbq)}>
            <span>
              <Badge tone={THBQ_TONE[s.cls] ?? "gray"}>{s.label}</Badge>
            </span>
          </Tip>
        );
      },
    },
    { key: "soTaiLieuScan", header: "PDF quét", sortable: true, align: "right", width: 90, cell: (r) => <button type="button" className="tabular-nums hover:text-fg-link" onClick={() => setOpenTl(r)}>{n(r.soTaiLieuScan)}</button> },
    ...(dg ? [] : [{ key: "soFileScan", header: "Tệp", sortable: true, align: "right", width: 70, cell: (r: V4bm) => n(r.soFileScan) } as Column<V4bm>]),
  ];

  return (
    <>
      {!dg && (
        <PageHeader
          title="Tài liệu đã quét"
          description="Hồ sơ trên máy này với PDF đã quét; mở hồ sơ để soát OCR và nhập văn bản."
          actions={
            <>
              <Button icon="quet" variant="primary" onClick={() => goto("scan")}>
                Quét
              </Button>
              <Menu
                items={[
                  { label: "Nhập văn bản từ Excel", icon: "nhapTep", onSelect: () => setDialog("import") },
                  { label: "Xuất Excel hồ sơ", icon: "tepExcel", onSelect: () => void xuatExcel(), disabled: shown.length === 0 },
                  "-",
                  { label: `${cols.soTo ? "Ẩn" : "Hiện"} cột Số tờ`, onSelect: () => setCols((c) => ({ ...c, soTo: !c.soTo })) },
                  { label: `${cols.nbd ? "Ẩn" : "Hiện"} cột Bắt đầu`, onSelect: () => setCols((c) => ({ ...c, nbd: !c.nbd })) },
                  { label: `${cols.nkt ? "Ẩn" : "Hiện"} cột Kết thúc`, onSelect: () => setCols((c) => ({ ...c, nkt: !c.nkt })) },
                ]}
              />
            </>
          }
        />
      )}
      <Card padding="none">
        <div className="border-b border-border-subtle p-3">
          <Toolbar>
            <Field label="Phông" className="w-60">
              <Select size="sm" value={maDa} onChange={(v) => { setMaDa(v); setMaDv(""); setMangId(""); }} placeholder="— Chọn phông —" options={phongs.map((p) => ({ value: p.maDuAn, label: p.tenDuAn }))} />
            </Field>
            <Field label="Phòng ban" className="w-48">
              <Select size="sm" value={maDv} disabled={!maDa} onChange={(v) => { setMaDv(v); setMangId(""); }} placeholder="Tất cả" options={pbOfPhong.map((p) => ({ value: p.maDv, label: p.tenDonVi }))} />
            </Field>
            <Field label="Mảng" className="w-44">
              <Select size="sm" value={mangId} disabled={!maDv} onChange={setMangId} placeholder="Tất cả" options={mangOfPb.map((m) => ({ value: String(m.mangCvid), label: m.tenMang }))} />
            </Field>
            {!dg && (
              <Field label="Tìm" className="w-56">
                <Input size="sm" prefix="tim" value={q} onChange={(e) => setQ(e.target.value)} placeholder="Tiêu đề, số, mã hồ sơ" />
              </Field>
            )}
            <Toolbar.Spacer />
            {dg && (
              <Button variant="primary" icon="goi" disabled={sel.size === 0} onClick={() => setDialog("inpack")}>
                Đóng gói ({n(sel.size)})
              </Button>
            )}
          </Toolbar>
          <Toolbar className="mt-2">
            <span className="text-sm text-fg-subtle">THBQ:</span>
            {thbqs.map((t) => (
              <button
                key={t.thbqid}
                type="button"
                className={`rounded-full border px-2.5 py-0.5 text-sm ${thbqSel.has(t.thbqid) ? "border-brand-500 bg-brand-50 text-brand-700" : "border-border text-fg-muted hover:border-border-strong"}`}
                aria-pressed={thbqSel.has(t.thbqid)}
                onClick={() =>
                  setThbqSel((s) => {
                    const m = new Set(s);
                    if (m.has(t.thbqid)) m.delete(t.thbqid);
                    else m.add(t.thbqid);
                    return m;
                  })
                }
              >
                {t.tenThbq}
              </button>
            ))}
            {!dg && (
              <>
                <Field label="Hết THBQ" inline>
                  <Checkbox checked={hetThbq} onChange={setHetThbq} />
                </Field>
                <Field label="Giữ lại" inline>
                  <Checkbox checked={giuLai} onChange={setGiuLai} />
                </Field>
              </>
            )}
            <Toolbar.Spacer />
            <span className="text-sm text-fg-subtle">{n(shown.length)} hồ sơ</span>
          </Toolbar>
        </div>
        <Table
          columns={columns}
          rows={pageRows}
          getRowId={(r) => String(r.id)}
          sort={sort}
          onSortChange={setSort}
          selection={{ selected: sel, onChange: setSel }}
          state={{ loading: !loaded, empty: chuaChonPhong(maDa) ? { title: "Chọn phông để xem hồ sơ" } : { title: q || thbqSel.size || hetThbq || giuLai ? "Không có hồ sơ khớp bộ lọc" : "Chưa có hồ sơ trong phông này", description: dg ? undefined : "Hồ sơ đến từ mục lục chỉnh lý hoặc cây thư mục." } }}
          rowActions={
            dg
              ? undefined
              : (r) => (
                  <>
                    <IconButton label="Văn bản trong hồ sơ" icon="taiLieu" size="sm" onClick={() => setOpenVb(r)} />
                    <IconButton label="PDF đã quét" icon="tepPdf" size="sm" onClick={() => setOpenTl(r)} />
                    <IconButton label="Quét vào hồ sơ" icon="quet" size="sm" onClick={() => quet(r)} />
                  </>
                )
          }
          maxHeight="calc(100dvh - 380px)"
        />
        <div className="border-t border-border-subtle px-3 py-2">
          <Pagination page={cur} pageSize={pageSize} total={shown.length} onChange={setPage} onPageSizeChange={(s) => { setPageSize(s); setPage(1); }} pageSizeOptions={[10, 25, 50, 100]} />
        </div>
      </Card>

      <Dialog open={dialog === "import"} onOpenChange={(o) => !o && setDialog(null)} title="Nhập văn bản từ Excel" description="Tệp Excel có cột Hồ sơ id (hoặc Mã / Hồ sơ số) để gắn văn bản vào đúng hồ sơ." size="sm" footer={<><Button onClick={() => setDialog(null)}>Đóng</Button><Button variant="primary" icon="taiLen" loading={importBusy} onClick={() => void nhapExcel(fileRef.current?.files?.[0])}>Nhập</Button></>}>
        <Field label="Tệp Excel">
          <input ref={fileRef} type="file" accept=".xlsx,.xls,.csv" className="block w-full text-sm file:mr-3 file:rounded file:border file:border-border file:bg-surface file:px-3 file:py-1 file:text-sm" />
        </Field>
      </Dialog>

      {dialog === "inpack" && <PackDialog dossiers={selected} thbqs={thbqs} onClose={() => setDialog(null)} />}
    </>
  );
}
