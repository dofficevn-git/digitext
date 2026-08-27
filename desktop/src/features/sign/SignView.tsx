// Copyright (c) 2026 DigiText. All rights reserved.

import { useCallback, useEffect, useState } from "react";
import { V41p } from "@/live/useLive";

import { isTauri, V3xq } from "@/api/ipc";
import { V4qu, V4qt, V4qv, V4qw, type V4qr, type V4qs } from "@/api/sign";
import { Icon } from "@/components/Icon";

function V4qx(V2gy: unknown): string {
  return V2gy instanceof V3xq || V2gy instanceof Error ? V2gy.message : String(V2gy);
}

export default function V4qy() {
  const [V2gz, V2h0] = useState<V4qr[]>([]);

  const [V2h1, V2h2] = useState("");
  const [V2h3, V2h4] = useState<string[]>([]);
  const [tokens, V2h5] = useState<V4qs[]>([]);
  const [V2h6, V2h7] = useState<number | null>(null);
  const [V2h8, V2h9] = useState("");
  const [V2ha, V2hb] = useState<string | null>(null);
  const [V2hc, V2hd] = useState<{ kind: "info" | "err" | "ok"; text: string } | null>(null);

  const V2he = useCallback(async () => {
    if (!isTauri()) {
      V2hd({
        kind: "info",
        text: "Chạy trong ứng dụng desktop để ký (chế độ trình duyệt không có token).",
      });
      return;
    }
    V2hb("jobs");
    try {
      V2h0(await V4qt());
      V2hd(null);
    } catch (V2hf) {
      V2hd({
        kind: "err",
        text: `Không lấy được hàng đợi ký: ${V4qx(V2hf)}`,
      });
    } finally {
      V2hb(null);
    }
  }, []);

  const V2hg = async () => {
    if (!isTauri()) return;
    try {
      const m = await V4qu();
      V2h4(m);
      V2h2((V2hh) => (V2hh.trim() === "" && m.length > 0 ? m[0] : V2hh));
    } catch {}
  };

  V41p(V2he, { topics: ["sign", "docs"] });

  useEffect(() => {
    void V2hg();
  }, []);

  const V2hi = async () => {
    V2hb("tokens");
    try {
      await V2hg();
      const t = await V4qv(V2h1);
      V2h5(t);
      V2h7(t.length > 0 ? t[0].index : null);
      V2hd(
        t.length === 0
          ? {
              kind: "err",
              text:
                "Không thấy token nào đang cắm — kiểm tra USB token đã cắm chưa, và thư viện PKCS#11 " +
                "(phần mềm token của CA) đã cài chưa.",
            }
          : { kind: "ok", text: `Thấy ${t.length} token.` },
      );
    } catch (V2hj) {
      V2hd({ kind: "err", text: `Dò token lỗi: ${V4qx(V2hj)}` });
    } finally {
      V2hb(null);
    }
  };

  const sign = async (V2hk: V4qr) => {
    if (V2h6 === null) {
      V2hd({
        kind: "err",
        text: "Chọn token trước khi ký (bấm “Dò token”).",
      });
      return;
    }
    if (!V2h8) {
      V2hd({ kind: "err", text: "Nhập mã PIN của token." });
      return;
    }
    V2hb(V2hk.id);
    try {
      const status = await V4qw({
        jobId: V2hk.id,
        modulePath: V2h1,
        slotIndex: V2h6,
        pin: V2h8,
      });
      if (status === "signed") {
        V2hd({ kind: "ok", text: `Đã ký “${V2hk.filename}”.` });
        V2h0((V2hl) => V2hl.filter((V2hm) => V2hm.id !== V2hk.id));
      } else {
        V2hd({
          kind: "err",
          text: `Máy chủ TỪ CHỐI chữ ký cho “${V2hk.filename}” (trạng thái: ${status}).`,
        });
        void V2he();
      }
    } catch (V2hn) {
      V2hd({ kind: "err", text: `Ký “${V2hk.filename}” lỗi: ${V4qx(V2hn)}` });
    } finally {
      V2hb(null);
    }
  };

  return (
    <section className="view active">
      <div className="page-head">
        <h2>Ký số</h2>
        <div className="sub">· PAdES · USB Token (Viettel-CA) · khoá không rời token</div>
      </div>

      {V2hc && (
        <div
          className={`banner ${V2hc.kind === "err" ? "error" : V2hc.kind === "ok" ? "success" : "info"}`}
        >
          <span className="bi">
            <Icon
              size={18}
              name={V2hc.kind === "err" ? "loi" : V2hc.kind === "ok" ? "thanhCong" : "thongTin"}
            />
          </span>
          <div className="bx">{V2hc.text}</div>
        </div>
      )}

      <div className="card mt-3.5">
        <h3>Token ký</h3>
        <div className="grid gap-2.5 mt-2 max-w-160">
          <label className="grid gap-1">
            <span className="muted text-sm">
              Đường dẫn module PKCS#11{" "}
              {V2h3.length > 0
                ? "(tự dò được, sửa nếu cần)"
                : "(để trống = tự dò)"}
            </span>
            <input
              value={V2h1}
              onChange={(V2ho) => V2h2(V2ho.target.value)}
              spellCheck={false}
              list="pkcs11-modules"
              placeholder="tự dò thư viện token trên máy"
            />
            <datalist id="pkcs11-modules">
              {V2h3.map((m) => (
                <option key={m} value={m} />
              ))}
            </datalist>
            {V2h3.length === 0 && (
              <span className="muted text-sm">
                Chưa thấy thư viện PKCS#11 nào — cài phần mềm token của CA (iCA SmartToken,
                Viettel-CA…) rồi bấm “Dò token”.
              </span>
            )}
          </label>
          <div className="flex gap-2 items-center flex-wrap">
            <button className="btn" onClick={V2hi} disabled={V2ha !== null}>
              {V2ha === "tokens" ? "Đang dò…" : "Dò token"}
            </button>
            {tokens.length > 0 && (
              <select value={V2h6 ?? ""} onChange={(V2hp) => V2h7(Number(V2hp.target.value))}>
                {tokens.map((t) => (
                  <option key={t.index} value={t.index}>
                    {t.label || `Token #${t.index}`}
                  </option>
                ))}
              </select>
            )}
          </div>
          <label className="grid gap-1 max-w-60">
            <span className="muted text-sm">Mã PIN</span>
            <input
              type="password"
              value={V2h8}
              onChange={(V2hq) => V2h9(V2hq.target.value)}
              autoComplete="off"
            />
          </label>
        </div>
      </div>

      <div className="card mt-3.5">
        <div className="flex justify-between items-center">
          <h3>Chờ ký ({V2gz.length})</h3>
          <button className="btn ghost" onClick={V2he} disabled={V2ha !== null}>
            {V2ha === "jobs" ? "Đang tải…" : "Làm mới"}
          </button>
        </div>
        {V2gz.length === 0 ? (
          <div className="muted mt-2.5">Không có tài liệu nào đang chờ ký.</div>
        ) : (
          <div className="tbl-scroll">
            <table className="w-full mt-2.5 border-collapse">
              <thead>
                <tr className="text-left text-fg-muted text-sm">
                  <th className="py-1.5 px-1">Tài liệu</th>
                  <th className="py-1.5 px-1">Hồ sơ</th>
                  <th className="py-1.5 px-1">Số trang</th>
                  <th className="py-1.5 px-1" />
                </tr>
              </thead>
              <tbody>
                {V2gz.map((V2hr) => (
                  <tr key={V2hr.id} className="border-t border-border-subtle">
                    <td className="py-2 px-1">{V2hr.filename}</td>
                    <td className="py-2 px-1 text-fg-muted">{V2hr.hoSoRef ?? "—"}</td>
                    <td className="py-2 px-1 text-fg-muted">{V2hr.pageCount ?? "—"}</td>
                    <td className="py-2 px-1 text-right">
                      <button className="btn" onClick={() => sign(V2hr)} disabled={V2ha !== null}>
                        {V2ha === V2hr.id ? "Đang ký…" : "Ký"}
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </section>
  );
}
