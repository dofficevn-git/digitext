// Copyright (c) 2026 DigiText. All rights reserved.
// Đợt đã đẩy lên máy chủ: ma trận phông × máy quản trị và danh sách các đợt nộp. Giữ dữ liệu cũ khi tải lại.

import { useCallback } from "react";
import { V41t } from "@/live/useLive";
import "./khogom.css";
import { V4pb, V4pf, V4ph, V4pj, V4pk, type V4pa, type V4pe, type V4pi } from "@/api/uploadBatches";
import { Icon } from "@/components/Icon";
import { Badge, Card, EmptyState, ErrorBanner, IconButton, Skeleton, Table, Tip, type Column } from "@/ui";

const TONE: Record<V4pi, { tone: "ok" | "gray" | "warn"; label: string }> = {
  ok: { tone: "ok", label: "Hoàn tất" },
  in_progress: { tone: "gray", label: "Đang gửi" },
  attention: { tone: "warn", label: "Cần kiểm tra" },
};

const fmt = (iso: string) => {
  const d = new Date(iso);
  return Number.isNaN(d.getTime()) ? iso : d.toLocaleString("vi-VN");
};
const EMPTY: V4pe = { admins: [], projects: [] };

export default function V4pp({ isSuper }: { isSuper: boolean }) {
  const load = useCallback(async () => {
    const [m, b] = await Promise.allSettled([V4pf(), V4pb()]);
    if (m.status === "rejected" && b.status === "rejected") throw m.reason;
    return { view: V4ph(m.status === "fulfilled" ? m.value : EMPTY), batches: b.status === "fulfilled" ? b.value : ([] as V4pa[]) };
  }, []);
  const { data, error, loading, refreshing, reload } = V41t(load, { topics: ["uploadBatches", "docs"] });
  const view = data?.view;
  const batches = data?.batches ?? [];

  const cols: Column<V4pa>[] = [
    { key: "createdAt", header: "Thời điểm", cell: (b) => fmt(b.createdAt), sortable: true, width: 160 },
    ...(isSuper ? [{ key: "forwardedByName", header: "Máy quản trị", cell: (b: V4pa) => b.forwardedByName ?? "—", sortable: true } as Column<V4pa>] : []),
    { key: "status", header: "Tình trạng", width: 130, cell: (b) => { const t = TONE[V4pj(b)]; return <Badge tone={t.tone}>{t.label}</Badge>; } },
    { key: "received", header: "Nhận / gửi", align: "right", width: 110, cell: (b) => <><b className={b.received < b.forwarded ? "text-warning-700" : undefined}>{b.received}</b> / {b.forwarded}</> },
    { key: "projectCount", header: "Phông", align: "right", width: 80 },
    { key: "detail", header: "Chi tiết", className: "whitespace-normal text-fg-subtle", cell: (b) => `${V4pk(b)}${b.note ? ` · ${b.note}` : ""}` },
  ];

  return (
    <div className="space-y-4">
      <div className="flex items-center gap-2">
        <p className="text-sm text-fg-subtle">{isSuper ? "Mỗi máy quản trị đẩy tài liệu lên máy chủ theo từng phông." : "Tài liệu bạn và các quản trị cùng dự án đã đẩy lên máy chủ, theo từng phông."}</p>
        <div className="flex-1" />
        <IconButton label="Tải lại" icon="lamMoi" variant="secondary" size="sm" loading={refreshing} onClick={reload} />
      </div>
      {error && data && <ErrorBanner error={error} onRetry={reload} />}

      <Card title="Tài liệu đã nộp lên máy chủ" description={view ? `${view.grandTotal.toLocaleString("vi-VN")} tài liệu · ${view.admins.length} máy quản trị · ${view.rows.length} phông` : undefined} padding="none">
        {loading && !data ? (
          <Skeleton.Rows rows={4} cols={4} />
        ) : !view || view.isEmpty ? (
          <EmptyState compact icon="taiLen" title="Chưa có đợt nộp nào" description="Ở màn Đưa lên máy chủ, bấm Đẩy lên máy chủ để bắt đầu." />
        ) : (
          <div className="tbl-scroll kg-matrix">
            <table className="tbl kg-table">
              <thead>
                <tr>
                  <th className="kg-corner">Phông</th>
                  {view.admins.map((a) => (
                    <th key={a.id} className="kg-colhead">
                      <div className="kg-colname">
                        <Icon name="mayTram" />
                        <b>{a.name ?? "—"}</b>
                        {a.attention && (
                          <Tip text="Có đợt máy chủ nhận thiếu, xem danh sách bên dưới">
                            <span className="text-warning-700">
                              <Icon name="canhBao" size={13} label="Cần kiểm tra" />
                            </span>
                          </Tip>
                        )}
                      </div>
                      <div className="text-fg-subtle">{view.columnTotals[a.id] ?? 0} tài liệu</div>
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {view.rows.map(({ project, total }) => (
                  <tr key={project.id}>
                    <th className="kg-rowhead kg-hoso">
                      <div>
                        <Icon name="thuMuc" /> {project.name}
                      </div>
                      <div className="text-fg-subtle">
                        {project.code} · {total} tài liệu
                      </div>
                    </th>
                    {view.admins.map((a) => {
                      const c = project.cells[a.id] ?? 0;
                      return (
                        <td key={a.id} className="kg-cell text-center">
                          {c > 0 ? <b>{c}</b> : <span className="text-fg-subtle">—</span>}
                        </td>
                      );
                    })}
                  </tr>
                ))}
              </tbody>
              <tfoot>
                <tr className="kg-grouprow">
                  <th className="kg-rowhead">Tổng</th>
                  {view.admins.map((a) => (
                    <td key={a.id} className="text-center">
                      <b>{view.columnTotals[a.id] ?? 0}</b>
                    </td>
                  ))}
                </tr>
              </tfoot>
            </table>
          </div>
        )}
        <p className="border-t border-border-subtle px-4 py-2 text-xs text-fg-subtle">{isSuper ? "Chỉ đếm tài liệu đẩy theo đợt; tải lẻ không có máy gửi nên không nằm trong bảng." : "Cột gồm cả quản trị cùng phụ trách phông với bạn."}</p>
      </Card>

      <Card title="Các đợt nộp" description={`${batches.length.toLocaleString("vi-VN")} đợt`} padding="none">
        <Table columns={cols} rows={batches} getRowId={(b) => b.id} state={{ loading: loading && !data, empty: { title: "Chưa có đợt nộp", description: "Mỗi lần bấm Đẩy lên máy chủ ở màn Đưa lên máy chủ là một đợt." } }} />
      </Card>
    </div>
  );
}
