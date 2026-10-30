# Đăng ký / đổi Scheduled Task "DigiText-MinIO" (SYSTEM, khi khởi động, tự chạy lại khi lỗi) gọi dev\minio-win.ps1.
# Ngày chuyển kho (docs/KE-HOACH-CHUYEN-KHO-MINIO.md):
#   bước 5.0 — chạy song song:  powershell -ExecutionPolicy Bypass -File dev\minio-win-task.ps1 -DataDir X:\DigiText-KhoMayChu -Port 9100
#   bước 5.4 — chuyển hẳn:      ... -DataDir X:\DigiText-KhoMayChu -Port 9000   (sau khi đã dừng MinIO trong Docker)
#   kho trung chuyển (17/10):   ... -DataDir F:\DigiText-KhoTam -Port 9200 -TaskName DigiText-MinIO-Tam
# Tự xin quyền admin (UAC). Loại trừ Windows Defender cho thư mục kho làm TAY trong Windows Security
# (script tự thêm loại trừ bị Defender coi là phần mềm độc hại — 15/10/2026).
param(
  [Parameter(Mandatory = $true)][string]$DataDir,
  [int]$Port = 9000,
  [string]$Exe = (Join-Path (Split-Path -Parent $PSScriptRoot) "infra\minio-win\minio.exe"),
  [string]$TaskName = "DigiText-MinIO"
)
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
  Start-Process powershell.exe -Verb RunAs -Wait -ArgumentList "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`" -DataDir `"$DataDir`" -Port $Port -Exe `"$Exe`" -TaskName `"$TaskName`""
  exit 0
}
$ErrorActionPreference = "Stop"
$repo = Split-Path -Parent $PSScriptRoot
$log = Join-Path $repo "logs\restart-services.log"
function Log($m) { Add-Content -Path $log -Value ("{0:yyyy-MM-dd HH:mm:ss} {1}" -f (Get-Date), $m) -Encoding UTF8 }

New-Item -ItemType Directory -Force -Path $DataDir | Out-Null
$script = Join-Path $repo "dev\minio-win.ps1"
$action = New-ScheduledTaskAction -Execute "powershell.exe" -Argument "-NoProfile -ExecutionPolicy Bypass -File `"$script`" -DataDir `"$DataDir`" -Port $Port -Exe `"$Exe`""
$trigger = New-ScheduledTaskTrigger -AtStartup
$settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -ExecutionTimeLimit ([TimeSpan]::Zero) `
  -RestartCount 999 -RestartInterval (New-TimeSpan -Minutes 1) -StartWhenAvailable
$principal = New-ScheduledTaskPrincipal -UserId "SYSTEM" -LogonType ServiceAccount -RunLevel Highest

if (Get-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue) {
  Stop-ScheduledTask -TaskName $TaskName
  # Chỉ dừng đúng tiến trình minio của kho này (có thể có nhiều MinIO cùng chạy).
  Get-CimInstance Win32_Process -Filter "Name='minio.exe'" | Where-Object { $_.CommandLine -like "*$DataDir*" } |
    ForEach-Object { Stop-Process -Id $_.ProcessId -Force }
  Start-Sleep 2
  Set-ScheduledTask -TaskName $TaskName -Action $action -Trigger $trigger -Settings $settings -Principal $principal | Out-Null
} else {
  Register-ScheduledTask -TaskName $TaskName -Action $action -Trigger $trigger -Settings $settings -Principal $principal | Out-Null
}
Start-ScheduledTask -TaskName $TaskName
Start-Sleep 8
$ok = try { (Invoke-WebRequest -UseBasicParsing -TimeoutSec 5 "http://127.0.0.1:$Port/minio/health/live").StatusCode } catch { $_.Exception.Message }
Log "$TaskName cổng $Port, dữ liệu $DataDir — health: $ok"
