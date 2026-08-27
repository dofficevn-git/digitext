// Copyright (c) 2026 DigiText. All rights reserved.

import { Icon } from "@/components/Icon";

export default function Banner({
  tone: Vw8,
  text,
  onClose: Vw9,
}: {
  tone: "err" | "ok" | "info";
  text: string;
  onClose?: () => void;
}) {
  const icon = Vw8 === "err" ? "canhBao" : Vw8 === "ok" ? "duyet" : "thongTin";
  return (
    <div className={`banner ${Vw8}`}>
      <span className="bi">
        <Icon name={icon} size={18} />
      </span>
      <span className="bx">{text}</span>
      {Vw9 && (
        <button className="linkbtn" onClick={Vw9}>
          <Icon name="dong" size={14} label="xJDDs25n" />
        </button>
      )}
    </div>
  );
}
