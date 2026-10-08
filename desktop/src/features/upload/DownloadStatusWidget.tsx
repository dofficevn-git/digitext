// Copyright (c) 2026 DigiText. All rights reserved.
// Khung tiến độ tải xuống (góc màn hình, cạnh khung đẩy tệp): thanh %, số tệp, dung lượng, tốc độ, thời gian
// còn lại, tệp đang tải, tệp lỗi. Nằm ở cấp ứng dụng nên chuyển màn vẫn thấy.

import { useEffect, useState } from "react";
import { Icon } from "@/components/Icon";
import { TEN_LOAI_TAI, useDownloads } from "@/state/downloads";
import "./uploadWidget.css";

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

export default function DownloadStatusWidget() {
  const { job, running, stop, dismiss } = useDownloads();
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
  const tocDoTep = troiQua > 0 ? xuLy / (troiQua / 60_000) : 0;
  const tocDoByte = troiQua > 0 ? job.bytesDone / (troiQua / 1000) : 0;
  const conLai = xuLy > 0 && running ? ((job.total - xuLy) * troiQua) / xuLy : 0;
  // Bản PDF/A / bản ký có cỡ khác bản gốc → dung lượng là ước lượng.
  const xapXi = job.kind === "original" ? "" : "~";

  return (
    <section className="uw" role="status" aria-live="polite" aria-label="Tiến độ tải xuống">
      <header className="uw-head">
        <Icon name="taiVe" size={15} className="flex-none text-brand-600" />
        <span className="uw-title">
          {running ? (job.stopping ? "Đang dừng…" : `Đang tải ${TEN_LOAI_TAI[job.kind]}`) : `Đã tải xong ${TEN_LOAI_TAI[job.kind]}`}
        </span>
        {running ? (
          <button type="button" className="uw-iconbtn" onClick={stop} disabled={job.stopping} title="Dừng tải (tệp đang tải dở vẫn hoàn tất)">
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
          <span className="font-medium tabular-nums text-fg-strong">
            {running ? `${n(xuLy)}/${n(job.total)} tệp` : `${n(job.done)}/${n(job.total)} tệp tải được`}
          </span>
          <span className="tabular-nums text-fg-subtle">
            {xapXi}
            {dungLuong(job.bytesDone)} / {xapXi}
            {dungLuong(job.bytesTotal)}
          </span>
        </div>
        <div className="h-2 overflow-hidden rounded-full bg-gray-150" role="progressbar" aria-valuemin={0} aria-valuemax={100} aria-valuenow={pct} aria-label="Đã tải">
          <div className={`h-full rounded-full transition-[width] duration-500 ${running ? "bg-run-solid" : job.failed.length ? "bg-warning-500" : "bg-brand-500"}`} style={{ width: `${pct}%` }} />
        </div>
        <div className="flex flex-wrap justify-between gap-x-3 text-xs text-fg-subtle tabular-nums">
          <span>
            {pct}% · {n(Math.round(tocDoTep))} tệp/phút · {xapXi}
            {dungLuong(tocDoByte)}/giây
          </span>
          <span>{running ? (xuLy > 0 ? `còn khoảng ${thoiGian(conLai)}` : "đang bắt đầu…") : `mất ${thoiGian(troiQua)}`}</span>
        </div>

        {running && job.current.length > 0 && (
          <div className="truncate text-xs text-fg-subtle" title={job.current.join("\n")}>
            Đang tải: {job.current.join(", ")}
          </div>
        )}
        {job.skipped > 0 && <div className="text-xs text-fg-subtle">Bỏ qua {job.skipNote}.</div>}
        {!running && job.done > 0 && (
          <div className="break-all text-xs text-fg-subtle" title={job.lastPath}>
            Lưu tại {job.destDir}
          </div>
        )}

        {job.failed.length > 0 && (
          <div className="rounded-md border border-danger-border bg-danger-bg px-2.5 py-1.5 text-xs text-danger-700">
            <button type="button" className="flex w-full items-center gap-1 text-left font-medium" onClick={() => setMoLoi((v) => !v)} aria-expanded={moLoi}>
              <Icon name={moLoi ? "xuong" : "moNhanh"} size={12} />
              {n(job.failed.length)} tệp lỗi
            </button>
            {moLoi && (
              <ul className="mt-1 max-h-40 space-y-1 overflow-auto">
                {job.failed.map((f) => (
                  <li key={`${f.folderPath}/${f.filename}`}>
                    <span className="font-medium">{f.filename}</span>: {f.message}
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
