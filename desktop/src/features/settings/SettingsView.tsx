// Copyright (c) 2026 DigiText. All rights reserved.
// Cài đặt: Máy chủ (địa chỉ, kết nối, phiên) · Lưu trữ (thư mục, ổ ngoài) · Máy quét (thiết bị đang thấy) ·
// Dữ liệu (dữ liệu chỉnh lý cũ, gửi số liệu, dịch vụ OCR) · Ứng dụng (phiên bản, cập nhật).

import { useCallback, useEffect, useState } from "react";
import { V41p } from "@/live/useLive";
import { open } from "@tauri-apps/plugin-dialog";
import { V460 } from "@/state";
import { V40z, V410 } from "@/api/archive";
import { V3zm, V3zn, V3zo } from "@/api/auth";
import { V4r1, V4r2, type V4qz } from "@/api/stats";
import { V4cc, V4cd } from "@/api/dossier";
import { V475 } from "@/api/scanner";
import type { V3y6 } from "@/domain/types";
import { isTauri } from "@/api/ipc";
import { V3z9 } from "@/domain/permissions";
import { V3zr } from "@/state/sessionCore";
import type { V42t } from "@/api/updater";
import ChuyenDuLieuCu from "@/features/chinhly/ChuyenDuLieuCu";
import { Icon, type V41v as IconName } from "@/components/Icon";
import { Badge, Button, Card, describeError, Field, Input, notifyError, PageHeader, toast } from "@/ui";

type Muc = "mayChu" | "luuTru" | "mayQuet" | "duLieu" | "ungDung";
const MUC: { key: Muc; label: string; icon: IconName }[] = [
  { key: "mayChu", label: "Máy chủ", icon: "mayChu" },
  { key: "luuTru", label: "Lưu trữ", icon: "kho" },
  { key: "mayQuet", label: "Máy quét", icon: "scanner" },
  { key: "duLieu", label: "Dữ liệu", icon: "chinhly" },
  { key: "ungDung", label: "Ứng dụng", icon: "caiDat" },
];

function trangThaiCapNhat(u: V42t, ver: string): string {
  switch (u.phase) {
    case "checking":
      return "Đang kiểm tra…";
    case "uptodate":
      return `Đã là bản mới nhất (v${u.version ?? ver})`;
    case "available":
      return `Có bản mới v${u.version}`;
    case "downloading":
      return `Đang tải v${u.version}…`;
    case "installing":
      return "Đang cài đặt…";
    case "relaunching":
      return "Đang khởi động lại…";
    case "error":
      return describeError(u.error).message;
    default:
      return "";
  }
}

/** Nút Lưu có trạng thái "Đã lưu" ngắn sau khi lưu xong. */
function NutLuu({ onSave, label = "Lưu", primary }: { onSave: () => Promise<void>; label?: string; primary?: boolean }) {
  const [busy, setBusy] = useState(false);
  const [daLuu, setDaLuu] = useState(false);
  useEffect(() => {
    if (!daLuu) return;
    const t = window.setTimeout(() => setDaLuu(false), 2500);
    return () => window.clearTimeout(t);
  }, [daLuu]);
  return (
    <Button
      variant={primary ? "primary" : "secondary"}
      icon={daLuu ? "duyet" : "luu"}
      loading={busy}
      onClick={async () => {
        setBusy(true);
        try {
          await onSave();
          setDaLuu(true);
        } finally {
          setBusy(false);
        }
      }}
    >
      {daLuu ? "Đã lưu" : label}
    </Button>
  );
}

function MucMayChu() {
  const { session, applySession } = V460();
  const [url, setUrl] = useState("");
  const [busy, setBusy] = useState(false);
  useEffect(() => {
    V3zm()
      .then((u) => setUrl(u ?? ""))
      .catch(() => {});
  }, []);
  const luu = async () => {
    try {
      await V3zn(url.trim() || null);
      toast.success("Đã lưu địa chỉ máy chủ.", { description: "Đổi máy chủ thì lần đăng nhập có mạng tiếp theo sẽ ghim lại khoá." });
    } catch (e) {
      notifyError(e, "lưu địa chỉ máy chủ");
      throw e;
    }
  };
  const kiemTra = async () => {
    setBusy(true);
    try {
      const s = await V3zo();
      applySession(s);
      toast.success(`Đã xác thực với máy chủ. Hạn dùng ngoại tuyến còn ${V3zr(s.offlineExpiry)} ngày.`);
    } catch (e) {
      notifyError(e, "kiểm tra kết nối");
    } finally {
      setBusy(false);
    }
  };
  return (
    <div className="space-y-6">
      <Field label="Địa chỉ máy chủ" hint="Ví dụ http://192.168.0.127:8080">
        <div className="flex gap-2">
          <Input value={url} onChange={(e) => setUrl(e.target.value)} placeholder="http://may-chu:8080" spellCheck={false} />
          <NutLuu onSave={luu} />
        </div>
      </Field>
      <Field label="Kết nối">
        <div className="flex flex-wrap items-center gap-3">
          <Button variant="primary" icon="lamMoi" loading={busy} onClick={() => void kiemTra()}>
            Kiểm tra kết nối
          </Button>
          <span className="text-sm text-fg-subtle">Mỗi lần xác thực có mạng thành công sẽ đặt lại hạn dùng ngoại tuyến 30 ngày.</span>
        </div>
      </Field>
      <Field label="Phiên hiện tại">
        {session ? (
          <p className="text-sm">
            {session.username} · {session.mode === "online" ? <Badge tone="ok">Đã xác thực có mạng</Badge> : <Badge tone="warn">Đang ngoại tuyến</Badge>} · hạn ngoại tuyến còn {V3zr(session.offlineExpiry)} ngày
          </p>
        ) : (
          <p className="text-sm text-fg-subtle">Chưa đăng nhập</p>
        )}
      </Field>
    </div>
  );
}

function MucLuuTru() {
  const [dir, setDir] = useState("");
  const load = useCallback(() => {
    V40z()
      .then(setDir)
      .catch(() => {});
  }, []);
  V41p(load, { topics: ["settings"], enabled: isTauri() });
  const chon = async () => {
    try {
      const d = await open({ directory: true, multiple: false, title: "Chọn thư mục lưu trữ" });
      if (typeof d === "string") setDir(d);
    } catch {
      /* người dùng huỷ */
    }
  };
  const luu = async () => {
    try {
      await V410(dir.trim() || null);
      setDir(await V40z());
      toast.success("Đã lưu thư mục lưu trữ.");
    } catch (e) {
      notifyError(e, "lưu thư mục");
      throw e;
    }
  };
  const macDinh = async () => {
    try {
      await V410(null);
      setDir(await V40z());
      toast.success("Đã dùng thư mục mặc định.");
    } catch (e) {
      notifyError(e, "đặt mặc định");
    }
  };
  return (
    <div className="space-y-6">
      <Field label="Thư mục lưu trữ" hint="PDF và ảnh quét lưu vào đây theo Cơ quan › Hộp › Hồ sơ. Đổi thư mục chỉ áp dụng cho tài liệu quét sau đó.">
        <div className="flex flex-wrap gap-2">
          <Input className="min-w-60 flex-1" value={dir} onChange={(e) => setDir(e.target.value)} placeholder="~/Documents/DigiText" spellCheck={false} />
          <Button icon="thuMuc" onClick={() => void chon()}>
            Chọn
          </Button>
          <NutLuu onSave={luu} primary />
          <Button variant="ghost" onClick={() => void macDinh()}>
            Dùng mặc định
          </Button>
        </div>
      </Field>
      <p className="text-sm text-fg-subtle">Dữ liệu lớn nên dùng ổ SSD gắn ngoài. Ổ không thấy được sẽ báo ở thanh trạng thái.</p>
    </div>
  );
}

function MucMayQuet() {
  const [list, setList] = useState<V3y6[] | null>(null);
  const [busy, setBusy] = useState(false);
  const do_ = useCallback(async () => {
    setBusy(true);
    try {
      setList(await V475());
    } catch (e) {
      setList([]);
      notifyError(e, "dò máy quét");
    } finally {
      setBusy(false);
    }
  }, []);
  useEffect(() => {
    if (isTauri()) void do_();
  }, [do_]);
  return (
    <div className="space-y-4">
      <Field label="Máy quét đang thấy">
        {list === null ? (
          <p className="text-sm text-fg-subtle">Đang dò…</p>
        ) : list.length === 0 ? (
          <p className="text-sm text-fg-subtle">Chưa thấy máy quét nào. Kiểm tra cáp, nguồn và trình điều khiển TWAIN đã cài.</p>
        ) : (
          <ul className="space-y-1">
            {list.map((s) => (
              <li key={s.id} className="flex items-center gap-2 text-sm">
                <Icon name="scanner" size={14} className="text-fg-subtle" />
                <span className="font-medium">{s.name}</span>
                <span className="text-fg-subtle">{s.model}</span>
                <Badge tone={s.connected ? "ok" : "gray"}>{s.connected ? "Đã kết nối" : "Không kết nối"}</Badge>
              </li>
            ))}
          </ul>
        )}
      </Field>
      <Button icon="lamMoi" loading={busy} onClick={() => void do_()}>
        Dò lại
      </Button>
      <p className="text-sm text-fg-subtle">Độ phân giải, màu, khay và quét hai mặt chọn ngay ở màn Quét cho từng lô.</p>
    </div>
  );
}

function MucDuLieu() {
  const { session } = V460();
  const coStats = V3z9(session?.permissions, "stats.view");
  const [stats, setStats] = useState<V4qz | null>(null);
  const [gui, setGui] = useState(false);
  const [ocrUrl, setOcrUrl] = useState("");
  const load = useCallback(() => {
    if (coStats)
      V4r1()
        .then(setStats)
        .catch(() => {});
    V4cc()
      .then((u) => setOcrUrl(u ?? ""))
      .catch(() => {});
  }, [coStats]);
  V41p(load, { topics: ["settings", "stats"], enabled: isTauri() });
  const guiSoLieu = async () => {
    setGui(true);
    try {
      setStats(await V4r2());
      toast.success("Đã gửi số liệu về máy chủ. Quản trị thấy trong Báo cáo.");
    } catch (e) {
      notifyError(e, "gửi số liệu");
    } finally {
      setGui(false);
    }
  };
  const luuOcr = async () => {
    try {
      await V4cd(ocrUrl.trim() || null);
      toast.success("Đã lưu địa chỉ dịch vụ OCR.");
    } catch (e) {
      notifyError(e, "lưu địa chỉ OCR");
      throw e;
    }
  };
  return (
    <div className="space-y-6">
      <ChuyenDuLieuCu />
      {coStats && (
        <Card title="Số liệu của tôi" description="Số trang quét, hồ sơ và PDF trên máy này. Gửi để quản trị thấy trong báo cáo.">
          <p className="text-sm">{stats ? `${stats.pagesScanned.toLocaleString("vi-VN")} trang quét · ${stats.hoSoCount.toLocaleString("vi-VN")} hồ sơ · ${stats.pdfCount.toLocaleString("vi-VN")} PDF · ${(stats.pdfBytes / 1_048_576).toFixed(1)} MB` : "—"}</p>
          <Button className="mt-3" icon="thongke" loading={gui} onClick={() => void guiSoLieu()}>
            Gửi số liệu về quản trị
          </Button>
        </Card>
      )}
      <Card title="Dịnh vụ OCR" description="Để trống thì dùng địa chỉ máy chủ mặc định.">
        <div className="flex flex-wrap gap-2">
          <Input className="min-w-60 flex-1" value={ocrUrl} onChange={(e) => setOcrUrl(e.target.value)} placeholder="http://192.168.0.205:8000" spellCheck={false} aria-label="Địa chỉ dịch vụ OCR" />
          <NutLuu onSave={luuOcr} />
        </div>
      </Card>
    </div>
  );
}

function MucUngDung() {
  const { appVersion, update, checkForUpdates } = V460();
  const dangChay = update.phase === "checking" || update.phase === "downloading" || update.phase === "installing" || update.phase === "relaunching";
  return (
    <div className="space-y-6">
      <Field label="Phiên bản">
        <p className="text-sm">DigiText v{appVersion || "…"}</p>
      </Field>
      <Field label="Cập nhật phần mềm" hint="Tự kiểm tra khi mở ứng dụng, tải từ GitHub Releases.">
        <div className="flex flex-wrap items-center gap-3">
          <Button icon="lamMoi" loading={dangChay} onClick={checkForUpdates}>
            Kiểm tra cập nhật
          </Button>
          <span className="text-sm text-fg-subtle">{trangThaiCapNhat(update, appVersion)}</span>
        </div>
      </Field>
    </div>
  );
}

export default function V4rs() {
  const [muc, setMuc] = useState<Muc>("mayChu");
  const hien = MUC.find((m) => m.key === muc) ?? MUC[0];
  return (
    <section className="view active">
      <PageHeader title="Cài đặt" description="Máy chủ, nơi lưu trữ, máy quét, dữ liệu và cập nhật của máy này." />
      <div className="settings-grid">
        <nav className="set-nav" aria-label="Mục cài đặt">
          {MUC.map((m) => (
            <button key={m.key} type="button" className={`si w-full text-left ${muc === m.key ? "on" : ""}`} aria-current={muc === m.key ? "page" : undefined} onClick={() => setMuc(m.key)}>
              <Icon name={m.icon} size={16} />
              {m.label}
            </button>
          ))}
        </nav>
        <div className="set-body">
          <h3 className="mb-4 mt-0 text-base font-semibold">{hien.label}</h3>
          {muc === "mayChu" && <MucMayChu />}
          {muc === "luuTru" && <MucLuuTru />}
          {muc === "mayQuet" && <MucMayQuet />}
          {muc === "duLieu" && <MucDuLieu />}
          {muc === "ungDung" && <MucUngDung />}
        </div>
      </div>
    </section>
  );
}
