// Copyright (c) 2026 DigiText. All rights reserved.
import { useCallback, useState } from "react";
import { V41p } from "@/live/useLive";
import { open } from "@tauri-apps/plugin-dialog";
import V4oq from "@/ui/Seg";
import { V460 } from "@/state";
import { V40z, V410 } from "@/api/archive";
import { V3zm, V3zn, V3zo } from "@/api/auth";
import { V4r1, V4r2, type V4qz } from "@/api/stats";
import { isTauri, V3xq } from "@/api/ipc";
import { V3z9 } from "@/domain/permissions";
import { V3zt, V3zr } from "@/state/sessionCore";
import type { V42t } from "@/api/updater";
import { Icon } from "@/components/Icon";

function V4rq(u: V42t, V2ml: string): string {
  switch (u.phase) {
    case "checking":
      return "Đang kiểm tra…";
    case "uptodate":
      return `Đã là bản mới nhất (v${u.version ?? V2ml})`;
    case "available":
      return `Có bản mới v${u.version}`;
    case "downloading":
      return `Đang tải v${u.version}…`;
    case "installing":
      return "Đang cài đặt…";
    case "relaunching":
      return "Đang khởi động lại…";
    case "error":
      return `Lỗi: ${u.error}`;
    default:
      return "";
  }
}

const V4rr = [
  "Máy quét & mặc định",
  "Dự án & tách tài liệu",
  "Băng thông",
  "Máy chủ & tải lên",
  "USB Token / Chứng thư",
  "Lưu trữ & ổ ngoài",
  "Ngôn ngữ",
  "Chẩn đoán & nhật ký",
];

export default function V4rs() {
  const {
    appVersion: V2mm,
    update: V2mn,
    checkForUpdates: V2mo,
    session: V2mp,
    applySession: V2mq,
  } = V460();
  const [V2mr, V2ms] = useState(V4rr[0]);
  const [V2mt, V2mu] = useState<"300" | "600">("600");
  const [V2mv, V2mw] = useState("");
  const [V2mx, V2my] = useState("");
  const [V2mz, V2n0] = useState("");
  const [V2n1, V2n2] = useState("");
  const [V2n3, V2n4] = useState(false);
  const [V2n5, V2n6] = useState<V4qz | null>(null);
  const [V2n7, V2n8] = useState("");
  const [V2n9, V2na] = useState(false);

  const V2nb = V3z9(V2mp?.permissions, "stats.view");

  const V2nc = useCallback(() => {
    if (V2mr === "Lưu trữ & ổ ngoài") {
      V40z()
        .then(V2mw)
        .catch(() => {});
    }
    if (V2mr === "Máy chủ & tải lên") {
      V3zm()
        .then((u) => V2n0(u ?? ""))
        .catch(() => {});
      if (V2nb)
        V4r1()
          .then(V2n6)
          .catch(() => {});
    }
  }, [V2mr, V2nb]);

  V41p(V2nc, { topics: ["settings", "stats", "scan"], enabled: isTauri() });

  const V2nd = async () => {
    V2na(true);
    V2n8("");
    try {
      const s = await V4r2();
      V2n6(s);
      V2n8(
        "Đã gửi số liệu về máy chủ — quản trị sẽ thấy trong thống kê.",
      );
    } catch (V2ne) {
      const V2nf = V2ne instanceof V3xq ? V2ne.code : "error";
      V2n8(V3zt(V2nf, V2ne instanceof Error ? V2ne.message : String(V2ne)));
    } finally {
      V2na(false);
    }
  };

  const V2ng = async () => {
    try {
      await V3zn(V2mz.trim() || null);
      V2n2(
        "Đã lưu (đổi máy chủ sẽ pin lại khoá khi đăng nhập online)",
      );
    } catch (V2nh) {
      V2n2("Lỗi: " + (V2nh instanceof Error ? V2nh.message : String(V2nh)));
    }
  };

  const V2ni = async () => {
    V2n4(true);
    V2n2("");
    try {
      const s = await V3zo();
      V2mq(s);
      V2n2(
        `Đã xác thực online — hạn offline mới: còn ${V3zr(s.offlineExpiry)} ngày`,
      );
    } catch (V2nj) {
      const V2nk = V2nj instanceof V3xq ? V2nj.code : "error";
      V2n2(V3zt(V2nk, V2nj instanceof Error ? V2nj.message : String(V2nj)));
    } finally {
      V2n4(false);
    }
  };

  const V2nl = async () => {
    try {
      const V2nm = await open({
        directory: true,
        multiple: false,
        title: "Chọn thư mục lưu trữ",
      });
      if (typeof V2nm === "string") V2mw(V2nm);
    } catch {}
  };
  const V2nn = async () => {
    try {
      await V410(V2mv.trim() || null);
      V2mw(await V40z());
      V2my("Đã lưu");
    } catch (V2no) {
      V2my("Lỗi: " + (V2no instanceof Error ? V2no.message : String(V2no)));
    }
  };
  const V2np = async () => {
    try {
      await V410(null);
      V2mw(await V40z());
      V2my("Đã về mặc định");
    } catch {}
  };

  const V2nq =
    V2mn.phase === "checking" ||
    V2mn.phase === "downloading" ||
    V2mn.phase === "installing" ||
    V2mn.phase === "relaunching";

  return (
    <section className="view active">
      <div className="page-head">
        <h2>Cài đặt</h2>
      </div>
      <div className="settings-grid">
        <div className="set-nav">
          {V4rr.map((s) => (
            <div key={s} className={`si ${V2mr === s ? "on" : ""}`} onClick={() => V2ms(s)}>
              {s}
            </div>
          ))}
        </div>
        <div className="set-body">
          <h3 className="mb-3.5">{V2mr}</h3>

          {V2mr === "Máy quét & mặc định" ? (
            <>
              <div className="set-field">
                <label>Máy quét mặc định</label>
                <select className="inp max-w-70" defaultValue="">
                  <option value="">Chưa phát hiện máy quét</option>
                </select>
              </div>
              <div className="set-field">
                <label>Độ phân giải mặc định</label>
                <V4oq
                  value={V2mt}
                  onChange={V2mu}
                  options={[
                    { value: "300", label: "300" },
                    { value: "600", label: "600 DPI" },
                  ]}
                />
              </div>
              <div className="set-field">
                <label>Cơ chế truyền</label>
                <span className="muted">MEMFILEXFER (chunked .tif) — tiết kiệm RAM</span>
              </div>
              <div className="set-field">
                <label>Dung sai khổ giấy</label>
                <input className="inp max-w-30" defaultValue="±3 mm" />
              </div>
              <div className="set-field">
                <label>Hiện UI driver khi quét</label>
                <span className="muted">Tắt (ShowUI=false) — ADF tự động</span>
              </div>
            </>
          ) : V2mr === "Chẩn đoán & nhật ký" ? (
            <>
              <div className="set-field">
                <label>Phiên bản</label>
                <span className="muted">DigiText v{V2mm || "…"}</span>
              </div>
              <div className="set-field">
                <label>Cập nhật phần mềm</label>
                <div className="flex items-center gap-3">
                  <button className="btn" onClick={V2mo} disabled={V2nq}>
                    <Icon name="lamMoi" />
                    Kiểm tra cập nhật
                  </button>
                  <span className="muted">{V4rq(V2mn, V2mm)}</span>
                </div>
              </div>
              <div className="set-field">
                <label>Kênh cập nhật</label>
                <span className="muted">GitHub Releases · tự cập nhật khi mở app</span>
              </div>
            </>
          ) : V2mr === "Máy chủ & tải lên" ? (
            <>
              <div className="set-field">
                <label>Địa chỉ máy chủ</label>
                <div className="flex gap-2 items-center flex-wrap">
                  <input
                    className="inp flex-1 min-w-60"
                    value={V2mz}
                    onChange={(V2nr) => V2n0(V2nr.target.value)}
                    placeholder="http://mac-mini:8080"
                  />
                  <button className="btn" onClick={() => void V2ng()}>
                    Lưu
                  </button>
                </div>
              </div>
              <div className="set-field">
                <label>Kết nối</label>
                <div className="flex gap-3 items-center flex-wrap">
                  <button className="btn primary" onClick={() => void V2ni()} disabled={V2n3}>
                    {V2n3 ? (
                      "Đang kiểm tra…"
                    ) : (
                      <>
                        <Icon name="lamMoi" />
                        Kiểm tra kết nối
                      </>
                    )}
                  </button>
                  {V2n1 && <span className="muted">{V2n1}</span>}
                </div>
              </div>
              <div className="set-field">
                <label>Phiên hiện tại</label>
                <span className="muted">
                  {V2mp
                    ? `${V2mp.username} · ${V2mp.mode === "online" ? "đã xác thực online" : "đang offline"} · hạn offline còn ${V3zr(V2mp.offlineExpiry)} ngày`
                    : "Chưa đăng nhập"}
                </span>
              </div>
              {V2nb && (
                <>
                  <div className="set-field">
                    <label>Số liệu của tôi</label>
                    <span className="muted">
                      {V2n5
                        ? `${V2n5.pagesScanned} trang quét · ${V2n5.hoSoCount} hồ sơ · ${V2n5.pdfCount} PDF · ${(V2n5.pdfBytes / 1_048_576).toFixed(1)} MB`
                        : "—"}
                    </span>
                  </div>
                  <div className="set-field">
                    <label>Đồng bộ số liệu</label>
                    <div className="flex gap-3 items-center flex-wrap">
                      <button className="btn" onClick={() => void V2nd()} disabled={V2n9}>
                        {V2n9 ? (
                          "Đang gửi…"
                        ) : (
                          <>
                            <Icon name="thongke" />
                            Gửi số liệu về quản trị
                          </>
                        )}
                      </button>
                      {V2n7 && <span className="muted">{V2n7}</span>}
                    </div>
                  </div>
                </>
              )}
              <div className="set-field">
                <label>Ghi chú</label>
                <span className="muted">
                  Mỗi lần xác thực online thành công sẽ reset hạn dùng offline 30 ngày. Gửi số liệu
                  để quản trị thấy số trang quét của bạn; tải lên FILE PDF sẽ bổ sung ở giai đoạn
                  sau.
                </span>
              </div>
            </>
          ) : V2mr === "Lưu trữ & ổ ngoài" ? (
            <>
              <div className="set-field">
                <label>Ổ lưu trữ mặc định</label>
                <div className="flex gap-2 items-center flex-wrap">
                  <input
                    className="inp flex-1 min-w-60"
                    value={V2mv}
                    onChange={(V2ns) => V2mw(V2ns.target.value)}
                    placeholder="~/Documents/DigiText"
                  />
                  <button className="btn" onClick={() => void V2nl()}>
                    <Icon name="thuMuc" />
                    Chọn thư mục…
                  </button>
                </div>
              </div>
              <div className="set-field">
                <label />
                <div className="flex gap-2 items-center">
                  <button className="btn primary" onClick={() => void V2nn()}>
                    Lưu
                  </button>
                  <button className="btn" onClick={() => void V2np()}>
                    Dùng mặc định
                  </button>
                  {V2mx && <span className="muted">{V2mx}</span>}
                </div>
              </div>
              <div className="set-field">
                <label>Ghi chú</label>
                <span className="muted">
                  PDF + ảnh JPEG lưu vào ổ này theo Cơ quan → Hộp → Hồ sơ. Dùng ổ ngoài (TB4 SSD)
                  khi dữ liệu lớn. Đổi ổ chỉ áp dụng cho tài liệu quét/đóng gói sau đó.
                </span>
              </div>
            </>
          ) : (
            <p className="muted py-3 px-0">Cấu hình cho “{V2mr}” sẽ hiển thị ở đây.</p>
          )}
        </div>
      </div>
    </section>
  );
}
