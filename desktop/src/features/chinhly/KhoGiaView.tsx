// Copyright (c) 2026 DigiText. All rights reserved.
// Kho và giá lưu trữ (chỉ online): kho (tên, ghi chú), giá = dải hộp [từ–đến] của một phòng ban trong kho;
// hồ sơ thuộc giá suy ra từ số hộp, hoặc gán tay qua trường gia_id.

import { useEffect, useMemo, useState } from "react";
import { V460 } from "@/state";
import { useProject } from "@/state/project";
import { clHoSoList, clRemoteDelete, clRemoteSave, str, num, type ClRow } from "@/api/chinhly";
import { CanMayChu, ChuaChonDuAn, n, Select, tenPhong, useClCatalog, useClSync } from "./common";
import { Button, Card, confirm, Dialog, ErrorBanner, Field, FormGrid, Input, Menu, notifyError, PageHeader, Select as UiSelect, Table, Tabs, type Column } from "@/ui";

type Tab = "kho" | "gia";

export default function KhoGiaView() {
  const { session } = V460();
  const projectId = useProject().projectId ?? ""; // dự án đang chọn (super admin không gắn dự án trong phiên)
  const can = (session?.permissions ?? []).includes("khogia.manage");
  const { cat, reload } = useClCatalog();
  const sync = useClSync();
  const online = sync?.phase !== "offline";
  const [tab, setTab] = useState<Tab>("kho");
  const [form, setForm] = useState<{ row: ClRow | null } | null>(null);
  const [d, setD] = useState<Record<string, unknown>>({});
  const [err, setErr] = useState<unknown>(null);
  const [busy, setBusy] = useState(false);
  const [phongId, setPhongId] = useState("");
  const [soHoSo, setSoHoSo] = useState<Map<string, number>>(new Map());
  const phongBans = useMemo(() => cat.phong_ban.filter((p) => !phongId || str(p, "phong_id") === phongId), [cat.phong_ban, phongId]);
  const tenKho = useMemo(() => new Map(cat.kho.map((k) => [k.id, str(k, "ten")])), [cat.kho]);
  const tenPb = useMemo(() => new Map(cat.phong_ban.map((k) => [k.id, str(k, "ten")])), [cat.phong_ban]);
  const g = (k: string) => (d[k] === null || d[k] === undefined ? "" : String(d[k]));
  const set = (k: string, v: unknown) => setD((p) => ({ ...p, [k]: v }));
  const gias = useMemo(() => cat.gia.filter((x) => !phongId || phongBans.some((p) => p.id === str(x, "phong_ban_id"))), [cat.gia, phongId, phongBans]);

  // Đếm hồ sơ trên từng giá (một truy vấn nhẹ mỗi giá, chỉ khi mở tab Giá).
  useEffect(() => {
    if (tab !== "gia" || cat.gia.length === 0) return;
    let alive = true;
    void Promise.all(cat.gia.map((x) => clHoSoList({ giaId: x.id, limit: 1 }).then((r) => [x.id, r.total] as const).catch(() => [x.id, -1] as const))).then((xs) => {
      if (alive) setSoHoSo(new Map(xs));
    });
    return () => {
      alive = false;
    };
  }, [tab, cat.gia]);

  const mo = (row: ClRow | null) => {
    setD(row ? { ...row } : tab === "gia" ? { kho_id: cat.kho[0]?.id ?? "" } : {});
    setForm({ row });
    setErr(null);
  };
  const luu = async () => {
    setBusy(true);
    setErr(null);
    try {
      const keys = tab === "kho" ? ["ten", "ghi_chu", "so_tt"] : ["kho_id", "phong_ban_id", "ten", "tu_hop", "den_hop", "ghi_chu", "so_tt"];
      const body: Record<string, unknown> = {};
      for (const k of keys) body[k] = d[k] === "" ? null : d[k];
      // Giá áp cho loại nào (0015): hộp 01 vĩnh viễn và hộp 01 có thời hạn là hai hộp khác nhau.
      if (tab === "gia") body.vinh_vien = d.vinh_vien === true || d.vinh_vien === "1" ? true : d.vinh_vien === false || d.vinh_vien === "0" ? false : null;
      await clRemoteSave(tab, projectId, form?.row?.id ?? null, body, form?.row?._sync?.ver);
      setForm(null);
      reload();
    } catch (e) {
      setErr(e);
    } finally {
      setBusy(false);
    }
  };
  const xoa = async (r: ClRow) => {
    const ok = await confirm({ title: `Xoá ${tab === "kho" ? "kho" : "giá"} "${str(r, "ten")}"?`, confirmLabel: "Xoá", danger: true });
    if (!ok) return;
    try {
      await clRemoteDelete(tab, projectId, r.id);
      reload();
    } catch (e) {
      notifyError(e, "xoá");
    }
  };

  const colsKho: Column<ClRow>[] = [
    { key: "ten", header: "Tên kho", sortable: true, cell: (r) => str(r, "ten") },
    { key: "so_gia", header: "Số giá", align: "right", width: 90, cell: (r) => n(cat.gia.filter((x) => str(x, "kho_id") === r.id).length) },
    { key: "ghi_chu", header: "Ghi chú", className: "text-fg-subtle", cell: (r) => str(r, "ghi_chu") },
  ];
  const colsGia: Column<ClRow>[] = [
    { key: "kho", header: "Kho", sortable: true, sortValue: (r) => tenKho.get(str(r, "kho_id")) ?? "", cell: (r) => tenKho.get(str(r, "kho_id")) ?? "" },
    { key: "pb", header: "Phòng ban", sortable: true, sortValue: (r) => tenPb.get(str(r, "phong_ban_id")) ?? "", cell: (r) => tenPb.get(str(r, "phong_ban_id")) ?? "" },
    { key: "ten", header: "Tên giá", sortable: true, cell: (r) => str(r, "ten") },
    { key: "tu_hop", header: "Từ hộp", align: "right", width: 90, cell: (r) => num(r, "tu_hop") ?? "" },
    { key: "den_hop", header: "Đến hộp", align: "right", width: 90, cell: (r) => num(r, "den_hop") ?? "" },
    { key: "loai", header: "Áp cho", width: 110, className: "text-sm", cell: (r) => (r.vinh_vien === true ? "Vĩnh viễn" : r.vinh_vien === false ? "Có thời hạn" : "Cả hai loại") },
    { key: "so_hs", header: "Hồ sơ", align: "right", width: 90, cell: (r) => { const v = soHoSo.get(r.id); return v == null ? "…" : v < 0 ? "—" : n(v); } },
    { key: "ghi_chu", header: "Ghi chú", className: "text-fg-subtle", cell: (r) => str(r, "ghi_chu") },
  ];
  const tenTab = tab === "kho" ? "kho" : "giá";

  return (
    <section className="view active">
      <PageHeader
        title="Kho và giá lưu trữ"
        description="Dùng chung toàn dự án. Giá là dải hộp của một phòng ban trong kho, hồ sơ trên giá suy ra từ số hộp."
        actions={
          can && (
            <Button variant="primary" icon="them" disabled={!online} onClick={() => mo(null)}>
              Thêm {tenTab}
            </Button>
          )
        }
      />
      <ChuaChonDuAn />
      <CanMayChu sync={sync} viec="Kho và giá chỉ sửa được khi có máy chủ, danh sách dưới là bản sao." />
      <Tabs<Tab> value={tab} onChange={setTab} items={[{ value: "kho", label: "Kho", icon: "kho", count: cat.kho.length }, { value: "gia", label: "Giá", icon: "luoi", count: cat.gia.length }]} />
      <Card className="mt-3" padding="none" actions={tab === "gia" && <Select size="sm" className="w-64" value={phongId} onChange={setPhongId} rows={cat.phong} render={tenPhong} empty="Mọi phông" />} title={tab === "gia" ? "Giá theo phông" : undefined}>
        {tab === "kho" ? (
          <Table columns={colsKho} rows={cat.kho} getRowId={(r) => r.id} dense rowActions={can ? (r) => <Menu size="sm" items={[{ label: "Sửa", icon: "sua", disabled: !online, onSelect: () => mo(r) }, { label: "Xoá", icon: "xoa", danger: true, disabled: !online, onSelect: () => void xoa(r) }]} /> : undefined} state={{ empty: { title: "Chưa có kho nào", icon: "kho" } }} />
        ) : (
          <Table columns={colsGia} rows={gias} getRowId={(r) => r.id} dense rowActions={can ? (r) => <Menu size="sm" items={[{ label: "Sửa", icon: "sua", disabled: !online, onSelect: () => mo(r) }, { label: "Xoá", icon: "xoa", danger: true, disabled: !online, onSelect: () => void xoa(r) }]} /> : undefined} state={{ empty: { title: phongId ? "Không có giá nào thuộc phông này" : "Chưa có giá nào", icon: "luoi" } }} />
        )}
      </Card>
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
