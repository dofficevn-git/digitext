// Copyright (c) 2026 DigiText. All rights reserved.

import { useCallback, useState } from "react";
import { V41p } from "@/live/useLive";

import { isTauri } from "@/api/ipc";
import { V4rw } from "@/api/packaging";
import {
  V4sd,
  V4sc,
  V4se,
  V4s6,
  V4s9,
  V4si,
  V4s3,
  V4s4,
  V4sh,
  V4s7,
  V4s2,
  type V4sb,
  type V4s0,
  type V4sg,
  type V4rz,
  type V4rx,
} from "@/api/submissions";
import { V4t3, type V4t2 } from "@/api/records";
import { V460 } from "@/state";

import { V4t6, V4t4, V4t7 } from "./submissionFlow";
import { Icon } from "@/components/Icon";

const V4t8: V4s0 = { maHoSo: "", tieuDe: "" };

export default function V4t9() {
  const { session: V2rn } = V460();
  const V2ro = V2rn?.permissions ?? [];
  const [V2rp, V2rq] = useState<V4rx[]>([]);
  const [V2rr, V2rs] = useState<V4rz | null>(null);
  const [V2rt, V2ru] = useState<V4sb | null>(null);
  const [V2rv, V2rw] = useState<V4sg | null>(null);
  const [V2rx, V2ry] = useState(false);
  const [V2rz, V2s0] = useState(false);
  const [V2s1, V2s2] = useState<string | null>(null);
  const [V2s3, V2s4] = useState<string[]>([]);

  const [V2s5, V2s6] = useState("");
  const [V2s7, V2s8] = useState("");
  const [V2s9, V2sa] = useState("");
  const [V2sb, V2sc] = useState("");
  const [V2sd, V2se] = useState("");
  const [V2sf, V2sg] = useState<V4s0[]>([{ ...V4t8 }]);

  const reload = useCallback(async () => {
    if (!isTauri()) return;
    try {
      V2rq(await V4s2());
    } catch (V2sh) {
      V2s2(`Không tải được danh sách lần nộp: ${String(V2sh)}`);
    }

    try {
      V2ru(await V4sc());
    } catch {
      V2ru(null);
    }
  }, []);

  V41p(reload, { topics: ["submissions", "notifications", "docs"] });

  const V2si = async (V2sj: string) => {
    V2s0(true);
    try {
      const d = await V4s3(V2sj);
      V2rs(d);
      V2ry(false);

      if (d.status === "approved" || d.status === "completed") {
        try {
          V2rw(await V4sh(V2sj));
        } catch {
          V2rw(null);
        }
      } else {
        V2rw(null);
      }
    } catch (V2sk) {
      V2s2(`Không tải được chi tiết: ${String(V2sk)}`);
    } finally {
      V2s0(false);
    }
  };

  const V2sl = async (V2sm: string) => {
    try {
      await V4sd(V2sm);
      V2ru(await V4sc());
    } catch (V2sn) {
      V2s2(`Đánh dấu đã đọc lỗi: ${String(V2sn)}`);
    }
  };

  const V2so = async () => {
    try {
      await V4se();
      V2ru(await V4sc());
    } catch (V2sp) {
      V2s2(`Đánh dấu đã đọc lỗi: ${String(V2sp)}`);
    }
  };

  const V2sq = (i: number, V2sr: Partial<V4s0>) =>
    V2sg((V2ss) => V2ss.map((V2st, V2su) => (V2su === i ? { ...V2st, ...V2sr } : V2st)));

  const V2sv = async () => {
    const V2sw = {
      projectId: V2rn?.projectId ?? null,
      orgName: V2s5,
      orgCode: V2s7,
      fondCode: V2s9,
      summary: V2sb.trim() || null,
      cachThuc: "truc_tuyen" as const,
      hanNop: V2sd || null,
      items: V2sf.filter((V2sx) => V2sx.maHoSo.trim() || V2sx.tieuDe.trim()),
    };
    const V2sy = V4t7(V2sw);
    V2s4(V2sy);
    if (V2sy.length > 0) return;
    V2s0(true);
    V2s2(null);
    try {
      const V2sz = await V4s4(V2sw);
      V2s2(`Đã đăng ký lần nộp — mã yêu cầu ${V2sz.maYeuCau}.`);
      V2ry(false);
      V2sg([{ ...V4t8 }]);
      await reload();
      V2rs(V2sz);
    } catch (V2t0) {
      V2s2(`Đăng ký thất bại: ${String(V2t0)}`);
    } finally {
      V2s0(false);
    }
  };

  const V2t1 = async (V2t2: string, V2t3: string, V2t4?: boolean) => {
    let V2t5: string | undefined;
    if (V2t4) {
      V2t5 = window.prompt("Lý do (bắt buộc):")?.trim() || undefined;
      if (!V2t5) return;
    } else if (V2t3 === "confirm" || V2t3 === "receive") {
      V2t5 = window.prompt("Ghi chú (tuỳ chọn):")?.trim() || undefined;
    }
    V2s0(true);
    V2s2(null);
    try {
      if (V2t3 === "approve") {
        const V2t6 = window
          .prompt("Số Quyết định phê duyệt (Điều 25.1):")
          ?.trim();
        if (!V2t6) return;
        const V2t7 = await V4s9(V2t2, V2t6, V2t5);
        V2s2(
          `Đã phê duyệt — ${V2t7.convertedAipIds.length} gói AIP chuyển hóa đang dựng nền (Điều 25.3.a).`,
        );
      } else {
        await V4s6(V2t2, V2t3 as never, V2t5);
      }
      await reload();
      await V2si(V2t2);
    } catch (V2t8) {
      V2s2(`Thao tác thất bại: ${String(V2t8)}`);
    } finally {
      V2s0(false);
    }
  };

  const V2t9 = async (V2ta: string, V2tb: string, V2tc: "dat" | "khong_dat") => {
    let V2td: string | undefined;
    if (V2tc === "khong_dat") {
      V2td = window.prompt("Lý do không đạt (Điều 24.2):")?.trim() || undefined;
      if (!V2td) return;
    }
    V2s0(true);
    try {
      await V4s7(V2ta, V2tb, V2tc, V2td);
      await V2si(V2ta);
    } catch (V2te) {
      V2s2(`Chấm hồ sơ thất bại: ${String(V2te)}`);
    } finally {
      V2s0(false);
    }
  };

  return (
    <section className="view active">
      <div className="page-head">
        <h2>Thu nộp lưu trữ lịch sử (Điều 20–25)</h2>
        <div className="page-actions">
          <button className="btn" onClick={() => void reload()} disabled={V2rz}>
            <Icon name="lamMoi" />
            Làm mới
          </button>
          {V2ro.includes("submission.create") && (
            <button className="btn primary" onClick={() => V2ry((v) => !v)}>
              + Đăng ký nộp
            </button>
          )}
        </div>
      </div>

      {V2s1 && <div className="card p-2.5 mb-2.5">{V2s1}</div>}

      {V2rt && V2rt.items.length > 0 && (
        <div className="card p-3 mb-3">
          <div className="flex justify-between items-center">
            <h3 className="m-0">
              <Icon name="thongBao" /> Thông báo
              {V2rt.unread > 0 ? ` (${V2rt.unread} chưa đọc)` : ""}
            </h3>
            {V2rt.unread > 0 && (
              <button className="btn sm" onClick={() => void V2so()} disabled={V2rz}>
                Đọc tất cả
              </button>
            )}
          </div>
          <ul className="mt-2 mx-0 mb-0 pl-4.5">
            {V2rt.items.slice(0, 8).map((n) => (
              <li key={n.id} className={`mb-1${n.readAt ? " opacity-55" : ""}`}>
                {n.kind.startsWith("sla_") && <Icon name="nhacHan" size={13} />} {n.message}{" "}
                {!n.readAt && (
                  <button className="btn sm" onClick={() => void V2sl(n.id)} disabled={V2rz}>
                    Đã đọc
                  </button>
                )}
              </li>
            ))}
          </ul>
        </div>
      )}

      {V2rx && (
        <div className="card p-4 mb-4">
          <h3 className="mt-0">Đăng ký nộp (Điều 20)</h3>
          <div className="grid gap-2.5 [grid-template-columns:repeat(auto-fill,minmax(220px,1fr))]">
            <div className="field">
              <label>Tên cơ quan, tổ chức nộp</label>
              <input className="inp" value={V2s5} onChange={(V2tf) => V2s6(V2tf.target.value)} />
            </div>
            <div className="field">
              <label>Mã cơ quan nộp</label>
              <input className="inp" value={V2s7} onChange={(V2tg) => V2s8(V2tg.target.value)} />
            </div>
            <div className="field">
              <label>Mã phông</label>
              <input className="inp" value={V2s9} onChange={(V2th) => V2sa(V2th.target.value)} />
            </div>
            <div className="field">
              <label>Tóm tắt khối tài liệu</label>
              <input className="inp" value={V2sb} onChange={(V2ti) => V2sc(V2ti.target.value)} />
            </div>
            <div className="field">
              <label>Hạn nộp tài liệu (Điều 20.2 — đăng ký trước ≥ 12 tháng)</label>
              <input
                className="inp"
                type="date"
                value={V2sd}
                onChange={(V2tj) => V2se(V2tj.target.value)}
              />
            </div>
          </div>

          <h4>Mục lục hồ sơ nộp (Điều 20.3.k)</h4>
          <div className="tbl-scroll">
            <table className="tbl">
              <thead>
                <tr>
                  <th>#</th>
                  <th>Mã hồ sơ</th>
                  <th>Tiêu đề</th>
                  <th>Số trang</th>
                  <th></th>
                </tr>
              </thead>
              <tbody>
                {V2sf.map((V2tk, i) => (
                  <tr key={i}>
                    <td>{i + 1}</td>
                    <td>
                      <input
                        className="inp"
                        value={V2tk.maHoSo}
                        onChange={(V2tl) => V2sq(i, { maHoSo: V2tl.target.value })}
                      />
                    </td>
                    <td>
                      <input
                        className="inp"
                        value={V2tk.tieuDe}
                        onChange={(V2tm) => V2sq(i, { tieuDe: V2tm.target.value })}
                      />
                    </td>
                    <td>
                      <input
                        className="inp w-22.5"
                        value={V2tk.soTrang ?? ""}
                        onChange={(V2tn) =>
                          V2sq(i, { soTrang: V2tn.target.value ? Number(V2tn.target.value) : null })
                        }
                      />
                    </td>
                    <td>
                      <button
                        className="btn sm"
                        onClick={() => V2sg((V2to) => V2to.filter((_V2tp, V2tq) => V2tq !== i))}
                        disabled={V2sf.length === 1}
                      >
                        <Icon name="xoa" size={14} label="Bỏ dòng" />
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <div className="flex gap-2 mt-2.5">
            <button className="btn" onClick={() => V2sg((V2tr) => [...V2tr, { ...V4t8 }])}>
              + Thêm hồ sơ
            </button>
            <button className="btn primary" onClick={() => void V2sv()} disabled={V2rz}>
              Gửi đăng ký
            </button>
          </div>
          {V2s3.length > 0 && (
            <ul className="mt-2.5 text-danger-700">
              {V2s3.map((V2ts) => (
                <li key={V2ts}>{V2ts}</li>
              ))}
            </ul>
          )}
        </div>
      )}

      <div className="card p-4 mb-4">
        <h3 className="mt-0">Danh sách lần nộp</h3>
        {V2rp.length === 0 ? (
          <p className="muted">Chưa có lần nộp nào.</p>
        ) : (
          <div className="tbl-scroll">
            <table className="tbl">
              <thead>
                <tr>
                  <th>Mã yêu cầu</th>
                  <th>Cơ quan nộp</th>
                  <th>Phông</th>
                  <th>Lần</th>
                  <th>Trạng thái</th>
                  <th></th>
                </tr>
              </thead>
              <tbody>
                {V2rp.map((r) => (
                  <tr key={r.id}>
                    <td>{r.maYeuCau}</td>
                    <td>{r.orgName}</td>
                    <td>{r.fondCode}</td>
                    <td>{r.soLanNop}</td>
                    <td title={r.reconcileNote ?? r.decisionNote ?? undefined}>
                      {V4t4[r.status] ?? r.status}
                    </td>
                    <td>
                      <button className="btn sm" onClick={() => void V2si(r.id)} disabled={V2rz}>
                        Chi tiết
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {V2rr && (
        <div className="card p-4">
          <h3 className="mt-0">
            {V2rr.maYeuCau} — {V4t4[V2rr.status] ?? V2rr.status}
          </h3>
          <p className="muted">
            {V2rr.orgName} · phông {V2rr.fondCode} · lần nộp {V2rr.soLanNop}
            {V2rr.reconcileNote ? ` · đối chiếu: ${V2rr.reconcileNote}` : ""}
            {V2rr.decisionNote ? ` · ý kiến: ${V2rr.decisionNote}` : ""}
            {V2rr.approvalDoc ? ` · quyết định: ${V2rr.approvalDoc}` : ""}
          </p>

          {V2rr.duplicateInSystem.length > 0 && (
            <p className="text-danger-700">
              <Icon name="canhBao" /> Trùng với hồ sơ đã có trong Hệ thống (Điều 24.1.b):{" "}
              {V2rr.duplicateInSystem.join(", ")} — viên chức quyết định tiếp nhận hay không.
            </p>
          )}

          <div className="flex gap-2 flex-wrap mb-3">
            {V4t6(V2rr.status, V2ro).map((a) => (
              <button
                key={a.action}
                className="btn primary"
                disabled={V2rz}
                onClick={() => void V2t1(V2rr.id, a.action, a.requiresNote)}
              >
                {a.label}
              </button>
            ))}
            {V2rr.status === "confirmed" && (
              <span className="muted">
                Nộp gói: tạo gói SIP ở màn “Đóng gói theo tiêu chuẩn” và chọn lần nộp này.
              </span>
            )}
            {(V2rr.status === "approved" || V2rr.status === "completed") &&
              V2ro.includes("submission.review") &&
              (
                [
                  ["thu_nop", "Lập biên bản thu nộp (Điều 25.2)"],
                  ["giao_nhan", "Biên bản giao nhận (Điều 25.5.a)"],
                  ["tra_lai", "Biên bản trả lại (Điều 25.5.a)"],
                ] as [V4t2, string][]
              ).map(([loai, V2tt]) => (
                <button
                  key={loai}
                  className="btn"
                  disabled={V2rz}
                  onClick={() => {
                    void (async () => {
                      V2s0(true);
                      try {
                        const V2tu = await V4t3(V2rr.id, undefined, loai);
                        V2s2(
                          `Đã lập biên bản số ${V2tu.soBienBan} — tải PDF ở màn Biên bản.`,
                        );
                      } catch (V2tv) {
                        V2s2(`Lập biên bản thất bại: ${String(V2tv)}`);
                      } finally {
                        V2s0(false);
                      }
                    })();
                  }}
                >
                  {V2tt}
                </button>
              ))}
            {(V2rr.status === "approved" || V2rr.status === "completed") && (
              <button
                className="btn"
                disabled={V2rz}
                onClick={() => {
                  void (async () => {
                    V2s0(true);
                    try {
                      const name = `ho-so-qua-trinh-${V2rr.maYeuCau.split("/").join("-")}.zip`;
                      const path = await V4si(V2rr.id, name);
                      V2s2(
                        `Đã lưu hồ sơ quá trình thu nộp (Điều 25.5.d): ${path}`,
                      );
                    } catch (V2tw) {
                      V2s2(
                        `Tải hồ sơ quá trình thất bại: ${String(V2tw)}`,
                      );
                    } finally {
                      V2s0(false);
                    }
                  })();
                }}
              >
                <Icon name="hoSo" />
                Hồ sơ quá trình (Điều 25.5.d)
              </button>
            )}
          </div>

          <h4>Mục lục hồ sơ ({V2rr.items.length})</h4>
          <div className="tbl-scroll">
            <table className="tbl">
              <thead>
                <tr>
                  <th>#</th>
                  <th>Mã hồ sơ</th>
                  <th>Tiêu đề</th>
                  <th>Kết quả (Điều 24)</th>
                  <th></th>
                </tr>
              </thead>
              <tbody>
                {V2rr.items.map((V2tx) => (
                  <tr key={V2tx.id}>
                    <td>{V2tx.stt}</td>
                    <td>{V2tx.maHoSo}</td>
                    <td>{V2tx.tieuDe}</td>
                    <td title={V2tx.reviewNote ?? undefined}>
                      {V2tx.reviewResult === "pending"
                        ? "—"
                        : V2tx.reviewResult === "dat"
                          ? "Đạt"
                          : `Không đạt${V2tx.reviewNote ? ` (${V2tx.reviewNote})` : ""}`}
                    </td>
                    <td>
                      {V2rr.status === "received" && V2ro.includes("submission.review") && (
                        <>
                          <button
                            className="btn sm"
                            disabled={V2rz}
                            onClick={() => void V2t9(V2rr.id, V2tx.id, "dat")}
                          >
                            Đạt
                          </button>{" "}
                          <button
                            className="btn sm"
                            disabled={V2rz}
                            onClick={() => void V2t9(V2rr.id, V2tx.id, "khong_dat")}
                          >
                            Không đạt
                          </button>
                        </>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          {V2rv && V2rv.totalReturned > 0 && (
            <>
              <h4>
                Hồ sơ trả lại (Điều 25.4) — {V2rv.totalReturned}/{V2rv.totalRegistered} (duyệt{" "}
                {V2rv.totalApproved})
              </h4>
              <div className="tbl-scroll">
                <table className="tbl">
                  <thead>
                    <tr>
                      <th>#</th>
                      <th>Mã hồ sơ</th>
                      <th>Tiêu đề</th>
                      <th>Lý do không đạt</th>
                      <th>Tải riêng (Điều 25.4.b)</th>
                    </tr>
                  </thead>
                  <tbody>
                    {V2rv.returnedItems.map((V2ty) => (
                      <tr key={V2ty.id}>
                        <td>{V2ty.stt}</td>
                        <td>{V2ty.maHoSo}</td>
                        <td>{V2ty.tieuDe}</td>
                        <td>{V2ty.reviewNote ?? "—"}</td>
                        <td>
                          {V2ty.sipPackageId ? (
                            <button
                              className="btn sm"
                              disabled={V2rz}
                              onClick={() => {
                                void (async () => {
                                  V2s0(true);
                                  try {
                                    const path = await V4rw(
                                      V2ty.sipPackageId as string,
                                      V2ty.sipZipName ?? `${V2ty.maHoSo}.zip`,
                                    );
                                    V2s2(`Đã tải hồ sơ trả lại: ${path}`);
                                  } catch (V2tz) {
                                    V2s2(
                                      `Tải hồ sơ trả lại thất bại: ${String(V2tz)}`,
                                    );
                                  } finally {
                                    V2s0(false);
                                  }
                                })();
                              }}
                            >
                              <Icon name="taiVe" />
                              {V2ty.sipZipName ?? "SIP"}
                            </button>
                          ) : (
                            "—"
                          )}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
              <p className="muted">
                Toàn bộ gói SIP đã nộp (màn Đóng gói → Tải về):{" "}
                {V2rv.sipPackages.map((p) => p.zipName ?? p.id).join(", ") || "—"}
              </p>
            </>
          )}

          <h4>Gói tin của lần nộp ({V2rr.packages.length})</h4>
          {V2rr.packages.length === 0 ? (
            <p className="muted">
              Chưa có gói nào — tạo gói SIP ở màn Đóng gói và gắn vào lần nộp.
            </p>
          ) : (
            <div className="tbl-scroll">
              <table className="tbl">
                <thead>
                  <tr>
                    <th>Nhãn</th>
                    <th>Loại</th>
                    <th>Trạng thái</th>
                    <th>Tài liệu</th>
                    <th>ZIP</th>
                  </tr>
                </thead>
                <tbody>
                  {V2rr.packages.map((p) => (
                    <tr key={p.id}>
                      <td title={p.objid ?? undefined}>{p.label}</td>
                      <td>
                        {p.kind}
                        {p.convertedFrom ? " (chuyển hóa)" : ""}
                      </td>
                      <td title={p.error ?? undefined}>{p.status}</td>
                      <td>{p.documentCount}</td>
                      <td>{p.zipName ?? "—"}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      )}
    </section>
  );
}
