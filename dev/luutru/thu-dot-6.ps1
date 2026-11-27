# Kịch bản thử API đợt 6 (27/11/2026, migration 0040 — "đủ như Digifile"): bảng mã TT05, cơ quan nộp lưu + khoá API,
# đăng ký nộp qua X-Api-Key, trùng trong đợt, nộp cùng hệ thống từ hồ sơ có sẵn, so sánh trùng, 10 thông tin tiếp nhận, tệp đính kèm,
# duyệt khai thác hai cấp, phân công, giao nhận bản giấy, chuỗi băm nhật ký, cột dự phòng / gói tài liệu rời, mã hồ sơ có mã cơ quan.
# Chạy TRÊN MÁY NÀY (máy dev), không đụng máy chủ thật. Cần: Postgres TẠM (container postgres:16, cổng 55432), `cargo build -p digitext-api`
# (debug), docker CLI. Script tự áp migration còn thiếu, XOÁ SẠCH users/projects của CSDL tạm rồi dựng dữ liệu nhỏ.
#   powershell -ExecutionPolicy Bypass -File dev\luutru\thu-dot-6.ps1 -MatKhauDb <mk>
param(
  [Parameter(Mandatory = $true)][string]$MatKhauDb,
  [string]$Container = "dt-luutru-thu",
  [int]$CongDb = 55432,
  [int]$CongApi = 18087,
  [string]$ThuMucTam = (Join-Path $env:TEMP "digitext-thu-dot-6")
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
function Call($m, $path, $tok, $body, $headers) {
  $h = @{}; if ($tok) { $h.Authorization = "Bearer $tok" }
  if ($headers) { foreach ($k in $headers.Keys) { $h[$k] = $headers[$k] } }
  try {
    $p = @{ Method = $m; Uri = "$B$path"; Headers = $h; UseBasicParsing = $true; TimeoutSec = 120 }
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
function Than($j) { if ($null -eq $j) { return "" }; if ($j -is [string]) { return $j }; $s = ($j | ConvertTo-Json -Compress -Depth 4); return $s.Substring(0, [Math]::Min(240, $s.Length)) }

Write-Host "===== 0. migrate (0040) + don du lieu cu + superadmin"
& $api migrate 2>&1 | Select-Object -Last 1
Chk "co cl_ma_tt05 / co_quan_nop / submission_files" ((Sql "SELECT count(*) FROM information_schema.tables WHERE table_name IN ('cl_ma_tt05','co_quan_nop','submission_files')") -eq "3") ""
Sql "TRUNCATE projects, users CASCADE" | Out-Null
Get-Content (Join-Path $PSScriptRoot "seed-dung-chung-0012.sql") -Raw -Encoding UTF8 | docker exec -i $Container psql -X -q -U digitext -d digitext | Out-Null
# TRUNCATE CASCADE quét cả cl_ma_tt05 (FK project_id) dù dòng dùng chung NULL → nạp lại seed 0040
Get-Content (Join-Path $PSScriptRoot "seed-ma-tt05-0040.sql") -Raw -Encoding UTF8 | docker exec -i $Container psql -X -q -U digitext -d digitext | Out-Null
Chk "11 ma TT05 dung chung con nguyen" ((Sql "SELECT count(*) FROM cl_ma_tt05 WHERE project_id IS NULL AND deleted_at IS NULL") -eq "11") ""
$mk = Join-Path $sb "mk.txt"; [IO.File]::WriteAllText($mk, "ThuNghiemCucBo2026")
& $api bootstrap-superadmin --username quantri --display-name "Quan tri" --password-file $mk 2>&1 | Select-Object -Last 1

$p = Start-Process -FilePath $api -ArgumentList "serve" -NoNewWindow -PassThru -RedirectStandardOutput (Join-Path $sb "api.out.log") -RedirectStandardError (Join-Path $sb "api.err.log")
try {
  $han = (Get-Date).AddSeconds(40)
  do { Start-Sleep -Milliseconds 600; try { $ok = (Invoke-WebRequest -UseBasicParsing -TimeoutSec 3 "http://127.0.0.1:$CongApi/healthz").StatusCode -eq 200 } catch { $ok = $false } } until ($ok -or (Get-Date) -gt $han)
  if (-not $ok) { Write-Host "API khong len"; Get-Content (Join-Path $sb "api.err.log") -Tail 15; exit 1 }

  Write-Host "===== 1. du lieu: du an, phong, 3 ho so da danh so, 2 tai lieu; co quan luu tru ma H49"
  $pid1 = [guid]::NewGuid().ToString(); $phA = [guid]::NewGuid().ToString(); $pbA = [guid]::NewGuid().ToString()
  Sql "INSERT INTO projects (id, code, name) VALUES ('$pid1', 'D6', 'Co quan thu dot 6')" | Out-Null
  Sql "INSERT INTO project_admins (project_id, admin_id) SELECT '$pid1', id FROM users WHERE username = 'quantri'" | Out-Null
  Sql "INSERT INTO cl_phong (id, project_id, ma_du_an, ten, phong_so, muc_luc_so) VALUES ('$phA', '$pid1', 'PA', 'Phong A', '01', '1')" | Out-Null
  Sql "INSERT INTO cl_phong_ban (id, project_id, phong_id, ma_don_vi, ten) VALUES ('$pbA', '$pid1', '$phA', 'VPA', 'Van phong A')" | Out-Null
  $hs = @{}
  foreach ($i in 1..3) {
    $id = [guid]::NewGuid().ToString(); $hs["0$i"] = $id
    Sql "INSERT INTO cl_ho_so (id, project_id, phong_id, phong_ban_id, so_tam, ho_so_so, tieu_de, ngay_ket_thuc, thbq_id) VALUES ('$id', '$pid1', '$phA', '$pbA', '0$i', '$i', 'Ho so so $i nam 2024', '2024', '00000000-0000-4000-8000-00000c010001')" | Out-Null
  }
  $dA = [guid]::NewGuid().ToString(); $dB = [guid]::NewGuid().ToString(); $vbA = [guid]::NewGuid().ToString()
  Sql "INSERT INTO cl_van_ban (id, project_id, ho_so_id, so_thu_tu, trich_yeu) VALUES ('$vbA', '$pid1', '$($hs['01'])', 1, 'VB A')" | Out-Null
  Sql "INSERT INTO documents (id, project_id, filename, ho_so_ref, size_bytes, blake3, storage_path, status, page_count, uploaded_at) VALUES ('$dA', '$pid1', 'a.pdf', 'HSA', 1000, '$('a' * 64)', 'x/a', 'archived', 2, now()), ('$dB', '$pid1', 'b.pdf', 'HSB', 1000, '$('b' * 64)', 'x/b', 'archived', 2, now())" | Out-Null
  Sql "INSERT INTO cl_van_ban_file (id, project_id, van_ban_id, document_id, name) VALUES ('$([guid]::NewGuid())', '$pid1', '$vbA', '$dA', 'a.pdf')" | Out-Null
  $qt = (Call POST "/auth/login" $null @{ username = "quantri"; password = "ThuNghiemCucBo2026" }).json.accessToken
  Chk "login quantri" ($qt) ""
  $u1 = Call POST "/users" $qt @{ username = "canbo"; password = "MatKhau12345"; displayName = "Can bo LT"; role = "admin"; projectId = $pid1 }
  $u2 = Call POST "/users" $qt @{ username = "nvc"; password = "MatKhau12345"; displayName = "Nhan vien C"; role = "user"; projectId = $pid1 }
  $u3 = Call POST "/users" $qt @{ username = "lanhdao"; password = "MatKhau12345"; displayName = "Lanh dao"; role = "manager" }
  Chk "tao canbo + nvc + lanhdao(manager)" ($u1.code -eq 201 -and $u2.code -eq 201 -and $u3.code -eq 201) ("codes=" + $u1.code + "," + $u2.code + "," + $u3.code)
  $tc = (Call POST "/auth/login" $null @{ username = "canbo"; password = "MatKhau12345" }).json.accessToken
  $tn = (Call POST "/auth/login" $null @{ username = "nvc"; password = "MatKhau12345" }).json.accessToken
  $tl = (Call POST "/auth/login" $null @{ username = "lanhdao"; password = "MatKhau12345" }).json.accessToken
  $r = Call PUT "/projects/$pid1/co-quan" $tc @{ maCoQuan = "H49"; ten = "LTLS thu"; loaiHinh = "luu_tru_lich_su" }; Chk "khai co quan luu tru H49" ($r.code -eq 200) ("code=" + $r.code)
  $maTt05 = Sql "SELECT ma_ho_so_tt05 FROM cl_ho_so_v WHERE id = '$($hs['01'])'"
  Chk "ma_ho_so_tt05 = H49.PA.01.2024.1.1 (B06)" ($maTt05 -eq "H49.PA.01.2024.1.1") ("ma=" + $maTt05)

  Write-Host "===== 2. bang ma TT05 (B03)"
  $r = Call GET "/cl/ma-tt05?projectId=$pid1" $tn; Chk "GET 11 ma chuan, 4 nhom" ($r.code -eq 200 -and @($r.json.rows).Count -eq 11 -and @($r.json.nhom).Count -eq 4) ("n=" + @($r.json.rows).Count)
  $r = Call GET "/cl/ma-tt05?projectId=$pid1&nhom=che_do_su_dung" $tn; Chk "loc nhom che_do_su_dung = 3" (@($r.json.rows).Count -eq 3) ""
  $r = Call GET "/cl/ma-tt05?projectId=$pid1&nhom=la" $tn; Chk "nhom la -> 400" ($r.code -eq 400) ("code=" + $r.code)
  $r = Call POST "/cl/ma-tt05" $tc @{ projectId = $pid1; nhom = "che_do_su_dung"; ma = "04"; ten = "Noi bo" }; $maRieng = $r.json
  Chk "canbo them ma rieng 04" ($r.code -eq 200 -and $maRieng.projectId -eq $pid1) ("code=" + $r.code)
  $r = Call POST "/cl/ma-tt05" $tc @{ projectId = $pid1; nhom = "che_do_su_dung"; ma = "04"; ten = "Trung" }; Chk "trung ma -> 409" ($r.code -eq 409) ("code=" + $r.code)
  $r = Call POST "/cl/ma-tt05" $tc @{ projectId = $pid1; dungChung = $true; nhom = "che_do_su_dung"; ma = "05"; ten = "x" }; Chk "canbo them ma dung chung -> 403" ($r.code -eq 403) ("code=" + $r.code)
  $r = Call PUT "/cl/ma-tt05/$($maRieng.id)" $tc @{ nhom = "che_do_su_dung"; ma = "04"; ten = "Noi bo co quan"; soTt = 4 }; Chk "sua ma rieng" ($r.code -eq 200 -and $r.json.ten -eq "Noi bo co quan") ("code=" + $r.code)
  $r = Call PUT "/cl/ma-tt05/00000000-0000-4000-8000-00000c050101" $tc @{ nhom = "che_do_su_dung"; ma = "01"; ten = "x" }; Chk "canbo sua ma chuan -> 403" ($r.code -eq 403) ("code=" + $r.code)
  $r = Call GET "/cl/ma-tt05?projectId=$pid1&nhom=che_do_su_dung" $tn; Chk "nvc thay 4 ma (3 chuan + 1 rieng)" (@($r.json.rows).Count -eq 4) ("n=" + @($r.json.rows).Count)
  $r = Call DELETE "/cl/ma-tt05/$($maRieng.id)" $tc; Chk "xoa ma rieng" ($r.code -eq 204) ("code=" + $r.code)

  Write-Host "===== 3. co quan nop luu + khoa API (D11, D12)"
  $r = Call POST "/co-quan-nop" $tc @{ projectId = $pid1; ma = "snv"; ten = "So Noi vu"; email = "snv@thu.vn" }; $cq = $r.json
  Chk "tao co quan nop SNV (ma viet hoa)" ($r.code -eq 201 -and $cq.ma -eq "SNV" -and $cq.coApiKey -eq $false) ("code=" + $r.code + " " + (Than $cq))
  $r = Call POST "/co-quan-nop" $tc @{ projectId = $pid1; ma = "SNV"; ten = "Trung" }; Chk "trung ma -> 409" ($r.code -eq 409) ("code=" + $r.code)
  $r = Call POST "/co-quan-nop" $tc @{ projectId = $pid1; ma = "STC"; ten = "So Tai chinh" }; $cq2 = $r.json; Chk "tao co quan STC" ($r.code -eq 201) ""
  $r = Call POST "/co-quan-nop" $tn @{ projectId = $pid1; ma = "X"; ten = "x" }; Chk "nvc (khong submission.review) -> 403" ($r.code -eq 403) ("code=" + $r.code)
  $r = Call POST "/co-quan-nop/$($cq.id)/api-key" $tc; Chk "cap khoa khi chua co tai khoan -> 400" ($r.code -eq 400) ("code=" + $r.code)
  $ucq = Call POST "/users" $qt @{ username = "snv_nop"; password = "MatKhau12345"; displayName = "Can bo SNV"; role = "user"; projectId = $pid1; coQuanNopId = $cq.id }
  Chk "tao tai khoan gan co quan SNV" ($ucq.code -eq 201 -and $ucq.json.coQuanNopId -eq $cq.id) ("code=" + $ucq.code + " " + (Than $ucq.json))
  $ucq2 = Call POST "/users" $qt @{ username = "stc_nop"; password = "MatKhau12345"; displayName = "Can bo STC"; role = "user"; projectId = $pid1; coQuanNopId = $cq2.id }
  $r = Call GET "/co-quan-nop/$($cq.id)/tai-khoan" $tc; Chk "tai khoan cua SNV = 1" (@($r.json).Count -eq 1 -and $r.json[0].username -eq "snv_nop") ""
  $r = Call POST "/co-quan-nop/$($cq.id)/api-key" $tc; $key = $r.json.apiKey
  Chk "cap khoa API (dtk_...)" ($r.code -eq 200 -and $key -like "dtk_*") ("code=" + $r.code)
  $r = Call GET "/co-quan-nop?projectId=$pid1" $tc; Chk "danh sach 2 co quan, SNV coApiKey=true soTaiKhoan=1" (@($r.json).Count -eq 2 -and (@($r.json | Where-Object { $_.ma -eq "SNV" })[0]).coApiKey -eq $true -and (@($r.json | Where-Object { $_.ma -eq "SNV" })[0]).soTaiKhoan -eq 1) (Than $r.json)
  $r = Call GET "/submissions" $null $null @{ "X-Api-Key" = $key }; Chk "GET /submissions bang X-Api-Key -> 200 rong" ($r.code -eq 200 -and @($r.json).Count -eq 0) ("code=" + $r.code)
  $r = Call GET "/submissions" $null $null @{ "X-Api-Key" = "dtk_sai" }; Chk "khoa sai -> 401" ($r.code -eq 401) ("code=" + $r.code)
  $r = Call POST "/submissions" $null @{ projectId = $pid1; fondCode = "PA"; summary = "Nop qua API"; items = @(@{ maHoSo = "HS-100"; tieuDe = "Mot" }, @{ maHoSo = "HS-101"; tieuDe = "Hai" }, @{ maHoSo = "hs-100"; tieuDe = "Trung" }) } @{ "X-Api-Key" = $key }
  $sub = $r.json
  Chk "dang ky qua X-Api-Key: 201, org tu danh muc, coQuanNopId" ($r.code -eq 201 -and $sub.orgCode -eq "SNV" -and $sub.orgName -eq "So Noi vu" -and $sub.coQuanNopId -eq $cq.id) ("code=" + $r.code + " " + (Than $sub))
  $it3 = @($sub.items | Where-Object { $_.stt -eq 3 })[0]
  Chk "dong 3 trung ma trong dot -> trungTrongDot, khong_dat" ($it3.trungTrongDot -eq $true -and $it3.reviewResult -eq "khong_dat") (Than $it3)
  $r = Call GET "/submissions/$($sub.id)" $null $null @{ "X-Api-Key" = $key }; Chk "co quan SNV xem dot cua minh" ($r.code -eq 200) ("code=" + $r.code)
  $t2 = (Call POST "/auth/login" $null @{ username = "stc_nop"; password = "MatKhau12345" }).json.accessToken
  $r = Call GET "/submissions/$($sub.id)" $t2; Chk "tai khoan STC xem dot cua SNV -> 403" ($r.code -eq 403) ("code=" + $r.code)
  $r = Call GET "/submissions" $t2; Chk "STC danh sach rong" (@($r.json).Count -eq 0) ("n=" + @($r.json).Count)
  $r = Call GET "/submissions" $tc; Chk "canbo thay 1 dot" (@($r.json).Count -eq 1) ""
  $r = Call POST "/submissions" $tc @{ projectId = $pid1; coQuanNopId = $cq2.id; fondCode = "PB"; items = @(@{ maHoSo = "X1"; tieuDe = "x" }) }; Chk "canbo dang ky ho STC bang coQuanNopId" ($r.code -eq 201 -and $r.json.orgCode -eq "STC") ("code=" + $r.code)
  $r = Call DELETE "/co-quan-nop/$($cq2.id)" $tc; Chk "xoa co quan da co dot nop -> 409" ($r.code -eq 409) ("code=" + $r.code)
  $r = Call DELETE "/co-quan-nop/$($cq.id)/api-key" $tc; Chk "thu hoi khoa" ($r.code -eq 204) ""
  $r = Call GET "/submissions" $null $null @{ "X-Api-Key" = $key }; Chk "khoa da thu hoi -> 401" ($r.code -eq 401) ("code=" + $r.code)

  Write-Host "===== 4. nop cung he thong (D04), so sanh (D07), 10 thong tin (D06)"
  $r = Call POST "/submissions/$($sub.id)/items/tu-ho-so" $tc @{ hoSoIds = @($hs["01"], $hs["02"]) }
  Chk "them 2 ho so co san: hoSoId, ma TT05" ($r.code -eq 200 -and @($r.json).Count -eq 2 -and $r.json[0].hoSoId -eq $hs["01"] -and $r.json[0].maHoSo -eq "H49.PA.01.2024.1.1") ("code=" + $r.code + " " + (Than $r.json))
  $r = Call POST "/submissions/$($sub.id)/items/tu-ho-so" $tc @{ hoSoIds = @($hs["01"]) }; Chk "them lai ho so 01 -> trung trong dot" ($r.code -eq 200 -and $r.json[0].trungTrongDot -eq $true) (Than $r.json)
  $r = Call POST "/submissions/$($sub.id)/items/tu-ho-so" $tc @{ hoSoIds = @([guid]::NewGuid().ToString()) }; Chk "ho so la -> 400" ($r.code -eq 400) ("code=" + $r.code)
  $d = (Call GET "/submissions/$($sub.id)" $tc).json
  $itA = @($d.items | Where-Object { $_.hoSoId -eq $hs["01"] -and -not $_.trungTrongDot })[0]
  $r = Call GET "/submissions/$($sub.id)/items/$($itA.id)/so-sanh" $tc
  Chk "so-sanh: trongKho 1 dong khop ma, trongDot 1 dong" ($r.code -eq 200 -and @($r.json.trongKho).Count -eq 1 -and $r.json.trongKho[0].khop -eq "ma" -and @($r.json.trongDot).Count -eq 1) ("code=" + $r.code + " " + (Than $r.json))
  $r = Call GET "/submissions/$($sub.id)/tong-quan" $tc
  Chk "tong-quan: hoSoDangKy=6 trungDot=2 daVaoHeThong=3 tongGoi=0" ($r.json.hoSoDangKy -eq 6 -and $r.json.hoSoTrungDot -eq 2 -and $r.json.hoSoDaVaoHeThong -eq 3 -and $r.json.tongGoi -eq 0) (Than $r.json)

  Write-Host "===== 5. tep dinh kem (D02)"
  $b64 = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes("Van ban de nghi nop luu - thu"))
  $r = Call POST "/submissions/$($sub.id)/files" $tc @{ loai = "van_ban_de_nghi"; filename = "de-nghi.txt"; contentBase64 = $b64 }; $f1 = $r.json
  Chk "dinh kem van ban de nghi" ($r.code -eq 201 -and $f1.sizeBytes -gt 0 -and $f1.blake3) ("code=" + $r.code + " " + (Than $f1))
  $r = Call POST "/submissions/$($sub.id)/files" $tc @{ loai = "sai"; filename = "x.txt"; contentBase64 = $b64 }; Chk "loai sai -> 400" ($r.code -eq 400) ("code=" + $r.code)
  $r = Call POST "/submissions/$($sub.id)/files" $tc @{ loai = "khac"; filename = "../x.txt"; contentBase64 = $b64 }; Chk "ten tep co / -> 400" ($r.code -eq 400) ("code=" + $r.code)
  $r = Call GET "/submissions/$($sub.id)/files" $tc; Chk "danh sach 1 tep" (@($r.json).Count -eq 1) ""
  $r = Call GET "/submissions/$($sub.id)/files/$($f1.id)" $tc; Chk "tai tep dung noi dung" ($r.code -eq 200 -and [Text.Encoding]::UTF8.GetString($r.bytes) -eq "Van ban de nghi nop luu - thu") ("code=" + $r.code)
  $r = Call GET "/submissions/$($sub.id)/files/$($f1.id)" $t2; Chk "STC tai tep cua SNV -> 403" ($r.code -eq 403) ("code=" + $r.code)
  $r = Call DELETE "/submissions/$($sub.id)/files/$($f1.id)" $tc; Chk "xoa tep" ($r.code -eq 204) ("code=" + $r.code)

  Write-Host "===== 6. khai thac: duyet hai cap (F04), phan cong (F12), giao nhan ban giay (F10)"
  $r = Call POST "/access" $tc @{ projectId = $pid1; kind = "ban_sao"; mucDich = "Thu"; documentIds = @($dA) }; $yc = $r.json
  Chk "canbo lap yeu cau ban sao" ($r.code -eq 201) ("code=" + $r.code)
  $it = $yc.items[0]
  $r = Call POST "/access/$($yc.id)/items/$($it.id)/review" $tc @{ verdict = "duyet"; dieuKien = $true }; Chk "cham tai lieu: duyet co dieu kien" ($r.code -in 200, 204) ("code=" + $r.code)
  $r = Call POST "/access/$($yc.id)/approve" $tc @{ note = "cap 1" }
  Chk "canbo (admin) duyet -> cho_cap_2" ($r.code -eq 200 -and $r.json.status -eq "cho_cap_2") ("code=" + $r.code + " tt=" + $r.json.status)
  $r = Call POST "/access/$($yc.id)/approve" $tc @{ note = "lai" }; Chk "canbo duyet lai van cho_cap_2" ($r.code -eq 200 -and $r.json.status -eq "cho_cap_2") ("code=" + $r.code + " " + (Than $r.json))
  $r = Call POST "/access/$($yc.id)/phan-cong" $tc @{ userId = $u2.json.id }; Chk "phan cong nvc" ($r.code -eq 200 -and $r.json.nguoiXuLy -eq $u2.json.id) ("code=" + $r.code)
  $r = Call POST "/access/$($yc.id)/giao-nhan" $tc @{ hinhThuc = "tru_so" }; Chk "giao nhan khi chua duyet -> 409" ($r.code -eq 409) ("code=" + $r.code)
  $r = Call POST "/access/$($yc.id)/approve" $tl @{ note = "cap 2 dong y" }
  Chk "lanh dao (manager) duyet cap 2 -> approved, cap2Boi" ($r.code -eq 200 -and $r.json.status -eq "approved" -and $r.json.cap2Boi -eq $u3.json.id -and $r.json.cap2At) ("code=" + $r.code + " " + (Than $r.json))
  $r = Call POST "/access/$($yc.id)/giao-nhan" $tc @{ hinhThuc = "la" }; Chk "hinh thuc la -> 400" ($r.code -eq 400) ("code=" + $r.code)
  $r = Call POST "/access/$($yc.id)/giao-nhan" $tc @{ hinhThuc = "buu_dien"; diaChi = "So 1 Hiep Hoa"; nguoiNhan = "Nguyen A"; ngayGiao = "2026-11-28" }
  Chk "ghi giao nhan ban giay" ($r.code -eq 200 -and $r.json.giaoNhan.hinhThuc -eq "buu_dien" -and $r.json.giaoNhan.nguoiNhan -eq "Nguyen A") ("code=" + $r.code + " " + (Than $r.json.giaoNhan))
  # yeu cau khong co dieu kien: duyet thang
  $r = Call POST "/access" $tc @{ projectId = $pid1; kind = "doc"; mucDich = "Thu 2"; documentIds = @($dA) }; $yc2 = $r.json
  Call POST "/access/$($yc2.id)/items/$($yc2.items[0].id)/review" $tc @{ verdict = "duyet" } | Out-Null
  $r = Call POST "/access/$($yc2.id)/approve" $tc @{ note = "ok" }; Chk "khong co dieu kien -> canbo duyet thang approved" ($r.json.status -eq "approved" -and -not $r.json.cap2Boi) ("code=" + $r.code + " " + (Than $r.json))
  $r = Call POST "/info" $tc @{ projectId = $pid1; hinhThuc = "danh_muc"; chuDe = "thu"; mucDich = "thu" }; $inf = $r.json
  $r = Call POST "/info/$($inf.id)/phan-cong" $tc @{ userId = $u2.json.id }; Chk "phan cong cung cap thong tin" ($r.code -eq 200 -and $r.json.nguoiXuLy -eq $u2.json.id) ("code=" + $r.code)
  $r = Call GET "/bao-cao/khai_thac_yeu_cau?projectId=$pid1" $tc; $row = @($r.json.rows)[0]
  Chk "bao cao khai thac co cot trong_han / pct_dung_han" ($r.code -eq 200 -and $null -ne $row.trong_han -and $null -ne $row.pct_dung_han) (Than $row)

  Write-Host "===== 7. cot du phong, goi tai lieu roi (B04, B05, B09) qua /cl/rows"
  $r = Call PUT "/cl/rows/ho_so/$($hs['02'])" $tc @{ projectId = $pid1; new = @{ du_phong_trang_thai = "01"; du_phong = $true; la_goi_tai_lieu = $true; ma_goi = "H49.2026.01.001" } }
  Chk "PUT ho so: du_phong + goi tai lieu" ($r.code -eq 200) ("code=" + $r.code + " " + (Than $r.json))
  $v = Sql "SELECT du_phong::text || '|' || du_phong_trang_thai || '|' || la_goi_tai_lieu::text || '|' || ma_goi FROM cl_ho_so_v WHERE id = '$($hs['02'])'"
  Chk "view co gia tri" ($v -eq "true|01|true|H49.2026.01.001") ("v=" + $v)
  $r = Call PUT "/cl/rows/van_ban/$vbA" $tc @{ projectId = $pid1; new = @{ du_phong_trang_thai = "02"; quy_trinh_xu_ly = "Tiep nhan -> Xu ly -> Luu" } }
  Chk "PUT van ban: quy_trinh_xu_ly" ($r.code -eq 200) ("code=" + $r.code)
  Chk "van ban luu quy trinh" ((Sql "SELECT quy_trinh_xu_ly FROM cl_van_ban WHERE id = '$vbA'") -eq "Tiep nhan -> Xu ly -> Luu") ""
  $gh = Sql "SELECT ma_ho_so_giay_tt05 FROM cl_ho_so_v WHERE id = '$($hs['01'])'"; Chk "ma_ho_so_giay_tt05 NULL khi chua co hop" (-not $gh) ("v=" + $gh)

  Write-Host "===== 8. chuoi bam nhat ky (H08)"
  $r = Call GET "/ops/audit/kiem-chuoi" $tc; Chk "chuoi nguyen ven, daKiem>=3" ($r.code -eq 200 -and $null -eq $r.json.dongLoi -and $r.json.daKiem -ge 3) (Than $r.json)
  $idCuoi = Sql "SELECT max(id) FROM audit_log WHERE hash IS NOT NULL"
  Sql "UPDATE audit_log SET action = action || '-sua' WHERE id = $idCuoi" | Out-Null
  $r = Call GET "/ops/audit/kiem-chuoi" $tc; Chk "sua 1 dong -> bao dung dong $idCuoi" ([string]$r.json.dongLoi -eq $idCuoi) (Than $r.json)
  $r = Call GET "/ops/audit/kiem-chuoi" $tn; Chk "nvc (khong bk.view) -> 403" ($r.code -eq 403) ("code=" + $r.code)

  Write-Host ("===== KET QUA: " + $(if ($script:sai -eq 0) { "TAT CA DAT" } else { "$($script:sai) buoc SAI" }))
} finally {
  if ($p -and -not $p.HasExited) { Stop-Process -Id $p.Id -Force }
}
exit $script:sai
