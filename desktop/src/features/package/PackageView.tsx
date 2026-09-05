// Copyright (c) 2026 DigiText. All rights reserved.

import { useCallback, useMemo, useState } from "react";
import { V41p } from "@/live/useLive";

import { V4ei, type V4eg } from "@/api/docs";
import { isTauri } from "@/api/ipc";
import { V4rv, V4rw, V4ru, type V4rt } from "@/api/packaging";
import { V4s2, type V4rx } from "@/api/submissions";
import { V460 } from "@/state";

import { V4su, V4sn, V4sk, V4ss, type V4sm, type V4sj } from "./packageWire";
import { Icon } from "@/components/Icon";

const V4sv: Record<string, string> = {
  queued: "Chờ dựng",
  building: "Đang dựng",
  built: "Hoàn tất",
  failed: "Lỗi",
};

function V4sw(V2ov: number | null): string {
  if (V2ov == null) return "—";
  if (V2ov < 1024) return `${V2ov} B`;
  if (V2ov < 1024 * 1024) return `${(V2ov / 1024).toFixed(1)} KB`;
  return `${(V2ov / 1024 / 1024).toFixed(1)} MB`;
}

export default function V4sx() {
  const { session: V2ow } = V460();
  const [V2ox, V2oy] = useState<V4sm>(V4sn);
  const [V2oz, V2p0] = useState("");
  const [V2p1, V2p2] = useState<V4eg[]>([]);
  const [V2p3, V2p4] = useState<Set<string>>(new Set());
  const [V2p5, V2p6] = useState<V4rt[]>([]);
  const [V2p7, V2p8] = useState<V4rx[]>([]);
  const [V2p9, V2pa] = useState("");
  const [V2pb, V2pc] = useState("");
  const [V2pd, V2pe] = useState("");
  const [V2pf, V2pg] = useState(false);
  const [V2ph, V2pi] = useState<string | null>(null);
  const [V2pj, V2pk] = useState<string[]>([]);

  const set = <K extends keyof V4sm>(V2pl: K, V2pm: V4sm[K]) =>
    V2oy((f) => ({ ...f, [V2pl]: V2pm }));

  const V2pn = V2ox.kind === "sip_hoso" || V2ox.kind === "sip_tailieu";
  const V2po = V2ox.kind === "aip_hoso" || V2ox.kind === "sip_hoso";
  const V2pp = V2ox.kind === "dip";

  const V2pq = useCallback(async () => {
    if (!isTauri()) return;
    try {
      V2p6(await V4ru());
    } catch (V2pr) {
      V2pi(`Không tải được danh sách gói: ${String(V2pr)}`);
    }
  }, []);

  V41p(V2pq, { topics: ["packages"] });

  const V2ps = V2ox.kind === "sip_hoso" || V2ox.kind === "sip_tailieu";
  const V2pt = useCallback(() => {
    V4s2()
      .then((V2pu) =>
        V2p8(V2pu.filter((r) => r.status === "confirmed" || r.status === "awaiting_reception")),
      )
      .catch(() => V2p8([]));
  }, []);

  V41p(V2pt, {
    topics: ["submissions"],
    enabled: V2ps && isTauri(),
  });

  const V2pv = async () => {
    V2pg(true);
    V2pi(null);
    try {
      const V2pw = await V4ei();
      const V2px = V2oz.trim();
      const V2py = V2px ? V2pw.filter((d) => (d.hoSoRef ?? "") === V2px) : V2pw;
      V2p2(V2py);
      V2p4(new Set(V2py.map((d) => d.id)));
      if (V2py.length === 0) {
        V2pi(
          V2px
            ? `Không có tài liệu nào trên máy chủ khớp mã hồ sơ "${V2px}".`
            : "Máy chủ chưa có tài liệu nào trong phạm vi của bạn.",
        );
      }
    } catch (V2pz) {
      V2pi(`Không tải được tài liệu từ máy chủ: ${String(V2pz)}`);
    } finally {
      V2pg(false);
    }
  };

  const V2q0 = (V2q1: string) =>
    V2p4((V2q2) => {
      const V2q3 = new Set(V2q2);
      if (V2q3.has(V2q1)) V2q3.delete(V2q1);
      else V2q3.add(V2q1);
      return V2q3;
    });

  const V2q4 = useMemo(
    () =>
      V2p1.filter((d) => V2p3.has(d.id)).map((d) => ({ documentId: d.id, filename: d.filename })),
    [V2p1, V2p3],
  );

  const V2q5 = async () => {
    const { request: V2q6, errors: V2q7 } = V4su(V2ox, V2q4);

    const V2q8 = V2pn ? V4ss(V2pb, V2pd) : { range: null, error: null };
    const V2q9 = V2q8.error ? [...V2q7, V2q8.error] : V2q7;
    V2pk(V2q9);
    if (!V2q6 || V2q9.length > 0) return;
    V2pg(true);
    V2pi(null);
    try {
      const row = await V4rv({
        projectId: V2ow?.projectId ?? null,
        hoSoRef: V2oz.trim() || null,
        submissionId: V2pn && V2p9 ? V2p9 : null,
        sttFrom: V2q8.range ? V2q8.range[0] : null,
        sttTo: V2q8.range ? V2q8.range[1] : null,
        request: V2q6,
      });
      V2pi(
        `Đã tạo yêu cầu đóng gói "${row.label}" — máy chủ sẽ dựng gói nền.`,
      );
      await V2pq();
    } catch (V2qa) {
      V2pi(`Tạo yêu cầu thất bại: ${String(V2qa)}`);
    } finally {
      V2pg(false);
    }
  };

  const V2qb = async (row: V4rt) => {
    if (!row.zipName) return;
    V2pg(true);
    try {
      const path = await V4rw(row.id, row.zipName);
      V2pi(`Đã lưu gói tin: ${path}`);
    } catch (V2qc) {
      V2pi(`Tải gói thất bại: ${String(V2qc)}`);
    } finally {
      V2pg(false);
    }
  };

  return (
    <section className="view active">
      <div className="page-head">
        <h2>Đóng gói theo tiêu chuẩn (SIP / AIP / DIP)</h2>
        <div className="page-actions">
          <button className="btn" onClick={() => void V2pq()} disabled={V2pf}>
            <Icon name="lamMoi" />
            Làm mới danh sách
          </button>
        </div>
      </div>

      {V2ph && <div className="card p-2.5 mb-2.5">{V2ph}</div>}

      <div className="card p-4 mb-4">
        <h3 className="mt-0">Tạo gói tin mới</h3>
        <div className="grid gap-2.5 [grid-template-columns:repeat(auto-fill,minmax(220px,1fr))]">
          <div className="field">
            <label>Loại gói (Điều 17 / 28 / 42)</label>
            <select
              className="inp"
              value={V2ox.kind}
              onChange={(V2qd) => set("kind", V2qd.target.value as V4sj)}
            >
              {(Object.keys(V4sk) as V4sj[]).map((k) => (
                <option key={k} value={k}>
                  {V4sk[k]}
                </option>
              ))}
            </select>
          </div>
          <div className="field">
            <label>Mã phông (Fondcode)</label>
            <input
              className="inp"
              value={V2ox.fondCode}
              onChange={(V2qe) => set("fondCode", V2qe.target.value)}
              placeholder="G09"
            />
          </div>
          <div className="field">
            <label>Mã cơ quan lưu trữ</label>
            <input
              className="inp"
              value={V2ox.archiveOrgCode}
              onChange={(V2qf) => set("archiveOrgCode", V2qf.target.value)}
              placeholder="V01"
            />
          </div>
          {!V2pp && (
            <>
              <div className="field">
                <label>{V2po ? "Mã hồ sơ lưu trữ" : "Mã gói tin"}</label>
                <input
                  className="inp"
                  value={V2ox.fileCode}
                  onChange={(V2qg) => set("fileCode", V2qg.target.value)}
                  placeholder="V01.G09.2021.01"
                />
              </div>
              <div className="field">
                <label>Tiêu đề</label>
                <input
                  className="inp"
                  value={V2ox.title}
                  onChange={(V2qh) => set("title", V2qh.target.value)}
                />
              </div>
              <div className="field">
                <label>Thời hạn lưu trữ</label>
                <select
                  className="inp"
                  value={V2ox.maintenance}
                  onChange={(V2qi) => set("maintenance", V2qi.target.value)}
                >
                  <option value="01">01 — Vĩnh viễn</option>
                  <option value="02">02 — 70 năm</option>
                  <option value="03">03 — 50 năm</option>
                  <option value="04">04 — 30 năm</option>
                  <option value="05">05 — 20 năm</option>
                  <option value="06">06 — 10 năm</option>
                  <option value="07">07 — Khác</option>
                </select>
              </div>
              <div className="field">
                <label>Chế độ sử dụng</label>
                <select
                  className="inp"
                  value={V2ox.mode}
                  onChange={(V2qj) => set("mode", V2qj.target.value)}
                >
                  <option value="01">01 — Công khai</option>
                  <option value="02">02 — Có điều kiện</option>
                  <option value="03">03 — Mật</option>
                </select>
              </div>
              <div className="field">
                <label>Mức độ tin cậy</label>
                <select
                  className="inp"
                  value={V2ox.confidenceLevel}
                  onChange={(V2qk) => set("confidenceLevel", V2qk.target.value)}
                >
                  <option value="01">01 — Gốc điện tử</option>
                  <option value="02">02 — Số hóa</option>
                  <option value="03">03 — Hỗn hợp</option>
                </select>
              </div>
            </>
          )}
          {V2po && (
            <>
              <div className="field">
                <label>Số lượng tờ (số hóa)</label>
                <input
                  className="inp"
                  value={V2ox.numberOfPaper}
                  onChange={(V2ql) => set("numberOfPaper", V2ql.target.value)}
                />
              </div>
              <div className="field">
                <label>Mã hồ sơ gốc giấy</label>
                <input
                  className="inp"
                  value={V2ox.paperFileCode}
                  onChange={(V2qm) => set("paperFileCode", V2qm.target.value)}
                  placeholder="V01.01.001"
                />
              </div>
            </>
          )}
          {(V2ox.kind === "aip_tailieu" || V2ox.kind === "sip_tailieu") && (
            <div className="field">
              <label>Nguồn gốc</label>
              <select
                className="inp"
                value={V2ox.source}
                onChange={(V2qn) => set("source", V2qn.target.value)}
              >
                <option value="0">0 — Văn bản đi</option>
                <option value="1">1 — Văn bản đến</option>
              </select>
            </div>
          )}
          {V2pn && (
            <>
              <div className="field">
                <label>Mã yêu cầu nộp (thoả thuận)</label>
                <input
                  className="inp"
                  value={V2ox.submissionAgreement}
                  onChange={(V2qo) => set("submissionAgreement", V2qo.target.value)}
                  placeholder="NL.BNV.2026.01"
                />
              </div>
              <div className="field">
                <label>Số thứ tự trong lần nộp</label>
                <input
                  className="inp"
                  value={V2ox.zipOrdinal}
                  onChange={(V2qp) => set("zipOrdinal", V2qp.target.value)}
                />
              </div>
              <div className="field">
                <label>Lần nộp (thu nộp Điều 22.3.c)</label>
                <select className="inp" value={V2p9} onChange={(V2qq) => V2pa(V2qq.target.value)}>
                  <option value="">— Không gắn —</option>
                  {V2p7.map((r) => (
                    <option key={r.id} value={r.id}>
                      {r.maYeuCau} · {r.orgName}
                    </option>
                  ))}
                </select>
              </div>
              <div className="field">
                <label>STT Mục lục — từ (PK-5, trống = tự suy)</label>
                <input
                  className="inp"
                  value={V2pb}
                  onChange={(V2qr) => V2pc(V2qr.target.value)}
                  placeholder="1"
                />
              </div>
              <div className="field">
                <label>STT Mục lục — đến</label>
                <input
                  className="inp"
                  value={V2pd}
                  onChange={(V2qs) => V2pe(V2qs.target.value)}
                  placeholder="1"
                />
              </div>
            </>
          )}
          {V2pp && (
            <>
              <div className="field">
                <label>Mã yêu cầu khai thác</label>
                <input
                  className="inp"
                  value={V2ox.requestId}
                  onChange={(V2qt) => set("requestId", V2qt.target.value)}
                  placeholder="YC.2026.0001"
                />
              </div>
              <div className="field">
                <label>Ngày yêu cầu (DD/MM/YYYY)</label>
                <input
                  className="inp"
                  value={V2ox.requestDate}
                  onChange={(V2qu) => set("requestDate", V2qu.target.value)}
                />
              </div>
              <div className="field">
                <label>Mục đích</label>
                <select
                  className="inp"
                  value={V2ox.purpose}
                  onChange={(V2qv) => set("purpose", V2qv.target.value)}
                >
                  <option value="01">01 — Cá nhân</option>
                  <option value="02">02 — Công vụ</option>
                  <option value="03">03 — Công vụ đặc biệt</option>
                </select>
              </div>
              <div className="field">
                <label>
                  <input
                    type="checkbox"
                    checked={V2ox.includePremis}
                    onChange={(V2qw) => set("includePremis", V2qw.target.checked)}
                  />{" "}
                  Kèm PREMIS (tùy chọn)
                </label>
              </div>
              <div className="field">
                <label>Mã xác thực lưu trữ (42.2.a)</label>
                <input
                  className="inp"
                  value={V2ox.certMaXacThuc}
                  onChange={(V2qx) => set("certMaXacThuc", V2qx.target.value)}
                  placeholder="XT.2026.0001"
                />
              </div>
              <div className="field">
                <label>Tên lưu trữ lịch sử (42.2.b)</label>
                <input
                  className="inp"
                  value={V2ox.certTenLuuTru}
                  onChange={(V2qy) => set("certTenLuuTru", V2qy.target.value)}
                />
              </div>
              <div className="field">
                <label>Người nhận — họ tên (42.2.c)</label>
                <input
                  className="inp"
                  value={V2ox.certNguoiNhanHoTen}
                  onChange={(V2qz) => set("certNguoiNhanHoTen", V2qz.target.value)}
                />
              </div>
              <div className="field">
                <label>Người nhận — mã định danh (42.2.c)</label>
                <input
                  className="inp"
                  value={V2ox.certNguoiNhanDinhDanh}
                  onChange={(V2r0) => set("certNguoiNhanDinhDanh", V2r0.target.value)}
                />
              </div>
              <div className="field">
                <label>Mã lưu trữ tài liệu gốc (42.2.d)</label>
                <input
                  className="inp"
                  value={V2ox.certMaLuuTruGoc}
                  onChange={(V2r1) => set("certMaLuuTruGoc", V2r1.target.value)}
                />
              </div>
              <div className="field">
                <label>Số, ký hiệu tài liệu gốc (42.2.đ)</label>
                <input
                  className="inp"
                  value={V2ox.certSoKyHieuGoc}
                  onChange={(V2r2) => set("certSoKyHieuGoc", V2r2.target.value)}
                />
              </div>
              <div className="field">
                <label>Tên loại tài liệu (42.2.e)</label>
                <input
                  className="inp"
                  value={V2ox.certTenLoai}
                  onChange={(V2r3) => set("certTenLoai", V2r3.target.value)}
                  placeholder="Quyết định"
                />
              </div>
              <div className="field">
                <label>Trích yếu / tiêu đề (42.2.g)</label>
                <input
                  className="inp"
                  value={V2ox.certTrichYeu}
                  onChange={(V2r4) => set("certTrichYeu", V2r4.target.value)}
                />
              </div>
              <div className="field">
                <label>Mục đích sử dụng (42.2.h)</label>
                <input
                  className="inp"
                  value={V2ox.certMucDich}
                  onChange={(V2r5) => set("certMucDich", V2r5.target.value)}
                />
              </div>
              <div className="field">
                <label>Thời hạn sử dụng (42.2.k)</label>
                <input
                  className="inp"
                  value={V2ox.certThoiHan}
                  onChange={(V2r6) => set("certThoiHan", V2r6.target.value)}
                />
              </div>
              <div className="field">
                <label>Số lượng bản (42.2.l)</label>
                <input
                  className="inp"
                  value={V2ox.certSoLuongBan}
                  onChange={(V2r7) => set("certSoLuongBan", V2r7.target.value)}
                />
              </div>
            </>
          )}
        </div>

        <div className="flex gap-2 items-end mt-3">
          <div className="field flex-1">
            <label>Mã hồ sơ trên máy chủ (hoSoRef) — lọc tài liệu đã nộp</label>
            <input
              className="inp"
              value={V2oz}
              onChange={(V2r8) => V2p0(V2r8.target.value)}
              placeholder="Hộp 01/Hồ sơ 001"
            />
          </div>
          <button className="btn" onClick={() => void V2pv()} disabled={V2pf || !isTauri()}>
            Nạp tài liệu từ máy chủ
          </button>
        </div>

        {V2p1.length > 0 && (
          <div className="tbl-scroll mt-2.5">
            <table className="tbl">
              <thead>
                <tr>
                  <th className="w-8"></th>
                  <th>Tệp</th>
                  <th>Mã hồ sơ</th>
                  <th>Trang</th>
                </tr>
              </thead>
              <tbody>
                {V2p1.map((d) => (
                  <tr key={d.id}>
                    <td>
                      <input type="checkbox" checked={V2p3.has(d.id)} onChange={() => V2q0(d.id)} />
                    </td>
                    <td>{d.filename}</td>
                    <td>{d.hoSoRef ?? "—"}</td>
                    <td>{d.pageCount ?? "—"}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}

        {V2pj.length > 0 && (
          <ul className="mt-2.5 text-danger-700">
            {V2pj.map((V2r9) => (
              <li key={V2r9}>{V2r9}</li>
            ))}
          </ul>
        )}

        <div className="mt-3">
          <button
            className="btn primary"
            onClick={() => void V2q5()}
            disabled={V2pf || V2q4.length === 0}
          >
            <Icon name="goi" />
            Tạo gói ({V2q4.length} tài liệu)
          </button>
        </div>
      </div>

      <div className="card p-4">
        <h3 className="mt-0">Danh sách gói tin</h3>
        {V2p5.length === 0 ? (
          <p className="muted">Chưa có gói tin nào. Tạo gói ở khung trên — máy chủ sẽ dựng nền.</p>
        ) : (
          <div className="tbl-scroll">
            <table className="tbl">
              <thead>
                <tr>
                  <th>Nhãn</th>
                  <th>Loại</th>
                  <th>Trạng thái</th>
                  <th>Tài liệu</th>
                  <th>STT Mục lục</th>
                  <th>ZIP</th>
                  <th>Kích thước</th>
                  <th></th>
                </tr>
              </thead>
              <tbody>
                {V2p5.map((p) => (
                  <tr key={p.id}>
                    <td title={p.objid ?? undefined}>{p.label}</td>
                    <td>{V4sk[p.kind as V4sj] ?? p.kind}</td>
                    <td title={p.error ?? undefined}>
                      {V4sv[p.status] ?? p.status}
                      {p.status === "failed" && p.error ? <Icon name="canhBao" size={13} /> : null}
                    </td>
                    <td>{p.documentCount}</td>
                    <td>
                      {p.sttFrom != null && p.sttTo != null ? `${p.sttFrom}–${p.sttTo}` : "—"}
                    </td>
                    <td>{p.zipName ?? "—"}</td>
                    <td>{V4sw(p.zipSize)}</td>
                    <td>
                      {p.status === "built" && p.zipName && (
                        <button className="btn sm" onClick={() => void V2qb(p)} disabled={V2pf}>
                          <Icon name="taiVe" />
                          Tải về
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
    </section>
  );
}
