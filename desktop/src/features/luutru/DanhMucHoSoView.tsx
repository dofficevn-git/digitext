// Copyright (c) 2026 DigiText. All rights reserved.
// Danh mục hồ sơ dự kiến của cơ quan (Điều 19.4 TT 05/2025, 26/11/2026): một danh mục / phông / năm, mỗi phòng ban soạn
// phần mình; trình → duyệt → ban hành (số quyết định); phiên bản mới chép dòng cũ; đối chiếu hồ sơ theo / ngoài danh mục.

import { useCallback, useEffect, useMemo, useState } from "react";
import { isTauri } from "@/api/ipc";
import { V41t } from "@/live/useLive";
import { V460 } from "@/state";
import { useProject } from "@/state/project";
import { V3z9 } from "@/domain/permissions";
import { str } from "@/api/chinhly";
import { V4t1 } from "@/api/records";
import { dmBanHanh, dmBienBan, dmDoiChieu, dmDuyet, dmGet, dmList, dmNhapDong, dmPhienBanMoi, dmSuaDong, dmTao, dmThemDong, dmTraLai, dmTrinh, dmXoaDong, type DanhMuc, type DongDanhMuc, type DongMoi } from "@/api/luutru";
import { useClCatalog, Select as ClSelect, tenPhong } from "@/features/chinhly/common";
import { Badge, Banner, Button, Card, confirm, Dialog, EmptyState, Field, FormGrid, Input, Menu, notifyError, PageHeader, prompt, Select as UiSelect, Stepper, Table, Tabs, Textarea, toast, type Column } from "@/ui";
import { BUOC_DANH_MUC, ngayVn, TT_DANH_MUC } from "./luuTruFlow";

const n = (v: number) => v.toLocaleString("vi-VN");
type Tab = "dong" | "doichieu";

export default function DanhMucHoSoView() {
  const { session } = V460();
  const { projectId: duAnChon } = useProject();
  const duAn = session?.projectId ?? duAnChon ?? null;
  const perms = session?.permissions ?? [];
  const canEdit = V3z9(perms, "danhmuc.edit");
  const canSubmit = V3z9(perms, "danhmuc.submit");
  const canApprove = V3z9(perms, "danhmuc.approve");
  const isSuper = session?.role === "super_admin";
  const pbCuaToi = session?.phongBanId ?? null;
  const { cat } = useClCatalog();
  const namNay = new Date().getFullYear();
  const [phongId, setPhongId] = useState("");
  const [nam, setNam] = useState(String(namNay + 1));
  const [selId, setSelId] = useState<string | null>(null);
  const [tab, setTab] = useState<Tab>("dong");
  const [busy, setBusy] = useState(false);
  const [dongDlg, setDongDlg] = useState<{ row: DongDanhMuc | null } | null>(null);
  const [danDlg, setDanDlg] = useState(false);
  const dungDuoc = isTauri() && !!duAn;
  useEffect(() => {
    if (!phongId && cat.phong.length > 0) setPhongId(str(cat.phong[0], "id"));
  }, [cat.phong, phongId]);

  const loader = useCallback(() => (duAn ? dmList(duAn, phongId || null, null) : Promise.resolve(null)), [duAn, phongId]);
  const { data, loading, error, reload } = V41t(loader, { topics: ["danhMuc", "notifications"], enabled: dungDuoc, errorText: () => "Không tải được danh mục hồ sơ." });
  const chiTietLoader = useCallback(() => (selId ? dmGet(selId) : Promise.resolve(null)), [selId]);
  const chiTiet = V41t(chiTietLoader, { topics: ["danhMuc"], enabled: dungDuoc && !!selId, errorText: () => "Không tải được danh mục." });
  const doiChieuLoader = useCallback(() => (selId && tab === "doichieu" ? dmDoiChieu(selId) : Promise.resolve(null)), [selId, tab]);
  const doiChieu = V41t(doiChieuLoader, { topics: ["danhMuc", "chinhLy"], enabled: dungDuoc && !!selId && tab === "doichieu", errorText: () => "Không đối chiếu được." });

  const list = useMemo(() => data?.rows ?? [], [data]);
  useEffect(() => {
    if (list.length > 0 && (!selId || !list.some((d) => d.id === selId))) setSelId(list[0].id);
  }, [list, selId]);
  const d = chiTiet.data;
  const dm = d?.danhMuc;
  const duThao = dm?.trang_thai === "du_thao";
  const suaDuoc = !!dm && duThao && canEdit;
  const phongBans = useMemo(() => cat.phong_ban.filter((p) => str(p, "phong_id") === (dm?.phong_id ?? phongId)), [cat.phong_ban, dm, phongId]);
  const capNhat = () => {
    reload();
    chiTiet.reload();
  };
  const chay = async (viec: string, f: () => Promise<unknown>, xong?: string) => {
    setBusy(true);
    try {
      await f();
      if (xong) toast.success(xong);
      capNhat();
    } catch (e) {
      notifyError(e, viec);
    } finally {
      setBusy(false);
    }
  };

  const tao = async () => {
    if (!duAn || !phongId) return;
    const y = Number(nam);
    const cu = list.find((x) => x.trang_thai === "ban_hanh");
    const ok = await confirm({ title: `Lập danh mục hồ sơ năm ${y}?`, body: cu ? `Chép các dòng từ danh mục đang hiệu lực năm ${cu.nam} để sửa.` : "Danh mục trống, thêm dòng theo từng phòng ban.", confirmLabel: "Lập danh mục" });
    if (!ok) return;
    await chay("lập danh mục", async () => {
      const r = await dmTao({ projectId: duAn, phongId, nam: y, saoChepTu: cu?.id ?? null });
      setSelId(r.id);
    }, `Đã lập danh mục năm ${y}.`);
  };
  const trinh = () => dm && void chay("trình duyệt", () => dmTrinh(dm.id), "Đã trình duyệt danh mục.");
  const duyet = () => dm && void chay("duyệt", () => dmDuyet(dm.id), "Đã duyệt danh mục.");
  const traLai = async () => {
    if (!dm) return;
    const lyDo = await prompt({ title: "Trả lại danh mục", label: "Lý do", type: "textarea", required: true });
    if (!lyDo) return;
    await chay("trả lại", () => dmTraLai(dm.id, lyDo), "Đã trả lại để sửa.");
  };
  const banHanh = async () => {
    if (!dm) return;
    const so = await prompt({ title: "Ban hành danh mục hồ sơ", label: "Số quyết định ban hành", required: true, placeholder: "Ví dụ: 12/QĐ-UBND" });
    if (!so) return;
    const ngay = await prompt({ title: "Ngày ban hành", label: "Ngày", type: "date", required: true, initial: new Date().toISOString().slice(0, 10) });
    if (!ngay) return;
    await chay("ban hành", () => dmBanHanh(dm.id, so, ngay), `Đã ban hành danh mục năm ${dm.nam}.`);
  };
  const phienBanMoi = async () => {
    if (!dm) return;
    const ok = await confirm({ title: `Lập phiên bản ${dm.phien_ban + 1} của danh mục năm ${dm.nam}?`, body: "Chép toàn bộ dòng sang bản dự thảo mới. Bản đang hiệu lực giữ nguyên tới khi bản mới ban hành.", confirmLabel: "Lập phiên bản mới" });
    if (!ok) return;
    await chay("lập phiên bản", async () => {
      const r = await dmPhienBanMoi(dm.id);
      setSelId(r.id);
    }, "Đã lập phiên bản mới.");
  };
  const bienBan = async () => {
    if (!dm) return;
    await chay("lập bản in", async () => {
      const r = await dmBienBan(dm.id);
      const path = await V4t1(r.id, `danh-muc-ho-so-${dm.nam}-v${dm.phien_ban}.pdf`);
      toast.success(`Đã lập ${r.soBienBan} và lưu: ${path}`);
    });
  };
  const xoaDong = async (r: DongDanhMuc) => {
    if (!dm) return;
    const ok = await confirm({ title: `Xoá dòng ${r.ma_dong}?`, body: r.so_ho_so ? `${n(r.so_ho_so)} hồ sơ đang gắn dòng này sẽ thành ngoài danh mục.` : undefined, confirmLabel: "Xoá", danger: true });
    if (!ok) return;
    await chay("xoá dòng", () => dmXoaDong(dm.id, r.id));
  };

  const colsDm: Column<DanhMuc>[] = [
    { key: "nam", header: "Năm", width: 70, cell: (x) => <span className="font-medium">{x.nam}</span> },
    { key: "phien_ban", header: "PB", width: 48, align: "right", cell: (x) => x.phien_ban },
    { key: "so_dong", header: "Dòng", width: 60, align: "right", cell: (x) => n(x.so_dong) },
    { key: "trang_thai", header: "Trạng thái", cell: (x) => <Badge tone={TT_DANH_MUC[x.trang_thai].tone}>{TT_DANH_MUC[x.trang_thai].label}</Badge> },
  ];
  const colsDong: Column<DongDanhMuc>[] = [
    { key: "so_tt", header: "#", width: 44, align: "right", cell: (r) => r.so_tt ?? "" },
    { key: "ten_phong_ban", header: "Phòng ban", width: 170, cell: (r) => r.ten_phong_ban ?? "—" },
    { key: "ma_dong", header: "Số, ký hiệu", width: 120, cell: (r) => <span className="font-medium">{r.ma_dong}</span> },
    { key: "tieu_de", header: "Tiêu đề hồ sơ dự kiến", className: "whitespace-normal", cell: (r) => r.tieu_de },
    { key: "ten_mang", header: "Mảng", width: 140, cell: (r) => r.ten_mang ?? "" },
    { key: "ten_thbq", header: "THBQ", width: 110, cell: (r) => r.ten_thbq ?? "" },
    { key: "nguoi_lap", header: "Người lập", width: 140, cell: (r) => r.nguoi_lap ?? "" },
    { key: "so_ho_so", header: "HS", width: 56, align: "right", cell: (r) => (r.so_ho_so ? n(r.so_ho_so) : "") },
  ];
  const dongCuaToi = (r: DongDanhMuc) => !pbCuaToi || r.phong_ban_id === pbCuaToi;

  return (
    <section className="view active space-y-4">
      <PageHeader
        title="Danh mục hồ sơ"
        description="Danh mục hồ sơ dự kiến lập trong năm của cơ quan"
        infoTitle="Danh mục hồ sơ (Điều 19.4)"
        info={
          <>
            <p>Mỗi phông một danh mục cho một năm; từng phòng ban kê các hồ sơ dự kiến lập (số ký hiệu, tiêu đề, thời hạn bảo quản, người lập). Văn thư / lưu trữ cơ quan trình, lãnh đạo duyệt rồi ban hành kèm số quyết định.</p>
            <p>Hồ sơ lập trong năm gắn vào một dòng của danh mục đang hiệu lực. Tab Đối chiếu cho biết dòng nào chưa có hồ sơ và hồ sơ nào nằm ngoài danh mục. Sửa giữa năm: lập phiên bản mới, số ký hiệu dòng giữ nguyên.</p>
          </>
        }
        actions={
          canEdit && (
            <span className="flex items-center gap-2">
              <UiSelect size="sm" aria-label="Năm lập" value={nam} onChange={setNam} options={[namNay - 1, namNay, namNay + 1].map((y) => ({ value: String(y), label: `Năm ${y}` }))} />
              <Button variant="primary" icon="them" disabled={!phongId || busy || list.some((x) => x.nam === Number(nam) && x.trang_thai !== "het_hieu_luc")} onClick={() => void tao()}>Lập danh mục</Button>
            </span>
          )
        }
      />
      {!duAn && <EmptyState title="Chưa chọn dự án" description="Chọn dự án (kho lưu trữ) ở thanh trên." />}
      {duAn && (
        <div className="grid gap-4 lg:grid-cols-[280px_minmax(0,1fr)]">
          <Card padding="none" title={<ClSelect size="sm" className="w-full" ariaLabel="Phông" value={phongId} onChange={setPhongId} rows={cat.phong} render={tenPhong} empty="Chọn phông" />}>
            <Table columns={colsDm} rows={list} getRowId={(x) => x.id} dense maxHeight={560} onRowClick={(x) => setSelId(x.id)} rowClassName={(x) => (x.id === selId ? "bg-brand-50" : undefined)} state={{ loading: loading && !data, error: data ? undefined : error, onRetry: reload, empty: { title: "Chưa có danh mục", description: canEdit ? "Chọn năm rồi bấm Lập danh mục." : undefined } }} />
          </Card>
          <div className="space-y-3">
            {!dm && <EmptyState title="Chọn một danh mục" />}
            {dm && (
              <>
                <Card
                  title={<span className="flex items-center gap-2">Danh mục hồ sơ năm {dm.nam} · phiên bản {dm.phien_ban} <Badge tone={TT_DANH_MUC[dm.trang_thai].tone}>{TT_DANH_MUC[dm.trang_thai].label}</Badge></span>}
                  description={`${dm.ten_phong} · ${n(dm.so_dong)} dòng của ${n(dm.so_phong_ban)} phòng ban${dm.so_quyet_dinh ? ` · QĐ ${dm.so_quyet_dinh} ngày ${ngayVn(dm.ngay_ban_hanh)}` : ""}`}
                  actions={
                    <>
                      {duThao && canSubmit && <Button size="sm" variant="primary" icon="duyet" disabled={busy || dm.so_dong === 0} onClick={trinh}>Trình duyệt</Button>}
                      {dm.trang_thai === "cho_duyet" && canApprove && (dm.trinh_boi !== session?.userId || isSuper) && (
                        <>
                          <Button size="sm" variant="dangerOutline" disabled={busy} onClick={() => void traLai()}>Trả lại</Button>
                          <Button size="sm" variant="primary" icon="duyet" disabled={busy} onClick={duyet}>Duyệt</Button>
                        </>
                      )}
                      {dm.trang_thai === "da_duyet" && canApprove && <Button size="sm" variant="primary" icon="bienBan" disabled={busy} onClick={() => void banHanh()}>Ban hành…</Button>}
                      {dm.trang_thai === "ban_hanh" && canEdit && <Button size="sm" icon="lamLai" disabled={busy} onClick={() => void phienBanMoi()}>Phiên bản mới</Button>}
                      {(dm.trang_thai === "ban_hanh" || dm.trang_thai === "da_duyet" || dm.trang_thai === "het_hieu_luc") && <Button size="sm" icon="inRa" disabled={busy} onClick={() => void bienBan()}>Bản in</Button>}
                    </>
                  }
                >
                  <Stepper steps={BUOC_DANH_MUC} current={dm.trang_thai === "ban_hanh" || dm.trang_thai === "het_hieu_luc" ? 4 : TT_DANH_MUC[dm.trang_thai].step} />
                  {dm.ly_do_tra && duThao && <Banner tone="warning" className="mt-2">Bị trả lại: {dm.ly_do_tra}</Banner>}
                  {dm.trinh_ten && dm.trang_thai !== "du_thao" && <p className="mt-2 text-sm text-fg-subtle">Trình: {dm.trinh_ten} ({ngayVn(dm.trinh_luc)}){dm.duyet_ten ? ` · Duyệt: ${dm.duyet_ten} (${ngayVn(dm.duyet_luc)})` : ""}{dm.ban_hanh_ten ? ` · Ban hành: ${dm.ban_hanh_ten}` : ""}</p>}
                </Card>
                <Tabs items={[{ value: "dong", label: "Dòng danh mục", count: d?.dong.length }, { value: "doichieu", label: "Đối chiếu hồ sơ" }]} value={tab} onChange={setTab} />
                {tab === "dong" && (
                  <Card
                    padding="none"
                    actions={
                      suaDuoc && (
                        <>
                          <Button size="sm" icon="nhapTep" disabled={busy} onClick={() => setDanDlg(true)}>Dán từ bảng tính</Button>
                          <Button size="sm" variant="primary" icon="them" disabled={busy} onClick={() => setDongDlg({ row: null })}>Thêm dòng</Button>
                        </>
                      )
                    }
                    title={pbCuaToi ? <span className="text-sm text-fg-subtle">Bạn chỉ sửa được dòng của {session?.tenPhongBan ?? "phòng mình"}.</span> : undefined}
                  >
                    <Table
                      columns={colsDong}
                      rows={d?.dong ?? []}
                      getRowId={(r) => r.id}
                      dense
                      maxHeight={480}
                      rowActions={suaDuoc ? (r) => (dongCuaToi(r) ? <Menu size="sm" items={[{ label: "Sửa", icon: "sua", onSelect: () => setDongDlg({ row: r }) }, { label: "Xoá", icon: "xoa", danger: true, onSelect: () => void xoaDong(r) }]} /> : null) : undefined}
                      state={{ loading: chiTiet.loading && !d, empty: { title: "Chưa có dòng nào", description: suaDuoc ? "Thêm từng dòng hoặc dán từ bảng tính (số ký hiệu, tiêu đề, THBQ, người lập)." : undefined } }}
                    />
                  </Card>
                )}
                {tab === "doichieu" && (
                  <div className="grid gap-3 xl:grid-cols-2">
                    <Card title={`Dòng danh mục (${n(doiChieu.data?.theoDanhMuc.length ?? 0)})`} description="Số hồ sơ đã lập theo từng dòng; 0 là chưa lập." padding="none">
                      <Table
                        columns={[
                          { key: "ma", header: "Số, ký hiệu", width: 120, cell: (r) => r.maDong },
                          { key: "td", header: "Tiêu đề", className: "whitespace-normal", cell: (r) => r.tieuDe },
                          { key: "pb", header: "Phòng ban", width: 150, cell: (r) => r.tenPhongBan ?? "" },
                          { key: "so", header: "HS", width: 60, align: "right", cell: (r) => <Badge tone={r.soHoSo ? "ok" : "warn"}>{n(r.soHoSo)}</Badge> },
                        ]}
                        rows={doiChieu.data?.theoDanhMuc ?? []}
                        getRowId={(r) => r.dongId}
                        dense
                        maxHeight={440}
                        state={{ loading: doiChieu.loading && !doiChieu.data, error: doiChieu.error ?? undefined, onRetry: doiChieu.reload }}
                      />
                    </Card>
                    <Card title={`Hồ sơ ngoài danh mục (${n(doiChieu.data?.ngoaiDanhMuc.length ?? 0)})`} description="Hồ sơ lập trong năm chưa gắn dòng nào; mở ở Lập hồ sơ để gắn." padding="none">
                      <Table
                        columns={[
                          { key: "so", header: "Số tạm", width: 90, cell: (r) => r.soTam },
                          { key: "td", header: "Tiêu đề", className: "whitespace-normal", cell: (r) => r.tieuDe },
                          { key: "pb", header: "Phòng ban", width: 150, cell: (r) => r.tenPhongBan ?? "—" },
                          { key: "kt", header: "Kết thúc", width: 100, cell: (r) => r.ngayKetThuc ?? "" },
                        ]}
                        rows={doiChieu.data?.ngoaiDanhMuc ?? []}
                        getRowId={(r) => r.id}
                        dense
                        maxHeight={440}
                        state={{ loading: doiChieu.loading && !doiChieu.data, empty: { title: "Mọi hồ sơ đều theo danh mục" } }}
                      />
                    </Card>
                  </div>
                )}
              </>
            )}
          </div>
        </div>
      )}
      {dongDlg && dm && (
        <DongDialog
          row={dongDlg.row}
          phongBans={phongBans}
          pbKhoa={pbCuaToi}
          cat={{ mang: cat.mang, thbq: cat.thbq }}
          busy={busy}
          onClose={() => setDongDlg(null)}
          onSave={async (dong) => {
            await chay(dongDlg.row ? "sửa dòng" : "thêm dòng", () => (dongDlg.row ? dmSuaDong(dm.id, dongDlg.row.id, dong) : dmThemDong(dm.id, dong)));
            setDongDlg(null);
          }}
        />
      )}
      {danDlg && dm && (
        <DanDialog
          phongBans={phongBans}
          pbKhoa={pbCuaToi}
          thbq={cat.thbq}
          busy={busy}
          onClose={() => setDanDlg(false)}
          onSave={async (rows) => {
            await chay("nhập dòng", async () => {
              const r = await dmNhapDong(dm.id, rows);
              toast.success(`Đã thêm ${n(r.them)}, sửa ${n(r.sua)}${r.loi.length ? `, ${n(r.loi.length)} dòng lỗi: ${r.loi.map((l) => `dòng ${l.dong}: ${l.loi}`).join("; ")}` : ""}.`);
            });
            setDanDlg(false);
          }}
        />
      )}
    </section>
  );
}

type Row = Record<string, unknown> & { id: string };

function DongDialog({ row, phongBans, pbKhoa, cat, busy, onClose, onSave }: { row: DongDanhMuc | null; phongBans: Row[]; pbKhoa: string | null; cat: { mang: Row[]; thbq: Row[] }; busy: boolean; onClose: () => void; onSave: (d: DongMoi) => Promise<void> }) {
  const [pb, setPb] = useState(row?.phong_ban_id ?? pbKhoa ?? (phongBans.length === 1 ? str(phongBans[0], "id") : ""));
  const [mang, setMang] = useState(row?.mang_id ?? "");
  const [ma, setMa] = useState(row?.ma_dong ?? "");
  const [tieuDe, setTieuDe] = useState(row?.tieu_de ?? "");
  const [thbq, setThbq] = useState(row?.thbq_id ?? "");
  const [nguoiLap, setNguoiLap] = useState(row?.nguoi_lap ?? "");
  const [ghiChu, setGhiChu] = useState(row?.ghi_chu ?? "");
  const [touched, setTouched] = useState(false);
  const mangs = useMemo(() => cat.mang.filter((m) => str(m, "phong_ban_id") === pb), [cat.mang, pb]);
  const errs = { pb: !pb ? "Chọn phòng ban." : undefined, ma: !ma.trim() ? "Chưa nhập số ký hiệu." : undefined, tieuDe: !tieuDe.trim() ? "Chưa nhập tiêu đề." : undefined };
  const valid = !errs.pb && !errs.ma && !errs.tieuDe;
  const save = () => {
    setTouched(true);
    if (!valid) return;
    void onSave({ phongBanId: pb, mangId: mang || null, maDong: ma.trim(), tieuDe: tieuDe.trim(), thbqId: thbq || null, nguoiLap: nguoiLap.trim() || null, ghiChu: ghiChu.trim() || null });
  };
  return (
    <Dialog open onOpenChange={(o) => !o && !busy && onClose()} title={row ? `Sửa dòng ${row.ma_dong}` : "Thêm dòng danh mục"} size="md" footer={<><Button onClick={onClose} disabled={busy}>Huỷ</Button><Button variant="primary" icon="luu" loading={busy} onClick={save}>Lưu</Button></>}>
      <FormGrid cols={2}>
        <Field label="Phòng ban" required error={touched ? errs.pb : undefined}>
          <ClSelect value={pb} onChange={(v) => { setPb(v); setMang(""); }} rows={phongBans} empty="Chọn" disabled={!!pbKhoa || !!row} />
        </Field>
        <Field label="Mảng công việc">
          <ClSelect value={mang} onChange={setMang} rows={mangs} empty="Chưa chọn" disabled={!pb} />
        </Field>
        <Field label="Số, ký hiệu hồ sơ" required error={touched ? errs.ma : undefined} hint="Giữ nguyên qua các phiên bản">
          <Input value={ma} onChange={(e) => setMa(e.target.value)} autoFocus />
        </Field>
        <Field label="Thời hạn bảo quản">
          <ClSelect value={thbq} onChange={setThbq} rows={cat.thbq} empty="Chưa chọn" />
        </Field>
      </FormGrid>
      <Field label="Tiêu đề hồ sơ dự kiến" required error={touched ? errs.tieuDe : undefined} className="mt-3">
        <Textarea rows={2} value={tieuDe} onChange={(e) => setTieuDe(e.target.value)} />
      </Field>
      <FormGrid cols={2} className="mt-3">
        <Field label="Người lập hồ sơ">
          <Input value={nguoiLap} onChange={(e) => setNguoiLap(e.target.value)} />
        </Field>
        <Field label="Ghi chú">
          <Input value={ghiChu} onChange={(e) => setGhiChu(e.target.value)} />
        </Field>
      </FormGrid>
    </Dialog>
  );
}

/** Dán từ Excel: mỗi dòng `số ký hiệu ⇥ tiêu đề ⇥ THBQ (tên, tuỳ chọn) ⇥ người lập (tuỳ chọn)`. */
function DanDialog({ phongBans, pbKhoa, thbq, busy, onClose, onSave }: { phongBans: Row[]; pbKhoa: string | null; thbq: Row[]; busy: boolean; onClose: () => void; onSave: (rows: DongMoi[]) => Promise<void> }) {
  const [pb, setPb] = useState(pbKhoa ?? (phongBans.length === 1 ? str(phongBans[0], "id") : ""));
  const [text, setText] = useState("");
  const rows = useMemo<DongMoi[]>(() => {
    if (!pb) return [];
    const out: DongMoi[] = [];
    for (const line of text.split(/\r?\n/)) {
      const c = line.split("\t").map((s) => s.trim());
      if (!c[0] || !c[1]) continue;
      if (c[0].toLowerCase().startsWith("số") && c[1].toLowerCase().startsWith("tiêu")) continue;
      const th = c[2] ? thbq.find((t) => str(t, "ten").toLowerCase() === c[2].toLowerCase() || str(t, "ma").toLowerCase() === c[2].toLowerCase()) : undefined;
      out.push({ phongBanId: pb, maDong: c[0], tieuDe: c[1], thbqId: th ? str(th, "id") : null, nguoiLap: c[3] || null });
    }
    return out;
  }, [text, pb, thbq]);
  return (
    <Dialog open onOpenChange={(o) => !o && !busy && onClose()} title="Dán dòng danh mục từ bảng tính" size="lg" footer={<><span className="self-center text-sm text-fg-subtle">{rows.length ? `${n(rows.length)} dòng hợp lệ` : ""}</span><span className="flex-1" /><Button onClick={onClose} disabled={busy}>Huỷ</Button><Button variant="primary" icon="nhapTep" loading={busy} disabled={rows.length === 0} onClick={() => void onSave(rows)}>Nhập {rows.length ? n(rows.length) : ""} dòng</Button></>}>
      <div className="space-y-3">
        <Field label="Phòng ban" required>
          <ClSelect value={pb} onChange={setPb} rows={phongBans} empty="Chọn" disabled={!!pbKhoa} />
        </Field>
        <Field label="Dán các cột: số ký hiệu, tiêu đề, thời hạn bảo quản, người lập" hint="Chép từ Excel rồi dán vào đây; dòng trùng số ký hiệu sẽ được cập nhật tiêu đề.">
          <Textarea rows={12} value={text} onChange={(e) => setText(e.target.value)} placeholder={"01/VP\tHồ sơ xây dựng chương trình công tác năm\tVĩnh viễn\tNguyễn Văn A"} className="font-mono text-sm" />
        </Field>
      </div>
    </Dialog>
  );
}
