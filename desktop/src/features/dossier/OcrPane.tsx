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
  ok: { label: "Đạt", tone: "ok" },
  invalid: { label: "Cấu trúc sai", tone: "err" },
  missing: { label: "Thiếu PDF", tone: "warn" },
  pending: { label: "Chưa OCR xong", tone: "gray" },
};

function BocTachCell({ c }: { c: V4f0 }) {
  return (
    <>
      <div
        title={
          c.ocrPages ? `${V4l7(c.ocrPages)} trang đã có kết quả OCR` : undefined
        }
      >
        {V4l7(c.ocrDone)}
      </div>
      {c.fieldDocs != null && c.fieldDocs > 0 && (
        <div
          className="muted text-sm"
          title="Tài liệu đã bóc 16 trường mục lục (nút ② Bóc tách 16 trường)"
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
        title="Máy chủ đang chạy bản cũ, chưa báo số văn bản đã chuyển PDF/A"
      >
        —
      </span>
    );
  }
  return (
    <>
      <div
        title={`Đã chuyển đổi ${t.docs}/${t.total} văn bản sang PDF/A lưu trữ (${t.percent}%). Bản ghép sinh ra ở nút "PDF 2 lớp" màn Máy chủ — OCR xong chưa có nghĩa là đã có tệp PDF/A.`}
      >
        {V4l7(t.docs)}
        <span className="muted text-sm">/{V4l7(t.total)}</span>
      </div>
      {t.old > 0 && (
        <div
          className="text-sm text-warning-700"
          title="Ghép bởi bản cũ, thiếu dấu hiệu PDF/A-2b — bấm “Ghép lại PDF/A” để đạt Điều 8.2.a"
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
            .join(" · ") || "không có tài liệu"}
          {V1so.some((r) => (r.state === "ok" || r.state === "invalid") && !r.pdfa2b) &&
            ` · chưa PDF/A: ${V1so.filter((r) => (r.state === "ok" || r.state === "invalid") && !r.pdfa2b).length}`}
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
                  title="Số trang không có chữ trong lớp text"
                >
                  Không chữ
                </th>
                <th title="Đủ dấu hiệu PDF/A-2b (XMP pdfaid + OutputIntent + /ID + font nhúng)">
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
                          title={`${r.pagesWithText} trang có chữ`}
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
                              ? "Đủ dấu hiệu PDF/A-2b"
                              : "Ghép bởi bản cũ — bấm “Ghép lại PDF/A” để đạt Điều 8.2.a"
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
          ? "Hãy chọn dự án đích ở ô bên cạnh trước khi tải tài liệu lên."
          : "Tài khoản chưa gắn dự án nào — không thể tải tài liệu lên.",
      );
      return;
    }
    let V1u0: string | null;
    try {
      const r = await open({
        multiple: false,
        title: "Chọn PDF để gán nhãn",
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
          ? `"${V1u2.filename}" đã có sẵn trên máy chủ.`
          : `Đã tải "${V1u2.filename}" lên máy chủ.`,
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
            ? `Không có tài liệu nào để OCR (bỏ qua ${r.skippedNoPageCount} tài liệu chưa rõ số trang).`
            : "Mọi tài liệu đã được đưa vào OCR trước đó — không có gì mới.",
        );
      } else {
        const V1u5 =
          r.skippedNoPageCount > 0
            ? ` (bỏ qua ${r.skippedNoPageCount} chưa rõ số trang)`
            : "";
        V1t4(
          `Đã đưa ${r.documents} tài liệu (${r.pages} trang) vào hàng đợi OCR${V1u5}.`,
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
          ? ` ${r.skippedSigned} tài liệu ĐÃ KÝ chưa đạt PDF/A — phải ký lại theo quy trình.`
          : "";
      V1t4(
        `Đã kiểm ${r.checked} tài liệu, xếp hàng ghép lại ${r.enqueued}.${V1ua}`,
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
            label="Tổng số dự án"
            value={report?.projectsCount ?? 0}
            title="Số dự án trong phạm vi phụ trách"
          />
          <V4la
            label="Tổng số file"
            value={totals?.docs ?? 0}
            title={`${V4l7(totals?.pages ?? 0)} trang`}
          />
          <V4la
            label="File đã OCR"
            value={totals?.ocrDone ?? 0}
            title={`Tài liệu đã có kết quả OCR đủ số trang (nút ① PDF 2 lớp)${
              totals?.ocrPages ? ` · ${V4l7(totals.ocrPages)}IHRyYW5nIMSRw6MgT0NS` : ""
            }${
              totals?.fieldDocs
                ? ` · ${V4l7(totals.fieldDocs)}IHTDoGkgbGnhu4d1IMSRw6MgYsOzYyAxNiB0csaw4budbmcgKOKRoSk=`
                : ""
            }`}
          />
          <V4la
            label="Đã duyệt (admin)"
            value={totals?.approved ?? 0}
            tone="ok"
            title="Admin đã xác nhận kết quả bóc tách"
          />
          <V4la
            label="Từ chối"
            value={totals?.rejected ?? 0}
            tone="err"
            title="Admin trả lại — phải quét/bóc lại"
          />
          <V4la
            label="Chờ xác nhận"
            value={totals?.pending ?? 0}
            tone="warn"
            title="Chưa có quyết định duyệt (gồm cả tài liệu máy chưa OCR xong)"
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
            title="Ngày bắt đầu"
          />
          <span className="muted">—</span>
          <input
            className="inp sm"
            type="date"
            value={V1tj.to}
            min={V1tj.from || undefined}
            onChange={(V1ud) => V1tk((r) => ({ ...r, to: V1ud.target.value }))}
            title="Ngày kết thúc (tính trọn ngày này)"
          />
          <button
            className="btn sm"
            onClick={() => V1tk({ from: "", to: "" })}
            title="Bỏ lọc, xem toàn bộ"
          >
            Toàn bộ
          </button>
          <button
            className="btn sm"
            onClick={() => V1tk(V4hg())}
            title="Từ đầu tháng đến hôm nay"
          >
            Tháng này
          </button>
          {V1tt ? (
            <span className="text-danger-700 text-xs">{V1tt}</span>
          ) : (
            <span className="muted text-sm">
              {V1tm
                ? "Đang tính số liệu…"
                : V1tj.from || V1tj.to
                  ? `Đang xem: ${V1tj.from || "đầu kỳ"} → ${V1tj.to || "nay"}`
                  : "Đang xem: toàn bộ"}
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
                    title="Trên: tài liệu đã OCR đủ số trang (nút ① PDF 2 lớp, đếm theo ocr_results). Dưới: tài liệu đã bóc 16 trường (nút ② VLM, đếm theo ocr_fields) — hai việc khác nhau nên đếm riêng."
                  >
                    Đã bóc tách
                  </th>
                  <th
                    className="num"
                    title="Số văn bản đã chuyển đổi sang PDF/A lưu trữ trên tổng số tài liệu (vết ghi khi kho đã nhận bản ghép). Đây là việc của nút “PDF 2 lớp” màn Máy chủ — OCR xong chưa có nghĩa là đã có tệp PDF/A."
                  >
                    Đã PDF A
                  </th>
                  <th className="num">Đã duyệt</th>
                  <th className="num">Từ chối</th>
                  <th className="num">Chờ xác nhận</th>
                  <th title="Tài liệu đã duyệt / tổng số tài liệu">
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
                          ? "Bấm lại để bỏ lọc"
                          : `Chỉ xem tài liệu của ${p.name}`
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
                          title={`${p.approved}/${p.docs} tài liệu đã duyệt`}
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
              ? "Đang tính số liệu…"
              : "Chưa có dự án nào trong phạm vi phụ trách."}
          </p>
        )}
      </div>

      <Banner
        tone="info"
        text="Chọn tài liệu đã số hoá để bóc tách trường: sửa lại khung máy bóc, hoặc chọn trường rồi vẽ ô cho phần máy còn thiếu, rồi lưu làm dữ liệu chuẩn. Hoặc “Tải PDF lên” để gán nhãn trên tài liệu tự chọn."
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
                title="Dự án đích để tải tài liệu lên"
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
              title="Đưa mọi tài liệu đã đồng bộ (chưa OCR) vào hàng đợi OCR trên máy chủ"
            >
              {V1tb ? (
                "Đang xếp hàng…"
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
              title="Kiểm tra PDF 2 lớp (PDF A lưu trữ) đã sinh + đúng cấu trúc"
            >
              {V1td ? (
                "Đang kiểm…"
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
              title="Ghép lại các tài liệu có PDF 2 lớp sinh bởi bản cũ (chưa đạt PDF/A theo Điều 8.2.a)"
            >
              {V1tf ? (
                "Đang xếp hàng…"
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
              title="Chọn một PDF trên máy, tải lên máy chủ rồi gán nhãn"
            >
              {V1sz ? (
                "Đang tải lên…"
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
