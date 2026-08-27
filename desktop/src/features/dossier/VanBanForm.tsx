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
      title={Vwa === "add" ? "Thêm văn bản" : "Sửa văn bản"}
      onClose={Vwd}
      onSave={Vwe}
    >
      <V49j label="Số thứ tự (bỏ trống = tự đánh số)">
        <input
          className="inp"
          type="number"
          value={Vwb.thuTu ? Vwb.thuTu : ""}
          onChange={(Vwh) => Vwc({ thuTu: Vwg(Vwh.target.value) ?? 0 })}
        />
      </V49j>
      <V49j label="Loại văn bản">
        <input
          className="inp"
          value={Vwb.loaiVanBan ?? ""}
          onChange={(Vwi) => Vwc({ loaiVanBan: Vwi.target.value })}
        />
      </V49j>
      <V49j label="Số/ký hiệu">
        <input
          className="inp"
          value={Vwb.soKyHieu ?? ""}
          onChange={(Vwj) => Vwc({ soKyHieu: Vwj.target.value })}
        />
      </V49j>
      <V49j label="Ngày tháng">
        <input
          className="inp"
          type="date"
          value={Vwb.ngayThang ?? ""}
          onChange={(Vwk) => Vwc({ ngayThang: Vwk.target.value })}
        />
      </V49j>
      <V49j label="Cơ quan ban hành">
        <input
          className="inp"
          value={Vwb.coQuanBanHanh ?? ""}
          onChange={(Vwl) => Vwc({ coQuanBanHanh: Vwl.target.value })}
        />
      </V49j>
      <V49j label="Trích yếu nội dung">
        <input
          className="inp"
          value={Vwb.trichYeu ?? ""}
          onChange={(Vwm) => Vwc({ trichYeu: Vwm.target.value })}
        />
      </V49j>
      <V49j label="Tờ số / Trang bắt đầu / Số trang">
        <div className="flex gap-2">
          <input
            className="inp"
            type="number"
            placeholder="Tờ số"
            value={Vwb.toSo ?? ""}
            onChange={(Vwn) => Vwc({ toSo: Vwg(Vwn.target.value) })}
          />
          <input
            className="inp"
            type="number"
            placeholder="Trang BĐ"
            value={Vwb.trangSo ?? ""}
            onChange={(Vwo) => Vwc({ trangSo: Vwg(Vwo.target.value) })}
          />
          <input
            className="inp"
            type="number"
            placeholder="Số trang"
            value={Vwb.soTrang ?? ""}
            onChange={(Vwp) => Vwc({ soTrang: Vwg(Vwp.target.value) })}
          />
        </div>
      </V49j>
      <V49j label="Ngôn ngữ">
        <input
          className="inp"
          value={Vwb.ngonNgu ?? ""}
          onChange={(Vwq) => Vwc({ ngonNgu: Vwq.target.value })}
        />
      </V49j>
      <V49j label="Chế độ sử dụng">
        <input
          className="inp"
          value={Vwb.cheDoSuDung ?? ""}
          onChange={(Vwr) => Vwc({ cheDoSuDung: Vwr.target.value })}
        />
      </V49j>
      <V49j label="Độ tin cậy">
        <input
          className="inp"
          value={Vwb.doTinCay ?? ""}
          onChange={(Vws) => Vwc({ doTinCay: Vws.target.value })}
        />
      </V49j>
      <V49j label="Bút tích">
        <input
          className="inp"
          value={Vwb.butTich ?? ""}
          onChange={(Vwt) => Vwc({ butTich: Vwt.target.value })}
        />
      </V49j>
      <V49j label="Tình trạng vật lý">
        <input
          className="inp"
          value={Vwb.tinhTrangVatLy ?? ""}
          onChange={(Vwu) => Vwc({ tinhTrangVatLy: Vwu.target.value })}
        />
      </V49j>
      <V49j label="Tài liệu số">
        <input
          className="inp"
          value={Vwb.taiLieuSo ?? ""}
          onChange={(Vwv) => Vwc({ taiLieuSo: Vwv.target.value })}
        />
      </V49j>
      <V49j label="Ghi chú">
        <input
          className="inp"
          value={Vwb.ghiChu ?? ""}
          onChange={(Vww) => Vwc({ ghiChu: Vww.target.value })}
        />
      </V49j>
    </Modal>
  );
}
