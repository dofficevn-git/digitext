// Copyright (c) 2026 DigiText. All rights reserved.
// Lập hồ sơ điện tử (chỉnh lý, bản sao đồng bộ): lọc theo phông / phòng ban / mảng / THBQ / trạng thái / trùng thừa,
// tìm, bảng 50 dòng sắp xếp phía máy chủ, thêm / sửa / xoá, nhập Excel, văn bản của hồ sơ.

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { isTauri } from "@/api/ipc";
import { V41t } from "@/live/useLive";
import { V460 } from "@/state";
import { useProject } from "@/state/project";
import { clDelete, clHoSoList, clRemote, str, THBQ_STATUS_LABEL, type ClHoSoFilter, type ClRow } from "@/api/chinhly";
import { CanMayChu, Chip, ChuaChonDuAn, n, Select, SyncBadge, SyncBadgeRow, tenPhong, THBQ_BADGE, useClCatalog, useClSync } from "./common";
import HoSoForm from "./HoSoForm";
import VanBanPanel from "./VanBanPanel";
import { docMauLapHoSo, xuatMauLapHoSo } from "./excel";
import NhapMucLucDialog from "./NhapMucLucDialog";
import { Badge, Banner, Button, Card, confirm, ErrorBanner, Field, FormGrid, Input, Menu, notifyError, PageHeader, Select as UiSelect, Table, toast, type Column, type SortState } from "@/ui";

const PAGE = 50;
const SORT_KEYS: Record<string, ClHoSoFilter["sort"]> = { so_tam: "so_tam", ho_so_so: "ho_so_so", ten_mang: "mang", ten_thbq: "thbq", ngay_ket_thuc: "ngay_kt" };
const AN_MAC_DINH = ["ma_ho_so", "so_to"];

export default function LapHoSoView() {
  const { session } = V460();
  const duAnDangChon = useProject().projectId; // không dùng session.projectId: super admin không gắn dự án
  const perms = session?.permissions ?? [];
  const canCreate = perms.includes("dossier.create");
  const canEdit = perms.includes("dossier.update");
  const canDelete = perms.includes("dossier.delete");
  const { cat, reload: reloadCat, error: catErr } = useClCatalog();
  const sync = useClSync();

  const [phongId, setPhongId] = useState("");
  const [phongBanId, setPhongBanId] = useState("");
  const [mangId, setMangId] = useState("");
  const [thbqIds, setThbqIds] = useState<string[]>([]);
  const [coThoiHan, setCoThoiHan] = useState(false);
  const [status, setStatus] = useState<"" | "VV" | "GL" | "HET">("");
  const [trungThua, setTrungThua] = useState(false);
  const [q, setQ] = useState("");
  const [qApplied, setQApplied] = useState("");
  const [page, setPage] = useState(1);
  const [sort, setSort] = useState<SortState | null>({ key: "so_tam", dir: "asc" });
  const [anCot, setAnCot] = useState<Set<string>>(() => new Set(AN_MAC_DINH));

  const [sel, setSel] = useState<Set<string>>(new Set());
  const [form, setForm] = useState<{ row: ClRow | null } | null>(null);
  const [open, setOpen] = useState<ClRow | null>(null);
  const [nhapKq, setNhapKq] = useState<{ text: string; errors: string[] } | null>(null);
  const [mucLuc, setMucLuc] = useState(false);
  const fileRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (!phongId && cat.phong.length > 0) setPhongId(str(cat.phong[0], "id"));
  }, [cat.phong, phongId]);
  useEffect(() => {
    const t = window.setTimeout(() => {
      setQApplied(q.trim());
      setPage(1);
    }, 400);
    return () => window.clearTimeout(t);
  }, [q]);

  const phongBans = useMemo(() => cat.phong_ban.filter((p) => str(p, "phong_id") === phongId), [cat.phong_ban, phongId]);
  const mangs = useMemo(() => cat.mang.filter((m) => str(m, "phong_ban_id") === phongBanId), [cat.mang, phongBanId]);

  const filter: ClHoSoFilter = useMemo(
    () => ({
      phongId: phongId || undefined,
      phongBanId: phongBanId || undefined,
      mangId: mangId || undefined,
      thbqIds,
      thbqCoThoiHan: coThoiHan,
      status: status || undefined,
      trungThua: trungThua ? true : undefined,
      q: qApplied || undefined,
      sort: (sort && SORT_KEYS[sort.key]) || "so_tam",
      desc: sort?.dir === "desc",
      limit: PAGE,
      offset: (page - 1) * PAGE,
    }),
    [phongId, phongBanId, mangId, thbqIds, coThoiHan, status, trungThua, qApplied, sort, page],
  );

  const loader = useCallback(() => (isTauri() ? clHoSoList(filter) : Promise.resolve({ rows: [] as ClRow[], total: 0 })), [filter]);
  const { data, error, loading, reload } = V41t(loader, { topics: ["chinhLy"], errorText: () => "Không tải được danh sách hồ sơ." });
  const rows = useMemo(() => data?.rows ?? [], [data]);
  const total = data?.total ?? 0;
  useEffect(() => {
    if (open) {
      const again = rows.find((x) => x.id === open.id);
      if (again && again !== open) setOpen(again);
    }
  }, [rows, open]);

  const datLoc = <T,>(setter: (v: T) => void) => (v: T) => {
    setter(v);
    setPage(1);
  };

  const xoaChon = async () => {
    if (sel.size === 0) return;
    const ok = await confirm({ title: `Xoá ${n(sel.size)} hồ sơ đã chọn?`, body: "Văn bản bên trong bị xoá theo. Có thể khôi phục trên máy chủ.", confirmLabel: "Xoá", danger: true });
    if (!ok) return;
    try {
      for (const id of sel) await clDelete("ho_so", id);
      setSel(new Set());
      toast.success(`Đã xoá ${n(sel.size)} hồ sơ.`);
      reload();
    } catch (e) {
      notifyError(e, "xoá hồ sơ");
    }
  };

  const phong = cat.phong.find((p) => p.id === phongId);
  const nhapExcel = async (f: File) => {
    setNhapKq(null);
    try {
      const d = await docMauLapHoSo(f);
      if (d.maPhong && phong && d.maPhong.toLowerCase() !== str(phong, "ma_du_an").toLowerCase()) {
        const ok = await confirm({ title: "Mã phông trong tệp khác phông đang chọn", body: `Ô A1 ghi "${d.maPhong}", phông đang chọn là ${str(phong, "ma_du_an")}. Vẫn nhập vào phông đang chọn?`, confirmLabel: "Nhập" });
        if (!ok) return;
      }
      if (d.rows.length === 0) {
        toast.warning(d.errors[0]?.message ?? "Tệp không có dòng dữ liệu, dữ liệu bắt đầu từ dòng 5.");
        return;
      }
      const r = await clRemote<{ created: number; updated: number; skipped: number; errors: { row: number; field?: string; message: string }[] }>("POST", "/cl/ho-so/import", { projectId: duAnDangChon, phongId, mode: "upsert", rows: d.rows });
      const errs = [...d.errors, ...r.errors.map((e) => ({ row: e.row + 4, message: `${e.field ? e.field + ": " : ""}${e.message}` }))];
      setNhapKq({ text: `Đã thêm ${n(r.created)}, cập nhật ${n(r.updated)}, bỏ qua ${n(r.skipped)}${errs.length ? `, ${errs.length} dòng lỗi` : ""}.`, errors: errs.map((e) => `Dòng ${e.row}: ${e.message}`) });
      reload();
    } catch (e) {
      notifyError(e, "nhập Excel");
    }
  };
  const toggleThbq = (id: string) => {
    setThbqIds((v) => (v.includes(id) ? v.filter((x) => x !== id) : [...v, id]));
    setPage(1);
  };

  const cols: Column<ClRow>[] = [
    { key: "_s", header: "", width: 80, cell: (r) => <SyncBadgeRow sync={str(r, "sync")} /> },
    { key: "ma_ho_so", header: "Mã hồ sơ", className: "text-xs text-fg-subtle", cell: (r) => str(r, "ma_ho_so") },
    { key: "so_tam", header: "Số tạm", sortable: true, cell: (r) => str(r, "so_tam") },
    { key: "ho_so_so", header: "Hồ sơ số", sortable: true, align: "right", width: 90, cell: (r) => str(r, "ho_so_so") },
    { key: "hop_so", header: "Hộp", align: "right", width: 64, cell: (r) => str(r, "hop_so") },
    {
      key: "tieu_de",
      header: "Tiêu đề",
      className: "max-w-[460px] whitespace-normal",
      cell: (r) => (
        <>
          {str(r, "tieu_de") || <span className="text-fg-subtle">Chưa có tiêu đề</span>}
          {r.trung_thua === true && (
            <Badge tone="gray" className="ml-1">
              trùng thừa
            </Badge>
          )}
        </>
      ),
    },
    { key: "ten_mang", header: "Mảng", sortable: true, className: "text-sm", cell: (r) => str(r, "ten_mang") },
    { key: "ten_thbq", header: "THBQ", sortable: true, className: "text-sm", cell: (r) => str(r, "ten_thbq") },
    { key: "so_to", header: "Số tờ", align: "right", width: 70, cell: (r) => str(r, "so_to") },
    { key: "ngay_bat_dau", header: "Bắt đầu", width: 100, cell: (r) => str(r, "ngay_bat_dau") },
    { key: "ngay_ket_thuc", header: "Kết thúc", sortable: true, width: 100, cell: (r) => str(r, "ngay_ket_thuc") },
    {
      key: "thbq_status",
      header: "Trạng thái",
      width: 110,
      cell: (r) => {
        const st = str(r, "thbq_status");
        return st ? <Badge tone={THBQ_BADGE[st] ?? "gray"}>{THBQ_STATUS_LABEL[st] ?? st}</Badge> : null;
      },
    },
    { key: "so_van_ban", header: "Văn bản", align: "right", width: 80, cell: (r) => str(r, "so_van_ban") },
  ];
  const colsHien = cols.filter((c) => !anCot.has(c.key));
  const TEN_COT: Record<string, string> = { hop_so: "Hộp", ma_ho_so: "Mã hồ sơ", so_to: "Số tờ", ngay_bat_dau: "Bắt đầu", so_van_ban: "Văn bản", ten_mang: "Mảng" };

  return (
    <section className="view active">
      <PageHeader
        title="Lập hồ sơ điện tử"
        description={`Hồ sơ tạm trong dự án ${session?.projectName ?? ""}. Làm được khi mất mạng, tự đồng bộ khi có máy chủ.`}
        meta={<SyncBadge sync={sync} />}
        actions={
          <>
            {canCreate && (
              <Button variant="primary" icon="them" disabled={!phongId} onClick={() => setForm({ row: null })}>
                Thêm hồ sơ
              </Button>
            )}
            <Menu
              items={[
                { label: sel.size > 0 ? `Xoá ${n(sel.size)} hồ sơ đã chọn` : "Xoá hồ sơ đã chọn", icon: "xoa", danger: true, disabled: !canDelete || sel.size === 0, onSelect: () => void xoaChon() },
                "-",
                { label: "Tải mẫu Excel lập hồ sơ", icon: "tepExcel", disabled: !phong, onSelect: () => void xuatMauLapHoSo(str(phong, "ma_du_an"), cat.thbq).catch((e) => notifyError(e, "tải mẫu")) },
                {
                  label: "Nhập mục lục hồ sơ",
                  icon: "nhapTep",
                  // Kiểm đầu luồng (2/11/2026): mã hồ sơ TT05 cần số phông + số mục lục; xếp hộp cần quyền Hệ thống hoá.
                  hint: !phong
                    ? "Chọn phông"
                    : !str(phong, "phong_so") || !str(phong, "muc_luc_so")
                      ? "Phông chưa có số phông / số mục lục → mã hồ sơ sẽ thiếu (sửa ở Danh mục › Phông)"
                      : !perms.includes("dossier.systematize")
                        ? "Tài khoản không có quyền Hệ thống hoá → hộp số, hồ sơ số sẽ không được xếp"
                        : "Tự nhận phòng ban, mảng, sheet; kiểm và điền thời gian; xếp hộp ngay",
                  disabled: !canCreate || !phong,
                  onSelect: () => setMucLuc(true),
                },
                { label: "Nhập hồ sơ từ Excel (mẫu 14 cột)", icon: "nhapTep", hint: "Cần máy chủ", disabled: !canCreate || !phongId || sync?.phase === "offline", onSelect: () => fileRef.current?.click() },
                "-",
                ...Object.entries(TEN_COT).map(([k, l]) => ({ label: `${anCot.has(k) ? "Hiện" : "Ẩn"} cột ${l}`, onSelect: () => setAnCot((s) => { const x = new Set(s); if (!x.delete(k)) x.add(k); return x; }) })),
              ]}
            />
            <input
              ref={fileRef}
              type="file"
              accept=".xlsx,.xls"
              hidden
              aria-hidden
              onChange={(e) => {
                const f = e.target.files?.[0];
                e.target.value = "";
                if (f) void nhapExcel(f);
              }}
            />
          </>
        }
      />
      <ChuaChonDuAn />
      {catErr && <ErrorBanner error={catErr} className="mb-4" />}
      <CanMayChu sync={sync} viec="Thay đổi được giữ ở máy và tự đẩy lên máy chủ khi có mạng." />
      {cat.phong.length === 0 && !catErr && <Banner tone="info" className="mb-4">Chưa có phông nào trong dự án. Quản trị tạo phông ở Danh mục khi có máy chủ, hoặc chờ đồng bộ kéo danh mục về.</Banner>}
      {nhapKq && (
        <Banner tone={nhapKq.errors.length ? "warning" : "success"} className="mb-4" onClose={() => setNhapKq(null)} detail={nhapKq.errors.length ? nhapKq.errors.join("\n") : undefined}>
          {nhapKq.text}
        </Banner>
      )}

      <Card className="mb-4" padding="sm">
        <FormGrid cols={4}>
          <Field label="Phông">
            <Select
              size="sm"
              value={phongId}
              onChange={(v) => {
                setPhongId(v);
                setPhongBanId("");
                setMangId("");
                setPage(1);
              }}
              rows={cat.phong}
              render={tenPhong}
              empty="Tất cả phông"
            />
          </Field>
          <Field label="Phòng ban">
            <Select
              size="sm"
              value={phongBanId}
              onChange={(v) => {
                setPhongBanId(v);
                setMangId("");
                setPage(1);
              }}
              rows={phongBans}
              empty="Tất cả"
            />
          </Field>
          <Field label="Mảng công việc">
            <Select size="sm" value={mangId} onChange={datLoc(setMangId)} rows={mangs} empty="Tất cả" disabled={!phongBanId} />
          </Field>
          <Field label="Trạng thái THBQ">
            <UiSelect size="sm" value={status} onChange={(v) => datLoc(setStatus)(v as typeof status)} placeholder="Tất cả" options={[{ value: "VV", label: "Vĩnh viễn" }, { value: "GL", label: "Giữ lại" }, { value: "HET", label: "Hết thời hạn bảo quản" }]} />
          </Field>
        </FormGrid>
        <div className="mt-3 flex flex-wrap items-center gap-2">
          <Input size="sm" prefix="tim" className="w-72" value={q} onChange={(e) => setQ(e.target.value)} placeholder="Tìm số tạm, tiêu đề, số hoặc mã hồ sơ" aria-label="Tìm hồ sơ" />
          <span className="text-sm text-fg-subtle">THBQ:</span>
          <Chip on={coThoiHan} onClick={() => datLoc(setCoThoiHan)(!coThoiHan)}>
            Có thời hạn (5 đến 70 năm)
          </Chip>
          {cat.thbq.map((t) => (
            <Chip key={t.id} on={thbqIds.includes(t.id)} onClick={() => toggleThbq(t.id)}>
              {str(t, "ten")}
            </Chip>
          ))}
          <Chip on={trungThua} onClick={() => datLoc(setTrungThua)(!trungThua)}>
            Chỉ trùng thừa
          </Chip>
        </div>
      </Card>

      <Card padding="none">
        <Table
          columns={colsHien}
          rows={rows}
          getRowId={(r) => r.id}
          dense
          sort={sort}
          onSortChange={(s) => {
            setSort(s);
            setPage(1);
          }}
          clientSort={false}
          selection={canDelete ? { selected: sel, onChange: setSel } : undefined}
          onRowClick={(r) => setOpen(r)}
          rowClassName={(r) => (open?.id === r.id ? "bg-brand-50" : undefined)}
          rowActions={canEdit ? (r) => <Menu size="sm" items={[{ label: "Sửa", icon: "sua", onSelect: () => setForm({ row: r }) }, { label: "Xem văn bản", icon: "taiLieu", onSelect: () => setOpen(r) }]} /> : undefined}
          pagination={{ page, pageSize: PAGE, total, onChange: setPage }}
          state={{ loading: loading && !data, error, onRetry: reload, empty: { title: qApplied || thbqIds.length || status || trungThua ? "Không có hồ sơ nào khớp bộ lọc" : "Chưa có hồ sơ nào", action: canCreate && phongId ? <Button icon="them" onClick={() => setForm({ row: null })}>Thêm hồ sơ</Button> : undefined } }}
        />
      </Card>

      {form && (
        <HoSoForm
          cat={cat}
          phongId={phongId}
          row={form.row}
          onClose={() => setForm(null)}
          onSaved={(_id, keepOpen) => {
            if (!keepOpen) setForm(null);
            toast.success("Đã lưu hồ sơ.");
            reload();
          }}
        />
      )}
      {open && <VanBanPanel cat={cat} hoSo={open} canEdit={canEdit} onClose={() => setOpen(null)} />}
      {mucLuc && phong && (
        <NhapMucLucDialog
          cat={cat}
          phong={phong}
          offline={sync?.phase === "offline"}
          onClose={() => setMucLuc(false)}
          onImported={() => {
            reloadCat();
            reload();
          }}
        />
      )}
    </section>
  );
}
