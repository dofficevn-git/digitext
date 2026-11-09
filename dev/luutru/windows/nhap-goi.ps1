# Nhập gói AIP/SIP từ ổ cứng giao kèm bộ hồ sơ nghiệm thu vào máy chủ DigiText Lưu trữ (bản Lưu trữ GĐ2, 9/11/2026).
# Nạp conf\api.env rồi gọi `digitext-api nhap-goi` (docs/KE-HOACH-BAN-LUU-TRU.md GĐ3). Chạy được nhiều lần: gói đã nhập báo "đã có".
#   powershell -ExecutionPolicy Bypass -File .\nhap-goi.ps1 -ThuMuc F:\BoHoSoNghiemThu
#   ... -NhanCaGoiLoi            nhận cả gói kiểm KHÔNG ĐẠT (trừ chữ ký số hỏng) — chỉ khi chủ đầu tư chấp nhận
param(
  [Parameter(Mandatory = $true)][string[]]$ThuMuc,
  [string]$DuAn = "",
  [string]$Nguoi = "",
  [string]$GhiChu = "",
  [switch]$NhanCaGoiLoi
)
$goc = $PSScriptRoot
$cfg = @{}
foreach ($t in "conf\may-chu.env", "conf\api.env") {
  $p = Join-Path $goc $t
  if (-not (Test-Path $p)) { Write-Host "Thiếu $p — chạy cai-dat.ps1 trước." -ForegroundColor Red; exit 1 }
  Get-Content $p -Encoding UTF8 | ForEach-Object {
    if ($_ -match '^\s*([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.*)$') {
      $v = $Matches[2].Trim().Trim('"'); $cfg[$Matches[1]] = $v
      [Environment]::SetEnvironmentVariable($Matches[1], $v, "Process")
    }
  }
}
if (-not $DuAn) { $DuAn = $cfg["MA_DU_AN"] }
if (-not $Nguoi) { $Nguoi = $cfg["QUAN_TRI"] }
$ts = @("nhap-goi", "--du-an", $DuAn, "--nguoi", $Nguoi)
if ($NhanCaGoiLoi) { $ts += "--nhan-ca-goi-loi" }
if ($GhiChu) { $ts += @("--ghi-chu", $GhiChu) }
$ts += $ThuMuc
& (Join-Path $goc "bin\digitext-api.exe") @ts
exit $LASTEXITCODE
