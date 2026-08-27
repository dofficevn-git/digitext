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
          <img
            className="captcha-img"
            src={V38i.image}
            alt={`TcOjIHjDoWMgbmjhuq1uIGfhu5NtIA==${V4wy}IGvDvSB04bux`}
            onClick={V38l}
            title="QuG6pW0gxJHhu4MgbOG6pXkgbcOjIGtow6Fj"
          />
        ) : (
          <div className="captcha-img captcha-img-empty">
            {V38m ? "xJBhbmcgdOG6o2nigKY=" : V38n ? "S2jDtG5nIHThuqNpIMSRxrDhu6NjIG3Dow==" : ""}
          </div>
        )}
        <button
          type="button"
          className="btn captcha-reload"
          onClick={V38l}
          disabled={V38m}
          title="TOG6pXkgbcOjIGtow6Fj"
          aria-label="TOG6pXkgbcOjIGtow6Fj"
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
        placeholder={`Tmjhuq1wIA==${V4wy}IGvDvSB04buxIHRyb25nIOG6o25o`}
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
