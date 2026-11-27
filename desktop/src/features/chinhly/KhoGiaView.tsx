// Copyright (c) 2026 DigiText. All rights reserved.
// Kho và giá lưu trữ (chỉ online): kho (tên, sức chứa), giá = dải hộp [từ–đến] của một phòng ban trong kho, có sức chứa và ngưỡng
// cảnh báo gần đầy (0035, 27/11/2026); hồ sơ thuộc giá suy ra từ số hộp hoặc gán tay qua gia_id. Tab "Gán hộp": hồ sơ chưa có hộp
// của một phông (ưu tiên hồ sơ nhập từ gói AIP) — đặt phòng ban rồi gán số hộp từng dòng hoặc dán từ bảng tính.

import { useCallback, useEffect, useMemo, useState } from "react";
import { isTauri } from "@/api/ipc";
import { V41t } from "@/live/useLive";
import { V460 } from "@/state";
import { useProject } from "@/state/project";
import { V3z9 } from "@/domain/permissions";
import { clRemoteDelete, clRemoteSave, str, num, type ClRow } from "@/api/chinhly";
import { kgChuaHop, kgGanHop, kgTongHop, kgXuatHoSo, type GiaTongHop, type GanHopKetQua } from "@/api/khoGia";
import { CanMayChu, ChuaChonDuAn, n, Select, tenPhong, useClCatalog, useClSync } from "./common";
import { Badge, Banner, Button, Card, confirm, Dialog, ErrorBanner, Field, FormGrid, Input, Menu, notifyError, PageHeader, Select as UiSelect, Table, Tabs, Textarea, toast, type Column } from "@/ui";

type Tab = "kho" | "gia" | "ganhop";

export default function KhoGiaView() {
  const { session } = V460();
  const projectId = useProject().projectId ?? ""; // dự án đang chọn (super admin không gắn dự án trong phiên)
  const can = V3z9(session?.permissions, "khogia.manage");
  const canGan = can || V3z9(session?.permissions, "dossier.systematize");
  const { cat, reload } = useClCatalog();
  const sync = useClSync();
  const online = sync?.phase !== "offline";
  const [tab, setTab] = useState<Tab>("kho");
  const [form, setForm] = useState<{ row: ClRow | null } | null>(null);
  const [d, setD] = useState<Record<string, unknown>>({});
  const [err, setErr] = useState<unknown>(null);
  const [busy, setBusy] = useState(false);
  const [phongId, setPhongId] = useState("");
  const g =(k: string) => (d[k] === null || d[k] === undefined ? "" : String(d[k]));
  const set = (k: string, v: unknown) => setD((p) => ({ ...p, [k]: v }));

  // Tổng hợp giá (0035): sức chứa, hộp / hồ sơ đang chứa, % đầy — một câu máy chủ thay cho đếm từng giá.
  const thLoader = useCallback(async () => {
    if (!isTauri() || !online || !projectId) return { rows: [] as GiaTongHop[], ganDay: 0, vuot: 0 };
    return kgTongHop(projectId, undefined, phongId || undefined);
  }, [online, projectId, phongId]);
  const th = V41t(thLoader, { topics: ["khogia", "chinhLy"], errorText: () => "Không tải được tình trạng giá." });
  const gias = useMemo(() => th.data?.rows ?? [], [th.data]);
  const giaTheoKho = useMemo(() => {
    const m = new Map<string, GiaTongHop[]>();
    for (const x of gias) m.set(x.kho_id, [...(m.get(x.kho_id) ?? []), x]);
    return m;
  }, [gias]);

  const mo = (row: ClRow | null) => {
    setD(row ? { ...row } : tab === "gia" ? { kho_id: cat.kho[0]?.id ?? "", canh_bao_pct: 90 } : {});
    setForm({ row });
    setErr(null);
  };
  const luu = async () => {
    setBusy(true);
    setErr(null);
    try {
      const bang = tab === "gia" ? "gia" : "kho";
      const keys = bang === "kho" ? ["ten", "ghi_chu", "so_tt", "suc_chua_hop"] : ["kho_id", "phong_ban_id", "ten", "tu_hop", "den_hop", "ghi_chu", "so_tt", "suc_chua_hop", "canh_bao_pct"];
      const body: Record<string, unknown> = {};
      for (const k of keys) body[k] = d[k] === "" ? null : d[k];
      if (bang === "gia" && (body.canh_bao_pct === null || body.canh_bao_pct === undefined)) body.canh_bao_pct = 90;
      // Giá áp cho loại nào (0015): hộp 01 vĩnh viễn và hộp 01 có thời hạn là hai hộp khác nhau.
      if (bang === "gia") body.vinh_vien = d.vinh_vien === true || d.vinh_vien === "1" ? true : d.vinh_vien === false || d.vinh_vien === "0" ? false : null;
      await clRemoteSave(bang, projectId, form?.row?.id ?? null, body, form?.row?._sync?.ver ?? (form?.row?.ver != null ? Number(form.row.ver) : undefined));
      setForm(null);
      reload();
      th.reload();
    } catch (e) {
      setErr(e);
    } finally {
      setBusy(false);
    }
  };
  const xoa = async (r: ClRow, bang: "kho" | "gia") => {
    const ok = await confirm({ title: `Xoá ${bang === "kho" ? "kho" : "giá"} "${str(r, "ten")}"?`, confirmLabel: "Xoá", danger: true });
    if (!ok) return;
    try {
      await clRemoteDelete(bang, projectId, r.id);
      reload();
      th.reload();
    } catch (e) {
      notifyError(e, "xoá");
    }
  };
  const xuat = async (pham_vi: { khoId?: string; giaId?: string }, ten: string) => {
    try {
      const p = await kgXuatHoSo(projectId, pham_vi, ten);
      if (p) toast.success(`Đã xuất: ${p}`);
    } catch (e) {
      notifyError(e, "xuất danh sách hồ sơ");
    }
  };

  const pctBadge = (x: GiaTongHop) => {
    if (x.pct_day === null || x.pct_day === undefined) return <span className="text-fg-subtle">—</span>;
    const tone = x.vuot ? "err" : x.gan_day ? "warn" : "ok";
    return <Badge tone={tone}>{x.pct_day.toLocaleString("vi-VN", { maximumFractionDigits: 0 })} %</Badge>;
  };

  const colsKho: Column<ClRow>[] = [
    { key: "ten", header: "Tên kho", sortable: true, cell: (r) => str(r, "ten") },
    { key: "so_gia", header: "Số giá", align: "right", width: 90, cell: (r) => n(giaTheoKho.get(r.id)?.length ?? cat.gia.filter((x) => str(x, "kho_id") === r.id).length) },
    { key: "suc_chua", header: "Sức chứa (hộp)", align: "right", width: 120, cell: (r) => (num(r, "suc_chua_hop") != null ? n(num(r, "suc_chua_hop") ?? 0) : <span className="text-fg-subtle">—</span>) },
    { key: "dang_chua", header: "Đang chứa (hộp)", align: "right", width: 130, cell: (r) => n((giaTheoKho.get(r.id) ?? []).reduce((s, x) => s + x.so_hop, 0)) },
    { key: "gan_day", header: "Giá gần đầy", align: "right", width: 110, cell: (r) => { const k = (giaTheoKho.get(r.id) ?? []).filter((x) => x.gan_day).length; return k ? <Badge tone="warn">{k}</Badge> : <span className="text-fg-subtle">0</span>; } },
    { key: "ghi_chu", header: "Ghi chú", className: "text-fg-subtle", cell: (r) => str(r, "ghi_chu") },
  ];
  const colsGia: Column<GiaTongHop>[] = [
    { key: "kho", header: "Kho", sortable: true, sortValue: (r) => r.ten_kho, cell: (r) => r.ten_kho },
    { key: "pb", header: "Phòng ban", sortable: true, sortValue: (r) => r.ten_phong_ban ?? "", cell: (r) => <>{r.ten_phong_ban ?? ""}{r.ten_phong && <span className="block text-xs text-fg-subtle">{r.ten_phong}</span>}</> },
    { key: "ten", header: "Tên giá", sortable: true, cell: (r) => r.ten },
    { key: "dai", header: "Dải hộp", align: "center", width: 110, cell: (r) => (r.tu_hop != null || r.den_hop != null ? `${r.tu_hop ?? ""}–${r.den_hop ?? ""}` : "—") },
    { key: "loai", header: "Áp cho", width: 110, className: "text-sm", cell: (r) => (r.vinh_vien === true ? "Vĩnh viễn" : r.vinh_vien === false ? "Có thời hạn" : "Cả hai loại") },
    { key: "suc_chua", header: "Sức chứa", align: "right", width: 90, cell: (r) => (r.suc_chua != null ? n(r.suc_chua) : "—") },
    { key: "so_hop", header: "Hộp", align: "right", width: 80, cell: (r) => n(r.so_hop) },
    { key: "so_ho_so", header: "Hồ sơ", align: "right", width: 90, cell: (r) => n(r.so_ho_so) },
    { key: "pct", header: "% đầy", align: "center", width: 90, cell: pctBadge },
    { key: "ghi_chu", header: "Ghi chú", className: "text-fg-subtle", cell: (r) => str(r, "ghi_chu") },
  ];
  const tenTab = tab === "kho" ? "kho" : "giá";

  return (
    <section className="view active">
      <PageHeader
        title="Kho và giá lưu trữ"
        description="Dùng chung toàn dự án."
        info={
          <>
            <p>Giá là dải hộp của một phòng ban trong kho; hồ sơ trên giá suy ra từ số hộp. Sức chứa để trống thì tính bằng độ dài dải hộp; đạt ngưỡng cảnh báo (mặc định 90 %) máy chủ báo quản trị dự án mỗi tuần.</p>
            <p>Tab Gán hộp dành cho hồ sơ nhập từ gói AIP chưa có hộp: chọn phông, phòng ban chủ hộp, nhập số hộp từng dòng hoặc dán hai cột (mã hồ sơ, số hộp) từ bảng tính.</p>
          </>
        }
        actions={
          can &&
          tab !== "ganhop" && (
            <Button variant="primary" icon="them" disabled={!online} onClick={() => mo(null)}>
              Thêm {tenTab}
            </Button>
          )
        }
      />
      <ChuaChonDuAn />
      <CanMayChu sync={sync} viec="Kho và giá chỉ sửa được khi có máy chủ, danh sách dưới là bản sao." />
      <Tabs<Tab>
        value={tab}
        onChange={setTab}
        items={[
          { value: "kho", label: "Kho", icon: "kho", count: cat.kho.length },
          { value: "gia", label: "Giá", icon: "luoi", count: cat.gia.length },
          ...(canGan ? [{ value: "ganhop" as Tab, label: "Gán hộp", icon: "goi" as const }] : []),
        ]}
      />
      {tab !== "ganhop" && (
        <Card className="mt-3" padding="none" actions={tab === "gia" && <Select size="sm" className="w-64" value={phongId} onChange={setPhongId} rows={cat.phong} render={tenPhong} empty="Mọi phông" />} title={tab === "gia" ? "Giá theo phông" : undefined}>
          {th.error != null && <ErrorBanner error={th.error} className="m-3" onRetry={th.reload} />}
          {tab === "kho" ? (
            <Table
              columns={colsKho}
              rows={cat.kho}
              getRowId={(r) => r.id}
              dense
              rowActions={(r) => (
                <Menu
                  size="sm"
                  items={[
                    { label: "Xuất hồ sơ trong kho (Excel)", icon: "tepExcel", disabled: !online, onSelect: () => void xuat({ khoId: r.id }, `kho-${str(r, "ten")}`) },
                    ...(can ? [{ label: "Sửa", icon: "sua" as const, disabled: !online, onSelect: () => mo(r) }, { label: "Xoá", icon: "xoa" as const, danger: true, disabled: !online, onSelect: () => void xoa(r, "kho") }] : []),
                  ]}
                />
              )}
              state={{ empty: { title: "Chưa có kho nào", icon: "kho" } }}
            />
          ) : (
            <Table
              columns={colsGia}
              rows={gias}
              getRowId={(r) => r.id}
              dense
              rowClassName={(r) => (r.vuot ? "bg-danger-50/50" : r.gan_day ? "bg-warning-50/40" : undefined)}
              rowActions={(r) => (
                <Menu
                  size="sm"
                  items={[
                    { label: "Xuất hồ sơ trên giá (Excel)", icon: "tepExcel", disabled: !online, onSelect: () => void xuat({ giaId: r.id }, `gia-${r.ten}`) },
                    ...(can ? [{ label: "Sửa", icon: "sua" as const, disabled: !online, onSelect: () => mo(cat.gia.find((x) => x.id === r.id) ?? (r as unknown as ClRow)) }, { label: "Xoá", icon: "xoa" as const, danger: true, disabled: !online, onSelect: () => void xoa(r as unknown as ClRow, "gia") }] : []),
                  ]}
                />
              )}
              state={{ loading: th.loading && !th.data, error: th.data ? undefined : th.error, onRetry: th.reload, empty: { title: phongId ? "Không có giá nào thuộc phông này" : "Chưa có giá nào", icon: "luoi" } }}
            />
          )}
        </Card>
      )}
      {tab === "ganhop" && <GanHopPanel projectId={projectId} online={online} onDone={() => { reload(); th.reload(); }} />}
      {form && (
        <Dialog
          open
          onOpenChange={(o) => !o && !busy && setForm(null)}
          title={form.row ? `Sửa ${tenTab}` : `Thêm ${tenTab}`}
          size="sm"
          footer={
            <>
              <Button onClick={() => setForm(null)} disabled={busy}>
                Huỷ
              </Button>
              <Button variant="primary" icon="luu" loading={busy} onClick={() => void luu()}>
                Lưu
              </Button>
            </>
          }
        >
          <div className="space-y-3">
            {err !== null && <ErrorBanner error={err} />}
            {tab === "gia" && (
              <>
                <Field label="Kho" required>
                  <Select value={g("kho_id")} onChange={(v) => set("kho_id", v)} rows={cat.kho} />
                </Field>
                <Field label="Phòng ban chủ dải hộp" required>
                  <Select value={g("phong_ban_id")} onChange={(v) => set("phong_ban_id", v)} rows={cat.phong_ban} render={(p) => `${str(cat.phong.find((x) => x.id === str(p, "phong_id")), "ma_du_an")} / ${str(p, "ten")}`} />
                </Field>
              </>
            )}
            <Field label={tab === "kho" ? "Tên kho" : "Tên giá"} required>
              <Input value={g("ten")} onChange={(e) => set("ten", e.target.value)} autoFocus />
            </Field>
            {tab === "gia" && (
              <FormGrid cols={2}>
                <Field label="Từ hộp">
                  <Input type="number" value={g("tu_hop")} onChange={(e) => set("tu_hop", e.target.value)} />
                </Field>
                <Field label="Đến hộp">
                  <Input type="number" value={g("den_hop")} onChange={(e) => set("den_hop", e.target.value)} />
                </Field>
              </FormGrid>
            )}
            <FormGrid cols={2}>
              <Field label="Sức chứa (hộp)" hint={tab === "gia" ? "Trống = tính bằng độ dài dải hộp" : "Chỉ để tham khảo"}>
                <Input type="number" min={1} value={g("suc_chua_hop")} onChange={(e) => set("suc_chua_hop", e.target.value)} />
              </Field>
              {tab === "gia" && (
                <Field label="Cảnh báo khi đầy (%)">
                  <Input type="number" min={1} max={100} value={g("canh_bao_pct")} onChange={(e) => set("canh_bao_pct", e.target.value)} />
                </Field>
              )}
            </FormGrid>
            {tab === "gia" && (
              <Field label="Áp cho" hint="Hộp vĩnh viễn và có thời hạn đánh số riêng — chọn loại nếu giá chỉ chứa một loại">
                <UiSelect
                  value={d.vinh_vien === true || d.vinh_vien === "1" ? "1" : d.vinh_vien === false || d.vinh_vien === "0" ? "0" : ""}
                  onChange={(v) => set("vinh_vien", v)}
                  placeholder="Cả hai loại"
                  options={[{ value: "1", label: "Vĩnh viễn" }, { value: "0", label: "Có thời hạn" }]}
                />
              </Field>
            )}
            <Field label="Ghi chú">
              <Input value={g("ghi_chu")} onChange={(e) => set("ghi_chu", e.target.value)} />
            </Field>
            <Field label="Thứ tự">
              <Input type="number" value={g("so_tt")} onChange={(e) => set("so_tt", e.target.value)} />
            </Field>
          </div>
        </Dialog>
      )}
    </section>
  );
}

/** Gán hộp cho hồ sơ chưa có hộp của một phông (hồ sơ nhập gói AIP): chọn phòng ban chủ hộp, nhập số hộp từng dòng hoặc dán từ bảng tính. */
function GanHopPanel({ projectId, online, onDone }: { projectId: string; online: boolean; onDone: () => void }) {
  const { cat } = useClCatalog();
  const [phongId, setPhongId] = useState("");
  const [phongBanId, setPhongBanId] = useState("");
  const [hop, setHop] = useState<Record<string, string>>({});
  const [dan, setDan] = useState("");
  const [moDan, setMoDan] = useState(false);
  const [busy, setBusy] = useState(false);
  const [kq, setKq] = useState<GanHopKetQua | null>(null);
  const phongBans = useMemo(() => cat.phong_ban.filter((p) => str(p, "phong_id") === phongId), [cat.phong_ban, phongId]);
  useEffect(() => {
    setPhongBanId(phongBans.length === 1 ? phongBans[0].id : "");
    setHop({});
    setKq(null);
  }, [phongId, phongBans]);
  const loader = useCallback(async () => {
    if (!isTauri() || !online || !projectId || !phongId) return { rows: [] as ClRow[] };
    return kgChuaHop(projectId, phongId);
  }, [online, projectId, phongId]);
  const { data, loading, error, reload } = V41t(loader, { topics: ["chinhLy", "khogia"], errorText: () => "Không tải được hồ sơ chưa gán hộp." });
  const rows = useMemo(() => data?.rows ?? [], [data]);
  const soDien = Object.values(hop).filter((v) => v.trim()).length;

  const apDan = () => {
    const theoMa = new Map<string, string>();
    for (const r of rows) {
      for (const k of [str(r, "ma_ho_so"), str(r, "ma_ho_so_goc"), str(r, "so_tam")]) if (k) theoMa.set(k.trim().toLowerCase(), r.id);
    }
    const next = { ...hop };
    let khop = 0;
    let truot = 0;
    for (const line of dan.split(/\r?\n/)) {
      const cells = line.split(/\t|;|,/).map((c) => c.trim());
      if (cells.length < 2 || !cells[0] || !cells[1]) continue;
      const id = theoMa.get(cells[0].toLowerCase());
      if (id) {
        next[id] = cells[1];
        khop++;
      } else truot++;
    }
    setHop(next);
    setMoDan(false);
    setDan("");
    toast[truot ? "warning" : "success"](`Đã khớp ${khop} dòng${truot ? `, ${truot} dòng không thấy hồ sơ` : ""}.`);
  };
  const gan = async () => {
    if (!phongBanId) {
      toast.warning("Chọn phòng ban chủ hộp.");
      return;
    }
    const ds = rows.filter((r) => hop[r.id]?.trim()).map((r) => ({ hoSoId: r.id, hopSo: hop[r.id].trim() }));
    if (ds.length === 0) return;
    setBusy(true);
    try {
      const r = await kgGanHop(projectId, phongBanId, ds);
      setKq(r);
      if (r.errors.length === 0) toast.success(`Đã gán hộp cho ${r.applied} hồ sơ (${r.hopsCreated} hộp mới).`);
      else toast.warning(`Gán ${r.applied} hồ sơ, ${r.errors.length} dòng lỗi.`);
      const okIds = new Set(ds.map((x) => x.hoSoId));
      for (const e of r.errors) okIds.delete(e.hoSoId);
      setHop((p) => Object.fromEntries(Object.entries(p).filter(([id]) => !okIds.has(id))));
      reload();
      onDone();
    } catch (e) {
      notifyError(e, "gán hộp");
    } finally {
      setBusy(false);
    }
  };
  const loiCua = (id: string) => kq?.errors.find((e) => e.hoSoId === id)?.message;

  return (
    <Card
      className="mt-3"
      padding="none"
      title="Hồ sơ chưa gán hộp"
      description="Hồ sơ nhập từ gói AIP đứng trước. Nhập số hộp rồi bấm Gán hộp; hộp chưa có sẽ được tạo theo phòng ban đã chọn."
      actions={
        <>
          <Select size="sm" className="w-56" value={phongId} onChange={setPhongId} rows={cat.phong} render={tenPhong} empty="Chọn phông" ariaLabel="Phông" />
          <Select size="sm" className="w-48" value={phongBanId} onChange={setPhongBanId} rows={phongBans} empty="Phòng ban chủ hộp" ariaLabel="Phòng ban" disabled={!phongId} />
          <Button size="sm" icon="nhapTep" disabled={rows.length === 0} onClick={() => setMoDan(true)}>
            Dán từ bảng tính
          </Button>
          <Button size="sm" variant="primary" icon="goi" disabled={!online || soDien === 0 || !phongBanId} loading={busy} onClick={() => void gan()}>
            Gán hộp ({soDien})
          </Button>
        </>
      }
    >
      {phongId && phongBans.length === 0 && (
        <Banner tone="warning" className="m-3">
          Phông này chưa có phòng ban. Thêm một phòng ban (ví dụ lấy tên phông) ở Danh mục rồi quay lại — hộp thuộc phòng ban.
        </Banner>
      )}
      <Table.Root dense maxHeight={560}>
        <Table.Head>
          <tr>
            <Table.HeadCell width={160}>Mã hồ sơ</Table.HeadCell>
            <Table.HeadCell>Tiêu đề</Table.HeadCell>
            <Table.HeadCell width={100}>Năm</Table.HeadCell>
            <Table.HeadCell width={120}>THBQ</Table.HeadCell>
            <Table.HeadCell width={110}>Hộp số</Table.HeadCell>
          </tr>
        </Table.Head>
        <Table.Body>
          {rows.map((r) => {
            const loi = loiCua(r.id);
            return (
              <Table.Row key={r.id} className={loi ? "bg-danger-50/50" : undefined}>
                <Table.Cell>
                  <span className="text-sm">{str(r, "ma_ho_so") || str(r, "ma_ho_so_goc") || `Số tạm ${str(r, "so_tam")}`}</span>
                  {str(r, "objid") && <Badge tone="gray" className="ml-1">AIP</Badge>}
                </Table.Cell>
                <Table.Cell className="max-w-[440px] whitespace-normal">
                  {str(r, "tieu_de")}
                  {loi && <span className="block text-xs text-danger-700">{loi}</span>}
                </Table.Cell>
                <Table.Cell>{str(r, "ngay_ket_thuc")}</Table.Cell>
                <Table.Cell>{str(r, "ten_thbq")}</Table.Cell>
                <Table.Cell>
                  <Input size="sm" value={hop[r.id] ?? ""} onChange={(e) => setHop((p) => ({ ...p, [r.id]: e.target.value }))} aria-label={`Hộp số của ${str(r, "tieu_de")}`} placeholder="VD 12" />
                </Table.Cell>
              </Table.Row>
            );
          })}
          {rows.length === 0 && (
            <Table.FullRow colSpan={5}>
              <p className="px-3 py-4 text-sm text-fg-subtle">{!phongId ? "Chọn phông để xem hồ sơ chưa gán hộp." : loading && !data ? "Đang tải…" : error != null ? "Không tải được." : "Mọi hồ sơ của phông đã có hộp."}</p>
            </Table.FullRow>
          )}
        </Table.Body>
      </Table.Root>
      {moDan && (
        <Dialog
          open
          onOpenChange={(o) => !o && setMoDan(false)}
          title="Dán từ bảng tính"
          description="Mỗi dòng: mã hồ sơ (hoặc mã trong gói, số tạm) và số hộp, cách nhau bằng tab, chấm phẩy hoặc phẩy."
          size="md"
          footer={
            <>
              <Button onClick={() => setMoDan(false)}>Huỷ</Button>
              <Button variant="primary" icon="duyet" disabled={!dan.trim()} onClick={apDan}>
                Khớp vào bảng
              </Button>
            </>
          }
        >
          <Textarea rows={12} value={dan} onChange={(e) => setDan(e.target.value)} placeholder={"01.01.2020.001\t12\n01.01.2020.002\t12"} autoFocus />
        </Dialog>
      )}
    </Card>
  );
}
