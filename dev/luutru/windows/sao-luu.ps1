# Sao lưu máy chủ DigiText Lưu trữ (bản Lưu trữ GĐ2, 9/11/2026) — cai-dat.ps1 đăng ký chạy 01:00 hằng đêm (SYSTEM).
#   1. CSDL: pg_dump -Fc → <SaoLuuDir>\csdl\digitext-yyyyMMdd-HHmm.dump, đọc lại mục lục để chắc tệp không hỏng, giữ N bản.
#   2. Kho tài liệu (thư mục kho): robocopy /E sang <SaoLuuDir>\kho — CHỈ thêm / cập nhật, không xoá bên đích (gói lưu trữ
#      ghi một lần; tài liệu đã tiêu huỷ thì xoá tay bản sao theo biên bản).
#   3. Ghi sổ sao lưu (bảng backup_runs) để màn Bảo quản và sao lưu thấy, không báo "quá 26 giờ chưa sao lưu".
# Thư mục đích nên ở ổ KHÁC ổ kho (Điều 30 TT05: sao lưu ra phương tiện riêng). Chạy tay:
#   powershell -ExecutionPolicy Bypass -File sao-luu.ps1
param([int]$Giu = 14)
$ErrorActionPreference = "Continue"
$goc = $PSScriptRoot
$log = Join-Path $goc "logs\sao-luu.log"
New-Item -ItemType Directory -Force (Join-Path $goc "logs") | Out-Null
function Ghi($m) { $l = "{0:yyyy-MM-dd HH:mm:ss} {1}" -f (Get-Date), $m; Write-Host $l; Add-Content -Path $log -Value $l -Encoding UTF8 }

$cfg = @{}
foreach ($t in "conf\may-chu.env", "conf\api.env") {
  Get-Content (Join-Path $goc $t) -Encoding UTF8 | ForEach-Object { if ($_ -match '^\s*([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.*)$') { $cfg[$Matches[1]] = $Matches[2].Trim().Trim('"') } }
}
$pgBin = $cfg["PG_BIN"]; $dbUrl = $cfg["DATABASE_URL"]; $dich = $cfg["SAO_LUU_DIR"]; $kho = $cfg["KHO_DIR"]
if (-not ($pgBin -and $dbUrl -and $dich -and $kho)) { Ghi "LỖI thiếu PG_BIN / DATABASE_URL / SAO_LUU_DIR / KHO_DIR trong conf\"; exit 1 }

$psql = Join-Path $pgBin "psql.exe"
# psql.exe (chương trình C) nhận tham số dòng lệnh theo bảng mã ANSI → chữ Việt trong -c thành '?'. SQL có chữ Việt đi qua
# tệp UTF-8 (-f) với PGCLIENTENCODING=UTF8.
function Psql-Tep($sql, [string[]]$ketNoi) {
  $t = Join-Path $env:TEMP ("dt-sql-" + [guid]::NewGuid().ToString("N") + ".sql")
  [IO.File]::WriteAllText($t, $sql, (New-Object System.Text.UTF8Encoding($false)))
  $env:PGCLIENTENCODING = "UTF8"
  & $psql -X -q -v ON_ERROR_STOP=1 @ketNoi -f $t
  $ma = $LASTEXITCODE
  Remove-Item $t -Force; Remove-Item Env:PGCLIENTENCODING -ErrorAction SilentlyContinue
  $global:LASTEXITCODE = $ma
}
function Sq($v) { if ($null -eq $v -or $v -eq "") { "NULL" } else { "'" + ($v -replace "'", "''") + "'" } }
# 0032 (26/11/2026): sổ sao lưu đủ cột Mẫu 02/03 — lịch (theo loại), đường dẫn, dung lượng, mã băm, thời gian; bản cũ bị xoá
# đánh dấu da_xoa_at để màn Bảo quản › Sao lưu không nhắc "bản cũ cần xoá" nữa. Máy chủ chưa áp 0032 thì cột không có → rơi
# về câu cũ.
$co0032 = $false
try { $co0032 = ((& $psql -X -t -A -d $dbUrl -c "SELECT 1 FROM information_schema.columns WHERE table_name='backup_runs' AND column_name='duong_dan'") -join "") -eq "1" } catch { $co0032 = $false }
function Ghi-So($loai, $nguon, $ketQua, $chiTiet, $duongDan = $null, $bytes = $null, $maBam = $null, $batDau = $null) {
  if ($co0032) {
    $t0 = if ($batDau) { "'" + $batDau.ToString("yyyy-MM-dd HH:mm:ss zzz") + "'" } else { "NULL" }
    $sql = "INSERT INTO backup_runs (id, loai, nguon, phuong_tien, ket_qua, chi_tiet, lich_id, duong_dan, dung_luong_bytes, ma_bam, bat_dau_at, ket_thuc_at, nguoi_thuc_hien) VALUES (gen_random_uuid(), '$loai', $(Sq $nguon), $(Sq $dich), '$ketQua', $(Sq $chiTiet), (SELECT id FROM bk_lich_sao_luu WHERE loai = '$loai' AND bat ORDER BY created_at LIMIT 1), $(Sq $duongDan), $(if ($null -ne $bytes) { $bytes } else { 'NULL' }), $(Sq $maBam), $t0, now(), 'sao-luu.ps1');"
  } else {
    $sql = "INSERT INTO backup_runs (id, loai, nguon, phuong_tien, ket_qua, chi_tiet) VALUES (gen_random_uuid(), '$loai', $(Sq $nguon), $(Sq $dich), '$ketQua', $(Sq $chiTiet));"
  }
  Psql-Tep $sql @("-d", $dbUrl) | Out-Null
}
function Danh-Dau-Da-Xoa([string[]]$duongDans) {
  if (-not $co0032 -or $duongDans.Count -eq 0) { return }
  $ds = ($duongDans | ForEach-Object { Sq $_ }) -join ", "
  Psql-Tep "UPDATE backup_runs SET da_xoa_at = now() WHERE da_xoa_at IS NULL AND duong_dan IN ($ds);" @("-d", $dbUrl) | Out-Null
}

# 1. CSDL
$csdlDir = Join-Path $dich "csdl"
New-Item -ItemType Directory -Force $csdlDir | Out-Null
$tep = Join-Path $csdlDir ("digitext-{0:yyyyMMdd-HHmm}.dump" -f (Get-Date))
$t0 = Get-Date
& (Join-Path $pgBin "pg_dump.exe") -d $dbUrl -Fc --no-owner -f "$tep.part"
$okCsdl = ($LASTEXITCODE -eq 0) -and (Test-Path "$tep.part")
if ($okCsdl) {
  & (Join-Path $pgBin "pg_restore.exe") --list "$tep.part" | Out-Null
  $okCsdl = $LASTEXITCODE -eq 0
}
if ($okCsdl) {
  Move-Item "$tep.part" $tep -Force
  $bytes = (Get-Item $tep).Length; $mb = [math]::Round($bytes / 1MB, 1)
  $bam = try { (Get-FileHash -Algorithm SHA256 $tep).Hash.ToLower() } catch { $null }
  Ghi "CSDL OK $tep ($mb MB, $([int]((Get-Date) - $t0).TotalSeconds) s)"
  Ghi-So "day_du" "CSDL PostgreSQL (pg_dump)" "dat" "$tep — $mb MB, đọc lại mục lục đạt" $tep $bytes $bam $t0
  $cu = @(Get-ChildItem $csdlDir -Filter "digitext-*.dump" | Sort-Object Name -Descending | Select-Object -Skip $Giu)
  $cu | Remove-Item -Force
  Danh-Dau-Da-Xoa @($cu | ForEach-Object { $_.FullName })
} else {
  Ghi "CSDL LỖI (pg_dump/pg_restore mã $LASTEXITCODE)"
  if (Test-Path "$tep.part") { Remove-Item "$tep.part" -Force }
  Ghi-So "day_du" "CSDL PostgreSQL (pg_dump)" "khong_dat" "pg_dump hoặc kiểm đọc lại lỗi — xem logs\sao-luu.log"
}

# 2. Kho tài liệu (robocopy mã < 8 là thành công).
$khoDich = Join-Path $dich "kho"
$t0 = Get-Date
robocopy $kho $khoDich /E /R:2 /W:5 /NP /NFL /NDL /XJ /LOG+:"$(Join-Path $goc 'logs\sao-luu-kho.log')" | Out-Null
$ma = $LASTEXITCODE
if ($ma -lt 8) {
  Ghi "Kho OK → $khoDich (robocopy mã $ma, $([int]((Get-Date) - $t0).TotalSeconds) s)"
  Ghi-So "gia_tang" "Kho tài liệu ($kho)" "dat" "robocopy /E mã $ma → $khoDich" $khoDich $null $null $t0
} else {
  Ghi "Kho LỖI robocopy mã $ma"
  Ghi-So "gia_tang" "Kho tài liệu ($kho)" "khong_dat" "robocopy mã $ma — xem logs\sao-luu-kho.log"
}
if (-not $okCsdl -or $ma -ge 8) { exit 1 }
