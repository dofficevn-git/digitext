// Copyright (c) 2026 DigiText. All rights reserved.
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { V460 } from "@/state";
import { isTauri } from "@/api/ipc";
import { V47d, V475, V47c, V476, type V474, type V46y } from "@/api/scanner";
import { V40i, V40k, V40y, type V40c } from "@/api/archive";
import { V4og } from "@/features/scan/defaults";
import { V4oi, V4oh } from "@/features/scan/selection";
import { V4oj } from "@/features/scan/vanBan";
import type { V3y4, V3y3, V3y8, V3y5, V3yb, V3yc, V3y6 } from "@/domain/types";
import { V3y9, V3ya } from "@/domain/types";
import { V3z9 } from "@/domain/permissions";
import V4oq from "@/ui/Seg";
import V4p4 from "@/features/editor/EditorView";
import { Icon } from "@/components/Icon";
import { Badge, Banner, Button, Card, Checkbox, Field, IconButton, Kbd, PageHeader, Select, Stepper, Tip } from "@/ui";

const V4p7 = (V27q: number) => V27q === 300 || V27q === 600;

const V4p8: Array<{ key: keyof V3y8; label: string; hint: string }> = [
  {
    key: "deskew",
    label: "Khử nghiêng",
    hint: "Tự phát hiện góc lệch rồi xoay trang cho thẳng.",
  },
  {
    key: "punchHoles",
    label: "Xoá lỗ bấm",
    hint: "Tìm đốm tròn cỡ lỗ bấm sát mép giấy rồi tô lại bằng màu nền.",
  },
  {
    key: "despeckle",
    label: "Xoá đốm",
    hint: "Lọc trung vị 3×3 — xoá hạt bẩn lẻ mà vẫn giữ nét chữ.",
  },
  {
    key: "denoise",
    label: "Khử nhiễu",
    hint: "Làm mịn nhẹ, giảm hạt nhiễu nền.",
  },
  {
    key: "sharpen",
    label: "Làm nét",
    hint: "Bù lại độ sắc của nét chữ sau khi làm mịn.",
  },
];

export default function V4p9() {
  const {
    goto,
    nodes: V27r,
    path,
    scanTarget: V27s,
    setScanTarget: V27t,
    reloadArchive: V27u,
    session: V27v,
  } = V460();
  const [V27w, V27x] = useState<"capture" | "review">("capture");

  const V27y = V3z9(V27v?.permissions, "scan.execute");

  const V27z = V27r[path[path.length - 1]];
  const target = V27z && V27z.type !== "root" ? V27z.name : "";
  const V280 = path
    .slice(1)
    .map((V281) => V27r[V281]?.name)
    .filter(Boolean)
    .join(" / ");

  const [V282, V283] = useState<V3y6[]>([]);
  const [V284, V285] = useState<string | undefined>(undefined);
  const [V286, V287] = useState(true);
  const [V288, V289] = useState<string | null>(null);

  const [V28a, V28b] = useState(false);
  const [V28c, V28d] = useState<{ text: string; tone: "ok" | "info" | "err" } | null>(null);
  const [V28e, V28f] = useState<V3y3>(V4og.dpi);
  const [color, V28g] = useState<V3y4>(V4og.color);
  const [V28h, V28i] = useState<V3y5>(V4og.source);
  const [V28j, V28k] = useState(V4og.duplex);

  const [V28l, V28m] = useState(true);
  const [V28n, V28o] = useState<V3y8>(V3y9);

  const [V28p, V28q] = useState<{ done: number; total: number } | null>(null);

  const [V28r, V28s] = useState<Set<number>>(() => new Set());

  const [V28t, V28u] = useState<V3yc[]>([]);
  const [V28v, V28w] = useState(false);
  const [V28x, V28y] = useState(false);
  const [V28z, V290] = useState(false);
  const [V291, V292] = useState<{ atPage: number } | null>(null);
  const [V293, V294] = useState<string | null>(null);
  const [V295, V296] = useState(1);

  const [V297, V298] = useState<number | null>(null);
  const V299 = useRef<V474 | null>(null);

  const [V29a, V29b] = useState(false);
  const [V29c, V29d] = useState<V40c | null>(null);
  const [V29e, V29f] = useState<string | null>(null);

  const [V29g, V29h] = useState<Set<number>>(() => new Set());
  const [V29i, V29j] = useState<{ x: number; y: number } | null>(null);
  const [V29k, V29l] = useState(false);
  const [V29m, V29n] = useState<{
    text: string;
    tone: "ok" | "err";
    pdfPath?: string;
  } | null>(null);

  const V29o = useCallback(() => {
    V287(true);
    V289(null);
    V475()
      .then((V29p) => {
        V283(V29p);
        V285((d) => (d && V29p.some((s) => s.id === d) ? d : V29p[0]?.id));
      })
      .catch((V29q) => {
        V283([]);
        V289(typeof V29q === "string" ? V29q : V29q instanceof Error ? V29q.message : String(V29q));
      })
      .finally(() => V287(false));
  }, []);

  const V29r = useCallback(async () => {
    V28d(null);
    V28b(true);
    try {
      const V29s = await V47c();
      const V29t = await open({
        multiple: false,
        directory: false,
        title: "Chọn bộ cài driver máy quét",
        filters: V29s.length ? [{ name: "Bộ cài driver", extensions: V29s }] : undefined,
      });
      if (typeof V29t !== "string") return;
      const V29u = await V47d(V29t);
      V28d({ text: V29u.message, tone: V29u.manual ? "info" : "ok" });
      V29o();
    } catch (V29v) {
      V28d({ text: V29v instanceof Error ? V29v.message : String(V29v), tone: "err" });
    } finally {
      V28b(false);
    }
  }, [V29o]);

  useEffect(() => {
    V29o();
    return () => {
      void V299.current?.cancel();
      V299.current?.dispose();
    };
  }, [V29o]);

  const V29w: V46y = {
    onState: (s) => {
      if (s.kind === "capturing") {
        V28w(true);
        V28y(false);
        V294(null);
      } else if (s.kind === "disconnected") {
        V28w(false);
        V28y(false);
        V292({ atPage: s.atPage });
      } else if (s.kind === "completed" || s.kind === "cancelled") {
        V28w(false);
        V28y(false);
      }
    },
    onPage: (p) => V28u((V29x) => [...V29x, p]),
    onError: (m) => {
      V294(m);
      V28y(false);
    },

    onEnhanced: (V29y, V29z, V2a0, V2a1) => {
      if (V29z) {
        V28u((V2a2) =>
          V2a2.map((p) =>
            p.index === V29y
              ? {
                  ...p,
                  preview: V29z.preview,
                  thumbnail: V29z.thumbnail,
                  widthPx: V29z.widthPx,
                  heightPx: V29z.heightPx,
                }
              : p,
          ),
        );

        V28s((V2a3) => new Set(V2a3).add(V29y));
      }
      V28q(V2a0 >= V2a1 ? null : { done: V2a0, total: V2a1 });
    },
  };

  const V2a4 = (V2a5: Partial<V3yb> = {}): V3yb => ({
    deviceId: V284,
    dpi: V28e,
    color,
    source: V28h,

    duplex: V28h === "adf" ? V28j : false,
    target: target || undefined,

    enhance: V28l ? V28n : V3ya,
    ...V2a5,
  });

  const V2a6 = async (V2a7: Partial<V3yb> = {}) => {
    if (!V284) {
      V294(
        "Chưa phát hiện máy quét. Cắm máy quét rồi bấm “Tìm lại”.",
      );
      return;
    }
    V28y(true);

    await V299.current?.cancel();
    V299.current?.dispose();
    V28u([]);
    V292(null);
    V294(null);
    V290(false);
    V28q(null);
    V28s(new Set());
    try {
      V299.current = await V476(V2a4({ startIndex: 1, ...V2a7 }), V29w);
    } catch (V2a8) {
      V294(V2a8 instanceof Error ? V2a8.message : String(V2a8));
      V28y(false);
    }
  };

  const V2a9 = async (n: number) => {
    V292(null);
    V294(null);
    V28y(true);
    try {
      V299.current = await V476(V2a4({ startIndex: n }), V29w);
    } catch (V2aa) {
      V294(V2aa instanceof Error ? V2aa.message : String(V2aa));
      V28y(false);
    }
  };

  const V2ab = async () => {
    await V299.current?.cancel();
    V299.current = null;
    V28w(false);
    V28y(false);
    V290(true);
  };

  const V2ac = async () => {
    V290(false);
    await V2a9(V28t.length + 1);
  };

  const V2ad = async () => {
    await V299.current?.cancel();
    V290(false);
    V27x("review");
  };

  const V2ae = async () => {
    if (!V27s || V28t.length === 0) return;
    V29b(true);
    V29f(null);
    V29d(null);
    try {
      const V2af = V28t.map((p) => ({
        sourcePath: p.sourcePath,
        dpi: p.dpi,
        paper: p.paper,
      }));
      const V2ag = await V40i(V27s.phong, V27s.hoSo, V2af);
      V29d(V2ag);
      V27u();
    } catch (V2ah) {
      V29f(V2ah instanceof Error ? V2ah.message : String(V2ah));
    } finally {
      V29b(false);
    }
  };

  const V2ai = async () => {
    await V299.current?.cancel();
    V299.current = null;
    V28u([]);
    V29d(null);
    V29f(null);
    V292(null);
    V290(false);
    V28w(false);
    V28y(false);
    V27x("capture");
  };

  const V2aj = async () => {
    V29j(null);
    if (!V27s) {
      V29n({
        text: "Cần quét vào một hồ sơ để gộp PDF (chưa có hồ sơ đích).",
        tone: "err",
      });
      return;
    }
    const V2ak = V4oi(V28t, V29g);
    if (V2ak.length === 0) return;
    V29l(true);
    V29n(null);
    try {
      const V2al = await V40k(V27s.phong, V27s.hoSo, V2ak);
      V29n({
        text: `Đã gộp ${V2al.pageCount} ảnh thành 1 PDF.`,
        tone: "ok",
        pdfPath: V2al.pdfPath,
      });
      V29h(new Set());
      V27u();
    } catch (V2am) {
      V29n({ text: V2am instanceof Error ? V2am.message : String(V2am), tone: "err" });
    } finally {
      V29l(false);
    }
  };

  useEffect(() => {
    if (!V29i) return;
    const V2an = () => V29j(null);
    const V2ao = (V2ap: KeyboardEvent) => {
      if (V2ap.key === "Escape") V29j(null);
    };
    window.addEventListener("click", V2an);
    window.addEventListener("contextmenu", V2an);
    window.addEventListener("keydown", V2ao);
    return () => {
      window.removeEventListener("click", V2an);
      window.removeEventListener("contextmenu", V2an);
      window.removeEventListener("keydown", V2ao);
    };
  }, [V29i]);

  useEffect(() => {
    if (V297 === null) return;
    const V2aq = (V2ar: KeyboardEvent) => {
      if (V2ar.key === "Escape") V298(null);
    };
    window.addEventListener("keydown", V2aq);
    return () => window.removeEventListener("keydown", V2aq);
  }, [V297]);

  useEffect(() => {
    const V2as = (V2at: KeyboardEvent) => {
      if (V27w !== "capture" || V297 !== null) return;
      const V2au = (V2at.target as HTMLElement | null)?.tagName;
      if (V2au === "INPUT" || V2au === "SELECT" || V2au === "TEXTAREA") return;
      if (V2at.key === "F2") {
        V2at.preventDefault();

        if (V27y && !(V28v || V28x || V286 || !V284)) void V2a6();
      } else if (V2at.key === "F3") {
        V2at.preventDefault();
        if (V28v) void V2ab();
        else if (V28z) void V2ac();
      } else if (V2at.key === "F4") {
        V2at.preventDefault();
        void V2ad();
      }
    };
    window.addEventListener("keydown", V2as);
    return () => window.removeEventListener("keydown", V2as);
  });

  const V2av = useMemo(() => {
    const V2aw = new Map<string, number>();
    for (const p of V28t) V2aw.set(p.paper, (V2aw.get(p.paper) ?? 0) + 1);
    let V2ax = "";
    let max = -1;
    for (const [V2ay, n] of V2aw) {
      if (n > max) {
        max = n;
        V2ax = V2ay;
      }
    }
    return V2ax;
  }, [V28t]);

  const V2az = useMemo(
    () =>
      new Set(
        V28t.filter((p) => !p.dpiOk || !p.paperKnown || p.paper !== V2av).map((p) => p.index),
      ),
    [V28t, V2av],
  );

  useEffect(() => {
    if (V27w !== "review") return;
    const f = V28t.find((p) => V2az.has(p.index));
    V296(f ? f.index : 1);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [V27w]);

  const V2b0 = useMemo(() => {
    const m = new Map<string, number>();
    for (const p of V28t) m.set(p.paper, (m.get(p.paper) ?? 0) + 1);
    return [...m.entries()];
  }, [V28t]);

  const V2b1 = useMemo(() => {
    const m = new Map<number, number>();
    for (const p of V28t) m.set(p.dpi, (m.get(p.dpi) ?? 0) + 1);
    return [...m.entries()];
  }, [V28t]);

  const V2b2 = V28t.filter((p) => V2az.has(p.index));
  const V2b3 = V28t[V295 - 1];

  return (
    <section className="view active">
      <PageHeader
        title="Quét tài liệu"
        description={V27s ? `Quét vào hồ sơ: ${V27s.label}` : V280 ? `Quét vào thư mục: ${V280}` : "Chưa chọn nơi lưu. Mở hồ sơ ở Kho tài liệu hoặc Cây thư mục rồi bấm Quét vào hồ sơ."}
        actions={
          !V27s && (
            <>
              <Button icon="kho" onClick={() => goto("drive")}>
                Kho tài liệu
              </Button>
              <Button icon="cayThuMuc" onClick={() => goto("tree")}>
                Cây thư mục
              </Button>
            </>
          )
        }
      />
      <Stepper className="mb-4" current={V27w === "review" ? 2 : V28v || V28t.length > 0 ? 1 : 0} steps={[{ key: "thietlap", label: "Thiết lập" }, { key: "quet", label: "Quét" }, { key: "soat", label: "Soát và lưu" }]} />

      {V28x && !V28v && (
        <Banner tone="info" className="mb-3" title="Đang khởi động máy quét…">
          Lần quét đầu với thiết bị này phải dò cấu hình nên hơi lâu, các lô sau sẽ nhanh hơn.
        </Banner>
      )}

      {V293 && (
        <Banner tone="danger" className="mb-3" title="Lỗi máy quét" onClose={() => V294(null)}>
          {V293}
        </Banner>
      )}

      {V291 && (
        <Banner
          tone="warning"
          className="mb-3"
          title={`Mất kết nối máy quét giữa lô. Đã lưu ${V28t.length} trang.`}
          actions={
            <>
              <Button size="sm" variant="primary" onClick={() => void V2a9(V28t.length + 1)}>
                Tiếp tục từ trang {V28t.length + 1}
              </Button>
              <Button
                size="sm"
                onClick={() => {
                  V292(null);
                  V27x("review");
                }}
              >
                Kết thúc lô
              </Button>
            </>
          }
        >
          Cắm lại máy quét rồi tiếp tục, không trang nào bị mất.
        </Banner>
      )}

      {V27w === "capture" && (
        <>
          <Card className="mb-4" title="Thiết lập máy quét">
            <div className="setup-grid">
              <div className="setup-row">
                <label>Quét vào</label>
                <Badge tone={target ? "info" : "gray"}>
                  <Icon name="thuMuc" size={13} />
                  {target || "Chưa chọn thư mục"}
                </Badge>
              </div>
              <div className="setup-row">
                <label>Độ phân giải</label>
                <V4oq
                  value={String(V28e)}
                  onChange={(v) => V28f(Number(v) as V3y3)}
                  options={[
                    { value: "300", label: "300" },
                    { value: "600", label: "600 DPI" },
                  ]}
                />
              </div>
              <div className="setup-row">
                <label>Máy quét</label>
                <div className="flex flex-wrap items-center gap-2">
                  <Select size="sm" value={V284 ?? ""} onChange={(v) => V285(v)} disabled={V286} className="w-64" aria-label="Máy quét">
                    {V286 && <option value="">Đang tìm máy quét…</option>}
                    {!V286 && V282.length === 0 && <option value="">Chưa phát hiện máy quét</option>}
                    {V282.map((s) => (
                      <option key={s.id} value={s.id}>
                        {s.name}
                        {s.connected ? "" : " (mất kết nối)"}
                      </option>
                    ))}
                  </Select>
                  <Button size="sm" icon="lamMoi" loading={V286} onClick={V29o}>
                    Tìm lại
                  </Button>
                  {isTauri() && (
                    <Tip text="Chọn bộ cài driver tải từ trang hãng, ứng dụng chạy bộ cài rồi tự tìm lại">
                      <Button size="sm" icon="taiVe" loading={V28a} onClick={() => void V29r()}>
                        Cài driver
                      </Button>
                    </Tip>
                  )}
                </div>
              </div>
              {V28c && (
                <div className="setup-row">
                  <label />
                  <span className={`muted text-sm${V28c.tone === "err" ? " text-danger-700" : ""}`}>
                    <Icon
                      size={13}
                      name={
                        V28c.tone === "ok" ? "duyet" : V28c.tone === "info" ? "thongTin" : "canhBao"
                      }
                    />{" "}
                    {V28c.text}
                  </span>
                </div>
              )}
              <div className="setup-row">
                <label>Màu</label>
                <V4oq
                  value={color}
                  onChange={V28g}
                  options={[
                    { value: "rgb", label: "RGB" },
                    { value: "gray", label: "Xám" },
                  ]}
                />
              </div>
              <div className="setup-row">
                <label>Nguồn giấy</label>
                <V4oq
                  value={V28h}
                  onChange={V28i}
                  options={[
                    { value: "adf", label: "ADF (tự động)" },
                    { value: "glass", label: "Kính" },
                  ]}
                />
              </div>
              {V28h === "adf" && (
                <div className="setup-row">
                  <label>Hai mặt</label>
                  <V4oq
                    value={V28j ? "on" : "off"}
                    onChange={(v) => V28k(v === "on")}
                    options={[
                      { value: "off", label: "1 mặt" },
                      { value: "on", label: "2 mặt (duplex)" },
                    ]}
                  />
                </div>
              )}
              <div className="setup-row">
                <label>Khổ giấy</label>
                <span className="muted">Tự nhận (ISO 216, ±3mm)</span>
              </div>
              <div className="setup-row">
                <label>Tự làm sạch</label>
                <V4oq
                  value={V28l ? "on" : "off"}
                  onChange={(v) => V28m(v === "on")}
                  options={[
                    { value: "on", label: "Bật" },
                    { value: "off", label: "Tắt" },
                  ]}
                />
              </div>
              {V28l && (
                <div className="setup-row">
                  <label />
                  <div className="enhance-steps">
                    {V4p8.map((s) => (
                      <Tip key={s.key} text={s.hint}>
                        <span>
                          <Field label={s.label} inline>
                            <Checkbox checked={V28n[s.key]} onChange={(v) => V28o((V2b6) => ({ ...V2b6, [s.key]: v }))} />
                          </Field>
                        </span>
                      </Tip>
                    ))}
                  </div>
                </div>
              )}
            </div>
            <div className="mt-4 flex flex-wrap items-center gap-2.5">
              <Button variant="primary" icon="quet" disabled={!V27y || V28v || V28x || V286 || !V284} onClick={() => void V2a6()}>
                Bắt đầu quét lô <Kbd keys={["F2"]} className="ml-1" />
              </Button>
              <Button disabled={!V27y || V28v || V28x || V286 || !V284} onClick={() => void V2a6({ maxPages: 1 })}>
                Quét thử 1 trang
              </Button>
              {!V27y && <span className="text-sm text-warning-700">Tài khoản của bạn không có quyền quét. Liên hệ quản trị dự án.</span>}
              {!V286 && !V284 && <span className="text-sm text-fg-subtle">{V288 ? `Lỗi tìm máy quét: ${V288}` : "Chưa phát hiện máy quét. Cắm thiết bị rồi bấm Tìm lại."}</span>}
            </div>
          </Card>

          {(V28v || V28t.length > 0) && (
            <div className="card">
              <div className="capbar">
                <div className="live">
                  {V28v ? (
                    <>
                      <span className="pulse" /> Đang quét lô
                    </>
                  ) : V28z ? (
                    <>
                      <Icon name="tamDung" />
                      Lô tạm dừng
                    </>
                  ) : (
                    <>
                      <Icon name="dungHan" />
                      Lô đã dừng
                    </>
                  )}
                </div>
                {target && <span className="muted">{target}</span>}
                <span className="badge gray tnum">{V28t.length} trang</span>
                {V28p && (
                  <span
                    className="badge info tnum"
                    title="Ảnh đang được làm sạch tự động ở nền — lô sẽ báo kết thúc khi xong."
                  >
                    <Icon name="tuDong" />
                    Đang làm sạch {V28p.done}/{V28p.total}
                  </span>
                )}
                <div className="spacer flex-1" />
                {V28z ? (
                  <Button size="sm" variant="primary" icon="chay" disabled={!V27y || V28x} onClick={() => void V2ac()}>
                    Tiếp tục quét <Kbd keys={["F3"]} />
                  </Button>
                ) : (
                  <Button size="sm" icon="tamDung" disabled={!V28v} onClick={() => void V2ab()}>
                    Tạm dừng <Kbd keys={["F3"]} />
                  </Button>
                )}
                <Button size="sm" variant="primary" disabled={V28t.length === 0} onClick={() => void V2ad()}>
                  Kết thúc lô <Kbd keys={["F4"]} />
                </Button>
              </div>
              <div className="thumbs">
                {V28t.map((p) => {
                  const V2b7 = V2az.has(p.index);
                  return (
                    <div
                      key={p.index}
                      className={`thumb${V2b7 ? " flag" : ""}${V29g.has(p.index) ? " sel" : ""}`}
                      onClick={() => V298(p.index)}
                      onContextMenu={(V2b8) => {
                        V2b8.preventDefault();
                        V2b8.stopPropagation();

                        V29h((s) => (s.has(p.index) ? s : new Set([p.index])));
                        V29j({ x: V2b8.clientX, y: V2b8.clientY });
                      }}
                      title={`Trang ${p.index} — bấm để xem full · chuột phải để gộp PDF`}
                      style={{
                        cursor: "pointer",
                        outline: V29g.has(p.index) ? "2px solid var(--color-brand-500)" : undefined,
                        outlineOffset: -2,
                      }}
                    >
                      <div className="doc">
                        {p.thumbnail && (
                          <img
                            src={p.thumbnail}
                            alt={`Trang ${p.index}`}
                            className="absolute inset-0 w-full h-full object-cover"
                          />
                        )}
                        <input
                          type="checkbox"
                          checked={V29g.has(p.index)}
                          onClick={(V2b9) => V2b9.stopPropagation()}
                          onChange={() => V29h((s) => V4oh(s, p.index))}
                          title="Chọn ảnh để gộp PDF"
                          aria-label={`Chọn trang ${p.index}`}

                          className="absolute top-1.5 right-1.5 z-2 w-4.5 h-4.5 cursor-pointer"
                        />
                        <div className="pno">
                          {p.index}
                          {V2b7 && <Icon name="canhBao" size={12} />}
                        </div>
                        {V28r.has(p.index) && (
                          <div
                            title="Đã làm sạch tự động (khử nghiêng · lỗ bấm · đốm · nhiễu · vết bẩn · làm nét)"
                            className="flag"
                          >
                            <Icon name="tuDong" size={12} />
                          </div>
                        )}
                      </div>
                      <div className="meta">
                        <span className="sz">{p.paper}</span>
                        <span className={`badge ${V2b7 ? "warn" : "ok"}`}>
                          {p.dpi}
                          {p.dpiOk ? <Icon name="duyet" size={12} /> : "!"}
                        </span>
                      </div>
                    </div>
                  );
                })}
              </div>
              {V29g.size > 0 && (
                <div className="flex items-center gap-2.5 mt-2.5 flex-wrap">
                  <span className="badge info">{V29g.size} ảnh đã chọn</span>
                  <Button size="sm" variant="primary" icon="gop" loading={V29k} disabled={!V27s} onClick={() => void V2aj()}>
                    Gộp thành PDF
                  </Button>
                  <Button size="sm" onClick={() => V29h(new Set())}>
                    Bỏ chọn
                  </Button>
                  {!V27s && (
                    <span className="muted text-sm">Cần quét vào một hồ sơ để gộp PDF.</span>
                  )}
                </div>
              )}
              {V29m && (
                <div className="mt-2">
                  <span className={`muted text-sm${V29m.tone === "err" ? " text-danger-700" : ""}`}>
                    <Icon size={13} name={V29m.tone === "ok" ? "duyet" : "canhBao"} /> {V29m.text}{" "}
                    {V29m.pdfPath && (
                      <Button size="sm" onClick={() => void V40y(V29m.pdfPath as string)}>
                        Mở PDF
                      </Button>
                    )}
                  </span>
                </div>
              )}
              {V29i && (
                <div
                  role="menu"
                  className="menu-pop at-cursor"
                  style={{ top: V29i.y, left: V29i.x }}
                >
                  <button
                    className="mi"
                    disabled={V29k || V29g.size === 0 || !V27s}
                    onClick={() => void V2aj()}
                  >
                    <Icon name="gop" />
                    Gộp {V29g.size} ảnh thành PDF
                  </button>
                  <button
                    className="mi"
                    onClick={() => {
                      V29h(new Set());
                      V29j(null);
                    }}
                  >
                    Bỏ chọn tất cả
                  </button>
                </div>
              )}
              {V28t.length > 0 && (
                <div className="tally">
                  <span>
                    Khổ giấy:{" "}
                    {V2b0.map(([V2ba, n], i) => (
                      <span key={V2ba}>
                        {i > 0 ? " · " : ""}
                        <b style={V2ba !== V2av ? { color: "var(--amber)" } : undefined}>
                          {V2ba}×{n}
                        </b>
                      </span>
                    ))}
                  </span>
                  <span>
                    DPI:{" "}
                    {V2b1.map(([d, n], i) => (
                      <span key={d}>
                        {i > 0 ? " · " : ""}
                        <b style={!V4p7(d) ? { color: "var(--amber)" } : undefined}>
                          {d}×{n}
                          {V4p7(d) ? "" : " (!)"}
                        </b>
                      </span>
                    ))}
                  </span>
                </div>
              )}
            </div>
          )}
        </>
      )}

      {V27w === "review" && (
        <>
          <Banner tone="info" className="mb-3" title={`Soát lô${target ? ` ${target}` : ""}: ${V28t.length} trang, ${V2b2.length} cảnh báo`}>
            Xem lại trang có cảnh báo, sửa ảnh nếu cần, rồi lưu PDF vào hồ sơ.
          </Banner>
          <div className="review">
            <div className="preview-pane">
              <div
                className="preview-big"
                onClick={() => V2b3 && V298(V2b3.index)}
                title={V2b3 ? "Bấm để xem full" : undefined}
                style={
                  V2b3?.preview || V2b3?.thumbnail
                    ? {
                        backgroundImage: `url(${V2b3.preview || V2b3.thumbnail})`,
                        backgroundSize: "contain",
                        backgroundRepeat: "no-repeat",
                        backgroundPosition: "center",
                        cursor: "pointer",
                      }
                    : undefined
                }
              />
              <div className="mt-2 flex items-center gap-2">
                <IconButton label="Trang trước" icon="truoc" size="sm" variant="secondary" disabled={V295 <= 1} onClick={() => V296((c) => Math.max(1, c - 1))} />
                <span className="text-sm tabular-nums">
                  Trang {V295} / {V28t.length}
                </span>
                <IconButton label="Trang sau" icon="sau" size="sm" variant="secondary" disabled={V295 >= V28t.length} onClick={() => V296((c) => Math.min(V28t.length, c + 1))} />
              </div>
            </div>
            <div>
              <Card className="mb-3.5" title={`Cảnh báo (${V2b2.length})`}>
                {V2b2.length === 0 ? (
                  <p className="text-sm text-fg-subtle">Không có cảnh báo. Mọi trang đúng khổ và đạt 300 hoặc 600 DPI.</p>
                ) : (
                  <>
                    <div className="mb-2 flex flex-wrap gap-1.5">
                      {V2b2.map((p) => (
                        <Button key={p.index} size="sm" icon="canhBao" onClick={() => V296(p.index)} className={V295 === p.index ? "border-warning-300 bg-warning-50 text-warning-800" : undefined}>
                          Trang {p.index}
                        </Button>
                      ))}
                    </div>
                    {V2b3 && V2az.has(V2b3.index) && (
                      <p className="muted mt-2 mx-0 mb-3">
                        Trang {V2b3.index}: khổ {V2b3.paper}
                        {V2b3.paper !== V2av ? ` (lệch khỏi ${V2av})` : ""} · DPI{" "}
                        {V2b3.dpi}
                        {V2b3.dpiOk ? "" : ", không phải 300 hay 600"}.
                      </p>
                    )}
                    <Button size="sm" variant="primary" icon="anh" onClick={() => V2b3 && V298(V2b3.index)}>
                      Sửa ảnh trang này
                    </Button>
                  </>
                )}
              </Card>
              {V27s ? (
                V29c ? (
                  <Card tone="info" title={`Đã lưu ${V4oj(V29c.pdfPath) ? `văn bản số ${V4oj(V29c.pdfPath)}, ` : ""}${V29c.pageCount} trang, ${(V29c.pdfSize / 1024 / 1024).toFixed(2)} MB`}>
                    <p className="mb-3 break-all text-sm text-fg-subtle">{V29c.pdfPath}</p>
                    <div className="flex flex-wrap gap-2">
                      <Button size="sm" variant="primary" icon="them" onClick={() => void V2ai()}>
                        Quét văn bản tiếp
                      </Button>
                      <Button size="sm" icon="xem" onClick={() => void V40y(V29c.pdfPath)}>
                        Mở PDF
                      </Button>
                      <Button
                        size="sm"
                        icon="kho"
                        onClick={() => {
                          V27t(null);
                          goto("drive");
                        }}
                      >
                        Xem trong Kho
                      </Button>
                      <Button
                        size="sm"
                        onClick={() => {
                          V27t(null);
                          goto("tree");
                        }}
                      >
                        Về Cây thư mục
                      </Button>
                    </div>
                  </Card>
                ) : (
                  <>
                    <Banner tone="info" className="mb-2.5" title={`Lưu vào hồ sơ: ${V27s.label}`}>
                      PDF và ảnh JPEG lưu vào thư mục theo cơ quan, hộp, hồ sơ số.
                    </Banner>
                    {V29e && (
                      <Banner tone="danger" className="mb-2.5" title="Lưu PDF lỗi">
                        {V29e}
                      </Banner>
                    )}
                    <Button variant="primary" icon="luu" className="w-full" loading={V29a} disabled={V28t.length === 0 || !!V28p} onClick={() => void V2ae()}>
                      {V28p ? `Đang làm sạch ảnh ${V28p.done}/${V28p.total}…` : `Lưu PDF vào hồ sơ ${V27s.hoSo.hoSoSo}`}
                    </Button>
                  </>
                )
              ) : (
                <Button variant="primary" icon="duyet" className="w-full" disabled={!!V28p} onClick={() => goto("upload")}>
                  {V28p ? `Đang làm sạch ảnh ${V28p.done}/${V28p.total}…` : "Đưa vào hàng tải lên"}
                </Button>
              )}
            </div>
          </div>
          <Button className="mt-3" icon="truoc" onClick={() => V27x("capture")}>
            Quay lại lô đang quét
          </Button>
        </>
      )}

      {V297 !== null && (
        <V4p4
          pages={V28t}
          startIndex={V297}
          onClose={() => V298(null)}
          onApply={(V2bb, V2bc) =>
            V28u((V2bd) =>
              V2bd.map((p) =>
                p.index === V2bb
                  ? {
                      ...p,
                      preview: V2bc.preview,
                      thumbnail: V2bc.thumbnail,
                      widthPx: V2bc.widthPx,
                      heightPx: V2bc.heightPx,

                      dpi: V2bc.dpi && V2bc.dpi > 0 ? V2bc.dpi : p.dpi,
                    }
                  : p,
              ),
            )
          }
        />
      )}
    </section>
  );
}
