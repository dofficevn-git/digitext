// Copyright (c) 2026 DigiText. All rights reserved.
import { V460 } from "@/state";
import { Icon } from "@/components/Icon";

export default function V4ww() {
  const { previewDoc: V37y, closePreview: V37z, goto } = V460();
  if (!V37y) return null;

  return (
    <div className="lightbox">
      <div className="lb-bar">
        <span className="lb-title">{V37y.name}.pdf</span>
        <span className="muted text-fg-subtle">
          {V37y.pages != null ? `· ${V37y.pages} trang` : ""}
        </span>
        <div className="spacer" />
        <button
          onClick={() => {
            V37z();
            goto("editor");
          }}
        >
          <Icon name="anh" />
          Mở trong trình xử lý
        </button>
        <button>
          <Icon name="taiVe" />
          Tải xuống
        </button>
        <button
          className="primary"
          onClick={() => {
            V37z();
            goto("sign");
          }}
        >
          <Icon name="kySo" />
          Gửi ký số
        </button>
        <button onClick={V37z}>
          <Icon name="dong" />
          Đóng
        </button>
      </div>
      <div className="lb-stage">
        <div className="lb-page" />
        <div className="lb-page" />
      </div>
    </div>
  );
}
