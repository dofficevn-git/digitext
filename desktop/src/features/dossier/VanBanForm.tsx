// Copyright (c) 2026 DigiText. All rights reserved.

import Modal, { V49j } from "@/ui/Modal";
import type { V4bp } from "@/api/dossier";

export default function V4ck({
  mode: Vwa,
  draft: Vwb,
  onChange: Vwc,
  onClose: Vwd,
  onSave: Vwe,
}: {
  mode: "add" | "edit";
  draft: V4bp;
  onChange: (Vwf: Partial<V4bp>) => void;
  onClose: () => void;
  onSave: () => void;
}) {
  const Vwg = (v: string): number | undefined => {
    const t = v.trim();
    if (t === "") return undefined;
    const n = Number.parseInt(t, 10);
    return Number.isNaN(n) ? undefined : n;
  };

  return (
    <Modal
      title={Vwa === "add" ? "VGjDqm0gdsSDbiBi4bqjbg==" : "U+G7rWEgdsSDbiBi4bqjbg=="}
      onClose={Vwd}
      onSave={Vwe}
    >
      <V49j label="U+G7kSB0aOG7qSB04buxIChi4buPIHRy4buRbmcgPSB04buxIMSRw6FuaCBz4buRKQ==">
        <input
          className="inp"
          type="number"
          value={Vwb.thuTu ? Vwb.thuTu : ""}
          onChange={(Vwh) => Vwc({ thuTu: Vwg(Vwh.target.value) ?? 0 })}
        />
      </V49j>
      <V49j label="TG/huqFpIHbEg24gYuG6o24=">
        <input
          className="inp"
          value={Vwb.loaiVanBan ?? ""}
          onChange={(Vwi) => Vwc({ loaiVanBan: Vwi.target.value })}
        />
      </V49j>
      <V49j label="U+G7kS9rw70gaGnhu4d1">
        <input
          className="inp"
          value={Vwb.soKyHieu ?? ""}
          onChange={(Vwj) => Vwc({ soKyHieu: Vwj.target.value })}
        />
      </V49j>
      <V49j label="TmfDoHkgdGjDoW5n">
        <input
          className="inp"
          type="date"
          value={Vwb.ngayThang ?? ""}
          onChange={(Vwk) => Vwc({ ngayThang: Vwk.target.value })}
        />
      </V49j>
      <V49j label="Q8ahIHF1YW4gYmFuIGjDoG5o">
        <input
          className="inp"
          value={Vwb.coQuanBanHanh ?? ""}
          onChange={(Vwl) => Vwc({ coQuanBanHanh: Vwl.target.value })}
        />
      </V49j>
      <V49j label="VHLDrWNoIHnhur91IG7hu5lpIGR1bmc=">
        <input
          className="inp"
          value={Vwb.trichYeu ?? ""}
          onChange={(Vwm) => Vwc({ trichYeu: Vwm.target.value })}
        />
      </V49j>
      <V49j label="VOG7nSBz4buRIC8gVHJhbmcgYuG6r3QgxJHhuqd1IC8gU+G7kSB0cmFuZw==">
        <div className="flex gap-2">
          <input
            className="inp"
            type="number"
            placeholder="VOG7nSBz4buR"
            value={Vwb.toSo ?? ""}
            onChange={(Vwn) => Vwc({ toSo: Vwg(Vwn.target.value) })}
          />
          <input
            className="inp"
            type="number"
            placeholder="VHJhbmcgQsSQ"
            value={Vwb.trangSo ?? ""}
            onChange={(Vwo) => Vwc({ trangSo: Vwg(Vwo.target.value) })}
          />
          <input
            className="inp"
            type="number"
            placeholder="U+G7kSB0cmFuZw=="
            value={Vwb.soTrang ?? ""}
            onChange={(Vwp) => Vwc({ soTrang: Vwg(Vwp.target.value) })}
          />
        </div>
      </V49j>
      <V49j label="TmfDtG4gbmfhu68=">
        <input
          className="inp"
          value={Vwb.ngonNgu ?? ""}
          onChange={(Vwq) => Vwc({ ngonNgu: Vwq.target.value })}
        />
      </V49j>
      <V49j label="Q2jhur8gxJHhu5kgc+G7rSBk4bulbmc=">
        <input
          className="inp"
          value={Vwb.cheDoSuDung ?? ""}
          onChange={(Vwr) => Vwc({ cheDoSuDung: Vwr.target.value })}
        />
      </V49j>
      <V49j label="xJDhu5kgdGluIGPhuq15">
        <input
          className="inp"
          value={Vwb.doTinCay ?? ""}
          onChange={(Vws) => Vwc({ doTinCay: Vws.target.value })}
        />
      </V49j>
      <V49j label="QsO6dCB0w61jaA==">
        <input
          className="inp"
          value={Vwb.butTich ?? ""}
          onChange={(Vwt) => Vwc({ butTich: Vwt.target.value })}
        />
      </V49j>
      <V49j label="VMOsbmggdHLhuqFuZyB24bqtdCBsw70=">
        <input
          className="inp"
          value={Vwb.tinhTrangVatLy ?? ""}
          onChange={(Vwu) => Vwc({ tinhTrangVatLy: Vwu.target.value })}
        />
      </V49j>
      <V49j label="VMOgaSBsaeG7h3Ugc+G7kQ==">
        <input
          className="inp"
          value={Vwb.taiLieuSo ?? ""}
          onChange={(Vwv) => Vwc({ taiLieuSo: Vwv.target.value })}
        />
      </V49j>
      <V49j label="R2hpIGNow7o=">
        <input
          className="inp"
          value={Vwb.ghiChu ?? ""}
          onChange={(Vww) => Vwc({ ghiChu: Vww.target.value })}
        />
      </V49j>
    </Modal>
  );
}
