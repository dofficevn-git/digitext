# Khởi động lại RIÊNG worker digitext-ocr (Scheduled Task DigiText-Worker, chạy bằng SYSTEM) — API giữ nguyên,
# phiên đẩy tệp từ máy trạm không bị gián đoạn. Tự xin quyền admin (UAC) nếu chưa có:
#   powershell -ExecutionPolicy Bypass -File D:\DigiText\dev\restart-worker.ps1
# Build khi worker đang chạy: đổi tên exe trước rồi build (như restart-services.ps1):
#   Rename-Item target\release\digitext-ocr.exe digitext-ocr.exe.old
#   cargo build --release -p digitext-ocr
$ErrorActionPreference = "Continue"
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
  Start-Process powershell.exe -Verb RunAs -ArgumentList "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`""
  exit 0
}
$root = Split-Path -Parent $PSScriptRoot
$log = Join-Path $root "logs\restart-services.log"
function Log($m) { Add-Content -Path $log -Value ("{0:yyyy-MM-dd HH:mm:ss} {1}" -f (Get-Date), $m) -Encoding UTF8 }
Stop-ScheduledTask -TaskName "DigiText-Worker"; Log "đã dừng DigiText-Worker (riêng worker)"
Get-Process digitext-ocr -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep 2
Start-ScheduledTask -TaskName "DigiText-Worker"; Log "đã chạy DigiText-Worker"
Start-Sleep 6
Get-Process digitext-ocr -ErrorAction SilentlyContinue | ForEach-Object { Log "đang chạy $($_.Name) pid=$($_.Id) từ $($_.Path)" }
Remove-Item (Join-Path $root "target\release\digitext-ocr.exe.old") -Force -ErrorAction SilentlyContinue
