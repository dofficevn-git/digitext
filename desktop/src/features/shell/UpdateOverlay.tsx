// Copyright (c) 2026 DigiText. All rights reserved.
import type { V42t } from "@/api/updater";

const V4x6 = (n: number) => (n / 1_000_000).toFixed(1).replace(".", ",");

export default function V4x7({ status, onDismiss: V3a7 }: { status: V42t; onDismiss: () => void }) {
  if (status.phase === "idle") return null;
  if (status.phase === "checking" && !status.manual) return null;
  if (status.phase === "uptodate" && !status.manual) return null;

  if (status.phase === "checking") {
    return (
      <div className="overlay">
        <div className="dialog text-center">
          <div className="login-logo mt-0 mx-auto mb-3.5">DT</div>
          <h3 className="mb-2">Đang kiểm tra cập nhật…</h3>
          <div className="progress">
            <span className="w-full" />
          </div>
        </div>
      </div>
    );
  }

  if (status.phase === "uptodate") {
    return (
      <div className="overlay">
        <div className="dialog text-center">
          <div className="login-logo mt-0 mx-auto mb-3.5">DT</div>
          <h3 className="mb-2">Đã là bản mới nhất</h3>
          <p className="muted text-sm mb-3.5">DigiText v{status.version ?? "?"}</p>
          <div className="actions justify-center">
            <button className="btn primary" onClick={V3a7}>
              Đóng
            </button>
          </div>
        </div>
      </div>
    );
  }

  if (status.phase === "error") {
    return (
      <div className="overlay">
        <div className="dialog">
          <h3>Không kiểm tra được cập nhật</h3>
          <p className="muted text-sm break-words">{status.error}</p>
          <div className="actions">
            <button className="btn primary" onClick={V3a7}>
              Bỏ qua &amp; tiếp tục
            </button>
          </div>
        </div>
      </div>
    );
  }

  const V3a8 = status.phase === "downloading";
  const V3a9 = V3a8 && status.total > 0 ? Math.round((status.downloaded / status.total) * 100) : 0;
  const width = V3a8 && status.total > 0 ? `${V3a9}%` : "100%";
  const text =
    status.phase === "available"
      ? "Đang chuẩn bị cập nhật…"
      : V3a8
        ? `Đang tải bản ${status.version}…${status.total > 0 ? ` ${V3a9}%` : ""}`
        : status.phase === "installing"
          ? "Đang cài đặt bản mới…"
          : "Đang khởi động lại…";

  return (
    <div className="overlay">
      <div className="dialog text-center">
        <div className="login-logo mt-0 mx-auto mb-3.5">DT</div>
        <h3 className="mb-2">Cập nhật DigiText</h3>
        <p className="muted text-sm mb-3.5">{text}</p>
        <div className="progress">
          <span style={{ width }} />
        </div>
        {V3a8 && status.total > 0 && (
          <div className="muted tnum text-xs mt-2">
            {V4x6(status.downloaded)} / {V4x6(status.total)} MB
          </div>
        )}
      </div>
    </div>
  );
}
