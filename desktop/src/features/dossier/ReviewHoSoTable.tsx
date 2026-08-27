// Copyright (c) 2026 DigiText. All rights reserved.

import { Fragment, useCallback, useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { V41p } from "@/live/useLive";
import { V3xq, isTauri } from "@/api/ipc";
import { V3zt } from "@/state/sessionCore";
import { V4f8, V4ek, V4f7, type V4eh, type V4ej } from "@/api/docs";
import { V4hl } from "./ocrReport";
import {
  bocTachState,
  V4kq,
  V4kr,
  V4kv,
  V4kw,
  V4kf,
  V4ke,
  V4kx,
  V4ku,
  pdfaPercent,
  pdfaState,
  V4kt,
  V4ks,
  type V4kg,
  type V4k8,
  type V4kb,
  type V4kc,
  type V4k7,
} from "./reviewScope";
import "@/features/upload/khogom.css";
import "./ocrHoSo.css";
import { Icon } from "@/components/Icon";
import type { V41v } from "@/components/Icon";

const V4ky = (V1qf: unknown) =>
  V3zt(
    V1qf instanceof V3xq ? V1qf.code : "error",
    V1qf instanceof Error ? V1qf.message : String(V1qf),
  );

const fmtN = (n: number) => n.toLocaleString("vi-VN");
const V4kz = (b: number) => (b / 1_048_576).toFixed(1);

const V4l0: Record<V4eh, { icon: V41v; nut: string; hoi: string; xong: string }> = {
  approved: {
    icon: "duyet",
    nut: "Duyệt đã chọn",
    hoi: "Duyệt",
    xong: "Đã duyệt",
  },
  rejected: {
    icon: "tuChoi",
    nut: "Từ chối đã chọn",
    hoi: "Từ chối",
    xong: "Đã từ chối",
  },
  pending: {
    icon: "moLai",
    nut: "Mở lại đã chọn",
    hoi: "Mở lại",
    xong: "Đã mở lại",
  },
};

function V4l1({
  state,
  onChange: V1qg,
  title: V1qh,
  disabled,
}: {
  state: V4kg;
  onChange: (V1qi: boolean) => void;
  title: string;
  disabled?: boolean;
}) {
  const V1qj = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (V1qj.current) V1qj.current.indeterminate = state === "some";
  }, [state]);
  return (
    <input
      ref={V1qj}
      type="checkbox"
      checked={state === "all"}
      disabled={disabled}
      title={V1qh}
      onChange={(V1qk) => V1qg(V1qk.target.checked)}
    />
  );
}

function V4l2({ t }: { t: V4k7 }) {
  return (
    <>
      <div>
        {fmtN(t.pages)} trang · {V4kz(t.sizeBytes)} MB
      </div>
      {t.unknownPages > 0 && (
        <div
          className="muted oh-sub"
          title="Máy chủ chưa ghi số trang cho các tài liệu này"
        >
          {fmtN(t.unknownPages)} tài liệu chưa rõ số trang
        </div>
      )}
    </>
  );
}

function OcrCell({ t }: { t: V4k7 }) {
  const p = V4kx(t);
  const V1ql = pdfaPercent(t);
  return (
    <>
      <div className="oh-bar">
        <span className="oh-num">
          {fmtN(t.ocrPages)}/{fmtN(t.ocrDenom)}
        </span>
        <div
          className="progress"
          title={`${t.ocrPages}/${t.ocrDenom} trang đã có kết quả OCR (nút ① PDF 2 lớp)${
            t.unknownPages > 0
              ? ` · ${t.unknownPages}IHTDoGkgbGnhu4d1IGNoxrBhIHLDtSBz4buRIHRyYW5n`
              : ""
          }`}
        >
          <span style={{ width: `${p}%` }} />
        </div>
        <span className="oh-sub">{p}%</span>
      </div>
      {t.fieldDocs > 0 && (
        <div
          className="muted oh-sub"
          title="Tài liệu đã bóc được 16 trường mục lục (nút ② Bóc tách 16 trường) — việc này không ghi kết quả OCR từng trang"
        >
          {fmtN(t.fieldDocs)}/{fmtN(t.files)} tài liệu đã bóc 16 trường
        </div>
      )}
      <div className="oh-bar oh-pdfa">
        <span className="oh-sub">PDF/A</span>
        <span className="oh-num oh-sub">
          {fmtN(t.pdfaDocs)}/{fmtN(t.files)}
        </span>
        <div
          className="progress"
          title={`Đã chuyển đổi ${t.pdfaDocs}/${t.files} văn bản sang PDF/A lưu trữ (${t.pdfaPages} trang). Bản ghép sinh ra ở nút "PDF 2 lớp" màn Máy chủ; OCR xong chưa có nghĩa là đã có tệp PDF/A.`}
        >
          <span style={{ width: `${V1ql}%` }} />
        </div>
        <span className="oh-sub">{V1ql}%</span>
      </div>
      {t.pdfaCu > 0 && (
        <div
          className="oh-sub text-warning-700"
          title="Bản ghép có từ bản cũ, thiếu dấu hiệu PDF/A-2b (XMP pdfaid + OutputIntent + /ID + font nhúng) — bấm “Ghép lại PDF/A” để đạt Điều 8.2.a"
        >
          {fmtN(t.pdfaCu)} tệp chưa đạt PDF/A-2b
        </div>
      )}
      {t.unknownOcr > 0 && (
        <div
          className="muted oh-sub"
          title="Máy chủ đang chạy bản cũ, chưa trả số trang đã bóc tách — hai con số trên chỉ tính phần máy chủ có báo"
        >
          {fmtN(t.unknownOcr)} tài liệu máy chủ chưa báo
        </div>
      )}
    </>
  );
}

function V4l3({
  decision: V1qm,
  count: V1qn,
  busy: V1qo,
  onCancel: V1qp,
  onConfirm: V1qq,
}: {
  decision: V4eh;
  count: number;
  busy: boolean;
  onCancel: () => void;
  onConfirm: (V1qr: string) => void;
}) {
  const [V1qs, V1qt] = useState("");
  const V1qu = V4l0[V1qm].hoi;
  const V1qv = V1qm === "rejected" && V1qs.trim() === "";
  return (
    <div className="overlay" onClick={V1qo ? undefined : V1qp}>
      <div className="dialog" onClick={(V1qw) => V1qw.stopPropagation()}>
        <h3>
          {V1qu} {fmtN(V1qn)} tài liệu?
        </h3>
        <p className="muted text-sm mt-0">
          {V1qm === "approved" &&
            "Toàn bộ tài liệu đã chọn sẽ được đánh dấu đã duyệt (ghi vết ai duyệt, lúc nào)."}
          {V1qm === "rejected" &&
            "Toàn bộ tài liệu đã chọn bị trả lại để quét/bóc lại — nêu rõ lý do để người quét biết phải sửa gì."}
          {V1qm === "pending" &&
            "Xoá quyết định duyệt đã ghi của toàn bộ tài liệu đã chọn để soát lại từ đầu."}
        </p>
        {V1qm !== "pending" && (
          <textarea
            className="inp w-full resize-y"
            rows={3}
            autoFocus
            placeholder={
              V1qm === "rejected"
                ? "Ví dụ: ảnh mờ, bóc sai số quyết định…"
                : "Ghi chú (không bắt buộc)"
            }
            value={V1qs}
            onChange={(V1qx) => V1qt(V1qx.target.value)}
          />
        )}
        <div className="actions">
          <button className="btn" onClick={V1qp} disabled={V1qo}>
            Huỷ
          </button>
          <button
            className="btn primary"
            onClick={() => V1qq(V1qs)}
            disabled={V1qo || V1qv}
            title={V1qv ? "Phải nêu lý do từ chối" : undefined}
          >
            {V1qo ? "Đang ghi…" : V1qu}
          </button>
        </div>
      </div>
    </div>
  );
}

function V4l4(f: V4k8): ReactNode {
  switch (bocTachState(f)) {
    case "unknown":
      return "máy chủ chưa báo";
    case "none":
      return "chưa bóc tách";
    case "fields":
      return (
        <>
          <Icon name="duyet" size={13} /> đã bóc 16 trường
        </>
      );
    case "partial":
      return (
        <>
          <Icon name="dangCho" size={13} /> {fmtN(f.ocrPages)}/{fmtN(f.pages ?? 0)} trang
        </>
      );
    default:
      return (
        <>
          <Icon name="duyet" size={13} /> {fmtN(f.ocrPages)} trang
          {f.fieldPages > 0 ? " · 16 trường" : ""}
        </>
      );
  }
}

function V4l5(f: V4k8): ReactNode {
  switch (pdfaState(f)) {
    case "none":
      return <span className="muted">chưa chuyển PDF/A</span>;
    case "cu":
      return (
        <span className="text-warning-700">
          <Icon name="canhBao" size={13} /> PDF/A chưa đạt 2b
        </span>
      );
    case "chuaRo":
      return (
        <span className="muted">
          <Icon name="kiemTraPdfa" size={13} /> PDF/A {fmtN(f.pdfaPages)} trang (chưa rõ 2b)
        </span>
      );
    default:
      return (
        <span className="muted">
          <Icon name="kiemTraPdfa" size={13} /> PDF/A-2b · {fmtN(f.pdfaPages)} trang
        </span>
      );
  }
}

function V4l6({
  row,
  selected: V1qy,
  busy: V1qz,
  canReview: V1r0,
  onToggle: V1r1,
  onOpen: V1r2,
}: {
  row: V4kb;
  selected: ReadonlySet<string>;
  busy: boolean;
  canReview: boolean;
  onToggle: (V1r3: string[], V1r4: boolean) => void;
  onOpen: (f: V4k8) => void;
}) {
  return (
    <>
      {row.files.map((f) => {
        const V1r5 = V4hl(f.review);
        return (
          <tr key={f.id} className="oh-filerow">
            <td className="oh-check">
              {V1r0 && (
                <V4l1
                  state={V1qy.has(f.id) ? "all" : "none"}
                  disabled={V1qz}
                  title={
                    V1qy.has(f.id) ? `Bỏ chọn ${f.filename}` : `Chọn ${f.filename}`
                  }
                  onChange={(V1r6) => V1r1([f.id], V1r6)}
                />
              )}
            </td>
            {}
            <td colSpan={3} />
            <td className="oh-fname oh-files">
              {f.filename} <span className={`badge ${V1r5.tone}`}>{V1r5.label}</span>
            </td>
            <td>
              {f.pages == null ? "chưa rõ số trang" : `${fmtN(f.pages)} trang`} ·{" "}
              {V4kz(f.sizeBytes)} MB
            </td>
            <td>
              <div className="oh-bar">
                <span
                  className="oh-sub muted"
                  title={`${f.ocrPages} trang có kết quả OCR (①) · ${f.fieldPages} trang đã bóc trường (②) · pha vòng đời: ${f.status}${
                    f.pages == null ? " · máy chủ chưa ghi số trang" : ""
                  }`}
                >
                  {V4l4(f)}
                </span>
                <div className="flex-1" />
                <button
                  className="btn sm primary"
                  onClick={() => V1r2(f)}
                  title="Bóc tách trường & gán nhãn OCR cho tài liệu này"
                >
                  <Icon name="tim" />
                  Bóc tách trường
                </button>
              </div>
              {}
              <div className="oh-sub">{V4l5(f)}</div>
            </td>
          </tr>
        );
      })}
    </>
  );
}

export default function ReviewHoSoTable({
  canReview: V1r7,
  actions: V1r8,
  onlyProject: V1r9,
  onOpenDoc: V1ra,
  onDone: V1rb,
}: {
  canReview: boolean;

  actions?: ReactNode;

  onlyProject?: string;
  onOpenDoc: (f: V4k8) => void;

  onDone?: () => void;
}) {
  const [docs, V1rc] = useState<V4ej[]>([]);
  const [V1rd, V1re] = useState(true);
  const [V1rf, V1rg] = useState<string | null>(null);
  const [V1rh, V1ri] = useState<string | null>(null);
  const [V1rj, V1rk] = useState<ReadonlySet<string>>(new Set<string>());
  const [V1rl, V1rm] = useState<ReadonlySet<string>>(new Set<string>());
  const [V1rn, V1ro] = useState(false);
  const [q, V1rp] = useState("");
  const [V1rq, V1rr] = useState<V4eh | null>(null);

  const [V1rs, V1rt] = useState<{ done: number; total: number } | null>(null);

  const V1ru = useCallback(async () => {
    if (!isTauri()) {
      V1re(false);
      return;
    }
    V1re(true);
    try {
      V1rc(await V4ek());
      V1rg(null);
    } catch (V1rv) {
      V1rg(V4ky(V1rv));
      V1rc([]);
    } finally {
      V1re(false);
    }
  }, []);

  V41p(V1ru, { topics: ["docs", "ocr"] });

  const V1rw = useMemo(() => {
    const V1rx = V4kq(V4kw(docs, { chiChoXacNhan: V1rn, q }));

    return V1r9 ? V1rx.filter((g) => g.key === `p:${V1r9}`) : V1rx;
  }, [docs, V1rn, q, V1r9]);

  useEffect(() => {
    V1rk((cu) => {
      const moi = V4kt(cu, V1rw);
      return moi.size === cu.size ? cu : moi;
    });
  }, [V1rw]);

  const V1ry = V1rs !== null;
  const V1rz = useMemo(() => V4ku(V1rw, V1rj), [V1rw, V1rj]);
  const V1s0 = useMemo(() => V1rw.flatMap((g) => g.ids), [V1rw]);

  const V1s1 = useCallback((V1s2: string) => {
    V1rm((cu) => {
      const V1s3 = new Set(cu);
      if (!V1s3.delete(V1s2)) V1s3.add(V1s2);
      return V1s3;
    });
  }, []);

  const V1s4 = useCallback((V1s5: string[], V1s6: boolean) => {
    V1rk((cu) => V4ks(cu, V1s5, V1s6));
  }, []);

  const V1s7 = async (V1s8: V4eh, V1s9: string) => {
    const V1sa = V1rz;
    if (V1sa.length === 0) return;
    const V1sb = V4kv(V1sa, V4f7);
    const V1sc = V1s9.trim() === "" ? undefined : V1s9.trim();
    let V1sd = 0;
    let V1se = 0;
    V1rt({ done: 0, total: V1sa.length });
    V1rg(null);
    V1ri(null);
    try {
      for (const lo of V1sb) {
        const r = await V4f8(lo, V1s8, V1sc);
        V1sd += r.updated;
        V1se += r.skipped;
        V1rt({ done: V1sd + V1se, total: V1sa.length });
      }
      V1rr(null);
      V1rk(new Set<string>());
      const V1sf =
        V1se > 0
          ? ` ${fmtN(V1se)} tài liệu không đổi được (đã bị xoá hoặc ngoài phạm vi dự án của bạn).`
          : "";
      V1ri(`${V4l0[V1s8].xong} ${fmtN(V1sd)} tài liệu.${V1sf}`);
    } catch (V1sg) {
      V1rg(
        `${V4ky(V1sg)} — đã ghi được ${fmtN(V1sd)}/${fmtN(V1sa.length)} tài liệu, phần còn lại giữ nguyên.`,
      );
      V1rr(null);
    } finally {
      V1rt(null);
      await V1ru();
      V1rb?.();
    }
  };

  return (
    <div className="card p-0 overflow-hidden">
      <div className="flex items-center gap-3 py-3 px-4 flex-wrap">
        <h3 className="m-0">Tài liệu trên máy chủ</h3>
        <input
          className="inp sm max-w-60"
          placeholder="Tìm theo tên file / hộp / hồ sơ…"
          value={q}
          onChange={(V1sh) => V1rp(V1sh.target.value)}
        />
        <label className="flex items-center gap-1.5 text-sm">
          <input
            type="checkbox"
            checked={V1rn}
            disabled={V1ry}
            onChange={(V1si) => V1ro(V1si.target.checked)}
          />
          Chỉ hiện phần chờ xác nhận
        </label>
        <div className="flex-1" />
        <span className="muted text-sm">
          {V1rd && docs.length === 0
            ? "Đang tải…"
            : `${fmtN(V1rw.reduce((n, g) => n + g.rows.length, 0))} hồ sơ · ${fmtN(V1s0.length)} tài liệu`}
        </span>
        {V1r8}
        <button className="btn sm" onClick={() => void V1ru()} disabled={V1rd || V1ry}>
          <Icon name="lamMoi" />
          Tải lại
        </button>
      </div>

      {V1rf && <p className="oh-note text-danger-700">{V1rf}</p>}
      {V1rh && <p className="oh-note text-success-700">{V1rh}</p>}

      {V1rd && docs.length === 0 ? (
        <p className="muted pt-0 px-4 pb-4">Đang tải danh sách tài liệu trên máy chủ…</p>
      ) : V1rw.length === 0 ? (
        <p className="muted pt-0 px-4 pb-4">
          {docs.length === 0
            ? "Chưa có tài liệu nào trên máy chủ. Đưa dữ liệu lên qua “Đồng bộ dữ liệu” trước."
            : V1r9
              ? "Dự án đang chọn chưa có hồ sơ nào khớp — bấm “Bỏ lọc dự án” ở bảng trên để xem tất cả."
              : V1rn
                ? "Không còn tài liệu nào chờ xác nhận — bỏ tick “Chỉ hiện phần chờ xác nhận” để xem lại toàn bộ."
                : "Không có hồ sơ nào khớp từ khoá tìm."}
        </p>
      ) : (
        <div className="tbl-scroll oh-wrap">
          <table className="tbl oh-table">
            <thead>
              <tr>
                <th className="oh-check">
                  {V1r7 && (
                    <V4l1
                      state={V4kr(V1s0, V1rj)}
                      disabled={V1ry || V1s0.length === 0}
                      title="Chọn/bỏ chọn toàn bộ tài liệu đang hiện"
                      onChange={(V1sj) => V1s4(V1s0, V1sj)}
                    />
                  )}
                </th>
                <th>Dự án</th>
                <th>Hộp số</th>
                <th>Hồ sơ số</th>
                <th className="oh-files">Tổng số file</th>
                <th>Tài liệu</th>
                <th title="Trang đã có kết quả OCR trên tổng số trang, và số văn bản đã chuyển đổi sang PDF/A lưu trữ trên tổng số file">
                  Bóc tách OCR/PDF A
                </th>
              </tr>
            </thead>
            <tbody>
              {V1rw.map((g: V4kc) => (
                <Fragment key={g.key}>
                  <tr className="oh-grouprow">
                    <td className="oh-check">
                      {V1r7 && (
                        <V4l1
                          state={V4kr(g.ids, V1rj)}
                          disabled={V1ry || g.ids.length === 0}
                          title={`Chọn cả dự án ${g.label} (${fmtN(g.ids.length)} tài liệu)`}
                          onChange={(V1sk) => V1s4(g.ids, V1sk)}
                        />
                      )}
                    </td>
                    <th className="oh-groupname" colSpan={3}>
                      <Icon name="thuMuc" /> Dự án: {g.label}
                    </th>
                    <td className="oh-num oh-files">{fmtN(g.tally.files)}</td>
                    <td>
                      <V4l2 t={g.tally} />
                    </td>
                    <td>
                      <OcrCell t={g.tally} />
                    </td>
                  </tr>
                  {g.rows.map((r) => {
                    const V1sl = V1rl.has(r.key);
                    return (
                      <Fragment key={r.key}>
                        <tr>
                          <td className="oh-check">
                            {V1r7 && (
                              <V4l1
                                state={V4kr(r.ids, V1rj)}
                                disabled={V1ry || r.ids.length === 0}
                                title={`Chọn cả hồ sơ ${r.hoSoSo || "—"} (${fmtN(r.ids.length)} tài liệu)`}
                                onChange={(V1sm) => V1s4(r.ids, V1sm)}
                              />
                            )}
                          </td>
                          {}
                          <th className="oh-duan kg-hoso">{g.label}</th>
                          <th className="kg-hoso">{r.hopSo || V4ke}</th>
                          <th className="kg-hoso">
                            {r.hoSoSo || V4kf}
                            {r.counts.rejected > 0 && (
                              <div className="muted oh-sub text-danger-700">
                                {fmtN(r.counts.rejected)} tài liệu bị từ chối
                              </div>
                            )}
                          </th>
                          <td className="oh-num oh-files">
                            <button
                              className="oh-expand"
                              onClick={() => V1s1(r.key)}
                              title={
                                V1sl
                                  ? "Thu gọn danh sách file"
                                  : "Xem từng file trong hồ sơ này"
                              }
                            >
                              <span className="oh-caret">
                                <Icon name={V1sl ? "xuong" : "moNhanh"} size={13} />
                              </span>
                              {fmtN(r.tally.files)}
                            </button>
                          </td>
                          <td>
                            <V4l2 t={r.tally} />
                          </td>
                          <td>
                            <OcrCell t={r.tally} />
                          </td>
                        </tr>
                        {V1sl && (
                          <V4l6
                            row={r}
                            selected={V1rj}
                            busy={V1ry}
                            canReview={V1r7}
                            onToggle={V1s4}
                            onOpen={V1ra}
                          />
                        )}
                      </Fragment>
                    );
                  })}
                </Fragment>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {V1r7 && (
        <div className="flex flex-wrap items-center gap-2.5 border-t border-border-subtle px-4 py-2.5">
          <span className="text-sm">
            {V1rs
              ? `Đang ghi ${fmtN(V1rs.done)}/${fmtN(V1rs.total)} tài liệu…`
              : `Đã chọn ${fmtN(V1rz.length)} tài liệu`}
          </span>
          {V1rz.length > 0 && !V1ry && (
            <button className="btn sm" onClick={() => V1rk(new Set<string>())}>
              Bỏ chọn
            </button>
          )}
          <div className="flex-1" />
          {(["approved", "rejected", "pending"] as const).map((d) => (
            <button
              key={d}
              className={d === "approved" ? "btn primary" : "btn"}
              disabled={V1ry || V1rz.length === 0}
              onClick={() => V1rr(d)}
              title={
                V1rz.length === 0
                  ? "Tích chọn dự án / hồ sơ / tài liệu trước"
                  : `${V4l0[d].hoi} ${fmtN(V1rz.length)} tài liệu đã chọn`
              }
            >
              <Icon name={V4l0[d].icon} />
              {V4l0[d].nut}
            </button>
          ))}
        </div>
      )}

      {V1rq && (
        <V4l3
          decision={V1rq}
          count={V1rz.length}
          busy={V1ry}
          onCancel={() => V1rr(null)}
          onConfirm={(V1sn) => void V1s7(V1rq, V1sn)}
        />
      )}
    </div>
  );
}
