# Kiểm một hay nhiều tệp PDF có phải PDF/A không — KHÔNG phụ thuộc trình xem PDF.
# Cách dùng: kéo thả tệp PDF (hoặc cả thư mục) vào "Kiem PDF-A.bat" trên Desktop, hoặc:
#   powershell -ExecutionPolicy Bypass -File D:\DigiText\dev\kiem-pdfa.ps1 <tệp hoặc thư mục> [...]
#
# In ra hai thứ cho mỗi tệp:
#   1. Tệp TỰ KHAI gì: dòng pdfaid trong metadata + có nhúng hồ sơ màu (OutputIntent) không.
#      (Dải "PDF/A" của Foxit/Acrobat chỉ đọc đúng dòng khai này, và chỉ hiện khi bật tuỳ chọn.)
#   2. veraPDF (bộ kiểm chuẩn ISO 19005, chạy qua Docker) kết luận ĐẠT / KHÔNG ĐẠT PDF/A-2b
#      kèm các luật vi phạm. Đây mới là kết quả dùng để nghiệm thu.
param([Parameter(ValueFromRemainingArguments = $true)][string[]]$Paths)
$ErrorActionPreference = "Continue"
[Console]::OutputEncoding = [Text.Encoding]::UTF8
if (-not $Paths -or $Paths.Count -eq 0) { Write-Host "Kéo thả tệp PDF vào tệp .bat này." -ForegroundColor Yellow; Read-Host "Enter để đóng"; exit 2 }

$files = @()
foreach ($p in $Paths) {
  if (Test-Path $p -PathType Container) { $files += Get-ChildItem $p -Filter *.pdf -File | Select-Object -First 200 }
  elseif (Test-Path $p) { $files += Get-Item $p }
}
if ($files.Count -eq 0) { Write-Host "Không thấy tệp PDF nào." -ForegroundColor Yellow; Read-Host "Enter để đóng"; exit 2 }

$latin1 = [Text.Encoding]::GetEncoding(28591)
$docker = [bool](Get-Command docker -ErrorAction SilentlyContinue)
$dat = 0; $khong = 0
foreach ($f in $files) {
  Write-Host ""
  Write-Host "== $($f.Name)" -ForegroundColor Cyan
  $txt = $latin1.GetString([IO.File]::ReadAllBytes($f.FullName))
  $part = [regex]::Match($txt, 'pdfaid:part[>="\s]+(\d)')
  $conf = [regex]::Match($txt, 'pdfaid:conformance[>="\s]+([A-Za-z])')
  $khai = if ($part.Success) { "PDF/A-$($part.Groups[1].Value)$($conf.Groups[1].Value.ToUpper())" } else { "KHÔNG khai PDF/A" }
  $mau = if ($txt.Contains("/OutputIntents")) { "có" } else { "KHÔNG có" }
  $ky = if ($txt.Contains("/ByteRange")) { "có" } else { "không" }
  Write-Host "   Tệp tự khai      : $khai"
  Write-Host "   Hồ sơ màu nhúng  : $mau"
  Write-Host "   Chữ ký số        : $ky"
  if ($docker) {
    $xml = & docker run --rm -v "$($f.DirectoryName):/data:ro" verapdf/cli:v1.30.2 -f 2b --format xml "/data/$($f.Name)" 2>$null | Out-String
    if ($xml -match 'isCompliant="true"') {
      $dat++; Write-Host "   veraPDF PDF/A-2b : ĐẠT" -ForegroundColor Green
    } elseif ($xml -match 'isCompliant="false"') {
      $khong++; Write-Host "   veraPDF PDF/A-2b : KHÔNG ĐẠT" -ForegroundColor Red
      [regex]::Matches($xml, 'clause="([^"]+)" testNumber="(\d+)"') | ForEach-Object { "      vi phạm luật ISO 19005-2 mục $($_.Groups[1].Value), phép thử $($_.Groups[2].Value)" } | Select-Object -Unique | Select-Object -First 8 | ForEach-Object { Write-Host $_ }
    } else {
      Write-Host "   veraPDF          : không chạy được (Docker Desktop đã bật chưa?)" -ForegroundColor Yellow
    }
  } else {
    Write-Host "   veraPDF          : bỏ qua — máy này không có Docker" -ForegroundColor Yellow
  }
}
Write-Host ""
Write-Host "TỔNG: $($files.Count) tệp · veraPDF ĐẠT $dat · KHÔNG ĐẠT $khong" -ForegroundColor White
Read-Host "Enter để đóng"
