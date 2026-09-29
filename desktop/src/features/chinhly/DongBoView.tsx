// Copyright (c) 2026 DigiText. All rights reserved.
// Đồng bộ và xung đột: trạng thái engine, xung đột (chọn theo trường), thay đổi chờ đẩy / bị từ chối (thử lại, bỏ).
// Chuyển dữ liệu cũ nằm ở Cài đặt › Dữ liệu.

import { useCallback, useState } from "react";
import { isTauri } from "@/api/ipc";
import { V41t } from "@/live/useLive";
import { clConflictResolve, clConflicts, clOutbox, clOutboxDiscard, clOutboxRetry, clSyncNow, str, type ClConflict, type ClOutboxRow } from "@/api/chinhly";
import { ChuaChonDuAn, fmtTime, n, SyncBadge, syncLabel, useClSync } from "./common";
import { Icon } from "@/components/Icon";
import { Badge, Banner, Button, Card, Collapsible, CollapsibleContent, CollapsibleTrigger, confirm, EmptyState, ErrorBanner, Menu, notifyError, PageHeader, Seg, Table, toast, type Column } from "@/ui";

const TEN_BANG: Record<string, string> = {
  cl_ho_so: "Hồ sơ",
  cl_van_ban: "Văn bản",
  cl_van_ban_file: "Tệp văn bản",
  cl_phong: "Phông",
  cl_phong_ban: "Phòng ban",
  cl_mang: "Mảng công việc",
  cl_thbq: "Thời hạn bảo quản",
  cl_ngon_ngu: "Ngôn ngữ",
  cl_loai_van_ban: "Loại văn bản",
  cl_khoi_phong: "Khối phông",
};
const NHAN: Record<string, string> = {
  so_tam: "Số tạm",
  tieu_de: "Tiêu đề",
  so_to: "Số tờ",
  ngay_bat_dau: "Ngày bắt đầu",
  ngay_ket_thuc: "Ngày kết thúc",
  ghi_chu: "Ghi chú",
  trich_yeu: "Trích yếu",
  so_thu_tu: "STT",
  so_ky_hieu: "Ký hiệu",
  ngay_thang: "Ngày tháng",
  co_quan_ban_hanh: "Cơ quan ban hành",
};
const nhan = (k: string) => NHAN[k] ?? k;
const mota = (r: Record<string, unknown> | null | undefined) => (r ? [str(r as never, "so_tam") && `số tạm ${str(r as never, "so_tam")}`, str(r as never, "tieu_de"), str(r as never, "trich_yeu"), str(r as never, "ten")].filter(Boolean).join(" · ") : "");

function XungDot({ c, onDone, defaultOpen }: { c: ClConflict; onDone: () => void; defaultOpen: boolean }) {
  const [open, setOpen] = useState(defaultOpen);
  const [pick, setPick] = useState<Record<string, "mine" | "theirs">>(() => Object.fromEntries(c.fields.map((f) => [f, "mine"])));
  const [busy, setBusy] = useState(false);
  const daXoa = !c.theirs || !!c.theirs.deleted_at;
  const chot = async (choice: "mine" | "theirs" | "merged") => {
    setBusy(true);
    try {
      let merged: Record<string, unknown> | undefined;
      if (choice === "merged") {
        merged = { ...c.mine };
        for (const [f, who] of Object.entries(pick)) if (who === "theirs" && c.theirs) merged[f] = c.theirs[f];
      }
      await clConflictResolve(c.id, choice, merged);
      toast.success("Đã giải quyết xung đột.");
      onDone();
    } catch (e) {
      notifyError(e, "giải quyết xung đột");
    } finally {
      setBusy(false);
    }
  };
  const cols: Column<string>[] = [
    { key: "f", header: "Trường", width: 140, cell: (f) => nhan(f) },
    { key: "base", header: "Bản gốc lúc kéo về", className: "text-fg-subtle whitespace-normal", cell: (f) => String(c.base?.[f] ?? "") },
    { key: "mine", header: "Của tôi", className: "whitespace-normal", cell: (f) => String(c.mine[f] ?? "") },
    { key: "theirs", header: "Máy chủ", className: "whitespace-normal", cell: (f) => String(c.theirs?.[f] ?? "") },
    { key: "pick", header: "Giữ", width: 180, cell: (f) => <Seg<"mine" | "theirs"> value={pick[f]} onChange={(v) => setPick((p) => ({ ...p, [f]: v }))} options={[{ value: "mine", label: "Của tôi" }, { value: "theirs", label: "Máy chủ" }]} /> },
  ];
  return (
    <Collapsible open={open} onOpenChange={setOpen} className="rounded-md border border-border-subtle">
      <CollapsibleTrigger className="flex w-full flex-wrap items-center gap-2 px-3 py-2 text-left hover:bg-surface-hover">
        <Icon name={open ? "xuong" : "moNhanh"} size={14} className="text-fg-subtle" />
        <b>{TEN_BANG[c.tbl] ?? c.tbl}</b>
        <span className="text-sm text-fg-subtle">{mota(c.mine)}</span>
        <Badge tone="warn">{daXoa ? "Đã xoá trên máy chủ" : `${c.fields.length} trường khác nhau`}</Badge>
        <span className="ml-auto text-xs text-fg-subtle">{fmtTime(c.createdAt)}</span>
      </CollapsibleTrigger>
      <CollapsibleContent className="border-t border-border-subtle p-3">
        {daXoa ? (
          <p className="text-sm">Bản ghi này đã bị xoá trên máy chủ. Tạo lại từ bản của bạn hay chấp nhận đã xoá?</p>
        ) : (
          <Table columns={cols} rows={c.fields} getRowId={(f) => f} dense />
        )}
        <div className="mt-3 flex flex-wrap gap-2">
          {daXoa ? (
            <>
              <Button size="sm" variant="primary" loading={busy} onClick={() => void chot("mine")}>
                Tạo lại từ bản của tôi
              </Button>
              <Button size="sm" disabled={busy} onClick={() => void chot("theirs")}>
                Chấp nhận đã xoá
              </Button>
            </>
          ) : (
            <>
              <Button size="sm" variant="primary" loading={busy} onClick={() => void chot("merged")}>
                Áp dụng lựa chọn theo trường
              </Button>
              <Button size="sm" disabled={busy} onClick={() => void chot("mine")}>
                Giữ toàn bộ của tôi
              </Button>
              <Button size="sm" disabled={busy} onClick={() => void chot("theirs")}>
                Lấy bản máy chủ
              </Button>
            </>
          )}
        </div>
      </CollapsibleContent>
    </Collapsible>
  );
}

export default function DongBoView() {
  const sync = useClSync();
  const loader = useCallback(async () => {
    if (!isTauri()) return { outbox: [] as ClOutboxRow[], conflicts: [] as ClConflict[] };
    const [outbox, conflicts] = await Promise.all([clOutbox(), clConflicts()]);
    return { outbox, conflicts };
  }, []);
  const { data, error, reload } = V41t(loader, { topics: ["clSync", "chinhLy"], errorText: () => "Không đọc được hàng đợi đồng bộ." });
  const outbox = data?.outbox ?? [];
  const conflicts = data?.conflicts ?? [];
  const [busy, setBusy] = useState(false);
  const sl = syncLabel(sync);

  const dongBo = async () => {
    setBusy(true);
    try {
      await clSyncNow();
      reload();
    } catch (e) {
      notifyError(e, "đồng bộ");
    } finally {
      setBusy(false);
    }
  };
  const bo = async (o: ClOutboxRow) => {
    const ok = await confirm({ title: "Bỏ thay đổi này?", body: "Bản ghi quay về bản trên máy chủ.", confirmLabel: "Bỏ", danger: true });
    if (!ok) return;
    try {
      await clOutboxDiscard(o.id);
      reload();
    } catch (e) {
      notifyError(e, "bỏ thay đổi");
    }
  };

  const cols: Column<ClOutboxRow>[] = [
    { key: "tbl", header: "Bảng", width: 140, cell: (o) => TEN_BANG[o.tbl] ?? o.tbl },
    { key: "row", header: "Bản ghi", className: "max-w-[420px] whitespace-normal", cell: (o) => mota(o.new) || mota(o.base) || o.rowId },
    { key: "op", header: "Thao tác", width: 90, cell: (o) => (o.op === "delete" ? "Xoá" : o.base ? "Sửa" : "Thêm") },
    {
      key: "status",
      header: "Trạng thái",
      width: 130,
      cell: (o) => (
        <>
          <Badge tone={o.status === "pending" ? "info" : "err"}>{o.status === "pending" ? "Chờ đẩy" : "Bị từ chối"}</Badge>
          {o.attempts > 0 && <span className="ml-1 text-xs text-fg-subtle">{o.attempts} lần</span>}
        </>
      ),
    },
    { key: "lastError", header: "Lỗi", className: "whitespace-normal text-sm text-danger-700", cell: (o) => o.lastError },
    { key: "at", header: "Sửa lúc", width: 110, className: "text-sm", cell: (o) => fmtTime(o.editedAt ?? o.createdAt) },
  ];

  return (
    <section className="view active">
      <PageHeader
        title="Đồng bộ và xung đột"
        description="Dữ liệu chỉnh lý giữa máy này và máy chủ."
        meta={<SyncBadge sync={sync} />}
        actions={
          <Button variant="primary" icon="lamMoi" loading={busy} onClick={() => void dongBo()}>
            Đồng bộ ngay
          </Button>
        }
      />
      <ChuaChonDuAn />
      {sync?.lastError && <Banner tone="danger" className="mb-4" title="Lần đồng bộ gần nhất lỗi">{sync.lastError}</Banner>}
      {error != null && <ErrorBanner error={error} className="mb-4" onRetry={reload} />}
      <p className="mb-4 text-sm text-fg-subtle">{sl.title}</p>

      <Card className="mb-4" title={`Xung đột cần bạn chọn (${n(conflicts.length)})`} description="Sửa khác trường với người khác thì máy chủ tự gộp, chỉ cùng trường khác giá trị mới hỏi.">
        {conflicts.length === 0 ? (
          <EmptyState compact icon="duyet" title="Không có xung đột" />
        ) : (
          <div className="space-y-2">
            {conflicts.map((c, i) => (
              <XungDot key={c.id} c={c} onDone={reload} defaultOpen={i === 0} />
            ))}
          </div>
        )}
      </Card>

      <Card padding="none" title={`Thay đổi chờ đẩy hoặc bị từ chối (${n(outbox.length)})`}>
        <Table
          columns={cols}
          rows={outbox}
          getRowId={(o) => String(o.id)}
          dense
          rowActions={(o) => <Menu size="sm" items={[...(o.status !== "pending" ? [{ label: "Thử lại", icon: "lamLai" as const, onSelect: () => void clOutboxRetry(o.id).then(reload).catch((e) => notifyError(e, "thử lại")) }] : []), { label: "Bỏ thay đổi", icon: "bo" as const, danger: true, onSelect: () => void bo(o) }]} />}
          state={{ empty: { title: "Mọi thay đổi đã lên máy chủ", icon: "duyet" } }}
        />
      </Card>
    </section>
  );
}
