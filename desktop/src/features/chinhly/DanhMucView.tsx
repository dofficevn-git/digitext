// Copyright (c) 2026 DigiText. All rights reserved.
// Danh mục chỉnh lý (bản sao đồng bộ với máy chủ): phông, phòng ban, mảng công việc, THBQ, ngôn ngữ, loại văn bản,
// khối phông. Sửa được offline; dòng dùng chung (chuẩn TT05) chỉ super admin sửa trên máy chủ.

import { useMemo, useState } from "react";
import { V460 } from "@/state";
import { clDelete, clSave, str, type ClCatalog, type ClRow, type ClTable } from "@/api/chinhly";
import { ChuaChonDuAn, Select, SyncBadgeRow, tenPhong, useClCatalog } from "./common";
import { Badge, Button, Card, confirm, Dialog, ErrorBanner, Field, Help, Input, Menu, notifyError, PageHeader, Select as UiSelect, Table, Tabs, Tip, type Column } from "@/ui";

type Tab = "phong" | "phong_ban" | "mang" | "thbq" | "ngon_ngu" | "loai_van_ban" | "khoi_phong";
const TABS: [Tab, string][] = [
  ["phong", "Phông"],
  ["phong_ban", "Phòng ban"],
  ["mang", "Mảng công việc"],
  ["thbq", "Thời hạn bảo quản"],
  ["ngon_ngu", "Ngôn ngữ"],
  ["loai_van_ban", "Loại văn bản"],
  ["khoi_phong", "Khối phông"],
];

interface Col {
  k: string;
  l: string;
  type?: "text" | "number" | "select";
  opts?: [string, string][];
  w?: number;
  required?: boolean;
  hint?: string;
}
const COLS: Record<Tab, Col[]> = {
  khoi_phong: [
    { k: "ma", l: "Mã khối", required: true },
    { k: "ten", l: "Tên khối", required: true },
    { k: "so_tt", l: "Thứ tự", type: "number", w: 80 },
  ],
  phong: [
    { k: "ma_du_an", l: "Mã phông", required: true, hint: "Dùng để tạo mã hồ sơ" },
    { k: "ten", l: "Tên phông", required: true },
    { k: "phong_so", l: "Phông số", w: 90 },
    { k: "muc_luc_so", l: "Mục lục số (vĩnh viễn)", w: 110, hint: "Đuôi mã hồ sơ của hồ sơ vĩnh viễn" },
    { k: "muc_luc_so_cth", l: "Mục lục số (có thời hạn)", w: 110, hint: "Đuôi mã hồ sơ của hồ sơ có thời hạn; để trống thì dùng mục lục số vĩnh viễn" },
    { k: "khoi_id", l: "Khối phông", type: "select" },
    { k: "ghi_chu", l: "Ghi chú" },
    { k: "so_tt", l: "Thứ tự", type: "number", w: 80 },
  ],
  phong_ban: [
    { k: "phong_id", l: "Phông", type: "select", required: true },
    { k: "ma_don_vi", l: "Mã đơn vị", hint: "Dùng để tạo mã hồ sơ" },
    { k: "ten", l: "Tên phòng ban", required: true },
    { k: "nhiem_ky", l: "Nhiệm kỳ" },
    { k: "so_tt", l: "Thứ tự", type: "number", w: 80 },
  ],
  mang: [
    { k: "phong_ban_id", l: "Phòng ban", type: "select", required: true },
    { k: "ma_mang", l: "Mã mảng", hint: "Mặc định 00" },
    { k: "ten", l: "Tên mảng công việc", required: true },
    { k: "so_tt", l: "Thứ tự", type: "number", w: 80 },
  ],
  thbq: [
    { k: "ten", l: "Tên", required: true },
    {
      k: "trang_thai",
      l: "Loại",
      type: "select",
      required: true,
      opts: [
        ["1", "Vĩnh viễn"],
        ["2", "Có thời hạn"],
        ["3", "Đánh giá lại"],
        ["4", "Theo tuổi thọ công trình"],
      ],
    },
    { k: "gia_tri", l: "Số năm", type: "number", w: 80 },
    { k: "ma_tt05", l: "Mã TT05", w: 90, hint: "01 đến 07" },
    { k: "so_tt", l: "Thứ tự", type: "number", w: 80 },
  ],
  ngon_ngu: [
    { k: "ma", l: "Mã", w: 80, required: true },
    { k: "ten", l: "Tên", required: true },
    { k: "so_tt", l: "Thứ tự", type: "number", w: 80 },
  ],
  loai_van_ban: [
    { k: "ma", l: "Mã", w: 80, required: true },
    { k: "ten", l: "Tên loại", required: true },
    { k: "so_tt", l: "Thứ tự", type: "number", w: 80 },
  ],
};

function Form({ tab, cat, row, onClose, onSaved }: { tab: Tab; cat: ClCatalog; row: ClRow | null; onClose: () => void; onSaved: () => void }) {
  const [d, setD] = useState<Record<string, unknown>>(() => (row ? { ...row } : tab === "mang" ? { ma_mang: "00" } : tab === "thbq" ? { trang_thai: "2", gia_tri: 0 } : {}));
  const [errs, setErrs] = useState<Record<string, string>>({});
  const [chung, setChung] = useState<unknown>(null);
  const [busy, setBusy] = useState(false);
  const g = (k: string) => (d[k] === null || d[k] === undefined ? "" : String(d[k]));
  const set = (k: string, v: unknown) => setD((p) => ({ ...p, [k]: v }));
  const selRows = (k: string): ClRow[] => (k === "phong_id" ? cat.phong : k === "phong_ban_id" ? cat.phong_ban : k === "khoi_id" ? cat.khoi_phong : []);
  const luu = async () => {
    setBusy(true);
    setErrs({});
    setChung(null);
    try {
      const patch: Record<string, unknown> = {};
      for (const c of COLS[tab]) patch[c.k] = d[c.k] === "" ? null : d[c.k];
      const r = await clSave(tab as ClTable, row ? row.id : null, patch);
      if (!r.ok) {
        const m: Record<string, string> = {};
        const khac: string[] = [];
        for (const e of r.errors) {
          if (e.field && COLS[tab].some((c) => c.k === e.field)) m[e.field] = e.message;
          else khac.push(e.message);
        }
        setErrs(m);
        if (khac.length) setChung(khac.join(" "));
        return;
      }
      onSaved();
    } catch (e) {
      setChung(e);
    } finally {
      setBusy(false);
    }
  };
  const tenTab = TABS.find((t) => t[0] === tab)?.[1].toLowerCase() ?? "";
  return (
    <Dialog
      open
      onOpenChange={(o) => !o && !busy && onClose()}
      title={row ? `Sửa ${tenTab}` : `Thêm ${tenTab}`}
      size="sm"
      footer={
        <>
          <Button onClick={onClose} disabled={busy}>
            Huỷ
          </Button>
          <Button variant="primary" icon="luu" loading={busy} onClick={() => void luu()}>
            Lưu
          </Button>
        </>
      }
    >
      <form
        className="space-y-3"
        onSubmit={(e) => {
          e.preventDefault();
          void luu();
        }}
      >
        {chung !== null && <ErrorBanner error={chung} />}
        {COLS[tab].map((c) => (
          <Field key={c.k} label={c.l} required={c.required} error={errs[c.k]} hint={c.hint}>
            {c.type === "select" && c.opts ? (
              <UiSelect value={g(c.k)} onChange={(v) => set(c.k, v)} placeholder="Chọn" options={c.opts.map(([value, label]) => ({ value, label }))} />
            ) : c.type === "select" ? (
              <Select value={g(c.k)} onChange={(v) => set(c.k, v)} rows={selRows(c.k)} render={c.k === "phong_id" ? tenPhong : undefined} />
            ) : (
              <Input type={c.type === "number" ? "number" : "text"} value={g(c.k)} onChange={(e) => set(c.k, e.target.value)} autoFocus={c === COLS[tab][0]} />
            )}
          </Field>
        ))}
        <button type="submit" hidden aria-hidden tabIndex={-1} />
      </form>
    </Dialog>
  );
}

export default function DanhMucView() {
  const { session } = V460();
  const perms = session?.permissions ?? [];
  const isSuper = session?.role === "super_admin";
  const canCreate = perms.includes("catalog.create");
  const canUpdate = perms.includes("catalog.update");
  const canDelete = perms.includes("catalog.delete");
  const { cat, reload, error } = useClCatalog();
  const [tab, setTab] = useState<Tab>("phong");
  const [form, setForm] = useState<{ row: ClRow | null } | null>(null);
  const rows = cat[tab];
  const ten = useMemo(() => {
    const m = new Map<string, string>();
    for (const r of [...cat.phong, ...cat.phong_ban, ...cat.khoi_phong]) m.set(r.id, str(r, "ten"));
    return m;
  }, [cat]);
  const chung = (r: ClRow) => r.project_id === null || r.project_id === undefined;
  const xoa = async (r: ClRow) => {
    const ok = await confirm({ title: `Xoá "${str(r, "ten")}"?`, body: "Bị chặn nếu còn hồ sơ hoặc danh mục con đang dùng.", confirmLabel: "Xoá", danger: true });
    if (!ok) return;
    try {
      await clDelete(tab as ClTable, r.id);
      reload();
    } catch (e) {
      notifyError(e, "xoá");
    }
  };

  const cols: Column<ClRow>[] = [
    {
      key: "_tt",
      header: "",
      width: 90,
      cell: (r) =>
        chung(r) ? (
          <Tip text="Bảng mã chuẩn TT05, dùng chung mọi dự án">
            <span>
              <Badge tone="info">Chuẩn TT05</Badge>
            </span>
          </Tip>
        ) : (
          <SyncBadgeRow sync={r._sync?.dirty ? "pending" : "clean"} />
        ),
    },
    ...COLS[tab].map<Column<ClRow>>((c) => ({
      key: c.k,
      header: c.l,
      width: c.w,
      align: c.type === "number" ? "right" : undefined,
      sortable: true,
      cell: (r) => (c.type === "select" && c.opts ? (c.opts.find(([v]) => v === str(r, c.k))?.[1] ?? str(r, c.k)) : c.type === "select" ? (ten.get(str(r, c.k)) ?? "") : str(r, c.k)),
      sortValue: (r) => (c.type === "select" && !c.opts ? (ten.get(str(r, c.k)) ?? "") : c.type === "number" ? Number(str(r, c.k)) || 0 : str(r, c.k)),
    })),
  ];

  return (
    <section className="view active">
      <PageHeader
        title="Danh mục chỉnh lý"
        description="Danh mục dùng chung toàn dự án, đồng bộ với máy chủ. Sửa được khi ngoại tuyến."
        actions={
          canCreate && (
            <Button variant="primary" icon="them" onClick={() => setForm({ row: null })}>
              Thêm
            </Button>
          )
        }
      />
      <ChuaChonDuAn />
      {error && <ErrorBanner error={error} className="mb-4" onRetry={reload} />}
      <Tabs<Tab> value={tab} onChange={setTab} items={TABS.map(([value, label]) => ({ value, label, count: cat[value].length }))} />
      <Card className="mt-3" padding="none">
        <Table
          columns={cols}
          rows={rows}
          getRowId={(r) => r.id}
          dense
          rowActions={(r) => {
            const locked = chung(r) && !isSuper;
            if (locked || (!canUpdate && !canDelete)) return null;
            return <Menu size="sm" items={[...(canUpdate ? [{ label: "Sửa", icon: "sua" as const, onSelect: () => setForm({ row: r }) }] : []), ...(canDelete ? [{ label: "Xoá", icon: "xoa" as const, danger: true, onSelect: () => void xoa(r) }] : [])]} />;
          }}
          state={{ empty: { title: `Chưa có ${TABS.find((t) => t[0] === tab)?.[1].toLowerCase()} nào`, action: canCreate ? <Button icon="them" onClick={() => setForm({ row: null })}>Thêm</Button> : undefined } }}
        />
      </Card>
      {form && (
        <Form
          tab={tab}
          cat={cat}
          row={form.row}
          onClose={() => setForm(null)}
          onSaved={() => {
            setForm(null);
            reload();
          }}
        />
      )}
      {tab === "phong" && (
        <Help title="Cách tạo mã hồ sơ" className="mt-4">
          <p>
            Mã hồ sơ = <code>Mã phông.Phông số.Năm kết thúc.Số hồ sơ[.Mục lục số]</code>. Mã văn bản theo TT05 = mã hồ sơ + số thứ tự 7 chữ số.
          </p>
        </Help>
      )}
    </section>
  );
}
