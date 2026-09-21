// Copyright (c) 2026 DigiText. All rights reserved.

import { useCallback, useEffect, useMemo, useState } from "react";
import { V41p } from "@/live/useLive";
import { isTauri, V3xq } from "@/api/ipc";
import { V3zt } from "@/state/sessionCore";
import {
  V4ex,
  V4ev,
  V4ez,
  V4ek,
  docsOcrRetryFailed,
  type V4ey,
  type V4ej,
} from "@/api/docs";
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
    pagesOcrFailed: p.pagesOcrFailed ?? 0,
    docsOcrFailed: p.docsOcrFailed ?? 0,
    pagesInkMeasured: p.pagesInkMeasured ?? 0,
    pagesInkLow: p.pagesInkLow ?? 0,
  };
}

export default function V4k5() {
  const [progress, V1nz] = useState<V4ey | null>(null);
  const [V1o0, V1o1] = useState<"" | "pdf2" | "fields" | "ocrretry">("");
  const [V1o2, V1o3] = useState("");
  const [V1oR, V1oS] = useState(false);

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
    V1ok ? ` Phạm vi đang lọc: ${fmtN(V1ov.docs.length)} tài liệu.` : "";

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
          ? `Đã đưa ${fmtN(r.documents)} tài liệu đã OCR xong vào hàng đợi ghép PDF 2 lớp.`
          : "Không có tài liệu nào sẵn sàng ghép PDF 2 lớp (cần OCR xong trước).") +
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

  const V1oT = async () => {
    V1o1("ocrretry");
    V1o3("");
    try {
      const r = await docsOcrRetryFailed(undefined, V1ow(), V1oR);
      V1o3(
        (r.pages > 0
          ? `Đã xếp hàng lại ${fmtN(r.pages)} trang OCR lỗi của ${fmtN(r.documents)} tài liệu.`
          : "Không có trang OCR lỗi nào trong phạm vi.") +
          (r.skippedSigned > 0
            ? ` Bỏ qua ${fmtN(r.skippedSigned)} tài liệu đã sang bước ký (không kéo ngược).`
            : "") +
          V1ox(),
      );
      await reload();
    } catch (e) {
      V1o3(V4k3(e));
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
                ? "— Tất cả THBQ —"
                : "— Danh mục chưa có THBQ —"}
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
              ? "Ghép PDF 2 lớp cho tài liệu đã OCR xong TRONG PHẠM VI đang lọc"
              : "Ghép PDF 2 lớp cho tài liệu đã OCR xong"
          }
        >
          <span className="srv-fill pdf2" style={{ width: `${V1p3}%` }} aria-hidden />
          <span className="srv-label">
            <b>① PDF 2 lớp</b>
            <span className="srv-sub">
              {V1o0 === "pdf2" ? "Đang gửi…" : ""}
              {view
                ? ` ${fmtN(view.docsPdf2Done)}/${fmtN(view.docsOcrDone)} tài liệu (${V1p3}%)`
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
              ? "Bóc tách 16 trường bằng AI cho tài liệu TRONG PHẠM VI đang lọc — chạy độc lập, không cần OCR trước"
              : "Bóc tách 16 trường thông tin bằng AI trên máy chủ — chạy độc lập, không cần OCR trước"
          }
        >
          <span className="srv-fill fields" style={{ width: `${V1p4}%` }} aria-hidden />
          <span className="srv-label">
            <b>② Bóc tách 16 trường</b>
            <span className="srv-sub">
              {V1o0 === "fields" ? "Đang gửi…" : ""}
              {view
                ? ` ${fmtN(view.docsFieldsDone)}/${fmtN(view.docs)} tài liệu (${V1p4}%)`
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
            title="Bóc lại các tài liệu đã hỏng vĩnh viễn — chỉ bấm sau khi đã sửa nguyên nhân, nếu không chúng sẽ hỏng y hệt"
          >
            <Icon name="lamMoi" />
            Thử lại {fmtN(view?.docsFieldsFailed ?? 0)} tài liệu hỏng
          </button>
        )}

        {(view?.pagesOcrFailed ?? 0) > 0 && (
          <>
            <button
              className="btn"
              onClick={() => void V1oT()}
              disabled={V1o0 !== ""}
              title="Xếp hàng lại các trang OCR thất bại vì dịch vụ OCR không phản hồi. Tài liệu đã sang bước ký sẽ được bỏ qua."
            >
              <Icon name="lamMoi" />
              {V1o0 === "ocrretry" ? "Đang gửi…" : `Thử lại ${fmtN(view?.pagesOcrFailed ?? 0)} trang OCR lỗi`}
            </button>
            <label className="muted text-sm" title="Trang hỏng vĩnh viễn (ảnh lỗi, định dạng lạ) thường hỏng y hệt nếu chưa sửa nguyên nhân">
              <input
                type="checkbox"
                checked={V1oR}
                onChange={(e) => V1oS(e.target.checked)}
                disabled={V1o0 !== ""}
              />{" "}
              kể cả trang hỏng vĩnh viễn
            </label>
          </>
        )}

        <button className="btn" onClick={() => void reload()} disabled={V1o0 !== ""}>
          <Icon name="lamMoi" />
          Làm mới
        </button>
      </div>

      {view && (
        <div className="muted srv-meta">
          {V1ok ? "Phạm vi đang lọc: " : ""}
          {fmtN(view.docs)} tài liệu · chờ OCR {fmtN(view.docsOcrPending)} · OCR xong{" "}
          {fmtN(view.docsOcrDone)} · có PDF 2 lớp {fmtN(view.docsPdf2Done)} · bóc 16 trường{" "}
          {fmtN(view.docsFieldsDone)}
          {view.docsFieldsFailed > 0
            ? ` (${fmtN(view.docsFieldsFailed)} lỗi — bấm “Thử lại tài liệu hỏng” sau khi sửa nguyên nhân)`
            : ""}
          {view.pagesOcrFailed > 0
            ? ` · OCR lỗi ${fmtN(view.pagesOcrFailed)} trang / ${fmtN(view.docsOcrFailed)} tài liệu`
            : ""}
          {view.pagesInkMeasured > 0
            ? view.pagesInkLow > 0
              ? ` · còn sót chữ ${fmtN(view.pagesInkLow)}/${fmtN(view.pagesInkMeasured)} trang đã đo (đã đưa vào hàng soát)`
              : ` · bắt đủ chữ ${fmtN(view.pagesInkMeasured)}/${fmtN(view.pagesInkMeasured)} trang đã đo`
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
            ? "Đang đọc kho máy chủ theo phạm vi…"
            : "Đang đọc tiến độ…"}
        </div>
      )}
      {V1o2 && <div className="muted text-sm mt-2">{V1o2}</div>}
    </div>
  );
}
