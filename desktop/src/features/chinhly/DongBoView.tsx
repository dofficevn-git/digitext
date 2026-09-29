// Copyright (c) 2026 DigiText. All rights reserved.
// Đồng bộ & xung đột: trạng thái engine, thay đổi chờ đẩy / bị từ chối (thử lại, bỏ), xung đột (chọn theo trường).

import { useCallback, useState } from "react";
import { isTauri } from "@/api/ipc";
import { V41p } from "@/live/useLive";
import { clConflictResolve, clConflicts, clMigratePreview, clMigrateRun, clOutbox, clOutboxDiscard, clOutboxRetry, clSyncNow, str, type ClConflict, type ClMigrateReport, type ClOutboxRow } from "@/api/chinhly";
import { ChonDuAn, errText, fmtTime, n, syncLabel, useClSync } from "./common";

const TEN_BANG: Record<string, string> = {
  cl_ho_so: "Hồ sơ",
  cl_van_ban: "Văn bản",
  cl_van_ban_file: "Tệp văn bản",
  cl_phong: "Phông",
  cl_phong_ban: "Phòng ban",
  cl_mang: "Mảng công việc",
  cl_thbq: "Thời hạn bảo quản",
  cl_ngon_ngu: "Ngôn ngữ",
  cl_loai_van_ban: "Loại văn bản",
  cl_khoi_phong: "Khối phông",
};
const NHAN: Record<string, string> = {
  so_tam: "Số tạm",
  tieu_de: "Tiêu đề",
  so_to: "Số tờ",
  ngay_bat_dau: "Ngày bắt đầu",
  ngay_ket_thuc: "Ngày kết thúc",
  ghi_chu: "Ghi chú",
  trich_yeu: "Trích yếu",
  so_thu_tu: "STT",
  so_ky_hieu: "Ký hiệu",
  ngay_thang: "Ngày tháng",
  co_quan_ban_hanh: "Cơ quan ban hành",
};
const nhan = (k: string) => NHAN[k] ?? k;
const mota = (r: Record<string, unknown> | null | undefined) => (r ? [str(r as never, "so_tam") && `số tạm ${str(r as never, "so_tam")}`, str(r as never, "tieu_de"), str(r as never, "trich_yeu"), str(r as never, "ten")].filter(Boolean).join(" · ") : "");

function XungDot({ c, onDone }: { c: ClConflict; onDone: () => void }) {
  const [pick, setPick] = useState<Record<string, "mine" | "theirs">>(() => Object.fromEntries(c.fields.map((f) => [f, "mine"])));
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);
  const daXoa = !c.theirs || !!c.theirs.deleted_at;
  const chot = async (choice: "mine" | "theirs" | "merged") => {
    setBusy(true);
    try {
      let merged: Record<string, unknown> | undefined;
      if (choice === "merged") {
        merged = { ...c.mine };
        for (const [f, who] of Object.entries(pick)) if (who === "theirs" && c.theirs) merged[f] = c.theirs[f];
      }
      await clConflictResolve(c.id, choice, merged);
      onDone();
    } catch (e) {
      setErr(errText(e));
    } finally {
      setBusy(false);
    }
  };
  return (
    <div className="card mt-2">
      <div className="flex items-center gap-2 flex-wrap">
        <b>{TEN_BANG[c.tbl] ?? c.tbl}</b>
        <span className="muted text-sm">{mota(c.mine)}</span>
        <span className="badge warn">{c.message ?? "Xung đột"}</span>
        <span className="muted text-sm">{fmtTime(c.createdAt)}</span>
      </div>
      {daXoa ? (
        <div className="text-sm mt-2">Bản ghi này đã bị xoá trên máy chủ. Bạn muốn tạo lại từ bản của mình hay bỏ?</div>
      ) : (
        <div className="tbl-scroll mt-2">
          <table className="tbl">
            <thead>
              <tr>
                <th>Trường</th>
                <th>Bản gốc (lúc kéo về)</th>
                <th>Của tôi</th>
                <th>Máy chủ (người khác)</th>
                <th>Giữ</th>
              </tr>
            </thead>
            <tbody>
              {c.fields.map((f) => (
                <tr key={f}>
                  <td>{nhan(f)}</td>
                  <td className="muted">{String(c.base?.[f] ?? "")}</td>
                  <td>{String(c.mine[f] ?? "")}</td>
                  <td>{String(c.theirs?.[f] ?? "")}</td>
                  <td className="whitespace-nowrap">
                    <label className="mr-2">
                      <input type="radio" checked={pick[f] === "mine"} onChange={() => setPick((p) => ({ ...p, [f]: "mine" }))} /> tôi
                    </label>
                    <label>
                      <input type="radio" checked={pick[f] === "theirs"} onChange={() => setPick((p) => ({ ...p, [f]: "theirs" }))} /> máy chủ
                    </label>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
      {err && <div className="text-danger text-sm mt-2">{err}</div>}
      <div className="flex gap-2 mt-2 flex-wrap">
        {daXoa ? (
          <>
            <button className="btn sm primary" disabled={busy} onClick={() => void chot("mine")}>
              Tạo lại từ bản của tôi
            </button>
            <button className="btn sm" disabled={busy} onClick={() => void chot("theirs")}>
              Bỏ, chấp nhận đã xoá
            </button>
          </>
        ) : (
          <>
            <button className="btn sm primary" disabled={busy} onClick={() => void chot("merged")}>
              Áp dụng lựa chọn theo trường
            </button>
            <button className="btn sm" disabled={busy} onClick={() => void chot("mine")}>
              Giữ toàn bộ của tôi
            </button>
            <button className="btn sm" disabled={busy} onClick={() => void chot("theirs")}>
              Lấy bản máy chủ
            </button>
          </>
        )}
      </div>
    </div>
  );
}

function ChuyenDuLieuCu({ onDone }: { onDone: () => void }) {
  const [rep, setRep] = useState<ClMigrateReport | null>(null);
  const [res, setRes] = useState<ClMigrateReport | null>(null);
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);
  const load = useCallback(async () => {
    try {
      setRep(await clMigratePreview());
    } catch (e) {
      setErr(errText(e));
    }
  }, []);
  V41p(load, { topics: ["chinhLy", "dossier"], enabled: isTauri() });
  const tong = rep ? rep.phong + rep.thbq + rep.phongBan + rep.mang + rep.hoSo + rep.vanBan : 0;
  const chay = async () => {
    setBusy(true);
    setErr(null);
    try {
      setRes(await clMigrateRun());
      onDone();
      void load();
    } catch (e) {
      setErr(errText(e));
    } finally {
      setBusy(false);
    }
  };
  if (!rep) return null;
  return (
    <div className="card mt-3.5">
      <h3 className="m-0">Dữ liệu chỉnh lý cũ trên máy này</h3>
      {tong === 0 ? (
        <div className="muted text-sm mt-1">Mọi hồ sơ, văn bản, danh mục lập trước đây trên máy đã được nối với máy chủ.</div>
      ) : (
        <div className="text-sm mt-1">
          Chưa lên máy chủ: {n(rep.hoSo)} hồ sơ, {n(rep.vanBan)} văn bản, {n(rep.phong)} phông, {n(rep.phongBan)} phòng ban, {n(rep.mang)} mảng, {n(rep.thbq)} THBQ. Bấm để đưa vào bản sao đồng bộ (số tạm trùng trong phông sẽ bị báo khi đẩy).
        </div>
      )}
      <div className="flex gap-2 mt-2 items-center flex-wrap">
        <button className="btn primary" disabled={busy || tong === 0} onClick={() => void chay()}>
          {busy ? "Đang chuyển…" : "Đưa dữ liệu cũ lên máy chủ"}
        </button>
        {res && (
          <span className="text-sm text-success-700">
            Đã chuyển {n(res.hoSo)} hồ sơ, {n(res.vanBan)} văn bản, {n(res.phong)} phông, {n(res.phongBan)} phòng ban, {n(res.mang)} mảng.
            {res.errors.length > 0 && <span className="text-danger"> {res.errors.length} lỗi: {res.errors.slice(0, 3).join("; ")}</span>}
          </span>
        )}
        {err && <span className="text-danger text-sm">{err}</span>}
      </div>
    </div>
  );
}

export default function DongBoView() {
  const sync = useClSync();
  const [outbox, setOutbox] = useState<ClOutboxRow[]>([]);
  const [conflicts, setConflicts] = useState<ClConflict[]>([]);
  const [err, setErr] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const load = useCallback(async () => {
    try {
      const [o, c] = await Promise.all([clOutbox(), clConflicts()]);
      setOutbox(o);
      setConflicts(c);
      setErr(null);
    } catch (e) {
      setErr(errText(e));
    }
  }, []);
  const { reload } = V41p(load, { topics: ["clSync", "chinhLy"], enabled: isTauri() });
  const sl = syncLabel(sync);

  const dongBo = async () => {
    setBusy(true);
    try {
      await clSyncNow();
      reload();
    } catch (e) {
      setErr(errText(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <>
      <div className="page-head">
        <h2>Đồng bộ & xung đột</h2>
        <div className="sub">· dữ liệu chỉnh lý giữa máy này và máy chủ</div>
      </div>
      <ChonDuAn />
      <div className="card mt-3.5">
        <div className="flex items-center gap-3 flex-wrap">
          <span className={`badge ${sl.dot === "ok" ? "ok" : sl.dot === "err" ? "err" : sl.dot === "warn" ? "warn" : "gray"}`} title={sl.title}>
            {sl.text}
          </span>
          <span className="text-sm muted">{sl.title}</span>
          <div className="flex-1" />
          <button className="btn primary" disabled={busy} onClick={() => void dongBo()}>
            {busy ? "Đang đồng bộ…" : "Đồng bộ ngay"}
          </button>
        </div>
        {sync?.lastError && <div className="text-danger text-sm mt-2">{sync.lastError}</div>}
        {err && <div className="text-danger text-sm mt-2">{err}</div>}
      </div>

      <ChuyenDuLieuCu onDone={reload} />

      <div className="card mt-3.5">
        <h3 className="m-0">Xung đột cần bạn chọn ({n(conflicts.length)})</h3>
        {conflicts.length === 0 && <div className="muted text-sm mt-1">Không có xung đột. Sửa khác trường với người khác thì máy chủ tự gộp, chỉ cùng trường khác giá trị mới hỏi.</div>}
        {conflicts.map((c) => (
          <XungDot key={c.id} c={c} onDone={reload} />
        ))}
      </div>

      <div className="card mt-3.5">
        <h3 className="m-0">Thay đổi chờ đẩy / bị từ chối ({n(outbox.length)})</h3>
        <div className="tbl-scroll mt-2">
          <table className="tbl">
            <thead>
              <tr>
                <th>Bảng</th>
                <th>Bản ghi</th>
                <th>Thao tác</th>
                <th>Trạng thái</th>
                <th>Lỗi</th>
                <th>Sửa lúc</th>
                <th />
              </tr>
            </thead>
            <tbody>
              {outbox.length === 0 && (
                <tr>
                  <td colSpan={7} className="muted">
                    Mọi thay đổi đã lên máy chủ.
                  </td>
                </tr>
              )}
              {outbox.map((o) => (
                <tr key={o.id}>
                  <td>{TEN_BANG[o.tbl] ?? o.tbl}</td>
                  <td className="max-w-[420px] truncate" title={mota(o.new) || mota(o.base)}>
                    {mota(o.new) || mota(o.base) || o.rowId}
                  </td>
                  <td>{o.op === "delete" ? "Xoá" : o.base ? "Sửa" : "Thêm"}</td>
                  <td>
                    <span className={`badge ${o.status === "pending" ? "info" : "err"}`}>{o.status === "pending" ? "chờ đẩy" : "bị từ chối"}</span>
                    {o.attempts > 0 && <span className="muted text-xs"> ({o.attempts} lần)</span>}
                  </td>
                  <td className="text-danger text-sm">{o.lastError}</td>
                  <td className="text-sm">{fmtTime(o.editedAt ?? o.createdAt)}</td>
                  <td className="whitespace-nowrap">
                    {o.status !== "pending" && (
                      <button className="btn sm" onClick={() => void clOutboxRetry(o.id).then(reload).catch((e) => setErr(errText(e)))}>
                        Thử lại
                      </button>
                    )}{" "}
                    <button
                      className="btn sm"
                      onClick={() => {
                        if (window.confirm("Bỏ thay đổi này và quay về bản máy chủ?")) void clOutboxDiscard(o.id).then(reload).catch((e) => setErr(errText(e)));
                      }}
                    >
                      Bỏ
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </>
  );
}
