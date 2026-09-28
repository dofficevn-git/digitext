// Copyright (c) 2026 DigiText. All rights reserved.
//
// Báo cáo tiến độ dự án (đề nghị chỉnh sửa mục 10): tổng dự án / trang / file, lọc ngày, mỗi dự án
// có thông tin hợp đồng (ngày ký, kết thúc, hạn chót, cán bộ phụ trách), tiến độ thời gian và % hoàn
// thành từng bước — Scan (trang/trang hợp đồng), PDF/A, Ký số, OCR đã duyệt — lấy số từ máy chủ.

import { useCallback, useEffect, useMemo, useState, type ReactNode } from "react";
import { isTauri, V3xq } from "@/api/ipc";
import {
  reportProjectProgress,
  reportSetContract,
  type ProjectContract,
  type ProjectProgress,
} from "@/api/progressReport";
import { V4e5 } from "@/features/dossier/excel";
import { Icon } from "@/components/Icon";

const n = (v: number) => v.toLocaleString("vi-VN");
const errText = (e: unknown) => (e instanceof V3xq || e instanceof Error ? e.message : String(e));
const pct = (a: number, b: number | null | undefined): number | null =>
  b && b > 0 ? Math.min(100, Math.floor((a / b) * 100)) : null;
const ngay = (s: string | null) => (s ? s.split("-").reverse().join("/") : "—");

/** % thời gian hợp đồng đã trôi qua (theo hôm nay). */
function timePct(c: ProjectContract, today = new Date()): number | null {
  if (!c.signedOn || !c.endOn) return null;
  const a = new Date(`${c.signedOn}T00:00:00`).getTime();
  const b = new Date(`${c.endOn}T23:59:59`).getTime();
  if (!(b > a)) return null;
  return Math.max(0, Math.min(100, Math.floor(((today.getTime() - a) / (b - a)) * 100)));
}

interface Buoc {
  key: string;
  ten: string;
  done: number;
  total: number | null;
  donVi: string;
  moTa: string;
}

function cacBuoc(p: ProjectProgress): Buoc[] {
  return [
    {
      key: "scan",
      ten: "SCAN tài liệu",
      done: p.pages,
      total: p.contract.contractPages,
      donVi: "trang",
      moTa: "Tổng số trang đã scan / số trang hợp đồng",
    },
    { key: "pdfa", ten: "PDF/A", done: p.pdfaFiles, total: p.files, donVi: "file", moTa: "Số file đã chạy PDF/A / tổng file" },
    { key: "ky", ten: "Ký số", done: p.signedFiles, total: p.files, donVi: "file", moTa: "Số file đã ký số / tổng file" },
    {
      key: "ocr",
      ten: "OCR",
      done: p.approvedFiles,
      total: p.files,
      donVi: "file",
      moTa: `File đã duyệt / tổng file · chưa duyệt ${n(p.files - p.approvedFiles - p.rejectedFiles)} · từ chối ${n(p.rejectedFiles)}`,
    },
  ];
}

function Vong({ value, label }: { value: number | null; label: string }) {
  const r = 34;
  const c = 2 * Math.PI * r;
  const v = value ?? 0;
  const mau = value == null ? "#9ca3af" : v >= 100 ? "#16a34a" : v >= 50 ? "#2563eb" : "#f59e0b";
  return (
    <svg width="84" height="84" viewBox="0 0 84 84" aria-label={label}>
      <circle cx="42" cy="42" r={r} fill="none" stroke="#e5e7eb" strokeWidth="8" />
      <circle
        cx="42"
        cy="42"
        r={r}
        fill="none"
        stroke={mau}
        strokeWidth="8"
        strokeDasharray={`${(c * v) / 100} ${c}`}
        transform="rotate(-90 42 42)"
        strokeLinecap="round"
      />
      <text x="42" y="47" textAnchor="middle" fontSize="16" fontWeight="700" fill="currentColor">
        {value == null ? "—" : `${value}%`}
      </text>
    </svg>
  );
}

function SuaHopDong({
  p,
  onSaved,
  onCancel,
}: {
  p: ProjectProgress;
  onSaved: (c: ProjectContract) => void;
  onCancel: () => void;
}) {
  const [c, setC] = useState<ProjectContract>(p.contract);
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);
  const so = (v: string) => (v.trim() === "" ? null : Math.max(0, Math.floor(Number(v)) || 0));
  const luu = async () => {
    setBusy(true);
    setErr(null);
    try {
      onSaved(await reportSetContract(p.projectId, c));
    } catch (e) {
      setErr(errText(e));
    } finally {
      setBusy(false);
    }
  };
  const o = (label: string, el: ReactNode) => (
    <label className="grid gap-1 text-sm">
      <span className="muted">{label}</span>
      {el}
    </label>
  );
  return (
    <div className="mt-3 p-3 border border-border-subtle rounded">
      <div className="grid gap-3" style={{ gridTemplateColumns: "repeat(auto-fill, minmax(180px, 1fr))" }}>
        {o("Ngày ký hợp đồng", <input type="date" value={c.signedOn ?? ""} onChange={(e) => setC({ ...c, signedOn: e.target.value || null })} />)}
        {o("Ngày kết thúc hợp đồng", <input type="date" value={c.endOn ?? ""} onChange={(e) => setC({ ...c, endOn: e.target.value || null })} />)}
        {o("Hạn chót hoàn thành", <input type="date" value={c.deadlineOn ?? ""} onChange={(e) => setC({ ...c, deadlineOn: e.target.value || null })} />)}
        {o("Số trang hợp đồng", <input type="number" min={0} value={c.contractPages ?? ""} onChange={(e) => setC({ ...c, contractPages: so(e.target.value) })} />)}
        {o("Số file hợp đồng", <input type="number" min={0} value={c.contractFiles ?? ""} onChange={(e) => setC({ ...c, contractFiles: so(e.target.value) })} />)}
        {o("Cán bộ phụ trách", <input value={c.managerName ?? ""} onChange={(e) => setC({ ...c, managerName: e.target.value })} />)}
      </div>
      {o("Ghi chú", <input className="w-full" value={c.note ?? ""} onChange={(e) => setC({ ...c, note: e.target.value })} />)}
      {err && <div className="text-danger text-sm mt-2">{err}</div>}
      <div className="flex gap-2 mt-3">
        <button className="btn primary" onClick={() => void luu()} disabled={busy}>
          {busy ? "Đang lưu…" : "Lưu hợp đồng"}
        </button>
        <button className="btn" onClick={onCancel} disabled={busy}>
          Huỷ
        </button>
      </div>
    </div>
  );
}

export default function ProjectProgressView() {
  const [rows, setRows] = useState<ProjectProgress[] | null>(null);
  const [projectId, setProjectId] = useState("");
  const [from, setFrom] = useState("");
  const [to, setTo] = useState("");
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);
  const [sua, setSua] = useState<string | null>(null);
  const [tatCa, setTatCa] = useState<ProjectProgress[]>([]);

  const load = useCallback(async () => {
    if (!isTauri()) return;
    setBusy(true);
    setErr(null);
    try {
      const r = await reportProjectProgress({ projectId: projectId || undefined, from: from || undefined, to: to || undefined });
      setRows(r);
      if (!projectId) setTatCa(r);
    } catch (e) {
      setErr(errText(e));
    } finally {
      setBusy(false);
    }
  }, [projectId, from, to]);

  useEffect(() => {
    void load();
  }, [load]);

  const tong = useMemo(
    () =>
      (rows ?? []).reduce(
        (a, r) => ({ duAn: a.duAn + 1, trang: a.trang + r.pages, file: a.file + r.files }),
        { duAn: 0, trang: 0, file: 0 },
      ),
    [rows],
  );

  const xuat = () => {
    const aoa: (string | number)[][] = [
      [
        "STT",
        "Mã dự án",
        "Tên dự án",
        "Ngày ký HĐ",
        "Ngày kết thúc HĐ",
        "Hạn chót",
        "Cán bộ phụ trách",
        "Trang đã scan",
        "Trang hợp đồng",
        "% Scan",
        "Tổng file",
        "File PDF/A",
        "% PDF/A",
        "File đã ký số",
        "% Ký số",
        "File OCR đã duyệt",
        "% OCR",
        "% thời gian đã qua",
        "% hoàn thành công việc",
      ],
    ];
    (rows ?? []).forEach((r, i) => {
      const b = cacBuoc(r).map((x) => pct(x.done, x.total));
      const cv = trungBinh(b);
      aoa.push([
        i + 1,
        r.code,
        r.name,
        ngay(r.contract.signedOn),
        ngay(r.contract.endOn),
        ngay(r.contract.deadlineOn),
        r.contract.managerName ?? "",
        r.pages,
        r.contract.contractPages ?? "",
        b[0] ?? "",
        r.files,
        r.pdfaFiles,
        b[1] ?? "",
        r.signedFiles,
        b[2] ?? "",
        r.approvedFiles,
        b[3] ?? "",
        timePct(r.contract) ?? "",
        cv ?? "",
      ]);
    });
    void V4e5([{ name: "TienDoDuAn", aoa }], `tien-do-du-an-${new Date().toISOString().slice(0, 10)}.xlsx`);
  };

  return (
    <section className="view active">
      <div className="page-head">
        <h2>Báo cáo tiến độ dự án</h2>
        <div className="sub">· số liệu trên máy chủ</div>
      </div>

      <div className="card mt-3.5">
        <div className="flex gap-3 items-end flex-wrap">
          <label className="grid gap-1 text-sm">
            <span className="muted">Chọn dự án</span>
            <select value={projectId} onChange={(e) => setProjectId(e.target.value)}>
              <option value="">— Tất cả dự án —</option>
              {tatCa.map((p) => (
                <option key={p.projectId} value={p.projectId}>
                  {p.code} · {p.name}
                </option>
              ))}
            </select>
          </label>
          <label className="grid gap-1 text-sm">
            <span className="muted">Ngày bắt đầu</span>
            <input type="date" value={from} max={to || undefined} onChange={(e) => setFrom(e.target.value)} />
          </label>
          <label className="grid gap-1 text-sm">
            <span className="muted">Ngày kết thúc</span>
            <input type="date" value={to} min={from || undefined} onChange={(e) => setTo(e.target.value)} />
          </label>
          <div className="flex-1" />
          <button className="btn" onClick={() => void load()} disabled={busy}>
            <Icon name="lamMoi" /> {busy ? "Đang lấy…" : "Lấy dữ liệu"}
          </button>
          <button className="btn primary" onClick={xuat} disabled={!rows?.length}>
            <Icon name="tepExcel" /> Xuất báo cáo
          </button>
        </div>
        {err && <div className="text-danger text-sm mt-2">{err}</div>}
        <div className="flex gap-6 flex-wrap mt-4">
          <div>
            <div className="muted text-sm">Tổng số dự án</div>
            <div className="text-2xl font-bold">{n(tong.duAn)}</div>
          </div>
          <div>
            <div className="muted text-sm">Tổng số trang</div>
            <div className="text-2xl font-bold">{n(tong.trang)}</div>
          </div>
          <div>
            <div className="muted text-sm">Tổng số file</div>
            <div className="text-2xl font-bold">{n(tong.file)}</div>
          </div>
        </div>
      </div>

      {(rows ?? []).map((r) => {
        const buoc = cacBuoc(r);
        const ptBuoc = buoc.map((b) => pct(b.done, b.total));
        const cv = trungBinh(ptBuoc);
        const tg = timePct(r.contract);
        const lech = cv != null && tg != null ? cv - tg : null;
        const quaHan =
          r.contract.deadlineOn != null &&
          new Date(`${r.contract.deadlineOn}T23:59:59`).getTime() < Date.now() &&
          (cv ?? 0) < 100;
        return (
          <div key={r.projectId} className="card mt-3.5">
            <div className="flex gap-2 items-center flex-wrap">
              <h3 className="m-0">
                {r.code} · {r.name}
              </h3>
              <div className="flex-1" />
              {sua !== r.projectId && (
                <button className="btn sm" onClick={() => setSua(r.projectId)}>
                  <Icon name="sua" /> Sửa hợp đồng
                </button>
              )}
            </div>
            <div className="flex gap-6 flex-wrap mt-2 text-sm">
              <span>
                <span className="muted">Ngày ký hợp đồng:</span> <b>{ngay(r.contract.signedOn)}</b>
              </span>
              <span>
                <span className="muted">Ngày kết thúc hợp đồng:</span> <b>{ngay(r.contract.endOn)}</b>
              </span>
              <span>
                <span className="muted">Hạn chót hoàn thành:</span> <b>{ngay(r.contract.deadlineOn)}</b>
              </span>
              <span>
                <span className="muted">Cán bộ phụ trách:</span> <b>{r.contract.managerName || "—"}</b>
              </span>
            </div>
            {sua === r.projectId && (
              <SuaHopDong
                p={r}
                onCancel={() => setSua(null)}
                onSaved={(c) => {
                  setSua(null);
                  setRows((l) => (l ?? []).map((x) => (x.projectId === r.projectId ? { ...x, contract: c } : x)));
                }}
              />
            )}

            <div className="mt-3">
              <div className="flex justify-between text-sm">
                <span>
                  Tiến độ thời gian hoàn thành: <b>{tg == null ? "— (chưa nhập ngày ký / kết thúc hợp đồng)" : `${tg}% thời gian hợp đồng đã qua`}</b>
                </span>
                <span>
                  Hoàn thành công việc: <b>{cv == null ? "—" : `${cv}%`}</b>
                </span>
              </div>
              <div className="mt-1 h-2 bg-border-subtle rounded relative">
                <div className="h-2 rounded bg-accent" style={{ width: `${cv ?? 0}%` }} />
                {tg != null && (
                  <div
                    title={`Thời gian đã qua ${tg}%`}
                    style={{ position: "absolute", left: `${tg}%`, top: -3, width: 2, height: 14, background: "#dc2626" }}
                  />
                )}
              </div>
              <div className="text-sm mt-1">
                Đánh giá:{" "}
                {lech == null ? (
                  <span className="muted">chưa đủ thông tin hợp đồng để đánh giá</span>
                ) : lech >= 0 ? (
                  <b className="text-success-700">Đúng / vượt tiến độ (+{lech} điểm %)</b>
                ) : (
                  <b className={lech < -10 ? "text-danger" : "text-warning-700"}>
                    Chậm tiến độ {-lech} điểm % — giảm trừ theo tỷ lệ {-lech}%
                  </b>
                )}
                {quaHan && <b className="text-danger"> · QUÁ HẠN CHÓT</b>}
              </div>
            </div>

            <div className="grid gap-4 mt-4" style={{ gridTemplateColumns: "repeat(auto-fill, minmax(200px, 1fr))" }}>
              {buoc.map((b, i) => (
                <div key={b.key} className="flex gap-3 items-center">
                  <Vong value={ptBuoc[i]} label={b.ten} />
                  <div className="text-sm">
                    <div className="font-bold">{b.ten}</div>
                    <div>
                      {n(b.done)}/{b.total != null ? n(b.total) : "?"} {b.donVi}
                    </div>
                    <div className="muted">{b.moTa}</div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        );
      })}
      {rows && rows.length === 0 && <div className="card mt-3.5 muted">Không có dự án nào trong phạm vi được xem.</div>}
    </section>
  );
}

function trungBinh(v: (number | null)[]): number | null {
  const co = v.filter((x): x is number => x != null);
  return co.length ? Math.floor(co.reduce((a, b) => a + b, 0) / co.length) : null;
}
