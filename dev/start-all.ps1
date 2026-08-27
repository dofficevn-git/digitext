# Bat toan bo he thong DigiText tren may dev nay.
#
#   .\dev\start-all.ps1          # bat ha tang + API, roi mo app desktop
#   .\dev\start-all.ps1 -NoApp   # chi bat ha tang + API (de tu chay tauri dev)
#
# API chay trong mot cua so PowerShell rieng de xem log; dong cua so do la tat API.
param([switch]$NoApp)

$ErrorActionPreference = "Continue"
$root = Split-Path -Parent $PSScriptRoot
$compose = Join-Path $root "infra\compose"

function Test-Port($p) {
    [bool](Get-NetTCPConnection -LocalPort $p -State Listen -ErrorAction SilentlyContinue)
}

# --- 1. Ha tang -------------------------------------------------------------
Write-Host "[1/3] Postgres / MinIO / Meilisearch..." -ForegroundColor Cyan
$files = @("-f", (Join-Path $compose "docker-compose.yml"))
$override = Join-Path $compose "docker-compose.override.yml"
if (Test-Path $override) { $files += @("-f", $override) }
docker compose @files up -d postgres minio meilisearch | Out-Null

# doi Postgres san sang thay vi doan thoi gian
$ok = $false
foreach ($i in 1..30) {
    docker exec digitext-dev-postgres-1 pg_isready -U digitext 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) { $ok = $true; break }
    Start-Sleep -Seconds 1
}
if ($ok) { Write-Host "      Postgres san sang" -ForegroundColor Green }
else { Write-Host "      Postgres CHUA san sang - kiem tra: docker logs digitext-dev-postgres-1" -ForegroundColor Red }

# --- 2. API -----------------------------------------------------------------
if (Test-Port 8080) {
    Write-Host "[2/3] API da chay san o 8080" -ForegroundColor Green
} else {
    Write-Host "[2/3] Khoi dong API (cua so rieng)..." -ForegroundColor Cyan
    Start-Process powershell -ArgumentList @(
        "-NoExit", "-Command", "& '$root\dev\run-api.ps1' serve"
    )
    foreach ($i in 1..90) { if (Test-Port 8080) { break }; Start-Sleep -Seconds 1 }
    if (Test-Port 8080) { Write-Host "      API len o http://127.0.0.1:8080" -ForegroundColor Green }
    else { Write-Host "      API chua len - xem cua so PowerShell vua mo" -ForegroundColor Red }
}

# --- 3. App desktop ---------------------------------------------------------
if ($NoApp) {
    Write-Host "[3/3] Bo qua app (-NoApp). Tu chay: cd desktop; npm run tauri dev" -ForegroundColor Yellow
    exit 0
}
if (Test-Port 5173) {
    Write-Host "[3/3] Cong 5173 dang bi chiem - tat tien trinh do truoc khi chay tauri dev" -ForegroundColor Red
    exit 1
}
Write-Host "[3/3] Mo app desktop (lan dau build lau)..." -ForegroundColor Cyan
Write-Host "      Dia chi may chu khi dang nhap: http://127.0.0.1:8080" -ForegroundColor Yellow
Push-Location (Join-Path $root "desktop")
try { npm run tauri dev } finally { Pop-Location }
