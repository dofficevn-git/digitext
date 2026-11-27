# Kịch bản thử API đợt 4 (27/11/2026): chuyển đổi tài liệu số ra giấy PL VI (0037) + phân quyền theo phông (0038) — chạy TRÊN MÁY NÀY
# (máy dev), không đụng máy chủ thật. Cần: Postgres TẠM (container postgres:16 đã áp 0001→0038), `cargo build -p digitext-api` (debug),
# docker CLI, python (đếm trang PDF bằng pymupdf nếu có). Script XOÁ SẠCH users/projects của CSDL tạm rồi dựng dữ liệu nhỏ.
#   powershell -ExecutionPolicy Bypass -File dev\luutru\thu-dot-4.ps1 -MatKhauDb <mk>
param(
  [Parameter(Mandatory = $true)][string]$MatKhauDb,
  [string]$Container = "dt-luutru-thu",
  [int]$CongDb = 55432,
  [int]$CongApi = 18085,
  [string]$ThuMucTam = (Join-Path $env:TEMP "digitext-thu-dot-4")
)
$ErrorActionPreference = "Continue"
$repo = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path; $sb = $ThuMucTam; New-Item -ItemType Directory -Force $sb | Out-Null
$env:DATABASE_URL = "postgres://digitext:$MatKhauDb@127.0.0.1:$CongDb/digitext"
$env:DIGITEXT_EDITION = "luu_tru"; $env:DIGITEXT_BIND = "127.0.0.1:$CongApi"; $env:DIGITEXT_PUBLIC_URL = "http://127.0.0.1:$CongApi"
$env:DIGITEXT_DATA_DIR = Join-Path $sb "data"; $env:DIGITEXT_UPDATES_DIR = Join-Path $sb "updates"
$env:DIGITEXT_JWT_KEY_PATH = Join-Path $sb "data\jwt-ed25519.pem"; $env:DIGITEXT_PDF_STORE_DIR = Join-Path $sb "kho"
$env:DIGITEXT_MIGRATIONS_DIR = Join-Path $repo "migrations"; $env:DIGITEXT_CERT_FONT = "C:\Windows\Fonts\times.ttf"
New-Item -ItemType Directory -Force (Join-Path $sb "data"), (Join-Path $sb "kho") | Out-Null
$api = Join-Path $repo "target\debug\digitext-api.exe"
$B = "http://127.0.0.1:$CongApi/api/v1"
function Sql($q) { docker exec $Container psql -X -t -A -U digitext -d digitext -c $q }
function Call($m, $path, $tok, $body) {
  $h = @{}; if ($tok) { $h.Authorization = "Bearer $tok" }
  try {
    $p = @{ Method = $m; Uri = "$B$path"; Headers = $h; UseBasicParsing = $true; TimeoutSec = 180 }
    if ($null -ne $body) { $p.ContentType = "application/json; charset=utf-8"; $p.Body = [Text.Encoding]::UTF8.GetBytes(($body | ConvertTo-Json -Depth 8 -Compress)) }
    $r = Invoke-WebRequest @p
    $raw = $r.RawContentStream.ToArray()
    $ct = [string]$r.Headers["Content-Type"]
    if ($ct -match "json") { $txt = [Text.Encoding]::UTF8.GetString($raw); return @{ code = [int]$r.StatusCode; json = $(if ($txt) { $txt | ConvertFrom-Json } else { $null }); bytes = $raw } }
    return @{ code = [int]$r.StatusCode; json = $null; bytes = $raw; ct = $ct }
  } catch {
    $resp = $_.Exception.Response
    if ($null -eq $resp) { return @{ code = 0; json = $null; loi = $_.Exception.Message } }
    $sr = New-Object IO.StreamReader($resp.GetResponseStream(), [Text.Encoding]::UTF8); $txt = $sr.ReadToEnd()
    if (-not $txt) { $txt = [string]$_.ErrorDetails.Message }
    return @{ code = [int]$resp.StatusCode; json = $(try { $txt | ConvertFrom-Json } catch { $txt }); raw = $txt }
  }
}
$script:sai = 0
function Chk($ten, $dk, $them) { if ($dk) { Write-Host ("  OK   {0} {1}" -f $ten, $them) -ForegroundColor Green } else { $script:sai++; Write-Host ("  SAI  {0} {1}" -f $ten, $them) -ForegroundColor Red } }
function Than($j) { if ($null -eq $j) { return "" }; if ($j -is [string]) { return $j }; $s = ($j | ConvertTo-Json -Compress -Depth 4); return $s.Substring(0, [Math]::Min(220, $s.Length)) }
function SoTrangPdf($path) { try { $o = & python -c "import pymupdf,sys; print(len(pymupdf.open(sys.argv[1])))" $path; return [int]$o } catch { return -1 } }

Write-Host "===== 0. don du lieu cu + superadmin"
Sql "TRUNCATE projects, users CASCADE" | Out-Null
# TRUNCATE CASCADE lam trong ca bang ma dung chung (cl_thbq / cl_ngon_ngu / cl_loai_van_ban, project_id NULL) -> nap lai tu 0012.
Get-Content (Join-Path $PSScriptRoot "seed-dung-chung-0012.sql") -Raw -Encoding UTF8 | docker exec -i $Container psql -X -q -U digitext -d digitext | Out-Null
$mk = Join-Path $sb "mk.txt"; [IO.File]::WriteAllText($mk, "ThuNghiemCucBo2026")
& $api bootstrap-superadmin --username quantri --display-name "Quan tri" --password-file $mk 2>&1 | Select-Object -Last 1

$p = Start-Process -FilePath $api -ArgumentList "serve" -NoNewWindow -PassThru -RedirectStandardOutput (Join-Path $sb "api.out.log") -RedirectStandardError (Join-Path $sb "api.err.log")
try {
  $han = (Get-Date).AddSeconds(40)
  do { Start-Sleep -Milliseconds 600; try { $ok = (Invoke-WebRequest -UseBasicParsing -TimeoutSec 3 "http://127.0.0.1:$CongApi/healthz").StatusCode -eq 200 } catch { $ok = $false } } until ($ok -or (Get-Date) -gt $han)
  if (-not $ok) { Write-Host "API khong len"; Get-Content (Join-Path $sb "api.err.log") -Tail 15; exit 1 }

  Write-Host "===== 1. du lieu: du an, 2 phong (A, B), moi phong 1 phong ban + 1 ho so + 1 van ban + 1 tai lieu co PDF"
  $pid1 = [guid]::NewGuid().ToString(); $phA = [guid]::NewGuid().ToString(); $phB = [guid]::NewGuid().ToString()
  $pbA = [guid]::NewGuid().ToString(); $pbB = [guid]::NewGuid().ToString()
  Sql "INSERT INTO projects (id, code, name) VALUES ('$pid1', 'D4', 'Co quan thu dot 4')" | Out-Null
  Sql "INSERT INTO project_admins (project_id, admin_id) SELECT '$pid1', id FROM users WHERE username = 'quantri'" | Out-Null
  Sql "INSERT INTO cl_phong (id, project_id, ma_du_an, ten, phong_so) VALUES ('$phA', '$pid1', 'PA', 'Phong A', '01'), ('$phB', '$pid1', 'PB', 'Phong B', '02')" | Out-Null
  Sql "INSERT INTO cl_phong_ban (id, project_id, phong_id, ma_don_vi, ten) VALUES ('$pbA', '$pid1', '$phA', 'VPA', 'Van phong A'), ('$pbB', '$pid1', '$phB', 'VPB', 'Van phong B')" | Out-Null
  $hsA = [guid]::NewGuid().ToString(); $hsB = [guid]::NewGuid().ToString(); $vbA = [guid]::NewGuid().ToString(); $vbB = [guid]::NewGuid().ToString()
  $dA = [guid]::NewGuid().ToString(); $dB = [guid]::NewGuid().ToString()
  Sql "INSERT INTO cl_ho_so (id, project_id, phong_id, phong_ban_id, so_tam, ho_so_so, tieu_de, ngay_ket_thuc) VALUES ('$hsA', '$pid1', '$phA', '$pbA', '01', '01', 'Ho so A', '2024'), ('$hsB', '$pid1', '$phB', '$pbB', '01', '01', 'Ho so B', '2024')" | Out-Null
  Sql "INSERT INTO cl_van_ban (id, project_id, ho_so_id, so_thu_tu, so_ky_hieu, ngay_thang, trich_yeu, co_quan_ban_hanh) VALUES ('$vbA', '$pid1', '$hsA', 1, '12/QD-UBND', '05/03/2024', 'Quyet dinh A', 'UBND'), ('$vbB', '$pid1', '$hsB', 1, '13/QD-UBND', '06/03/2024', 'Quyet dinh B', 'UBND')" | Out-Null
  Sql "INSERT INTO documents (id, project_id, filename, ho_so_ref, size_bytes, blake3, storage_path, status, page_count, uploaded_at) VALUES ('$dA', '$pid1', 'a.pdf', 'HSA', 1000, '$('a' * 64)', 'x/a', 'archived', 2, now()), ('$dB', '$pid1', 'b.pdf', 'HSB', 1000, '$('b' * 64)', 'x/b', 'archived', 2, now())" | Out-Null
  Sql "INSERT INTO cl_van_ban_file (id, project_id, van_ban_id, document_id, name) VALUES ('$([guid]::NewGuid())', '$pid1', '$vbA', '$dA', 'a.pdf'), ('$([guid]::NewGuid())', '$pid1', '$vbB', '$dB', 'b.pdf')" | Out-Null
  $qt = (Call POST "/auth/login" $null @{ username = "quantri"; password = "ThuNghiemCucBo2026" }).json.accessToken
  Chk "login quantri" ($qt) ""
  # PDF thật cho kho: lấy một báo cáo PDF của chính API (2 trang khong chac; chi can la PDF hop le) ghi vao {project}-warm/{doc}.pdf
  $r = Call GET "/bao-cao/phong?projectId=$pid1&format=pdf" $qt
  Chk "co PDF mau tu bao cao" ($r.code -eq 200 -and $r.bytes.Length -gt 1000) ("bytes=" + $r.bytes.Length)
  $warm = Join-Path $sb "kho\$pid1-warm"; New-Item -ItemType Directory -Force $warm | Out-Null
  [IO.File]::WriteAllBytes((Join-Path $warm "$dA.pdf"), $r.bytes); [IO.File]::WriteAllBytes((Join-Path $warm "$dB.pdf"), $r.bytes)
  $soTrangGoc = SoTrangPdf (Join-Path $warm "$dA.pdf")
  $u1 = Call POST "/users" $qt @{ username = "canbo"; password = "MatKhau12345"; displayName = "Can bo LT"; role = "admin"; projectId = $pid1 }
  $u2 = Call POST "/users" $qt @{ username = "nvc"; password = "MatKhau12345"; displayName = "Nhan vien C"; role = "user"; projectId = $pid1 }
  Chk "tao canbo + nvc" ($u1.code -eq 201 -and $u2.code -eq 201) ""
  $tc = (Call POST "/auth/login" $null @{ username = "canbo"; password = "MatKhau12345" }).json.accessToken
  $tn = (Call POST "/auth/login" $null @{ username = "nvc"; password = "MatKhau12345" }).json.accessToken

  Write-Host "===== 2. phan quyen theo phong (0038)"
  $r = Call GET "/users/$($u2.json.id)/phong?projectId=$pid1" $qt; Chk "GET phong cua nvc: rong (khong gioi han)" ($r.code -eq 200 -and @($r.json.phongIds).Count -eq 0) ("code=" + $r.code)
  $r = Call GET "/cl/ho-so?projectId=$pid1" $tn; Chk "nvc chua gioi han: thay 2 ho so" ($r.json.total -eq 2) ("total=" + $r.json.total)
  $r = Call PUT "/users/$($u2.json.id)/phong" $qt @{ projectId = $pid1; phongIds = @($phA) }
  Chk "PUT gioi han nvc o phong A" ($r.code -eq 200 -and @($r.json.phongIds).Count -eq 1 -and $r.json.tenPhong[0][1] -match "Phong A") ("code=" + $r.code + " " + (Than $r.json))
  $r = Call PUT "/users/$($u2.json.id)/phong" $tn @{ projectId = $pid1; phongIds = @() }; Chk "nvc tu bo gioi han -> 403/400" ($r.code -ge 400) ("code=" + $r.code)
  $qid = (Sql "SELECT id FROM users WHERE username = 'quantri'")
  $r = Call PUT "/users/$qid/phong" $qt @{ projectId = $pid1; phongIds = @($phA) }; Chk "gioi han super_admin -> 400" ($r.code -eq 400) ("code=" + $r.code)
  $r = Call GET "/cl/pham-vi-phong?projectId=$pid1" $tn; Chk "/cl/pham-vi-phong nvc = [A]" (@($r.json.phongIds).Count -eq 1 -and $r.json.phongIds[0] -eq $phA) (Than $r.json)
  $r = Call GET "/cl/pham-vi-phong?projectId=$pid1" $tc; Chk "canbo khong gioi han (null)" ($r.code -eq 200 -and $null -eq $r.json.phongIds) ""
  $r = Call GET "/cl/ho-so?projectId=$pid1" $tn; Chk "nvc: /cl/ho-so chi phong A" ($r.json.total -eq 1 -and $r.json.rows[0].phong_id -eq $phA) ("total=" + $r.json.total)
  $r = Call GET "/cl/ho-so?projectId=$pid1&phongId=$phB" $tn; Chk "nvc chon phong B -> 403" ($r.code -eq 403) ("code=" + $r.code)
  $r = Call GET "/cl/ho-so/$hsB" $tn; Chk "nvc GET ho so B -> 403" ($r.code -eq 403) ("code=" + $r.code)
  $r = Call GET "/cl/ho-so/$hsA" $tn; Chk "nvc GET ho so A -> 200" ($r.code -eq 200) ("code=" + $r.code)
  $r = Call GET "/cl/ho-so/$hsB/van-ban" $tn; Chk "nvc van ban cua ho so B -> 403" ($r.code -eq 403) ("code=" + $r.code)
  $r = Call GET "/cl/van-ban?projectId=$pid1" $tn; Chk "nvc tim van ban: 1" ($r.json.total -eq 1) ("total=" + $r.json.total)
  $r = Call GET "/cl/catalog?projectId=$pid1" $tn; Chk "nvc catalog: 1 phong, 1 phong ban" (@($r.json.phong).Count -eq 1 -and @($r.json.phong_ban).Count -eq 1 -and $r.json.phong[0].id -eq $phA) ("phong=" + @($r.json.phong).Count + " pb=" + @($r.json.phong_ban).Count)
  $r = Call GET "/cl/catalog?projectId=$pid1" $tc; Chk "canbo catalog: 2 phong" (@($r.json.phong).Count -eq 2) ("phong=" + @($r.json.phong).Count)
  $r = Call GET "/tra-cuu?projectId=$pid1" $tn; Chk "nvc tra cuu: 1 dong (phong A)" ($r.json.tong -eq 1 -and $r.json.dong[0].phongId -eq $phA) ("tong=" + $r.json.tong)
  $r = Call GET "/tra-cuu/bo-loc?projectId=$pid1" $tn; Chk "nvc bo loc: 1 phong" (@($r.json.phong).Count -eq 1) ""
  $r = Call GET "/tra-cuu/tai-lieu/$dB/doc" $tn; Chk "nvc doc tai lieu phong B -> 403" ($r.code -eq 403) ("code=" + $r.code)
  $r = Call GET "/tra-cuu/tai-lieu/$dA/doc" $tn; Chk "nvc doc tai lieu phong A -> 200 PDF" ($r.code -eq 200 -and [Text.Encoding]::ASCII.GetString($r.bytes[0..4]) -eq "%PDF-") ("code=" + $r.code)
  $r = Call GET "/cl/sync?projectId=$pid1&since=0&limit=2000" $tn
  $rows = @($r.json.rows); $phongSync = @($rows | Where-Object { $_.table -eq "cl_phong" }); $hsSync = @($rows | Where-Object { $_.table -eq "cl_ho_so" })
  Chk "nvc pull offline: 1 phong, 1 ho so, 1 van ban" ($phongSync.Count -eq 1 -and $hsSync.Count -eq 1 -and @($rows | Where-Object { $_.table -eq "cl_van_ban" }).Count -eq 1) ("phong=" + $phongSync.Count + " hs=" + $hsSync.Count)
  $r = Call GET "/cl/sync?projectId=$pid1&since=0&limit=2000" $tc; Chk "canbo pull: 2 phong" (@($r.json.rows | Where-Object { $_.table -eq "cl_phong" }).Count -eq 2) ""
  $r = Call POST "/cl/muon-tra" $tn @{ projectId = $pid1; hoSoIds = @($hsB); nguoiMuon = "X" }; Chk "nvc lap phieu muon ho so B -> 400 (ngoai pham vi)" ($r.code -eq 400) ("code=" + $r.code)
  $r = Call POST "/cl/muon-tra" $tn @{ projectId = $pid1; hoSoIds = @($hsA); nguoiMuon = "X" }; Chk "nvc lap phieu muon ho so A -> 201" ($r.code -eq 201) ("code=" + $r.code)
  $r = Call GET "/cl/muon-tra?projectId=$pid1" $tc; $mtB = Call GET "/cl/muon-tra?projectId=$pid1" $tn
  Chk "danh sach muon: canbo 1, nvc 1" ($r.json.total -eq 1 -and $mtB.json.total -eq 1) ""
  $r = Call GET "/bao-cao/phong?projectId=$pid1" $tn; Chk "bao cao phong cua nvc: 1 dong" (@($r.json.rows).Count -eq 1) ("n=" + @($r.json.rows).Count)
  $r = Call GET "/bao-cao/phong?projectId=$pid1&phongId=$phB" $tn; Chk "bao cao phong B cua nvc -> 403" ($r.code -eq 403) ("code=" + $r.code)
  $r = Call GET "/cl/thong-ke/chinh-ly?projectId=$pid1" $tn; Chk "thong ke chinh ly nvc: 1 phong ban" (@($r.json).Count -eq 1) ("n=" + @($r.json).Count)
  $r = Call GET "/cl/lich-su?projectId=$pid1&phongId=$phB" $tn; Chk "lich su phong B -> 403" ($r.code -eq 403) ("code=" + $r.code)
  $r = Call GET "/cl/kho-gia/chua-hop?projectId=$pid1&phongId=$phB" $tn; Chk "chua-hop phong B -> 403" ($r.code -eq 403) ("code=" + $r.code)
  $r = Call POST "/cl/ho-so/import" $tn @{ projectId = $pid1; phongId = $phB; rows = @() }; Chk "nhap muc luc phong B -> 403" ($r.code -eq 403) ("code=" + $r.code)
  $r = Call PUT "/users/$($u2.json.id)/phong" $qt @{ projectId = $pid1; phongIds = @() }; Chk "bo gioi han" ($r.code -eq 200 -and @($r.json.phongIds).Count -eq 0) ""
  $r = Call GET "/cl/ho-so?projectId=$pid1" $tn; Chk "nvc lai thay 2 ho so" ($r.json.total -eq 2) ("total=" + $r.json.total)

  Write-Host "===== 3. chuyen doi tai lieu so ra giay (0037, PL VI)"
  $nam = (Get-Date).Year
  $r = Call POST "/chuyen-doi" $tn @{ projectId = $pid1; documentId = $dA }; Chk "nvc (khong access.review) -> 403" ($r.code -eq 403) ("code=" + $r.code)
  $r = Call POST "/chuyen-doi" $tc @{ projectId = $pid1; documentId = $dA; mucDich = "Cung cap theo yeu cau"; nguoiNhan = "So Noi vu"; canCu = "Phieu YC 01" }
  $cd = $r.json; Chk "canbo chuyen doi tai lieu A: 201, so CD-$nam-000001, chu ky 'Chua ky'" ($r.code -eq 201 -and $cd.so_chuyen_doi -eq "CD-$nam-000001" -and $cd.ky_so_ket_luan -match '^Ch.a k.$' -and $cd.ma_tai_lieu -match "0000001$") ("code=" + $r.code + " " + (Than $cd))
  $r = Call GET "/chuyen-doi/$($cd.id)/download" $tc
  $f = Join-Path $sb "chuyen-doi-1.pdf"; if ($r.bytes) { [IO.File]::WriteAllBytes($f, $r.bytes) }
  $soTrang = SoTrangPdf $f
  Chk "tai PDF da ghep: %PDF, so trang = goc + 1" ($r.code -eq 200 -and [Text.Encoding]::ASCII.GetString($r.bytes[0..4]) -eq "%PDF-" -and ($soTrangGoc -lt 0 -or $soTrang -eq $soTrangGoc + 1)) ("code=" + $r.code + " trang=" + $soTrang + " goc=" + $soTrangGoc)
  $r = Call POST "/chuyen-doi" $tc @{ projectId = $pid1; documentId = $dB }; Chk "chuyen doi lan 2 -> CD-$nam-000002" ($r.code -eq 201 -and $r.json.so_chuyen_doi -eq "CD-$nam-000002") ("so=" + $r.json.so_chuyen_doi)
  $r = Call GET "/chuyen-doi?projectId=$pid1" $tc; Chk "so chuyen doi: 2 dong, co ten nguoi + ten phong" ($r.json.total -eq 2 -and $r.json.rows[0].nguoi_thuc_hien_ten -eq "Can bo LT" -and $r.json.rows[0].ten_phong) (Than $r.json.rows[0])
  $r = Call GET "/chuyen-doi?projectId=$pid1&format=xlsx" $tc; Chk "so chuyen doi Excel" ($r.code -eq 200 -and $r.bytes[0] -eq 0x50 -and $r.bytes[1] -eq 0x4B) ("code=" + $r.code)
  $r = Call GET "/tra-cuu/nhat-ky?projectId=$pid1" $tc; Chk "nhat ky co khai_thac.chuyen_doi x2" (@($r.json | Where-Object { $_.action -eq "khai_thac.chuyen_doi" }).Count -eq 2) ("n=" + @($r.json | Where-Object { $_.action -eq "khai_thac.chuyen_doi" }).Count)
  $r = Call GET "/bao-cao/chuyen_doi?projectId=$pid1" $tc; Chk "bao cao #19 so chuyen doi: 2 dong" (@($r.json.rows).Count -eq 2) ("n=" + @($r.json.rows).Count)
  $r = Call GET "/bao-cao" $tc; Chk "danh muc bao cao ban Luu tru = 19" (@($r.json.baoCao).Count -eq 19) ("n=" + @($r.json.baoCao).Count)
  # Gioi han nvc o phong A, cap access.review cho nvc roi chuyen doi tai lieu B -> 403, A -> 201
  $r = Call PUT "/users/$($u2.json.id)/phong" $qt @{ projectId = $pid1; phongIds = @($phA) } | Out-Null
  $r = Call PUT "/users/$($u2.json.id)/permissions" $qt @{ grants = @("access.review"); denies = @() }; Chk "cap access.review cho nvc" ($r.code -eq 200) ("code=" + $r.code)
  $tn = (Call POST "/auth/login" $null @{ username = "nvc"; password = "MatKhau12345" }).json.accessToken
  $r = Call POST "/chuyen-doi" $tn @{ projectId = $pid1; documentId = $dB }; Chk "nvc (gioi han A) chuyen doi tai lieu B -> 403" ($r.code -eq 403) ("code=" + $r.code)
  $r = Call POST "/chuyen-doi" $tn @{ projectId = $pid1; documentId = $dA }; Chk "nvc chuyen doi tai lieu A -> 201 (000003)" ($r.code -eq 201 -and $r.json.so_chuyen_doi -eq "CD-$nam-000003") ("code=" + $r.code)
  $r = Call GET "/chuyen-doi?projectId=$pid1" $tn; Chk "so cua nvc chi dong phong A (2)" ($r.json.total -eq 2) ("total=" + $r.json.total)
  Write-Host ("   tep xem mat: " + $f)

  Write-Host ("===== KET QUA: " + $(if ($script:sai -eq 0) { "TAT CA DAT" } else { "$($script:sai) buoc SAI" }))
} finally {
  if ($p -and -not $p.HasExited) { Stop-Process -Id $p.Id -Force }
}
exit $script:sai
