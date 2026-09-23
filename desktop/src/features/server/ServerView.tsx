// Copyright (c) 2026 DigiText. All rights reserved.

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { V41p } from "@/live/useLive";
import type { ReactNode } from "react";
import { useNavigation } from "@/state";
import { isTauri, V3xq } from "@/api/ipc";
import { V3zt } from "@/state/sessionCore";
import { V4el, V4fc, V4ek, type V4f9, type V4ej } from "@/api/docs";
import {
  V4vz,
  V4vq,
  V4vt,
  V4vj,
  V4vi,
  V4w1,
  V4vv,
  V4w2,
  V4vw,
  V4vy,
  type V4vu,
  type V4vl,
  type V4vx,
} from "./serverTree";
import { V4w5, V4w9 } from "./pdfa";
import { V4k1 } from "./progress";
import "./server.css";
import { Icon } from "@/components/Icon";
import { open as openDialog } from "@tauri-apps/plugin-dialog";

// Thư mục lưu lần trước — chỉ là tiện ích nhớ, hỏng cũng không sao.
const DL_DIR_KEY = "server.download.dir";
function nhoThuMuc(): string | undefined {
  try {
    return localStorage.getItem(DL_DIR_KEY) ?? undefined;
  } catch {
    return undefined;
  }
}
function luuThuMuc(d: string): void {
  try {
    localStorage.setItem(DL_DIR_KEY, d);
  } catch {
    /* bỏ qua */
  }
}

const V4wa = (V33e: unknown) =>
  V3zt(
    V33e instanceof V3xq ? V33e.code : "error",
    V33e instanceof Error ? V33e.message : String(V33e),
  );

const V4wb = (b: number) => (b / 1_048_576).toFixed(1);
const V4wc = (n: number) => n.toLocaleString("vi-VN");

function V4wd(status: string): string {
  if (status === "signed" || status === "archived") return "ok";
  if (status === "sign_rejected") return "err";
  if (status === "to_sign" || status === "signing" || status === "ocr_done") return "info";
  if (status === "disposed") return "gray";
  return "warn";
}

const V4we = 18;

const V4wf = 6000;

function V4wg({
  state,
  onToggle: V33f,
  title: V33g,
}: {
  state: V4vx;
  onToggle: () => void;
  title: string;
}) {
  return (
    <input
      type="checkbox"
      className="srv-tick"
      aria-label={V33g}
      title={V33g}
      checked={state === "all"}
      ref={(V33h) => {
        if (V33h) V33h.indeterminate = state === "some";
      }}
      onChange={V33f}
      onClick={(V33i) => V33i.stopPropagation()}
    />
  );
}

export default function V4wh() {
  const { serverReveal: V33j, clearServerReveal: V33k } = useNavigation();
  const [V33l, V33m] = useState<{ relPath: string; filename: string } | null>(null);
  const [V33n, V33o] = useState<{ folderPath: string; filename: string } | null>(null);
  const V33p = useRef<HTMLTableRowElement | null>(null);
  const [tree, V33q] = useState<V4vl | null>(null);
  const [open, V33r] = useState<Set<string>>(new Set());
  const [V33s, V33t] = useState<Set<string>>(new Set());
  const [V33u, V33v] = useState(false);
  const [V33w, V33x] = useState<V4f9[] | null>(null);
  const [V33y, V33z] = useState(false);
  const [V340, V341] = useState<{ done: number; total: number } | null>(null);
  const V342 = useRef(false);
  const [V343, V344] = useState("");
  // Tải bản PDF/A (lớp chữ, về Kho/_pdfa) thay vì bản gốc đã tải lên (về Kho/_nhan_ve).
  const [taiPdfa, setTaiPdfa] = useState(false);

  const V345 = useRef(false);

  const V346 = useCallback(async () => {
    if (!isTauri()) return;
    V33v(true);
    try {
      const V347 = V4vq(await V4ek());
      V33q(V347);
      if (!V345.current) {
        V345.current = true;
        V33r(V4vt(V347));
      }
    } catch (V348) {
      V344(V4wa(V348));
    } finally {
      V33v(false);
    }
  }, []);

  V41p(V346, { topics: ["docs", "ocr", "sign", "uploadBatches"] });

  const V349 = useMemo<V4vu[]>(() => (tree ? V4vv(tree) : []), [tree]);
  const V34a = useMemo<Map<string, string[]>>(() => (tree ? V4vw(tree) : new Map()), [tree]);
  const V34b = useMemo<V4ej[]>(() => V349.map((f) => f.doc), [V349]);

  const V34c = useMemo(() => (V33w ? V4w9(V33w, V34b) : []), [V33w, V34b]);
  const V34d = useMemo(() => new Set(V34c.map((p) => p.id)), [V34c]);

  useEffect(() => {
    if (!V33j) return;
    V33m(V33j);
    V33k();
  }, [V33j, V33k]);

  useEffect(() => {
    if (!V33l || !tree) return;
    const V34e = V4w1(tree, V33l);
    if (V34e === null) {
      V344(
        `Chưa thấy “${V33l.filename}” trong kho máy chủ (có thể chưa đồng bộ xong).`,
      );
    } else {
      V33r((V34f) => new Set([...V34f, ...V4vz(V34e)]));

      V345.current = true;
      V33o({ folderPath: V34e, filename: V33l.filename });
    }
    V33m(null);
  }, [V33l, tree]);

  useEffect(() => {
    if (!V33n) return;
    V33p.current?.scrollIntoView({ block: "center", behavior: "smooth" });
    const t = setTimeout(() => V33o(null), V4wf);
    return () => clearTimeout(t);
  }, [V33n, tree]);

  const V34g = (path: string) =>
    V33r((V34h) => {
      const V34i = new Set(V34h);
      if (V34i.has(path)) V34i.delete(path);
      else V34i.add(path);
      return V34i;
    });

  const V34j = (V34k: readonly string[]) =>
    V33t((V34l) => {
      const V34m = new Set(V34l);
      if (V4vy(V34k, V34l) === "all") for (const V34n of V34k) V34m.delete(V34n);
      else for (const V34o of V34k) V34m.add(V34o);
      return V34m;
    });

  const V34p = useMemo(() => V349.filter((f) => V33s.has(f.doc.id)), [V349, V33s]);
  const V34q = V34p.reduce((n, f) => n + (f.doc.pageCount ?? 0), 0);

  const V34r = async (V34s: V4vu[], V34t: string) => {
    if (V34s.length === 0) return;
    // Người dùng CHỌN thư mục lưu mỗi lần tải — không mặc định về Documents\DigiText nữa.
    let destDir: string;
    try {
      const r = await openDialog({
        directory: true,
        multiple: false,
        defaultPath: nhoThuMuc(),
        title: `Chọn thư mục lưu ${V4wc(V34s.length)} tệp ${taiPdfa ? "PDF/A" : "bản gốc"} (giữ nguyên cây thư mục)`,
      });
      if (typeof r !== "string" || !r) return;
      destDir = r;
    } catch (e) {
      V344(`Không mở được hộp chọn thư mục: ${V4wa(e)}`);
      return;
    }
    luuThuMuc(destDir);
    V342.current = false;
    V341({ done: 0, total: V34s.length });
    V344("");
    let V34u = 0;
    let last = "";
    let V34v = "";
    let chuaPdfa = 0;
    for (const [i, V34w] of V34s.entries()) {
      if (V342.current) break;
      V341({ done: i, total: V34s.length });
      if (taiPdfa && !(V34w.doc.pdfaPages ?? 0)) {
        chuaPdfa++;
        continue;
      }
      try {
        last = await V4el(V34w.doc.id, V34w.doc.filename, V34w.folderPath, taiPdfa, destDir);
        V34u++;
      } catch (V34x) {
        if (V34v === "") V34v = V4wa(V34x);
      }
    }
    V341(null);
    const fail = V34s.length - V34u - chuaPdfa;
    const dung = V342.current ? " (đã dừng giữa chừng)" : "";
    V344(
      `Đã tải ${V4wc(V34u)}/${V4wc(V34s.length)} tệp ${taiPdfa ? "PDF/A " : ""}${V34t}${dung}.` +
        (V34u > 0 ? ` Lưu tại: ${last}` : "") +
        (chuaPdfa > 0 ? ` — ${V4wc(chuaPdfa)} tệp chưa có bản PDF/A (chạy “PDF 2 lớp” trước)` : "") +
        (fail > 0 ? ` — ${V4wc(fail)} tệp lỗi: ${V34v}` : ""),
    );
  };

  const V34y = async () => {
    V33z(true);
    V344("");
    try {
      const V34z = await V4fc();
      V33x(V34z);
      const n = V4w9(V34z, V34b).length;
      if (n === 0)
        V344(
          `Đã soát ${V4wc(V34z.length)} tài liệu — không tệp nào lỗi PDF/A.`,
        );
    } catch (V350) {
      V344(V4wa(V350));
    } finally {
      V33z(false);
    }
  };

  const V351 = (V352: string, V353: string) => {
    V33r((V354) => new Set([...V354, ...V4vz(V352)]));
    V345.current = true;
    V33o({ folderPath: V352, filename: V353 });
  };

  const V355: ReactNode[] = [];
  if (tree) {
    const V356 = (d: V4ej, V357: number, V358: string) => {
      const V359 = V4w2(V358, d.filename, V33n);
      const V35a = V34d.has(d.id);
      V355.push(
        <tr key={d.id} className={V359 ? "srv-hit" : undefined} ref={V359 ? V33p : undefined}>
          <td className="srv-tickcell">
            <V4wg
              state={V33s.has(d.id) ? "all" : "none"}
              onToggle={() => V34j([d.id])}
              title={`Chọn ${d.filename}`}
            />
          </td>
          <td>
            <span className="srv-name" style={{ paddingLeft: V357 * V4we + V4we }}>
              <Icon name="taiLieu" /> {d.filename}
              <span className={`badge ${V4wd(d.status)}`}>{V4k1(d.status)}</span>
              {V35a && (
                <span
                  className="badge err"
                  title="Xem bảng “Tệp PDF/A lỗi” phía trên"
                >
                  PDF/A lỗi
                </span>
              )}
              <span className="srv-size">{V4wb(d.sizeBytes)} MB</span>
            </span>
          </td>
          <td className="num srv-count">—</td>
          <td className="num srv-count">—</td>
          <td className="num srv-count">—</td>
          <td className="num">{d.pageCount ?? "—"}</td>
          <td>
            <button
              className="btn"
              disabled={V340 !== null}
              onClick={() => void V34r([{ doc: d, folderPath: V358 }], `“${d.filename}”`)}
              title={taiPdfa ? "Tải bản PDF/A" : "Tải bản gốc"}
            >
              <Icon name="taiVe" />
              {taiPdfa ? "Tải PDF/A" : "Tải"}
            </button>
          </td>
        </tr>,
      );
    };
    const V35b = (f: V4vl, V35c: number) => {
      for (const V35d of f.folders) {
        const V35e = open.has(V35d.path);
        const V35f = V34a.get(V35d.path) ?? [];
        V355.push(
          <tr key={`f:${V35d.path}`} className="srv-folder">
            <td className="srv-tickcell">
              <V4wg
                state={V4vy(V35f, V33s)}
                onToggle={() => V34j(V35f)}
                title={`Chọn cả ${V35d.name} (${V4wc(V35f.length)} tệp)`}
              />
            </td>
            <td onClick={() => V34g(V35d.path)}>
              <span className="srv-name" style={{ paddingLeft: V35c * V4we }}>
                <span className="srv-caret" aria-hidden>
                  <Icon name={V35e ? "xuong" : "moNhanh"} size={13} />
                </span>
                <Icon name={V35d.depth === 0 ? "hoSo" : "thuMuc"} size={15} /> {V35d.name}
              </span>
            </td>
            <td className="num srv-count">{V35d.boxCount ?? "—"}</td>
            <td className="num srv-count">{V35d.dossierCount ?? "—"}</td>
            <td className="num srv-count">{V4wc(V35d.docCount)}</td>
            <td className="num srv-count">{V4wc(V35d.pageCount)}</td>
            <td>
              <button
                className="btn"
                disabled={V340 !== null || V35d.docCount === 0}
                title={`Tải cả ${V4wc(V35d.docCount)} tệp trong “${V35d.name}”, giữ nguyên cây thư mục`}
                onClick={() => void V34r(V4vv(V35d), `trong “${V35d.name}”`)}
              >
                <Icon name="taiVe" />
                Tải cả nhánh
              </button>
            </td>
          </tr>,
        );
        if (V35e) V35b(V35d, V35c + 1);
      }
      for (const d of f.docs) V356(d, V35c, f.path);
    };
    V35b(tree, 0);
  }

  const V35g = V34a.get("") ?? [];

  return (
    <section className="view active">
      <div className="page-head">
        <h2>Máy chủ</h2>
        <div className="sub">
          Kho dữ liệu đã đẩy lên máy chủ — xem theo dự án, tải xuống, soát tệp PDF/A lỗi. Chạy OCR /
          PDF 2 lớp / Bóc tách 16 trường ở mục “Số hoá › OCR”.
        </div>
      </div>

      {V343 && (
        <div className="card srv-msg mb-3">
          <span>{V343}</span>
          <button className="btn sm" onClick={() => V344("")}>
            <Icon name="dong" size={14} label="Đóng thông báo" />
          </button>
        </div>
      )}

      {V33w && V34c.length > 0 && (
        <div className="card p-0 overflow-hidden mb-4">
          <div className="srv-bar">
            <h3 className="m-0">Tệp PDF/A lỗi ({V4wc(V34c.length)})</h3>
            <span className="muted text-sm">
              đã soát {V4wc(V33w.length)} tài liệu (máy chủ chỉ soát tối đa 1.000 tài liệu mỗi lượt)
              — sửa bằng “Ghép lại PDF/A” ở mục Số hoá › OCR
            </span>
            <div className="flex-1" />
            <button className="btn" onClick={() => V33x(null)}>
              <Icon name="dong" />
              Đóng
            </button>
          </div>
          <div className="tbl-scroll max-h-65">
            <table className="tbl">
              <thead>
                <tr>
                  <th>Lỗi</th>
                  <th>Đường dẫn</th>
                  <th>Lý do</th>
                  <th />
                </tr>
              </thead>
              <tbody>
                {V34c.map((p) => (
                  <tr key={p.id}>
                    <td>
                      <span className={`badge ${p.kind === "not-pdfa" ? "warn" : "err"}`}>
                        {V4w5(p.kind)}
                      </span>
                    </td>
                    <td className="srv-path" title={p.path}>
                      {p.path}
                    </td>
                    <td className="muted text-sm">{p.reason}</td>
                    <td>
                      <button
                        className="btn sm"
                        disabled={p.folderPath === ""}
                        onClick={() => V351(p.folderPath, p.filename)}
                        title="Mở đúng nhánh chứa tệp trong cây bên dưới"
                      >
                        Đi tới
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      <div className="card p-0 overflow-hidden">
        <div className="srv-bar">
          <h3 className="m-0">Kho dữ liệu máy chủ</h3>
          <span className="muted text-sm">
            {V4wc(tree?.docCount ?? 0)} tệp · {V4wc(tree?.pageCount ?? 0)} trang ·{" "}
            {V4wb(tree?.sizeBytes ?? 0)} MB
            {}
            {V34p.length > 0
              ? ` — đã chọn ${V4wc(V34p.length)} tệp (${V4wc(V34q)} trang)`
              : ""}
          </span>
          <div className="flex-1" />
          {V340 ? (
            <>
              <span className="muted text-sm">
                Đang tải {V4wc(V340.done + 1)}/{V4wc(V340.total)}…
              </span>
              <button className="btn" onClick={() => (V342.current = true)}>
                <Icon name="dungHan" />
                Dừng
              </button>
            </>
          ) : (
            <>
              <button
                className="btn primary"
                disabled={V34p.length === 0}
                onClick={() => void V34r(V34p, "đã chọn")}
                title="Tải mọi tệp đã tích về thư mục anh chọn, giữ nguyên cây thư mục"
              >
                <Icon name="taiVe" />
                Tải xuống ({V4wc(V34p.length)})
              </button>
              <label
                className="flex items-center gap-1 text-sm"
                title="Bật: tải bản PDF/A có lớp chữ. Tắt: tải bản gốc đã tải lên. Thư mục lưu được chọn khi bấm tải."
              >
                <input
                  type="checkbox"
                  checked={taiPdfa}
                  onChange={(e) => setTaiPdfa(e.target.checked)}
                />
                Bản PDF/A
              </label>
              <button
                className="btn"
                onClick={() => void V34y()}
                disabled={V33y}
                title="Soát PDF 2 lớp trên máy chủ, liệt kê tệp lỗi kèm đường dẫn"
              >
                {V33y ? (
                  "Đang soát…"
                ) : (
                  <>
                    <Icon name="kiemTraPdfa" />
                    Soát PDF/A lỗi
                  </>
                )}
              </button>
              <button className="btn" onClick={() => void V346()} disabled={V33u}>
                <Icon name="lamMoi" />
                Làm mới
              </button>
            </>
          )}
        </div>
        {!tree || tree.docCount === 0 ? (
          <p className="muted p-4">
            {V33u
              ? "Đang tải kho máy chủ…"
              : "Chưa có tài liệu nào trên máy chủ. Dữ liệu lên máy chủ qua màn “Đồng bộ dữ liệu” (máy quản trị thu thập rồi đẩy lên)."}
          </p>
        ) : (
          <div className="tbl-scroll">
            <table className="tbl srv-tree">
              <thead>
                <tr>
                  <th className="srv-tickcell">
                    <V4wg
                      state={V4vy(V35g, V33s)}
                      onToggle={() => V34j(V35g)}
                      title="Chọn tất cả"
                    />
                  </th>
                  <th>Dự án</th>
                  <th className="num" title={`Số hộp (cấp ${V4vi + 1} của cây)`}>
                    Số hộp
                  </th>
                  <th
                    className="num"
                    title={`Số hồ sơ (cấp ${V4vj + 1} của cây)`}
                  >
                    Số hồ sơ
                  </th>
                  <th className="num">Số file</th>
                  <th className="num">Số trang</th>
                  <th>Tải xuống</th>
                </tr>
              </thead>
              <tbody>{V355}</tbody>
            </table>
          </div>
        )}
      </div>
    </section>
  );
}
