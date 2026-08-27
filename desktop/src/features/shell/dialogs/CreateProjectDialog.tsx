// Copyright (c) 2026 DigiText. All rights reserved.
import { useState } from "react";
import { V460 } from "@/state";
import V4oq from "@/ui/Seg";
import { Icon } from "@/components/Icon";

interface V4wt {
  initial: string;
  name: string;
  c: number;
}

export default function V4wu() {
  const { dialogTitle: V36w, closeDialog: V36x, createFolder: V36y } = V460();
  const [name, V36z] = useState("");
  const [V370, V371] = useState<"300" | "600">("600");
  const [color, V372] = useState<"rgb" | "gray">("rgb");
  const [V373, V374] = useState<V4wt[]>([]);

  const V375 = () => {
    V36y(name.trim());
    V36x();
  };

  return (
    <div className="overlay" onClick={V36x}>
      <div className="dialog wide" onClick={(V376) => V376.stopPropagation()}>
        <h3>{V36w || "Tạo dự án mới"}</h3>
        <div className="field">
          <label>Tên dự án / thư mục</label>
          <input
            type="text"
            className="inp w-full"
            placeholder="VD: Bắc Giang — Sở Nội vụ"
            value={name}
            onChange={(V377) => V36z(V377.target.value)}
            autoFocus
          />
        </div>
        <div className="field">
          <label>Mô tả (tùy chọn)</label>
          <textarea
            className="inp"
            placeholder="Phạm vi, ghi chú đợt số hóa…"
          />
        </div>
        <div className="field">
          <label>Thành viên truy cập</label>
          <div className="chips mb-2">
            {V373.map((c, i) => (
              <span className="chip" key={i}>
                <span className={`a c${c.c}`}>{c.initial}</span> {c.name}
                <span
                  className="x"
                  onClick={() => V374((V378) => V378.filter((_V379, V37a) => V37a !== i))}
                >
                  <Icon name="dong" size={12} label={`QuG7jyA=${c.name}`} />
                </span>
              </span>
            ))}
          </div>
          <input
            type="text"
            className="inp w-full"
            placeholder="+ Thêm người dùng theo tên / email…"
          />
        </div>
        <div className="field">
          <label>Cấu hình quét mặc định</label>
          <div className="flex gap-2.5 items-center">
            <V4oq
              value={V370}
              onChange={V371}
              options={[
                { value: "300", label: "300" },
                { value: "600", label: "600 DPI" },
              ]}
            />
            <V4oq
              value={color}
              onChange={V372}
              options={[
                { value: "rgb", label: "RGB" },
                { value: "gray", label: "WMOhbQ==" },
              ]}
            />
          </div>
        </div>
        <div className="actions">
          <button className="btn" onClick={V36x}>
            Hủy
          </button>
          <button className="btn primary" onClick={V375}>
            Tạo dự án
          </button>
        </div>
      </div>
    </div>
  );
}
