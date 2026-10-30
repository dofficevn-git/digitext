# Canh dịch vụ DigiText trên máy chủ (OCR PC) — chạy mỗi phút bằng Scheduled Task "DigiText-Watchdog" (SYSTEM).
# Thay alienware-autoheal.sh (restart container unhealthy) + yêu cầu "watchdog mạng" sau các lần Wi-Fi rớt làm
# API mất kết nối ra ngoài trên máy .127 (10/2026). Chỉ khởi động lại thứ đã hỏng; mỗi lần làm gì đều ghi logs\watchdog.log.
#
# Đăng ký (PowerShell admin):
#   $a = New-ScheduledTaskAction -Execute powershell.exe -Argument "-NoProfile -ExecutionPolicy Bypass -File D:\DigiText\dev\watchdog.ps1"
#   $t = New-ScheduledTaskTrigger -Once -At (Get-Date) -RepetitionInterval (New-TimeSpan -Minutes 1)
#   $p = New-ScheduledTaskPrincipal -UserId SYSTEM -LogonType ServiceAccount -RunLevel Highest
#   $s = New-ScheduledTaskSettingsSet -ExecutionTimeLimit (New-TimeSpan -Minutes 5) -MultipleInstances IgnoreNew -StartWhenAvailable
#   Register-ScheduledTask DigiText-Watchdog -Action $a -Trigger $t -Principal $p -Settings $s
#
# Kiểm theo thứ tự: API tại chỗ → API qua tên miền (Caddy + NAT 443 + DNS, không còn tunnel) → container Docker
# unhealthy → worker OCR /readyz → MinIO. Không đụng Postgres (Docker restart policy tự lo).
param(
  [string]$ApiLocal = "http://127.0.0.1:8080/healthz",
  [string]$ApiPublic = "https://api.digitext.vn/healthz",
  [string]$WorkerReady = "http://127.0.0.1:9095/readyz",
  [string]$MinioLive = "http://127.0.0.1:9000/minio/health/live",
  [string]$CaddyService = "DigiText-Caddy",
  # Số lần hỏng LIÊN TIẾP trước khi khởi động lại (tránh khởi động lại vì một lần mạng khựng).
  [int]$Nguong = 3
)
$ErrorActionPreference = "Continue"
$repo = Split-Path -Parent $PSScriptRoot
$log = Join-Path $repo "logs\watchdog.log"
$trangThai = Join-Path $repo "logs\watchdog-state.json"
function Ghi($m) { Add-Content -Path $log -Value ("{0:yyyy-MM-dd HH:mm:ss} {1}" -f (Get-Date), $m) -Encoding UTF8 }
function Kiem($url, $giay = 15) {
  try { return (Invoke-WebRequest -UseBasicParsing -TimeoutSec $giay $url).StatusCode -eq 200 } catch { return $false }
}

$st = @{}
if (Test-Path $trangThai) {
  try { (Get-Content $trangThai -Raw | ConvertFrom-Json).PSObject.Properties | ForEach-Object { $st[$_.Name] = [int]$_.Value } } catch {}
}
function Hong($ten) { $st[$ten] = ($st[$ten] + 1); return $st[$ten] }
function Tot($ten) { if ($st[$ten] -gt 0) { Ghi "$ten đã trở lại sau $($st[$ten]) lần hỏng" }; $st[$ten] = 0 }
function KhoiDongLaiService($ten) {
  $svc = Get-Service $ten -ErrorAction SilentlyContinue
  if (-not $svc) { Ghi "  không có service $ten"; return }
  Ghi "  khởi động lại service $ten (đang $($svc.Status))"
  try { Restart-Service $ten -Force -ErrorAction Stop } catch { Ghi "  lỗi: $_" }
}

# 1. API tại chỗ — task DigiText-API (dev/service-supervisor.ps1) tự chạy lại; ở đây chỉ ghi nhận và đá task nếu chết hẳn.
if (Kiem $ApiLocal 10) { Tot "api" } else {
  $n = Hong "api"
  Ghi "API tại chỗ không trả lời (lần $n)"
  if ($n -ge $Nguong) {
    $t = Get-ScheduledTask -TaskName "DigiText-API" -ErrorAction SilentlyContinue
    if ($t -and $t.State -ne "Running") { Ghi "  task DigiText-API đang $($t.State) → Start"; Start-ScheduledTask -TaskName "DigiText-API" }
  }
}

# 2. API qua tên miền: hỏng mà API tại chỗ tốt → Caddy / NAT / DNS. Caddy là thứ duy nhất ở đây sửa được.
if ($st["api"] -eq 0) {
  if (Kiem $ApiPublic) { Tot "public" } else {
    $n = Hong "public"
    Ghi "API qua $ApiPublic không tới được (lần $n)"
    if ($n -ge $Nguong -and ($n % $Nguong) -eq 0) { KhoiDongLaiService $CaddyService }
  }
}

# 3. Container Docker unhealthy (OCR, vLLM, ClamAV…) — như alienware-autoheal.sh.
$unhealthy = @(docker ps --filter health=unhealthy --format "{{.Names}}" 2>$null | Where-Object { $_ })
foreach ($c in $unhealthy) {
  $n = Hong "docker:$c"
  Ghi "container $c unhealthy (lần $n)"
  if ($n -ge $Nguong) { Ghi "  docker restart $c"; docker restart $c 2>&1 | Out-Null; $st["docker:$c"] = 0 }
}
foreach ($k in @($st.Keys)) { if ($k -like "docker:*" -and $unhealthy -notcontains $k.Substring(7)) { Tot $k } }

# 4. Worker OCR: /readyz 200 = worker lên VÀ thấy dịch vụ OCR.
if (Kiem $WorkerReady 20) { Tot "worker" } else {
  $n = Hong "worker"
  Ghi "worker /readyz không đạt (lần $n)"
  if ($n -ge $Nguong) {
    $t = Get-ScheduledTask -TaskName "DigiText-Worker" -ErrorAction SilentlyContinue
    if ($t -and $t.State -ne "Running") { Ghi "  task DigiText-Worker đang $($t.State) → Start"; Start-ScheduledTask -TaskName "DigiText-Worker" }
  }
}

# 5. MinIO (task DigiText-MinIO tự chạy lại; chỉ ghi nhận).
if (Kiem $MinioLive 10) { Tot "minio" } else { $n = Hong "minio"; Ghi "MinIO không trả lời (lần $n)" }

$st | ConvertTo-Json -Compress | Set-Content -Path $trangThai -Encoding UTF8
