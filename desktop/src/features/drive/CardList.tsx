// Copyright (c) 2026 DigiText. All rights reserved.
import { useState } from "react";
import { V460 } from "@/state";
import { V41y } from "@/mocks/fixtures";
import { V40y } from "@/api/archive";
import { Icon } from "@/components/Icon";

interface V486 {
  kids: string[];
  marked: Set<string>;
  onToggleMark: (V487: string) => void;
  onMove: (V488: string, V489: string[]) => void;
}

function V48a(b?: number): string {
  if (!b) return "—";
  if (b < 1024) return `${b} B`;
  if (b < 1024 * 1024) return `${(b / 1024).toFixed(0)} KB`;
  return `${(b / 1024 / 1024).toFixed(1)} MB`;
}

export default function V48b({ kids: Vgp, marked: Vgq, onToggleMark: Vgr, onMove: Vgs }: V486) {
  const { nodes: Vgt, select, enterFolder: Vgu, openPreview: Vgv } = V460();
  const [Vgw, Vgx] = useState<string | null>(null);

  const Vgy = (Vgz: string): string[] =>
    Vgq.has(Vgz) && Vgq.size > 0 ? [...Vgq].map((m) => Vgt[m]?.path ?? m) : [Vgt[Vgz]?.path ?? Vgz];
  const Vh0 = (Vh1: React.DragEvent, Vh2: string) => {
    Vh1.dataTransfer.setData("text/plain", JSON.stringify(Vgy(Vh2)));
    Vh1.dataTransfer.effectAllowed = "move";
  };
  const Vh3 = (Vh4: React.DragEvent, Vh5?: string) => {
    Vh4.preventDefault();
    Vgx(null);
    if (!Vh5) return;
    try {
      const Vh6 = JSON.parse(Vh4.dataTransfer.getData("text/plain")) as string[];
      if (Array.isArray(Vh6) && Vh6.length) Vgs(Vh5, Vh6);
    } catch {}
  };

  return (
    <div className="tbl-scroll">
      <table className="tbl drive-list">
        <thead>
          <tr>
            <th className="w-7" />
            <th>Tên</th>
            <th>Chủ sở hữu</th>
            <th className="num">Kích thước</th>
            <th>Trạng thái</th>
          </tr>
        </thead>
        <tbody>
          {Vgp.map((Vh7) => {
            const n = Vgt[Vh7];
            const Vh8 = n.type === "folder";
            const Vh9 = V41y[n.status ?? "scanned"];
            return (
              <tr
                key={Vh7}
                className={`${Vgq.has(Vh7) ? "marked" : ""}${Vgw === Vh7 ? " dragover" : ""}`}
                draggable
                onDragStart={(Vha) => Vh0(Vha, Vh7)}
                onClick={() => select(Vh7)}
                onDoubleClick={() => (Vh8 ? Vgu(Vh7) : n.path ? void V40y(n.path) : Vgv(n))}
                onDragOver={
                  Vh8
                    ? (Vhb) => {
                        if (n.path) {
                          Vhb.preventDefault();
                          Vgx(Vh7);
                        }
                      }
                    : undefined
                }
                onDragLeave={Vh8 ? () => Vgx((d) => (d === Vh7 ? null : d)) : undefined}
                onDrop={Vh8 ? (Vhc) => Vh3(Vhc, n.path) : undefined}
              >
                <td onClick={(Vhd) => Vhd.stopPropagation()}>
                  <input type="checkbox" checked={Vgq.has(Vh7)} onChange={() => Vgr(Vh7)} />
                </td>
                <td className="nm">
                  <span className="ic">
                    <Icon name={Vh8 ? "thuMuc" : "taiLieu"} size={15} />
                  </span>
                  {Vh8 ? n.name : `${n.name}.pdf`}
                </td>
                <td>{n.owner ?? "Tôi"}</td>
                <td className="num">{Vh8 ? `${(n.kids ?? []).length} mục` : V48a(n.size)}</td>
                <td>
                  {Vh8 ? (
                    <span className="muted">—</span>
                  ) : (
                    <span className={`badge ${Vh9.tone}`}>
                      <Icon name={Vh9.icon} size={12} />
                      {Vh9.label}
                    </span>
                  )}
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}
