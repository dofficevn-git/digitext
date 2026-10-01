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

## 1b. Tab "Đóng gói nhanh" (bỏ qua Điều 8.2.a)

Cùng màn Đóng gói, tab **Đóng gói nhanh** chạy đúng luồng Từ kho ngoài nhưng **chỉ đối chiếu với Excel**
(mã phông, mã hồ sơ, đường dẫn, tên tệp, thiếu/thừa tệp, số trang). Không mở từng PDF để kiểm
độ phân giải, PDF/A, lớp chữ, chữ ký số; bộ dựng cũng không chặn tệp không phải PDF/A
(`allowNonTt05`, tương đương `--allow-non-tt05` ở dòng lệnh). Dùng khi chủ dự án chấp nhận bản quét
hiện có; gói có thể chưa đạt Điều 8.2.a TT05 và màn có dải cảnh báo nói rõ điều đó.

## 2. Dùng bằng dòng lệnh (kho lớn, chạy qua đêm)

```
digitext ext-import check --deep --root "E:\16.CSDL_Dien Xa" --excel "E:\16.CSDL_Dien Xa\Metadata-DIENXA.xlsx"
digitext ext-import pack  --root … --excel … --out "F:\goi-tin" --kind aip_hoso
```

`digitext ext-import --help` liệt kê đủ tuỳ chọn (`--kind sip_hoso --agreement <mã>`, `--org`,
`--org-name <tên cơ quan lưu trữ>`, `--fond-name <tên cơ quan hình thành phông>`,
`--packager <tên>|<mã>` (đơn vị đóng gói), `--only`, `--limit`, `--allow-partial`, `--force`,
`--threads`). Đặt biến môi trường `DIGITEXT_PACKAGE_MANUAL=<tệp.pdf>` để kèm hướng dẫn sử dụng
vào `documentation/UserManual.pdf` của mỗi gói.

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

Thư mục gốc: chọn thư mục **chứa** đoạn đầu của đường dẫn trong Excel (Excel ghi `UBND\DCXD\HOP 24\…`
thì chọn thư mục có `UBND` bên trong, không chọn `…\UBND\DCXD`). Chọn lệch thì bước kiểm tự dò lên
các cấp cha; không dò được sẽ báo `THU-MUC-GOC-KHONG-CO` kèm thư mục nên chọn.

LỖI (chặn đóng gói):

| Mã | Ý nghĩa |
|---|---|
| `TEP-KHONG-CO-TREN-DIA` / `TEP-KHONG-CO-TRONG-EXCEL` | Excel khai mà không có tệp / tệp có mà Excel không khai |
| `EXCEL-TRUNG-TEP`, `TEP-RONG`, `PDF-HONG` | cùng tệp khai hai dòng; tệp 0 byte; PDF không mở được |
| `HS-TRUNG-MA` | một mã hồ sơ dùng cho nhiều thư mục hồ sơ |
| `HS-NHIEU-MA` | một thư mục hồ sơ mà Excel ghi nhiều mã hồ sơ / mã phông. Tệp `BIA.pdf` (bìa hồ sơ) và `MLVB.pdf` (mục lục văn bản) không mang mã trong tên: nhận mã phông, mã hồ sơ theo các tệp cùng thư mục, không kiểm tên, **vẫn đóng gói** với STT đánh tiếp sau số lớn nhất của hồ sơ (thông tin `BIA-MLVB`). Ô «Mã hồ sơ» ghi mã văn bản (có đuôi STT 7 số, có cả mục lục số) được tự cắt về mã hồ sơ |
| `HS-MA-LECH-PHONG` | mã hồ sơ không bắt đầu bằng mã phông |
| `TM-LECH-SO-HO-SO` | số hồ sơ của thư mục khác số hồ sơ trong mã |
| `TL-LECH-THU-MUC` | «Tệp tin hồ sơ» khác thư mục thật của tệp |
| `TL-MA-LECH-TEN-TEP` | «Mã lưu trữ tài liệu» khác tên tệp thật |
| `TL-TEN-LECH-PHONG` / `-NAM` / `-SO-HO-SO` / `-MUC-LUC` | tên tệp mang mã phông / năm / số hồ sơ / mục lục số khác mã hồ sơ (tách theo quy tắc `mã cơ quan.phông.năm.số hồ sơ[.mục lục].STT`, mốc là đoạn năm) |
| `TL-TRUNG-STT`, `HS-LECH-TONG-TAI-LIEU` | trùng số thứ tự tệp; tổng số tài liệu khai khác số dòng |
| `HS-THIEU-TRUONG`, `HS-THIEU-SO-TO` | thiếu trường bắt buộc |
| `PDF-LECH-SO-TRANG` | số trang Excel khác số trang PDF thật |
| `PDF-KHONG-PHAI-PDFA`, `PDF-THIEU-LOP-CHU`, `PDF-DPI-THAP`, `PDF-KHONG-MAU-24BIT`, `PDF-CHUA-KY-SO` | tệp chưa đạt **Điều 8.2.a TT05** |

**Bản đồ / bản vẽ (ngưỡng 300 dpi thay vì 200).** Xếp loại cố ý BẢO THỦ, chỉ hai dấu hiệu CHẮC:
(1) khổ trang có cạnh dài ≥ 45 cm (lớn hơn A3), hoặc (2) cột «Tên loại tài liệu» ghi bản đồ / bản vẽ /
sơ đồ / bản trích đo / bình đồ / mặt cắt / mặt bằng. Mỗi tệp bị xếp là bản đồ có dòng THÔNG TIN
`TL-XEP-BAN-DO` ghi rõ lý do để soát. Trích yếu có từ khoá («tỉ lệ 1/…», «sơ đồ»…) mà tên loại
không nói và trang ≤ A3 → chỉ CẢNH BÁO `TL-NGHI-BAN-DO`, vẫn áp 200 dpi. Muốn ép 300 dpi cho tệp
đó: ghi «Tên loại tài liệu» = «Bản đồ» hoặc «Bản vẽ» trong Excel.

CẢNH BÁO (vẫn đóng gói): ngày sai dạng `dd/mm/yyyy` hoặc thiếu, ngày ban hành ngoài khoảng thời
gian hồ sơ, thiếu trích yếu/tên loại, các dòng cùng hồ sơ ghi thông tin hồ sơ khác nhau, STT
không liên tục, tên tệp không có STT 7 số, thời hạn lưu trữ của tài liệu khác của hồ sơ.

Dữ liệu bẩn được hiểu giúp (không chặn): ngày lưu thành số Excel (`39052` → `01/12/2006`),
`9/7/2012` → `09/07/2012`, `2017-11-01` → `01/11/2017`, «Số lượng tờ» = `36 '` → 36.

**Dự án có yêu cầu ISO không?** Ô «Dự án này yêu cầu đạt chuẩn ISO 19005» (app, nhớ theo tệp Excel;
CLI `--verapdf`). Mặc định TẮT vì TT05 chỉ ghi "PDF/A hai lớp", không dẫn chiếu ISO. Bật khi bên
nghiệm thu kiểm bằng veraPDF hoặc đơn vị muốn chắc chắn: từng tệp bị kiểm đúng chuẩn theo loại tự
khai, không đạt là LỖI chặn đóng gói. Tắt: chỉ đòi tệp khai PDF/A + hồ sơ màu.

PDF/A: **nhận mọi loại theo ISO 19005** (1a/1b/2a/2b/2u/3a/3b/3u/4/4e/4f) vì TT05 chỉ đòi "PDF/A hai
lớp". Kiểm cấu trúc (khai `pdfaid` + hồ sơ màu) luôn chạy; bật `--verapdf` / ô «Xác minh chuẩn ISO
bằng veraPDF» thì từng tệp được veraPDF kiểm **theo đúng loại nó tự khai**, ở bước kiểm sâu và ngay
trước khi dựng gói (mã lỗi `PDF-KHONG-DAT-ISO`, `PDF-THIEU-HO-SO-MAU`, `PDF-LOAI-PDFA-LA`). Cần Docker;
kho lớn mất vài giờ. Đã thử (22/9): tệp PDF/A-2B của DigiText qua trọn bộ, chỉ vướng «chưa ký số».

## 4b. Nút "Kiểm tra PDF/A" (tệp / thư mục bất kỳ)

Màn **Đóng gói** → khối **Kiểm tra PDF/A**; dòng lệnh: `digitext ext-import pdfa <tệp|thư mục>… [--verapdf] [--csv out.csv]`.
Cho biết từng tệp **là loại PDF/A nào** (PDF/A-1B, 2B, 3B… đọc từ `pdfaid` trong XMP, kể cả XMP nén)
hay không phải PDF/A, kèm lý do; tích «veraPDF» (máy có Docker + image `verapdf/cli:v1.30.2`) thì
kiểm đúng chuẩn ISO 19005 theo loại tệp tự khai (tệp không khai → kiểm theo 2b) và liệt kê luật vi phạm.
Bảng đầy đủ ghi ra `_digitext_kiem_tra_pdfa.csv` cạnh lựa chọn. Trần 5.000 tệp/lượt trong app.

**TT05 đòi loại nào?** Khoản 2 Điều 8 chỉ ghi *"định dạng PDF/A hai lớp"* — KHÔNG chỉ định phiên bản.
Mọi loại PDF/A (1b, 2b, 3b…) đều thoả chữ nghĩa; DigiText tạo PDF/A-2B. "Bôi được chữ bằng Ctrl+A"
chỉ chứng minh có LỚP CHỮ, không chứng minh là PDF/A; dải PDF/A của Foxit chỉ hiện khi bật
Preferences → Documents → PDF/A View Mode.

Kho Điền Xá (21/9): veraPDF trên 1 tệp/hồ sơ = 863 tệp → **0 đạt** cả 2b lẫn 1b; quét 34.354 tệp:
0 tệp có `pdfaid`, 0 có khối XMP, 0 có `/OutputIntents`; 34.354 có chữ ký, 34.354 do iTextSharp tạo.
Kết quả: `test/dienxa/vera-863/`.

## 5. Gói tin ra sao

- Một gói / hồ sơ, tên `urn_<mã phông>_uuid_<UUID>.zip` (AIP) hoặc `<số thứ tự>_uuid-<UUID>.zip`
  (SIP) — bố cục E-ARK CSIP y hệt gói DigiText tự tạo và gói của dự án Đầm Hà (cùng bộ dựng
  `digitext-package`, tự kiểm bằng `validate::f_2p7` trước khi ghi ra đĩa):
  - `METS.xml` gốc: 3–4 agent (phần mềm; cơ quan hình thành phông CREATOR; đơn vị đóng gói CREATOR
    nếu khai; cơ quan lưu trữ ARCHIVIST — có **tên** lẫn mã), SIP có `altRecordID SUBMISSIONAGREEMENT`;
    `xsi:schemaLocation` trỏ đủ vào `schemas/` (METS, xlink, CSIP, SIP).
  - `metadata/descriptive/EAD.xml`, `metadata/preservation/PREMIS.xml` (AIP): thực thể trí tuệ,
    biểu diễn, từng tệp (SHA-256, kích thước, định dạng + phiên bản PDF, tên gốc, tiêu đề, số trang,
    cơ quan ban hành, vị trí), sự kiện tạo gói + chuyển giao từng tệp, agent phần mềm + cơ quan.
  - `representations/rep1/`: `METS.xml` (dmdSec trỏ ngược EAD hồ sơ và từng `EAD_doc_*.xml`,
    structMap `TYPE="NORMALIZED"` gồm Metadata / Data / MetadataLink), `data/`, `metadata/`.
  - `schemas/`: `METS.xsd`, `xlink.xsd`, `DILCISExtensionMETS.xsd` (+ `DILCISExtensionSIPMETS.xsd`
    với SIP), `EAD*.xsd`, `PREMIS.xsd` — gói tự đứng, kiểm được khi không có mạng.
  - `documentation/UserManual.pdf` **sinh cho từng gói** như mẫu Đầm Hà: thông tin tổ chức (đơn vị
    tạo lập, cơ quan hình thành phông, cơ quan lưu trữ), phần mềm, ngày tạo gói, chuẩn áp dụng,
    thông tin hồ sơ (mã yêu cầu nộp, STT, tên ZIP, OBJID, tiêu đề, thời gian, ngôn ngữ, THBQ…),
    cấu trúc thư mục, danh sách văn bản. **Ngày giờ = đồng hồ và múi giờ của máy đang đóng gói**
    (cùng giá trị với METS CREATEDATE, PREMIS) — chỉnh ngày máy là chỉnh được ngày tạo gói.
    Cần font chữ Việt (`DIGITEXT_CERT_FONT` hoặc font hệ thống); đặt `DIGITEXT_PACKAGE_MANUAL`
    trỏ tới một PDF có sẵn thì dùng tệp đó thay cho bản sinh tự động.
- Tệp tài liệu phải mang dấu PDF/A (XMP `pdfaid` + OutputIntent); không có thì bộ dựng từ chối
  (`--allow-non-tt05` để bỏ qua khi chạy thử).
- Tệp trong gói được **đổi tên** thành `<mã hồ sơ>[.<mục lục số>].<STT 7 số>.pdf` (quy tắc
  `mã cơ quan.phông số.năm.số hồ sơ.mục lục số (nếu có).STT văn bản`; mục lục số lấy từ tên tệp
  gốc, vd `H49.101.01.1993.153.01.0000002.pdf` → hồ sơ 153, mục lục 01, STT 2);
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
