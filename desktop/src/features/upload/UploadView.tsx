// Copyright (c) 2026 DigiText. All rights reserved.

import { Fragment, useCallback, useMemo, useRef, useState } from "react";
import { V41p } from "@/live/useLive";
import { open } from "@tauri-apps/plugin-dialog";
import { V460, useCollects, useUploads } from "@/state";
import { isTauri, V3xq } from "@/api/ipc";
import { V3yw } from "@/domain/permissions";
import { V3zt } from "@/state/sessionCore";
import V4pp from "./BatchesPanel";
import { V4ps } from "./addFilesMessage";
import { V4pt } from "./collectMessage";
import { V4pu } from "./forwardMessage";
import { V4iv, V4it, V4iy, V4iw, V4ix, V4ia, type V4ij, type V4ii } from "./syncMatrix";
import { V4jf, V4j9, V4j4, V4j0, V4jg, V4j2, V4j3, type V4iz } from "./khoFilter";
import { V4pw, V4q6, V4q2, V4px, V4q0, V4q4, type V4pv } from "./collectedFilter";
import { listHoSo, V40w, type V40d, type V40v } from "@/api/archive";
import { V48j } from "@/api/catalog";
import { V4br, type V4bm } from "@/api/dossier";
import { V4cm, V4cl } from "@/features/dossier/logic";
import type { V3ym, V3yl, V3yo } from "@/domain/types";
import "./khogom.css";
import { V4ak, type V4a9 } from "@/api/users";
import {
  V43i,
  V43j,
  V431,
  V435,
  V436,
  V434,
  V432,
  V433,
  type V43g,
  type V43h,
  type V42z,
} from "@/api/lansync";
import { V4qa } from "./siteCodeWidth";
import { V4qb, V4qc } from "./sharePort";
import { Icon } from "@/components/Icon";

const V4qd = "Dán mã site (hoặc bấm “Sinh mã mạnh”)";

const V4qe = {
  flex: "0 0 auto",
  fontFamily: "ui-monospace, SFMono-Regular, Menlo, Consolas, monospace",
} as const;

function V4qf({ isAdmin: V2cn }: { isAdmin: boolean }) {
  return (
    <div className="kg-blocked">
      <b>
        <Icon name="canhBao" /> Mã site LAN hiện tại YẾU
      </b>{" "}
      — kiểu mã ngắn/6 số dò được trong vài giây trên mạng nội bộ. Hệ thống vẫn chạy bình thường để
      không gián đoạn công việc, nhưng nên đổi sớm:{" "}
      {V2cn
        ? "bấm “Sinh mã mạnh” rồi phát lại cây thư mục — các máy nhân viên sẽ tự nhận mã mới."
        : "báo quản trị bấm “Sinh mã mạnh” rồi phát lại cây thư mục cho bạn."}
    </div>
  );
}

const V4qg = (b: number) => (b / 1_048_576).toFixed(1);
const fmtN = (n: number) => n.toLocaleString("vi-VN");

const V4qh = "__unsorted__";

function V4qi({
  value: V2co,
  label: V2cp,
  tone: V2cq,
  title: V2cr,
}: {
  value: number;
  label: string;
  tone?: "ok" | "warn" | "err";
  title?: string;
}) {
  return (
    <div className={`kg-stat${V2cq ? ` ${V2cq}` : ""}`} title={V2cr}>
      <b>{fmtN(V2co)}</b>
      <span>{V2cp}</span>
    </div>
  );
}

const V4qj = 5000;

const V4qk = 400;

const V4ql = (V2cs: unknown) =>
  V3zt(
    V2cs instanceof V3xq ? V2cs.code : "error",
    V2cs instanceof Error ? V2cs.message : String(V2cs),
  );

export default function V4qm() {
  const { session: V2ct } = V460();
  const V2cu = !!V2ct && V3yw(V2ct.role, "admin");
  const V2cv = !!V2ct && V3yw(V2ct.role, "operate");

  return (
    <section className="view active">
      <div className="page-head">
        <h2>Đồng bộ dữ liệu</h2>
        <div className="sub">
          {V2ct?.projectName ? `· ${V2ct.projectName}` : ""}
          {V2ct?.mode === "offline"
            ? " · một số thao tác cần máy chủ"
            : ""}
        </div>
      </div>

      {V2cu ? <V4qo /> : V2cv ? <V4qn /> : <p className="muted">Không có gì để đồng bộ.</p>}
    </section>
  );
}

function V4qn() {
  const [V2cw, V2cx] = useState<V42z | null>(null);
  const [V2cy, V2cz] = useState("");
  const [V2d0, V2d1] = useState("");
  const [V2d2, V2d3] = useState(false);
  const [V2d4, V2d5] = useState("");

  const V2d6 = useCallback(async () => {
    if (!isTauri()) return;
    try {
      V2cx(await V431());
      V2cz((await V432()) ?? "");
      V2d1(String((await V435()) ?? ""));
    } catch (V2d7) {
      V2d5(V4ql(V2d7));
    }
  }, []);

  V41p(V2d6, { topics: ["lan", "settings", "tree"] });

  const V2d8 = async () => {
    const V2d9 = V4qc(V2d0);
    if (V2d9) {
      V2d5(V2d9);
      return;
    }
    V2d3(true);
    V2d5("");
    try {
      await V436(V2d0.trim());
      await V433(V2cy.trim());
      await V2d6();
      V2d5(
        V2cy.trim()
          ? "Đã lưu mã site và bật chia sẻ."
          : "Đã tắt chia sẻ.",
      );
    } catch (V2da) {
      V2d5(V4ql(V2da));
    } finally {
      V2d3(false);
    }
  };

  return (
    <div className="card max-w-155">
      <h3 className="mt-0">Chia sẻ dữ liệu với quản trị (LAN)</h3>
      {V2cw?.sharing ? (
        <p className="mt-0 mx-0 mb-1.5">
          <Icon name="thanhCong" /> Đang chia sẻ <b>{V2cw.fileCount}</b> tài liệu cho máy quản trị
          {V2cw.port != null ? (
            <>
              {" "}
              (cổng <b>{V2cw.port}</b>
              {V2cw.port === V4qb ? " — cổng chung" : ""})
            </>
          ) : null}
          . Máy quản trị sẽ <b>tự thu thập</b> — bạn không cần gửi thủ công.
        </p>
      ) : (
        <p className="muted mt-0">
          {V2cw && !V2cw.hasSiteCode
            ? "Chưa bật — mã site thường TỰ đến khi bạn “Nhận cây thư mục”. Hoặc nhập tay bên dưới."
            : "Chưa chia sẻ."}
        </p>
      )}
      {V2cw?.siteCodeWeak && <V4qf isAdmin={false} />}
      <div className="field">
        <label>Mã site LAN (tự nhận khi nhận cây thư mục — hoặc dán tay)</label>
        <div className="flex gap-2 flex-wrap items-center">
          <input
            className="inp"
            style={{ ...V4qe, width: V4qa(V2cy) }}
            value={V2cy}
            placeholder={V4qd}
            onChange={(V2db) => V2cz(V2db.target.value)}
          />
          <button className="btn primary" onClick={() => void V2d8()} disabled={V2d2}>
            {V2d2 ? "Đang lưu…" : "Lưu & bật"}
          </button>
        </div>
        <div className="muted text-sm">
          Để TRỐNG rồi bấm “Lưu &amp; bật” = xoá mã và tắt chia sẻ — dùng khi máy lỡ nhận nhầm mã
          của site khác, sau đó nhận lại cây thư mục để lấy mã đúng.
        </div>
      </div>
      {V2cw?.deviceFingerprint && (
        <div className="muted text-sm mb-2">
          Vân tay máy này: <code>{V2cw.deviceFingerprint}</code> — quản trị đối chiếu chuỗi này khi
          gom lần đầu.
        </div>
      )}
      <div className="field">
        <label>
          Cổng chia sẻ cố định (tuỳ chọn — bỏ trống = cổng chung {V4qb}, máy quản trị tự quét thấy;
          chỉ ghim khác khi cổng chung bị chiếm/firewall riêng)
        </label>
        <input
          className="inp max-w-40"
          value={V2d0}
          placeholder={`VD: ${V4qb}`}
          onChange={(V2dc) => V2d1(V2dc.target.value)}
        />
      </div>
      {V2cw && V2cw.blocked.length > 0 && (
        <div className="kg-blocked">
          <b>
            <Icon name="canhBao" /> {V2cw.blocked.length} hồ sơ CHƯA chia sẻ được
          </b>{" "}
          — máy quản trị sẽ không gom được những hồ sơ này:
          <ul>
            {V2cw.blocked.slice(0, 8).map((b) => (
              <li key={b.hoSoRef}>
                Hộp/HS <b>{b.hoSoRef}</b>: {b.reason}
              </li>
            ))}
          </ul>
          {V2cw.blocked.length > 8 && <div>…và {V2cw.blocked.length - 8} hồ sơ khác.</div>}
        </div>
      )}
      {V2d4 && <div className="muted text-sm">{V2d4}</div>}
    </div>
  );
}

function V4qo() {
  const { session: V2dd } = V460();

  const { startForward: V2de, retry: V2df, running: V2dg } = useUploads();

  const { startCollect: V2dh, startAddFiles: V2di, running: V2dj } = useCollects();
  const V2dk = V2dd?.role === "super_admin";
  const [V2dl, V2dm] = useState("");
  const [V2dn, V2do] = useState<V42z | null>(null);
  const [collected, V2dp] = useState<V43h | null>(null);
  const [V2dq, V2dr] = useState<V40v[]>([]);
  const [V2ds, V2dt] = useState<V40d[]>([]);
  const [V2du, V2dv] = useState(false);

  const V2dw = useRef(false);
  const [V2dx, V2dy] = useState<"" | "collect" | "forward" | "code" | "retry" | "add">("");
  const [V2dz, V2e0] = useState("");

  const [V2e1, V2e2] = useState<V4pv>(V4pw);

  const [flt, V2e3] = useState<V4iz>(V4j0);
  const [V2e4, V2e5] = useState<V3yl[]>([]);
  const [V2e6, V2e7] = useState<V3ym[]>([]);
  const [V2e8, V2e9] = useState<V3yo[]>([]);
  const [V2ea, V2eb] = useState<V4bm[]>([]);

  const [V2ec, V2ed] = useState(false);
  const [V2ee, V2ef] = useState<V4a9[]>([]);
  const [V2eg, V2eh] = useState("");

  const [V2ei, V2ej] = useState("");

  const [V2ek, V2el] = useState("");

  const [V2em, V2en] = useState("");
  const V2eo = useRef<HTMLDivElement | null>(null);

  const V2ep = useCallback(async () => {
    if (!isTauri()) return;
    try {
      V2dm((await V432()) ?? "");
      V2do(await V431());

      V2dp(await V43i(V4qj));
    } catch (V2eq) {
      V2e0(V4ql(V2eq));
    }

    try {
      const [V2er, V2es] = await Promise.all([V40w(), listHoSo()]);
      V2dr(V2er);
      V2dt(V2es);

      if (!V2dw.current) V2dv(V2es.length > V4qk);
    } catch {
      V2dr([]);
      V2dt([]);
    }

    try {
      const [cat, V2et] = await Promise.all([V48j(), V4br()]);
      V2e5(cat.phongBans);
      V2e7(cat.mangs);
      V2e9(cat.thbqs);
      V2eb(V2et);
    } catch {
      V2e5([]);
      V2e7([]);
      V2e9([]);
      V2eb([]);
    }
  }, []);

  V41p(V2ep, {
    topics: ["collected", "lan", "hoso", "catalog", "dossier", "settings"],
  });

  const V2eu = async () => {
    V2dy("code");
    V2e0("");
    try {
      await V433(V2dl.trim());
      await V2ep();
      V2e0(
        "Đã lưu mã site LAN. Phổ biến mã này cho các máy nhân viên (hoặc phát lại cây thư mục để họ tự nhận).",
      );
    } catch (V2ev) {
      V2e0(V4ql(V2ev));
    } finally {
      V2dy("");
    }
  };

  const V2ew = async () => {
    V2dy("code");
    V2e0("");
    try {
      const c = await V434();
      V2dm(c);
      await V2ep();
      V2e0(
        `Đã sinh mã site mới: ${c}. Hãy PHÁT LẠI cây thư mục để các máy nhân viên tự nhận mã ` +
          "mới — máy nào chưa nhận sẽ tạm thời không gom được.",
      );
    } catch (V2ex) {
      V2e0(V4ql(V2ex));
    } finally {
      V2dy("");
    }
  };
  const V2ey = async () => {
    V2dy("collect");
    V2e0("");
    try {
      V2e0(V4pt(await V2dh()));
      await V2ep();
    } catch (V2ez) {
      V2e0(V4ql(V2ez));
    } finally {
      V2dy("");
    }
  };
  const V2f0 = async () => {
    V2dy("forward");
    V2e0("");
    try {
      V2e0(V4pu(await V2de()));
      await V2ep();
    } catch (V2f1) {
      V2e0(V4ql(V2f1));
    } finally {
      V2dy("");
    }
  };

  const V2f2 = async () => {
    V2dy("retry");
    V2e0("");
    try {
      const n = await V43j();
      V2e0(
        n > 0
          ? `Đã bỏ ${n} bản bị từ chối khỏi kho gom. Bấm “① Thu thập” để lấy lại từ máy nhân viên.`
          : "Không có bản nào bị từ chối.",
      );
      await V2ep();
    } catch (V2f3) {
      V2e0(V4ql(V2f3));
    } finally {
      V2dy("");
    }
  };

  const V2f4 = async (V2f5: number, V2f6: string) => {
    V2e0("");
    try {
      await V2df(V2f5);
      await V2ep();
    } catch (V2f7) {
      V2e0(`Gửi lại “${V2f6}” lỗi: ${V4ql(V2f7)}`);
    }
  };

  const V2f8 = async (V2f9: number[]) => {
    V2e0("");
    try {
      await V2df(V2f9);
      await V2ep();
    } catch (V2fa) {
      V2e0(`Gửi lại ${V2f9.length} tài liệu lỗi: ${V4ql(V2fa)}`);
    }
  };

  const V2fb = useCallback(async () => {
    V2ed(true);
    if (V2ee.length) return;
    try {
      const V2fc = await V4ak();
      V2ef(V2fc);
      V2eh((V2fd) => V2fd || V2dd?.projectId || (V2fc.length === 1 ? V2fc[0].id : ""));
    } catch {
      V2ef([]);
    }
  }, [V2ee.length, V2dd?.projectId]);

  const V2fe = async () => {
    if (V2ec) {
      V2ed(false);
      return;
    }
    await V2fb();
  };

  const V2ff = async (V2fg: string, V2fh: string, V2fi: string) => {
    await V2fb();
    V2ej(V2fg);
    V2el(`${V2fh}/${V2fi}`);
    V2e0("");
    requestAnimationFrame(() => V2eo.current?.scrollIntoView({ block: "center" }));
  };

  const V2fj = async (V2fk: boolean) => {
    let V2fl: string[] = [];
    try {
      const r = await open({
        multiple: true,
        directory: V2fk,
        title: V2fk
          ? "Chọn thư mục tài liệu"
          : "Chọn tệp PDF để thêm vào Kho",
        filters: V2fk ? undefined : [{ name: "PDF", extensions: ["pdf"] }],
      });
      V2fl = typeof r === "string" ? [r] : Array.isArray(r) ? r : [];
    } catch (V2fm) {
      V2e0(V4ql(V2fm));
      return;
    }
    if (!V2fl.length) return;
    V2dy("add");
    V2e0("");
    try {
      const r = await V2di({
        paths: V2fl,
        projectId: V2eg.trim() || undefined,
        destRel: V2fn || undefined,

        phongMa: V2ek && V2ek !== V4qh ? V2ei || undefined : undefined,
        projectLabel: V2g9 || undefined,
      });
      V2e0(V4ps(r));
      await V2ep();
    } catch (V2fo) {
      V2e0(V4ql(V2fo));
    } finally {
      V2dy("");
    }
  };

  const V2fp = collected?.pending ?? 0;
  const V2fq = collected?.rejected ?? 0;

  const tree = useMemo(() => V4it(V2ds), [V2ds]);
  const V2fr = useMemo(() => V4j9(V2ea, V2ds, V2e8), [V2ea, V2ds, V2e8]);

  const V2fs = useMemo(() => V4q2(collected?.rows ?? [], V2e1), [collected, V2e1]);

  const V2ft = useMemo(() => V4jf(V2fs, flt, tree, V2fr), [V2fs, flt, tree, V2fr]);
  const V2fu = V2ft.rows;
  const V2fv = V4px(V2e1) || V4j2(flt);

  const V2fw = useMemo(() => {
    if (V2fv || !collected) return V4q4(V2fu);
    return {
      collected: collected.total,
      forwarded: collected.forwarded,
      notForwarded: collected.total - collected.forwarded,
      rejected: collected.rejected,
    };
  }, [V2fv, collected, V2fu]);

  const V2fx = useMemo(() => V4q6(V2fu), [V2fu]);

  const V2fy = useMemo(() => V4jg(V2ds, flt, V2fr), [V2ds, flt, V2fr]);

  const V2fz = useMemo(
    () =>
      V4iv({
        phong: V2dq,
        hoSo: V2fy,
        collected: V2fu,
        adminUsername: V2dd?.username ?? null,
        hideEmptyRows: V2du,
      }),
    [V2dq, V2fy, V2fu, V2dd, V2du],
  );

  const V2g0 = (collected?.rows.length ?? 0) >= V4qj;

  const V2g1 = useMemo(() => V2ds.filter((h) => (h.phongId ?? "") === V2ei), [V2ds, V2ei]);
  const V2fn = useMemo(() => {
    if (!V2dq.length) return V2em.trim();
    if (!V2ei || !V2ek || V2ek === V4qh) return "";
    const ten = V2dq.find((p) => p.maDuAn === V2ei)?.tenDuAn || V2ei;
    return `${ten}/${V2ek}`;
  }, [V2dq, V2ei, V2ek, V2em]);

  const V2g2 = V2dq.length > 0 && !V2ek;

  // Nhãn dự án máy chủ đã chọn (hoặc dự án duy nhất của tài khoản) — thư mục cấp 1 trong Kho.
  const V2g9 = useMemo(() => {
    const p = V2ee.find((x) => x.id === V2eg.trim()) ?? (V2ee.length === 1 ? V2ee[0] : undefined);
    return p ? `${p.code} · ${p.name}`.split("/").join("-").split("\\").join("-") : "";
  }, [V2ee, V2eg]);
  const V2gaDest = useMemo(() => {
    const hoSo = !!(V2ek && V2ek !== V4qh);
    const tail = V2fn || "Chưa phân loại";
    return hoSo || !V2g9 ? tail : `${V2g9}/${tail}`;
  }, [V2ek, V2fn, V2g9]);

  const V2g3 = useMemo(() => V4cl(V2e4, flt.maDa), [V2e4, flt.maDa]);
  const V2g4 = useMemo(() => V4cm(V2e6, flt.maDv), [V2e6, flt.maDv]);

  const V2g5 = (v: string) => V2e3((f) => ({ ...f, maDa: v, maDv: "", mangCvid: "" }));
  const V2g6 = (v: string) => V2e3((f) => ({ ...f, maDv: v, mangCvid: "" }));

  return (
    <>
      <div className="card mb-4">
        {V2dn?.siteCodeWeak && <V4qf isAdmin />}
        <div className="field">
          <label>Mã site LAN</label>
          <div className="flex gap-2 flex-wrap items-center">
            <input
              className="inp"
              style={{ ...V4qe, width: V4qa(V2dl) }}
              value={V2dl}
              placeholder={V4qd}
              onChange={(V2g7) => V2dm(V2g7.target.value)}
            />
            <button className="btn" onClick={() => void V2eu()} disabled={V2dx !== ""}>
              Lưu mã
            </button>
            <button className="btn" onClick={() => void V2ew()} disabled={V2dx !== ""}>
              Sinh mã mạnh
            </button>
          </div>
        </div>

        <div className="flex gap-3 flex-wrap items-center mt-3">
          <button
            className="btn primary"
            onClick={() => void V2ey()}
            disabled={V2dx !== "" || V2dg || V2dj}
          >
            {V2dx === "collect" || V2dj
              ? "Đang thu thập…"
              : "① Thu thập từ máy user (LAN)"}
          </button>
          <button
            className="btn"
            onClick={() => void V2f0()}
            disabled={V2dx !== "" || V2dg || V2dj || V2fp === 0}
          >
            {V2dx === "forward" || V2dg
              ? "Đang đẩy…"
              : "② Đẩy lên máy chủ"}
          </button>
          <button className="btn" onClick={() => void V2fe()} disabled={V2dx !== "" || V2dj}>
            {V2dx === "add"
              ? "Đang thêm…"
              : "③ Thêm tài liệu từ ổ cứng"}
          </button>
          {V2fq > 0 && (
            <button className="btn" onClick={() => void V2f2()} disabled={V2dx !== ""}>
              {V2dx === "retry"
                ? "Đang gom lại…"
                : `Gom lại bản bị từ chối (${V2fq})`}
            </button>
          )}
          {V2dz && <span className="muted text-sm">{V2dz}</span>}
        </div>

        {V2ec && (
          <div ref={V2eo} className="field kg-add mt-3 border-t border-border-subtle pt-3">
            <label>
              Thêm PDF có sẵn (ổ cứng, USB, thư mục scan cũ) vào Kho rồi đẩy lên máy chủ
            </label>
            {}
            <div className="kg-declare">
              <label className="kg-dfield">
                <span>Dự án</span>
                <select className="inp" value={V2eg} onChange={(V2g8) => V2eh(V2g8.target.value)}>
                  <option value="">
                    {V2ee.length
                      ? "— Chọn dự án đích —"
                      : "Dự án của tài khoản này"}
                  </option>
                  {V2ee.map((p) => (
                    <option key={p.id} value={p.id}>
                      {p.code} · {p.name}
                    </option>
                  ))}
                </select>
              </label>
              {V2dq.length > 0 ? (
                <>
                  <label className="kg-dfield">
                    <span>Phông (mục lục)</span>
                    <select
                      className="inp"
                      value={V2ei}
                      onChange={(V2g9) => {
                        V2ej(V2g9.target.value);
                        V2el("");
                      }}
                    >
                      <option value="">— Chọn phông —</option>
                      {V2dq.map((p) => (
                        <option key={p.maDuAn} value={p.maDuAn}>
                          {p.tenDuAn || p.maDuAn}
                        </option>
                      ))}
                    </select>
                  </label>
                  <label className="kg-dfield kg-dwide">
                    <span>Hộp số · Hồ sơ số</span>
                    <select
                      className="inp"
                      value={V2ek}
                      disabled={!V2ei}
                      onChange={(V2ga) => V2el(V2ga.target.value)}
                    >
                      <option value="">
                        {V2ei
                          ? "— Chọn hồ sơ theo danh mục —"
                          : "— Chọn phông trước —"}
                      </option>
                      {V2g1.map((h) => (
                        <option key={h.id} value={`${h.hopSo}/${h.hoSoSo}`}>
                          Hộp {h.hopSo || "—"} · Hồ sơ {h.hoSoSo || "—"}
                          {h.tieuDe ? ` — ${h.tieuDe}` : ""}
                        </option>
                      ))}
                      <option value={V4qh}>— Chưa phân loại (không kê khai) —</option>
                    </select>
                  </label>
                </>
              ) : (
                <label className="kg-dfield kg-dwide">
                  <span>Vị trí trong Kho</span>
                  <input
                    className="inp"
                    value={V2em}
                    placeholder="VD: Sở Tài chính/01/007"
                    onChange={(V2gb) => V2en(V2gb.target.value)}
                  />
                </label>
              )}
            </div>
            <div className="flex gap-2 flex-wrap items-center">
              <button
                className="btn primary"
                onClick={() => void V2fj(false)}
                disabled={V2dx !== "" || V2g2}
              >
                {V2dx === "add" ? "Đang thêm…" : "Chọn tệp PDF…"}
              </button>
              <button
                className="btn"
                onClick={() => void V2fj(true)}
                disabled={V2dx !== "" || V2g2}
              >
                Chọn thư mục…
              </button>
              <span className="muted text-sm">
                {V2g2
                  ? "Chọn phông + hồ sơ theo danh mục rồi mới chọn tệp."
                  : `Sẽ lưu vào Kho: ${V2gaDest}`}
              </span>
            </div>
            <div className="muted text-sm">
              Tệp được CHÉP vào Kho tài liệu rồi đi cùng một đường với dữ liệu thu thập từ nhân
              viên: có nối lại khi đứt mạng, khử trùng theo nội dung và nằm trong đợt nộp để đối
              soát. Tệp đã nằm sẵn trong Kho thì không bị chép nhân đôi. Thư mục được duyệt cả cấp
              con, chỉ lấy .pdf.
            </div>
          </div>
        )}
      </div>

      {collected && (
        <div className="card kg-card p-0 overflow-hidden mb-4">
          {}
          <div className="kg-flt">
            <label className="kg-dfield">
              <span>Phông</span>
              <select className="inp" value={flt.maDa} onChange={(V2gc) => V2g5(V2gc.target.value)}>
                <option value="">— Tất cả phông —</option>
                {V2dq.map((p) => (
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
                disabled={!flt.maDa || V2g3.length === 0}
                onChange={(V2gd) => V2g6(V2gd.target.value)}
              >
                <option value="">{V4j4("phongBan", !!flt.maDa, V2g3.length)}</option>
                {V2g3.map((V2ge) => (
                  <option key={V2ge.maDv} value={V2ge.maDv}>
                    {V2ge.tenDonVi || V2ge.maDonVi || V2ge.maDv}
                  </option>
                ))}
              </select>
            </label>
            <label className="kg-dfield">
              <span>Mảng CV</span>
              <select
                className="inp"
                value={flt.mangCvid}
                disabled={!flt.maDv || V2g4.length === 0}
                onChange={(V2gf) => V2e3((f) => ({ ...f, mangCvid: V2gf.target.value }))}
              >
                <option value="">{V4j4("mang", !!flt.maDv, V2g4.length)}</option>
                {V2g4.map((m) => (
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
                disabled={V2e8.length === 0}
                onChange={(V2gg) => V2e3((f) => ({ ...f, thbq: V2gg.target.value }))}
              >
                <option value="">
                  {V2e8.length
                    ? "— Tất cả THBQ —"
                    : "— Danh mục chưa có THBQ —"}
                </option>
                {V2e8.map((t) => (
                  <option key={t.thbqid} value={String(t.thbqid)}>
                    {t.tenThbq}
                  </option>
                ))}
              </select>
            </label>
            <label className="kg-dfield kg-dwide">
              <span>Tìm kiếm</span>
              <input
                className="inp"
                value={flt.search}
                placeholder="Tên tệp · số hồ sơ · người quét · tiêu đề hồ sơ"
                onChange={(V2gh) => V2e3((f) => ({ ...f, search: V2gh.target.value }))}
              />
            </label>
          </div>

          <div className="kg-bar">
            <div className="kg-stats">
              <V4qi value={V2fw.collected} label="Tổng file đã thu thập" />
              <V4qi
                value={V2fw.forwarded}
                label="Đã đồng bộ lên máy chủ"
                tone="ok"
              />
              <V4qi
                value={V2fw.notForwarded}
                label="Chưa đồng bộ lên máy chủ"
                tone="warn"
                title={`Đang chờ đẩy ${fmtN(V2fp)}${V2fq > 0 ? ` · máy chủ từ chối ${fmtN(V2fq)}` : ""}`}
              />
              <V4qi
                value={V2fx.length}
                label="File lỗi"
                tone={V2fx.length > 0 ? "err" : undefined}
                title="Bản máy chủ từ chối, hoặc đã đẩy lỗi ít nhất một lần mà vẫn chưa lên"
              />
            </div>
            <span className="kg-spacer" />
            <div className="kg-filter">
              <span className="muted">Ngày thu thập</span>
              <input
                type="date"
                className="inp"
                aria-label="Từ ngày thu thập"
                value={V2e1.from}
                max={V2e1.to || undefined}
                onChange={(V2gi) => V2e2((r) => ({ ...r, from: V2gi.target.value }))}
              />
              <span className="muted">→</span>
              <input
                type="date"
                className="inp"
                aria-label="Đến ngày thu thập"
                value={V2e1.to}
                min={V2e1.from || undefined}
                onChange={(V2gj) => V2e2((r) => ({ ...r, to: V2gj.target.value }))}
              />
              {}
              {V4px(V2e1) && (
                <button className="btn" onClick={() => V2e2(V4pw)}>
                  Xoá lọc ngày
                </button>
              )}
            </div>
            <label className="kg-toggle">
              <input
                type="checkbox"
                checked={V2du}
                onChange={(V2gk) => {
                  V2dw.current = true;
                  V2dv(V2gk.target.checked);
                }}
              />{" "}
              Chỉ hiện hồ sơ đã có dữ liệu
            </label>
          </div>

          {V2fz.groups.length === 0 ? (
            <p className="kg-note">
              {V4j2(flt)
                ? "Không có tài liệu nào khớp bộ lọc đang chọn — chọn lại “Tất cả phông” hoặc bỏ bớt tiêu chí."
                : V2fv
                  ? "Không có bản nào thu thập trong khoảng ngày đã chọn — nới khoảng hoặc bấm “Xoá lọc ngày”."
                  : "Chưa có hồ sơ nào trong mục lục và kho gom cũng trống — bấm “① Thu thập từ máy user (LAN)” để lấy tài liệu nhân viên đã đóng gói."}
            </p>
          ) : (
            <div className="tbl-scroll kg-matrix">
              <table className="tbl kg-table">
                <thead>
                  <tr>
                    <th className="kg-corner kg-c1">Hộp số</th>
                    <th className="kg-corner kg-c2">Hồ sơ số</th>
                    <th className="kg-corner kg-c3">Tổng số file</th>
                    {V2fz.columns.map((c) => (
                      <th key={c.key} className="kg-colhead">
                        <div className="kg-colname">
                          <Icon name={c.isAdmin ? "mayTram" : "nguoi"} size={15} /> <b>{c.label}</b>
                        </div>
                        <div className="muted">
                          {c.tally.total} tài liệu · {V4qg(c.tally.sizeBytes)} MB
                        </div>
                      </th>
                    ))}
                  </tr>
                </thead>
                <tbody>
                  {V2fz.groups.map((g) => (
                    <Fragment key={g.key}>
                      <tr className="kg-grouprow">
                        <th className="kg-rowhead kg-groupname" colSpan={3}>
                          <Icon name={g.key === V4ia ? "canhBao" : "thuMuc"} size={15} /> Dự án:{" "}
                          {g.label}
                        </th>
                        <td colSpan={V2fz.columns.length}>
                          <span className="muted">
                            {g.key === V4ia
                              ? "Không ghép được vào hồ sơ nào trong mục lục — kiểm tra phông/số hồ sơ."
                              : `${g.rows.length} hồ sơ · ${g.tally.total} tài liệu đã gom`}
                          </span>
                        </td>
                      </tr>
                      {g.rows.map((r) => (
                        <tr key={r.key}>
                          <th className="kg-rowhead kg-c1 kg-hoso">{r.hopSo || "—"}</th>
                          <th className="kg-rowhead kg-c2 kg-hoso">
                            <div>{r.hoSoSo || "—"}</div>
                            {r.tieuDe && <div className="muted">{r.tieuDe}</div>}
                          </th>
                          <th className="kg-rowhead kg-c3">
                            <div className="kg-count">
                              <span>{r.tally.total}</span>
                              {}
                              {r.inTree && g.key !== V4ia && g.key !== "" && (
                                <button
                                  className="kg-add-btn"
                                  title={`Bổ sung tài liệu vào hộp ${r.hopSo || "—"} · hồ sơ ${r.hoSoSo || "—"}`}
                                  onClick={() => void V2ff(g.key, r.hopSo, r.hoSoSo)}
                                >
                                  <Icon name="them" size={13} />
                                </button>
                              )}
                            </div>
                          </th>
                          {V2fz.columns.map((c) => (
                            <td key={c.key} className="kg-cell">
                              <V4qq
                                cell={r.cells.get(c.key)}
                                col={c}
                                queued={V2dg}
                                onResend={(V2gl) => void V2f8(V2gl)}
                              />
                            </td>
                          ))}
                        </tr>
                      ))}
                    </Fragment>
                  ))}
                </tbody>
              </table>
            </div>
          )}

          {V2fx.length > 0 && <V4qp rows={V2fx} queued={V2dg} onResend={V2f4} />}

          {(V2g0 || V2fz.hiddenRows > 0 || V2fz.unmatched > 0 || V2fv) && (
            <p className="kg-note">
              {V2fv && (
                <>
                  Đang lọc: hiện {fmtN(V2fu.length)}/{fmtN(collected.rows.length)} bản đã nạp.{" "}
                </>
              )}
              {}
              {V2ft.unknown > 0 && V4j3(flt) && (
                <>
                  Đã loại {fmtN(V2ft.unknown)} tài liệu chưa có trong mục lục (không xét được phòng
                  ban / mảng CV / THBQ) — chạy “Đồng bộ từ Kho tài liệu” ở màn Khai thác rồi lọc
                  lại.{" "}
                </>
              )}
              {V2g0 && (
                <>
                  Kho có {collected.total.toLocaleString("vi-VN")} bản, bảng chỉ nạp{" "}
                  {V4qj.toLocaleString("vi-VN")} bản mới nhất — các ô chưa liệt kê đủ.{" "}
                </>
              )}
              {V2fz.hiddenRows > 0 && (
                <>
                  Đang ẩn {V2fz.hiddenRows} hồ sơ chưa có dữ liệu
                  {!V2dw.current
                    ? " (mục lục lớn nên tự ẩn — bỏ tick để xem cả cây)"
                    : ""}
                  .{" "}
                </>
              )}
              {V2fz.unmatched > 0 && (
                <>{V2fz.unmatched} tài liệu chưa khớp cây thư mục (xem nhóm cuối bảng).</>
              )}
            </p>
          )}
        </div>
      )}

      {}
      <V4pp isSuper={V2dk} />
    </>
  );
}

function V4qp({
  rows: V2gm,
  queued: V2gn,
  onResend: V2go,
}: {
  rows: V43g[];
  queued: boolean;
  onResend: (V2gp: number, V2gq: string) => void;
}) {
  return (
    <details className="kg-errs" open>
      <summary>
        <Icon name="canhBao" /> {fmtN(V2gm.length)} file lỗi — chưa lên được máy chủ
      </summary>
      <div className="tbl-scroll">
        <table className="tbl kg-errtbl">
          <thead>
            <tr>
              <th>Tên file</th>
              <th>Hồ sơ</th>
              <th>Lý do</th>
              <th>Đường dẫn trong Kho</th>
              <th>Thu thập lúc</th>
              <th />
            </tr>
          </thead>
          <tbody>
            {V2gm.map((r) => (
              <tr key={r.id}>
                <td>{r.filename}</td>
                <td>{r.hoSoRef || "—"}</td>
                <td className="kg-why">
                  {r.forwardFatal
                    ? "Máy chủ từ chối vĩnh viễn"
                    : `Lỗi ${r.forwardTries}×`}
                  {r.forwardError ? ` — ${r.forwardError}` : ""}
                </td>
                <td>
                  <code className="kg-path" title={r.storagePath}>
                    {r.storagePath}
                  </code>
                </td>
                <td>{V4q0(r.collectedAt)}</td>
                <td>
                  <button
                    className="kg-resend"
                    onClick={() => V2go(r.id, r.filename)}
                    title={
                      V2gn
                        ? `Xếp hàng gửi lại “${r.filename}” — chạy khi phiên hiện tại xong`
                        : `Gửi lại “${r.filename}” lên máy chủ`
                    }
                  >
                    <Icon name="lamMoi" size={13} />
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </details>
  );
}

function V4qq({
  cell,
  col: V2gr,
  queued: V2gs,
  onResend: V2gt,
}: {
  cell: V4ij | undefined;
  col: V4ii;

  queued: boolean;
  onResend: (V2gu: number[]) => void;
}) {
  if (!cell || cell.files.length === 0) {
    return (
      <span
        className="kg-empty"
        title={`${V2gr.label}: chưa có tài liệu trong hồ sơ này`}
      >
        ·
      </span>
    );
  }
  const t = cell.tally;
  const V2gv = V4iy(cell);
  return (
    <div className="kg-files" title={`${V2gr.label} — ${V4ix(t)} · ${V4qg(t.sizeBytes)} MB`}>
      <span className={`kg-dot ${V4iw(t)}`} />
      <span className="kg-fcount">
        <Icon name="taiLieu" /> ({fmtN(t.total)})
      </span>
      {V2gv.length > 0 && (
        <button
          className="kg-resend"
          onClick={() => V2gt(V2gv)}
          title={
            V2gs
              ? `Xếp hàng gửi lại ${V2gv.length} tài liệu — chạy khi phiên hiện tại xong`
              : `Gửi lại ${V2gv.length} tài liệu của ô này lên máy chủ`
          }
        >
          <Icon name="lamMoi" size={13} />
        </button>
      )}
    </div>
  );
}
