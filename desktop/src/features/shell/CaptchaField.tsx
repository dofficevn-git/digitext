// Copyright (c) 2026 DigiText. All rights reserved.

import { Icon } from "@/components/Icon";
import { V4wy } from "@/state/captchaCore";
import type { V4x1 } from "./useCaptcha";

export default function V4x3({
  captcha: V38g,
  onEnter: V38h,
  disabled,
}: {
  captcha: V4x1;
  onEnter: () => void;
  disabled?: boolean;
}) {
  const {
    challenge: V38i,
    answer: V38j,
    setAnswer: V38k,
    refresh: V38l,
    loading: V38m,
    loadError: V38n,
  } = V38g;

  return (
    <div className="field captcha-field">
      {}
      <label htmlFor="captcha-answer">Mã xác nhận</label>
      <div className="captcha-row">
        {V38i ? (
          <button type="button" className="captcha-img border-0 bg-transparent p-0" onClick={V38l} title="Lấy mã khác">
            <img src={V38i.image} alt={`Mã xác nhận gồm ${V4wy} ký tự`} className="block h-full w-full" />
          </button>
        ) : (
          <div className="captcha-img captcha-img-empty">
            {V38m ? "Đang tải…" : V38n ? "Không tải được mã" : ""}
          </div>
        )}
        <button
          type="button"
          className="btn captcha-reload"
          onClick={V38l}
          disabled={V38m}
          title="Lấy mã khác"
          aria-label="Lấy mã khác"
        >
          <Icon name="lamMoi" size={15} />
        </button>
      </div>
      <input
        id="captcha-answer"
        type="text"
        className="inp captcha-input"
        value={V38j}
        maxLength={V4wy}
        autoComplete="off"
        spellCheck={false}
        placeholder={`Nhập ${V4wy} ký tự trong ảnh`}
        disabled={disabled}
        onChange={(V38o) => V38k(V38o.target.value)}
        onKeyDown={(V38p) => V38p.key === "Enter" && V38h()}
      />
      {V38n && (
        <p className="mt-1.5 text-sm text-fg-subtle">
          Không lấy được mã xác nhận — bấm nút làm mới để thử lại.
        </p>
      )}
    </div>
  );
}
