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
    nut: "RHV54buHdCDEkcOjIGNo4buNbg==",
    hoi: "RHV54buHdA==",
    xong: "xJDDoyBkdXnhu4d0",
  },
  rejected: {
    icon: "tuChoi",
    nut: "VOG7qyBjaOG7kWkgxJHDoyBjaOG7jW4=",
    hoi: "VOG7qyBjaOG7kWk=",
    xong: "xJDDoyB04burIGNo4buRaQ==",
  },
  pending: {
    icon: "moLai",
    nut: "TeG7nyBs4bqhaSDEkcOjIGNo4buNbg==",
    hoi: "TeG7nyBs4bqhaQ==",
    xong: "xJDDoyBt4bufIGzhuqFp",
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
          title="TcOheSBjaOG7pyBjaMawYSBnaGkgc+G7kSB0cmFuZyBjaG8gY8OhYyB0w6BpIGxp4buHdSBuw6B5"
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
          title={`${t.ocrPages}/${t.ocrDenom}IHRyYW5nIMSRw6MgY8OzIGvhur90IHF14bqjIE9DUiAobsO6dCDikaAgUERGIDIgbOG7m3Ap${
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
          title="VMOgaSBsaeG7h3UgxJHDoyBiw7NjIMSRxrDhu6NjIDE2IHRyxrDhu51uZyBt4bulYyBs4bulYyAobsO6dCDikaEgQsOzYyB0w6FjaCAxNiB0csaw4budbmcpIOKAlCB2aeG7h2MgbsOgeSBraMO0bmcgZ2hpIGvhur90IHF14bqjIE9DUiB04burbmcgdHJhbmc="
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
          title={`xJDDoyBjaHV54buDbiDEkeG7lWkg${t.pdfaDocs}/${t.files}IHbEg24gYuG6o24gc2FuZyBQREYvQSBsxrB1IHRy4buvICg=${t.pdfaPages}IHRyYW5nKS4gQuG6o24gZ2jDqXAgc2luaCByYSDhu58gbsO6dCAiUERGIDIgbOG7m3AiIG3DoG4gTcOheSBjaOG7pzsgT0NSIHhvbmcgY2jGsGEgY8OzIG5naMSpYSBsw6AgxJHDoyBjw7MgdOG7h3AgUERGL0Eu`}
        >
          <span style={{ width: `${V1ql}%` }} />
        </div>
        <span className="oh-sub">{V1ql}%</span>
      </div>
      {t.pdfaCu > 0 && (
        <div
          className="oh-sub text-warning-700"
          title="QuG6o24gZ2jDqXAgY8OzIHThu6sgYuG6o24gY8WpLCB0aGnhur91IGThuqV1IGhp4buHdSBQREYvQS0yYiAoWE1QIHBkZmFpZCArIE91dHB1dEludGVudCArIC9JRCArIGZvbnQgbmjDum5nKSDigJQgYuG6pW0g4oCcR2jDqXAgbOG6oWkgUERGL0HigJ0gxJHhu4MgxJHhuqF0IMSQaeG7gXUgOC4yLmE="
        >
          {fmtN(t.pdfaCu)} tệp chưa đạt PDF/A-2b
        </div>
      )}
      {t.unknownOcr > 0 && (
        <div
          className="muted oh-sub"
          title="TcOheSBjaOG7pyDEkWFuZyBjaOG6oXkgYuG6o24gY8WpLCBjaMawYSB0cuG6oyBz4buRIHRyYW5nIMSRw6MgYsOzYyB0w6FjaCDigJQgaGFpIGNvbiBz4buRIHRyw6puIGNo4buJIHTDrW5oIHBo4bqnbiBtw6F5IGNo4bunIGPDsyBiw6Fv"
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
            "VG/DoG4gYuG7mSB0w6BpIGxp4buHdSDEkcOjIGNo4buNbiBz4bq9IMSRxrDhu6NjIMSRw6FuaCBk4bqldSDEkcOjIGR1eeG7h3QgKGdoaSB24bq/dCBhaSBkdXnhu4d0LCBsw7pjIG7DoG8pLg=="}
          {V1qm === "rejected" &&
            "VG/DoG4gYuG7mSB0w6BpIGxp4buHdSDEkcOjIGNo4buNbiBi4buLIHRy4bqjIGzhuqFpIMSR4buDIHF1w6l0L2LDs2MgbOG6oWkg4oCUIG7DqnUgcsO1IGzDvSBkbyDEkeG7gyBuZ8aw4budaSBxdcOpdCBiaeG6v3QgcGjhuqNpIHPhu61hIGfDrC4="}
          {V1qm === "pending" &&
            "WG/DoSBxdXnhur90IMSR4buLbmggZHV54buHdCDEkcOjIGdoaSBj4bunYSB0b8OgbiBi4buZIHTDoGkgbGnhu4d1IMSRw6MgY2jhu41uIMSR4buDIHNvw6F0IGzhuqFpIHThu6sgxJHhuqd1Lg=="}
        </p>
        {V1qm !== "pending" && (
          <textarea
            className="inp w-full resize-y"
            rows={3}
            autoFocus
            placeholder={
              V1qm === "rejected"
                ? "VsOtIGThu6U6IOG6o25oIG3hu50sIGLDs2Mgc2FpIHPhu5EgcXV54bq/dCDEkeG7i25o4oCm"
                : "R2hpIGNow7ogKGtow7RuZyBi4bqvdCBideG7mWMp"
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
            title={V1qv ? "UGjhuqNpIG7DqnUgbMO9IGRvIHThu6sgY2jhu5Fp" : undefined}
          >
            {V1qo ? "xJBhbmcgZ2hp4oCm" : V1qu}
          </button>
        </div>
      </div>
    </div>
  );
}

function V4l4(f: V4k8): ReactNode {
  switch (bocTachState(f)) {
    case "unknown":
      return "bcOheSBjaOG7pyBjaMawYSBiw6Fv";
    case "none":
      return "Y2jGsGEgYsOzYyB0w6FjaA==";
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
          {f.fieldPages > 0 ? "IMK3IDE2IHRyxrDhu51uZw==" : ""}
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
                    V1qy.has(f.id) ? `QuG7jyBjaOG7jW4g${f.filename}` : `Q2jhu41uIA==${f.filename}`
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
              {f.pages == null ? "Y2jGsGEgcsO1IHPhu5EgdHJhbmc=" : `${fmtN(f.pages)} trang`} ·{" "}
              {V4kz(f.sizeBytes)} MB
            </td>
            <td>
              <div className="oh-bar">
                <span
                  className="oh-sub muted"
                  title={`${f.ocrPages}IHRyYW5nIGPDsyBr4bq/dCBxdeG6oyBPQ1IgKOKRoCkgwrcg${f.fieldPages}IHRyYW5nIMSRw6MgYsOzYyB0csaw4budbmcgKOKRoSkgwrcgcGhhIHbDsm5nIMSR4budaTog${f.status}${
                    f.pages == null ? "IMK3IG3DoXkgY2jhu6cgY2jGsGEgZ2hpIHPhu5EgdHJhbmc=" : ""
                  }`}
                >
                  {V4l4(f)}
                </span>
                <div className="flex-1" />
                <button
                  className="btn sm primary"
                  onClick={() => V1r2(f)}
                  title="QsOzYyB0w6FjaCB0csaw4budbmcgJiBnw6FuIG5ow6NuIE9DUiBjaG8gdMOgaSBsaeG7h3UgbsOgeQ=="
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
          ? ` ${fmtN(V1se)}IHTDoGkgbGnhu4d1IGtow7RuZyDEkeG7lWkgxJHGsOG7o2MgKMSRw6MgYuG7iyB4b8OhIGhv4bq3YyBuZ2/DoGkgcGjhuqFtIHZpIGThu7Egw6FuIGPhu6dhIGLhuqFuKS4=`
          : "";
      V1ri(`${V4l0[V1s8].xong} ${fmtN(V1sd)}IHTDoGkgbGnhu4d1Lg==${V1sf}`);
    } catch (V1sg) {
      V1rg(
        `${V4ky(V1sg)}IOKAlCDEkcOjIGdoaSDEkcaw4bujYyA=${fmtN(V1sd)}/${fmtN(V1sa.length)}IHTDoGkgbGnhu4d1LCBwaOG6p24gY8OybiBs4bqhaSBnaeG7ryBuZ3V5w6puLg==`,
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
          placeholder="VMOsbSB0aGVvIHTDqm4gZmlsZSAvIGjhu5lwIC8gaOG7kyBzxqHigKY="
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
            ? "xJBhbmcgdOG6o2nigKY="
            : `${fmtN(V1rw.reduce((n, g) => n + g.rows.length, 0))}IGjhu5Mgc8ahIMK3IA==${fmtN(V1s0.length)}IHTDoGkgbGnhu4d1`}
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
            ? "Q2jGsGEgY8OzIHTDoGkgbGnhu4d1IG7DoG8gdHLDqm4gbcOheSBjaOG7py4gxJDGsGEgZOG7ryBsaeG7h3UgbMOqbiBxdWEg4oCcxJDhu5NuZyBi4buZIGThu68gbGnhu4d14oCdIHRyxrDhu5tjLg=="
            : V1r9
              ? "ROG7sSDDoW4gxJFhbmcgY2jhu41uIGNoxrBhIGPDsyBo4buTIHPGoSBuw6BvIGto4bubcCDigJQgYuG6pW0g4oCcQuG7jyBs4buNYyBk4buxIMOhbuKAnSDhu58gYuG6o25nIHRyw6puIMSR4buDIHhlbSB04bqldCBj4bqjLg=="
              : V1rn
                ? "S2jDtG5nIGPDsm4gdMOgaSBsaeG7h3UgbsOgbyBjaOG7nSB4w6FjIG5o4bqtbiDigJQgYuG7jyB0aWNrIOKAnENo4buJIGhp4buHbiBwaOG6p24gY2jhu50geMOhYyBuaOG6rW7igJ0gxJHhu4MgeGVtIGzhuqFpIHRvw6BuIGLhu5ku"
                : "S2jDtG5nIGPDsyBo4buTIHPGoSBuw6BvIGto4bubcCB04burIGtob8OhIHTDrG0u"}
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
                      title="Q2jhu41uL2Lhu48gY2jhu41uIHRvw6BuIGLhu5kgdMOgaSBsaeG7h3UgxJFhbmcgaGnhu4du"
                      onChange={(V1sj) => V1s4(V1s0, V1sj)}
                    />
                  )}
                </th>
                <th>Dự án</th>
                <th>Hộp số</th>
                <th>Hồ sơ số</th>
                <th className="oh-files">Tổng số file</th>
                <th>Tài liệu</th>
                <th title="VHJhbmcgxJHDoyBjw7Mga+G6v3QgcXXhuqMgT0NSIHRyw6puIHThu5VuZyBz4buRIHRyYW5nLCB2w6Agc+G7kSB2xINuIGLhuqNuIMSRw6MgY2h1eeG7g24gxJHhu5VpIHNhbmcgUERGL0EgbMawdSB0cuG7ryB0csOqbiB04buVbmcgc+G7kSBmaWxl">
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
                          title={`Q2jhu41uIGPhuqMgZOG7sSDDoW4g${g.label} (${fmtN(g.ids.length)}IHTDoGkgbGnhu4d1KQ==`}
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
                                title={`Q2jhu41uIGPhuqMgaOG7kyBzxqEg${r.hoSoSo || "—"} (${fmtN(r.ids.length)}IHTDoGkgbGnhu4d1KQ==`}
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
                                  ? "VGh1IGfhu41uIGRhbmggc8OhY2ggZmlsZQ=="
                                  : "WGVtIHThu6tuZyBmaWxlIHRyb25nIGjhu5Mgc8ahIG7DoHk="
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
              ? `xJBhbmcgZ2hpIA==${fmtN(V1rs.done)}/${fmtN(V1rs.total)}IHTDoGkgbGnhu4d14oCm`
              : `xJDDoyBjaOG7jW4g${fmtN(V1rz.length)}IHTDoGkgbGnhu4d1`}
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
                  ? "VMOtY2ggY2jhu41uIGThu7Egw6FuIC8gaOG7kyBzxqEgLyB0w6BpIGxp4buHdSB0csaw4bubYw=="
                  : `${V4l0[d].hoi} ${fmtN(V1rz.length)}IHTDoGkgbGnhu4d1IMSRw6MgY2jhu41u`
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
