// Copyright (c) 2026 DigiText. All rights reserved.

import { useEffect, useState } from "react";
import { V45k } from "@/state/collects";
import { V45e, V45b, V458, V459, V454, V456, V455, type V44v } from "@/state/collectsCore";
import type { V44o } from "@/api/collectProgress";
import { V4wj, V4wk } from "./transferBits";
import "./uploadWidget.css";
import { Icon } from "@/components/Icon";

const V4wl = 1000;

export default function V4wm() {
  const {
    collects: V35k,
    cancel: V35l,
    dismiss: V35m,
    dismissItem: V35n,
    toggleCollapsed: V35o,
    lastError: V35p,
    running: V35q,
  } = V45k();
  const [V35r, V35s] = useState(() => Date.now());

  useEffect(() => {
    if (!V35k.running) return;
    const t = setInterval(() => V35s(Date.now()), V4wl);
    return () => clearInterval(t);
  }, [V35k.running]);

  if (!V456(V35k)) return null;

  const V35t = V45b(V35k, V35q);

  const V35u = V455(V35k);

  return (
    <section className="uw" role="status" aria-live="polite" aria-label={V35t.title}>
      <header className="uw-head">
        <span className="uw-title">
          <Icon name={V35t.icon} size={14} /> {V458(V35k)}
        </span>
        <button
          className="uw-iconbtn"
          onClick={V35o}
          title={V35k.collapsed ? "Mở rộng bảng" : "Thu nhỏ bảng"}
          aria-expanded={!V35k.collapsed}
        >
          <Icon name={V35k.collapsed ? "len" : "xuong"} size={14} />
        </button>
        <button
          className="uw-iconbtn"
          onClick={V35m}
          disabled={V35q}
          title={V35q ? V35t.closeBusy : "Đóng bảng"}
        >
          <Icon name="dong" size={14} />
        </button>
      </header>

      {!V35k.collapsed && (
        <>
          {V35k.stage !== "" && <div className="uw-stage">{V35k.stage}</div>}

          <div className="uw-list">
            {V35u.map((V35v) => (
              <V4wn
                key={V35v.key}
                item={V35v}
                now={V35r}
                kind={V35k.kind}
                direct={V35k.direct}
                cancelTitle={V35t.cancelRow}
                onCancel={() => void V35l(V35v.key)}
                onDrop={() => V35n(V35v.key)}
              />
            ))}
            {V35k.items.length === 0 && (
              <div className="uw-row">
                <span className="uw-detail">{V35t.empty}</span>
              </div>
            )}
          </div>

          <footer className="uw-foot">
            <span>{V35t.foot}</span>
            {V35q && (
              <>
                <span className="flex-1" />
                <button className="uw-linkbtn" onClick={() => void V35l()}>
                  {V35t.stop}
                </button>
              </>
            )}
          </footer>

          {V35p && (
            <div className="uw-foot uw-err" role="alert">
              <Icon name="canhBao" /> {V35p}
            </div>
          )}
        </>
      )}
    </section>
  );
}

function V4wn({
  item: V35w,
  now: V35x,
  kind: V35y,
  direct: V363,
  cancelTitle: V35z,
  onCancel: V360,
  onDrop: V361,
}: {
  item: V44v;
  now: number;
  kind: V44o;
  direct: boolean;
  cancelTitle: string;
  onCancel: () => void;
  onDrop: () => void;
}) {
  const V362 = V45e(V35w);
  const cho =
    V35w.hoSoRef ?? (V35y === "disk" ? "Chưa phân loại" : "chưa rõ hồ sơ");
  return (
    <div className={`uw-row${V35w.state === "error" ? " err" : ""}`}>
      {V35w.state === "downloading" ? (
        <V4wj
          pct={V454(V35w)}
          label={V363 ? "Đang ghi sổ" : V35y === "disk" ? "Đang chép vào Kho" : "Đang gom về máy"}
        />
      ) : (
        <V4wk state={V35w.state === "skipped" ? "skipped" : V35w.state} />
      )}
      <div className="uw-body">
        <span className="uw-name" title={`${V35w.filename} — ${cho}`}>
          {V35w.filename}
        </span>
        <div className={`uw-detail${V35w.state === "error" ? " err" : ""}`}>
          {V459(V35w, V35x, V35y, V363)}
        </div>
      </div>
      <button
        className="uw-iconbtn"
        onClick={V362 ? V360 : V361}
        title={V362 ? V35z : "Ẩn khỏi bảng"}
      >
        <Icon name="dong" size={14} />
      </button>
    </div>
  );
}
