// Copyright (c) 2026 DigiText. All rights reserved.
import V4wx from "@/features/shell/AppShell";
import V4x4 from "@/features/shell/Login";
import V4x5 from "@/features/shell/LockScreen";
import V4x7 from "@/features/shell/UpdateOverlay";
import { V460 } from "@/state";

export default function V4x8() {
  const { status, update: V3aa, dismissUpdate: V3ab } = V460();

  return (
    <>
      {status === "loading" && (
        <div className="login-screen">
          <div className="login-card">
            <div className="login-logo">DT</div>
            <h1>DigiText</h1>
            <div className="sub">Đang khởi động…</div>
          </div>
        </div>
      )}
      {status === "signedOut" && <V4x4 />}
      {status === "locked" && <V4x5 />}
      {status === "active" && <V4wx />}
      <V4x7 status={V3aa} onDismiss={V3ab} />
    </>
  );
}
