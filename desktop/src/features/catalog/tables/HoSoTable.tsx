// Copyright (c) 2026 DigiText. All rights reserved.

import { useCallback, useEffect, useMemo, useState } from "react";
import { V41p } from "@/live/useLive";
import V49a from "@/ui/DataTable";
import Modal, { V49j } from "@/ui/Modal";
import { V460 } from "@/state";
import { isTauri } from "@/api/ipc";
import { V40h, V40t, V40f, listHoSo, V40y, V40u } from "@/api/archive";
import { V49w, V49x, type V49v } from "@/api/sync";
import { V4a0, V4a4, V4a1, V4a2, V4a5, type V49z, type V49y } from "@/api/lan";
import { V4a6, V4a7 } from "@/features/catalog/treePeerAddr";
import { V432, V433 } from "@/api/lansync";
import { V4ak, type V4a9 } from "@/api/users";
import { V3z9 } from "@/domain/permissions";
import { V4aw, V4at, V4b0 } from "@/domain/hoSoTime";
import type { V3yr, V3yk } from "@/domain/types";
import V4bg from "@/features/catalog/ImportExcelDialog";
import { V48w } from "@/features/catalog/hoSoMapping";
import { V4bi } from "@/features/catalog/dongBoMucLuc";
import { chuaChonPhong } from "@/features/dossier/phongScope";
import { V49l, V49o, V49n, type V49k } from "./shared";
import { Icon } from "@/components/Icon";

export function HoSoTable({ data, setData: Vsi }: V49k) {
  const { setScanTarget: Vsj, goto, resetToRoot: Vsk, session: Vsl } = V460();

  const Vsm = V49n("dossier.create", "dossier.update", "dossier.delete");

  const Vsn = V3z9(Vsl?.permissions, "tree.assign");
  const Vso = V3z9(Vsl?.permissions, "scan.execute");
  const Vsp = Vsl?.role === "user";

  const [Vsq, Vsr] = useState(false);
  const [Vss, Vst] = useState<V4a9[]>([]);
  const [Vsu, Vsv] = useState("");
  const [Vsw, Vsx] = useState("");
  const [Vsy, Vsz] = useState(false);

  const [Vt0, Vt1] = useState<V49y | null>(null);
  const [Vt2, Vt3] = useState(false);
  const [Vt4, Vt5] = useState<V49z[]>([]);
  const [Vt6, Vt7] = useState<string>("");

  const [Vt8, Vt9] = useState("");
  const [Vta, Vtb] = useState("");
  const [Vtc, Vtd] = useState(false);
  const [Vte, Vtf] = useState(false);
  const [Vtg, Vth] = useState<string>("all");
  const [scanPhong, Vti] = useState<string>("");
  const [Vtj, Vtk] = useState<Record<string, { pdfPath: string; status: string }>>({});
  const [Vtl, Vtm] = useState<{ mode: "add" | "edit"; draft: V3yr; orig?: number } | null>(null);

  useEffect(() => {
    if (!scanPhong && data.phongs[0]) Vti(data.phongs[0].maDuAn);
  }, [data.phongs, scanPhong]);

  const reload = useCallback(() => {
    if (!isTauri()) return;

    if (chuaChonPhong(scanPhong)) {
      Vsi((s) => ({ ...s, hoSos: [] }));
      Vtk({});
      return;
    }
    listHoSo(scanPhong)
      .then((rs) => {
        Vsi((s) => ({ ...s, hoSos: rs.map(V48w) }));
        const m: Record<string, { pdfPath: string; status: string }> = {};
        for (const r of rs)
          if (r.pdfPath) m[`${r.hopSo}|${r.hoSoSo}`] = { pdfPath: r.pdfPath, status: r.status };
        Vtk(m);
      })
      .catch(() => {});
  }, [Vsi, scanPhong]);

  V41p(reload, { topics: ["hoso", "catalog"] });

  const Vtn = () => Math.max(0, ...data.hoSos.map((h) => h.id)) + 1;
  const Vto = Array.from(new Set(data.hoSos.map((h) => h.nguon).filter((x): x is string => !!x)));

  const Vtp = useMemo(
    () => V4b0(Vtg === "all" ? data.hoSos : data.hoSos.filter((h) => h.nguon === Vtg)),
    [data.hoSos, Vtg],
  );

  const Vtq = data.phongs.find((p) => p.maDuAn === scanPhong);
  const Vtr = (p: V3yk) => ({
    maDuAn: p.maDuAn,
    tenDuAn: p.tenDuAn,
    maKhoi: p.maKhoi as string,
    phongSo: p.phongSo,
    mucLucSo: p.mucLucSo,
  });
  const Vts = (h: V3yr) => ({
    hopSo: h.hopSo,
    hoSoSo: h.hoSoSo,
    tieuDe: h.tieuDe,
    thoiGian: h.thoiGian,
    soTo: h.soTo,
    thbq: h.thbq,
    ghiChu: h.ghiChu,
    nhom: h.nhom,
    nam: h.nam,
    ngayBatDau: h.ngayBatDau,
    ngayKetThuc: h.ngayKetThuc,
  });

  const Vtt = (h: V3yr) => {
    if (!Vtq) return;
    Vsj({
      phong: Vtr(Vtq),
      hoSo: Vts(h),
      label: `${Vtq.tenDuAn}IMK3IEjhu5lwIA==${h.hopSo} · HS ${h.hoSoSo}`,
    });
    goto("scan");
  };

  const Vtu = async () => {
    if (!Vtq) return;
    if (isTauri()) {
      try {
        await V40h(Vtr(Vtq), Vtp.map(Vts));

        Vsk();
      } catch {}
    }
    goto("drive");
  };

  const Vtv = (Vtw: unknown) => (Vtw instanceof Error ? Vtw.message : String(Vtw));

  const Vtx = async () => {
    Vsx("");
    Vsr(true);
    try {
      const Vty = await V4ak();
      Vst(Vty);
      if (Vty.length === 1) Vsv(Vty[0].id);
    } catch (Vtz) {
      Vsx(Vtv(Vtz));
    }
  };

  const Vu0 = async () => {
    if (!Vtq || !Vsu || Vsy) return;
    Vsz(true);
    Vsx("");
    try {
      const Vu1 = await V49w(Vsu, Vtr(Vtq), Vtp.map(Vts));
      Vsr(false);
      Vsx(
        `xJDDoyBwaMOhdCBjw6J5IHRoxrAgbeG7pWMgKHBoacOqbiBi4bqjbiA=${Vu1}KSDigJQgbcOheSB1c2VyIHPhur0gbmjhuq1uIGtoaSBi4bqlbSAiTmjhuq1uIGPDonkgdGjGsCBt4bulYyIu`,
      );
    } catch (Vu2) {
      Vsx(Vtv(Vu2));
    } finally {
      Vsz(false);
    }
  };

  const Vu3 = async () => {
    if (Vsy) return;
    Vsz(true);
    Vsx("");
    try {
      const r = await V49x();
      if (r.applied) {
        Vsk();
        reload();
        Vsx(
          `xJDDoyBuaOG6rW4gY8OieSB0aMawIG3hu6VjIG3hu5tpIChwaGnDqm4gYuG6o24g${r.version}KSDigJQgdOG6oW8g${r.created}IGjhu5Mgc8ahLg==`,
        );
        goto("drive");
      } else {
        Vsx("Q8OieSB0aMawIG3hu6VjIMSRw6MgbMOgIGLhuqNuIG3hu5tpIG5o4bqldC4=");
      }
    } catch (Vu4) {
      Vsx(Vtv(Vu4));
    } finally {
      Vsz(false);
    }
  };

  const Vu5 = async () => {
    if (Vsy) return;
    Vsz(true);
    Vsx("");
    try {
      if (Vt0?.publishing) {
        await V4a2();
        Vt1(null);
        Vsx("xJDDoyBk4burbmcgcGjDoXQgcXVhIExBTi4=");
      } else if (Vtq) {
        const Vu6 = await V4a1(Vtq.maDuAn, 1, Vtr(Vtq), Vtp.map(Vts));
        Vt1(Vu6);
      }
    } catch (Vu7) {
      Vsx(Vtv(Vu7));
    } finally {
      Vsz(false);
    }
  };

  const Vu8 = async () => {
    Vt3(true);
    Vt5([]);
    Vt7("");
    Vtd(true);
    Vsx("");
    try {
      Vt5(await V4a4());
    } catch (Vu9) {
      Vsx(Vtv(Vu9));
    } finally {
      Vtd(false);
    }
  };

  const Vua = async () => {
    if (Vta.trim().length < 6 || Vsy) return;
    const Vub = Vt8.trim();
    let target: { addr: string; port: number } | null = null;
    if (Vub) {
      const Vuc = V4a7(Vub);
      if (Vuc) {
        Vsx(Vuc);
        return;
      }
      target = V4a6(Vub);
    } else {
      const Vud = Vt4.find((p) => p.instance === Vt6);
      if (Vud) target = { addr: Vud.addr, port: Vud.port };
    }
    if (!target) return;
    Vsz(true);
    Vsx("");
    try {
      const r = await V4a5(target.addr, target.port, Vta.trim());
      Vt3(false);
      Vtb("");
      Vt9("");

      Vsk();
      reload();
      Vsx(
        `xJDDoyBuaOG6rW4gY8OieSB0aMawIG3hu6VjIHF1YSBMQU4g4oCUIHThuqFvIA==${r.created}IGjhu5Mgc8ahLg==`,
      );
      goto("drive");
      await Vue(r);
    } catch (Vuf) {
      Vsx(Vtv(Vuf));
    } finally {
      Vsz(false);
    }
  };

  const Vue = async (r: V49v) => {
    const Vug = r.offeredSiteCode?.trim();
    if (!Vug || !isTauri()) return;
    const Vuh = (await V432())?.trim() ?? "";
    if (Vuh === Vug) return;
    const Vui = r.peerFingerprint ?? "KGtow7RuZyByw7Up";
    const Vuj = Vuh
      ? `TcOheSBuw6B5IMSQQU5HIGTDuW5nIG3hu5l0IG3DoyBzaXRlIGtow6FjLiBOaOG6rW4gbcOjIG3hu5tpIHPhur0gY2h1eeG7g24gbcOheSBzYW5nIHNpdGUgY+G7p2EgbcOheSBwaMOhdCDEkcOzLgoK`
      : "";
    const Vuk = window.confirm(
      `${Vuj}TcOheSBwaMOhdCBjw6J5IGPDsyB2w6JuIHRheToKCiAgICA=${Vui}\n\n` +
        "SMOjeSDEkeG7kWkgY2hp4bq/dSBjaHXhu5dpIG7DoHkgduG7m2kgY2h14buXaSBxdeG6o24gdHLhu4sgxJHhu41jIGNobyBi4bqhbiAobcOgbiDigJzEkOG7k25nIGLhu5kgZOG7ryBsaeG7h3XigJ0gdHLDqm4gbcOheSA=" +
        "cXXhuqNuIHRy4buLIGhp4buDbiB0aOG7iyDEkcO6bmcgY2h14buXaSDEkcOzKS4KCk5o4bqtbiBtw6Mgc2l0ZSBMQU4gdOG7qyBtw6F5IG7DoHk/",
    );
    if (!Vuk) {
      Vsx(
        "xJDDoyBuaOG6rW4gY8OieSB0aMawIG3hu6VjLCBuaMawbmcgQuG7jiBRVUEgbcOjIHNpdGUgKGLhuqFuIGtow7RuZyB4w6FjIG5o4bqtbiB2w6JuIHRheSBtw6F5IHBow6F0KS4=",
      );
      return;
    }
    try {
      await V433(Vug, true);
      Vsx(
        "xJDDoyBuaOG6rW4gY8OieSB0aMawIG3hu6VjIHbDoCBtw6Mgc2l0ZSBMQU4g4oCUIG3DoXkgbsOgeSBi4bqvdCDEkeG6p3UgY2hpYSBz4bq7IGNobyBxdeG6o24gdHLhu4su",
      );
    } catch (Vul) {
      Vsx(Vtv(Vul));
    }
  };

  const set = (p: Partial<V3yr>) => Vtl && Vtm({ ...Vtl, draft: { ...Vtl.draft, ...p } });

  const Vum = (h: V3yr) => {
    const { batDau: Vun, ketThuc: Vuo } = V4at(h);
    Vtm({
      mode: "edit",
      draft: { ...h, ngayBatDau: Vun, ngayKetThuc: Vuo },
      orig: h.id,
    });
  };
  const Vup = () => {
    if (!Vtl || !Vtl.draft.tieuDe.trim()) return;
    const Vuq = V4aw(Vtl.draft);
    if (isTauri()) {
      if (!Vuq.hoSoSo.trim()) {
        window.alert(
          "VnVpIGzDsm5nIG5o4bqtcCBI4buTIHPGoSBz4buRIChraMOzYSDEkeG7i25oIGRhbmggaOG7kyBzxqEpLg==",
        );
        return;
      }
      if (Vtl.mode === "edit" && Vtl.orig != null) {
        V40u(Vtl.orig, Vts(Vuq)).then(reload).catch(V49l);
      } else {
        if (!Vtq) {
          window.alert(
            "VnVpIGzDsm5nIGNo4buNbiBwaMO0bmcgKEPGoSBxdWFuKSDhu58gYuG7mSBs4buNYyB0csaw4bubYyBraGkgdGjDqm0gaOG7kyBzxqEgbeG7m2ku",
          );
          return;
        }
        V40f(Vtr(Vtq), [Vts(Vuq)])
          .then((r) => {
            Vsx(V4bi(r.dossier));
            reload();
          })
          .catch(V49l);
      }
    } else {
      Vsi((s) =>
        Vtl.mode === "add"
          ? { ...s, hoSos: [...s.hoSos, Vuq] }
          : { ...s, hoSos: s.hoSos.map((x) => (x.id === Vtl.orig ? Vuq : x)) },
      );
    }
    Vtm(null);
  };
  const Vur = (Vus: number) => {
    if (isTauri()) {
      V40t(Vus).then(reload).catch(V49l);
    } else {
      Vsi((s) => ({ ...s, hoSos: s.hoSos.filter((x) => x.id !== Vus) }));
    }
  };
  const Vut = (Vuu: (string | number)[]) => {
    if (isTauri()) {
      Promise.all(Vuu.map((Vuv) => V40t(Number(Vuv))))
        .then(reload)
        .catch(V49l);
    } else {
      Vsi((s) => ({ ...s, hoSos: s.hoSos.filter((h) => !Vuu.includes(h.id)) }));
    }
  };
  const Vuw = (records: V3yr[]) => {
    if (isTauri()) {
      if (!Vtq) {
        window.alert(
          "VnVpIGzDsm5nIGNo4buNbiBwaMO0bmcgKEPGoSBxdWFuKSDhu58gYuG7mSBs4buNYyB0csaw4bubYyBraGkgbmjhuq1wIMSR4buDIGzGsHUgdsOgbyBjxqEgc+G7nyBk4buvIGxp4buHdS4=",
        );
        return;
      }
      V40f(Vtr(Vtq), records.map(Vts))
        .then((r) => {
          Vsx(`xJDDoyBuaOG6rXAg${r.hoSo}IGjhu5Mgc8ahIHbDoG8gZGFuaCBt4bulYy4g${V4bi(r.dossier)}`);
          reload();
        })
        .catch(V49l);
    } else {
      Vsi((s) => ({ ...s, hoSos: [...s.hoSos, ...records] }));
    }
  };

  return (
    <>
      {Vsw && <div className="muted mt-0 mx-0 mb-2.5 text-sm">{Vsw}</div>}
      {Vt0?.publishing && (
        <div className="mt-0 mx-0 mb-2.5 rounded-lg bg-brand-50 px-3 py-2 text-sm text-brand-600">
          <Icon name="phatLan" /> Đang phát qua LAN · mã ghép nối:{" "}
          <b className="text-lg tracking-[0.12em]">{Vt0.code}</b> · đọc mã này cho user để họ nhận
          trong mạng nội bộ.
        </div>
      )}
      <V49a<V3yr>
        addLabel="VGjDqm0gaOG7kyBzxqE="
        onAdd={
          !Vsm.canCreate
            ? undefined
            : () =>
                Vtm({
                  mode: "add",
                  draft: {
                    id: Vtn(),
                    hopSo: "",
                    hoSoSo: "",
                    tieuDe: "",
                    thoiGian: "",
                    soTo: "",
                    thbq: "",
                  },
                })
        }
        extraActions={
          <>
            {Vsn && (
              <button
                className="btn"
                disabled={!Vtq || Vtp.length === 0}
                title="VOG6oW8gdGjGsCBt4bulYyBDxqEgcXVhbiDihpIgSOG7mXAg4oaSIEjhu5Mgc8ahIHbDoCBoaeG7g24gdGjhu4sgdHJvbmcgS2hvIHTDoGkgbGnhu4d1"
                onClick={() => void Vtu()}
              >
                <Icon name="thuMuc" />
                Tạo cây thư mục
              </button>
            )}
            {Vsn && (
              <button
                className="btn"
                disabled={!Vtq || Vtp.length === 0 || Vsy}
                title="UGjDoXQgY8OieSB0aMawIG3hu6VjIGzDqm4gbcOheSBjaOG7pyDEkeG7gyBtw6F5IHVzZXIgbmjhuq1uIHbhu4E="
                onClick={() => void Vtx()}
              >
                <Icon name="phatChoUser" />
                Phát cho user
              </button>
            )}
            {Vsn && (
              <button
                className="btn"
                disabled={(!Vtq && !Vt0?.publishing) || Vsy}
                title="UGjDoXQgY8OieSB0aMawIG3hu6VjIHF1YSBt4bqhbmcgbuG7mWkgYuG7mSAoTEFOKSDigJQgY2hvIHNpdGUga2jDtG5nIGPDsyBtw6F5IGNo4bun"
                onClick={() => void Vu5()}
              >
                {Vt0?.publishing ? (
                  <>
                    <Icon name="dungHan" />
                    Dừng phát LAN
                  </>
                ) : (
                  <>
                    <Icon name="phatLan" />
                    Phát qua LAN
                  </>
                )}
              </button>
            )}
            {Vsp && (
              <button
                className="btn"
                disabled={Vsy}
                title="Tmjhuq1uIGPDonkgdGjGsCBt4bulYyBxdeG6o24gdHLhu4sgxJHDoyBwaMOhdCBjaG8gZOG7sSDDoW4gY+G7p2EgYuG6oW4gKHF1YSBtw6F5IGNo4bunKQ=="
                onClick={() => void Vu3()}
              >
                <Icon name="taiVe" />
                Nhận cây thư mục
              </button>
            )}
            {Vsp && (
              <button
                className="btn"
                disabled={Vsy}
                title="VMOsbSBtw6F5IHF14bqjbiB0cuG7iyDEkWFuZyBwaMOhdCBjw6J5IHRoxrAgbeG7pWMgdHJvbmcgbeG6oW5nIG7hu5lpIGLhu5kgKExBTik="
                onClick={() => void Vu8()}
              >
                <Icon name="tim" />
                Nhận qua LAN
              </button>
            )}
            {Vsm.canCreate && (
              <button className="btn" onClick={() => Vtf(true)}>
                <Icon name="nhapTep" />
                Nhập Excel
              </button>
            )}
          </>
        }
        rows={Vtp}
        getId={(h) => h.id}
        matches={(h, q) =>
          h.tieuDe.toLowerCase().includes(q) ||
          h.hoSoSo.toLowerCase().includes(q) ||
          h.hopSo.toLowerCase().includes(q)
        }
        extraFilters={
          <div className="dt-filter">
            <label>Cơ quan:</label>
            <select className="inp" value={scanPhong} onChange={(Vux) => Vti(Vux.target.value)}>
              {data.phongs.map((p) => (
                <option key={p.maDuAn} value={p.maDuAn}>
                  {p.tenDuAn}
                </option>
              ))}
            </select>
            {Vto.length > 0 && (
              <>
                <label>Nguồn:</label>
                <select className="inp" value={Vtg} onChange={(Vuy) => Vth(Vuy.target.value)}>
                  <option value="all">— Tất cả —</option>
                  {Vto.map((s) => (
                    <option key={s} value={s}>
                      {s}
                    </option>
                  ))}
                </select>
              </>
            )}
          </div>
        }
        columns={[
          { head: "SOG7mXAgc+G7kQ==", cell: (h) => h.hopSo, cls: "tnum" },
          { head: "SOG7kyBzxqEgc+G7kQ==", cell: (h) => h.hoSoSo, cls: "tnum" },
          { head: "VGnDqnUgxJHhu4EgaOG7kyBzxqE=", cell: (h) => h.tieuDe },
          { head: "QuG6r3QgxJHhuqd1", cell: (h) => V4at(h).batDau, cls: "tnum" },
          { head: "S+G6v3QgdGjDumM=", cell: (h) => V4at(h).ketThuc, cls: "tnum" },
          { head: "U+G7kSB04bud", cell: (h) => h.soTo, cls: "num" },
          { head: "THBQ", cell: (h) => h.thbq },
          {
            head: "TmjDs20gLyBOxINt",
            cell: (h) => (
              <span className="muted">{[h.nhom, h.nam].filter(Boolean).join(" · ")}</span>
            ),
          },
        ]}
        rowActions={(h) => {
          const Vuz = Vtj[`${h.hopSo}|${h.hoSoSo}`];
          return V49o(
            () => Vum(h),
            () => Vur(h.id),
            <>
              {Vso && (
                <button className="linkbtn text-brand-500" disabled={!Vtq} onClick={() => Vtt(h)}>
                  <Icon name="quet" /> Scan
                </button>
              )}
              {Vuz && (
                <button className="linkbtn add2" onClick={() => void V40y(Vuz.pdfPath)}>
                  <Icon name="xem" /> PDF
                </button>
              )}
            </>,
            Vsm,
          );
        }}
        onDeleteMany={Vsm.canDelete ? Vut : undefined}
      />

      {Vte && <V4bg startId={Vtn()} onClose={() => Vtf(false)} onImport={Vuw} />}

      {Vtl && (
        <Modal
          title={Vtl.mode === "add" ? "VGjDqm0gaOG7kyBzxqE=" : "U+G7rWEgaOG7kyBzxqE="}
          onClose={() => Vtm(null)}
          onSave={Vup}
        >
          <V49j label="SOG7mXAgc+G7kQ==">
            <input
              className="inp"
              value={Vtl.draft.hopSo}
              onChange={(Vv0) => set({ hopSo: Vv0.target.value })}
            />
          </V49j>
          <V49j label="SOG7kyBzxqEgc+G7kQ==">
            <input
              className="inp"
              value={Vtl.draft.hoSoSo}
              onChange={(Vv1) => set({ hoSoSo: Vv1.target.value })}
            />
          </V49j>
          <V49j label="VGnDqnUgxJHhu4EgaOG7kyBzxqE=">
            <input
              className="inp"
              value={Vtl.draft.tieuDe}
              onChange={(Vv2) => set({ tieuDe: Vv2.target.value })}
            />
          </V49j>
          <V49j label="VGjhu51pIGdpYW4gYuG6r3QgxJHhuqd1">
            <input
              className="inp"
              placeholder="VkQ6IDAxLzAxLzIwMTAgaG/hurdjIDIwMTA="
              value={Vtl.draft.ngayBatDau ?? ""}
              onChange={(Vv3) => set({ ngayBatDau: Vv3.target.value })}
            />
          </V49j>
          <V49j label="VGjhu51pIGdpYW4ga+G6v3QgdGjDumM=">
            <input
              className="inp"
              placeholder="VkQ6IDMxLzEyLzIwMTIgaG/hurdjIDIwMTI="
              value={Vtl.draft.ngayKetThuc ?? ""}
              onChange={(Vv4) => set({ ngayKetThuc: Vv4.target.value })}
            />
          </V49j>
          <V49j label="U+G7kSB04bud">
            <input
              className="inp"
              value={Vtl.draft.soTo}
              onChange={(Vv5) => set({ soTo: Vv5.target.value })}
            />
          </V49j>
          <V49j label="VGjhu51pIGjhuqFuIGLhuqNvIHF14bqjbg==">
            <input
              className="inp"
              value={Vtl.draft.thbq}
              onChange={(Vv6) => set({ thbq: Vv6.target.value })}
            />
          </V49j>
          <V49j label="R2hpIGNow7o=">
            <input
              className="inp"
              value={Vtl.draft.ghiChu ?? ""}
              onChange={(Vv7) => set({ ghiChu: Vv7.target.value })}
            />
          </V49j>
        </Modal>
      )}

      {Vsq && (
        <Modal
          title="UGjDoXQgY8OieSB0aMawIG3hu6VjIGNobyB1c2Vy"
          onClose={() => Vsr(false)}
          onSave={Vu0}
        >
          <p className="muted mt-0 text-sm">
            Đẩy phông <b>{Vtq?.tenDuAn}</b> + {Vtp.length} hồ sơ lên máy chủ cho một dự án. Máy user
            thuộc dự án sẽ nhận về khi bấm “Nhận cây thư mục”.
          </p>
          <V49j label="ROG7sSDDoW4gxJHDrWNo">
            <select className="inp" value={Vsu} onChange={(Vv8) => Vsv(Vv8.target.value)}>
              <option value="">— chọn dự án —</option>
              {Vss.map((p) => (
                <option key={p.id} value={p.id}>
                  {p.code} · {p.name}
                </option>
              ))}
            </select>
          </V49j>
          {Vss.length === 0 && (
            <p className="muted text-sm">
              Bạn chưa phụ trách dự án nào — tạo dự án ở màn “Quản lý người dùng → Dự án”.
            </p>
          )}
          {Vsw && <div className="login-error">{Vsw}</div>}
        </Modal>
      )}

      {Vt2 && (
        <Modal
          title="Tmjhuq1uIGPDonkgdGjGsCBt4bulYyBxdWEgTEFO"
          onClose={() => Vt3(false)}
          onSave={Vua}
        >
          {Vtc ? (
            <p className="muted">Đang quét mạng nội bộ…</p>
          ) : Vt4.length === 0 ? (
            <p className="muted">
              Không tìm thấy máy nào đang phát (đã hỏi cả mDNS lẫn broadcast). Kiểm tra máy quản trị
              đã bấm “Phát qua LAN” và hai máy cùng mạng. Nếu mạng chặn quảng bá, hãy nhập thẳng địa
              chỉ máy quản trị ở ô dưới.
            </p>
          ) : (
            <V49j label="TcOheSBxdeG6o24gdHLhu4s=">
              <select
                className="inp"
                value={Vt6}
                onChange={(Vv9) => Vt7(Vv9.target.value)}
                disabled={Vt8.trim() !== ""}
              >
                <option value="">— chọn máy —</option>
                {Vt4.map((p) => (
                  <option key={p.instance} value={p.instance}>
                    {p.project || p.addr} · v{p.version} · {p.addr}:{p.port}
                    {p.fingerprint
                      ? `IMK3IHbDom4gdGF5IA==${p.fingerprint}`
                      : "IMK3IChi4bqjbiBjxaksIGtow7RuZyBjw7MgdsOibiB0YXkp"}
                    {p.source === "broadcast" ? "IMK3ICh0w6xtIHF1YSBicm9hZGNhc3Qp" : ""}
                  </option>
                ))}
              </select>
              <div className="muted text-sm">
                Tên và vân tay ở đây lấy từ quảng bá mạng nên <b>chưa xác minh</b> — ai trong mạng
                cũng đặt được. Hãy chọn máy có vân tay đúng như quản trị đọc cho bạn; vân tay xác
                minh được sẽ hiện lại khi nhận mã site. Máy tìm qua broadcast không kèm tên dự án
                (gói quảng bá cố tình không mang tên), chỉ có địa chỉ và vân tay.
              </div>
            </V49j>
          )}
          <V49j label="SG/hurdjIG5o4bqtcCB0aOG6s25nIMSR4buLYSBjaOG7iSBtw6F5IHF14bqjbiB0cuG7iyAoa2hpIG3huqFuZyBjaOG6t24gcXXhuqNuZyBiw6Ep">
            <input
              className="inp"
              value={Vt8}
              placeholder={`VkQ6IDE5Mi4xNjguMS4xMCBob+G6t2MgMTkyLjE2OC4xLjEwOg==${V4a0}`}
              onChange={(Vva) => Vt9(Vva.target.value)}
            />
            <div className="muted text-sm">
              Không ghi cổng = cổng chung {V4a0}. Cổng thật hiện ở máy quản trị trong ô “Đang phát
              qua LAN”. Nhập ô này thì danh sách bên trên bị bỏ qua.
            </div>
          </V49j>
          <V49j label="TcOjIGdow6lwIG7hu5FpICg2IHPhu5Ep">
            <input
              className="inp"
              inputMode="numeric"
              maxLength={6}
              value={Vta}
              placeholder="YWRtaW4gxJHhu41jIGNobyBi4bqhbg=="
              onChange={(Vvb) => Vtb(Vvb.target.value.replace(/\D/g, ""))}
            />
          </V49j>
          <button className="btn mt-1" disabled={Vtc} onClick={() => void Vu8()}>
            <Icon name="lamMoi" />
            Quét lại
          </button>
          {Vsw && <div className="login-error">{Vsw}</div>}
        </Modal>
      )}
    </>
  );
}
