# Gửi app DigiText sang máy khác và phát hành bản cập nhật

Mục tiêu: máy cài app (khác mạng với máy chủ) đăng nhập được, tải tài liệu lên,
theo dõi OCR/PDF-A — và mỗi lần có bản mới thì cập nhật gọn, không mất dữ liệu.

## 0. Ba điều app đòi hỏi (đọc trước, đỡ mò)

| Điều kiện | Ở đâu trong code | Hệ quả |
|---|---|---|
| Địa chỉ máy chủ **phải là `https://`** (chỉ cho `http://` với localhost) | `desktop/src-tauri/src/auth/commands.rs` `f_yb` | Không thể trỏ app tới `http://192.168.x.x:8080`. Cần TLS thật (chứng chỉ được Windows tin). App **không** có chỗ nhập CA riêng → chứng chỉ tự ký không dùng được. |
| App chỉ nói chuyện với **API `:8080`** | không có tham chiếu MinIO/Postgres trong `src-tauri` | Chỉ cần mở đúng một cổng/đường HTTPS tới API. |
| API mặc định bind `127.0.0.1:8080` | `crates/digitext-api/src/config.rs` (`DIGITEXT_BIND`) | Nếu đặt proxy cùng máy (khuyên) thì **giữ nguyên** — an toàn hơn; chỉ đổi khi proxy nằm máy khác. |

Tính năng **"① Thu thập từ máy user (LAN)"** (mDNS + TCP nội bộ) chỉ chạy trong cùng
mạng — máy khác mạng vẫn dùng được **"③ Thêm tài liệu từ ổ cứng" → "② Đẩy lên máy chủ"**,
và mọi màn hình máy chủ (OCR, PDF 2 lớp, bóc trường, đóng gói).

**Thêm từ ổ đĩa — cây thư mục (22/10/2026):** máy **tự nhận tên thư mục đã chọn là PHÔNG**,
thư mục con là Hộp, trong hộp là Hồ sơ (`<dự án>/<phông>/<hộp>/<hồ sơ>/tệp.pdf`; máy chủ lấy
phông = tầng 1 của `rel_path`). Trước khi gửi có hộp thoại **Xác nhận cây thư mục**
(`collect_peek_folders` chỉ đọc hai cấp đầu); nếu lỡ chọn thư mục của cả dự án thì chuyển
"Thư mục con là phông" (`subAsPhong`). Ô "Gắn theo danh mục hồ sơ" chỉ dành cho tệp lẻ.

## 1. Dựng bộ cài

```powershell
# máy dev (Windows), có cargo + node
cd D:\DigiText\desktop
npm run tauri build
```

Kết quả (`desktop\src-tauri\target\release\bundle\`):

- `nsis\DigiText_<ver>_x64-setup.exe` — **gửi file này**. Cài đè được lên bản cũ,
  dữ liệu người dùng nằm ở `%APPDATA%\vn.digitext.desktop\` không bị đụng.
- `msi\DigiText_<ver>_x64_en-US.msi` — dùng khi IT triển khai bằng GPO/Intune.

Installer tự thêm rule tường lửa "DigiText LAN" nếu chạy có quyền admin
(`src-tauri/windows/hooks.nsh`); không có quyền thì Windows hỏi lúc mở app lần đầu —
với máy khác mạng, bấm Cancel cũng không sao (rule chỉ phục vụ thu thập LAN).

## 2. Đường HTTPS cho máy khác mạng — ĐÃ LÀM: Cloudflare Tunnel, `https://api.digitext.vn`

Trạng thái 18/9/2026: app ở bất kỳ mạng nào trỏ **`https://api.digitext.vn`** là tới API
trên máy này (Windows `DESKTOP-2DMGE1I`, 192.168.0.127). Cách nối:

- Zone `digitext.vn` và tunnel nằm ở tài khoản Cloudflare **Dofficevn@gmail.com**
  (Zero Trust team `fragrant-firefly-da40`, gói Free). Tunnel kiểu *remotely managed*:
  route `api.digitext.vn → http://localhost:8080` cấu hình trên dashboard
  (Zero Trust → Networks → Tunnels & Mesh → tunnel → *Published application routes*),
  **không có file config cục bộ**; sửa route trên dashboard là tunnel nhận sau vài giây.
- Trên máy này chạy service Windows **`cloudflared-digitext`** (tự khởi động, tự restart
  khi lỗi), lệnh: `cloudflared.exe tunnel run --token-file C:\ProgramData\cloudflared-digitext	oken`,
  log `C:\ProgramData\cloudflared-digitext\cloudflared.log`. Token chỉ SYSTEM/Administrators đọc.
  Service này **độc lập** với service `Cloudflared` sẵn có (tunnel `zalo-smart-commune`,
  tài khoản khác, phục vụ `*.khodulieu.vn`) — hai service không đụng nhau.
- API vẫn bind `127.0.0.1:8080`; không mở cổng nào ra ngoài. TLS kết thúc ở Cloudflare,
  Cloudflare→máy này đi trong tunnel mã hoá.
- Dự phòng: `https://api.khodulieu.vn` cũng trỏ vào cùng API (luật ingress trong config
  của service `Cloudflared`, file `C:\Windows\System32\config\systemprofile\.cloudflared\config.yml`).

Kiểm nhanh từ máy bất kỳ: `curl.exe https://api.digitext.vn/healthz` → `ok`.

Vận hành (PowerShell **admin**):

```powershell
Get-Service cloudflared-digitext            # Running?
Restart-Service cloudflared-digitext
Get-Content C:\ProgramData\cloudflared-digitext\cloudflared.log -Tail 20
# đổi token (tạo tunnel mới trên dashboard): ghi đè file token rồi Restart-Service
```

Chuyển sang máy chủ mới: cài `cloudflared`, tạo cùng service với token của tunnel
(hoặc *Add a connector* trên dashboard), địa chỉ trong app **không đổi**.

Giới hạn của Cloudflare cần biết:

- **100 MB/request** (gói miễn phí). Luồng "② Đẩy lên máy chủ" dùng PATCH khúc 4 MB
  (`/documents/upload`) → không vướng. Riêng `POST /documents` (tải nguyên tệp, nút
  "Tải PDF lên") sẽ hỏng với tệp > 100 MB — dùng luồng Đồng bộ dữ liệu cho tệp lớn.
- **100 s/response**: request nào máy chủ xử lý quá 100 s bị cắt (524). Các thao tác
  dài (OCR toàn bộ, PDF 2 lớp, đóng gói) đều enqueue rồi poll nên không vướng.

Rác còn lại trên Cloudflare (không ảnh hưởng, dọn khi tiện): tunnel `digitext` đầu tiên
(id `45424f54-…`, không còn connector) với route `api.digifile.vn` — xoá cả hai; bản ghi
nhầm `api.digitext.vn.khodulieu.vn` trong zone `khodulieu.vn` — xoá.

### Phương án thay thế (chưa dùng)

- **Tailscale + `tailscale serve --bg 8080`**: HTTPS riêng tư `<máy>.<tailnet>.ts.net`,
  không cần domain, cả hai máy phải cài Tailscale.
- **Caddy + Let's Encrypt** trên máy chủ: cần IP công cộng và mở 80/443.

## 3. Mỗi lần có bản mới — ĐÃ BẬT tự cập nhật (từ v0.15.1)

App kiểm `https://api.digitext.vn/updates/latest.json` lúc mở; có bản mới → tải bộ cài
đã ký → cài (chế độ passive) → khởi động lại. Nút *Cài đặt → Chẩn đoán → Kiểm tra cập nhật*
làm cùng việc theo yêu cầu. Máy còn chạy **v0.15.0** (bản chưa có updater) phải cài tay
**một lần** bộ cài ≥ 0.15.1, sau đó tự động.

Phát hành = **một lệnh** trên máy này:

```powershell
powershell -ExecutionPolicy Bypass -File dev\release-desktop.ps1 -Version 0.16.0 -Notes "Sửa X, thêm Y"
```

Nó tăng version ở 3 file (`tauri.conf.json`, `package.json`, `Cargo.toml`), build NSIS
**có ký** bằng `%USERPROFILE%\.tauri\digitext.key`, chép `DigiText_<ver>_x64-setup.exe` +
`.sig` vào `D:\DigiText\updates\` (`DIGITEXT_UPDATES_DIR` của API) và ghi `latest.json`.
API phát thư mục đó ở `/updates/<tệp>` (không cần đăng nhập; route
`crates/digitext-api/src/routes/updates.rs`, chỉ tên phẳng + đuôi cho phép, `Cache-Control:
no-store`). Không cần khởi động lại API.

Sau lệnh: `git commit` 3 file version (+ `git tag v0.16.0`). Kiểm:
`curl.exe https://api.digitext.vn/updates/latest.json`.

Quy tắc an toàn đã cài:

- App chỉ cài bản có chữ ký khớp khoá công khai nhúng trong `tauri.conf.json` →
  `plugins.updater.pubkey`. **Mất `digitext.key` = không phát hành được nữa** (phải đổi
  khoá và cài tay lại mọi máy) — sao lưu file đó ở nơi an toàn, không đưa vào repo.
- **Máy chủ trước, app sau**: API không ép version app; app mới gọi route chưa có mới lỗi.
  Bản app cần route mới → build/khởi động lại API trước rồi mới chạy release.
- Máy nào đã lên bản: `select name, app_version, last_seen from devices order by 2`.
- Lượt kiểm tự động im lặng khi không có mạng; lỗi lúc tải/cài thì luôn hiện hộp thoại.
- Script .ps1 có tiếng Việt phải lưu **UTF-8 có BOM** (PowerShell 5.1 đọc không BOM thành ANSI → lỗi cú pháp).

Trước v0.15.1 (bản gửi tay đầu tiên): quy trình thủ công là gửi `-setup.exe` cài đè;
vẫn dùng được cho máy không có internet.

## 4. Khi dự án lên git mới (GitHub): phát hành bằng tag (thay cho chạy script tay)

Đây là đường đi gọn nhất cho "mỗi lần có bản mới": **đẩy một tag → GitHub Actions dựng
bộ cài → tạo Release → app trên các máy tự thấy bản mới**. Đã có sẵn workflow mẫu
`.github/workflows/desktop-release.yml` (tắt cho tới khi bật updater; xem bước 3).

Làm một lần:

1. Tạo repo mới, `git remote add origin …`, đẩy `main`.
2. Sinh khoá ký **của mình** (khoá riêng KHÔNG vào repo):
   `cd desktop && npx tauri signer generate -w $HOME/.tauri/digitext.key` → in ra
   khoá công khai. Vào repo → Settings → Secrets → thêm
   `TAURI_SIGNING_PRIVATE_KEY` (nội dung file .key) và
   `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
3. Updater đã bật, endpoint hiện là máy chủ của mình (`api.digitext.vn/updates`). Muốn phát
   qua GitHub Releases: đổi `plugins.updater.endpoints` sang
   `https://github.com/<owner>/<repo>/releases/latest/download/latest.json`.
   **Repo private**: app không có token nên không tải được asset → cần repo **public riêng
   chỉ chứa Releases**. Đơn giản hơn: giữ endpoint máy chủ, workflow chỉ build rồi `scp`
   bộ cài + `.sig` + `latest.json` vào `D:\DigiText\updates` (hoặc chạy `release-desktop.ps1`).

Mỗi lần phát hành:

```powershell
# tăng version ở 3 file (mục 3.1), commit
git tag v0.16.0 ; git push origin main --tags
# → Actions chạy ~10 phút → Release v0.16.0 có DigiText_0.16.0_x64-setup.exe,
#   .exe.sig, latest.json → app tự cập nhật lần mở kế tiếp (hoặc bấm "Kiểm tra cập nhật")
```

Máy chủ (API/worker) không theo tag này — build và khởi động lại tay như mục 3.2,
**trước** khi đẩy tag nếu bản app cần route mới.
