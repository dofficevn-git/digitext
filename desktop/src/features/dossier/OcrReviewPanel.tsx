// Copyright (c) 2026 DigiText. All rights reserved.

import { useCallback, useEffect, useState, type ReactNode } from "react";
import "./laphoso.css";
import { V3xq } from "@/api/ipc";
import { V40x, V40p } from "@/api/archive";
import { V4bz, V4by, V4c0, type V4bm, type V4ca, type V4bp } from "@/api/dossier";
import { V4ei } from "@/api/docs";
import { V4fg, V4fk, V4fl, type V4fj } from "@/api/ocr";
import { V4cy, V4cw, V4cz, V4cx } from "./logic";
import { V4ge, V4gf, V4gj, V4gi } from "./ocrPrefill";
import V4h3 from "./OcrBoxLayer";
import type { V4gk } from "./ocrBoxes";
import { Icon } from "@/components/Icon";

function V4h4(path: string): string {
  const V1bx = path.split(/[\\/]/);
  return (V1bx[V1bx.length - 1] ?? "").toLowerCase();
}

function V4h5(V1by: unknown): string {
  return V1by instanceof V3xq || V1by instanceof Error ? V1by.message : String(V1by);
}

function V4h6({ label: V1bz, children: V1c0 }: { label: string; children: ReactNode }) {
  return (
    <div className="col-lg-12">
      <div className="input-group groupwidth">
        <label className="input-group-addon">
          <b>{V1bz}</b>
        </label>
        {V1c0}
      </div>
    </div>
  );
}

export default function V4h7({
  dossier,
  file,
  onBack: V1c1,
}: {
  dossier: V4bm;
  file: V4ca;
  onBack: () => void;
}) {
  const [V1c2, V1c3] = useState<string[]>([]);
  const [V1c4, V1c5] = useState<string | null>(null);
  const [V1c6, V1c7] = useState<V4bp>(V4cw());
  const [V1c8, V1c9] = useState<number | null>(null);
  const [V1ca, V1cb] = useState(true);
  const [V1cc, V1cd] = useState(false);
  const [V1ce, V1cf] = useState<string | null>(null);
  const [V1cg, V1ch] = useState<string | null>(null);

  const [V1ci, V1cj] = useState<string | null>(null);

  const [ocr, V1ck] = useState<Record<number, V4fj>>({});
  const [V1cl, V1cm] = useState<Record<number, V4gk[]>>({});
  const [V1cn, V1co] = useState<Set<number>>(new Set());
  const [V1cp, V1cq] = useState<string | null>(null);
  const [V1cr, V1cs] = useState<number | null>(null);

  const V1ct = async (V1cu: number) => {
    V1cs(V1cu);
    V1cf(null);
    try {
      const V1cv = await V4fk(file.filePath, V1cu + 1);
      V1ck((m) => ({ ...m, [V1cu]: V1cv }));
      V1cm((m) => ({ ...m, [V1cu]: V1cv.boxes.map((b) => ({ ...b })) }));
      V1ch(`OCR trang ${V1cu + 1}: ${V1cv.boxes.length} box.`);
    } catch (V1cw) {
      V1cf(V4h5(V1cw));
    } finally {
      V1cs(null);
    }
  };

  const V1cx = (V1cy: number, V1cz: V4gk) => {
    V1cm((m) => ({
      ...m,
      [V1cy]: (m[V1cy] ?? []).map((b) => (b.key === V1cz.key ? V1cz : b)),
    }));
    V1co((s) => new Set(s).add(V1cy));
  };

  const V1d0 = async (V1d1: number) => {
    const V1d2 = V1cl[V1d1];
    if (!V1d2) return;
    V1cf(null);
    try {
      await V4fl(
        file.filePath,
        V1d1 + 1,
        V1d2.map((b) => ({
          key: b.key,
          x: b.x,
          y: b.y,
          w: b.w,
          h: b.h,
          text: b.text ?? "",
          conf: b.conf ?? null,
        })),
      );
      V1co((s) => {
        const n = new Set(s);
        n.delete(V1d1);
        return n;
      });
      V1ch(
        `xJDDoyBsxrB1IA==${V1d2.length}IGJveCAoZOG7ryBsaeG7h3UgdsOgbmcpIOKAlCB0cmFuZyA=${V1d1 + 1}.`,
      );
    } catch (V1d3) {
      V1cf(V4h5(V1d3));
    }
  };

  const V1d4 = useCallback(async () => {
    const V1d5 = await V4by(dossier.id);
    const V1d6 = V4h4(file.fileName);
    const V1d7 = V1d5.find((v) => v.pdfPath && V4h4(v.pdfPath) === V1d6);
    let V1d8 = V1d7 ? V4cx(V1d7) : V4cw();
    V1c9(V1d7 ? V1d7.id : null);
    let V1d9: string | null = null;
    try {
      const docs = await V4ei();
      const doc = V4gf(docs, V4ge(dossier), file.fileName);
      if (doc) {
        const fields = await V4fg(doc.id);
        const r = V4gi(V1d8, fields);
        if (r.filled.length > 0) {
          V1d8 = r.draft;
          V1d9 = `xJDDoyDEkWnhu4FuIHPhurVuIHThu6sgT0NSIG3DoXkgY2jhu6c6IA==${V4gj(r.filled)}LiBLaeG7g20gdHJhIHLhu5NpIGLhuqVtIEzGsHUgxJHhu4MgZ2hpIHbDoG8gQ1NETC4=`;
        }
      }
    } catch {}
    V1c7(V1d8);
    V1cj(V1d9);
  }, [dossier, file.fileName]);

  useEffect(() => {
    let V1da = true;
    V1cb(true);
    V1cf(null);
    V1ch(null);
    Promise.all([
      V40p(file.filePath)
        .then((p) => {
          if (V1da) {
            V1c3(p);
            V1c5(null);
          }
        })
        .catch((V1db) => {
          if (V1da) {
            V1c3([]);
            V1c5(V4h5(V1db));
          }
        }),
      V1d4().catch((V1dc) => V1da && V1cf(V4h5(V1dc))),
    ]).finally(() => {
      if (V1da) V1cb(false);
    });
    return () => {
      V1da = false;
    };
  }, [file.filePath, V1d4]);

  const set = (p: Partial<V4bp>) => V1c7((d) => ({ ...d, ...p }));

  const V1dd = (s: string) => (s.trim() === "" ? undefined : Number(s));

  const V1de = async () => {
    const v = V4cz(V1c6);
    if (v) {
      V1cf(v);
      V1ch(null);
      return;
    }
    V1cd(true);
    V1cf(null);
    V1ch(null);
    try {
      const body = V4cy(V1c6);
      if (V1c8 != null) {
        await V4c0(V1c8, body);
      } else {
        const V1df = await V4bz(dossier.id, { ...body, pdfPath: file.filePath });
        V1c9(V1df);
      }
      await V1d4();
      V1ch("xJDDoyBsxrB1Lg==");
    } catch (V1dg) {
      V1cf(V4h5(V1dg));
    } finally {
      V1cd(false);
    }
  };

  return (
    <div className="lhs">
      <div className="content-panel">
        <div className="mb-2 flex items-center gap-2.5">
          <button className="btn btn-outline-primary btn-round" onClick={V1c1}>
            <Icon name="hoanTac" /> Về danh sách
          </button>
          <b className="overflow-hidden text-ellipsis whitespace-nowrap">{file.fileName}</b>
        </div>

        {V1ce && <p className="muted text-danger-700">{V1ce}</p>}
        {V1cg && <p className="muted text-success-700">{V1cg}</p>}
        {V1ci && (
          <p className="muted text-info-700">
            <Icon name="tuDong" /> {V1ci}
          </p>
        )}

        <div className="ocr-view">
          {}
          <div className="ocr-left">
            {V1ca ? (
              <div className="ocr-empty">Đang tải văn bản…</div>
            ) : V1c2.length > 0 ? (
              V1c2.map((src, i) => (
                <div key={i} className="ocr-page">
                  <div className="ocr-page-tools flex gap-2 items-center my-1 mx-0">
                    <button
                      className="btn btn-sm btn-outline-primary"
                      disabled={V1cr !== null}
                      onClick={() => V1ct(i)}
                      title="Q2jhuqF5IE9DUiAoVHJpdG9uKSDEkeG7gyBiw7NjIHTDoWNoIGJveCB0csOqbiB0cmFuZyBuw6B5"
                    >
                      <Icon name="tuDong" />{" "}
                      {V1cr === i
                        ? "xJBhbmcgT0NS4oCm"
                        : ocr[i]
                          ? "T0NSIGzhuqFp"
                          : "T0NSIGLDs2MgdMOhY2ggYm94"}
                    </button>
                    {ocr[i] && (
                      <span className="muted">{V1cl[i]?.length ?? 0} box — kéo/chỉnh khung</span>
                    )}
                    {V1cn.has(i) && (
                      <button className="btn btn-sm btn-success" onClick={() => V1d0(i)}>
                        <Icon name="luu" /> Lưu box (gold)
                      </button>
                    )}
                  </div>
                  {ocr[i] ? (
                    <V4h3
                      src={src}
                      boxes={V1cl[i] ?? []}
                      pageW={ocr[i].pageW}
                      pageH={ocr[i].pageH}
                      selectedKey={V1cp}
                      onSelect={V1cq}
                      onChange={(b) => V1cx(i, b)}
                    />
                  ) : (
                    <img src={src} alt={`Trang ${i + 1}`} draggable={false} />
                  )}
                </div>
              ))
            ) : (
              <div className="ocr-empty">
                {V1c4 ??
                  "S2jDtG5nIGhp4buDbiB0aOG7iyDEkcaw4bujYyBu4buZaSBkdW5nIChraMO0bmcgcGjhuqNpIFBERiDhuqNuaCku"}
                <div className="mt-2.5">
                  <a
                    className="ic-word"
                    onClick={() => V40x(file.filePath).catch((V1dh) => V1cf(V4h5(V1dh)))}
                    title="TeG7nyBi4bqxbmcgdHLDrG5oIHhlbSBQREY="
                  >
                    <Icon name="tepPdf" /> Mở file
                  </a>
                </div>
              </div>
            )}
          </div>

          {}
          <div className="ocr-right" id="rightpanel">
            <V4h6 label="U+G7kSB0aOG7qSB04buxIFZC">
              <input
                type="number"
                className="form-control pull-right"
                value={V1c6.thuTu ?? 0}
                onChange={(V1di) => set({ thuTu: Number(V1di.target.value) || 0 })}
              />
            </V4h6>
            <V4h6 label="TG/huqFpIHbEg24gYuG6o24=">
              <input
                type="text"
                className="form-control pull-right"
                value={V1c6.loaiVanBan ?? ""}
                onChange={(V1dj) => set({ loaiVanBan: V1dj.target.value })}
              />
            </V4h6>
            <V4h6 label="U+G7kSBWxINuIELhuqNu">
              <input
                type="text"
                className="form-control pull-right"
                value=""
                readOnly
                title="RGVza3RvcCBn4buZcCBz4buRICsga8O9IGhp4buHdSB2w6BvIMO0ICdLSCB2xINuIGLhuqNuJw=="
              />
            </V4h6>
            <V4h6 label="S0ggdsSDbiBi4bqjbg==">
              <input
                type="text"
                className="form-control pull-right"
                value={V1c6.soKyHieu ?? ""}
                onChange={(V1dk) => set({ soKyHieu: V1dk.target.value })}
              />
            </V4h6>
            <V4h6 label="TmfDoHkgdGjDoW5n">
              <input
                type="text"
                className="form-control pull-right"
                placeholder="dd/mm/yyyy"
                value={V1c6.ngayThang ?? ""}
                onChange={(V1dl) => set({ ngayThang: V1dl.target.value })}
              />
            </V4h6>
            <V4h6 label="Q1EgYmFuIGjDoG5o">
              <input
                type="text"
                className="form-control pull-right"
                value={V1c6.coQuanBanHanh ?? ""}
                onChange={(V1dm) => set({ coQuanBanHanh: V1dm.target.value })}
              />
            </V4h6>
            <V4h6 label="VHLDrWNoIHnhur91">
              <textarea
                className="form-control pull-right"
                rows={5}
                value={V1c6.trichYeu ?? ""}
                onChange={(V1dn) => set({ trichYeu: V1dn.target.value })}
              />
            </V4h6>
            <V4h6 label="VOG7nSBz4buR">
              <input
                type="number"
                className="form-control pull-right"
                value={V1c6.toSo ?? ""}
                onChange={(V1do) => set({ toSo: V1dd(V1do.target.value) })}
              />
            </V4h6>
            <V4h6 label="U+G7kSB0cmFuZw==">
              <input
                type="number"
                className="form-control pull-right"
                value={V1c6.soTrang ?? ""}
                onChange={(V1dp) => set({ soTrang: V1dd(V1dp.target.value) })}
              />
            </V4h6>
            <V4h6 label="R2hpIGNow7o=">
              <input
                type="text"
                className="form-control pull-right"
                value={V1c6.ghiChu ?? ""}
                onChange={(V1dq) => set({ ghiChu: V1dq.target.value })}
              />
            </V4h6>
            <V4h6 label="TmfDtG4gbmfhu68=">
              <input
                type="text"
                className="form-control pull-right"
                value={V1c6.ngonNgu ?? ""}
                onChange={(V1dr) => set({ ngonNgu: V1dr.target.value })}
              />
            </V4h6>
            <V4h6 label="Q8SQIHPhu60gZOG7pW5n">
              <input
                type="text"
                className="form-control pull-right"
                value={V1c6.cheDoSuDung ?? ""}
                onChange={(V1ds) => set({ cheDoSuDung: V1ds.target.value })}
              />
            </V4h6>
            <V4h6 label="xJDhu5kgdGluIGPhuq15">
              <input
                type="text"
                className="form-control pull-right"
                value={V1c6.doTinCay ?? ""}
                onChange={(V1dt) => set({ doTinCay: V1dt.target.value })}
              />
            </V4h6>
            <V4h6 label="QsO6dCB0w61jaA==">
              <input
                type="text"
                className="form-control pull-right"
                value={V1c6.butTich ?? ""}
                onChange={(V1du) => set({ butTich: V1du.target.value })}
              />
            </V4h6>
            <V4h6 label="VFQgduG6rXQgbMO9">
              <input
                type="text"
                className="form-control pull-right"
                value={V1c6.tinhTrangVatLy ?? ""}
                onChange={(V1dv) => set({ tinhTrangVatLy: V1dv.target.value })}
              />
            </V4h6>

            <div className="col-lg-12 text-center mt-1.5">
              <button
                className="btn btn-outline-success btn-round"
                onClick={() => void V1de()}
                disabled={V1cc}
              >
                <Icon name="luu" /> {V1cc ? "xJBhbmcgbMawdeKApg==" : "TMawdQ=="}
              </button>{" "}
              <button className="btn btn-outline-danger btn-round" onClick={V1c1} disabled={V1cc}>
                <Icon name="dong" /> Đóng
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
