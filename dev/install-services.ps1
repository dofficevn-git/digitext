# Đăng ký API + worker DigiText chạy tự động khi Windows khởi động (không cần đăng nhập),
# tự chạy lại khi chết. Dùng Task Scheduler với tài khoản SYSTEM, gọi dev\service-supervisor.ps1.
# Chạy MỘT LẦN bằng PowerShell quyền admin:
#   powershell -ExecutionPolicy Bypass -File D:\DigiText\dev\install-services.ps1
# Gỡ:  powershell -ExecutionPolicy Bypass -File D:\DigiText\dev\install-services.ps1 -Uninstall
# Xem: Get-ScheduledTask DigiText-*  |  logs\api.log, logs\worker.log, logs\*.err.log
#
# Docker Desktop (Postgres/MinIO) lên chậm hơn task này vài chục giây — supervisor chờ và
# chạy lại nên không cần xếp thứ tự. Cloudflare tunnel là service riêng, đã tự khởi động.
param([switch]$Uninstall)
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$sup = Join-Path $root "dev\service-supervisor.ps1"
$tasks = @(
  @{ Name = "DigiText-API";    Role = "api";    Desc = "DigiText API :8080 (dev\service-supervisor.ps1 -Role api)" },
  @{ Name = "DigiText-Worker"; Role = "worker"; Desc = "DigiText worker OCR/PDF-A/đóng gói (dev\service-supervisor.ps1 -Role worker)" }
)
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) { throw "Cần PowerShell quyền Administrator." }

foreach ($t in $tasks) {
  $existing = Get-ScheduledTask -TaskName $t.Name -ErrorAction SilentlyContinue
  if ($existing) {
    Stop-ScheduledTask -TaskName $t.Name -ErrorAction SilentlyContinue
    Unregister-ScheduledTask -TaskName $t.Name -Confirm:$false
    Write-Host "-> đã gỡ task cũ $($t.Name)"
  }
  if ($Uninstall) { continue }
  $action = New-ScheduledTaskAction -Execute "powershell.exe" `
    -Argument "-NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File `"$sup`" -Role $($t.Role)" `
    -WorkingDirectory $root
  $trigger = New-ScheduledTaskTrigger -AtStartup
  $settings = New-ScheduledTaskSettingsSet -ExecutionTimeLimit ([TimeSpan]::Zero) -RestartCount 999 `
    -RestartInterval (New-TimeSpan -Minutes 1) -StartWhenAvailable -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries `
    -MultipleInstances IgnoreNew
  $principal = New-ScheduledTaskPrincipal -UserId "SYSTEM" -LogonType ServiceAccount -RunLevel Highest
  Register-ScheduledTask -TaskName $t.Name -Action $action -Trigger $trigger -Settings $settings -Principal $principal -Description $t.Desc | Out-Null
  Start-ScheduledTask -TaskName $t.Name
  Write-Host "-> đã đăng ký + khởi động $($t.Name)" -ForegroundColor Green
}
if ($Uninstall) { Write-Host "-> đã gỡ hết"; exit 0 }
Start-Sleep 8
Get-ScheduledTask DigiText-* | Select-Object TaskName, State | Format-Table -AutoSize
