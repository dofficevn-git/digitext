// Copyright (c) 2026 DigiText. All rights reserved.
//
// CTV: "Việc của tôi" — gói được giao (còn N việc), hộp thư mục bị trả; màn làm việc: ảnh + 18 trường
// (OcrFieldsView), Trước / Tiếp / Xác nhận hoàn thành (Ctrl+Enter); gói kiểm tra: Đạt / Có lỗi.
// Mỗi lúc một mục, khoá mềm 10 phút (máy chủ). Kế hoạch 29/9/2026 (từ doc_extract_pla).

import { useCallback, useEffect, useState } from "react";
import { isTauri, V3xq } from "@/api/ipc";
import {
  ITEM_STATUS_LABEL,
  KIND_LABEL,
  workCheck,
  workClaimItem,
  workDone,
  workGetPackage,
  workMyPackages,
  workNavigate,
  workNextItem,
  workRejectedItems,
  workRelease,
  type WorkItem,
  type WorkPackage,
} from "@/api/work";
import V4i7 from "@/features/upload/OcrFieldsView";
import { Icon } from "@/components/Icon";

const n = (v: number) => v.toLocaleString("vi-VN");
const errText = (e: unknown) => (e instanceof V3xq || e instanceof Error ? e.message : String(e));
const ngay = (s: string | null) => (s ? s.split("-").reverse().join("/") : "—");
const conViec = (p: WorkPackage) => (p.kind === "kiem_tra" ? p.nPending : p.nPending + p.nRejected);

function LamViec({ pkg, startItem, onBack }: { pkg: WorkPackage; startItem?: WorkItem | null; onBack: () => void }) {
  const [item, setItem] = useState<WorkItem | null | undefined>(undefined);
  const [total, setTotal] = useState(pkg.nTotal);
  const [con, setCon] = useState(conViec(pkg));
  const [msg, setMsg] = useState<{ kind: "ok" | "err" | "info"; text: string } | null>(null);
  const [busy, setBusy] = useState(false);
  const [now, setNow] = useState(Date.now());
  const [note, setNote] = useState("");

  const capNhatDem = useCallback(async () => {
    try {
      const d = await workGetPackage(pkg.id);
      setTotal(d.nTotal);
      setCon(conViec(d));
    } catch {}
  }, [pkg.id]);

  const nhan = useCallback(async () => {
    setBusy(true);
    setMsg(null);
    try {
      const it = await workNextItem(pkg.id);
      setItem(it);
      setNote("");
      if (!it) setMsg({ kind: "info", text: "Hết việc trong gói này (hoặc mục còn lại đang do người khác làm)." });
    } catch (e) {
      setMsg({ kind: "err", text: errText(e) });
      setItem(null);
    } finally {
      setBusy(false);
    }
  }, [pkg.id]);

  useEffect(() => {
    if (startItem) {
      // Mở thẳng mục bị trả: nhận đích danh mục đó; không được (người khác đang giữ) thì nhận mục kế.
      setBusy(true);
      void workClaimItem(startItem.id)
        .then((it) => {
          if (it) {
            setItem(it);
            setNote("");
            setBusy(false);
          } else return nhan();
        })
        .catch(() => nhan());
    } else {
      void nhan();
    }
  }, [nhan, startItem]);

  useEffect(() => {
    const t = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(t);
  }, []);

  const chuyen = async (dir: "prev" | "next") => {
    if (!item) return;
    setBusy(true);
    setMsg(null);
    try {
      const it = await workNavigate(item.id, dir);
      if (it) {
        setItem(it);
        setNote("");
      } else setMsg({ kind: "info", text: dir === "next" ? "Đã tới mục cuối còn làm được." : "Đã tới mục đầu." });
    } catch (e) {
      setMsg({ kind: "err", text: errText(e) });
    } finally {
      setBusy(false);
    }
  };

  const xacNhan = async () => {
    if (!item || busy) return;
    setBusy(true);
    setMsg(null);
    try {
      await workDone(item.id);
      setMsg({ kind: "ok", text: `Đã xác nhận ${item.filename}.` });
      await capNhatDem();
      await nhan();
    } catch (e) {
      setMsg({ kind: "err", text: errText(e) });
      setBusy(false);
    }
  };

  const cham = async (ok: boolean) => {
    if (!item || busy) return;
    if (!ok && !note.trim()) {
      setMsg({ kind: "err", text: "Ghi lỗi gì vào ô ghi chú trước khi chấm Có lỗi." });
      return;
    }
    setBusy(true);
    setMsg(null);
    try {
      await workCheck(item.id, ok, note.trim() || undefined);
      setMsg({ kind: "ok", text: `Đã chấm ${ok ? "Đạt" : "Có lỗi"}: ${item.filename}.` });
      await capNhatDem();
      await nhan();
    } catch (e) {
      setMsg({ kind: "err", text: errText(e) });
      setBusy(false);
    }
  };

  useEffect(() => {
    const h = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.key === "Enter") {
        e.preventDefault();
        if (pkg.kind === "xac_nhan") void xacNhan();
        else void cham(true);
      }
    };
    window.addEventListener("keydown", h);
    return () => window.removeEventListener("keydown", h);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [item, busy, note, pkg.kind]);

  const thoat = () => {
    if (item) void workRelease(item.id).catch(() => null);
    onBack();
  };

  const conKhoa = item?.leaseUntil ? Math.max(0, Math.round((new Date(item.leaseUntil).getTime() - now) / 1000)) : null;

  return (
    <div className="mt-3.5">
      <div className="card mb-3 flex items-center gap-3 flex-wrap">
        <button className="btn" onClick={thoat}>
          ← Gói của tôi
        </button>
        <b>{pkg.name}</b>
        <span className="badge gray">{KIND_LABEL[pkg.kind]}</span>
        <span className="text-sm">
          Còn <b>{n(con)}</b>/{n(total)} mục
        </span>
        {item && (
          <span className="muted text-sm">
            · mục #{item.ord} · {item.relPath ?? item.hoSoRef ?? ""} · <span className="badge gray">{ITEM_STATUS_LABEL[item.status]}</span>
            {conKhoa != null && ` · khoá còn ${Math.floor(conKhoa / 60)}:${String(conKhoa % 60).padStart(2, "0")}`}
          </span>
        )}
        {item?.status === "rejected" && item.reviewNote && (
          <span className="text-danger text-sm">
            <Icon name="canhBao" size={14} /> Bị trả: {item.reviewNote}
          </span>
        )}
        <div className="flex-1" />
        {msg && <span className={`text-sm ${msg.kind === "err" ? "text-danger" : msg.kind === "ok" ? "text-success-700" : "muted"}`}>{msg.text}</span>}
      </div>
      {item === undefined ? (
        <div className="card muted">Đang nhận việc…</div>
      ) : item === null ? (
        <div className="card">
          <p className="muted">{msg?.text ?? "Không còn mục nào."}</p>
          <button className="btn" onClick={() => void nhan()}>
            <Icon name="lamMoi" /> Thử nhận lại
          </button>
        </div>
      ) : (
        <V4i7
          key={item.id}
          doc={{ id: item.documentId, filename: item.filename, reviewStatus: "pending" }}
          onBack={thoat}
          workMode
          footer={
            <>
              <button className="btn" onClick={() => void chuyen("prev")} disabled={busy}>
                ◀ Trước
              </button>
              <button className="btn" onClick={() => void chuyen("next")} disabled={busy}>
                Tiếp ▶
              </button>
              {pkg.kind === "xac_nhan" ? (
                <button className="btn primary" onClick={() => void xacNhan()} disabled={busy} title="Ctrl+Enter">
                  <Icon name="duyet" /> Xác nhận hoàn thành
                </button>
              ) : (
                <>
                  <input className="inp max-w-60" placeholder="ghi lỗi (nếu có)" value={note} onChange={(e) => setNote(e.target.value)} />
                  <button className="btn primary" onClick={() => void cham(true)} disabled={busy} title="Ctrl+Enter">
                    Đạt
                  </button>
                  <button className="btn" onClick={() => void cham(false)} disabled={busy}>
                    Có lỗi
                  </button>
                </>
              )}
            </>
          }
        />
      )}
    </div>
  );
}

export default function MyWorkView() {
  const [pkgs, setPkgs] = useState<WorkPackage[]>([]);
  const [rejected, setRejected] = useState<WorkItem[]>([]);
  const [err, setErr] = useState<string | null>(null);
  const [lam, setLam] = useState<{ pkg: WorkPackage; item?: WorkItem } | null>(null);

  const load = useCallback(async () => {
    if (!isTauri()) return;
    setErr(null);
    try {
      const [p, r] = await Promise.all([workMyPackages(), workRejectedItems()]);
      setPkgs(p);
      setRejected(r);
    } catch (e) {
      setErr(errText(e));
    }
  }, []);
  useEffect(() => {
    void load();
  }, [load]);

  if (lam) {
    return (
      <section className="view active">
        <div className="page-head">
          <h2>Việc của tôi</h2>
        </div>
        <LamViec pkg={lam.pkg} startItem={lam.item} onBack={() => { setLam(null); void load(); }} />
      </section>
    );
  }

  return (
    <section className="view active">
      <div className="page-head">
        <h2>Việc của tôi</h2>
        <div className="sub">· gói xác nhận trường được giao</div>
      </div>
      {err && <div className="banner error mt-3"><div className="bx">{err}</div></div>}
      {rejected.length > 0 && (
        <div className="card mt-3.5">
          <h3 className="mt-0">
            <Icon name="canhBao" /> Mục bị trả lại ({rejected.length})
          </h3>
          <div className="tbl-scroll">
            <table className="tbl">
              <thead>
                <tr>
                  <th>Tài liệu</th>
                  <th>Đường dẫn</th>
                  <th>Lý do</th>
                  <th />
                </tr>
              </thead>
              <tbody>
                {rejected.map((i) => {
                  const pkg = pkgs.find((p) => p.id === i.packageId);
                  return (
                    <tr key={i.id}>
                      <td>{i.filename}</td>
                      <td className="muted">{i.relPath ?? i.hoSoRef ?? "—"}</td>
                      <td className="text-danger">{i.reviewNote}</td>
                      <td>
                        {pkg && (
                          <button className="btn sm primary" onClick={() => setLam({ pkg, item: i })}>
                            Làm lại
                          </button>
                        )}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        </div>
      )}
      <div className="card mt-3.5">
        <div className="flex items-center gap-2">
          <h3 className="m-0">Gói được giao</h3>
          <div className="flex-1" />
          <button className="btn sm" onClick={() => void load()}>
            <Icon name="lamMoi" /> Làm mới
          </button>
        </div>
        {pkgs.length === 0 ? (
          <p className="muted mt-2">Chưa được giao gói nào. Khi quản trị giao việc, gói sẽ hiện ở đây.</p>
        ) : (
          <div className="grid gap-3 mt-3" style={{ gridTemplateColumns: "repeat(auto-fill, minmax(280px, 1fr))" }}>
            {pkgs.map((p) => {
              const con = conViec(p);
              const qh = !!p.dueAt && new Date(`${p.dueAt}T23:59:59`).getTime() < Date.now() && con > 0;
              return (
                <div key={p.id} className="border border-border-subtle rounded p-3">
                  <div className="flex items-center gap-2">
                    <b>{p.name}</b>
                    <span className="badge gray">{KIND_LABEL[p.kind]}</span>
                  </div>
                  <div className={`text-sm mt-1 ${qh ? "text-danger" : "muted"}`}>Hạn: {ngay(p.dueAt)}{qh ? " — QUÁ HẠN" : ""}</div>
                  <div className="mt-1">
                    Còn <b>{n(con)}</b> việc / {n(p.nTotal)} · đã duyệt {n(p.nApproved)}
                    {p.nRejected > 0 && p.kind === "xac_nhan" && <span className="text-danger"> · bị trả {n(p.nRejected)}</span>}
                  </div>
                  <div className="h-2 rounded bg-border-subtle mt-2">
                    <div className="h-2 rounded bg-accent" style={{ width: `${((p.nTotal - con) / Math.max(1, p.nTotal)) * 100}%` }} />
                  </div>
                  <button className="btn primary mt-3" disabled={con === 0} onClick={() => setLam({ pkg: p })}>
                    <Icon name="chay" /> {con === 0 ? "Đã xong" : "Làm việc"}
                  </button>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </section>
  );
}
