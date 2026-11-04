# Dừng hẳn máy chủ DigiText trên máy .127 lúc cắt chuyển sang OCR PC (GĐ2, 2/11/2026): tắt task API + Worker,
# bỏ "chạy lúc khởi động" để không tự bật lại. GIỮ MinIO và Postgres (dump chốt, rollback 14 ngày), GIỮ cloudflared
# (trả DNS về tunnel nếu rollback). Tự xin quyền admin (UAC):
#   powershell -ExecutionPolicy Bypass -File D:\DigiText\dev\dung-dich-vu-127.ps1
$ErrorActionPreference = "Continue"
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
  Start-Process powershell.exe -Verb RunAs -ArgumentList "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`""
  exit 0
}
$root = Split-Path -Parent $PSScriptRoot
$log = Join-Path $root "logs\restart-services.log"
function Log($m) { Add-Content -Path $log -Value ("{0:yyyy-MM-dd HH:mm:ss} {1}" -f (Get-Date), $m) -Encoding UTF8 }
foreach ($t in "DigiText-API", "DigiText-Worker") {
  Stop-ScheduledTask -TaskName $t -ErrorAction SilentlyContinue
  Disable-ScheduledTask -TaskName $t -ErrorAction SilentlyContinue | Out-Null
  Log "CẮT CHUYỂN: đã dừng + vô hiệu task $t (bật lại: Enable-ScheduledTask $t; Start-ScheduledTask $t)"
}
Get-Process digitext-api, digitext-ocr -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep 2
$con = Get-Process digitext-api, digitext-ocr -ErrorAction SilentlyContinue
Log ("CẮT CHUYỂN: tiến trình còn lại: " + $(if ($con) { ($con | ForEach-Object { $_.Name }) -join "," } else { "không" }))
