// Copyright (c) 2026 DigiText. All rights reserved.
// Xác nhận cây thư mục trước khi đẩy lên máy chủ (22/10/2026): tên thư mục đã chọn là PHÔNG,
// thư mục con là Hộp/Hồ sơ. Nếu người dùng chọn nhầm thư mục chứa cả dự án thì chuyển sang chế độ
// "thư mục con là phông". Máy chỉ đọc hai cấp đầu nên hộp thoại mở tức thì với cả cây 30.000 tệp.

import { useMemo } from "react";
import type { PeekFolder } from "@/api/lansync";
import { Badge, Button, Dialog, Seg } from "@/ui";
import { Icon } from "@/components/Icon";

export type CheDoThuMuc = "phong" | "duAn";

const fmtN = (n: number) => n.toLocaleString("vi-VN");
/** Tên thư mục trông như cấp Hộp: "HOP 01", "Hộp 12", "HỘP 3", "Box 4". */
export const laHop = (name: string) => /^(h[oộô]p|box)\b/i.test(name.trim());

/** Đoán chế độ: có thư mục con tên "HOP…" → thư mục đã chọn là phông; không có HOP nào mà hầu hết
 *  thư mục con lại có thư mục con bên trong → thư mục đã chọn là cả dự án. Mặc định là phông. */
export function doanCheDo(folders: PeekFolder[]): CheDoThuMuc {
  const dirs = folders.filter((f) => f.isDir && f.subs.length > 0);
  if (!dirs.length) return "phong";
  const coHop = dirs.some((f) => f.subs.some((s) => laHop(s.name)));
  if (coHop) return "phong";
  const sau = dirs.every((f) => f.subs.filter((s) => s.dirs > 0).length * 2 >= f.subs.length);
  return sau ? "duAn" : "phong";
}

const CHE_DO: { value: CheDoThuMuc; label: string }[] = [
  { value: "phong", label: "Mỗi thư mục là một phông" },
  { value: "duAn", label: "Thư mục con là phông" },
];

export function XacNhanCayThuMuc({ duAn, folders, cheDo, onCheDo, nhanNut, onClose, onOk }: { duAn: string; folders: PeekFolder[]; cheDo: CheDoThuMuc; onCheDo: (c: CheDoThuMuc) => void; nhanNut: string; onClose: () => void; onOk: () => void }) {
  const dirs = folders.filter((f) => f.isDir);
  const files = folders.length - dirs.length;
  // Danh sách phông sẽ tạo theo chế độ đang chọn.
  const phongs = useMemo(() => {
    if (cheDo === "phong") return dirs.map((f) => ({ ten: f.name, hop: f.subs.length, hopDung: f.subs.filter((s) => laHop(s.name)).length, le: f.pdfs }));
    return dirs.flatMap((f) => f.subs.map((s) => ({ ten: s.name, hop: s.dirs, hopDung: -1, le: 0 })));
  }, [dirs, cheDo]);
  const leGoc = cheDo === "duAn" ? dirs.reduce((a, f) => a + f.pdfs, 0) : 0;
  const nghiNgo = cheDo === "phong" && dirs.some((f) => f.subs.length > 0 && !f.subs.some((s) => laHop(s.name)) && f.subs.filter((s) => s.dirs > 0).length * 2 >= f.subs.length);
  const khongHop = cheDo === "phong" ? phongs.filter((p) => p.hop === 0 && p.le === 0).length : phongs.filter((p) => p.hop === 0).length;
  const MAX = 40;

  return (
    <Dialog
      open
      onOpenChange={(o) => !o && onClose()}
      title="Xác nhận cây thư mục"
      description={`Máy chủ xếp theo ${duAn}/‹phông›/‹hộp›/‹hồ sơ›. Kiểm tra tên phông bên dưới trước khi gửi.`}
      size="md"
      footer={
        <>
          <Button onClick={onClose}>Huỷ</Button>
          <Button variant="primary" icon="taiLen" onClick={onOk} disabled={!phongs.length && !files}>
            {nhanNut}
          </Button>
        </>
      }
    >
      <div className="space-y-3">
        <Seg<CheDoThuMuc> options={CHE_DO} value={cheDo} onChange={onCheDo} full />
        <p className="text-sm text-fg-subtle">
          {cheDo === "phong" ? "Tên thư mục đã chọn là tên phông; thư mục con bên trong là Hộp, trong hộp là Hồ sơ." : "Thư mục đã chọn là cả dự án: mỗi thư mục con cấp 1 là một phông, bên trong là Hộp/Hồ sơ."}
        </p>
        {nghiNgo && (
          <p className="flex items-start gap-2 text-sm text-warning-700">
            <Icon name="canhBao" className="mt-0.5 h-4 w-4 shrink-0" />
            <span>Bên trong không thấy thư mục “HOP…” mà toàn thư mục còn có cấp con: có thể anh/chị đã chọn thư mục của cả dự án. Thử chuyển sang “Thư mục con là phông”.</span>
          </p>
        )}
        <div className="rounded border border-border">
          <div className="flex items-center justify-between border-b border-border-subtle bg-gray-50 px-3 py-1.5 text-sm">
            <span className="font-medium">
              {fmtN(phongs.length)} phông sẽ nhận
              {files > 0 ? ` · ${fmtN(files)} tệp lẻ` : ""}
              {leGoc > 0 ? ` · ${fmtN(leGoc)} tệp lẻ ở gốc` : ""}
            </span>
            {khongHop > 0 && (
              <Badge tone="warn">{fmtN(khongHop)} phông không có hộp</Badge>
            )}
          </div>
          <ul className="max-h-72 divide-y divide-border-subtle overflow-auto text-sm">
            {phongs.slice(0, MAX).map((p, i) => (
              <li key={`${p.ten}-${i}`} className="flex items-center gap-2 px-3 py-1.5">
                <Icon name="thuMuc" className="h-4 w-4 shrink-0 text-fg-subtle" />
                <span className="min-w-0 flex-1 truncate font-medium" title={p.ten}>
                  {p.ten}
                </span>
                <span className="shrink-0 tabular-nums text-fg-subtle">
                  {p.hop > 0 ? (p.hopDung === 0 ? `${fmtN(p.hop)} thư mục con` : `${fmtN(p.hop)} hộp`) : "không có hộp"}
                  {p.le > 0 ? ` · ${fmtN(p.le)} tệp lẻ` : ""}
                </span>
              </li>
            ))}
            {phongs.length > MAX && <li className="px-3 py-1.5 text-fg-subtle">… và {fmtN(phongs.length - MAX)} phông nữa</li>}
            {!phongs.length && <li className="px-3 py-2 text-fg-subtle">Không có thư mục con nào. {files > 0 ? "Tệp lẻ sẽ vào “Chưa phân loại”." : "Hãy chọn lại thư mục."}</li>}
          </ul>
        </div>
        <p className="text-sm text-fg-subtle">Tệp lẻ (không nằm trong thư mục phông) vào {duAn}/Chưa phân loại. Tệp trùng nội dung đã có trên máy chủ sẽ bị bỏ qua.</p>
      </div>
    </Dialog>
  );
}
