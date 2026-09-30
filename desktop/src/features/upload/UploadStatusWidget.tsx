// Copyright (c) 2026 DigiText. All rights reserved.

import { useEffect, useMemo, useState } from "react";
import { useNavigation } from "@/state";
import { V44m } from "@/state/uploads";
import { V44b, V44c, V445, V442, V446, V43z, V444, V441, type V43p } from "@/state/uploadsCore";
import { V4wj, V4wk } from "./transferBits";
import "./uploadWidget.css";
import { Icon } from "@/components/Icon";

const V4wo = 1000;

export default function V4wp() {
  const {
    uploads: V363,
    cancel: V364,
    retry: V365,
    queuedRetry: V366,
    dismiss: V367,
    dismissItem: V368,
    toggleCollapsed: V369,
    lastError: V36a,
    notice: V36b,
    running: V36c,
  } = V44m();
  const { gotoServerFile: V36d } = useNavigation();
  const [V36e, V36f] = useState(() => Date.now());

  useEffect(() => {
    if (!V363.running) return;
    const t = setInterval(() => V36f(Date.now()), V4wo);
    return () => clearInterval(t);
  }, [V363.running]);

  const V36g = useMemo(() => V363.items.filter((i) => i.state === "error"), [V363.items]);

  const V36h = useMemo(() => V441(V363), [V363]);

  if (!V442(V363)) return null;

  const V36i = V444(V363);
  const V36j = V36c;

  return (
    <section
      className="uw"
      role="status"
      aria-live="polite"
      aria-label="Trạng thái tải lên"
    >
      <header className="uw-head">
        <span className="uw-title">{V445(V363)}</span>
        <button
          className="uw-iconbtn"
          onClick={V369}
          title={V363.collapsed ? "Mở rộng bảng" : "Thu nhỏ bảng"}
          aria-expanded={!V363.collapsed}
        >
          <Icon name={V363.collapsed ? "len" : "xuong"} size={14} />
        </button>
        <button
          className="uw-iconbtn"
          onClick={V367}
          disabled={V36j}
          title={
            V36j
              ? "Đang tải lên — thu nhỏ bảng, hoặc hủy phiên trước khi đóng"
              : "Đóng bảng"
          }
        >
          <Icon name="dong" size={14} />
        </button>
      </header>

      {!V363.collapsed && (
        <>
          <div className="uw-list">
            {V36h.map((V36k) => (
              <V4wq
                key={V36k.id}
                item={V36k}
                now={V36e}
                onCancel={() => void V364(V36k.id)}
                onRetry={() => void V365(V36k.id)}
                queued={V366.includes(V36k.id)}
                onDrop={() => V368(V36k.id)}
                onReveal={() => V36d({ relPath: V36k.relPath ?? "", filename: V36k.filename })}
              />
            ))}
            {V363.items.length === 0 && (
              <div className="uw-row">
                <span className="uw-detail">Đang chuẩn bị {V363.total} tệp…</span>
              </div>
            )}
          </div>

          <footer className="uw-foot">
            {V36j ? (
              <>
                <span>
                  {V36i.done}/{Math.max(V363.total, V36i.done + V36i.errors + V36i.cancelled)} tệp xong
                </span>
                <span className="flex-1" />
                <button className="uw-linkbtn" onClick={() => void V364()}>
                  Hủy tất cả
                </button>
              </>
            ) : (
              <>
                <span>
                  {V36i.done} tệp lên máy chủ
                  {V36i.errors > 0 ? ` · ${V36i.errors} lỗi` : ""}
                  {V36i.cancelled > 0 ? ` · ${V36i.cancelled} đã hủy` : ""}
                </span>
                <span className="flex-1" />
                {V36g.length > 0 && (
                  <button
                    className="uw-linkbtn"
                    onClick={() => void V365(V36g.map((V36l) => V36l.id))}
                  >
                    Thử lại {V36g.length} tệp lỗi
                  </button>
                )}
              </>
            )}
          </footer>

          {V36a && (
            <div className="uw-foot uw-err" role="alert">
              <Icon name="canhBao" /> {V36a}
            </div>
          )}
          {!V36a && V36b && (
            <div className="uw-foot uw-note" role="status">
              <Icon name="thongTin" /> {V36b}
            </div>
          )}
        </>
      )}
    </section>
  );
}

function V4wq({
  item: V36m,
  now: V36n,
  queued: V36o,
  onCancel: V36p,
  onRetry: V36q,
  onDrop: V36r,
  onReveal: V36s,
}: {
  item: V43p;
  now: number;

  queued: boolean;
  onCancel: () => void;
  onRetry: () => void;
  onDrop: () => void;
  onReveal: () => void;
}) {
  const V36t = V36m.state === "done" || V36m.state === "skipped";
  const V36u = V44b(V36m);

  return (
    <div className={`uw-row${V36m.state === "error" ? " err" : ""}`}>
      <V4wr item={V36m} />
      <div className="uw-body">
        {V36t ? (
          <button
            className="uw-name"
            onClick={V36s}
            title={`Mở vị trí lưu: ${V36m.filename}`}
          >
            {V36m.filename}
          </button>
        ) : (
          <span className="uw-name" title={V36m.filename}>
            {V36m.filename}
          </span>
        )}
        <div className={`uw-detail${V36m.state === "error" ? " err" : ""}`}>
          {V446(V36m, V36n)}
          {V36o ? " · đang chờ gửi lại" : ""}
        </div>
      </div>
      {V44c(V36m) && (
        <button
          className={`uw-iconbtn${V36o ? " queued" : ""}`}
          onClick={V36q}
          title={
            V36o
              ? "Đang chờ tới lượt gửi lại"
              : "Gửi lại tệp này lên máy chủ"
          }
        >
          <Icon name={V36o ? "dangCho" : "lamMoi"} size={14} />
        </button>
      )}
      {V36t && (
        <button
          className="uw-iconbtn"
          onClick={V36s}
          title="Đi đến vị trí lưu tệp"
        >
          <Icon name="thuMuc" size={14} />
        </button>
      )}
      <button
        className="uw-iconbtn"
        onClick={V36u ? V36p : V36r}
        title={V36u ? "Hủy tải tệp này" : "Ẩn khỏi bảng"}
      >
        <Icon name="dong" size={14} />
      </button>
    </div>
  );
}

function V4wr({ item: V36v }: { item: V43p }) {
  if (V36v.state === "uploading") return <V4wj pct={V43z(V36v)} label="Đang tải lên" />;

  return <V4wk state={V36v.state} />;
}
