# Chạy thử đối soát sản phẩm trên dữ liệu Bắc Giang thật (4/11/2026)

Mục đích: kiểm màn **Kiểm tra sản phẩm** (`GET /cl/doi-soat`, R3) trên dữ liệu thật trước khi người dùng thao tác.
Không ghi gì lên máy chủ.

## Cách làm

- **Kho máy chủ:** câu SQL thật của `documents::list_for_project_qc` chạy bằng `psql` trong container Postgres trên
  OCR PC, dự án LTLSBNinh (205.007 tài liệu), xuất JSON. Thời gian câu SQL: 0,33 s; câu hồ sơ của một phông: 0,03 s.
- **Mục lục:** 23 tệp ở `test/MLHS` đọc bằng bộ nhập thật (`npm run kiem:mucluc`, biến `KIEM_XUAT_JSON=<thư mục>` ghi
  hồ sơ ra JSON). Mục lục Bắc Giang **chưa nhập** vào Dữ liệu chủ trên máy chủ, nên không gọi được route qua mạng.
- **Ghép:** hàm `doi_soat::ghep` của API (đúng mã chạy trên máy chủ), test bỏ qua mặc định:

```
DIGITEXT_THU_DOCS=<thu-docs.json> DIGITEXT_THU_CAP=<cap.json> \
  cargo test -p digitext-api --lib chay_that_json -- --ignored --nocapture
```

`cap.json` = `[{"hoSo": "<tệp JSON hồ sơ>", "thuMuc": "<thư mục phông trên máy chủ>"}]`. Mỗi phông ghép dưới 0,4 s.

## Lỗi tìm ra và đã sửa

1. **Mục lục không có cột hộp không ghép được tệp nào** (Sở Công nghiệp BG, Sở Lâm nghiệp HB; tệp trên máy chủ nằm
   trong `HOP 01/HO SO 01`). Sửa: bộ ghép dùng chung `tu_ocr::BoGhep` — khớp (hộp, hồ sơ) trước, rồi theo riêng số hồ sơ
   khi mục lục không kê hộp cho hồ sơ đó (hoặc tài liệu không nằm trong thư mục hộp). Áp cho cả Tạo văn bản từ OCR.
   Kết quả: Lâm nghiệp 0 → 140 hồ sơ có tệp, Công nghiệp BG 0 → 148, Ty VHTT Hà Bắc 264 → 281.
2. **Số "tệp không khớp" kẹt ở 2.000** vì đếm sau khi cắt danh sách. Sửa: đếm trước khi cắt (Sở GD&ĐT thật ra 13.133).

## Kết quả sau khi sửa

| Phông (tệp mục lục) | Hồ sơ | Có tệp | Đạt | Thiếu trang | Thừa trang | Chưa quét | Tệp không khớp |
|---|---:|---:|---:|---:|---:|---:|---:|
| UBND tỉnh Bắc Giang (1997 - 2017) | 881 | 862 | 691 | 166 | 5 | 19 | 617 |
| Sở Tài chính Vật giá Hà Bắc (1957-1996) | 2.727 | 1.841 | 1.554 | 234 | 53 | 886 | 2.232 |
| Sở Khoa học và Công nghệ Bắc Giang | 1.805 | 1.472 | 862 | 133 | 477 | 333 | 826 |
| Sở NN&PTNT Bắc Giang | 532 | 512 | 337 | 29 | 146 | 20 | 135 |
| Sở NN&PTNT Hà Bắc | 479 | 473 | 359 | 77 | 34 | 6 | 2 |
| Sở GD&ĐT 1997-2015 | 368 | 17 | 8 | 2 | 7 | 351 | 13.133 |
| Ty Văn hoá Thông tin Hà Bắc (vĩnh viễn) | 319 | 281 | 248 | 31 | 0 | 38 | 677 |
| Sở Thương mại và Du lịch Bắc Giang | 198 | 193 | 148 | 28 | 17 | 5 | 256 |
| Sở Thương nghiệp Hà Bắc | 639 | 543 | 477 | 44 | 22 | 96 | 113 |
| Ty Thuỷ lợi Hà Bắc | 649 | 78 | 21 | 32 | 25 | 571 | 4.043 |
| UBND tỉnh Bắc Giang (2005-2017) sổ 2 | 165 | 159 | 123 | 30 | 6 | 6 | 159 |
| Sở Lâm nghiệp Hà Bắc | 158 | 140 | 117 | 18 | 1 | 18 | 155 |
| Sở Công nghiệp Bắc Giang | 165 | 148 | 64 | 43 | 41 | 17 | 172 |
| Sở Tài chính Vật giá Bắc Giang (1997-2002) | 58 | 56 | 39 | 7 | 10 | 2 | 0 |
| Văn phòng UBND tỉnh Bắc Giang (2005-2017) | 82 | 69 | 60 | 2 | 7 | 13 | 1 |
| Sở Văn hoá Thông tin Bắc Giang | 161 | 160 | 144 | 15 | 1 | 1 | 26 |
| Sở Công Thương Bắc Giang (khai thác) | 46 | 46 | 37 | 2 | 7 | 0 | 0 |
| Sở Thương mại và Du lịch Hà Bắc | 119 | 107 | 93 | 8 | 6 | 12 | 3 |
| Ty Văn hoá Bắc Giang 1956-1962 | 150 | 147 | 137 | 8 | 2 | 3 | 20 |
| Chi cục Văn thư Lưu trữ Bắc Giang (2005-2020) | 52 | 42 | 21 | 17 | 4 | 10 | 164 |
| Ty Thuỷ sản Hà Bắc | 6 | 6 | 5 | 1 | 0 | 0 | 0 |
| Ty Chăn nuôi Thú y Hà Bắc | 3 | 3 | 3 | 0 | 0 | 0 | 0 |
| **Tổng** | **9.762** | **7.355** | **5.548** | **927** | **871** | **2.407** | **22.734** |

Sở Công nghiệp Hà Bắc (409 hồ sơ) không có thư mục trên máy chủ nên không có trong bảng.

## Cần người nghiệp vụ xem

- **Sở GD&ĐT:** bộ nhập chọn sheet "P 1956-1996" trong tệp tên "1997-2015", nên chỉ 17 hồ sơ có tệp. Khi nhập thật phải
  chọn đúng sheet.
- **Ty Thuỷ lợi Hà Bắc ↔ thư mục "PHONG SO THUY LOI HB":** chỉ 78/649 hồ sơ ghép được, 4.043 tệp lệch. Có thể là hai phông
  khác nhau (Ty và Sở), hoặc đánh số hồ sơ khác mục lục.
- **Sở KH&CN Bắc Giang:** 477 hồ sơ có số trang quét hơn gấp đôi số tờ. Cần xem cột "số tờ" của mục lục này có phải là số
  văn bản hay số trang không.
- **Thiếu trang (927 hồ sơ):** số trang quét ít hơn số tờ kê khai. Cần đối chiếu tay vài hồ sơ, ví dụ Ty Thuỷ sản hồ sơ 3
  (125 tờ, 121 trang).
- **Tệp không khớp:** phần lớn là hồ sơ có trên máy chủ mà không có trong mục lục (ví dụ UBND 2005-2017 hộp 1344 hồ sơ
  10000). Cần bổ sung mục lục hoặc sửa thư mục.
- **Máy chủ chưa có biên bản nào và chưa có gói AIP đã dựng**, nên chưa chạy thử được phần kiểm gói của bộ hồ sơ nghiệm
  thu trên dữ liệu thật.
