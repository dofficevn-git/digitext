// Copyright (c) 2026 DigiText. All rights reserved.

import { useCallback, useState } from "react";
import { V41p } from "@/live/useLive";
import { open } from "@tauri-apps/plugin-dialog";
import { V460 } from "@/state";
import { V3xq, isTauri } from "@/api/ipc";
import { V3zt } from "@/state/sessionCore";
import { V3z9 } from "@/domain/permissions";
import { V4et, V4fc, V4fb, V4f3, V4er, type V4f2, type V4f0, type V4f9 } from "@/api/docs";
import { V4ak, type V4a9 } from "@/api/users";
import V4i7, { type V4i6 } from "../upload/OcrFieldsView";
import V4k5 from "../server/ServerJobs";
import Banner from "./Banner";
import ReviewHoSoTable from "./ReviewHoSoTable";
import { V4hg, pdfaSummary, V4hi, V4hj, V4hh, type V4he } from "./ocrReport";
import { Icon } from "@/components/Icon";

const V4l7 = (n: number) => n.toLocaleString("vi-VN");

const V4l8: Record<V4f9["state"], { label: string; tone: "ok" | "err" | "warn" | "gray" }> = {
  ok: { label: "xJDhuqF0", tone: "ok" },
  invalid: { label: "Q+G6pXUgdHLDumMgc2Fp", tone: "err" },
  missing: { label: "VGhp4bq/dSBQREY=", tone: "warn" },
  pending: { label: "Q2jGsGEgT0NSIHhvbmc=", tone: "gray" },
};

function BocTachCell({ c }: { c: V4f0 }) {
  return (
    <>
      <div
        title={
          c.ocrPages ? `${V4l7(c.ocrPages)}IHRyYW5nIMSRw6MgY8OzIGvhur90IHF14bqjIE9DUg==` : undefined
        }
      >
        {V4l7(c.ocrDone)}
      </div>
      {c.fieldDocs != null && c.fieldDocs > 0 && (
        <div
          className="muted text-sm"
          title="VMOgaSBsaeG7h3UgxJHDoyBiw7NjIDE2IHRyxrDhu51uZyBt4bulYyBs4bulYyAobsO6dCDikaEgQsOzYyB0w6FjaCAxNiB0csaw4budbmcp"
        >
          {V4l7(c.fieldDocs)} bóc trường
        </div>
      )}
    </>
  );
}

function PdfaCell({ c }: { c: V4f0 }) {
  const t = pdfaSummary(c);
  if (!t.known) {
    return (
      <span
        className="muted"
        title="TcOheSBjaOG7pyDEkWFuZyBjaOG6oXkgYuG6o24gY8WpLCBjaMawYSBiw6FvIHPhu5EgdsSDbiBi4bqjbiDEkcOjIGNodXnhu4NuIFBERi9B"
      >
        —
      </span>
    );
  }
  return (
    <>
      <div
        title={`xJDDoyBjaHV54buDbiDEkeG7lWkg${t.docs}/${t.total}IHbEg24gYuG6o24gc2FuZyBQREYvQSBsxrB1IHRy4buvICg=${t.percent}JSkuIELhuqNuIGdow6lwIHNpbmggcmEg4bufIG7DunQgIlBERiAyIGzhu5twIiBtw6BuIE3DoXkgY2jhu6cg4oCUIE9DUiB4b25nIGNoxrBhIGPDsyBuZ2jEqWEgbMOgIMSRw6MgY8OzIHThu4dwIFBERi9BLg==`}
      >
        {V4l7(t.docs)}
        <span className="muted text-sm">/{V4l7(t.total)}</span>
      </div>
      {t.old > 0 && (
        <div
          className="text-sm text-warning-700"
          title="R2jDqXAgYuG7n2kgYuG6o24gY8WpLCB0aGnhur91IGThuqV1IGhp4buHdSBQREYvQS0yYiDigJQgYuG6pW0g4oCcR2jDqXAgbOG6oWkgUERGL0HigJ0gxJHhu4MgxJHhuqF0IMSQaeG7gXUgOC4yLmE="
        >
          {V4l7(t.old)} chưa đạt 2b
        </div>
      )}
    </>
  );
}

function V4l9({ rows: V1so, onClose: V1sp }: { rows: V4f9[]; onClose: () => void }) {
  const V1sq = V1so.reduce<Record<string, number>>(
    (m, r) => ((m[r.state] = (m[r.state] ?? 0) + 1), m),
    {},
  );
  return (
    <div className="card p-0 overflow-hidden mb-3">
      <div className="flex items-center gap-3 py-3 px-4 flex-wrap">
        <h3 className="m-0">Kiểm tra PDF A (PDF 2 lớp lưu trữ)</h3>
        <span className="muted text-sm">
          {(["ok", "invalid", "missing", "pending"] as const)
            .filter((s) => V1sq[s])
            .map((s) => `${V4l8[s].label}: ${V1sq[s]}`)
            .join(" · ") || "a2jDtG5nIGPDsyB0w6BpIGxp4buHdQ=="}
          {V1so.some((r) => (r.state === "ok" || r.state === "invalid") && !r.pdfa2b) &&
            `IMK3IGNoxrBhIFBERi9BOiA=${V1so.filter((r) => (r.state === "ok" || r.state === "invalid") && !r.pdfa2b).length}`}
        </span>
        <div className="flex-1" />
        <button className="btn sm" onClick={V1sp}>
          <Icon name="dong" />
          Đóng
        </button>
      </div>
      {V1so.length > 0 && (
        <div className="tbl-scroll">
          <table className="tbl">
            <thead>
              <tr>
                <th>Tài liệu</th>
                <th>Hồ sơ</th>
                <th>Trạng thái</th>
                <th className="num">Trang</th>
                <th
                  className="num"
                  title="U+G7kSB0cmFuZyBraMO0bmcgY8OzIGNo4buvIHRyb25nIGzhu5twIHRleHQ="
                >
                  Không chữ
                </th>
                <th title="xJDhu6cgZOG6pXUgaGnhu4d1IFBERi9BLTJiIChYTVAgcGRmYWlkICsgT3V0cHV0SW50ZW50ICsgL0lEICsgZm9udCBuaMO6bmcp">
                  PDF/A
                </th>
                <th>Ghi chú</th>
              </tr>
            </thead>
            <tbody>
              {V1so.map((r) => {
                const V1sr = V4l8[r.state];
                const V1ss =
                  r.state === "ok" || r.state === "invalid"
                    ? `${r.pages}${r.expectedPages != null ? `/${r.expectedPages}` : ""}`
                    : (r.expectedPages ?? "—");
                return (
                  <tr key={r.id}>
                    <td>{r.filename}</td>
                    <td>{r.hoSoRef ?? "—"}</td>
                    <td>
                      <span className={`badge ${V1sr.tone}`} title={r.status}>
                        {V1sr.label}
                      </span>
                    </td>
                    <td className="num">{V1ss}</td>
                    <td className="num">
                      {r.state === "ok" || r.state === "invalid" ? (
                        <span
                          className={r.pagesWithoutText > 0 ? "text-warning-700" : undefined}
                          title={`${r.pagesWithText}IHRyYW5nIGPDsyBjaOG7rw==`}
                        >
                          {r.pagesWithoutText}
                        </span>
                      ) : (
                        "—"
                      )}
                    </td>
                    <td>
                      {r.state === "ok" || r.state === "invalid" ? (
                        <span
                          className={`badge ${r.pdfa2b ? "ok" : "warn"}`}
                          title={
                            r.pdfa2b
                              ? "xJDhu6cgZOG6pXUgaGnhu4d1IFBERi9BLTJi"
                              : "R2jDqXAgYuG7n2kgYuG6o24gY8WpIOKAlCBi4bqlbSDigJxHaMOpcCBs4bqhaSBQREYvQeKAnSDEkeG7gyDEkeG6oXQgxJBp4buBdSA4LjIuYQ=="
                          }
                        >
                          {r.pdfa2b ? "2b" : "Y2jGsGE="}
                        </span>
                      ) : (
                        "—"
                      )}
                    </td>
                    <td className="muted text-sm">
                      {r.reason ??
                        (r.state === "ok" ? (
                          <>
                            <Icon name="duyet" size={13} /> có lớp text + ảnh nền
                          </>
                        ) : (
                          ""
                        ))}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

function V4la({
  label: V1st,
  value: V1su,
  tone: V1sv,
  title: V1sw,
}: {
  label: string;
  value: number;
  tone?: "ok" | "err" | "warn";
  title?: string;
}) {
  return (
    <div className="tile" title={V1sw}>
      <span
        className={`num${V1sv ? ` text-${V1sv === "ok" ? "success" : V1sv === "err" ? "danger" : "warning"}-700` : ""}`}
      >
        {V4l7(V1su)}
      </span>
      <span className="lbl">{V1st}</span>
    </div>
  );
}

const V4lb = (V1sx: unknown) =>
  V3zt(
    V1sx instanceof V3xq ? V1sx.code : "error",
    V1sx instanceof Error ? V1sx.message : String(V1sx),
  );

export default function OcrPane() {
  const { session: V1sy } = V460();
  const [V1sz, V1t0] = useState(false);
  const [V1t1, V1t2] = useState<string | null>(null);
  const [V1t3, V1t4] = useState<string | null>(null);

  const [V1t5, V1t6] = useState<V4a9[]>([]);
  const [V1t7, V1t8] = useState("");

  const [V1t9, V1ta] = useState<V4i6 | null>(null);

  const [V1tb, V1tc] = useState(false);
  const [V1td, V1te] = useState(false);
  const [V1tf, V1tg] = useState(false);
  const [V1th, V1ti] = useState<V4f9[] | null>(null);

  const [V1tj, V1tk] = useState<V4he>(() => V4hg());
  const [report, V1tl] = useState<V4f2 | null>(null);
  const [V1tm, V1tn] = useState(true);
  const [V1to, V1tp] = useState("");

  const V1tq = V1sy?.projectId ?? null;
  const V1tr = V1tq ?? V1t7;
  const V1ts = V3z9(V1sy?.permissions, "document.review");
  const V1tt = V4hh(V1tj);

  const V1tu = useCallback(async () => {
    if (!isTauri() || V4hh(V1tj)) {
      V1tn(false);
      return;
    }
    V1tn(true);
    try {
      V1tl(await V4f3(V1tj.from || undefined, V1tj.to || undefined));
    } catch (V1tv) {
      V1t2(V4lb(V1tv));
      V1tl(null);
    } finally {
      V1tn(false);
    }
  }, [V1tj]);
  V41p(V1tu, { topics: ["docs", "ocr"] });

  const V1tw = useCallback(() => {
    V4ak()
      .then((V1tx) => {
        const V1ty = V1tx.filter((p) => p.isActive);
        V1t6(V1ty);
        if (V1ty.length === 1) V1t8(V1ty[0].id);
      })
      .catch(() => {});
  }, []);
  V41p(V1tw, {
    topics: ["projects"],
    enabled: !V1tq && isTauri(),
  });

  const V1tz = async () => {
    if (!V1tr) {
      V1t2(
        V1t5.length > 1
          ? "SMOjeSBjaOG7jW4gZOG7sSDDoW4gxJHDrWNoIOG7nyDDtCBiw6puIGPhuqFuaCB0csaw4bubYyBraGkgdOG6o2kgdMOgaSBsaeG7h3UgbMOqbi4="
          : "VMOgaSBraG/huqNuIGNoxrBhIGfhuq9uIGThu7Egw6FuIG7DoG8g4oCUIGtow7RuZyB0aOG7gyB04bqjaSB0w6BpIGxp4buHdSBsw6puLg==",
      );
      return;
    }
    let V1u0: string | null;
    try {
      const r = await open({
        multiple: false,
        title: "Q2jhu41uIFBERiDEkeG7gyBnw6FuIG5ow6Nu",
        filters: [{ name: "PDF", extensions: ["pdf"] }],
      });
      V1u0 = typeof r === "string" ? r : null;
    } catch (V1u1) {
      V1t2(V4lb(V1u1));
      return;
    }
    if (!V1u0) return;
    V1t0(true);
    V1t2(null);
    V1t4(null);
    try {
      const V1u2 = await V4er(V1tr, V1u0);
      void V1tu();

      V1t4(
        V1u2.duplicate
          ? `"${V1u2.filename}IiDEkcOjIGPDsyBz4bq1biB0csOqbiBtw6F5IGNo4bunLg==`
          : `xJDDoyB04bqjaSAi${V1u2.filename}IiBsw6puIG3DoXkgY2jhu6cu`,
      );
      V1ta({ id: V1u2.id, filename: V1u2.filename, reviewStatus: "pending" });
    } catch (V1u3) {
      V1t2(V4lb(V1u3));
    } finally {
      V1t0(false);
    }
  };

  const V1u4 = async () => {
    V1tc(true);
    V1t2(null);
    V1t4(null);
    try {
      const r = await V4et(V1tr || undefined);
      if (r.documents === 0) {
        V1t4(
          r.skippedNoPageCount > 0
            ? `S2jDtG5nIGPDsyB0w6BpIGxp4buHdSBuw6BvIMSR4buDIE9DUiAoYuG7jyBxdWEg${r.skippedNoPageCount}IHTDoGkgbGnhu4d1IGNoxrBhIHLDtSBz4buRIHRyYW5nKS4=`
            : "TeG7jWkgdMOgaSBsaeG7h3UgxJHDoyDEkcaw4bujYyDEkcawYSB2w6BvIE9DUiB0csaw4bubYyDEkcOzIOKAlCBraMO0bmcgY8OzIGfDrCBt4bubaS4=",
        );
      } else {
        const V1u5 =
          r.skippedNoPageCount > 0
            ? `IChi4buPIHF1YSA=${r.skippedNoPageCount}IGNoxrBhIHLDtSBz4buRIHRyYW5nKQ==`
            : "";
        V1t4(
          `xJDDoyDEkcawYSA=${r.documents}IHTDoGkgbGnhu4d1ICg=${r.pages}IHRyYW5nKSB2w6BvIGjDoG5nIMSR4bujaSBPQ1I=${V1u5}.`,
        );
      }
      void V1tu();
    } catch (V1u6) {
      V1t2(V4lb(V1u6));
    } finally {
      V1tc(false);
    }
  };

  const V1u7 = async () => {
    V1te(true);
    V1t2(null);
    V1t4(null);
    try {
      V1ti(await V4fc());
    } catch (V1u8) {
      V1t2(V4lb(V1u8));
    } finally {
      V1te(false);
    }
  };

  const V1u9 = async () => {
    V1tg(true);
    V1t2(null);
    V1t4(null);
    try {
      const r = await V4fb();
      const V1ua =
        r.skippedSigned > 0
          ? ` ${r.skippedSigned}IHTDoGkgbGnhu4d1IMSQw4MgS8OdIGNoxrBhIMSR4bqhdCBQREYvQSDigJQgcGjhuqNpIGvDvSBs4bqhaSB0aGVvIHF1eSB0csOsbmgu`
          : "";
      V1t4(
        `xJDDoyBraeG7g20g${r.checked}IHTDoGkgbGnhu4d1LCB44bq/cCBow6BuZyBnaMOpcCBs4bqhaSA=${r.enqueued}.${V1ua}`,
      );
      if (V1th) V1ti(await V4fc());
    } catch (V1ub) {
      V1t2(V4lb(V1ub));
    } finally {
      V1tg(false);
    }
  };

  if (V1t9) {
    return (
      <V4i7
        doc={V1t9}
        onBack={() => {
          V1ta(null);
          void V1tu();
        }}
      />
    );
  }

  const totals = report?.totals;

  return (
    <>
      {V1t1 && <Banner tone="err" text={V1t1} onClose={() => V1t2(null)} />}
      {V1t3 && <Banner tone="ok" text={V1t3} onClose={() => V1t4(null)} />}

      {}
      <div className="card mb-3">
        <div className="kpi-row">
          <V4la
            label="VOG7lW5nIHPhu5EgZOG7sSDDoW4="
            value={report?.projectsCount ?? 0}
            title="U+G7kSBk4buxIMOhbiB0cm9uZyBwaOG6oW0gdmkgcGjhu6UgdHLDoWNo"
          />
          <V4la
            label="VOG7lW5nIHPhu5EgZmlsZQ=="
            value={totals?.docs ?? 0}
            title={`${V4l7(totals?.pages ?? 0)} trang`}
          />
          <V4la
            label="RmlsZSDEkcOjIE9DUg=="
            value={totals?.ocrDone ?? 0}
            title={`VMOgaSBsaeG7h3UgxJHDoyBjw7Mga+G6v3QgcXXhuqMgT0NSIMSR4bunIHPhu5EgdHJhbmcgKG7DunQg4pGgIFBERiAyIGzhu5twKQ==${
              totals?.ocrPages ? ` · ${V4l7(totals.ocrPages)}IHRyYW5nIMSRw6MgT0NS` : ""
            }${
              totals?.fieldDocs
                ? ` · ${V4l7(totals.fieldDocs)}IHTDoGkgbGnhu4d1IMSRw6MgYsOzYyAxNiB0csaw4budbmcgKOKRoSk=`
                : ""
            }`}
          />
          <V4la
            label="xJDDoyBkdXnhu4d0IChhZG1pbik="
            value={totals?.approved ?? 0}
            tone="ok"
            title="QWRtaW4gxJHDoyB4w6FjIG5o4bqtbiBr4bq/dCBxdeG6oyBiw7NjIHTDoWNo"
          />
          <V4la
            label="VOG7qyBjaOG7kWk="
            value={totals?.rejected ?? 0}
            tone="err"
            title="QWRtaW4gdHLhuqMgbOG6oWkg4oCUIHBo4bqjaSBxdcOpdC9iw7NjIGzhuqFp"
          />
          <V4la
            label="Q2jhu50geMOhYyBuaOG6rW4="
            value={totals?.pending ?? 0}
            tone="warn"
            title="Q2jGsGEgY8OzIHF1eeG6v3QgxJHhu4tuaCBkdXnhu4d0IChn4buTbSBj4bqjIHTDoGkgbGnhu4d1IG3DoXkgY2jGsGEgT0NSIHhvbmcp"
          />
        </div>

        <div className="mt-3.5 flex flex-wrap items-center gap-2.5 border-t border-border-subtle pt-3">
          <span className="muted text-sm">Ngày tải lên máy chủ:</span>
          <input
            className="inp sm"
            type="date"
            value={V1tj.from}
            max={V1tj.to || undefined}
            onChange={(V1uc) => V1tk((r) => ({ ...r, from: V1uc.target.value }))}
            title="TmfDoHkgYuG6r3QgxJHhuqd1"
          />
          <span className="muted">—</span>
          <input
            className="inp sm"
            type="date"
            value={V1tj.to}
            min={V1tj.from || undefined}
            onChange={(V1ud) => V1tk((r) => ({ ...r, to: V1ud.target.value }))}
            title="TmfDoHkga+G6v3QgdGjDumMgKHTDrW5oIHRy4buNbiBuZ8OgeSBuw6B5KQ=="
          />
          <button
            className="btn sm"
            onClick={() => V1tk({ from: "", to: "" })}
            title="QuG7jyBs4buNYywgeGVtIHRvw6BuIGLhu5k="
          >
            Toàn bộ
          </button>
          <button
            className="btn sm"
            onClick={() => V1tk(V4hg())}
            title="VOG7qyDEkeG6p3UgdGjDoW5nIMSR4bq/biBow7RtIG5heQ=="
          >
            Tháng này
          </button>
          {V1tt ? (
            <span className="text-danger-700 text-xs">{V1tt}</span>
          ) : (
            <span className="muted text-sm">
              {V1tm
                ? "xJBhbmcgdMOtbmggc+G7kSBsaeG7h3XigKY="
                : V1tj.from || V1tj.to
                  ? `xJBhbmcgeGVtOiA=${V1tj.from || "xJHhuqd1IGvhu7M="} → ${V1tj.to || "nay"}`
                  : "xJBhbmcgeGVtOiB0b8OgbiBi4buZ"}
            </span>
          )}
        </div>
      </div>

      {}
      <div className="card p-0 overflow-hidden mb-3">
        <div className="flex items-center gap-3 py-3 px-4 flex-wrap">
          <h3 className="m-0">Theo dõi dự án</h3>
          <span className="muted text-sm">Bấm một dự án để lọc danh sách tài liệu bên dưới</span>
          <div className="flex-1" />
          {V1to && (
            <button className="btn sm" onClick={() => V1tp("")}>
              <Icon name="tuChoi" />
              Bỏ lọc dự án
            </button>
          )}
        </div>
        {report && report.projects.length > 0 ? (
          <div className="tbl-scroll">
            <table className="tbl">
              <thead>
                <tr>
                  <th className="num">#</th>
                  <th>Dự án</th>
                  <th className="num">Tổng file</th>
                  <th
                    className="num"
                    title="VHLDqm46IHTDoGkgbGnhu4d1IMSRw6MgT0NSIMSR4bunIHPhu5EgdHJhbmcgKG7DunQg4pGgIFBERiAyIGzhu5twLCDEkeG6v20gdGhlbyBvY3JfcmVzdWx0cykuIETGsOG7m2k6IHTDoGkgbGnhu4d1IMSRw6MgYsOzYyAxNiB0csaw4budbmcgKG7DunQg4pGhIFZMTSwgxJHhur9tIHRoZW8gb2NyX2ZpZWxkcykg4oCUIGhhaSB2aeG7h2Mga2jDoWMgbmhhdSBuw6puIMSR4bq/bSByacOqbmcu"
                  >
                    Đã bóc tách
                  </th>
                  <th
                    className="num"
                    title="U+G7kSB2xINuIGLhuqNuIMSRw6MgY2h1eeG7g24gxJHhu5VpIHNhbmcgUERGL0EgbMawdSB0cuG7ryB0csOqbiB04buVbmcgc+G7kSB0w6BpIGxp4buHdSAoduG6v3QgZ2hpIGtoaSBraG8gxJHDoyBuaOG6rW4gYuG6o24gZ2jDqXApLiDEkMOieSBsw6Agdmnhu4djIGPhu6dhIG7DunQg4oCcUERGIDIgbOG7m3DigJ0gbcOgbiBNw6F5IGNo4bunIOKAlCBPQ1IgeG9uZyBjaMawYSBjw7MgbmdoxKlhIGzDoCDEkcOjIGPDsyB04buHcCBQREYvQS4="
                  >
                    Đã PDF A
                  </th>
                  <th className="num">Đã duyệt</th>
                  <th className="num">Từ chối</th>
                  <th className="num">Chờ xác nhận</th>
                  <th title="VMOgaSBsaeG7h3UgxJHDoyBkdXnhu4d0IC8gdOG7lW5nIHPhu5EgdMOgaSBsaeG7h3U=">
                    % hoàn thành
                  </th>
                </tr>
              </thead>
              <tbody>
                {report.projects.map((p, i) => {
                  const V1ue = p.projectId === V1to;
                  return (
                    <tr
                      key={p.projectId}
                      onClick={() => V1tp(V1ue ? "" : p.projectId)}
                      style={{
                        cursor: "pointer",
                        background: V1ue ? "var(--line-2)" : undefined,
                      }}
                      title={
                        V1ue
                          ? "QuG6pW0gbOG6oWkgxJHhu4MgYuG7jyBs4buNYw=="
                          : `Q2jhu4kgeGVtIHTDoGkgbGnhu4d1IGPhu6dhIA==${p.name}`
                      }
                    >
                      <td className="num">{i + 1}</td>
                      <td>
                        {p.name}
                        <span className="muted text-sm"> · {p.code}</span>
                        {!p.isActive && <span className="badge gray ml-1.5">ngưng</span>}
                      </td>
                      <td className="num">{V4l7(p.docs)}</td>
                      <td className="num">
                        <BocTachCell c={p} />
                      </td>
                      <td className="num">
                        <PdfaCell c={p} />
                      </td>
                      <td className={`num${p.approved > 0 ? " text-success-700" : ""}`}>
                        {V4l7(p.approved)}
                      </td>
                      <td className={`num${p.rejected > 0 ? " text-danger-700" : ""}`}>
                        {V4l7(p.rejected)}
                      </td>
                      <td className={`num${p.pending > 0 ? " text-warning-700" : ""}`}>
                        {V4l7(p.pending)}
                      </td>
                      <td>
                        <div
                          className="progress min-w-22.5 mb-1"
                          title={`${p.approved}/${p.docs}IHTDoGkgbGnhu4d1IMSRw6MgZHV54buHdA==`}
                        >
                          <span style={{ width: `${Math.min(100, V4hi(p))}%` }} />
                        </div>
                        <span className="text-sm">{V4hj(p)}</span>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
              {totals && report.projects.length > 1 && (
                <tfoot>
                  <tr className="font-semibold">
                    <td />
                    <td>Tổng cộng</td>
                    <td className="num">{V4l7(totals.docs)}</td>
                    <td className="num">
                      <BocTachCell c={totals} />
                    </td>
                    <td className="num">
                      <PdfaCell c={totals} />
                    </td>
                    <td className="num">{V4l7(totals.approved)}</td>
                    <td className="num">{V4l7(totals.rejected)}</td>
                    <td className="num">{V4l7(totals.pending)}</td>
                    <td>{V4hj(totals)}</td>
                  </tr>
                </tfoot>
              )}
            </table>
          </div>
        ) : (
          <p className="muted pt-0 px-4 pb-4">
            {V1tm
              ? "xJBhbmcgdMOtbmggc+G7kSBsaeG7h3XigKY="
              : "Q2jGsGEgY8OzIGThu7Egw6FuIG7DoG8gdHJvbmcgcGjhuqFtIHZpIHBo4bulIHRyw6FjaC4="}
          </p>
        )}
      </div>

      <Banner
        tone="info"
        text="Q2jhu41uIHTDoGkgbGnhu4d1IMSRw6Mgc+G7kSBob8OhIMSR4buDIGLDs2MgdMOhY2ggdHLGsOG7nW5nOiBz4butYSBs4bqhaSBraHVuZyBtw6F5IGLDs2MsIGhv4bq3YyBjaOG7jW4gdHLGsOG7nW5nIHLhu5NpIHbhur0gw7QgY2hvIHBo4bqnbiBtw6F5IGPDsm4gdGhp4bq/dSwgcuG7k2kgbMawdSBsw6BtIGThu68gbGnhu4d1IGNodeG6qW4uIEhv4bq3YyDigJxU4bqjaSBQREYgbMOqbuKAnSDEkeG7gyBnw6FuIG5ow6NuIHRyw6puIHTDoGkgbGnhu4d1IHThu7EgY2jhu41uLg=="
      />

      {V1th && <V4l9 rows={V1th} onClose={() => V1ti(null)} />}

      {}
      <V4k5 />

      {}
      <ReviewHoSoTable
        canReview={V1ts}
        onlyProject={V1to}
        onOpenDoc={(f) => V1ta({ ...f, reviewStatus: f.review })}
        onDone={() => void V1tu()}
        actions={
          <>
            {!V1tq && V1t5.length > 0 && (
              <select
                className="inp sm max-w-55"
                value={V1t7}
                onChange={(V1uf) => V1t8(V1uf.target.value)}
                title="ROG7sSDDoW4gxJHDrWNoIMSR4buDIHThuqNpIHTDoGkgbGnhu4d1IGzDqm4="
              >
                <option value="">— Chọn dự án —</option>
                {V1t5.map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.code} · {p.name}
                  </option>
                ))}
              </select>
            )}
            <button
              className="btn sm"
              onClick={() => void V1u4()}
              disabled={V1tb}
              title="xJDGsGEgbeG7jWkgdMOgaSBsaeG7h3UgxJHDoyDEkeG7k25nIGLhu5kgKGNoxrBhIE9DUikgdsOgbyBow6BuZyDEkeG7o2kgT0NSIHRyw6puIG3DoXkgY2jhu6c="
            >
              {V1tb ? (
                "xJBhbmcgeOG6v3AgaMOgbmfigKY="
              ) : (
                <>
                  <Icon name="chay" />
                  OCR toàn bộ
                </>
              )}
            </button>
            <button
              className="btn sm"
              onClick={() => void V1u7()}
              disabled={V1td}
              title="S2nhu4NtIHRyYSBQREYgMiBs4bubcCAoUERGIEEgbMawdSB0cuG7rykgxJHDoyBzaW5oICsgxJHDum5nIGPhuqV1IHRyw7pj"
            >
              {V1td ? (
                "xJBhbmcga2nhu4Nt4oCm"
              ) : (
                <>
                  <Icon name="kiemTraPdfa" />
                  Kiểm tra PDF A
                </>
              )}
            </button>
            <button
              className="btn sm"
              onClick={() => void V1u9()}
              disabled={V1tf}
              title="R2jDqXAgbOG6oWkgY8OhYyB0w6BpIGxp4buHdSBjw7MgUERGIDIgbOG7m3Agc2luaCBi4bufaSBi4bqjbiBjxakgKGNoxrBhIMSR4bqhdCBQREYvQSB0aGVvIMSQaeG7gXUgOC4yLmEp"
            >
              {V1tf ? (
                "xJBhbmcgeOG6v3AgaMOgbmfigKY="
              ) : (
                <>
                  <Icon name="ghepLai" />
                  Ghép lại PDF/A
                </>
              )}
            </button>
            <button
              className="btn sm primary"
              onClick={() => void V1tz()}
              disabled={V1sz}
              title="Q2jhu41uIG3hu5l0IFBERiB0csOqbiBtw6F5LCB04bqjaSBsw6puIG3DoXkgY2jhu6cgcuG7k2kgZ8OhbiBuaMOjbg=="
            >
              {V1sz ? (
                "xJBhbmcgdOG6o2kgbMOqbuKApg=="
              ) : (
                <>
                  <Icon name="taiLen" />
                  Tải PDF lên
                </>
              )}
            </button>
          </>
        }
      />
    </>
  );
}
