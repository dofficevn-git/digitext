# Phát hành app desktop DigiText — một lệnh:
#   powershell -ExecutionPolicy Bypass -File dev\release-desktop.ps1 -Version 0.16.0 -Notes "Sửa lỗi X, thêm Y"
#
# Làm gì: tăng version ở 3 file → build NSIS đã KÝ (khoá ~/.tauri/digitext.key) →
# chép bộ cài + .sig vào thư mục phát hành (DIGITEXT_UPDATES_DIR của API, mặc định
# D:\DigiText\updates) → ghi updates\latest.json → app trên các máy tự thấy bản mới
# qua https://api.digitext.vn/updates/latest.json.
#
# Cần: cargo + node trên máy này; khoá ở $env:USERPROFILE\.tauri\digitext.key (tạo bằng
# `npx tauri signer generate`); API đang chạy ở máy này (không cần khởi động lại —
# route /updates đọc thẳng từ đĩa).
param(
  [Parameter(Mandatory = $true)][string]$Version,
  [string]$Notes = "",
  [string]$UpdatesDir = "D:\DigiText\updates",
  [string]$BaseUrl = "https://api.digitext.vn/updates",
  # Máy chủ API ở máy khác (OCR PC): chép bộ cài + .sig + latest.json sang thư mục phát hành của nó
  # (UNC \\may\updates hoặc ổ mạng). Rỗng = API chạy ngay máy này (30/10/2026).
  [string]$UpdatesRemote = "",
  # Máy chủ API là OCR PC (từ 2/11/2026): chép qua scp (SSH Tailscale). Rỗng = không chép. Thư mục đích là
  # C:\DigiText\updates trên OCR PC — API đọc thẳng (DIGITEXT_UPDATES_DIR / cwd C:\DigiText).
  [string]$UpdatesScp = "ADMIN@100.122.226.63:C:/DigiText/updates",
  [switch]$SkipBump
)
$ErrorActionPreference = "Stop"
if ($Version -notmatch '^\d+\.\d+\.\d+$') { throw "Version phải dạng x.y.z (vd 0.16.0)" }
$repo = Split-Path -Parent $PSScriptRoot
$desktop = Join-Path $repo "desktop"
$key = Join-Path $env:USERPROFILE ".tauri\digitext.key"
if (-not (Test-Path $key)) { throw "Thiếu khoá ký $key — chạy: cd desktop; npx tauri signer generate -w `"$key`" -p <mật khẩu>; lưu mật khẩu vào $key.pw" }
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"

function Set-Version($path, $pattern, $replacement) {
  $txt = [IO.File]::ReadAllText($path)
  $new = [regex]::Replace($txt, $pattern, $replacement, 1)
  if ($new -eq $txt) { throw "Không tìm thấy version trong $path" }
  [IO.File]::WriteAllText($path, $new, (New-Object Text.UTF8Encoding $false))
}
if (-not $SkipBump) {
  Set-Version "$desktop\src-tauri\tauri.conf.json" '"version":\s*"\d+\.\d+\.\d+"' "`"version`": `"$Version`""
  Set-Version "$desktop\package.json" '"version":\s*"\d+\.\d+\.\d+"' "`"version`": `"$Version`""
  Set-Version "$desktop\src-tauri\Cargo.toml" '(?m)^version\s*=\s*"\d+\.\d+\.\d+"' "version = `"$Version`""
  Write-Host "-> version $Version ghi vào tauri.conf.json, package.json, Cargo.toml" -ForegroundColor Cyan
}

# Ký bằng khoá riêng + mật khẩu ở digitext.key.pw. Windows KHÔNG giữ được biến môi trường
# rỗng ($env:X = "" là xoá biến) → khoá bắt buộc phải có mật khẩu, nếu không CLI sẽ hỏi
# ở stdin và treo vô hạn. Biến chỉ sống trong tiến trình này.
$pwFile = "$key.pw"
if (-not (Test-Path $pwFile)) { throw "Thiếu mật khẩu khoá $pwFile" }
$env:TAURI_SIGNING_PRIVATE_KEY = (Get-Content $key -Raw).Trim()
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = (Get-Content $pwFile -Raw).Trim()
Push-Location $desktop
try {
  Write-Host "-> npm run tauri build (vài phút)…" -ForegroundColor Cyan
  # Không bọc 2>&1: tauri CLI ghi Info ra stderr, PowerShell 5.1 + ErrorActionPreference=Stop
  # sẽ coi là lỗi và dừng. Để nguyên luồng, chỉ xét exit code.
  $ErrorActionPreference = "Continue"
  & cmd.exe /c "npm run tauri build"
  $rc = $LASTEXITCODE
  $ErrorActionPreference = "Stop"
  if ($rc -ne 0) { throw "tauri build thất bại (exit $rc)" }
} finally { Pop-Location }

$nsis = Join-Path $desktop "src-tauri\target\release\bundle\nsis"
$setup = Join-Path $nsis "DigiText_${Version}_x64-setup.exe"
$sig = "$setup.sig"
if (-not (Test-Path $setup)) { throw "Không thấy $setup" }
if (-not (Test-Path $sig)) { throw "Không thấy chữ ký $sig — createUpdaterArtifacts chưa bật hoặc khoá sai" }

New-Item -ItemType Directory -Force $UpdatesDir | Out-Null
Copy-Item $setup $UpdatesDir -Force
Copy-Item $sig $UpdatesDir -Force
$name = Split-Path -Leaf $setup
$latest = [ordered]@{
  version = $Version
  notes = $Notes
  pub_date = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
  platforms = [ordered]@{
    "windows-x86_64" = [ordered]@{
      signature = (Get-Content $sig -Raw).Trim()
      url = "$BaseUrl/$name"
    }
  }
}
$json = $latest | ConvertTo-Json -Depth 5
[IO.File]::WriteAllText((Join-Path $UpdatesDir "latest.json"), $json, (New-Object Text.UTF8Encoding $false))
Write-Host "-> $UpdatesDir\latest.json → $Version ($name, $([math]::Round((Get-Item $setup).Length/1MB,1)) MB)" -ForegroundColor Green
if ($UpdatesRemote) {
  if (-not (Test-Path $UpdatesRemote)) { throw "Không tới được thư mục phát hành máy chủ $UpdatesRemote" }
  # Bộ cài + .sig trước, latest.json sau cùng: app chỉ thấy bản mới khi tệp đã nằm đủ.
  Copy-Item $setup $UpdatesRemote -Force
  Copy-Item $sig $UpdatesRemote -Force
  Copy-Item (Join-Path $UpdatesDir "latest.json") $UpdatesRemote -Force
  $xa = Get-Item (Join-Path $UpdatesRemote $name)
  if ($xa.Length -ne (Get-Item $setup).Length) { throw "Bộ cài chép sang $UpdatesRemote lệch kích thước" }
  Write-Host "-> đã chép sang máy chủ: $UpdatesRemote" -ForegroundColor Green
}
if ($UpdatesScp) {
  # Bộ cài + .sig trước, latest.json sau cùng (như trên). scp.exe có sẵn trên Windows 10+/OpenSSH.
  & scp.exe -q $setup $sig "$UpdatesScp/"
  if ($LASTEXITCODE -ne 0) { throw "scp bộ cài sang $UpdatesScp lỗi (mã $LASTEXITCODE) — máy chủ chưa có bản mới, latest.json chưa đổi" }
  & scp.exe -q (Join-Path $UpdatesDir "latest.json") "$UpdatesScp/latest.json"
  if ($LASTEXITCODE -ne 0) { throw "scp latest.json sang $UpdatesScp lỗi (mã $LASTEXITCODE)" }
  try {
    $xa = (Invoke-RestMethod -Uri "$BaseUrl/latest.json?t=$(Get-Date -Format HHmmss)" -TimeoutSec 30).version
    if ($xa -ne $Version) { Write-Warning "latest.json công khai đang là $xa, chưa phải $Version — kiểm lại thư mục phát hành trên OCR PC" }
    else { Write-Host "-> máy chủ đã phát $Version qua $BaseUrl" -ForegroundColor Green }
  } catch { Write-Warning "không kiểm được $BaseUrl/latest.json: $($_.Exception.Message)" }
}
Write-Host "   Kiểm: curl.exe $BaseUrl/latest.json" -ForegroundColor DarkGray
Write-Host "   Nhớ: git commit 3 file version (+ tag v$Version)." -ForegroundColor DarkGray
