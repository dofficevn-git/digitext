# Khởi động lại RIÊNG API digitext-api (Scheduled Task DigiText-API, chạy bằng SYSTEM) — worker OCR giữ nguyên,
# hàng OCR/PDF-A đang chạy không bị gián đoạn. Tự xin quyền admin (UAC) nếu chưa có:
#   powershell -ExecutionPolicy Bypass -File D:\DigiText\dev\restart-api.ps1
# Build khi API đang chạy: đổi tên exe trước rồi build (như restart-services.ps1):
#   Rename-Item target\release\digitext-api.exe digitext-api.exe.old
#   cargo build --release -p digitext-api
$ErrorActionPreference = "Continue"
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
  Start-Process powershell.exe -Verb RunAs -ArgumentList "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`""
  exit 0
}
$root = Split-Path -Parent $PSScriptRoot
$log = Join-Path $root "logs\restart-services.log"
function Log($m) { Add-Content -Path $log -Value ("{0:yyyy-MM-dd HH:mm:ss} {1}" -f (Get-Date), $m) -Encoding UTF8 }
Stop-ScheduledTask -TaskName "DigiText-API"; Log "đã dừng DigiText-API (riêng API)"
Get-Process digitext-api -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep 2
Start-ScheduledTask -TaskName "DigiText-API"; Log "đã chạy DigiText-API"
Start-Sleep 6
Get-Process digitext-api -ErrorAction SilentlyContinue | ForEach-Object { Log "đang chạy $($_.Name) pid=$($_.Id) từ $($_.Path)" }
Remove-Item (Join-Path $root "target\release\digitext-api.exe.old") -Force -ErrorAction SilentlyContinue
