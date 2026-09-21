# Đóng gói kho tài liệu từ bên ngoài

Cập nhật: 21/09/2026

Dùng khi tài liệu **đã được quét ở nơi khác**: có một thư mục chia theo Phông / Hộp / Hồ sơ và
**một tệp Excel metadata** (mỗi dòng một tài liệu, mang luôn thông tin phông và hồ sơ). DigiText
đối chiếu thư mục với Excel, **khớp hết mới đóng gói**, mỗi hồ sơ thành một gói TT 05/2025
(AIP hoặc SIP). Không tải gì lên máy chủ, không cần Postgres/MinIO.

## 1. Dùng trong ứng dụng

Màn **Đóng gói theo tiêu chuẩn** → khối **Đóng gói kho tài liệu từ bên ngoài**:

1. Chọn tệp Excel metadata. 2. Chọn thư mục kho (thư mục mà đường dẫn trong cột «Tệp tin tài
liệu» tính từ đó; trỏ nhầm vào thư mục con cùng tên vẫn tự nhận).
3. **Kiểm tra khớp với Excel.** Để nguyên ô «Mở từng PDF» thì mỗi tệp được mở ra soi (kho 67 GB
   mất khoảng 10 phút). Kết quả hiện ngay: ĐÃ KHỚP / CHƯA KHỚP, bảng loại vấn đề, và báo cáo
   chi tiết ghi cạnh tệp Excel trong `_digitext_kiem_tra\`.
4. **Đóng gói.** Nút chỉ bật khi không còn LỖI (hoặc người dùng chủ động tích «vẫn đóng trước
   các hồ sơ đã khớp»). Chọn loại gói, thư mục ghi gói. Chạy lại được — hồ sơ đã có gói thì bỏ qua.

Quyền cần có: *Tạo gói tin*.

## 2. Dùng bằng dòng lệnh (kho lớn, chạy qua đêm)

```
digitext ext-import check --deep --root "E:\16.CSDL_Dien Xa" --excel "E:\16.CSDL_Dien Xa\Metadata-DIENXA.xlsx"
digitext ext-import pack  --root … --excel … --out "F:\goi-tin" --kind aip_hoso
```

`digitext ext-import --help` liệt kê đủ tuỳ chọn (`--kind sip_hoso --agreement <mã>`, `--org`,
`--only`, `--limit`, `--allow-partial`, `--force`, `--threads`).

## 3. Excel phải có gì

Một sheet, dòng tiêu đề nằm trong 15 dòng đầu, ba khối cột theo thứ tự **phông → hồ sơ → tài
liệu**. Tên cột được nhận theo nội dung (không phân biệt hoa thường, dấu), cột trùng tên giữa
các khối («Ngôn ngữ», «Ghi chú», «Thời hạn lưu trữ», «Số lượng trang») được tách theo khối.

| Bắt buộc | Cột |
|---|---|
| Phông | Mã phông |
| Hồ sơ | Mã hồ sơ · Tiêu đề hồ sơ · Thời hạn lưu trữ · Số lượng tờ |
| Tài liệu | Tệp tin tài liệu (đường dẫn tương đối tới PDF) |

Các cột khác (thời gian bắt đầu/kết thúc, chế độ sử dụng, tên loại, số/ký hiệu, ngày ban hành,
cơ quan ban hành, trích yếu, số trang, từ khoá, bút tích…) nếu có sẽ được đưa vào EAD.

## 4. Bước kiểm bắt những gì

**Đơn vị hồ sơ là THƯ MỤC chứa tệp**, không phải «Mã hồ sơ» — vì chính mã hồ sơ có thể sai.

LỖI (chặn đóng gói):

| Mã | Ý nghĩa |
|---|---|
| `TEP-KHONG-CO-TREN-DIA` / `TEP-KHONG-CO-TRONG-EXCEL` | Excel khai mà không có tệp / tệp có mà Excel không khai |
| `EXCEL-TRUNG-TEP`, `TEP-RONG`, `PDF-HONG` | cùng tệp khai hai dòng; tệp 0 byte; PDF không mở được |
| `HS-TRUNG-MA` | một mã hồ sơ dùng cho nhiều thư mục hồ sơ |
| `HS-NHIEU-MA` | một thư mục hồ sơ mà Excel ghi nhiều mã hồ sơ / mã phông |
| `HS-MA-LECH-PHONG` | mã hồ sơ không bắt đầu bằng mã phông |
| `TM-LECH-SO-HO-SO` | số hồ sơ của thư mục khác số hồ sơ trong mã |
| `TL-LECH-THU-MUC` | «Tệp tin hồ sơ» khác thư mục thật của tệp |
| `TL-MA-LECH-TEN-TEP` | «Mã lưu trữ tài liệu» khác tên tệp thật |
| `TL-TEN-LECH-PHONG` / `-NAM` / `-SO-HO-SO` | tên tệp mang mã phông / năm / số hồ sơ khác mã hồ sơ |
| `TL-TRUNG-STT`, `HS-LECH-TONG-TAI-LIEU` | trùng số thứ tự tệp; tổng số tài liệu khai khác số dòng |
| `HS-THIEU-TRUONG`, `HS-THIEU-SO-TO` | thiếu trường bắt buộc |
| `PDF-LECH-SO-TRANG` | số trang Excel khác số trang PDF thật |
| `PDF-KHONG-PHAI-PDFA`, `PDF-THIEU-LOP-CHU`, `PDF-DPI-THAP`, `PDF-KHONG-MAU-24BIT`, `PDF-CHUA-KY-SO` | tệp chưa đạt **Điều 8.2.a TT05** |

CẢNH BÁO (vẫn đóng gói): ngày sai dạng `dd/mm/yyyy` hoặc thiếu, ngày ban hành ngoài khoảng thời
gian hồ sơ, thiếu trích yếu/tên loại, các dòng cùng hồ sơ ghi thông tin hồ sơ khác nhau, STT
không liên tục, tên tệp không có STT 7 số, thời hạn lưu trữ của tài liệu khác của hồ sơ.

Dữ liệu bẩn được hiểu giúp (không chặn): ngày lưu thành số Excel (`39052` → `01/12/2006`),
`9/7/2012` → `09/07/2012`, `2017-11-01` → `01/11/2017`, «Số lượng tờ» = `36 '` → 36.

Lưu ý về PDF/A: bước kiểm nhận tệp **có khai** PDF/A trong XMP. Hợp chuẩn ISO 19005 đầy đủ
kiểm bằng veraPDF (`dev/verapdf-gate.sh`) — nên chạy mẫu trước khi nộp.

## 5. Gói tin ra sao

- Một gói / hồ sơ, tên `urn_<mã phông>_uuid_<UUID>.zip` (AIP) — bố cục METS / EAD / PREMIS /
  schemas / `representations/rep1/data` y hệt gói DigiText tự tạo (cùng bộ dựng `digitext-package`,
  tự kiểm bằng `validate::f_2p7` trước khi ghi ra đĩa).
- Tệp trong gói được **đổi tên** thành `<mã hồ sơ>.<STT 7 số>.pdf` (quy tắc mã tài liệu TT05);
  tên tệp gốc ghi trong phần mô tả của tài liệu. **Nội dung tệp giữ nguyên từng byte** — chữ ký số
  không bị ảnh hưởng.
- Thư mục ra có thêm `ket-qua-dong-goi.csv` (từng hồ sơ: gói, SHA-256, lý do nếu hỏng) và
  `trang-thai-dong-goi.json` (để chạy lại bỏ qua hồ sơ đã xong).
- Nghiệm thu độc lập: `python ml/tools/check_tt05_package.py <gói.zip>`.

## 6. Kết quả trên kho Điền Xá (21/9/2026)

34.354 tài liệu / 863 hồ sơ / 67,5 GB / 95.657 trang; kiểm sâu mất 9,4 phút.
Thư mục ↔ Excel: **không thiếu, không thừa tệp nào.** Nhưng chưa đóng gói được vì:

1. **0/34.354 tệp là PDF/A** (producer iTextSharp, PDF 1.x thường) dù 100% đã ký số → trái Điều
   8.2.a. Chuyển sang PDF/A sẽ phá chữ ký → phải chuyển đổi rồi **ký lại**.
2. 205 hồ sơ có cột «Mã phông» (`H49.132.02`) lệch mã hồ sơ (`H49.132.01…`).
3. 41 thư mục hồ sơ dùng trùng mã hồ sơ (21 mã); 2 thư mục mang hai mã hồ sơ.
4. 1.906 dòng «Mã lưu trữ tài liệu» khác tên tệp thật; 188 tệp lệch năm; 76 tệp lệch số hồ sơ.
5. 140 tệp có trang **không có lớp chữ** (đã đối chiếu PyMuPDF: đúng cả 140 — 12 tệp PyMuPDF "thấy chữ"
   chỉ là dòng hiển thị của chữ ký số); 23 tệp < 200 dpi; 9 tệp lệch số trang.
6. Lớp chữ sẵn có là OCR của máy quét Canon, **không nhận tiếng Việt** ("cQNG HoA xA HOr curi NGHIA") —
   đạt hình thức "hai lớp" nhưng tìm kiếm gần như vô dụng. Không phải lỗi chặn; nêu để biết.

Báo cáo đầy đủ: `test/dienxa/kiem-tra-sau/`.
