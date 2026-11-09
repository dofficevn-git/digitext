# Ap migration tren may chu: nap crates/digitext-api/.env roi goi digitext-api.exe migrate.
# Chay TRUC TIEP tren may chu (console hoac RDP), dung chay qua ssh - PowerShell qua ssh
# cat mat dau $ nen bien moi truong khong vao duoc.
#
#   powershell -ExecutionPolicy Bypass -File C:\DigiText\dev\ap-migration.ps1 --chi-xem
#   powershell -ExecutionPolicy Bypass -File C:\DigiText\dev\ap-migration.ps1 --da-ap-den 0023
#
# Lan dau tren may chu da co du lieu PHAI co --da-ap-den <NNNN>: danh dau cac migration
# tu 0001 den NNNN la "da ap" (khong chay lai), roi moi ap nhung tep sau do.
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$envFile = Join-Path $root "crates\digitext-api\.env"
$exe = Join-Path $root "target\release\digitext-api.exe"

if (-not (Test-Path $envFile)) { throw "Khong thay $envFile" }
if (-not (Test-Path $exe)) { throw "Khong thay $exe" }

Get-Content $envFile | ForEach-Object {
    if ($_ -match '^\s*([A-Z_][A-Z0-9_]*)\s*=\s*(.*)$') {
        [Environment]::SetEnvironmentVariable($Matches[1], $Matches[2].Trim('"'), "Process")
    }
}
if (-not $env:DATABASE_URL) { throw "DATABASE_URL chua duoc dat trong $envFile" }

$db = ($env:DATABASE_URL -split '@')[-1]
Write-Host "-> db=...@$db" -ForegroundColor Cyan
Write-Host "-> thu muc migration: $(Join-Path $root 'migrations')" -ForegroundColor Cyan
Write-Host ""

Push-Location $root
try {
    & $exe migrate @args
    $ma = $LASTEXITCODE
}
finally {
    Pop-Location
}
exit $ma
