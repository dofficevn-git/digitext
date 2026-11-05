// Copyright (c) 2026 DigiText. All rights reserved.
// Bảo quản và sao lưu: kiểm tra toàn vẹn tài liệu (mục tiêu toàn bộ trong 3 năm), danh mục tệp lỗi, sổ sao lưu
// và ghi nhận lần sao lưu. Quá 26 giờ không có sao lưu đạt thì cảnh báo.

import { useCallback, useMemo, useState } from "react";
import { V41t } from "@/live/useLive";
import { V4v5, V4v4, V4v1, V4v2, type GoiLoi, type V4v3, type V4v0, type V4uz } from "@/api/infoops";
import { isTauri } from "@/api/ipc";
import { V460 } from "@/state";
import { V4v8 } from "./infoFlow";
import { Badge, Banner, Button, Card, Dialog, Field, Help, Input, KpiRow, KpiTile, notifyError, PageHeader, Select, Table, Textarea, Tip, toast, type Column } from "@/ui";

const fmtN = (v: number) => v.toLocaleString("vi-VN");
const GIO_CANH_BAO = 26;

function GhiNhanDialog({ onClose, onDone }: { onClose: () => void; onDone: () => void }) {
  const [loai, setLoai] = useState<V4v3["loai"]>("gia_tang");
  const [nguon, setNguon] = useState("CSDL tài liệu lưu trữ + mã nguồn phần mềm");
  const [phuongTien, setPhuongTien] = useState("");
  const [ketQua, setKetQua] = useState<V4v3["ketQua"]>("dat");
  const [chiTiet, setChiTiet] = useState("");
  const [busy, setBusy] = useState(false);
  const [loiNguon, setLoiNguon] = useState<string | undefined>();
  const luu = async () => {
    if (!nguon.trim()) {
      setLoiNguon("Ghi rõ nguồn dữ liệu đã sao lưu.");
      return;
    }
    setBusy(true);
    try {
      await V4v5({ loai, nguon: nguon.trim(), phuongTien: phuongTien.trim() || null, ketQua, chiTiet: chiTiet.trim() || null });
      toast.success("Đã ghi nhận lần sao lưu.", { description: "Nhớ lập nhật ký hoặc biên bản sao lưu ở màn Biên bản." });
      onDone();
      onClose();
    } catch (e) {
      notifyError(e, "ghi nhận sao lưu");
    } finally {
      setBusy(false);
    }
  };
  return (
    <Dialog
      open
      onOpenChange={(o) => !o && !busy && onClose()}
      closable={!busy}
      title="Ghi nhận lần sao lưu"
      size="md"
      footer={
        <>
          <Button onClick={onClose} disabled={busy}>
            Huỷ
          </Button>
          <Button variant="primary" icon="luu" loading={busy} onClick={() => void luu()}>
            Ghi nhận
          </Button>
        </>
      }
    >
      <div className="space-y-3">
        <Field label="Loại sao lưu">
          <Select value={loai} onChange={(v) => setLoai(v as V4v3["loai"])} options={Object.entries(V4v8).map(([value, label]) => ({ value, label }))} />
        </Field>
        <Field label="Nguồn dữ liệu" required error={loiNguon} hint="Cơ sở dữ liệu và mã nguồn phần mềm">
          <Input value={nguon} onChange={(e) => { setNguon(e.target.value); setLoiNguon(undefined); }} />
        </Field>
        <Field label="Phương tiện lưu trữ">
          <Input value={phuongTien} onChange={(e) => setPhuongTien(e.target.value)} placeholder="Ổ cứng ngoài, băng từ, đám mây…" />
        </Field>
        <Field label="Kết quả kiểm tra sau sao lưu">
          <Select value={ketQua} onChange={(v) => setKetQua(v as V4v3["ketQua"])} options={[{ value: "dat", label: "Đạt" }, { value: "khong_dat", label: "Không đạt" }]} />
        </Field>
        <Field label="Chi tiết, biện pháp khắc phục">
          <Textarea rows={2} value={chiTiet} onChange={(e) => setChiTiet(e.target.value)} />
        </Field>
      </div>
    </Dialog>
  );
}

export default function V4vd() {
  const { session } = V460();
  const canManage = (session?.permissions ?? []).includes("bk.manage");
  const loader = useCallback(async () => {
    if (!isTauri()) return { report: null as V4v0 | null, backups: [] as V4v3[] };
    const [report, backups] = await Promise.all([V4v1(), V4v4()]);
    return { report, backups };
  }, []);
  const { data, error, loading, reload } = V41t(loader, { topics: ["ops"], errorText: () => "Không tải được số liệu bảo quản." });
  const report = data?.report ?? null;
  const backups = useMemo(() => data?.backups ?? [], [data]);
  const [dangKiem, setDangKiem] = useState(false);
  const [moGhi, setMoGhi] = useState(false);

  const saoLuuGanNhat = useMemo(() => backups.filter((b) => b.ketQua === "dat").map((b) => new Date(b.createdAt).getTime()).reduce((a, b) => Math.max(a, b), 0), [backups]);
  const gioTuSaoLuu = saoLuuGanNhat ? (Date.now() - saoLuuGanNhat) / 3_600_000 : null;
  const quaHan = data != null && (gioTuSaoLuu === null || gioTuSaoLuu > GIO_CANH_BAO);

  const chayKiem = async () => {
    setDangKiem(true);
    try {
      const r = await V4v2();
      const goi = r.goiDaKiem ? `, ${fmtN(r.goiDaKiem)} gói` : "";
      const loi = r.loi + (r.goiLoi ?? 0);
      if (loi > 0) toast.warning(`Đã kiểm ${fmtN(r.daKiem)} tài liệu${goi}, ${fmtN(loi)} lỗi.`);
      else toast.success(`Đã kiểm ${fmtN(r.daKiem)} tài liệu${goi}, không có lỗi.`);
      reload();
    } catch (e) {
      notifyError(e, "chạy kiểm tra");
    } finally {
      setDangKiem(false);
    }
  };

  const colsLoi: Column<V4uz>[] = [
    { key: "filename", header: "Tên tệp", className: "whitespace-normal" },
    { key: "hoSoRef", header: "Mã hồ sơ", width: 160, cell: (t) => t.hoSoRef ?? "—" },
    { key: "chiTiet", header: "Chi tiết lỗi", className: "whitespace-normal text-danger-700", cell: (t) => t.chiTiet ?? "—" },
    { key: "checkedAt", header: "Thời điểm kiểm", width: 150, cell: (t) => new Date(t.checkedAt).toLocaleString("vi-VN") },
  ];
  const colsGoiLoi: Column<GoiLoi>[] = [
    { key: "label", header: "Gói", className: "whitespace-normal" },
    { key: "objid", header: "OBJID", width: 260, className: "break-all", cell: (g) => g.objid ?? "—" },
    { key: "chiTiet", header: "Chi tiết lỗi", className: "whitespace-normal text-danger-700", cell: (g) => g.chiTiet ?? "—" },
    { key: "checkedAt", header: "Thời điểm kiểm", width: 150, cell: (g) => new Date(g.checkedAt).toLocaleString("vi-VN") },
  ];
  const colsSaoLuu: Column<V4v3>[] = [
    { key: "createdAt", header: "Thời điểm", width: 150, sortable: true, cell: (b) => new Date(b.createdAt).toLocaleString("vi-VN") },
    { key: "loai", header: "Loại", width: 160, cell: (b) => V4v8[b.loai] ?? b.loai },
    { key: "nguon", header: "Nguồn", className: "whitespace-normal" },
    { key: "phuongTien", header: "Phương tiện", cell: (b) => b.phuongTien ?? "—" },
    {
      key: "ketQua",
      header: "Kết quả",
      width: 110,
      cell: (b) => (
        <Tip text={b.chiTiet ? b.chiTiet.slice(0, 80) : "Không có chi tiết"}>
          <span>
            <Badge tone={b.ketQua === "dat" ? "ok" : "err"}>{b.ketQua === "dat" ? "Đạt" : "Không đạt"}</Badge>
          </span>
        </Tip>
      ),
    },
  ];
  const tyLe = report && report.tongTaiLieu > 0 ? Math.round((report.daKiem3Nam / report.tongTaiLieu) * 100) : 0;

  return (
    <section className="view active">
      <PageHeader
        title="Bảo quản và sao lưu"
        description="Kiểm tra toàn vẹn tài liệu định kỳ và ghi sổ các lần sao lưu."
        actions={
          canManage && (
            <>
              <Button variant="primary" icon="kiemTra" loading={dangKiem} onClick={() => void chayKiem()}>
                Chạy kiểm tra một lô
              </Button>
              <Button icon="them" onClick={() => setMoGhi(true)}>
                Ghi nhận sao lưu
              </Button>
            </>
          )
        }
      />
      {error != null && <Banner tone="danger" className="mb-4" onClose={undefined} actions={<Button size="sm" onClick={reload}>Thử lại</Button>}>Không tải được số liệu bảo quản.</Banner>}
      {quaHan && (
        <Banner tone="danger" className="mb-4" title={gioTuSaoLuu === null ? "Chưa có lần sao lưu đạt nào" : `Đã ${Math.floor(gioTuSaoLuu)} giờ không có sao lưu đạt`}>
          Sao lưu ít nhất mỗi ngày. Sau khi sao lưu xong, bấm Ghi nhận sao lưu để ghi sổ.
        </Banner>
      )}

      <Card className="mb-4">
        <KpiRow>
          <KpiTile label="Tài liệu đã kiểm trong 3 năm" value={report?.daKiem3Nam ?? 0} sub={report ? `${tyLe}% của ${fmtN(report.tongTaiLieu)} tài liệu` : undefined} />
          <KpiTile label="Tệp lỗi" value={report?.soLoi ?? 0} tone={report && report.soLoi > 0 ? "err" : "ok"} />
          {report?.tongGoi ? (
            <KpiTile label="Gói lưu trữ đã kiểm" value={report.goiDaKiem ?? 0} sub={`${fmtN(report.tongGoi)} gói zip trong kho${report.goiSoLoi ? ` · ${fmtN(report.goiSoLoi)} lỗi` : ""}`} tone={report.goiSoLoi ? "err" : "ok"} />
          ) : null}
          <KpiTile label="Lần sao lưu đã ghi" value={backups.length} />
          <KpiTile label="Sao lưu đạt gần nhất" value={saoLuuGanNhat ? new Date(saoLuuGanNhat).toLocaleString("vi-VN", { hour: "2-digit", minute: "2-digit", day: "2-digit", month: "2-digit" }) : "—"} tone={quaHan ? "err" : "ok"} />
        </KpiRow>
        <Tip text={`${fmtN(report?.daKiem3Nam ?? 0)} / ${fmtN(report?.tongTaiLieu ?? 0)} tài liệu`}>
          <div className="mt-3 h-2 rounded bg-border-subtle">
            <div className="h-2 rounded bg-accent" style={{ width: `${Math.min(100, tyLe)}%` }} />
          </div>
        </Tip>
        <p className="mt-1 text-sm text-fg-subtle">Mục tiêu kiểm tra toàn bộ tài liệu trong 3 năm. Máy chủ tự kiểm một lô mỗi ngày.</p>
      </Card>

      {report && report.tepLoi.length > 0 && (
        <Card className="mb-4" tone="danger" padding="none" title={`Tệp lỗi (${fmtN(report.tepLoi.length)})`} description="Dùng danh mục này khi lập biên bản kiểm tra.">
          <Table columns={colsLoi} rows={report.tepLoi} getRowId={(t) => t.documentId} dense maxHeight={320} />
        </Card>
      )}

      {report?.goiLoi && report.goiLoi.length > 0 && (
        <Card className="mb-4" tone="danger" padding="none" title={`Gói lỗi (${fmtN(report.goiLoi.length)})`} description="Zip gói trong kho không còn khớp mã băm SHA-256 lúc nhập hoặc dựng — khôi phục từ bản sao lưu.">
          <Table columns={colsGoiLoi} rows={report.goiLoi} getRowId={(g) => g.packageId} dense maxHeight={320} />
        </Card>
      )}

      <Card padding="none" title="Sổ sao lưu">
        <Table columns={colsSaoLuu} rows={backups} getRowId={(b) => b.id} dense maxHeight={480} state={{ loading: loading && !data, empty: { title: "Chưa có lần sao lưu nào", description: "Quá 26 giờ không có sao lưu đạt, hệ thống cảnh báo quản trị.", icon: "baoquan" } }} />
      </Card>

      <Help title="Căn cứ" className="mt-4">
        <p>Kiểm tra tài liệu: Điều 29 Thông tư 05/2025/TT-BNV, mục tiêu toàn bộ trong 3 năm, danh mục tệp lỗi theo Điều 29.4.đ. Tài liệu nhập từ gói AIP còn được so mã băm BLAKE3 với lúc nhập; zip gói lưu trữ so mã băm SHA-256. Sao lưu: Điều 30, ghi loại, nguồn, phương tiện, kết quả kiểm tra sau sao lưu và nơi bảo quản.</p>
      </Help>

      {moGhi && <GhiNhanDialog onClose={() => setMoGhi(false)} onDone={reload} />}
    </section>
  );
}
