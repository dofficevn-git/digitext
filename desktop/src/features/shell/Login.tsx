// Copyright (c) 2026 DigiText. All rights reserved.

import { useEffect, useState } from "react";
import { Icon } from "@/components/Icon";
import { V460 } from "@/state";
import { V3zm, V3zn } from "@/api/auth";
import { isTauri, V3xq } from "@/api/ipc";
import { V3zt, V3zs } from "@/state/sessionCore";
import { V3za } from "@/domain/permissions";
import V4x3 from "./CaptchaField";
import { V4x2 } from "./useCaptcha";

export default function V4x4() {
  const { login: V38q, accounts: V38r, appVersion: V38s } = V460();
  const [V38t, V38u] = useState("");
  const [V38v, V38w] = useState("");
  const [V38x, V38y] = useState(false);
  const [V38z, V390] = useState("");
  const [V391, V392] = useState(false);
  const [V393, V394] = useState("");
  const [V395, V396] = useState("");
  const V397 = V4x2();

  useEffect(() => {
    if (isTauri()) {
      V3zm()
        .then((u) => V394(u ?? ""))
        .catch(() => {});
    }
  }, []);

  const V398 = async () => {
    const name = V38t.trim();
    if (!name || !V38v || V38x || !V397.ready || !V397.challenge) return;
    V38y(true);
    V390("");
    try {
      await V38q(name, V38v, V397.challenge.id, V397.answer);
    } catch (V399) {
      const V39a = V399 instanceof V3xq ? V399.code : "error";
      const V39b = V399 instanceof Error ? V399.message : String(V399);
      V390(V3zt(V39a, V39b));

      V397.refresh();
    } finally {
      V38y(false);
    }
  };

  const V39c = async () => {
    try {
      await V3zn(V393.trim() || null);
      V396("Đã lưu");
    } catch (V39d) {
      V396(V39d instanceof Error ? V39d.message : String(V39d));
    }
  };

  return (
    <div className="login-screen">
      <div className="login-card">
        <div className="login-logo">DT</div>
        <h1>DigiText</h1>
        <div className="sub">Số hóa &amp; ký số tài liệu</div>

        {V38r.length > 0 && (
          <div className="acct-tiles">
            {V38r.map((a) => (
              <div
                key={a.userId}
                className={`acct-tile ${V38t === a.username ? "sel" : ""}`}
                onClick={() => V38u(a.username)}
                title={`${V3za(a.role)}${a.projectName ? ` · ${a.projectName}` : ""}`}
              >
                <div className="av">{a.displayName[0]?.toUpperCase() ?? "?"}</div>
                <div className="nm">
                  <b>{a.displayName}</b>
                  <small>
                    {a.username}
                    {V3zs(a.offlineExpiry) && (
                      <span className="ml-1 inline-flex items-center gap-1 text-warning-700">
                        <Icon name="canhBao" size={11} />
                        sắp hết hạn ngoại tuyến
                      </span>
                    )}
                  </small>
                </div>
              </div>
            ))}
          </div>
        )}

        <form
          onSubmit={(V39e) => {
            V39e.preventDefault();
            void V398();
          }}
        >
          <div className="field">
            <label htmlFor="login-username">Tài khoản</label>
            <input
              id="login-username"
              name="username"
              type="text"
              autoComplete="username"
              value={V38t}
              autoFocus
              onChange={(V39f) => V38u(V39f.target.value)}
            />
          </div>
          <div className="field">
            <label htmlFor="login-password">Mật khẩu</label>
            <input
              id="login-password"
              name="password"
              type="password"
              autoComplete="current-password"
              value={V38v}
              onChange={(V39g) => V38w(V39g.target.value)}
            />
          </div>

          <V4x3 captcha={V397} onEnter={() => void V398()} disabled={V38x} />

          {}
          {V38z && (
            <div className="login-error" role="alert">
              {V38z}
            </div>
          )}

          <button
            type="submit"
            className="btn primary lg w-full"
            disabled={!V38t.trim() || !V38v || V38x || !V397.ready}
          >
            {V38x ? "Đang đăng nhập…" : "Đăng nhập"}
          </button>
        </form>

        {V391 && (
          <div className="server-config">
            <div className="field">
              <label>Địa chỉ máy chủ</label>
              <input
                type="text"
                value={V393}
                placeholder="http://mac-mini:8080"
                onChange={(V39h) => V394(V39h.target.value)}
              />
            </div>
            <div className="flex items-center gap-2">
              <button type="button" className="btn" onClick={() => void V39c()}>
                Lưu
              </button>
              {V395 && (
                <span className="inline-flex items-center gap-1 text-sm text-fg-subtle">
                  {V395 === "Đã lưu" && (
                    <Icon name="duyet" size={13} className="text-success-700" />
                  )}
                  {V395}
                </span>
              )}
            </div>
            <p className="mt-2 text-sm text-fg-subtle">
              Lần đăng nhập ĐẦU TIÊN của mỗi tài khoản cần kết nối máy chủ; các lần sau dùng ngoại
              tuyến được tới 30 ngày.
            </p>
          </div>
        )}

        <div className="login-devstrip">
          <button
            type="button"
            className="server-gear inline-flex items-center gap-1.5"
            onClick={() => V392((v) => !v)}
            aria-expanded={V391}
            title="Cấu hình máy chủ"
          >
            <Icon name="mayChu" size={13} />
            Máy chủ
          </button>
          <span>v{V38s || "…"}</span>
        </div>
      </div>
    </div>
  );
}
