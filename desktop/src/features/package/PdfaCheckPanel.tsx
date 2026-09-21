// Copyright (c) 2026 DigiText. All rights reserved.
// KIỂM TRA PDF/A cho tệp / thư mục bất kỳ: tệp tự khai loại nào (PDF/A-1B, 2B, 3B…), có đủ dấu
// hiệu bắt buộc không, và (khi máy có Docker) veraPDF — bộ kiểm chuẩn ISO 19005 — kết luận
// ĐẠT / KHÔNG ĐẠT kèm luật vi phạm. "Ctrl+A bôi được chữ" chỉ là có LỚP CHỮ, chưa phải PDF/A.

import { Fragment, useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";

import {
  onExtimportProgress,
  pdfaCheck,
  pdfaVerapdfAvailable,
  type ExtProgress,
  type PdfaFileReport,
  type PdfaSummary,
} from "@/api/extimport";
import { isTauri } from "@/api/ipc";

function fmt(n: number): string {
  return n.toLocaleString("vi-VN");
}

function badgeOf(f: PdfaFileReport): { cls: string; text: string } {
  if (f.error) return { cls: "err", text: "HỎNG" };
  if (f.verapdf) return f.verapdf.compliant ? { cls: "ok", text: "ĐẠT" } : { cls: "err", text: "KHÔNG ĐẠT" };
  if (f.looksPdfa) return { cls: "info", text: "CÓ KHAI" };
  return { cls: "err", text: "KHÔNG PHẢI PDF/A" };
}

export default function PdfaCheckPanel() {
  const [paths, setPaths] = useState<string[]>([]);
  const [useVera, setUseVera] = useState(true);
  const [veraOk, setVeraOk] = useState<boolean | null>(null);
  const [busy, setBusy] = useState(false);
  const [progress, setProgress] = useState<ExtProgress | null>(null);
  const [res, setRes] = useState<PdfaSummary | null>(null);
  const [openRow, setOpenRow] = useState<string | null>(null);
  const [err, setErr] = useState("");

  useEffect(() => {
    if (!isTauri()) return;
    pdfaVerapdfAvailable()
      .then(setVeraOk)
      .catch(() => setVeraOk(false));
    let off: (() => void) | undefined;
    void onExtimportProgress((p) => {
      if (p.phase === "pdfa") setProgress(p);
    }).then((u) => {
      off = u;
    });
    return () => off?.();
  }, []);

  const pickFiles = async () => {
    const r = await open({ multiple: true, title: "Chọn tệp PDF cần kiểm", filters: [{ name: "PDF", extensions: ["pdf"] }] });
    if (Array.isArray(r) && r.length) setPaths(r);
    else if (typeof r === "string") setPaths([r]);
  };
  const pickDir = async () => {
    const r = await open({ directory: true, multiple: false, title: "Chọn thư mục chứa PDF (quét cả thư mục con)" });
    if (typeof r === "string") setPaths([r]);
  };

  const run = async () => {
    setErr("");
    setRes(null);
    setProgress(null);
    setBusy(true);
    try {
      setRes(await pdfaCheck({ paths, useVerapdf: useVera && veraOk === true }));
    } catch (e) {
      setErr(String((e as { message?: string })?.message ?? e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="card p-4 mb-4">
      <h3 className="mt-0">Kiểm tra PDF/A</h3>
      <p className="muted mt-0">
        Chọn tệp hoặc cả thư mục — hệ thống cho biết từng tệp <b>là loại PDF/A nào</b> (PDF/A-1B, 2B, 3B…) hay không
        phải PDF/A, kèm lý do. Bôi được chữ bằng Ctrl+A chỉ chứng minh tệp có lớp chữ, không chứng minh là PDF/A.
      </p>

      <div className="flex items-center gap-2 flex-wrap">
        <button className="btn" onClick={() => void pickFiles()} disabled={busy}>Chọn tệp PDF…</button>
        <button className="btn" onClick={() => void pickDir()} disabled={busy}>Chọn thư mục…</button>
        <span className="muted">
          {paths.length === 0 ? "Chưa chọn gì" : paths.length === 1 ? paths[0] : `${fmt(paths.length)} tệp đã chọn`}
        </span>
      </div>

      <div className="flex items-center gap-4 mt-3 flex-wrap">
        <label className="flex items-center gap-1.5">
          <input
            type="checkbox"
            checked={useVera && veraOk === true}
            onChange={(e) => setUseVera(e.target.checked)}
            disabled={busy || veraOk !== true}
          />
          Kiểm đúng chuẩn ISO bằng veraPDF (kết quả dùng để nghiệm thu)
        </label>
        {veraOk === false && (
          <span className="muted">Máy này chưa có Docker + veraPDF — chỉ xem được loại tệp tự khai và các dấu hiệu bắt buộc.</span>
        )}
        <button className="btn primary" onClick={() => void run()} disabled={!isTauri() || busy || paths.length === 0}>
          {busy ? "Đang kiểm…" : "Kiểm tra PDF/A"}
        </button>
      </div>

      {busy && progress && progress.total > 0 && (
        <div className="banner mt-3">
          Đang kiểm PDF/A: {fmt(progress.done)}/{fmt(progress.total)} bước
        </div>
      )}
      {err && <div className="err-banner mt-3">{err}</div>}

      {res && (
        <div className="mt-4">
          <div className={`banner ${res.notOk === 0 ? "" : "warn"}`}>
            <b>{fmt(res.total)} tệp</b> · {res.verapdfUsed ? "veraPDF ĐẠT" : "có khai PDF/A, đủ dấu hiệu"}: <b>{fmt(res.ok)}</b> ·
            không đạt: <b>{fmt(res.notOk)}</b> · Theo loại:{" "}
            {Object.entries(res.byKind)
              .map(([k, v]) => `${k} — ${fmt(v)}`)
              .join(" · ")}
            {!res.verapdfUsed ? " · (chưa chạy veraPDF — đây mới là tệp TỰ KHAI)" : ""}
            {res.truncated ? " · Quá 5.000 tệp: chỉ kiểm 5.000 tệp đầu, chọn thư mục nhỏ hơn để kiểm phần còn lại." : ""}
          </div>
          <div className="tbl-scroll mt-2.5">
            <table className="tbl">
              <thead>
                <tr>
                  <th>Kết luận</th>
                  <th>Loại tự khai</th>
                  <th>Tệp</th>
                  <th>Hồ sơ màu</th>
                  <th>Lớp chữ</th>
                  <th>dpi</th>
                  <th>Chữ ký số</th>
                  <th>Phần mềm tạo</th>
                </tr>
              </thead>
              <tbody>
                {res.files.map((f) => {
                  const b = badgeOf(f);
                  const detail = [...(f.error ? [f.error] : f.problems), ...(f.verapdf?.rules ?? [])];
                  return (
                    <Fragment key={f.path}>
                      <tr onClick={() => setOpenRow(openRow === f.path ? null : f.path)} style={{ cursor: "pointer" }} title={f.path}>
                        <td>
                          <span className={`badge ${b.cls}`}>{b.text}</span>
                        </td>
                        <td>{f.declared ?? "không khai"}</td>
                        <td>{f.name}</td>
                        <td>{f.hasOutputIntent ? "có" : "KHÔNG"}</td>
                        <td>
                          {f.textPages}/{f.pages} trang
                        </td>
                        <td>{f.minDpi ?? "—"}</td>
                        <td>{f.signed ? "có" : "không"}</td>
                        <td>{f.producer ?? "—"}</td>
                      </tr>
                      {openRow === f.path && detail.length > 0 && (
                        <tr>
                          <td colSpan={8}>
                            <ul className="m-0 pl-5">
                              {detail.map((d, i) => (
                                <li key={i}>{d}</li>
                              ))}
                            </ul>
                          </td>
                        </tr>
                      )}
                    </Fragment>
                  );
                })}
              </tbody>
            </table>
          </div>
          <p className="muted mt-2">
            Bấm vào một dòng để xem lý do chi tiết.
            {res.total > res.files.length ? ` Bảng hiện ${fmt(res.files.length)} tệp đầu (tệp không đạt xếp trước).` : ""}
            {res.reportFile ? (
              <>
                {" "}Bảng đầy đủ (mở bằng Excel): <code>{res.reportFile}</code>
              </>
            ) : null}
          </p>
        </div>
      )}
    </div>
  );
}
