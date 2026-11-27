// Copyright (c) 2026 DigiText. All rights reserved.
// Hộp thoại "Cơ quan lưu trữ (TT05)" của dự án (0039, 27/11/2026 — đợt 5, Digifile B01): mã cơ quan lưu trữ (Phụ lục I), tên /
// cấp trên / địa chỉ, loại hình, người ký + chức vụ (khối ký báo cáo, trang thể thức chuyển đổi), hạn nộp lưu / mốc nhắc / quyền
// sử dụng (Điều 19). Chưa khai thì máy chủ dùng mặc định: tên dự án, 60 ngày, nhắc 15/7/1, quyền 360 ngày.

import { useEffect, useState } from "react";
import { coQuanGet, coQuanSet, LOAI_HINH, type CoQuanDto, type CoQuanMoi } from "@/api/coQuan";
import { Badge, Button, Dialog, ErrorBanner, errorText, Field, FormGrid, Input, notifyError, Select, Textarea, toast, Toolbar } from "@/ui";

type Form = {
  maCoQuan: string;
  ten: string;
  tenCapTren: string;
  diaChi: string;
  dienThoai: string;
  email: string;
  loaiHinh: "luu_tru_co_quan" | "luu_tru_lich_su";
  nguoiKy: string;
  chucVuNguoiKy: string;
  hanNopNgay: string;
  nhacTruocNgay: string;
  quyenSuDungNgay: string;
  ghiChu: string;
};

function tuDto(d: CoQuanDto): Form {
  const c = d.cauHinh;
  const h = d.hieuDung;
  return {
    maCoQuan: c?.maCoQuan ?? "",
    ten: c?.ten ?? h.ten,
    tenCapTren: c?.tenCapTren ?? "",
    diaChi: c?.diaChi ?? "",
    dienThoai: c?.dienThoai ?? "",
    email: c?.email ?? "",
    loaiHinh: c?.loaiHinh ?? "luu_tru_co_quan",
    nguoiKy: c?.nguoiKy ?? "",
    chucVuNguoiKy: c?.chucVuNguoiKy ?? "",
    hanNopNgay: String(c?.hanNopNgay ?? h.hanNopNgay),
    nhacTruocNgay: (c?.nhacTruocNgay ?? h.nhacTruocNgay).join(", "),
    quyenSuDungNgay: String(c?.quyenSuDungNgay ?? h.quyenSuDungNgay),
    ghiChu: c?.ghiChu ?? "",
  };
}

function docMoc(s: string): number[] | null {
  const xs = s.split(/[,;\s]+/).filter(Boolean).map(Number);
  if (xs.some((x) => !Number.isInteger(x) || x < 0 || x > 3650)) return null;
  return xs;
}

export default function CoQuanDialog({ projectId, projectLabel, canEdit, onClose, onSaved }: { projectId: string; projectLabel?: string; canEdit: boolean; onClose: () => void; onSaved?: () => void }) {
  const [dto, setDto] = useState<CoQuanDto | null>(null);
  const [f, setF] = useState<Form | null>(null);
  const [err, setErr] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    let huy = false;
    coQuanGet(projectId)
      .then((d) => {
        if (huy) return;
        setDto(d);
        setF(tuDto(d));
      })
      .catch((e) => !huy && setErr(errorText(e)));
    return () => {
      huy = true;
    };
  }, [projectId]);
  const set = <K extends keyof Form>(k: K, v: Form[K]) => setF((o) => (o ? { ...o, [k]: v } : o));
  const han = f ? Number(f.hanNopNgay) : NaN;
  const qsd = f ? Number(f.quyenSuDungNgay) : NaN;
  const moc = f ? docMoc(f.nhacTruocNgay) : null;
  const loiHan = f && (!Number.isInteger(han) || han < 1 || han > 3650) ? "1 đến 3650 ngày" : undefined;
  const loiQsd = f && (!Number.isInteger(qsd) || qsd < 0 || qsd > 3650) ? "0 đến 3650 ngày" : undefined;
  const loiMoc = f && !moc ? "Các số ngày cách nhau bởi dấu phẩy, ví dụ 15, 7, 1" : undefined;
  const loiEmail = f && f.email.trim() && !f.email.includes("@") ? "Email không hợp lệ" : undefined;
  const hopLe = !!f && !loiHan && !loiQsd && !loiMoc && !loiEmail;

  const save = async () => {
    if (!f || !hopLe || busy) return;
    setBusy(true);
    try {
      const body: CoQuanMoi = {
        maCoQuan: f.maCoQuan.trim() || null,
        ten: f.ten.trim() || null,
        tenCapTren: f.tenCapTren.trim() || null,
        diaChi: f.diaChi.trim() || null,
        dienThoai: f.dienThoai.trim() || null,
        email: f.email.trim() || null,
        loaiHinh: f.loaiHinh,
        nguoiKy: f.nguoiKy.trim() || null,
        chucVuNguoiKy: f.chucVuNguoiKy.trim() || null,
        hanNopNgay: han,
        nhacTruocNgay: moc ?? undefined,
        quyenSuDungNgay: qsd,
        ghiChu: f.ghiChu.trim() || null,
      };
      const d = await coQuanSet(projectId, body);
      setDto(d);
      setF(tuDto(d));
      toast.success("Đã lưu cấu hình cơ quan lưu trữ.");
      onSaved?.();
      onClose();
    } catch (e) {
      notifyError(e, "lưu cấu hình cơ quan");
    } finally {
      setBusy(false);
    }
  };

  return (
    <Dialog
      open
      onOpenChange={(o) => !o && !busy && onClose()}
      title={`Cơ quan lưu trữ (TT05)${projectLabel ? ` — ${projectLabel}` : ""}`}
      description="Dùng cho tiêu đề và khối ký của báo cáo, trang thể thức chuyển đổi (Phụ lục VI), hạn nộp lưu và mốc nhắc (Điều 19). Chưa khai thì dùng tên dự án và tham số mặc định."
      size="lg"
      footer={
        <>
          {dto && <Badge tone={dto.hieuDung.daKhai ? "ok" : "warn"}>{dto.hieuDung.daKhai ? "Đã khai" : "Chưa khai — đang dùng mặc định"}</Badge>}
          <Toolbar.Spacer />
          <Button onClick={onClose} disabled={busy}>
            Đóng
          </Button>
          {canEdit && (
            <Button variant="primary" loading={busy} disabled={!hopLe} onClick={() => void save()}>
              Lưu
            </Button>
          )}
        </>
      }
    >
      {err ? (
        <ErrorBanner error={err} />
      ) : !f ? (
        <p className="text-sm text-fg-subtle">Đang tải…</p>
      ) : (
        <fieldset disabled={!canEdit || busy} className="m-0 border-0 p-0">
          <FormGrid>
            <Field label="Mã cơ quan lưu trữ" hint="Phụ lục I TT05 (ví dụ H49). Dùng khi đóng gói và trên trang thể thức." hintAs="tip">
              <Input value={f.maCoQuan} onChange={(e) => set("maCoQuan", e.target.value.toUpperCase())} placeholder="H49" />
            </Field>
            <Field label="Loại hình">
              <Select value={f.loaiHinh} onChange={(v) => set("loaiHinh", v as Form["loaiHinh"])} options={LOAI_HINH.map(([value, label]) => ({ value, label }))} />
            </Field>
            <Field label="Tên cơ quan" required>
              <Input value={f.ten} onChange={(e) => set("ten", e.target.value)} />
            </Field>
            <Field label="Cơ quan cấp trên" hint="In trên đầu báo cáo, dòng trên tên cơ quan" hintAs="tip">
              <Input value={f.tenCapTren} onChange={(e) => set("tenCapTren", e.target.value)} />
            </Field>
            <Field label="Địa chỉ">
              <Input value={f.diaChi} onChange={(e) => set("diaChi", e.target.value)} />
            </Field>
            <Field label="Điện thoại">
              <Input value={f.dienThoai} onChange={(e) => set("dienThoai", e.target.value)} />
            </Field>
            <Field label="Email" error={loiEmail}>
              <Input value={f.email} onChange={(e) => set("email", e.target.value)} />
            </Field>
            <Field label="Người ký" hint="Tên in sẵn dưới khối ký của báo cáo và bản chuyển đổi; để trống thì chừa chỗ ký" hintAs="tip">
              <Input value={f.nguoiKy} onChange={(e) => set("nguoiKy", e.target.value)} />
            </Field>
            <Field label="Chức vụ người ký" hint="Mặc định LÃNH ĐẠO CƠ QUAN" hintAs="tip">
              <Input value={f.chucVuNguoiKy} onChange={(e) => set("chucVuNguoiKy", e.target.value)} placeholder="Giám đốc" />
            </Field>
            <Field label="Hạn nộp lưu (ngày)" error={loiHan} hint="Tính từ ngày kết thúc hồ sơ — Điều 19" hintAs="tip">
              <Input inputMode="numeric" value={f.hanNopNgay} onChange={(e) => set("hanNopNgay", e.target.value)} />
            </Field>
            <Field label="Nhắc trước hạn (ngày)" error={loiMoc}>
              <Input value={f.nhacTruocNgay} onChange={(e) => set("nhacTruocNgay", e.target.value)} placeholder="15, 7, 1" />
            </Field>
            <Field label="Quyền sử dụng sau nộp (ngày)" error={loiQsd} hint="Người nộp còn đọc được hồ sơ đã nộp — Điều 19.5" hintAs="tip">
              <Input inputMode="numeric" value={f.quyenSuDungNgay} onChange={(e) => set("quyenSuDungNgay", e.target.value)} />
            </Field>
          </FormGrid>
          <Field label="Ghi chú" className="mt-3">
            <Textarea rows={2} value={f.ghiChu} onChange={(e) => set("ghiChu", e.target.value)} />
          </Field>
        </fieldset>
      )}
    </Dialog>
  );
}
