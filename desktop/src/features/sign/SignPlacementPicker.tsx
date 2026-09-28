// Copyright (c) 2026 DigiText. All rights reserved.
//
// Chọn vị trí khối chữ ký trên TỆP ĐẦU TIÊN của lô: vẽ trang (đầu/cuối) bằng pdf.js, bấm vào
// đâu thì góc trên-trái khối chữ ký đặt ở đó. Toạ độ lưu theo TỈ LỆ trang nên áp được cho mọi
// tệp trong lô dù khổ giấy khác nhau (máy chủ tự quy đổi theo MediaBox từng tệp).

import { useEffect, useRef, useState } from "react";
import * as pdfjs from "pdfjs-dist";
import workerUrl from "pdfjs-dist/build/pdf.worker.min.mjs?url";

import { signPreviewPdf, type SignPlacement } from "@/api/sign";

pdfjs.GlobalWorkerOptions.workerSrc = workerUrl;

// Cỡ gần đúng của khối chữ ký 2 dòng ("Ký bởi…", "Ký ngày…") máy chủ dựng, đơn vị pt.
const BLOCK_W_PT = 230;
const BLOCK_H_PT = 48;
const VIEW_W_PX = 560;

function b64ToBytes(b64: string): Uint8Array {
  const bin = atob(b64);
  const out = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
  return out;
}

interface Props {
  jobId: string;
  filename: string;
  value: SignPlacement | null;
  onChange: (p: SignPlacement | null) => void;
}

export default function SignPlacementPicker({ jobId, filename, value, onChange }: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [doc, setDoc] = useState<pdfjs.PDFDocumentProxy | null>(null);
  const [lastPage, setLastPage] = useState(value?.lastPage ?? false);
  const [page, setPage] = useState<{ num: number; total: number; wPt: number; hPt: number; rotate: number } | null>(null);
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setBusy(true);
    setErr(null);
    setDoc(null);
    (async () => {
      try {
        const data = b64ToBytes(await signPreviewPdf(jobId));
        const d = await pdfjs.getDocument({ data, isEvalSupported: false }).promise;
        if (!cancelled) setDoc(d);
      } catch (e) {
        if (!cancelled) setErr(`Không mở được PDF/A của tệp đầu tiên: ${e instanceof Error ? e.message : String(e)}`);
      } finally {
        if (!cancelled) setBusy(false);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [jobId]);

  useEffect(() => {
    if (!doc || !canvasRef.current) return;
    let task: pdfjs.RenderTask | null = null;
    (async () => {
      const num = lastPage ? doc.numPages : 1;
      const p = await doc.getPage(num);
      const base = p.getViewport({ scale: 1 });
      const vp = p.getViewport({ scale: VIEW_W_PX / base.width });
      const c = canvasRef.current!;
      c.width = Math.floor(vp.width);
      c.height = Math.floor(vp.height);
      task = p.render({ canvasContext: c.getContext("2d")!, viewport: vp });
      await task.promise.catch(() => {});
      setPage({ num, total: doc.numPages, wPt: base.width, hPt: base.height, rotate: p.rotate });
    })();
    return () => task?.cancel();
  }, [doc, lastPage]);

  const pick = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const r = e.currentTarget.getBoundingClientRect();
    const x = Math.min(1, Math.max(0, (e.clientX - r.left) / r.width));
    const y = Math.min(1, Math.max(0, (e.clientY - r.top) / r.height));
    onChange({ lastPage, x, y });
  };

  const changePage = (lp: boolean) => {
    setLastPage(lp);
    if (value) onChange({ ...value, lastPage: lp });
  };

  return (
    <div className="grid gap-2">
      <div className="flex gap-3 items-center flex-wrap text-sm">
        <span className="muted">
          Tệp đầu tiên: <b>{filename}</b>
          {page && ` · trang ${page.num}/${page.total}`}
        </span>
        <label className="flex items-center gap-1">
          <input type="radio" checked={!lastPage} onChange={() => changePage(false)} /> Trang đầu
        </label>
        <label className="flex items-center gap-1">
          <input type="radio" checked={lastPage} onChange={() => changePage(true)} /> Trang cuối
        </label>
        {value && (
          <button className="btn ghost" onClick={() => onChange(null)}>
            Bỏ chọn (về góc trên-phải trang đầu)
          </button>
        )}
      </div>
      {err && <div className="banner error"><div className="bx">{err}</div></div>}
      {busy && <div className="muted text-sm">Đang tải PDF/A của tệp đầu tiên…</div>}
      {page && page.rotate !== 0 && (
        <div className="banner info">
          <div className="bx">
            Trang này được xoay {page.rotate}° trong tệp PDF — vị trí trên màn hình có thể lệch so với
            khi ký. Nên ký thử một tệp rồi mở kiểm tra trước khi ký cả lô.
          </div>
        </div>
      )}
      <div className="relative inline-block border border-border-subtle" style={{ width: VIEW_W_PX, maxWidth: "100%" }}>
        <canvas ref={canvasRef} onClick={pick} className="block w-full cursor-crosshair" />
        {value && page && value.lastPage === lastPage && (
          <div
            className="absolute pointer-events-none"
            style={{
              left: `${value.x * 100}%`,
              top: `${value.y * 100}%`,
              width: `${(BLOCK_W_PT / page.wPt) * 100}%`,
              height: `${(BLOCK_H_PT / page.hPt) * 100}%`,
              border: "2px dashed #d11",
              background: "rgba(221,17,17,0.08)",
              color: "#d11",
              fontSize: 11,
              padding: 2,
            }}
          >
            Chữ ký số
          </div>
        )}
      </div>
      <div className="muted text-sm">
        {value
          ? `Đã chọn: cách lề trái ${Math.round(value.x * 100)}%, cách mép trên ${Math.round(value.y * 100)}% — áp cho TẤT CẢ tệp trong lô (khung đỏ là cỡ gần đúng).`
          : "Bấm vào trang để đặt góc trên-trái của khối chữ ký. Chưa chọn = góc trên-phải trang đầu."}
      </div>
    </div>
  );
}
