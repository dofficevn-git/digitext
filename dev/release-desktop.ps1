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
Write-Host "   Kiểm: curl.exe $BaseUrl/latest.json" -ForegroundColor DarkGray
Write-Host "   Nhớ: git commit 3 file version (+ tag v$Version)." -ForegroundColor DarkGray
