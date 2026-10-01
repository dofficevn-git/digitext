// Copyright (c) 2026 DigiText. All rights reserved.
// Thanh trạng thái: Máy chủ · Phiên · Máy quét (khi có quyền quét) · Chỉnh lý · Ổ lưu trữ (chỉ khi lỗi).
// Mỗi ô là nút: bấm mở Cài đặt / Quét / Đồng bộ. Hỏi lại mỗi 6 s, tự dừng khi cửa sổ ẩn.

import { useCallback, useEffect, useRef, useState } from "react";
import { isTauri } from "@/api/ipc";
import { V460 } from "@/state";
import { V3z9 } from "@/domain/permissions";
import { V3zr } from "@/state/sessionCore";
import { Icon } from "@/components/Icon";
import { V475 } from "@/api/scanner";
import { V47g, V47i } from "@/api/health";
import { V45w } from "@/live/status";
import { V41p } from "@/live/useLive";
import { V462 as cn } from "@/lib/utils";
import { Tip } from "@/ui";
import { V47q, type V47l, type V47m } from "./statusItems";
import { syncLabel, useClSync } from "@/features/chinhly/common";
import { clSyncNow } from "@/api/chinhly";

function Chip({ dot, icon, label, value, title, onClick, right }: { dot: string; icon: React.ComponentProps<typeof Icon>["name"]; label: string; value: string; title: string; onClick?: () => void; right?: boolean }) {
  const inner = (
    <>
      <span className={cn("dot", dot !== "unknown" && dot)} />
      <Icon name={icon} size={13} className="flex-none text-fg-subtle" />
      <span className="text-fg-subtle">{label}:</span>
      <b>{value}</b>
    </>
  );
  return (
    <Tip text={title}>
      {onClick ? (
        <button type="button" className={cn("stat", right && "right")} onClick={onClick}>
          {inner}
        </button>
      ) : (
        <div className={cn("stat", right && "right")}>{inner}</div>
      )}
    </Tip>
  );
}

function ClSyncChip() {
  const s = useClSync();
  const l = syncLabel(s);
  const { session, gotoHoSo, project } = V460();
  const active = !!session && !session.locked && !!project.projectId;
  useEffect(() => {
    if (active && isTauri()) clSyncNow().catch(() => {});
  }, [active]);
  if (!session || !V3z9(session.permissions, "dossier.read")) return null;
  return <Chip dot={l.dot} icon="chinhly" label="Chỉnh lý" value={l.text} title={l.title} onClick={() => gotoHoSo("dongbo")} />;
}

export default function StatusBar() {
  const { status, session, goto } = V460();
  const { serverLive } = V45w();
  const [scanners, setScanners] = useState<number | null>(null);
  const [serverUp, setServerUp] = useState<boolean | null>(null);
  const [storage, setStorage] = useState<V47l["storage"]>(null);
  const canScan = !!session && V3z9(session.permissions, "scan.view");

  // Một lượt ping hỏng (máy chủ đang bận, mạng nghẽn thoáng qua) chưa phải mất kết nối: chỉ báo sau 2 lượt liên tiếp.
  const pingHong = useRef(0);
  const pollMain = useCallback(async () => {
    const [up, st] = await Promise.all([V47g().catch(() => false), V47i().catch(() => null)]);
    pingHong.current = up ? 0 : pingHong.current + 1;
    setServerUp((cu) => (up ? true : pingHong.current >= 2 || cu === null ? false : cu));
    setStorage(st ? { configured: st.configured, exists: st.exists } : null);
  }, []);
  V41p(pollMain, { topics: ["session", "settings"], enabled: isTauri(), pollMs: 6_000 });

  const pollScanner = useCallback(async () => {
    try {
      setScanners((await V475()).length);
    } catch {
      setScanners(null);
    }
  }, []);
  V41p(pollScanner, { topics: ["settings"], enabled: isTauri() && canScan, pollMs: 30_000 });

  if (!isTauri()) {
    return (
      <footer className="statusbar">
        <div className="stat">
          <span className="dot busy" />
          Bản xem trước trên trình duyệt
        </div>
      </footer>
    );
  }

  const input: V47l = {
    loading: status === "loading",
    session: session ? { locked: session.locked, mode: session.mode, offlineDays: V3zr(session.offlineExpiry) } : null,
    scanners,
    serverUp,
    storage,
    live: serverLive,
  };
  const items = V47q(input, { showScanner: canScan });
  const open = (key: V47m) => {
    if (key === "scanner") goto("scan");
    else goto("settings");
  };

  return (
    <footer className="statusbar">
      {items.map((it) => (
        <Chip key={it.key} dot={it.dot} icon={it.icon} label={it.label} value={it.value} title={it.title} onClick={() => open(it.key)} right={it.key === "drive"} />
      ))}
      <ClSyncChip />
    </footer>
  );
}
