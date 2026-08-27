// Copyright (c) 2026 DigiText. All rights reserved.
import { useState } from "react";
import { V460 } from "@/state";
import { V41y } from "@/mocks/fixtures";
import { V40x } from "@/api/archive";
import V47w from "@/ui/Avatars";
import V47y from "@/ui/Badge";
import { Icon } from "@/components/Icon";

interface V480 {
  kids: string[];
  marked: Set<string>;
  onToggleMark: (V481: string) => void;
  onMove: (V482: string, V483: string[]) => void;
}

function V484(b?: number): string {
  if (!b) return "—";
  if (b < 1024) return `${b} B`;
  if (b < 1024 * 1024) return `${(b / 1024).toFixed(0)} KB`;
  return `${(b / 1024 / 1024).toFixed(1)} MB`;
}

export default function V485({ kids: Vfx, marked: Vfy, onToggleMark: Vfz, onMove: Vg0 }: V480) {
  const { nodes: Vg1, selected: Vg2, select, enterFolder: Vg3, openPreview: Vg4 } = V460();
  const [Vg5, Vg6] = useState<string | null>(null);

  const Vg7 = (Vg8: string): string[] =>
    Vfy.has(Vg8) && Vfy.size > 0 ? [...Vfy].map((m) => Vg1[m]?.path ?? m) : [Vg1[Vg8]?.path ?? Vg8];

  const Vg9 = (Vga: React.DragEvent, Vgb: string) => {
    Vga.dataTransfer.setData("text/plain", JSON.stringify(Vg7(Vgb)));
    Vga.dataTransfer.effectAllowed = "move";
  };
  const Vgc = (Vgd: React.DragEvent, Vge?: string) => {
    Vgd.preventDefault();
    Vg6(null);
    if (!Vge) return;
    try {
      const Vgf = JSON.parse(Vgd.dataTransfer.getData("text/plain")) as string[];
      if (Array.isArray(Vgf) && Vgf.length) Vg0(Vge, Vgf);
    } catch {}
  };

  return (
    <div className="cards">
      {Vfx.map((Vgg) => {
        const n = Vg1[Vgg];
        const Vgh = `${Vg2 === Vgg ? " sel" : ""}${Vfy.has(Vgg) ? " marked" : ""}`;

        if (n.type === "folder") {
          return (
            <div
              key={Vgg}
              className={`fcard${Vgh}${Vg5 === Vgg ? " dragover" : ""}`}
              draggable
              onDragStart={(Vgi) => Vg9(Vgi, Vgg)}
              onClick={() => select(Vgg)}
              onDoubleClick={() => Vg3(Vgg)}
              onDragOver={(Vgj) => {
                if (n.path) {
                  Vgj.preventDefault();
                  Vg6(Vgg);
                }
              }}
              onDragLeave={() => Vg6((d) => (d === Vgg ? null : d))}
              onDrop={(Vgk) => Vgc(Vgk, n.path)}
            >
              <input
                type="checkbox"
                className="card-check"
                checked={Vfy.has(Vgg)}
                onClick={(Vgl) => Vgl.stopPropagation()}
                onChange={() => Vfz(Vgg)}
              />
              <div className="ficon">
                <Icon name="thuMuc" size={28} />
              </div>
              <div className="fname">{n.name}</div>
              <div className="fmeta">
                <V47w members={n.members} />
                <span>{(n.kids ?? []).length} mục</span>
              </div>
            </div>
          );
        }

        const Vgm = V41y[n.status ?? "scanned"];
        return (
          <div
            key={Vgg}
            className={`fcard doc${Vgh}`}
            draggable
            onDragStart={(Vgn) => Vg9(Vgn, Vgg)}
            title={n.path ? "QuG6pW0gxJHhu4MgbeG7nyBQREY=" : undefined}
            onClick={() => {
              select(Vgg);
              if (n.path) void V40x(n.path).catch(() => {});
            }}
            onDoubleClick={() => (n.path ? void V40x(n.path).catch(() => {}) : Vg4(n))}
          >
            <input
              type="checkbox"
              className="card-check"
              checked={Vfy.has(Vgg)}
              onClick={(Vgo) => Vgo.stopPropagation()}
              onChange={() => Vfz(Vgg)}
            />
            <div className="thumbwrap">
              <div className="mini" />
              <V47y tone={Vgm.tone}>
                <Icon name={Vgm.icon} size={12} />
                {Vgm.label}
              </V47y>
            </div>
            <div className="docbody">
              <div className="fname">{n.name}.pdf</div>
              <div className="fmeta">{V484(n.size)}</div>
            </div>
          </div>
        );
      })}
    </div>
  );
}
