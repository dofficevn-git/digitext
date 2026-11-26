// Copyright (c) 2026 DigiText. All rights reserved.
// Sự cố và phục hồi (Điều 31, 0032): ghi sự cố, gắn bản sao lưu đã dùng để phục hồi, biên bản Mẫu 04 điền sẵn.

import { useCallback, useMemo, useState } from "react";
import { isTauri } from "@/api/ipc";
import { V41t } from "@/live/useLive";
import { V460 } from "@/state";
import { V3z9 } from "@/domain/permissions";
import { saoLuuRuns, scGanBienBan, scList, scPrefill, scSua, scTao, type SaoLuuRun, type SuCo, type SuCoBody } from "@/api/baoquan";
import { LapBienBanDialog, type NoiDungDien } from "@/features/records/RecordsView";
import { Badge, Button, Card, Dialog, Field, FormGrid, Input, Menu, notifyError, Select, Table, Textarea, toast, type Column } from "@/ui";
import { fmtN, ganBienBanMoiNhat, LOAI_SAO_LUU, MUC_DO, ngayGio, toLocalInput, TT_SU_CO } from "./baoQuanFlow";

export default function SuCoPanel() {
  const { session } = V460();
  const canManage = V3z9(session?.permissions, "bk.manage");
  const [dlg, setDlg] = useState<{ row: SuCo | null } | null>(null);
  const [bienBan, setBienBan] = useState<{ suCo: SuCo; nd: NoiDungDien } | null>(null);
  const [busy, setBusy] = useState(false);
  const loader = useCallback(async () => {
    const [sc, runs] = await Promise.all([scList(false, 300), saoLuuRuns(null, 100)]);
    return { sc: sc.rows, runs: runs.rows };
  }, []);
  const { data, loading, error, reload } = V41t(loader, { topics: ["ops"], enabled: isTauri(), errorText: () => "Không tải được sự cố." });
  const rows = useMemo(() => data?.sc ?? [], [data]);
  const mo = rows.filter((s) => s.trang_thai !== "dong").length;
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
  const moBienBan = (s: SuCo) => void chay("điền biên bản", async () => setBienBan({ suCo: s, nd: await scPrefill(s.id) }));
  const cols: Column<SuCo>[] = [
    { key: "ma", header: "Mã", width: 120, cell: (s) => <span className="font-medium">{s.ma}</span> },
    { key: "thoi_diem", header: "Thời điểm", width: 140, sortable: true, cell: (s) => ngayGio(s.thoi_diem) },
    { key: "muc_do", header: "Mức độ", width: 120, cell: (s) => <Badge tone={MUC_DO[s.muc_do].tone}>{MUC_DO[s.muc_do].label}</Badge> },
    { key: "mo_ta", header: "Sự cố", className: "whitespace-normal", cell: (s) => <><div>{s.mo_ta}</div>{s.nguyen_nhan && <div className="text-xs text-fg-subtle">Nguyên nhân: {s.nguyen_nhan}</div>}</> },
    { key: "backup_run", header: "Phục hồi từ", width: 200, className: "text-sm", cell: (s) => (s.backup_run ? `${LOAI_SAO_LUU[s.backup_run.loai] ?? s.backup_run.loai} · ${ngayGio(s.backup_run.created_at)}` : "—") },
    { key: "trang_thai", header: "Trạng thái", width: 120, cell: (s) => <Badge tone={TT_SU_CO[s.trang_thai].tone}>{TT_SU_CO[s.trang_thai].label}</Badge> },
    { key: "so_bien_ban", header: "Biên bản", width: 140, cell: (s) => s.so_bien_ban ?? "—" },
  ];
  return (
    <div className="space-y-4">
      <Card padding="none" title={`Sự cố (${fmtN(mo)} đang mở)`} description="Điều 31: cách ly, phục hồi từ bản sao lưu gần nhất, kiểm tra sau phục hồi, lập biên bản Mẫu 04." actions={canManage && <Button size="sm" variant="primary" icon="canhBao" onClick={() => setDlg({ row: null })}>Ghi sự cố</Button>}>
        <Table
          columns={cols}
          rows={rows}
          getRowId={(s) => s.id}
          dense
          maxHeight={560}
          rowActions={canManage ? (s) => <Menu size="sm" items={[{ label: "Sửa / cập nhật xử lý", icon: "sua", onSelect: () => setDlg({ row: s }) }, { label: s.record_id ? "Lập lại biên bản Mẫu 04" : "Biên bản Mẫu 04…", icon: "bienBan", disabled: busy, onSelect: () => moBienBan(s) }]} /> : undefined}
          state={{ loading: loading && !data, error: data ? undefined : error, onRetry: reload, empty: { title: "Chưa ghi sự cố nào", icon: "thanhCong" } }}
        />
      </Card>
      {dlg && (
        <SuCoDialog
          row={dlg.row}
          runs={data?.runs ?? []}
          busy={busy}
          onClose={() => setDlg(null)}
          onSave={async (b) => {
            await chay(dlg.row ? "cập nhật sự cố" : "ghi sự cố", () => (dlg.row ? scSua(dlg.row.id, b) : scTao(b)), dlg.row ? "Đã cập nhật." : "Đã ghi sự cố; quản trị được báo nếu mức cao.");
            setDlg(null);
          }}
        />
      )}
      {bienBan && (
        <LapBienBanDialog
          kinds={["su_co"]}
          projectId={bienBan.suCo.project_id}
          prefill={bienBan.nd}
          onClose={() => setBienBan(null)}
          onCreated={() => {
            const s = bienBan.suCo;
            void ganBienBanMoiNhat("su_co", s.project_id, (rid) => scGanBienBan(s.id, rid)).then(() => reload());
          }}
        />
      )}
    </div>
  );
}

function SuCoDialog({ row, runs, busy, onClose, onSave }: { row: SuCo | null; runs: SaoLuuRun[]; busy: boolean; onClose: () => void; onSave: (b: SuCoBody) => Promise<void> }) {
  const [thoiDiem, setThoiDiem] = useState(toLocalInput(row?.thoi_diem));
  const [moTa, setMoTa] = useState(row?.mo_ta ?? "");
  const [nguyenNhan, setNguyenNhan] = useState(row?.nguyen_nhan ?? "");
  const [mucDo, setMucDo] = useState<SuCo["muc_do"]>(row?.muc_do ?? "trung_binh");
  const [phamVi, setPhamVi] = useState(row?.pham_vi ?? "");
  const [bienPhap, setBienPhap] = useState(row?.bien_phap ?? "");
  const [runId, setRunId] = useState(row?.backup_run_id ?? "");
  const [ppPhucHoi, setPp] = useState(row?.phuong_phap_phuc_hoi ?? "");
  const [phucHoiLuc, setPhucHoiLuc] = useState(row?.phuc_hoi_luc ? toLocalInput(row.phuc_hoi_luc) : "");
  const [ketQua, setKetQua] = useState(row?.ket_qua_kiem_tra ?? "");
  const [duLieuMat, setDuLieuMat] = useState(row?.du_lieu_mat ?? "");
  const [tt, setTt] = useState<SuCo["trang_thai"]>(row?.trang_thai ?? "moi");
  const iso = (v: string) => (v ? new Date(v).toISOString() : null);
  return (
    <Dialog open onOpenChange={(o) => !o && !busy && onClose()} title={row ? `Sự cố ${row.ma}` : "Ghi sự cố"} size="lg" footer={<><Button onClick={onClose} disabled={busy}>Huỷ</Button><Button variant="primary" icon="luu" loading={busy} disabled={!moTa.trim()} onClick={() => void onSave({ projectId: row?.project_id ?? null, thoiDiem: iso(thoiDiem), moTa: moTa.trim(), nguyenNhan: nguyenNhan.trim() || null, mucDo, phamVi: phamVi.trim() || null, bienPhap: bienPhap.trim() || null, backupRunId: runId || null, phuongPhapPhucHoi: ppPhucHoi.trim() || null, phucHoiLuc: iso(phucHoiLuc), ketQuaKiemTra: ketQua.trim() || null, duLieuMat: duLieuMat.trim() || null, trangThai: tt })}>Lưu</Button></>}>
      <div className="space-y-3">
        <FormGrid cols={3}>
          <Field label="Thời điểm xảy ra" required><Input type="datetime-local" value={thoiDiem} onChange={(e) => setThoiDiem(e.target.value)} /></Field>
          <Field label="Mức độ"><Select value={mucDo} onChange={(v) => setMucDo(v as SuCo["muc_do"])} options={Object.entries(MUC_DO).map(([value, x]) => ({ value, label: x.label }))} /></Field>
          <Field label="Trạng thái"><Select value={tt} onChange={(v) => setTt(v as SuCo["trang_thai"])} options={Object.entries(TT_SU_CO).map(([value, x]) => ({ value, label: x.label }))} /></Field>
        </FormGrid>
        <Field label="Mô tả sự cố" required><Textarea rows={2} value={moTa} onChange={(e) => setMoTa(e.target.value)} autoFocus /></Field>
        <FormGrid cols={2}>
          <Field label="Nguyên nhân" hint="Phần cứng, phần mềm, con người (Điều 31.1)"><Input value={nguyenNhan} onChange={(e) => setNguyenNhan(e.target.value)} /></Field>
          <Field label="Phạm vi ảnh hưởng"><Input value={phamVi} onChange={(e) => setPhamVi(e.target.value)} placeholder="Kho tài liệu dự án X; CSDL" /></Field>
        </FormGrid>
        <Field label="Biện pháp cách ly, khắc phục" hint="Mỗi dòng một biện pháp (Điều 31.2, 31.3)"><Textarea rows={3} value={bienPhap} onChange={(e) => setBienPhap(e.target.value)} /></Field>
        <FormGrid cols={2}>
          <Field label="Phục hồi từ bản sao lưu" hint="Điều 31.4"><Select value={runId} onChange={setRunId} placeholder="— Chưa phục hồi —" options={runs.filter((r) => r.ket_qua === "dat").map((r) => ({ value: r.id, label: `${ngayGio(r.created_at)} · ${LOAI_SAO_LUU[r.loai] ?? r.loai} · ${r.nguon}` }))} /></Field>
          <Field label="Phục hồi lúc"><Input type="datetime-local" value={phucHoiLuc} onChange={(e) => setPhucHoiLuc(e.target.value)} /></Field>
          <Field label="Phương pháp phục hồi"><Input value={ppPhucHoi} onChange={(e) => setPp(e.target.value)} placeholder="pg_restore --clean; chép lại kho từ bản sao" /></Field>
          <Field label="Kết quả kiểm tra sau phục hồi" hint="Điều 31.5"><Input value={ketQua} onChange={(e) => setKetQua(e.target.value)} /></Field>
        </FormGrid>
        <Field label="Dữ liệu mất (nếu có)"><Input value={duLieuMat} onChange={(e) => setDuLieuMat(e.target.value)} placeholder="Không mất dữ liệu" /></Field>
      </div>
    </Dialog>
  );
}
