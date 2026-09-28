// Copyright (c) 2026 DigiText. All rights reserved.

import { useCallback, useEffect, useState } from "react";
import { V41p } from "@/live/useLive";

import { isTauri, V3xq } from "@/api/ipc";
import {
  V4qu,
  V4qt,
  V4qv,
  V4qw,
  onSignBatchProgress,
  signBatch,
  signBatchCancel,
  type SignBatchProgress,
  type SignBatchResult,
  type SignPlacement,
  type V4qr,
  type V4qs,
} from "@/api/sign";
import { V4ak, type V4a9 } from "@/api/users";
import { Icon } from "@/components/Icon";
import SignPlacementPicker from "./SignPlacementPicker";

function V4qx(V2gy: unknown): string {
  return V2gy instanceof V3xq || V2gy instanceof Error ? V2gy.message : String(V2gy);
}

// Bảng chỉ vẽ tối đa ngần này dòng (lô có thể hàng nghìn tài liệu) — ký hàng loạt vẫn ký hết.
const TABLE_MAX_ROWS = 200;

export default function V4qy() {
  const [V2gz, V2h0] = useState<V4qr[]>([]);

  const [V2h1, V2h2] = useState("");
  const [V2h3, V2h4] = useState<string[]>([]);
  const [tokens, V2h5] = useState<V4qs[]>([]);
  const [V2h6, V2h7] = useState<number | null>(null);
  const [V2h8, V2h9] = useState("");
  const [V2ha, V2hb] = useState<string | null>(null);
  const [V2hc, V2hd] = useState<{ kind: "info" | "err" | "ok"; text: string } | null>(null);

  const [projects, setProjects] = useState<V4a9[]>([]);
  const [projectId, setProjectId] = useState<string>(() => {
    try {
      return localStorage.getItem("sign.projectId") ?? "";
    } catch {
      return "";
    }
  });
  const [placement, setPlacement] = useState<SignPlacement | null>(null);
  const [batch, setBatch] = useState<{ done: number; total: number; signed: number; bad: number } | null>(null);
  const [batchErrors, setBatchErrors] = useState<{ file: string; error: string }[]>([]);

  const V2he = useCallback(async () => {
    if (!isTauri()) {
      V2hd({
        kind: "info",
        text: "Chạy trong ứng dụng desktop để ký (chế độ trình duyệt không có token).",
      });
      return;
    }
    if (!projectId) {
      V2h0([]);
      return;
    }
    V2hb("jobs");
    try {
      V2h0(await V4qt(projectId));
      V2hd(null);
    } catch (V2hf) {
      V2hd({
        kind: "err",
        text: `Không lấy được hàng đợi ký: ${V4qx(V2hf)}`,
      });
    } finally {
      V2hb(null);
    }
  }, [projectId]);

  const V2hg = async () => {
    if (!isTauri()) return;
    try {
      const m = await V4qu();
      V2h4(m);
      V2h2((V2hh) => (V2hh.trim() === "" && m.length > 0 ? m[0] : V2hh));
    } catch {}
  };

  V41p(V2he, { topics: ["sign", "docs"] });

  useEffect(() => {
    void V2hg();
    if (isTauri()) {
      V4ak()
        .then((p) => setProjects(p.filter((x) => x.isActive)))
        .catch(() => {});
    }
  }, []);

  useEffect(() => {
    try {
      localStorage.setItem("sign.projectId", projectId);
    } catch {
      /* chỉ là tiện ích nhớ */
    }
    setPlacement(null);
    setBatchErrors([]);
    setBatch(null);
    void V2he();
  }, [projectId, V2he]);

  const V2hi = async () => {
    V2hb("tokens");
    try {
      await V2hg();
      const t = await V4qv(V2h1);
      V2h5(t);
      V2h7(t.length > 0 ? t[0].index : null);
      V2hd(
        t.length === 0
          ? {
              kind: "err",
              text:
                "Không thấy token nào đang cắm — kiểm tra USB token đã cắm chưa, và thư viện PKCS#11 " +
                "(phần mềm token của CA) đã cài chưa.",
            }
          : { kind: "ok", text: `Thấy ${t.length} token.` },
      );
    } catch (V2hj) {
      V2hd({ kind: "err", text: `Dò token lỗi: ${V4qx(V2hj)}` });
    } finally {
      V2hb(null);
    }
  };

  const ready = (): boolean => {
    if (V2h6 === null) {
      V2hd({ kind: "err", text: "Chọn token trước khi ký (bấm “Dò token”)." });
      return false;
    }
    if (!V2h8) {
      V2hd({ kind: "err", text: "Nhập mã PIN của token." });
      return false;
    }
    return true;
  };

  const sign = async (V2hk: V4qr) => {
    if (!ready()) return;
    V2hb(V2hk.id);
    try {
      const status = await V4qw({
        jobId: V2hk.id,
        modulePath: V2h1,
        slotIndex: V2h6!,
        pin: V2h8,
        placement,
      });
      if (status === "signed") {
        V2hd({ kind: "ok", text: `Đã ký “${V2hk.filename}”.` });
        V2h0((V2hl) => V2hl.filter((V2hm) => V2hm.id !== V2hk.id));
      } else {
        V2hd({
          kind: "err",
          text: `Máy chủ TỪ CHỐI chữ ký cho “${V2hk.filename}” (trạng thái: ${status}).`,
        });
        void V2he();
      }
    } catch (V2hn) {
      V2hd({ kind: "err", text: `Ký “${V2hk.filename}” lỗi: ${V4qx(V2hn)}` });
    } finally {
      V2hb(null);
    }
  };

  const signAll = async () => {
    if (!ready() || V2gz.length === 0) return;
    const project = projects.find((p) => p.id === projectId);
    const where = placement
      ? `${placement.lastPage ? "trang cuối" : "trang đầu"}, cách lề trái ${Math.round(placement.x * 100)}%, cách mép trên ${Math.round(placement.y * 100)}%`
      : "góc trên-phải trang đầu (mặc định)";
    if (
      !window.confirm(
        `Ký số ${V2gz.length} tài liệu của dự án “${project?.name ?? projectId}”?\n\nVị trí chữ ký: ${where}.\n\nChữ ký số không gỡ được sau khi ký.`,
      )
    )
      return;
    const byId = new Map(V2gz.map((j) => [j.id, j.filename]));
    const jobIds = V2gz.map((j) => j.id);
    setBatch({ done: 0, total: jobIds.length, signed: 0, bad: 0 });
    setBatchErrors([]);
    V2hb("batch");
    const unlisten = await onSignBatchProgress((p: SignBatchProgress) => {
      setBatch((b) =>
        b
          ? {
              done: p.done,
              total: p.total,
              signed: b.signed + (p.status === "signed" ? 1 : 0),
              bad: b.bad + (p.status === "signed" ? 0 : 1),
            }
          : b,
      );
      if (p.status === "signed") {
        V2h0((l) => l.filter((j) => j.id !== p.jobId));
      } else {
        setBatchErrors((e) => [...e.slice(-199), { file: byId.get(p.jobId) ?? p.jobId, error: p.error ?? p.status }]);
      }
    });
    try {
      const r: SignBatchResult = await signBatch({
        jobIds,
        modulePath: V2h1,
        slotIndex: V2h6!,
        pin: V2h8,
        placement,
      });
      V2hd({
        kind: r.failed + r.rejected > 0 || r.stoppedReason ? "err" : "ok",
        text:
          `Đã ký ${r.signed}/${r.total} tài liệu` +
          (r.rejected ? ` · máy chủ từ chối ${r.rejected}` : "") +
          (r.failed ? ` · lỗi ${r.failed}` : "") +
          (r.cancelled ? ` · chưa ký ${r.cancelled}` : "") +
          (r.stoppedReason ? ` — dừng: ${r.stoppedReason}` : "") +
          ".",
      });
    } catch (e) {
      V2hd({ kind: "err", text: `Ký hàng loạt lỗi: ${V4qx(e)}` });
    } finally {
      unlisten();
      V2hb(null);
      void V2he();
    }
  };

  const first = V2gz[0];

  return (
    <section className="view active">
      <div className="page-head">
        <h2>Ký số</h2>
        <div className="sub">· PAdES · USB Token · khoá không rời token · ký hàng loạt theo dự án</div>
      </div>

      {V2hc && (
        <div
          className={`banner ${V2hc.kind === "err" ? "error" : V2hc.kind === "ok" ? "success" : "info"}`}
        >
          <span className="bi">
            <Icon
              size={18}
              name={V2hc.kind === "err" ? "loi" : V2hc.kind === "ok" ? "thanhCong" : "thongTin"}
            />
          </span>
          <div className="bx">{V2hc.text}</div>
        </div>
      )}

      <div className="card mt-3.5">
        <h3>Token ký</h3>
        <div className="grid gap-2.5 mt-2 max-w-160">
          <label className="grid gap-1">
            <span className="muted text-sm">
              Đường dẫn module PKCS#11{" "}
              {V2h3.length > 0
                ? "(tự dò được, sửa nếu cần)"
                : "(để trống = tự dò)"}
            </span>
            <input
              value={V2h1}
              onChange={(V2ho) => V2h2(V2ho.target.value)}
              spellCheck={false}
              list="pkcs11-modules"
              placeholder="tự dò thư viện token trên máy"
            />
            <datalist id="pkcs11-modules">
              {V2h3.map((m) => (
                <option key={m} value={m} />
              ))}
            </datalist>
            {V2h3.length === 0 && (
              <span className="muted text-sm">
                Chưa thấy thư viện PKCS#11 nào — cài phần mềm token của CA (iCA SmartToken,
                Viettel-CA…) rồi bấm “Dò token”.
              </span>
            )}
          </label>
          <div className="flex gap-2 items-center flex-wrap">
            <button className="btn" onClick={V2hi} disabled={V2ha !== null}>
              {V2ha === "tokens" ? "Đang dò…" : "Dò token"}
            </button>
            {tokens.length > 0 && (
              <select value={V2h6 ?? ""} onChange={(V2hp) => V2h7(Number(V2hp.target.value))}>
                {tokens.map((t) => (
                  <option key={t.index} value={t.index}>
                    {t.label || `Token #${t.index}`}
                  </option>
                ))}
              </select>
            )}
          </div>
          <label className="grid gap-1 max-w-60">
            <span className="muted text-sm">Mã PIN</span>
            <input
              type="password"
              value={V2h8}
              onChange={(V2hq) => V2h9(V2hq.target.value)}
              autoComplete="off"
            />
          </label>
        </div>
      </div>

      <div className="card mt-3.5">
        <h3>Dự án cần ký</h3>
        <div className="flex gap-2 items-center flex-wrap mt-2">
          <select value={projectId} onChange={(e) => setProjectId(e.target.value)} disabled={V2ha === "batch"}>
            <option value="">— Chọn dự án —</option>
            {projects.map((p) => (
              <option key={p.id} value={p.id}>
                {p.code} · {p.name}
              </option>
            ))}
          </select>
          <button className="btn ghost" onClick={V2he} disabled={V2ha !== null || !projectId}>
            {V2ha === "jobs" ? "Đang tải…" : "Làm mới"}
          </button>
          {projectId && <span className="muted text-sm">{V2gz.length} tài liệu chờ ký</span>}
        </div>
      </div>

      {first && (
        <div className="card mt-3.5">
          <h3>Vị trí chữ ký (chọn trên tệp đầu tiên, áp cho cả lô)</h3>
          <div className="mt-2">
            <SignPlacementPicker
              jobId={first.id}
              filename={first.filename}
              value={placement}
              onChange={setPlacement}
            />
          </div>
          <div className="flex gap-2 items-center flex-wrap mt-3">
            <button className="btn primary" onClick={signAll} disabled={V2ha !== null}>
              {V2ha === "batch" ? "Đang ký…" : `Ký tất cả (${V2gz.length})`}
            </button>
            {V2ha === "batch" && (
              <button className="btn" onClick={() => void signBatchCancel()}>
                Dừng
              </button>
            )}
            {batch && (
              <span className="text-sm">
                {batch.done}/{batch.total} · đã ký {batch.signed}
                {batch.bad > 0 && <span className="text-danger"> · lỗi {batch.bad}</span>}
              </span>
            )}
          </div>
          {batch && (
            <div className="mt-2 h-2 bg-border-subtle rounded">
              <div
                className="h-2 rounded bg-accent"
                style={{ width: `${batch.total ? (batch.done / batch.total) * 100 : 0}%` }}
              />
            </div>
          )}
          {batchErrors.length > 0 && (
            <div className="mt-2 text-sm max-h-48 overflow-auto">
              {batchErrors.map((e, i) => (
                <div key={i} className="text-danger">
                  {e.file}: {e.error}
                </div>
              ))}
            </div>
          )}
        </div>
      )}

      <div className="card mt-3.5">
        <div className="flex justify-between items-center">
          <h3>Chờ ký ({V2gz.length})</h3>
        </div>
        {!projectId ? (
          <div className="muted mt-2.5">Chọn dự án để xem tài liệu chờ ký.</div>
        ) : V2gz.length === 0 ? (
          <div className="muted mt-2.5">Không có tài liệu nào đang chờ ký.</div>
        ) : (
          <div className="tbl-scroll">
            <table className="w-full mt-2.5 border-collapse">
              <thead>
                <tr className="text-left text-fg-muted text-sm">
                  <th className="py-1.5 px-1">Tài liệu</th>
                  <th className="py-1.5 px-1">Hồ sơ</th>
                  <th className="py-1.5 px-1">Số trang</th>
                  <th className="py-1.5 px-1" />
                </tr>
              </thead>
              <tbody>
                {V2gz.slice(0, TABLE_MAX_ROWS).map((V2hr) => (
                  <tr key={V2hr.id} className="border-t border-border-subtle">
                    <td className="py-2 px-1">{V2hr.filename}</td>
                    <td className="py-2 px-1 text-fg-muted">{V2hr.hoSoRef ?? "—"}</td>
                    <td className="py-2 px-1 text-fg-muted">{V2hr.pageCount ?? "—"}</td>
                    <td className="py-2 px-1 text-right">
                      <button className="btn" onClick={() => sign(V2hr)} disabled={V2ha !== null}>
                        {V2ha === V2hr.id ? "Đang ký…" : "Ký"}
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
            {V2gz.length > TABLE_MAX_ROWS && (
              <div className="muted text-sm mt-2">
                Hiện {TABLE_MAX_ROWS}/{V2gz.length} dòng — “Ký tất cả” ký đủ {V2gz.length} tài liệu.
              </div>
            )}
          </div>
        )}
      </div>
    </section>
  );
}
