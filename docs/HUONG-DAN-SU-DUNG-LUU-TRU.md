# Hướng dẫn sử dụng DigiText Lưu trữ theo vai trò

Bản 0.25.26 (27/11/2026). Phần mềm lưu trữ số của một cơ quan theo TT 05/2025/TT-BNV, chạy trên máy trạm Windows nối máy chủ của cơ quan.
Mọi mục menu ở thanh bên trái; mục nào không thấy là tài khoản chưa có quyền (Quản trị › Vai trò và quyền).

## Mọi người dùng

- **Đăng nhập**: màn đăng nhập › Máy chủ › nhập `https://<địa chỉ máy chủ>`; lần đầu cài chứng chỉ gốc từ `https://<máy chủ>/tai-ve`
  nếu dùng mạng nội bộ.
- **Tra cứu › Tra cứu tài liệu**: tìm không dấu trên mục lục và lớp chữ; mở bản đọc; **In ra giấy** tạo bản chuyển đổi Phụ lục VI
  (có trang thể thức, vào Sổ chuyển đổi).
- **Lưu trữ số › Yêu cầu khai thác**: gửi yêu cầu đọc / bản sao / bản sao có ký số / gói DIP; theo dõi phí, hạn sử dụng.
- Chuông thông báo: nhắc hạn nộp lưu, mượn quá hạn, kết quả duyệt.

## Nhân viên phòng ban (vai trò *Nhân viên phòng ban*)

1. **Lưu trữ hiện hành › Lập hồ sơ**: thêm hồ sơ (chọn dòng Danh mục hồ sơ của năm nếu có), nhập văn bản, đính tệp. Các ô Chế độ sử dụng,
   Mức độ tin cậy, Tình trạng vật lý, Tình trạng dự phòng lấy theo **bảng mã TT05**.
2. **Hệ thống hoá**: đánh số hồ sơ chính thức, xếp hộp (sau khi hồ sơ xong).
3. **Nộp lưu**: chọn hồ sơ đã kết thúc → *Nộp lưu*; hạn 60 ngày (hoặc theo cấu hình cơ quan), nhắc 15 / 7 / 1 ngày. Hồ sơ bị trả lại thì
   hoàn thiện rồi nộp lại. Sau khi tiếp nhận hồ sơ khoá sửa; người nộp còn đọc 360 ngày.
4. Chỉ thấy hồ sơ phòng mình.

## Cán bộ lưu trữ (vai trò *Cán bộ lưu trữ*)

- **Lưu trữ hiện hành › Phông, phòng ban / Danh mục hồ sơ / Tiếp nhận nộp lưu**: duyệt danh mục, chấm từng hồ sơ nộp (nhận / trả lại),
  biên bản giao nhận và sổ nộp lưu tự lập.
- **Nhập Hệ thống**: *Nhập gói AIP* từ đơn vị số hoá; *Thu nộp lưu trữ lịch sử* (Điều 20–25) với đăng ký → xác nhận → nộp gói →
  tiếp nhận → chấm → phê duyệt; mỗi đợt có **tệp đính kèm** (văn bản đề nghị, lịch sử phông…), **thêm hồ sơ có sẵn** (nộp cùng hệ thống),
  nút **So sánh** cho hồ sơ nghi trùng, **Thông tin tiếp nhận** 10 mục; *Cơ quan nộp lưu*: khai cơ quan, xem tài khoản, cấp / thu hồi
  khoá API (xem `API-THU-NOP.md`).
- **Giao nhận tài liệu, Số hoá, Kiểm tra sản phẩm, Ký số, Nghiệm thu**: khi cơ quan tự số hoá (cần máy chủ có dịch vụ OCR nếu muốn
  nhận dạng chữ).
- **Lưu trữ số**: duyệt *Yêu cầu khai thác* (chấm từng tài liệu; tài liệu "có điều kiện" sẽ chuyển **chờ duyệt cấp 2**), phân công người
  xử lý, ghi **giao nhận bản sao giấy**; *Cung cấp thông tin* (danh mục / trích xuất / tổng hợp); *Sổ chuyển đổi*; *Bảo quản và sao lưu*
  (kế hoạch kiểm tra Điều 29, sao lưu Điều 30, phương tiện, sự cố Điều 31, Mẫu 01–04); *Tiêu huỷ* (quét hết hạn / trùng, hội đồng,
  thẩm định, quyết định, thực hiện); *Gói tin SIP/AIP/DIP*; *Sổ biên bản*.
- **Kho lưu trữ**: kho / giá / hộp, gán hộp, sức chứa; mượn – trả có duyệt, gia hạn, quá hạn.
- **Báo cáo › Thống kê TT05**: 19 báo cáo, xuất Excel / PDF; cột đúng hạn / quá hạn ở báo cáo khai thác.
- **Danh mục › Mã TT05**: thêm mã riêng của cơ quan cho chế độ sử dụng, mức độ tin cậy, tình trạng vật lý, tình trạng dự phòng.

## Lãnh đạo (vai trò *Lãnh đạo*, nền Quản lý)

- Duyệt / ban hành danh mục hồ sơ; duyệt kế hoạch kiểm tra; duyệt phiếu mượn; **duyệt cấp 2** yêu cầu khai thác có tài liệu tiếp cận có
  điều kiện (Điều 36.2); xem toàn bộ báo cáo và Tổng quan.

## Quản trị hệ thống

- **Quản trị › Người dùng**: tạo tài khoản, gắn phòng ban, gắn **cơ quan nộp lưu** (tài khoản của cơ quan ngoài), phạm vi phông.
- **Quản trị › Vai trò và quyền**: sửa bộ quyền từng vai trò.
- **Quản trị › Cơ quan lưu trữ**: mã cơ quan lưu trữ (đi vào mã hồ sơ `ma_ho_so_tt05`, mã hồ sơ giấy), tên, cấp trên, người ký, hạn nộp lưu,
  mốc nhắc, quyền sử dụng.
- **Bảo quản › Tổng quan**: kiểm **chuỗi băm nhật ký** (`/ops/audit/kiem-chuoi`) khi nghi nhật ký bị sửa.
- Máy chủ: `docs/CAI-DAT-MAY-CHU-LUU-TRU.md`.
