# CÀI MÁY CHỦ DIGITEXT LƯU TRỮ TRÊN WINDOWS (bản Lưu trữ GĐ2, 9/11/2026 — docs/CAI-DAT-MAY-CHU-LUU-TRU.md).
# Chạy bằng PowerShell QUYỀN QUẢN TRỊ trong thư mục đã giải nén gói luutru-server-win:
#   powershell -ExecutionPolicy Bypass -File .\cai-dat.ps1 -DiaChi luutru.co-quan.vn            (có tên miền, Let's Encrypt)
#   powershell -ExecutionPolicy Bypass -File .\cai-dat.ps1 -DiaChi 192.168.1.20                  (mạng nội bộ, chứng chỉ tự cấp)
# Gỡ (giữ nguyên dữ liệu, CSDL, kho):  .\cai-dat.ps1 -Go
#
# Điều kiện trước: PostgreSQL 15–17 đã cài bằng bộ cài chuẩn (https://www.postgresql.org/download/windows/), nhớ mật
# khẩu tài khoản postgres. Chạy lại script nhiều lần được: cấu hình đã có (conf\*.env, mật khẩu) giữ nguyên.
#
# Các bước: tạo CSDL + tài khoản digitext → sinh cấu hình conf\ → đăng ký 3 dịch vụ (API, worker, Caddy) chạy khi mở máy
# → áp migrations → tạo tài khoản quản trị đầu tiên → mở tường lửa 80/443 → chứng chỉ gốc cho máy trạm (mạng nội bộ)
# → sao lưu 01:00 hằng đêm.
param(
  [string]$DiaChi = "",                       # tên miền hoặc tên máy / IP máy trạm dùng để kết nối
  [string]$KhoDir = "D:\DigiText-Kho",        # kho tài liệu dạng thư mục (tài liệu, gói AIP) — ổ đủ lớn
  [string]$SaoLuuDir = "E:\DigiText-SaoLuu",  # nên ở ổ khác ổ kho
  [string]$PgBin = "",                        # để trống: tự tìm C:\Program Files\PostgreSQL\<bản>\bin
  [string]$PgCong = "5432",
  [string]$QuanTri = "quantri",               # tên đăng nhập quản trị đầu tiên
  [string]$MaDuAn = "LUUTRU",                 # mã kho lưu trữ (dự án) tạo lần đầu — dùng ở lệnh nhap-goi --du-an
  [string]$TenCoQuan = "",                    # tên cơ quan / kho lưu trữ, vd "Lưu trữ lịch sử tỉnh Bắc Giang"
  [switch]$TlsNoiBo,                          # ép dùng chứng chỉ tự cấp kể cả khi DiaChi là tên miền
  [switch]$Go
)
$ErrorActionPreference = "Continue"
$goc = $PSScriptRoot
$conf = Join-Path $goc "conf"
$logDir = Join-Path $goc "logs"
New-Item -ItemType Directory -Force $conf, $logDir, (Join-Path $goc "data"), (Join-Path $goc "updates") | Out-Null
$logTep = Join-Path $logDir "cai-dat.log"
function Ghi($m, $mau = "Gray") { $l = "{0:yyyy-MM-dd HH:mm:ss} {1}" -f (Get-Date), $m; Write-Host $l -ForegroundColor $mau; Add-Content -Path $logTep -Value $l -Encoding UTF8 }
function Dung($m) { Ghi "LỖI: $m" Red; exit 1 }
function Cho($url, $giay, $ten) {
  $han = (Get-Date).AddSeconds($giay)
  do { try { if ((Invoke-WebRequest -UseBasicParsing -TimeoutSec 5 $url).StatusCode -eq 200) { Ghi "  $ten đã chạy"; return $true } } catch {}; Start-Sleep 3 } until ((Get-Date) -gt $han)
  return $false
}
function Ngau-Nhien($n) {
  $chu = [char[]]"ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz23456789"
  $b = New-Object byte[] $n
  [System.Security.Cryptography.RandomNumberGenerator]::Create().GetBytes($b)
  -join ($b | ForEach-Object { $chu[$_ % $chu.Length] })
}
function Doc-Env($tep) {
  $kv = [ordered]@{}
  if (Test-Path $tep) { Get-Content $tep -Encoding UTF8 | ForEach-Object { if ($_ -match '^\s*([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.*)$') { $kv[$Matches[1]] = $Matches[2].Trim().Trim('"') } } }
  return $kv
}
function Ghi-Env($tep, $kv, $dau) {
  $dong = @("# $dau", "# Sinh bởi cai-dat.ps1 lúc $(Get-Date -Format 'dd/MM/yyyy HH:mm'). Sửa xong thì khởi động lại dịch vụ (Restart-ScheduledTask hoặc khởi động lại máy).")
  foreach ($k in $kv.Keys) { $dong += "$k=$($kv[$k])" }
  Set-Content -Path $tep -Value $dong -Encoding UTF8
}

$laAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $laAdmin) { Dung "cần PowerShell quyền Administrator (chuột phải → Run as administrator)." }
# Kho tài liệu là thư mục (FsObjectStore, ghi tệp tạm rồi đổi tên) — không dùng MinIO: máy chủ một máy không cần, bớt một
# dịch vụ + một bộ mật khẩu, và MinIO trên Windows từng chết theo stderr (21/10/2026).
$dichVu = @(
  @{ Ten = "DigiText-LuuTru-API";    Vai = "api";    Mo = "Máy chủ DigiText Lưu trữ (127.0.0.1:8080)" },
  @{ Ten = "DigiText-LuuTru-Worker"; Vai = "worker"; Mo = "Việc nền lưu trữ: đóng gói DIP, kiểm toàn vẹn, tiêu huỷ" },
  @{ Ten = "DigiText-LuuTru-Caddy";  Vai = "caddy";  Mo = "Cổng HTTPS 443 (Caddy)" }
)

if ($Go) {
  foreach ($d in $dichVu + @(@{ Ten = "DigiText-LuuTru-SaoLuu" }, @{ Ten = "DigiText-LuuTru-MinIO" })) {
    if (Get-ScheduledTask -TaskName $d.Ten -ErrorAction SilentlyContinue) {
      Stop-ScheduledTask -TaskName $d.Ten -ErrorAction SilentlyContinue
      Unregister-ScheduledTask -TaskName $d.Ten -Confirm:$false
      Ghi "đã gỡ $($d.Ten)"
    }
  }
  Get-Process digitext-api, digitext-ocr, caddy -ErrorAction SilentlyContinue | Where-Object { $_.Path -like "$goc\*" } | Stop-Process -Force
  Ghi "Đã gỡ dịch vụ. Dữ liệu (CSDL, $KhoDir, conf\) giữ nguyên." Green
  exit 0
}

Ghi "=== CÀI DIGITEXT LƯU TRỮ tại $goc" Cyan
foreach ($f in "bin\digitext-api.exe", "bin\digitext-ocr.exe", "bin\caddy.exe", "giam-sat.ps1", "sao-luu.ps1") {
  if (-not (Test-Path (Join-Path $goc $f))) { Dung "gói thiếu $f — giải nén lại luutru-server-win.zip" }
}
if (-not (Get-ChildItem (Join-Path $goc "migrations") -Filter "*.sql" -ErrorAction SilentlyContinue)) { Dung "gói thiếu thư mục migrations" }

# --- 0. Cấu hình chung (giữ giá trị cũ khi chạy lại)
$mc = Doc-Env (Join-Path $conf "may-chu.env")
if (-not $DiaChi) { $DiaChi = $mc["DIA_CHI"] }
if (-not $DiaChi) { Dung "thiếu -DiaChi (tên miền, hoặc tên máy / IP trong mạng nội bộ mà máy trạm dùng để kết nối)" }
$DiaChi = $DiaChi.Trim().ToLower() -replace '^https?://', '' -replace '/.*$', ''
$laIp = $DiaChi -match '^\d{1,3}(\.\d{1,3}){3}$'
$noiBo = $TlsNoiBo -or $laIp -or ($DiaChi -notmatch '\.') -or ($DiaChi -match '\.(local|lan|internal|home|corp)$')
if (-not $PgBin) { $PgBin = $mc["PG_BIN"] }
if (-not $PgBin) {
  $PgBin = Get-ChildItem "C:\Program Files\PostgreSQL" -Directory -ErrorAction SilentlyContinue | Sort-Object { [int]($_.Name -replace '\D', '') } -Descending |
    ForEach-Object { Join-Path $_.FullName "bin" } | Where-Object { Test-Path (Join-Path $_ "psql.exe") } | Select-Object -First 1
}
if (-not $PgBin -or -not (Test-Path (Join-Path $PgBin "psql.exe"))) { Dung "không thấy PostgreSQL (psql.exe). Cài PostgreSQL 16 bản Windows trước, hoặc chỉ đường bằng -PgBin." }
$psql = Join-Path $PgBin "psql.exe"
$banPg = (& $psql --version) -replace '.*?(\d+)\..*', '$1'
Ghi "PostgreSQL: $PgBin (bản $banPg) · địa chỉ: $DiaChi · chứng chỉ: $(if ($noiBo) { 'tự cấp (mạng nội bộ)' } else { "Let's Encrypt" })"
if ([int]$banPg -lt 15) { Dung "cần PostgreSQL 15 trở lên (đang là $banPg) — để CSDL dùng sắp xếp tiếng Việt ICU" }
New-Item -ItemType Directory -Force $KhoDir, $SaoLuuDir | Out-Null
if (-not $mc["MA_DU_AN"]) { $mc["MA_DU_AN"] = $MaDuAn.Trim().ToUpper() }; if (-not $mc["QUAN_TRI"]) { $mc["QUAN_TRI"] = $QuanTri }
$mc["DIA_CHI"] = $DiaChi; $mc["PG_BIN"] = $PgBin; $mc["KHO_DIR"] = (Resolve-Path $KhoDir).Path; $mc["SAO_LUU_DIR"] = (Resolve-Path $SaoLuuDir).Path
Ghi-Env (Join-Path $conf "may-chu.env") $mc "Thông số máy chủ (không bí mật)"

# --- 1. CSDL + tài khoản digitext
$api = Doc-Env (Join-Path $conf "api.env")
$mkDb = if ($api["DATABASE_URL"] -match '^postgres://digitext:([^@]+)@') { $Matches[1] } else { Ngau-Nhien 24 }
$dbUrl = "postgres://digitext:$mkDb@127.0.0.1:$PgCong/digitext"
$coSan = $false
$env:PGPASSWORD = $mkDb
& $psql -X -q -h 127.0.0.1 -p $PgCong -U digitext -d digitext -c "SELECT 1" 2>$null | Out-Null
if ($LASTEXITCODE -eq 0) { $coSan = $true; Ghi "  CSDL digitext đã có, kết nối được" }
if (-not $coSan) {
  $mkPg = Read-Host "Mật khẩu tài khoản 'postgres' (đặt lúc cài PostgreSQL)" -AsSecureString
  $env:PGPASSWORD = [Runtime.InteropServices.Marshal]::PtrToStringAuto([Runtime.InteropServices.Marshal]::SecureStringToBSTR($mkPg))
  $taoRole = "DO `$`$ BEGIN IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'digitext') THEN CREATE ROLE digitext LOGIN PASSWORD '$mkDb'; ELSE ALTER ROLE digitext LOGIN PASSWORD '$mkDb'; END IF; END `$`$;"
  & $psql -X -q -h 127.0.0.1 -p $PgCong -U postgres -d postgres -v ON_ERROR_STOP=1 -c $taoRole
  if ($LASTEXITCODE -ne 0) { Dung "không tạo được tài khoản digitext (sai mật khẩu postgres?)" }
  $coDb = & $psql -X -t -A -h 127.0.0.1 -p $PgCong -U postgres -d postgres -c "SELECT 1 FROM pg_database WHERE datname = 'digitext'"
  if ("$coDb".Trim() -ne "1") {
    & $psql -X -q -h 127.0.0.1 -p $PgCong -U postgres -d postgres -v ON_ERROR_STOP=1 -c "CREATE DATABASE digitext OWNER digitext TEMPLATE template0 ENCODING 'UTF8' LOCALE_PROVIDER icu ICU_LOCALE 'vi-VN' LOCALE 'C'"
    if ($LASTEXITCODE -ne 0) { Dung "không tạo được CSDL digitext" }
  }
  Ghi "  đã tạo tài khoản + CSDL digitext"
}
Remove-Item Env:PGPASSWORD -ErrorAction SilentlyContinue

# --- 2. Cấu hình dịch vụ
$s3 = [ordered]@{ DIGITEXT_PDF_STORE_DIR = $mc["KHO_DIR"] }
$boKhoa = "DIGITEXT_MINIO_ENDPOINT", "DIGITEXT_MINIO_BUCKET", "DIGITEXT_MINIO_ACCESS_KEY", "DIGITEXT_MINIO_SECRET_KEY", "DIGITEXT_PDF_STORE_BUCKET"
$api["DATABASE_URL"] = $dbUrl
$api["DIGITEXT_EDITION"] = "luu_tru"
$api["DIGITEXT_BIND"] = "127.0.0.1:8080"
$api["DIGITEXT_BEHIND_PROXY"] = "true"
$api["DIGITEXT_PUBLIC_URL"] = "https://$DiaChi"
$api["DIGITEXT_DATA_DIR"] = Join-Path $goc "data"
$api["DIGITEXT_UPDATES_DIR"] = Join-Path $goc "updates"
$api["DIGITEXT_JWT_KEY_PATH"] = Join-Path $goc "data\jwt-ed25519.pem"
$api["DIGITEXT_MIGRATIONS_DIR"] = Join-Path $goc "migrations"
foreach ($k in $s3.Keys) { $api[$k] = $s3[$k] }
foreach ($k in $boKhoa) { $api.Remove($k) }
Ghi-Env (Join-Path $conf "api.env") $api "API DigiText Lưu trữ"
$wk = Doc-Env (Join-Path $conf "worker.env")
$wk["DIGITEXT_DATABASE_URL"] = $dbUrl
$wk["DIGITEXT_CHI_LUU_TRU"] = "true"
$wk["DIGITEXT_METRICS_ADDR"] = "127.0.0.1:9100"
foreach ($k in $s3.Keys) { $wk[$k] = $s3[$k] }
foreach ($k in $boKhoa) { $wk.Remove($k) }
Ghi-Env (Join-Path $conf "worker.env") $wk "Worker — chế độ chỉ lưu trữ (không OCR)"
$dataCaddy = Join-Path $goc "caddy-data"
$tls = if ($noiBo) { "`n`ttls internal" } else { "" }
$http = if ($noiBo) {
@"

# Mạng nội bộ: cổng 80 chỉ phát chứng chỉ gốc + tệp cài cho máy trạm chưa tin HTTPS; còn lại chuyển sang HTTPS.
http://$DiaChi {
`t@chungchi path /updates/chung-chi-goc.crt /updates/cai-chung-chi-goc.cmd /tai-ve
`thandle @chungchi {
`t`treverse_proxy 127.0.0.1:8080
`t}
`t# redir đứng trước handle trong thứ tự directive của Caddy → phải bọc vào handle dự phòng.
`thandle {
`t`tredir https://{host}{uri}
`t}
}
"@ } else { "" }
$caddy = @"
# Sinh bởi cai-dat.ps1 — Caddy trước API DigiText Lưu trữ (API chỉ nghe 127.0.0.1:8080).
{
`tstorage file_system {
`t`troot $dataCaddy
`t}
`tlog {
`t`toutput file $logDir\caddy-truy-cap.log {
`t`t`troll_size 50MiB
`t`t`troll_keep 5
`t`t}
`t}
}

$DiaChi {$tls
`treverse_proxy 127.0.0.1:8080 {
`t`ttransport http {
`t`t`tread_timeout 30m
`t`t`twrite_timeout 30m
`t`t}
`t}
}
$http
"@
Set-Content -Path (Join-Path $conf "Caddyfile") -Value $caddy -Encoding UTF8
& (Join-Path $goc "bin\caddy.exe") validate --config (Join-Path $conf "Caddyfile") --adapter caddyfile 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) { Dung "Caddyfile sinh ra không hợp lệ — xem conf\Caddyfile" }
Ghi "  đã ghi conf\ (api.env, worker.env, Caddyfile)"

# --- 3. Dịch vụ
function Dang-Ky($d) {
  if (Get-ScheduledTask -TaskName $d.Ten -ErrorAction SilentlyContinue) {
    Stop-ScheduledTask -TaskName $d.Ten -ErrorAction SilentlyContinue
    Unregister-ScheduledTask -TaskName $d.Ten -Confirm:$false
  }
  $a = New-ScheduledTaskAction -Execute "powershell.exe" -WorkingDirectory $goc `
    -Argument "-NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File `"$(Join-Path $goc 'giam-sat.ps1')`" -Vai $($d.Vai)"
  $s = New-ScheduledTaskSettingsSet -ExecutionTimeLimit ([TimeSpan]::Zero) -RestartCount 999 -RestartInterval (New-TimeSpan -Minutes 1) `
    -StartWhenAvailable -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -MultipleInstances IgnoreNew
  $p = New-ScheduledTaskPrincipal -UserId "SYSTEM" -LogonType ServiceAccount -RunLevel Highest
  Register-ScheduledTask -TaskName $d.Ten -Action $a -Trigger (New-ScheduledTaskTrigger -AtStartup) -Settings $s -Principal $p -Description $d.Mo | Out-Null
  Start-ScheduledTask -TaskName $d.Ten
  Ghi "  đã đăng ký + chạy $($d.Ten)"
}
Get-Process digitext-api, digitext-ocr, caddy -ErrorAction SilentlyContinue | Where-Object { $_.Path -like "$goc\*" } | Stop-Process -Force

# --- 4. Migrations (trước API)
$env:DATABASE_URL = $dbUrl
& (Join-Path $goc "bin\digitext-api.exe") migrate --thu-muc (Join-Path $goc "migrations")
if ($LASTEXITCODE -ne 0) { Dung "áp migrations lỗi (xem dòng lỗi ở trên)" }
Remove-Item Env:DATABASE_URL -ErrorAction SilentlyContinue

Dang-Ky $dichVu[0]
if (-not (Cho "http://127.0.0.1:8080/healthz" 90 "API")) { Dung "API không chạy — xem logs\api.err.log" }
Dang-Ky $dichVu[1]

# --- 5. Tài khoản quản trị đầu tiên (chỉ khi CSDL chưa có người dùng)
$env:PGPASSWORD = $mkDb
$soNguoi = & $psql -X -t -A -h 127.0.0.1 -p $PgCong -U digitext -d digitext -c "SELECT count(*) FROM users"
Remove-Item Env:PGPASSWORD -ErrorAction SilentlyContinue
if ("$soNguoi".Trim() -eq "0") {
  do {
    $m1 = [Runtime.InteropServices.Marshal]::PtrToStringAuto([Runtime.InteropServices.Marshal]::SecureStringToBSTR((Read-Host "Mật khẩu cho tài khoản quản trị '$QuanTri' (tối thiểu 8 ký tự)" -AsSecureString)))
    $m2 = [Runtime.InteropServices.Marshal]::PtrToStringAuto([Runtime.InteropServices.Marshal]::SecureStringToBSTR((Read-Host "Nhập lại" -AsSecureString)))
    if ($m1 -ne $m2) { Write-Host "Hai lần nhập không khớp." -ForegroundColor Yellow } elseif ($m1.Length -lt 8) { Write-Host "Tối thiểu 8 ký tự." -ForegroundColor Yellow }
  } until ($m1 -eq $m2 -and $m1.Length -ge 8)
  $tam = Join-Path $env:TEMP ("dt-" + [guid]::NewGuid().ToString("N") + ".txt")
  [IO.File]::WriteAllText($tam, $m1)
  foreach ($k in $api.Keys) { [Environment]::SetEnvironmentVariable($k, $api[$k], "Process") }
  & (Join-Path $goc "bin\digitext-api.exe") bootstrap-superadmin --username $QuanTri --display-name "Quản trị hệ thống" --password-file $tam
  $ma = $LASTEXITCODE
  Remove-Item $tam -Force
  $m1 = $null; $m2 = $null
  if ($ma -ne 0) { Dung "không tạo được tài khoản quản trị" }
  Ghi "  đã tạo tài khoản quản trị '$QuanTri'" Green
} else {
  Ghi "  CSDL đã có $("$soNguoi".Trim()) người dùng — không tạo quản trị mới"
}
# Kho lưu trữ (dự án) đầu tiên — gói AIP nhập vào đây. Chỉ tạo khi CSDL chưa có dự án nào.
$env:PGPASSWORD = $mkDb
$soDuAn = & $psql -X -t -A -h 127.0.0.1 -p $PgCong -U digitext -d digitext -c "SELECT count(*) FROM projects"
if ("$soDuAn".Trim() -eq "0") {
  if (-not $TenCoQuan) { $TenCoQuan = Read-Host "Tên cơ quan / kho lưu trữ (vd: Lưu trữ lịch sử tỉnh ...)" }
  if (-not $TenCoQuan) { $TenCoQuan = "Kho lưu trữ" }
  $ma = $mc["MA_DU_AN"] -replace "'", ""
  $ten = $TenCoQuan.Trim() -replace "'", "''"
  & $psql -X -q -h 127.0.0.1 -p $PgCong -U digitext -d digitext -v ON_ERROR_STOP=1 -c "INSERT INTO projects (id, code, name) VALUES (gen_random_uuid(), '$ma', '$ten')"
  if ($LASTEXITCODE -eq 0) { Ghi "  đã tạo kho lưu trữ $ma — $TenCoQuan" Green } else { Ghi "  CẢNH BÁO: không tạo được dự án $ma" Yellow }
}
Remove-Item Env:PGPASSWORD -ErrorAction SilentlyContinue

# --- 6. Caddy + tường lửa
foreach ($cong in 443, 80) {
  $ten = "DigiText Luu tru $cong"
  if (-not (Get-NetFirewallRule -DisplayName $ten -ErrorAction SilentlyContinue)) {
    New-NetFirewallRule -DisplayName $ten -Direction Inbound -Protocol TCP -LocalPort $cong -Action Allow | Out-Null
    Ghi "  đã mở tường lửa cổng $cong"
  }
}
Dang-Ky $dichVu[2]

# --- 7. Chứng chỉ gốc cho máy trạm (chỉ mạng nội bộ)
if ($noiBo) {
  $root = Join-Path $dataCaddy "pki\authorities\local\root.crt"
  $han = (Get-Date).AddSeconds(60)
  while (-not (Test-Path $root) -and (Get-Date) -lt $han) { Start-Sleep 2 }
  if (Test-Path $root) {
    Copy-Item $root (Join-Path $goc "updates\chung-chi-goc.crt") -Force
    # Tệp .cmd tự chứa chứng chỉ: 5 dòng lệnh đầu, phần còn lại là PEM (more +5 cắt ra). certutil -user: kho của người
    # dùng hiện tại, không cần quyền quản trị; Windows hỏi xác nhận một lần.
    $cmd = @(
      '@echo off',
      'set "F=%TEMP%\digitext-chung-chi-goc.crt"',
      'more +5 "%~f0" > "%F%"',
      'certutil -user -addstore -f Root "%F%" && (echo Da cai chung chi goc may chu DigiText Luu tru.) || (echo Khong cai duoc chung chi.)',
      'del "%F%" & pause & exit /b'
    ) + (Get-Content $root)
    Set-Content -Path (Join-Path $goc "updates\cai-chung-chi-goc.cmd") -Value $cmd -Encoding ASCII
    # Cài luôn cho chính máy chủ (để kiểm tra bằng trình duyệt trên máy này).
    certutil -addstore -f Root $root | Out-Null
    Ghi "  chứng chỉ gốc: updates\chung-chi-goc.crt + cai-chung-chi-goc.cmd (máy trạm tải ở https://$DiaChi/tai-ve hoặc http://$DiaChi/tai-ve)"
  } else {
    Ghi "  CẢNH BÁO: Caddy chưa sinh chứng chỉ gốc sau 60 s — xem logs\caddy.err.log, chạy lại cai-dat.ps1" Yellow
  }
}
if (-not (Cho "https://$DiaChi/healthz" 120 "HTTPS $DiaChi")) {
  Ghi "  CẢNH BÁO: chưa vào được https://$DiaChi/healthz từ chính máy chủ — kiểm DNS / tên máy, cổng 443, logs\caddy.err.log" Yellow
}

# --- 8. Sao lưu hằng đêm
$tenSl = "DigiText-LuuTru-SaoLuu"
if (Get-ScheduledTask -TaskName $tenSl -ErrorAction SilentlyContinue) { Unregister-ScheduledTask -TaskName $tenSl -Confirm:$false }
$a = New-ScheduledTaskAction -Execute "powershell.exe" -WorkingDirectory $goc -Argument "-NoProfile -ExecutionPolicy Bypass -File `"$(Join-Path $goc 'sao-luu.ps1')`""
Register-ScheduledTask -TaskName $tenSl -Action $a -Trigger (New-ScheduledTaskTrigger -Daily -At 01:00) `
  -Principal (New-ScheduledTaskPrincipal -UserId "SYSTEM" -LogonType ServiceAccount -RunLevel Highest) -Description "Sao lưu CSDL + kho DigiText Lưu trữ" | Out-Null
Ghi "  sao lưu 01:00 hằng đêm → $($mc['SAO_LUU_DIR'])"

$coBoCai = Test-Path (Join-Path $goc "updates\latest.json")
Ghi "=== XONG. Máy trạm mở https://$DiaChi/tai-ve để tải bộ cài DigiText Lưu trữ$(if (-not $coBoCai) { ' (CHƯA có bộ cài trong updates\ — chép gói phát hành vào)' })." Green
Ghi "    Nhập gói AIP từ ổ cứng: .\nhap-goi.ps1 -ThuMuc <thư mục gói> (hoặc màn Nhập gói AIP trên máy trạm)."
