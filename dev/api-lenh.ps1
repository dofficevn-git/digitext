# Chay mot lenh cua digitext-api.exe tren may chu: nap crates/digitext-api/.env roi
# chuyen nguyen tham so sang exe. Dung cho migrate, hook-key, nhap-goi...
#
# Chay TRUC TIEP tren may chu (console hoac RDP), dung chay qua ssh - PowerShell qua ssh
# cat mat dau $ nen bien moi truong khong vao duoc.
#
#   # ap migration (lan dau tren may chu da co du lieu PHAI co --da-ap-den <NNNN>:
#   # danh dau 0001..NNNN la "da ap" roi moi ap nhung tep sau do)
#   powershell -ExecutionPolicy Bypass -File C:\DigiText\dev\api-lenh.ps1 migrate --da-ap-den 0023 --chi-xem
#   powershell -ExecutionPolicy Bypass -File C:\DigiText\dev\api-lenh.ps1 migrate --da-ap-den 0023
#
#   # cap khoa webhook cho he thong ngoai (xem docs/WEBHOOK-DU-AN.md)
#   powershell -ExecutionPolicy Bypass -File C:\DigiText\dev\api-lenh.ps1 hook-key xem
#   powershell -ExecutionPolicy Bypass -File C:\DigiText\dev\api-lenh.ps1 hook-key tao --nguon crm --ten "CRM hop dong" --quan-tri hieu
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$envFile = Join-Path $root "crates\digitext-api\.env"
$exe = Join-Path $root "target\release\digitext-api.exe"

if ($args.Count -eq 0) { throw "Thieu lenh. Vi du: api-lenh.ps1 migrate --chi-xem | api-lenh.ps1 hook-key xem" }
if ($args[0] -eq "serve") { throw "Khong chay serve bang script nay - API chay bang Scheduled Task DigiText-API." }
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
Write-Host "-> $exe $($args -join ' ')" -ForegroundColor Cyan
Write-Host ""

# Chay tu thu muc goc repo de exe thay thu muc migrations/.
Push-Location $root
try {
    & $exe @args
    $ma = $LASTEXITCODE
}
finally {
    Pop-Location
}
exit $ma
