# Chay digitext-api tren Windows. Tuong duong dev/run-api.sh cho bash.
# Nap crates/digitext-api/.env roi goi cargo run tu thu muc goc repo.
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$envFile = Join-Path $root "crates\digitext-api\.env"
$example = Join-Path $root "crates\digitext-api\.env.example"

if (-not (Test-Path $envFile)) {
    Copy-Item $example $envFile
    Write-Host "Chua co $envFile - da tao tu .env.example." -ForegroundColor Yellow
    Write-Host "Sua DATABASE_URL cho khop infra/compose/.env roi chay lai." -ForegroundColor Yellow
    exit 1
}

Get-Content $envFile | ForEach-Object {
    if ($_ -match '^\s*([A-Z_][A-Z0-9_]*)\s*=\s*(.*)$') {
        [Environment]::SetEnvironmentVariable($Matches[1], $Matches[2].Trim('"'), "Process")
    }
}

if (-not $env:DATABASE_URL) { throw "DATABASE_URL chua duoc dat trong $envFile" }

$bind = if ($env:DIGITEXT_BIND) { $env:DIGITEXT_BIND } else { "127.0.0.1:8080" }
$host_part = ($env:DATABASE_URL -split '@')[-1]
Write-Host "-> bind=$bind  db=...@$host_part" -ForegroundColor Cyan

# cargo ghi tien do bien dich ra stderr. Voi ErrorActionPreference='Stop',
# PowerShell coi moi dong stderr cua native exe la loi nghiem trong va giet
# script ngay dong "Compiling ..." dau tien. Phai ha xuong Continue.
$ErrorActionPreference = "Continue"

Push-Location $root
try { cargo run -p digitext-api -- @args } finally { Pop-Location }
