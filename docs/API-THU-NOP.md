# API thu nộp trực tuyến cho hệ thống của cơ quan nộp lưu (Điều 18.2, 20–25 TT 05/2025/TT-BNV)

Từ bản 0.25.26 (migration 0040). Dành cho đơn vị phát triển phần mềm của cơ quan nộp tài liệu vào lưu trữ lịch sử dùng DigiText Lưu trữ.

## 1. Xác thực

Lưu trữ lịch sử khai cơ quan ở **Nhập Hệ thống › Cơ quan nộp lưu**, tạo ít nhất một tài khoản gắn cơ quan (Quản trị › Người dùng,
ô "Cơ quan nộp lưu"), rồi bấm **Cấp khoá API**. Khoá dạng `dtk_…`, chỉ hiện một lần; máy chủ chỉ giữ SHA-256 của khoá.

Mọi yêu cầu gửi kèm header:

```
X-Api-Key: dtk_...
```

Khoá mang danh tính của tài khoản đầu tiên gắn cơ quan: quyền, phạm vi dự án và phạm vi cơ quan áp như người dùng đó đăng nhập.
Tài khoản của cơ quan nộp **chỉ thấy đợt nộp của cơ quan mình**. Thu hồi khoá ở cùng màn; khoá bị thu hồi trả `401`.

Địa chỉ gốc: `https://<máy chủ lưu trữ>/api/v1`. Chứng chỉ máy chủ nội bộ: tải chứng chỉ gốc ở `https://<máy chủ>/tai-ve`.

## 2. Luồng

| Bước | Điều | Gọi | Ghi chú |
|---|---|---|---|
| Đăng ký đợt nộp | 20 | `POST /submissions` | `coQuanNopId`, `orgName`, `orgCode` tự lấy từ khoá; thân gồm `projectId`, `fondCode`, `summary`, `phuongAnPhanLoai`, `cachThuc` (`truc_tuyen` / `truc_tiep`), `duKienThoiGian`, `lienHe`, `totalPages`, `totalBytes`, `hanNop` (ISO ngày; nên trước 12 tháng), `items[]` (`maHoSo`, `tieuDe`, `thoiGianBatDau`, `thoiGianKetThuc`, `soTrang`, `mucDoTiepCan`, `ghiChu`). Dòng trùng `maHoSo` trong cùng đợt được lưu nhưng tự đánh **không đạt** (`trungTrongDot`). |
| Đính kèm | 20.3 | `POST /submissions/{id}/files` | `{ "loai": "van_ban_de_nghi" \| "lich_su_phong" \| "phuong_an_phan_loai" \| "huong_dan_xdgt" \| "khac", "filename", "contentBase64", "ghiChu" }`, tối đa 25 MB / tệp. `GET …/files`, `GET …/files/{fid}` (tải), `DELETE …/files/{fid}` khi đợt còn đăng ký / đã xác nhận. |
| Theo dõi xác nhận | 21 | `GET /submissions/{id}` | `status`: `registered` → `confirmed` (đồng ý thu, có `decisionNote`) hoặc `refused`. |
| Nộp gói SIP | 22 | `POST /packages` với `submissionId` (xem `docs/PROJECT-MAP.md` §5 packages) hoặc nộp trực tiếp qua màn Nhập gói AIP của lưu trữ | Gói kiểm bằng `digitext-pkgcheck`; đối chiếu đủ / thiếu với mục lục: `POST /submissions/{id}/reconcile`. |
| Tiếp nhận, chấm, phê duyệt | 23–25 | `GET /submissions/{id}` | Theo dõi `status` `awaiting_reception` → `received` → `approved` → `completed`; từng dòng `items[].reviewResult` (`dat` / `khong_dat`, `reviewNote`). |
| 10 thông tin tiếp nhận | 23 | `GET /submissions/{id}/tong-quan` | `tongGoi`, `hoSoDangKy`, `hoSoDaDuyet`, `hoSoKhongDat`, `hoSoTrungDot`, `hoSoDaVaoHeThong`, `dungLuongGoi`, `totalPages`, `totalBytes`, `soLanNop`, các mốc thời gian. |
| Hồ sơ trả lại | 25.4 | `GET /submissions/{id}/returned` | Danh sách không duyệt kèm gói SIP để tải về. |
| Biên bản | 22, 25 | `GET /records?projectId=…&kind=thu_nop` | Biên bản thu nộp / giao nhận / trả lại do lưu trữ lập; tải `GET /records/{rid}/download`. |

Mã lỗi: `400` thân sai, `401` thiếu / sai khoá, `403` ngoài phạm vi cơ quan, `404` không có, `409` trạng thái không cho phép. Thân lỗi
`{ "code": "...", "message": "..." }` tiếng Việt.

## 3. Ví dụ (PowerShell)

```powershell
$h = @{ "X-Api-Key" = "dtk_..." ; "Content-Type" = "application/json; charset=utf-8" }
$b = @{ projectId = "<id kho>"; fondCode = "UBND-X"; summary = "Tài liệu 2015-2020"; hanNop = "2027-12-31";
        items = @(@{ maHoSo = "UBND-X.01.2018.01"; tieuDe = "Tập lưu quyết định 2018"; soTrang = 120 }) } | ConvertTo-Json -Depth 5
$dot = Invoke-RestMethod -Method Post -Uri https://luutru.coquan.vn/api/v1/submissions -Headers $h -Body ([Text.Encoding]::UTF8.GetBytes($b))
Invoke-RestMethod -Uri "https://luutru.coquan.vn/api/v1/submissions/$($dot.id)/tong-quan" -Headers $h
```

## 4. Chưa có

OAuth2 / OpenAPI phát sinh tự động (Digifile D12 có Swagger khi Development) — tài liệu này là mô tả tay. Kết nối LGSP / trục liên
thông (I03) chưa làm.
