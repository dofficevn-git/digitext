// Copyright (c) 2026 DigiText. All rights reserved.
// Màn khoá: mở khoá bằng mật khẩu của người đang dùng, hoặc đổi sang tài khoản khác trên máy này.

import { useState } from "react";
import { Icon } from "@/components/Icon";
import { V460 } from "@/state";
import { V3xq } from "@/api/ipc";
import { V3zt } from "@/state/sessionCore";
import { V3za } from "@/domain/permissions";
import V4x3 from "./CaptchaField";
import { V4x2 } from "./useCaptcha";
import { Banner, Button, Field, Input } from "@/ui";

export default function V4x5() {
  const { session, unlock, switchUser, logout } = V460();
  const [mode, setMode] = useState<"unlock" | "switch">("unlock");
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState("");
  const captcha = V4x2();

  if (!session) return null;
  const doi = mode === "switch";

  const chay = async (f: () => Promise<void>) => {
    if (busy) return;
    setBusy(true);
    setErr("");
    try {
      await f();
      setPassword("");
    } catch (e) {
      const code = e instanceof V3xq ? e.code : "error";
      setErr(V3zt(code, e instanceof Error ? e.message : String(e)));
      if (doi) captcha.refresh();
    } finally {
      setBusy(false);
    }
  };

  const submit = () => {
    if (mode === "unlock") {
      if (!password) return;
      void chay(() => unlock(password));
    } else {
      if (!username.trim() || !password || !captcha.ready || !captcha.challenge) return;
      const { id } = captcha.challenge;
      void chay(() => switchUser(username.trim(), password, id, captcha.answer));
    }
  };

  return (
    <div className="login-screen">
      <div className="login-card">
        <div className="lock-avatar" aria-hidden>
          {session.displayName[0]?.toUpperCase() ?? "?"}
        </div>
        <h1>{mode === "unlock" ? session.displayName : "Đổi người dùng"}</h1>
        <div className="sub">{mode === "unlock" ? `${V3za(session.role)}${session.projectName ? ` · ${session.projectName}` : ""} · đang khoá` : "Đăng nhập bằng tài khoản khác trên máy này"}</div>

        <form
          className="space-y-3 text-left"
          onSubmit={(e) => {
            e.preventDefault();
            submit();
          }}
        >
          {doi && (
            <Field label="Tài khoản">
              <Input name="username" type="text" autoComplete="username" value={username} autoFocus onChange={(e) => setUsername(e.target.value)} />
            </Field>
          )}
          <Field label="Mật khẩu">
            <Input name="password" type="password" autoComplete="current-password" value={password} autoFocus={mode === "unlock"} onChange={(e) => setPassword(e.target.value)} />
          </Field>

          {doi && <V4x3 captcha={captcha} onEnter={submit} disabled={busy} />}

          {err && (
            <Banner tone="danger" compact>
              {err}
            </Banner>
          )}

          <Button type="submit" variant="primary" className="w-full" loading={busy} disabled={!password || (doi && (!username.trim() || !captcha.ready))}>
            {mode === "unlock" ? "Mở khoá" : "Đăng nhập"}
          </Button>
        </form>

        <div className="login-devstrip">
          <button
            type="button"
            className="server-gear inline-flex items-center gap-1.5"
            onClick={() => {
              setMode((m) => {
                if (m === "unlock") captcha.refresh();
                return m === "unlock" ? "switch" : "unlock";
              });
              setErr("");
              setPassword("");
            }}
          >
            {mode === "unlock" ? (
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
          <button type="button" className="server-gear" onClick={() => void logout()}>
            Đăng xuất
          </button>
        </div>
      </div>
    </div>
  );
}
