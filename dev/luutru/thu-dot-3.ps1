# Kịch bản thử API đợt 3 (27/11/2026): báo cáo TT05 (0036), mượn – trả có duyệt (0034), kho / giá sức chứa + gán hộp (0035) —
# chạy TRÊN MÁY NÀY (máy dev), không đụng máy chủ thật. Cần: Postgres TẠM (container postgres:16 đã áp 0001→0036),
# `cargo build -p digitext-api` (debug), docker CLI. Script XOÁ SẠCH users/projects của CSDL tạm rồi dựng dữ liệu nhỏ.
#   powershell -ExecutionPolicy Bypass -File dev\luutru\thu-dot-3.ps1 -MatKhauDb <mk>
param(
  [Parameter(Mandatory = $true)][string]$MatKhauDb,
  [string]$Container = "dt-luutru-thu",
  [int]$CongDb = 55432,
  [int]$CongApi = 18084,
  [string]$ThuMucTam = (Join-Path $env:TEMP "digitext-thu-dot-3")
)
$ErrorActionPreference = "Continue"
$repo = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path; $sb = $ThuMucTam; New-Item -ItemType Directory -Force $sb | Out-Null
$env:DATABASE_URL = "postgres://digitext:$MatKhauDb@127.0.0.1:$CongDb/digitext"
$env:DIGITEXT_EDITION = "luu_tru"; $env:DIGITEXT_BIND = "127.0.0.1:$CongApi"; $env:DIGITEXT_PUBLIC_URL = "http://127.0.0.1:$CongApi"
$env:DIGITEXT_DATA_DIR = Join-Path $sb "data"; $env:DIGITEXT_UPDATES_DIR = Join-Path $sb "updates"
$env:DIGITEXT_JWT_KEY_PATH = Join-Path $sb "data\jwt-ed25519.pem"; $env:DIGITEXT_PDF_STORE_DIR = Join-Path $sb "kho"
$env:DIGITEXT_MIGRATIONS_DIR = Join-Path $repo "migrations"; $env:DIGITEXT_CERT_FONT = "C:\Windows\Fonts\times.ttf"
$env:DIGITEXT_MT_NHAC_GIAY = "8"
New-Item -ItemType Directory -Force (Join-Path $sb "data"), (Join-Path $sb "kho") | Out-Null
$api = Join-Path $repo "target\debug\digitext-api.exe"
$B = "http://127.0.0.1:$CongApi/api/v1"
function Sql($q) { docker exec $Container psql -X -t -A -U digitext -d digitext -c $q }
function Call($m, $path, $tok, $body) {
  $h = @{}; if ($tok) { $h.Authorization = "Bearer $tok" }
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
function Than($j) { if ($null -eq $j) { return "" }; if ($j -is [string]) { return $j }; $s = ($j | ConvertTo-Json -Compress -Depth 4); return $s.Substring(0, [Math]::Min(220, $s.Length)) }

Write-Host "===== 0. don du lieu cu + superadmin"
Sql "TRUNCATE projects, users CASCADE" | Out-Null
$mk = Join-Path $sb "mk.txt"; [IO.File]::WriteAllText($mk, "ThuNghiemCucBo2026")
& $api bootstrap-superadmin --username quantri --display-name "Quan tri" --password-file $mk 2>&1 | Select-Object -Last 1

$p = Start-Process -FilePath $api -ArgumentList "serve" -NoNewWindow -PassThru -RedirectStandardOutput (Join-Path $sb "api.out.log") -RedirectStandardError (Join-Path $sb "api.err.log")
try {
  $han = (Get-Date).AddSeconds(40)
  do { Start-Sleep -Milliseconds 600; try { $ok = (Invoke-WebRequest -UseBasicParsing -TimeoutSec 3 "http://127.0.0.1:$CongApi/healthz").StatusCode -eq 200 } catch { $ok = $false } } until ($ok -or (Get-Date) -gt $han)
  if (-not $ok) { Write-Host "API khong len"; Get-Content (Join-Path $sb "api.err.log") -Tail 15; exit 1 }

  Write-Host "===== 1. du lieu: du an, phong, 2 phong ban, kho / gia, 5 ho so (05 chua co phong ban = nhap goi), 2 hop"
  $pid1 = [guid]::NewGuid().ToString(); $phong = [guid]::NewGuid().ToString(); $pbA = [guid]::NewGuid().ToString(); $pbB = [guid]::NewGuid().ToString()
  $kho = [guid]::NewGuid().ToString(); $gia = [guid]::NewGuid().ToString()
  Sql "INSERT INTO projects (id, code, name) VALUES ('$pid1', 'D3', 'Co quan thu dot 3')" | Out-Null
  Sql "INSERT INTO project_admins (project_id, admin_id) SELECT '$pid1', id FROM users WHERE username = 'quantri'" | Out-Null
  Sql "INSERT INTO cl_phong (id, project_id, ma_du_an, ten, phong_so) VALUES ('$phong', '$pid1', 'UBND', 'Phong UBND huyen', '01')" | Out-Null
  Sql "INSERT INTO cl_phong_ban (id, project_id, phong_id, ma_don_vi, ten) VALUES ('$pbA', '$pid1', '$phong', 'VP', 'Van phong'), ('$pbB', '$pid1', '$phong', 'NV', 'Phong Noi vu')" | Out-Null
  Sql "INSERT INTO cl_kho (id, project_id, ten, suc_chua_hop) VALUES ('$kho', '$pid1', 'Kho A', 100)" | Out-Null
  Sql "INSERT INTO cl_gia (id, project_id, kho_id, phong_ban_id, ten, tu_hop, den_hop, canh_bao_pct) VALUES ('$gia', '$pid1', '$kho', '$pbA', 'Gia 1', 1, 3, 60)" | Out-Null
  $hs = @{}
  foreach ($x in @(@("01", $pbA), @("02", $pbA), @("03", $pbA), @("04", $pbA), @("05", $null))) {
    $id = [guid]::NewGuid().ToString(); $hs[$x[0]] = $id
    $pb = $(if ($x[1]) { "'$($x[1])'" } else { "NULL" })
    Sql "INSERT INTO cl_ho_so (id, project_id, phong_id, phong_ban_id, so_tam, tieu_de, ngay_ket_thuc, che_do_su_dung, created_by) VALUES ('$id', '$pid1', '$phong', $pb, '$($x[0])', 'Ho so so $($x[0])', '2025', 'Cong khai', (SELECT id FROM users WHERE username='quantri'))" | Out-Null
  }
  $hop1 = [guid]::NewGuid().ToString(); $hop2 = [guid]::NewGuid().ToString()
  Sql "INSERT INTO cl_hop (id, project_id, phong_ban_id, hop_so) VALUES ('$hop1', '$pid1', '$pbA', '1'), ('$hop2', '$pid1', '$pbA', '2')" | Out-Null
  Sql "UPDATE cl_ho_so SET hop_id = '$hop1' WHERE id = '$($hs['01'])'; UPDATE cl_ho_so SET hop_id = '$hop2' WHERE id = '$($hs['02'])'" | Out-Null
  $qt = (Call POST "/auth/login" $null @{ username = "quantri"; password = "ThuNghiemCucBo2026" }).json.accessToken
  Chk "login quantri" ($qt) ""
  $u1 = Call POST "/users" $qt @{ username = "nvb"; password = "MatKhau12345"; displayName = "Nhan vien B"; role = "user"; projectId = $pid1 }
  Chk "tao nvb (user: loan.manage, khong loan.approve)" ($u1.code -eq 201) ("code=" + $u1.code)
  $u2 = Call POST "/users" $qt @{ username = "canbo"; password = "MatKhau12345"; displayName = "Can bo LT"; role = "admin"; projectId = $pid1 }
  Chk "tao canbo (admin: loan.approve)" ($u2.code -eq 201) ("code=" + $u2.code)
  $tb = (Call POST "/auth/login" $null @{ username = "nvb"; password = "MatKhau12345" }).json.accessToken
  $tc = (Call POST "/auth/login" $null @{ username = "canbo"; password = "MatKhau12345" }).json.accessToken
  Chk "login nvb + canbo" ($tb -and $tc) ""

  Write-Host "===== 2. muon tra co duyet (0034)"
  $nam = (Get-Date).Year
  $r = Call POST "/cl/muon-tra" $tb @{ projectId = $pid1; hoSoIds = @($hs["01"], $hs["02"]); nguoiMuon = "Tran Van C"; donVi = "Phong TC"; noiDung = "Tra cuu" }
  $ph1 = $r.json; Chk "nvb lap phieu 2 ho so -> cho_duyet, so phieu MT-$nam-0001" ($r.code -eq 201 -and $ph1.soPhieu -eq "MT-$nam-0001" -and @($ph1.rows).Count -eq 2 -and $ph1.rows[0].trang_thai -eq "cho_duyet") ("code=" + $r.code + " " + (Than $ph1))
  $d1 = @($ph1.rows | Where-Object { $_.ho_so_id -eq $hs["01"] })[0]; $d2 = @($ph1.rows | Where-Object { $_.ho_so_id -eq $hs["02"] })[0]
  $r = Call POST "/cl/muon-tra" $tb @{ projectId = $pid1; hoSoIds = @($hs["03"]); nguoiMuon = "X"; giaoNgay = $true }; Chk "nvb giao ngay -> 403" ($r.code -eq 403) ("code=" + $r.code)
  $r = Call POST "/cl/muon-tra/$($d1.id)/duyet" $tb; Chk "nvb duyet -> 403" ($r.code -eq 403) ("code=" + $r.code)
  $r = Call PUT "/cl/rows/muon_tra/$($d1.id)" $tb @{ projectId = $pid1; new = @{ trang_thai = "dang_muon" } }
  Chk "PUT /cl/rows doi cho_duyet->dang_muon bi chan (loan_state)" ($r.code -ge 400 -and ([string]$r.raw) -match "loan_state") ("code=" + $r.code + " " + [string]$r.raw)
  $r = Call POST "/cl/rows/muon_tra" $tb @{ projectId = $pid1; new = @{ ho_so_id = $hs["04"]; nguoi_muon = "Y"; trang_thai = "dang_muon" } }
  Chk "POST /cl/rows tao thang dang_muon khong co approve -> loan_state" ($r.code -ge 400 -and ([string]$r.raw) -match "loan_state") ("code=" + $r.code + " " + [string]$r.raw)
  $r = Call GET "/notifications" $qt; Chk "quan tri nhan loan_request" (@($r.json.items | Where-Object { $_.kind -eq "loan_request" }).Count -ge 1) ("kinds=" + (($r.json.items | ForEach-Object { $_.kind } | Select-Object -Unique) -join ","))
  $r = Call POST "/cl/muon-tra/$($d1.id)/duyet" $tc; Chk "canbo duyet dong 01 -> da_duyet" ($r.code -eq 200 -and $r.json.trang_thai -eq "da_duyet" -and $r.json.nguoi_duyet_ten -eq "Can bo LT") ("code=" + $r.code + " tt=" + $r.json.trang_thai)
  $r = Call POST "/cl/muon-tra/$($d2.id)/tu-choi" $tc; Chk "tu choi khong ly do -> 400" ($r.code -eq 400) ("code=" + $r.code)
  $r = Call POST "/cl/muon-tra/$($d2.id)/tu-choi" $tc @{ lyDo = "Ho so dang chinh ly" }; Chk "tu choi dong 02" ($r.code -eq 200 -and $r.json.trang_thai -eq "tu_choi" -and $r.json.ly_do_tu_choi -eq "Ho so dang chinh ly") ("code=" + $r.code)
  $r = Call GET "/notifications" $tb; Chk "nvb nhan loan_decided x2" (@($r.json.items | Where-Object { $_.kind -eq "loan_decided" }).Count -eq 2) ("n=" + @($r.json.items | Where-Object { $_.kind -eq "loan_decided" }).Count)
  $r = Call POST "/cl/muon-tra/$($d1.id)/tra" $tc; Chk "tra khi chua giao -> 400" ($r.code -eq 400) ("code=" + $r.code)
  $r = Call POST "/cl/muon-tra/$($d1.id)/giao" $tc @{ ngay = (Get-Date).ToString("yyyy-MM-dd") }; Chk "giao dong 01 -> dang_muon" ($r.code -eq 200 -and $r.json.trang_thai -eq "dang_muon" -and $r.json.giao_boi_ten -eq "Can bo LT") ("code=" + $r.code + " tt=" + $r.json.trang_thai)
  $r = Call GET "/cl/ho-so/$($hs['01'])?projectId=$pid1" $tc; Chk "view cl_ho_so_v dang_muon=true" ($r.json.dang_muon -eq $true) ("dang_muon=" + $r.json.dang_muon)
  $r = Call POST "/cl/muon-tra" $tc @{ projectId = $pid1; hoSoIds = @($hs["01"]); nguoiMuon = "Z"; giaoNgay = $true }; Chk "giao ngay ho so dang muon -> 400" ($r.code -eq 400) ("code=" + $r.code + " " + (Than $r.json))
  $hanCu = [datetime]$d1.ngay_du_kien_tra
  $r = Call POST "/cl/muon-tra/$($d1.id)/gia-han" $tc @{ hanMoi = $hanCu.AddDays(-1).ToString("yyyy-MM-dd") }; Chk "gia han lui -> 400" ($r.code -eq 400) ("code=" + $r.code)
  $r = Call POST "/cl/muon-tra/$($d1.id)/gia-han" $tc @{ hanMoi = $hanCu.AddDays(7).ToString("yyyy-MM-dd"); lyDo = "Can them" }
  Chk "gia han lan 1: so_lan=1, han_tra_goc = han cu" ($r.code -eq 200 -and $r.json.so_lan_gia_han -eq 1 -and ([datetime]$r.json.han_tra_goc).Date -eq $hanCu.Date -and @($r.json.gia_han_log).Count -eq 1) ("code=" + $r.code + " goc=" + $r.json.han_tra_goc)
  $r = Call POST "/cl/muon-tra/$($d1.id)/gia-han" $tc @{ hanMoi = $hanCu.AddDays(14).ToString("yyyy-MM-dd") }; Chk "gia han lan 2" ($r.code -eq 200 -and $r.json.so_lan_gia_han -eq 2) ("code=" + $r.code)
  $r = Call POST "/cl/muon-tra/$($d1.id)/gia-han" $tc @{ hanMoi = $hanCu.AddDays(21).ToString("yyyy-MM-dd") }; Chk "gia han lan 3 -> 400 (toi da 2)" ($r.code -eq 400) ("code=" + $r.code)
  # Qua han: lui han ve hom qua, cho job nhac (DIGITEXT_MT_NHAC_GIAY=8)
  Sql "UPDATE cl_muon_tra SET ngay_du_kien_tra = CURRENT_DATE - 3, nhac_lan_cuoi = NULL WHERE id = '$($d1.id)'" | Out-Null
  $r = Call GET "/cl/muon-tra?projectId=$pid1&trangThai=qua_han" $tb; Chk "danh sach qua_han = 1" ($r.json.total -eq 1 -and $r.json.rows[0].qua_han -eq $true) ("total=" + $r.json.total)
  $r = Call GET "/cl/muon-tra/tong-quan?projectId=$pid1" $tb; Chk "tong quan: dangMuon=1 quaHan=1 choDuyet=0" ($r.json.dangMuon -eq 1 -and $r.json.quaHan -eq 1 -and $r.json.choDuyet -eq 0) (Than $r.json)
  Start-Sleep -Seconds 10
  $r = Call GET "/notifications" $tb; $nOver = @($r.json.items | Where-Object { $_.kind -eq "loan_overdue" }).Count
  Chk "job nhac: nvb (nguoi lap) nhan loan_overdue" ($nOver -ge 1) ("n=" + $nOver)
  $r = Call GET "/notifications" $qt; Chk "quan tri du an cung nhan loan_overdue" (@($r.json.items | Where-Object { $_.kind -eq "loan_overdue" }).Count -ge 1) ""
  Chk "nhac_lan_cuoi = hom nay" ((Sql "SELECT (nhac_lan_cuoi = CURRENT_DATE)::text FROM cl_muon_tra WHERE id = '$($d1.id)'") -eq "true") ""
  $r = Call POST "/cl/muon-tra/$($d1.id)/tra" $tb @{ ngay = (Get-Date).ToString("yyyy-MM-dd") }; Chk "nvb (loan.manage) ghi tra -> da_tra, tra_truoc_han=false" ($r.code -eq 200 -and $r.json.trang_thai -eq "da_tra" -and $r.json.tra_truoc_han -eq $false) ("code=" + $r.code)
  $r = Call GET "/cl/muon-tra/$($d1.id)/phieu.pdf" $tc; Chk "phieu muon PDF" ($r.code -eq 200 -and $r.bytes.Length -gt 1000 -and [Text.Encoding]::ASCII.GetString($r.bytes[0..4]) -eq "%PDF-") ("code=" + $r.code + " bytes=" + $r.bytes.Length)
  $r = Call POST "/cl/muon-tra" $tb @{ projectId = $pid1; hoSoIds = @($hs["03"], $hs["04"]); nguoiMuon = "Le Thi D"; ngayDuKienTra = (Get-Date).AddDays(10).ToString("yyyy-MM-dd") }
  $ph2 = $r.json; Chk "phieu 2 (MT-$nam-0002, 2 dong cho duyet)" ($r.code -eq 201 -and $ph2.soPhieu -eq "MT-$nam-0002") ("so=" + $ph2.soPhieu)
  $r = Call POST "/cl/muon-tra/nhom/$($ph2.nhomId)/duyet" $tc; Chk "duyet ca phieu: daChuyen=2" ($r.code -eq 200 -and $r.json.daChuyen -eq 2 -and @($r.json.rows | Where-Object { $_.trang_thai -eq "da_duyet" }).Count -eq 2) ("code=" + $r.code + " da=" + $r.json.daChuyen)
  $r = Call POST "/cl/muon-tra/nhom/$($ph2.nhomId)/duyet" $tc; Chk "duyet ca phieu lan 2 -> 400 (khong con dong cho)" ($r.code -eq 400) ("code=" + $r.code)
  $r = Call POST "/cl/muon-tra/$(@($ph2.rows)[0].id)/huy" $tb @{ lyDo = "Nguoi muon rut" }; Chk "huy 1 dong da duyet" ($r.code -eq 200 -and $r.json.trang_thai -eq "huy") ("code=" + $r.code)
  $r = Call GET "/cl/muon-tra?projectId=$pid1&trangThai=mo" $tc; Chk "loc 'mo' = 1 (1 dong da_duyet con lai)" ($r.json.total -eq 1) ("total=" + $r.json.total)

  Write-Host "===== 3. kho / gia suc chua + gan hop (0035)"
  $r = Call GET "/cl/kho-gia/tong-hop?projectId=$pid1" $tc; $g1 = @($r.json.rows)[0]
  Chk "tong hop gia: suc_chua=3 so_hop=2 pct=66.7 gan_day (nguong 60)" ($r.code -eq 200 -and $g1.suc_chua -eq 3 -and $g1.so_hop -eq 2 -and $g1.so_ho_so -eq 2 -and $g1.gan_day -eq $true -and $g1.vuot -eq $false -and $r.json.ganDay -eq 1) (Than $g1)
  $r = Call GET "/cl/kho-gia/ho-so?projectId=$pid1&giaId=$gia" $tc; Chk "ho so tren gia = 2" ($r.json.total -eq 2) ("total=" + $r.json.total)
  $r = Call GET "/cl/kho-gia/ho-so?projectId=$pid1&khoId=$kho&format=xlsx" $tc; Chk "xuat Excel ho so theo kho" ($r.code -eq 200 -and $r.bytes.Length -gt 500 -and $r.bytes[0] -eq 0x50 -and $r.bytes[1] -eq 0x4B) ("code=" + $r.code + " bytes=" + $r.bytes.Length)
  $r = Call GET "/cl/kho-gia/chua-hop?projectId=$pid1&phongId=$phong" $tc
  Chk "chua hop: 3 ho so (03, 04, 05), ho so chua phong ban truoc" ($r.code -eq 200 -and @($r.json.rows).Count -eq 3) ("n=" + @($r.json.rows).Count)
  $r = Call POST "/cl/ho-so/gan-hop" $tc @{ projectId = $pid1; phongBanId = $pbA; rows = @(@{ hoSoId = $hs["05"]; hopSo = "3" }, @{ hoSoId = $hs["03"]; hopSo = "3" }, @{ hoSoId = [guid]::NewGuid().ToString(); hopSo = "4" }) }
  Chk "gan hop: applied=2 phongBanDat=1 hopsCreated=1 errors=1" ($r.code -eq 200 -and $r.json.applied -eq 2 -and $r.json.phongBanDat -eq 1 -and $r.json.hopsCreated -eq 1 -and @($r.json.errors).Count -eq 1) (Than $r.json)
  $r = Call GET "/cl/kho-gia/tong-hop?projectId=$pid1" $tc; $g1 = @($r.json.rows)[0]; Chk "sau gan: so_hop=3 so_ho_so=4 pct=100 khong vuot" ($g1.so_hop -eq 3 -and $g1.so_ho_so -eq 4 -and $g1.pct_day -eq 100 -and $g1.vuot -eq $false) (Than $g1)
  $r = Call POST "/cl/ho-so/gan-hop" $tc @{ projectId = $pid1; phongBanId = $pbB; rows = @(@{ hoSoId = $hs["05"]; hopSo = "9" }) }
  Chk "gan lai ho so 05 sang phong ban khac -> loi theo dong" ($r.code -eq 200 -and $r.json.applied -eq 0 -and @($r.json.errors).Count -eq 1) (Than $r.json)
  $r = Call POST "/cl/ho-so/gan-hop" $tb @{ projectId = $pid1; phongBanId = $pbA; rows = @(@{ hoSoId = $hs["04"]; hopSo = "4" }) }; Chk "nvb (khong systematize/khogia) -> 403" ($r.code -eq 403) ("code=" + $r.code)
  Chk "cl_lich_su co 'boxing' cho 2 ho so" ((Sql "SELECT count(*) FROM cl_lich_su WHERE action = 'boxing' AND project_id = '$pid1'") -eq "2") ""
  Start-Sleep -Seconds 9
  $r = Call GET "/notifications" $qt; Chk "job nhac: quan tri nhan kho_day (gia 1 >= 60%)" (@($r.json.items | Where-Object { $_.kind -eq "kho_day" }).Count -ge 1) ("kinds=" + (($r.json.items | ForEach-Object { $_.kind } | Select-Object -Unique) -join ","))

  Write-Host "===== 4. bao cao TT05 (0036)"
  $r = Call GET "/bao-cao" $tc; $defs = @($r.json.baoCao)
  Chk "danh muc ban Luu tru: 18 bao cao (khong thu_nop), 5 nhom" ($r.code -eq 200 -and $defs.Count -eq 18 -and @($r.json.nhom).Count -eq 5 -and -not ($defs | Where-Object { $_.ma -eq "thu_nop" })) ("n=" + $defs.Count)
  $loiSql = 0
  foreach ($d in $defs) {
    $r = Call GET "/bao-cao/$($d.ma)?projectId=$pid1" $tc
    if ($r.code -ne 200) { $loiSql++; Write-Host ("   !! " + $d.ma + " code=" + $r.code + " " + (Than $r.json)) -ForegroundColor Yellow }
  }
  Chk "18 bao cao JSON deu chay (SQL hop le)" ($loiSql -eq 0) ("loi=" + $loiSql)
  $r = Call GET "/bao-cao/phong?projectId=$pid1" $tc; $row = @($r.json.rows)[0]
  Chk "bao cao phong: so_ho_so=5 so_hop=3 so_phong_ban=2, dong tong" ($row.so_ho_so -eq 5 -and $row.so_hop -eq 3 -and $row.so_phong_ban -eq 2 -and $r.json.tong -and $r.json.tong[4] -eq "5") (Than $row)
  $r = Call GET "/bao-cao/phong?projectId=$pid1&namTu=2026" $tc; Chk "loc nam 2026 -> so_ho_so=0" (@($r.json.rows)[0].so_ho_so -eq 0) ""
  $r = Call GET "/bao-cao/phong?projectId=$pid1&namTu=2026&namDen=2020" $tc; Chk "nam tu > nam den -> 400" ($r.code -eq 400) ("code=" + $r.code)
  $r = Call GET "/bao-cao/muon_tra?projectId=$pid1" $tc; Chk "so muon tra: 4 dong (2 phieu x 2), khong dong tong" (@($r.json.rows).Count -eq 4 -and $null -eq $r.json.tong) ("n=" + @($r.json.rows).Count)
  $r = Call GET "/bao-cao/che_do_su_dung?projectId=$pid1" $tc; Chk "che do su dung: 'Cong khai' 5 ho so" (@($r.json.rows | Where-Object { $_.gia_tri -eq "Cong khai" })[0].so_ho_so -eq 5) (Than $r.json.rows)
  $r = Call GET "/bao-cao/kho_gia?projectId=$pid1" $tc; Chk "bao cao kho gia: 1 gia, so_hop=3" (@($r.json.rows).Count -eq 1 -and @($r.json.rows)[0].so_hop -eq 3) ""
  $r = Call GET "/bao-cao/phong?projectId=$pid1&format=xlsx" $tc; Chk "Excel bao cao phong" ($r.code -eq 200 -and $r.bytes[0] -eq 0x50 -and $r.bytes[1] -eq 0x4B -and $r.ct -match "spreadsheetml") ("code=" + $r.code + " ct=" + $r.ct)
  [IO.File]::WriteAllBytes((Join-Path $sb "bao-cao-phong.xlsx"), $r.bytes)
  $r = Call GET "/bao-cao/phong?projectId=$pid1&format=pdf" $tc; Chk "PDF bao cao phong (A4 ngang, 13 cot)" ($r.code -eq 200 -and [Text.Encoding]::ASCII.GetString($r.bytes[0..4]) -eq "%PDF-") ("code=" + $r.code + " bytes=" + $r.bytes.Length)
  [IO.File]::WriteAllBytes((Join-Path $sb "bao-cao-phong.pdf"), $r.bytes)
  $r = Call GET "/bao-cao/muon_tra?projectId=$pid1&format=pdf" $tc; Chk "PDF so muon tra" ($r.code -eq 200 -and [Text.Encoding]::ASCII.GetString($r.bytes[0..4]) -eq "%PDF-") ("code=" + $r.code)
  [IO.File]::WriteAllBytes((Join-Path $sb "so-muon-tra.pdf"), $r.bytes)
  $r = Call GET "/bao-cao/phong?projectId=$pid1&format=docx" $tc; Chk "format la -> 400" ($r.code -eq 400) ("code=" + $r.code)
  $r = Call GET "/bao-cao/khong_co?projectId=$pid1" $tc; Chk "ma bao cao la -> 404" ($r.code -eq 404) ("code=" + $r.code)
  Write-Host ("   tep xem mat: " + (Join-Path $sb "bao-cao-phong.pdf") + " ; " + (Join-Path $sb "so-muon-tra.pdf") + " ; " + (Join-Path $sb "bao-cao-phong.xlsx"))

  Write-Host ("===== KET QUA: " + $(if ($script:sai -eq 0) { "TAT CA DAT" } else { "$($script:sai) buoc SAI" }))
} finally {
  if ($p -and -not $p.HasExited) { Stop-Process -Id $p.Id -Force }
}
exit $script:sai
