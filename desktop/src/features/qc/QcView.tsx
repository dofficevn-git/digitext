// Copyright (c) 2026 DigiText. All rights reserved.
// Kiểm tra sản phẩm (Bước 5 Điều 9, Điều 8 TT 05/2025, 3/11/2026): đối soát mục lục hồ sơ của một phông với tài liệu đã
// lên máy chủ — từng hồ sơ: số tờ kê khai ↔ số trang quét, số tệp, PDF/A-2b, ký số, bị từ chối; tệp không ghép được hồ sơ;
// danh sách quét lại (dưới 200 dpi) từ worker PDF/A. Kết luận do máy chủ tính (`GET /cl/doi-soat`). Lập biên bản kiểm tra
// (kind kiem_tra_san_pham) ngay trong màn với số liệu điền sẵn; chỉ cảnh báo, không chặn bước sau.

import { useCallback, useMemo, useState } from "react";
import { V41t } from "@/live/useLive";
import { isTauri } from "@/api/ipc";
import { V460 } from "@/state";
import { useProject } from "@/state/project";
import { clCatalogList, clDoiSoat, str, type ClRow, type DoiSoatHoSo, type DoiSoatKetLuan, type DoiSoatResp } from "@/api/chinhly";
import { docsPdfaRejected, type PdfaRejectedRow } from "@/api/docs";
import { V4e3 } from "@/features/dossier/excel";
import { SoBienBan, type NguonDien, type NoiDungDien } from "@/features/records/RecordsView";
import { Badge, Button, Card, Checkbox, EmptyState, Help, KpiRow, KpiTile, PageHeader, Select, Table, type Column, type KpiTone } from "@/ui";

const n = (v: number) => v.toLocaleString("vi-VN");
const KET_LUAN: Record<DoiSoatKetLuan, { label: string; tone: "ok" | "warn" | "err" | "info" | "gray" }> = {
  dat: { label: "Đạt", tone: "ok" },
  chua_quet: { label: "Chưa quét", tone: "gray" },
  thieu_trang: { label: "Thiếu trang", tone: "err" },
  thua_trang: { label: "Nghi thừa trang", tone: "warn" },
  khong_pdfa: { label: "Chưa PDF/A", tone: "warn" },
  pdfa_cu: { label: "PDF/A cũ (không phải 2b)", tone: "warn" },
  bi_tu_choi: { label: "Có tệp bị từ chối", tone: "err" },
};
const LOI_ANH: DoiSoatKetLuan[] = ["khong_pdfa", "pdfa_cu", "bi_tu_choi"];
const LOI_BIEN_MUC: DoiSoatKetLuan[] = ["thieu_trang", "thua_trang"];
const hienSoTo = (h: DoiSoatHoSo) => (h.soTo == null ? "—" : n(h.soTo));

export default function QcView() {
  const { session } = V460();
  const proj = useProject();
  const projectId = session?.projectId ?? proj.projectId ?? "";
  // Danh mục phông: tải thẳng (không qua useClCatalog) để DEV xem trước được với mock.
  const taiPhong = useCallback(async () => (await clCatalogList()).phong, []);
  const catalog = V41t(taiPhong, { topics: ["chinhLy", "catalog"], enabled: isTauri() || import.meta.env.DEV, errorText: () => "Không tải được danh mục phông." });
  const phongList = useMemo<ClRow[]>(() => catalog.data ?? [], [catalog.data]);
  const phongOptions = useMemo(() => phongList.map((p) => ({ value: p.id, label: `${str(p, "ma_du_an") || "—"} · ${str(p, "ten")}` })), [phongList]);
  const [phongId, setPhongId] = useState("");
  const [thuMuc, setThuMuc] = useState("");
  const [chiLech, setChiLech] = useState(false);
  const phongIdEff = phongId || phongOptions[0]?.value || "";
  const phongChon = useMemo(() => phongList.find((p) => p.id === phongIdEff) ?? null, [phongList, phongIdEff]);

  const loader = useCallback(async (): Promise<DoiSoatResp | null> => {
    if (!projectId || !phongIdEff) return null;
    return clDoiSoat(projectId, phongIdEff, thuMuc || undefined);
  }, [projectId, phongIdEff, thuMuc]);
  const { data, error, loading, reload } = V41t(loader, { topics: ["docs", "chinhLy"], pollMs: 180_000, enabled: (isTauri() || import.meta.env.DEV) && !!projectId && !!phongIdEff, errorText: () => "Không đối soát được." });
  const taiQuetLai = useCallback(async () => (isTauri() || import.meta.env.DEV ? docsPdfaRejected() : []), []);
  const quetLai = V41t(taiQuetLai, { topics: ["docs"], pollMs: 300_000, enabled: (isTauri() || import.meta.env.DEV) && !!projectId, errorText: () => "Không tải được danh sách quét lại." });
  const quetLaiRows = useMemo<PdfaRejectedRow[]>(() => (quetLai.data ?? []).filter((r) => !thuMuc || (r.relPath ?? "").split("/")[0]?.trim().toLowerCase() === thuMuc.toLowerCase()), [quetLai.data, thuMuc]);

  const tk = data?.tongKet;
  const rows = useMemo(() => (data?.hoSo ?? []).filter((h) => !chiLech || h.ketLuan !== "dat"), [data, chiLech]);
  const thuMucOptions = useMemo(() => (data?.thuMucCo ?? []).map((t) => ({ value: t, label: t })), [data]);
  const tyLeDat = tk && tk.hoSoMucLuc > 0 ? Math.round((tk.dat / tk.hoSoMucLuc) * 100) : 0;
  const toneDat: KpiTone = tyLeDat >= 95 ? "ok" : tyLeDat >= 70 ? "warn" : "err";

  const cols: Column<DoiSoatHoSo>[] = [
    { key: "hopSo", header: "Hộp", width: 70, sortable: true },
    { key: "hoSoSo", header: "Hồ sơ", width: 80, sortable: true },
    { key: "maHoSo", header: "Mã hồ sơ", width: 170, cell: (h) => h.maHoSo ?? "" },
    { key: "tieuDe", header: "Tiêu đề", cell: (h) => <span className="line-clamp-2">{h.tieuDe}</span> },
    { key: "soTo", header: "Số tờ (MLHS)", align: "right", width: 100, sortable: true, cell: hienSoTo },
    { key: "pages", header: "Trang quét", align: "right", width: 100, sortable: true, cell: (h) => n(h.pages) },
    { key: "files", header: "Tệp", align: "right", width: 60, sortable: true, cell: (h) => n(h.files) },
    { key: "pdfaDocs", header: "PDF/A", align: "right", width: 70, cell: (h) => (h.files ? `${n(h.pdfaDocs)}/${n(h.files)}` : "") },
    { key: "signedDocs", header: "Ký số", align: "right", width: 70, cell: (h) => (h.signedDocs ? n(h.signedDocs) : "") },
    { key: "ketLuan", header: "Kết luận", width: 170, sortable: true, cell: (h) => <Badge tone={KET_LUAN[h.ketLuan].tone}>{KET_LUAN[h.ketLuan].label}</Badge> },
  ];

  const xuatExcel = () => {
    if (!data) return;
    const ten = phongChon ? str(phongChon, "ma_du_an") || "phong" : "phong";
    void V4e3(
      data.hoSo.map((h) => ({ Hộp: h.hopSo, "Hồ sơ": h.hoSoSo, "Mã hồ sơ": h.maHoSo ?? "", "Tiêu đề": h.tieuDe, "Số tờ MLHS": h.soTo ?? "", "Trang quét": h.pages, Tệp: h.files, "PDF/A": h.pdfaDocs, "PDF/A cũ": h.pdfaCu, "Ký số": h.signedDocs, "Bị từ chối": h.rejectedDocs, "Kết luận": KET_LUAN[h.ketLuan].label })),
      "Đối soát",
      `kiem-tra-san-pham-${ten}.xlsx`,
    );
  };

  const nguonDien: NguonDien = useMemo(
    () => ({
      label: "Phông đã đối soát",
      options: phongChon ? [{ value: phongChon.id, label: `${str(phongChon, "ma_du_an") || "—"} · ${str(phongChon, "ten")}` }] : phongOptions.slice(0, 1),
      dien: async (): Promise<NoiDungDien> => {
        if (!data || !tk) return {};
        const loi = data.hoSo.filter((h) => h.ketLuan !== "dat" && h.ketLuan !== "chua_quet");
        const loiAnh = loi.filter((h) => LOI_ANH.includes(h.ketLuan)).length + quetLaiRows.length;
        const loiBienMuc = loi.filter((h) => LOI_BIEN_MUC.includes(h.ketLuan)).length + tk.tepKhongKhop;
        const p = phongChon;
        return {
          fields: {
            duAn: proj.projectLabel || projectId,
            maPhong: p ? str(p, "ma_du_an") : "",
            tenPhong: p ? str(p, "ten") : "",
            tongHoSo: String(tk.hoSoMucLuc),
            hoSoDaQuet: String(tk.hoSoCoTep),
            tongTep: String(tk.tep),
            tongTrang: String(tk.trang),
            soToMucLuc: String(tk.soToMucLuc),
            hoSoDat: String(tk.dat),
            loiAnh: String(loiAnh),
            loiBienMuc: String(loiBienMuc),
            tepKhongKhop: String(tk.tepKhongKhop),
            tepQuetLai: String(quetLaiRows.length),
            ketLuan: tk.dat === tk.hoSoMucLuc ? "Đạt yêu cầu Điều 8 TT 05/2025" : `Còn ${n(tk.hoSoMucLuc - tk.dat)} hồ sơ phải xử lý trước khi ký số, đóng gói`,
          },
          lists: {
            loi: loi.slice(0, 50).map((h) => ({ hop: h.hopSo, hoSo: h.hoSoSo, tieuDe: h.tieuDe, soTo: h.soTo == null ? "" : String(h.soTo), trang: String(h.pages), loi: KET_LUAN[h.ketLuan].label })),
            quetLai: quetLaiRows.slice(0, 50).map((r) => ({ tep: r.filename, lyDo: r.reason })),
          },
        };
      },
    }),
    [data, tk, phongChon, phongOptions, proj.projectLabel, projectId, quetLaiRows],
  );

  if (!projectId) {
    return (
      <section className="view active">
        <PageHeader title="Kiểm tra sản phẩm" description="Bước 5 Điều 9 TT 05/2025: đối soát mục lục với tài liệu đã số hoá, kiểm Điều 8." />
        <Card padding="md">
          <EmptyState icon="duAn" title="Chọn dự án đang làm ở thanh trên" description="Kiểm tra theo từng phông của dự án." />
        </Card>
      </section>
    );
  }
  return (
    <section className="view active">
      <PageHeader
        title="Kiểm tra sản phẩm"
        description={`Bước 5 Điều 9 TT 05/2025: từng hồ sơ trong mục lục đối chiếu với tài liệu đã lên máy chủ — số tờ kê khai so với trang quét, PDF/A-2b, ký số, tệp bị từ chối. Dự án ${proj.projectLabel || ""}.`}
        actions={
          <>
            <Select size="sm" value={phongIdEff} onChange={(v) => { setPhongId(v); setThuMuc(""); }} options={phongOptions} placeholder="Chọn phông" aria-label="Phông" />
            {thuMucOptions.length > 1 && <Select size="sm" value={thuMuc} onChange={setThuMuc} options={thuMucOptions} placeholder="Mọi thư mục trên máy chủ" aria-label="Thư mục phông trên máy chủ" />}
            <Button icon="tepExcel" onClick={xuatExcel} disabled={!data || data.hoSo.length === 0}>
              Xuất Excel
            </Button>
          </>
        }
      />
      <Help title="Cách đọc kết luận" className="mb-3">
        <p className="text-sm">
          <b>Thiếu trang</b>: trang quét ít hơn số tờ kê khai trong mục lục. <b>Nghi thừa trang</b>: quét nhiều hơn gấp đôi số tờ (tờ hai mặt thì trang bằng hai lần tờ là bình thường). <b>Chưa PDF/A</b> / <b>PDF/A cũ</b>: chưa đạt Điều 8 (PDF/A-2b). Dự án có nhiều phông cùng số hộp thì chọn thêm thư mục phông trên máy chủ. Đây là cảnh báo để xử lý, không chặn ký số hay đóng gói.
        </p>
      </Help>
      {phongOptions.length === 0 && !catalog.loading ? (
        <Card padding="md">
          <EmptyState icon="hoSo" title="Dự án chưa có phông nào trong Dữ liệu chủ" description="Nhập mục lục hồ sơ (Dữ liệu chủ › Lập hồ sơ) trước khi kiểm tra." />
        </Card>
      ) : (
        <>
          <KpiRow className="mb-3">
            <KpiTile label="Hồ sơ trong mục lục" value={tk?.hoSoMucLuc ?? 0} loading={loading && !data} />
            <KpiTile label="Đã có tệp quét" value={tk?.hoSoCoTep ?? 0} sub={tk ? `${n(tk.tep)} tệp · ${n(tk.trang)} trang` : undefined} loading={loading && !data} />
            <KpiTile label="Đạt" value={tk?.dat ?? 0} sub={tk ? `${tyLeDat}% mục lục` : undefined} tone={tk ? toneDat : undefined} loading={loading && !data} />
            <KpiTile label="Lệch số tờ" value={tk?.lechSoTo ?? 0} tone={tk?.lechSoTo ? "err" : undefined} loading={loading && !data} />
            <KpiTile label="Chưa PDF/A-2b" value={(tk?.khongPdfa ?? 0) + (tk?.pdfaCu ?? 0)} tone={tk && tk.khongPdfa + tk.pdfaCu > 0 ? "warn" : undefined} loading={loading && !data} />
            <KpiTile label="Tệp không khớp mục lục" value={tk?.tepKhongKhop ?? 0} tone={tk?.tepKhongKhop ? "warn" : undefined} loading={loading && !data} />
          </KpiRow>
          <Card title="Đối soát từng hồ sơ" padding="none" actions={<Checkbox checked={chiLech} onChange={setChiLech} text="Chỉ hồ sơ chưa đạt" />}>
            <Table columns={cols} rows={rows} getRowId={(h) => h.hoSoId} dense maxHeight={420} rowClassName={(h) => (h.ketLuan === "thieu_trang" || h.ketLuan === "bi_tu_choi" ? "bg-danger-50/40" : undefined)} state={{ loading: loading && !data, error: data ? undefined : error, onRetry: reload, empty: { title: chiLech ? "Mọi hồ sơ đều đạt" : "Phông chưa có hồ sơ" } }} />
          </Card>
          <div className="mt-3 grid gap-3 lg:grid-cols-2">
            <Card title={`Tệp trên máy chủ không khớp mục lục${data ? ` · ${n(data.khongKhop.length)}` : ""}`} padding="none">
              <Table
                columns={[
                  { key: "filename", header: "Tệp", cell: (r: DoiSoatResp["khongKhop"][number]) => <span className="font-mono text-xs">{r.filename}</span> },
                  { key: "hop", header: "Hộp", width: 70 },
                  { key: "hoSo", header: "Hồ sơ", width: 80 },
                  { key: "lyDo", header: "Lý do" },
                ]}
                rows={data?.khongKhop ?? []}
                getRowId={(r) => r.documentId}
                dense
                maxHeight={260}
                state={{ loading: loading && !data, empty: { title: "Mọi tệp đều ghép được hồ sơ" } }}
              />
            </Card>
            <Card title={`Quét lại (dưới 200 dpi, Điều 8)${quetLai.data ? ` · ${n(quetLaiRows.length)}` : ""}`} padding="none">
              <Table
                columns={[
                  { key: "filename", header: "Tệp", cell: (r: PdfaRejectedRow) => <span className="font-mono text-xs">{r.filename}</span> },
                  { key: "relPath", header: "Thư mục", cell: (r: PdfaRejectedRow) => r.relPath ?? "" },
                  { key: "reason", header: "Lý do" },
                  { key: "outcome", header: "", width: 110, cell: (r: PdfaRejectedRow) => (r.outcome === "exception" ? <Badge tone="warn">Ngoại lệ</Badge> : <Badge tone="err">Chưa PDF/A</Badge>) },
                ]}
                rows={quetLaiRows}
                getRowId={(r) => r.id}
                dense
                maxHeight={260}
                state={{ loading: quetLai.loading && !quetLai.data, error: quetLai.data ? undefined : quetLai.error, onRetry: quetLai.reload, empty: { title: "Không có tệp phải quét lại" } }}
              />
            </Card>
          </div>
          <div className="mt-3">
            <SoBienBan kinds={["kiem_tra_san_pham"]} projectId={projectId} title="Biên bản kiểm tra sản phẩm" description="Lập sau mỗi đợt kiểm tra một phông; số liệu điền sẵn từ bảng đối soát trên." nguonDien={nguonDien} nutLap="Lập biên bản kiểm tra…" embedded />
          </div>
        </>
      )}
    </section>
  );
}
