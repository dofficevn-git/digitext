// Copyright (c) 2026 DigiText. All rights reserved.
//
// Đóng gói từ bảng hồ sơ (đề nghị chỉnh sửa mục 8): tích chọn hồ sơ trong mục lục → chọn loại gói
// (SIP hồ sơ / SIP tài liệu / AIP hồ sơ / AIP tài liệu / DIP) → ghép với tài liệu đã lên máy chủ theo
// phông / hộp / hồ sơ → tạo gói trên máy chủ (mỗi hồ sơ một gói; gói mức tài liệu: mỗi tài liệu một
// gói). Thông tin mô tả lấy từ bản ghi hồ sơ (mã hồ sơ, tiêu đề, THBQ, thời gian, số tờ).

import { useEffect, useMemo, useState, type ReactElement } from "react";
import { V41p } from "@/live/useLive";
import { isTauri, V3xq } from "@/api/ipc";
import { V4ek, type V4ej } from "@/api/docs";
import { V4ru, V4rv, V4rw, type V4rt } from "@/api/packaging";
import type { V4bm } from "@/api/dossier";
import type { V3yo } from "@/domain/types";
import { Icon } from "@/components/Icon";
import { V4su, V4sn, V4sk, type V4sj, type V4sm } from "./packageWire";

const errText = (e: unknown) => (e instanceof V3xq || e instanceof Error ? e.message : String(e));
const so = (s: string | null | undefined): number | null => {
  const m = /(\d+)/.exec(s ?? "");
  return m ? Number(m[1]) : null;
};
const phongOf = (d: V4ej) => {
  const p = (d.relPath ?? "").split("/").filter((x) => x.trim() !== "");
  return p.length >= 2 ? p[0].trim() : "";
};
/** "HOP 01/HO SO 5" → [1, 5]. */
function hopHoSo(d: V4ej): [number | null, number | null] {
  const ref = d.hoSoRef ?? "";
  const i = ref.indexOf("/");
  if (i > 0) return [so(ref.slice(0, i)), so(ref.slice(i + 1))];
  const p = (d.relPath ?? "").split("/").filter((x) => x.trim() !== "");
  return p.length >= 3 ? [so(p[1]), so(p[2])] : [null, null];
}
/** Tên THBQ → mã thời hạn bảo quản (TT02): 01 vĩnh viễn, 02 70 năm … 07 khác. */
function maintenanceOf(tenThbq: string): string {
  const t = tenThbq.toLowerCase();
  if (t.includes("vĩnh viễn")) return "01";
  for (const [nam, ma] of [
    ["70", "02"],
    ["50", "03"],
    ["30", "04"],
    ["20", "05"],
    ["10", "06"],
  ] as const)
    if (t.includes(nam)) return ma;
  return "07";
}
function nho(key: string): string {
  try {
    return localStorage.getItem(key) ?? "";
  } catch {
    return "";
  }
}
function luu(key: string, v: string) {
  try {
    localStorage.setItem(key, v);
  } catch {
    /* tiện ích nhớ */
  }
}

const KINDS: V4sj[] = ["sip_hoso", "sip_tailieu", "aip_hoso", "aip_tailieu", "dip"];

interface KetQua {
  hoSo: string;
  ok: number;
  loi: string[];
}

export function PackDialog({
  dossiers,
  thbqs,
  onClose,
}: {
  dossiers: V4bm[];
  thbqs: V3yo[];
  onClose: () => void;
}) {
  const [docs, setDocs] = useState<V4ej[] | null>(null);
  const [err, setErr] = useState<string | null>(null);
  const [kind, setKind] = useState<V4sj>("aip_hoso");
  const [projectId, setProjectId] = useState(() => nho("pack.projectId"));
  const [phong, setPhong] = useState("");
  const [orgCode, setOrgCode] = useState(() => nho("pack.archiveOrgCode"));
  const [fondCode, setFondCode] = useState(() => dossiers[0]?.maDa ?? "");
  const [agreement, setAgreement] = useState("");
  const [dip, setDip] = useState<Partial<V4sm>>({ requestDate: new Date().toISOString().slice(0, 10) });
  const [busy, setBusy] = useState(false);
  const [ketQua, setKetQua] = useState<KetQua[] | null>(null);

  useEffect(() => {
    if (!isTauri()) return;
    V4ek()
      .then(setDocs)
      .catch((e) => setErr(`Không tải được danh sách tài liệu máy chủ: ${errText(e)}`));
  }, []);

  const duAn = useMemo(() => {
    const m = new Map<string, string>();
    for (const d of docs ?? []) if (d.projectId) m.set(d.projectId, `${d.projectCode ?? ""} · ${d.projectName ?? ""}`);
    return [...m.entries()];
  }, [docs]);
  useEffect(() => {
    if (duAn.length && !duAn.some(([id]) => id === projectId)) setProjectId(duAn[0][0]);
  }, [duAn, projectId]);
  const dsPhong = useMemo(() => {
    const s = new Set<string>();
    for (const d of docs ?? []) if (d.projectId === projectId && phongOf(d)) s.add(phongOf(d));
    return [...s].sort(new Intl.Collator("vi", { numeric: true }).compare);
  }, [docs, projectId]);
  useEffect(() => {
    if (!dsPhong.includes(phong)) {
      // Đoán phông theo mã phông của hồ sơ; không có thì lấy phông đầu.
      const ma = (dossiers[0]?.maDa ?? "").toUpperCase();
      setPhong(dsPhong.find((p) => ma && p.toUpperCase().includes(ma)) ?? dsPhong[0] ?? "");
    }
  }, [dsPhong, phong, dossiers]);

  // Ghép hồ sơ mục lục ↔ tài liệu máy chủ theo số hộp + số hồ sơ trong phông đã chọn.
  const ghep = useMemo(() => {
    const trongPhong = (docs ?? []).filter(
      (d) => d.projectId === projectId && (dsPhong.length === 0 || phongOf(d) === phong),
    );
    return dossiers.map((r) => {
      const hop = so(r.hopSo);
      const hs = so(r.soTam);
      const ds = trongPhong
        .filter((d) => {
          const [h, s] = hopHoSo(d);
          return h != null && s != null && h === hop && s === hs;
        })
        .sort((a, b) => a.filename.localeCompare(b.filename, "vi", { numeric: true }));
      return { r, ds };
    });
  }, [docs, dossiers, projectId, phong, dsPhong.length]);

  const tenThbq = (id?: number) => thbqs.find((t) => t.thbqid === id)?.tenThbq ?? "";

  const chay = async () => {
    setBusy(true);
    setErr(null);
    luu("pack.projectId", projectId);
    luu("pack.archiveOrgCode", orgCode.trim());
    const out: KetQua[] = [];
    for (const { r, ds } of ghep) {
      const nhan = `Hộp ${r.hopSo ?? "—"} · HS ${r.soTam}`;
      const kq: KetQua = { hoSo: nhan, ok: 0, loi: [] };
      out.push(kq);
      if (ds.length === 0) {
        kq.loi.push("không có tài liệu nào trên máy chủ khớp phông/hộp/hồ sơ này");
        continue;
      }
      const base: V4sm = {
        ...V4sn,
        ...(kind === "dip" ? dip : {}),
        kind,
        fondCode: fondCode.trim(),
        archiveOrgCode: orgCode.trim(),
        fileCode: (r.maHoSo ?? "").trim() || `${fondCode.trim()}.${r.hopSo ?? ""}.${r.soTam}`,
        title: r.tieuDe,
        maintenance: kind.startsWith("sip") ? "01" : maintenanceOf(tenThbq(r.thbq)),
        numberOfPaper: r.soTo ?? "",
        numberOfPage: String(ds.reduce((a, d) => a + (d.pageCount ?? 0), 0) || ""),
        paperFileCode: (r.maHoSo ?? "").trim() || `${r.hopSo ?? ""}.${r.soTam}`,
        startDate: r.ngayBatDau ?? "",
        endDate: r.ngayKetThuc ?? "",
        submissionAgreement: agreement,
        certTrichYeu: dip.certTrichYeu || r.tieuDe,
        certMaLuuTruGoc: dip.certMaLuuTruGoc || (r.maHoSo ?? ""),
      };
      // Gói mức tài liệu: mỗi tài liệu một gói; còn lại: cả hồ sơ một gói.
      const nhom = kind === "aip_tailieu" || kind === "sip_tailieu" ? ds.map((d) => [d]) : [ds];
      for (const [i, g] of nhom.entries()) {
        const form: V4sm =
          nhom.length > 1 ? { ...base, fileCode: `${base.fileCode}.${String(i + 1).padStart(3, "0")}`, title: g[0].filename } : base;
        const { request, errors } = V4su(
          form,
          g.map((d) => ({ documentId: d.id, filename: d.filename })),
        );
        if (!request) {
          kq.loi.push(...errors);
          break;
        }
        try {
          await V4rv({ projectId, hoSoRef: g[0].hoSoRef, request });
          kq.ok += 1;
        } catch (e) {
          kq.loi.push(errText(e));
        }
      }
      setKetQua([...out]);
    }
    setKetQua(out);
    setBusy(false);
  };

  const khop = ghep.filter((g) => g.ds.length > 0).length;
  const f = (label: string, el: ReactElement) => (
    <label className="grid gap-1 text-sm">
      <span className="muted">{label}</span>
      {el}
    </label>
  );

  return (
    <div className="lhs-overlay" onClick={onClose}>
      <div className="lhs-modal" style={{ maxWidth: 900, width: "95vw" }} onClick={(e) => e.stopPropagation()}>
        <div className="m-head">
          <h4>Đóng gói {dossiers.length} hồ sơ</h4>
          <button className="m-close" onClick={onClose}>
            ×
          </button>
        </div>
        <div className="m-body">
        <div className="grid gap-3 mt-3" style={{ gridTemplateColumns: "repeat(auto-fill, minmax(220px, 1fr))" }}>
          {f(
            "Lựa chọn đóng gói",
            <select value={kind} onChange={(e) => setKind(e.target.value as V4sj)}>
              {KINDS.map((k) => (
                <option key={k} value={k}>
                  {V4sk[k]}
                </option>
              ))}
            </select>,
          )}
          {f(
            "Dự án trên máy chủ",
            <select value={projectId} onChange={(e) => setProjectId(e.target.value)}>
              {duAn.map(([id, ten]) => (
                <option key={id} value={id}>
                  {ten}
                </option>
              ))}
            </select>,
          )}
          {dsPhong.length > 0 &&
            f(
              "Phông trên máy chủ",
              <select value={phong} onChange={(e) => setPhong(e.target.value)}>
                {dsPhong.map((p) => (
                  <option key={p} value={p}>
                    {p}
                  </option>
                ))}
              </select>,
            )}
          {f("Mã phông (Fondcode)", <input value={fondCode} onChange={(e) => setFondCode(e.target.value)} />)}
          {f("Mã cơ quan lưu trữ", <input value={orgCode} onChange={(e) => setOrgCode(e.target.value)} />)}
          {kind.startsWith("sip") &&
            f("Mã yêu cầu nộp (thoả thuận nộp)", <input value={agreement} onChange={(e) => setAgreement(e.target.value)} />)}
        </div>
        {kind === "dip" && (
          <div className="grid gap-3 mt-3" style={{ gridTemplateColumns: "repeat(auto-fill, minmax(220px, 1fr))" }}>
            {(
              [
                ["requestId", "Mã yêu cầu khai thác"],
                ["requestDate", "Ngày yêu cầu (YYYY-MM-DD)"],
                ["certMaXacThuc", "Mã xác thực lưu trữ"],
                ["certTenLuuTru", "Tên lưu trữ lịch sử"],
                ["certNguoiNhanHoTen", "Họ tên người nhận"],
                ["certNguoiNhanDinhDanh", "Mã định danh người nhận"],
                ["certTenLoai", "Tên loại tài liệu"],
                ["certMucDich", "Mục đích sử dụng"],
              ] as const
            ).map(([k, label]) =>
              f(
                label,
                <input
                  key={k}
                  value={(dip[k] as string | undefined) ?? ""}
                  onChange={(e) => setDip((d) => ({ ...d, [k]: e.target.value }))}
                />,
              ),
            )}
          </div>
        )}

        {err && <div className="text-danger text-sm mt-2">{err}</div>}
        <div className="tbl-scroll mt-3" style={{ maxHeight: 280 }}>
          <table className="dataTable">
            <thead>
              <tr>
                <th>Hộp số</th>
                <th>HS số</th>
                <th>Tiêu đề</th>
                <th>Tài liệu trên máy chủ</th>
                {ketQua && <th>Kết quả</th>}
              </tr>
            </thead>
            <tbody>
              {ghep.map(({ r, ds }, i) => {
                const kq = ketQua?.[i];
                return (
                  <tr key={r.id}>
                    <td className="text-center">{r.hopSo}</td>
                    <td className="text-center">{r.soTam}</td>
                    <td className="text-left">{r.tieuDe}</td>
                    <td className={`text-center ${ds.length ? "" : "text-danger-700"}`}>
                      {docs == null ? "…" : ds.length ? `${ds.length} tài liệu` : "không khớp"}
                    </td>
                    {ketQua && (
                      <td className="text-left text-sm">
                        {kq ? (
                          <>
                            {kq.ok > 0 && <span className="text-success-700">Đã tạo {kq.ok} gói. </span>}
                            {kq.loi.length > 0 && <span className="text-danger-700">{kq.loi.join(" · ")}</span>}
                          </>
                        ) : (
                          "…"
                        )}
                      </td>
                    )}
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
        <div className="flex items-center gap-2 mt-3">
          <span className="muted text-sm">
            {khop}/{dossiers.length} hồ sơ khớp tài liệu trên máy chủ. Máy chủ dựng gói nền — theo dõi ở “Danh sách gói tin”.
          </span>
          <div className="flex-1" />
          <button className="btn btn-success btn-round" onClick={() => void chay()} disabled={busy || khop === 0 || !docs}>
            <Icon name="goi" /> {busy ? "Đang tạo gói…" : `Đóng gói ${V4sk[kind]}`}
          </button>
        </div>
        </div>
      </div>
    </div>
  );
}

const TRANG_THAI: Record<string, string> = {
  queued: "Chờ dựng",
  building: "Đang dựng",
  built: "Xong",
  failed: "Lỗi",
};

/** Danh sách gói tin trên máy chủ + tải về (thư mục _goi_tin trong Kho). */
export function PackageList() {
  const [list, setList] = useState<V4rt[]>([]);
  const [msg, setMsg] = useState<string | null>(null);
  const load = async () => {
    if (!isTauri()) return;
    try {
      setList(await V4ru());
    } catch (e) {
      setMsg(errText(e));
    }
  };
  V41p(load, { topics: ["docs"], pollMs: 10000 });
  const tai = async (p: V4rt) => {
    try {
      const path = await V4rw(p.id, p.zipName ?? `${p.label}.zip`);
      setMsg(`Đã tải về ${path}`);
    } catch (e) {
      setMsg(errText(e));
    }
  };
  return (
    <div className="content-panel pt-3">
      <h4>
        <b>Danh sách gói tin</b>
      </h4>
      {msg && <div className="muted text-sm">{msg}</div>}
      {list.length === 0 ? (
        <p className="muted">Chưa có gói tin nào.</p>
      ) : (
        <div className="tbl-scroll">
          <table className="dataTable">
            <thead>
              <tr>
                <th>Nhãn</th>
                <th>Loại</th>
                <th>Trạng thái</th>
                <th>Tài liệu</th>
                <th>Kích thước</th>
                <th />
              </tr>
            </thead>
            <tbody>
              {list.map((p) => (
                <tr key={p.id}>
                  <td className="text-left">{p.label}</td>
                  <td>{V4sk[p.kind as V4sj] ?? p.kind}</td>
                  <td title={p.error ?? undefined} className={p.status === "failed" ? "text-danger-700" : undefined}>
                    {TRANG_THAI[p.status] ?? p.status}
                    {p.status === "failed" && p.error ? ` — ${p.error}` : ""}
                  </td>
                  <td className="text-center">{p.documentCount}</td>
                  <td className="text-center">{p.zipSize != null ? `${(p.zipSize / 1048576).toFixed(1)} MB` : "—"}</td>
                  <td>
                    {p.status === "built" && p.zipName && (
                      <button className="btn btn-outline-primary btn-round" onClick={() => void tai(p)}>
                        <Icon name="taiVe" /> Tải về
                      </button>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
