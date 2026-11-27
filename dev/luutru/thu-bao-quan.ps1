# Kịch bản thử API bảo quản Điều 29–31 + quét trùng Điều 48 (0032/0033) — chạy TRÊN MÁY NÀY (máy dev), không đụng máy chủ thật.
# Cần: Postgres TẠM (container postgres:16 đã áp 0001→0033), `cargo build -p digitext-api` (debug), docker CLI. Script XOÁ SẠCH
# users/projects của CSDL tạm rồi dựng: 3 tài liệu không có tệp trong kho (→ kiểm lỗi), 2 tài liệu cùng BLAKE3, 2 hồ sơ trùng mục lục.
# Luồng: kế hoạch lập → xem trước → trình → duyệt (super admin tự duyệt được) → chạy → hoàn thành → lỗi mở → khắc phục → kiểm lại
# → kết luận điền sẵn; lịch sao lưu ghi nhận / bản cũ cần xoá; phương tiện sắp hết hạn; sự cố + prefill Mẫu 04; sweep trùng.
#   powershell -ExecutionPolicy Bypass -File dev\luutru\thu-bao-quan.ps1 -MatKhauDb <mk>
param(
  [Parameter(Mandatory = $true)][string]$MatKhauDb,
  [string]$Container = "dt-luutru-thu",
  [int]$CongDb = 55432,
  [int]$CongApi = 18083,
  [string]$ThuMucTam = (Join-Path $env:TEMP "digitext-thu-bao-quan")
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
    $p = @{ Method = $m; Uri = "$B$path"; Headers = $h; UseBasicParsing = $true; TimeoutSec = 120 }
    if ($null -ne $body) { $p.ContentType = "application/json; charset=utf-8"; $p.Body = [Text.Encoding]::UTF8.GetBytes(($body | ConvertTo-Json -Depth 8 -Compress)) }
    $r = Invoke-WebRequest @p
    $txt = [Text.Encoding]::UTF8.GetString($r.RawContentStream.ToArray())
    return @{ code = [int]$r.StatusCode; json = $(if ($txt) { $txt | ConvertFrom-Json } else { $null }) }
  } catch {
    $resp = $_.Exception.Response
    if ($null -eq $resp) { return @{ code = 0; json = $null; loi = $_.Exception.Message } }
    $sr = New-Object IO.StreamReader($resp.GetResponseStream(), [Text.Encoding]::UTF8); $txt = $sr.ReadToEnd()
    return @{ code = [int]$resp.StatusCode; json = $(try { $txt | ConvertFrom-Json } catch { $txt }) }
  }
}
$script:sai = 0
function Chk($ten, $dk, $them) { if ($dk) { Write-Host ("  OK   {0} {1}" -f $ten, $them) -ForegroundColor Green } else { $script:sai++; Write-Host ("  SAI  {0} {1}" -f $ten, $them) -ForegroundColor Red } }

Write-Host "===== 0. don du lieu cu + superadmin"
Sql "TRUNCATE projects, users CASCADE" | Out-Null
# TRUNCATE CASCADE lam trong ca bang ma dung chung (cl_thbq / cl_ngon_ngu / cl_loai_van_ban, project_id NULL) -> nap lai tu 0012.
Get-Content (Join-Path $PSScriptRoot "seed-dung-chung-0012.sql") -Raw -Encoding UTF8 | docker exec -i $Container psql -X -q -U digitext -d digitext | Out-Null
# Không TRUNCATE bk_phuong_tien (CASCADE sẽ xoá luôn bk_lich_sao_luu); lịch mặc định của 0032 dựng lại nếu thiếu.
Sql "DELETE FROM bk_su_co; DELETE FROM backup_runs; DELETE FROM bk_phuong_tien; DELETE FROM bk_lich_sao_luu" | Out-Null
Sql "INSERT INTO bk_lich_sao_luu (id, loai, ten, chu_ky_gio, do_tre_cho_phep_gio, giu_so_ban, nguon) SELECT gen_random_uuid(), 'gia_tang', 'Sao luu gia tang hang ngay', 24, 2, 14, 'Kho tai lieu' WHERE NOT EXISTS (SELECT 1 FROM bk_lich_sao_luu WHERE loai = 'gia_tang')" | Out-Null
Sql "INSERT INTO bk_lich_sao_luu (id, loai, ten, chu_ky_gio, do_tre_cho_phep_gio, giu_so_ban, nguon) SELECT gen_random_uuid(), 'day_du', 'Sao luu day du CSDL hang ngay', 24, 2, 14, 'CSDL PostgreSQL (pg_dump)' WHERE NOT EXISTS (SELECT 1 FROM bk_lich_sao_luu WHERE loai = 'day_du')" | Out-Null
Sql "INSERT INTO bk_lich_sao_luu (id, loai, ten, chu_ky_gio, do_tre_cho_phep_gio, giu_so_ban, nguon) SELECT gen_random_uuid(), 'day_du_3_nam', 'Sao luu day du 3 nam', 26280, 720, 3, 'CSDL + kho + ma nguon' WHERE NOT EXISTS (SELECT 1 FROM bk_lich_sao_luu WHERE loai = 'day_du_3_nam')" | Out-Null
$mk = Join-Path $sb "mk.txt"; [IO.File]::WriteAllText($mk, "ThuNghiemCucBo2026")
& $api bootstrap-superadmin --username quantri --display-name "Quan tri" --password-file $mk 2>&1 | Select-Object -Last 1

$p = Start-Process -FilePath $api -ArgumentList "serve" -NoNewWindow -PassThru -RedirectStandardOutput (Join-Path $sb "api.out.log") -RedirectStandardError (Join-Path $sb "api.err.log")
try {
  $han = (Get-Date).AddSeconds(40)
  do { Start-Sleep -Milliseconds 600; try { $ok = (Invoke-WebRequest -UseBasicParsing -TimeoutSec 3 "http://127.0.0.1:$CongApi/healthz").StatusCode -eq 200 } catch { $ok = $false } } until ($ok -or (Get-Date) -gt $han)
  if (-not $ok) { Write-Host "API khong len"; Get-Content (Join-Path $sb "api.err.log") -Tail 15; exit 1 }

  Write-Host "===== 1. du lieu: du an, phong, 2 ho so trung muc luc, 3 tai lieu (2 cung blake3)"
  $pid1 = [guid]::NewGuid().ToString(); $phong = [guid]::NewGuid().ToString(); $pbA = [guid]::NewGuid().ToString()
  Sql "INSERT INTO projects (id, code, name) VALUES ('$pid1', 'BQ', 'Kho thu bao quan')" | Out-Null
  Sql "INSERT INTO cl_phong (id, project_id, ma_du_an, ten, phong_so) VALUES ('$phong', '$pid1', 'BQ', 'Phong thu', '01')" | Out-Null
  Sql "INSERT INTO cl_phong_ban (id, project_id, phong_id, ma_don_vi, ten) VALUES ('$pbA', '$pid1', '$phong', 'VP', 'Van phong')" | Out-Null
  # Thông báo bảo quản gửi quản trị DỰ ÁN (project_admins): gắn quantri làm quản trị kho này để nhận plan_done / incident.
  Sql "INSERT INTO project_admins (project_id, admin_id) SELECT '$pid1', id FROM users WHERE username = 'quantri'" | Out-Null
  $hs1 = [guid]::NewGuid().ToString(); $hs2 = [guid]::NewGuid().ToString()
  Sql "INSERT INTO cl_ho_so (id, project_id, phong_id, phong_ban_id, so_tam, tieu_de, so_ky_hieu_hs, ngay_ket_thuc, created_at) VALUES ('$hs1', '$pid1', '$phong', '$pbA', '01', 'Tap luu Quyet dinh nam 2025', '12/QD', '2025', now() - interval '2 day')" | Out-Null
  Sql "INSERT INTO cl_ho_so (id, project_id, phong_id, phong_ban_id, so_tam, tieu_de, so_ky_hieu_hs, ngay_ket_thuc, created_at) VALUES ('$hs2', '$pid1', '$phong', '$pbA', '02', 'TAP LUU QUYET DINH NAM 2025', '12/QD', '2025', now() - interval '1 day')" | Out-Null
  $d1 = [guid]::NewGuid().ToString(); $d2 = [guid]::NewGuid().ToString(); $d3 = [guid]::NewGuid().ToString()
  $b3 = "a" * 64
  Sql "INSERT INTO documents (id, project_id, filename, ho_so_ref, size_bytes, blake3, storage_path, status, page_count, uploaded_at) VALUES ('$d1', '$pid1', 'tep-1.pdf', 'HS01', 1000, '$b3', 'x/1', 'archived', 3, now() - interval '3 hour'), ('$d2', '$pid1', 'tep-2.pdf', 'HS01', 1000, '$b3', 'x/2', 'archived', 3, now() - interval '2 hour'), ('$d3', '$pid1', 'tep-3.pdf', 'HS02', 2000, '$('b' * 64)', 'x/3', 'archived', 5, now() - interval '1 hour')" | Out-Null
  $vb = [guid]::NewGuid().ToString()
  Sql "INSERT INTO cl_van_ban (id, project_id, ho_so_id, so_thu_tu, trich_yeu) VALUES ('$vb', '$pid1', '$hs2', 1, 'Quyet dinh 12')" | Out-Null
  Sql "INSERT INTO cl_van_ban_file (id, project_id, van_ban_id, document_id, name) VALUES ('$([guid]::NewGuid())', '$pid1', '$vb', '$d3', 'tep-3.pdf')" | Out-Null
  Write-Host ("   documents: " + (Sql "SELECT count(*) FROM documents"))
  $qt = (Call POST "/auth/login" $null @{ username = "quantri"; password = "ThuNghiemCucBo2026" }).json.accessToken
  Chk "login quantri" ($qt) ""

  Write-Host "===== 2. ke hoach kiem tra (Dieu 29)"
  $r = Call POST "/bao-quan/ke-hoach" $qt @{ projectId = $pid1; nam = 2026; ten = "Kiem tra nam 2026"; phamVi = @{ lauChuaKiem = $true; goi = $true } }
  $kh = $r.json; Chk "lap ke hoach du thao" ($r.code -eq 201 -and $kh.trang_thai -eq "du_thao" -and $kh.ma -eq "KT-2026-01") ("code=" + $r.code + " ma=" + $kh.ma)
  $r = Call POST "/bao-quan/ke-hoach/xem-truoc" $qt @{ projectId = $pid1; phamVi = @{ lauChuaKiem = $true } }
  Chk "xem truoc 3 tai lieu" ($r.json.taiLieu -eq 3) ("taiLieu=" + $r.json.taiLieu + " moTa=" + $r.json.moTa)
  $r = Call POST "/bao-quan/ke-hoach/xem-truoc" $qt @{ projectId = $pid1; phamVi = @{ xacSuat = 0 } }; Chk "xacSuat=0 bi tu choi" ($r.code -eq 400) ("code=" + $r.code)
  $r = Call POST "/bao-quan/ke-hoach/$($kh.id)/chay" $qt; Chk "chay khi chua duyet -> 400" ($r.code -eq 400) ("code=" + $r.code)
  $r = Call POST "/bao-quan/ke-hoach/$($kh.id)/trinh" $qt; Chk "trinh" ($r.code -eq 200 -and $r.json.trang_thai -eq "cho_duyet") ("code=" + $r.code)
  $r = Call POST "/bao-quan/ke-hoach/$($kh.id)/duyet" $qt; Chk "duyet (super tu duyet) chot 3 tep" ($r.code -eq 200 -and $r.json.trang_thai -eq "da_duyet" -and $r.json.so_tep -eq 3) ("code=" + $r.code + " so_tep=" + $r.json.so_tep + " than=" + ($r.json | ConvertTo-Json -Compress -Depth 3).Substring(0, [Math]::Min(200, ($r.json | ConvertTo-Json -Compress -Depth 3).Length)))
  $r = Call POST "/bao-quan/ke-hoach/$($kh.id)/chay?limit=10" $qt
  Chk "chay: 3 tep deu loi (khong co trong kho), hoan thanh" ($r.code -eq 200 -and $r.json.daKiem -eq 3 -and $r.json.loi -eq 3 -and $r.json.hoanThanh) ("than=" + ($r.json | ConvertTo-Json -Compress))
  $r = Call GET "/bao-quan/ke-hoach/$($kh.id)" $qt; Chk "ke hoach hoan_thanh so_loi=3" ($r.json.trang_thai -eq "hoan_thanh" -and $r.json.so_loi -eq 3 -and $r.json.so_loi_mo -eq 3) ("tt=" + $r.json.trang_thai + " so_loi=" + $r.json.so_loi)
  $r = Call GET "/bao-quan/ke-hoach/$($kh.id)/tep?trangThai=loi" $qt; Chk "tep loi 3 dong co tenTep" ($r.json.total -eq 3 -and $r.json.rows[0].tenTep) ("total=" + $r.json.total)
  $r = Call GET "/bao-quan/loi?projectId=$pid1" $qt; $loi = @($r.json.rows); Chk "danh muc loi 3 dong moi" ($loi.Count -eq 3 -and $loi[0].trang_thai -eq "moi" -and $loi[0].ke_hoach_ma -eq "KT-2026-01") ("count=" + $loi.Count)
  $r = Call POST "/bao-quan/loi/$($loi[0].id)/khac-phuc" $qt @{ khacPhuc = "Khoi phuc tep tu ban sao luu 25/11" }; Chk "khac phuc" ($r.code -eq 200) ("code=" + $r.code)
  $r = Call POST "/bao-quan/loi/$($loi[0].id)/kiem-lai" $qt; Chk "kiem lai van loi (kho trong)" ($r.code -eq 200 -and $r.json.ketQua -eq "loi") ("ketQua=" + $r.json.ketQua)
  $r = Call GET "/bao-quan/loi?projectId=$pid1" $qt; $l0 = @($r.json.rows) | Where-Object { $_.id -eq $loi[0].id }; Chk "dong -> kiem_lai_loi, giu khac_phuc" ($l0.trang_thai -eq "kiem_lai_loi" -and $l0.khac_phuc) ("tt=" + $l0.trang_thai)
  $r = Call POST "/bao-quan/loi/$($loi[1].id)/bo-qua" $qt @{ lyDo = "Tep thu, khong co that" }; Chk "bo qua" ($r.code -eq 200) ("code=" + $r.code)
  $r = Call GET "/bao-quan/ke-hoach/$($kh.id)/ket-luan" $qt
  Chk "ket luan dien san Mau 01" ($r.json.fields.tongSoTep -eq "3" -and $r.json.fields.soLoi -eq "3" -and @($r.json.lists.tepLoi).Count -eq 3 -and $r.json.lists.tepLoi[0].diaChi) ("phamVi=" + $r.json.fields.phamVi + " tepLoi[0]=" + ($r.json.lists.tepLoi[0] | ConvertTo-Json -Compress))
  $r = Call GET "/ops/integrity" $qt; Chk "/ops/integrity dem 3 loi" ($r.json.soLoi -eq 3) ("soLoi=" + $r.json.soLoi)
  $r = Call POST "/ops/integrity/run?limit=5" $qt; Chk "/ops/integrity/run (kiem tu dong) khong tao them dong loi mo" ($r.code -eq 200) ("code=" + $r.code)
  $r = Call GET "/bao-quan/loi?projectId=$pid1" $qt; Chk "van 2 dong mo (1 bo qua)" (@($r.json.rows).Count -eq 2) ("count=" + @($r.json.rows).Count)
  $r = Call GET "/notifications" $qt; Chk "quan tri nhan plan_done" (@($r.json.items | Where-Object { $_.kind -eq "plan_done" }).Count -ge 1) ("kinds=" + (($r.json.items | ForEach-Object { $_.kind } | Select-Object -Unique) -join ","))

  Write-Host "===== 3. lich sao luu (Dieu 30)"
  $r = Call GET "/bao-quan/lich" $qt; $lich = @($r.json.rows); Chk "3 lich mac dinh, chua co ban" ($lich.Count -eq 3 -and ($lich | Where-Object { $_.tinhTrang -eq "chua_co" }).Count -eq 3) ("count=" + $lich.Count)
  $lDay = $lich | Where-Object { $_.loai -eq "day_du" } | Select-Object -First 1
  $r = Call POST "/bao-quan/sao-luu" $qt @{ loai = "day_du"; nguon = "CSDL PostgreSQL (pg_dump)"; ketQua = "dat"; duongDan = "E:\SaoLuu\csdl\digitext-20261126-0100.dump"; dungLuongBytes = 52428800; maBam = ("c" * 64) }
  Chk "ghi nhan sao luu gan lich theo loai" ($r.code -eq 201 -and $r.json.lich_id -eq $lDay.id -and $r.json.duong_dan) ("code=" + $r.code + " lich_id=" + $r.json.lich_id)
  $r = Call GET "/bao-quan/sao-luu/tinh-trang" $qt; $lDay2 = @($r.json.rows) | Where-Object { $_.id -eq $lDay.id }; Chk "lich day_du dung han, soBan=1" ($lDay2.tinhTrang -eq "dung_han" -and $lDay2.soBan -eq 1) ("tt=" + $lDay2.tinhTrang + " han=" + $lDay2.hanKeTiep)
  $lGt = $lich | Where-Object { $_.loai -eq "gia_tang" } | Select-Object -First 1
  Sql "INSERT INTO backup_runs (id, loai, nguon, ket_qua, lich_id, duong_dan, created_at) SELECT gen_random_uuid(), 'gia_tang', 'Kho', 'dat', '$($lGt.id)', 'E:\SaoLuu\kho-' || g, now() - (g || ' day')::interval FROM generate_series(2, 17) g" | Out-Null
  $r = Call GET "/bao-quan/sao-luu/tinh-trang" $qt; $lGt2 = @($r.json.rows) | Where-Object { $_.id -eq $lGt.id }
  Chk "gia_tang: 16 ban, giu 14 -> can xoa 2, tre (ban moi nhat 2 ngay)" ($lGt2.soBan -eq 16 -and $lGt2.soBanCanXoa -eq 2 -and @($lGt2.banCuCanXoa).Count -eq 2 -and $lGt2.tinhTrang -eq "tre") ("soBan=" + $lGt2.soBan + " canXoa=" + $lGt2.soBanCanXoa + " tt=" + $lGt2.tinhTrang + " treGio=" + $lGt2.treGio)
  $cu = @($lGt2.banCuCanXoa)[0]
  $r = Call POST "/bao-quan/sao-luu/$($cu.id)/da-xoa" $qt; Chk "danh dau da xoa" ($r.code -eq 200) ("code=" + $r.code)
  $r = Call POST "/bao-quan/sao-luu/$($cu.id)/da-xoa" $qt; Chk "danh dau lan 2 -> 400" ($r.code -eq 400) ("code=" + $r.code)
  $r = Call GET "/bao-quan/sao-luu/tinh-trang" $qt; $lGt3 = @($r.json.rows) | Where-Object { $_.id -eq $lGt.id }; Chk "con can xoa 1" ($lGt3.soBanCanXoa -eq 1) ("canXoa=" + $lGt3.soBanCanXoa)
  $r = Call PUT "/bao-quan/lich/$($lGt.id)" $qt @{ ten = "Gia tang hang ngay"; chuKyGio = 24; doTreChoPhepGio = 72; giuSoBan = 20; bat = $true }
  $r = Call GET "/bao-quan/sao-luu/tinh-trang" $qt; $lGt4 = @($r.json.rows) | Where-Object { $_.id -eq $lGt.id }; Chk "sua lich: tre cho phep 72h -> dung han, giu 20 -> khong can xoa" ($lGt4.tinhTrang -eq "dung_han" -and $lGt4.soBanCanXoa -eq 0) ("tt=" + $lGt4.tinhTrang + " canXoa=" + $lGt4.soBanCanXoa)
  $r = Call GET "/ops/backups" $qt; Chk "/ops/backups cu van doc duoc" ($r.code -eq 200 -and @($r.json).Count -ge 17) ("count=" + @($r.json).Count)

  Write-Host "===== 4. phuong tien (Dieu 30.4)"
  $han = (Get-Date).AddDays(100).ToString("yyyy-MM-dd")
  $r = Call POST "/bao-quan/phuong-tien" $qt @{ ma = "HDD-SL-01"; loai = "o_cung"; moTa = "WD 4TB"; dungLuongBytes = 4000000000000; noiBaoQuan = "Tu sat phong may"; hanDoBen = $han }
  $pt = $r.json; Chk "tao phuong tien" ($r.code -eq 201 -and $pt.ma -eq "HDD-SL-01") ("code=" + $r.code)
  $r = Call POST "/bao-quan/phuong-tien" $qt @{ ma = "HDD-SL-01"; loai = "o_cung" }; Chk "trung ma -> 409" ($r.code -eq 409) ("code=" + $r.code)
  $r = Call GET "/bao-quan/phuong-tien" $qt; $pt2 = @($r.json.rows) | Where-Object { $_.id -eq $pt.id }; Chk "sap het han (100 ngay <= 365)" ($pt2.sap_het -eq $true -and $pt2.con_ngay -le 100) ("con_ngay=" + $pt2.con_ngay)
  $r = Call PUT "/bao-quan/phuong-tien/$($pt.id)" $qt @{ ma = "HDD-SL-01"; loai = "o_cung"; hanDoBen = (Get-Date).AddYears(3).ToString("yyyy-MM-dd"); trangThai = "dang_dung" }
  Chk "sua han 3 nam -> khong sap het" ($r.code -eq 200 -and $r.json.han_do_ben) ("code=" + $r.code)
  $r = Call GET "/bao-quan/phuong-tien" $qt; $pt3 = @($r.json.rows) | Where-Object { $_.id -eq $pt.id }; Chk "sap_het=false" ($pt3.sap_het -eq $false) ("sap_het=" + $pt3.sap_het)

  Write-Host "===== 5. su co (Dieu 31)"
  $runDay = (Call GET "/bao-quan/sao-luu?lichId=$($lDay.id)" $qt).json.rows[0]
  $r = Call POST "/bao-quan/su-co" $qt @{ moTa = "O kho loi sector"; nguyenNhan = "Phan cung"; mucDo = "cao"; phamVi = "Kho tai lieu"; bienPhap = "Cach ly o`nChuyen kho sang o moi"; backupRunId = $runDay.id; phuongPhapPhucHoi = "Chep lai tu ban sao luu" }
  $sc = $r.json; Chk "ghi su co SC-<nam>-001" ($r.code -eq 201 -and $sc.ma -match "^SC-\d{4}-001$" -and $sc.backup_run.id -eq $runDay.id) ("code=" + $r.code + " ma=" + $sc.ma)
  $r = Call GET "/bao-quan/su-co/$($sc.id)/prefill" $qt
  Chk "prefill Mau 04 co nguon phuc hoi + 2 bien phap" ($r.json.fields.nguonPhucHoi -match "pg_dump" -and @($r.json.lists.bienPhap).Count -eq 2 -and $r.json.fields.mucDo -eq "Cao") ("nguon=" + $r.json.fields.nguonPhucHoi)
  $r = Call PUT "/bao-quan/su-co/$($sc.id)" $qt @{ moTa = "O kho loi sector"; mucDo = "cao"; trangThai = "da_phuc_hoi"; ketQuaKiemTra = "Kiem 3 tep dat"; backupRunId = $runDay.id }
  Chk "cap nhat da_phuc_hoi" ($r.code -eq 200 -and $r.json.trang_thai -eq "da_phuc_hoi") ("code=" + $r.code)
  $r = Call GET "/notifications" $qt; Chk "quan tri nhan incident (muc cao)" (@($r.json.items | Where-Object { $_.kind -eq "incident" }).Count -ge 1) ""
  $r = Call GET "/bao-quan/tong-quan" $qt; Chk "tong quan: loiMo=2 suCoMo=1 phuongTienSapHet=0" ($r.json.loiMo -eq 2 -and $r.json.suCoMo -eq 1 -and $r.json.phuongTienSapHet -eq 0 -and @($r.json.lich).Count -eq 3) ("than=" + (($r.json | Select-Object khChoDuyet, khDangChay, loiMo, suCoMo, lichTre, banCuCanXoa, phuongTienSapHet) | ConvertTo-Json -Compress))

  Write-Host "===== 6. quet trung (Dieu 48.2, 0033)"
  $r = Call POST "/disposal/sweep" $qt
  Chk "sweep: blake3=1 (tep-2 trung tep-1), muc_luc=1 (tep-3 cua ho so 02 trung ho so 01)" ($r.code -eq 200 -and $r.json.trungBlake3 -eq 1 -and $r.json.trungMucLuc -eq 1) ("than=" + ($r.json | ConvertTo-Json -Compress))
  $r = Call GET "/disposal?projectId=$pid1" $qt; $ents = @($r.json)
  Chk "danh muc huy 2 dong co phatHien" ($ents.Count -eq 2 -and @($ents | Where-Object { $_.phatHien -eq "blake3" -and $_.documentId -eq $d2 }).Count -eq 1 -and @($ents | Where-Object { $_.phatHien -eq "muc_luc" -and $_.dupHoSoId -eq $hs1 }).Count -eq 1) ("rows=" + ($ents | ForEach-Object { $_.phatHien + ":" + $_.filename } | Sort-Object) -join ",")
  $r = Call POST "/disposal/sweep" $qt; Chk "sweep lan 2 khong tang" ($r.json.trungLap -eq 0) ("trungLap=" + $r.json.trungLap)
  Chk "trung_thua KHONG tu danh" ((Sql "SELECT count(*) FROM cl_ho_so WHERE trung_thua") -eq "0") ""

  Write-Host ("===== KET QUA: " + $(if ($script:sai -eq 0) { "TAT CA DAT" } else { "$($script:sai) buoc SAI" }))
} finally {
  if ($p -and -not $p.HasExited) { Stop-Process -Id $p.Id -Force }
}
exit $script:sai
