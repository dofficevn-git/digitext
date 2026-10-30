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
# Chi Postgres trong Docker. MinIO chay native Windows (Scheduled Task DigiText-MinIO, dev\minio-win-task.ps1),
# Meilisearch khong crate nao dung — bo tu 30/10/2026.
Write-Host "[1/4] Postgres..." -ForegroundColor Cyan
$files = @("-f", (Join-Path $compose "docker-compose.yml"))
$override = Join-Path $compose "docker-compose.override.yml"
if (Test-Path $override) { $files += @("-f", $override) }
docker compose @files up -d postgres | Out-Null

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
    Write-Host "[2/4] API da chay san o 8080" -ForegroundColor Green
} else {
    Write-Host "[2/4] Khoi dong API (cua so rieng)..." -ForegroundColor Cyan
    Start-Process powershell -ArgumentList @(
        "-NoExit", "-Command", "& '$root\dev\run-api.ps1' serve"
    )
    foreach ($i in 1..90) { if (Test-Port 8080) { break }; Start-Sleep -Seconds 1 }
    if (Test-Port 8080) { Write-Host "      API len o http://127.0.0.1:8080" -ForegroundColor Green }
    else { Write-Host "      API chua len - xem cua so PowerShell vua mo" -ForegroundColor Red }
}

# --- 3. Worker OCR -----------------------------------------------------------
# Worker chay tren may nay, goi dich vu OCR theo DIGITEXT_OCR_HTTP_ENDPOINT
# trong crates/digitext-ocr/.env (hien la Alienware; cung may thi 127.0.0.1:5002).
if (Test-Port 9095) {
    Write-Host "[3/4] Worker OCR da chay san (9095)" -ForegroundColor Green
} else {
    Write-Host "[3/4] Khoi dong worker OCR (cua so rieng)..." -ForegroundColor Cyan
    Start-Process powershell -ArgumentList @(
        "-NoExit", "-Command", "& '$root\dev\run-ocr-worker.ps1'"
    )
    foreach ($i in 1..90) { if (Test-Port 9095) { break }; Start-Sleep -Seconds 1 }
    if (Test-Port 9095) {
        $ready = try { (Invoke-WebRequest http://127.0.0.1:9095/readyz -UseBasicParsing -TimeoutSec 15).StatusCode -eq 200 } catch { $false }
        if ($ready) { Write-Host "      Worker len, thay dich vu OCR" -ForegroundColor Green }
        else { Write-Host "      Worker len nhung KHONG thay dich vu OCR - kiem http://127.0.0.1:9095/readyz" -ForegroundColor Red }
    } else { Write-Host "      Worker chua len - xem cua so PowerShell vua mo" -ForegroundColor Red }
}

# --- 4. App desktop ---------------------------------------------------------
if ($NoApp) {
    Write-Host "[4/4] Bo qua app (-NoApp). Tu chay: cd desktop; npm run tauri dev" -ForegroundColor Yellow
    exit 0
}
if (Test-Port 5173) {
    Write-Host "[4/4] Cong 5173 dang bi chiem - tat tien trinh do truoc khi chay tauri dev" -ForegroundColor Red
    exit 1
}
Write-Host "[4/4] Mo app desktop (lan dau build lau)..." -ForegroundColor Cyan
Write-Host "      Dia chi may chu khi dang nhap: http://127.0.0.1:8080" -ForegroundColor Yellow
Push-Location (Join-Path $root "desktop")
try { npm run tauri dev } finally { Pop-Location }
