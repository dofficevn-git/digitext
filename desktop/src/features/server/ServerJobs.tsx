// Copyright (c) 2026 DigiText. All rights reserved.
// Tab "Chạy xử lý" của OCR & bóc tách: hai việc trên máy chủ (ghép PDF 2 lớp, bóc tách 16 trường), thu hẹp
// phạm vi theo phông / phòng ban / mảng / THBQ, thử lại phần lỗi.

import { useCallback, useEffect, useMemo, useState } from "react";
import { V41p } from "@/live/useLive";
import { isTauri } from "@/api/ipc";
import { V4ex, V4ev, V4ez, docsForGroups, docsOcrRetryFailed, docsPdfaRejected, docsReviewSummary, type V4ey, type ReviewSummaryRow } from "@/api/docs";
import { V4e5 } from "@/features/dossier/excel";
import { listHoSo, V40w, type V40d, type V40v } from "@/api/archive";
import { V48j } from "@/api/catalog";
import { V4br, type V4bm } from "@/api/dossier";
import { V4cm, V4cl } from "@/features/dossier/logic";
import type { V3ym, V3yl, V3yo } from "@/domain/types";
import { V4j9, V4j4, V4j0, V4j1, V4j3, type V4iz } from "@/features/upload/khoFilter";
import { V4it } from "@/features/upload/syncMatrix";
import { V4jj, locHoSoGop, tienDoGop, type V4jt } from "./jobScope";
import { V4k2, V4k0, V4jz } from "./progress";
import "./server.css";
import { Button, Card, Checkbox, Field, FormGrid, Help, notifyError, Skeleton, toast } from "@/ui";

const fmtN = (n: number) => n.toLocaleString("vi-VN");

function V4k4(p: V4ey): V4jt {
  return {
    docs: p.docs,
    docsOcrPending: p.docsOcrPending,
    docsOcrDone: p.docsOcrDone,
    docsPdf2Done: p.docsPdf2Done,
    docsPdf2Rejected: p.docsPdf2Rejected ?? 0,
    docsPdf2RejectedDpi: p.docsPdf2RejectedDpi ?? 0,
    docsPdf2RejectedNoText: p.docsPdf2RejectedNoText ?? 0,
    docsPdf2RejectedOther: p.docsPdf2RejectedOther ?? 0,
    docsFieldsDone: p.docsFieldsDone ?? 0,
    docsFieldsActive: p.docsFieldsActive ?? 0,
    docsFieldsFailed: p.docsFieldsFailed ?? 0,
    pagesOcrFailed: p.pagesOcrFailed ?? 0,
    docsOcrFailed: p.docsOcrFailed ?? 0,
    pagesInkMeasured: p.pagesInkMeasured ?? 0,
    pagesInkLow: p.pagesInkLow ?? 0,
  };
}

function ProgressButton({ label, sub, percent, kind, busy, disabled, onClick }: { label: string; sub: string; percent: number; kind: "pdf2" | "fields"; busy: boolean; disabled: boolean; onClick: () => void }) {
  return (
    <button type="button" className="srv-progbtn" onClick={onClick} disabled={disabled} aria-busy={busy || undefined} aria-label={`${label}: ${sub}`}>
      <span className={`srv-fill ${kind}`} style={{ width: `${percent}%` }} aria-hidden />
      <span className="srv-label">
        <b>{label}</b>
        <span className="srv-sub">{busy ? "Đang gửi…" : sub}</span>
      </span>
    </button>
  );
}

export default function V4k5() {
  const [progress, V1nz] = useState<V4ey | null>(null);
  const [V1o0, V1o1] = useState<"" | "pdf2" | "fields" | "ocrretry">("");
  const [keCaHong, setKeCaHong] = useState(false);
  const [moPhamVi, setMoPhamVi] = useState(false);

  const [flt, V1o4] = useState<V4iz>(V4j0);
  const [V1o5, V1o6] = useState<V40v[]>([]);
  const [V1o7, V1o8] = useState<V40d[]>([]);
  const [V1o9, V1oa] = useState<V3yl[]>([]);
  const [V1ob, V1oc] = useState<V3ym[]>([]);
  const [V1od, V1oe] = useState<V3yo[]>([]);
  const [V1of, V1og] = useState<V4bm[]>([]);

  const [tree, V1oh] = useState<ReviewSummaryRow[] | null>(null);
  const [V1oi, V1oj] = useState(false);

  const V1ok = V4j1(flt);

  const V1ol = useCallback(async () => {
    if (!isTauri()) return;
    try {
      V1nz(await V4ez());
    } catch (e) {
      notifyError(e, "đọc tiến độ");
    }
  }, []);

  const V1on = useCallback(async () => {
    if (!isTauri()) return;
    V1oj(true);
    try {
      V1oh(await docsReviewSummary({}));
    } catch (e) {
      notifyError(e, "đọc kho máy chủ");
    } finally {
      V1oj(false);
    }
  }, []);

  V41p(V1ol, { topics: ["docs", "ocr", "ocrFields"], enabled: !V1ok });
  V41p(V1on, { topics: ["docs", "ocr", "ocrFields"], enabled: V1ok });

  useEffect(() => {
    if (!isTauri()) return;
    void (async () => {
      try {
        const [V1op, V1oq] = await Promise.all([V40w(), listHoSo()]);
        V1o6(V1op);
        V1o8(V1oq);
      } catch {
        V1o6([]);
        V1o8([]);
      }
      try {
        const [cat, V1or] = await Promise.all([V48j(), V4br()]);
        V1oa(cat.phongBans);
        V1oc(cat.mangs);
        V1oe(cat.thbqs);
        V1og(V1or);
      } catch {
        V1oa([]);
        V1oc([]);
        V1oe([]);
        V1og([]);
      }
    })();
  }, []);

  const V1os = useMemo(() => V4jj(V1o5), [V1o5]);
  const V1ot = useMemo(() => V4it(V1o7), [V1o7]);
  const V1ou = useMemo(() => V4j9(V1of, V1o7, V1od), [V1of, V1o7, V1od]);
  const V1ov = useMemo(() => locHoSoGop(tree ?? [], flt, V1os, V1ot, V1ou), [tree, flt, V1os, V1ot, V1ou]);

  const view = useMemo<V4jt | null>(() => {
    if (V1ok) return tree === null ? null : tienDoGop(V1ov.rows);
    return progress === null ? null : V4k4(progress);
  }, [V1ok, tree, V1ov, progress]);

  // Danh sách id tài liệu trong phạm vi lọc — chỉ nạp khi bấm chạy (theo hồ sơ, ≤ 500 hồ sơ mỗi lượt).
  const V1ow = async (): Promise<string[] | undefined> => {
    if (!V1ok) return undefined;
    const docs = await docsForGroups(V1ov.rows.map((r) => ({ projectId: r.projectId, phong: r.phong, hop: r.hop, hoSo: r.hoSo })));
    return docs.filter((d) => d.status !== "disposed").map((d) => d.id);
  };
  const V1ox = () => (V1ok ? ` Phạm vi đang lọc: ${fmtN(V1ov.rows.reduce((a, r) => a + r.files, 0))} tài liệu.` : "");

  const reload = async () => {
    if (V1ok) await V1on();
    else await V1ol();
  };

  // Báo cáo Excel các tài liệu máy chủ từ chối ghép PDF/A: cây thư mục, tên tệp, dpi, lý do, cách xử lý.
  const [xuatLoi, setXuatLoi] = useState(false);
  const xuatLoiPdfa = async () => {
    setXuatLoi(true);
    try {
      const rows = await docsPdfaRejected();
      if (rows.length === 0) {
        toast.info("Không có tài liệu nào bị từ chối ghép PDF/A.");
        return;
      }
      const dpiOf = (r: string) => {
        const m = /độ phân giải ([0-9.]+) dpi/.exec(r);
        return m ? Number(m[1]) : "";
      };
      const cachXuLy = (r: string) =>
        /độ phân giải/.test(r) ? "Quét lại ≥ 200 dpi (300 dpi với bản đồ, bản vẽ) rồi đưa lên thay thế" : "Kiểm tra bản quét: OCR không nhận ra chữ";
      const aoa: (string | number)[][] = [
        ["STT", "Dự án", "Phông / Hộp / Hồ sơ (cây máy chủ)", "Tên tệp", "Số trang", "Dung lượng (KB)", "Đưa lên lúc", "Độ phân giải (dpi)", "Kết quả", "Lý do máy chủ ghi nhận", "Cách xử lý"],
        ...rows.map((r, i) => [
          i + 1,
          r.projectCode,
          (r.relPath ?? "").split("/").join(" / "),
          r.filename,
          r.pageCount ?? 0,
          Math.round(r.sizeBytes / 1024),
          new Date(r.uploadedAt).toLocaleString("vi-VN"),
          dpiOf(r.reason),
          r.outcome === "exception" ? "Đã ghép PDF/A theo ngoại lệ dự án" : "Chưa ghép (bị từ chối)",
          r.reason,
          cachXuLy(r.reason),
        ]),
      ];
      const tongHop = new Map<string, number>();
      for (const r of rows) {
        const d = dpiOf(r.reason);
        const k = `${r.projectCode}\t${d === "" ? "OCR không ra chữ" : `${d} dpi`}`;
        tongHop.set(k, (tongHop.get(k) ?? 0) + 1);
      }
      const tong: (string | number)[][] = [["Dự án", "Nguyên nhân", "Số tài liệu"], ...[...tongHop.entries()].sort().map(([k, v]) => [...k.split("\t"), v]), ["", "Tổng", rows.length]];
      const ngay = new Date().toISOString().slice(0, 10);
      await V4e5([{ name: "PDF-A không đạt", aoa }, { name: "Tổng hợp", aoa: tong }], `pdfa-khong-dat-${ngay}.xlsx`);
      toast.success(`Đã xuất ${fmtN(rows.length)} tài liệu bị từ chối ghép PDF/A.`);
    } catch (e) {
      notifyError(e, "xuất báo cáo PDF/A lỗi");
    } finally {
      setXuatLoi(false);
    }
  };
  // "8 không ghép được PDF/A (6 OCR không ra chữ, 2 dưới 200 dpi)" — nói đúng lý do, không gán hết cho dpi.
  const lyDoTuChoi = (v: V4jt): string => {
    const parts: string[] = [];
    if (v.docsPdf2RejectedNoText > 0) parts.push(`${fmtN(v.docsPdf2RejectedNoText)} OCR không ra chữ`);
    if (v.docsPdf2RejectedDpi > 0) parts.push(`${fmtN(v.docsPdf2RejectedDpi)} dưới 200 dpi`);
    if (v.docsPdf2RejectedOther > 0) parts.push(`${fmtN(v.docsPdf2RejectedOther)} lý do khác`);
    return parts.length ? ` (${parts.join(", ")})` : "";
  };
  const chayPdf2 = async () => {
    V1o1("pdf2");
    try {
      const r = await V4ev(undefined, await V1ow());
      const bo = (r.skippedRejected ?? 0) > 0 ? ` Bỏ qua ${fmtN(r.skippedRejected ?? 0)} tài liệu bản quét dưới 200 dpi, cần quét lại.` : "";
      if (r.documents > 0) toast.success(`Đã đưa ${fmtN(r.documents)} tài liệu vào hàng đợi ghép PDF 2 lớp.${bo}${V1ox()}`);
      else toast.info(`Không có tài liệu nào sẵn sàng ghép PDF 2 lớp, cần OCR xong trước.${bo}${V1ox()}`);
      await reload();
    } catch (e) {
      notifyError(e, "ghép PDF 2 lớp");
    } finally {
      V1o1("");
    }
  };

  const chayBocTach = async (thuLaiHong = false) => {
    V1o1("fields");
    try {
      const r = await V4ex(undefined, thuLaiHong, await V1ow());
      toast.success(V4k2(r) + V1ox());
      await reload();
    } catch (e) {
      notifyError(e, "bóc tách 16 trường");
    } finally {
      V1o1("");
    }
  };

  const thuLaiOcr = async () => {
    V1o1("ocrretry");
    try {
      const r = await docsOcrRetryFailed(undefined, await V1ow(), keCaHong);
      const boQua = r.skippedSigned > 0 ? ` Bỏ qua ${fmtN(r.skippedSigned)} tài liệu đã sang bước ký.` : "";
      if (r.pages > 0) toast.success(`Đã xếp hàng lại ${fmtN(r.pages)} trang OCR lỗi của ${fmtN(r.documents)} tài liệu.${boQua}${V1ox()}`);
      else toast.info(`Không có trang OCR lỗi nào trong phạm vi.${boQua}`);
      await reload();
    } catch (e) {
      notifyError(e, "thử lại OCR");
    } finally {
      V1o1("");
    }
  };

  const pctPdf2 = view ? V4jz(view) : 0;
  const pctFields = view ? V4k0(view) : 0;
  const chuaCo = V1ok ? tree === null : progress === null;
  const busy = V1o0 !== "";

  const dsPhongBan = useMemo(() => V4cl(V1o9, flt.maDa), [V1o9, flt.maDa]);
  const dsMang = useMemo(() => V4cm(V1ob, flt.maDv), [V1ob, flt.maDv]);
  const soLoc = [flt.maDa, flt.maDv, flt.mangCvid, flt.thbq].filter(Boolean).length;

  return (
    <div className="space-y-4">
      <Card
        title="Chạy xử lý trên máy chủ"
        description="Mỗi việc chạy độc lập. PDF 2 lớp cần OCR xong; bóc tách 16 trường chạy thẳng trên bản quét."
        actions={
          <>
            <Button size="sm" variant="ghost" icon="tim" onClick={() => setMoPhamVi((v) => !v)} aria-expanded={moPhamVi}>
              {soLoc > 0 ? `Phạm vi (${soLoc})` : "Phạm vi"}
            </Button>
            <Button size="sm" icon="lamMoi" onClick={() => void reload()} disabled={busy}>
              Làm mới
            </Button>
          </>
        }
      >
        {moPhamVi && (
          <FormGrid cols={4} className="mb-4">
            <Field label="Phông">
              <select className="inp" value={flt.maDa} onChange={(e) => V1o4((f) => ({ ...f, maDa: e.target.value, maDv: "", mangCvid: "" }))}>
                <option value="">Tất cả phông</option>
                {V1o5.map((p) => (
                  <option key={p.maDuAn} value={p.maDuAn}>
                    {p.tenDuAn || p.maDuAn}
                  </option>
                ))}
              </select>
            </Field>
            <Field label="Phòng ban">
              <select className="inp" value={flt.maDv} disabled={!flt.maDa || dsPhongBan.length === 0} onChange={(e) => V1o4((f) => ({ ...f, maDv: e.target.value, mangCvid: "" }))}>
                <option value="">{V4j4("phongBan", !!flt.maDa, dsPhongBan.length)}</option>
                {dsPhongBan.map((d) => (
                  <option key={d.maDv} value={d.maDv}>
                    {d.tenDonVi || d.maDonVi || d.maDv}
                  </option>
                ))}
              </select>
            </Field>
            <Field label="Mảng công việc">
              <select className="inp" value={flt.mangCvid} disabled={!flt.maDv || dsMang.length === 0} onChange={(e) => V1o4((f) => ({ ...f, mangCvid: e.target.value }))}>
                <option value="">{V4j4("mang", !!flt.maDv, dsMang.length)}</option>
                {dsMang.map((m) => (
                  <option key={m.mangCvid} value={String(m.mangCvid)}>
                    {m.tenMang || m.maMang}
                  </option>
                ))}
              </select>
            </Field>
            <Field label="Thời hạn bảo quản">
              <select className="inp" value={flt.thbq} disabled={V1od.length === 0} onChange={(e) => V1o4((f) => ({ ...f, thbq: e.target.value }))}>
                <option value="">{V1od.length ? "Tất cả" : "Danh mục chưa có THBQ"}</option>
                {V1od.map((t) => (
                  <option key={t.thbqid} value={String(t.thbqid)}>
                    {t.tenThbq}
                  </option>
                ))}
              </select>
            </Field>
          </FormGrid>
        )}

        {chuaCo ? (
          <Skeleton.Rows rows={2} />
        ) : (
          <div className="srv-actions">
            <ProgressButton
              label="PDF 2 lớp"
              kind="pdf2"
              percent={pctPdf2}
              busy={V1o0 === "pdf2"}
              disabled={busy || chuaCo}
              onClick={() => void chayPdf2()}
              sub={view ? `${fmtN(view.docsPdf2Done)}/${fmtN(view.docsOcrDone)} tài liệu (${pctPdf2}%)${view.docsPdf2Rejected > 0 ? ` · ${fmtN(view.docsPdf2Rejected)} không ghép được` : ""}` : "…"}
            />
            <ProgressButton
              label="Bóc tách 16 trường"
              kind="fields"
              percent={pctFields}
              busy={V1o0 === "fields"}
              disabled={busy || chuaCo}
              onClick={() => void chayBocTach()}
              sub={view ? `${fmtN(view.docsFieldsDone)}/${fmtN(view.docs)} tài liệu (${pctFields}%)` : "…"}
            />
            <div className="flex flex-col gap-2 justify-center">
              {(view?.docsPdf2Rejected ?? 0) > 0 && (
                <Button size="sm" icon="tepExcel" loading={xuatLoi} disabled={busy} onClick={() => void xuatLoiPdfa()}>
                  Xuất Excel {fmtN(view?.docsPdf2Rejected ?? 0)} tài liệu không ghép được PDF/A
                </Button>
              )}
              {(view?.docsFieldsFailed ?? 0) > 0 && (
                <Button size="sm" icon="lamMoi" disabled={busy} onClick={() => void chayBocTach(true)}>
                  Bóc lại {fmtN(view?.docsFieldsFailed ?? 0)} tài liệu hỏng
                </Button>
              )}
              {(view?.pagesOcrFailed ?? 0) > 0 && (
                <div className="flex flex-wrap items-center gap-3">
                  <Button size="sm" icon="lamMoi" loading={V1o0 === "ocrretry"} disabled={busy} onClick={() => void thuLaiOcr()}>
                    Thử lại {fmtN(view?.pagesOcrFailed ?? 0)} trang OCR lỗi
                  </Button>
                  <Checkbox checked={keCaHong} onChange={setKeCaHong} disabled={busy} text="Kể cả trang hỏng vĩnh viễn" />
                </div>
              )}
            </div>
          </div>
        )}

        {view && (
          <p className="mt-3 text-sm text-fg-subtle">
            {V1ok ? "Phạm vi đang lọc: " : ""}
            {fmtN(view.docs)} tài liệu · chờ OCR {fmtN(view.docsOcrPending)} · OCR xong {fmtN(view.docsOcrDone)} · có PDF 2 lớp {fmtN(view.docsPdf2Done)}
            {view.docsPdf2Rejected > 0 ? ` · ${fmtN(view.docsPdf2Rejected)} không ghép được PDF/A${lyDoTuChoi(view)}` : ""} · bóc 16 trường {fmtN(view.docsFieldsDone)}
            {view.docsFieldsFailed > 0 ? ` (${fmtN(view.docsFieldsFailed)} lỗi)` : ""}
            {view.pagesOcrFailed > 0 ? ` · OCR lỗi ${fmtN(view.pagesOcrFailed)} trang / ${fmtN(view.docsOcrFailed)} tài liệu` : ""}
            {view.pagesInkMeasured > 0 ? (view.pagesInkLow > 0 ? ` · còn sót chữ ${fmtN(view.pagesInkLow)}/${fmtN(view.pagesInkMeasured)} trang đã đo` : ` · bắt đủ chữ ${fmtN(view.pagesInkMeasured)} trang đã đo`) : ""}
            {V1ov.unknown > 0 && V4j3(flt) ? ` Đã loại ${fmtN(V1ov.unknown)} tài liệu chưa có trong mục lục nên không xét được phòng ban, mảng, THBQ.` : ""}
            {V1oi ? " Đang đọc lại…" : ""}
          </p>
        )}
      </Card>

      <Help title="Hai việc này khác nhau thế nào">
        <p>PDF 2 lớp: ghép lớp chữ OCR vào bản quét thành tệp PDF/A-2b để lưu trữ và ký số. Chỉ chạy được với tài liệu đã OCR xong, và trang bị đo còn sót chữ sẽ vào hàng soát.</p>
        <p>Bóc tách 16 trường: máy đọc trang đầu để điền 16 trường mục lục (số ký hiệu, ngày, trích yếu…). Kết quả soát và duyệt ở tab Soát & duyệt.</p>
        <p>Tài liệu bản quét dưới 200 dpi bị máy chủ từ chối ghép PDF/A theo Điều 8.2.a, xem danh sách ở Kho máy chủ.</p>
      </Help>
    </div>
  );
}
