# Chuyển kho MinIO ra khỏi ổ ảo Docker — PHẦN A (buổi tối dừng hệ thống, docs/KE-HOACH-CHUYEN-KHO-MINIO.md).
#   Chạy:  powershell -ExecutionPolicy Bypass -File D:\DigiText\dev\chuyen-kho-A.ps1     (tự xin UAC)
# Làm: dừng API + worker → đóng gói nốt phần mới ra F:\DigiText-KhoTar → XOÁ kho MinIO cũ trong Docker
#      (container + volume digitext-dev_miniodata) → thu gọn ổ ảo (dev\thu-nho-docker.ps1).
# Xong phần A: KHỞI ĐỘNG LẠI MÁY, đăng nhập (Docker tự chạy), rồi chạy PHẦN B (dev\chuyen-kho-B.ps1).
# 16/10/2026: sau thu gọn, Docker không tự lên được vì WSL treo → khởi động lại máy là cách chắc chắn.
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
  Start-Process powershell.exe -Verb RunAs -ArgumentList "-NoProfile -ExecutionPolicy Bypass -NoExit -File `"$PSCommandPath`""
  exit 0
}
$ErrorActionPreference = "Stop"
$repo = Split-Path -Parent $PSScriptRoot
$log = Join-Path $repo "logs\chuyen-kho.log"
$tar = "F:\DigiText-KhoTar"
function Ghi($m) { $l = "{0:yyyy-MM-dd HH:mm:ss} A: {1}" -f (Get-Date), $m; Write-Host $l; Add-Content -Path $log -Value $l -Encoding UTF8 }
function Dung($m) { Ghi "DỪNG: $m"; throw $m }

Ghi "=== Bắt đầu phần A"
if (-not (Test-Path "$tar\danh-sach.csv")) { Dung "không thấy $tar\danh-sach.csv — cắm ổ F: rồi chạy lại" }
$kiem = Get-Content (Join-Path $repo "logs\kiem-kho-tar.log") -Tail 1 -ErrorAction SilentlyContinue
if ($kiem -notmatch "XONG kiểm: .* thiếu 0; lỗi 0") { Dung "chưa có kết quả kiểm .tar đạt (logs\kiem-kho-tar.log): $kiem" }
Ghi "Kiểm .tar trước đó: $kiem"

# 1. Dừng API + worker để không còn ai ghi vào kho.
foreach ($t in "DigiText-API", "DigiText-Worker") { Stop-ScheduledTask -TaskName $t; Ghi "đã dừng $t" }
Get-Process digitext-api, digitext-ocr -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep 3

# 2. Đóng gói nốt phần phát sinh từ lúc đóng gói lớn.
Ghi "đóng gói phần mới…"
& python -I (Join-Path $repo "dev\dong-goi-kho.py") xuat --nguon http://127.0.0.1:9000 --thu-muc $tar 2>&1 | ForEach-Object { Ghi "  $_" }
if ($LASTEXITCODE -ne 0) { Dung "đóng gói phần mới có lỗi — KHÔNG xoá kho cũ. Bật lại dịch vụ: dev\restart-services.ps1" }
$dong = Import-Csv "$tar\danh-sach.csv" | Measure-Object | Select-Object -ExpandProperty Count
Ghi "danh sách .tar: $dong tệp"

# 3. Xoá kho MinIO cũ trong Docker (bản trên F: đã kiểm SHA-256).
Ghi "dừng + xoá container digitext-dev-minio-1 và volume digitext-dev_miniodata"
docker stop digitext-dev-minio-1 2>&1 | ForEach-Object { Ghi "  $_" }
docker rm digitext-dev-minio-1 2>&1 | ForEach-Object { Ghi "  $_" }
docker volume rm digitext-dev_miniodata 2>&1 | ForEach-Object { Ghi "  $_" }
if (docker volume ls -q --filter name=digitext-dev_miniodata) { Dung "volume vẫn còn" }

# 4. Thu gọn ổ ảo (dừng Docker; script tự bật lại — nếu Docker không lên thì khởi động lại máy).
Ghi "thu gọn ổ ảo (dev\thu-nho-docker.ps1)…"
& (Join-Path $repo "dev\thu-nho-docker.ps1")
Ghi "=== Xong phần A. KHỞI ĐỘNG LẠI MÁY, đăng nhập, đợi Docker chạy, rồi chạy dev\chuyen-kho-B.ps1"
