// Copyright (c) 2026 DigiText. All rights reserved.
// Kế hoạch kiểm tra tài liệu hằng năm (Điều 29, 0032): lập → trình → duyệt (chốt tệp) → máy chủ kiểm dần → kết luận bằng biên
// bản Mẫu 01 điền sẵn; danh mục lỗi với khắc phục / kiểm lại từng dòng.

import { useCallback, useMemo, useState } from "react";
import { isTauri } from "@/api/ipc";
import { V41t } from "@/live/useLive";
import { V460 } from "@/state";
import { useProject } from "@/state/project";
import { V3z9 } from "@/domain/permissions";
import { str } from "@/api/chinhly";
import { khChay, khDuyet, khGanBienBan, khGet, khHuy, khKetLuan, khList, khSua, khTao, khTep, khTrinh, khTuChoi, khXemTruoc, loiBoQua, loiKhacPhuc, loiKiemLai, loiList, type KeHoach, type KeHoachBody, type LoiBaoQuan, type PhamViKiemTra, type TepKeHoach } from "@/api/baoquan";
import { useClCatalog, tenPhong } from "@/features/chinhly/common";
import { LapBienBanDialog, type NoiDungDien } from "@/features/records/RecordsView";
import { Badge, Button, Card, Checkbox, confirm, Dialog, EmptyState, Field, FormGrid, Input, Menu, MultiSelect, notifyError, prompt, Seg, Stepper, Table, Textarea, Tip, toast, type Column } from "@/ui";
import { BUOC_KE_HOACH, fmtN, ganBienBanMoiNhat, ngay, ngayGio, TT_KE_HOACH, TT_LOI, TT_TEP } from "./baoQuanFlow";

export default function KeHoachPanel() {
  const { session } = V460();
  const { projectId: duAnChon } = useProject();
  const duAn = session?.projectId ?? duAnChon ?? null;
  const perms = session?.permissions ?? [];
  const canManage = V3z9(perms, "bk.manage");
  const canApprove = V3z9(perms, "bk.approve");
  const isSuper = session?.role === "super_admin";
  const { cat } = useClCatalog();
  const [selId, setSelId] = useState<string | null>(null);
  const [tepLoc, setTepLoc] = useState<"" | "loi" | "cho" | "dat">("");
  const [dlg, setDlg] = useState<{ row: KeHoach | null } | null>(null);
  const [bienBan, setBienBan] = useState<NoiDungDien | null>(null);
  const [busy, setBusy] = useState(false);
  const dungDuoc = isTauri() && !!duAn;

  const loader = useCallback(() => (duAn ? khList(duAn) : Promise.resolve(null)), [duAn]);
  const { data, loading, error, reload } = V41t(loader, { topics: ["ops"], enabled: dungDuoc, errorText: () => "Không tải được kế hoạch kiểm tra." });
  const list = useMemo(() => data?.rows ?? [], [data]);
  const chiTietLoader = useCallback(async () => (selId ? { kh: await khGet(selId), tep: await khTep(selId, tepLoc || undefined, 300) } : null), [selId, tepLoc]);
  const chiTiet = V41t(chiTietLoader, { topics: ["ops"], enabled: dungDuoc && !!selId, errorText: () => "Không tải được kế hoạch." });
  const loiLoader = useCallback(() => (duAn ? loiList({ projectId: duAn, mo: true, limit: 300 }) : Promise.resolve(null)), [duAn]);
  const loi = V41t(loiLoader, { topics: ["ops"], enabled: dungDuoc, errorText: () => "Không tải được danh mục lỗi." });

  const k = chiTiet.data?.kh ?? null;
  const capNhat = () => {
    reload();
    chiTiet.reload();
    loi.reload();
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

  const trinh = () => k && void chay("trình duyệt", () => khTrinh(k.id), `Đã trình kế hoạch ${k.ma}.`);
  const duyet = async () => {
    if (!k) return;
    const ok = await confirm({ title: `Duyệt kế hoạch ${k.ma}?`, body: "Danh sách tệp theo phạm vi được chốt ngay lúc duyệt; máy chủ tự kiểm dần mỗi giờ.", confirmLabel: "Duyệt" });
    if (!ok) return;
    await chay("duyệt", () => khDuyet(k.id), "Đã duyệt, bắt đầu kiểm tự động.");
  };
  const tuChoi = async () => {
    if (!k) return;
    const lyDo = await prompt({ title: `Trả lại kế hoạch ${k.ma}`, label: "Lý do", type: "textarea", required: true });
    if (!lyDo) return;
    await chay("trả lại", () => khTuChoi(k.id, lyDo), "Đã trả lại để sửa.");
  };
  const huy = async () => {
    if (!k) return;
    const ok = await confirm({ title: `Huỷ kế hoạch ${k.ma}?`, confirmLabel: "Huỷ kế hoạch", danger: true });
    if (!ok) return;
    await chay("huỷ", () => khHuy(k.id), "Đã huỷ.");
  };
  const kiemNgay = () => k && void chay("kiểm một lô", async () => {
    const r = await khChay(k.id, 200);
    toast[r.loi ? "warning" : "success"](`Đã kiểm ${fmtN(r.daKiem)} tệp, ${fmtN(r.loi)} lỗi${r.hoanThanh ? " — kế hoạch đã kiểm hết." : "."}`);
  });
  const ketLuan = () => k && void chay("điền biên bản", async () => setBienBan(await khKetLuan(k.id)));
  const khacPhuc = async (l: LoiBaoQuan) => {
    const v = await prompt({ title: `Khắc phục: ${l.ten_tep ?? ""}`, body: l.chi_tiet ?? undefined, label: "Biện pháp đã làm", type: "textarea", required: true, placeholder: "Ví dụ: khôi phục tệp từ bản sao lưu 25/11, kiểm lại mã băm" });
    if (!v) return;
    await chay("khắc phục", () => loiKhacPhuc(l.id, v), "Đã ghi khắc phục. Bấm Kiểm lại để xác nhận.");
  };
  const kiemLai = (l: LoiBaoQuan) => void chay("kiểm lại", async () => {
    const r = await loiKiemLai(l.id);
    toast[r.ketQua === "dat" ? "success" : "warning"](r.ketQua === "dat" ? "Kiểm lại đạt, dòng lỗi đã đóng." : `Vẫn lỗi: ${r.chiTiet ?? ""}`);
  });
  const boQua = async (l: LoiBaoQuan) => {
    const v = await prompt({ title: "Bỏ qua dòng lỗi", label: "Lý do", type: "textarea", required: true });
    if (!v) return;
    await chay("bỏ qua", () => loiBoQua(l.id, v));
  };

  const colsKh: Column<KeHoach>[] = [
    { key: "ma", header: "Mã", width: 110, cell: (x) => <span className="font-medium">{x.ma}</span> },
    { key: "ten", header: "Tên kế hoạch", className: "whitespace-normal", cell: (x) => x.ten },
    { key: "so", header: "Tệp", align: "right", width: 90, cell: (x) => (x.so_tep ? `${fmtN(x.so_dat + x.so_loi)}/${fmtN(x.so_tep)}` : "—") },
    { key: "trang_thai", header: "Trạng thái", width: 120, cell: (x) => <Badge tone={TT_KE_HOACH[x.trang_thai].tone}>{TT_KE_HOACH[x.trang_thai].label}</Badge> },
  ];
  const colsTep: Column<TepKeHoach>[] = [
    { key: "tenTep", header: "Tệp / gói", className: "whitespace-normal", cell: (t) => t.tenTep ?? "—" },
    { key: "maHoSo", header: "Mã hồ sơ", width: 150, cell: (t) => t.maHoSo ?? (t.packageId ? "gói AIP" : "—") },
    { key: "trangThai", header: "Kết quả", width: 100, cell: (t) => <Badge tone={TT_TEP[t.trangThai]?.tone ?? "gray"}>{TT_TEP[t.trangThai]?.label ?? t.trangThai}</Badge> },
    { key: "chiTiet", header: "Chi tiết", className: "whitespace-normal text-sm text-danger-700", cell: (t) => t.chiTiet ?? "" },
    { key: "kiemLuc", header: "Kiểm lúc", width: 140, cell: (t) => ngayGio(t.kiemLuc) },
  ];
  const colsLoi: Column<LoiBaoQuan>[] = [
    { key: "ten_tep", header: "Tệp / gói", className: "whitespace-normal", cell: (l) => l.ten_tep ?? "—" },
    { key: "ma_ho_so", header: "Mã hồ sơ", width: 140, cell: (l) => l.ma_ho_so ?? (l.package_id ? "gói AIP" : "—") },
    { key: "chi_tiet", header: "Lỗi", className: "whitespace-normal text-sm text-danger-700", cell: (l) => l.chi_tiet ?? "" },
    { key: "ke_hoach_ma", header: "Kế hoạch", width: 100, cell: (l) => l.ke_hoach_ma ?? "tự động" },
    { key: "phat_hien_luc", header: "Phát hiện", width: 110, cell: (l) => ngay(l.phat_hien_luc) },
    { key: "trang_thai", header: "Trạng thái", width: 130, cell: (l) => <Badge tone={TT_LOI[l.trang_thai].tone}>{TT_LOI[l.trang_thai].label}</Badge> },
    { key: "khac_phuc", header: "Khắc phục", className: "whitespace-normal text-sm", cell: (l) => (l.khac_phuc ? <Tip text={`${l.khac_phuc_ten ?? ""} · ${ngayGio(l.khac_phuc_luc)}`}><span>{l.khac_phuc}</span></Tip> : "") },
  ];

  const buoc = k ? (k.trang_thai === "hoan_thanh" ? BUOC_KE_HOACH.length : TT_KE_HOACH[k.trang_thai].step) : 0;
  const tyLe = k && k.so_tep ? Math.round(((k.so_dat + k.so_loi) / k.so_tep) * 100) : 0;

  if (!duAn) return <EmptyState title="Chưa chọn dự án" description="Kế hoạch kiểm tra lập theo dự án (kho). Chọn ở thanh trên." />;
  return (
    <div className="space-y-4">
      <div className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_minmax(0,1.4fr)]">
        <Card padding="none" title="Kế hoạch kiểm tra" actions={canManage && <Button size="sm" variant="primary" icon="them" onClick={() => setDlg({ row: null })}>Lập kế hoạch</Button>}>
          <Table columns={colsKh} rows={list} getRowId={(x) => x.id} dense maxHeight={420} onRowClick={(x) => setSelId(x.id)} rowClassName={(x) => (x.id === selId ? "bg-brand-50" : undefined)} state={{ loading: loading && !data, error: data ? undefined : error, onRetry: reload, empty: { title: "Chưa có kế hoạch kiểm tra", description: "Điều 29: mỗi năm lập kế hoạch kiểm tra, mọi tệp được kiểm ít nhất một lần trong 3 năm." } }} />
        </Card>
        <div className="space-y-3">
          {!k && <EmptyState title="Chọn một kế hoạch" />}
          {k && (
            <>
              <Card
                title={<span className="flex items-center gap-2">{k.ma} · {k.ten} <Badge tone={TT_KE_HOACH[k.trang_thai].tone}>{TT_KE_HOACH[k.trang_thai].label}</Badge></span>}
                description={`Năm ${k.nam} · lập bởi ${k.nguoi_lap_ten ?? "—"}${k.duyet_ten ? ` · duyệt ${k.duyet_ten} ${ngay(k.duyet_luc)}` : ""}`}
                actions={
                  <>
                    {k.trang_thai === "du_thao" && canManage && (
                      <>
                        <Button size="sm" icon="sua" onClick={() => setDlg({ row: k })}>Sửa</Button>
                        <Button size="sm" variant="primary" icon="duyet" disabled={busy} onClick={trinh}>Trình duyệt</Button>
                      </>
                    )}
                    {k.trang_thai === "cho_duyet" && canApprove && (k.nguoi_lap !== session?.userId || isSuper) && (
                      <>
                        <Button size="sm" variant="dangerOutline" disabled={busy} onClick={() => void tuChoi()}>Trả lại</Button>
                        <Button size="sm" variant="primary" icon="duyet" disabled={busy} onClick={() => void duyet()}>Duyệt</Button>
                      </>
                    )}
                    {(k.trang_thai === "da_duyet" || k.trang_thai === "dang_thuc_hien") && canManage && <Button size="sm" variant="primary" icon="kiemTra" loading={busy} onClick={kiemNgay}>Kiểm ngay một lô</Button>}
                    {k.trang_thai === "hoan_thanh" && canManage && <Button size="sm" variant={k.record_id ? "secondary" : "primary"} icon="bienBan" disabled={busy} onClick={ketLuan}>{k.record_id ? "Lập lại biên bản" : "Kết luận (Mẫu 01)"}</Button>}
                    {canManage && ["du_thao", "cho_duyet", "da_duyet"].includes(k.trang_thai) && <Menu size="sm" items={[{ label: "Huỷ kế hoạch", icon: "xoa", danger: true, onSelect: () => void huy() }]} />}
                  </>
                }
              >
                <Stepper steps={BUOC_KE_HOACH} current={buoc} />
                {k.ly_do_tu_choi && k.trang_thai === "du_thao" && <p className="mt-2 text-sm text-danger-700">Bị trả lại: {k.ly_do_tu_choi}</p>}
                <p className="mt-2 text-sm text-fg-subtle">Phạm vi: {moTaPhamVi(k.pham_vi, cat.phong)}{k.phuong_phap ? ` · ${k.phuong_phap}` : ""}</p>
                {k.so_tep > 0 && (
                  <div className="mt-3">
                    <div className="flex items-center justify-between text-sm"><span>Đã kiểm {fmtN(k.so_dat + k.so_loi)} / {fmtN(k.so_tep)} tệp ({tyLe}%)</span><span className={k.so_loi ? "text-danger-700" : "text-success-700"}>{fmtN(k.so_loi)} lỗi</span></div>
                    <div className="mt-1 h-2 rounded bg-border-subtle"><div className="h-2 rounded bg-accent" style={{ width: `${Math.min(100, tyLe)}%` }} /></div>
                  </div>
                )}
                {k.record_id && <p className="mt-2 text-sm text-fg-subtle">Đã có biên bản kết luận trong sổ biên bản (loại Kiểm tra tài liệu).</p>}
              </Card>
              {k.so_tep > 0 && (
                <Card padding="none" title={<Seg options={[{ value: "", label: "Tất cả" }, { value: "loi", label: "Lỗi" }, { value: "cho", label: "Chờ" }, { value: "dat", label: "Đạt" }]} value={tepLoc} onChange={setTepLoc} />} description={chiTiet.data ? `${fmtN(chiTiet.data.tep.total)} tệp` : undefined}>
                  <Table columns={colsTep} rows={chiTiet.data?.tep.rows ?? []} getRowId={(t) => t.id} dense maxHeight={360} state={{ loading: chiTiet.loading && !chiTiet.data, empty: { title: "Không có tệp ở mức này" } }} />
                </Card>
              )}
            </>
          )}
        </div>
      </div>
      <Card padding="none" title={`Danh mục lỗi đang mở (${fmtN(loi.data?.rows.length ?? 0)})`} description="Mọi lần kiểm lỗi (tự động, kế hoạch, tay) đều mở một dòng; khắc phục rồi kiểm lại để đóng.">
        <Table
          columns={colsLoi}
          rows={loi.data?.rows ?? []}
          getRowId={(l) => l.id}
          dense
          maxHeight={400}
          rowActions={canManage ? (l) => <Menu size="sm" items={[{ label: "Khắc phục…", icon: "sua", disabled: busy || l.trang_thai === "da_khac_phuc", onSelect: () => void khacPhuc(l) }, { label: "Kiểm lại", icon: "kiemTra", disabled: busy, onSelect: () => kiemLai(l) }, { label: "Bỏ qua…", icon: "daHuy", danger: true, disabled: busy, onSelect: () => void boQua(l) }]} /> : undefined}
          state={{ loading: loi.loading && !loi.data, error: loi.data ? undefined : loi.error, onRetry: loi.reload, empty: { title: "Không có tệp lỗi đang mở", icon: "thanhCong" } }}
        />
      </Card>
      {dlg && (
        <KeHoachDialog
          row={dlg.row}
          duAn={duAn}
          phong={cat.phong}
          busy={busy}
          onClose={() => setDlg(null)}
          onSave={async (b) => {
            await chay(dlg.row ? "sửa kế hoạch" : "lập kế hoạch", async () => {
              const r = dlg.row ? await khSua(dlg.row.id, b) : await khTao({ ...b, projectId: duAn });
              setSelId(r.id);
            }, dlg.row ? "Đã lưu kế hoạch." : "Đã lập kế hoạch dự thảo.");
            setDlg(null);
          }}
        />
      )}
      {bienBan && k && (
        <LapBienBanDialog
          kinds={["kiem_tra"]}
          projectId={duAn}
          prefill={bienBan}
          onClose={() => setBienBan(null)}
          onCreated={() => {
            void ganBienBanMoiNhat("kiem_tra", duAn, (rid) => khGanBienBan(k.id, rid)).then(() => capNhat());
          }}
        />
      )}
    </div>
  );
}

type Row = Record<string, unknown> & { id: string };

function moTaPhamVi(pv: PhamViKiemTra, phong: Row[]): string {
  const p: string[] = [];
  const ids = pv.phongIds ?? [];
  p.push(ids.length ? `phông ${ids.map((id) => str(phong.find((x) => x.id === id), "ten") || "?").join(", ")}` : "toàn bộ tài liệu của kho");
  if (pv.xacSuat && pv.xacSuat < 100) p.push(`lấy mẫu ${pv.xacSuat}%`);
  if (pv.lauChuaKiem) p.push("chỉ tệp chưa kiểm 3 năm");
  if (pv.goi) p.push("kèm gói AIP");
  return p.join("; ");
}

function KeHoachDialog({ row, duAn, phong, busy, onClose, onSave }: { row: KeHoach | null; duAn: string; phong: Row[]; busy: boolean; onClose: () => void; onSave: (b: KeHoachBody) => Promise<void> }) {
  const namNay = new Date().getFullYear();
  const [ten, setTen] = useState(row?.ten ?? `Kiểm tra tài liệu lưu trữ số năm ${namNay}`);
  const [nam, setNam] = useState(String(row?.nam ?? namNay));
  const [phongIds, setPhongIds] = useState<string[]>(row?.pham_vi.phongIds ?? []);
  const [xacSuat, setXacSuat] = useState(row?.pham_vi.xacSuat ? String(row.pham_vi.xacSuat) : "");
  const [lauChuaKiem, setLau] = useState(row?.pham_vi.lauChuaKiem ?? true);
  const [goi, setGoi] = useState(row?.pham_vi.goi ?? true);
  const [phuongPhap, setPp] = useState(row?.phuong_phap ?? "");
  const [ghiChu, setGhiChu] = useState(row?.ghi_chu ?? "");
  const [xemTruoc, setXemTruoc] = useState<string | null>(null);
  const [dangXem, setDangXem] = useState(false);
  const phamVi = (): PhamViKiemTra => ({ phongIds, xacSuat: xacSuat ? Math.min(100, Math.max(1, Number(xacSuat))) : null, lauChuaKiem, goi });
  const xem = async () => {
    setDangXem(true);
    try {
      const r = await khXemTruoc(duAn, phamVi());
      setXemTruoc(`${fmtN(r.taiLieu)} tài liệu${r.goi ? `, ${fmtN(r.goi)} gói` : ""} trong phạm vi; ước ${fmtN(r.uocLuong)} tệp sẽ kiểm. ${r.moTa}`);
    } catch (e) {
      notifyError(e, "xem trước");
    } finally {
      setDangXem(false);
    }
  };
  return (
    <Dialog
      open
      onOpenChange={(o) => !o && !busy && onClose()}
      title={row ? `Sửa kế hoạch ${row.ma}` : "Lập kế hoạch kiểm tra"}
      size="lg"
      footer={
        <>
          <Button disabled={dangXem} loading={dangXem} onClick={() => void xem()}>Xem trước số tệp</Button>
          <span className="flex-1" />
          <Button onClick={onClose} disabled={busy}>Huỷ</Button>
          <Button variant="primary" icon="luu" loading={busy} disabled={!ten.trim()} onClick={() => void onSave({ ten: ten.trim(), nam: Number(nam), phamVi: phamVi(), phuongPhap: phuongPhap.trim() || null, ghiChu: ghiChu.trim() || null })}>Lưu</Button>
        </>
      }
    >
      <div className="space-y-3">
        <FormGrid cols={3}>
          <Field label="Tên kế hoạch" required className="sm:col-span-2"><Input value={ten} onChange={(e) => setTen(e.target.value)} autoFocus /></Field>
          <Field label="Năm"><Input type="number" value={nam} onChange={(e) => setNam(e.target.value)} disabled={!!row} /></Field>
        </FormGrid>
        <Field label="Phạm vi" hint="Trống = toàn bộ tài liệu của kho">
          <MultiSelect label="Phông" value={phongIds} onChange={setPhongIds} options={phong.map((p) => ({ value: p.id, label: tenPhong(p) }))} />
        </Field>
        <FormGrid cols={3}>
          <Field label="Lấy mẫu xác suất (%)" hint="Trống = kiểm toàn bộ"><Input type="number" min={1} max={100} value={xacSuat} onChange={(e) => setXacSuat(e.target.value)} placeholder="Ví dụ 10" /></Field>
          <Field label=" " inline className="self-end"><Checkbox checked={lauChuaKiem} onChange={setLau} text="Chỉ tệp chưa kiểm trong 3 năm" /></Field>
          <Field label=" " inline className="self-end"><Checkbox checked={goi} onChange={setGoi} text="Kèm gói AIP (sha256 zip)" /></Field>
        </FormGrid>
        <FormGrid cols={2}>
          <Field label="Phương pháp" hint="In vào biên bản Mẫu 01"><Input value={phuongPhap} onChange={(e) => setPp(e.target.value)} placeholder="Băm lại tệp, kiểm PDF/A, đối chiếu số trang" /></Field>
          <Field label="Ghi chú"><Textarea rows={2} value={ghiChu} onChange={(e) => setGhiChu(e.target.value)} /></Field>
        </FormGrid>
        {xemTruoc && <p className="rounded-md bg-surface-active px-3 py-2 text-sm">{xemTruoc}</p>}
      </div>
    </Dialog>
  );
}
