// Copyright (c) 2026 DigiText. All rights reserved.
//
// Quản trị: "Giao việc xác nhận" — tạo gói (theo hồ sơ liền mạch / ngẫu nhiên, chia thành K gói), giao
// CTV, theo dõi tiến độ, duyệt / từ chối từng mục hoặc cả gói. Kế hoạch 29/9/2026 (từ doc_extract_pla).

import React, { useCallback, useEffect, useMemo, useState } from "react";
import { isTauri, V3xq } from "@/api/ipc";
import { V4ek, type V4ej } from "@/api/docs";
import { V4ac, V4ak, type V4a8, type V4a9 } from "@/api/users";
import {
  ITEM_STATUS_LABEL,
  KIND_LABEL,
  MODE_LABEL,
  PKG_STATUS_LABEL,
  workClosePackage,
  workCreatePackage,
  workGetPackage,
  workListPackages,
  workPatchPackage,
  workReview,
  workSetMembers,
  type CreatePackageResp,
  type ItemStatus,
  type WorkItem,
  type WorkKind,
  type WorkMode,
  type WorkPackage,
  type WorkPackageDetail,
} from "@/api/work";
import V4i7 from "@/features/upload/OcrFieldsView";
import { Icon } from "@/components/Icon";

const n = (v: number) => v.toLocaleString("vi-VN");
const errText = (e: unknown) => (e instanceof V3xq || e instanceof Error ? e.message : String(e));
const ngay = (s: string | null) => (s ? s.split("-").reverse().join("/") : "—");
const quaHan = (p: WorkPackage) =>
  !!p.dueAt && p.status !== "done" && p.status !== "closed" && new Date(`${p.dueAt}T23:59:59`).getTime() < Date.now();

function TienDo({ p }: { p: WorkPackage }) {
  const t = p.nTotal || 1;
  const seg = [
    ["#16a34a", p.nApproved, "đã duyệt"],
    ["#2563eb", p.nDone + p.nChecked, "đã xác nhận / kiểm tra"],
    ["#dc2626", p.nRejected, "bị trả"],
  ] as const;
  return (
    <div className="h-2 rounded bg-border-subtle flex overflow-hidden" title={seg.map(([, v, l]) => `${l}: ${n(v)}`).join(" · ")}>
      {seg.map(([c, v]) => (
        <div key={c} style={{ width: `${(v / t) * 100}%`, background: c }} />
      ))}
    </div>
  );
}

/** Phạm vi phông / hộp / hồ sơ suy từ cây tài liệu máy chủ. */
function useScope(docs: V4ej[], projectId: string) {
  return useMemo(() => {
    const phong = new Set<string>();
    const hop = new Map<string, Set<string>>();
    const hoso = new Map<string, Set<string>>();
    for (const d of docs) {
      if (d.projectId !== projectId) continue;
      const p = (d.relPath ?? "").split("/").filter((x) => x.trim() !== "");
      if (p.length < 1) continue;
      phong.add(p[0]);
      if (p.length >= 2) {
        if (!hop.has(p[0])) hop.set(p[0], new Set());
        hop.get(p[0])!.add(p[1]);
        if (p.length >= 3) {
          const k = `${p[0]}/${p[1]}`;
          if (!hoso.has(k)) hoso.set(k, new Set());
          hoso.get(k)!.add(p[2]);
        }
      }
    }
    const col = new Intl.Collator("vi", { numeric: true }).compare;
    return {
      phong: [...phong].sort(col),
      hop: (ph: string) => [...(hop.get(ph) ?? [])].sort(col),
      hoso: (ph: string, h: string) => [...(hoso.get(`${ph}/${h}`) ?? [])].sort(col),
    };
  }, [docs, projectId]);
}

function TaoGoi({
  projectId,
  packages,
  docs,
  onCreated,
  onClose,
}: {
  projectId: string;
  packages: WorkPackage[];
  docs: V4ej[];
  onCreated: () => void;
  onClose: () => void;
}) {
  const scope = useScope(docs, projectId);
  const [name, setName] = useState("");
  const [kind, setKind] = useState<WorkKind>("xac_nhan");
  const [mode, setMode] = useState<WorkMode>("hoso");
  const [phong, setPhong] = useState("");
  const [hop, setHop] = useState("");
  const [hoso, setHoso] = useState("");
  const [size, setSize] = useState(50);
  const [count, setCount] = useState(1);
  const [source, setSource] = useState("");
  const [pct, setPct] = useState(20);
  const [dueAt, setDueAt] = useState("");
  const [note, setNote] = useState("");
  const [users, setUsers] = useState<V4a8[]>([]);
  const [q, setQ] = useState("");
  const [members, setMembers] = useState<Set<string>>(new Set());
  const [preview, setPreview] = useState<CreatePackageResp | null>(null);
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  useEffect(() => {
    V4ac(projectId)
      .then((u) => (u.length > 0 ? u : V4ac()))
      .then((u) => setUsers(u.filter((x) => x.isActive)))
      .catch(() => {});
  }, [projectId]);

  const prefixes = useMemo(() => {
    const p = [phong, hop, hoso].filter(Boolean);
    return p.length ? [p.join("/")] : [];
  }, [phong, hop, hoso]);

  const body = () => ({
    projectId,
    name,
    kind,
    mode,
    prefixes,
    size,
    count,
    memberIds: [...members],
    dueAt: dueAt || null,
    note: note || null,
    sourcePackageId: kind === "kiem_tra" ? source || null : null,
    samplePct: kind === "kiem_tra" ? pct : null,
  });

  const xemTruoc = useCallback(async () => {
    setErr(null);
    try {
      setPreview(await workCreatePackage({ ...body(), dryRun: true }));
    } catch (e) {
      setErr(errText(e));
      setPreview(null);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [projectId, name, kind, mode, prefixes, size, count, members, dueAt, note, source, pct]);

  useEffect(() => {
    const t = window.setTimeout(() => void xemTruoc(), 300);
    return () => window.clearTimeout(t);
  }, [xemTruoc]);

  const tao = async () => {
    if (members.size === 0 && !window.confirm("Chưa chọn cộng tác viên nào. Tạo gói trống người làm?")) return;
    setBusy(true);
    setErr(null);
    try {
      const r = await workCreatePackage(body());
      window.alert(`Đã tạo ${r.packageIds.length} gói (${r.sizes.map(n).join(", ")} tài liệu).${r.toExtract ? ` ${n(r.toExtract)} tài liệu đang được máy bóc trường.` : ""}`);
      onCreated();
    } catch (e) {
      setErr(errText(e));
    } finally {
      setBusy(false);
    }
  };

  const dsUser = users.filter((u) => !q || `${u.displayName} ${u.username}`.toLowerCase().includes(q.toLowerCase()));
  // Ô nhập luôn co theo cột lưới (min-w-0) — select không có min-width:0 sẽ giãn theo option dài nhất và đè lên ô bên.
  const f = (label: string, el: React.ReactElement<{ className?: string }>) => (
    <label className="grid gap-1 text-sm min-w-0">
      <span className="muted truncate">{label}</span>
      {React.cloneElement(el, { className: `${el.type === "select" ? "" : "inp "}w-full min-w-0 ${el.props.className ?? ""}`.trim() })}
    </label>
  );

  return (
    <div
      className="card mt-3.5"
      onKeyDown={(e) => {
        if (e.ctrlKey && e.key === "Enter") void tao();
      }}
    >
      <div className="flex items-center gap-2">
        <h3 className="m-0">Tạo gói việc</h3>
        <div className="flex-1" />
        <button className="btn" onClick={onClose}>
          Đóng
        </button>
      </div>
      <div className="grid gap-3 mt-3" style={{ gridTemplateColumns: "repeat(auto-fill, minmax(220px, 1fr))" }}>
        {f("Tên gói (để trống = tự đặt)", <input value={name} onChange={(e) => setName(e.target.value)} />)}
        {f(
          "Loại gói",
          <select
            value={kind}
            onChange={(e) => setKind(e.target.value as WorkKind)}
            title={kind === "kiem_tra" ? "CTV khác soát lại một phần mẫu của gói đã xác nhận" : "CTV sửa và xác nhận 18 trường của từng tài liệu"}
          >
            <option value="xac_nhan">{KIND_LABEL.xac_nhan}</option>
            <option value="kiem_tra">{KIND_LABEL.kiem_tra}</option>
          </select>,
        )}
        {kind === "kiem_tra" ? (
          <>
            {f(
              "Gói xác nhận nguồn",
              <select value={source} onChange={(e) => setSource(e.target.value)}>
                <option value="">— Chọn gói —</option>
                {packages
                  .filter((p) => p.kind === "xac_nhan" && p.nDone + p.nChecked + p.nApproved > 0)
                  .map((p) => (
                    <option key={p.id} value={p.id}>
                      {p.name} ({n(p.nDone + p.nChecked + p.nApproved)} đã xác nhận)
                    </option>
                  ))}
              </select>,
            )}
            {f("% mẫu kiểm tra", <input type="number" min={1} max={100} value={pct} onChange={(e) => setPct(Number(e.target.value) || 20)} />)}
          </>
        ) : (
          <>
            {f(
              "Cách chia",
              <select
                value={mode}
                onChange={(e) => setMode(e.target.value as WorkMode)}
                title={mode === "hoso" ? "Lấy tài liệu theo thứ tự hồ sơ, không cắt giữa hồ sơ" : "Bốc ngẫu nhiên trong phạm vi đã chọn"}
              >
                <option value="hoso">{MODE_LABEL.hoso}</option>
                <option value="ngau_nhien">{MODE_LABEL.ngau_nhien}</option>
              </select>,
            )}
            {f(
              "Phông",
              <select value={phong} onChange={(e) => { setPhong(e.target.value); setHop(""); setHoso(""); }}>
                <option value="">— Cả dự án —</option>
                {scope.phong.map((p) => (
                  <option key={p} value={p}>{p}</option>
                ))}
              </select>,
            )}
            {f(
              "Hộp",
              <select value={hop} disabled={!phong} onChange={(e) => { setHop(e.target.value); setHoso(""); }}>
                <option value="">— Cả phông —</option>
                {phong && scope.hop(phong).map((h) => (
                  <option key={h} value={h}>{h}</option>
                ))}
              </select>,
            )}
            {f(
              "Hồ sơ",
              <select value={hoso} disabled={!hop} onChange={(e) => setHoso(e.target.value)}>
                <option value="">— Cả hộp —</option>
                {phong && hop && scope.hoso(phong, hop).map((h) => (
                  <option key={h} value={h}>{h}</option>
                ))}
              </select>,
            )}
            {f("Cỡ gói (tài liệu)", <input type="number" min={1} value={size} onChange={(e) => setSize(Math.max(1, Number(e.target.value) || 1))} />)}
            {f("Số gói", <input type="number" min={1} max={200} value={count} onChange={(e) => setCount(Math.max(1, Number(e.target.value) || 1))} />)}
          </>
        )}
        {f("Hạn hoàn thành", <input type="date" value={dueAt} onChange={(e) => setDueAt(e.target.value)} />)}
        {f("Ghi chú", <input value={note} onChange={(e) => setNote(e.target.value)} />)}
      </div>

      <div className="mt-3">
        <div className="flex items-center gap-2 text-sm">
          <span className="muted">Cộng tác viên ({members.size} đã chọn)</span>
          <input className="inp max-w-60" placeholder="tìm tên / tài khoản" value={q} onChange={(e) => setQ(e.target.value)} />
          <button className="btn sm" onClick={() => setMembers(new Set(dsUser.map((u) => u.id)))}>
            Chọn tất cả
          </button>
          <button className="btn sm" onClick={() => setMembers(new Set())}>
            Bỏ chọn
          </button>
        </div>
        <div className="flex gap-2 flex-wrap mt-2 max-h-40 overflow-auto">
          {dsUser.map((u) => (
            <label key={u.id} className="flex items-center gap-1 text-sm border border-border-subtle rounded px-2 py-1">
              <input
                type="checkbox"
                checked={members.has(u.id)}
                onChange={(e) =>
                  setMembers((s) => {
                    const m = new Set(s);
                    e.target.checked ? m.add(u.id) : m.delete(u.id);
                    return m;
                  })
                }
              />
              {u.displayName} <span className="muted">({u.username})</span>
            </label>
          ))}
          {dsUser.length === 0 && <span className="muted text-sm">Không có tài khoản nào trong dự án.</span>}
        </div>
      </div>

      <div className="flex items-center gap-3 mt-3 flex-wrap">
        <span className="text-sm">
          {preview ? (
            preview.sizes.length ? (
              <>
                Sẽ tạo <b>{preview.sizes.length}</b> gói: {preview.sizes.map(n).join(", ")} tài liệu ·{" "}
                <span className={preview.sizes.reduce((a, b) => a + b, 0) === preview.available ? "text-success-700" : "text-warning-700"}>
                  {Math.round((preview.sizes.reduce((a, b) => a + b, 0) / Math.max(1, preview.available)) * 100)}% trong {n(preview.available)} tài liệu đủ điều kiện
                </span>
                {preview.toExtract > 0 && (
                  <span className="text-warning-700"> · {n(preview.toExtract)} tài liệu chưa bóc trường — máy sẽ bóc trước, CTV mở ra là có trường máy điền</span>
                )}
              </>
            ) : (
              <span className="text-danger">Không có tài liệu đủ điều kiện (đã OCR xong, chưa giao) trong phạm vi.</span>
            )
          ) : (
            "Đang tính…"
          )}
        </span>
        <div className="flex-1" />
        {err && <span className="text-danger text-sm">{err}</span>}
        <button className="btn primary" onClick={() => void tao()} disabled={busy || !preview?.sizes.length}>
          <Icon name="them" /> {busy ? "Đang tạo…" : "Tạo gói (Ctrl+Enter)"}
        </button>
      </div>
    </div>
  );
}

function ChiTietGoi({ id, onBack, onChanged }: { id: string; onBack: () => void; onChanged: () => void }) {
  const [pkg, setPkg] = useState<WorkPackageDetail | null>(null);
  const [err, setErr] = useState<string | null>(null);
  const [msg, setMsg] = useState<string | null>(null);
  const [loc, setLoc] = useState<ItemStatus | "">("");
  const [sel, setSel] = useState<Set<string>>(new Set());
  const [open, setOpen] = useState<WorkItem | null>(null);
  const [suaTv, setSuaTv] = useState(false);
  const [users, setUsers] = useState<V4a8[]>([]);
  const [tv, setTv] = useState<Set<string>>(new Set());

  const load = useCallback(async () => {
    try {
      const p = await workGetPackage(id);
      setPkg(p);
      setTv(new Set(p.members.map((m) => m.userId)));
    } catch (e) {
      setErr(errText(e));
    }
  }, [id]);
  useEffect(() => {
    void load();
  }, [load]);
  useEffect(() => {
    if (!pkg || !suaTv) return;
    V4ac(pkg.projectId)
      .then((u) => (u.length > 0 ? u : V4ac()))
      .then((u) => setUsers(u.filter((x) => x.isActive)))
      .catch(() => {});
  }, [pkg, suaTv]);

  const items = useMemo(() => (pkg?.items ?? []).filter((i) => !loc || i.status === loc), [pkg, loc]);
  const duyetDuoc = (i: WorkItem) => ["done", "redone", "checked_ok", "checked_err"].includes(i.status);

  const review = async (decision: "approve" | "reject", all: boolean) => {
    if (!pkg) return;
    let note: string | undefined;
    if (decision === "reject") {
      const r = window.prompt("Lý do từ chối (CTV sẽ thấy để sửa):");
      if (!r?.trim()) return;
      note = r.trim();
    }
    const ids = all ? undefined : [...sel];
    if (!all && ids!.length === 0) return;
    if (all && !window.confirm(`${decision === "approve" ? "Duyệt" : "Từ chối"} TẤT CẢ mục đã xác nhận trong gói?`)) return;
    try {
      const r = await workReview(pkg.id, { itemIds: ids, decision, note });
      setMsg(`${decision === "approve" ? "Đã duyệt" : "Đã từ chối"} ${n(r.changed)} mục${r.skipped ? ` · bỏ qua ${n(r.skipped)} (chưa xác nhận hoặc do chính bạn xác nhận)` : ""}.`);
      setSel(new Set());
      await load();
      onChanged();
    } catch (e) {
      setErr(errText(e));
    }
  };

  if (!pkg) return <div className="card mt-3.5 muted">{err ?? "Đang tải…"}</div>;
  if (open) {
    return (
      <div className="mt-3.5">
        <V4i7 doc={{ id: open.documentId, filename: open.filename, reviewStatus: "pending" }} onBack={() => { setOpen(null); void load(); }} />
      </div>
    );
  }
  const tiles: [ItemStatus | "", string, number][] = [
    ["", "Tất cả", pkg.nTotal],
    ["pending", "Chờ làm", pkg.nPending],
    ["done", "Đã xác nhận", pkg.nDone],
    ["checked_ok", "Kiểm tra đạt", pkg.items.filter((i) => i.status === "checked_ok").length],
    ["checked_err", "Kiểm tra có lỗi", pkg.items.filter((i) => i.status === "checked_err").length],
    ["rejected", "Bị trả", pkg.nRejected],
    ["approved", "Đã duyệt", pkg.nApproved],
  ];
  return (
    <div className="card mt-3.5">
      <div className="flex items-center gap-2 flex-wrap">
        <button className="btn" onClick={onBack}>
          ← Danh sách gói
        </button>
        <h3 className="m-0">{pkg.name}</h3>
        <span className="badge gray">{KIND_LABEL[pkg.kind]}</span>
        <span className={`badge ${pkg.status === "done" ? "ok" : pkg.status === "under_review" ? "warn" : "gray"}`}>{PKG_STATUS_LABEL[pkg.status]}</span>
        <span className={`text-sm ${quaHan(pkg) ? "text-danger" : "muted"}`}>Hạn: {ngay(pkg.dueAt)}</span>
        <div className="flex-1" />
        <button className="btn sm" onClick={() => setSuaTv((v) => !v)}>
          <Icon name="nguoi" /> CTV: {pkg.members.map((m) => m.displayName).join(", ") || "chưa giao"}
        </button>
        <button
          className="btn sm"
          onClick={() => {
            const d = window.prompt("Hạn hoàn thành (YYYY-MM-DD, để trống = bỏ hạn):", pkg.dueAt ?? "");
            if (d === null) return;
            void workPatchPackage(pkg.id, { dueAt: d.trim() || null }).then(load).catch((e) => setErr(errText(e)));
          }}
        >
          Đổi hạn
        </button>
        {pkg.status !== "closed" && (
          <button
            className="btn sm"
            onClick={() => {
              if (window.confirm("Đóng gói? Mục chưa duyệt sẽ huỷ và tài liệu được giao lại ở gói khác.")) void workClosePackage(pkg.id).then(() => { void load(); onChanged(); });
            }}
          >
            Đóng gói
          </button>
        )}
      </div>
      {suaTv && (
        <div className="mt-2 flex gap-2 flex-wrap items-center text-sm">
          {users.map((u) => (
            <label key={u.id} className="flex items-center gap-1 border border-border-subtle rounded px-2 py-1">
              <input
                type="checkbox"
                checked={tv.has(u.id)}
                onChange={(e) =>
                  setTv((s) => {
                    const m = new Set(s);
                    e.target.checked ? m.add(u.id) : m.delete(u.id);
                    return m;
                  })
                }
              />
              {u.displayName}
            </label>
          ))}
          <button className="btn sm primary" onClick={() => void workSetMembers(pkg.id, [...tv]).then(() => { setSuaTv(false); void load(); }).catch((e) => setErr(errText(e)))}>
            Lưu CTV
          </button>
        </div>
      )}
      <div className="mt-3">
        <TienDo p={pkg} />
      </div>
      <div className="flex gap-2 flex-wrap mt-3">
        {tiles.map(([k, label, v]) => (
          <button key={k} className={`btn sm ${loc === k ? "primary" : ""}`} onClick={() => setLoc(k)}>
            {label}: {n(v)}
          </button>
        ))}
      </div>
      {err && <div className="text-danger text-sm mt-2">{err}</div>}
      {msg && <div className="text-success-700 text-sm mt-2">{msg}</div>}
      <div className="flex gap-2 items-center mt-3 flex-wrap">
        <button className="btn sm primary" disabled={sel.size === 0} onClick={() => void review("approve", false)}>
          <Icon name="duyet" /> Duyệt đã chọn ({sel.size})
        </button>
        <button className="btn sm" disabled={sel.size === 0} onClick={() => void review("reject", false)}>
          Từ chối đã chọn
        </button>
        <button className="btn sm" onClick={() => void review("approve", true)}>
          Duyệt tất cả đã xác nhận
        </button>
      </div>
      <div className="tbl-scroll mt-2">
        <table className="tbl">
          <thead>
            <tr>
              <th>
                <input
                  type="checkbox"
                  checked={items.length > 0 && items.filter(duyetDuoc).every((i) => sel.has(i.id))}
                  onChange={(e) => setSel(e.target.checked ? new Set(items.filter(duyetDuoc).map((i) => i.id)) : new Set())}
                />
              </th>
              <th>#</th>
              <th>Tài liệu</th>
              <th>Đường dẫn</th>
              <th>Trạng thái</th>
              <th>Người làm</th>
              <th>Kiểm tra</th>
              <th>Ghi chú</th>
              <th />
            </tr>
          </thead>
          <tbody>
            {items.map((i) => (
              <tr key={i.id}>
                <td>
                  {duyetDuoc(i) && (
                    <input
                      type="checkbox"
                      checked={sel.has(i.id)}
                      onChange={(e) =>
                        setSel((s) => {
                          const m = new Set(s);
                          e.target.checked ? m.add(i.id) : m.delete(i.id);
                          return m;
                        })
                      }
                    />
                  )}
                </td>
                <td className="num">{i.ord}</td>
                <td>{i.filename}</td>
                <td className="muted">{i.relPath ?? i.hoSoRef ?? "—"}</td>
                <td>
                  <span className={`badge ${i.status === "approved" ? "ok" : i.status === "rejected" || i.status === "checked_err" ? "err" : i.status === "pending" ? "gray" : "info"}`}>
                    {ITEM_STATUS_LABEL[i.status]}
                  </span>
                </td>
                <td>{i.doneByName ?? "—"}</td>
                <td>{i.checkedByName ?? "—"}</td>
                <td className="text-sm">{[i.checkNote, i.reviewNote].filter(Boolean).join(" · ")}</td>
                <td>
                  <button className="btn sm" onClick={() => setOpen(i)}>
                    Mở
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

export default function WorkManageView() {
  const [projects, setProjects] = useState<V4a9[]>([]);
  const [projectId, setProjectId] = useState("");
  const [pkgs, setPkgs] = useState<WorkPackage[]>([]);
  const [docs, setDocs] = useState<V4ej[]>([]);
  const [tao, setTao] = useState(false);
  const [open, setOpen] = useState<string | null>(null);
  const [err, setErr] = useState<string | null>(null);

  useEffect(() => {
    if (!isTauri()) return;
    V4ak()
      .then((p) => setProjects(p.filter((x) => x.isActive)))
      .catch(() => {});
    V4ek().then(setDocs).catch(() => {});
  }, []);
  const load = useCallback(async () => {
    if (!isTauri()) return;
    try {
      setPkgs(await workListPackages(projectId || undefined));
    } catch (e) {
      setErr(errText(e));
    }
  }, [projectId]);
  useEffect(() => {
    void load();
  }, [load]);

  return (
    <section className="view active">
      <div className="page-head">
        <h2>Giao việc xác nhận trường</h2>
        <div className="sub">· gói việc cho cộng tác viên · duyệt kết quả</div>
      </div>
      {open ? (
        <ChiTietGoi id={open} onBack={() => setOpen(null)} onChanged={() => void load()} />
      ) : (
        <>
          <div className="card mt-3.5">
            <div className="flex gap-2 items-center flex-wrap">
              <select value={projectId} onChange={(e) => setProjectId(e.target.value)}>
                <option value="">— Tất cả dự án —</option>
                {projects.map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.code} · {p.name}
                  </option>
                ))}
              </select>
              <button className="btn" onClick={() => void load()}>
                <Icon name="lamMoi" /> Làm mới
              </button>
              <div className="flex-1" />
              <button className="btn primary" disabled={!projectId} onClick={() => setTao(true)} title={projectId ? "" : "Chọn dự án trước"}>
                <Icon name="them" /> Tạo gói
              </button>
            </div>
            {err && <div className="text-danger text-sm mt-2">{err}</div>}
          </div>
          {tao && projectId && (
            <TaoGoi projectId={projectId} packages={pkgs} docs={docs} onCreated={() => { setTao(false); void load(); }} onClose={() => setTao(false)} />
          )}
          <div className="card mt-3.5">
            <div className="tbl-scroll">
              <table className="tbl">
                <thead>
                  <tr>
                    <th>Gói</th>
                    <th>Loại</th>
                    <th>CTV</th>
                    <th>Hạn</th>
                    <th className="num">Tổng</th>
                    <th className="num">Chờ</th>
                    <th className="num">Đã XN</th>
                    <th className="num">Bị trả</th>
                    <th className="num">Đã duyệt</th>
                    <th>Tiến độ</th>
                    <th>Trạng thái</th>
                  </tr>
                </thead>
                <tbody>
                  {pkgs.length === 0 ? (
                    <tr>
                      <td colSpan={11} className="muted">
                        Chưa có gói việc nào. Chọn dự án rồi bấm “Tạo gói”.
                      </td>
                    </tr>
                  ) : (
                    pkgs.map((p) => (
                      <tr key={p.id} style={{ cursor: "pointer" }} onClick={() => setOpen(p.id)}>
                        <td>
                          <b>{p.name}</b>
                          <div className="muted text-sm">{MODE_LABEL[p.mode]}</div>
                        </td>
                        <td>{KIND_LABEL[p.kind]}</td>
                        <td className="text-sm">{p.members.map((m) => m.displayName).join(", ") || <span className="text-warning-700">chưa giao</span>}</td>
                        <td className={quaHan(p) ? "text-danger" : ""}>{ngay(p.dueAt)}</td>
                        <td className="num">{n(p.nTotal)}</td>
                        <td className="num">{n(p.nPending)}</td>
                        <td className="num">{n(p.nDone + p.nChecked)}</td>
                        <td className="num">{n(p.nRejected)}</td>
                        <td className="num">{n(p.nApproved)}</td>
                        <td style={{ minWidth: 120 }}>
                          <TienDo p={p} />
                        </td>
                        <td>
                          <span className={`badge ${p.status === "done" ? "ok" : p.status === "under_review" ? "warn" : "gray"}`}>{PKG_STATUS_LABEL[p.status]}</span>
                        </td>
                      </tr>
                    ))
                  )}
                </tbody>
              </table>
            </div>
          </div>
        </>
      )}
    </section>
  );
}
