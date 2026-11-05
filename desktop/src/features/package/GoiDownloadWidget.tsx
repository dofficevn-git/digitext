// Copyright (c) 2026 DigiText. All rights reserved.
// Khung tiến độ tải gói tin (góc màn hình, cùng chỗ với khung đẩy tệp và khung tải tài liệu):
// thanh %, số gói, dung lượng, tốc độ, thời gian còn lại, gói đang tải, gói lỗi.

import { useEffect, useState } from "react";
import { Icon } from "@/components/Icon";
import { useGoiDownloads } from "@/state/goiDownloads";
import "@/features/upload/uploadWidget.css";

const n = (v: number) => v.toLocaleString("vi-VN");
function dungLuong(b: number): string {
  if (b >= 1e9) return `${(b / 1e9).toLocaleString("vi-VN", { maximumFractionDigits: 1 })} GB`;
  return `${(b / 1e6).toLocaleString("vi-VN", { maximumFractionDigits: 0 })} MB`;
}
function thoiGian(ms: number): string {
  const s = Math.round(ms / 1000);
  if (s < 60) return `${s} giây`;
  const m = Math.round(s / 60);
  if (m < 60) return `${m} phút`;
  return `${Math.floor(m / 60)} giờ ${m % 60} phút`;
}

export default function GoiDownloadWidget() {
  const { job, running, stop, dismiss } = useGoiDownloads();
  const [now, setNow] = useState(() => Date.now());
  const [moLoi, setMoLoi] = useState(false);
  useEffect(() => {
    if (!running) return;
    const t = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(t);
  }, [running]);
  if (!job) return null;

  const xuLy = job.done + job.failed.length;
  const pct = job.total > 0 ? Math.floor((xuLy / job.total) * 100) : 100;
  const troiQua = (job.finishedAt ?? now) - job.startedAt;
  const tocDoByte = troiQua > 0 ? job.bytesDone / (troiQua / 1000) : 0;
  const conLai = xuLy > 0 && running ? ((job.total - xuLy) * troiQua) / xuLy : 0;

  return (
    <section className="uw" role="status" aria-live="polite" aria-label="Tiến độ tải gói tin">
      <header className="uw-head">
        <Icon name="taiVe" size={15} className="flex-none text-brand-600" />
        <span className="uw-title">{running ? (job.stopping ? "Đang dừng…" : "Đang tải gói tin") : "Đã tải xong gói tin"}</span>
        {running ? (
          <button type="button" className="uw-iconbtn" onClick={stop} disabled={job.stopping} title="Dừng tải (gói đang tải dở vẫn hoàn tất)">
            <Icon name="dungHan" size={14} />
          </button>
        ) : (
          <button type="button" className="uw-iconbtn" onClick={dismiss} title="Đóng">
            <Icon name="dong" size={14} />
          </button>
        )}
      </header>

      <div className="space-y-2 px-3 py-2.5 text-sm">
        <div className="flex items-baseline justify-between gap-2">
          <span className="font-medium tabular-nums text-fg-strong">{running ? `${n(xuLy)}/${n(job.total)} gói` : `${n(job.done)}/${n(job.total)} gói tải được`}</span>
          <span className="tabular-nums text-fg-subtle">
            {dungLuong(job.bytesDone)} / {dungLuong(job.bytesTotal)}
          </span>
        </div>
        <div className="h-2 overflow-hidden rounded-full bg-gray-150" role="progressbar" aria-valuemin={0} aria-valuemax={100} aria-valuenow={pct} aria-label="Đã tải">
          <div className={`h-full rounded-full transition-[width] duration-500 ${running ? "bg-run-solid" : job.failed.length ? "bg-warning-500" : "bg-brand-500"}`} style={{ width: `${pct}%` }} />
        </div>
        <div className="flex flex-wrap justify-between gap-x-3 text-xs text-fg-subtle tabular-nums">
          <span>
            {pct}% · {dungLuong(tocDoByte)}/giây
          </span>
          <span>{running ? (xuLy > 0 ? `còn khoảng ${thoiGian(conLai)}` : "đang bắt đầu…") : `mất ${thoiGian(troiQua)}`}</span>
        </div>

        {running && job.current && (
          <div className="truncate text-xs text-fg-subtle" title={job.current}>
            Đang tải: {job.current}
          </div>
        )}
        {!running && job.done > 0 && (
          <div className="break-all text-xs text-fg-subtle" title={job.lastPath}>
            Lưu tại {job.destDir}
            {job.tachTheoLoai ? " (mỗi loại gói một thư mục con)" : ""}
          </div>
        )}

        {job.failed.length > 0 && (
          <div className="rounded-md border border-danger-border bg-danger-bg px-2.5 py-1.5 text-xs text-danger-700">
            <button type="button" className="flex w-full items-center gap-1 text-left font-medium" onClick={() => setMoLoi((v) => !v)} aria-expanded={moLoi}>
              <Icon name={moLoi ? "xuong" : "moNhanh"} size={12} />
              {n(job.failed.length)} gói lỗi
            </button>
            {moLoi && (
              <ul className="mt-1 max-h-40 space-y-1 overflow-auto">
                {job.failed.map((f) => (
                  <li key={f.label}>
                    <span className="font-medium">{f.label}</span>: {f.message}
                  </li>
                ))}
              </ul>
            )}
          </div>
        )}
      </div>
    </section>
  );
}
