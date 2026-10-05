// Copyright (c) 2026 DigiText. All rights reserved.
// Ví dụ dạy máy bóc trường: xem, lọc, tắt ví dụ sai (5/10/2026).
// Khi bóc một trang, máy chủ đưa cho Qwen 3 ví dụ có chữ đầu trang giống nhất (ưu tiên cùng dự án).
// Ví dụ sinh ra khi người có quyền duyệt xác nhận trường; ví dụ sai được dùng nhiều thì phải tắt trước.

import { useCallback, useEffect, useState } from "react";
import { kieExampleSetActive, kieExamplesList, type KieExample, type KieScope } from "@/api/kie";
import type { V4a9 } from "@/api/users";
import { V41t } from "@/live/useLive";
import { V4fn, V4fo } from "@/features/upload/ocrLogic";
import { Badge, Button, Card, Dialog, Field, Input, notifyError, Seg, Select, Table, toast, Toolbar, type Column } from "@/ui";

const fmtN = (n: number) => n.toLocaleString("vi-VN");
const fmtTime = (s: string | null) => (s ? new Date(s).toLocaleString("vi-VN", { dateStyle: "short", timeStyle: "short" }) : "—");

type ActiveFilter = "on" | "off" | "all";

function nguon(e: KieExample) {
  if (e.projectCode) return <Badge tone="info">{e.projectCode}</Badge>;
  return <Badge tone="gray">Chung · {e.source.split("/")[0]}</Badge>;
}

function tomTat(e: KieExample) {
  const f = e.fields ?? {};
  const dau = [f.loaiVanBan, f.soKyHieu, f.ngayThang].filter((x) => x && String(x).trim()).join(" · ");
  return (
    <div className="min-w-0">
      <div className="truncate">{dau || <span className="text-fg-subtle">Không có loại / số / ngày</span>}</div>
      {f.trichYeu && <div className="truncate text-sm text-fg-subtle">{f.trichYeu}</div>}
    </div>
  );
}

export default function KieExamplesPanel({ projectId, projects }: { projectId: string; projects: V4a9[] }) {
  const [duAn, setDuAn] = useState(projectId);
  useEffect(() => setDuAn(projectId), [projectId]);
  const [scope, setScope] = useState<KieScope>("all");
  const [active, setActive] = useState<ActiveFilter>("on");
  const [order, setOrder] = useState<"used" | "recent">("used");
  const [q, setQ] = useState("");
  const [qDebounced, setQDebounced] = useState("");
  useEffect(() => {
    const t = setTimeout(() => setQDebounced(q.trim()), 400);
    return () => clearTimeout(t);
  }, [q]);
  const [page, setPage] = useState(1);
  const [pageSize, setPageSize] = useState(50);
  useEffect(() => setPage(1), [duAn, scope, active, order, qDebounced, pageSize]);
  const [xem, setXem] = useState<KieExample | null>(null);
  const [dangDoi, setDangDoi] = useState<string | null>(null);

  const load = useCallback(
    () =>
      kieExamplesList({
        projectId: duAn || null,
        scope,
        active: active === "all" ? null : active === "on",
        q: qDebounced,
        order,
        limit: pageSize,
        offset: (page - 1) * pageSize,
      }),
    [duAn, scope, active, qDebounced, order, page, pageSize],
  );
  const { data, error, loading, reload, setData } = V41t(load, { topics: ["ocrFields"], errorText: () => "Không tải được danh sách ví dụ." });

  async function doiTrangThai(e: KieExample) {
    setDangDoi(e.id);
    try {
      await kieExampleSetActive(e.id, !e.active);
      setData((d) => ({ total: d?.total ?? 0, items: (d?.items ?? []).map((x) => (x.id === e.id ? { ...x, active: !e.active } : x)) }));
      setXem((x) => (x && x.id === e.id ? { ...x, active: !e.active } : x));
      toast.success(e.active ? "Đã tắt ví dụ — máy không dùng ví dụ này nữa" : "Đã bật lại ví dụ");
    } catch (err) {
      notifyError(err, "Không đổi được trạng thái ví dụ");
    } finally {
      setDangDoi(null);
    }
  }

  const cols: Column<KieExample>[] = [
    { key: "nguon", header: "Nguồn", cell: nguon, width: 130 },
    {
      key: "taiLieu",
      header: "Tài liệu",
      cell: (e) => (
        <div className="min-w-0">
          <div className="truncate">{e.filename ?? e.extRef ?? "—"}</div>
          {e.pageNo != null && <div className="text-xs text-fg-subtle">trang {e.pageNo}</div>}
        </div>
      ),
      width: 260,
    },
    { key: "noiDung", header: "Giá trị đúng", cell: tomTat },
    { key: "usedCount", header: "Lượt dùng", align: "right", cell: (e) => fmtN(e.usedCount), width: 90 },
    { key: "lastUsedAt", header: "Dùng gần nhất", cell: (e) => fmtTime(e.lastUsedAt), width: 140, hideBelow: 1200 },
    { key: "active", header: "Trạng thái", cell: (e) => (e.active ? <Badge tone="ok">Đang dùng</Badge> : <Badge tone="gray">Đã tắt</Badge>), width: 110 },
  ];

  const total = data?.total ?? 0;
  return (
    <Card
      title="Ví dụ dạy máy bóc trường"
      description="Khi bóc một trang, máy đưa cho Qwen 3 ví dụ có đầu trang giống nhất, ưu tiên cùng dự án. Ví dụ sinh ra khi người duyệt xác nhận trường. Ví dụ sai mà được dùng nhiều thì tắt trước."
      padding="none"
    >
      <Toolbar className="px-4 py-3">
        <Select
          size="sm"
          className="w-56"
          value={duAn}
          onChange={setDuAn}
          placeholder="Tất cả dự án"
          options={projects.map((p) => ({ value: p.id, label: `${p.code} · ${p.name}` }))}
          aria-label="Dự án"
        />
        <Seg<KieScope>
          value={scope}
          onChange={setScope}
          options={[
            { value: "all", label: "Tất cả" },
            { value: "project", label: "Của dự án" },
            { value: "general", label: "Ví dụ chung" },
          ]}
        />
        <Seg<ActiveFilter>
          value={active}
          onChange={setActive}
          options={[
            { value: "on", label: "Đang dùng" },
            { value: "off", label: "Đã tắt" },
            { value: "all", label: "Mọi trạng thái" },
          ]}
        />
        <Select
          size="sm"
          className="w-44"
          value={order}
          onChange={(v) => setOrder(v === "recent" ? "recent" : "used")}
          options={[
            { value: "used", label: "Dùng nhiều nhất" },
            { value: "recent", label: "Mới cập nhật" },
          ]}
          aria-label="Sắp xếp"
        />
        <Field label="Tìm" inline>
          <Input size="sm" prefix="tim" value={q} onChange={(e) => setQ(e.target.value)} placeholder="Chữ trong ví dụ, tên tệp…" />
        </Field>
        <span className="text-sm text-fg-subtle">{fmtN(total)} ví dụ</span>
      </Toolbar>
      <Table
        columns={cols}
        rows={data?.items ?? []}
        getRowId={(e) => e.id}
        onRowClick={setXem}
        rowClassName={(e) => (e.active ? undefined : "opacity-60")}
        rowActions={(e) => (
          <Button size="sm" variant={e.active ? "ghost" : "secondary"} loading={dangDoi === e.id} onClick={() => void doiTrangThai(e)}>
            {e.active ? "Tắt" : "Bật lại"}
          </Button>
        )}
        state={{
          loading: loading && !data,
          error,
          onRetry: reload,
          empty: { title: "Chưa có ví dụ nào", description: "Ví dụ sinh ra khi người có quyền duyệt xác nhận trường của một tài liệu." },
        }}
        pagination={{ page, pageSize, total, onChange: setPage, onPageSizeChange: setPageSize }}
      />
      <Dialog
        open={xem != null}
        onOpenChange={(o) => !o && setXem(null)}
        size="lg"
        title={xem ? (xem.filename ?? xem.extRef ?? "Ví dụ") : ""}
        description={xem ? `${xem.projectCode ?? `Ví dụ chung · ${xem.source}`} · dùng ${fmtN(xem.usedCount)} lần · ${xem.createdByName ? `duyệt bởi ${xem.createdByName}` : "nhập sẵn"}` : undefined}
        footer={
          xem && (
            <Button variant={xem.active ? "danger" : "primary"} loading={dangDoi === xem.id} onClick={() => void doiTrangThai(xem)}>
              {xem.active ? "Tắt ví dụ" : "Bật lại"}
            </Button>
          )
        }
      >
        {xem && (
          <div className="grid gap-4 md:grid-cols-2">
            <div>
              <h3 className="mb-2 text-sm font-medium">Giá trị đúng</h3>
              <dl className="grid grid-cols-[9rem_1fr] gap-x-3 gap-y-1 text-sm">
                {V4fn.filter((k) => xem.fields?.[k] != null && String(xem.fields[k]).trim() !== "").map((k) => (
                  <div key={k} className="contents">
                    <dt className="text-fg-subtle">{V4fo(k)}</dt>
                    <dd className="break-words">{String(xem.fields[k])}</dd>
                  </div>
                ))}
              </dl>
            </div>
            <div>
              <h3 className="mb-2 text-sm font-medium">Chữ OCR đầu trang (phần dùng để so khớp)</h3>
              <pre className="max-h-96 overflow-auto whitespace-pre-wrap rounded border border-border-subtle bg-gray-50 p-2 text-xs">{xem.ocrText}</pre>
            </div>
          </div>
        )}
      </Dialog>
    </Card>
  );
}
