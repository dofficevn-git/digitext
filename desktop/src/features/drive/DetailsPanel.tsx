// Copyright (c) 2026 DigiText. All rights reserved.
import { useCallback, useState } from "react";
import { V41p } from "@/live/useLive";
import { confirm, message } from "@tauri-apps/plugin-dialog";
import { V460 } from "@/state";
import { V47u } from "@/features/drive/scanTarget";
import { V41y } from "@/mocks/fixtures";
import { V414, V412, V419, V40n, V40y, V40o, V413, type V418 } from "@/api/archive";
import type { V3y1 } from "@/domain/types";
import V47w from "@/ui/Avatars";
import { Icon } from "@/components/Icon";

function V48c({ node }: { node: V3y1 }) {
  const { removeArchiveNode: Vhe, renameNode: Vhf, addFolderNode: Vhg } = V460();
  const [Vhh, Vhi] = useState<"none" | "rename" | "newfolder">("none");
  const [Vhj, Vhk] = useState("");
  const [Vhl, Vhm] = useState("");

  if (!node.path) return null;
  const path = node.path;
  const Vhn = node.type === "folder";

  const Vho = async () => {
    const name = Vhj.trim();
    if (!name) return;
    Vhm("");
    try {
      if (Vhh === "rename") {
        const Vhp = await V413(path, name);
        Vhf(node.id, name, path, Vhp.path);
      } else {
        const Vhq = await V414(path, name);
        Vhg(node.id, name, Vhq.path);
      }
      Vhi("none");
      Vhk("");
    } catch (Vhr) {
      Vhm(Vhr instanceof Error ? Vhr.message : String(Vhr));
    }
  };

  const Vhs = async () => {
    Vhm("");
    try {
      const Vht = await confirm(
        `WMOzYSAi${node.name}"${Vhn ? "IHbDoCB0b8OgbiBi4buZIG7hu5lpIGR1bmcgYsOqbiB0cm9uZw==" : ""}PyBLaMO0bmcgdGjhu4MgaG/DoG4gdMOhYy4=`,
        { title: "WMOhYyBuaOG6rW4geMOzYQ==", kind: "warning" },
      );
      if (!Vht) return;
      await V412(path);
      Vhe(node.id);
    } catch (Vhu) {
      Vhm(Vhu instanceof Error ? Vhu.message : String(Vhu));
    }
  };

  return (
    <div className="dmanage">
      {Vhh !== "none" ? (
        <div className="dinline">
          <input
            className="inp"
            autoFocus
            placeholder={
              Vhh === "rename" ? "VMOqbiBt4bubaeKApg==" : "VMOqbiB0aMawIG3hu6VjIGNvbuKApg=="
            }
            value={Vhj}
            onChange={(Vhv) => Vhk(Vhv.target.value)}
            onKeyDown={(Vhw) => {
              if (Vhw.key === "Enter") void Vho();
              if (Vhw.key === "Escape") {
                Vhi("none");
                Vhm("");
              }
            }}
          />
          <button className="btn sm primary" onClick={() => void Vho()}>
            OK
          </button>
          <button
            className="btn sm"
            onClick={() => {
              Vhi("none");
              Vhm("");
            }}
          >
            Hủy
          </button>
        </div>
      ) : (
        <div className="dmanage-btns">
          <button
            className="btn sm"
            onClick={() => {
              Vhk(node.name);
              Vhi("rename");
            }}
          >
            <Icon name="sua" />
            Đổi tên
          </button>
          {Vhn && (
            <button
              className="btn sm"
              onClick={() => {
                Vhk("");
                Vhi("newfolder");
              }}
            >
              <Icon name="thuMuc" />
              Thư mục con
            </button>
          )}
          <button className="btn sm danger" onClick={() => void Vhs()}>
            <Icon name="xoa" />
            Xóa
          </button>
        </div>
      )}
      {Vhl && <div className="dmanage-err">{Vhl}</div>}
    </div>
  );
}

export default function V48d() {
  const {
    nodes: Vhx,
    selected: Vhy,
    enterFolder: Vhz,
    openPreview: Vi0,
    openDialog: Vi1,
    goto,
    setScanTarget: Vi2,
    openEditor: Vi3,
  } = V460();
  const [Vi4, Vi5] = useState<V418 | null>(null);
  const [Vi6, Vi7] = useState(false);
  const [Vi8, Vi9] = useState<string | null>(null);
  const n = Vhy ? Vhx[Vhy] : null;
  const Via = n && n.type === "folder" ? n.path : undefined;
  const Vib = useCallback(() => {
    Vi5(null);
    if (Via)
      V419(Via)
        .then(Vi5)
        .catch(() => {});
  }, [Via]);
  V41p(Vib, { topics: ["hoso", "dossier"] });

  const Vic = async (Vid: string, Vie?: string) => {
    Vi7(true);
    Vi9(null);
    try {
      const Vif = await V419(Vid);
      if (!Vif) {
        Vi9(
          "S2jDtG5nIHTDrG0gdGjhuqV5IGjhu5Mgc8ahIMSRw6MgxJHDs25nIGfDs2kgY2hvIHTDoGkgbGnhu4d1IG7DoHku",
        );
        return;
      }
      const Vig = await V40n(Vif.id, Vie);
      Vi3(Vig, Vie ?? Vig.hoSo.pdfPath ?? null);
    } catch (Vih) {
      Vi9(Vih instanceof Error ? Vih.message : String(Vih));
    } finally {
      Vi7(false);
    }
  };

  const Vii = async (Vij: string) => {
    Vi7(true);
    Vi9(null);
    try {
      const Vik = await V40o(Vij);
      Vi3(Vik, Vij);
    } catch (Vil) {
      Vi9(Vil instanceof Error ? Vil.message : String(Vil));
    } finally {
      Vi7(false);
    }
  };

  if (!Vhy || !n) {
    return (
      <div className="details">
        <div className="dempty">Chọn dự án, thư mục hoặc tài liệu để xem chi tiết.</div>
      </div>
    );
  }

  if (n.type === "folder") {
    return (
      <div className="details">
        <div className="dhero">
          <div className="big">
            <Icon name="thuMuc" size={32} />
          </div>
          <div className="dn">{n.name}</div>
          <div className="muted text-sm">Thư mục · chủ sở hữu: {n.owner ?? "VMO0aQ=="}</div>
        </div>
        <div className="drow">
          <span>Thành viên</span>
          <V47w members={n.members} />
        </div>
        <div className="drow">
          <span>Mục con</span>
          <b>{(n.kids ?? []).length}</b>
        </div>
        <div className="dactions">
          <button className="btn primary" onClick={() => Vhz(n.id)}>
            <Icon name="moThuMuc" />
            Mở thư mục
          </button>
          <button
            className={Vi4 ? "btn primary" : "btn"}
            onClick={() => {
              if (Vi4) {
                Vi2(V47u(Vi4));
                goto("scan");
              } else {
                void message(
                  "VGjGsCBt4bulYyBuw6B5IGNoxrBhIGfhuq9uIGjhu5Mgc8ahIG7Dqm4ga2jDtG5nIGzGsHUgUERGIHbDoG8gxJHDonkgxJHGsOG7o2MuIEjDo3kgY2jhu41uIMSRw7puZyA=" +
                    "dGjGsCBt4bulYyBI4buSIFPGoCAoQ8ahIHF1YW4g4oaSIEjhu5lwIOKGkiBI4buTIHPGoSkgcuG7k2kgYuG6pW0gUXXDqXQu",
                  { title: "Q2jGsGEgY2jhu41uIHRoxrAgbeG7pWMgaOG7kyBzxqE=", kind: "warning" },
                );
              }
            }}
          >
            <Icon name="quet" />
            {Vi4 ? "UXXDqXQgdGnhur9wIHbDoG8gaOG7kyBzxqEgbsOgeQ==" : "UXXDqXQgdsOgbyDEkcOieQ=="}
          </button>
          {Vi4 && n.path && (
            <button className="btn" disabled={Vi6} onClick={() => void Vic(n.path!)}>
              {Vi6 ? (
                "xJBhbmcgbeG7n+KApg=="
              ) : (
                <>
                  <Icon name="anh" />
                  Chỉnh sửa ảnh / trang
                </>
              )}
            </button>
          )}
          {!n.path && (
            <button className="btn" onClick={() => Vi1("members", n.name)}>
              <Icon name="thanhVien" />
              Quản lý thành viên
            </button>
          )}
        </div>
        {Vi8 && <div className="dmanage-err">{Vi8}</div>}
        <V48c node={n} />
      </div>
    );
  }

  const Vim = V41y[n.status ?? "scanned"];
  return (
    <div className="details">
      <div className="dhero">
        <div className="big">
          <Icon name="taiLieu" size={32} />
        </div>
        <div className="dn">{n.name}.pdf</div>
        <div className="muted text-sm">Tài liệu PDF</div>
      </div>
      <div className="drow">
        <span>Số trang</span>
        <b>{n.pages ?? "—"}</b>
      </div>
      <div className="drow">
        <span>Trạng thái</span>
        <span className={`badge ${Vim.tone}`}>
          <Icon name={Vim.icon} size={12} />
          {Vim.label}
        </span>
      </div>
      <div className="dactions">
        {n.path ? (
          <button className="btn primary" onClick={() => void V40y(n.path!)}>
            <Icon name="xem" />
            Mở PDF
          </button>
        ) : (
          <button className="btn primary" onClick={() => Vi0(n)}>
            <Icon name="xem" />
            Xem trước
          </button>
        )}
        {n.path ? (
          <button
            className="btn"
            disabled={Vi6}
            onClick={() => void Vii(n.path!)}
            title="TeG7nyBs4bqhaSBjw6FjIHRyYW5nIMSR4buDIHhvYXkvY+G6r3Qva2jhu60gbmdoacOqbmcsIHhvw6Evc+G6r3AgeOG6v3AvY2jDqG4gdHJhbmcgcuG7k2kgxJHDs25nIGfDs2kgbOG6oWkgKGdoaSDEkcOoIFBERiku"
          >
            {Vi6 ? (
              "xJBhbmcgbeG7n+KApg=="
            ) : (
              <>
                <Icon name="anh" />
                Chỉnh sửa ảnh / trang
              </>
            )}
          </button>
        ) : (
          <button className="btn" onClick={() => goto("editor")}>
            <Icon name="anh" />
            Xử lý ảnh
          </button>
        )}
        <button className="btn" onClick={() => goto("sign")}>
          <Icon name="kySo" />
          Gửi ký số
        </button>
      </div>
      {Vi8 && <div className="dmanage-err">{Vi8}</div>}
      <V48c node={n} />
    </div>
  );
}
