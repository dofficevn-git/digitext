# Giám sát MỘT dịch vụ của máy chủ DigiText Lưu trữ (bản Lưu trữ GĐ2, 9/11/2026): nạp cấu hình trong conf\, chạy
# chương trình trong bin\, chết thì chờ rồi chạy lại. cai-dat.ps1 đăng ký mỗi vai một Scheduled Task "At startup" (SYSTEM).
# Chạy tay để thử:  powershell -ExecutionPolicy Bypass -File giam-sat.ps1 -Vai api
#
# Cùng cách với dev\service-supervisor.ps1 của máy chủ thi công: stdout/stderr ghi thẳng ra tệp bằng Start-Process
# (KHÔNG qua pipeline PowerShell — tiến trình từng chết vì PowerShell 5.1 coi mỗi dòng stderr là lỗi, 21/10/2026).
param(
  [Parameter(Mandatory = $true)][ValidateSet("api", "worker", "caddy")][string]$Vai
)
$ErrorActionPreference = "Continue"
$goc = $PSScriptRoot
$logDir = Join-Path $goc "logs"
New-Item -ItemType Directory -Force $logDir | Out-Null
$log = Join-Path $logDir "$Vai.log"

function Ghi($m) { Add-Content -Path $log -Value ("{0:yyyy-MM-dd HH:mm:ss} [giam-sat] {1}" -f (Get-Date), $m) -Encoding UTF8 }

function Nap-Env($tep) {
  $kv = @{}
  Get-Content $tep -Encoding UTF8 | ForEach-Object {
    if ($_ -match '^\s*([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.*)$') {
      $v = $Matches[2].Trim().Trim('"')
      [Environment]::SetEnvironmentVariable($Matches[1], $v, "Process")
      $kv[$Matches[1]] = $v
    }
  }
  return $kv
}

switch ($Vai) {
  "api"    { $exe = "bin\digitext-api.exe"; $envTep = "conf\api.env" }
  "worker" { $exe = "bin\digitext-ocr.exe"; $envTep = "conf\worker.env" }
  "caddy"  { $exe = "bin\caddy.exe";        $envTep = $null }
}
$exe = Join-Path $goc $exe

while ($true) {
  if (-not (Test-Path $exe)) { Ghi "thiếu $exe"; Start-Sleep 60; continue }
  $kv = @{}
  if ($envTep) {
    $p = Join-Path $goc $envTep
    if (-not (Test-Path $p)) { Ghi "thiếu $p — chạy lại cai-dat.ps1"; Start-Sleep 60; continue }
    $kv = Nap-Env $p
  }
  switch ($Vai) {
    "api"    { $thamSo = @("serve") }
    "worker" { $thamSo = @("run") }
    "caddy"  { $thamSo = @("run", "--config", (Join-Path $goc "conf\Caddyfile"), "--adapter", "caddyfile") }
  }
  foreach ($t in "$log", (Join-Path $logDir "$Vai.err.log")) {
    if ((Test-Path $t) -and (Get-Item $t).Length -gt 50MB) { Move-Item $t "$t.1" -Force }
  }
  Ghi "khởi động $exe $($thamSo -join ' ')"
  $bd = Get-Date
  try {
    $p = Start-Process -FilePath $exe -ArgumentList $thamSo -WorkingDirectory $goc -NoNewWindow -PassThru `
      -RedirectStandardOutput (Join-Path $logDir "$Vai.out.log") -RedirectStandardError (Join-Path $logDir "$Vai.err.log")
    $p.WaitForExit()
    Ghi "thoát mã $($p.ExitCode) sau $([int]((Get-Date) - $bd).TotalSeconds) s"
  } catch {
    Ghi "không chạy được: $($_.Exception.Message)"
  }
  # Chết ngay lúc mới mở máy (PostgreSQL chưa lên) → chờ lâu hơn.
  Start-Sleep $(if (((Get-Date) - $bd).TotalSeconds -lt 20) { 15 } else { 5 })
}
