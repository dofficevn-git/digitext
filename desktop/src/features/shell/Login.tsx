// Copyright (c) 2026 DigiText. All rights reserved.
// Đăng nhập: chọn nhanh tài khoản đã dùng trên máy, tài khoản + mật khẩu + mã xác nhận, cấu hình máy chủ gập.

import { useEffect, useState } from "react";
import { Icon } from "@/components/Icon";
import { V460 } from "@/state";
import { V3zm, V3zn } from "@/api/auth";
import { isTauri, V3xq } from "@/api/ipc";
import { V3zt, V3zs } from "@/state/sessionCore";
import { V3za } from "@/domain/permissions";
import V4x3 from "./CaptchaField";
import { V4x2 } from "./useCaptcha";
import { Banner, Button, Field, Input, notifyError, toast } from "@/ui";

export default function V4x4() {
  const { login, accounts, appVersion } = V460();
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState("");
  const [moMayChu, setMoMayChu] = useState(false);
  const [serverUrl, setServerUrl] = useState("");
  const captcha = V4x2();

  useEffect(() => {
    if (isTauri()) {
      V3zm()
        .then((u) => setServerUrl(u ?? ""))
        .catch(() => {});
    }
  }, []);

  const dangNhap = async () => {
    const name = username.trim();
    if (!name || !password || busy || !captcha.ready || !captcha.challenge) return;
    setBusy(true);
    setErr("");
    try {
      await login(name, password, captcha.challenge.id, captcha.answer);
    } catch (e) {
      const code = e instanceof V3xq ? e.code : "error";
      setErr(V3zt(code, e instanceof Error ? e.message : String(e)));
      if (code === "server_not_configured") setMoMayChu(true);
      captcha.refresh();
    } finally {
      setBusy(false);
    }
  };

  const luuMayChu = async () => {
    try {
      await V3zn(serverUrl.trim() || null);
      toast.success("Đã lưu địa chỉ máy chủ.");
    } catch (e) {
      notifyError(e, "lưu địa chỉ máy chủ");
    }
  };

  return (
    <div className="login-screen">
      <div className="login-card">
        <div className="login-logo" aria-hidden>
          DT
        </div>
        <h1>DigiText</h1>
        <div className="sub">Số hoá và ký số tài liệu</div>

        {accounts.length > 0 && (
          <div className="acct-tiles" role="group" aria-label="Tài khoản đã dùng trên máy này">
            {accounts.map((a) => (
              <button
                key={a.userId}
                type="button"
                className={`acct-tile ${username === a.username ? "sel" : ""}`}
                aria-pressed={username === a.username}
                onClick={() => setUsername(a.username)}
              >
                <div className="av" aria-hidden>
                  {a.displayName[0]?.toUpperCase() ?? "?"}
                </div>
                <div className="nm">
                  <b>{a.displayName}</b>
                  <small>
                    {a.username} · {V3za(a.role)}
                    {a.projectName ? ` · ${a.projectName}` : ""}
                    {V3zs(a.offlineExpiry) && (
                      <span className="ml-1 inline-flex items-center gap-1 text-warning-700">
                        <Icon name="canhBao" size={11} />
                        sắp hết hạn ngoại tuyến
                      </span>
                    )}
                  </small>
                </div>
              </button>
            ))}
          </div>
        )}

        <form
          className="space-y-3 text-left"
          onSubmit={(e) => {
            e.preventDefault();
            void dangNhap();
          }}
        >
          <Field label="Tài khoản">
            <Input name="username" type="text" autoComplete="username" value={username} autoFocus onChange={(e) => setUsername(e.target.value)} />
          </Field>
          <Field label="Mật khẩu">
            <Input name="password" type="password" autoComplete="current-password" value={password} onChange={(e) => setPassword(e.target.value)} />
          </Field>

          <V4x3 captcha={captcha} onEnter={() => void dangNhap()} disabled={busy} />

          {err && (
            <Banner tone="danger" compact>
              {err}
            </Banner>
          )}

          <Button type="submit" variant="primary" className="w-full" loading={busy} disabled={!username.trim() || !password || !captcha.ready}>
            Đăng nhập
          </Button>
        </form>

        {moMayChu && (
          <div className="server-config space-y-3">
            <Field label="Địa chỉ máy chủ">
              <div className="flex gap-2">
                <Input type="text" value={serverUrl} placeholder="http://may-chu:8080" spellCheck={false} onChange={(e) => setServerUrl(e.target.value)} />
                <Button onClick={() => void luuMayChu()}>Lưu</Button>
              </div>
            </Field>
            <p className="text-sm text-fg-subtle">Lần đăng nhập đầu tiên của mỗi tài khoản cần kết nối máy chủ. Các lần sau dùng ngoại tuyến được tới 30 ngày.</p>
          </div>
        )}

        <div className="login-devstrip">
          <button type="button" className="server-gear inline-flex items-center gap-1.5" onClick={() => setMoMayChu((v) => !v)} aria-expanded={moMayChu}>
            <Icon name="mayChu" size={13} />
            Máy chủ
          </button>
          <span>v{appVersion || "…"}</span>
        </div>
      </div>
    </div>
  );
}
