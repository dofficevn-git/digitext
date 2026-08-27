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
      label: `${Vtq.tenDuAn} · Hộp ${h.hopSo} · HS ${h.hoSoSo}`,
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
        `Đã phát cây thư mục (phiên bản ${Vu1}) — máy user sẽ nhận khi bấm "Nhận cây thư mục".`,
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
          `Đã nhận cây thư mục mới (phiên bản ${r.version}) — tạo ${r.created} hồ sơ.`,
        );
        goto("drive");
      } else {
        Vsx("Cây thư mục đã là bản mới nhất.");
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
        Vsx("Đã dừng phát qua LAN.");
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
        `Đã nhận cây thư mục qua LAN — tạo ${r.created} hồ sơ.`,
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
    const Vui = r.peerFingerprint ?? "(không rõ)";
    const Vuj = Vuh
      ? `Máy này ĐANG dùng một mã site khác. Nhận mã mới sẽ chuyển máy sang site của máy phát đó.\n\n`
      : "";
    const Vuk = window.confirm(
      `${Vuj}Máy phát cây có vân tay:\n\n    ${Vui}\n\n` +
        "Hãy đối chiếu chuỗi này với chuỗi quản trị đọc cho bạn (màn “Đồng bộ dữ liệu” trên máy " +
        "quản trị hiển thị đúng chuỗi đó).\n\nNhận mã site LAN từ máy này?",
    );
    if (!Vuk) {
      Vsx(
        "Đã nhận cây thư mục, nhưng BỎ QUA mã site (bạn không xác nhận vân tay máy phát).",
      );
      return;
    }
    try {
      await V433(Vug, true);
      Vsx(
        "Đã nhận cây thư mục và mã site LAN — máy này bắt đầu chia sẻ cho quản trị.",
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
          "Vui lòng nhập Hồ sơ số (khóa định danh hồ sơ).",
        );
        return;
      }
      if (Vtl.mode === "edit" && Vtl.orig != null) {
        V40u(Vtl.orig, Vts(Vuq)).then(reload).catch(V49l);
      } else {
        if (!Vtq) {
          window.alert(
            "Vui lòng chọn phông (Cơ quan) ở bộ lọc trước khi thêm hồ sơ mới.",
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
          "Vui lòng chọn phông (Cơ quan) ở bộ lọc trước khi nhập để lưu vào cơ sở dữ liệu.",
        );
        return;
      }
      V40f(Vtr(Vtq), records.map(Vts))
        .then((r) => {
          Vsx(`Đã nhập ${r.hoSo} hồ sơ vào danh mục. ${V4bi(r.dossier)}`);
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
        addLabel="Thêm hồ sơ"
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
                title="Tạo thư mục Cơ quan → Hộp → Hồ sơ và hiển thị trong Kho tài liệu"
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
                title="Phát cây thư mục lên máy chủ để máy user nhận về"
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
                title="Phát cây thư mục qua mạng nội bộ (LAN) — cho site không có máy chủ"
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
                title="Nhận cây thư mục quản trị đã phát cho dự án của bạn (qua máy chủ)"
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
                title="Tìm máy quản trị đang phát cây thư mục trong mạng nội bộ (LAN)"
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
          { head: "Hộp số", cell: (h) => h.hopSo, cls: "tnum" },
          { head: "Hồ sơ số", cell: (h) => h.hoSoSo, cls: "tnum" },
          { head: "Tiêu đề hồ sơ", cell: (h) => h.tieuDe },
          { head: "Bắt đầu", cell: (h) => V4at(h).batDau, cls: "tnum" },
          { head: "Kết thúc", cell: (h) => V4at(h).ketThuc, cls: "tnum" },
          { head: "Số tờ", cell: (h) => h.soTo, cls: "num" },
          { head: "THBQ", cell: (h) => h.thbq },
          {
            head: "Nhóm / Năm",
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
          title={Vtl.mode === "add" ? "Thêm hồ sơ" : "Sửa hồ sơ"}
          onClose={() => Vtm(null)}
          onSave={Vup}
        >
          <V49j label="Hộp số">
            <input
              className="inp"
              value={Vtl.draft.hopSo}
              onChange={(Vv0) => set({ hopSo: Vv0.target.value })}
            />
          </V49j>
          <V49j label="Hồ sơ số">
            <input
              className="inp"
              value={Vtl.draft.hoSoSo}
              onChange={(Vv1) => set({ hoSoSo: Vv1.target.value })}
            />
          </V49j>
          <V49j label="Tiêu đề hồ sơ">
            <input
              className="inp"
              value={Vtl.draft.tieuDe}
              onChange={(Vv2) => set({ tieuDe: Vv2.target.value })}
            />
          </V49j>
          <V49j label="Thời gian bắt đầu">
            <input
              className="inp"
              placeholder="VD: 01/01/2010 hoặc 2010"
              value={Vtl.draft.ngayBatDau ?? ""}
              onChange={(Vv3) => set({ ngayBatDau: Vv3.target.value })}
            />
          </V49j>
          <V49j label="Thời gian kết thúc">
            <input
              className="inp"
              placeholder="VD: 31/12/2012 hoặc 2012"
              value={Vtl.draft.ngayKetThuc ?? ""}
              onChange={(Vv4) => set({ ngayKetThuc: Vv4.target.value })}
            />
          </V49j>
          <V49j label="Số tờ">
            <input
              className="inp"
              value={Vtl.draft.soTo}
              onChange={(Vv5) => set({ soTo: Vv5.target.value })}
            />
          </V49j>
          <V49j label="Thời hạn bảo quản">
            <input
              className="inp"
              value={Vtl.draft.thbq}
              onChange={(Vv6) => set({ thbq: Vv6.target.value })}
            />
          </V49j>
          <V49j label="Ghi chú">
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
          title="Phát cây thư mục cho user"
          onClose={() => Vsr(false)}
          onSave={Vu0}
        >
          <p className="muted mt-0 text-sm">
            Đẩy phông <b>{Vtq?.tenDuAn}</b> + {Vtp.length} hồ sơ lên máy chủ cho một dự án. Máy user
            thuộc dự án sẽ nhận về khi bấm “Nhận cây thư mục”.
          </p>
          <V49j label="Dự án đích">
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
          title="Nhận cây thư mục qua LAN"
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
            <V49j label="Máy quản trị">
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
                      ? ` · vân tay ${p.fingerprint}`
                      : " · (bản cũ, không có vân tay)"}
                    {p.source === "broadcast" ? " · (tìm qua broadcast)" : ""}
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
          <V49j label="Hoặc nhập thẳng địa chỉ máy quản trị (khi mạng chặn quảng bá)">
            <input
              className="inp"
              value={Vt8}
              placeholder={`VD: 192.168.1.10 hoặc 192.168.1.10:${V4a0}`}
              onChange={(Vva) => Vt9(Vva.target.value)}
            />
            <div className="muted text-sm">
              Không ghi cổng = cổng chung {V4a0}. Cổng thật hiện ở máy quản trị trong ô “Đang phát
              qua LAN”. Nhập ô này thì danh sách bên trên bị bỏ qua.
            </div>
          </V49j>
          <V49j label="Mã ghép nối (6 số)">
            <input
              className="inp"
              inputMode="numeric"
              maxLength={6}
              value={Vta}
              placeholder="admin đọc cho bạn"
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
