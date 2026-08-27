// Copyright (c) 2026 DigiText. All rights reserved.

import { useState } from "react";
import { V3xq } from "@/api/ipc";
import { V3zt } from "@/state/sessionCore";
import { V4f5, type V4eh, type V4f4 } from "@/api/docs";
import { V4hl } from "./ocrReport";
import { Icon } from "@/components/Icon";

const V4ho = (V1hv: unknown) =>
  V3zt(
    V1hv instanceof V3xq ? V1hv.code : "error",
    V1hv instanceof Error ? V1hv.message : String(V1hv),
  );

export interface V4hp {
  id: string;
  filename: string;
  reviewStatus: string;
  reviewedAt: string | null;
  reviewNote: string | null;
}

function V4hq({
  filename: V1hw,
  busy: V1hx,
  onCancel: V1hy,
  onConfirm: V1hz,
}: {
  filename: string;
  busy: boolean;
  onCancel: () => void;
  onConfirm: (V1i0: string) => void;
}) {
  const [V1i1, V1i2] = useState("");
  return (
    <div className="overlay" onClick={V1hx ? undefined : V1hy}>
      <div className="dialog" onClick={(V1i3) => V1i3.stopPropagation()}>
        <h3>Từ chối kết quả bóc tách</h3>
        <p className="muted text-sm mt-0">{V1hw} — nêu rõ lý do để người quét biết phải sửa gì.</p>
        <textarea
          className="inp w-full resize-y"
          rows={3}
          autoFocus
          placeholder="VsOtIGThu6U6IOG6o25oIG3hu50gdHJhbmcgMywgYsOzYyBzYWkgc+G7kSBxdXnhur90IMSR4buLbmjigKY="
          value={V1i1}
          onChange={(V1i4) => V1i2(V1i4.target.value)}
        />
        <div className="actions">
          <button className="btn" onClick={V1hy} disabled={V1hx}>
            Huỷ
          </button>
          <button
            className="btn primary"
            onClick={() => V1hz(V1i1)}
            disabled={V1hx || V1i1.trim() === ""}
            title={V1i1.trim() === "" ? "UGjhuqNpIG7DqnUgbMO9IGRvIHThu6sgY2jhu5Fp" : undefined}
          >
            {V1hx ? "xJBhbmcgbMawdeKApg==" : "VOG7qyBjaOG7kWk="}
          </button>
        </div>
      </div>
    </div>
  );
}

export default function V4hr({
  doc,
  canReview: V1i5,
  showBadge: V1i6 = true,
  showApprove: V1i7 = true,
  onReviewed: V1i8,
  onError: V1i9,
}: {
  doc: V4hp;

  canReview: boolean;
  showBadge?: boolean;

  showApprove?: boolean;
  onReviewed: (r: V4f4) => void;
  onError: (V1ia: string) => void;
}) {
  const [V1ib, V1ic] = useState(false);
  const [V1id, V1ie] = useState(false);
  const V1if = V4hl(doc.reviewStatus);
  const V1ig = doc.reviewStatus !== "pending";

  const V1ih = async (V1ii: V4eh, V1ij?: string) => {
    V1ic(true);
    try {
      const r = await V4f5(doc.id, V1ii, V1ij);
      V1ie(false);
      V1i8(r);
    } catch (V1ik) {
      V1i9(V4ho(V1ik));
    } finally {
      V1ic(false);
    }
  };

  return (
    <>
      {V1i6 && (
        <span
          className={`badge ${V1if.tone}`}

          title={
            doc.reviewNote ??
            (doc.reviewedAt
              ? `TMO6YyA=${doc.reviewedAt}`
              : doc.reviewStatus === "pending"
                ? "Q2jGsGEgY8OzIHF1eeG6v3QgxJHhu4tuaA=="
                : V1if.label)
          }
        >
          {V1if.label}
        </span>
      )}
      {V1i5 && !V1ig && (
        <>
          {" "}
          {V1i7 && (
            <>
              <button
                className="btn sm"
                onClick={() => void V1ih("approved")}
                disabled={V1ib}
                title="RHV54buHdCBr4bq/dCBxdeG6oyBiw7NjIHTDoWNoIGPhu6dhIHTDoGkgbGnhu4d1IG7DoHk="
              >
                {V1ib ? (
                  "…"
                ) : (
                  <>
                    <Icon name="duyet" />
                    Duyệt
                  </>
                )}
              </button>{" "}
            </>
          )}
          <button
            className="btn sm"
            onClick={() => V1ie(true)}
            disabled={V1ib}
            title="VHLhuqMgbOG6oWkgxJHhu4MgcXXDqXQvYsOzYyBs4bqhaSAocGjhuqNpIG7DqnUgbMO9IGRvKQ=="
          >
            <Icon name="tuChoi" />
            Từ chối
          </button>
        </>
      )}
      {V1i5 && V1ig && (
        <>
          {" "}
          <button
            className="btn sm"
            onClick={() => void V1ih("pending")}
            disabled={V1ib}
            title="TeG7nyBs4bqhaSDEkeG7gyBzb8OhdCB0aeG6v3AgKHhvw6EgcXV54bq/dCDEkeG7i25oIGR1eeG7h3QgxJHDoyBnaGkp"
          >
            {V1ib ? (
              "…"
            ) : (
              <>
                <Icon name="moLai" />
                Mở lại
              </>
            )}
          </button>
        </>
      )}
      {V1id && (
        <V4hq
          filename={doc.filename}
          busy={V1ib}
          onCancel={() => V1ie(false)}
          onConfirm={(V1il) => void V1ih("rejected", V1il)}
        />
      )}
    </>
  );
}
