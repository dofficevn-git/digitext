// Copyright (c) 2026 DigiText. All rights reserved.
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { CSSProperties } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { V460 } from "@/state";
import type { V3ye, V3yf, V3yc } from "@/domain/types";
import { V477, V47a, V478, V47b, V479 } from "@/api/scanner";
import { V40i, V40s, V40r } from "@/api/archive";
import V4oq from "@/ui/Seg";
import { V4ov } from "./tone";
import { Icon } from "@/components/Icon";
import type { V41v } from "@/components/Icon";

interface V4ow {
  x: number;
  y: number;
  w: number;
  h: number;
}
type V4ox = "tl" | "tr" | "bl" | "br";
type V4oy =
  | { mode: "move"; sx: number; sy: number; x: number; y: number }
  | {
      mode: "resize";
      c: V4ox;
      sx: number;
      sy: number;
      x: number;
      y: number;
      w: number;
      h: number;
    }
  | { mode: "pan"; sx: number; sy: number; x: number; y: number };

const clamp = (v: number, lo: number, hi: number) => Math.max(lo, Math.min(hi, v));
const V4oz: V3yc[] = [];

const V4p0 = (d: number) => {
  const m = ((((d + 180) % 360) + 360) % 360) - 180;
  return m === -180 ? 180 : m;
};

interface V4p1 {
  pages?: V3yc[];
  startIndex?: number;
  onClose?: () => void;

  onApply?: (V4p2: number, V4p3: V3yf) => void;
}

export default function V4p4({
  pages: V23e,
  startIndex: V23f,
  onClose: V23g,
  onApply: V23h,
}: V4p1 = {}) {
  const {
    goto,
    editSession: V23i,
    editPdfOut: V23j,
    reloadArchive: V23k,
    setEditorDirty: V23l,
  } = V460();
  const V23m = !!V23g;

  const V23n = !V23m && !!V23i;

  const [V23o, V23p] = useState<V3yc[]>(V23i?.pages ?? []);
  useEffect(() => {
    V23p(V23i?.pages ?? []);
    V23l(false);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [V23i?.batchId]);

  useEffect(() => () => V23l(false), [V23l]);
  const V23q = useMemo(() => V23e ?? (V23n ? V23o : V4oz), [V23e, V23n, V23o]);

  const [V23r, V23s] = useState(V23f ?? V23q[0]?.index ?? 1);

  useEffect(() => {
    if (V23q.length && !V23q.some((p) => p.index === V23r)) V23s(V23q[0].index);
  }, [V23q, V23r]);

  const [V23t, V23u] = useState(0);
  const [V23v, V23w] = useState(0);
  const [V23x, V23y] = useState(false);
  const [V23z, V240] = useState(false);
  const [V241, V242] = useState(100);
  const [V243, V244] = useState({ x: 0, y: 0 });
  const [V245, V246] = useState<V4ow | null>(null);
  const [V247, V248] = useState<"color" | "gray" | "bw">("color");

  const [V249, V24a] = useState<"keep" | "300" | "600">("keep");
  const [V24b, V24c] = useState<"this" | "all">("this");

  const [V24d, V24e] = useState(0);
  const [V24f, V24g] = useState(0);
  const [V24h, V24i] = useState(100);
  const [V24j, V24k] = useState(128);
  const [V24l, V24m] = useState(false);
  const [V24n, V24o] = useState(false);
  const [V24p, V24q] = useState(false);
  const [V24r, V24s] = useState<string | null>(null);
  const [V24t, V24u] = useState(false);
  const [V24v, V24w] = useState(false);

  const V24x = useRef<HTMLDivElement>(null);
  const V24y = useRef<V4oy | null>(null);

  const V24z = useRef(new Map<number, number>());

  const V250 = V23q.length;
  const V251 = V23q.findIndex((p) => p.index === V23r);
  const V252 = V23q.find((p) => p.index === V23r);
  const V253 = V252?.preview || V252?.thumbnail || "";
  const V254 = V252?.sourcePath ?? "";
  const V255 = V23m || V23n;

  useEffect(() => {
    for (const p of V23q) {
      if (!V24z.current.has(p.index)) V24z.current.set(p.index, p.dpi);
    }
  }, [V23q]);

  useEffect(() => {
    const V256 = (V257: PointerEvent) => {
      const d = V24y.current;
      const V258 = V24x.current;
      if (!d || !V258) return;
      const s = V241 / 100;
      const V259 = (V257.clientX - d.sx) / s;
      const V25a = (V257.clientY - d.sy) / s;
      if (d.mode === "pan") {
        V244({ x: d.x + (V257.clientX - d.sx), y: d.y + (V257.clientY - d.sy) });
        return;
      }
      const V25b = V258.clientWidth;
      const H = V258.clientHeight;

      V246((V25c) => {
        if (!V25c) return V25c;
        if (d.mode === "move") {
          return {
            x: clamp(d.x + V259, 0, V25b - V25c.w),
            y: clamp(d.y + V25a, 0, H - V25c.h),
            w: V25c.w,
            h: V25c.h,
          };
        }
        let x = d.x;
        let y = d.y;
        let w = d.w;
        let h = d.h;
        if (d.c.includes("l")) {
          x = d.x + V259;
          w = d.w - V259;
        }
        if (d.c.includes("r")) w = d.w + V259;
        if (d.c.includes("t")) {
          y = d.y + V25a;
          h = d.h - V25a;
        }
        if (d.c.includes("b")) h = d.h + V25a;
        if (w > 40 && h > 40 && x >= 0 && y >= 0 && x + w <= V25b && y + h <= H)
          return { x, y, w, h };
        return V25c;
      });
    };
    const V25d = () => {
      V24y.current = null;
    };
    window.addEventListener("pointermove", V256);
    window.addEventListener("pointerup", V25d);
    return () => {
      window.removeEventListener("pointermove", V256);
      window.removeEventListener("pointerup", V25d);
    };
  }, [V241]);

  useEffect(() => {
    if (!V255 || !V254) {
      V24u(false);
      V24w(false);
      return;
    }
    let V25e = true;
    V47b(V254)
      .then((s) => {
        if (V25e) {
          V24u(s.canUndo);
          V24w(s.canRedo);
        }
      })
      .catch(() => {
        if (V25e) {
          V24u(false);
          V24w(false);
        }
      });
    return () => {
      V25e = false;
    };
  }, [V254, V255]);

  const V25f = () => {
    const V25g = V24x.current;
    if (!V25g || !V252 || V252.widthPx <= 0 || V252.heightPx <= 0) return null;
    const V25h = V25g.clientWidth;
    const H = V25g.clientHeight;
    const scale = Math.min(V25h / V252.widthPx, H / V252.heightPx);
    const w = V252.widthPx * scale;
    const h = V252.heightPx * scale;
    return { x: (V25h - w) / 2, y: (H - h) / 2, w, h };
  };

  const V25i = () => {
    if (V245) {
      V246(null);
      return;
    }

    const r = V25f();
    if (r) V246({ x: r.x + r.w * 0.05, y: r.y + r.h * 0.05, w: r.w * 0.9, h: r.h * 0.9 });
    else {
      const V25j = V24x.current;
      if (V25j)
        V246({
          x: V25j.clientWidth * 0.1,
          y: V25j.clientHeight * 0.08,
          w: V25j.clientWidth * 0.8,
          h: V25j.clientHeight * 0.84,
        });
    }
  };

  const V25k = () => {
    V23u(0);
    V23w(0);
    V23y(false);
    V240(false);
    V242(100);
    V244({ x: 0, y: 0 });
    V246(null);
    V24e(0);
    V24g(0);
    V24i(100);
    V24a("keep");
  };

  const V25l = useCallback(
    (V25m: number, V25n: V3yf) => {
      if (V23n) {
        V23p((V25o) =>
          V25o.map((p) =>
            p.index === V25m
              ? {
                  ...p,
                  preview: V25n.preview,
                  thumbnail: V25n.thumbnail,
                  widthPx: V25n.widthPx,
                  heightPx: V25n.heightPx,
                  dpi: V25n.dpi && V25n.dpi > 0 ? V25n.dpi : p.dpi,
                }
              : p,
          ),
        );
      } else {
        V23h?.(V25m, V25n);
      }
    },
    [V23n, V23h],
  );

  const V25p = () =>
    (V24b === "all" ? V23q : V23q.filter((p) => p.index === V23r)).filter((p) => p.sourcePath);

  const V25q = async () => {
    const V25r = V25p();
    if (!V255 || !V25r.length || V24l) return;
    V24m(true);
    V24s(null);
    try {
      const r = V245 ? V25f() : null;
      const V25s =
        V245 && r && r.w > 0 && r.h > 0
          ? {
              x: clamp((V245.x - r.x) / r.w, 0, 1),
              y: clamp((V245.y - r.y) / r.h, 0, 1),
              w: clamp(V245.w / r.w, 0, 1),
              h: clamp(V245.h / r.h, 0, 1),
            }
          : null;
      const V25t = Math.round(V23t / 90);
      const V25u = V23v + (V23t - V25t * 90);
      const V25v = V249 === "keep" ? 0 : Number(V249);
      for (const p of V25r) {
        const V25w = await V477(p.sourcePath, {
          rotateQuarters: V25t,
          brightness: V24d,
          contrast: V24f,
          gamma: V24h / 100,
          colorMode: V247,
          bwThreshold: V24j,
          flipH: V23x,
          flipV: V23z,
          deskewDeg: V25u,
          crop: V25s,
          sourceDpi: p.dpi,
          targetDpi: V25v,
        });
        V25l(p.index, V25w);
        if (p.index === V23r) {
          V24u(!!V25w.canUndo);
          V24w(!!V25w.canRedo);
        }
      }
      V25k();
      if (V23n) V23l(true);
    } catch (V25x) {
      V24s(V25x instanceof Error ? V25x.message : String(V25x));
    } finally {
      V24m(false);
    }
  };

  const V25y = async () => {
    V25k();
    V248("color");
    V24k(128);
    const V25z = V23q.find((p) => p.index === V23r);
    if (V255 && V25z?.sourcePath) {
      V24m(true);
      V24s(null);
      try {
        const V260 = await V478(V25z.sourcePath);
        V25l(V25z.index, { ...V260, dpi: V24z.current.get(V25z.index) ?? 0 });
        V24u(false);
        V24w(false);
      } catch (V261) {
        V24s(V261 instanceof Error ? V261.message : String(V261));
      } finally {
        V24m(false);
      }
    }
  };

  const V262 = async (V263: Partial<V3ye>) => {
    const V264 = V25p();
    if (!V255 || !V264.length || V24l) return;
    V24m(true);
    V24s(null);
    try {
      for (const p of V264) {
        const V265 = await V477(p.sourcePath, {
          rotateQuarters: 0,
          brightness: 0,
          contrast: 0,
          gamma: 1,
          colorMode: "color",
          bwThreshold: V24j,
          ...V263,
        });
        V25l(p.index, V265);
        if (p.index === V23r) {
          V24u(!!V265.canUndo);
          V24w(!!V265.canRedo);
        }
      }
      if (V23n) V23l(true);
    } catch (V266) {
      V24s(V266 instanceof Error ? V266.message : String(V266));
    } finally {
      V24m(false);
    }
  };

  const V267 = () => {
    if (Math.abs(V23v) < 0.05) {
      V24s(
        "Chọn độ nghiêng ở thanh trượt trước khi khử nghiêng.",
      );
      return;
    }
    void V262({ deskewDeg: V23v }).then(() => V23w(0));
  };

  const V268 = async (V269: boolean) => {
    const V26a = V23q.find((p) => p.index === V23r);
    if (!V255 || !V26a?.sourcePath || V24l) return;
    V24m(true);
    V24s(null);
    try {
      const V26b = V269 ? await V479(V26a.sourcePath) : await V47a(V26a.sourcePath);
      V25l(V26a.index, V26b);
      V24u(!!V26b.canUndo);
      V24w(!!V26b.canRedo);
      if (V23n) V23l(true);
    } catch (V26c) {
      V24s(V26c instanceof Error ? V26c.message : String(V26c));
    } finally {
      V24m(false);
    }
  };

  const V26d = (V26e: number) => {
    if (V23o.length <= 1) {
      V24s("Hồ sơ phải còn ít nhất 1 trang.");
      return;
    }
    V23p((V26f) => V26f.filter((p) => p.index !== V26e));
    V23l(true);
  };

  const V26g = (V26h: number, V26i: -1 | 1) => {
    V23p((V26j) => {
      const i = V26j.findIndex((p) => p.index === V26h);
      const V26k = i + V26i;
      if (i < 0 || V26k < 0 || V26k >= V26j.length) return V26j;
      const V26l = [...V26j];
      [V26l[i], V26l[V26k]] = [V26l[V26k], V26l[i]];
      return V26l;
    });
    V23l(true);
  };

  const V26m = async () => {
    if (!V23i || V24p) return;
    V24s(null);
    V24q(true);
    try {
      const V26n = await open({
        multiple: true,
        directory: false,
        title: "Chọn ảnh hoặc PDF để chèn",
        filters: [
          {
            name: "Ảnh hoặc PDF",
            extensions: ["pdf", "png", "jpg", "jpeg", "tif", "tiff", "bmp"],
          },
        ],
      });
      const V26o = Array.isArray(V26n) ? V26n : V26n ? [V26n] : [];
      if (!V26o.length) return;
      const V26p = await V40s(V23i.batchId, V26o);
      V23p((V26q) => {
        const V26r = V26q.findIndex((p) => p.index === V23r);
        const V26s = V26r < 0 ? V26q.length : V26r + 1;
        return [...V26q.slice(0, V26s), ...V26p, ...V26q.slice(V26s)];
      });
      V23l(true);
    } catch (V26t) {
      V24s(V26t instanceof Error ? V26t.message : String(V26t));
    } finally {
      V24q(false);
    }
  };

  const V26u = async () => {
    if (!V23i || V24n) return;
    if (V23o.length === 0) {
      V24s("Không có trang nào để đóng gói.");
      return;
    }

    const V26v =
      V23t !== 0 ||
      V23v !== 0 ||
      V23x ||
      V23z ||
      !!V245 ||
      V24d !== 0 ||
      V24f !== 0 ||
      V24h !== 100 ||
      V249 !== "keep";
    if (V26v && !V24l) {
      await V25q();
    }
    V24o(true);
    V24s(null);
    try {
      const V26w = V23i.hoSo;
      const V26x = V23o.map((p) => ({
        sourcePath: p.sourcePath,
        dpi: p.dpi,
        paper: p.paper,
      }));
      if (V26w.id > 0) {
        await V40i(
          {
            maDuAn: V26w.maDuAn,
            tenDuAn: V26w.tenDuAn,
            maKhoi: V26w.maKhoi,
            phongSo: V26w.phongSo,
            mucLucSo: V26w.mucLucSo,
          },
          {
            hopSo: V26w.hopSo,
            hoSoSo: V26w.hoSoSo,
            tieuDe: V26w.tieuDe,
            thbq: V26w.thbq,
            nhom: V26w.nhom,
            nam: V26w.nam,
            thoiGian: V26w.thoiGian,
            ngayBatDau: V26w.ngayBatDau,
            ngayKetThuc: V26w.ngayKetThuc,
          },
          V26x,
          undefined,
          V23j ?? undefined,
          V26w.id,
        );
      } else {
        const V26y = V23j ?? V26w.pdfPath;
        if (!V26y) {
          V24s("Không xác định được file PDF để ghi đè.");
          V24o(false);
          return;
        }
        await V40r(V26y, V26x);
      }
      V23k();
      V23l(false);
      goto("drive");
    } catch (V26z) {
      V24s(V26z instanceof Error ? V26z.message : String(V26z));
      V24o(false);
    }
  };

  const V270 = (V23x ? -1 : 1) * (V241 / 100);
  const V271 = (V23z ? -1 : 1) * (V241 / 100);
  const V272 = `translate(${V243.x}px, ${V243.y}px) rotate(${V23t + V23v}deg) scale(${V270}, ${V271})`;
  const V273: CSSProperties | undefined = V245
    ? { inset: "auto", left: V245.x, top: V245.y, width: V245.w, height: V245.h }
    : undefined;

  const V274 = useMemo(
    () =>
      V4ov({
        brightness: V24d,
        contrast: V24f,
        gamma: V24h,
        colorMode: V247,
        threshold: V24j,
      }),
    [V24d, V24f, V24h, V247, V24j],
  );

  return (
    <section className={V23m ? "ed-overlay" : "view active"}>
      <svg width="0" height="0" className="absolute" aria-hidden>
        <filter id="ed-tone" colorInterpolationFilters="sRGB">
          <feComponentTransfer>
            <feFuncR type="linear" slope="1" intercept={V274.brightness} />
            <feFuncG type="linear" slope="1" intercept={V274.brightness} />
            <feFuncB type="linear" slope="1" intercept={V274.brightness} />
          </feComponentTransfer>
          <feComponentTransfer>
            <feFuncR type="linear" slope={V274.contrastSlope} intercept={V274.contrastIntercept} />
            <feFuncG type="linear" slope={V274.contrastSlope} intercept={V274.contrastIntercept} />
            <feFuncB type="linear" slope={V274.contrastSlope} intercept={V274.contrastIntercept} />
          </feComponentTransfer>
          <feComponentTransfer>
            <feFuncR type="gamma" exponent={V274.gammaExponent} />
            <feFuncG type="gamma" exponent={V274.gammaExponent} />
            <feFuncB type="gamma" exponent={V274.gammaExponent} />
          </feComponentTransfer>
          {V274.grayMatrix && <feColorMatrix type="matrix" values={V274.grayMatrix} />}
          {V274.bwSlope !== undefined && (
            <feComponentTransfer>
              <feFuncR type="linear" slope={V274.bwSlope} intercept={V274.bwIntercept} />
              <feFuncG type="linear" slope={V274.bwSlope} intercept={V274.bwIntercept} />
              <feFuncB type="linear" slope={V274.bwSlope} intercept={V274.bwIntercept} />
            </feComponentTransfer>
          )}
        </filter>
      </svg>

      <div className="editor-bar">
        <button className="btn sm" onClick={() => (V23m ? V23g?.() : goto("drive"))}>
          {V23m ? (
            <Icon name="dong" label="Đóng" />
          ) : (
            <Icon name="truoc" label="Quay lại" />
          )}
        </button>
        <h2 className="text-lg">Trình xử lý ảnh</h2>
        <span className="sub muted">
          {V23n && V23i
            ? `— ${V23i.hoSo.hoSoSo}${V23i.hoSo.tieuDe ? ` · ${V23i.hoSo.tieuDe}` : ""} · ${V250} trang`
            : V250 > 0
              ? `— trang ${V251 + 1} / ${V250}`
              : "— Chưa mở tài liệu"}
        </span>
        <div className="spacer flex-1" />
        {V24r && (
          <span className="muted text-warning-700 text-sm">
            <Icon name="canhBao" /> {V24r}
          </span>
        )}
        <button
          className="btn sm"
          disabled={V24l || !V24t}
          onClick={() => void V268(true)}
          title="Lùi một bước xử lý của trang này (giữ 5 bước gần nhất)."
        >
          <Icon name="hoanTac" />
          Hoàn tác
        </button>
        <button
          className="btn sm"
          disabled={V24l || !V24v}
          onClick={() => void V268(false)}
          title="Tiến lại bước vừa hoàn tác."
        >
          <Icon name="lamLai" />
          Làm lại
        </button>
        <button
          className="btn sm"
          disabled={V24l || !V255}
          onClick={V25y}
          title="Bỏ toàn bộ chỉnh sửa của trang này, trả về đúng ảnh quét gốc."
        >
          <Icon name="datLai" />
          Đặt lại
        </button>
        {V255 ? (
          <>
            <button className="btn primary sm" disabled={V24l || !V252?.sourcePath} onClick={V25q}>
              {V24l ? (
                "Đang xử lý…"
              ) : (
                <>
                  <Icon name="tuDong" />
                  Áp dụng
                </>
              )}
            </button>
            {V23m ? (
              <button className="btn sm" onClick={V23g}>
                <Icon name="duyet" />
                Xong
              </button>
            ) : (
              <button className="btn primary sm" disabled={V24n} onClick={V26u}>
                {V24n ? (
                  "Đang đóng gói…"
                ) : (
                  <>
                    <Icon name="luu" />
                    Lưu & đóng gói lại
                  </>
                )}
              </button>
            )}
          </>
        ) : (
          <button className="btn primary sm" onClick={() => goto("upload")}>
            <Icon name="luu" />
            Lưu &amp; xuất PDF
          </button>
        )}
      </div>

      {V23n && V24r && (
        <div className="banner err mt-0 mx-0 mb-2.5">
          <span className="bi">
            <Icon name="canhBao" size={18} />
          </span>
          <div className="bx">
            <b>Lưu/xử lý lỗi</b>
            {V24r}
          </div>
          <div className="actions">
            <button className="btn sm" onClick={() => V24s(null)}>
              Bỏ qua
            </button>
          </div>
        </div>
      )}
      {V23n && !V24r && (
        <div className="banner info mt-0 mx-0 mb-2.5">
          <span className="bi">
            <Icon name="sua" size={18} />
          </span>
          <div className="bx">
            <b>Chỉnh sửa lại tài liệu đã đóng gói</b>
            Sửa ảnh/xoá/sắp xếp/chèn trang rồi bấm “Lưu & đóng gói lại” — PDF cũ sẽ được ghi đè tại
            chỗ.
          </div>
        </div>
      )}

      <div className="editor-grid">
        {}
        <div className="ed-pages">
          {V23n && (
            <button
              className="btn sm w-full mb-2"
              disabled={V24p}
              onClick={() => void V26m()}
              title="Chèn trang mới từ tệp ảnh hoặc PDF (chèn sau trang đang chọn)."
            >
              {V24p ? (
                "Đang chèn…"
              ) : (
                <>
                  <Icon name="them" />
                  Chèn trang (ảnh/PDF)
                </>
              )}
            </button>
          )}
          {V23q.length === 0 ? (
            <div className="muted py-3 px-1.5 text-sm text-center">Chưa có trang</div>
          ) : (
            V23q.map((p, i) => (
              <div
                key={p.index}
                className={`ed-thumb ${p.index === V23r ? "on" : ""}`}
                onClick={() => V23s(p.index)}
              >
                <div
                  className={`doc${p.thumbnail ? " has-img" : ""}`}
                  style={
                    p.thumbnail
                      ? {
                          backgroundImage: `url(${p.thumbnail})`,
                          backgroundSize: "cover",
                          backgroundPosition: "center",
                        }
                      : undefined
                  }
                />
                <div className="cap">{i + 1}</div>
                {V23n && (
                  <div className="ed-thumb-ops" onClick={(V275) => V275.stopPropagation()}>
                    <button title="Lên" disabled={i === 0} onClick={() => V26g(p.index, -1)}>
                      <Icon name="len" size={14} />
                    </button>
                    <button
                      title="Xuống"
                      disabled={i === V23q.length - 1}
                      onClick={() => V26g(p.index, 1)}
                    >
                      <Icon name="xuong" size={14} />
                    </button>
                    <button title="Xoá trang" className="del" onClick={() => V26d(p.index)}>
                      <Icon name="xoa" size={14} />
                    </button>
                  </div>
                )}
              </div>
            ))
          )}
        </div>

        {}
        <div
          className="ed-stage"
          onPointerDown={(V276) => {
            if (V241 <= 100) return;
            const t = V276.target as HTMLElement;
            if (t.classList.contains("h") || t.classList.contains("crop-frame")) return;
            V24y.current = {
              mode: "pan",
              sx: V276.clientX,
              sy: V276.clientY,
              x: V243.x,
              y: V243.y,
            };
          }}
          style={V241 > 100 ? { cursor: "grab" } : undefined}
        >
          {V250 > 0 && (
            <div className="pagepill">
              Trang {V251 + 1} / {V250}
              {V252 ? ` · ${V252.paper} · ${V252.dpi} DPI` : ""}
            </div>
          )}
          <div
            className={`ed-page${V253 ? " has-img" : ""}`}
            ref={V24x}
            style={{ transform: V272 }}
          >
            {V253 && (
              <img
                className="ed-img"
                src={V253}
                alt={`Trang ${V23r}`}
                draggable={false}
                style={V274.active ? { filter: "url(#ed-tone)" } : undefined}
              />
            )}
            <div
              className={`crop-frame ${V245 ? "" : "hidden"}`}
              style={V273}
              onPointerDown={(V277) => {
                if ((V277.target as HTMLElement).classList.contains("h") || !V245) return;
                V24y.current = {
                  mode: "move",
                  sx: V277.clientX,
                  sy: V277.clientY,
                  x: V245.x,
                  y: V245.y,
                };
              }}
            >
              {(["tl", "tr", "bl", "br"] as V4ox[]).map((c) => (
                <i
                  key={c}
                  className={`h ${c}`}
                  onPointerDown={(V278) => {
                    V278.stopPropagation();
                    if (!V245) return;
                    V24y.current = {
                      mode: "resize",
                      c,
                      sx: V278.clientX,
                      sy: V278.clientY,
                      x: V245.x,
                      y: V245.y,
                      w: V245.w,
                      h: V245.h,
                    };
                  }}
                />
              ))}
            </div>
          </div>
          <div className="zoombar">
            <button onClick={() => V242((z) => Math.max(30, z - 10))}>−</button>
            <span>{V241}%</span>
            <button onClick={() => V242((z) => Math.min(300, z + 10))}>+</button>
            <span className="opacity-50">|</span>
            <button
              title="Vừa khung"
              onClick={() => {
                V242(100);
                V244({ x: 0, y: 0 });
              }}
            >
              <Icon name="phongToAnh" size={14} />
            </button>
          </div>
        </div>

        {}
        <div className="ed-tools">
          <div className="tgroup">
            <h4>Xoay &amp; Lật</h4>
            <div className="tbtns">
              <V4p5
                label="Xoay trái"
                icon="xoayTrai"
                onClick={() => V23u((r) => V4p0(r - 90))}
              />
              <V4p5
                label="Xoay phải"
                icon="xoayPhai"
                onClick={() => V23u((r) => V4p0(r + 90))}
              />
              <V4p5
                label="Lật ngang"
                icon="latNgang"
                onClick={() => V23y((f) => !f)}
                active={V23x}
              />
              <V4p5
                label="Lật dọc"
                icon="latDoc"
                onClick={() => V240((f) => !f)}
                active={V23z}
              />
              <V4p5 label="180°" icon="xoay180" onClick={() => V23u((r) => V4p0(r + 180))} />
            </div>
            <div className="slider-row mt-3">
              <label>
                Góc xoay <b>{V23t}°</b>
              </label>
              <input
                type="range"
                min={-180}
                max={180}
                value={V23t}
                onChange={(V279) => V23u(+V279.target.value)}
              />
            </div>
          </div>

          <div className="tgroup">
            <h4>Cắt &amp; Căn chỉnh</h4>
            <div className="tbtns">
              <V4p5
                label="Cắt thủ công"
                icon="catThuCong"
                onClick={V25i}
                active={!!V245}
              />
              <V4p5
                label="Tự cắt viền"
                icon="catVien"
                onClick={() => void V262({ autoCrop: true })}
                disabled={V24l}
              />
              <V4p5
                label="Làm thẳng"
                icon="lamThang"
                onClick={() => void V262({ autoDeskew: true })}
                disabled={V24l}
              />
              <V4p5
                label="Căn giữa"
                icon="canGiua"
                onClick={() => void V262({ align: "center" })}
                disabled={V24l}
              />
              <V4p5
                label="Căn lề"
                icon="canLe"
                onClick={() => void V262({ align: "left" })}
                disabled={V24l}
              />
              <V4p5
                label="Khử nghiêng"
                icon="khuNghieng"
                onClick={V267}
                disabled={V24l}
                title="Áp ngay góc ở thanh trượt “Độ nghiêng” bên dưới."
              />
            </div>
            <div className="slider-row mt-3">
              <label>
                Độ nghiêng <b>{V23v.toFixed(1).replace(".", ",")}°</b>
              </label>
              <input
                type="range"
                min={-10}
                max={10}
                step={0.1}
                value={V23v}
                onChange={(V27a) => V23w(+V27a.target.value)}
              />
            </div>
          </div>

          <div className="tgroup">
            <h4>Tông màu</h4>
            <V4p6
              label="Độ sáng"
              min={-100}
              max={100}
              initial={0}
              value={V24d}
              onChange={V24e}
            />
            <V4p6
              label="Tương phản"
              min={-100}
              max={100}
              initial={0}
              value={V24f}
              onChange={V24g}
            />
            <V4p6
              label="Gamma"
              min={50}
              max={250}
              initial={100}
              value={V24h}
              onChange={V24i}
              format={(v) => (v / 100).toFixed(1).replace(".", ",")}
            />
            <button
              className="btn sm w-full"
              disabled={V24l || !V255}
              onClick={() => void V262({ autoTone: true })}
              title="Tự kéo giãn dải sáng (auto-levels) — bản quét xám xịt về lại nền trắng chữ đen."
            >
              <Icon name="tuDong" />
              Tự động cân chỉnh
            </button>
          </div>

          <div className="tgroup">
            <h4>Chế độ màu</h4>
            <V4oq
              full
              value={V247}
              onChange={V248}
              options={[
                { value: "color", label: "Màu" },
                { value: "gray", label: "Xám" },
                { value: "bw", label: "Đen trắng" },
              ]}
            />
            <div className="mt-3">
              <V4p6
                label="Ngưỡng (B&W)"
                min={0}
                max={255}
                initial={128}
                value={V24j}
                onChange={V24k}
                disabled={V247 !== "bw"}
              />
            </div>
          </div>

          <div className="tgroup">
            <h4>Làm sạch</h4>
            <div className="tbtns">
              <V4p5
                label="Khử nhiễu"
                icon="khuNhieu"
                onClick={() => void V262({ denoise: true })}
                disabled={V24l}
              />
              <V4p5
                label="Khử đốm"
                icon="khuDom"
                onClick={() => void V262({ despeckle: true })}
                disabled={V24l}
              />
              <V4p5
                label="Xóa lỗ bấm"
                icon="xoaLoBam"
                onClick={() => void V262({ punchHoles: true })}
                disabled={V24l}
                title="Tìm đốm tròn cỡ lỗ bấm sát mép giấy rồi tô lại bằng màu nền."
              />
              <V4p5
                label="Làm nét"
                icon="lamNet"
                onClick={() => void V262({ sharpen: true })}
                disabled={V24l}
              />
            </div>
          </div>

          <div className="tgroup">
            <h4>Kích thước &amp; DPI</h4>
            <V4oq
              full
              value={V249}
              onChange={V24a}
              options={[
                { value: "keep", label: "Giữ nguyên" },
                { value: "300", label: "300 DPI" },
                { value: "600", label: "600 DPI" },
              ]}
            />
            <div className="muted text-xs mt-1.5">
              Đổi DPI giữ nguyên khổ giấy, chỉ đổi số điểm ảnh — áp khi bấm “Áp dụng”.
            </div>
            <div className="apply-row mt-2.5">
              <span className="muted text-xs self-center">Áp dụng:</span>
              <V4oq
                value={V24b}
                onChange={V24c}
                options={[
                  { value: "this", label: "Trang này" },
                  { value: "all", label: "Tất cả" },
                ]}
              />
            </div>
            {V24b === "all" && (
              <div className="muted text-xs mt-1.5 text-warning-700">
                <Icon name="canhBao" /> Mọi thao tác xử lý sẽ chạy trên cả {V250} trang.
              </div>
            )}
          </div>

          <div className="ai-note">
            <Icon name="thongTin" /> Không có công cụ AI ở bước này. Tăng cường/đọc chữ bằng AI
            (OCR) chạy tự động ở máy chủ sau khi tải lên.
          </div>
        </div>
      </div>
    </section>
  );
}

function V4p5({
  label: V27b,
  icon,
  onClick: V27c,
  active: V27d,
  disabled,
  title: V27e,
}: {
  label: string;
  icon: V41v;
  onClick?: () => void;
  active?: boolean;
  disabled?: boolean;
  title?: string;
}) {
  const [V27f, V27g] = useState(false);
  return (
    <div
      className={`tbtn ${V27d || V27f ? "flash" : ""} ${disabled ? "off" : ""}`}
      title={V27e}
      onClick={() => {
        if (disabled || !V27c) return;
        V27c();
        V27g(true);
        setTimeout(() => V27g(false), 250);
      }}
    >
      <span className="gi">
        <Icon name={icon} size={18} />
      </span>
      {V27b}
    </div>
  );
}

function V4p6({
  label: V27h,
  min,
  max,
  initial: V27i,
  step: V27j = 1,
  format: V27k,
  disabled,
  value: V27l,
  onChange: V27m,
}: {
  label: string;
  min: number;
  max: number;
  initial: number;
  step?: number;
  format?: (v: number) => string;
  disabled?: boolean;

  value?: number;
  onChange?: (v: number) => void;
}) {
  const [V27n, V27o] = useState(V27i);
  const v = V27l ?? V27n;
  const set = (n: number) => (V27m ? V27m(n) : V27o(n));
  return (
    <div className="slider-row">
      <label>
        {V27h} <b>{V27k ? V27k(v) : v}</b>
      </label>
      <input
        type="range"
        min={min}
        max={max}
        step={V27j}
        value={v}
        disabled={disabled}
        onChange={(V27p) => set(+V27p.target.value)}
      />
    </div>
  );
}
