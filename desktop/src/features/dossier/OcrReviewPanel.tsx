// Copyright (c) 2026 DigiText. All rights reserved.
// Soát OCR một tệp PDF trong hồ sơ: ảnh trang bên trái (OCR bóc khung tuỳ chọn), form văn bản bên phải
// được điền sẵn từ trường máy chủ đã bóc; bấm Lưu để ghi vào hồ sơ (tự đẩy sang chỉnh lý).

import { useCallback, useEffect, useState } from "react";
import "./ocrReview.css";
import { V40p, V40x } from "@/api/archive";
import { V4by, V4bz, V4c0, type V4bm, type V4bp, type V4ca } from "@/api/dossier";
import { V4ei } from "@/api/docs";
import { V4fg, V4fk, type V4fj } from "@/api/ocr";
import { V4cw, V4cx, V4cy, V4cz } from "./logic";
import { V4ge, V4gf, V4gi, V4gj } from "./ocrPrefill";
import V4h3 from "./OcrBoxLayer";
import type { V4gk } from "./ocrBoxes";
import { Banner, Button, Field, Input, notifyError, PageHeader, Textarea, toast, errorText } from "@/ui";

function baseName(path: string): string {
  const p = path.split(/[\\/]/);
  return (p[p.length - 1] ?? "").toLowerCase();
}

export default function V4h7({ dossier, file, onBack }: { dossier: V4bm; file: V4ca; onBack: () => void }) {
  const [pages, setPages] = useState<string[]>([]);
  const [pageErr, setPageErr] = useState<string | null>(null);
  const [draft, setDraft] = useState<V4bp>(V4cw());
  const [vbId, setVbId] = useState<number | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [prefill, setPrefill] = useState<string | null>(null);
  const [ocr, setOcr] = useState<Record<number, V4fj>>({});
  const [boxes, setBoxes] = useState<Record<number, V4gk[]>>({});
  const [selKey, setSelKey] = useState<string | null>(null);
  const [ocrPage, setOcrPage] = useState<number | null>(null);

  const chayOcr = async (i: number) => {
    setOcrPage(i);
    try {
      const r = await V4fk(file.filePath, i + 1);
      setOcr((m) => ({ ...m, [i]: r }));
      setBoxes((m) => ({ ...m, [i]: r.boxes.map((b) => ({ ...b })) }));
      toast.info(`Trang ${i + 1}: ${r.boxes.length} khung.`);
    } catch (e) {
      notifyError(e, "OCR trang");
    } finally {
      setOcrPage(null);
    }
  };
  const suaBox = (i: number, b: V4gk) => setBoxes((m) => ({ ...m, [i]: (m[i] ?? []).map((x) => (x.key === b.key ? b : x)) }));

  const napForm = useCallback(async () => {
    const vbs = await V4by(dossier.id);
    const mine = vbs.find((v) => v.pdfPath && baseName(v.pdfPath) === baseName(file.fileName));
    let d = mine ? V4cx(mine) : V4cw();
    setVbId(mine ? mine.id : null);
    let note: string | null = null;
    try {
      const docs = await V4ei();
      const doc = V4gf(docs, V4ge(dossier), file.fileName);
      if (doc) {
        const r = V4gi(d, await V4fg(doc.id));
        if (r.filled.length > 0) {
          d = r.draft;
          note = `Đã điền sẵn từ máy chủ: ${V4gj(r.filled)}. Kiểm tra rồi bấm Lưu.`;
        }
      }
    } catch {
      /* không có trường máy chủ */
    }
    setDraft(d);
    setPrefill(note);
  }, [dossier, file.fileName]);

  useEffect(() => {
    let alive = true;
    setLoading(true);
    Promise.all([
      V40p(file.filePath)
        .then((p) => alive && (setPages(p), setPageErr(null)))
        .catch((e) => alive && (setPages([]), setPageErr(errorText(e)))),
      napForm().catch((e) => alive && notifyError(e, "nạp văn bản")),
    ]).finally(() => alive && setLoading(false));
    return () => {
      alive = false;
    };
  }, [file.filePath, napForm]);

  const set = (p: Partial<V4bp>) => setDraft((d) => ({ ...d, ...p }));
  const num = (s: string) => (s.trim() === "" ? undefined : Number(s));
  const luu = async () => {
    const err = V4cz(draft);
    if (err) {
      toast.error(err);
      return;
    }
    setSaving(true);
    try {
      const body = V4cy(draft);
      if (vbId != null) await V4c0(vbId, body);
      else setVbId(await V4bz(dossier.id, { ...body, pdfPath: file.filePath }));
      await napForm();
      toast.success("Đã lưu văn bản.");
    } catch (e) {
      notifyError(e, "lưu văn bản");
    } finally {
      setSaving(false);
    }
  };
  const text = (key: keyof V4bp, label: string) => (
    <Field label={label}>
      <Input size="sm" value={(draft[key] as string | undefined) ?? ""} onChange={(e) => set({ [key]: e.target.value } as Partial<V4bp>)} />
    </Field>
  );

  return (
    // eslint-disable-next-line jsx-a11y/no-static-element-interactions -- chỉ bắt Ctrl+Enter để lưu, không phải nút
    <div
      onKeyDown={(e) => {
        if (e.ctrlKey && e.key === "Enter") void luu();
      }}
    >
      <PageHeader
        onBack={onBack}
        title={file.fileName}
        description={`Hồ sơ ${dossier.soTam}${dossier.tieuDe ? ` · ${dossier.tieuDe}` : ""}`}
        actions={
          <>
            <Button icon="tepPdf" onClick={() => V40x(file.filePath).catch((e) => notifyError(e, "mở PDF"))}>
              Mở PDF
            </Button>
            <Button variant="primary" icon="luu" loading={saving} onClick={() => void luu()}>
              Lưu (Ctrl+Enter)
            </Button>
          </>
        }
      />
      {prefill && (
        <Banner tone="info" className="mb-3" onClose={() => setPrefill(null)}>
          {prefill}
        </Banner>
      )}
      <div className="ocr-view">
        <div className="ocr-left">
          {loading ? (
            <div className="ocr-empty">Đang tải trang…</div>
          ) : pages.length > 0 ? (
            pages.map((src, i) => (
              <div key={i} className="ocr-page">
                <div className="ocr-page-tools">
                  <span>Trang {i + 1}</span>
                  <Button size="sm" icon="tuDong" loading={ocrPage === i} disabled={ocrPage !== null} onClick={() => void chayOcr(i)} className="border-gray-600 bg-gray-700 text-gray-100 hover:bg-gray-600">
                    {ocr[i] ? "OCR lại" : "OCR bóc khung"}
                  </Button>
                  {ocr[i] && <span>{boxes[i]?.length ?? 0} khung, kéo để chỉnh</span>}
                </div>
                {ocr[i] ? <V4h3 src={src} boxes={boxes[i] ?? []} pageW={ocr[i].pageW} pageH={ocr[i].pageH} selectedKey={selKey} onSelect={setSelKey} onChange={(b) => suaBox(i, b)} /> : <img src={src} alt={`Trang ${i + 1}`} draggable={false} />}
              </div>
            ))
          ) : (
            <div className="ocr-empty">{pageErr ?? "Không hiển thị được nội dung (không phải PDF ảnh)."}</div>
          )}
        </div>
        <div className="ocr-right space-y-2">
          <div className="grid grid-cols-2 gap-2">
            <Field label="Số thứ tự">
              <Input size="sm" type="number" value={draft.thuTu ?? 0} onChange={(e) => set({ thuTu: Number(e.target.value) || 0 })} />
            </Field>
            {text("loaiVanBan", "Loại văn bản")}
            {text("soKyHieu", "Số, ký hiệu")}
            <Field label="Ngày tháng">
              <Input size="sm" placeholder="dd/mm/yyyy" value={draft.ngayThang ?? ""} onChange={(e) => set({ ngayThang: e.target.value })} />
            </Field>
          </div>
          {text("coQuanBanHanh", "Cơ quan ban hành")}
          <Field label="Trích yếu">
            <Textarea rows={4} value={draft.trichYeu ?? ""} onChange={(e) => set({ trichYeu: e.target.value })} />
          </Field>
          <div className="grid grid-cols-2 gap-2">
            <Field label="Tờ số">
              <Input size="sm" type="number" value={draft.toSo ?? ""} onChange={(e) => set({ toSo: num(e.target.value) })} />
            </Field>
            <Field label="Số trang">
              <Input size="sm" type="number" value={draft.soTrang ?? ""} onChange={(e) => set({ soTrang: num(e.target.value) })} />
            </Field>
            {text("ngonNgu", "Ngôn ngữ")}
            {text("cheDoSuDung", "Chế độ sử dụng")}
            {text("doTinCay", "Độ tin cậy")}
            {text("butTich", "Bút tích")}
            {text("tinhTrangVatLy", "Tình trạng vật lý")}
            {text("ghiChu", "Ghi chú")}
          </div>
        </div>
      </div>
    </div>
  );
}
