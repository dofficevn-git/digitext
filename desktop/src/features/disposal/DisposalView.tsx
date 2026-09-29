// Copyright (c) 2026 DigiText. All rights reserved.
// Tiêu huỷ tài liệu hết giá trị (Điều 48 đến 50 Thông tư 05/2025): tab Danh mục hết giá trị (chọn nhiều → lập đợt)
// và tab Đợt xét huỷ (Stepper Hội đồng → Thẩm định → Quyết định → Đã huỷ, vết quá trình).

import { useCallback, useMemo, useState } from "react";
import { V41t } from "@/live/useLive";
import { V4uk, V4ui, V4uh, V4ug, V4uc, V4uf, V4ue, V4ud, type V4u9, type V4ub, type V4u8 } from "@/api/disposal";
import { V4ei, type V4eg } from "@/api/docs";
import { isTauri } from "@/api/ipc";
import { V460 } from "@/state";
import { BUOC_DOT, LEGAL, V4un, V4uq, V4um, V4uo, V4ul, V4up, type DotAction } from "./disposalFlow";
import { Badge, Banner, Button, Card, confirm, Dialog, EmptyState, Help, Input, Menu, notifyError, PageHeader, prompt, Stepper, Table, Tabs, Tip, toast, type Column } from "@/ui";

const fmtN = (v: number) => v.toLocaleString("vi-VN");
type Tab = "danhmuc" | "dot";

function HanLuuTruDialog({ onClose }: { onClose: () => void }) {
  const [docs, setDocs] = useState<V4eg[] | null>(null);
  const [q, setQ] = useState("");
  const [err, setErr] = useState<unknown>(null);
  useState(() => {
    V4ei()
      .then(setDocs)
      .catch((e) => setErr(e));
  });
  const datHan = async (doc: V4eg) => {
    const v = await prompt({ title: "Hạn lưu trữ", body: doc.filename, label: "Ngày hết hạn", type: "date", placeholder: "Để trống để bỏ hạn", confirmLabel: "Đặt hạn" });
    if (v === null) return;
    try {
      await V4ue(doc.id, v ? `${v}T00:00:00+07:00` : null);
      toast.success(v ? `Đã đặt hạn ${v.split("-").reverse().join("/")} cho ${doc.filename}.` : `Đã bỏ hạn cho ${doc.filename}.`);
    } catch (e) {
      notifyError(e, "đặt hạn");
    }
  };
  const t = q.trim().toLowerCase();
  const rows = (docs ?? []).filter((d) => !t || d.filename.toLowerCase().includes(t) || (d.hoSoRef ?? "").toLowerCase().includes(t));
  const cols: Column<V4eg>[] = [
    { key: "filename", header: "Tên tệp", className: "whitespace-normal" },
    { key: "hoSoRef", header: "Mã hồ sơ", width: 160, cell: (d) => d.hoSoRef ?? "—" },
  ];
  return (
    <Dialog open onOpenChange={(o) => !o && onClose()} title="Hạn lưu trữ tài liệu" description="Đặt ngày hết hạn để hệ thống tự đưa vào danh mục hết giá trị khi quét." size="lg" footer={<Button onClick={onClose}>Đóng</Button>}>
      <Input size="sm" prefix="tim" className="mb-3 w-72" placeholder="Tìm tên tệp, mã hồ sơ" value={q} onChange={(e) => setQ(e.target.value)} aria-label="Tìm tài liệu" />
      <Table columns={cols} rows={rows} getRowId={(d) => d.id} dense maxHeight={400} rowActions={(d) => <Button size="sm" icon="hanLuuTru" onClick={() => void datHan(d)}>Đặt hạn</Button>} state={{ loading: docs === null && err === null, error: err ?? undefined, empty: { title: t ? "Không có tài liệu nào khớp" : "Chưa có tài liệu nào trên máy chủ" } }} />
    </Dialog>
  );
}

export default function V4ur() {
  const { session } = V460();
  const perms = session?.permissions ?? [];
  const canReview = perms.includes("disposal.review");
  const canDecide = perms.includes("disposal.decide");
  const loader = useCallback(async () => {
    if (!isTauri()) return { entries: [] as V4u8[], batches: [] as V4u9[] };
    const [entries, batches] = await Promise.all([V4uc(), V4ug()]);
    return { entries, batches };
  }, []);
  const { data, error, loading, reload } = V41t(loader, { topics: ["disposal", "docs"], errorText: () => "Không tải được dữ liệu tiêu huỷ." });
  const entries = useMemo(() => data?.entries ?? [], [data]);
  const batches = useMemo(() => data?.batches ?? [], [data]);
  const [tab, setTab] = useState<Tab>("danhmuc");
  const [sel, setSel] = useState<Set<string>>(new Set());
  const [chiTiet, setChiTiet] = useState<V4ub | null>(null);
  const [busy, setBusy] = useState(false);
  const [moHan, setMoHan] = useState(false);

  const chay = async (f: () => Promise<void>, ctx: string) => {
    setBusy(true);
    try {
      await f();
    } catch (e) {
      notifyError(e, ctx);
    } finally {
      setBusy(false);
    }
  };

  const quet = () =>
    chay(async () => {
      const r = await V4ud();
      toast.success(`Quét xong: ${fmtN(r.trungLap)} trùng lặp, ${fmtN(r.hetThoiHan)} hết thời hạn.`);
      reload();
    }, "quét");

  const giuLai = (e: V4u8) =>
    chay(async () => {
      const ly = await prompt({ title: `Giữ lại "${e.filename}"`, label: "Lý do giữ lại", type: "textarea", required: true, body: "Lý do được lưu vết trong nhật ký.", confirmLabel: "Giữ lại" });
      if (ly === null) return;
      await V4uf(e.id, "keep", ly.trim());
      toast.success("Đã giữ lại, loại khỏi danh mục.");
      reload();
    }, "giữ lại");

  const ghiChu = (e: V4u8) =>
    chay(async () => {
      const v = await prompt({ title: "Ghi chú", body: e.filename, label: "Bản giấy, vật mang tin khác", type: "textarea", initial: e.ghiChu ?? "", confirmLabel: "Lưu" });
      if (v === null) return;
      await V4uf(e.id, "note", v.trim());
      reload();
    }, "ghi chú");

  const lapDot = () =>
    chay(async () => {
      const nam = await prompt({ title: `Lập đợt xét huỷ cho ${fmtN(sel.size)} tài liệu`, label: "Năm của đợt", type: "number", initial: String(new Date().getFullYear()), required: true, validate: (v) => (Number(v) >= 2000 ? null : "Năm không hợp lệ"), confirmLabel: "Lập đợt" });
      if (nam === null) return;
      const b = await V4ui(Number(nam), [...sel]);
      setSel(new Set());
      toast.success(`Đã lập đợt xét huỷ năm ${nam}. Bước tiếp theo: Hội đồng.`);
      reload();
      setChiTiet(await V4uh(b.id));
      setTab("dot");
    }, "lập đợt");

  const moDot = (id: string) => chay(async () => setChiTiet(await V4uh(id)), "mở đợt");

  const hanhDong = (b: V4u9, a: DotAction) =>
    chay(async () => {
      if (a === "council" || a === "appraise") {
        const v = await prompt({ title: a === "council" ? "Kết quả làm việc của Hội đồng" : "Ý kiến thẩm định của cơ quan quản lý", label: "Nội dung", type: "textarea", required: true, confirmLabel: "Ghi nhận" });
        if (v === null) return;
        await V4uk(b.id, a, { note: v.trim() });
      } else if (a === "decide") {
        const so = await prompt({ title: "Quyết định huỷ", label: "Số quyết định", required: true, confirmLabel: "Tiếp tục" });
        if (so === null) return;
        let tuQuyet = false;
        if (V4uq(b)) {
          tuQuyet = await confirm({ title: "Chưa có ý kiến thẩm định", body: "Chỉ được tự quyết khi cơ quan không thuộc nguồn nộp lưu trữ lịch sử. Xác nhận tự quyết?", confirmLabel: "Tự quyết", danger: true });
          if (!tuQuyet) return;
        }
        await V4uk(b.id, "decide", { quyetDinhSo: so.trim(), tuQuyet });
      } else {
        const ok = await confirm({ title: `Huỷ ${fmtN(chiTiet?.entries.length ?? 0)} tài liệu trong hệ thống?`, body: "Tệp của các tài liệu trong đợt bị xoá vĩnh viễn khỏi kho. Không hoàn tác được.", confirmLabel: "Huỷ vĩnh viễn", danger: true, typeToConfirm: "HUỶ" });
        if (!ok) return;
        const r = await V4uk(b.id, "execute", {});
        toast.success(`Đã huỷ ${fmtN(r?.disposed ?? 0)} tài liệu. Vết đầy đủ trong nhật ký đợt.`);
      }
      reload();
      setChiTiet(await V4uh(b.id));
    }, "xử lý đợt");

  const choXemXet = entries.filter((e) => e.status === "xem_xet" && !e.batchId);
  const colsDm: Column<V4u8>[] = [
    { key: "filename", header: "Tên tệp", className: "whitespace-normal", sortable: true },
    { key: "hoSoRef", header: "Mã hồ sơ", width: 150, cell: (e) => e.hoSoRef ?? "—" },
    { key: "pageCount", header: "Trang", align: "right", width: 70, cell: (e) => e.pageCount ?? "—" },
    {
      key: "lyDo",
      header: "Lý do",
      width: 170,
      sortable: true,
      cell: (e) => (
        <Tip text={`Theo ${LEGAL[e.lyDo]}`}>
          <span>{V4ul[e.lyDo]}</span>
        </Tip>
      ),
    },
    { key: "ghiChu", header: "Ghi chú", className: "whitespace-normal text-fg-subtle", cell: (e) => e.ghiChu ?? "" },
    { key: "status", header: "Trạng thái", width: 120, cell: (e) => <Badge tone={e.status === "da_huy" ? "err" : e.status === "giu_lai" ? "ok" : e.batchId ? "info" : "warn"}>{e.batchId && e.status === "xem_xet" ? "Trong đợt" : V4um[e.status]}</Badge> },
  ];
  const colsDot: Column<V4u9>[] = [
    { key: "nam", header: "Năm", width: 80, sortable: true },
    { key: "status", header: "Trạng thái", width: 200, cell: (b) => <Badge tone={b.status === "da_huy" ? "err" : b.status === "quyet_dinh" ? "warn" : "info"}>{V4un[b.status]}</Badge> },
    { key: "quyetDinhSo", header: "Quyết định", cell: (b) => (b.quyetDinhSo ? <>{b.quyetDinhSo}{b.tuQuyet && <Badge tone="gray" className="ml-1">tự quyết</Badge>}</> : "—") },
    { key: "createdAt", header: "Lập ngày", width: 110, cell: (b) => new Date(b.createdAt).toLocaleDateString("vi-VN") },
  ];
  const colsTrongDot: Column<V4u8>[] = [
    { key: "filename", header: "Tên tệp", className: "whitespace-normal" },
    { key: "lyDo", header: "Lý do", width: 170, cell: (e) => V4ul[e.lyDo] },
    { key: "ghiChu", header: "Ghi chú", className: "whitespace-normal text-fg-subtle", cell: (e) => e.ghiChu ?? "" },
    { key: "status", header: "Trạng thái", width: 120, cell: (e) => <Badge tone={e.status === "da_huy" ? "err" : "info"}>{V4um[e.status]}</Badge> },
  ];
  const buocDot = chiTiet ? BUOC_DOT.findIndex((b) => b.key === chiTiet.status) + (chiTiet.status === "da_huy" ? 1 : 0) : 0;

  return (
    <section className="view active">
      <PageHeader
        title="Tiêu huỷ tài liệu hết giá trị"
        description="Danh mục do hệ thống quét hằng ngày, xét huỷ theo đợt qua Hội đồng, thẩm định và quyết định."
        actions={
          canReview && (
            <>
              <Button variant="primary" icon="tim" loading={busy} onClick={() => void quet()}>
                Quét ngay
              </Button>
              <Menu items={[{ label: "Hạn lưu trữ tài liệu", icon: "hanLuuTru", onSelect: () => setMoHan(true) }]} />
            </>
          )
        }
      />
      {error != null && <Banner tone="danger" className="mb-4" actions={<Button size="sm" onClick={reload}>Thử lại</Button>}>Không tải được dữ liệu tiêu huỷ.</Banner>}
      <Tabs<Tab> value={tab} onChange={setTab} items={[{ value: "danhmuc", label: "Danh mục hết giá trị", icon: "huy", count: choXemXet.length }, { value: "dot", label: "Đợt xét huỷ", icon: "bienBan", count: batches.length }]} />
      <div className="mt-3 space-y-4">
        {tab === "danhmuc" && (
          <Card
            padding="none"
            title="Tài liệu hết giá trị"
            description={choXemXet.length > 0 ? `${fmtN(choXemXet.length)} tài liệu đang chờ xem xét, chưa vào đợt.` : undefined}
            actions={
              canReview && (
                <Button variant="primary" icon="bienBan" disabled={busy || sel.size === 0} onClick={() => void lapDot()}>
                  Lập đợt xét huỷ ({fmtN(sel.size)})
                </Button>
              )
            }
          >
            <Table
              columns={colsDm}
              rows={entries}
              getRowId={(e) => e.id}
              dense
              maxHeight={520}
              selection={canReview ? { selected: sel, onChange: setSel, canSelect: (e) => e.status === "xem_xet" && !e.batchId } : undefined}
              rowActions={canReview ? (e) => (e.status === "xem_xet" && !e.batchId ? <Menu size="sm" items={[{ label: "Ghi chú", icon: "sua", onSelect: () => void ghiChu(e) }, { label: "Giữ lại", icon: "hoanTac", onSelect: () => void giuLai(e) }]} /> : null) : undefined}
              state={{ loading: loading && !data, empty: { title: "Danh mục trống", description: "Hệ thống tự quét trùng lặp và hết thời hạn mỗi ngày.", icon: "huy" } }}
            />
          </Card>
        )}
        {tab === "dot" && (
          <>
            <Card padding="none" title="Các đợt xét huỷ">
              <Table columns={colsDot} rows={batches} getRowId={(b) => b.id} dense onRowClick={(b) => void moDot(b.id)} rowClassName={(b) => (chiTiet?.id === b.id ? "bg-brand-50" : undefined)} state={{ loading: loading && !data, empty: { title: "Chưa có đợt xét huỷ nào", description: "Chọn tài liệu ở tab Danh mục hết giá trị rồi lập đợt.", icon: "bienBan" } }} />
            </Card>
            {chiTiet && (
              <Card title={`Đợt xét huỷ năm ${chiTiet.nam}`} description={V4un[chiTiet.status]}>
                <Stepper className="mb-4" current={buocDot} steps={BUOC_DOT.map((b) => ({ key: b.key, label: b.label }))} />
                <dl className="mb-4 grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-sm">
                  <dt className="text-fg-subtle">Hội đồng</dt>
                  <dd>{chiTiet.hoiDongNote ?? <span className="text-fg-subtle">Chưa chốt</span>}</dd>
                  <dt className="text-fg-subtle">Thẩm định</dt>
                  <dd>{chiTiet.thamDinhNote ?? <span className="text-fg-subtle">Chưa có</span>}</dd>
                  <dt className="text-fg-subtle">Quyết định</dt>
                  <dd>
                    {chiTiet.quyetDinhSo ?? <span className="text-fg-subtle">Chưa có</span>}
                    {chiTiet.tuQuyet && <Badge tone="gray" className="ml-1">tự quyết</Badge>}
                  </dd>
                </dl>
                <div className="mb-4 flex flex-wrap gap-2">
                  {V4up(chiTiet, canReview, canDecide).map((a) => (
                    <Tip key={a.action} text={`Theo ${a.legal}`}>
                      <span>
                        <Button variant={a.action === "execute" ? "danger" : "primary"} icon={a.action === "execute" ? "huy" : "duyet"} disabled={busy} onClick={() => void hanhDong(chiTiet, a.action)}>
                          {a.label}
                        </Button>
                      </span>
                    </Tip>
                  ))}
                  {V4up(chiTiet, canReview, canDecide).length === 0 && chiTiet.status !== "da_huy" && <span className="text-sm text-fg-subtle">Đang chờ người có quyền ở bước này.</span>}
                </div>
                <h4 className="mb-2 text-sm font-semibold">Tài liệu trong đợt ({fmtN(chiTiet.entries.length)})</h4>
                <Table columns={colsTrongDot} rows={chiTiet.entries} getRowId={(e) => e.id} dense maxHeight={320} />
                <h4 className="mb-2 mt-4 text-sm font-semibold">Vết quá trình</h4>
                {chiTiet.log.length === 0 ? (
                  <EmptyState compact title="Chưa có vết nào" />
                ) : (
                  <ol className="relative ml-2 border-l border-border-subtle pl-4 text-sm">
                    {chiTiet.log.map((l, i) => (
                      <li key={i} className="relative mb-2">
                        <span className="absolute -left-[21px] top-1.5 size-2.5 rounded-full border-2 border-surface bg-brand-500" aria-hidden />
                        <span className="text-fg-subtle">{new Date(l.createdAt).toLocaleString("vi-VN")}</span> · {V4uo[l.action] ?? l.action}
                        {l.detail ? <span className="text-fg-muted">: {l.detail}</span> : null}
                      </li>
                    ))}
                  </ol>
                )}
              </Card>
            )}
          </>
        )}
      </div>
      <Help title="Căn cứ" className="mt-4">
        <p>Danh mục tài liệu hết giá trị: trùng lặp theo Điều 48.2, hết thời hạn lưu trữ theo Điều 48.3, lập danh mục theo Điều 48.4. Đợt xét huỷ: Hội đồng theo Điều 49.2, thẩm định theo Điều 49.4, quyết định theo Điều 49.6, huỷ trong hệ thống theo Điều 49.7, tự quyết theo Điều 49.8 khi không thuộc nguồn nộp lưu trữ lịch sử.</p>
      </Help>
      {moHan && <HanLuuTruDialog onClose={() => setMoHan(false)} />}
    </section>
  );
}
