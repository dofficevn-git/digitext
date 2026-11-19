# Quy ước giao diện desktop — "ít chữ, dễ hiểu" (từ 0.25, 19/11/2026)

Kế hoạch gốc: `C:\Users\HieuPC\.claude\plans\t-i-mu-n-n-ng-c-p-stateful-whistle.md` (4 đợt). Tài liệu này là quy ước áp
cho mọi màn mới và mọi lần sửa màn cũ. Màu, phông chữ giao diện (Segoe UI), token: `desktop/src/styles/theme.css`
(đồng bộ DO365 từ 0.19) — không đổi.

## 1. Nguyên tắc

Chữ trên mặt màn chỉ để **gọi tên** và **báo việc cần làm**. Giải thích, căn cứ pháp lý, cách dùng nằm sau nút **(i)**
hoặc chú thích khi di chuột — không xoá, chỉ không chiếm chỗ.

| Chỗ | Quy ước | Thành phần |
|---|---|---|
| Mô tả dưới tiêu đề màn | Tối đa 1 câu ≤ 70 ký tự, hoặc bỏ. "Bước N Điều 9…" không nằm ở đây | `PageHeader description` (dev cảnh báo khi > 70) |
| Giải thích màn, căn cứ pháp lý | Nút (i) cạnh tiêu đề | `PageHeader info` / `infoTitle` (thay `Help`) |
| Chưa chọn dự án | Không viết câu riêng từng màn | `CanDuAn` (chọn dự án ngay tại chỗ) |
| Mô tả trong thẻ | Bỏ khi tiêu đề đủ nghĩa; còn lại ≤ 60 ký tự | `Card description` |
| Gợi ý dưới ô nhập | Chỉ giữ ràng buộc phải biết trước khi gõ; giải thích thêm → (i) cạnh nhãn | `Field hint` + `hintAs="tip"` |
| Banner | Chỉ cảnh báo / lỗi cần hành động; mỗi màn tối đa 1 banner cùng lúc (ưu tiên lỗi) | `Banner` |
| Nút | Động từ ≤ 3 từ; số lượng là huy hiệu, không viết vào nhãn | `Button count` |
| Mục menu | ≤ 28 ký tự | `Menu` |
| Bộ lọc | Một hàng: ô tìm + ô chọn; chip chỉ cho bộ lọc đang bật; có "Xoá lọc" | `FilterBar`, `MultiSelect` |
| Cột bảng | Tên ≤ 2 từ; đơn vị / giải thích vào chú thích | `Table` |
| Thanh trạng thái | Tốt = chấm + icon; có vấn đề mới hiện chữ | `shell/StatusBar` |
| Menu trái | Một dòng mỗi nhóm; số bước trong nút tròn; giải thích ở chú thích | `shell/Sidebar` |
| Trạng thái rỗng | 1 dòng tiêu đề + tối đa 1 nút | `EmptyState` |

Giữ các quy ước 0.17.1: không ①②③ / ✓●⚠, không HOA nhấn, tooltip ≤ 80 ký tự, kết quả thao tác → toast, cần quyết định →
`confirm()`.

## 2. Bố cục chuẩn một màn

```
<section className="view active space-y-4">
  <PageHeader title info actions />        tối đa 1 nút chính + menu ⋯
  <KpiRow />                               nếu màn có số liệu
  <FilterBar />                            một hàng
  <Card padding="none"><Table /></Card>
</section>
```

Khoảng cách dọc do `space-y-4` trên `section` lo; không rải `mb-3` / `mb-4` từng khối. Mỗi màn một `PageHeader` (nhánh
chưa chọn dự án vẫn dùng chính tiêu đề đó, bên dưới là `CanDuAn`).

## 3. Logo chữ

`ui/Wordmark` là chỗ DUY NHẤT vẽ chữ **DIGITEXT** (thanh trên, đăng nhập, màn nạp): in hoa, đậm, nghiêng 12°, phông
`--font-display` = Russo One (OFL, `desktop/src/assets/fonts/`, 7 kB, chỉ ký tự Latin). Bản Lưu trữ kèm chữ nhỏ
"Lưu trữ" bằng Segoe UI. `--font-display` không dùng cho chữ giao diện. Đổi phông logo: thay tệp `.woff2` + `@font-face`
trong `theme.css`. `config.ts appName` vẫn dùng cho tiêu đề cửa sổ / hộp thoại.

## 4. Ví dụ trước / sau

- Tiêu đề màn Kiểm tra sản phẩm — trước: mô tả "Bước 5 Điều 9 TT 05/2025: từng hồ sơ trong mục lục đối chiếu với tài liệu
  đã lên máy chủ — số tờ kê khai so với trang quét, PDF/A-2b, ký số…" (≈ 190 ký tự, luôn hiện). Sau: không mô tả; đoạn đó
  nằm trong `info`.
- Lập hồ sơ — trước: thẻ lọc 4 ô + 16 chip THBQ luôn hiện. Sau: `FilterBar` một hàng, THBQ là `MultiSelect`, chip chỉ hiện
  cho giá trị đang lọc.
- Nút — trước "Đóng gói 12 hồ sơ khớp". Sau `<Button count={12}>Đóng gói</Button>`.
- Thanh trạng thái — trước "Máy chủ: Đã kết nối · Phiên: Trực tuyến · Chỉnh lý: Đã đồng bộ 08:24 19-11". Sau: ba chấm xanh +
  icon; mất mạng thì hiện "Máy chủ: Mất kết nối".

## 5. Tiến độ

| Đợt | Nội dung | Trạng thái |
|---|---|---|
| 1 (0.25.0) | Logo chữ, menu trái một dòng, thanh trạng thái, bộ `ui/*` (`Wordmark`, `InfoButton`, `PageHeader info`, `CanDuAn`, `FilterBar`, `MultiSelect`, `Field hintAs`, `Button count`) | mã xong 19/11/2026 |
| 2 (0.25.0) | Dữ liệu chủ + Số hoá: Lập hồ sơ dùng `FilterBar`/`MultiSelect` + `NhacChinhLy` (một dải nhắc); mô tả → `info` ở Danh mục, Hệ thống hoá, Đồng bộ, Mục lục, Cây thư mục, Kho máy chủ, Đưa lên máy chủ; `MultiSelect plain` cho chọn cột; `HoSoHubView` hết `section` lồng | mã xong 19/11/2026, phát hành chung đợt 1 |
| 3 (0.25.1) | Kiểm tra, Ký số, Đóng gói, Nghiệm thu, Giao nhận: bỏ tiền tố "Bước N Điều 9…" và khối `Help` → `PageHeader info` (một nội dung `giaiThich` cho cả nhánh chưa chọn dự án, dùng `CanDuAn`); mô tả thẻ rút gọn; hint giải thích → `hintAs="tip"` | mã xong 19/11/2026 |
| 4 (0.25.3) | Lưu trữ số, Báo cáo, Tiện ích, Quản trị, Đăng nhập; gỡ `Help`; luật eslint chặn mô tả dài | chưa |
