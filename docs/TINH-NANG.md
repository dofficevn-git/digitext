# DigiText — Mô tả tính năng

Cập nhật: 01/10/2026 · Phiên bản ứng dụng 0.17.1

DigiText là hệ thống **số hoá và quản lý tài liệu lưu trữ** theo nghiệp vụ lưu trữ nhà nước Việt Nam, bám Thông tư 05/2025/TT-BNV. Phủ trọn vòng đời: quét → thu thập → OCR → PDF/A → bóc tách thông tin → duyệt → ký số → đóng gói → bảo quản → khai thác → tiêu huỷ.

## 1. Kiến trúc

| Thành phần | Vai trò |
|---|---|
| Ứng dụng máy trạm (Windows) | Nơi cán bộ làm việc. Có CSDL cục bộ nên **làm việc được khi mất mạng**, có mạng thì đồng bộ. Tự cập nhật phiên bản mới. |
| Máy chủ DigiText | Lưu dữ liệu, xếp hàng và điều phối xử lý, phân quyền, nhật ký. Truy cập từ mọi mạng qua `https://api.digitext.vn`. |
| Máy GPU | Nhận dạng chữ (OCR) và mô hình AI đọc ảnh. Có thể gộp chung với máy chủ. |

## 2. Thu thập và số hoá

- **Quét trực tiếp** từ máy quét (khay nạp tự động), lưu thẳng vào đúng hồ sơ.
- **Thêm tài liệu có sẵn** từ ổ cứng/USB/thư mục quét cũ; **thu thập qua mạng LAN** từ máy nhân viên về máy quản trị.
- **Đẩy lên máy chủ** theo đợt nộp: chia khúc, **nối lại khi đứt mạng**, khử trùng lặp theo nội dung tệp, đối soát số lượng.
- **Kho tài liệu** trên máy trạm tổ chức **theo từng dự án**; tách riêng bản gốc, bản tải về và bản PDF/A.

## 3. Nhận dạng chữ (OCR)

- OCR tiếng Việt có dấu chạy GPU: khoảng **1 giây/trang**. Mô hình đã tinh chỉnh trên chính loại văn bản hành chính đang số hoá — độ chính xác lớp chữ in **≈ 98–99%**.
- **Chữ viết tay**: dòng máy đọc kém tự động được mô hình AI đọc lại, có ngữ cảnh cả đoạn. Đủ để tìm kiếm theo nội dung; tên riêng/chữ số viết tay vẫn cần người soát.
- **Không sót dòng**: ngoài mô hình phát hiện dòng, có lớp tách dòng theo nét mực bắt cả chữ viết tay nét mảnh; mỗi trang được đo **độ phủ mực** (bao nhiêu % nét mực đã nằm trong lớp chữ), trang dưới 90% tự vào hàng soát.
- Đọc được cả tệp quét đen trắng/nén (CCITT, JBIG2), không chỉ JPEG.
- **Chịu lỗi**: máy GPU tắt thì hàng đợi tự chờ, không hỏng việc; trang lỗi được đếm riêng và có nút **chạy lại**; tài liệu thiếu trang không bị coi là xong.

## 4. PDF/A hai lớp

- Tự dựng **PDF/A-2b hai lớp** (ảnh quét + lớp chữ ẩn tìm kiếm được), loại bỏ lớp chữ rác do máy quét tạo sẵn.
- **Kiểm đúng Điều 8 TT05 trước khi dựng**: ảnh màu, ≥ 24 bit, ≥ 200 dpi (300 dpi với bản đồ/bản vẽ). Tệp không đạt bị từ chối kèm lý do rõ ràng.
- Đã kiểm độc lập bằng veraPDF: **đạt 100%** trên các lô thử.
- Tải về được **bản gốc hoặc bản PDF/A** tuỳ chọn.

## 5. Bóc tách 16 trường thông tin

- AI đọc ảnh trang và điền sẵn **16 trường mô tả văn bản** (số ký hiệu, ngày tháng, loại văn bản, cơ quan ban hành, trích yếu, người ký…).
- Kết quả luôn là **gợi ý chờ người duyệt**: xem vùng máy bóc ngay trên ảnh, sửa, vẽ bổ sung, bấm Duyệt. Dữ liệu đã duyệt dùng để cải thiện mô hình.

## 6. Chỉnh lý và lập hồ sơ

- Danh mục chỉnh lý (phông, phòng ban, mảng công việc, thời hạn bảo quản, loại văn bản, ngôn ngữ, khối phông) là **bản sao đồng bộ với máy chủ**: sửa được khi mất mạng, có mạng tự đẩy; xung đột cùng trường được hỏi và chọn theo từng trường.
- Lập hồ sơ điện tử (bảng 50 dòng, lọc theo phông / phòng ban / mảng / THBQ, nhập Excel theo mẫu), văn bản của hồ sơ, hệ thống hoá 4 bước (xem trước → đánh số → xuất mục lục tạm → nhập tệp đã điền hộp / bó / tập), kho và giá, mượn trả hồ sơ giấy (in phiếu).
- Tra cứu và kết xuất: metadata TT05 / TT02, mục lục hồ sơ 6 kiểu (Excel và in), mục lục văn bản, nhãn hộp, sản phẩm có bìa, phụ lục tra tìm, lịch sử thay đổi.
- Cây thư mục quét (Cơ quan › Hộp › Hồ sơ) lấy từ mục lục chỉnh lý hoặc Excel, phát cho máy nhân viên qua máy chủ hoặc mạng LAN (mã 6 số).

## 7. Ký số

- Ký bằng **USB token** của các CA Việt Nam (tự dò thiết bị), chữ ký **hiển thị góc trên bên phải trang đầu** đúng TT05.
- Máy chủ kiểm chuỗi chứng thư tin cậy trước khi chấp nhận chữ ký. Hỗ trợ **dấu thời gian** và **xác minh lâu dài (LTV)** để hàng chục năm sau vẫn kiểm được.
- Trạng thái: phần mềm sẵn sàng; cần cắm token và khai báo CA của đơn vị.

## 8. Đóng gói theo TT05

- Tạo gói **SIP** (nộp lưu), **AIP** (bảo quản), **DIP** (khai thác có xác thực) theo bố cục E-ARK CSIP như gói của các dự án đã nghiệm thu (Đầm Hà): METS gốc + METS biểu diễn, EAD hồ sơ + EAD từng văn bản, PREMIS đầy đủ (đối tượng, sự kiện, agent), bộ XSD tự đứng, checksum SHA-256, tên cơ quan hình thành phông / cơ quan lưu trữ / đơn vị đóng gói ghi trong gói, kèm được hướng dẫn sử dụng.
- Chỉ nhận tệp mang dấu PDF/A (TT05 Điều 8.2.a); tệp chưa phải PDF/A bị từ chối ngay khi đóng gói.
- **Tự kiểm gói** trước khi xuất; có thêm **bộ kiểm độc lập** để nghiệm thu gói của chính mình hoặc của đơn vị khác (đã dùng kiểm 2 gói bên ngoài: phát hiện 72/72 tệp không phải PDF/A).
- Gắn gói SIP với lần nộp lưu và dải số thứ tự mục lục; biên bản giao nhận.
- **Đóng gói kho tài liệu từ bên ngoài**: nhận thư mục đã quét sẵn (Phông/Hộp/Hồ sơ) + một tệp Excel metadata; đối chiếu thư mục với Excel (mã phông, mã hồ sơ, đường dẫn, tên tệp, thiếu/thừa tệp, số trang, điều kiện Điều 8 TT05) và **khớp hết mới đóng gói**, mỗi hồ sơ một gói; có báo cáo lỗi theo từng dòng Excel.

## 9. Bảo quản, khai thác, tiêu huỷ

- **Kho bất biến (WORM)**: gói tin đã lưu không xoá/sửa được; kiểm tra toàn vẹn định kỳ; sao lưu.
- **Khai thác hồ sơ**: tiếp nhận yêu cầu, duyệt, tính phí, cấp bản sao có xác thực (DIP).
- **Tiêu huỷ tài liệu hết giá trị**: lập danh mục, hội đồng thẩm định, quyết định, xoá an toàn.
- Tìm kiếm toàn văn; thống kê, báo cáo, biên bản theo mẫu.

## 10. Quản trị và vận hành

- Giao diện 0.17.1: menu hai cấp theo vòng đời (Tổng quan · Số hoá · Xử lý · Chỉnh lý · Lưu trữ số · Báo cáo · Quản trị), trang Tổng quan theo quyền, ô **Dự án đang làm** dùng chung, tìm nhanh Ctrl+K, chuông thông báo, dùng được hoàn toàn bằng bàn phím và trình đọc màn hình (nhãn ô, hộp thoại chuẩn, bảng sắp xếp được).
- Phân quyền theo vai trò và **theo từng dự án**; quản lý thiết bị; **nhật ký kiểm toán** mọi thao tác.
- Bảng tiến độ theo dự án: số tệp, đã OCR, đã PDF/A, đã bóc trường, đã duyệt; cập nhật thời gian thực.
- Máy chủ chạy dạng **dịch vụ tự khởi động, tự hồi** khi lỗi hoặc mất điện.
- **Tự cập nhật ứng dụng**: phát hành một lệnh, máy trạm tự tải bản mới đã ký số khi mở.

## 11. Giới hạn hiện tại

- Chữ viết tay nối nét: đọc ở mức tìm kiếm được, chưa đạt độ chính xác của chữ in.
- Ký số chưa chạy thật với token của đơn vị.
- Chưa có bộ dữ liệu chuẩn do người gõ để đo chính thức; số liệu chính xác hiện dựa trên đối chiếu máy và soát mẫu.
- Máy chủ đang đặt trên một máy trạm; khi có máy chủ GPU riêng sẽ gộp về một máy.
