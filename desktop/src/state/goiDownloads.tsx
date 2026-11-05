// Copyright (c) 2026 DigiText. All rights reserved.
// Tải gói tin từ máy chủ về máy (Đóng gói › Gói tin › Tải về đã chọn) — chạy ở cấp ứng dụng để chuyển màn
// vẫn tải tiếp và vẫn thấy tiến độ (khung ở góc màn hình, cạnh khung đẩy tệp). Tải tuần tự: gói AIP hồ sơ
// nặng cả trăm MB, kéo song song chỉ làm nghẽn đường chứ không nhanh hơn.

import { createContext, useCallback, useContext, useMemo, useRef, useState } from "react";
import type { ReactNode } from "react";
import { V4rw, type V4rt } from "@/api/packaging";
import { errorText, toast } from "@/ui";

export interface GoiTaiLoi {
  label: string;
  message: string;
}

export interface GoiTaiJob {
  destDir: string;
  /** Chọn lẫn nhiều loại gói → mỗi loại một thư mục con theo mã loại. */
  tachTheoLoai: boolean;
  total: number;
  done: number;
  failed: GoiTaiLoi[];
  bytesDone: number;
  bytesTotal: number;
  startedAt: number;
  finishedAt: number | null;
  stopping: boolean;
  /** Nhãn gói đang tải, null khi chưa bắt đầu hoặc đã xong. */
  current: string | null;
  lastPath: string;
}

interface Ctx {
  job: GoiTaiJob | null;
  running: boolean;
  start: (goi: V4rt[], destDir: string) => void;
  stop: () => void;
  dismiss: () => void;
}

const GoiDownloadsContext = createContext<Ctx | null>(null);

export function GoiDownloadsProvider({ children }: { children: ReactNode }) {
  const [job, setJob] = useState<GoiTaiJob | null>(null);
  const stopRef = useRef(false);
  const running = !!job && job.finishedAt === null;

  const start = useCallback<Ctx["start"]>((goi, destDir) => {
    if (goi.length === 0) return;
    const tachTheoLoai = new Set(goi.map((p) => p.kind)).size > 1;
    stopRef.current = false;
    setJob({
      destDir,
      tachTheoLoai,
      total: goi.length,
      done: 0,
      failed: [],
      bytesDone: 0,
      bytesTotal: goi.reduce((s, p) => s + (p.zipSize ?? 0), 0),
      startedAt: Date.now(),
      finishedAt: null,
      stopping: false,
      current: null,
      lastPath: "",
    });
    void (async () => {
      let soXong = 0;
      let cuoi = "";
      const loi: GoiTaiLoi[] = [];
      for (const p of goi) {
        if (stopRef.current) break;
        setJob((j) => (j ? { ...j, current: p.label } : j));
        try {
          const path = await V4rw(p.id, p.zipName ?? `${p.label}.zip`, tachTheoLoai ? `${destDir}\\${p.kind}` : destDir);
          soXong += 1;
          cuoi = path;
          setJob((j) => (j ? { ...j, done: j.done + 1, bytesDone: j.bytesDone + (p.zipSize ?? 0), lastPath: path, current: null } : j));
        } catch (e) {
          const m = { label: p.label, message: errorText(e, "tải gói") };
          loi.push(m);
          setJob((j) => (j ? { ...j, bytesDone: j.bytesDone + (p.zipSize ?? 0), failed: [...j.failed, m], current: null } : j));
        }
      }
      const dung = stopRef.current;
      setJob((j) => (j ? { ...j, finishedAt: Date.now(), stopping: false, current: null } : j));
      const n = (v: number) => v.toLocaleString("vi-VN");
      const noi = tachTheoLoai ? `${destDir} (mỗi loại gói một thư mục con)` : destDir;
      const msg = `Đã tải ${n(soXong)}/${n(goi.length)} gói tin${dung ? " (đã dừng giữa chừng)" : ""}.`;
      const desc = [loi.length ? `${n(loi.length)} gói lỗi` : "", cuoi ? `Lưu tại ${noi}` : ""].filter(Boolean).join(" · ");
      (loi.length || dung ? toast.warning : toast.success)(msg, { description: desc || undefined });
    })();
  }, []);

  const stop = useCallback(() => {
    stopRef.current = true;
    setJob((j) => (j ? { ...j, stopping: true } : j));
  }, []);
  const dismiss = useCallback(() => setJob((j) => (j && j.finishedAt !== null ? null : j)), []);

  // Bản dev trên trình duyệt: ?dlgoi=1 tự chạy một đợt tải giả (xem trước khung tiến độ).
  const thuRef = useRef(false);
  if (import.meta.env.DEV && !thuRef.current && new URLSearchParams(window.location.search).has("dlgoi")) {
    thuRef.current = true;
    const gia = Array.from(
      { length: 12 },
      (_, i) =>
        ({
          id: `g${i}`,
          kind: i % 2 ? "sip_hoso" : "aip_hoso",
          label: `Tập lưu Quyết định của Văn phòng UBND tỉnh Bắc Giang năm 2016 (từ số ${i * 30 + 1}- ${i * 30 + 30})`,
          zipName: `urn_H05.13.24.01_uuid_${i}.zip`,
          zipSize: 62_500_000,
        }) as V4rt,
    );
    setTimeout(() => start(gia, "D:/Tai ve/Goi tin"), 0);
  }

  const value = useMemo(() => ({ job, running, start, stop, dismiss }), [job, running, start, stop, dismiss]);
  return <GoiDownloadsContext.Provider value={value}>{children}</GoiDownloadsContext.Provider>;
}

export function useGoiDownloads(): Ctx {
  const c = useContext(GoiDownloadsContext);
  if (!c) throw new Error("useGoiDownloads ngoài GoiDownloadsProvider");
  return c;
}
