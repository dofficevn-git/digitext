// Copyright (c) 2026 DigiText. All rights reserved.
// Phương tiện lưu trữ (Điều 30.4, 0032): danh mục ổ cứng / băng từ / NAS… với hạn độ bền; máy chủ nhắc chuyển đổi trước 1 năm.

import { useCallback, useMemo, useState } from "react";
import { isTauri } from "@/api/ipc";
import { V41t } from "@/live/useLive";
import { V460 } from "@/state";
import { V3z9 } from "@/domain/permissions";
import { ptList, ptSua, ptTao, type PhuongTien, type PhuongTienBody } from "@/api/baoquan";
import { Badge, Button, Card, Dialog, Field, FormGrid, Input, Menu, notifyError, Select, Table, Textarea, toast, type Column } from "@/ui";
import { fmtMB, fmtN, LOAI_PHUONG_TIEN, ngay, TT_PHUONG_TIEN } from "./baoQuanFlow";

export default function PhuongTienPanel() {
  const { session } = V460();
  const canManage = V3z9(session?.permissions, "bk.manage");
  const [dlg, setDlg] = useState<{ row: PhuongTien | null } | null>(null);
  const [busy, setBusy] = useState(false);
  const loader = useCallback(() => ptList(), []);
  const { data, loading, error, reload } = V41t(loader, { topics: ["ops"], enabled: isTauri(), errorText: () => "Không tải được danh mục phương tiện." });
  const rows = useMemo(() => data?.rows ?? [], [data]);
  const sapHet = rows.filter((p) => p.sap_het).length;
  const luu = async (b: PhuongTienBody, row: PhuongTien | null) => {
    setBusy(true);
    try {
      if (row) await ptSua(row.id, b);
      else await ptTao(b);
      toast.success(row ? "Đã lưu phương tiện." : "Đã thêm phương tiện.");
      setDlg(null);
      reload();
    } catch (e) {
      notifyError(e, "lưu phương tiện");
    } finally {
      setBusy(false);
    }
  };
  const cols: Column<PhuongTien>[] = [
    { key: "ma", header: "Mã", width: 120, cell: (p) => <span className="font-medium">{p.ma}</span> },
    { key: "loai", header: "Loại", width: 140, cell: (p) => LOAI_PHUONG_TIEN.find((l) => l.value === p.loai)?.label ?? p.loai },
    { key: "mo_ta", header: "Mô tả", className: "whitespace-normal", cell: (p) => p.mo_ta ?? "" },
    { key: "dung_luong_bytes", header: "Dung lượng", align: "right", width: 100, cell: (p) => fmtMB(p.dung_luong_bytes) },
    { key: "noi_bao_quan", header: "Nơi bảo quản", width: 160, cell: (p) => p.noi_bao_quan ?? "—" },
    { key: "ngay_dua_vao", header: "Đưa vào", width: 100, cell: (p) => ngay(p.ngay_dua_vao) },
    {
      key: "han_do_ben",
      header: "Hạn độ bền",
      width: 170,
      cell: (p) =>
        p.han_do_ben ? (
          <span className="flex items-center gap-2">
            {ngay(p.han_do_ben)}
            {p.trang_thai === "dang_dung" && p.con_ngay !== null && (p.con_ngay < 0 ? <Badge tone="err">Quá hạn {fmtN(-p.con_ngay)} ngày</Badge> : p.sap_het ? <Badge tone="warn">Còn {fmtN(p.con_ngay)} ngày</Badge> : null)}
          </span>
        ) : (
          "—"
        ),
    },
    { key: "so_lan_dung", header: "Lần dùng", align: "right", width: 80, cell: (p) => fmtN(p.so_lan_dung) },
    { key: "trang_thai", header: "Trạng thái", width: 110, cell: (p) => <Badge tone={TT_PHUONG_TIEN[p.trang_thai]?.tone ?? "gray"}>{TT_PHUONG_TIEN[p.trang_thai]?.label ?? p.trang_thai}{p.thay_boi_ma ? ` → ${p.thay_boi_ma}` : ""}</Badge> },
  ];
  return (
    <div className="space-y-4">
      <Card
        padding="none"
        title="Phương tiện lưu trữ"
        description={sapHet ? `${fmtN(sapHet)} phương tiện cần chuyển đổi trong 1 năm (Điều 30.4).` : "Khai hạn độ bền để máy chủ nhắc chuyển đổi trước 1 năm."}
        actions={canManage && <Button size="sm" variant="primary" icon="them" onClick={() => setDlg({ row: null })}>Thêm phương tiện</Button>}
      >
        <Table
          columns={cols}
          rows={rows}
          getRowId={(p) => p.id}
          dense
          maxHeight={520}
          rowActions={canManage ? (p) => <Menu size="sm" items={[{ label: "Sửa", icon: "sua", onSelect: () => setDlg({ row: p }) }]} /> : undefined}
          state={{ loading: loading && !data, error: data ? undefined : error, onRetry: reload, empty: { title: "Chưa khai phương tiện lưu trữ", description: "Ổ sao lưu, băng từ, NAS… kèm hạn độ bền nhà sản xuất công bố." } }}
        />
      </Card>
      {dlg && <PhuongTienDialog row={dlg.row} khac={rows.filter((p) => p.id !== dlg.row?.id)} busy={busy} onClose={() => setDlg(null)} onSave={(b) => luu(b, dlg.row)} />}
    </div>
  );
}

function PhuongTienDialog({ row, khac, busy, onClose, onSave }: { row: PhuongTien | null; khac: PhuongTien[]; busy: boolean; onClose: () => void; onSave: (b: PhuongTienBody) => Promise<void> }) {
  const [ma, setMa] = useState(row?.ma ?? "");
  const [loai, setLoai] = useState(row?.loai ?? "o_cung");
  const [moTa, setMoTa] = useState(row?.mo_ta ?? "");
  const [gb, setGb] = useState(row?.dung_luong_bytes ? String(Math.round(row.dung_luong_bytes / 1_073_741_824)) : "");
  const [noi, setNoi] = useState(row?.noi_bao_quan ?? "");
  const [duaVao, setDuaVao] = useState(row?.ngay_dua_vao ?? "");
  const [han, setHan] = useState(row?.han_do_ben ?? "");
  const [tt, setTt] = useState<PhuongTien["trang_thai"]>(row?.trang_thai ?? "dang_dung");
  const [thayBoi, setThayBoi] = useState(row?.thay_boi_id ?? "");
  const [ghiChu, setGhiChu] = useState(row?.ghi_chu ?? "");
  return (
    <Dialog open onOpenChange={(o) => !o && !busy && onClose()} title={row ? `Sửa phương tiện ${row.ma}` : "Thêm phương tiện lưu trữ"} size="md" footer={<><Button onClick={onClose} disabled={busy}>Huỷ</Button><Button variant="primary" icon="luu" loading={busy} disabled={!ma.trim()} onClick={() => void onSave({ ma: ma.trim(), loai, moTa: moTa.trim() || null, dungLuongBytes: gb ? Math.round(Number(gb) * 1_073_741_824) : null, noiBaoQuan: noi.trim() || null, ngayDuaVao: duaVao || null, hanDoBen: han || null, trangThai: tt, thayBoiId: thayBoi || null, ghiChu: ghiChu.trim() || null })}>Lưu</Button></>}>
      <div className="space-y-3">
        <FormGrid cols={2}>
          <Field label="Mã / số hiệu" required><Input value={ma} onChange={(e) => setMa(e.target.value)} placeholder="HDD-SL-01" autoFocus /></Field>
          <Field label="Loại"><Select value={loai} onChange={setLoai} options={LOAI_PHUONG_TIEN} /></Field>
        </FormGrid>
        <Field label="Mô tả" hint="Hãng, model, số seri"><Input value={moTa} onChange={(e) => setMoTa(e.target.value)} /></Field>
        <FormGrid cols={2}>
          <Field label="Dung lượng (GB)"><Input type="number" min={0} value={gb} onChange={(e) => setGb(e.target.value)} /></Field>
          <Field label="Nơi bảo quản" hint="Điều 30.3.đ"><Input value={noi} onChange={(e) => setNoi(e.target.value)} /></Field>
          <Field label="Ngày đưa vào dùng"><Input type="date" value={duaVao} onChange={(e) => setDuaVao(e.target.value)} /></Field>
          <Field label="Hạn độ bền" hint="Nhắc chuyển đổi trước 1 năm (Điều 30.4)"><Input type="date" value={han} onChange={(e) => setHan(e.target.value)} /></Field>
          <Field label="Trạng thái"><Select value={tt} onChange={(v) => setTt(v as PhuongTien["trang_thai"])} options={Object.entries(TT_PHUONG_TIEN).map(([value, x]) => ({ value, label: x.label }))} /></Field>
          {tt === "da_thay" && <Field label="Thay bởi"><Select value={thayBoi} onChange={setThayBoi} placeholder="— Chọn —" options={khac.map((p) => ({ value: p.id, label: p.ma }))} /></Field>}
        </FormGrid>
        <Field label="Ghi chú"><Textarea rows={2} value={ghiChu} onChange={(e) => setGhiChu(e.target.value)} /></Field>
      </div>
    </Dialog>
  );
}
