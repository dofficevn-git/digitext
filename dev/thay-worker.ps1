$ErrorActionPreference = "Continue"
$moi = 'C:\DigiText-ChuyenMay\digitext-ocr-songsong.exe'
$dich = 'C:\DigiText\target\release\digitext-ocr.exe'
if (-not (Test-Path $moi)) { "KHONG THAY $moi - dung lai"; exit 1 }
"=== ban moi"
Get-Item $moi | Select-Object Length, LastWriteTime | Format-Table -AutoSize | Out-String
(Get-FileHash $moi -Algorithm SHA256).Hash

"=== dung tien trinh worker (supervisor se tu bat lai sau vai giay)"
$p = Get-Process digitext-ocr -ErrorAction SilentlyContinue
if ($p) { "dang chay pid=$($p.Id), start=$($p.StartTime)"; Stop-Process -Id $p.Id -Force; Start-Sleep 4 } else { "khong thay tien trinh" }

"=== thay tep"
$ts = Get-Date -Format 'yyyyMMdd-HHmm'
$han = (Get-Date).AddSeconds(60)
do {
  try { Rename-Item $dich "digitext-ocr.exe.bak-$ts" -Force -ErrorAction Stop; $doi = $true } catch { $doi = $false; Start-Sleep 2 }
} until ($doi -or (Get-Date) -gt $han)
if (-not $doi) { "KHONG doi ten duoc (tep con bi giu) - dung lai"; exit 1 }
Copy-Item $moi $dich -Force
"ban cu giu lai: digitext-ocr.exe.bak-$ts"
(Get-FileHash $dich -Algorithm SHA256).Hash

"=== cho supervisor bat lai (toi da 120s)"
$t0 = Get-Date; $len = $null
do {
  Start-Sleep 5
  $len = Get-Process digitext-ocr -ErrorAction SilentlyContinue
} until ($len -or ((Get-Date) - $t0).TotalSeconds -gt 120)
if ($len) { "da len sau {0:N0}s: pid=$($len.Id)" -f ((Get-Date) - $t0).TotalSeconds } else { "CHUA LEN - xem C:\DigiText\logs" }
Start-Sleep 20
"=== log worker 12 dong cuoi"
Get-Content C:\DigiText\logs\worker.out.log -Tail 12 -ErrorAction SilentlyContinue
"=== XONG"
