# Chay worker OCR (digitext-ocr) tren Windows.
# Nap crates/digitext-ocr/.env roi goi cargo run tu thu muc goc repo.
#
#   .\dev\run-ocr-worker.ps1
#
# Worker doi HAI dieu kien tien quyet, thieu la no thoat ngay:
#   1. Bang engine_versions phai co dong role='recognition', active=true.
#   2. Neu DIGITEXT_OCR_ENGINE=http thi phai co DIGITEXT_OCR_HTTP_ENDPOINT.
$root = Split-Path -Parent $PSScriptRoot
$envFile = Join-Path $root "crates\digitext-ocr\.env"

if (-not (Test-Path $envFile)) {
    Write-Host "Chua co $envFile" -ForegroundColor Red
    Write-Host "Xem infra/launchd-or-systemd/systemd/ocr.env.example de biet cac khoa." -ForegroundColor Yellow
    exit 1
}

Get-Content $envFile | ForEach-Object {
    if ($_ -match '^\s*([A-Z_][A-Z0-9_]*)\s*=\s*(.*)$') {
        [Environment]::SetEnvironmentVariable($Matches[1], $Matches[2].Trim('"'), "Process")
    }
}

if (-not $env:DIGITEXT_DATABASE_URL) { throw "DIGITEXT_DATABASE_URL chua duoc dat trong $envFile" }

$engine = if ($env:DIGITEXT_OCR_ENGINE) { $env:DIGITEXT_OCR_ENGINE } else { "triton" }
$target = if ($engine -eq "http") { $env:DIGITEXT_OCR_HTTP_ENDPOINT } else { $env:DIGITEXT_TRITON_ENDPOINT }
Write-Host "-> engine=$engine  dich=$target  worker=$env:DIGITEXT_WORKER_ID" -ForegroundColor Cyan

# cargo ghi tien do bien dich ra stderr; voi ErrorActionPreference='Stop' thi
# PowerShell coi moi dong stderr cua native exe la loi nghiem trong va giet
# script ngay dong "Compiling ..." dau tien.
$ErrorActionPreference = "Continue"

Push-Location $root
try { cargo run -p digitext-ocr -- @args } finally { Pop-Location }
