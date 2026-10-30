# Sao lưu CSDL Postgres (Docker digitext-dev-postgres-1) bằng pg_dump -Fc, giữ N bản gần nhất — 30/10/2026.
# Trước đây KHÔNG có sao lưu nào. Scheduled Task "DigiText-Backup-CSDL" (SYSTEM, 01:00 hàng đêm):
#   $a = New-ScheduledTaskAction -Execute powershell.exe -Argument "-NoProfile -ExecutionPolicy Bypass -File D:\DigiText\dev\backup-csdl.ps1 -Dir F:\DigiText-Backup\csdl"
#   $t = New-ScheduledTaskTrigger -Daily -At 01:00
#   $p = New-ScheduledTaskPrincipal -UserId SYSTEM -LogonType ServiceAccount -RunLevel Highest
#   Register-ScheduledTask DigiText-Backup-CSDL -Action $a -Trigger $t -Principal $p
# Chạy tay để lấy bản dump chuyển máy:  powershell -ExecutionPolicy Bypass -File dev\backup-csdl.ps1 -Dir F:\DigiText-ChuyenMay -Giu 0
param(
  [Parameter(Mandatory = $true)][string]$Dir,
  [int]$Giu = 14,
  [string]$Container = "digitext-dev-postgres-1",
  [string]$Ten = ""
)
$ErrorActionPreference = "Stop"
$repo = Split-Path -Parent $PSScriptRoot
$log = Join-Path $repo "logs\backup-csdl.log"
function Ghi($m) { $l = "{0:yyyy-MM-dd HH:mm:ss} {1}" -f (Get-Date), $m; Write-Host $l; Add-Content -Path $log -Value $l -Encoding UTF8 }

New-Item -ItemType Directory -Force -Path $Dir | Out-Null
if (-not $Ten) { $Ten = "digitext-{0:yyyyMMdd-HHmm}.dump" -f (Get-Date) }
$dich = Join-Path $Dir $Ten
$tam = "$dich.part"
$t0 = Get-Date
try {
  # pg_dump chạy trong container (đúng phiên bản client), ghi ra stdout → tệp trên Windows. Không bọc 2>&1
  # (PowerShell 5.1 coi stderr là lỗi). Không -i: cmd đọc stdout nhị phân nguyên vẹn.
  cmd /c "docker exec $Container sh -c ""pg_dump -U `$POSTGRES_USER -d `$POSTGRES_DB -Fc --no-owner"" > ""$tam"""
  if ($LASTEXITCODE -ne 0) { throw "pg_dump thoát mã $LASTEXITCODE" }
  $mb = [math]::Round((Get-Item $tam).Length / 1MB, 1)
  if ($mb -lt 1) { throw "tệp dump chỉ $mb MB — bất thường" }
  # Đọc lại mục lục để chắc tệp không hỏng.
  cmd /c "docker exec -i $Container pg_restore --list < ""$tam"" > nul"
  if ($LASTEXITCODE -ne 0) { throw "pg_restore --list không đọc được tệp dump" }
  Move-Item $tam $dich -Force
  Ghi "OK $dich ($mb MB, $([int]((Get-Date) - $t0).TotalSeconds) s)"
} catch {
  Ghi "LỖI $_"
  if (Test-Path $tam) { Remove-Item $tam -Force }
  exit 1
}
if ($Giu -gt 0) {
  Get-ChildItem $Dir -Filter "digitext-*.dump" | Sort-Object LastWriteTime -Descending | Select-Object -Skip $Giu |
    ForEach-Object { Ghi "xoá bản cũ $($_.Name)"; Remove-Item $_.FullName -Force }
}
