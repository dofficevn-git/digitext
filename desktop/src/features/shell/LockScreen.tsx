// Copyright (c) 2026 DigiText. All rights reserved.

import { useState } from "react";
import { Icon } from "@/components/Icon";
import { V460 } from "@/state";
import { V3xq } from "@/api/ipc";
import { V3zt } from "@/state/sessionCore";
import { V3za } from "@/domain/permissions";
import V4x3 from "./CaptchaField";
import { V4x2 } from "./useCaptcha";

export default function V4x5() {
  const { session: V39i, unlock: V39j, switchUser: V39k, logout: V39l } = V460();
  const [V39m, V39n] = useState<"unlock" | "switch">("unlock");
  const [V39o, V39p] = useState("");
  const [V39q, V39r] = useState("");
  const [V39s, V39t] = useState(false);
  const [V39u, V39v] = useState("");
  const V39w = V4x2();

  if (!V39i) return null;

  const V39x = V39m === "switch";

  const V39y = async (V39z: () => Promise<void>) => {
    if (V39s) return;
    V39t(true);
    V39v("");
    try {
      await V39z();
      V39r("");
    } catch (V3a0) {
      const V3a1 = V3a0 instanceof V3xq ? V3a0.code : "error";
      V39v(V3zt(V3a1, V3a0 instanceof Error ? V3a0.message : String(V3a0)));

      if (V39x) V39w.refresh();
    } finally {
      V39t(false);
    }
  };

  const V3a2 = () => {
    if (V39m === "unlock") {
      if (!V39q) return;
      void V39y(() => V39j(V39q));
    } else {
      if (!V39o.trim() || !V39q || !V39w.ready || !V39w.challenge) return;
      const { id: V3a3 } = V39w.challenge;
      void V39y(() => V39k(V39o.trim(), V39q, V3a3, V39w.answer));
    }
  };

  return (
    <div className="login-screen">
      <div className="login-card">
        <div className="lock-avatar">{V39i.displayName[0]?.toUpperCase() ?? "?"}</div>
        <h1>{V39m === "unlock" ? V39i.displayName : "Đổi người dùng"}</h1>
        <div className="sub">
          {V39m === "unlock"
            ? `${V3za(V39i.role)}${V39i.projectName ? ` · ${V39i.projectName}` : ""} · đang khoá`
            : "Đăng nhập bằng tài khoản khác trên máy này"}
        </div>

        <form
          onSubmit={(V3a4) => {
            V3a4.preventDefault();
            V3a2();
          }}
        >
          {V39m === "switch" && (
            <div className="field">
              <label htmlFor="lock-username">Tài khoản</label>
              <input
                id="lock-username"
                name="username"
                type="text"
                autoComplete="username"
                value={V39o}
                autoFocus
                onChange={(V3a5) => V39p(V3a5.target.value)}
              />
            </div>
          )}
          <div className="field">
            <label htmlFor="lock-password">Mật khẩu</label>
            <input
              id="lock-password"
              name="password"
              type="password"
              autoComplete="current-password"
              value={V39q}
              autoFocus={V39m === "unlock"}
              onChange={(V3a6) => V39r(V3a6.target.value)}
            />
          </div>

          {V39x && <V4x3 captcha={V39w} onEnter={V3a2} disabled={V39s} />}

          {V39u && (
            <div className="login-error" role="alert">
              {V39u}
            </div>
          )}

          <button
            type="submit"
            className="btn primary lg w-full"
            disabled={V39s || !V39q || (V39x && (!V39o.trim() || !V39w.ready))}
          >
            {V39s
              ? "Đang xử lý…"
              : V39m === "unlock"
                ? "Mở khoá"
                : "Đăng nhập"}
          </button>
        </form>

        <div className="login-devstrip">
          <button
            type="button"
            className="server-gear inline-flex items-center gap-1.5"
            onClick={() => {
              V39n((m) => {
                if (m === "unlock") V39w.refresh();
                return m === "unlock" ? "switch" : "unlock";
              });
              V39v("");
              V39r("");
            }}
          >
            {V39m === "unlock" ? (
              <>
                <Icon name="doiNguoiDung" size={13} />
                Đổi người dùng
              </>
            ) : (
              <>
                <Icon name="truoc" size={13} />
                Quay lại mở khoá
              </>
            )}
          </button>
          <button type="button" className="server-gear" onClick={() => void V39l()}>
            Đăng xuất
          </button>
        </div>
      </div>
    </div>
  );
}
