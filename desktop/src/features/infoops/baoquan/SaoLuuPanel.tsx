// Copyright (c) 2026 DigiText. All rights reserved.
// Sao lưu (Điều 30, 0032): lịch ngày / tháng / 3 năm với tình trạng đúng hạn – trễ, bản cũ cần xoá, sổ sao lưu đủ cột Mẫu 02/03,
// ghi nhận tay, biên bản sao lưu và nhật ký điền sẵn từ sổ.

import { useCallback, useMemo, useState } from "react";
import { isTauri } from "@/api/ipc";
import { V41t } from "@/live/useLive";
import { V460 } from "@/state";
import { V3z9 } from "@/domain/permissions";
import { lichSua, lichTao, ptList, saoLuuDaXoa, saoLuuGhi, saoLuuRuns, type LichBody, type PhuongTien, type SaoLuuRun, type TinhTrangLich } from "@/api/baoquan";
import { SoBienBan, type NguonDien, type NoiDungDien } from "@/features/records/RecordsView";
import { Badge, Button, Card, Checkbox, confirm, Dialog, Field, FormGrid, Input, notifyError, Select, Table, Textarea, Tip, toast, type Column } from "@/ui";
import { fmtMB, fmtN, LOAI_SAO_LUU, ngayGio, TT_LICH } from "./baoQuanFlow";
import { lichList } from "@/api/baoquan";

export default function SaoLuuPanel() {
  const { session } = V460();
  const canManage = V3z9(session?.permissions, "bk.manage");
  const [busy, setBusy] = useState(false);
  const [lichDlg, setLichDlg] = useState<{ row: TinhTrangLich | null } | null>(null);
  const [ghiDlg, setGhiDlg] = useState(false);
  const loader = useCallback(async () => {
    const [lich, runs, pt] = await Promise.all([lichList(), saoLuuRuns(null, 300), ptList()]);
    return { lich: lich.rows, runs: runs.rows, pt: pt.rows };
  }, []);
  const { data, loading, error, reload } = V41t(loader, { topics: ["ops"], enabled: isTauri(), errorText: () => "Không tải được sổ sao lưu." });
  const lich = useMemo(() => data?.lich ?? [], [data]);
  const runs = useMemo(() => data?.runs ?? [], [data]);
  const banCu = useMemo(() => lich.flatMap((l) => l.banCuCanXoa.map((r) => ({ ...r, lich_ten: l.ten }))), [lich]);
  const chay = async (viec: string, f: () => Promise<unknown>, xong?: string) => {
    setBusy(true);
    try {
      await f();
      if (xong) toast.success(xong);
      reload();
    } catch (e) {
      notifyError(e, viec);
    } finally {
      setBusy(false);
    }
  };
  const daXoa = async (r: SaoLuuRun) => {
    const ok = await confirm({ title: "Đã xoá bản sao lưu này khỏi phương tiện?", body: `${r.duong_dan ?? r.nguon} · ${ngayGio(r.created_at)}. Chỉ ghi nhận; tệp thật do người vận hành xoá.`, confirmLabel: "Đã xoá" });
    if (!ok) return;
    await chay("ghi nhận xoá", () => saoLuuDaXoa(r.id));
  };

  // Điền sẵn biên bản sao lưu / nhật ký từ sổ của một lịch.
  const nguonDien: NguonDien = useMemo(
    () => ({
      label: "Điền từ lịch sao lưu",
      options: lich.map((l) => ({ value: l.id, label: l.ten })),
      dien: async (kind, lichId): Promise<NoiDungDien> => {
        const rs = runs.filter((r) => r.lich_id === lichId);
        const l = lich.find((x) => x.id === lichId);
        if (kind === "nhat_ky_sao_luu") {
          return {
            fields: { ky: `Đến ngày ${new Date().toLocaleDateString("vi-VN")} — ${l?.ten ?? ""}` },
            lists: {
              entries: rs.slice(0, 60).map((r) => ({
                thoiGian: ngayGio(r.created_at),
                loai: LOAI_SAO_LUU[r.loai] ?? r.loai,
                nguonDuLieu: r.nguon,
                phuongTien: r.phuong_tien_ma ?? r.phuong_tien ?? "",
                dungLuong: r.dung_luong_bytes ? fmtMB(r.dung_luong_bytes) : "",
                soHieuPhuongTien: r.phuong_tien_ma ?? "",
                noiBaoQuan: "",
                ketQua: r.ket_qua === "dat" ? "Đạt" : "Không đạt",
                nguoiThucHien: r.nguoi_thuc_hien ?? r.reported_ten ?? "",
              })),
            },
          };
        }
        const r = rs.find((x) => x.ket_qua === "dat") ?? rs[0];
        if (!r) return { fields: {} };
        return {
          fields: {
            loaiSaoLuu: LOAI_SAO_LUU[r.loai] ?? r.loai,
            nguonDuLieu: r.nguon,
            phuongTien: r.phuong_tien_ma ?? r.phuong_tien ?? "",
            thoiGian: r.bat_dau_at && r.ket_thuc_at ? `${ngayGio(r.bat_dau_at)} – ${ngayGio(r.ket_thuc_at)}` : ngayGio(r.created_at),
            dungLuong: r.dung_luong_bytes ? fmtMB(r.dung_luong_bytes) : "",
            maBam: r.ma_bam ?? "",
            ketQuaKiemTra: r.ket_qua === "dat" ? `Đạt${r.chi_tiet ? ` — ${r.chi_tiet}` : ""}` : `Không đạt${r.chi_tiet ? ` — ${r.chi_tiet}` : ""}`,
            hanLuu: l ? `Giữ ${l.giuSoBan} bản gần nhất theo lịch` : "",
            nguoiThucHien: r.nguoi_thuc_hien ?? r.reported_ten ?? "",
            ngay: new Date().toLocaleDateString("vi-VN"),
          },
        };
      },
    }),
    [lich, runs],
  );

  const colsRun: Column<SaoLuuRun>[] = [
    { key: "created_at", header: "Thời điểm", width: 140, sortable: true, cell: (r) => ngayGio(r.created_at) },
    { key: "lich_ten", header: "Lịch / loại", width: 180, cell: (r) => r.lich_ten ?? LOAI_SAO_LUU[r.loai] ?? r.loai },
    { key: "nguon", header: "Nguồn", className: "whitespace-normal", cell: (r) => r.nguon },
    { key: "duong_dan", header: "Đường dẫn", className: "whitespace-normal break-all text-xs text-fg-subtle", cell: (r) => r.duong_dan ?? r.phuong_tien ?? "—" },
    { key: "dung_luong_bytes", header: "Dung lượng", align: "right", width: 100, cell: (r) => fmtMB(r.dung_luong_bytes) },
    { key: "phuong_tien_ma", header: "Phương tiện", width: 110, cell: (r) => r.phuong_tien_ma ?? "—" },
    {
      key: "ket_qua",
      header: "Kết quả",
      width: 110,
      cell: (r) => (
        <Tip text={r.chi_tiet ? r.chi_tiet.slice(0, 120) : "Không có chi tiết"}>
          <span><Badge tone={r.ket_qua === "dat" ? "ok" : "err"}>{r.ket_qua === "dat" ? "Đạt" : "Không đạt"}</Badge></span>
        </Tip>
      ),
    },
    { key: "nguoi_thuc_hien", header: "Người", width: 130, cell: (r) => r.nguoi_thuc_hien ?? r.reported_ten ?? "script" },
    { key: "da_xoa_at", header: "Bản sao", width: 110, cell: (r) => (r.da_xoa_at ? <Badge tone="gray">Đã xoá</Badge> : r.ket_qua === "dat" ? <Badge tone="ok">Còn giữ</Badge> : "—") },
  ];
  const colsBanCu: Column<SaoLuuRun & { lich_ten: string }>[] = [
    { key: "lich_ten", header: "Lịch", width: 200, cell: (r) => r.lich_ten },
    { key: "created_at", header: "Thời điểm", width: 140, cell: (r) => ngayGio(r.created_at) },
    { key: "duong_dan", header: "Đường dẫn / nguồn", className: "whitespace-normal break-all text-sm", cell: (r) => r.duong_dan ?? r.nguon },
    { key: "dung_luong_bytes", header: "Dung lượng", align: "right", width: 100, cell: (r) => fmtMB(r.dung_luong_bytes) },
  ];

  return (
    <div className="space-y-4">
      <div className="grid gap-3 md:grid-cols-3">
        {lich.map((l) => (
          <Card key={l.id} title={<span className="flex items-center gap-2">{l.ten} <Badge tone={TT_LICH[l.tinhTrang].tone}>{l.tinhTrang === "tre" ? `Trễ ${fmtN(l.treGio)} giờ` : TT_LICH[l.tinhTrang].label}</Badge></span>} description={LOAI_SAO_LUU[l.loai]} actions={canManage && <Button size="sm" variant="ghost" icon="sua" onClick={() => setLichDlg({ row: l })}>Sửa</Button>}>
            <dl className="grid grid-cols-2 gap-x-3 gap-y-1 text-sm">
              <dt className="text-fg-subtle">Chu kỳ</dt><dd>{l.chuKyGio % 24 === 0 ? `${l.chuKyGio / 24} ngày` : `${l.chuKyGio} giờ`} (+{l.doTreChoPhepGio} giờ trễ)</dd>
              <dt className="text-fg-subtle">Bản đạt gần nhất</dt><dd>{ngayGio(l.lanDat)}</dd>
              <dt className="text-fg-subtle">Hạn kế tiếp</dt><dd>{ngayGio(l.hanKeTiep)}</dd>
              <dt className="text-fg-subtle">Bản đang giữ</dt><dd className={l.soBanCanXoa ? "text-warning-700" : undefined}>{fmtN(l.soBan)} / giữ {l.giuSoBan}{l.soBanCanXoa ? ` · cần xoá ${l.soBanCanXoa}` : ""}</dd>
              <dt className="text-fg-subtle">Nguồn</dt><dd className="truncate">{l.nguon ?? "—"}</dd>
              <dt className="text-fg-subtle">Phương tiện</dt><dd>{l.phuongTienMa ?? "—"}</dd>
            </dl>
          </Card>
        ))}
        {canManage && lich.length < 6 && (
          <Card padding="sm" className="flex items-center justify-center">
            <Button variant="ghost" icon="them" onClick={() => setLichDlg({ row: null })}>Thêm lịch</Button>
          </Card>
        )}
      </div>
      {banCu.length > 0 && (
        <Card padding="none" tone="warning" title={`Bản cũ cần xoá (${fmtN(banCu.length)})`} description="Vượt số bản phải giữ của lịch. Xoá tệp trên phương tiện rồi bấm Đã xoá để ghi nhận (Điều 30).">
          <Table columns={colsBanCu} rows={banCu} getRowId={(r) => r.id} dense maxHeight={240} rowActions={canManage ? (r) => <Button size="sm" variant="ghost" icon="xoa" disabled={busy} onClick={() => void daXoa(r)}>Đã xoá</Button> : undefined} />
        </Card>
      )}
      <Card padding="none" title="Sổ sao lưu" description="Script sao lưu đêm tự ghi; ghi nhận tay cho sao lưu 3 năm ra phương tiện riêng." actions={canManage && <Button size="sm" icon="them" onClick={() => setGhiDlg(true)}>Ghi nhận sao lưu</Button>}>
        <Table columns={colsRun} rows={runs} getRowId={(r) => r.id} dense maxHeight={420} state={{ loading: loading && !data, error: data ? undefined : error, onRetry: reload, empty: { title: "Chưa có lần sao lưu nào", description: "Lịch gia tăng hằng ngày quá hạn sẽ cảnh báo quản trị." } }} />
      </Card>
      <SoBienBan kinds={["sao_luu", "nhat_ky_sao_luu"]} title="Biên bản sao lưu và nhật ký (Mẫu 02, 03)" description="Điền sẵn từ sổ sao lưu của một lịch." nguonDien={nguonDien} nutLap="Lập biên bản…" embedded />
      {lichDlg && (
        <LichDialog
          row={lichDlg.row}
          pt={data?.pt ?? []}
          busy={busy}
          onClose={() => setLichDlg(null)}
          onSave={async (b) => {
            await chay(lichDlg.row ? "sửa lịch" : "thêm lịch", () => (lichDlg.row ? lichSua(lichDlg.row.id, b) : lichTao(b)), "Đã lưu lịch sao lưu.");
            setLichDlg(null);
          }}
        />
      )}
      {ghiDlg && (
        <GhiNhanDialog
          lich={lich}
          pt={data?.pt ?? []}
          busy={busy}
          onClose={() => setGhiDlg(false)}
          onSave={async (b) => {
            await chay("ghi nhận sao lưu", () => saoLuuGhi(b), "Đã ghi nhận lần sao lưu.");
            setGhiDlg(false);
          }}
        />
      )}
    </div>
  );
}

function LichDialog({ row, pt, busy, onClose, onSave }: { row: TinhTrangLich | null; pt: PhuongTien[]; busy: boolean; onClose: () => void; onSave: (b: LichBody) => Promise<void> }) {
  const [loai, setLoai] = useState<LichBody["loai"]>(row?.loai ?? "gia_tang");
  const [ten, setTen] = useState(row?.ten ?? "");
  const [chuKy, setChuKy] = useState(String(row?.chuKyGio ?? 24));
  const [doTre, setDoTre] = useState(String(row?.doTreChoPhepGio ?? 2));
  const [giu, setGiu] = useState(String(row?.giuSoBan ?? 14));
  const [bat, setBat] = useState(row?.bat ?? true);
  const [nguon, setNguon] = useState(row?.nguon ?? "");
  const [ptId, setPtId] = useState(row?.phuongTienId ?? "");
  const [ghiChu, setGhiChu] = useState(row?.ghiChu ?? "");
  const ok = ten.trim() && Number(chuKy) > 0 && Number(giu) >= 1;
  return (
    <Dialog open onOpenChange={(o) => !o && !busy && onClose()} title={row ? `Sửa lịch «${row.ten}»` : "Thêm lịch sao lưu"} size="md" footer={<><Button onClick={onClose} disabled={busy}>Huỷ</Button><Button variant="primary" icon="luu" loading={busy} disabled={!ok} onClick={() => void onSave({ loai, ten: ten.trim(), chuKyGio: Number(chuKy), doTreChoPhepGio: Number(doTre) || 0, giuSoBan: Number(giu), bat, nguon: nguon.trim() || null, phuongTienId: ptId || null, ghiChu: ghiChu.trim() || null })}>Lưu</Button></>}>
      <div className="space-y-3">
        {!row && (
          <Field label="Loại (Điều 30.1)"><Select value={loai ?? "gia_tang"} onChange={(v) => setLoai(v as LichBody["loai"])} options={Object.entries(LOAI_SAO_LUU).map(([value, label]) => ({ value, label }))} /></Field>
        )}
        <Field label="Tên lịch" required><Input value={ten} onChange={(e) => setTen(e.target.value)} autoFocus /></Field>
        <FormGrid cols={3}>
          <Field label="Chu kỳ (giờ)" hint="24 = hằng ngày, 720 = 30 ngày"><Input type="number" min={1} value={chuKy} onChange={(e) => setChuKy(e.target.value)} /></Field>
          <Field label="Trễ cho phép (giờ)"><Input type="number" min={0} value={doTre} onChange={(e) => setDoTre(e.target.value)} /></Field>
          <Field label="Số bản giữ"><Input type="number" min={1} value={giu} onChange={(e) => setGiu(e.target.value)} /></Field>
        </FormGrid>
        <Field label="Nguồn dữ liệu" hint="CSDL, kho tài liệu, mã nguồn phần mềm (Điều 30.3.c)"><Input value={nguon} onChange={(e) => setNguon(e.target.value)} /></Field>
        <Field label="Phương tiện lưu trữ"><Select value={ptId} onChange={setPtId} placeholder="— Chưa gắn —" options={pt.filter((p) => p.trang_thai === "dang_dung" || p.id === ptId).map((p) => ({ value: p.id, label: `${p.ma} · ${p.loai}` }))} /></Field>
        <Field label="Ghi chú"><Textarea rows={2} value={ghiChu} onChange={(e) => setGhiChu(e.target.value)} /></Field>
        <Checkbox checked={bat} onChange={setBat} text="Đang áp dụng (tắt thì không cảnh báo trễ)" />
      </div>
    </Dialog>
  );
}

function GhiNhanDialog({ lich, pt, busy, onClose, onSave }: { lich: TinhTrangLich[]; pt: PhuongTien[]; busy: boolean; onClose: () => void; onSave: (b: Parameters<typeof saoLuuGhi>[0]) => Promise<void> }) {
  const [lichId, setLichId] = useState(lich.find((l) => l.loai === "day_du_3_nam")?.id ?? lich[0]?.id ?? "");
  const l = lich.find((x) => x.id === lichId);
  const [nguon, setNguon] = useState(l?.nguon ?? "CSDL + kho tài liệu + mã nguồn phần mềm");
  const [ptId, setPtId] = useState(l?.phuongTienId ?? "");
  const [duongDan, setDuongDan] = useState("");
  const [mb, setMb] = useState("");
  const [maBam, setMaBam] = useState("");
  const [ketQua, setKetQua] = useState<"dat" | "khong_dat">("dat");
  const [chiTiet, setChiTiet] = useState("");
  const chonLich = (id: string) => {
    setLichId(id);
    const x = lich.find((y) => y.id === id);
    if (x?.nguon) setNguon(x.nguon);
    if (x?.phuongTienId) setPtId(x.phuongTienId);
  };
  return (
    <Dialog open onOpenChange={(o) => !o && !busy && onClose()} title="Ghi nhận lần sao lưu" size="md" footer={<><Button onClick={onClose} disabled={busy}>Huỷ</Button><Button variant="primary" icon="luu" loading={busy} disabled={!l || !nguon.trim()} onClick={() => l && void onSave({ loai: l.loai, lichId: l.id, nguon: nguon.trim(), phuongTienId: ptId || null, phuongTien: pt.find((p) => p.id === ptId)?.ma ?? null, duongDan: duongDan.trim() || null, dungLuongBytes: mb ? Math.round(Number(mb) * 1_048_576) : null, maBam: maBam.trim() || null, ketQua, chiTiet: chiTiet.trim() || null })}>Ghi nhận</Button></>}>
      <div className="space-y-3">
        <Field label="Lịch sao lưu"><Select value={lichId} onChange={chonLich} options={lich.map((x) => ({ value: x.id, label: `${x.ten} — ${LOAI_SAO_LUU[x.loai]}` }))} /></Field>
        <Field label="Nguồn dữ liệu" required hint="Điều 30.3.c"><Input value={nguon} onChange={(e) => setNguon(e.target.value)} /></Field>
        <FormGrid cols={2}>
          <Field label="Phương tiện"><Select value={ptId} onChange={setPtId} placeholder="— Chọn —" options={pt.filter((p) => p.trang_thai === "dang_dung").map((p) => ({ value: p.id, label: `${p.ma} · ${p.loai}` }))} /></Field>
          <Field label="Dung lượng (MB)"><Input type="number" min={0} value={mb} onChange={(e) => setMb(e.target.value)} /></Field>
        </FormGrid>
        <Field label="Đường dẫn bản sao"><Input value={duongDan} onChange={(e) => setDuongDan(e.target.value)} placeholder="E:\DigiText-SaoLuu\3nam\2026-11-26" /></Field>
        <Field label="Mã băm / cách kiểm đọc lại" hint="SHA-256 tệp dump, hoặc 'pg_restore --list đạt'"><Input value={maBam} onChange={(e) => setMaBam(e.target.value)} /></Field>
        <FormGrid cols={2}>
          <Field label="Kết quả kiểm sau sao lưu (Điều 30.3.d)"><Select value={ketQua} onChange={(v) => setKetQua(v as "dat" | "khong_dat")} options={[{ value: "dat", label: "Đạt" }, { value: "khong_dat", label: "Không đạt" }]} /></Field>
          <Field label="Chi tiết"><Input value={chiTiet} onChange={(e) => setChiTiet(e.target.value)} /></Field>
        </FormGrid>
      </div>
    </Dialog>
  );
}
