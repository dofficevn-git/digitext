// Copyright (c) 2026 DigiText. All rights reserved.
// Cơ quan nộp lưu (0040, đợt 6 — Digifile D11 / D12): danh mục cơ quan nộp vào lưu trữ lịch sử của kho đang chọn, tài khoản gắn
// cơ quan (chỉ thấy đợt nộp của mình) và khoá API cho hệ thống của cơ quan gọi đăng ký / nộp gói bằng header X-Api-Key.

import { useCallback, useState } from "react";
import { isTauri } from "@/api/ipc";
import { coQuanNopCapKey, coQuanNopList, coQuanNopSua, coQuanNopTaiKhoan, coQuanNopTao, coQuanNopThuHoiKey, coQuanNopXoa, type CoQuanNop, type CoQuanNopMoi, type TaiKhoanCoQuan } from "@/api/coQuanNop";
import { V41t } from "@/live/useLive";
import { V460 } from "@/state";
import { useProject } from "@/state/project";
import { V3z9 } from "@/domain/permissions";
import { Badge, Button, Card, confirm, Dialog, EmptyState, ErrorBanner, Field, FormGrid, IconButton, Input, Menu, notifyError, PageHeader, Table, Textarea, toast, Toolbar, type Column } from "@/ui";

export default function CoQuanNopView() {
  const { session } = V460();
  const has = (p: Parameters<typeof V3z9>[1]) => V3z9(session?.permissions, p);
  const canEdit = has("submission.review");
  const { projectId } = useProject();
  const duAn = session?.projectId ?? projectId ?? null;
  const [form, setForm] = useState<{ row: CoQuanNop | null } | null>(null);
  const [key, setKey] = useState<{ ten: string; apiKey: string } | null>(null);
  const [tk, setTk] = useState<{ row: CoQuanNop; rows: TaiKhoanCoQuan[] } | null>(null);
  const [busy, setBusy] = useState(false);

  const load = useCallback(() => (duAn ? coQuanNopList(duAn) : Promise.resolve([] as CoQuanNop[])), [duAn]);
  const { data, error, loading, refreshing, reload } = V41t(load, { topics: ["submissions", "users"], enabled: isTauri() && !!duAn });
  const rows = data ?? [];

  const run = async (ctx: string, f: () => Promise<void>) => {
    setBusy(true);
    try {
      await f();
      reload();
    } catch (e) {
      notifyError(e, ctx);
    } finally {
      setBusy(false);
    }
  };

  const capKey = async (r: CoQuanNop) => {
    const ok = await confirm({
      title: r.coApiKey ? `Cấp lại khoá API cho ${r.ten}?` : `Cấp khoá API cho ${r.ten}?`,
      body: r.coApiKey ? "Khoá cũ sẽ hết hiệu lực ngay. Hệ thống của cơ quan phải cập nhật khoá mới." : "Khoá chỉ hiện một lần, hãy chép và gửi cho cơ quan qua kênh an toàn.",
      confirmLabel: "Cấp khoá",
    });
    if (!ok) return;
    await run("cấp khoá API", async () => {
      const k = await coQuanNopCapKey(r.id);
      setKey({ ten: r.ten, apiKey: k.apiKey });
    });
  };
  const thuHoi = async (r: CoQuanNop) => {
    const ok = await confirm({ title: `Thu hồi khoá API của ${r.ten}?`, body: "Hệ thống của cơ quan sẽ không gọi được nữa cho đến khi cấp khoá mới.", confirmLabel: "Thu hồi", danger: true });
    if (!ok) return;
    await run("thu hồi khoá API", async () => {
      await coQuanNopThuHoiKey(r.id);
      toast.success("Đã thu hồi khoá API.");
    });
  };
  const xoa = async (r: CoQuanNop) => {
    const ok = await confirm({ title: `Xoá cơ quan ${r.ten}?`, body: "Chỉ xoá được khi cơ quan chưa có đợt nộp nào. Tài khoản gắn cơ quan sẽ được gỡ liên kết.", confirmLabel: "Xoá", danger: true });
    if (!ok) return;
    await run("xoá cơ quan nộp lưu", async () => {
      await coQuanNopXoa(r.id);
      toast.success("Đã xoá.");
    });
  };
  const xemTk = (r: CoQuanNop) => run("tải tài khoản", async () => setTk({ row: r, rows: await coQuanNopTaiKhoan(r.id) }));

  const cols: Column<CoQuanNop>[] = [
    { key: "ma", header: "Mã", width: 110, sortable: true, cell: (r) => <span className="font-medium text-fg-strong">{r.ma}</span> },
    { key: "ten", header: "Cơ quan", className: "whitespace-normal", sortable: true, cell: (r) => <div><div>{r.ten}</div>{r.diaChi && <div className="text-xs text-fg-subtle">{r.diaChi}</div>}</div> },
    { key: "lienHe", header: "Liên hệ", className: "whitespace-normal text-sm", cell: (r) => [r.nguoiLienHe, r.dienThoai, r.email].filter(Boolean).join(" · ") || "—" },
    { key: "soTaiKhoan", header: "Tài khoản", align: "right", width: 90 },
    { key: "soDotNop", header: "Đợt nộp", align: "right", width: 90 },
    { key: "api", header: "Khoá API", width: 150, cell: (r) => (r.coApiKey ? <Badge tone="ok">{r.apiKeyGoiAt ? `Đã dùng ${r.apiKeyGoiAt.slice(0, 10).split("-").reverse().join("/")}` : "Đã cấp"}</Badge> : <span className="text-fg-subtle">Chưa cấp</span>) },
  ];

  return (
    <section className="view active">
      <PageHeader
        title="Cơ quan nộp lưu"
        description="Cơ quan, tổ chức nộp tài liệu vào lưu trữ lịch sử"
        infoTitle="Cơ quan nộp lưu"
        info={
          <>
            <p>Mỗi cơ quan nộp lưu có một dòng ở đây. Tài khoản gắn cơ quan (Quản trị › Người dùng) chỉ thấy đợt nộp của cơ quan mình (Điều 20.1).</p>
            <p>Khoá API cho hệ thống của cơ quan tự đăng ký đợt nộp, nộp gói SIP và tra trạng thái qua <code>/api/v1/submissions</code> với header <code>X-Api-Key</code> (Điều 18.2). Khoá mang danh tính tài khoản đầu tiên của cơ quan, nên phải gắn tài khoản trước.</p>
          </>
        }
        actions={
          <>
            <IconButton label="Tải lại" icon="lamMoi" variant="secondary" loading={refreshing} onClick={reload} />
            {canEdit && (
              <Button variant="primary" icon="them" disabled={!duAn} onClick={() => setForm({ row: null })}>
                Thêm cơ quan
              </Button>
            )}
          </>
        }
      />
      {!duAn && <EmptyState title="Chưa chọn kho lưu trữ" description="Chọn kho lưu trữ (dự án) ở thanh trên." />}
      {error && data && <ErrorBanner error={error} onRetry={reload} className="mb-3" />}
      {duAn && (
        <Card padding="none">
          <Table
            columns={cols}
            rows={rows}
            getRowId={(r) => r.id}
            rowActions={(r) => (
              <Menu
                size="sm"
                items={[
                  { label: "Tài khoản của cơ quan", icon: "nguoiDung", onSelect: () => void xemTk(r) },
                  ...(canEdit
                    ? [
                        { label: "Sửa", icon: "sua" as const, onSelect: () => setForm({ row: r }) },
                        { label: r.coApiKey ? "Cấp lại khoá API" : "Cấp khoá API", icon: "kySo" as const, disabled: busy, onSelect: () => void capKey(r) },
                        ...(r.coApiKey ? [{ label: "Thu hồi khoá API", icon: "tuChoi" as const, danger: true, disabled: busy, onSelect: () => void thuHoi(r) }] : []),
                        "-" as const,
                        { label: "Xoá", icon: "xoa" as const, danger: true, disabled: busy || r.soDotNop > 0, onSelect: () => void xoa(r) },
                      ]
                    : []),
                ]}
              />
            )}
            state={{ loading: loading && !data, error: data ? undefined : error, onRetry: reload, empty: { title: "Chưa có cơ quan nộp lưu", description: canEdit ? "Bấm Thêm cơ quan. Sau đó tạo tài khoản cho cơ quan ở Quản trị › Người dùng." : undefined } }}
          />
        </Card>
      )}
      {form && duAn && (
        <FormDialog
          projectId={duAn}
          row={form.row}
          onClose={() => setForm(null)}
          onSaved={() => {
            setForm(null);
            reload();
          }}
        />
      )}
      {key && (
        <Dialog open onOpenChange={(o) => !o && setKey(null)} title={`Khoá API — ${key.ten}`} size="md" footer={<Button variant="primary" onClick={() => setKey(null)}>Đã chép</Button>}>
          <p className="mb-2 text-sm">Khoá chỉ hiện một lần. Gửi cho cơ quan qua kênh an toàn; hệ thống của họ gửi kèm header <code>X-Api-Key</code>.</p>
          <Input readOnly value={key.apiKey} onFocus={(e) => e.currentTarget.select()} />
          <Button className="mt-2" size="sm" onClick={() => void navigator.clipboard?.writeText(key.apiKey).then(() => toast.success("Đã chép khoá."))}>
            Chép
          </Button>
        </Dialog>
      )}
      {tk && (
        <Dialog open onOpenChange={(o) => !o && setTk(null)} title={`Tài khoản — ${tk.row.ten}`} size="md" footer={<Button onClick={() => setTk(null)}>Đóng</Button>}>
          {tk.rows.length === 0 ? (
            <EmptyState compact title="Chưa có tài khoản" description="Tạo ở Quản trị › Người dùng và chọn cơ quan nộp lưu này." />
          ) : (
            <Table columns={[{ key: "username", header: "Đăng nhập", width: 140 }, { key: "displayName", header: "Tên" }, { key: "isActive", header: "", width: 90, cell: (u: TaiKhoanCoQuan) => (u.isActive ? <Badge tone="ok">Hoạt động</Badge> : <Badge tone="gray">Khoá</Badge>) }]} rows={tk.rows} getRowId={(u) => u.id} dense />
          )}
        </Dialog>
      )}
    </section>
  );
}

function FormDialog({ projectId, row, onClose, onSaved }: { projectId: string; row: CoQuanNop | null; onClose: () => void; onSaved: () => void }) {
  const [f, setF] = useState<CoQuanNopMoi>({ ma: row?.ma ?? "", ten: row?.ten ?? "", diaChi: row?.diaChi ?? "", nguoiLienHe: row?.nguoiLienHe ?? "", dienThoai: row?.dienThoai ?? "", email: row?.email ?? "", ghiChu: row?.ghiChu ?? "" });
  const [busy, setBusy] = useState(false);
  const set = <K extends keyof CoQuanNopMoi>(k: K, v: CoQuanNopMoi[K]) => setF((o) => ({ ...o, [k]: v }));
  const ok = f.ma.trim() && f.ten.trim() && (!f.email || f.email.includes("@"));
  const luu = async () => {
    if (!ok || busy) return;
    setBusy(true);
    try {
      if (row) await coQuanNopSua(row.id, f);
      else await coQuanNopTao(projectId, f);
      toast.success(row ? "Đã lưu." : "Đã thêm cơ quan nộp lưu.");
      onSaved();
    } catch (e) {
      notifyError(e, "lưu cơ quan nộp lưu");
    } finally {
      setBusy(false);
    }
  };
  return (
    <Dialog
      open
      onOpenChange={(o) => !o && !busy && onClose()}
      title={row ? `Sửa ${row.ma}` : "Thêm cơ quan nộp lưu"}
      size="lg"
      footer={
        <>
          <Toolbar.Spacer />
          <Button onClick={onClose} disabled={busy}>
            Huỷ
          </Button>
          <Button variant="primary" loading={busy} disabled={!ok} onClick={() => void luu()}>
            Lưu
          </Button>
        </>
      }
    >
      <FormGrid cols={2}>
        <Field label="Mã cơ quan" required hint="Mã cơ quan, tổ chức theo Phụ lục I (viết hoa)" hintAs="tip">
          <Input value={f.ma} onChange={(e) => set("ma", e.target.value.toUpperCase())} autoFocus />
        </Field>
        <Field label="Tên cơ quan" required>
          <Input value={f.ten} onChange={(e) => set("ten", e.target.value)} />
        </Field>
        <Field label="Địa chỉ" className="col-span-2">
          <Input value={f.diaChi ?? ""} onChange={(e) => set("diaChi", e.target.value)} />
        </Field>
        <Field label="Người liên hệ">
          <Input value={f.nguoiLienHe ?? ""} onChange={(e) => set("nguoiLienHe", e.target.value)} />
        </Field>
        <Field label="Điện thoại">
          <Input value={f.dienThoai ?? ""} onChange={(e) => set("dienThoai", e.target.value)} />
        </Field>
        <Field label="Email" error={f.email && !f.email.includes("@") ? "Email không hợp lệ" : undefined}>
          <Input value={f.email ?? ""} onChange={(e) => set("email", e.target.value)} />
        </Field>
      </FormGrid>
      <Field label="Ghi chú" className="mt-3">
        <Textarea rows={2} value={f.ghiChu ?? ""} onChange={(e) => set("ghiChu", e.target.value)} />
      </Field>
    </Dialog>
  );
}
