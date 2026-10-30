# Chạy kho MinIO trực tiếp trên Windows (thay MinIO trong Docker) — docs/KE-HOACH-CHUYEN-KHO-MINIO.md.
#   powershell -ExecutionPolicy Bypass -File dev\minio-win.ps1 -DataDir X:\DigiText-KhoMayChu [-Port 9000]
# Tài khoản gốc lấy từ infra/compose/.env (MINIO_USER / MINIO_PASSWORD) — đúng khoá API/worker đang dùng.
# Scheduled Task "DigiText-MinIO" (SYSTEM, khi khởi động) gọi script này; log ở logs\minio.log.
# minio.exe build từ mã nguồn RELEASE.2025-10-15T17-29-55Z (MinIO không còn phát hành bản Windows):
#   go1.24.8, CGO_ENABLED=0, go build -trimpath → SHA-256 a7ee0d09a53bda6fcd8160963d0d80f8cbc22bcb76afd0d6113538808d44e2f1
# Exe mặc định nằm trong repo: infra\minio-win\minio.exe (gitignore) — chép tay sang máy mới cùng checkout (30/10/2026).
param(
  [Parameter(Mandatory = $true)][string]$DataDir,
  [int]$Port = 9000,
  [string]$Exe = (Join-Path (Split-Path -Parent $PSScriptRoot) "infra\minio-win\minio.exe")
)
$ErrorActionPreference = "Stop"
$repo = Split-Path -Parent $PSScriptRoot
# Mỗi cổng một tệp log: MinIO đang chạy giữ tệp của nó, MinIO thứ hai ghi chung sẽ lỗi và thoát (17/10/2026).
$log = Join-Path $repo "logs\minio-$Port.log"
if (-not (Test-Path $Exe)) { throw "Không thấy $Exe" }
New-Item -ItemType Directory -Force -Path $DataDir | Out-Null

$envFile = Join-Path $repo "infra\compose\.env"
$kv = @{}
foreach ($line in [IO.File]::ReadAllLines($envFile)) {
  if ($line -match '^\s*(MINIO_USER|MINIO_PASSWORD)\s*=\s*(.*)\s*$') { $kv[$Matches[1]] = $Matches[2].Trim('"') }
}
if (-not $kv.MINIO_USER -or -not $kv.MINIO_PASSWORD) { throw "Thiếu MINIO_USER / MINIO_PASSWORD trong $envFile" }

$env:MINIO_ROOT_USER = $kv.MINIO_USER
$env:MINIO_ROOT_PASSWORD = $kv.MINIO_PASSWORD
# Ổ khựng thì chờ, không ngắt ổ; quét nền chậm nhất (như bản Docker, 14/10/2026).
$env:MINIO_DRIVE_MAX_TIMEOUT = "2m"
$env:MINIO_SCANNER_SPEED = "slowest"
$env:MINIO_UPDATE = "off"

# 21/10/2026: KHÔNG chạy `& $Exe ... *>> $log` dưới $ErrorActionPreference = "Stop": PowerShell 5.1 coi mỗi dòng
# stderr của MinIO là lỗi → dừng script, MinIO chết theo. Ổ HDD SMR khựng là MinIO cảnh báo ra stderr → kho sập
# (12:40 19/10, 17:22 21/10). Để cmd ghi log, và MinIO thoát vì bất cứ lý do gì thì chạy lại sau 5 giây.
$ErrorActionPreference = "Continue"
while ($true) {
  Add-Content -Path $log -Value ("{0:yyyy-MM-dd HH:mm:ss} khởi động MinIO cổng {1}, dữ liệu {2}" -f (Get-Date), $Port, $DataDir) -Encoding UTF8
  # Bọc cả lệnh trong một cặp ngoặc kép nữa: cmd /c bỏ cặp ngoài cùng, giữ nguyên ngoặc của từng đường dẫn.
  cmd /c "`"`"$Exe`" server `"$DataDir`" --address 127.0.0.1:$Port --console-address 127.0.0.1:$($Port + 1) --quiet >> `"$log`" 2>&1`""
  Add-Content -Path $log -Value ("{0:yyyy-MM-dd HH:mm:ss} MinIO thoát mã {1} — chạy lại sau 5 giây" -f (Get-Date), $LASTEXITCODE) -Encoding UTF8
  Start-Sleep -Seconds 5
}
