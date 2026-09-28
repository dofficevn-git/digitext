// Copyright (c) 2026 DigiText. All rights reserved.
//
// Ký số hàng loạt theo dự án — bố cục theo mẫu tool ký trong đề nghị chỉnh sửa (mục 5):
// Đầu vào (dự án/phông) · Đầu ra (thư mục lưu bản đã ký) · Token + chứng thư (thumbprint, đơn vị) ·
// Hình thức ký (có/không dấu, ảnh con dấu, dấu thời gian) · Bắt đầu / Tạm dừng / Huỷ bỏ ·
// thời gian đã chạy + ước còn · nhật ký · danh sách lỗi kèm đường dẫn + ký lại.

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { V41p } from "@/live/useLive";

import { isTauri, V3xq } from "@/api/ipc";
import {
  V4qu,
  V4qt,
  V4qv,
  onSignBatchProgress,
  signBatch,
  signBatchCancel,
  signBatchPause,
  signCertInfo,
  signLoadStamp,
  signRetry,
  type SignBatchProgress,
  type SignBatchResult,
  type SignCertInfo,
  type SignPlacement,
  type V4qr,
  type V4qs,
} from "@/api/sign";
import { V4el } from "@/api/docs";
import { V4ak, type V4a9 } from "@/api/users";
import { V4e3 } from "@/features/dossier/excel";
import { Icon } from "@/components/Icon";
import SignPlacementPicker from "./SignPlacementPicker";

function V4qx(V2gy: unknown): string {
  return V2gy instanceof V3xq || V2gy instanceof Error ? V2gy.message : String(V2gy);
}

// Bảng chỉ vẽ tối đa ngần này dòng (lô có thể hàng nghìn tài liệu) — ký hàng loạt vẫn ký hết.
const TABLE_MAX_ROWS = 200;
const LOG_MAX = 500;

/** Nhớ lựa chọn của người ký giữa các lần mở (chỉ tiện ích — lỗi thì bỏ qua). */
function nho<T>(key: string, fallback: T): T {
  try {
    const v = localStorage.getItem(key);
    return v == null ? fallback : (JSON.parse(v) as T);
  } catch {
    return fallback;
  }
}
function luu(key: string, v: unknown): void {
  try {
    localStorage.setItem(key, JSON.stringify(v));
  } catch {
    /* chỉ là tiện ích nhớ */
  }
}

const phongOf = (j: V4qr): string => {
  const parts = (j.relPath ?? "").split("/").filter((s) => s.trim() !== "");
  return parts.length >= 2 ? parts[0].trim() : "";
};
const duongDan = (j: V4qr): string => j.relPath || j.hoSoRef || "";

function hhmmss(ms: number): string {
  const s = Math.max(0, Math.round(ms / 1000));
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  return `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}:${String(s % 60).padStart(2, "0")}`;
}

interface LoiKy {
  id: string;
  filename: string;
  path: string;
  error: string;
}
interface DongNhatKy {
  at: string;
  text: string;
  kind: "ok" | "err" | "info";
}

export default function V4qy() {
  const [V2gz, V2h0] = useState<V4qr[]>([]);

  const [V2h1, V2h2] = useState("");
  const [V2h3, V2h4] = useState<string[]>([]);
  const [tokens, V2h5] = useState<V4qs[]>([]);
  const [V2h6, V2h7] = useState<number | null>(null);
  const [V2h8, V2h9] = useState("");
  const [V2ha, V2hb] = useState<string | null>(null);
  const [V2hc, V2hd] = useState<{ kind: "info" | "err" | "ok"; text: string } | null>(null);
  const [cert, setCert] = useState<SignCertInfo | null>(null);

  const [projects, setProjects] = useState<V4a9[]>([]);
  const [projectId, setProjectId] = useState<string>(() => {
    try {
      return localStorage.getItem("sign.projectId") ?? "";
    } catch {
      return "";
    }
  });
  const [phongLoc, setPhongLoc] = useState("");
  const [outDir, setOutDir] = useState<string>(() => nho("sign.outDir", ""));
  const [withStamp, setWithStamp] = useState<boolean>(() => nho("sign.withStamp", true));
  const [stampPath, setStampPath] = useState<string>(() => nho("sign.stampPath", ""));
  const [stampB64, setStampB64] = useState<string | null>(null);
  const [timestamp, setTimestamp] = useState<boolean>(() => nho("sign.timestamp", false));

  const [placement, setPlacement] = useState<SignPlacement | null>(null);
  const [batch, setBatch] = useState<{ done: number; total: number; signed: number; bad: number } | null>(null);
  const [startedAt, setStartedAt] = useState<number | null>(null);
  const [now, setNow] = useState(Date.now());
  const [paused, setPaused] = useState(false);
  const [loi, setLoi] = useState<LoiKy[]>([]);
  const [nhatKy, setNhatKy] = useState<DongNhatKy[]>([]);
  const taiVe = useRef<Promise<void>>(Promise.resolve());

  const ghi = (text: string, kind: DongNhatKy["kind"] = "info") =>
    setNhatKy((l) => [...l.slice(-(LOG_MAX - 1)), { at: new Date().toLocaleTimeString("vi-VN"), text, kind }]);

  useEffect(() => luu("sign.outDir", outDir), [outDir]);
  useEffect(() => luu("sign.withStamp", withStamp), [withStamp]);
  useEffect(() => luu("sign.stampPath", stampPath), [stampPath]);
  useEffect(() => luu("sign.timestamp", timestamp), [timestamp]);

  // Nạp lại ảnh dấu đã chọn lần trước.
  useEffect(() => {
    if (!isTauri() || !stampPath) {
      setStampB64(null);
      return;
    }
    signLoadStamp(stampPath)
      .then(setStampB64)
      .catch(() => setStampB64(null));
  }, [stampPath]);

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
    V2hb((b) => b ?? "jobs");
    try {
      V2h0(await V4qt(projectId));
      // KHÔNG xoá thông báo ở đây: sau khi ký xong màn tự làm mới danh sách — xoá thì kết quả /
      // lỗi (vd. sai PIN) vừa hiện biến mất ngay, người dùng không biết chuyện gì xảy ra.
    } catch (V2hf) {
      V2hd({
        kind: "err",
        text: `Không lấy được hàng đợi ký: ${V4qx(V2hf)}`,
      });
    } finally {
      V2hb((b) => (b === "jobs" ? null : b));
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
    setPhongLoc("");
    setLoi([]);
    setBatch(null);
    void V2he();
  }, [projectId, V2he]);

  // Đồng hồ "đã chạy / ước còn" khi đang ký.
  useEffect(() => {
    if (V2ha !== "batch") return;
    const t = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(t);
  }, [V2ha]);

  const dsPhong = useMemo(() => {
    const s = new Set<string>();
    for (const j of V2gz) {
      const p = phongOf(j);
      if (p) s.add(p);
    }
    return [...s].sort(new Intl.Collator("vi", { numeric: true }).compare);
  }, [V2gz]);
  const jobs = useMemo(
    () => (phongLoc ? V2gz.filter((j) => phongOf(j) === phongLoc) : V2gz),
    [V2gz, phongLoc],
  );

  const V2hi = async () => {
    V2hb("tokens");
    try {
      await V2hg();
      const t = await V4qv(V2h1);
      V2h5(t);
      V2h7(t.length > 0 ? t[0].index : null);
      setCert(null);
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
    if (withStamp && stampPath && !stampB64) {
      V2hd({ kind: "err", text: "Ảnh con dấu chưa đọc được — chọn lại ảnh hoặc chọn “Ký không dấu”." });
      return false;
    }
    return true;
  };

  const xemChungThu = async () => {
    if (!ready()) return;
    V2hb("cert");
    try {
      const c = await signCertInfo({ modulePath: V2h1, slotIndex: V2h6!, pin: V2h8 });
      setCert(c);
      V2hd({ kind: "ok", text: `PIN đúng — chứng thư của ${c.subject}.` });
    } catch (e) {
      setCert(null);
      V2hd({ kind: "err", text: `Đọc chứng thư lỗi: ${V4qx(e)}` });
    } finally {
      V2hb(null);
    }
  };

  const chonDauRa = async () => {
    const d = await openDialog({ directory: true, multiple: false, title: "Chọn thư mục lưu bản đã ký" });
    if (typeof d === "string") setOutDir(d);
  };
  const chonConDau = async () => {
    const f = await openDialog({
      multiple: false,
      title: "Chọn ảnh con dấu (PNG nền trong)",
      filters: [{ name: "Ảnh", extensions: ["png", "jpg", "jpeg"] }],
    });
    if (typeof f !== "string") return;
    try {
      setStampB64(await signLoadStamp(f));
      setStampPath(f);
    } catch (e) {
      V2hd({ kind: "err", text: V4qx(e) });
    }
  };

  const runBatch = async (list: V4qr[], nhan: string) => {
    if (!ready() || list.length === 0) return;
    const project = projects.find((p) => p.id === projectId);
    const where = placement
      ? `${placement.lastPage ? "trang cuối" : "trang đầu"}, cách lề trái ${Math.round(placement.x * 100)}%, cách mép trên ${Math.round(placement.y * 100)}%`
      : "góc trên-phải trang đầu (mặc định)";
    const kieu = withStamp ? (stampB64 ? "có dấu (ảnh con dấu đã chọn)" : "có dấu") : "không dấu";
    if (
      !window.confirm(
        `${nhan} ${list.length} tài liệu của dự án “${project?.name ?? projectId}”${phongLoc ? ` · phông ${phongLoc}` : ""}?\n\n` +
          `Vị trí chữ ký: ${where}.\nHình thức: ${kieu}; dấu thời gian: ${timestamp ? "có" : "không"}.\n` +
          `Đầu ra: ${outDir || "chỉ lưu trên máy chủ"}.\n\nChữ ký số không gỡ được sau khi ký.`,
      )
    )
      return;
    const byId = new Map(list.map((j) => [j.id, j]));
    const jobIds = list.map((j) => j.id);
    setBatch({ done: 0, total: jobIds.length, signed: 0, bad: 0 });
    setStartedAt(Date.now());
    setNow(Date.now());
    setPaused(false);
    setLoi([]);
    ghi(`Bắt đầu: ${nhan.toLowerCase()} ${jobIds.length} tài liệu.`);
    V2hb("batch");
    const unlisten = await onSignBatchProgress((p: SignBatchProgress) => {
      const j = byId.get(p.jobId);
      const ten = j?.filename ?? p.jobId;
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
        V2h0((l) => l.filter((x) => x.id !== p.jobId));
        ghi(`Đã ký ${ten}`, "ok");
        if (outDir && j) {
          // Tải bản đã ký về thư mục đầu ra theo đúng cây phông/hộp/hồ sơ (lần lượt từng tệp).
          taiVe.current = taiVe.current.then(() =>
            V4el(j.id, j.filename, j.relPath ?? undefined, false, outDir, "signed")
              .then(() => undefined)
              .catch((e) => ghi(`Tải bản đã ký ${ten} về đầu ra lỗi: ${V4qx(e)}`, "err")),
          );
        }
      } else {
        const error = p.error ?? p.status;
        setLoi((e) => [...e, { id: p.jobId, filename: ten, path: j ? duongDan(j) : "", error }]);
        ghi(`LỖI ${ten}${j ? ` (${duongDan(j)})` : ""}: ${error}`, "err");
      }
    });
    try {
      const r: SignBatchResult = await signBatch({
        jobIds,
        modulePath: V2h1,
        slotIndex: V2h6!,
        pin: V2h8,
        placement,
        style: { withStamp, stampPng: withStamp ? stampB64 : null, timestamp },
      });
      const text =
        `Đã ký ${r.signed}/${r.total} tài liệu` +
        (r.rejected ? ` · máy chủ từ chối ${r.rejected}` : "") +
        (r.failed ? ` · lỗi ${r.failed}` : "") +
        (r.cancelled ? ` · chưa ký ${r.cancelled}` : "") +
        (r.stoppedReason ? ` — dừng: ${r.stoppedReason}` : "") +
        ".";
      V2hd({ kind: r.failed + r.rejected > 0 || r.stoppedReason ? "err" : "ok", text });
      ghi(text, r.failed + r.rejected > 0 ? "err" : "ok");
    } catch (e) {
      V2hd({ kind: "err", text: `Ký hàng loạt lỗi: ${V4qx(e)}` });
      ghi(`Ký hàng loạt lỗi: ${V4qx(e)}`, "err");
    } finally {
      unlisten();
      V2hb(null);
      setPaused(false);
      await taiVe.current;
      if (outDir) ghi(`Bản đã ký đã lưu vào ${outDir}.`);
      void V2he();
    }
  };

  const tamDung = async () => {
    const moi = !paused;
    await signBatchPause(moi);
    setPaused(moi);
    ghi(moi ? "Tạm dừng — ký xong tệp đang dở thì chờ." : "Tiếp tục ký.");
  };

  const kyLaiLoi = async () => {
    if (loi.length === 0) return;
    V2hb("retry");
    try {
      const ids = loi.map((e) => e.id);
      const r = await signRetry(ids);
      ghi(`Đưa ${r.requeued} tài liệu lỗi về hàng chờ ký${r.skipped ? ` (bỏ qua ${r.skipped})` : ""}.`);
      const list = await V4qt(projectId);
      V2h0(list);
      const set = new Set(ids);
      V2hb(null);
      await runBatch(
        list.filter((j) => set.has(j.id)),
        "Ký lại",
      );
    } catch (e) {
      V2hd({ kind: "err", text: `Ký lại lỗi: ${V4qx(e)}` });
      V2hb(null);
    }
  };

  const xuatLoi = () =>
    void V4e3(
      loi.map((e, i) => ({ STT: i + 1, "Tên file": e.filename, "Đường dẫn": e.path, "Lý do": e.error })),
      "Lỗi ký số",
      `loi-ky-so-${new Date().toISOString().slice(0, 10)}.xlsx`,
    );

  const first = jobs[0];
  const running = V2ha === "batch";
  const elapsed = startedAt ? now - startedAt : 0;
  const eta =
    batch && batch.done > 0 && batch.done < batch.total ? (elapsed / batch.done) * (batch.total - batch.done) : null;

  return (
    <section className="view active">
      <div className="page-head">
        <h2>Ký số</h2>
        <div className="sub">· PAdES · USB Token · ký hàng loạt theo dự án</div>
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
        <div className="grid gap-3 mt-1" style={{ gridTemplateColumns: "140px 1fr" }}>
          <span className="muted text-sm self-center">Đầu vào</span>
          <div className="flex gap-2 items-center flex-wrap">
            <select value={projectId} onChange={(e) => setProjectId(e.target.value)} disabled={V2ha !== null}>
              <option value="">— Chọn dự án —</option>
              {projects.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.code} · {p.name}
                </option>
              ))}
            </select>
            {dsPhong.length > 0 && (
              <select value={phongLoc} onChange={(e) => setPhongLoc(e.target.value)} disabled={V2ha !== null}>
                <option value="">— Tất cả phông ({dsPhong.length}) —</option>
                {dsPhong.map((p) => (
                  <option key={p} value={p}>
                    {p}
                  </option>
                ))}
              </select>
            )}
            <button className="btn ghost" onClick={V2he} disabled={V2ha !== null || !projectId}>
              {V2ha === "jobs" ? "Đang tải…" : "Làm mới"}
            </button>
            {projectId && <span className="muted text-sm">{jobs.length.toLocaleString("vi-VN")} tài liệu chờ ký</span>}
          </div>

          <span className="muted text-sm self-center">Đầu ra</span>
          <div className="flex gap-2 items-center flex-wrap">
            <input
              className="flex-1 min-w-80"
              value={outDir}
              onChange={(e) => setOutDir(e.target.value)}
              placeholder="(để trống = chỉ lưu bản đã ký trên máy chủ)"
              spellCheck={false}
            />
            <button className="btn" onClick={() => void chonDauRa()} disabled={running}>
              Chọn
            </button>
          </div>

          <span className="muted text-sm self-center">Token</span>
          <div className="flex gap-2 items-center flex-wrap">
            <input
              className="min-w-80"
              value={V2h1}
              onChange={(V2ho) => V2h2(V2ho.target.value)}
              spellCheck={false}
              list="pkcs11-modules"
              placeholder="thư viện PKCS#11 — để trống = tự dò"
              title="Thư viện PKCS#11 của phần mềm token (iCA, Viettel-CA…)"
            />
            <datalist id="pkcs11-modules">
              {V2h3.map((m) => (
                <option key={m} value={m} />
              ))}
            </datalist>
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
            <input
              type="password"
              className="max-w-40"
              value={V2h8}
              onChange={(V2hq) => {
                V2h9(V2hq.target.value);
                setCert(null);
              }}
              autoComplete="off"
              placeholder="Mã PIN"
            />
          </div>

          <span className="muted text-sm self-center">Thumbprint</span>
          <div className="flex gap-2 items-center flex-wrap">
            <input className="flex-1 min-w-80" readOnly value={cert?.thumbprint ?? ""} placeholder="(bấm “Kiểm tra” để đọc chứng thư trên token)" />
            <button className="btn" onClick={() => void xemChungThu()} disabled={V2ha !== null}>
              {V2ha === "cert" ? "Đang đọc…" : "Kiểm tra"}
            </button>
          </div>

          <span className="muted text-sm self-center">Tên đơn vị</span>
          <input
            readOnly
            value={cert ? [cert.subject, cert.organization].filter(Boolean).join(" — ") : ""}
            title={cert ? `Cấp bởi ${cert.issuer} · số hiệu ${cert.serial} · hết hạn ${cert.notAfter}` : undefined}
          />

          <span className="muted text-sm self-center">Hình thức ký</span>
          <div className="flex gap-4 items-center flex-wrap">
            <label className="flex items-center gap-1.5">
              <input type="radio" checked={withStamp} onChange={() => setWithStamp(true)} disabled={running} /> Ký có dấu
            </label>
            <label className="flex items-center gap-1.5">
              <input type="radio" checked={!withStamp} onChange={() => setWithStamp(false)} disabled={running} /> Ký không dấu
            </label>
          </div>

          {withStamp && (
            <>
              <span className="muted text-sm self-center">Con dấu</span>
              <div className="flex gap-2 items-center flex-wrap">
                <input className="flex-1 min-w-80" readOnly value={stampPath} placeholder="(không chọn ảnh = dấu đỏ dựng từ chứng thư)" />
                <button className="btn" onClick={() => void chonConDau()} disabled={running}>
                  Chọn
                </button>
                {stampPath && (
                  <button className="btn ghost" onClick={() => setStampPath("")} disabled={running}>
                    Bỏ ảnh
                  </button>
                )}
                {stampB64 && (
                  <img
                    src={`data:image/png;base64,${stampB64}`}
                    alt="Ảnh con dấu"
                    style={{ height: 48, width: "auto", border: "1px solid var(--border-subtle, #ddd)" }}
                  />
                )}
              </div>
            </>
          )}

          <span className="muted text-sm self-center">Xác thực thời gian (TSA)</span>
          <div className="flex gap-4 items-center flex-wrap">
            <label className="flex items-center gap-1.5">
              <input type="radio" checked={timestamp} onChange={() => setTimestamp(true)} disabled={running} /> Có
            </label>
            <label className="flex items-center gap-1.5">
              <input type="radio" checked={!timestamp} onChange={() => setTimestamp(false)} disabled={running} /> Không
            </label>
            <span className="muted text-sm">Dấu thời gian do máy chủ lấy từ dịch vụ TSA đã cấu hình.</span>
          </div>
        </div>

        <div className="flex gap-2 items-center flex-wrap mt-4">
          <button className="btn primary" onClick={() => void runBatch(jobs, "Ký số")} disabled={V2ha !== null || jobs.length === 0}>
            <Icon name="chay" /> Bắt đầu ({jobs.length.toLocaleString("vi-VN")})
          </button>
          <button className="btn" onClick={() => void tamDung()} disabled={!running}>
            {paused ? "Tiếp tục" : "Tạm dừng"}
          </button>
          <button className="btn" onClick={() => void signBatchCancel()} disabled={!running}>
            Huỷ bỏ
          </button>
          {batch && (
            <span className="text-sm">
              {running && batch.done === 0
                ? "Đang mở token và kiểm PIN…"
                : `${batch.done.toLocaleString("vi-VN")}/${batch.total.toLocaleString("vi-VN")} · đã ký ${batch.signed.toLocaleString("vi-VN")}`}
              {batch.bad > 0 && <span className="text-danger"> · lỗi {batch.bad}</span>}
              {paused && <b> · đang tạm dừng</b>}
            </span>
          )}
        </div>
        {batch && (
          <>
            <div className="mt-2 h-2 bg-border-subtle rounded">
              <div
                className="h-2 rounded bg-accent"
                style={{ width: `${batch.total ? (batch.done / batch.total) * 100 : 0}%` }}
              />
            </div>
            <div className="flex justify-between text-sm mt-1">
              <span>Ước còn: {eta != null ? hhmmss(eta) : "--"}</span>
              <span>Đã chạy: {hhmmss(elapsed)}</span>
            </div>
          </>
        )}
        {nhatKy.length > 0 && (
          <div
            className="mt-2 text-sm max-h-48 overflow-auto border border-border-subtle rounded p-2"
            style={{ fontFamily: "ui-monospace, monospace" }}
          >
            {nhatKy.map((d, i) => (
              <div key={i} className={d.kind === "err" ? "text-danger" : d.kind === "ok" ? "text-success-700" : undefined}>
                [{d.at}] {d.text}
              </div>
            ))}
          </div>
        )}
      </div>

      {loi.length > 0 && (
        <div className="card mt-3.5">
          <div className="flex gap-2 items-center flex-wrap">
            <h3 className="m-0">File ký lỗi ({loi.length})</h3>
            <div className="flex-1" />
            <button className="btn primary" onClick={() => void kyLaiLoi()} disabled={V2ha !== null}>
              {V2ha === "retry" ? "Đang chuẩn bị…" : `Ký lại các file lỗi (${loi.length})`}
            </button>
            <button className="btn" onClick={xuatLoi}>
              <Icon name="tepExcel" /> Xuất danh sách lỗi
            </button>
          </div>
          <div className="tbl-scroll mt-2">
            <table className="w-full border-collapse">
              <thead>
                <tr className="text-left text-fg-muted text-sm">
                  <th className="py-1.5 px-1">Tên file</th>
                  <th className="py-1.5 px-1">Đường dẫn (phông / hộp / hồ sơ)</th>
                  <th className="py-1.5 px-1">Lý do</th>
                </tr>
              </thead>
              <tbody>
                {loi.slice(0, TABLE_MAX_ROWS).map((e) => (
                  <tr key={e.id} className="border-t border-border-subtle">
                    <td className="py-2 px-1">{e.filename}</td>
                    <td className="py-2 px-1 text-fg-muted">{e.path || "—"}</td>
                    <td className="py-2 px-1 text-danger">{e.error}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {first && (
        <div className="card mt-3.5">
          <h3>Vị trí chữ ký (chọn trên tệp đầu tiên, áp cho cả lô — mặc định góc trên bên phải)</h3>
          <div className="mt-2">
            <SignPlacementPicker
              jobId={first.id}
              filename={first.filename}
              value={placement}
              onChange={setPlacement}
            />
          </div>
        </div>
      )}

      <div className="card mt-3.5">
        <h3>Chờ ký ({jobs.length.toLocaleString("vi-VN")})</h3>
        {!projectId ? (
          <div className="muted mt-2.5">Chọn dự án để xem tài liệu chờ ký.</div>
        ) : jobs.length === 0 ? (
          <div className="muted mt-2.5">Không có tài liệu nào đang chờ ký.</div>
        ) : (
          <div className="tbl-scroll">
            <table className="w-full mt-2.5 border-collapse">
              <thead>
                <tr className="text-left text-fg-muted text-sm">
                  <th className="py-1.5 px-1">Tài liệu</th>
                  <th className="py-1.5 px-1">Đường dẫn</th>
                  <th className="py-1.5 px-1">Số trang</th>
                </tr>
              </thead>
              <tbody>
                {jobs.slice(0, TABLE_MAX_ROWS).map((V2hr) => (
                  <tr key={V2hr.id} className="border-t border-border-subtle">
                    <td className="py-2 px-1">{V2hr.filename}</td>
                    <td className="py-2 px-1 text-fg-muted">{duongDan(V2hr) || "—"}</td>
                    <td className="py-2 px-1 text-fg-muted">{V2hr.pageCount ?? "—"}</td>
                  </tr>
                ))}
              </tbody>
            </table>
            {jobs.length > TABLE_MAX_ROWS && (
              <div className="muted text-sm mt-2">
                Hiện {TABLE_MAX_ROWS}/{jobs.length.toLocaleString("vi-VN")} dòng — “Bắt đầu” ký đủ{" "}
                {jobs.length.toLocaleString("vi-VN")} tài liệu.
              </div>
            )}
          </div>
        )}
      </div>
    </section>
  );
}
