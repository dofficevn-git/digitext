// Copyright (c) 2026 DigiText. All rights reserved.
// Tải tài liệu từ máy chủ về máy / ổ cứng (Kho máy chủ › Tải xuống) — chạy ở cấp ứng dụng để chuyển màn vẫn
// tải tiếp và vẫn thấy tiến độ (khung ở góc màn hình, cạnh khung đẩy tệp). Tải song song vài tệp một lúc.

import { createContext, useCallback, useContext, useMemo, useRef, useState } from "react";
import type { ReactNode } from "react";
import { V4el, type DownloadKind, type V4ej } from "@/api/docs";
import { errorText, toast } from "@/ui";

export const TEN_LOAI_TAI: Record<DownloadKind, string> = { original: "bản gốc", pdfa: "PDF/A chưa ký số", signed: "đã ký số" };
const SONG_SONG = 3;

export interface DownloadFile {
  doc: V4ej;
  folderPath: string;
}

export interface DownloadError {
  filename: string;
  folderPath: string;
  message: string;
}

export interface DownloadJob {
  kind: DownloadKind;
  label: string;
  destDir: string;
  total: number;
  done: number;
  failed: DownloadError[];
  /** Bỏ qua vì không đúng loại bản (chưa có PDF/A, chưa ký…), kèm lý do gộp. */
  skipped: number;
  skipNote: string;
  /** Dung lượng gốc đã xử lý / tổng — ước lượng (bản PDF/A, bản ký có cỡ khác bản gốc). */
  bytesDone: number;
  bytesTotal: number;
  startedAt: number;
  finishedAt: number | null;
  stopping: boolean;
  lastPath: string;
  current: string[];
}

interface Ctx {
  job: DownloadJob | null;
  running: boolean;
  start: (files: DownloadFile[], kind: DownloadKind, destDir: string, label: string, accept: (d: V4ej) => string | null) => void;
  stop: () => void;
  dismiss: () => void;
}

const DownloadsContext = createContext<Ctx | null>(null);

export function DownloadsProvider({ children }: { children: ReactNode }) {
  const [job, setJob] = useState<DownloadJob | null>(null);
  const stopRef = useRef(false);
  const running = !!job && job.finishedAt === null;

  const start = useCallback<Ctx["start"]>((files, kind, destDir, label, accept) => {
    // accept(d) trả null = tải; trả chuỗi = bỏ qua với lý do (gom đếm theo lý do).
    const lyDo = new Map<string, number>();
    const can: DownloadFile[] = [];
    for (const f of files) {
      const r = accept(f.doc);
      if (r === null) can.push(f);
      else lyDo.set(r, (lyDo.get(r) ?? 0) + 1);
    }
    const skipped = files.length - can.length;
    const skipNote = [...lyDo].map(([k, v]) => `${v.toLocaleString("vi-VN")} ${k}`).join(", ");
    stopRef.current = false;
    const now = Date.now();
    setJob({
      kind, label, destDir, total: can.length, done: 0, failed: [], skipped, skipNote,
      bytesDone: 0, bytesTotal: can.reduce((s, f) => s + f.doc.sizeBytes, 0),
      startedAt: now, finishedAt: null, stopping: false, lastPath: "", current: [],
    });
    let next = 0;
    let soXong = 0;
    let soLoi = 0;
    let cuoi = "";
    const lane = async () => {
      while (!stopRef.current && next < can.length) {
        const f = can[next++];
        setJob((j) => (j ? { ...j, current: [...j.current, f.doc.filename] } : j));
        try {
          const path = await V4el(f.doc.id, f.doc.filename, f.folderPath, kind === "pdfa", destDir, kind);
          soXong++;
          cuoi = path;
          setJob((j) => (j ? { ...j, done: j.done + 1, bytesDone: j.bytesDone + f.doc.sizeBytes, lastPath: path, current: j.current.filter((x) => x !== f.doc.filename) } : j));
        } catch (e) {
          soLoi++;
          setJob((j) =>
            j
              ? {
                  ...j,
                  bytesDone: j.bytesDone + f.doc.sizeBytes,
                  failed: [...j.failed, { filename: f.doc.filename, folderPath: f.folderPath, message: errorText(e, "tải tệp") }],
                  current: j.current.filter((x) => x !== f.doc.filename),
                }
              : j,
          );
        }
      }
    };
    void Promise.all(Array.from({ length: Math.min(SONG_SONG, Math.max(1, can.length)) }, lane)).then(() => {
      setJob((j) => (j ? { ...j, finishedAt: Date.now(), stopping: false, current: [] } : j));
      const n = (v: number) => v.toLocaleString("vi-VN");
      const msg = `Đã tải ${n(soXong)} tệp ${TEN_LOAI_TAI[kind]} ${label}${stopRef.current ? " (đã dừng giữa chừng)" : ""}.`;
      const desc = [soLoi ? `${n(soLoi)} tệp lỗi` : "", skipNote ? `bỏ qua ${skipNote}` : "", cuoi ? `Lưu tại ${destDir}` : ""].filter(Boolean).join(" · ");
      (soLoi || skipped ? toast.warning : toast.success)(msg, { description: desc || undefined });
    });
  }, []);

  const stop = useCallback(() => {
    stopRef.current = true;
    setJob((j) => (j ? { ...j, stopping: true } : j));
  }, []);
  const dismiss = useCallback(() => setJob((j) => (j && j.finishedAt !== null ? null : j)), []);

  // Bản dev trên trình duyệt: ?dl=1 tự chạy một đợt tải giả (xem trước khung tiến độ).
  const thuRef = useRef(false);
  if (import.meta.env.DEV && !thuRef.current && new URLSearchParams(window.location.search).has("dl")) {
    thuRef.current = true;
    const gia = Array.from({ length: 60 }, (_, i) => ({ doc: { id: `d${i}`, filename: `H05.13.24.01.2008.307.01.${String(i + 1).padStart(7, "0")}.pdf`, relPath: null, hoSoRef: null, pageCount: 3, sizeBytes: 2_400_000, status: "to_sign" } as V4ej, folderPath: "PHONG SKHCNMTBG MOI/HOP 135/HO SO 729" }));
    setTimeout(() => start(gia, "pdfa", "D:/Tai ve/LTLSBN", "đã chọn", () => null), 0);
  }

  const value = useMemo(() => ({ job, running, start, stop, dismiss }), [job, running, start, stop, dismiss]);
  return <DownloadsContext.Provider value={value}>{children}</DownloadsContext.Provider>;
}

export function useDownloads(): Ctx {
  const c = useContext(DownloadsContext);
  if (!c) throw new Error("useDownloads ngoài DownloadsProvider");
  return c;
}
