// Copyright (c) 2026 DigiText. All rights reserved.

import { useCallback, useEffect, useMemo, useState } from "react";
import { V41p } from "@/live/useLive";
import { isTauri, V3xq } from "@/api/ipc";
import { V3zt } from "@/state/sessionCore";
import { V4ex, V4ev, V4ez, V4ek, type V4ey, type V4ej } from "@/api/docs";
import { listHoSo, V40w, type V40d, type V40v } from "@/api/archive";
import { V48j } from "@/api/catalog";
import { V4br, type V4bm } from "@/api/dossier";
import { V4cm, V4cl } from "@/features/dossier/logic";
import type { V3ym, V3yl, V3yo } from "@/domain/types";
import { V4j9, V4j4, V4j0, V4j1, V4j3, type V4iz } from "@/features/upload/khoFilter";
import { V4it } from "@/features/upload/syncMatrix";
import { V4jj, V4jw, V4jo, V4jv, type V4jt } from "./jobScope";
import { V4k2, V4k0, V4jz } from "./progress";
import "@/features/upload/khogom.css";
import "./server.css";
import { Icon } from "@/components/Icon";

const V4k3 = (V1ny: unknown) =>
  V3zt(
    V1ny instanceof V3xq ? V1ny.code : "error",
    V1ny instanceof Error ? V1ny.message : String(V1ny),
  );

const fmtN = (n: number) => n.toLocaleString("vi-VN");

function V4k4(p: V4ey): V4jt {
  return {
    docs: p.docs,
    docsOcrPending: p.docsOcrPending,
    docsOcrDone: p.docsOcrDone,
    docsPdf2Done: p.docsPdf2Done,
    docsFieldsDone: p.docsFieldsDone ?? 0,
    docsFieldsActive: p.docsFieldsActive ?? 0,
    docsFieldsFailed: p.docsFieldsFailed ?? 0,
  };
}

export default function V4k5() {
  const [progress, V1nz] = useState<V4ey | null>(null);
  const [V1o0, V1o1] = useState<"" | "pdf2" | "fields">("");
  const [V1o2, V1o3] = useState("");

  const [flt, V1o4] = useState<V4iz>(V4j0);
  const [V1o5, V1o6] = useState<V40v[]>([]);
  const [V1o7, V1o8] = useState<V40d[]>([]);
  const [V1o9, V1oa] = useState<V3yl[]>([]);
  const [V1ob, V1oc] = useState<V3ym[]>([]);
  const [V1od, V1oe] = useState<V3yo[]>([]);
  const [V1of, V1og] = useState<V4bm[]>([]);

  const [tree, V1oh] = useState<V4ej[] | null>(null);
  const [V1oi, V1oj] = useState(false);

  const V1ok = V4j1(flt);

  const V1ol = useCallback(async () => {
    if (!isTauri()) return;
    try {
      V1nz(await V4ez());
    } catch (V1om) {
      V1o3(V4k3(V1om));
    }
  }, []);

  const V1on = useCallback(async () => {
    if (!isTauri()) return;
    V1oj(true);
    try {
      V1oh(await V4ek());
    } catch (V1oo) {
      V1o3(V4k3(V1oo));
      V1oh([]);
    } finally {
      V1oj(false);
    }
  }, []);

  V41p(V1ol, { topics: ["docs", "ocr", "ocrFields"], enabled: !V1ok });
  V41p(V1on, { topics: ["docs", "ocr", "ocrFields"], enabled: V1ok });

  useEffect(() => {
    if (!isTauri()) return;
    void (async () => {
      try {
        const [V1op, V1oq] = await Promise.all([V40w(), listHoSo()]);
        V1o6(V1op);
        V1o8(V1oq);
      } catch {
        V1o6([]);
        V1o8([]);
      }
      try {
        const [cat, V1or] = await Promise.all([V48j(), V4br()]);
        V1oa(cat.phongBans);
        V1oc(cat.mangs);
        V1oe(cat.thbqs);
        V1og(V1or);
      } catch {
        V1oa([]);
        V1oc([]);
        V1oe([]);
        V1og([]);
      }
    })();
  }, []);

  const V1os = useMemo(() => V4jj(V1o5), [V1o5]);
  const V1ot = useMemo(() => V4it(V1o7), [V1o7]);
  const V1ou = useMemo(() => V4j9(V1of, V1o7, V1od), [V1of, V1o7, V1od]);
  const V1ov = useMemo(
    () => V4jo(tree ?? [], flt, V1os, V1ot, V1ou),
    [tree, flt, V1os, V1ot, V1ou],
  );

  const view = useMemo<V4jt | null>(() => {
    if (V1ok) return tree === null ? null : V4jv(V1ov.docs);
    return progress === null ? null : V4k4(progress);
  }, [V1ok, tree, V1ov, progress]);

  const V1ow = () => (V1ok ? V4jw(V1ov.docs) : undefined);

  const V1ox = () =>
    V1ok ? `IFBo4bqhbSB2aSDEkWFuZyBs4buNYzog${fmtN(V1ov.docs.length)}IHTDoGkgbGnhu4d1Lg==` : "";

  const reload = async () => {
    if (V1ok) await V1on();
    else await V1ol();
  };

  const V1oy = async () => {
    V1o1("pdf2");
    V1o3("");
    try {
      const r = await V4ev(undefined, V1ow());
      V1o3(
        (r.documents > 0
          ? `xJDDoyDEkcawYSA=${fmtN(r.documents)}IHTDoGkgbGnhu4d1IMSRw6MgT0NSIHhvbmcgdsOgbyBow6BuZyDEkeG7o2kgZ2jDqXAgUERGIDIgbOG7m3Au`
          : "S2jDtG5nIGPDsyB0w6BpIGxp4buHdSBuw6BvIHPhurVuIHPDoG5nIGdow6lwIFBERiAyIGzhu5twIChj4bqnbiBPQ1IgeG9uZyB0csaw4bubYyku") +
          V1ox(),
      );
      await reload();
    } catch (V1oz) {
      V1o3(V4k3(V1oz));
    } finally {
      V1o1("");
    }
  };

  const V1p0 = async (V1p1 = false) => {
    V1o1("fields");
    V1o3("");
    try {
      const r = await V4ex(undefined, V1p1, V1ow());
      V1o3(V4k2(r) + V1ox());
      await reload();
    } catch (V1p2) {
      V1o3(V4k3(V1p2));
    } finally {
      V1o1("");
    }
  };

  const V1p3 = view ? V4jz(view) : 0;
  const V1p4 = view ? V4k0(view) : 0;

  const V1p5 = V1ok ? tree === null : progress === null;

  const V1p6 = useMemo(() => V4cl(V1o9, flt.maDa), [V1o9, flt.maDa]);
  const V1p7 = useMemo(() => V4cm(V1ob, flt.maDv), [V1ob, flt.maDv]);

  const V1p8 = (v: string) => V1o4((f) => ({ ...f, maDa: v, maDv: "", mangCvid: "" }));

  return (
    <div className="card mb-3">
      <h3 className="mt-0">Xử lý trên máy chủ</h3>
      <p className="muted mt-0">
        Sau khi <b>OCR toàn bộ</b> xong, bấm <b>PDF 2 lớp</b> để ghép bản PDF có lớp chữ tìm kiếm
        được; bấm <b>Bóc tách 16 trường</b> để AI đọc thẳng ảnh trang và điền sẵn 16 trường thông
        tin (ĐỘC LẬP, không cần OCR trước, kết quả luôn chờ người soát). Chọn phạm vi bên dưới để
        chỉ chạy (và chỉ xem tiến độ) cho một phông / phòng ban / mảng CV / THBQ; không chọn gì là
        toàn bộ kho máy chủ.
      </p>

      {}
      <div className="kg-flt srv-scope">
        <label className="kg-dfield">
          <span>Phông</span>
          <select className="inp" value={flt.maDa} onChange={(V1p9) => V1p8(V1p9.target.value)}>
            <option value="">— Tất cả phông —</option>
            {V1o5.map((p) => (
              <option key={p.maDuAn} value={p.maDuAn}>
                {p.tenDuAn || p.maDuAn}
              </option>
            ))}
          </select>
        </label>
        <label className="kg-dfield">
          <span>Phòng ban</span>
          <select
            className="inp"
            value={flt.maDv}
            disabled={!flt.maDa || V1p6.length === 0}
            onChange={(V1pa) => V1o4((f) => ({ ...f, maDv: V1pa.target.value, mangCvid: "" }))}
          >
            <option value="">{V4j4("phongBan", !!flt.maDa, V1p6.length)}</option>
            {V1p6.map((V1pb) => (
              <option key={V1pb.maDv} value={V1pb.maDv}>
                {V1pb.tenDonVi || V1pb.maDonVi || V1pb.maDv}
              </option>
            ))}
          </select>
        </label>
        <label className="kg-dfield">
          <span>Mảng CV</span>
          <select
            className="inp"
            value={flt.mangCvid}
            disabled={!flt.maDv || V1p7.length === 0}
            onChange={(V1pc) => V1o4((f) => ({ ...f, mangCvid: V1pc.target.value }))}
          >
            <option value="">{V4j4("mang", !!flt.maDv, V1p7.length)}</option>
            {V1p7.map((m) => (
              <option key={m.mangCvid} value={String(m.mangCvid)}>
                {m.tenMang || m.maMang}
              </option>
            ))}
          </select>
        </label>
        <label className="kg-dfield">
          <span>THBQ</span>
          <select
            className="inp"
            value={flt.thbq}
            disabled={V1od.length === 0}
            onChange={(V1pd) => V1o4((f) => ({ ...f, thbq: V1pd.target.value }))}
          >
            <option value="">
              {V1od.length
                ? "4oCUIFThuqV0IGPhuqMgVEhCUSDigJQ="
                : "4oCUIERhbmggbeG7pWMgY2jGsGEgY8OzIFRIQlEg4oCU"}
            </option>
            {V1od.map((t) => (
              <option key={t.thbqid} value={String(t.thbqid)}>
                {t.tenThbq}
              </option>
            ))}
          </select>
        </label>
      </div>

      <div className="srv-actions">
        <button
          className="srv-progbtn"
          onClick={() => void V1oy()}
          disabled={V1o0 !== "" || V1p5}
          title={
            V1ok
              ? "R2jDqXAgUERGIDIgbOG7m3AgY2hvIHTDoGkgbGnhu4d1IMSRw6MgT0NSIHhvbmcgVFJPTkcgUEjhuqBNIFZJIMSRYW5nIGzhu41j"
              : "R2jDqXAgUERGIDIgbOG7m3AgY2hvIHTDoGkgbGnhu4d1IMSRw6MgT0NSIHhvbmc="
          }
        >
          <span className="srv-fill pdf2" style={{ width: `${V1p3}%` }} aria-hidden />
          <span className="srv-label">
            <b>① PDF 2 lớp</b>
            <span className="srv-sub">
              {V1o0 === "pdf2" ? "xJBhbmcgZ+G7rWnigKY=" : ""}
              {view
                ? ` ${fmtN(view.docsPdf2Done)}/${fmtN(view.docsOcrDone)}IHTDoGkgbGnhu4d1ICg=${V1p3}%)`
                : " …"}
            </span>
          </span>
        </button>

        <button
          className="srv-progbtn"
          onClick={() => void V1p0()}
          disabled={V1o0 !== "" || V1p5}
          title={
            V1ok
              ? "QsOzYyB0w6FjaCAxNiB0csaw4budbmcgYuG6sW5nIEFJIGNobyB0w6BpIGxp4buHdSBUUk9ORyBQSOG6oE0gVkkgxJFhbmcgbOG7jWMg4oCUIGNo4bqheSDEkeG7mWMgbOG6rXAsIGtow7RuZyBj4bqnbiBPQ1IgdHLGsOG7m2M="
              : "QsOzYyB0w6FjaCAxNiB0csaw4budbmcgdGjDtG5nIHRpbiBi4bqxbmcgQUkgdHLDqm4gbcOheSBjaOG7pyDigJQgY2jhuqF5IMSR4buZYyBs4bqtcCwga2jDtG5nIGPhuqduIE9DUiB0csaw4bubYw=="
          }
        >
          <span className="srv-fill fields" style={{ width: `${V1p4}%` }} aria-hidden />
          <span className="srv-label">
            <b>② Bóc tách 16 trường</b>
            <span className="srv-sub">
              {V1o0 === "fields" ? "xJBhbmcgZ+G7rWnigKY=" : ""}
              {view
                ? ` ${fmtN(view.docsFieldsDone)}/${fmtN(view.docs)}IHTDoGkgbGnhu4d1ICg=${V1p4}%)`
                : " …"}
            </span>
          </span>
        </button>

        {}
        {(view?.docsFieldsFailed ?? 0) > 0 && (
          <button
            className="btn"
            onClick={() => void V1p0(true)}
            disabled={V1o0 !== ""}
            title="QsOzYyBs4bqhaSBjw6FjIHTDoGkgbGnhu4d1IMSRw6MgaOG7j25nIHbEqW5oIHZp4buFbiDigJQgY2jhu4kgYuG6pW0gc2F1IGtoaSDEkcOjIHPhu61hIG5ndXnDqm4gbmjDom4sIG7hur91IGtow7RuZyBjaMO6bmcgc+G6vSBo4buPbmcgeSBo4buHdA=="
          >
            <Icon name="lamMoi" />
            Thử lại {fmtN(view?.docsFieldsFailed ?? 0)} tài liệu hỏng
          </button>
        )}

        <button className="btn" onClick={() => void reload()} disabled={V1o0 !== ""}>
          <Icon name="lamMoi" />
          Làm mới
        </button>
      </div>

      {view && (
        <div className="muted srv-meta">
          {V1ok ? "UGjhuqFtIHZpIMSRYW5nIGzhu41jOiA=" : ""}
          {fmtN(view.docs)} tài liệu · chờ OCR {fmtN(view.docsOcrPending)} · OCR xong{" "}
          {fmtN(view.docsOcrDone)} · có PDF 2 lớp {fmtN(view.docsPdf2Done)} · bóc 16 trường{" "}
          {fmtN(view.docsFieldsDone)}
          {view.docsFieldsFailed > 0
            ? ` (${fmtN(view.docsFieldsFailed)}IGzhu5dpIOKAlCBi4bqlbSDigJxUaOG7rSBs4bqhaSB0w6BpIGxp4buHdSBo4buPbmfigJ0gc2F1IGtoaSBz4butYSBuZ3V5w6puIG5ow6JuKQ==`
            : ""}
          {}
          {V1ov.unknown > 0 && V4j3(flt) && (
            <>
              {" "}
              Đã loại {fmtN(V1ov.unknown)} tài liệu chưa có trong mục lục (không xét được phòng ban
              / mảng CV / THBQ) — chạy “Đồng bộ từ Kho tài liệu” ở màn Khai thác rồi lọc lại.
            </>
          )}
        </div>
      )}
      {(V1p5 || V1oi) && (
        <div className="muted srv-meta">
          {V1ok
            ? "xJBhbmcgxJHhu41jIGtobyBtw6F5IGNo4bunIHRoZW8gcGjhuqFtIHZp4oCm"
            : "xJBhbmcgxJHhu41jIHRp4bq/biDEkeG7meKApg=="}
        </div>
      )}
      {V1o2 && <div className="muted text-sm mt-2">{V1o2}</div>}
    </div>
  );
}
