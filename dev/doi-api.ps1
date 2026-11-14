# Đổi exe của API rồi khởi động lại — MỘT lệnh, không thể để trống chỗ.
#
# Vì sao có tệp này (14/11/2026): quy trình gõ tay là bốn lệnh rời —
#   scp … → ssh kiểm cỡ → ren exe cũ → move exe mới vào.
# Bỏ sót lệnh scp mà vẫn chạy `ren` thì thư mục KHÔNG CÒN exe nào, dịch vụ chết,
# và không có gì báo cho tới khi ai đó gọi API và nhận 502. Đã xảy ra thật.
# Ở đây thứ tự bị đảo lại cho an toàn: kiểm bản mới TRƯỚC, chỉ động vào bản đang
# chạy khi bản mới đã nằm sẵn trong thư mục. Hỏng ở bất kỳ bước nào cũng tự lùi lại.
#
# Trên MÁY CHỦ OCR PC, console/RDP (script tự xin UAC):
#   powershell -ExecutionPolicy Bypass -File C:\DigiText\dev\doi-api.ps1 -Co 26068992
#
#   -Moi <đường dẫn>  tệp exe mới. Mặc định C:\DigiText-ChuyenMay\digitext-api.exe
#   -Co  <số byte>    cỡ phải khớp. KHÔNG có thì chỉ cảnh báo, vẫn chạy.
#   -KhongKiemSong    bỏ bước gọi thử API sau khi khởi động (chỉ dùng khi mạng chặn).
param(
  [string]$Moi = "C:\DigiText-ChuyenMay\digitext-api.exe",
  [long]$Co = 0,
  [switch]$KhongKiemSong
)
$ErrorActionPreference = "Stop"

$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $isAdmin) {
  $a = "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`" -Moi `"$Moi`" -Co $Co"
  if ($KhongKiemSong) { $a += " -KhongKiemSong" }
  Start-Process powershell.exe -Verb RunAs -ArgumentList $a
  exit 0
}

$root = Split-Path -Parent $PSScriptRoot
$dich = Join-Path $root "target\release\digitext-api.exe"
$tam = "$dich.moi"
$log = Join-Path $root "logs\restart-services.log"
function Log($m) {
  Write-Host $m
  Add-Content -Path $log -Value ("{0:yyyy-MM-dd HH:mm:ss} doi-api: {1}" -f (Get-Date), $m) -Encoding UTF8
}
function Dung($m) { Log "DỪNG — $m"; Read-Host "Enter để đóng"; exit 1 }

# ---- 1. Kiểm bản mới TRƯỚC. Chưa chắc chắn thì chưa động gì tới bản đang chạy.
if (-not (Test-Path $Moi)) {
  Dung "không có tệp $Moi. Chưa copy sang máy chủ? Bản đang chạy giữ nguyên, API vẫn sống."
}
$coMoi = (Get-Item $Moi).Length
if ($coMoi -lt 1000000) { Dung "$Moi chỉ $coMoi byte — không phải exe đầy đủ." }
if ($Co -gt 0 -and $coMoi -ne $Co) {
  Dung "$Moi cỡ $coMoi byte, không khớp $Co byte đang chờ. Copy lại, hoặc bỏ tham số -Co nếu cố ý."
}
$cu = if (Test-Path $dich) { (Get-Item $dich).Length } else { 0 }
if ($cu -eq 0) { Log "CHÚ Ý: hiện KHÔNG có $dich — đang vá một lần đổi hỏng dở." }
elseif ($cu -eq $coMoi) { Log "CHÚ Ý: bản đang chạy cũng $cu byte — có thể đã đổi rồi." }
Log "bản mới $coMoi byte · bản đang chạy $cu byte"

# ---- 2. Đưa bản mới vào thư mục đích trước khi đụng tới bản cũ.
Copy-Item $Moi $tam -Force
if ((Get-Item $tam).Length -ne $coMoi) { Remove-Item $tam -Force; Dung "copy sang $tam bị thiếu byte." }

# ---- 3. Dừng dịch vụ, rồi mới tráo. Khoảng trống giữa hai lệnh ren chỉ vài mili giây.
Stop-ScheduledTask -TaskName "DigiText-API" -ErrorAction SilentlyContinue
Get-Process digitext-api -ErrorAction SilentlyContinue | Stop-Process -Force
Start-Sleep 2

$luu = ""
if ($cu -gt 0) {
  $luu = "$dich.cu-" + (Get-Date -Format "yyyyMMdd-HHmm")
  Move-Item $dich $luu -Force
  Log "bản đang chạy cất ở $(Split-Path -Leaf $luu)"
}
try {
  Move-Item $tam $dich -Force
} catch {
  if ($luu) { Move-Item $luu $dich -Force; Log "đã lùi lại bản cũ" }
  Start-ScheduledTask -TaskName "DigiText-API"
  Dung "không tráo được: $_"
}

# ---- 4. Chạy lại và kiểm nó thực sự sống, không chỉ "đã gọi lệnh start".
Start-ScheduledTask -TaskName "DigiText-API"
Start-Sleep 6
$p = Get-Process digitext-api -ErrorAction SilentlyContinue
if (-not $p) {
  Log "tiến trình KHÔNG chạy sau 6 giây — lùi lại bản cũ"
  Get-Process digitext-api -ErrorAction SilentlyContinue | Stop-Process -Force
  if ($luu) { Move-Item $dich "$dich.hong" -Force; Move-Item $luu $dich -Force }
  Start-ScheduledTask -TaskName "DigiText-API"
  Dung "bản mới không chạy được. Đã lùi về bản cũ, exe hỏng để ở digitext-api.exe.hong"
}
Log "đang chạy pid=$($p.Id) · $((Get-Item $dich).Length) byte"

if (-not $KhongKiemSong) {
  # 401 là ĐÚNG: cửa sống và đang đòi khoá. 502/000 mới là chết.
  $ma = try {
    (Invoke-WebRequest -Uri "https://api.digitext.vn/api/v1/ncc/tu-kiem/goi" -Method POST `
      -ContentType "application/json" -Body "{}" -TimeoutSec 20 -UseBasicParsing).StatusCode
  } catch { if ($_.Exception.Response) { [int]$_.Exception.Response.StatusCode } else { 0 } }
  if ($ma -eq 401) { Log "API trả 401 — sống, đang đòi khoá. XONG." }
  elseif ($ma -eq 404) { Log "CHÚ Ý: 404 — API sống nhưng KHÔNG có cửa /ncc/tu-kiem/goi. Đổi nhầm bản cũ?" }
  else { Log "CHÚ Ý: gọi thử ra $ma. Tiến trình đang chạy, nhưng kiểm lại Caddy/cổng." }
}
Read-Host "Enter để đóng"
