// Copyright (c) 2026 DigiText. All rights reserved.

import { useCallback, useState } from "react";
import { V41p } from "@/live/useLive";
import "./laphoso.css";
import { V3xq } from "@/api/ipc";
import { V40x } from "@/api/archive";
import { V4cb, type V4bm, type V4ca } from "@/api/dossier";
import V4h7 from "./OcrReviewPanel";
import { Icon } from "@/components/Icon";

export default function V4h8({ dossier, onBack: V1dw }: { dossier: V4bm; onBack: () => void }) {
  const [V1dx, V1dy] = useState<V4ca[]>([]);
  const [V1dz, V1e0] = useState(true);
  const [V1e1, V1e2] = useState<string | null>(null);
  const [V1e3, V1e4] = useState<V4ca | null>(null);

  const reload = useCallback(() => {
    V1e0(true);
    V4cb(dossier.id)
      .then((rs) => {
        V1dy(rs);
        V1e2(null);
      })
      .catch((V1e5) =>
        V1e2(V1e5 instanceof V3xq || V1e5 instanceof Error ? V1e5.message : String(V1e5)),
      )
      .finally(() => V1e0(false));
  }, [dossier.id]);

  V41p(reload, { topics: ["hoso", "dossier", "scan"] });

  const open = (r: V4ca) => {
    V40x(r.filePath).catch((V1e6) =>
      V1e2(V1e6 instanceof V3xq || V1e6 instanceof Error ? V1e6.message : String(V1e6)),
    );
  };

  if (V1e3) {
    return (
      <V4h7
        dossier={dossier}
        file={V1e3}
        onBack={() => {
          V1e4(null);
          reload();
        }}
      />
    );
  }

  return (
    <div className="lhs">
      <div className="content-panel">
        <div className="mb-2">
          <button className="btn btn-outline-primary btn-round" onClick={V1dw}>
            <Icon name="hoanTac" /> Về hồ sơ
          </button>
        </div>
        <div className="center">
          <h4>
            <b>DANH SÁCH TÀI LIỆU THUỘC HỒ SƠ - {dossier.soTam}</b>
            {dossier.tieuDe && (
              <>
                <br />
                <span className="font-normal italic">({dossier.tieuDe})</span>
              </>
            )}
          </h4>
        </div>
        <hr />

        {V1e1 && <p className="muted text-danger-700">{V1e1}</p>}

        <div className="tbl-scroll">
          <table className="dataTable">
            <thead>
              <tr>
                <th>Mã VB</th>
                <th className="w-10">STT</th>
                <th className="w-15">Số VB</th>
                <th className="w-22.5">Loại VB</th>
                <th className="w-27.5">Ký hiệu VB</th>
                <th className="w-22.5">Ngày tháng</th>
                <th>Tác giả</th>
                <th>Trích yếu</th>
                <th className="w-17.5">Số trang</th>
                <th className="w-17.5">TT Vật lý</th>
                <th className="w-20">Chức năng</th>
              </tr>
            </thead>
            <tbody>
              {V1dz ? (
                <tr>
                  <td className="dt-empty" colSpan={11}>
                    Đang tải dữ liệu…
                  </td>
                </tr>
              ) : V1dx.length === 0 ? (
                <tr>
                  <td className="dt-empty" colSpan={11}>
                    Chưa số hoá — hồ sơ chưa có file PDF nào.
                  </td>
                </tr>
              ) : (
                V1dx.map((r, i) => (
                  <tr key={r.filePath} className={i % 2 === 0 ? "odd" : "even"}>
                    <td className="text-center">
                      <a
                        className="ic-word"
                        onClick={() => open(r)}
                        title="TeG7ny9YZW0gZmlsZSBQREY="
                      >
                        {r.fileName}
                      </a>
                    </td>
                    <td className="text-center">{i + 1}</td>
                    <td className="text-center" />
                    <td className="text-center">{r.loaiVanBan}</td>
                    <td className="text-center">{r.soKyHieu}</td>
                    <td className="text-center">{r.ngayThang}</td>
                    <td className="text-justify">{r.tacGia}</td>
                    <td className="text-justify">{r.trichYeu}</td>
                    <td className="text-center">{r.pageCount}</td>
                    <td className="text-center">{r.tinhTrangVatLy}</td>
                    <td className="text-center">
                      <a
                        className="ic-word"
                        onClick={() => V1e4(r)}
                        title="WGVtIHbEg24gYuG6o24gJiBPQ1IgKDIgcGjhuqduKQ=="
                      >
                        <span className="ic-ocr">OCR</span>
                      </a>
                      &nbsp;&nbsp;
                      <a
                        className="ic-word"
                        onClick={() => open(r)}
                        title="TeG7ny9YZW0gZmlsZSBQREY="
                      >
                        <Icon name="tepPdf" />
                      </a>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
