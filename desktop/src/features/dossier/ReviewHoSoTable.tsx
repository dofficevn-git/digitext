// Copyright (c) 2026 DigiText. All rights reserved.
// Tab "Soát & duyệt": tài liệu trên máy chủ gom theo dự án › hồ sơ, tiến độ OCR / PDF/A từng dòng, chọn nhiều
// rồi duyệt / từ chối / mở lại; bấm một tệp để mở màn bóc tách trường.

import { Fragment, useCallback, useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { V41t } from "@/live/useLive";
import { isTauri } from "@/api/ipc";
import { V4f8, V4f7, docsReviewSummary, docsReviewGroupDocs, type V4eh, type ReviewSummaryRow } from "@/api/docs";
import { V4hl } from "./ocrReport";
import { bocTachState, V4kr, V4kv, V4kf, V4ke, V4kx, pdfaPercent, pdfaState, V4ks, docToFile, groupsFromSummary, reviewKeyOf, type V4kg, type V4k8, type V4kb, type V4k7 } from "./reviewScope";
import "./ocrHoSo.css";
import { Icon } from "@/components/Icon";
import type { V41v } from "@/components/Icon";
import { Badge, Button, Card, Checkbox, Dialog, EmptyState, ErrorBanner, Field, IconButton, Input, Pagination, Select, Skeleton, Table, Textarea, Tip, Toolbar, notifyError, pageSlice, toast, usePagination } from "@/ui";

const fmtN = (n: number) => n.toLocaleString("vi-VN");
const mb = (b: number) => (b / 1_048_576).toFixed(1);

const QUYET_DINH: Record<V4eh, { icon: V41v; nut: string; hoi: string; xong: string }> = {
  approved: { icon: "duyet", nut: "Duyệt", hoi: "Duyệt", xong: "Đã duyệt" },
  rejected: { icon: "tuChoi", nut: "Từ chối", hoi: "Từ chối", xong: "Đã từ chối" },
  pending: { icon: "moLai", nut: "Mở lại", hoi: "Mở lại", xong: "Đã mở lại" },
};

function TriCheck({ state, onChange, label, disabled }: { state: V4kg; onChange: (v: boolean) => void; label: string; disabled?: boolean }) {
  const ref = useRef<HTMLInputElement>(null);
  useEffect(() => {
    if (ref.current) ref.current.indeterminate = state === "some";
  }, [state]);
  return <input ref={ref} type="checkbox" className="size-4 cursor-pointer accent-brand-600" checked={state === "all"} disabled={disabled} aria-label={label} onChange={(e) => onChange(e.target.checked)} />;
}

function TallyCell({ t }: { t: V4k7 }) {
  return (
    <>
      <div>
        {fmtN(t.pages)} trang · {mb(t.sizeBytes)} MB
      </div>
      {t.unknownPages > 0 && <div className="text-xs text-fg-subtle">{fmtN(t.unknownPages)} tài liệu chưa rõ số trang</div>}
    </>
  );
}

function OcrCell({ t }: { t: V4k7 }) {
  const p = V4kx(t);
  const pa = pdfaPercent(t);
  return (
    <>
      <div className="oh-bar">
        <span className="oh-num">
          {fmtN(t.ocrPages)}/{fmtN(t.ocrDenom)}
        </span>
        <Tip text="Trang đã có kết quả OCR">
          <div className="progress">
            <span style={{ width: `${p}%` }} />
          </div>
        </Tip>
        <span className="oh-sub">{p}%</span>
      </div>
      {t.fieldDocs > 0 && (
        <div className="text-xs text-fg-subtle">
          {fmtN(t.fieldDocs)}/{fmtN(t.files)} tài liệu đã bóc 16 trường
        </div>
      )}
      <div className="oh-bar oh-pdfa">
        <span className="oh-sub">PDF/A</span>
        <span className="oh-num oh-sub">
          {fmtN(t.pdfaDocs)}/{fmtN(t.files)}
        </span>
        <Tip text="Văn bản đã có bản PDF/A, sinh ra ở việc PDF 2 lớp">
          <div className="progress">
            <span style={{ width: `${pa}%` }} />
          </div>
        </Tip>
        <span className="oh-sub">{pa}%</span>
      </div>
      {t.pdfaCu > 0 && <div className="text-xs text-warning-700">{fmtN(t.pdfaCu)} tệp chưa đạt PDF/A-2b, cần ghép lại</div>}
      {t.unknownOcr > 0 && <div className="text-xs text-fg-subtle">{fmtN(t.unknownOcr)} tài liệu máy chủ chưa báo</div>}
    </>
  );
}

function QuyetDinhDialog({ decision, count, busy, onCancel, onConfirm }: { decision: V4eh; count: number; busy: boolean; onCancel: () => void; onConfirm: (note: string) => void }) {
  const [note, setNote] = useState("");
  const q = QUYET_DINH[decision];
  const thieuLyDo = decision === "rejected" && note.trim() === "";
  return (
    <Dialog
      open
      onOpenChange={(o) => !o && !busy && onCancel()}
      closable={!busy}
      title={`${q.hoi} ${fmtN(count)} tài liệu`}
      size="sm"
      description={
        decision === "approved"
          ? "Toàn bộ tài liệu đã chọn được đánh dấu đã duyệt, có ghi vết người và thời điểm."
          : decision === "rejected"
            ? "Tài liệu đã chọn bị trả lại để quét hoặc bóc lại. Nêu rõ lý do để người quét biết phải sửa gì."
            : "Xoá quyết định duyệt của tài liệu đã chọn để soát lại từ đầu."
      }
      footer={
        <>
          <Button onClick={onCancel} disabled={busy}>
            Huỷ
          </Button>
          <Button variant={decision === "rejected" ? "danger" : "primary"} icon={q.icon} loading={busy} disabled={thieuLyDo} onClick={() => onConfirm(note)}>
            {q.hoi}
          </Button>
        </>
      }
    >
      {decision !== "pending" && (
        <Field label={decision === "rejected" ? "Lý do trả lại" : "Ghi chú"} required={decision === "rejected"} hint={decision === "rejected" ? undefined : "Không bắt buộc"}>
          <Textarea rows={3} autoFocus value={note} onChange={(e) => setNote(e.target.value)} placeholder={decision === "rejected" ? "Ví dụ: ảnh mờ, bóc sai số quyết định…" : ""} />
        </Field>
      )}
    </Dialog>
  );
}

function trangThaiBocTach(f: V4k8): ReactNode {
  switch (bocTachState(f)) {
    case "unknown":
      return "máy chủ chưa báo";
    case "none":
      return "chưa bóc tách";
    case "fields":
      return "đã bóc 16 trường";
    case "partial":
      return `OCR ${fmtN(f.ocrPages)}/${fmtN(f.pages ?? 0)} trang`;
    default:
      return `OCR ${fmtN(f.ocrPages)} trang${f.fieldPages > 0 ? " · 16 trường" : ""}`;
  }
}

function trangThaiPdfa(f: V4k8): ReactNode {
  switch (pdfaState(f)) {
    case "none":
      return <span className="text-fg-subtle">chưa có PDF/A</span>;
    case "cu":
      return <span className="text-warning-700">PDF/A chưa đạt 2b</span>;
    case "chuaRo":
      return <span className="text-fg-subtle">PDF/A {fmtN(f.pdfaPages)} trang</span>;
    default:
      return <span className="text-fg-subtle">PDF/A-2b · {fmtN(f.pdfaPages)} trang</span>;
  }
}

function DongTep({ row, selected, busy, canReview, onToggle, onOpen }: { row: V4kb; selected: ReadonlySet<string>; busy: boolean; canReview: boolean; onToggle: (ids: string[], v: boolean) => void; onOpen: (f: V4k8) => void }) {
  return (
    <>
      {row.files.map((f) => {
        const rv = V4hl(f.review);
        return (
          <Table.Row key={f.id} className="oh-filerow">
            <Table.Cell className="oh-check">{canReview && <TriCheck state={selected.has(f.id) ? "all" : "none"} disabled={busy} label={`Chọn ${f.filename}`} onChange={(v) => onToggle([f.id], v)} />}</Table.Cell>
            <Table.Cell colSpan={4} />
            <Table.Cell className="oh-fname oh-files">
              {f.filename} <Badge tone={rv.tone}>{rv.label}</Badge>
            </Table.Cell>
            <Table.Cell>
              {f.pages == null ? "chưa rõ số trang" : `${fmtN(f.pages)} trang`} · {mb(f.sizeBytes)} MB
            </Table.Cell>
            <Table.Cell>
              <div className="oh-bar">
                <span className="oh-sub text-fg-subtle">{trangThaiBocTach(f)}</span>
                <div className="flex-1" />
                <Button size="sm" variant="primary" icon="tim" onClick={() => onOpen(f)}>
                  Bóc tách trường
                </Button>
              </div>
              <div className="oh-sub">{trangThaiPdfa(f)}</div>
            </Table.Cell>
          </Table.Row>
        );
      })}
    </>
  );
}

export default function ReviewHoSoTable({ canReview, actions, onlyProject, onOpenDoc, onDone }: { canReview: boolean; actions?: ReactNode; onlyProject?: string; onOpenDoc: (f: V4k8) => void; onDone?: () => void }) {
  // Số liệu gộp theo hồ sơ lấy từ máy chủ (vài nghìn dòng); tệp của từng hồ sơ nạp khi mở hoặc tích chọn.
  const [q, setQ] = useState("");
  const [qTim, setQTim] = useState("");
  useEffect(() => {
    const t = window.setTimeout(() => setQTim(q.trim()), 300);
    return () => window.clearTimeout(t);
  }, [q]);
  const [chiChoXacNhan, setChiChoXacNhan] = useState(false);
  const loader = useCallback(
    () => (isTauri() ? docsReviewSummary({ projectId: onlyProject || undefined, q: qTim || undefined, pendingOnly: chiChoXacNhan }) : Promise.resolve([] as ReviewSummaryRow[])),
    [onlyProject, qTim, chiChoXacNhan],
  );
  const { data, error, loading, reload } = V41t(loader, { topics: ["docs", "ocr"], errorText: () => "Không tải được danh sách hồ sơ trên máy chủ." });
  const summary = useMemo(() => data ?? [], [data]);

  const [files, setFiles] = useState<ReadonlyMap<string, V4k8[]>>(() => new Map());
  const [selected, setSelected] = useState<ReadonlySet<string>>(new Set<string>());
  const [moRong, setMoRong] = useState<ReadonlySet<string>>(new Set<string>());
  const [dangNap, setDangNap] = useState<ReadonlySet<string>>(new Set<string>());
  const [quyetDinh, setQuyetDinh] = useState<V4eh | null>(null);
  const [tienDo, setTienDo] = useState<{ done: number; total: number } | null>(null);

  const [phongLoc, setPhongLoc] = useState("");
  const dsPhong = useMemo(() => {
    const s = new Set<string>();
    for (const r of summary) if (r.phong) s.add(r.phong);
    return [...s].sort(new Intl.Collator("vi", { numeric: true }).compare);
  }, [summary]);
  useEffect(() => {
    if (phongLoc && !dsPhong.includes(phongLoc)) setPhongLoc("");
  }, [dsPhong, phongLoc]);
  const nhom = useMemo(() => groupsFromSummary(phongLoc ? summary.filter((r) => r.phong === phongLoc) : summary, files), [summary, phongLoc, files]);
  const flat = useMemo(() => nhom.flatMap((g) => g.rows.map((r) => ({ g, r }))), [nhom]);
  const trang = usePagination(100);
  const { setPage } = trang;
  useEffect(() => setPage(1), [qTim, chiChoXacNhan, phongLoc, onlyProject, setPage]);
  const pageRows = useMemo(() => pageSlice(flat, trang), [flat, trang]);

  const busy = tienDo !== null;
  const daChon = useMemo(() => [...selected], [selected]);
  const tongTep = useMemo(() => nhom.reduce((n, g) => n + g.tally.files, 0), [nhom]);
  const idsTrang = useMemo(() => pageRows.flatMap((x) => x.r.ids), [pageRows]);

  // Nạp tệp cho các hồ sơ chưa có trong bộ đệm; trả về bộ đệm sau khi gộp.
  const napTep = useCallback(
    async (rows: V4kb[]): Promise<ReadonlyMap<string, V4k8[]>> => {
      const can = rows.filter((r) => !files.has(r.key));
      if (can.length === 0) return files;
      setDangNap((cu) => new Set([...cu, ...can.map((r) => r.key)]));
      try {
        const theoDuAn = new Map<string, V4kb[]>();
        for (const r of can) theoDuAn.set(r.projectId, [...(theoDuAn.get(r.projectId) ?? []), r]);
        const moi = new Map(files);
        for (const r of can) moi.set(r.key, []);
        for (const [projectId, rs] of theoDuAn) {
          const docs = await docsReviewGroupDocs({ projectId, groups: rs.map((r) => ({ phong: r.phong, hop: r.hopSo, hoSo: r.hoSoSo })) });
          for (const d of docs) {
            if (d.status === "disposed") continue;
            const k = reviewKeyOf(d);
            moi.set(k, [...(moi.get(k) ?? []), docToFile(d)]);
          }
        }
        setFiles(moi);
        return moi;
      } finally {
        setDangNap((cu) => {
          const s = new Set(cu);
          for (const r of can) s.delete(r.key);
          return s;
        });
      }
    },
    [files],
  );

  const toggleMoRong = useCallback(
    (row: V4kb) => {
      setMoRong((cu) => {
        const s = new Set(cu);
        if (!s.delete(row.key)) s.add(row.key);
        return s;
      });
      if (!row.loaded) napTep([row]).catch((e) => notifyError(e, "nạp tệp của hồ sơ"));
    },
    [napTep],
  );
  const chon = useCallback((ids: string[], v: boolean) => setSelected((cu) => V4ks(cu, ids, v)), []);
  const chonHoSo = useCallback(
    async (rows: V4kb[], v: boolean) => {
      try {
        const m = await napTep(rows);
        const ids = rows.flatMap((r) => (m.get(r.key) ?? []).map((f) => f.id));
        chon(ids, v);
      } catch (e) {
        notifyError(e, "chọn hồ sơ");
      }
    },
    [napTep, chon],
  );

  const ghiQuyetDinh = async (d: V4eh, note: string) => {
    const ids = daChon;
    if (ids.length === 0) return;
    const lo = V4kv(ids, V4f7);
    const ghiChu = note.trim() === "" ? undefined : note.trim();
    let updated = 0;
    let skipped = 0;
    setTienDo({ done: 0, total: ids.length });
    try {
      for (const phan of lo) {
        const r = await V4f8(phan, d, ghiChu);
        updated += r.updated;
        skipped += r.skipped;
        setTienDo({ done: updated + skipped, total: ids.length });
      }
      setQuyetDinh(null);
      setSelected(new Set<string>());
      toast.success(`${QUYET_DINH[d].xong} ${fmtN(updated)} tài liệu.`, skipped > 0 ? { description: `${fmtN(skipped)} tài liệu không đổi được vì đã bị xoá hoặc ngoài phạm vi dự án của bạn.` } : undefined);
    } catch (e) {
      notifyError(e, `ghi quyết định (đã ghi ${fmtN(updated)}/${fmtN(ids.length)})`);
      setQuyetDinh(null);
    } finally {
      setTienDo(null);
      setFiles(new Map());
      reload();
      onDone?.();
    }
  };

  const soHoSo = flat.length;

  return (
    <Card padding="none">
      <Toolbar className="px-4 py-3">
        <Input size="sm" prefix="tim" className="w-64" placeholder="Tìm tên tệp, hộp, hồ sơ" value={q} onChange={(e) => setQ(e.target.value)} aria-label="Tìm tài liệu" />
        {dsPhong.length > 0 && <Select size="sm" className="w-56" value={phongLoc} onChange={setPhongLoc} placeholder={`Tất cả phông (${dsPhong.length})`} options={dsPhong.map((p) => ({ value: p, label: p }))} aria-label="Lọc theo phông" />}
        <Checkbox checked={chiChoXacNhan} disabled={busy} onChange={setChiChoXacNhan} label="Chỉ phần chờ xác nhận" />
        <Toolbar.Spacer />
        <span className="text-sm text-fg-subtle">{loading && summary.length === 0 ? "Đang tải…" : `${fmtN(soHoSo)} hồ sơ · ${fmtN(tongTep)} tài liệu`}</span>
        {actions}
        <IconButton label="Tải lại" icon="lamMoi" size="sm" onClick={reload} disabled={busy} />
      </Toolbar>

      {error !== null && error !== undefined && <ErrorBanner error={error} className="mx-4 mb-3" onRetry={reload} />}

      {loading && summary.length === 0 ? (
        <div className="px-4 pb-4">
          <Skeleton.Rows rows={6} />
        </div>
      ) : flat.length === 0 ? (
        <EmptyState
          compact
          icon="taiLieu"
          title={summary.length === 0 && !qTim && !chiChoXacNhan ? "Chưa có tài liệu nào trên máy chủ" : "Không có hồ sơ nào khớp bộ lọc"}
          description={summary.length === 0 && !qTim && !chiChoXacNhan ? "Đưa dữ liệu lên qua Số hoá › Đưa lên máy chủ." : onlyProject ? "Dự án đang chọn chưa có hồ sơ nào khớp." : chiChoXacNhan ? "Không còn tài liệu nào chờ xác nhận." : undefined}
        />
      ) : (
        <>
          <Table.Root className="oh-table" maxHeight="min(62vh, 640px)">
            <Table.Head>
              <tr>
                <Table.HeadCell className="oh-check">
                  {canReview && <TriCheck state={V4kr(idsTrang, selected)} disabled={busy || pageRows.length === 0} label="Chọn toàn bộ hồ sơ ở trang này" onChange={(v) => void chonHoSo(pageRows.map((x) => x.r), v)} />}
                </Table.HeadCell>
                <Table.HeadCell>Dự án</Table.HeadCell>
                <Table.HeadCell>Phông</Table.HeadCell>
                <Table.HeadCell>Hộp số</Table.HeadCell>
                <Table.HeadCell>Hồ sơ số</Table.HeadCell>
                <Table.HeadCell className="oh-files">Tổng số tệp</Table.HeadCell>
                <Table.HeadCell>Tài liệu</Table.HeadCell>
                <Table.HeadCell>OCR · PDF/A</Table.HeadCell>
              </tr>
            </Table.Head>
            <Table.Body>
              {pageRows.map(({ g, r }, i) => {
                const mo = moRong.has(r.key);
                const dauNhom = i === 0 || pageRows[i - 1].g.key !== g.key;
                return (
                  <Fragment key={r.key}>
                    {dauNhom && (
                      <Table.Row className="oh-grouprow">
                        <Table.Cell className="oh-check" />
                        <th scope="row" className="oh-groupname px-3" colSpan={4}>
                          <Icon name="thuMuc" /> Dự án: {g.label}
                        </th>
                        <Table.Cell className="oh-num oh-files">{fmtN(g.tally.files)}</Table.Cell>
                        <Table.Cell>
                          <TallyCell t={g.tally} />
                        </Table.Cell>
                        <Table.Cell>
                          <OcrCell t={g.tally} />
                        </Table.Cell>
                      </Table.Row>
                    )}
                    <Table.Row>
                      <Table.Cell className="oh-check">{canReview && <TriCheck state={r.loaded ? V4kr(r.ids, selected) : "none"} disabled={busy || dangNap.has(r.key)} label={`Chọn cả hồ sơ ${r.hoSoSo || "chưa gán"}`} onChange={(v) => void chonHoSo([r], v)} />}</Table.Cell>
                      <th scope="row" className="oh-duan px-3 text-left font-normal">{g.label}</th>
                      <th scope="row" className="px-3 text-left font-normal">{r.phong || "—"}</th>
                      <th scope="row" className="px-3 text-left font-normal">{r.hopSo || V4ke}</th>
                      <th scope="row" className="px-3 text-left font-normal">
                        {r.hoSoSo || V4kf}
                        {r.counts.rejected > 0 && <div className="text-xs text-danger-700">{fmtN(r.counts.rejected)} tài liệu bị từ chối</div>}
                      </th>
                      <Table.Cell className="oh-num oh-files">
                        <button type="button" className="oh-expand" onClick={() => toggleMoRong(r)} aria-expanded={mo} aria-label={mo ? "Thu gọn danh sách tệp" : "Xem từng tệp trong hồ sơ"}>
                          <span className="oh-caret">
                            <Icon name={mo ? "xuong" : "moNhanh"} size={13} />
                          </span>
                          {fmtN(r.tally.files)}
                        </button>
                      </Table.Cell>
                      <Table.Cell>
                        <TallyCell t={r.tally} />
                      </Table.Cell>
                      <Table.Cell>
                        <OcrCell t={r.tally} />
                      </Table.Cell>
                    </Table.Row>
                    {mo && !r.loaded && (
                      <Table.Row className="oh-filerow">
                        <Table.Cell className="oh-check" />
                        <Table.Cell colSpan={7} className="text-sm text-fg-subtle">
                          Đang nạp tệp của hồ sơ…
                        </Table.Cell>
                      </Table.Row>
                    )}
                    {mo && r.loaded && <DongTep row={r} selected={selected} busy={busy} canReview={canReview} onToggle={chon} onOpen={onOpenDoc} />}
                  </Fragment>
                );
              })}
            </Table.Body>
          </Table.Root>
          {flat.length > trang.pageSize && (
            <div className="px-4 py-2">
              <Pagination page={trang.page} pageSize={trang.pageSize} total={flat.length} onChange={trang.setPage} onPageSizeChange={trang.setPageSize} pageSizeOptions={[50, 100, 200]} />
            </div>
          )}
        </>
      )}

      {canReview && (
        <Toolbar className="border-t border-border-subtle px-4 py-2.5">
          <span className="text-sm">{tienDo ? `Đang ghi ${fmtN(tienDo.done)}/${fmtN(tienDo.total)} tài liệu…` : `Đã chọn ${fmtN(daChon.length)} tài liệu`}</span>
          {daChon.length > 0 && !busy && (
            <Button size="sm" variant="ghost" onClick={() => setSelected(new Set<string>())}>
              Bỏ chọn
            </Button>
          )}
          <Toolbar.Spacer />
          {(["approved", "rejected", "pending"] as const).map((d) => (
            <Button key={d} size="sm" variant={d === "approved" ? "primary" : d === "rejected" ? "dangerOutline" : "secondary"} icon={QUYET_DINH[d].icon} disabled={busy || daChon.length === 0} onClick={() => setQuyetDinh(d)}>
              {QUYET_DINH[d].nut}
            </Button>
          ))}
        </Toolbar>
      )}

      {quyetDinh && <QuyetDinhDialog decision={quyetDinh} count={daChon.length} busy={busy} onCancel={() => setQuyetDinh(null)} onConfirm={(note) => void ghiQuyetDinh(quyetDinh, note)} />}
    </Card>
  );
}
