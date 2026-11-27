# Kịch bản thử API đợt 5 (27/11/2026): cấu hình cơ quan lưu trữ + lịch sử phông (0039), bịt phạm vi phông ở /cl/rows, push đồng bộ,
# khai thác, cung cấp thông tin, nộp lưu, danh mục hồ sơ — chạy TRÊN MÁY NÀY (máy dev), không đụng máy chủ thật.
# Cần: Postgres TẠM (container postgres:16, cổng 55432), `cargo build -p digitext-api` (debug), docker CLI, python + pymupdf (đọc chữ PDF).
# Script tự áp migration còn thiếu (0039) bằng `digitext-api migrate`, XOÁ SẠCH users/projects của CSDL tạm rồi dựng dữ liệu nhỏ.
#   powershell -ExecutionPolicy Bypass -File dev\luutru\thu-dot-5.ps1 -MatKhauDb <mk>
param(
  [Parameter(Mandatory = $true)][string]$MatKhauDb,
  [string]$Container = "dt-luutru-thu",
  [int]$CongDb = 55432,
  [int]$CongApi = 18086,
  [string]$ThuMucTam = (Join-Path $env:TEMP "digitext-thu-dot-5")
)
$ErrorActionPreference = "Continue"
$repo = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path; $sb = $ThuMucTam; New-Item -ItemType Directory -Force $sb | Out-Null
$env:DATABASE_URL = "postgres://digitext:$MatKhauDb@127.0.0.1:$CongDb/digitext"
$env:DIGITEXT_EDITION = "luu_tru"; $env:DIGITEXT_BIND = "127.0.0.1:$CongApi"; $env:DIGITEXT_PUBLIC_URL = "http://127.0.0.1:$CongApi"
$env:DIGITEXT_DATA_DIR = Join-Path $sb "data"; $env:DIGITEXT_UPDATES_DIR = Join-Path $sb "updates"
$env:DIGITEXT_JWT_KEY_PATH = Join-Path $sb "data\jwt-ed25519.pem"; $env:DIGITEXT_PDF_STORE_DIR = Join-Path $sb "kho"
$env:DIGITEXT_MIGRATIONS_DIR = Join-Path $repo "migrations"; $env:DIGITEXT_CERT_FONT = "C:\Windows\Fonts\times.ttf"; $env:PYTHONIOENCODING = "utf-8"
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
function ChuPdf($path) { try { $o = & python -c "import pymupdf,sys; d=pymupdf.open(sys.argv[1]); print(' '.join(p.get_text() for p in d).replace(chr(10),' '))" $path; return [string]$o } catch { return "" } }
function SoTrangPdf($path) { try { $o = & python -c "import pymupdf,sys; print(len(pymupdf.open(sys.argv[1])))" $path; return [int]$o } catch { return -1 } }

Write-Host "===== 0. migrate (0039) + don du lieu cu + superadmin"
& $api migrate 2>&1 | Select-Object -Last 2
Chk "bang co_quan_luu_tru + phong_lich_su" ((Sql "SELECT count(*) FROM information_schema.tables WHERE table_name IN ('co_quan_luu_tru','phong_lich_su')") -eq "2") ""
Sql "TRUNCATE projects, users CASCADE" | Out-Null
Get-Content (Join-Path $PSScriptRoot "seed-dung-chung-0012.sql") -Raw -Encoding UTF8 | docker exec -i $Container psql -X -q -U digitext -d digitext | Out-Null
$mk = Join-Path $sb "mk.txt"; [IO.File]::WriteAllText($mk, "ThuNghiemCucBo2026")
& $api bootstrap-superadmin --username quantri --display-name "Quan tri" --password-file $mk 2>&1 | Select-Object -Last 1

$p = Start-Process -FilePath $api -ArgumentList "serve" -NoNewWindow -PassThru -RedirectStandardOutput (Join-Path $sb "api.out.log") -RedirectStandardError (Join-Path $sb "api.err.log")
try {
  $han = (Get-Date).AddSeconds(40)
  do { Start-Sleep -Milliseconds 600; try { $ok = (Invoke-WebRequest -UseBasicParsing -TimeoutSec 3 "http://127.0.0.1:$CongApi/healthz").StatusCode -eq 200 } catch { $ok = $false } } until ($ok -or (Get-Date) -gt $han)
  if (-not $ok) { Write-Host "API khong len"; Get-Content (Join-Path $sb "api.err.log") -Tail 15; exit 1 }

  Write-Host "===== 1. du lieu: du an, 2 phong (A, B), moi phong 1 phong ban + 1 ho so + 1 van ban + 1 tai lieu"
  $pid1 = [guid]::NewGuid().ToString(); $phA = [guid]::NewGuid().ToString(); $phB = [guid]::NewGuid().ToString()
  $pbA = [guid]::NewGuid().ToString(); $pbB = [guid]::NewGuid().ToString()
  Sql "INSERT INTO projects (id, code, name) VALUES ('$pid1', 'D5', 'Co quan thu dot 5')" | Out-Null
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
  $u1 = Call POST "/users" $qt @{ username = "canbo"; password = "MatKhau12345"; displayName = "Can bo LT"; role = "admin"; projectId = $pid1 }
  $u2 = Call POST "/users" $qt @{ username = "nvc"; password = "MatKhau12345"; displayName = "Nhan vien C"; role = "user"; projectId = $pid1 }
  Chk "tao canbo + nvc" ($u1.code -eq 201 -and $u2.code -eq 201) ""
  $r = Call PUT "/users/$($u2.json.id)/phong" $qt @{ projectId = $pid1; phongIds = @($phA) }; Chk "gioi han nvc o phong A" ($r.code -eq 200) ("code=" + $r.code)
  $tc = (Call POST "/auth/login" $null @{ username = "canbo"; password = "MatKhau12345" }).json.accessToken
  $tn = (Call POST "/auth/login" $null @{ username = "nvc"; password = "MatKhau12345" }).json.accessToken

  Write-Host "===== 2. cau hinh co quan luu tru (0039, B01)"
  $r = Call GET "/projects/$pid1/co-quan" $tc
  Chk "GET co-quan chua khai: hieuDung ten = ten du an, 60/15-7-1/360, daKhai=false" ($r.code -eq 200 -and $null -eq $r.json.cauHinh -and $r.json.hieuDung.ten -eq "Co quan thu dot 5" -and $r.json.hieuDung.hanNopNgay -eq 60 -and $r.json.hieuDung.daKhai -eq $false -and $r.json.hieuDung.chucVuNguoiKy -match "^L.NH") ("code=" + $r.code + " " + (Than $r.json.hieuDung))
  $r = Call PUT "/projects/$pid1/co-quan" $tn @{ ten = "X" }; Chk "nvc (user) PUT co-quan -> 403" ($r.code -eq 403) ("code=" + $r.code)
  $r = Call PUT "/projects/$pid1/co-quan" $tc @{ hanNopNgay = 0 }; Chk "hanNopNgay=0 -> 400" ($r.code -eq 400) ("code=" + $r.code)
  $r = Call PUT "/projects/$pid1/co-quan" $tc @{ maCoQuan = "h49"; ten = "UBND xa Thu Nghiem"; tenCapTren = "UBND huyen Thu"; loaiHinh = "luu_tru_co_quan"; nguoiKy = "Nguyen Van Ky"; chucVuNguoiKy = "Chu tich"; hanNopNgay = 90; nhacTruocNgay = @(30, 7, 1, 7); quyenSuDungNgay = 300; email = "lt@thu.vn" }
  Chk "PUT co-quan (canbo/admin) 200, ma viet hoa, moc sach 30/7/1" ($r.code -eq 200 -and $r.json.cauHinh.maCoQuan -eq "H49" -and $r.json.hieuDung.hanNopNgay -eq 90 -and (@($r.json.hieuDung.nhacTruocNgay) -join ",") -eq "30,7,1" -and $r.json.hieuDung.daKhai -eq $true) ("code=" + $r.code + " " + (Than $r.json.hieuDung))
  $r = Call GET "/projects/$pid1/co-quan" $tn; Chk "nvc GET co-quan (thuoc du an) 200, ten moi" ($r.code -eq 200 -and $r.json.hieuDung.ten -eq "UBND xa Thu Nghiem") ("code=" + $r.code)
  $ad = Sql "SELECT count(*) FROM audit_log WHERE action = 'project.co_quan'"; Chk "audit project.co_quan" ([int]$ad -ge 1) ("n=" + $ad)
  # hạn nộp theo cấu hình: hồ sơ kết thúc 2024 → 31/12/2024 + 90 = 31/03/2025
  $r = Call GET "/nop-luu/ho-so-can-nop?projectId=$pid1" $tc
  $hs = @($r.json.rows | Where-Object { $_.id -eq $hsA })
  Chk "ho-so-can-nop: han_nop = 31/12/2024 + 90 ngay = 2025-03-31" ($hs.Count -eq 1 -and ([string]$hs[0].han_nop) -match "^2025-03-31") ("han=" + $(if ($hs.Count) { $hs[0].han_nop } else { "?" }))
  $r = Call GET "/nop-luu/ho-so-can-nop?projectId=$pid1" $tn; Chk "nvc ho-so-can-nop chi phong A (1)" ($r.json.total -eq 1 -and $r.json.rows[0].phong_id -eq $phA) ("total=" + $r.json.total)
  $r = Call GET "/nop-luu/tong-quan?projectId=$pid1" $tn; Chk "nvc tong-quan nop luu: chuaNop=1" ($r.json.chuaNop -eq 1) (Than $r.json)
  $r = Call GET "/bao-cao/phong?projectId=$pid1&format=pdf" $tc
  $f = Join-Path $sb "bao-cao-phong.pdf"; [IO.File]::WriteAllBytes($f, $r.bytes); $chu = ChuPdf $f
  Chk "PDF bao cao: co ten co quan moi + cap tren + chuc vu/nguoi ky tu cau hinh" ($chu -match "UBND XA THU NGHIEM" -and $chu -match "UBND HUYEN THU" -and $chu -match "CHU TICH" -and $chu -match "Nguyen Van Ky") ("chu=" + $(if ($chu) { $chu.Substring(0, [Math]::Min(160, $chu.Length)) } else { "(rong)" }))

  Write-Host "===== 3. lich su don vi hinh thanh phong (0039, B02)"
  $r = Call GET "/cl/phong/$phA/lich-su" $tc; Chk "GET lich-su rong" ($r.code -eq 200 -and @($r.json.rows).Count -eq 0) ("code=" + $r.code)
  $r = Call POST "/cl/phong/$phA/lich-su" $tc @{ ghiChu = "x" }; Chk "POST thieu noi dung -> 400" ($r.code -eq 400) ("code=" + $r.code)
  $r = Call POST "/cl/phong/$phA/lich-su" $tc @{ lichSuDonVi = "Thanh lap 1998"; thoiGianTaiLieu = "1998-2024"; trangThaiPhong = "mo" }
  Chk "POST phien ban 1" ($r.code -eq 200 -and $r.json.phienBan -eq 1 -and $r.json.nguoiLap -eq "Can bo LT") ("code=" + $r.code + " " + (Than $r.json))
  $r = Call POST "/cl/phong/$phA/lich-su" $tc @{ lichSuDonVi = "Thanh lap 1998; doi ten 2010"; lichSuPhong = "Nop bo sung 2026"; trangThaiPhong = "dong"; nguon = "Thu nop dot 2" }
  Chk "POST phien ban 2" ($r.code -eq 200 -and $r.json.phienBan -eq 2 -and $r.json.trangThaiPhong -eq "dong") ("code=" + $r.code)
  $r = Call GET "/cl/phong/$phA/lich-su" $tc; Chk "GET 2 phien ban, moi nhat truoc" (@($r.json.rows).Count -eq 2 -and $r.json.rows[0].phienBan -eq 2) ("n=" + @($r.json.rows).Count)
  # vai trò user mặc định CÓ catalog.update (sửa danh mục khi chỉnh lý) → tước quyền rồi thử; mục 4 cấp lại bộ quyền khác (denies rỗng).
  $r = Call PUT "/users/$($u2.json.id)/permissions" $qt @{ grants = @(); denies = @("catalog.update") }; Chk "tuoc catalog.update cua nvc" ($r.code -eq 200) ("code=" + $r.code)
  $tn = (Call POST "/auth/login" $null @{ username = "nvc"; password = "MatKhau12345" }).json.accessToken
  $r = Call POST "/cl/phong/$phA/lich-su" $tn @{ lichSuDonVi = "x" }; Chk "nvc (bi tuoc catalog.update) POST -> 403" ($r.code -eq 403) ("code=" + $r.code)
  $r = Call GET "/cl/phong/$phB/lich-su" $tn; Chk "nvc GET lich su phong B -> 403" ($r.code -eq 403) ("code=" + $r.code)
  $r = Call GET "/cl/phong/$phA/lich-su" $tn; Chk "nvc GET lich su phong A -> 200" ($r.code -eq 200) ("code=" + $r.code)

  Write-Host "===== 4. bit pham vi phong: /cl/rows, push, khai thac, cung cap thong tin, danh muc ho so"
  $r = Call GET "/cl/rows/ho_so/$hsB" $tn; Chk "nvc GET /cl/rows/ho_so B -> 403" ($r.code -eq 403) ("code=" + $r.code)
  $r = Call GET "/cl/rows/ho_so/$hsA" $tn; Chk "nvc GET /cl/rows/ho_so A -> 200" ($r.code -eq 200) ("code=" + $r.code)
  $r = Call GET "/cl/rows/van_ban/$vbB" $tn; Chk "nvc GET /cl/rows/van_ban B -> 403" ($r.code -eq 403) ("code=" + $r.code)
  $r = Call GET "/cl/rows/phong_ban/$pbB" $tn; Chk "nvc GET /cl/rows/phong_ban B -> 403" ($r.code -eq 403) ("code=" + $r.code)
  $r = Call PUT "/cl/rows/ho_so/$hsB" $tn @{ projectId = $pid1; new = @{ tieu_de = "Doi ten" } }; Chk "nvc PUT ho so B -> 403" ($r.code -eq 403) ("code=" + $r.code)
  $r = Call PUT "/cl/rows/ho_so/$hsA" $tn @{ projectId = $pid1; new = @{ phong_id = $phB } }; Chk "nvc doi ho so A sang phong B -> 403" ($r.code -eq 403) ("code=" + $r.code)
  $r = Call PUT "/cl/rows/ho_so/$hsA" $tn @{ projectId = $pid1; new = @{ tieu_de = "Ho so A sua" } }; Chk "nvc PUT ho so A -> 200" ($r.code -eq 200) ("code=" + $r.code + " " + (Than $r.json))
  $r = Call POST "/cl/rows/ho_so" $tn @{ projectId = $pid1; new = @{ phong_id = $phB; phong_ban_id = $pbB; so_tam = "99"; tieu_de = "Lan vao B" } }; Chk "nvc tao ho so o phong B -> 403" ($r.code -eq 403) ("code=" + $r.code)
  $r = Call POST "/cl/rows/van_ban" $tn @{ projectId = $pid1; new = @{ ho_so_id = $hsB; so_thu_tu = 2; trich_yeu = "VB lan" } }; Chk "nvc tao van ban trong ho so B -> 403" ($r.code -eq 403) ("code=" + $r.code)
  $r = Call DELETE "/cl/rows/ho_so/$hsB`?projectId=$pid1" $tn; Chk "nvc DELETE ho so B -> 403" ($r.code -eq 403) ("code=" + $r.code)
  $r = Call GET "/cl/rows/ho_so/$hsB" $tc; Chk "canbo (khong gioi han) GET ho so B -> 200" ($r.code -eq 200) ("code=" + $r.code)
  Sql "UPDATE cl_ho_so SET tieu_de = 'Ho so B' WHERE id = '$hsB'" | Out-Null
  # push đồng bộ: op sửa hồ sơ B bị từ chối tại chỗ, op sửa hồ sơ A áp được
  $opB = [guid]::NewGuid().ToString(); $opA = [guid]::NewGuid().ToString()
  $r = Call POST "/cl/sync/push" $tn @{ projectId = $pid1; deviceId = $null; ops = @(
      @{ opId = $opB; table = "cl_ho_so"; id = $hsB; op = "upsert"; new = @{ tieu_de = "B sua offline" } },
      @{ opId = $opA; table = "cl_ho_so"; id = $hsA; op = "upsert"; new = @{ ghi_chu = "A sua offline" } }
    ) }
  $rb = @($r.json.results | Where-Object { $_.id -eq $hsB }); $ra = @($r.json.results | Where-Object { $_.id -eq $hsA })
  Chk "push: op ho so B rejected phong_scope, op ho so A applied" ($r.code -eq 200 -and $rb.Count -eq 1 -and $rb[0].status -eq "rejected" -and $rb[0].reason -eq "phong_scope" -and $ra.Count -eq 1 -and $ra[0].status -ne "rejected") ("code=" + $r.code + " " + (Than $r.json.results))
  $tdB = Sql "SELECT tieu_de FROM cl_ho_so WHERE id = '$hsB'"; Chk "ho so B khong bi sua" ($tdB -eq "Ho so B") ("tieu_de=" + $tdB)
  # khai thác: yêu cầu có tài liệu phông B
  $r = Call POST "/access" $tn @{ projectId = $pid1; kind = "doc"; mucDich = "Nghien cuu"; documentIds = @($dB) }; Chk "nvc yeu cau khai thac tai lieu B -> 403" ($r.code -eq 403) ("code=" + $r.code)
  $r = Call POST "/access" $tn @{ projectId = $pid1; kind = "doc"; mucDich = "Nghien cuu"; documentIds = @($dA) }; Chk "nvc yeu cau khai thac tai lieu A -> 201" ($r.code -eq 201) ("code=" + $r.code + " " + (Than $r.json))
  $r = Call POST "/access" $tc @{ projectId = $pid1; kind = "doc"; mucDich = "Kiem tra"; documentIds = @($dB) }; $ycB = $r.json.id; Chk "canbo yeu cau tai lieu B -> 201" ($r.code -eq 201) ("code=" + $r.code)
  $r = Call PUT "/users/$($u2.json.id)/permissions" $qt @{ grants = @("access.review"); denies = @() }; Chk "cap access.review cho nvc" ($r.code -eq 200) ("code=" + $r.code)
  $tn = (Call POST "/auth/login" $null @{ username = "nvc"; password = "MatKhau12345" }).json.accessToken
  $r = Call GET "/access" $tn; Chk "nvc (duyet) danh sach khai thac: khong thay yeu cau tai lieu B" ($r.code -eq 200 -and @($r.json | Where-Object { $_.id -eq $ycB }).Count -eq 0 -and @($r.json).Count -ge 1) ("n=" + @($r.json).Count)
  $r = Call GET "/access/$ycB" $tn; Chk "nvc GET yeu cau tai lieu B -> 403" ($r.code -eq 403) ("code=" + $r.code)
  $r = Call GET "/access/$ycB" $tc; Chk "canbo GET yeu cau tai lieu B -> 200" ($r.code -eq 200) ("code=" + $r.code)
  # danh mục hồ sơ: lập cho phông B bị chặn
  $r = Call POST "/danh-muc-ho-so" $tc @{ projectId = $pid1; phongId = $phB; nam = 2026 }; Chk "canbo lap danh muc phong B -> 200" ($r.code -eq 200) ("code=" + $r.code)
  $r = Call GET "/danh-muc-ho-so?projectId=$pid1" $tn; Chk "nvc list danh muc: 0 (phong B bi an)" ($r.code -eq 200 -and @($r.json.rows).Count -eq 0) ("n=" + @($r.json.rows).Count)
  $r = Call GET "/danh-muc-ho-so?projectId=$pid1&phongId=$phB" $tn; Chk "nvc list danh muc phong B -> 403" ($r.code -eq 403) ("code=" + $r.code)
  $r = Call GET "/danh-muc-ho-so?projectId=$pid1" $tc; Chk "canbo list danh muc: 1" (@($r.json.rows).Count -eq 1) ("n=" + @($r.json.rows).Count)

  Write-Host "===== 5. chuyen doi PL VI dung ten co quan + chuc vu tu cau hinh"
  $r = Call GET "/bao-cao/phong?projectId=$pid1&format=pdf" $qt
  $warm = Join-Path $sb "kho\$pid1-warm"; New-Item -ItemType Directory -Force $warm | Out-Null
  [IO.File]::WriteAllBytes((Join-Path $warm "$dA.pdf"), $r.bytes)
  $r = Call POST "/chuyen-doi" $tc @{ projectId = $pid1; documentId = $dA; mucDich = "Thu" }
  Chk "chuyen doi 201" ($r.code -eq 201) ("code=" + $r.code + " " + (Than $r.json))
  if ($r.code -eq 201) {
    $d = Call GET "/chuyen-doi/$($r.json.id)/download" $tc; $f2 = Join-Path $sb "chuyen-doi.pdf"; [IO.File]::WriteAllBytes($f2, $d.bytes); $chu2 = ChuPdf $f2
    Chk "PDF chuyen doi: co ten co quan moi + CHU TICH + Nguyen Van Ky (nguoi ky mac dinh)" ($chu2 -match "UBND xa Thu Nghiem" -and $chu2 -match "CHU TICH" -and $chu2 -match "Nguyen Van Ky") ("trang=" + (SoTrangPdf $f2))
  }

  Write-Host ("===== KET QUA: " + $(if ($script:sai -eq 0) { "TAT CA DAT" } else { "$($script:sai) buoc SAI" }))
} finally {
  if ($p -and -not $p.HasExited) { Stop-Process -Id $p.Id -Force }
}
exit $script:sai
