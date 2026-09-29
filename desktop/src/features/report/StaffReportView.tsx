// Copyright (c) 2026 DigiText. All rights reserved.
//
// Báo cáo nhân viên (đề nghị chỉnh sửa mục 11): theo dự án + khoảng ngày, mỗi người × công việc
// (Scan, Kiểm tra scan, Nhập dữ liệu trường, Kiểm tra nhập liệu, Xác nhận dữ liệu, Kiểm tra xác nhận):
// sản lượng giao, sản lượng thi công, năng suất TB/ngày, % hoàn thành, số lỗi, tỷ lệ lỗi, KPI.

import { useCallback, useEffect, useMemo, useState } from "react";
import { isTauri, V3xq } from "@/api/ipc";
import { reportSetTarget, reportStaff, type StaffRow, type StaffTask } from "@/api/progressReport";
import { V4ak, type V4a9 } from "@/api/users";
import { V4e5 } from "@/features/dossier/excel";
import { Icon } from "@/components/Icon";

const n = (v: number) => v.toLocaleString("vi-VN");
const errText = (e: unknown) => (e instanceof V3xq || e instanceof Error ? e.message : String(e));

export const TASKS: { key: StaffTask; label: string; unit: string }[] = [
  { key: "scan", label: "Scan tài liệu", unit: "trang" },
  { key: "kiem_tra_scan", label: "Kiểm tra scan", unit: "tài liệu" },
  { key: "nhap_lieu", label: "Nhập dữ liệu trường thông tin", unit: "tài liệu" },
  { key: "kiem_tra_nhap_lieu", label: "Kiểm tra nhập liệu", unit: "tài liệu" },
  { key: "xac_nhan", label: "Xác nhận dữ liệu", unit: "tài liệu" },
  { key: "kiem_tra_xac_nhan", label: "Kiểm tra xác nhận dữ liệu", unit: "tài liệu" },
];
const taskOf = (k: StaffTask) => TASKS.find((t) => t.key === k) ?? TASKS[0];

/** Ngưỡng KPI mặc định — cần bên dùng xác nhận (đề nghị chỉnh sửa mục 11). */
export const KPI = { xuatSac: { hoanThanh: 100, loi: 1 }, dat: { hoanThanh: 80, loi: 3 } };

export function kpiOf(hoanThanh: number | null, tyLeLoi: number): { label: string; tone: string } {
  if (hoanThanh == null) {
    return tyLeLoi > KPI.dat.loi ? { label: "Cảnh báo lỗi", tone: "err" } : { label: "Chưa giao sản lượng", tone: "gray" };
  }
  if (hoanThanh >= KPI.xuatSac.hoanThanh && tyLeLoi <= KPI.xuatSac.loi) return { label: "Xuất sắc", tone: "ok" };
  if (hoanThanh >= KPI.dat.hoanThanh && tyLeLoi <= KPI.dat.loi) return { label: "Đạt", tone: "info" };
  return { label: "Cảnh báo lỗi", tone: "err" };
}

interface Dong extends StaffRow {
  hoanThanh: number | null;
  tyLeLoi: number;
  nangSuat: number | null;
}

function tinh(r: StaffRow): Dong {
  const coSo = r.task === "scan" ? r.files : r.done;
  const tyLeLoi = coSo > 0 ? Math.round((r.errors / coSo) * 1000) / 10 : 0;
  return {
    ...r,
    hoanThanh: r.target && r.target > 0 ? Math.floor((r.done / r.target) * 100) : null,
    tyLeLoi,
    nangSuat: r.days > 0 ? Math.round(r.done / r.days) : null,
  };
}

export default function StaffReportView() {
  const [projects, setProjects] = useState<V4a9[]>([]);
  const [projectId, setProjectId] = useState("");
  const [from, setFrom] = useState("");
  const [to, setTo] = useState("");
  const [rows, setRows] = useState<StaffRow[] | null>(null);
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);
  const [giao, setGiao] = useState<{ userId: string; task: StaffTask; qty: string } | null>(null);

  useEffect(() => {
    if (!isTauri()) return;
    V4ak()
      .then((p) => setProjects(p.filter((x) => x.isActive)))
      .catch(() => {});
  }, []);

  const load = useCallback(async () => {
    if (!isTauri()) return;
    setBusy(true);
    setErr(null);
    try {
      setRows(await reportStaff({ projectId: projectId || undefined, from: from || undefined, to: to || undefined }));
    } catch (e) {
      setErr(errText(e));
    } finally {
      setBusy(false);
    }
  }, [projectId, from, to]);

  useEffect(() => {
    void load();
  }, [load]);

  const dong = useMemo(() => (rows ?? []).map(tinh), [rows]);
  const tong = useMemo(
    () => ({
      trang: dong.filter((d) => d.task === "scan").reduce((a, d) => a + d.done, 0),
      xacNhan: dong.filter((d) => d.task === "xac_nhan").reduce((a, d) => a + d.done, 0),
      truong: dong.filter((d) => d.task === "nhap_lieu").reduce((a, d) => a + d.fields, 0),
    }),
    [dong],
  );
  const nguoi = useMemo(() => {
    const m = new Map<string, { userId: string; ten: string }>();
    for (const d of dong) m.set(d.userId, { userId: d.userId, ten: d.displayName || d.username });
    return [...m.values()];
  }, [dong]);

  const luuGiao = async () => {
    if (!giao || !projectId) return;
    const q = giao.qty.trim() === "" ? null : Math.max(0, Math.floor(Number(giao.qty)) || 0);
    try {
      await reportSetTarget({ projectId, userId: giao.userId, task: giao.task, quantity: q });
      setGiao(null);
      await load();
    } catch (e) {
      setErr(errText(e));
    }
  };

  const xuat = () => {
    const aoa: (string | number)[][] = [
      [
        "STT",
        "Nhân viên",
        "Tài khoản",
        "Công việc",
        "Đơn vị",
        "Sản lượng giao",
        "Sản lượng thi công",
        "Năng suất TB/ngày",
        "% hoàn thành",
        "Số lỗi",
        "Tỷ lệ lỗi (%)",
        "Đánh giá KPI",
      ],
    ];
    dong.forEach((d, i) => {
      const t = taskOf(d.task);
      aoa.push([
        i + 1,
        d.displayName,
        d.username,
        t.label,
        t.unit,
        d.target ?? "",
        d.done,
        d.nangSuat ?? "",
        d.hoanThanh ?? "",
        d.errors,
        d.tyLeLoi,
        kpiOf(d.hoanThanh, d.tyLeLoi).label,
      ]);
    });
    void V4e5([{ name: "TongKetNhanVien", aoa }], `tong-ket-nhan-vien-${new Date().toISOString().slice(0, 10)}.xlsx`);
  };

  return (
    <section className="view active">
      <div className="page-head">
        <h2>Tổng kết theo nhân viên</h2>
        <div className="sub">· sản lượng, năng suất, lỗi và KPI — số liệu trên máy chủ</div>
      </div>

      <div className="card mt-3.5">
        <div className="flex gap-3 items-end flex-wrap">
          <label className="grid gap-1 text-sm">
            <span className="muted">Dự án</span>
            <select value={projectId} onChange={(e) => setProjectId(e.target.value)}>
              <option value="">— Tất cả dự án —</option>
              {projects.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.code} · {p.name}
                </option>
              ))}
            </select>
          </label>
          <label className="grid gap-1 text-sm">
            <span className="muted">Từ ngày</span>
            <input type="date" value={from} max={to || undefined} onChange={(e) => setFrom(e.target.value)} />
          </label>
          <label className="grid gap-1 text-sm">
            <span className="muted">Đến ngày</span>
            <input type="date" value={to} min={from || undefined} onChange={(e) => setTo(e.target.value)} />
          </label>
          <div className="flex-1" />
          <button className="btn" onClick={() => void load()} disabled={busy}>
            <Icon name="lamMoi" /> {busy ? "Đang lấy…" : "Lấy dữ liệu"}
          </button>
          <button className="btn primary" onClick={xuat} disabled={dong.length === 0}>
            <Icon name="tepExcel" /> Xuất báo cáo
          </button>
        </div>
        {err && <div className="text-danger text-sm mt-2">{err}</div>}
        <div className="flex gap-6 flex-wrap mt-4">
          <div>
            <div className="muted text-sm">Tổng sản lượng trang</div>
            <div className="text-2xl font-bold">{n(tong.trang)}</div>
          </div>
          <div>
            <div className="muted text-sm">Tổng sản lượng file xác nhận</div>
            <div className="text-2xl font-bold">{n(tong.xacNhan)}</div>
          </div>
          <div>
            <div className="muted text-sm">Tổng sản lượng các trường đã nhập dữ liệu</div>
            <div className="text-2xl font-bold">{n(tong.truong)}</div>
          </div>
        </div>
      </div>

      <div className="card mt-3.5">
        {projectId && nguoi.length > 0 && (
          <div className="flex gap-2 items-center flex-wrap mb-3 text-sm">
            <span className="muted">Giao sản lượng:</span>
            <select
              value={giao?.userId ?? ""}
              onChange={(e) => setGiao({ userId: e.target.value, task: giao?.task ?? "scan", qty: giao?.qty ?? "" })}
            >
              <option value="">— Nhân viên —</option>
              {nguoi.map((u) => (
                <option key={u.userId} value={u.userId}>
                  {u.ten}
                </option>
              ))}
            </select>
            <select
              value={giao?.task ?? "scan"}
              onChange={(e) => setGiao({ userId: giao?.userId ?? "", task: e.target.value as StaffTask, qty: giao?.qty ?? "" })}
            >
              {TASKS.map((t) => (
                <option key={t.key} value={t.key}>
                  {t.label} ({t.unit})
                </option>
              ))}
            </select>
            <input
              type="number"
              min={0}
              className="max-w-32"
              placeholder="số lượng"
              value={giao?.qty ?? ""}
              onChange={(e) => setGiao({ userId: giao?.userId ?? "", task: giao?.task ?? "scan", qty: e.target.value })}
            />
            <button className="btn sm" onClick={() => void luuGiao()} disabled={!giao?.userId}>
              Lưu
            </button>
            <span className="muted">(để trống số lượng = bỏ giao)</span>
          </div>
        )}
        {!projectId && <div className="muted text-sm mb-2">Chọn một dự án để giao sản lượng cho từng nhân viên.</div>}
        <div className="tbl-scroll">
          <table className="tbl">
            <thead>
              <tr>
                <th>STT</th>
                <th>Tên hoặc tài khoản nhân viên</th>
                <th>Công việc</th>
                <th className="num">Sản lượng giao</th>
                <th className="num">Sản lượng thi công</th>
                <th className="num">Năng suất TB/ngày</th>
                <th className="num">% hoàn thành</th>
                <th className="num">Số lỗi</th>
                <th className="num">Tỷ lệ lỗi</th>
                <th>Đánh giá KPI</th>
              </tr>
            </thead>
            <tbody>
              {dong.length === 0 ? (
                <tr>
                  <td colSpan={10} className="muted">
                    {busy ? "Đang lấy dữ liệu…" : "Không có dữ liệu."}
                  </td>
                </tr>
              ) : (
                dong.map((d, i) => {
                  const t = taskOf(d.task);
                  const k = kpiOf(d.hoanThanh, d.tyLeLoi);
                  return (
                    <tr key={`${d.userId}-${d.task}`}>
                      <td>{i + 1}</td>
                      <td>
                        {d.displayName} <span className="muted">({d.username})</span>
                      </td>
                      <td>{t.label}</td>
                      <td className="num">{d.target != null ? `${n(d.target)} ${t.unit}` : "—"}</td>
                      <td className="num" title={d.task === "nhap_lieu" ? `${n(d.fields)} trường` : undefined}>
                        {n(d.done)} {t.unit}
                      </td>
                      <td className="num">{d.nangSuat != null ? n(d.nangSuat) : "—"}</td>
                      <td className="num">{d.hoanThanh != null ? `${d.hoanThanh}%` : "—"}</td>
                      <td className="num">{n(d.errors)}</td>
                      <td className="num">{d.tyLeLoi}%</td>
                      <td>
                        <span className={`badge ${k.tone}`}>{k.label}</span>
                      </td>
                    </tr>
                  );
                })
              )}
            </tbody>
          </table>
        </div>
        <div className="muted text-sm mt-2">
          Scan tính theo trang người đó quét (tệp đẩy lên máy chủ đứng tên người quét). Lỗi = tài liệu của người đó bị
          từ chối khi xác nhận hoặc bị kiểm tra chéo chấm có lỗi. Xác nhận dữ liệu và kiểm tra nhập liệu lấy từ gói việc (màn
          Giao việc xác nhận); kiểm tra scan và kiểm tra xác nhận chưa có nơi ghi nhận. KPI: Xuất sắc
          (hoàn thành ≥ {KPI.xuatSac.hoanThanh}%, lỗi ≤ {KPI.xuatSac.loi}%) · Đạt (≥ {KPI.dat.hoanThanh}%, lỗi ≤{" "}
          {KPI.dat.loi}%) · còn lại Cảnh báo lỗi.
        </div>
      </div>
    </section>
  );
}
