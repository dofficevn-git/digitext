// Copyright (c) 2026 DigiText. All rights reserved.
// Chọn vị trí khối chữ ký trên một tệp mẫu (ít trang) của lô: vẽ trang (đầu / cuối) bằng pdf.js, bấm vào đâu thì góc
// trên-trái khối chữ ký đặt ở đó. Toạ độ lưu theo tỉ lệ trang nên áp được cho mọi tệp trong lô.

import { useEffect, useRef, useState } from "react";
import * as pdfjs from "pdfjs-dist";
import workerUrl from "pdfjs-dist/build/pdf.worker.min.mjs?url";
import { signPreviewPdf, type SignPlacement } from "@/api/sign";
import { Banner, Button, ErrorBanner, Seg } from "@/ui";

pdfjs.GlobalWorkerOptions.workerSrc = workerUrl;

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
  const [err, setErr] = useState<unknown>(null);

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
        if (!cancelled) setErr(e);
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

  const datTai = (x: number, y: number) => onChange({ lastPage, x: Math.min(1, Math.max(0, x)), y: Math.min(1, Math.max(0, y)) });
  const pick = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const r = e.currentTarget.getBoundingClientRect();
    datTai((e.clientX - r.left) / r.width, (e.clientY - r.top) / r.height);
  };
  // Bàn phím: mũi tên dịch 2 %, Enter đặt giữa trang nếu chưa chọn.
  const phim = (e: React.KeyboardEvent<HTMLCanvasElement>) => {
    const step = 0.02;
    const cur = value && value.lastPage === lastPage ? value : null;
    if (e.key === "Enter" && !cur) {
      e.preventDefault();
      datTai(0.5, 0.5);
      return;
    }
    if (!cur) return;
    const d: Record<string, [number, number]> = { ArrowLeft: [-step, 0], ArrowRight: [step, 0], ArrowUp: [0, -step], ArrowDown: [0, step] };
    const m = d[e.key];
    if (!m) return;
    e.preventDefault();
    datTai(cur.x + m[0], cur.y + m[1]);
  };

  const changePage = (lp: boolean) => {
    setLastPage(lp);
    if (value) onChange({ ...value, lastPage: lp });
  };

  return (
    <div className="grid gap-3">
      <div className="flex flex-wrap items-center gap-3 text-sm">
        <span className="text-fg-subtle">
          Tệp mẫu: <b className="text-fg">{filename}</b>
          {page && ` · trang ${page.num}/${page.total}`}
        </span>
        <Seg<"dau" | "cuoi"> value={lastPage ? "cuoi" : "dau"} onChange={(v) => changePage(v === "cuoi")} options={[{ value: "dau", label: "Trang đầu" }, { value: "cuoi", label: "Trang cuối" }]} />
        {value && (
          <Button size="sm" variant="ghost" onClick={() => onChange(null)}>
            Về mặc định
          </Button>
        )}
      </div>
      {err !== null && <ErrorBanner error={err} ctx="mở PDF/A của tệp mẫu" />}
      {busy && <p className="text-sm text-fg-subtle">Đang tải PDF/A của tệp mẫu…</p>}
      {page && page.rotate !== 0 && <Banner tone="warning">Trang này bị xoay {page.rotate}° trong tệp nên vị trí trên màn hình có thể lệch khi ký. Nên ký thử một tệp rồi mở kiểm tra trước khi ký cả lô.</Banner>}
      <div className="relative inline-block border border-border-subtle" style={{ width: VIEW_W_PX, maxWidth: "100%" }}>
        <canvas ref={canvasRef} onClick={pick} onKeyDown={phim} tabIndex={0} role="button" aria-label="Bấm hoặc dùng mũi tên để đặt vị trí khối chữ ký" className="block w-full cursor-crosshair outline-none focus-visible:ring-2 focus-visible:ring-ring/40" />
        {value && page && value.lastPage === lastPage && (
          <div
            className="pointer-events-none absolute border-2 border-dashed border-danger-600 bg-danger-50/70 p-0.5 text-xs text-danger-700"
            style={{ left: `${value.x * 100}%`, top: `${value.y * 100}%`, width: `${(BLOCK_W_PT / page.wPt) * 100}%`, height: `${(BLOCK_H_PT / page.hPt) * 100}%` }}
          >
            Chữ ký số
          </div>
        )}
      </div>
      <p className="text-sm text-fg-subtle">
        {value ? `Đã chọn: cách lề trái ${Math.round(value.x * 100)}%, cách mép trên ${Math.round(value.y * 100)}%. Áp cho mọi tệp trong lô, khung đỏ là cỡ gần đúng.` : "Bấm vào trang để đặt góc trên-trái của khối chữ ký. Chưa chọn thì ký ở góc trên-phải trang đầu."}
      </p>
    </div>
  );
}
