// Copyright (c) 2026 DigiText. All rights reserved.
// Tiếp nhận hồ sơ nộp lưu — góc nhìn CÁN BỘ LƯU TRỮ CƠ QUAN (Điều 19, 26/11/2026): phiếu chờ, chấm từng hồ sơ (nhận / trả lại),
// hoàn tất tiếp nhận (tự lập biên bản giao nhận), trả cả phiếu, mở khoá ngoại lệ; sổ nộp lưu theo năm (Excel, bản in).

import { useCallback, useMemo, useState } from "react";
import { isTauri } from "@/api/ipc";
import { V41t } from "@/live/useLive";
import { V460 } from "@/state";
import { useProject } from "@/state/project";
import { V3z9 } from "@/domain/permissions";
import { V4t1 } from "@/api/records";
import { nopLuuBienBan, nopLuuCham, nopLuuGet, nopLuuList, nopLuuMoKhoa, nopLuuSo, nopLuuSoBienBan, nopLuuTiepNhan, nopLuuTraLai, type HoSoTrongPhieu, type PhieuNopLuu, type TrangThaiPhieu } from "@/api/luutru";
import { useClCatalog, Select as ClSelect } from "@/features/chinhly/common";
import { writeXlsx } from "@/features/chinhly/excel";
import { Badge, Button, Card, confirm, EmptyState, IconButton, Menu, notifyError, PageHeader, prompt, Seg, Select as UiSelect, Stepper, Table, Tabs, toast, type Column } from "@/ui";
import { BUOC_PHIEU, KET_QUA, ngayVn, TT_PHIEU } from "./luuTruFlow";

const n = (v: number) => v.toLocaleString("vi-VN");
type Tab = "phieu" | "so";

export default function TiepNhanView() {
  const { session } = V460();
  const { projectId: duAnChon } = useProject();
  const duAn = session?.projectId ?? duAnChon ?? null;
  const perms = session?.permissions ?? [];
  const canNhan = V3z9(perms, "luutru.receive");
  const canMoKhoa = V3z9(perms, "luutru.unlock");
  const { cat } = useClCatalog();
  const [tab, setTab] = useState<Tab>("phieu");
  const [tt, setTt] = useState<TrangThaiPhieu | "">("cho_tiep_nhan");
  const [selId, setSelId] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const namNay = new Date().getFullYear();
  const [nam, setNam] = useState(String(namNay));
  const [pbSo, setPbSo] = useState("");
  const dungDuoc = isTauri() && !!duAn;

  const loader = useCallback(() => (duAn ? nopLuuList({ projectId: duAn, trangThai: tt, limit: 300 }) : Promise.resolve(null)), [duAn, tt]);
  const { data, loading, error, reload } = V41t(loader, { topics: ["nopLuu", "notifications"], enabled: dungDuoc, errorText: () => "Không tải được phiếu nộp lưu." });
  const chiTietLoader = useCallback(() => (selId ? nopLuuGet(selId) : Promise.resolve(null)), [selId]);
  const chiTiet = V41t(chiTietLoader, { topics: ["nopLuu"], enabled: dungDuoc && !!selId, errorText: () => "Không tải được phiếu." });
  const soLoader = useCallback(() => (duAn && tab === "so" ? nopLuuSo(duAn, Number(nam), pbSo || null) : Promise.resolve(null)), [duAn, tab, nam, pbSo]);
  const so = V41t(soLoader, { topics: ["nopLuu"], enabled: dungDuoc && tab === "so", errorText: () => "Không tải được sổ nộp lưu." });

  const phieu = useMemo(() => data?.rows ?? [], [data]);
  const d = chiTiet.data;
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

  const cham = async (h: HoSoTrongPhieu, ketQua: "nhan" | "tra_lai" | "cho") => {
    if (!d) return;
    let lyDo: string | null = null;
    if (ketQua === "tra_lai") {
      lyDo = await prompt({ title: `Trả lại hồ sơ ${h.hoSoSo || h.soTam}`, label: "Lý do trả lại", type: "textarea", required: true, placeholder: "Ví dụ: thiếu chứng từ kết thúc, sai thời hạn bảo quản…" });
      if (!lyDo) return;
    }
    await chay("chấm hồ sơ", () => nopLuuCham(d.phieu.id, h.hoSoId, ketQua, lyDo));
  };
  const tiepNhan = async () => {
    if (!d) return;
    const soTra = d.hoSo.filter((h) => h.ketQua === "tra_lai").length;
    const ok = await confirm({ title: `Hoàn tất tiếp nhận phiếu ${d.phieu.ma_phieu}?`, body: `${n(d.hoSo.length - soTra)} hồ sơ sẽ nhận vào lưu trữ cơ quan và khoá; ${n(soTra)} hồ sơ trả lại. Biên bản giao nhận được lập tự động.`, confirmLabel: "Tiếp nhận" });
    if (!ok) return;
    await chay("tiếp nhận", () => nopLuuTiepNhan(d.phieu.id), `Đã tiếp nhận phiếu ${d.phieu.ma_phieu}.`);
  };
  const traCaPhieu = async () => {
    if (!d) return;
    const lyDo = await prompt({ title: `Trả lại cả phiếu ${d.phieu.ma_phieu}`, label: "Lý do", type: "textarea", required: true });
    if (!lyDo) return;
    await chay("trả lại phiếu", () => nopLuuTraLai(d.phieu.id, lyDo), `Đã trả lại phiếu ${d.phieu.ma_phieu}.`);
  };
  const moKhoa = async (h: HoSoTrongPhieu) => {
    if (!d) return;
    const lyDo = await prompt({ title: `Mở khoá hồ sơ ${h.hoSoSo || h.soTam}`, body: "Hồ sơ trở về trạng thái chưa nộp để đơn vị sửa rồi nộp lại. Việc này được ghi nhật ký.", label: "Lý do mở khoá", type: "textarea", required: true });
    if (!lyDo) return;
    await chay("mở khoá", () => nopLuuMoKhoa(d.phieu.id, h.hoSoId, lyDo), "Đã mở khoá hồ sơ.");
  };
  const lapLaiBienBan = async () => {
    if (!d) return;
    await chay("lập biên bản", async () => {
      const r = await nopLuuBienBan(d.phieu.id);
      toast.success(`Đã lập biên bản ${r.soBienBan}.`);
    });
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
  const xuatSo = async () => {
    const rows = so.data?.rows ?? [];
    const aoa: (string | number | null)[][] = [
      [`SỔ NỘP LƯU HỒ SƠ VÀO LƯU TRỮ CƠ QUAN NĂM ${nam}`],
      ["STT", "Ngày", "Đơn vị nộp", "Số phiếu", "HS nộp", "Nhận", "Trả lại", "Người giao", "Người nhận", "Biên bản", "Trạng thái"],
      ...rows.map((p, i) => [i + 1, ngayVn(p.tiep_nhan_luc ?? p.created_at), p.ten_phong_ban ?? "", p.ma_phieu, p.so_ho_so, p.so_nhan, p.so_tra_lai, p.nguoi_nop_ten ?? "", p.tiep_nhan_ten ?? "", p.so_bien_ban ?? "", TT_PHIEU[p.trang_thai].label]),
    ];
    try {
      await writeXlsx([{ name: "Sổ nộp lưu", aoa, cols: [{ wch: 5 }, { wch: 12 }, { wch: 28 }, { wch: 18 }, { wch: 8 }, { wch: 8 }, { wch: 8 }, { wch: 22 }, { wch: 22 }, { wch: 18 }, { wch: 14 }], merges: ["A1:K1"] }], `so-nop-luu-${nam}.xlsx`);
      toast.success("Đã xuất sổ nộp lưu.");
    } catch (e) {
      notifyError(e, "xuất Excel");
    }
  };
  const inSo = async () => {
    if (!duAn) return;
    await chay("in sổ", async () => {
      const r = await nopLuuSoBienBan(duAn, Number(nam), pbSo || null);
      const path = await V4t1(r.id, `so-nop-luu-${nam}.pdf`);
      toast.success(`Đã lập ${r.soBienBan} và lưu: ${path}`);
    });
  };

  const colsPhieu: Column<PhieuNopLuu>[] = [
    { key: "ma_phieu", header: "Số phiếu", width: 150, cell: (p) => <span className="font-medium">{p.ma_phieu}</span> },
    { key: "ten_phong_ban", header: "Đơn vị nộp", cell: (p) => p.ten_phong_ban ?? "—" },
    { key: "created_at", header: "Ngày nộp", width: 105, cell: (p) => ngayVn(p.created_at) },
    { key: "so_ho_so", header: "HS", align: "right", width: 56, cell: (p) => n(p.so_ho_so) },
    { key: "trang_thai", header: "Trạng thái", width: 130, cell: (p) => <Badge tone={TT_PHIEU[p.trang_thai].tone}>{TT_PHIEU[p.trang_thai].label}</Badge> },
  ];
  const dangCho = d?.phieu.trang_thai === "cho_tiep_nhan";
  const colsHoSo: Column<HoSoTrongPhieu>[] = [
    { key: "stt", header: "#", align: "right", width: 44, cell: (h) => h.stt ?? "" },
    { key: "soTam", header: "Số tạm", width: 80, cell: (h) => h.hoSoSo || h.soTam },
    { key: "tieuDe", header: "Tiêu đề hồ sơ", className: "whitespace-normal", cell: (h) => h.tieuDe },
    { key: "tg", header: "Thời gian", width: 150, cell: (h) => [h.ngayBatDau, h.ngayKetThuc].filter(Boolean).join(" – ") || "—" },
    { key: "soTo", header: "Số tờ", align: "right", width: 60, cell: (h) => h.soTo ?? "" },
    { key: "tenThbq", header: "THBQ", width: 100, cell: (h) => h.tenThbq ?? "" },
    { key: "soVanBan", header: "VB", align: "right", width: 50, cell: (h) => n(h.soVanBan) },
    { key: "ketQua", header: "Kết quả", width: 110, cell: (h) => <Badge tone={KET_QUA[h.ketQua]?.tone ?? "gray"}>{KET_QUA[h.ketQua]?.label ?? h.ketQua}</Badge> },
    { key: "lyDo", header: "Lý do", className: "whitespace-normal text-sm", cell: (h) => h.lyDo ?? "" },
  ];
  const colsSo: Column<PhieuNopLuu>[] = [
    { key: "ngay", header: "Ngày", width: 105, cell: (p) => ngayVn(p.tiep_nhan_luc ?? p.created_at) },
    { key: "ten_phong_ban", header: "Đơn vị nộp", cell: (p) => p.ten_phong_ban ?? "—" },
    { key: "ma_phieu", header: "Số phiếu", width: 150, cell: (p) => p.ma_phieu },
    { key: "so_ho_so", header: "HS nộp", align: "right", width: 70, cell: (p) => n(p.so_ho_so) },
    { key: "so_nhan", header: "Nhận", align: "right", width: 60, cell: (p) => n(p.so_nhan) },
    { key: "so_tra_lai", header: "Trả lại", align: "right", width: 70, cell: (p) => n(p.so_tra_lai) },
    { key: "nguoi_nop_ten", header: "Người giao", width: 150, cell: (p) => p.nguoi_nop_ten ?? "" },
    { key: "tiep_nhan_ten", header: "Người nhận", width: 150, cell: (p) => p.tiep_nhan_ten ?? "" },
    { key: "so_bien_ban", header: "Biên bản", width: 140, cell: (p) => (p.bien_ban_id ? <Button size="sm" variant="ghost" icon="bienBan" onClick={() => void taiBienBan(p)}>{p.so_bien_ban ?? "Tải"}</Button> : "—") },
  ];

  return (
    <section className="view active space-y-4">
      <PageHeader
        title="Tiếp nhận nộp lưu"
        description="Phiếu nộp lưu của các phòng ban và sổ nộp lưu"
        infoTitle="Tiếp nhận hồ sơ vào lưu trữ cơ quan"
        info={
          <>
            <p>Chấm từng hồ sơ (nhận hoặc trả lại có lý do) rồi Hoàn tất tiếp nhận: hồ sơ nhận được khoá, biên bản giao nhận lập tự động (Điều 19 Thông tư 05/2025/TT-BNV). Hồ sơ chưa chấm coi là nhận.</p>
            <p>Mở khoá ngoại lệ chỉ dành cho người có quyền và được ghi nhật ký. Sổ nộp lưu xuất Excel hoặc in thành biên bản có số.</p>
          </>
        }
      />
      {!duAn && <EmptyState title="Chưa chọn dự án" description="Chọn dự án (kho lưu trữ) ở thanh trên." />}
      {duAn && !canNhan && <EmptyState title="Bạn không có quyền tiếp nhận nộp lưu" description="Cần quyền «Tiếp nhận / trả lại hồ sơ nộp lưu» (luutru.receive)." />}
      {duAn && canNhan && (
        <>
          <Tabs items={[{ value: "phieu", label: "Phiếu nộp lưu", count: data?.total }, { value: "so", label: "Sổ nộp lưu" }]} value={tab} onChange={setTab} />
          {tab === "phieu" && (
            <div className="grid gap-4 lg:grid-cols-[minmax(0,1fr)_minmax(0,1.3fr)]">
              <Card padding="none" title={<Seg options={[{ value: "cho_tiep_nhan", label: "Chờ" }, { value: "da_tiep_nhan", label: "Đã nhận" }, { value: "tra_lai", label: "Trả lại" }, { value: "", label: "Tất cả" }]} value={tt} onChange={setTt} />}>
                <Table columns={colsPhieu} rows={phieu} getRowId={(p) => p.id} dense maxHeight={600} onRowClick={(p) => setSelId(p.id)} rowClassName={(p) => (p.id === selId ? "bg-brand-50" : undefined)} state={{ loading: loading && !data, error: data ? undefined : error, onRetry: reload, empty: { title: tt === "cho_tiep_nhan" ? "Không có phiếu nào chờ tiếp nhận" : "Không có phiếu" } }} />
              </Card>
              <div className="space-y-3">
                {!selId && <EmptyState title="Chọn một phiếu để chấm" />}
                {selId && d && (
                  <>
                    <Card
                      title={<span className="flex items-center gap-2">Phiếu {d.phieu.ma_phieu} <Badge tone={TT_PHIEU[d.phieu.trang_thai].tone}>{TT_PHIEU[d.phieu.trang_thai].label}</Badge></span>}
                      description={`${d.phieu.ten_phong_ban ?? ""} · ${d.phieu.nguoi_nop_ten ?? ""} · nộp ${ngayVn(d.phieu.created_at)} · ${n(d.phieu.so_ho_so)} hồ sơ, ${n(d.phieu.so_van_ban ?? 0)} văn bản`}
                      actions={
                        <>
                          {dangCho && <Button size="sm" variant="dangerOutline" disabled={busy} onClick={() => void traCaPhieu()}>Trả cả phiếu</Button>}
                          {dangCho && <Button size="sm" variant="primary" icon="duyet" loading={busy} onClick={() => void tiepNhan()}>Hoàn tất tiếp nhận</Button>}
                          {d.phieu.trang_thai === "da_tiep_nhan" && !d.phieu.bien_ban_id && <Button size="sm" icon="bienBan" disabled={busy} onClick={() => void lapLaiBienBan()}>Lập biên bản</Button>}
                          {d.phieu.bien_ban_id && <Button size="sm" icon="taiVe" onClick={() => void taiBienBan(d.phieu)}>Biên bản {d.phieu.so_bien_ban ?? ""}</Button>}
                        </>
                      }
                    >
                      <Stepper steps={BUOC_PHIEU} current={d.phieu.trang_thai === "da_tiep_nhan" ? (d.phieu.bien_ban_id ? 2 : 1) : TT_PHIEU[d.phieu.trang_thai].step} />
                      {d.phieu.ly_do_tra && <p className="mt-2 text-sm text-danger-700">Lý do trả lại: {d.phieu.ly_do_tra}</p>}
                    </Card>
                    <Card title={`Hồ sơ (${n(d.hoSo.length)})`} padding="none">
                      <Table
                        columns={colsHoSo}
                        rows={d.hoSo}
                        getRowId={(h) => h.id}
                        dense
                        maxHeight={440}
                        rowActions={(h) =>
                          dangCho ? (
                            <>
                              <IconButton label="Nhận" icon="duyet" size="sm" disabled={busy || h.ketQua === "nhan"} onClick={() => void cham(h, "nhan")} />
                              <IconButton label="Trả lại" icon="tuChoi" size="sm" disabled={busy} onClick={() => void cham(h, "tra_lai")} />
                              {h.ketQua !== "cho" && <IconButton label="Bỏ chấm" icon="hoanTac" size="sm" disabled={busy} onClick={() => void cham(h, "cho")} />}
                            </>
                          ) : canMoKhoa && h.nopLuuTrangThai === "da_nop" ? (
                            <Menu size="sm" items={[{ label: "Mở khoá hồ sơ…", icon: "khoa", danger: true, onSelect: () => void moKhoa(h) }]} />
                          ) : null
                        }
                      />
                    </Card>
                  </>
                )}
              </div>
            </div>
          )}
          {tab === "so" && (
            <Card
              padding="none"
              title={
                <span className="flex items-center gap-2">
                  <UiSelect size="sm" aria-label="Năm" value={nam} onChange={setNam} options={[namNay - 2, namNay - 1, namNay, namNay + 1].map((y) => ({ value: String(y), label: `Năm ${y}` }))} />
                  {cat.phong_ban.length > 0 && <ClSelect size="sm" className="w-56" ariaLabel="Phòng ban" value={pbSo} onChange={setPbSo} rows={cat.phong_ban} empty="Mọi phòng ban" />}
                </span>
              }
              actions={
                <>
                  <Button size="sm" icon="tepExcel" disabled={!so.data?.rows.length} onClick={() => void xuatSo()}>Excel</Button>
                  <Button size="sm" icon="inRa" disabled={busy || !so.data?.rows.length} onClick={() => void inSo()}>In sổ</Button>
                </>
              }
            >
              <Table columns={colsSo} rows={so.data?.rows ?? []} getRowId={(p) => p.id} dense maxHeight={560} state={{ loading: so.loading && !so.data, error: so.data ? undefined : so.error, onRetry: so.reload, empty: { title: `Chưa có phiếu nào được tiếp nhận trong năm ${nam}` } }} />
            </Card>
          )}
        </>
      )}
    </section>
  );
}
