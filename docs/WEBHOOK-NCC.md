# Cửa đọc cho cổng nhà cung cấp

Tài liệu cho lập trình viên của hệ thống quản lý hợp đồng (CRM / DO365).

> **Chín cửa, một khoá.** Tất cả dùng chung khoá và chung cách ký ở mục 1 — đọc mục 1 một
> lần là viết được cả chín.
>
> | Mục | Cửa | Lấp mục menu |
> |---|---|---|
> | 1–8 | `POST /ncc/san-luong` | Báo cáo sản lượng |
> | 9 | `POST /ncc/tu-kiem/da-nop` | Tự kiểm chất lượng |
> | 10 | `POST /ncc/tu-kiem/muc-luc` | Tự kiểm mục lục |
> | 10.9 | `POST /ncc/tu-kiem/muc-luc/mau` | *(nút «Tải tệp mẫu» trong màn trên)* |
> | 10.10 | `POST /ncc/tu-kiem/muc-luc/bao-cao` | *(nút «Tải bảng Kiểm tra» trong màn trên)* |
> | 11 | `POST /ncc/tu-kiem/pdf` | Tự kiểm PDF/A |
> | 12 | `POST /ncc/tu-kiem/goi` | Tự kiểm gói tin |
> | 13 | `POST /ncc/tu-kiem/goi/dot` | *(nút «Kiểm cả đợt» trong màn trên)* |
> | 14 | `POST /ncc/tu-kiem/kho` | Tự kiểm kho trước khi đóng gói |
>
> Mục 12 **đổi quyết định 13/11/2026**: trước đây ghi là không làm cửa web vì gói cỡ GB. Nay cửa
> nhận **bản kê siêu dữ liệu** (0,247% cỡ gói) nên không phải tải gói lên nữa.

Đây là **chiều ngược lại** của [WEBHOOK-DU-AN.md](WEBHOOK-DU-AN.md). Ở đó bên bạn *đẩy*
dự án sang DigiText; ở đây bên bạn *hỏi* DigiText "dự án này, khoảng ngày này, đội này
làm được bao nhiêu" rồi tự vẽ trong giao diện của mình.

Mục tiêu nghiệp vụ: nhà thầu phụ mở cổng của bạn là thấy ngay sản lượng **đã làm thật**,
không phải gõ lại lần hai, và đến lúc nghiệm thu không còn cảnh hai bên cầm hai con số.

| | |
|---|---|
| Địa chỉ | `POST https://api.digitext.vn/api/v1/ncc/san-luong` |
| Kiểu dữ liệu | `application/json`, mã hoá UTF-8 |
| Xác thực | **đúng khoá bạn đang dùng cho webhook dự án** — không cấp khoá mới |

---

## 1. Xác thực — không có gì mới phải viết

Giống hệt webhook dự án: cùng khoá, cùng ba header, cùng chuỗi ký
`<timestamp> + "." + <thân gói>`. Hàm ký bạn đã viết dùng lại được nguyên vẹn.

| Header | Bắt buộc | Nội dung |
|---|---|---|
| `X-DigiText-Key` | luôn | khoá đã cấp, dạng `dk_xxxxxxxx.<bí mật>` |
| `X-DigiText-Timestamp` | khi khoá bật ký | giây Unix (UTC) |
| `X-DigiText-Signature` | khi khoá bật ký | `sha256=<HMAC-SHA256 hex>` |

> **Vì sao đường ĐỌC lại là `POST`.** Để tham số trong thân JSON thì chúng nằm trong vùng
> ký. Nếu dùng `GET` với query string thì query **nằm ngoài chữ ký**: ai bắt được một gói
> tin có thể đổi `maNgoai` sang dự án khác trong vòng 5 phút mà chữ ký vẫn hợp lệ, và mã
> dự án còn lọt vào nhật ký của mọi proxy trên đường. Đổi lại bên bạn được lợi: gọi thẳng
> bằng hàm `goi()` sẵn có, không viết thêm dòng ký nào.

Hai điều nên biết, nói thẳng để khỏi hiểu nhầm:

- Chữ ký **vẫn phát lại được trong 5 phút** — không có kho nonce. Với lệnh đọc thì vô hại,
  nhưng đừng tưởng đã chống phát lại.
- **Phạm vi đọc do khoá quyết định, không do thân gói.** Khoá của bạn chỉ thấy dự án do
  chính bạn tạo. Hỏi `maNgoai` của hệ thống khác trả về cùng một lỗi 404 như mã không tồn
  tại — cố ý không phân biệt.

---

## 2. Thân gói

```json
{
  "maNgoai": "PRJ-26-017",
  "doi": "so-hoa",
  "tuNgay": "2026-10-12",
  "denNgay": "2026-11-10"
}
```

| Trường | Bắt buộc | Ý nghĩa |
|---|---|---|
| `maNgoai` | **có** | mã dự án bên bạn — đúng mã đã gửi lúc tạo dự án |
| `doi` | không | `so-hoa` · `chinh-ly` · `tat-ca`. Không gửi = `tat-ca` |
| `tuNgay` | không | `YYYY-MM-DD`. Không gửi = `denNgay` lùi 29 ngày |
| `denNgay` | không | `YYYY-MM-DD`. Không gửi = hôm nay (giờ Việt Nam) |

`doi` nhận nhiều cách viết: `so-hoa`, `so_hoa`, `Số hoá`, `scan`… và `chinh-ly`,
`Chỉnh lý`… Không nhận ra thì trả 400 kèm danh sách giá trị hợp lệ.

Khoảng ngày tối đa **366 ngày** một lần hỏi. `tuNgay` sau `denNgay` → 400.

---

## 3. Đội của bạn ↔ chỉ số của DigiText

DigiText **không biết nhà thầu nào là ai** — nó không có bảng nhà thầu, không có tổ đội.
Nó chỉ biết dây chuyền nào chạy ra bao nhiêu. Nên `doi` được hiểu là **nhóm chỉ số**:

| `doi` | Chỉ số trả về (`ma` trong `chiSo[]`) |
|---|---|
| `so-hoa` | `trangQuet` · `tepDuaLen` · `trangOcrXong` · `tepPdfa` · `tepKySo` · `tepDuyetDat` · `tepTraLai` |
| `chinh-ly` | `hoSoMoi` · `hoSoSua` · `vanBanMoi` · `truongDaNhap` |
| `tat-ca` | cả 11 chỉ số trên |

Bên bạn biết (dự án, đội) đó giao cho nhà cung cấp nào — DigiText không cần biết, và
không bên nào phải giữ danh sách của bên kia.

> ⚠️ **Giới hạn phải nắm.** Nếu **hai nhà cung cấp cùng làm một đội trên một dự án** thì
> số trả về là số **gộp của cả hai**, không tách được. Phản hồi luôn mang cờ
> `"gopNhieuNhaThau": true` để nhắc điều này. Muốn tách thì bên bạn tạo **hai dự án với
> `maNgoai` khác nhau** khi đẩy sang — phía DigiText không phải sửa gì.

---

## 4. Phản hồi

```json
{
  "maNgoai": "PRJ-26-017",
  "maDuAn": "DA26007",
  "tenDuAn": "Chỉnh lý tài liệu UBND xã Việt Yên",
  "loaiDuAn": "chinh_ly_so_hoa",
  "dangHoatDong": true,
  "doi": "so_hoa",
  "tuNgay": "2026-11-01",
  "denNgay": "2026-11-10",

  "chiSo": [
    { "ma": "trangQuet", "ten": "Trang đã quét", "donVi": "trang" },
    { "ma": "tepDuaLen", "ten": "Tệp đưa lên", "donVi": "tệp" }
  ],
  "ngay": [
    { "ngay": "2026-11-01", "trangQuet": 0,     "tepDuaLen": 0 },
    { "ngay": "2026-11-03", "trangQuet": 11840, "tepDuaLen": 412 }
  ],
  "tong": { "trangQuet": 39960, "tepDuaLen": 1364 },

  "luyKe": { "tep": 12040, "trang": 398200, "tepPdfa": 11800, "tepKySo": 11010,
             "tepOcrXong": 11950, "tepDuyetDat": 10420, "tepTraLai": 122 },

  "hopDong": { "ngayKy": "2026-03-01", "ngayKetThuc": "2027-03-31", "hanChot": null,
               "soTrang": 300000, "soTep": null, "soHop": 1200, "soHoSo": 36000,
               "soTo": null, "nguoiPhuTrach": "Nguyễn Văn A", "nguon": "digitext" },

  "tienDo": { "phanTram": 42.5, "phanTramThoiGian": 55.0, "lech": -12.5,
              "conLaiNgay": 141, "duKienXongNgay": "2027-05-02",
              "congThuc": "tb-4-buoc-may-chu" },

  "capNhatLuc": "2026-11-10T03:12:00Z",
  "doTreToiDaGiay": 120,
  "gopNhieuNhaThau": true
}
```

**`chiSo[]` là bảng mô tả — hãy vẽ theo nó, đừng viết cứng tên cột.** Nhờ vậy một màn
hình dùng chung được cho cả hai đội, và sau này DigiText thêm chỉ số thì bên bạn không
phải phát hành lại phần mềm.

Những chỗ cần lưu ý:

| Trường | Điều phải biết |
|---|---|
| `ngay[]` | **luôn đủ mọi ngày trong khoảng**, ngày không ai làm thì các số bằng 0. Vẽ đồ thị không bị nhảy ngày |
| `hopDong` | **`null` khi chưa ai khai khối lượng hợp đồng.** Xem mục 5 |
| `tienDo` | **vắng mặt** khi không có mẫu số để tính. Đừng mặc định 0% |
| `tienDo.lech` | `phanTram − phanTramThoiGian`. Âm = chậm hơn tiến độ. Màu đèn do bên bạn tự quy định |
| `doTreToiDaGiay` | bảng tổng hợp chạy lại mỗi 120 giây ⇒ số của **hôm nay** có thể trễ tới chừng đó. Số của 35 ngày gần đây còn có thể đổi khi có người duyệt lại |
| `dangHoatDong` | `false` = dự án đã đóng. Khi đó `hopDong` là `null`, không có `tienDo`, và có thêm `ghiChu` giải thích |

Phản hồi **không bao giờ chứa** tên người dùng, năng suất cá nhân, định mức KPI hay ghi
chú nội bộ — cổng nhà cung cấp là người ngoài.

---

## 5. Màn hình phải chịu được hai trạng thái

Thanh **% tiến độ** chỉ vẽ được khi `hopDong.soTrang` (hoặc `soTep`) có giá trị. Mà khối
lượng ấy chỉ đến từ **chính bên bạn**, qua webhook dự án.

Thực tế đo ngày 10/11/2026 trên dữ liệu thật: trong **420 dự án lưu trữ, chỉ 8 dự án**
có khai số trang. Nghĩa là nếu dựng màn hình hôm nay thì gần như nhà cung cấp nào mở ra
cũng thấy thanh tiến độ trống.

Vậy nên:

1. **`hopDong != null`** → vẽ thanh %, so `phanTram` với `phanTramThoiGian`.
2. **`hopDong == null`** → bày số đếm và nói thẳng *"chưa khai khối lượng hợp đồng"*,
   kèm lối bấm sang màn khai khối lượng. Đừng vẽ thanh 0%.

Và có **hai việc phía bạn** quyết định màn này có ý nghĩa hay không — xin làm **đúng thứ
tự này**:

**① Sửa chỗ cộng dồn trong `khoiLuongCua()` trước.** Bảng khối lượng hợp đồng khai theo
**định mức dự toán**: một khối lượng bị chẻ thành nhiều dòng chi phí, mỗi dòng cùng khối
lượng nhưng khác đơn giá. Hàm hiện cộng mọi dòng cùng đơn vị nên đếm bội:

```
PRJ-26-1219   54.200 Trang A4  290.078.400  Chuẩn hóa, chuyển đổi và số hóa tài liệu
              54.200 Trang A4    6.233.000  Hao phí thiết bị, VPP phục vụ chuẩn hóa...
              → cộng ra 108.400, số đúng là 54.200

PRJ-26-1213    103,5 Mét       760.725.000  Chi phí nhân công chỉnh lý...
               103,5 Mét        33.637.500  Chi phí máy móc thiết bị...
               103,5 Mét         4.347.000  Chi phí công cụ dụng cụ...
               103,5 Mét       119.025.000  Chi phí vật tư văn phòng phẩm...
              → cộng ra 414, số đúng là 103,5
```

Cách lưu của bên bạn **không sai** — cộng cột tiền ra đúng giá trị hợp đồng. Sai nằm ở
khâu đọc. Quy tắc đề nghị: bỏ các dòng có tên bắt đầu bằng `Chi phí` / `Hao phí`; nếu bỏ
hết thì lấy **giá trị lớn nhất** trong các dòng còn lại cùng đơn vị, **đừng lấy tổng**.
Sạch hơn nữa: thêm cờ `laThanhPhanGia: true` vào dòng chi phí, khỏi phải đoán theo chữ.

Lưu ý không có quy tắc số học nào thay được việc này: `PRJ-26-1332` có hai dòng
`340.000` và `125.000` trang là **hai gói việc thật**, ở đó lấy tổng mới đúng.

**② Rồi mới nới bộ nhận đơn vị** trong `donVi()`: hiện `/^(trang)$/` không khớp
`"Trang A4"`, và `"Mét"` viết hoa cũng rơi. Làm ngược thứ tự thì số sai gấp 2–4 lần chảy
thẳng vào thanh % mà nhà cung cấp nhìn.

---

## 6. Mã lỗi

| Mã HTTP | `code` | Nghĩa | Nên làm gì |
|---|---|---|---|
| 200 | — | xong | |
| 400 | `bad_request` | thiếu `maNgoai`, `doi` không hiểu, ngày sai định dạng, khoảng quá dài | sửa gói tin, **đừng gọi lại y nguyên** |
| 401 | `hook_unauthorized` | thiếu/sai khoá, thiếu chữ ký, lệch giờ quá 5 phút | kiểm khoá và đồng hồ NTP |
| 403 | `hook_ip_forbidden` | IP không nằm trong danh sách cho phép của khoá | xin mở IP |
| 404 | `ncc_du_an_khong_thay` | không có dự án nào mang `maNgoai` đó | dự án chưa được đẩy sang, hoặc gõ sai mã |
| 429 | — | gửi sai khoá quá nhiều từ cùng một địa chỉ | chờ theo `Retry-After` |
| 5xx | — | máy chủ trục trặc | gọi lại, giãn dần như webhook dự án |

---

## 7. Ví dụ

**curl** — dựng chuỗi JSON trước rồi ký chính chuỗi đó:

```bash
BODY='{"maNgoai":"PRJ-26-017","doi":"so-hoa"}'
TS=$(date +%s)
SIG=$(printf '%s.%s' "$TS" "$BODY" | openssl dgst -sha256 -hmac "$HMAC_SECRET" -hex | awk '{print $2}')
curl -X POST https://api.digitext.vn/api/v1/ncc/san-luong \
  -H "Content-Type: application/json" \
  -H "X-DigiText-Key: $DIGITEXT_KEY" \
  -H "X-DigiText-Timestamp: $TS" \
  -H "X-DigiText-Signature: sha256=$SIG" \
  --data-raw "$BODY"
```

**TypeScript** — dùng lại nguyên hàm `goi()` đã có trong `digitext.ts`:

```ts
const r = await goi('/api/v1/ncc/san-luong', {
  maNgoai: prj.code,
  doi: quyen.doi,                 // 'so-hoa' | 'chinh-ly'
  tuNgay, denNgay,
});
if (!r.ok) return respondError(res, r.status === 404 ? 404 : 502, r.loi);
res.json(r.than);
```

---

## 8. Việc cần làm để nối xong

1. Thêm route `/ncc/san-luong` ở phía bạn: lấy `doi` từ `quyen` của `PortalUser` đang
   đăng nhập, lấy `maNgoai` từ dự án đang xem, gọi sang DigiText, trả thẳng kết quả.
   **Đừng cho phía trình duyệt tự chọn `maNgoai`** — lấy từ phiên, kẻo nhà cung cấp này
   xem được dự án của nhà cung cấp khác.
2. Thêm `'san-luong'` vào `type TrangNcc`.
3. Viết component màn hình: bảng theo `chiSo[]` + `ngay[]`, dòng cộng từ `tong`, khối
   tiến độ theo mục 5.
4. Nối vào `trangNgoai`.
5. Truyền handler thứ 4 cho `dongBen(..., 'chart', 'Báo cáo sản lượng', () => diTrang('san-luong'))`
   — nhãn «sắp có» tự biến mất.
6. Thêm dòng tương ứng ở menu «Thêm» cho điện thoại.
7. Sửa `khoiLuongCua()` và `donVi()` theo mục 5, **theo đúng thứ tự ① rồi ②**.
8. Thử: gọi `/api/v1/hooks/kiem-tra` không kèm header phải ra **401** (cửa sống); rồi gọi
   `/ncc/san-luong` với một dự án thật.

Vướng gì thì báo lại — phía DigiText sửa được trong ngày.

---

## 9. Cửa thứ hai — «Tự kiểm chất lượng»: lỗi máy chủ đã bắt

| | |
|---|---|
| Địa chỉ | `POST https://api.digitext.vn/api/v1/ncc/tu-kiem/da-nop` |
| Xác thực | y như mục 1 — **cùng khoá, cùng hàm `goi()`**, không có gì mới |

### 9.1. Nó trả lời câu gì

*"Sản phẩm tôi đã nộp, máy chủ đã bắt lỗi chỗ nào?"*

Hôm nay nhà thầu chỉ biết mình sai **sau khi** cả đợt bị trả lại. Ba con số dưới đây đã
nằm trong cơ sở dữ liệu của DigiText từ lúc tệp được tải lên và lúc người duyệt bấm nút —
chưa ai cho nhà thầu xem thôi.

**Cửa này KHÔNG tải tệp lên và KHÔNG chạy bộ kiểm nào.** Nó chỉ đọc. Nên nó nhanh, không
có hạn ngạch, gọi bao nhiêu lần cũng được.

### 9.2. Thân gói

```json
{ "maNgoai": "PRJ-26-017", "gioiHan": 200 }
```

| Trường | Bắt buộc | |
|---|---|---|
| `maNgoai` | **có** | Mã dự án bên bạn — đúng mã đã gửi khi tạo dự án |
| `gioiHan` | không | Số dòng tối đa trong `tep[]`. Mặc định 200, trần 1000. Hỏi quá thì **bị kẹp xuống, không báo lỗi** |

`maNgoai` **lấy từ phiên, đừng để trình duyệt tự chọn** — lý do như mục 8.

### 9.3. Phản hồi

```json
{
  "maNgoai": "PRJ-26-017",
  "maDuAn": "LT-2026-017",
  "tenDuAn": "Chỉnh lý, số hoá tài liệu Sở Nội vụ",
  "dangHoatDong": true,
  "tongTep": 1364,
  "loai": [
    { "ma": "tra_lai",        "ten": "Người duyệt trả lại",                      "soTep": 12 },
    { "ma": "pdfa_cu",        "ten": "Có PDF/A nhưng không đạt PDF/A-2b",        "soTep": 3  },
    { "ma": "boc_truong_loi", "ten": "Bóc trường bị lỗi",                        "soTep": 0  }
  ],
  "tep": [
    { "loai": "tra_lai", "ten": "0001.pdf", "phong": "Phòng Tổ chức",
      "hop": "Hộp 01", "hoSo": "HS.001", "soTrang": 12,
      "luc": "2026-11-09T03:14:22+00:00" }
  ],
  "biCat": false,
  "gioiHan": 200,
  "coLyDo": false
}
```

**Vẽ bằng `loai[]`, đừng viết cứng ba mã.** Khi DigiText thêm loại lỗi thứ tư, màn hình
của bạn tự có thêm dòng — không phải phát hành lại. Dùng `ten` làm nhãn, `ma` làm khoá.

| Trường | |
|---|---|
| `tongTep` | Tổng số tệp của dự án (đã trừ tệp tiêu huỷ) — để bày tỉ lệ, ví dụ "12/1364" |
| `loai[].soTep` | Số đếm **toàn dự án**, KHÔNG bị `gioiHan` cắt |
| `tep[]` | Danh sách tệp cụ thể, **mới duyệt trước**. Một tệp vướng hai loại thì ra hai dòng |
| `biCat` | `true` = còn nữa mà `gioiHan` cắt. Bày "còn nữa" và cho nâng `gioiHan` |
| `coLyDo` | `false` = phản hồi **không có** khoá `lyDo`. Xem 9.5 |

`soTrang` và `luc` có thể là `null` (tệp chưa đếm trang; loại lỗi không đi qua người duyệt).

### 9.4. Ba loại lỗi nghĩa là gì — và vì sao chỉ có ba

| Mã | Nghĩa | Nhà thầu phải làm gì |
|---|---|---|
| `tra_lai` | Người duyệt bên chủ đầu tư bấm **trả lại** tệp này | Sửa rồi nộp lại |
| `pdfa_cu` | Tệp **có** PDF/A nhưng không đạt **PDF/A-2b** — thường là bản PDF/A-1b đời cũ | Ghép lại PDF/A đúng chuẩn |
| `boc_truong_loi` | Tiến trình bóc trường trên máy chủ báo lỗi trên tệp này | Thường là ảnh quá mờ — quét lại trang đó |

Cố tình **không** có "thiếu lớp chữ", "chưa có PDF/A", "OCR chưa xong": tệp mới nộp đương
nhiên chưa qua các bước đó. Báo ra thì nhà thầu tưởng mình làm sai trong khi dây chuyền
chỉ đang chạy. Ba mã trên đều đọc từ cột có thật, không suy đoán.

### 9.5. ⚠️ `lyDo` — mặc định KHÔNG có

Phản hồi **không** mang nguyên văn ghi chú của người duyệt. Lý do: ô ghi chú đó bên
DigiText đang được viết với giả định *"chỉ người trong nhà đọc"* — người duyệt có thể gõ
câu không định cho nhà thầu thấy.

Nên màn hình của bạn phải chạy được ở **cả hai trạng thái**:

* `coLyDo: false` (hôm nay) — bày tên tệp + loại lỗi, kèm câu *"liên hệ quản lý dự án để
  biết lý do"*.
* `coLyDo: true` (nếu phía DigiText bật sau) — mỗi dòng `tra_lai` có thêm `lyDo`, bày ra.
  Các loại khác vẫn `lyDo: null`.

**Đọc `coLyDo` để quyết định, đừng dò xem khoá `lyDo` có tồn tại hay không** — tồn tại mà
`null` là chuyện bình thường.

### 9.6. Mã lỗi

Y như mục 6. Riêng cửa này thêm:

| HTTP | `code` | |
|---|---|---|
| 400 | `bad_request` | Thiếu `maNgoai`, hoặc thân gói không phải JSON |
| 404 | `ncc_du_an_khong_thay` | `maNgoai` không có, **hoặc** có nhưng thuộc nguồn khác — cùng một lỗi, cố ý |

Dự án **đã đóng** vẫn trả **200** kèm `dangHoatDong: false`, và vẫn đếm được lỗi. Không có
trường hợp nào ra 500.

### 9.7. Ví dụ

```bash
TS=$(date +%s)
BODY='{"maNgoai":"PRJ-26-017"}'
SIG=$(printf '%s.%s' "$TS" "$BODY" | openssl dgst -sha256 -hmac "$BIMAT" -hex | awk '{print $2}')
curl -s -X POST https://api.digitext.vn/api/v1/ncc/tu-kiem/da-nop \
  -H "Content-Type: application/json" \
  -H "X-DigiText-Key: $KEYID.$SECRET" \
  -H "X-DigiText-Timestamp: $TS" \
  -H "X-DigiText-Signature: sha256=$SIG" \
  -d "$BODY"
```

### 9.8. Việc cần làm để nối xong

1. Route `/ncc/tu-kiem` phía bạn: `maNgoai` **lấy từ phiên**, gọi sang DigiText, trả thẳng.
2. Thêm `'tu-kiem'` vào `type TrangNcc`.
3. Component màn hình: ba thẻ số từ `loai[]` (vẽ bằng vòng lặp, không viết cứng ba mã) +
   bảng `tep[]`. Không có lỗi nào thì bày trạng thái rỗng tử tế — *"máy chủ chưa phát hiện
   lỗi nào trên 1364 tệp đã nộp"* — chứ đừng bày bảng trắng.
4. Nối vào `trangNgoai`, rồi truyền handler cho
   `dongBen(..., 'shieldcheck', 'Tự kiểm chất lượng', () => diTrang('tu-kiem'))` — nhãn
   «sắp có» tự mất. Thêm dòng ở menu điện thoại.
5. Xử `biCat`: bày "còn nữa", cho nâng `gioiHan` (trần 1000).
6. Xử `coLyDo` theo 9.5 — **đây là chỗ dễ quên nhất**, vì hôm nay nó luôn `false` nên code
   không xử vẫn chạy, đến lúc DigiText bật mới lộ.

### 9.9. Chưa có trong cửa này — sẽ là cửa riêng, đừng chờ

* **Tự kiểm gói tin** (METS/PREMIS/EAD/TT05) và **kiểm PDF/A tệp lẻ** cho thứ **chưa nộp**:
  phải **tải tệp lên**, vì bộ kiểm là mã Rust chạy trên máy chủ, trình duyệt không chạy
  được. Sẽ có hạn ngạch. Cửa riêng, làm sau.
* **Tự kiểm mục lục hồ sơ (MLHS)**: bộ đọc tệp Excel là JavaScript, **chạy ngay trong
  trình duyệt** nhà thầu — tệp Excel không rời máy họ. Phía DigiText sẽ cấp bộ đọc đó kèm
  một cửa JSON nhỏ để kiểm trùng với hồ sơ đã có. Cũng là cửa riêng.

Nên trong thanh bên, **ba thứ này nên là ba mục menu khác nhau**, đừng nhồi một màn: cơ
chế khác nhau (có/không tải tệp lên), và người dùng cần biết tệp của mình có rời máy hay
không.

---

## 10. Cửa thứ ba — «Tự kiểm mục lục hồ sơ»: gửi tệp Excel lên, máy chủ đọc

| | |
|---|---|
| Địa chỉ | `POST https://api.digitext.vn/api/v1/ncc/tu-kiem/muc-luc` |
| Xác thực | y như mục 1 — **cùng khoá, cùng hàm `goi()`** |
| Trần tệp | 8 MiB (mục lục lớn nhất trong 23 tệp thật của Bắc Giang là 504 KiB) |
| Trần dòng | 5.000 dòng hồ sơ — đúng trần của lệnh nhập thật |

### 10.1. Nó trả lời câu gì

*"Tệp mục lục tôi sắp nộp có nhập được không, sai ở dòng nào?"*

Nhà thầu chỉnh lý làm xong mục lục, nộp, rồi bị trả cả tệp vì vài dòng sai. Cửa này cho họ biết
trước — **ngay trên cổng của bạn, không phải chờ ai mở DigiText**.

**Toàn bộ việc đọc và kiểm chạy ở máy chủ DigiText.** Bên bạn **không chứa một dòng luật nào**:
chỉ gửi tệp lên và bày kết quả. Bộ đọc là chính bộ đang chạy trong DigiText desktop, đã đối
chứng khớp **23/23 mục lục thật** và **10.171/10.171 dòng hồ sơ**.

### 10.2. Thân gói

```json
{
  "maNgoai": "PRJ-26-017",
  "tenTep": "Phông Sở Nội vụ.xlsx",
  "noiDung": "UEsDBBQABgAIAAAAIAA…",
  "sheet": "MLHS",
  "thbqMacDinh": "Vĩnh viễn",
  "chiTiet": true
}
```

| Trường | Bắt buộc | |
|---|---|---|
| `maNgoai` | **có** | Mã dự án bên bạn. **Lấy từ phiên**, đừng để trình duyệt tự chọn |
| `noiDung` | **có** | Nội dung tệp `.xlsx` / `.xls` / `.xlsm`, **mã hoá base64** |
| `tenTep` | không | Chỉ để hiện lại cho người dùng |
| `sheet` | không | Không gửi → máy chủ tự chọn sheet đầu tiên có dòng tiêu đề cột (sheet 1 thường là bìa). Gửi tên **không có trong tệp** → **400**, kèm danh sách sheet có thật |
| `thbqMacDinh` | không | Tên THBQ dùng cho ô THBQ trống. Không gửi → ô trống là **lỗi** từng dòng |
| `chiTiet` | không | `false` → không trả `dong[]`, chỉ trả số đếm. Mặc định `true` |

**Vì sao base64 trong JSON chứ không phải multipart**: chữ ký là HMAC trên `<ts>.<thân gói>`.
Để tệp trong JSON thì bạn dùng lại **y nguyên** hàm `goi()` đã viết — không thêm một dòng mã ký
nào. Giá phải trả là gói tin to thêm ~33%, với tệp vài trăm KiB thì không đáng kể.

### 10.3. Phản hồi

```json
{
  "maNgoai": "PRJ-26-017", "maDuAn": "LT-2026-017", "tenDuAn": "…",
  "tenTep": "Phông Sở Nội vụ.xlsx",
  "sheet": "MLHS In Chuẩn",
  "sheets": ["Bìa", "MLHS In Chuẩn", "Sheet2", "Danh mục"],
  "dongTieuDeCot": 7,
  "cot": ["hopSo", "hoSoSo", "tieuDe", "thoiGian", "soTo", "thbq", "ghiChu"],
  "soHoSo": 639, "soPhongBan": 1, "khongChiaPhongBan": true, "soHop": 42,
  "thbqTep": "Vĩnh viễn",
  "soLoi": 25, "soCanhBao": 150, "soDaDien": 310,
  "nhapDuoc": false,
  "vanDe": [],
  "dong": [
    { "dong": 134, "hoSoSo": "87", "hopSo": "6",
      "loi": ["Thiếu thời gian bắt đầu - kết thúc (ô trống, tiêu đề không ghi năm)"],
      "canhBao": [],
      "thoiGian": { "batDau": "", "ketThuc": "", "ketLuan": "loi", "ghiChu": [] } }
  ],
  "biCat": false,
  "trungVoiKho": [{ "phong": "Phông Sở Nội vụ", "soTrung": 3 }],
  "phong": {
    "trongTep": "Phông Sở Nội vụ",
    "khop": "Phông Sở Nội vụ",
    "soTrung": 3,
    "soPhongTrongDuAn": 2
  },
  "thbqDanhMuc": [
    { "ten": "Vĩnh viễn", "vinhVien": true },
    { "ten": "70 năm", "vinhVien": false }
  ]
}
```

| Trường | |
|---|---|
| **`nhapDuoc`** | **Con số duy nhất cần bày to.** `false` = nộp lên sẽ bị từ chối |
| `soLoi` · `soCanhBao` | Lỗi **chặn** nhập · cảnh báo (vẫn nhập được) |
| `soDaDien` | Số dòng máy chủ **tự suy** được thời gian từ tiêu đề — nên nói ra, kẻo người ta tưởng tệp của mình đã ghi đủ |
| `vanDe[]` | Vấn đề **cấu trúc** (`{dong, muc, noiDung}`, `muc` = `loi` \| `canh_bao`). `dong: 0` = cả tệp, ví dụ không tìm thấy dòng tiêu đề cột |
| `dong[]` | Từng dòng hồ sơ có vấn đề — **không có tiêu đề hồ sơ**, cố ý. Xem 10.3b |
| `biCat` | `true` = còn dòng nữa, danh sách bị cắt ở 1.000 |
| **`trungVoiKho[]`** | **Phần không trình duyệt nào làm được** — xem 10.4 |
| `sheets[]` · `sheet` | Cho người dùng đổi sheet nếu máy chọn sai; gửi lại với `sheet`. **Bắt buộc phải bày** — xem 10.4b |
| `phong` | Phông ghi trong tệp và việc đối chiếu với phông đã mở — xem 10.4 |
| **`thbqDanhMuc[]`** | Danh mục thời hạn bảo quản **của chính dự án này**. Dùng nó để đổ ô «THBQ khi ô trống» — **đừng gõ cứng danh sách bên bạn** |

### 10.3b. `dong[]` — và vì sao `canhBao` gộp cả ghi chú thời gian

Mỗi phần tử:

| Khoá | |
|---|---|
| `dong` · `hoSoSo` · `hopSo` | Định vị dòng trong tệp. **Không trả tiêu đề hồ sơ** — cố ý |
| `loi[]` | Lỗi chặn nhập của riêng dòng đó |
| `canhBao[]` | Cảnh báo của dòng, **đã gộp sẵn** ghi chú thời gian (xem dưới) |
| `thoiGian` | `{batDau, ketThuc, ketLuan, ghiChu[]}` — `ketLuan` ∈ `dung` · `da_dien` · `canh_bao` · `loi` |

**Bất biến đáng tin:** số dòng có `canhBao` không rỗng **bằng đúng** `soCanhBao`, và số dòng có
`loi` không rỗng bằng đúng phần lỗi-theo-dòng của `soLoi` (phần còn lại của `soLoi` là lỗi **cấu
trúc**, nằm ở `vanDe[]`). Cứ bày thẳng, không phải tự đếm lại.

Vì sao phải nói ra điều đó: một loại cảnh báo rất phổ biến — **kết luận thời gian** — không nằm
trong `canh_bao` của dòng mà nằm trong `thoiGian.ghiChu` ("năm 1700 ngoài dải 1800–2100",
"ngày kết thúc trước ngày bắt đầu"…). Bản cửa ngày 13/11/2026 đếm chúng vào `soCanhBao` nhưng
không đưa vào `dong[]`, nên màn hình hiện **«Cảnh báo 59»** rồi mở ra **không có dòng nào**.
Nay `canhBao` đã gộp sẵn, nên bày `canhBao` là đủ; `thoiGian` để riêng cho bên nào muốn hiện
nhãn kết luận hoặc ngày máy tự suy.

`biCat: true` nghĩa là danh sách dừng ở 1.000 dòng, còn nữa.

### 10.4. `trungVoiKho` — chỗ quan trọng nhất, và dễ bỏ qua nhất

Tệp Excel **tự nó không thể biết** hồ sơ số của nó có trùng với hồ sơ nhà thầu đã nộp đợt trước
hay không. Đó lại là lý do bị trả lại phổ biến nhất. Cửa này đối chiếu với kho thật.

Khoá duy nhất của hệ thống là `(phông, loại bảo quản, số tạm)` — **theo phông, không theo dự
án**. Một dự án có thể nhiều phông, mà cổng không biết tệp này sẽ nhập vào phông nào, nên máy
chủ đếm cho **từng phông**:

```json
"trungVoiKho": [
  { "phong": "Phông Sở Nội vụ", "soTrung": 3 },
  { "phong": "Phông Văn phòng", "soTrung": 0 }
]
```

Đọc là: *"nếu nhập vào Phông Sở Nội vụ thì 3 hồ sơ bị trùng"*. Mảng **rỗng** = không trùng ở
phông nào. Bày nguyên văn như vậy, đừng cộng lại thành một số — cộng lại thì vô nghĩa.

**Nhưng thường không phải bày cả bảng.** Tệp mục lục có ghi `Phông: …` ở phần đầu; máy chủ bóc
ra và đối chiếu với các phông đã mở trong dự án, trả về trong khối `phong`:

| Khoá | |
|---|---|
| `trongTep` | Tên phông ghi trong tệp. `""` = tệp không ghi |
| `khop` | Tên phông trong DigiText khớp với nó. `null` = không khớp hoặc tệp không ghi |
| `soTrung` | Khi `khop` có: số hồ sơ trùng **của đúng phông đó**. `null` khi không khớp |
| `soPhongTrongDuAn` | Số phông đã mở trong dự án |

Bốn trạng thái, bày khác nhau:

| | Bày |
|---|---|
| `khop` có tên | *«Đối chiếu với phông **X** ghi trong tệp: **3 hồ sơ** trùng số tạm với kho.»* Một câu, một số |
| `trongTep` có, `khop` null | Bảng `trungVoiKho` **và** cảnh báo: *«Tệp ghi phông "X" nhưng dự án chưa có phông tên này — kiểm lại tên phông, hoặc báo cán bộ phụ trách.»* |
| `trongTep` rỗng | Bảng `trungVoiKho` như trên. Có thể gợi ý thêm dòng `Phông: …` vào tệp |
| `soPhongTrongDuAn: 0` | *«Dự án chưa mở phông nào — chưa đối chiếu được với kho. Hai tầng kiểm còn lại vẫn chạy đủ.»* Không phải lỗi |

**Nhà thầu không tạo phông, và cũng không chọn phông.** Phông do cán bộ lưu trữ mở khi khởi tạo
dự án, và chọn lúc nhập mục lục vào hệ thống. Tên phông trong tệp chỉ để **đối chiếu** — đúng như
trong phần mềm desktop của DigiText, nơi cùng tên ấy chỉ dùng để cảnh báo lệch phông. Đừng dựng ô
«tạo phông» hay «chọn phông» ở cổng: một mã phông bị tách làm nhiều phông vì tên viết khác nhau
là mất luôn phép chống trùng số tạm, vốn dựa trên phông.

Phép so tên: hạ chữ thường, nén khoảng trắng thừa. **Dấu tiếng Việt vẫn tính** — "Phong" và
"Phông" là hai phông khác nhau.

### 10.4b. Chọn sheet — bỏ ô này là màn hình nói dối

Không gửi `sheet` thì máy chủ đọc **sheet đầu tiên tìm thấy dòng tiêu đề cột**. Mục lục thật
thường có sheet bìa, sheet "Hướng dẫn", hoặc mỗi phông một sheet. Khi ấy nhà thầu kéo tệp vào,
màn báo «nhập được, 0 lỗi» — mà đó là số của sheet khác. Họ nộp, rồi bị trả lại.

Sau lần kiểm đầu, nếu `sheets.length > 1` thì bày *«Đã đọc sheet **X**»* kèm ô chọn từ `sheets[]`.
Đổi sheet thì gọi lại cửa với **cùng base64 đang giữ trong bộ nhớ** và `sheet` mới — đừng bắt
người ta chọn tệp lần hai.

Xin `sheet` không có trong tệp thì cửa trả **400 kèm danh sách sheet thật**, chứ không im lặng
đọc sheet khác.

### 10.5. Mã lỗi

| HTTP | | |
|---|---|---|
| 400 | `bad_request` | Thiếu `maNgoai` / `noiDung` · `noiDung` không phải base64 · không phải tệp Excel · tệp rỗng · quá 8 MiB · quá 5.000 dòng · `sheet` không có trong tệp |
| 404 | `ncc_du_an_khong_thay` | `maNgoai` không có, hoặc thuộc nguồn khác |
| 401 | `hook_unauthorized` | Khoá / chữ ký / lệch giờ |

**Danh mục thời hạn bảo quản**: máy chủ cần nó để khớp cột THBQ, và **luôn có sẵn** — DigiText
gieo bộ 14 thời hạn theo TT 05 cho mọi dự án; dự án nào tự thêm thời hạn riêng thì bộ của nó là
14 dòng chung **cộng** dòng riêng. Nhà thầu không phải làm gì, bên bạn không phải kiểm gì.

### 10.6. Việc cần làm để nối xong

1. Route `/ncc/tu-kiem-muc-luc` phía bạn: nhận tệp từ `<input type="file">`, đọc thành base64,
   `maNgoai` **lấy từ phiên**, gọi sang DigiText, trả thẳng kết quả.
2. Thêm `'tu-kiem-muc-luc'` vào `type TrangNcc`, viết component, nối `trangNgoai`, truyền handler
   cho một **dòng menu riêng** (xem 10.7), thêm dòng ở menu điện thoại.
3. Màn hình: ô kéo-thả tệp → một thẻ kết luận lớn theo `nhapDuoc` → ba số `soLoi` / `soCanhBao` /
   `soDaDien` → khối `trungVoiKho` → bảng `dong[]`.
4. Chặn ở phía bạn **trước khi gửi**: chỉ nhận `.xlsx` / `.xls` / `.xlsm`, và tệp > 8 MiB thì báo
   ngay, đừng để người ta chờ tải lên xong mới nhận 400.
5. Xử `biCat`, **xử `sheets[]` (bắt buộc — xem 10.4b)**, và xử trường hợp `vanDe[]` có `dong: 0`.
5b. Ô «THBQ khi ô trống»: đổ từ `thbqDanhMuc[]` của lần kiểm trước, **không gõ cứng**. Lần kiểm
   đầu chưa có danh sách thì để ô đó là «— coi là lỗi —» và chỉ bật sau khi có kết quả; hoặc gọi
   trước cửa tệp mẫu (10.9), nó cũng trả `thbqDanhMuc`.
5c. Khối `phong`: bốn trạng thái ở 10.4, mỗi trạng thái một câu khác nhau.
5d. Nút **«Tải tệp mẫu»** ngay cạnh ô kéo-thả — xem 10.9.
6. **Nói rõ cho người dùng là tệp được gửi lên máy chủ DigiText để kiểm** — họ đang đưa dữ liệu
   nghiệp vụ đi, phải biết nó đi đâu.

### 10.7. Năm mục menu, không nhồi một màn

Năm phép kiểm có cơ chế khác nhau, nên để riêng:

| Mục menu | Cửa | Có tải tệp lên không |
|---|---|---|
| Báo cáo sản lượng | `/ncc/san-luong` | không |
| Tự kiểm chất lượng | `/ncc/tu-kiem/da-nop` | không |
| Tự kiểm mục lục | `/ncc/tu-kiem/muc-luc` | **có** |
| Tự kiểm PDF/A | `/ncc/tu-kiem/pdf` | **có** |
| Tự kiểm gói tin | `/ncc/tu-kiem/goi` | **không** — chỉ gửi bản kê, xem mục 12 |

Người dùng cần biết tệp của mình có rời máy hay không. Nhồi cả năm vào một màn thì họ không biết,
và lúc hỏng thì năm chỗ hỏng khác nhau trông giống nhau.

### 10.8. Hai cửa từng ghi "làm sau" ở đây

**Kiểm PDF/A tệp lẻ** đã làm xong — mục 11. **Kiểm gói tin** (METS/PREMIS/EAD/TT05/CSIP) nay **đã có** —
mục 12, theo cách không phải tải gói lên.

### 10.9. Tệp mẫu và hướng dẫn điền — `POST /api/v1/ncc/tu-kiem/muc-luc/mau`

Nhà thầu không đoán được DigiText nhận cột tên gì. Cửa này phát **tệp mẫu `.xlsx`** do máy chủ
sinh ra từ **chính bảng nhận cột của bộ đọc** — mẫu không thể lệch khỏi thứ bộ đọc chấp nhận, và
có một test bắt điều đó mỗi lần dựng.

Thân gói chỉ có một trường:

```json
{ "maNgoai": "PRJ-26-017" }
```

Phản hồi:

```json
{
  "maNgoai": "PRJ-26-017", "maDuAn": "LT-2026-017", "tenDuAn": "…",
  "tenTep": "mau-muc-luc-ho-so-LT-2026-017.xlsx",
  "kieu": "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
  "noiDung": "UEsDBBQ…",
  "cot": [
    { "khoa": "hoSoSo", "nhan": "Hồ sơ số", "cungNhan": ["Số HS", "Số tạm", "Hồ sơ mới", "Mã hồ sơ", "ĐVBQ"],
      "batBuoc": true, "kieu": "chữ hoặc số", "giaiThich": "Số tạm của hồ sơ. Không được trùng…" }
  ],
  "huongDan": [{ "tieuDe": "Dòng tiêu đề cột", "y": ["Máy chủ tự tìm dòng tiêu đề…"] }],
  "thbqDanhMuc": [{ "ten": "Vĩnh viễn", "vinhVien": true }],
  "phongs": ["Phông Sở Nội vụ", "Phông Văn phòng"],
  "gioiHan": { "mib": 8, "dong": 5000 }
}
```

| Trường | |
|---|---|
| `noiDung` | Tệp `.xlsx` mã hoá base64. Giải mã rồi cho tải về với tên `tenTep` |
| `cot[]` | Bảng cột để bày ngay trên màn: tên cột, có bắt buộc không, kiểu, **các cách viết cũng được nhận** |
| `huongDan[]` | `{tieuDe, y[]}` — bày thành các mục gập được. **Đừng chép thành HTML tĩnh bên bạn**: luật MLHS đã sửa 4 lần trong 5 tuần, bản chép tay sẽ dạy sai |
| `thbqDanhMuc[]` | Dùng đổ ô «THBQ khi ô trống» ngay từ đầu, không phải chờ kiểm lần một |
| `phongs[]` | Các phông đã mở trong dự án. Bày để nhà thầu biết ghi `Phông: …` thế nào cho khớp |
| `gioiHan` | Trần tệp của cửa 10. Dùng để chặn phía bạn trước khi gửi |

Tệp mẫu có 3 sheet: **«Mục lục hồ sơ»** (dòng `Phông: …`, dòng tiêu đề cột, 5 dòng ví dụ gồm cả
dòng phòng ban `I. …` và dòng mảng `1. …`), **«Danh mục THBQ»** (danh mục thật của dự án), và
**«Hướng dẫn»**.

Hai điều đáng nói với người dùng:

- Mẫu đã điền sẵn 5 dòng ví dụ. Nộp nguyên mẫu lên cửa 10 thì **qua sạch, 0 lỗi 0 cảnh báo** —
  dùng để họ thử đường đi trước khi đụng vào tệp thật.
- Mẫu này **khác** «mẫu lập hồ sơ» của phần mềm desktop. Mẫu lập hồ sơ đọc theo **vị trí cột**,
  14 cột cố định; mục lục hồ sơ đọc theo **tên cột**, phòng ban/mảng nằm trong cột Tiêu đề. Đưa
  nhầm thì bộ đọc vẫn nhận dòng tiêu đề nhưng đẻ ra hồ sơ rác từ dòng số thứ tự.

Mã lỗi: như 10.5, chỉ có 400 (thiếu `maNgoai`), 404 (`ncc_du_an_khong_thay`), 401.

---

### 10.10. Trả tệp về cho nhà thầu sửa — hai đường, đừng lẫn

Soát xong mà chỉ bày lỗi trên màn thì người sửa phải vừa nhìn màn vừa dò tệp. Như bản desktop
của DigiText, có **hai** tệp tải về, khác nhau về bản chất:

| | Ai dựng | Là gì |
|---|---|---|
| **Bảng Kiểm tra** `.xlsx` | **Máy chủ** — cửa `muc-luc/bao-cao` | Bảng mới: mỗi hồ sơ một dòng, kết luận + lý do |
| **Tệp gốc đã điền** `<tên>_DADIEN.xlsx` | **Cổng**, trong trình duyệt | Chính tệp nhà thầu gửi, giữ nguyên bố cục, điền sẵn + tô màu |

Cái thứ hai phải do cổng dựng: nó **ghi đè lên chính tệp của nhà thầu**, giữ nguyên định dạng,
cột thừa, ô gộp, logo… Máy chủ không giữ tệp gốc và không dựng lại được bố cục đó.

#### a) `POST /api/v1/ncc/tu-kiem/muc-luc/bao-cao` — bảng Kiểm tra

Thân gói **y hệt** `tu-kiem/muc-luc` (gửi lại chính tệp đó). Trả:

```json
{ "tenTep": "kiem-muc-luc-Book2.xlsx",
  "kieu": "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
  "noiDung": "UEsDBB…",
  "soHoSo": 288, "soLoi": 9, "soCanhBao": 136, "nhapDuoc": false }
```

Giải base64 rồi cho tải. Không phải dựng gì thêm.

#### b) Tệp gốc đã điền — `ketXuat: true`

Gửi `"ketXuat": true` kèm thân gói của `tu-kiem/muc-luc`. Phản hồi có thêm hai khối:

```json
"cotViTri": { "hoSoSo": 1, "tieuDe": 2, "thoiGian": 3, "soTo": 4, "thbq": 5 },
"dongDayDu": [
  { "dong": 308, "hoSoSo": "K39", "thoiGianGoc": "",
    "thoiGian": { "batDau": "01/01/2019", "ketThuc": "31/12/2019",
                  "ketLuan": "da_dien", "ghiChu": ["Ô trống — điền theo năm trong tiêu đề"] },
    "loi": [ { "noiDung": "Thiếu thời hạn bảo quản…", "cot": "thbq" } ],
    "canhBao": [] }
]
```

| | |
|---|---|
| `cotViTri` | Chỉ số cột **0-based** trong sheet đã đọc. Cần nó mới ghi đúng ô — `cot` (danh sách tên) chỉ đủ để bày "máy nhận ra mấy cột" |
| `dongDayDu` | **MỌI** dòng hồ sơ, kể cả dòng sạch. Dòng sạch mới là dòng máy điền được thời gian |
| `…loi[].cot` · `…canhBao[].cot` | Ô phải tô, do máy chủ quyết. **Đừng đoán theo câu chữ** — đổi một câu thông báo là cổng tô sai ô mà không ai biết |

> ⚠️ **`canhBao` trong `dongDayDu` là bản THÔ**, khác `dong[]` của màn hình: ở đây nó **không**
> gộp `thoiGian.ghiChu`. Gộp vào là ghi chú thời gian bị tô sang ô khác, trong khi nó phải tô
> vào chính ô thời gian. (Màn hình thì ngược lại — xem 10.3b.)

#### Cách ghi — làm đúng như bản desktop

Mã mẫu: `desktop/src/features/chinhly/mucLucXuat.ts`, hàm `danhDauTepGoc`. Dùng **ExcelJS**
(`wb.xlsx.load` tệp gốc → sửa ô → `writeBuffer`). Tóm tắt luật:

1. **Điền thời gian.** Ghép `"{batDau} {ketThuc}"` vào ô `cotViTri.thoiGian`; tệp tách hai cột
   thì ghi vào `ngayBatDau` / `ngayKetThuc`.
   **Ô đã đúng sẵn thì ĐỪNG ghi đè** — chỉ khác cách viết (`7/6` ↔ `07/06`, xuống dòng ↔ dấu
   cách) không phải là sai. So bằng cách rút mọi `d/m/yyyy` khỏi `thoiGianGoc`; còn đúng 2 ngày,
   đúng thứ tự, không còn chữ thừa ⇒ giữ nguyên, không tô.
2. **Ba màu, không hơn.** `C6EFCE` xanh = đã sửa · `FFFF00` vàng = cần xem lại ·
   `FF0000` đỏ + chữ trắng đậm = lỗi phải sửa.
3. **Ghi chú vào ô** (`cell.note`) nói rõ *Cũ → Mới* và lý do. Ô bị nhiều vấn đề thì **nối**
   ghi chú, đừng ghi đè.
4. **Ô trong vùng gộp**: ghi vào ô gốc (trên-trái), tức `cell.master ?? cell`.
5. ⚠️ **ExcelJS dùng chung một đối tượng style cho các ô cùng định dạng.** Gán thẳng `cell.fill`
   sẽ tô cả loạt ô khác (cả cột Tiêu đề đỏ au). Phải thay `cell.style` bằng **bản sao** rồi mới
   đặt `fill`.
6. Chỉ làm được với `.xlsx`/`.xlsm`. Tệp `.xls` thì báo người dùng lưu lại dạng `.xlsx`.

Đặt tên tệp tải về `<tên gốc>_DADIEN.xlsx` — trùng quy ước desktop, người dùng hai bên gọi
cùng một tên.

## 11. Cửa thứ tư — «Tự kiểm PDF/A»: một tệp PDF mẫu

| | |
|---|---|
| Địa chỉ | `POST https://api.digitext.vn/api/v1/ncc/tu-kiem/pdf` |
| Xác thực | y như mục 1 |
| Trần tệp | 32 MiB, **một tệp mỗi lời gọi** |

### 11.1. Nó trả lời câu gì

*"Tệp PDF tôi làm ra có đạt Điều 8 khoản 2 điểm a TT 05/2025 không?"*

Giá trị thật nằm ở chỗ kiểm **một tệp mẫu trước khi sản xuất cả lô**. Sai cách ghép PDF/A thì
30.000 tệp cùng sai, và biết ra sau khi nộp là làm lại từ đầu.

### 11.2. Thân gói

```json
{ "maNgoai": "PRJ-26-017", "tenTep": "0001.pdf", "noiDung": "JVBERi0xLjcK…",
  "loaiTaiLieu": "Bản đồ địa chính", "trichYeu": "…" }
```

| Trường | Bắt buộc | |
|---|---|---|
| `maNgoai` · `noiDung` | **có** | `noiDung` = tệp PDF mã hoá base64 |
| `tenTep` | không | Chỉ để hiện lại |
| `loaiTaiLieu` | không | Ghi bản đồ/bản vẽ/sơ đồ… → **ngưỡng dpi đổi 200 → 300** (TT05 Điều 8.2.a) |
| `trichYeu` | không | Chỉ sinh dấu hiệu **nghi**, ngưỡng **vẫn 200** |

### 11.3. Phản hồi

```json
{
  "tenTep": "0001.pdf", "soByte": 1248133,
  "soTrang": 8, "soTrangCoLopChu": 0,
  "dpiNhoNhat": 256, "doSauBitNhoNhat": 24,
  "tuKhaiPdfa": null, "coHoSoMau": false, "laPdfa": false,
  "daKySo": false, "canhDaiNhatCm": 29.7,
  "laBanVe": false, "liDoXepLoai": null, "dpiToiThieuCanCo": 200,
  "datPdfa2b": false,
  "chuaDat": [
    { "ma": "PDF-KHONG-PHAI-PDFA",  "noiDung": "tệp KHÔNG phải PDF/A (không khai pdfaid trong XMP) — TT05 Điều 8.2.a…" },
    { "ma": "PDF-THIEU-LOP-CHU",    "noiDung": "8/8 trang không có lớp chữ — TT05 Điều 8.2.a đòi PDF/A HAI LỚP" }
  ],
  "datChuan": false,
  "chuaKySo": true,
  "tepLoi": null,
  "ghiChu": "Kiểm dấu hiệu cấu trúc theo TT 05/2025 Điều 8.2.a. KHÔNG chạy veraPDF…"
}
```

| Trường | |
|---|---|
| **`datChuan`** | Con số cần bày to. `true` ⇔ `chuaDat[]` rỗng |
| `chuaDat[]` | `{ma, noiDung}` — **bày `noiDung` nguyên văn**, mỗi câu đã có dẫn chiếu điều khoản. `ma` để bạn nhóm/dịch nếu muốn |
| **`chuaKySo`** | Tách riêng, **không** nằm trong `chuaDat[]` — xem 11.4 |
| `datPdfa2b` | Đúng cái dấu máy chủ ghi vào tệp lúc tải lên. `false` là nguyên nhân của loại lỗi `pdfa_cu` ở mục 9 |
| `dpiToiThieuCanCo` | 200 hoặc 300, theo `laBanVe` |
| `tepLoi` | Khác `null` = tệp bị mã hoá hoặc hỏng, các số khác không tin được |

Mã lỗi hiện có: `PDF-KHONG-PHAI-PDFA` · `PDF-LOAI-PDFA-LA` · `PDF-THIEU-HO-SO-MAU` ·
`PDF-THIEU-LOP-CHU` · `PDF-DPI-THAP` · `PDF-KHONG-MAU-24BIT`. **Đừng viết cứng danh sách này** —
lặp trên `chuaDat[]` và bày `noiDung`.

### 11.4. Vì sao `chuaKySo` tách riêng

TT 05 đòi tệp có chữ ký số, nhưng trong dây chuyền DigiText **ký số làm sau khi nhà thầu giao sản
phẩm**. Nếu trộn vào `chuaDat[]` thì mọi tệp của nhà thầu đều "chưa đạt" vì một lý do không phải
việc của họ. Bày nó như một dòng thông tin ("sẽ ký sau khi nghiệm thu"), không như một lỗi — trừ
khi hợp đồng giao cho nhà thầu ký.

### 11.5. ⚠️ Giới hạn phải nói với người dùng

Cửa này **không chạy veraPDF** — bộ kiểm hợp chuẩn ISO 19005 đầy đủ cần Docker trên máy chủ, chưa
dựng. Ở đây là **kiểm dấu hiệu cấu trúc**: đủ để bắt mọi lỗi hay gặp (không khai pdfaid, thiếu hồ
sơ màu, thiếu lớp chữ, dpi thấp, màu dưới 24 bit), nhưng `datChuan: true` **chưa phải** kết luận
"đạt ISO 19005". Bày `ghiChu` ra, đừng bỏ.

### 11.6. Việc cần làm

1. Route phía bạn; `maNgoai` lấy từ phiên.
2. Màn: ô chọn **một** tệp → thẻ kết luận theo `datChuan` → danh sách `chuaDat[]` → dòng
   `chuaKySo` nếu `true` → khối số (trang, lớp chữ, dpi, loại PDF/A tự khai) → `ghiChu`.
3. Chặn ở phía bạn: chỉ `.pdf`, > 32 MiB báo ngay.
4. Nói rõ tệp được gửi lên máy chủ DigiText để kiểm.

---

## 12. Cửa thứ năm — «Tự kiểm gói tin»: gửi bản kê, không gửi gói

> **Đổi quyết định 13/11/2026.** Bản trước của tài liệu này nói sẽ **không** làm cửa web cho
> việc kiểm gói, vì gói AIP cỡ hàng trăm MB đến vài GB. Lý do đó chỉ đúng nếu phải tải cả gói
> lên. Đo trên gói AIP thật: **131 MB nội dung / 326 KB siêu dữ liệu — 0,247%**. Phần luật kiểm
> nặng nhất chỉ đọc mấy tệp XML đó, nên cửa này nhận **bản kê** chứ không nhận gói.

### 12.1. Nó trả lời câu gì

*"Gói tôi sắp giao có đúng cấu trúc không — METS/PREMIS/EAD có đủ, có khớp nhau, có tham chiếu
tới tệp không tồn tại không?"*

Chạy **đủ** phần kiểm cấu trúc, y như bản desktop:

- METS gốc + METS từng representation: hồ sơ bắt buộc, `PROFILE`, `OBJID`, `metsHdr`
- PREMIS: đối tượng, sự kiện, `originalName`, `formatName`
- EAD: trường bắt buộc, mã cơ quan, **định dạng ngày `dd/mm/yyyy`**
- TT02 / lược đồ `schemas/*.xsd`
- `href` có trỏ tới tệp **có thật trong gói** không · tệp **mồ côi** không ai tham chiếu
- `SIZE` trong METS có khớp cỡ tệp thật không — **chạy được**, vì bản kê có cỡ tệp
- Đối chiếu EAD ↔ METS

### 12.2. Ba phép kiểm **không** chạy, và tài liệu nói thẳng

Ba dòng dưới đây **cũng chính là** `chuaKiem[]` trong phản hồi — đọc từ đó, đừng chép vào mã cổng:

| Không chạy | Vì | Thay bằng |
|---|---|---|
| PDF/A từng tệp (lớp chữ, dpi, chữ ký) | cần byte thật của PDF | cửa `tu-kiem/pdf` (mục 11) cho tệp mẫu |
| Mã băm SHA-256 trong METS | cần byte thật | bản kiểm đầy đủ lúc nộp gói |
| Mã băm trong PREMIS | cần byte thật | bản kiểm đầy đủ lúc nộp gói |

Hệ quả **phải nói với người dùng**: gói qua được cửa này **vẫn có thể bị từ chối** lúc giao.
Trên gói mẫu thật, bản đầy đủ kết luận **KHÔNG ĐẠT** vì 7/7 tệp không phải PDF/A — bản rút gọn
không thấy điều đó.

Vì vậy `ketLuan` của cửa này **không bao giờ** là `"ĐẠT"` trơn khi còn phép kiểm bị bỏ:

| `ketLuan` | Nghĩa |
|---|---|
| `KHÔNG ĐẠT` | Có lỗi cấu trúc. Sửa rồi kiểm lại |
| `CHƯA KIỂM ĐỦ` | Cấu trúc sạch, nhưng nội dung tệp chưa ai xem |
| `CHƯA KIỂM ĐỦ (có cảnh báo)` | nt, kèm cảnh báo |
| `ĐẠT` / `ĐẠT (có cảnh báo)` | **chỉ xuất hiện khi gửi đủ nội dung** — không phải trường hợp của cổng |

### 12.3. Thân gói

```json
{
  "maNgoai": "PRJ-26-017",
  "tenGoi": "AIP.zip",
  "coGoi": 131723288,
  "tep": [
    { "duongDan": "urn_H49_uuid_ABC/METS.xml", "co": 12040, "noiDung": "PD94bWw…" },
    { "duongDan": "urn_H49_uuid_ABC/metadata/descriptive/EAD.xml", "co": 9870, "noiDung": "PD94bWw…" },
    { "duongDan": "urn_H49_uuid_ABC/representations/rep1/data/0000000.pdf", "co": 1351408 }
  ]
}
```

| Trường | | |
|---|---|---|
| `tep[].duongDan` | **bắt buộc** | Đường dẫn **trong gói**, dùng dấu `/`. Không được bắt đầu bằng `/`, không chứa `..` |
| `tep[].co` | nên có | Cỡ tệp, byte. Thiếu thì phép kiểm `SIZE` của METS coi như 0 và báo lệch |
| `tep[].noiDung` | chỉ tệp siêu dữ liệu | Base64. Gửi cho **mọi** tệp `.xml` / `.xsd`; tệp nội dung thì **bỏ trống** |
| `tenGoi` · `coGoi` | không | Chỉ để hiện lại |

**Trần**: 20.000 mục · 12 MiB tổng siêu dữ liệu. Gói 1 GB gửi lên chừng 2,5 MB.

⚠️ Gửi kèm nội dung của tệp PDF là **vô ích và sẽ vượt trần** — cửa không dùng tới.

### 12.4. Cổng lấy bản kê ở đâu ra

Hai cách, cổng nên đỡ cả hai:

**a) Người dùng chọn cả thư mục gói** (`<input type="file" webkitdirectory>`) — đơn giản nhất.
Duyệt danh sách, đọc nội dung các tệp `.xml`/`.xsd`, còn lại chỉ lấy tên + `size`. Không đọc byte
nào của PDF.

**b) Người dùng chọn tệp `.zip`** — không được đọc cả tệp vào bộ nhớ. **Thư mục trung tâm của
zip đã chứa sẵn tên và cỡ của mọi mục**: đọc ~64 KB cuối tệp để tìm *End of Central Directory*,
duyệt central directory → có ngay bản kê, rồi mới đọc riêng các mục `.xml` tại offset của chúng.
Tổng số byte đọc từ đĩa dưới 1 MB, dù gói 10 GB.

> **Gói lồng gói.** Gói giao thật hay có dạng `AIP.zip` → bên trong đúng **một** tệp
> `AIP/uuid_….zip` → gói thật nằm trong đó. Cổng phải đi xuống một lớp khi thấy zip chỉ chứa
> đúng một tệp `.zip`.
>
> ⚠️ **Sửa 13/11/2026 — bản trước của mục này nói sai.** Lớp trong **CÓ nén** (deflate, phương
> thức 8), không phải lưu thẳng. Đo lại trên gói mẫu: nén 122.428.283 byte / gốc 122.421.254
> byte — deflate làm *to thêm* 7.029 byte, tỉ lệ 100,006%, nên nhìn số thì tưởng là lưu thẳng.
> Hệ quả thật: **không đọc được lớp trong theo offset**. Phải cho chạy qua
> `DecompressionStream("deflate-raw")` rồi mới thấy thư mục trung tâm của nó. Vẫn làm được và
> vẫn không tốn bộ nhớ (vừa giải vừa bỏ, chỉ giữ phần đuôi), nhưng là **duyệt hết 122 MB** chứ
> không phải nhảy thẳng tới offset — hãy tính vào ước lượng của đợt 2.

#### Chọn nhầm thư mục bọc ngoài thì sao

Máy chủ **tự gỡ các lớp thư mục bọc ngoài** cho tới khi `METS.xml` nằm ngay dưới đúng một thư
mục gốc, rồi mới kiểm. Nên cứ gửi nguyên đường dẫn như trình duyệt đưa:

| Người dùng chọn | Đường dẫn gửi lên | Máy chủ hiểu |
|---|---|---|
| đúng thư mục gói | `urn_…/METS.xml` | giữ nguyên |
| thư mục bọc một lớp | `AIP/urn_…/METS.xml` | gỡ `AIP/` |
| thư mục bọc nhiều lớp | `Desktop/giao/AIP/urn_…/METS.xml` | gỡ hết phần thừa |

Có gỡ thì kết quả kèm một dòng `phatHien` mức `dat`: *"Bỏ qua N lớp thư mục bọc ngoài…"* — bày
ra, đừng ẩn, để người dùng biết máy đã hiểu đúng gói nào.

**Chỉ gỡ khi MỌI tệp đều nằm trong lớp bọc đó.** Chọn thư mục chứa **hai** gói thì máy không
đoán: nó gỡ lớp bọc rồi báo lỗi *"Gói có 2 thư mục gốc (urn_A, urn_B) — phải đúng một"*, gọi
đúng tên hai thư mục để người dùng chọn lại.

Còn thư mục **không có `METS.xml` nào** thì ra 400 `"không mục nào có noiDung"` — đó là người
dùng trỏ vào thư mục chứa `AIP.zip` chưa giải nén. Hãy bắt riêng trường hợp này bằng câu *"Thư
mục này không có tệp mô tả — hãy chọn thư mục đã giải nén gói (thư mục chứa METS.xml)"*.

(Lưu ý cho gói `.zip` thật: `load()` ở bản desktop **vẫn nghiêm**, lớp bọc thừa trong zip là lỗi
cấu trúc E-ARK CSIP. Chỗ nới tay này chỉ dành cho bản kê của cổng, vì ở đó lớp thừa là do người
dùng chọn thư mục, không phải do gói sai.)

### 12.5. Phản hồi

```json
{
  "maNgoai": "PRJ-26-017", "maDuAn": "LT-2026-017", "tenDuAn": "…",
  "tenGoi": "AIP.zip", "soMuc": 28, "soTepSieuDuLieu": 21, "coGoi": 131723288,
  "loai": "AIP", "objid": "urn:H49.155.03:uuid:A649…", "profile": "…",
  "coQuanTao": "…", "phanMemTao": "…", "ngayTao": "…",
  "maPhong": "H49.155.03", "tieuDe": "…",
  "soTaiLieu": 7, "soTrang": 412, "soTepDuLieu": 7,
  "ketLuan": "CHƯA KIỂM ĐỦ (có cảnh báo)",
  "soLoi": 0, "soCanhBao": 11, "chuaKiemNoiDung": 21,
  "chuaKiem": [
    { "ma": "pdfa", "ten": "PDF/A từng tệp (lớp chữ, dpi, chữ ký)", "soPhep": 7, "kiemODau": "Tự kiểm PDF/A cho tệp mẫu" },
    { "ma": "bam_mets", "ten": "Mã băm SHA-256 khai trong METS", "soPhep": 7, "kiemODau": "Bản kiểm đầy đủ lúc nộp gói" },
    { "ma": "bam_premis", "ten": "Mã băm (fixity) khai trong PREMIS", "soPhep": 7, "kiemODau": "Bản kiểm đầy đủ lúc nộp gói" }
  ],
  "phatHien": [
    { "muc": "canh_bao", "nhom": "EAD", "noiDung": "7 trường <issuedDate> không theo dd/mm/yyyy…" },
    { "muc": "dat", "nhom": "METS", "noiDung": "Không có tệp mồ côi — cả 28 tệp đều được METS tham chiếu" }
  ],
  "ghiChu": "Cửa này kiểm CẤU TRÚC gói từ siêu dữ liệu…"
}
```

| Trường | |
|---|---|
| **`ketLuan`** | Bày to. Bảng nghĩa ở 12.2 |
| **`chuaKiemNoiDung`** | **Số PHÉP KIỂM** đã bỏ — không phải số tệp. Khác 0 thì **phải** bày kèm `chuaKiem[]`, đừng giấu |
| **`chuaKiem[]`** | Chính số đó tách theo nhóm: `{ma, ten, soPhep, kiemODau}`. **Dựng cột «Chưa kiểm» từ mảng này**, đừng viết cứng ba dòng bên cổng — thêm nhóm thứ tư thì cổng tự có, không phải sửa |
| `phatHien[]` | `muc` ∈ `loi` · `canh_bao` · `dat`. Mặc định nên ẩn `dat`, có nút mở ra |
| `soMuc` · `soTepSieuDuLieu` | Để người dùng thấy cổng đã đọc đúng gói |

> **Đừng đọc nhầm số 21.** Trên gói mẫu `chuaKiemNoiDung` = 21 mà `soTepSieuDuLieu` cũng = 21 —
> **trùng nhau hoàn toàn ngẫu nhiên**. 21 phép kiểm = 7 tệp dữ liệu × 3 nhóm, còn 21 tệp mô tả là
> số tệp `.xml` trong gói. Muốn chắc thì cộng `chuaKiem[].soPhep` (7 + 7 + 7), đừng suy từ số tệp.
> Nhãn nên viết **"21 phép kiểm thuộc 3 nhóm chưa chạy"**, kẻo đặt cạnh bảng 3 dòng thì người đọc
> hỏi 18 cái còn lại đâu.

### 12.6. Mã lỗi

| HTTP | | |
|---|---|---|
| 400 | `bad_request` | Thiếu `maNgoai` / `tep[]` · mục thiếu `duongDan` · `duongDan` có `..` hoặc bắt đầu bằng `/` · `noiDung` không phải base64 · quá 20.000 mục · siêu dữ liệu quá 12 MiB · **không mục nào có `noiDung`** |
| 404 | `ncc_du_an_khong_thay` | `maNgoai` không có, hoặc thuộc nguồn khác |
| 401 | `hook_unauthorized` | Khoá / chữ ký / lệch giờ |

### 12.7. Việc cần làm để nối xong

1. Route `/ncc/tu-kiem-goi` phía bạn, `maNgoai` **lấy từ phiên**.
2. Đọc bản kê trong trình duyệt (12.4). Bắt đầu bằng cách **a** — chọn thư mục — rồi thêm **b**.
3. Màn hình: thẻ `ketLuan` lớn → ba số `soLoi` / `soCanhBao` / `chuaKiemNoiDung` → thông tin gói
   (`loai`, `maPhong`, `soTepDuLieu`) → bảng `phatHien[]` gom theo `nhom`.
4. **Luôn bày `ghiChu`** khi `chuaKiemNoiDung > 0`. Gói "CHƯA KIỂM ĐỦ" vẫn có thể bị từ chối
   lúc giao — nói trước còn hơn để người ta tưởng đã xong.
5. Dòng menu riêng, như bốn cửa kia (xem 10.7).

---

## 13. Kiểm **cả đợt giao** — `POST /api/v1/ncc/tu-kiem/goi/dot`

### 13.1. Nó bắt loại lỗi nào

Loại lỗi mà kiểm từng gói **không bao giờ** thấy: gói nào cũng sạch, nhưng đặt cạnh nhau thì cả
đợt hỏng.

| | Mức | Vì sao |
|---|---|---|
| Hai gói cùng `OBJID` | **lỗi** | Công cụ đóng gói dùng lại UUID. Kho chỉ giữ được một gói ⇒ mất gói kia |
| Hai gói cùng tên tệp | **lỗi** | Ghi đè nhau lúc chép sang ổ giao |
| Đợt lẫn nhiều mã phông | cảnh báo | Gần như luôn là gói lạc sang đợt khác — nhưng có đơn vị giao gộp thật, nên **không** phải lỗi |
| Gói không đọc được `OBJID` | cảnh báo | Nói thẳng là **chưa** đối chiếu trùng lặp được cho gói đó |

Cộng luôn cả đợt: tổng tài liệu · trang · tệp dữ liệu, và số gói còn lỗi cấu trúc.

### 13.2. Gửi gì — chính thứ bạn vừa nhận

Kiểm từng gói trước bằng mục 12, giữ lại kết quả, rồi gửi **tóm tắt** sang đây. Đúng những khoá
cửa 12 vừa trả, chép thẳng sang là chạy:

```json
{ "maNgoai": "PRJ-26-017",
  "goi": [
    { "tenGoi": "AIP-001.zip", "objid": "urn:H49…A649", "maPhong": "H49.155.03",
      "soTaiLieu": 7, "soTrang": 123, "soTepDuLieu": 7,
      "ketLuan": "CHƯA KIỂM ĐỦ (có cảnh báo)", "soLoi": 0, "soCanhBao": 11 }
  ] }
```

**Đừng gửi lại bản kê của từng gói** — một đợt 50 gói là hơn 20 MB, mà cửa này không dùng tới.
Trần: **500 gói** một đợt (500 × ~400 byte tóm tắt là chưa tới 200 KB; trần đặt để chặn gói tin
bậy, không phải vì nặng).

### 13.3. Nhận

```json
{ "soGoi": 47, "soGoiKhongDat": 3,
  "tongTaiLieu": 329, "tongTrang": 5781, "tongTepDuLieu": 329,
  "soLoiTrongGoi": 9, "soCanhBaoTrongGoi": 517,
  "soLoiDot": 1, "soCanhBaoDot": 1,
  "giaoDuoc": false,
  "maPhong": [ { "ma": "H49.155.03", "soGoi": 46 }, { "ma": "H50.112.01", "soGoi": 1 } ],
  "phongTrongDuAn": ["UBND huyện Đầm Hà", "Phòng Nội vụ"],
  "phatHien": [ { "muc": "loi", "nhom": "Đợt giao", "noiDung": "2 gói cùng OBJID…" } ],
  "ghiChu": "Cửa này chỉ tìm lỗi GIỮA các gói…" }
```

| Trường | |
|---|---|
| **`giaoDuoc`** | `true` khi đợt không có lỗi đợt **và** không gói nào còn lỗi cấu trúc. Bày to |
| `soLoiDot` · `soCanhBaoDot` | Của **riêng** phép kiểm đợt |
| `soLoiTrongGoi` · `soCanhBaoTrongGoi` | **Cộng từ** kết quả từng gói bạn gửi lên. Hai cặp này khác nhau, đừng trộn |
| `maPhong[]` | Mã phông trong đợt + số gói mang mã đó. Cái nào `soGoi` nhỏ bất thường là cái lạc đàn |
| `phongTrongDuAn[]` | Phông của dự án, để bày **cạnh** `maPhong[]` |

> ⚠️ **Máy chủ KHÔNG kết luận "gói sai phông".** Mã trong METS dài hơn mã phông của dự án — đo
> trên gói thật: `H49.155.3.01.2013.09.01` so với `H49.155.03`. So khớp tự động là báo oan.
> Cổng bày hai danh sách cạnh nhau, người đọc tự nhìn.

> ⚠️ `giaoDuoc: true` **không** có nghĩa là gói sẽ được nhận. Từng gói vẫn ở mức `CHƯA KIỂM ĐỦ`
> vì nội dung tệp chưa ai xem (mục 12.2). Đừng để chữ "giao được" nuốt mất điều đó.

### 13.4. Việc cần làm để nối xong

1. Màn «Tự kiểm gói tin» cho chọn **thư mục chứa nhiều gói**: mỗi thư mục con có `METS.xml` là
   một gói. Gọi cửa 12 **lần lượt** cho từng gói.
2. Gọi 47 lần liên tiếp **không bị chặn**: chốt chống dò khoá chỉ chạy khi đệm xác thực trượt và
   chỉ đếm lần *sai*. Nhưng **gọi tuần tự hoặc tối đa 3 luồng** — mỗi lời gọi là một lần phân
   tích XML trên máy chủ.
3. Bảng mỗi gói một dòng: tên · kết luận · số lỗi · số cảnh báo. Bấm vào dòng mới xem chi tiết.
4. Xong hết thì gọi cửa 13 một lần, bày `giaoDuoc` + `phatHien[]` **phía trên** bảng — lỗi đợt
   quan trọng hơn lỗi của từng gói.
5. Gói nào gọi hỏng (mạng, 5xx) thì ghi rõ trong bảng và **loại khỏi** gói tin gửi cửa 13,
   kẻo "không trùng OBJID" là kết luận trên một đợt thiếu gói.

---


---

## 14. Cửa thứ sáu — «Tự kiểm kho trước khi đóng gói»: `POST /api/v1/ncc/tu-kiem/kho`

Mục 12 kiểm **gói đã đóng xong**. Cửa này kiểm **thứ chưa đóng**: một thư mục PDF đã quét
(Phông ▸ Hộp ▸ Hồ sơ) và một tệp Excel metadata phẳng, mỗi dòng một tài liệu.

Đây là bước kiểm của tab «Đóng gói nhanh» trên máy trạm DigiText, đưa nguyên lên máy chủ.

**Vì sao nên làm cửa này trước cả mục 12 trong quy trình của nhà thầu:** nó bắt lỗi **sớm
nhất** trong cả dây chuyền. Sai mã phông, tên tệp lệch mã hồ sơ, thiếu một tệp — biết ngay
khi còn là thư mục quét, sửa mất mười phút. Phát hiện ở mục 12 thì gói trăm MB đã đóng
xong, phải đóng lại từ đầu.

### 14.1. Khác mục 12 ở điểm quan trọng nhất: **không bỏ phép kiểm nào**

Mục 12 phải bỏ ba phép kiểm (PDF/A, mã băm METS, mã băm PREMIS) vì bản kê không mang byte
nội dung. Cửa này **không bỏ gì cả** — vì bước kiểm nông vốn đã không cần byte nội dung:
nó chỉ đọc **tên tệp, cỡ tệp và tệp Excel**. Nhánh mở PDF trong bộ kiểm nằm trọn sau một
điều kiện mà chế độ nhanh luôn tắt.

Nói cách khác: kết quả cửa này **bằng đúng** kết quả chạy «Đóng gói nhanh» trên máy trạm.
Bên DigiText có test đối chiếu hai đường cho ra cùng danh sách lỗi.

> **Nhưng nó không nói gì về CHẤT LƯỢNG tệp.** Không độ phân giải, không PDF/A, không lớp
> chữ, không chữ ký số. Kho qua được cửa này vẫn có thể chưa đạt Điều 8.2.a TT05. Muốn kiểm
> chất lượng thì gửi một tệp mẫu sang mục 11.

### 14.2. Thân gói

```json
{
  "maNgoai": "PRJ-26-017",
  "tenExcel": "Metadata.xlsx",
  "excel": "UEsDBBQA…",
  "tep": [
    { "duongDan": "KHO/UBND/HOP01/HS 001/H49.101.01.1993.001.0000001.pdf", "co": 1048576 },
    { "duongDan": "KHO/UBND/HOP01/HS 001/H49.101.01.1993.001.0000002.pdf", "co": 2097152 }
  ]
}
```

| Khoá | Bắt buộc | Nghĩa |
|---|---|---|
| `maNgoai` | ✔ | mã dự án bên hệ thống của bạn |
| `excel` | ✔ | nội dung tệp Excel metadata (`.xlsx`/`.xls`), **mã base64** |
| `tenExcel` | | tên tệp, chỉ để hiện lại. Mặc định `Metadata.xlsx` |
| `tep[].duongDan` | ✔ | đường dẫn **tương đối**. Nhận cả `/` lẫn `\` |
| `tep[].co` | | cỡ tệp, byte. Thiếu = 0, chỉ ảnh hưởng phép kiểm tệp rỗng |

**Trần:** Excel ≤ 24 MiB · `tep[]` ≤ 200.000 mục · thân gói ≤ 64 MiB. Kho lớn hơn thì gửi
từng phông.

Đường dẫn tuyệt đối (`/…`, `C:\…`) và đoạn `..` bị từ chối bằng **400**.

### 14.3. Lấy bản kê ở trình duyệt — không đọc nội dung tệp

Ô chọn thư mục cho đủ dữ liệu mà **không** phải đọc một byte nội dung nào:

```html
<input type="file" webkitdirectory directory multiple />
```

```ts
const tep = [...input.files].map(f => ({
  duongDan: f.webkitRelativePath,   // "KHO/UBND/HOP01/HS 001/….pdf"
  co: f.size,                        // chỉ siêu dữ liệu, KHÔNG đọc nội dung
}))
```

`f.size` và `f.webkitRelativePath` là siêu dữ liệu, có ngay — kho 50.000 tệp dựng xong
danh sách trong chớp mắt. Đừng gọi `f.arrayBuffer()`: không cần, và sẽ nuốt hết bộ nhớ.

Riêng tệp Excel thì phải đọc thật, nhưng chỉ một tệp.

### 14.4. Căn lề đường dẫn — chỗ dễ hiểu sai nhất

`webkitRelativePath` **luôn** gắn tên thư mục người dùng vừa chọn vào đầu. Excel thì ghi
đường dẫn từ gốc kho. Hai bên gần như không bao giờ trùng nhau sẵn:

| Excel ghi | Người dùng chọn thư mục | Trình duyệt kê | Máy chủ làm |
|---|---|---|---|
| `UBND\DCXD\…` | thư mục chứa `UBND` | `KHO\UBND\DCXD\…` | bỏ `KHO` |
| `UBND\DCXD\…` | thẳng vào `UBND` | `UBND\DCXD\…` | không phải chỉnh |
| `UBND\DCXD\…` | thẳng vào `DCXD` | `DCXD\…` | thêm `UBND` |

Máy chủ tự suy bằng cách so tên tệp giữa hai danh sách, và **chỉ nhận khi việc căn làm số
tệp khớp tăng lên** — suy sai mà vẫn áp thì mọi tệp đều hoá "thiếu", tệ hơn để nguyên.

Kết quả trả về ở `canLe`:

```json
"canLe": { "bo": "KHO", "them": "", "khop": 4 }
```

**Hãy hiện câu này ra màn hình.** Người dùng chọn nhầm một cấp thư mục là nhìn thấy ngay,
không phải đoán qua danh sách lỗi. `canLe: null` nghĩa là bản kê vốn đã khớp.

Không suy được cách căn nào → `canLe: null` + lỗi `THU-MUC-GOC-KHONG-CO` nói rõ nên chọn
thư mục nào.

### 14.5. Phản hồi

```json
{
  "maNgoai": "PRJ-26-017", "maDuAn": "DA26001", "tenDuAn": "…",
  "tenExcel": "Metadata.xlsx", "sheet": "Metadata", "dongTieuDe": 1,
  "soDongExcel": 2,

  "soTepKe": 3, "soTepDoiChieu": 2, "boQuaKhongPdf": 1, "ngoaiPhamVi": 0,
  "canLe": { "bo": "KHO", "them": "", "khop": 1 },

  "soHoSo": 1, "soHoSoDat": 0, "soLoi": 2, "soCanhBao": 0, "dongGoiDuoc": false,

  "cotNhanDuoc": [ { "truong": "hs_code", "tieuDe": "Mã hồ sơ" } ],
  "hoSo": [ { "thuMuc": "UBND\\HOP01\\HS 001", "maHoSo": "…", "tieuDe": "…",
              "maPhong": "H49.101", "soTaiLieu": 2, "co": 3145728,
              "soLoi": 2, "soCanhBao": 0 } ],
  "vanDe": [ {
      "muc": "loi", "ma": "TEP-KHONG-CO-TREN-DIA",
      "tieuDe": "Excel khai tệp nhưng trên đĩa không có",
      "cachSua": "Kiểm tra lại đường dẫn ở cột «Tệp tin tài liệu»…",
      "hoSo": "UBND\\HOP01\\HS 001", "maHoSo": "H49.101.01.1993.001",
      "dongExcel": 3, "tep": "….pdf", "noiDung": "…", "soLan": 1
  } ],
  "hoSoBiCat": 0, "vanDeBiCat": 0
}
```

Ba số đếm dễ bỏ sót, nên hiện cả ba để người dùng khỏi hoang mang vì "tôi chọn 5000 tệp mà
nó chỉ kiểm 4800":

| Khoá | Nghĩa |
|---|---|
| `soTepKe` | số mục bạn gửi lên |
| `boQuaKhongPdf` | mục không phải `.pdf` (Thumbs.db, thư mục rác…) — bỏ, đúng như bản máy trạm |
| `ngoaiPhamVi` | mục nằm ngoài các thư mục cấp đầu mà Excel nhắc tới — bỏ |
| `soTepDoiChieu` | số còn lại thực sự đem đối chiếu |

`vanDe[]` bị cắt ở 2.000 mục và `hoSo[]` ở 5.000 — `vanDeBiCat`/`hoSoBiCat` nói đã cắt bao
nhiêu. Các con số tổng (`soLoi`, `soCanhBao`, `soHoSo`) thì **vẫn là số đầy đủ**, không bị
cắt theo.

### 14.6. Cột «Cách sửa» — đừng chép tay sang bên bạn

Mỗi vấn đề đã mang sẵn `tieuDe` + `cachSua` lấy từ **chính bảng hướng dẫn** bản máy trạm
đang dùng (44 mã lỗi). Dựng bảng thẳng từ hai khoá này. Chép 44 chuỗi sang mã nguồn bên bạn
thì ba tháng nữa hai bên lệch nhau — đúng cái bệnh hai nơi mà cả tài liệu này đang tránh.

Gợi ý bày màn hình: nhóm theo `ma`, mỗi nhóm một dòng gập lại được, tiêu đề là `tieuDe` +
tổng `soLan`, mở ra mới liệt kê từng dòng Excel. Một kho sai quy ước đặt tên có thể ra hàng
nghìn lỗi **cùng một mã** — liệt kê phẳng là không ai đọc nổi.

### 14.7. `dongGoiDuoc` nghĩa là gì

`true` khi **không còn lỗi nào** (cảnh báo thì không chặn). Đó là điều kiện để bước đóng
gói bên DigiText chịu chạy: nghiệp vụ đòi khớp hết rồi mới đóng gói.

Nhưng nó **không** hứa gói sẽ đạt TT05 — xem lại 14.1. Nhãn nên là «Đủ điều kiện đóng gói»,
đừng viết «Đạt».

### 14.8. Nhịp gọi

Mỗi lời gọi là một lần đọc Excel + đối chiếu vài chục nghìn dòng trên máy chủ. Gọi **một
lần cho mỗi lần người dùng bấm**, đừng gọi lại khi họ chỉ lọc hay sắp xếp bảng — giữ kết
quả ở phía trình duyệt.
