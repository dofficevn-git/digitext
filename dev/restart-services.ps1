# Khởi động lại API + worker DigiText (Scheduled Task chạy bằng SYSTEM) sau khi build bản mới.
# Tự xin quyền admin (UAC) nếu chưa có:
#   powershell -ExecutionPolicy Bypass -File D:\DigiText\dev\restart-services.ps1
#
# Build khi dịch vụ đang chạy: exe đang bị khoá nên ĐỔI TÊN nó trước (Windows cho đổi tên
# tệp đang chạy), build, rồi chạy script này:
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
foreach ($t in "DigiText-API", "DigiText-Worker") { Stop-ScheduledTask -TaskName $t; Log "đã dừng $t" }
Get-Process digitext-api, digitext-ocr -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep 2
foreach ($t in "DigiText-API", "DigiText-Worker") { Start-ScheduledTask -TaskName $t; Log "đã chạy $t" }
Start-Sleep 6
Get-Process digitext-api, digitext-ocr -ErrorAction SilentlyContinue | ForEach-Object { Log "đang chạy $($_.Name) pid=$($_.Id) từ $($_.Path)" }
Remove-Item (Join-Path $root "target\release\*.exe.old") -Force -ErrorAction SilentlyContinue
