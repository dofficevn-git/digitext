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

## 2. Đường HTTPS cho máy khác mạng — ĐÃ LÀM: Cloudflare Tunnel, `https://api.khodulieu.vn`

Trạng thái 18/9/2026: app ở bất kỳ mạng nào trỏ **`https://api.khodulieu.vn`** là tới API
trên máy này. Cách nối:

- Máy này chạy service Windows `Cloudflared` (tunnel `zalo-smart-commune`,
  id `3eda566d-…`) dùng chung cho nhiều dự án khác trên `khodulieu.vn`.
- Config của service: `C:\Windows\System32\config\systemprofile\.cloudflared\config.yml`
  (cần admin để sửa; bản sao tham khảo `%USERPROFILE%\.cloudflared\config.yml` — KHÔNG
  phải file service đọc). Đã thêm luật:

  ```yaml
    - hostname: api.khodulieu.vn
      service: http://localhost:8080
  ```

- DNS `api.khodulieu.vn` là CNAME tới tunnel (`cloudflared tunnel route dns …`, đã có).
- API vẫn bind `127.0.0.1:8080` — không mở cổng nào ra ngoài. TLS kết thúc ở Cloudflare,
  đoạn Cloudflare→máy này đi trong tunnel mã hoá.

Kiểm nhanh từ máy bất kỳ: `curl.exe https://api.khodulieu.vn/healthz` → `ok`.

Đổi/kiểm sau này (PowerShell **admin**):

```powershell
$cfg="C:\Windows\System32\config\systemprofile\.cloudflared\config.yml"
& "C:\Program Files (x86)\cloudflared\cloudflared.exe" --config $cfg tunnel ingress validate
Restart-Service Cloudflared      # các hostname khác trên tunnel gián đoạn ~3 s
```

Giới hạn của Cloudflare cần biết:

- **100 MB/request** (gói miễn phí). Luồng "② Đẩy lên máy chủ" dùng PATCH khúc 4 MB
  (`/documents/upload`) → không vướng. Riêng `POST /documents` (tải nguyên tệp, nút
  "Tải PDF lên") sẽ hỏng với tệp > 100 MB — dùng luồng Đồng bộ dữ liệu cho tệp lớn.
- **100 s/response**: request nào máy chủ xử lý quá 100 s bị cắt (524). Các thao tác
  dài (OCR toàn bộ, PDF 2 lớp, đóng gói) đều là enqueue rồi poll nên không vướng.
- Máy chủ tắt/đổi máy → chỉ cần cài cloudflared trên máy mới với cùng luật; địa chỉ
  trong app không đổi.

Ghi chú: trong cùng config còn `do360-api/do360-admin.khodulieu.vn → localhost:8080` của
dự án cũ — hai hostname đó hiện đang rơi vào API DigiText. Nếu dự án đó đã bỏ, xoá hai
dòng ấy.

### Phương án thay thế (chưa dùng)

- **Tailscale + `tailscale serve --bg 8080`**: HTTPS riêng tư `<máy>.<tailnet>.ts.net`,
  không cần domain, cả hai máy phải cài Tailscale.
- **Caddy + Let's Encrypt** trên máy chủ: cần IP công cộng và mở 80/443.

## 3. Mỗi lần có bản mới — checklist phát hành

Tự cập nhật (tauri-plugin-updater) **đã bị gỡ chủ ý** (`desktop/src/api/updater.ts`
giải thích: bản cũ tin khoá của một tài khoản GitHub bên ngoài). Nút
"Kiểm tra cập nhật" hiện chỉ báo "Bản dựng này không bật tự cập nhật". Vậy quy trình
hiện tại là **thủ công**:

1. **Tăng số phiên bản ở 3 chỗ, cùng một số**:
   `desktop/src-tauri/tauri.conf.json` (`version`), `desktop/src-tauri/Cargo.toml`
   (`version`), `desktop/package.json` (`version`). Số này hiện ở màn đăng nhập,
   Cài đặt → Chẩn đoán, và được gửi lên máy chủ khi đăng nhập (`app_version`).
2. **Máy chủ trước, app sau.** API không ép phiên bản app tối thiểu, nên app cũ vẫn
   đăng nhập được vào API mới; ngược lại (app mới gọi route API chưa có) sẽ lỗi.
   Nếu bản mới có route API mới → cập nhật máy chủ (`cargo build --release -p digitext-api`,
   khởi động lại `dev/run-api.ps1`) trước khi phát app.
3. `npm run tauri build` → gửi `DigiText_<ver>_x64-setup.exe` → người dùng đóng app,
   chạy setup (cài đè, không cần gỡ), mở lại.
4. **Kiểm tra máy nào đã lên bản mới** — máy chủ ghi `app_version` + `last_seen`
   theo thiết bị khi đăng nhập:

   ```powershell
   # token admin lấy từ đăng nhập; hoặc xem thẳng CSDL
   docker exec -i digitext-dev-postgres-1 psql -U digitext -d digitext -c \
     "select name, app_version, last_seen from devices order by last_seen desc"
   ```

   (`GET /devices` trả cùng dữ liệu cho app/công cụ.)

### Muốn bật tự cập nhật (làm sau, khi đã ổn định)

Làm được và an toàn nếu **khoá là của mình, máy chủ là của mình**:

1. `npm i @tauri-apps/plugin-updater`, thêm `tauri-plugin-updater` vào `Cargo.toml`.
2. `npx tauri signer generate -w ~/.tauri/digitext.key` → giữ khoá riêng **ngoài repo**;
   dán khoá công khai vào `tauri.conf.json` → `plugins.updater.pubkey`,
   `endpoints = ["https://<máy>.<tailnet>.ts.net/updates/latest.json"]`,
   `bundle.createUpdaterArtifacts = true`.
3. Máy chủ phục vụ tĩnh `updates/latest.json` + file `.exe` + `.sig` (thêm route tĩnh
   vào API hoặc một MinIO bucket public sau proxy).
4. Khôi phục `check()/downloadAndInstall()` trong `desktop/src/api/updater.ts` — kiểu
   dữ liệu và giao diện đã để nguyên, chỉ thiếu phần gọi.

Khi đó checklist mục 3 rút còn: tăng version → build (đã ký) → tải `latest.json` +
exe + sig lên máy chủ → app tự cập nhật lúc mở.

## 4. Khi dự án lên git mới (GitHub): phát hành bằng tag, app tự cập nhật

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
3. Bật updater (chi tiết ở mục "Muốn bật tự cập nhật" trên) với
   `endpoints = ["https://github.com/<owner>/<repo>/releases/latest/download/latest.json"]`.
   **Lưu ý repo private**: app không có token nên không tải được asset của repo private
   → dùng một repo **public riêng chỉ chứa Releases** (ví dụ `<owner>/digitext-releases`),
   workflow đẩy Release sang đó bằng `GH_TOKEN` có quyền repo ấy. Không đưa source lên
   repo public.

Mỗi lần phát hành:

```powershell
# tăng version ở 3 file (mục 3.1), commit
git tag v0.16.0 ; git push origin main --tags
# → Actions chạy ~10 phút → Release v0.16.0 có DigiText_0.16.0_x64-setup.exe,
#   .exe.sig, latest.json → app tự cập nhật lần mở kế tiếp (hoặc bấm "Kiểm tra cập nhật")
```

Máy chủ (API/worker) không theo tag này — build và khởi động lại tay như mục 3.2,
**trước** khi đẩy tag nếu bản app cần route mới.
