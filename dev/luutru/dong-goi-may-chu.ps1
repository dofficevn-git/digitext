# ĐÓNG GÓI MÁY CHỦ DIGITEXT LƯU TRỮ CHO WINDOWS (bản Lưu trữ GĐ2, 9/11/2026) — chạy trên máy build (repo này):
#   powershell -ExecutionPolicy Bypass -File dev\luutru\dong-goi-may-chu.ps1
#   ... -KhongBuild          dùng target\release có sẵn
#   ... -Caddy <caddy.exe>   dùng caddy.exe có sẵn thay vì tải bản chính thức từ GitHub
#   ... -Linux               đóng THÊM gói Linux: dựng image Docker, docker save, kèm infra\compose\luutru (không kèm mã nguồn)
# Ra: dist\luutru-server-win-<phiên bản>.zip — giải nén trên máy chủ chủ đầu tư rồi chạy cai-dat.ps1 (quyền quản trị);
#     dist\luutru-server-linux-<phiên bản>.tar.gz — giải nén rồi chạy sudo ./cai-dat.sh <địa chỉ>.
# Bộ cài máy trạm: lấy từ updates-luutru\ (dev\release-desktop.ps1 -Edition luutru) nếu có — chép vào updates\ của gói.
param(
  [switch]$KhongBuild,
  [switch]$Linux,
  [string]$Caddy = "",
  [string]$CaddyBan = "2.10.2"
)
$ErrorActionPreference = "Stop"
$repo = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
Set-Location $repo
$ver = (Select-String -Path "crates\digitext-api\Cargo.toml" -Pattern '^version\s*=\s*"(.+)"' | Select-Object -First 1).Matches[0].Groups[1].Value
if (-not $ver) { $ver = (Select-String -Path "Cargo.toml" -Pattern '^version\s*=\s*"(.+)"' | Select-Object -First 1).Matches[0].Groups[1].Value }
$ngay = Get-Date -Format "yyyyMMdd"
$ten = "luutru-server-win-$ver-$ngay"
$ra = Join-Path $repo "dist\$ten"
Write-Host "=== Đóng gói $ten" -ForegroundColor Cyan

if (-not $KhongBuild) {
  cargo build --release -p digitext-api -p digitext-ocr
  if ($LASTEXITCODE -ne 0) { throw "cargo build lỗi" }
}
foreach ($f in "target\release\digitext-api.exe", "target\release\digitext-ocr.exe") {
  if (-not (Test-Path $f)) { throw "thiếu $f" }
}

if (Test-Path $ra) { Remove-Item $ra -Recurse -Force }
New-Item -ItemType Directory -Force "$ra\bin", "$ra\migrations", "$ra\updates" | Out-Null
Copy-Item "target\release\digitext-api.exe", "target\release\digitext-ocr.exe" "$ra\bin\"

if ($Caddy) {
  Copy-Item $Caddy "$ra\bin\caddy.exe"
} else {
  $cache = Join-Path $repo "dist\cache\caddy_$CaddyBan"
  if (-not (Test-Path "$cache\caddy.exe")) {
    New-Item -ItemType Directory -Force $cache | Out-Null
    $url = "https://github.com/caddyserver/caddy/releases/download/v$CaddyBan/caddy_${CaddyBan}_windows_amd64.zip"
    Write-Host "  tải Caddy ${CaddyBan}: $url"
    Invoke-WebRequest -UseBasicParsing $url -OutFile "$cache\caddy.zip"
    Expand-Archive "$cache\caddy.zip" $cache -Force
  }
  Copy-Item "$cache\caddy.exe" "$ra\bin\caddy.exe"
}

Copy-Item "migrations\*.sql" "$ra\migrations\"
# Script: PowerShell 5.1 cần UTF-8 CÓ BOM mới đọc đúng chữ Việt.
$bom = New-Object System.Text.UTF8Encoding($true)
foreach ($f in Get-ChildItem "dev\luutru\windows\*.ps1") {
  [IO.File]::WriteAllText((Join-Path $ra $f.Name), [IO.File]::ReadAllText($f.FullName), $bom)
}
if (Test-Path "updates-luutru\latest.json") {
  Get-ChildItem "updates-luutru" -File | Where-Object { $_.Name -match '\.(exe|sig|json)$' } | Copy-Item -Destination "$ra\updates\"
  Write-Host "  kèm bộ cài máy trạm: $((Get-Content updates-luutru\latest.json | ConvertFrom-Json).version)"
} else {
  Write-Host "  CHƯA có bộ cài máy trạm (updates-luutru\) — chạy dev\release-desktop.ps1 -Edition luutru" -ForegroundColor Yellow
}
Copy-Item "docs\CAI-DAT-MAY-CHU-LUU-TRU.md" "$ra\HUONG-DAN-CAI-DAT.md"

$zip = "$ra.zip"
if (Test-Path $zip) { Remove-Item $zip -Force }
Compress-Archive -Path "$ra\*" -DestinationPath $zip
$mb = [math]::Round((Get-Item $zip).Length / 1MB, 1)
Write-Host "=== XONG: $zip ($mb MB)" -ForegroundColor Green

if ($Linux) {
  $tenL = "luutru-server-linux-$ver-$ngay"
  $raL = Join-Path $repo "dist\$tenL"
  if (Test-Path $raL) { Remove-Item $raL -Recurse -Force }
  New-Item -ItemType Directory -Force $raL | Out-Null
  docker build -f infra\compose\luutru\Dockerfile -t digitext-luutru:local .
  if ($LASTEXITCODE -ne 0) { throw "docker build lỗi" }
  docker save -o "$raL\digitext-luutru-image.tar" digitext-luutru:local
  if ($LASTEXITCODE -ne 0) { throw "docker save lỗi" }
  $vao = [IO.File]::OpenRead("$raL\digitext-luutru-image.tar")
  $ra2 = [IO.File]::Create("$raL\digitext-luutru-image.tar.gz")
  $gz = New-Object IO.Compression.GZipStream($ra2, [IO.Compression.CompressionLevel]::Optimal)
  $vao.CopyTo($gz); $gz.Dispose(); $ra2.Dispose(); $vao.Dispose()
  Remove-Item "$raL\digitext-luutru-image.tar"
  Copy-Item "infra\compose\luutru\docker-compose.yml", "infra\compose\luutru\cai-dat.sh", "infra\compose\luutru\sao-luu.sh", "infra\compose\luutru\Dockerfile" $raL
  Copy-Item "docs\CAI-DAT-MAY-CHU-LUU-TRU.md" "$raL\HUONG-DAN-CAI-DAT.md"
  New-Item -ItemType Directory -Force "$raL\updates" | Out-Null
  if (Test-Path "updates-luutru\latest.json") { Get-ChildItem "updates-luutru" -File | Where-Object { $_.Name -match '\.(exe|sig|json)$' } | Copy-Item -Destination "$raL\updates\" }
  $tgz = "$raL.tar.gz"
  if (Test-Path $tgz) { Remove-Item $tgz -Force }
  tar -czf $tgz -C (Join-Path $repo "dist") $tenL
  if ($LASTEXITCODE -ne 0) { throw "tar lỗi" }
  Write-Host "=== XONG: $tgz ($([math]::Round((Get-Item $tgz).Length / 1MB, 1)) MB)" -ForegroundColor Green
}
