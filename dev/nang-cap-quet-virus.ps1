$ErrorActionPreference = "Continue"
$sql = 'C:\DigiText-ChuyenMay\0027_quet_virus_tai_lieu.sql'
$moi = 'C:\DigiText-ChuyenMay\digitext-ocr-quetvirus.exe'
$dich = 'C:\DigiText\target\release\digitext-ocr.exe'

"=== 1. ap migration 0027 (them cot virus_scan vao documents)"
if (-not (Test-Path $sql)) { "KHONG THAY $sql - dung lai"; exit 1 }
docker cp $sql digitext-dev-postgres-1:/tmp/0027.sql
docker exec digitext-dev-postgres-1 psql -U digitext -d digitext -v ON_ERROR_STOP=1 -f /tmp/0027.sql
if ($LASTEXITCODE -ne 0) { "AP MIGRATION LOI - dung lai, chua thay worker"; exit 1 }

"=== 2. kiem cot da co chua"
$co = docker exec digitext-dev-postgres-1 psql -U digitext -d digitext -At -c "select count(*) from information_schema.columns where table_name='documents' and column_name in ('virus_scan','virus_scan_at')"
"so cot tim thay: $co (can 2)"
if ("$co".Trim() -ne "2") { "CHUA DU COT - dung lai, chua thay worker"; exit 1 }

"=== 3. thay worker"
if (-not (Test-Path $moi)) { "KHONG THAY $moi - dung lai"; exit 1 }
(Get-FileHash $moi -Algorithm SHA256).Hash
$p = Get-Process digitext-ocr -ErrorAction SilentlyContinue
if ($p) { "dang chay pid=$($p.Id)"; Stop-Process -Id $p.Id -Force; Start-Sleep 4 }
$ts = Get-Date -Format 'yyyyMMdd-HHmm'
$han = (Get-Date).AddSeconds(60)
do {
  try { Rename-Item $dich "digitext-ocr.exe.bak-$ts" -Force -ErrorAction Stop; $doi = $true } catch { $doi = $false; Start-Sleep 2 }
} until ($doi -or (Get-Date) -gt $han)
if (-not $doi) { "KHONG doi ten duoc (tep con bi giu) - dung lai"; exit 1 }
Copy-Item $moi $dich -Force
"ban cu giu lai: digitext-ocr.exe.bak-$ts"
(Get-FileHash $dich -Algorithm SHA256).Hash

"=== 4. cho supervisor bat lai (toi da 120s)"
$t0 = Get-Date; $len = $null
do { Start-Sleep 5; $len = Get-Process digitext-ocr -ErrorAction SilentlyContinue } until ($len -or ((Get-Date) - $t0).TotalSeconds -gt 120)
if ($len) { "da len: pid=$($len.Id)" } else { "CHUA LEN - xem C:\DigiText\logs"; exit 1 }
Start-Sleep 30

"=== 5. log worker"
Get-Content C:\DigiText\logs\worker.out.log -Tail 14 -ErrorAction SilentlyContinue
"=== 6. trang thai goi"
docker exec digitext-dev-postgres-1 psql -U digitext -d digitext -c "select status, count(*) from packages group by 1 order by 2 desc"
"=== 7. tai lieu da quet virus"
docker exec digitext-dev-postgres-1 psql -U digitext -d digitext -c "select coalesce(left(virus_scan,22),'(chua quet)') as ket_qua, count(*) from documents where status <> 'disposed' group by 1 order by 2 desc"
"=== XONG"
