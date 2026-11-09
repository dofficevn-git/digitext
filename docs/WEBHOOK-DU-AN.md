# Webhook tạo dự án trong DigiText

Tài liệu cho lập trình viên của hệ thống gửi (CRM / phần mềm quản lý hợp đồng).

Khi bên bạn có dự án mới, gọi một lần `POST` sang địa chỉ dưới đây. DigiText tự tạo dự
án, ghi khối lượng hợp đồng và mốc thời gian, gán người quản trị, rồi trả về mã dự án.
Không cần ai bên DigiText bấm gì thêm; màn Báo cáo tiến độ tính % hoàn thành ngay từ
khối lượng bạn gửi.

| | |
|---|---|
| Địa chỉ | `POST https://api.digitext.vn/api/v1/hooks/du-an` |
| Thử khoá (không tạo gì) | `POST https://api.digitext.vn/api/v1/hooks/kiem-tra` |
| Kiểu dữ liệu | `application/json`, mã hoá UTF-8 |
| Xác thực | khoá trong header, thêm chữ ký HMAC-SHA256 nếu khoá của bạn có bật |

Khoá do bên DigiText cấp, gồm hai phần: `X-DigiText-Key` và (nếu bật ký) một bí mật ký
riêng. Hai thứ này **chỉ hiện một lần** lúc cấp — mất thì xin cấp lại, không tra lại được.

---

## 1. Xác thực

Mọi lời gọi gửi kèm:

| Header | Bắt buộc | Nội dung |
|---|---|---|
| `X-DigiText-Key` | luôn | khoá được cấp, dạng `dk_xxxxxxxx.<bí mật>` |
| `X-DigiText-Timestamp` | khi khoá bật ký | giờ hiện tại, tính bằng **giây** Unix (UTC) |
| `X-DigiText-Signature` | khi khoá bật ký | `sha256=<HMAC-SHA256 dạng hex>` |

Chuỗi đem ký là **dấu thời gian, một dấu chấm, rồi nguyên văn thân gói**:

```
<timestamp> + "." + <thân gói JSON>
```

Ký bằng HMAC-SHA256 với bí mật ký, xuất ra hex chữ thường.

> **Chỗ hay sai nhất:** phải ký đúng **chuỗi byte bạn thực sự gửi đi**. Nếu bạn dựng JSON,
> ký nó, rồi để thư viện HTTP tự tuần tự hoá lại object thì thứ tự khoá hoặc khoảng trắng
> có thể đổi và chữ ký sẽ không khớp. Hãy tạo chuỗi JSON trước, ký chuỗi đó, rồi gửi
> đúng chuỗi đó làm thân gói.

Dấu thời gian lệch quá **5 phút** so với máy chủ thì bị từ chối — nhớ cho máy gửi đồng
bộ giờ qua NTP.

Nếu hệ thống bên bạn không ký được, xin DigiText cấp khoá dạng không bắt buộc ký; khi đó
chỉ cần `X-DigiText-Key`. Kênh vẫn chạy trên HTTPS, nhưng nên xin lọc theo IP kèm theo.

### Ví dụ

**curl** (khoá không bắt buộc ký):

```bash
curl -X POST https://api.digitext.vn/api/v1/hooks/du-an \
  -H "Content-Type: application/json" \
  -H "X-DigiText-Key: dk_0bcf....8e29f1" \
  -d '{"maNgoai":"HD-2026-0147","maDuAn":"VYEN","tenDuAn":"Chỉnh lý tài liệu UBND xã Việt Yên","loaiDuAn":"chinh_ly_so_hoa","khoiLuong":{"to":1250000,"hoSo":9983,"hop":1420},"ngayKetThucHopDong":"2027-03-31"}'
```

**C#**:

```csharp
var body = JsonSerializer.Serialize(goiTin);           // dựng chuỗi TRƯỚC
var ts   = DateTimeOffset.UtcNow.ToUnixTimeSeconds().ToString();
using var h = new HMACSHA256(Encoding.UTF8.GetBytes(biMatKy));
var sig = Convert.ToHexString(h.ComputeHash(Encoding.UTF8.GetBytes($"{ts}.{body}"))).ToLower();

var req = new HttpRequestMessage(HttpMethod.Post, "https://api.digitext.vn/api/v1/hooks/du-an");
req.Headers.Add("X-DigiText-Key", khoa);
req.Headers.Add("X-DigiText-Timestamp", ts);
req.Headers.Add("X-DigiText-Signature", "sha256=" + sig);
req.Content = new StringContent(body, Encoding.UTF8, "application/json");
var res = await client.SendAsync(req);
```

**PHP**:

```php
$body = json_encode($goiTin, JSON_UNESCAPED_UNICODE);
$ts   = (string) time();
$sig  = hash_hmac('sha256', $ts . '.' . $body, $biMatKy);

$ch = curl_init('https://api.digitext.vn/api/v1/hooks/du-an');
curl_setopt_array($ch, [
    CURLOPT_POST => true,
    CURLOPT_POSTFIELDS => $body,
    CURLOPT_RETURNTRANSFER => true,
    CURLOPT_HTTPHEADER => [
        'Content-Type: application/json',
        'X-DigiText-Key: ' . $khoa,
        'X-DigiText-Timestamp: ' . $ts,
        'X-DigiText-Signature: sha256=' . $sig,
    ],
]);
$res = curl_exec($ch);
```

**Python**:

```python
body = json.dumps(goi_tin, ensure_ascii=False).encode()
ts = str(int(time.time()))
sig = hmac.new(bi_mat_ky.encode(), ts.encode() + b"." + body, hashlib.sha256).hexdigest()
r = requests.post(
    "https://api.digitext.vn/api/v1/hooks/du-an",
    data=body,
    headers={
        "Content-Type": "application/json",
        "X-DigiText-Key": khoa,
        "X-DigiText-Timestamp": ts,
        "X-DigiText-Signature": "sha256=" + sig,
    },
)
```

### Giữ khoá

Khoá và bí mật ký **chỉ hiện một lần** lúc DigiText cấp. Trong cơ sở dữ liệu của DigiText,
phần bí mật của khoá chỉ còn bản băm — không ai tra lại được, kể cả người quản trị hệ
thống. Chép ngay lúc nhận; mất thì phải xin cấp khoá mới.

- Cất vào nơi chứa bí mật của phần mềm bạn: biến môi trường, tệp cấu hình được mã hoá,
  hoặc secret store. **Đừng viết thẳng vào mã nguồn** rồi đẩy lên git.
- Đừng nhận khoá qua Zalo hay email thường — khoá này tạo được dự án thật trong DigiText.
  Hãy nhận qua kênh riêng, hoặc nhận khoá một đường và bí mật ký một đường khác.
- **Cấp lại khoá là cắt khoá cũ ngay lập tức**, không có khoảng thời gian hai khoá cùng
  chạy. Vì vậy hãy làm sao đổi được khoá mà **không phải biên dịch lại và phát hành lại**
  phần mềm; và khi đổi thì hẹn giờ với DigiText, kẻo lời gọi ngay sau đó nhận `401`.
- Nếu nghi khoá bị lộ, báo DigiText thu hồi ngay chứ đừng chờ tới kỳ phát hành sau.

---

## 2. Thân gói

```json
{
  "event": "du_an.tao",
  "maNgoai": "HD-2026-0147",
  "maDuAn": "VYEN",
  "tenDuAn": "Chỉnh lý tài liệu UBND xã Việt Yên",
  "loaiDuAn": "chinh_ly_so_hoa",
  "khoiLuong": { "to": 1250000, "hoSo": 9983, "hop": 1420 },
  "ngayKyHopDong": "2026-10-01",
  "ngayKetThucHopDong": "2027-03-31",
  "hanHoanThanh": "2027-03-15",
  "canBoPhuTrach": "Nguyễn Văn A",
  "ghiChu": "Giai đoạn 1",
  "quanTri": ["nvanh", "ptbinh@congty.vn"]
}
```

| Trường | Bắt buộc | Ý nghĩa |
|---|---|---|
| `maNgoai` | **có** | Mã dự án/hợp đồng **ở hệ thống của bạn**. Đây là chốt chống tạo trùng — xem mục 4. Tối đa 120 ký tự. |
| `tenDuAn` | khi tạo mới | Tên dự án, tối đa 300 ký tự. |
| `maDuAn` | không | Mã viết tắt dùng trong DigiText. Không gửi thì DigiText tự đặt (`DA26001`…) và báo lại trong `canhBao`. Xem mục 3. |
| `loaiDuAn` | không | Một trong: `so_hoa`, `chinh_ly`, `chinh_ly_so_hoa`, `luu_tru`, `khac`. Nhận cả cách viết tiếng Việt ("chỉnh lý số hoá"). |
| `khoiLuong` | không | Khối lượng theo hợp đồng, xem bảng dưới. |
| `ngayKyHopDong` | không | Ngày ký. |
| `ngayKetThucHopDong` | không | Ngày kết thúc theo hợp đồng. |
| `hanHoanThanh` | không | Hạn chót hoàn thành, nếu khác ngày kết thúc. Không gửi thì lấy bằng `ngayKetThucHopDong`. |
| `canBoPhuTrach` | không | Tên người phụ trách phía bạn. |
| `ghiChu` | không | Ghi chú tự do. |
| `quanTri` | không | Danh sách **tên đăng nhập hoặc email** của người sẽ quản trị dự án trong DigiText. Xem mục 5. |
| `event` | không | Mặc định `du_an.tao`. Xem mục 6. |

### Khối lượng công việc

Gửi đơn vị nào bên bạn có; không có thì bỏ trống.

| Khoá | Tên khác cũng nhận | Đơn vị |
|---|---|---|
| `to` | `soTo` | tờ |
| `trang` | `soTrang` | trang |
| `hoSo` | `soHoSo` | hồ sơ |
| `hop` | `soHop` | hộp |
| `tep` | `soTep` | tệp tài liệu |

Số nguyên, không âm.

### Ngày tháng

Nhận `YYYY-MM-DD` (nên dùng) hoặc `dd/mm/yyyy`. Năm phải ghi đủ bốn chữ số. Ngày kết
thúc không được sớm hơn ngày ký.

---

## 3. Mã dự án

Mã dự án là cái người làm việc trong DigiText nhìn thấy và gõ hằng ngày, nên nó ngắn và
viết tắt: `VYEN`, `STC`, `BG2026`.

- Dài 2–20 ký tự, chỉ **chữ không dấu, số, `-` và `_`**. Chữ thường sẽ được chuyển thành
  chữ hoa.
- **Không có dấu cách, không có dấu chấm.**
- Nếu bạn không gửi `maDuAn`, DigiText tự đặt `DA26001`, `DA26002`… và trả về trong
  `canhBao` để người quản trị đổi lại sau. Dự án vẫn hoạt động bình thường.
- Nếu mã bạn gửi đã có trong DigiText nhưng **do người ta tạo tay** (chưa gắn với hệ
  thống nào), DigiText nhận dự án đó về, gắn `maNgoai` của bạn vào và cập nhật tên/khối
  lượng theo gói tin — không tạo dự án thứ hai. Có cảnh báo kèm theo.
- Nếu mã đó đang thuộc một `maNgoai` khác thì trả về **409**, bạn đổi `maDuAn` khác.

---

## 4. Gửi lại nhiều lần thì sao

Webhook nào cũng có lúc gửi lại: mạng nghẽn, bên bạn không nhận được phản hồi, hoặc
người dùng bấm hai lần. **`maNgoai` là chốt**: một `maNgoai` chỉ ứng với một dự án trong
DigiText, mãi mãi.

- Lần đầu: tạo dự án, trả **201**, `taoMoi = true`.
- Những lần sau cùng `maNgoai`: cập nhật dự án cũ, trả **200**, `taoMoi = false`. Không
  bao giờ sinh dự án thứ hai.

Nên bạn cứ gửi lại thoải mái khi không chắc. Đừng sinh `maNgoai` mới cho cùng một hợp
đồng — làm thế là tạo dự án mới.

**Trường nào không gửi thì giữ nguyên.** Muốn sửa riêng ngày kết thúc thì gửi
`maNgoai` + `ngayKetThucHopDong`, các số khối lượng cũ vẫn còn. Trường nào bạn gửi thì
ghi đè, kể cả tên dự án — nên đừng gửi `tenDuAn` rỗng hay sai.

---

## 5. Người quản trị dự án

Dự án về mà chưa có người quản trị thì **chỉ super admin của DigiText nhìn thấy nó**.
Hai cách tránh:

1. Gửi `quanTri` là tên đăng nhập hoặc email của người trong DigiText. Tên nào không có
   trong hệ thống sẽ bị bỏ qua và báo trong `canhBao`.
2. Nhờ DigiText đặt sẵn một người quản trị mặc định cho khoá của bạn. Khi đó bạn không
   cần gửi `quanTri` lần nào.

---

## 6. Các loại sự kiện

| `event` | Làm gì |
|---|---|
| `du_an.tao` (mặc định) | Tạo dự án; nếu `maNgoai` đã có thì cập nhật. |
| `du_an.capnhat` | Cập nhật; nếu `maNgoai` chưa có thì tạo mới (phải có `tenDuAn`). |
| `du_an.dong` | Đóng dự án: ẩn khỏi danh sách đang làm và khỏi báo cáo tiến độ. Chỉ cần `maNgoai`. |
| `du_an.mo` | Mở lại dự án đã đóng. Chỉ cần `maNgoai`. |

`du_an.dong` / `du_an.mo` trên một `maNgoai` chưa từng gửi về sẽ trả 404.

---

## 7. Phản hồi

**Thành công** — `201` khi tạo mới, `200` khi cập nhật:

```json
{
  "projectId": "01a1be45-713f-7422-81b9-909791926518",
  "maNgoai": "HD-2026-0147",
  "maDuAn": "VYEN",
  "tenDuAn": "Chỉnh lý tài liệu UBND xã Việt Yên",
  "loaiDuAn": "chinh_ly_so_hoa",
  "taoMoi": true,
  "maTuSinh": false,
  "quanTri": ["nvanh"],
  "canhBao": []
}
```

Nên lưu `projectId` lại ở hệ thống bạn — về sau tra cứu hay đối chiếu sẽ cần.

`canhBao` là những chuyện đã nhận nhưng nên biết (mã do máy tự đặt, không tìm thấy tài
khoản quản trị, dự án chưa có quản trị…). **Có cảnh báo không có nghĩa là thất bại.** Nên
ghi lại hoặc hiển thị cho người vận hành.

**Lỗi** — luôn có dạng:

```json
{ "code": "bad_request", "message": "câu giải thích bằng tiếng Việt" }
```

| Mã | Nghĩa | Bên bạn nên làm gì |
|---|---|---|
| `400` | Gói tin sai (thiếu trường, sai định dạng, loại dự án lạ) | **Đừng gửi lại y nguyên.** Đọc `message`, sửa rồi gửi. |
| `401` | Khoá sai, chữ ký không khớp, hoặc lệch giờ | Kiểm khoá, cách ký và đồng hồ máy. Gửi lại không giúp gì. |
| `403` | IP không nằm trong danh sách cho phép | Báo DigiText thêm IP. |
| `404` | `du_an.dong`/`du_an.mo` cho dự án chưa từng gửi về | Gửi `du_an.tao` trước. |
| `409` | `maDuAn` đã thuộc hợp đồng khác | Đổi `maDuAn`, hoặc bỏ trống để DigiText tự đặt. |
| `429`, `5xx` | Máy chủ đang quá tải hoặc lỗi | **Gửi lại**, giãn dần: 1 phút, 5 phút, 15 phút, 1 giờ. `maNgoai` lo phần chống trùng. |

Mọi lời gọi có khoá đúng đều được ghi nhật ký bên DigiText (gói gốc, kết quả, lý do từ
chối), nên khi hai bên nói ngược nhau thì có chỗ đối chiếu.

---

## 8. Thử trước khi nối thật

`POST /api/v1/hooks/kiem-tra` dùng đúng cách xác thực như trên nhưng **không tạo gì**:

```json
{
  "ok": true,
  "nguon": "crm",
  "ten": "CRM hợp đồng",
  "batBuocKyHmac": true,
  "gioMayChu": "2026-11-09T03:05:56Z",
  "loaiDuAnChoPhep": ["so_hoa", "chinh_ly", "chinh_ly_so_hoa", "luu_tru", "khac"]
}
```

Dùng nó để kiểm ba thứ hay sai nhất: khoá đúng chưa, có phải ký không, và đồng hồ máy
bạn lệch bao nhiêu so với `gioMayChu`.

---

## 9. Việc cần làm để nối xong

- [ ] Nhận khoá (và bí mật ký) từ DigiText; chép ngay vì chỉ hiện một lần, cất vào nơi
      chứa bí mật, đừng để trong mã nguồn.
- [ ] Đổi khoá được bằng cấu hình, không phải phát hành lại phần mềm.
- [ ] Gọi `/hooks/kiem-tra` thành công.
- [ ] Chốt `maNgoai` lấy từ trường nào bên bạn — phải **cố định suốt đời hợp đồng**.
- [ ] Gửi một dự án thật, kiểm `maDuAn` và khối lượng hiện đúng trong DigiText.
- [ ] Gửi lại đúng gói đó, xác nhận trả `200` với `taoMoi: false` và không sinh dự án thứ hai.
- [ ] Làm phần gửi lại khi gặp `5xx`, và chỗ ghi log `400`/`409` cho người vận hành xem.
- [ ] Thống nhất ai nhận quản trị dự án: gửi `quanTri`, hay đặt mặc định cho khoá.
