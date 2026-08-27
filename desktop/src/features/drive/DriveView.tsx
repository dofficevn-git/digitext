// Copyright (c) 2026 DigiText. All rights reserved.
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { V41p } from "@/live/useLive";
import { confirm, message } from "@tauri-apps/plugin-dialog";
import { V460 } from "@/state";
import { V412, V419, V415 } from "@/api/archive";
import type { V418 } from "@/api/archive";
import { V47u } from "@/features/drive/scanTarget";
import V485 from "@/features/drive/CardGrid";
import V48b from "@/features/drive/CardList";
import V48d from "@/features/drive/DetailsPanel";
import { Icon } from "@/components/Icon";

export default function DriveView() {
  const {
    nodes: Vin,
    path,
    driveMode: Vio,
    setDriveMode: Vip,
    navigateCrumb: Viq,
    openDialog: Vir,
    goto,
    setScanTarget: Vis,
    reloadArchive: Vit,
  } = V460();
  const [query, Viu] = useState("");
  const [Viv, Viw] = useState(false);
  const [Vix, Viy] = useState<"name" | "size">("name");
  const [Viz, Vj0] = useState<"asc" | "desc">("asc");
  const [Vj1, Vj2] = useState<Set<string>>(new Set());
  const Vj3 = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!Viv) return;
    const Vj4 = (Vj5: MouseEvent) => {
      if (Vj3.current && !Vj3.current.contains(Vj5.target as Node)) Viw(false);
    };
    document.addEventListener("click", Vj4);
    return () => document.removeEventListener("click", Vj4);
  }, [Viv]);

  V41p(Vit, { topics: ["hoso", "collected", "scan", "catalog"] });

  useEffect(() => Vj2(new Set()), [path]);

  const Vj6 = Vin[path[path.length - 1]];
  const Vj7 = Vj6.kids ?? [];
  const q = query.trim().toLowerCase();
  const Vj8 = q ? Vj7.filter((Vj9) => Vin[Vj9].name.toLowerCase().includes(q)) : Vj7;

  const Vja = Vj6?.type === "folder" ? Vj6.path : undefined;
  const [Vjb, Vjc] = useState<V418 | null>(null);
  const Vjd = useCallback(() => {
    Vjc(null);
    if (Vja)
      V419(Vja)
        .then(Vjc)
        .catch(() => {});
  }, [Vja]);
  V41p(Vjd, { topics: ["hoso"] });

  const Vje = async () => {
    if (Vjb) {
      Vis(V47u(Vjb));
      goto("scan");
      return;
    }
    await message(
      "VGjGsCBt4bulYyBuw6B5IGNoxrBhIGfhuq9uIGjhu5Mgc8ahIG7Dqm4ga2jDtG5nIGzGsHUgUERGIHbDoG8gxJHDonkgxJHGsOG7o2MuIEjDo3kgbeG7nyDEkcO6bmcgdGjGsCBt4bulYyBI4buSIFPGoCA=" +
        "KG3hu6ljIHRo4bqlcCBuaOG6pXQ6IEPGoSBxdWFuIOKGkiBI4buZcCDihpIgSOG7kyBzxqEpIHLhu5NpIGLhuqVtIFF1w6l0Lg==",
      { title: "Q2jGsGEgY2jhu41uIHRoxrAgbeG7pWMgaOG7kyBzxqE=", kind: "warning" },
    );
  };

  const Vjf = useMemo(() => {
    const Vjg = [...Vj8];
    Vjg.sort((a, b) => {
      const Vjh = Vin[a];
      const Vji = Vin[b];
      const Vjj = Vjh.type === "folder" ? 0 : 1;
      const Vjk = Vji.type === "folder" ? 0 : 1;
      if (Vjj !== Vjk) return Vjj - Vjk;
      const Vjl =
        Vix === "size" ? (Vjh.size ?? 0) - (Vji.size ?? 0) : Vjh.name.localeCompare(Vji.name, "vi");
      return Viz === "asc" ? Vjl : -Vjl;
    });
    return Vjg;
  }, [Vj8, Vin, Vix, Viz]);

  const Vjm = (Vjn: string) =>
    Vj2((m) => {
      const n = new Set(m);
      n.has(Vjn) ? n.delete(Vjn) : n.add(Vjn);
      return n;
    });

  const Vjo = async () => {
    try {
      const Vjp = await confirm(
        `WMOzYSA=${Vj1.size}IG3hu6VjIMSRw6MgY2jhu41uPyBLaMO0bmcgdGjhu4MgaG/DoG4gdMOhYy4=`,
        {
          title: "WMOhYyBuaOG6rW4geMOzYQ==",
          kind: "warning",
        },
      );
      if (!Vjp) return;
      for (const p of Vj1) {
        try {
          await V412(p);
        } catch {}
      }
      Vj2(new Set());
      Vit();
    } catch {}
  };

  const Vjq = async (Vjr: string, Vjs: string[]) => {
    for (const p of Vjs) {
      if (p === Vjr) continue;
      try {
        await V415(p, Vjr);
      } catch {}
    }
    Vj2(new Set());
    Vit();
  };

  return (
    <section className="view active">
      <div className="toolbar">
        <div className="search">
          <Icon name="tim" className="flex-none text-fg-subtle" />
          <input
            placeholder="VMOsbSB0aMawIG3hu6VjLCB0w6BpIGxp4buHdeKApg=="
            value={query}
            onChange={(Vjt) => Viu(Vjt.target.value)}
          />
        </div>
        <div className="spacer flex-1" />
        <select
          className="inp text-sm"
          value={`${Vix}-${Viz}`}
          onChange={(Vju) => {
            const [b, d] = Vju.target.value.split("-");
            Viy(b as "name" | "size");
            Vj0(d as "asc" | "desc");
          }}
          title="U+G6r3AgeOG6v3A="
        >
          <option value="name-asc">Tên A→Z</option>
          <option value="name-desc">Tên Z→A</option>
          <option value="size-asc">Nhỏ→Lớn</option>
          <option value="size-desc">Lớn→Nhỏ</option>
        </select>
        <div className="view-toggle">
          <button className={Vio === "grid" ? "on" : ""} onClick={() => Vip("grid")}>
            <Icon name="luoi" size={15} label="WGVtIGThuqFuZyB0aOG6uw==" />
          </button>
          <button className={Vio === "list" ? "on" : ""} onClick={() => Vip("list")}>
            <Icon name="danhSach" size={15} label="WGVtIGThuqFuZyBkYW5oIHPDoWNo" />
          </button>
        </div>
        <div className="menu" ref={Vj3}>
          <button
            className="btn primary"
            onClick={(Vjv) => {
              Vjv.stopPropagation();
              Viw((o) => !o);
            }}
          >
            <Icon name="them" />
            Tạo
            <Icon name="xuong" size={14} />
          </button>
          {Viv && (
            <div className="menu-pop">
              <div
                className="mi"
                onClick={() => {
                  Vir("create", "VOG6oW8gdGjGsCBt4bulYyBjb24=");
                  Viw(false);
                }}
              >
                <span className="ic">
                  <Icon name="moThuMuc" />
                </span>{" "}
                Thư mục con
              </div>
              <div className="divider" />
              <div
                className="mi"
                onClick={() => {
                  Viw(false);
                  void Vje();
                }}
              >
                <span className="ic">
                  <Icon name="quet" />
                </span>{" "}
                Quét vào thư mục này
              </div>
            </div>
          )}
        </div>
      </div>

      <div className="breadcrumb">
        {path.map((Vjw, i) => {
          const last = i === path.length - 1;
          return last ? (
            <span key={Vjw} className="cur">
              {Vin[Vjw].name}
            </span>
          ) : (
            <span key={Vjw}>
              <a onClick={() => Viq(i)}>{Vin[Vjw].name}</a>
              <span className="sp"> › </span>
            </span>
          );
        })}
      </div>

      {}
      <div className="drive-grid with-details">
        <div>
          {Vj1.size > 0 ? (
            <div className="sel-bar">
              <b>{Vj1.size} đã chọn</b>
              <div className="spacer flex-1" />
              <span className="muted text-sm">Kéo-thả vào thư mục để di chuyển</span>
              <button className="btn sm danger" onClick={() => void Vjo()}>
                <Icon name="xoa" />
                Xóa
              </button>
              <button className="btn sm" onClick={() => Vj2(new Set())}>
                Bỏ chọn
              </button>
            </div>
          ) : (
            <div className="flex items-center gap-2.5 mb-2.5">
              {path.length > 1 && (
                <button className={Vjb ? "btn sm primary" : "btn sm"} onClick={() => void Vje()}>
                  <Icon name="quet" />
                  {Vjb
                    ? "UXXDqXQgdGnhur9wIHbDoG8gaOG7kyBzxqEgbsOgeQ=="
                    : "UXXDqXQgdsOgbyB0aMawIG3hu6VjIG7DoHk="}
                </button>
              )}
              <div className="spacer flex-1" />
              <span className="muted text-sm">{Vj8.length} mục</span>
            </div>
          )}

          {Vj7.length === 0 ? (
            <div className="muted p-10 text-center rounded-lg border-2 border-dashed border-border">
              Thư mục trống.{" "}
              <a className="cursor-pointer text-fg-link" onClick={() => goto("scan")}>
                Quét tài liệu vào đây
              </a>{" "}
              hoặc tạo thư mục con.
            </div>
          ) : Vio === "list" ? (
            <V48b kids={Vjf} marked={Vj1} onToggleMark={Vjm} onMove={Vjq} />
          ) : (
            <V485 kids={Vjf} marked={Vj1} onToggleMark={Vjm} onMove={Vjq} />
          )}
        </div>

        <V48d />
      </div>
    </section>
  );
}
