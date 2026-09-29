// Copyright (c) 2026 DigiText. All rights reserved.

import { useCallback, useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { V41p } from "@/live/useLive";
import { V3xq } from "@/api/ipc";
import { V460 } from "@/state";
import { V3z9 } from "@/domain/permissions";
import { V3zt } from "@/state/sessionCore";
import { V4en, V4f5, V4ep, type V4eh } from "@/api/docs";
import V4hr, { type V4hp } from "../dossier/ReviewControls";
import { V40p } from "@/api/archive";
import { V4fh, V4fg, type V4fe } from "@/api/ocr";
import { V4hz, V4hv, V4hw, V4hu, V4i0, type V4hy } from "./pageChunks";
import { V4gv, V4gt, type V4gk } from "../dossier/ocrBoxes";
import V4h3 from "../dossier/OcrBoxLayer";
import { V4gb, V4fq, V4fo, V4fw, V4g8, V4fz, V4g7, V4fn, V4fv, V4fy, type V4fs } from "./ocrLogic";
import { Icon } from "@/components/Icon";

const V4i1 = (V1iv: unknown) =>
  V3zt(
    V1iv instanceof V3xq ? V1iv.code : "error",
    V1iv instanceof Error ? V1iv.message : String(V1iv),
  );

const V4i2 = (V1iw: number, V1ix: string) => `${V1iw}|${V1ix}`;

const V4i3 = "(chưa có — nhập tay nếu văn bản có)";

const V4i4 = (m: Record<string, unknown>) => Object.keys(m).length > 0;

function V4i5({
  value: V1iy,
  onChange: V1iz,
  placeholder,
}: {
  value: string;
  onChange: (v: string) => void;
  placeholder?: string;
}) {
  const V1j0 = useRef<HTMLTextAreaElement>(null);
  const [scroll, V1j1] = useState(false);

  useEffect(() => {
    const V1j2 = V1j0.current;
    if (!V1j2) return;
    V1j2.style.height = "auto";
    const g = V4gb(V1j2.scrollHeight);
    V1j2.style.height = `${g.height}px`;
    V1j1(g.scroll);
  }, [V1iy]);

  return (
    <textarea
      ref={V1j0}
      className="inp"
      rows={1}
      placeholder={placeholder}
      value={V1iy}
      onChange={(V1j3) => V1iz(V1j3.target.value)}
      style={{
        width: "100%",
        boxSizing: "border-box",

        resize: "none",
        overflowY: scroll ? "auto" : "hidden",
      }}
    />
  );
}

export interface V4i6 {
  id: string;
  filename: string;

  sizeBytes?: number;
  reviewStatus: V4eh | string;
  reviewedAt?: string | null;
  reviewNote?: string | null;
}

/** `workMode`: CTV làm mục trong gói việc — chỉ Lưu, không có nút duyệt / từ chối tài liệu;
 *  `footer`: các nút thêm ở thanh dưới (Trước / Tiếp / Xác nhận hoàn thành…). */
export default function V4i7({
  doc,
  onBack: V1j4,
  workMode = false,
  footer,
}: {
  doc: V4i6;
  onBack: () => void;
  workMode?: boolean;
  footer?: ReactNode;
}) {
  const { session: V1j5 } = V460();
  const V1j6 = !workMode && V3z9(V1j5?.permissions, "document.review");

  const [review, V1j7] = useState<V4hp>({
    id: doc.id,
    filename: doc.filename,
    reviewStatus: doc.reviewStatus,
    reviewedAt: doc.reviewedAt ?? null,
    reviewNote: doc.reviewNote ?? null,
  });
  const [V1j8, V1j9] = useState<string[]>([]);

  const [V1ja, V1jb] = useState<string | null>(null);
  const [V1jc, V1jd] = useState(false);
  const [V1je, V1jf] = useState(false);
  const [V1jg, V1jh] = useState<V4hy | null>(null);
  const [V1ji, V1jj] = useState<string | null>(null);

  const V1jk = useRef<HTMLDivElement>(null);

  const V1jl = useRef(false);
  const [V1jm, V1jn] = useState<Record<number, { w: number; h: number }>>({});
  const [V1jo, V1jp] = useState<V4fe[]>([]);
  const [V1jq, V1jr] = useState<Record<string, string>>({});
  const [V1js, V1jt] = useState<Record<string, number[]>>({});
  const [V1ju, V1jv] = useState<string | null>(null);
  const [V1jw, V1jx] = useState(true);

  const [V1jy, V1jz] = useState(false);
  const [V1k0, V1k1] = useState<string | null>(null);
  const [V1k2, V1k3] = useState<string | null>(null);

  const V1k4 = useCallback(async () => {
    V1jp(await V4fg(doc.id));
  }, [doc.id]);

  V41p(V1k4, { topics: ["ocrFields"], skipInitial: true });

  useEffect(() => {
    let V1k5: (() => void) | null = null;
    let V1k6 = false;
    void V4ep((V1k7) => {
      if (V1k7.id === doc.id) V1jh({ received: V1k7.received, total: V1k7.total });
    }).then((V1k8) => {
      if (V1k6) V1k8();
      else V1k5 = V1k8;
    });
    return () => {
      V1k6 = true;
      V1k5?.();
    };
  }, [doc.id]);

  useEffect(() => {
    let V1k9 = true;
    V1jx(true);
    V1k1(null);
    V1k3(null);
    V1jr({});
    V1jt({});
    V1j9([]);

    V1jn({});
    V1jb(null);
    V1jd(false);
    V1jj(null);
    V1jh(null);
    V1jl.current = false;
    Promise.all([
      V1k4().catch((V1ka) => V1k9 && V1k1(V4i1(V1ka))),

      V4en(doc.id, doc.sizeBytes)
        .then(async (r) => {
          const { from, count: V1kb } = V4hu(0);
          const p = await V40p(r.path, from, V1kb);
          if (!V1k9) return;
          V1jb(r.path);
          V1j9(p);
          V1jd(V4hv(p.length, V1kb));
        })
        .catch((V1kc) => {
          if (!V1k9) return;
          V1j9([]);

          V1jj(V4i1(V1kc));
        }),
    ]).finally(() => {
      if (V1k9) V1jx(false);
    });
    return () => {
      V1k9 = false;
    };
  }, [doc.id, doc.sizeBytes, V1k4]);

  const V1kd = useCallback(
    async (V1ke: number) => {
      if (!V1ja || !V1jc || V1jl.current) return;
      V1jl.current = true;
      V1jf(true);
      try {
        let V1kf = V1j8.length;
        let V1kg = true;
        const V1kh: string[] = [];
        while (V1kg && V1kf < V1ke) {
          const { from, count: V1ki } = V4hu(V1kf);
          const p = await V40p(V1ja, from, V1ki);
          V1kh.push(...p);
          V1kf += p.length;
          V1kg = V4hv(p.length, V1ki);
        }
        if (V1kh.length > 0) V1j9((V1kj) => [...V1kj, ...V1kh]);
        V1jd(V1kg);
      } catch (V1kk) {
        V1jj(V4i1(V1kk));

        V1jd(false);
      } finally {
        V1jl.current = false;
        V1jf(false);
      }
    },
    [V1ja, V1jc, V1j8.length],
  );

  const V1kl = useCallback(() => V1kd(V1j8.length + 1), [V1kd, V1j8.length]);

  const fields = useMemo(() => V4fy(V4fv(V1jo)), [V1jo]);
  const V1km = useMemo(() => V4fz(fields), [fields]);

  const V1kn = useMemo(
    () => V4g8(fields, { drafts: V1jq, bboxDrafts: V1js }),
    [fields, V1jq, V1js],
  );

  const V1ko = (f: V4fs) => V1jq[V4i2(f.pageNo, f.fieldKey)] ?? f.value;
  const V1kp = (f: V4fs) => V1js[V4i2(f.pageNo, f.fieldKey)] ?? f.bbox;

  const V1kq = (f: V4fs) => V4g7(f, { drafts: V1jq, bboxDrafts: V1js });

  const V1kr = (V1ks: V4gk) => {
    V1jt((m) => ({ ...m, [V1ks.key]: V4gv(V1ks) }));
  };

  const V1kt = async () => {
    V1jz(true);
    V1k1(null);
    V1k3(null);
    try {
      for (const s of V1kn) {
        await V4fh({
          documentId: doc.id,
          pageNo: s.pageNo,
          fieldKey: s.fieldKey,
          value: s.value,

          engineVersionId: s.engineVersionId || undefined,
          bbox: s.bbox ?? undefined,
          verified: true,
        });
      }
      await V1k4();
      V1jr({});
      V1jt({});

      const V1ku = V1kn.length === 0 && (V4i4(V1jq) || V4i4(V1js));
      const V1kv =
        V1kn.length > 0
          ? `Đã lưu ${V1kn.length} trường. `
          : V1ku
            ? "Không có thay đổi thực sự (chỉ khác khoảng trắng/cách gõ) nên không ghi gì. "
            : "";
      if (workMode) {
        V1k3(`${V1kv}Kiểm tra đủ rồi thì bấm “Xác nhận hoàn thành”.`);
      } else if (!V1j6) {
        V1k3(
          `${V1kv}Bạn không có quyền duyệt tài liệu nên trạng thái duyệt giữ nguyên.`,
        );
      } else if (review.reviewStatus === "approved") {
        V1k3(`${V1kv}Tài liệu đã ở trạng thái đã duyệt.`);
      } else {
        const r = await V4f5(doc.id, "approved");
        V1j7((v) => ({ ...v, ...r }));
        V1k3(`${V1kv}Đã duyệt tài liệu này.`);
      }
    } catch (V1kw) {
      V1k1(V4i1(V1kw));
    } finally {
      V1jz(false);
    }
  };

  return (
    <div className="card p-0 overflow-hidden">
      <div className="flex items-center gap-3 py-3 px-4 flex-wrap">
        <button className="btn" onClick={V1j4}>
          ← Về danh sách
        </button>
        <b className="overflow-hidden text-ellipsis whitespace-nowrap">{doc.filename}</b>
        {}
        {!workMode && (
        <span className="whitespace-nowrap">
          <V4hr
            doc={review}
            canReview={V1j6}

            showApprove={false}
            onReviewed={(r) => {
              V1j7((v) => ({ ...v, ...r }));
              V1k1(null);
              V1k3(
                r.reviewStatus === "approved"
                  ? "Đã duyệt tài liệu này."
                  : r.reviewStatus === "rejected"
                    ? "Đã từ chối tài liệu này."
                    : "Đã mở lại để soát tiếp.",
              );
            }}
            onError={V1k1}
          />
        </span>
        )}
        <div className="flex-1" />
        <button className="btn sm" onClick={() => void V1k4()}>
          <Icon name="lamMoi" />
          Tải lại
        </button>
      </div>

      {V1k0 && <p className="muted text-danger-700 py-0 px-4">{V1k0}</p>}
      {V1k2 && <p className="muted text-success-700 py-0 px-4">{V1k2}</p>}

      <div className="flex gap-3 p-3 items-start">
        {}
        <div
          ref={V1jk}
          onScroll={(V1kx) => {
            if (V1jc && !V1je && V4hw(V1kx.currentTarget)) void V1kl();
          }}
          className="grow shrink basis-[55%] max-h-[72vh] overflow-auto"
        >
          {V1jw ? (
            <div className="muted p-4">{V4hz(V1jg)}</div>
          ) : V1j8.length === 0 ? (
            <div className={`muted p-4${V1ji ? " text-danger-700" : ""}`}>
              {V1ji ??
                "Không hiển thị được nội dung (không phải PDF ảnh)."}
            </div>
          ) : (
            V1j8.map((src, i) => {
              const V1ky = i + 1;
              const size = V1jm[V1ky];
              const V1kz = size
                ? V4fw(fields, V1ky)
                    .map((f) =>
                      V4gt(V4i2(V1ky, f.fieldKey), V1kp(f), V1ko(f), f.confidence, f.fieldKey),
                    )
                    .filter((b): b is V4gk => b != null)
                : [];
              return (
                <div key={i} className="mb-2.5 border border-border">
                  <V4h3
                    src={src}
                    boxes={V1kz}
                    pageW={size?.w ?? 0}
                    pageH={size?.h ?? 0}
                    selectedKey={V1ju}
                    onSelect={V1jv}
                    onChange={V1kr}
                    boxLabel={(b) => V4fo(b.field ?? "")}

                    boxColorFn={(b) => V4fq(b.field ?? b.key)}
                    onNatural={(w, h) =>
                      V1jn((m) =>
                        m[V1ky]?.w === w && m[V1ky]?.h === h ? m : { ...m, [V1ky]: { w, h } },
                      )
                    }
                  />
                </div>
              );
            })
          )}

          {}
          {!V1jw && V1j8.length > 0 && (
            <div className="muted pt-1.5 px-1 pb-3 flex items-center gap-2">
              <span>{V4i0(V1j8.length, V1jc, V1je)}</span>
              {V1jc && !V1je && (
                <button className="btn sm" onClick={() => void V1kl()}>
                  <Icon name="xuong" />
                  Xem thêm trang
                </button>
              )}
            </div>
          )}
          {V1ji && V1j8.length > 0 && (
            <p className="muted text-danger-700 pt-0 px-1 pb-3">{V1ji}</p>
          )}
        </div>

        {}
        <div className="grow shrink basis-[45%] max-h-[72vh] overflow-auto">
          {}
          {!V1jw && (
            <div className="muted pt-0 px-0.5 pb-2">
              Đã có giá trị{" "}
              <b>
                {V1km}/{V4fn.length}
              </b>{" "}
              trường. Trường để trống là máy chưa bóc được — nhập tay rồi bấm <b>Duyệt</b> ở cuối
              màn để lưu tất cả.
            </div>
          )}
          {V1jw ? (
            <div className="muted p-4">Đang tải trường bóc tách…</div>
          ) : (
            fields.map((f) => {
              const V1l0 = V4i2(f.pageNo, f.fieldKey);
              const V1l1 = f.fieldKey === "trichYeu";
              const V1l2 = V1l0 in V1js;
              const V1l3 = V1ju === V1l0;
              return (
                <div
                  key={V1l0}
                  className="card"
                  style={{
                    margin: "0 0 8px",
                    padding: 10,

                    borderLeft: `4px ${f.placeholder ? "dashed" : "solid"} ${V4fq(f.fieldKey)}`,
                    outline: V1l3 ? "2px solid var(--color-brand-500)" : "none",
                  }}
                  onClick={() => {
                    V1jv(V1l0);

                    if (f.pageNo > V1j8.length) void V1kd(f.pageNo);
                  }}
                >
                  <div className="flex items-center gap-2 mb-1">
                    <span
                      aria-hidden
                      title="Màu ô trên trang tương ứng"
                      style={{
                        width: 10,
                        height: 10,
                        borderRadius: 2,
                        background: V4fq(f.fieldKey),
                        flex: "0 0 auto",
                      }}
                    />
                    <b>{f.label}</b>
                    <span className="muted text-xs">trang {f.pageNo}</span>
                    {f.placeholder ? (
                      <span
                        className="badge gray"
                        title="Máy chưa bóc được trường này"
                      >
                        chưa có
                      </span>
                    ) : f.corrected ? (
                      <span className="badge ok">đã soát</span>
                    ) : (
                      <span className="badge gray">máy bóc</span>
                    )}
                    {V1l2 && (
                      <span className="badge gray" title="Khung đã chỉnh, chưa lưu">
                        khung đã sửa
                      </span>
                    )}
                    {}
                    {V1kq(f) && (
                      <span
                        className="badge warn"
                        title="Sửa đổi chưa lưu — bấm Duyệt ở cuối màn để ghi"
                      >
                        chưa lưu
                      </span>
                    )}
                    {f.confidence != null && !f.corrected && (
                      <span className="muted text-xs">{Math.round(f.confidence * 100)}%</span>
                    )}
                    <div className="flex-1" />
                    {f.verified && (
                      <span className="badge ok">
                        <Icon name="duyet" />
                        verified
                      </span>
                    )}
                  </div>
                  {V1l1 ? (
                    <V4i5
                      placeholder={f.placeholder ? V4i3 : undefined}
                      value={V1ko(f)}
                      onChange={(v) => V1jr((d) => ({ ...d, [V1l0]: v }))}
                    />
                  ) : (
                    <input
                      className="inp w-full box-border"
                      placeholder={f.placeholder ? V4i3 : undefined}
                      value={V1ko(f)}
                      onChange={(V1l4) => V1jr((d) => ({ ...d, [V1l0]: V1l4.target.value }))}
                    />
                  )}
                </div>
              );
            })
          )}
        </div>
      </div>

      {}
      <div className="sticky bottom-0 flex items-center gap-2.5 border-t border-border bg-surface px-4 py-2.5">
        <span className="muted">
          {V1kn.length > 0
            ? `${V1kn.length} thay đổi chưa lưu.`
            : "Không có thay đổi nào chưa lưu."}
        </span>
        <div className="flex-1" />
        {footer}
        <button className="btn" onClick={V1j4} disabled={V1jy}>
          Đóng
        </button>
        <button
          className="btn primary"
          onClick={() => void V1kt()}
          disabled={V1jy || V1jw}
          title={
            V1j6
              ? "Lưu các sửa đổi rồi đánh dấu tài liệu đã duyệt"
              : "Lưu các sửa đổi (bạn không có quyền duyệt tài liệu)"
          }
        >
          {V1jy ? (
            "Đang lưu…"
          ) : V1j6 ? (
            <>
              <Icon name="duyet" />
              Duyệt
            </>
          ) : (
            <>
              <Icon name="luu" />
              Lưu
            </>
          )}
        </button>
      </div>
    </div>
  );
}
