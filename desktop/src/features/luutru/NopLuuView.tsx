// Copyright (c) 2026 DigiText. All rights reserved.
// Nộp hồ sơ vào lưu trữ cơ quan — góc nhìn PHÒNG BAN (Điều 19, 26/11/2026): hồ sơ của phòng còn phải nộp (hạn 60 ngày kể từ
// ngày kết thúc) và các phiếu đã nộp. Cán bộ lưu trữ tiếp nhận ở màn Tiếp nhận nộp lưu.

import { useCallback, useEffect, useMemo, useState } from "react";
import { isTauri } from "@/api/ipc";
import { V41t } from "@/live/useLive";
import { V460 } from "@/state";
import { useProject } from "@/state/project";
import { V3z9 } from "@/domain/permissions";
import { V4t1 } from "@/api/records";
import { nopLuuCanNop, nopLuuGet, nopLuuLap, nopLuuList, nopLuuRut, nopLuuTongQuan, type HoSoCanNop, type PhieuNopLuu } from "@/api/luutru";
import { useClCatalog, Select as ClSelect } from "@/features/chinhly/common";
import { Badge, Button, Card, confirm, EmptyState, KpiRow, KpiTile, notifyError, PageHeader, Seg, Stepper, Table, Tabs, toast, type Column } from "@/ui";
import { BUOC_PHIEU, hanNop, KET_QUA, ngayVn, TT_HO_SO_NOP, TT_PHIEU } from "./luuTruFlow";

const n = (v: number) => v.toLocaleString("vi-VN");
type Tab = "cannop" | "phieu";
type Loc = "" | "den_han" | "qua_han";

export default function NopLuuView() {
  const { session } = V460();
  const { projectId: duAnChon } = useProject();
  const duAn = session?.projectId ?? duAnChon ?? null;
  const perms = session?.permissions ?? [];
  const canNop = V3z9(perms, "luutru.submit");
  const canNhan = V3z9(perms, "luutru.receive");
  const { cat } = useClCatalog();
  const pbCuaToi = session?.phongBanId ?? null;
  const [tab, setTab] = useState<Tab>("cannop");
  const [loc, setLoc] = useState<Loc>("");
  const [pbChon, setPbChon] = useState<string>("");
  const phongBan = pbCuaToi ?? (pbChon || null);
  const [sel, setSel] = useState<Set<string>>(new Set());
  const [busy, setBusy] = useState(false);
  const [selId, setSelId] = useState<string | null>(null);
  const dungDuoc = isTauri() && !!duAn;

  const loader = useCallback(async () => {
    if (!duAn) return null;
    const [tq, cn, ds] = await Promise.all([nopLuuTongQuan(duAn, phongBan), nopLuuCanNop({ projectId: duAn, phongBanId: phongBan, loc, limit: 500 }), nopLuuList({ projectId: duAn, phongBanId: phongBan, limit: 200 })]);
    return { tq, cn, ds };
  }, [duAn, phongBan, loc]);
  const { data, loading, error, reload } = V41t(loader, { topics: ["nopLuu", "chinhLy", "notifications"], enabled: dungDuoc, errorText: () => "Không tải được danh sách nộp lưu." });
  const chiTietLoader = useCallback(() => (selId ? nopLuuGet(selId) : Promise.resolve(null)), [selId]);
  const chiTiet = V41t(chiTietLoader, { topics: ["nopLuu"], enabled: dungDuoc && !!selId, errorText: () => "Không tải được phiếu." });

  const rows = useMemo(() => data?.cn.rows ?? [], [data]);
  const phieu = useMemo(() => data?.ds.rows ?? [], [data]);
  useEffect(() => setSel(new Set()), [loc, phongBan]);

  const nop = async () => {
    if (!duAn || sel.size === 0) return;
    const ok = await confirm({ title: `Nộp ${n(sel.size)} hồ sơ vào lưu trữ cơ quan?`, body: "Hồ sơ sẽ khoá không sửa được cho tới khi lưu trữ cơ quan tiếp nhận hoặc bạn rút phiếu.", confirmLabel: "Nộp lưu" });
    if (!ok) return;
    setBusy(true);
    try {
      const p = await nopLuuLap({ projectId: duAn, phongBanId: phongBan, hoSoIds: [...sel] });
      toast.success(`Đã lập phiếu nộp lưu ${p.ma_phieu} (${n(p.so_ho_so)} hồ sơ).`);
      setSel(new Set());
      setSelId(p.id);
      setTab("phieu");
      reload();
    } catch (e) {
      notifyError(e, "nộp lưu");
    } finally {
      setBusy(false);
    }
  };

  const rut = async (p: PhieuNopLuu) => {
    const ok = await confirm({ title: `Rút phiếu ${p.ma_phieu}?`, body: "Hồ sơ trong phiếu mở khoá để sửa, nộp lại sau bằng phiếu mới.", confirmLabel: "Rút phiếu", danger: true });
    if (!ok) return;
    setBusy(true);
    try {
      await nopLuuRut(p.id);
      toast.success(`Đã rút phiếu ${p.ma_phieu}.`);
      reload();
      chiTiet.reload();
    } catch (e) {
      notifyError(e, "rút phiếu");
    } finally {
      setBusy(false);
    }
  };

  const taiBienBan = async (p: PhieuNopLuu) => {
    if (!p.bien_ban_id) return;
    try {
      const path = await V4t1(p.bien_ban_id, `bien-ban-giao-nhan-${p.ma_phieu.replace(/[\\/]/g, "-")}.pdf`);
      toast.success(`Đã lưu biên bản: ${path}`);
    } catch (e) {
      notifyError(e, "tải biên bản");
    }
  };

  const colsCanNop: Column<HoSoCanNop>[] = [
    { key: "so_tam", header: "Số tạm", width: 90, cell: (r) => r.so_tam },
    { key: "tieu_de", header: "Tiêu đề hồ sơ", className: "whitespace-normal", cell: (r) => r.tieu_de || <span className="text-fg-subtle">Chưa có tiêu đề</span> },
    ...(pbCuaToi ? [] : [{ key: "ten_phong_ban", header: "Phòng ban", width: 160, cell: (r: HoSoCanNop) => r.ten_phong_ban ?? "—" }]),
    { key: "ngay_ket_thuc", header: "Kết thúc", width: 100, cell: (r) => r.ngay_ket_thuc ?? "—" },
    { key: "ten_thbq", header: "THBQ", width: 110, cell: (r) => r.ten_thbq ?? "—" },
    { key: "so_van_ban", header: "VB", align: "right", width: 60, cell: (r) => n(r.so_van_ban ?? 0) },
    {
      key: "han",
      header: "Hạn nộp",
      width: 170,
      cell: (r) => {
        const h = hanNop(r.con_ngay);
        return (
          <span className="flex items-center gap-2">
            <Badge tone={h.tone}>{h.label}</Badge>
            {r.nop_luu_trang_thai === "tra_lai" && <Badge tone="err">Bị trả lại</Badge>}
          </span>
        );
      },
    },
  ];

  const colsPhieu: Column<PhieuNopLuu>[] = [
    { key: "ma_phieu", header: "Số phiếu", width: 150, cell: (p) => <span className="font-medium">{p.ma_phieu}</span> },
    { key: "created_at", header: "Ngày nộp", width: 110, cell: (p) => ngayVn(p.created_at) },
    ...(pbCuaToi ? [] : [{ key: "ten_phong_ban", header: "Phòng ban", cell: (p: PhieuNopLuu) => p.ten_phong_ban ?? "—" }]),
    { key: "so_ho_so", header: "Hồ sơ", align: "right", width: 70, cell: (p) => n(p.so_ho_so) },
    { key: "trang_thai", header: "Trạng thái", width: 140, cell: (p) => <Badge tone={TT_PHIEU[p.trang_thai].tone}>{TT_PHIEU[p.trang_thai].label}</Badge> },
  ];

  const d = chiTiet.data;
  const tq = data?.tq;

  return (
    <section className="view active space-y-4">
      <PageHeader
        title="Nộp lưu"
        description={pbCuaToi ? `Hồ sơ của ${session?.tenPhongBan ?? "phòng ban"} nộp vào lưu trữ cơ quan` : "Hồ sơ của các phòng ban nộp vào lưu trữ cơ quan"}
        infoTitle="Nộp hồ sơ vào lưu trữ hiện hành"
        info={
          <>
            <p>Hồ sơ kết thúc rồi phải nộp vào lưu trữ cơ quan trong 60 ngày (Điều 19 Thông tư 05/2025/TT-BNV). Hạn tính từ ngày kết thúc ghi trên hồ sơ; chỉ ghi năm thì lấy 31/12.</p>
            <p>Chọn hồ sơ rồi bấm Nộp lưu: hồ sơ khoá lại cho tới khi lưu trữ cơ quan tiếp nhận. Rút phiếu khi cần sửa. Sau tiếp nhận, đơn vị nộp còn đọc được hồ sơ trong 360 ngày.</p>
          </>
        }
        actions={tab === "cannop" && canNop && <Button variant="primary" icon="nhapKho" disabled={sel.size === 0 || busy} loading={busy} onClick={() => void nop()}>{sel.size > 0 ? `Nộp lưu ${n(sel.size)} hồ sơ` : "Nộp lưu"}</Button>}
      />
      {!duAn && <EmptyState title="Chưa chọn dự án" description="Chọn dự án (kho lưu trữ) ở thanh trên." />}
      {duAn && (
        <>
          <KpiRow className="grid grid-cols-2 gap-3 md:grid-cols-4">
            <KpiTile icon="canhBao" label="Quá hạn nộp" value={tq?.quaHan ?? 0} tone={tq?.quaHan ? "err" : "ok"} loading={loading && !data} onClick={() => { setTab("cannop"); setLoc("qua_han"); }} />
            <KpiTile icon="nhacHan" label="Đến hạn trong 15 ngày" value={tq?.denHan ?? 0} tone={tq?.denHan ? "warn" : undefined} loading={loading && !data} onClick={() => { setTab("cannop"); setLoc("den_han"); }} />
            <KpiTile icon="hoSo" label="Chưa nộp" value={tq?.chuaNop ?? 0} loading={loading && !data} onClick={() => { setTab("cannop"); setLoc(""); }} />
            <KpiTile icon="dangCho" label="Phiếu chờ tiếp nhận" value={tq?.phieuChoTiepNhan ?? 0} tone={tq?.phieuChoTiepNhan ? "warn" : undefined} loading={loading && !data} onClick={() => setTab("phieu")} />
          </KpiRow>
          <Tabs items={[{ value: "cannop", label: "Hồ sơ cần nộp", count: data?.cn.total }, { value: "phieu", label: "Phiếu đã nộp", count: data?.ds.total }]} value={tab} onChange={setTab} />
          {tab === "cannop" && (
            <Card padding="none" title={<span className="flex items-center gap-3"><Seg options={[{ value: "", label: "Tất cả" }, { value: "den_han", label: "Đến hạn" }, { value: "qua_han", label: "Quá hạn" }]} value={loc} onChange={setLoc} />{!pbCuaToi && cat.phong_ban.length > 0 && <ClSelect size="sm" className="w-56" ariaLabel="Phòng ban" value={pbChon} onChange={setPbChon} rows={cat.phong_ban} empty="Mọi phòng ban" />}</span>}>
              <Table
                columns={colsCanNop}
                rows={rows}
                getRowId={(r) => r.id}
                dense
                maxHeight={520}
                selection={canNop ? { selected: sel, onChange: setSel } : undefined}
                state={{ loading: loading && !data, error: data ? undefined : error, onRetry: reload, empty: { title: loc ? "Không có hồ sơ nào ở mức này" : "Không còn hồ sơ phải nộp", description: "Hồ sơ kết thúc rồi sẽ hiện ở đây kèm hạn nộp." } }}
              />
            </Card>
          )}
          {tab === "phieu" && (
            <div className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_minmax(0,1.2fr)]">
              <Card padding="none">
                <Table columns={colsPhieu} rows={phieu} getRowId={(p) => p.id} dense maxHeight={560} onRowClick={(p) => setSelId(p.id)} rowClassName={(p) => (p.id === selId ? "bg-brand-50" : undefined)} state={{ loading: loading && !data, error: data ? undefined : error, onRetry: reload, empty: { title: "Chưa nộp phiếu nào" } }} />
              </Card>
              <div className="space-y-3">
                {!selId && <EmptyState title="Chọn một phiếu để xem" />}
                {selId && d && (
                  <>
                    <Card
                      title={<span className="flex items-center gap-2">Phiếu {d.phieu.ma_phieu} <Badge tone={TT_PHIEU[d.phieu.trang_thai].tone}>{TT_PHIEU[d.phieu.trang_thai].label}</Badge></span>}
                      description={`${d.phieu.ten_phong_ban ?? ""} · ${d.phieu.nguoi_nop_ten ?? ""} · nộp ${ngayVn(d.phieu.created_at)}${d.phieu.tiep_nhan_luc ? ` · tiếp nhận ${ngayVn(d.phieu.tiep_nhan_luc)} bởi ${d.phieu.tiep_nhan_ten ?? ""}` : ""}`}
                      actions={
                        <>
                          {d.phieu.trang_thai === "cho_tiep_nhan" && (d.phieu.nguoi_nop === session?.userId || canNhan) && <Button size="sm" variant="dangerOutline" disabled={busy} onClick={() => void rut(d.phieu)}>Rút phiếu</Button>}
                          {d.phieu.bien_ban_id && <Button size="sm" icon="bienBan" onClick={() => void taiBienBan(d.phieu)}>Biên bản {d.phieu.so_bien_ban ?? ""}</Button>}
                        </>
                      }
                    >
                      <Stepper steps={BUOC_PHIEU} current={d.phieu.trang_thai === "da_tiep_nhan" ? (d.phieu.bien_ban_id ? 2 : 1) : TT_PHIEU[d.phieu.trang_thai].step} />
                      {d.phieu.ly_do_tra && <p className="mt-2 text-sm text-danger-700">Lý do trả lại: {d.phieu.ly_do_tra}</p>}
                      {d.phieu.ghi_chu && <p className="mt-1 text-sm text-fg-subtle">Ghi chú: {d.phieu.ghi_chu}</p>}
                    </Card>
                    <Card title={`Hồ sơ trong phiếu (${n(d.hoSo.length)})`} padding="none">
                      <Table
                        columns={[
                          { key: "stt", header: "#", align: "right", width: 44, cell: (h) => h.stt ?? "" },
                          { key: "soTam", header: "Số tạm", width: 80, cell: (h) => h.hoSoSo || h.soTam },
                          { key: "tieuDe", header: "Tiêu đề", className: "whitespace-normal", cell: (h) => h.tieuDe },
                          { key: "ketQua", header: "Kết quả", width: 120, cell: (h) => <Badge tone={KET_QUA[h.ketQua]?.tone ?? "gray"}>{KET_QUA[h.ketQua]?.label ?? h.ketQua}</Badge> },
                          { key: "tt", header: "Hồ sơ", width: 130, cell: (h) => (h.nopLuuTrangThai && TT_HO_SO_NOP[h.nopLuuTrangThai] ? <Badge tone={TT_HO_SO_NOP[h.nopLuuTrangThai].tone}>{TT_HO_SO_NOP[h.nopLuuTrangThai].label}</Badge> : <span className="text-fg-subtle">—</span>) },
                          { key: "lyDo", header: "Lý do", className: "whitespace-normal text-sm", cell: (h) => h.lyDo ?? "" },
                        ]}
                        rows={d.hoSo}
                        getRowId={(h) => h.id}
                        dense
                        maxHeight={400}
                      />
                    </Card>
                  </>
                )}
                {selId && !d && chiTiet.loading && <Card>Đang tải phiếu…</Card>}
              </div>
            </div>
          )}
        </>
      )}
    </section>
  );
}
