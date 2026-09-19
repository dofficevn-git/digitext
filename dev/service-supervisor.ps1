# Giám sát MỘT tiến trình DigiText (api | worker) như một dịch vụ: nạp .env của crate,
# chạy binary release, tiến trình chết thì chờ rồi chạy lại. Dùng cho Scheduled Task
# "At startup" (SYSTEM) — xem dev\install-services.ps1. Chạy tay để thử:
#   powershell -ExecutionPolicy Bypass -File dev\service-supervisor.ps1 -Role api
#
# Vì sao không dùng dev\run-*.ps1: hai script đó gọi `cargo run` (biên dịch lại khi mở máy,
# cần cargo trong PATH của người dùng); dịch vụ phải chạy binary đã build, không phụ thuộc
# PATH, và tự hồi khi Postgres/MinIO (Docker) lên chậm hơn.
param(
  [Parameter(Mandatory = $true)][ValidateSet("api", "worker")][string]$Role
)
$ErrorActionPreference = "Continue"
$root = Split-Path -Parent $PSScriptRoot
$logDir = Join-Path $root "logs"
New-Item -ItemType Directory -Force $logDir | Out-Null
$log = Join-Path $logDir "$Role.log"

function Log($msg) {
  $line = "{0:yyyy-MM-dd HH:mm:ss} [supervisor] {1}" -f (Get-Date), $msg
  Add-Content -Path $log -Value $line -Encoding UTF8
}

switch ($Role) {
  "api"    { $crate = "digitext-api";  $exe = Join-Path $root "target\release\digitext-api.exe"; $exeArgs = @("serve") }
  "worker" { $crate = "digitext-ocr";  $exe = Join-Path $root "target\release\digitext-ocr.exe"; $exeArgs = @() }
}
$envFile = Join-Path $root "crates\$crate\.env"

while ($true) {
  if (-not (Test-Path $exe)) { Log "thiếu $exe — build: cargo build --release -p $crate"; Start-Sleep 60; continue }
  if (-not (Test-Path $envFile)) { Log "thiếu $envFile"; Start-Sleep 60; continue }
  # Nạp .env mỗi vòng để sửa cấu hình + chờ tiến trình chết là nhận.
  Get-Content $envFile | ForEach-Object {
    if ($_ -match '^\s*([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.*)$') {
      [Environment]::SetEnvironmentVariable($Matches[1], $Matches[2].Trim().Trim('"'), "Process")
    }
  }
  # Xoay log khi > 50 MB (giữ một bản cũ).
  if ((Test-Path $log) -and (Get-Item $log).Length -gt 50MB) {
    Move-Item $log "$log.1" -Force
  }
  Log "khởi động $exe $($exeArgs -join ' ')"
  $started = Get-Date
  try {
    # PowerShell 5.1 không nhận -ArgumentList rỗng → chỉ thêm khi có tham số.
    $sp = @{ FilePath = $exe; WorkingDirectory = $root; NoNewWindow = $true; PassThru = $true
             RedirectStandardOutput = (Join-Path $logDir "$Role.out.log")
             RedirectStandardError = (Join-Path $logDir "$Role.err.log") }
    if ($exeArgs.Count -gt 0) { $sp.ArgumentList = $exeArgs }
    $p = Start-Process @sp
    $p.WaitForExit()
    Log "tiến trình thoát mã $($p.ExitCode) sau $([int]((Get-Date) - $started).TotalSeconds)s"
  } catch {
    Log "không chạy được: $($_.Exception.Message)"
  }
  # Chết nhanh (Postgres chưa lên lúc mới mở máy…) thì chờ lâu hơn một chút.
  $wait = if (((Get-Date) - $started).TotalSeconds -lt 20) { 15 } else { 5 }
  Start-Sleep $wait
}
