// Copyright (c) 2026 DigiText. All rights reserved.
// Hệ thống hoá hồ sơ (chỉ online): lọc tập hồ sơ trong phông → chọn thứ tự → Xem trước → Áp dụng (đánh số chính
// thức) → Xuất mục lục tạm, điền hộp / bó / tập → Nhập lại (gán hộp, tự tạo hộp theo phòng ban) · gán lẻ cho dòng chọn.

import { useMemo, useRef, useState } from "react";
import { V460 } from "@/state";
import { clRemote, str, THBQ_STATUS_LABEL } from "@/api/chinhly";
import { CanMayChu, Chip, ChuaChonDuAn, n, Select, tenPhong, THBQ_BADGE, useClCatalog, useClSync } from "./common";
import { docMucLucTam, xuatMucLucTam, type PreviewRow } from "./excel";
import { Badge, Banner, Button, Card, Checkbox, confirm, Field, FormGrid, Input, notifyError, PageHeader, prompt, Select as UiSelect, Stepper, Table, Tip, toast, Toolbar, type Column } from "@/ui";

const ORDERS: [number, string][] = [
  [1, "Mặt hoạt động, rồi thời gian"],
  [2, "Thời gian, rồi mặt hoạt động"],
  [3, "THBQ tăng dần, mặt hoạt động, thời gian"],
  [4, "THBQ giảm dần, mặt hoạt động, thời gian"],
  [5, "THBQ tăng dần, thời gian, mặt hoạt động"],
  [6, "THBQ giảm dần, thời gian, mặt hoạt động"],
];

interface Preview {
  snapshotVer: number;
  order: number;
  fileType: number;
  rows: PreviewRow[];
}
interface ImportRes {
  applied: number;
  hopsCreated: number;
  boTapCreated: number;
  errors: { row: number; message: string }[];
}

export default function HeThongHoaView() {
  const { session } = V460();
  const projectId = session?.projectId ?? "";
  const canDo = (session?.permissions ?? []).includes("dossier.systematize");
  const { cat } = useClCatalog();
  const sync = useClSync();
  const online = sync?.phase !== "offline" && sync?.phase !== "error";

  const [phongId, setPhongId] = useState("");
  const [phongBanId, setPhongBanId] = useState("");
  const [mangId, setMangId] = useState("");
  const [thbqIds, setThbqIds] = useState<string[]>([]);
  const [status, setStatus] = useState("");
  const [trungThua, setTrungThua] = useState<"" | "1" | "0">("");
  const [chuaDanhSo, setChuaDanhSo] = useState(true);
  const [order, setOrder] = useState(1);
  const [fileType, setFileType] = useState<1 | 2>(1);
  const [startAt, setStartAt] = useState(1);

  const [preview, setPreview] = useState<Preview | null>(null);
  const [applied, setApplied] = useState(false);
  const [daXuat, setDaXuat] = useState(false);
  const [busy, setBusy] = useState(false);
  const [sel, setSel] = useState<Set<string>>(new Set());
  const [chiCanhBao, setChiCanhBao] = useState(false);
  const [importRes, setImportRes] = useState<ImportRes | null>(null);
  const fileRef = useRef<HTMLInputElement>(null);

  const phong = cat.phong.find((p) => p.id === phongId);
  const phongBans = useMemo(() => cat.phong_ban.filter((p) => str(p, "phong_id") === phongId), [cat.phong_ban, phongId]);
  const mangs = useMemo(() => cat.mang.filter((m) => str(m, "phong_ban_id") === phongBanId), [cat.mang, phongBanId]);
  const tenOrder = ORDERS.find((o) => o[0] === order)?.[1] ?? "";
  const tenDonVi = fileType === 1 ? "hộp" : "bó / tập";

  const body = () => ({
    projectId,
    phongId,
    filter: { phongBanId: phongBanId || undefined, mangIds: mangId ? [mangId] : [], thbqIds, status: status || undefined, trungThua: trungThua === "" ? undefined : trungThua === "1", chuaDanhSo },
    sortOrder: order,
    fileType,
    startAt,
  });

  const xemTruoc = async () => {
    setBusy(true);
    setApplied(false);
    setDaXuat(false);
    setImportRes(null);
    try {
      const p = await clRemote<Preview>("POST", "/cl/he-thong-hoa/preview", body());
      setPreview(p);
      setSel(new Set());
      if (p.rows.length === 0) toast.info("Không có hồ sơ nào khớp bộ lọc.");
    } catch (e) {
      notifyError(e, "xem trước");
    } finally {
      setBusy(false);
    }
  };
  const apDung = async () => {
    if (!preview) return;
    const ok = await confirm({ title: `Đánh số chính thức ${n(preview.rows.length)} hồ sơ?`, body: `Bắt đầu từ số ${startAt}, theo thứ tự "${tenOrder}". Số cũ nếu có sẽ bị thay.`, confirmLabel: "Đánh số" });
    if (!ok) return;
    setBusy(true);
    try {
      const p = await clRemote<Preview>("POST", "/cl/he-thong-hoa/apply", { ...body(), snapshotVer: preview.snapshotVer });
      setPreview(p);
      setApplied(true);
      toast.success(`Đã đánh số ${n(p.rows.length)} hồ sơ.`, { description: `Xuất mục lục tạm, điền ${tenDonVi} rồi nhập lại.` });
    } catch (e) {
      notifyError(e, "đánh số");
    } finally {
      setBusy(false);
    }
  };
  const xuat = async () => {
    if (!preview) return;
    try {
      await xuatMucLucTam(preview.rows, fileType, phongId, str(phong, "ten"), tenOrder);
      setDaXuat(true);
    } catch (e) {
      notifyError(e, "xuất mục lục tạm");
    }
  };
  const nhap = async (f: File) => {
    setBusy(true);
    setImportRes(null);
    try {
      const d = await docMucLucTam(f);
      if (d.rows.length === 0) {
        toast.warning(d.errors[0]?.message ?? "Tệp không có dòng dữ liệu.");
        setImportRes({ applied: 0, hopsCreated: 0, boTapCreated: 0, errors: d.errors });
        return;
      }
      const r = await clRemote<{ applied: number; hopsCreated: number; boTapCreated: number; errors: { row: number; hoSoId: string; message: string }[] }>("POST", "/cl/he-thong-hoa/boxing", { projectId, fileType: d.fileType, rows: d.rows });
      setImportRes({ ...r, errors: [...d.errors, ...r.errors.map((e) => ({ row: e.row + 2, message: e.message }))] });
      toast.success(`Đã gán ${n(r.applied)} hồ sơ. Tạo mới ${n(r.hopsCreated)} hộp, ${n(r.boTapCreated)} bó / tập.`);
      if (preview) void xemTruoc();
    } catch (e) {
      notifyError(e, "nhập tệp đã điền");
    } finally {
      setBusy(false);
    }
  };
  const ganLe = async () => {
    if (sel.size === 0) return;
    const v = await prompt({
      title: `Gán ${tenDonVi} cho ${n(sel.size)} hồ sơ đã chọn`,
      label: fileType === 1 ? "Số hộp" : "Bó số / tập số",
      placeholder: fileType === 1 ? "12" : "3/1",
      body: fileType === 1 ? "Để trống để bỏ gán." : "Ghi bó và tập cách nhau dấu gạch chéo. Để trống để bỏ gán.",
      confirmLabel: "Gán",
    });
    if (v === null) return;
    const t = v.trim();
    setBusy(true);
    try {
      const rows = [...sel].map((id) => {
        if (fileType === 1) return { hoSoId: id, hopSo: t };
        const [b, tp] = t.split("/").map((x) => Number.parseInt(x.trim(), 10));
        return { hoSoId: id, boSo: Number.isNaN(b) ? undefined : b, tapSo: Number.isNaN(tp) ? undefined : tp, clear: t === "" };
      });
      const r = await clRemote<{ applied: number; errors: { message: string }[] }>("POST", "/cl/he-thong-hoa/boxing", { projectId, fileType, rows });
      if (r.errors.length) toast.warning(`Đã gán ${n(r.applied)} hồ sơ, ${r.errors.length} lỗi.`, { description: r.errors[0].message });
      else toast.success(`Đã gán ${n(r.applied)} hồ sơ.`);
      void xemTruoc();
    } catch (e) {
      notifyError(e, "gán");
    } finally {
      setBusy(false);
    }
  };

  const khoa = !online || !canDo;
  const rowsHien = useMemo(() => (preview ? (chiCanhBao ? preview.rows.filter((r) => r.warnings.length > 0) : preview.rows) : []), [preview, chiCanhBao]);
  const canhBao = preview?.rows.filter((r) => r.warnings.length > 0).length ?? 0;
  const buoc = !preview ? 0 : !applied ? 1 : !daXuat && !importRes ? 2 : 3;

  const cols: Column<PreviewRow>[] = [
    { key: "stt", header: applied ? "Hồ sơ số" : "Hồ sơ số dự kiến", align: "right", width: 100, cell: (r) => <b>{r.stt}</b> },
    { key: "hoSoSoCu", header: "Số cũ", align: "right", width: 70, className: "text-fg-subtle", cell: (r) => r.hoSoSoCu ?? "" },
    { key: "hop", header: fileType === 1 ? "Hộp" : "Bó / tập", width: 90, cell: (r) => (fileType === 1 ? (r.hopSo ?? "") : [r.boSo, r.tapSo].filter((x) => x != null).join(" / ")) },
    { key: "maHoSoMoi", header: "Mã hồ sơ mới", className: "text-xs", cell: (r) => r.maHoSoMoi ?? <span className="text-fg-subtle">—</span> },
    { key: "soTam", header: "Số tạm", width: 90, cell: (r) => r.soTam },
    { key: "tieuDe", header: "Tiêu đề", className: "max-w-[420px] whitespace-normal", cell: (r) => r.tieuDe },
    { key: "tenPhongBan", header: "Phòng ban", className: "text-sm", cell: (r) => r.tenPhongBan ?? "", hideBelow: 1400 },
    { key: "tenMang", header: "Mảng", className: "text-sm", cell: (r) => r.tenMang ?? "", hideBelow: 1400 },
    { key: "tenThbq", header: "THBQ", className: "text-sm", cell: (r) => r.tenThbq ?? "" },
    { key: "thbqStatus", header: "Trạng thái", width: 100, cell: (r) => (r.thbqStatus ? <Badge tone={THBQ_BADGE[r.thbqStatus] ?? "gray"}>{THBQ_STATUS_LABEL[r.thbqStatus] ?? r.thbqStatus}</Badge> : null) },
    { key: "ngayKetThuc", header: "Kết thúc", width: 100, cell: (r) => r.ngayKetThuc ?? "" },
    { key: "soTo", header: "Số tờ", align: "right", width: 70, cell: (r) => r.soTo ?? "" },
    {
      key: "warnings",
      header: "Cảnh báo",
      className: "whitespace-normal text-xs",
      cell: (r) =>
        r.warnings.length ? (
          <Tip text={r.warnings.join("; ").slice(0, 80)}>
            <span>
              <Badge tone="warn">{r.warnings.length}</Badge> <span className="text-warning-700">{r.warnings[0]}</span>
            </span>
          </Tip>
        ) : null,
    },
  ];

  return (
    <section className="view active">
      <PageHeader title="Hệ thống hoá hồ sơ" description="Đánh số hồ sơ chính thức, xuất mục lục tạm, gán hộp, bó, tập. Cần kết nối máy chủ." />
      <ChuaChonDuAn />
      {!canDo && <Banner tone="warning" className="mb-4">Tài khoản chưa có quyền hệ thống hoá.</Banner>}
      <CanMayChu sync={sync} viec="Hệ thống hoá thay đổi số hồ sơ của cả tập nên chỉ làm khi có máy chủ." />

      <Stepper className="mb-4" current={buoc} steps={[{ key: "xem", label: "Xem trước", hint: "Lọc và chọn thứ tự" }, { key: "ap", label: "Áp dụng đánh số" }, { key: "xuat", label: "Xuất mục lục tạm", hint: `Điền ${tenDonVi} trong Excel` }, { key: "nhap", label: "Nhập tệp đã điền" }]} />

      <Card className="mb-4" title="Tập hồ sơ và thứ tự đánh số">
        <FormGrid cols={4}>
          <Field label="Phông" required>
            <Select
              value={phongId}
              onChange={(v) => {
                setPhongId(v);
                setPhongBanId("");
                setMangId("");
                setPreview(null);
              }}
              rows={cat.phong}
              render={tenPhong}
            />
          </Field>
          <Field label="Phòng ban">
            <Select
              value={phongBanId}
              onChange={(v) => {
                setPhongBanId(v);
                setMangId("");
              }}
              rows={phongBans}
              empty="Cả phông"
            />
          </Field>
          <Field label="Mảng công việc">
            <Select value={mangId} onChange={setMangId} rows={mangs} empty="Cả phòng ban" disabled={!phongBanId} />
          </Field>
          <Field label="Trạng thái THBQ">
            <UiSelect value={status} onChange={setStatus} placeholder="Tất cả" options={[{ value: "VV", label: "Vĩnh viễn" }, { value: "GL", label: "Giữ lại" }, { value: "HET", label: "Hết THBQ" }]} />
          </Field>
          <Field label="Trùng thừa">
            <UiSelect value={trungThua} onChange={(v) => setTrungThua(v as typeof trungThua)} placeholder="Tất cả" options={[{ value: "0", label: "Không trùng thừa" }, { value: "1", label: "Chỉ trùng thừa" }]} />
          </Field>
          <Field label="Thứ tự đánh số">
            <UiSelect value={String(order)} onChange={(v) => setOrder(Number(v))} options={ORDERS.map(([k, l]) => ({ value: String(k), label: `${k}. ${l}` }))} />
          </Field>
          <Field label="Loại tệp">
            <UiSelect value={String(fileType)} onChange={(v) => setFileType(Number(v) as 1 | 2)} options={[{ value: "1", label: "Hộp số / hồ sơ số (hồ sơ chuẩn)" }, { value: "2", label: "Bó số / tập số (hết hạn, trùng thừa)" }]} />
          </Field>
          <Field label="Bắt đầu từ số">
            <Input type="number" min={1} value={startAt} onChange={(e) => setStartAt(Math.max(1, Number(e.target.value) || 1))} />
          </Field>
        </FormGrid>
        <div className="mt-3 flex flex-wrap items-center gap-2">
          <span className="text-sm text-fg-subtle">THBQ:</span>
          {cat.thbq.map((t) => (
            <Chip key={t.id} on={thbqIds.includes(t.id)} onClick={() => setThbqIds((v) => (v.includes(t.id) ? v.filter((x) => x !== t.id) : [...v, t.id]))}>
              {str(t, "ten")}
            </Chip>
          ))}
          <Chip on={chuaDanhSo} onClick={() => setChuaDanhSo((v) => !v)}>
            Chỉ hồ sơ chưa đánh số
          </Chip>
        </div>
        <Toolbar className="mt-4">
          <Button variant={buoc === 0 ? "primary" : "secondary"} icon="xem" loading={busy && !preview} disabled={khoa || busy || !phongId} onClick={() => void xemTruoc()}>
            Xem trước
          </Button>
          <Button variant={buoc === 1 ? "primary" : "secondary"} icon="duyet" disabled={khoa || busy || !preview || preview.rows.length === 0 || applied} onClick={() => void apDung()}>
            Áp dụng đánh số
          </Button>
          <Button variant={buoc === 2 ? "primary" : "secondary"} icon="tepExcel" disabled={!preview || preview.rows.length === 0} onClick={() => void xuat()}>
            Xuất mục lục tạm
          </Button>
          <Button variant={buoc === 3 ? "primary" : "secondary"} icon="nhapTep" disabled={khoa || busy} onClick={() => fileRef.current?.click()}>
            Nhập tệp đã điền {tenDonVi}
          </Button>
          <input
            ref={fileRef}
            type="file"
            accept=".xlsx,.xls"
            hidden
            aria-hidden
            onChange={(e) => {
              const f = e.target.files?.[0];
              e.target.value = "";
              if (f) void nhap(f);
            }}
          />
        </Toolbar>
        {importRes && importRes.errors.length > 0 && (
          <Banner tone="warning" className="mt-3" title={`${importRes.errors.length} dòng lỗi khi nhập`} detail={importRes.errors.map((e) => `Dòng ${e.row}: ${e.message}`).join("\n")} onClose={() => setImportRes(null)}>
            Đã gán {n(importRes.applied)} hồ sơ. Sửa các dòng lỗi trong tệp rồi nhập lại.
          </Banner>
        )}
      </Card>

      {preview && (
        <Card
          padding="none"
          title={`${n(preview.rows.length)} hồ sơ`}
          description={canhBao > 0 ? `${n(canhBao)} hồ sơ có cảnh báo, xem cột cuối.` : undefined}
          actions={
            <>
              {canhBao > 0 && <Checkbox checked={chiCanhBao} onChange={setChiCanhBao} label="Chỉ dòng có cảnh báo" />}
              <Button size="sm" disabled={khoa || busy || sel.size === 0} onClick={() => void ganLe()}>
                Gán {tenDonVi} cho {n(sel.size)} dòng chọn
              </Button>
            </>
          }
        >
          <Table columns={cols} rows={rowsHien} getRowId={(r) => r.hoSoId} dense maxHeight={560} selection={{ selected: sel, onChange: setSel }} rowClassName={(r) => (r.warnings.length ? "bg-warning-50" : undefined)} state={{ empty: { title: chiCanhBao ? "Không có dòng nào có cảnh báo" : "Không có hồ sơ nào khớp bộ lọc" } }} />
        </Card>
      )}
    </section>
  );
}
