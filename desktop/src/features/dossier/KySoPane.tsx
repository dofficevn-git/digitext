// Copyright (c) 2026 DigiText. All rights reserved.

import { V460 } from "@/state";
import Banner from "./Banner";
import { Icon } from "@/components/Icon";

export default function V4hd() {
  const { goto } = V460();
  return (
    <>
      <Banner
        tone="info"
        text="Ký số PAdES trên desktop dùng USB token qua màn “Ký số”. Bấm nút bên dưới để mở hàng đợi ký số."
      />
      <div className="py-6 px-1">
        <button className="btn primary" onClick={() => goto("sign")}>
          <Icon name="kySo" />
          Mở màn Ký số
        </button>
      </div>
    </>
  );
}
