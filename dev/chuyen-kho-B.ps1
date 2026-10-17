# Chuyển kho MinIO — PHẦN B (sau phần A + khởi động lại máy). Chạy lại được nhiều lần.
#   Chạy:  powershell -ExecutionPolicy Bypass -File D:\DigiText\dev\chuyen-kho-B.ps1     (tự xin UAC)
# Làm: chờ Docker + Postgres → MinIO Windows (DigiText-MinIO, E:\DigiText-KhoMayChu) chuyển sang cổng 9000 →
#      nạp kho từ F:\DigiText-KhoTar (kiểm SHA-256 từng tệp) → bật API + worker → kiểm tra.
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
  Start-Process powershell.exe -Verb RunAs -ArgumentList "-NoProfile -ExecutionPolicy Bypass -NoExit -File `"$PSCommandPath`""
  exit 0
}
$ErrorActionPreference = "Stop"
$repo = Split-Path -Parent $PSScriptRoot
$log = Join-Path $repo "logs\chuyen-kho.log"
$tar = "F:\DigiText-KhoTar"
function Ghi($m) { $l = "{0:yyyy-MM-dd HH:mm:ss} B: {1}" -f (Get-Date), $m; Write-Host $l; Add-Content -Path $log -Value $l -Encoding UTF8 }
function Dung($m) { Ghi "DỪNG: $m"; throw $m }

Ghi "=== Bắt đầu phần B"
if (-not (Test-Path "$tar\danh-sach.csv")) { Dung "không thấy $tar — cắm ổ F:" }
$han = (Get-Date).AddMinutes(10)
do { Start-Sleep 5; docker exec digitext-dev-postgres-1 pg_isready -q *> $null } until ($LASTEXITCODE -eq 0 -or (Get-Date) -gt $han)
if ($LASTEXITCODE -ne 0) { Dung "Docker/Postgres chưa chạy — mở Docker Desktop rồi chạy lại phần B" }
Ghi "Postgres sẵn sàng"
if (docker ps -a -q --filter name=digitext-dev-minio-1) { Dung "MinIO cũ trong Docker vẫn còn — chạy phần A trước" }

# 1. MinIO Windows sang cổng 9000 (API/worker vẫn trỏ 127.0.0.1:9000, không phải sửa .env).
& (Join-Path $repo "dev\minio-win-task.ps1") -DataDir "E:\DigiText-KhoMayChu" -Port 9000
$ok = try { (Invoke-WebRequest -UseBasicParsing -TimeoutSec 10 http://127.0.0.1:9000/minio/health/live).StatusCode } catch { 0 }
if ($ok -ne 200) { Dung "MinIO mới chưa trả lời ở cổng 9000" }
Ghi "MinIO mới (E:\DigiText-KhoMayChu) chạy ở cổng 9000"
# Kho tạm trên USB không dùng nữa.
if (Get-ScheduledTask -TaskName "DigiText-MinIO-Tam" -ErrorAction SilentlyContinue) {
  Stop-ScheduledTask -TaskName "DigiText-MinIO-Tam"; Disable-ScheduledTask -TaskName "DigiText-MinIO-Tam" | Out-Null
  Get-CimInstance Win32_Process -Filter "Name='minio.exe'" | Where-Object { $_.CommandLine -like "*DigiText-KhoTam*" } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force }
  Ghi "đã tắt DigiText-MinIO-Tam"
}

# 2. Nạp kho.
Ghi "nạp kho từ $tar (lâu)…"
& python -I (Join-Path $repo "dev\dong-goi-kho.py") nap --dich http://127.0.0.1:9000 --thu-muc $tar 2>&1 | ForEach-Object { Ghi "  $_" }
if ($LASTEXITCODE -ne 0) { Dung "nạp có lỗi — chạy lại phần B để nạp nốt (tệp đã nạp được bỏ qua)" }

# 3. Bật API + worker.
foreach ($t in "DigiText-API", "DigiText-Worker") { Stop-ScheduledTask -TaskName $t -ErrorAction SilentlyContinue }
Get-Process digitext-api, digitext-ocr -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep 2
foreach ($t in "DigiText-API", "DigiText-Worker") { Start-ScheduledTask -TaskName $t; Ghi "đã chạy $t" }
Start-Sleep 15
$api = try { (Invoke-WebRequest -UseBasicParsing -TimeoutSec 15 http://127.0.0.1:8080/healthz).StatusCode } catch { $_.Exception.Message }
Ghi "API: $api; tiến trình: $((Get-Process digitext-api, digitext-ocr -ErrorAction SilentlyContinue | ForEach-Object Name) -join ', ')"
Ghi "=== Xong phần B. Kho DigiText chạy trên E:\DigiText-KhoMayChu (MinIO Windows, cổng 9000)."
