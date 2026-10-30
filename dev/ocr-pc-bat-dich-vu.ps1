# BẬT toàn bộ dịch vụ DigiText trên OCR PC — chạy MỘT LẦN lúc cắt chuyển (GĐ2), PowerShell admin (30/10/2026).
#   powershell -ExecutionPolicy Bypass -File C:\DigiText\dev\ocr-pc-bat-dich-vu.ps1 -CfToken <token Zone.DNS:Edit> [-KhoDir D:\DigiText-KhoMayChu]
# Điều kiện trước: dữ liệu đã nạp (kho MinIO, CSDL), target\release đã build, image OCR/vLLM đã build/pull,
# .env đã sửa cho máy này (docs/CHUYEN-MAY-OCR-PC.md §3.2). Script chỉ đăng ký + khởi động, không chép dữ liệu.
# Thứ tự: MinIO → API + worker → Docker OCR + vLLM → Caddy (443) → watchdog → backup đêm. Dừng ở bước đầu lỗi.
param(
  [Parameter(Mandatory = $true)][string]$CfToken,
  [string]$KhoDir = "D:\DigiText-KhoMayChu",
  [string]$BackupDir = "C:\DigiText-Backup\csdl",
  [switch]$KhongGpu   # chưa có GPU (vllm-vnkie còn chạy) → bỏ qua bước OCR/vLLM
)
$ErrorActionPreference = "Stop"
$repo = Split-Path -Parent $PSScriptRoot
$log = Join-Path $repo "logs\ocr-pc-bat-dich-vu.log"
function Ghi($m) { $l = "{0:yyyy-MM-dd HH:mm:ss} {1}" -f (Get-Date), $m; Write-Host $l; Add-Content -Path $log -Value $l -Encoding UTF8 }
function Cho($url, $giay, $ten) {
  $han = (Get-Date).AddSeconds($giay)
  do { try { if ((Invoke-WebRequest -UseBasicParsing -TimeoutSec 5 $url).StatusCode -eq 200) { Ghi "  $ten OK ($url)"; return } } catch {}; Start-Sleep 3 } until ((Get-Date) -gt $han)
  throw "$ten không lên sau $giay s ($url)"
}
# Docker dưới task SYSTEM / SSH: bỏ credential helper (wincred cần phiên đăng nhập) — cùng mẹo như lúc dựng máy.
$env:Path = (($env:Path -split ";") | Where-Object { $_ -notlike "*Docker*" }) -join ";"
$env:DOCKER_CONFIG = "C:\DigiText-docker"
if (-not (Test-Path "$env:DOCKER_CONFIG\config.json")) { New-Item -ItemType Directory -Force $env:DOCKER_CONFIG | Out-Null; Set-Content "$env:DOCKER_CONFIG\config.json" "{}" -Encoding ascii }
$docker = "C:\Program Files\Docker\Docker\resources\bin\docker.exe"

Ghi "=== BẬT DIGITEXT TRÊN $env:COMPUTERNAME (repo $repo)"
# 0. Tiền đề
foreach ($f in "target\release\digitext-api.exe", "target\release\digitext-ocr.exe", "crates\digitext-api\.env", "crates\digitext-ocr\.env", "infra\compose\.env", "infra\minio-win\minio.exe", "jwt-ed25519.pem", "infra\caddy\Caddyfile") {
  if (-not (Test-Path (Join-Path $repo $f))) { throw "thiếu $f" }
}
if (-not (Test-Path "C:\DigiText-Caddy\caddy.exe")) { throw "thiếu C:\DigiText-Caddy\caddy.exe" }
# Không được còn MinIO tạm (phiên SSH) chiếm cổng 9000.
Get-Process minio -ErrorAction SilentlyContinue | ForEach-Object { Ghi "  dừng minio tạm PID $($_.Id)"; Stop-Process -Id $_.Id -Force }
Start-Sleep 2

# 1. Postgres + ClamAV (Docker) — thường đã chạy.
& $docker compose -f (Join-Path $repo "infra\compose\docker-compose.yml") up -d 2>&1 | Out-Null
$ok = & $docker exec digitext-dev-postgres-1 pg_isready -U digitext 2>&1; Ghi "1. Postgres: $ok"

# 2. MinIO (task SYSTEM, at startup)
Ghi "2. MinIO → $KhoDir :9000"
& powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $repo "dev\minio-win-task.ps1") -DataDir $KhoDir -Port 9000 | Out-Null
Cho "http://127.0.0.1:9000/minio/health/live" 60 "MinIO"

# 3. API + worker (task SYSTEM, supervisor)
Ghi "3. API + worker"
& powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $repo "dev\install-services.ps1") | Out-Null
Cho "http://127.0.0.1:8080/healthz" 120 "API"

# 4. OCR GPU + vLLM (Docker compose)
if ($KhongGpu) { Ghi "4. bỏ qua OCR/vLLM (-KhongGpu)" } else {
  Ghi "4. vLLM + OCR"
  & $docker compose -f (Join-Path $repo "infra\compose\vllm\docker-compose.yml") --env-file (Join-Path $repo "infra\compose\vllm\.env") up -d 2>&1 | Out-Null
  & $docker compose -f (Join-Path $repo "infra\compose\ocr\docker-compose.yml") -f (Join-Path $repo "infra\compose\ocr\docker-compose.gpu.yml") --env-file (Join-Path $repo "infra\compose\ocr\.env") up -d 2>&1 | Out-Null
  Cho "http://127.0.0.1:5012/api/ocr/health" 900 "OCR"
  Cho "http://127.0.0.1:8003/health" 1200 "vLLM"
  Cho "http://127.0.0.1:9095/readyz" 120 "worker thấy OCR"
}

# 5. Caddy (service, cổng 443, chứng chỉ DNS-01)
Ghi "5. Caddy"
[Environment]::SetEnvironmentVariable("CF_API_TOKEN", $CfToken, "Machine")
& C:\DigiText-Caddy\caddy.exe validate --config (Join-Path $repo "infra\caddy\Caddyfile") 2>&1 | Out-Null
if (-not (Get-Service DigiText-Caddy -ErrorAction SilentlyContinue)) {
  sc.exe create DigiText-Caddy binPath= "C:\DigiText-Caddy\caddy.exe run --config $repo\infra\caddy\Caddyfile" start= auto DisplayName= "DigiText Caddy (api.digitext.vn:443)" | Out-Null
}
if (-not (Get-NetFirewallRule -Name "DigiText-Caddy-443" -ErrorAction SilentlyContinue)) {
  New-NetFirewallRule -Name "DigiText-Caddy-443" -DisplayName "DigiText Caddy 443" -Direction Inbound -Protocol TCP -LocalPort 443 -Action Allow | Out-Null
}
Restart-Service DigiText-Caddy
Start-Sleep 5
Ghi "  Caddy: $((Get-Service DigiText-Caddy).Status) — chứng chỉ xin qua DNS-01, xem C:\DigiText-Caddy\caddy.log"

# 6. Watchdog mỗi phút + backup CSDL 01:00 (task SYSTEM)
Ghi "6. watchdog + backup"
$p = New-ScheduledTaskPrincipal -UserId SYSTEM -LogonType ServiceAccount -RunLevel Highest
if (-not (Get-ScheduledTask DigiText-Watchdog -ErrorAction SilentlyContinue)) {
  $a = New-ScheduledTaskAction -Execute powershell.exe -Argument "-NoProfile -ExecutionPolicy Bypass -File `"$repo\dev\watchdog.ps1`""
  $t = New-ScheduledTaskTrigger -Once -At (Get-Date).AddMinutes(1) -RepetitionInterval (New-TimeSpan -Minutes 1)
  $s = New-ScheduledTaskSettingsSet -ExecutionTimeLimit (New-TimeSpan -Minutes 5) -MultipleInstances IgnoreNew -StartWhenAvailable
  Register-ScheduledTask DigiText-Watchdog -Action $a -Trigger $t -Principal $p -Settings $s | Out-Null
}
if (-not (Get-ScheduledTask DigiText-Backup-CSDL -ErrorAction SilentlyContinue)) {
  New-Item -ItemType Directory -Force $BackupDir | Out-Null
  $a = New-ScheduledTaskAction -Execute powershell.exe -Argument "-NoProfile -ExecutionPolicy Bypass -File `"$repo\dev\backup-csdl.ps1`" -Dir `"$BackupDir`""
  $t = New-ScheduledTaskTrigger -Daily -At 01:00
  Register-ScheduledTask DigiText-Backup-CSDL -Action $a -Trigger $t -Principal $p | Out-Null
}
Get-ScheduledTask DigiText-* | Select-Object TaskName, State | Format-Table -AutoSize | Out-String | ForEach-Object { Ghi $_ }
Ghi "=== XONG. Tiếp: đổi DNS api.digitext.vn → A 113.190.254.219 (đám mây xám), NAT 443, rồi kiểm docs/CHUYEN-MAY-OCR-PC.md §5."
