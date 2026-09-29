// Copyright (c) 2026 DigiText. All rights reserved.

import { useEffect, useState } from "react";
import { isTauri } from "@/api/ipc";
import { V460 } from "@/state";
import { Icon } from "@/components/Icon";
import { V475 } from "@/api/scanner";
import { V47g, V47i } from "@/api/health";
import { V45w } from "@/live/status";
import { V47q, type V47l } from "./statusItems";
import { syncLabel, useClSync } from "@/features/chinhly/common";
import { clSyncNow } from "@/api/chinhly";

function ClSyncStat() {
  const s = useClSync();
  const l = syncLabel(s);
  const { session: ss } = V460();
  const active = !!ss && !ss.locked && !!ss.projectId;
  // Đăng nhập / mở khoá xong → đồng bộ chỉnh lý ngay (worker nền chỉ chạy mỗi 60 s).
  useEffect(() => {
    if (active && isTauri()) clSyncNow().catch(() => {});
  }, [active]);
  return (
    <div className="stat" title={l.title}>
      <span className={`dot ${l.dot === "unknown" ? "" : l.dot}`} />
      <Icon name="chinhly" size={13} className="flex-none text-fg-subtle" />
      Chỉnh lý: <b>{l.text}</b>
    </div>
  );
}

const V47r = 6_000;
const V47s = 3;

export default function V47t() {
  const { status, session: Vf9 } = V460();
  const { serverLive: Vfa } = V45w();
  const [Vfb, Vfc] = useState<number | null>(null);
  const [Vfd, Vfe] = useState<boolean | null>(null);
  const [Vff, Vfg] = useState<V47l["storage"]>(null);

  useEffect(() => {
    if (!isTauri()) return;
    let Vfh = true;
    let Vfi = 0;

    const Vfj = async () => {
      try {
        const Vfk = await V475();
        if (Vfh) Vfc(Vfk.length);
      } catch {
        if (Vfh) Vfc(null);
      }
    };
    const Vfl = async () => {
      try {
        const up = await V47g();
        if (Vfh) Vfe(up);
      } catch {
        if (Vfh) Vfe(false);
      }
    };
    const Vfm = async () => {
      try {
        const s = await V47i();
        if (Vfh) Vfg({ configured: s.configured, exists: s.exists });
      } catch {
        if (Vfh) Vfg(null);
      }
    };

    const Vfn = () => {
      void Vfl();
      void Vfm();

      if (Vfi % V47s === 0) void Vfj();
      Vfi += 1;
    };

    Vfn();
    const Vfo = setInterval(Vfn, V47r);
    return () => {
      Vfh = false;
      clearInterval(Vfo);
    };
  }, []);

  if (!isTauri()) {
    return (
      <footer className="statusbar">
        <div
          className="stat"
          title="UI đang chạy trong trình duyệt (vite dev), chưa kết nối backend Tauri."
        >
          <span className="dot busy" />
          Chế độ xem trước
        </div>
      </footer>
    );
  }

  const Vfp: V47l = {
    loading: status === "loading",
    session: Vf9 ? { locked: Vf9.locked, mode: Vf9.mode } : null,
    scanners: Vfb,
    serverUp: Vfd,
    storage: Vff,
    live: Vfa,
  };
  const Vfq = V47q(Vfp);

  return (
    <footer className="statusbar">
      {Vfq.map((Vfr) => (
        <div
          key={Vfr.key}
          className={`stat${Vfr.key === "drive" ? " right" : ""}`}
          title={Vfr.title}
        >
          <span className={`dot ${Vfr.dot === "unknown" ? "" : Vfr.dot}`} />
          <Icon name={Vfr.icon} size={13} className="flex-none text-fg-subtle" />
          {Vfr.label}: <b>{Vfr.value}</b>
        </div>
      ))}
      <ClSyncStat />
    </footer>
  );
}
