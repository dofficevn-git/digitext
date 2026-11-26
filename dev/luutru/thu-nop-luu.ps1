# Kịch bản thử API lưu trữ hiện hành (Điều 19) — chạy TRÊN MÁY NÀY (máy dev), không đụng máy chủ thật.
# Cần: Postgres TẠM (container postgres:16, đã áp 0001→0031 bằng `digitext-api migrate`), `cargo build -p digitext-api`
# (debug), docker CLI. Script XOÁ SẠCH users/projects của CSDL tạm đó rồi dựng lại: quantri (super admin), nva (nhân viên
# phòng ban A), canbo (quản trị dự án = cán bộ lưu trữ) → nộp lưu → khoá → tiếp nhận → biên bản → sổ → danh mục trọn luồng.
# Ví dụ:
#   docker run -d --name dt-luutru-thu -e POSTGRES_USER=digitext -e POSTGRES_PASSWORD=<mk> -e POSTGRES_DB=digitext -p 55432:5432 postgres:16
#   powershell -ExecutionPolicy Bypass -File dev\luutru\thu-nop-luu.ps1 -MatKhauDb <mk>
param(
  [Parameter(Mandatory = $true)][string]$MatKhauDb,
  [string]$Container = "dt-luutru-thu",
  [int]$CongDb = 55432,
  [int]$CongApi = 18082,
  [string]$ThuMucTam = (Join-Path $env:TEMP "digitext-thu-nop-luu")
)
$ErrorActionPreference = "Continue"
$repo = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path; $sb = $ThuMucTam; New-Item -ItemType Directory -Force $sb | Out-Null
$env:DATABASE_URL = "postgres://digitext:$MatKhauDb@127.0.0.1:$CongDb/digitext"
$env:DIGITEXT_EDITION = "luu_tru"; $env:DIGITEXT_BIND = "127.0.0.1:$CongApi"; $env:DIGITEXT_PUBLIC_URL = "http://127.0.0.1:$CongApi"
$env:DIGITEXT_DATA_DIR = Join-Path $sb "data-nl"; $env:DIGITEXT_UPDATES_DIR = Join-Path $sb "updates"
$env:DIGITEXT_JWT_KEY_PATH = Join-Path $sb "data-nl\jwt-ed25519.pem"; $env:DIGITEXT_PDF_STORE_DIR = Join-Path $sb "kho-nl"
$env:DIGITEXT_MIGRATIONS_DIR = Join-Path $repo "migrations"; $env:DIGITEXT_CERT_FONT = "C:\Windows\Fonts\times.ttf"
New-Item -ItemType Directory -Force (Join-Path $sb "data-nl"), (Join-Path $sb "kho-nl") | Out-Null
$api = Join-Path $repo "target\debug\digitext-api.exe"
$B = "http://127.0.0.1:$CongApi/api/v1"
function Sql($q) { docker exec $Container psql -X -t -A -U digitext -d digitext -c $q }
function Call($m, $path, $tok, $body) {
  $h = @{}; if ($tok) { $h.Authorization = "Bearer $tok" }
  try {
    $p = @{ Method = $m; Uri = "$B$path"; Headers = $h; UseBasicParsing = $true; TimeoutSec = 60 }
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
function Chk($ten, $dk, $them) { if ($dk) { Write-Host ("  OK   {0} {1}" -f $ten, $them) -ForegroundColor Green } else { Write-Host ("  SAI  {0} {1}" -f $ten, $them) -ForegroundColor Red } }

Write-Host "===== 0. don du lieu cu + superadmin"
Sql "DELETE FROM projects WHERE code = 'LTHH'" | Out-Null
# CSDL tạm: xoá sạch bằng TRUNCATE CASCADE (audit_log, records… tham chiếu users không cascade nên DELETE bị chặn).
Sql "TRUNCATE projects, users CASCADE" | Out-Null; Write-Host ("   users con lai: " + (Sql "SELECT count(*) FROM users"))
$mk = Join-Path $sb "mk-nl.txt"; [IO.File]::WriteAllText($mk, "ThuNghiemCucBo2026")
& $api bootstrap-superadmin --username quantri --display-name "Quan tri" --password-file $mk 2>&1 | Select-Object -Last 1

$p = Start-Process -FilePath $api -ArgumentList "serve" -NoNewWindow -PassThru -RedirectStandardOutput (Join-Path $sb "api-nl.out.log") -RedirectStandardError (Join-Path $sb "api-nl.err.log")
try {
  $han = (Get-Date).AddSeconds(40)
  do { Start-Sleep -Milliseconds 600; try { $ok = (Invoke-WebRequest -UseBasicParsing -TimeoutSec 3 "http://127.0.0.1:$CongApi/healthz").StatusCode -eq 200 } catch { $ok = $false } } until ($ok -or (Get-Date) -gt $han)
  if (-not $ok) { Write-Host "API khong len"; Get-Content (Join-Path $sb "api-nl.err.log") -Tail 15; exit 1 }
  Write-Host "--- vai tro ban Luu tru:"; Sql "SELECT code FROM roles WHERE code IN ('nhan_vien_phong_ban','can_bo_luu_tru','lanh_dao') ORDER BY 1"

  Write-Host "===== 1. du an, phong, phong ban (SQL), nguoi dung (API)"
  $pid1 = [guid]::NewGuid().ToString(); $phong = [guid]::NewGuid().ToString(); $pbA = [guid]::NewGuid().ToString(); $pbB = [guid]::NewGuid().ToString()
  Sql "INSERT INTO projects (id, code, name) VALUES ('$pid1', 'LTHH', 'UBND huyen Binh Lieu')" | Out-Null
  Sql "INSERT INTO cl_phong (id, project_id, ma_du_an, ten, phong_so) VALUES ('$phong', '$pid1', 'UBND', 'Phong UBND huyen', '01')" | Out-Null
  Sql "INSERT INTO cl_phong_ban (id, project_id, phong_id, ma_don_vi, ten) VALUES ('$pbA', '$pid1', '$phong', 'VP', 'Van phong'), ('$pbB', '$pid1', '$phong', 'NV', 'Phong Noi vu')" | Out-Null
  $qt = (Call POST "/auth/login" $null @{ username = "quantri"; password = "ThuNghiemCucBo2026" }).json.accessToken
  Chk "login quantri" ($qt) ""
  $u1 = Call POST "/users" $qt @{ username = "nva"; password = "MatKhau12345"; displayName = "Nhan vien A"; role = "user"; projectId = $pid1; phongBanId = $pbA }
  Chk "tao nvA co phongBanId" ($u1.code -eq 201 -and $u1.json.phongBanId -eq $pbA) "code=$($u1.code)"
  $u2 = Call POST "/users" $qt @{ username = "canbo"; password = "MatKhau12345"; displayName = "Can bo LT"; role = "admin"; projectId = $pid1 }
  Chk "tao canbo (admin du an)" ($u2.code -eq 201) "code=$($u2.code)"
  $hs = @{}
  foreach ($x in @(@("01", $pbA), @("02", $pbA), @("03", $pbB))) {
    $id = [guid]::NewGuid().ToString(); $hs[$x[0]] = $id
    Sql "INSERT INTO cl_ho_so (id, project_id, phong_id, phong_ban_id, so_tam, tieu_de, ngay_ket_thuc, created_by) VALUES ('$id', '$pid1', '$phong', '$($x[1])', '$($x[0])', 'Ho so so $($x[0])', '2025', (SELECT id FROM users WHERE username='quantri'))" | Out-Null
  }

  Write-Host "===== 2. nvA: pham vi, tong quan, nop luu, bi khoa"
  $ta = (Call POST "/auth/login" $null @{ username = "nva"; password = "MatKhau12345" }).json
  Chk "profile nvA co tenPhongBan" ($ta.user.tenPhongBan -eq "Van phong") "tenPhongBan=$($ta.user.tenPhongBan)"
  $ta = $ta.accessToken
  $r = Call GET "/cl/ho-so?projectId=$pid1" $ta; Chk "nvA chi thay 2 ho so phong A" ($r.json.total -eq 2) "total=$($r.json.total)"
  $r = Call GET "/nop-luu/tong-quan?projectId=$pid1" $ta; Chk "tong quan nvA quaHan=2" ($r.json.quaHan -eq 2) ("quaHan=" + $r.json.quaHan + " chuaNop=" + $r.json.chuaNop)
  $r = Call GET "/nop-luu/ho-so-can-nop?projectId=$pid1&loc=qua_han" $ta; Chk "ho so can nop qua han" ($r.json.total -eq 2 -and $r.json.rows[0].con_ngay -lt 0) ("total=" + $r.json.total + " conNgay=" + $r.json.rows[0].con_ngay)
  $r = Call POST "/nop-luu" $ta @{ projectId = $pid1; hoSoIds = @($hs["01"], $hs["02"]) }; $phieu = $r.json
  Chk "lap phieu" ($r.code -eq 200 -and $phieu.trang_thai -eq "cho_tiep_nhan") ("code=" + $r.code + " ma_phieu=" + $phieu.ma_phieu)
  $r = Call PUT "/cl/rows/ho_so/$($hs['01'])" $ta @{ projectId = $pid1; new = @{ tieu_de = "Sua khi dang cho" } }
  $than = $(if ($r.json -is [string]) { $r.json } else { $r.json | ConvertTo-Json -Compress -Depth 5 })
  Chk "sua ho so da nop -> locked" ($r.code -ge 400 -and $than -match "locked") ("code=" + $r.code + " than=" + $than)
  $r = Call POST "/nop-luu/$($hs['01'])/tiep-nhan" $ta; Chk "nvA khong tiep nhan duoc (403/404)" ($r.code -in 403, 404) ("code=" + $r.code)

  Write-Host "===== 3. canbo: cham, tiep nhan, bien ban, so"
  $tc = (Call POST "/auth/login" $null @{ username = "canbo"; password = "MatKhau12345" }).json.accessToken
  $r = Call GET "/nop-luu?projectId=$pid1&trangThai=cho_tiep_nhan" $tc; Chk "canbo thay 1 phieu cho" ($r.json.total -eq 1) ("total=" + $r.json.total)
  $r = Call POST "/nop-luu/$($phieu.id)/ho-so/$($hs['02'])/ket-qua" $tc @{ ketQua = "tra_lai"; lyDo = "Thieu chung tu ket thuc" }; Chk "cham tra lai 02" ($r.code -eq 200) ("code=" + $r.code)
  $r = Call POST "/nop-luu/$($phieu.id)/tiep-nhan" $tc @{ ghiChu = "Nhan 1, tra 1" }
  Chk "tiep nhan" ($r.code -eq 200 -and $r.json.phieu.trang_thai -eq "da_tiep_nhan" -and $r.json.phieu.bien_ban_id) ("code=" + $r.code + " so_bien_ban=" + $r.json.phieu.so_bien_ban)
  $bb = $r.json.phieu.bien_ban_id
  $r = Call GET "/records?projectId=$pid1&kind=giao_nhan_nop_luu" $tc
  $rec = @($r.json)[0]; if ($r.json.items) { $rec = $r.json.items[0] }
  Chk "records co bien ban GNL" ($rec.soBienBan -match "BB-GNL") ("so=" + $rec.soBienBan)
  if ($bb) {
    $h = @{ Authorization = "Bearer $tc" }
    Invoke-WebRequest -UseBasicParsing -Headers $h -Uri "$B/records/$bb/download" -OutFile (Join-Path $sb "bien-ban-gnl.pdf")
    Chk "tai PDF bien ban" ((Get-Item (Join-Path $sb "bien-ban-gnl.pdf")).Length -gt 1000) ""
    Invoke-WebRequest -UseBasicParsing -Headers $h -Uri "$B/records/$bb/docx" -OutFile (Join-Path $sb "bien-ban-gnl.docx")
    Chk "tai DOCX bien ban" ((Get-Item (Join-Path $sb "bien-ban-gnl.docx")).Length -gt 1000) ""
  }
  $r = Call GET "/nop-luu/so-nop-luu?projectId=$pid1" $tc; Chk "so nop luu 1 dong" (@($r.json.rows).Count -eq 1) ("rows=" + @($r.json.rows).Count)
  $r = Call GET "/notifications" $ta; Chk "nvA nhan thong bao ket qua" (@($r.json.items | Where-Object { $_.kind -eq "nop_luu_ket_qua" }).Count -ge 1) ("kinds=" + (($r.json.items | ForEach-Object { $_.kind }) -join ","))
  $r = Call GET "/cl/ho-so?projectId=$pid1&nopLuu=da_nop" $ta; Chk "nvA van thay ho so da nop (360 ngay)" ($r.json.total -eq 1) ("total=" + $r.json.total)

  Write-Host "===== 4. danh muc ho so"
  $r = Call POST "/danh-muc-ho-so" $tc @{ projectId = $pid1; phongId = $phong; nam = 2026 }; $dm = $r.json
  Chk "canbo tao danh muc" ($r.code -eq 200 -and $dm.trang_thai -eq "du_thao") ("code=" + $r.code)
  $r = Call POST "/danh-muc-ho-so/$($dm.id)/dong" $ta @{ phongBanId = $pbA; maDong = "01"; tieuDe = "Tap luu quyet dinh" }; Chk "nvA them dong phong A" ($r.code -eq 200) ("code=" + $r.code)
  $r = Call POST "/danh-muc-ho-so/$($dm.id)/dong" $ta @{ phongBanId = $pbB; maDong = "02"; tieuDe = "Dong phong B" }; Chk "nvA them dong phong B bi tu choi" ($r.code -eq 400) ("code=" + $r.code)
  $r = Call POST "/danh-muc-ho-so/$($dm.id)/dong/nhap" $tc @{ rows = @(@{ phongBanId = $pbB; maDong = "02"; tieuDe = "Ho so hoi nghi" }, @{ phongBanId = $pbB; maDong = ""; tieuDe = "thieu ma" }) }
  Chk "canbo nhap 2 dong (1 loi)" ($r.json.them -eq 1 -and @($r.json.loi).Count -eq 1) ("them=" + $r.json.them + " loi=" + @($r.json.loi).Count)
  $r = Call POST "/danh-muc-ho-so/$($dm.id)/trinh" $tc; Chk "canbo trinh" ($r.json.trang_thai -eq "cho_duyet") ("tt=" + $r.json.trang_thai)
  $r = Call POST "/danh-muc-ho-so/$($dm.id)/duyet" $tc; Chk "canbo tu duyet bi chan (409)" ($r.code -eq 409) ("code=" + $r.code)
  $r = Call POST "/danh-muc-ho-so/$($dm.id)/duyet" $qt; Chk "quantri duyet" ($r.json.trang_thai -eq "da_duyet") ("tt=" + $r.json.trang_thai)
  $r = Call POST "/danh-muc-ho-so/$($dm.id)/ban-hanh" $qt @{ soQuyetDinh = "12/QD-UBND"; ngayBanHanh = "2026-01-15" }; Chk "ban hanh" ($r.json.trang_thai -eq "ban_hanh") ("tt=" + $r.json.trang_thai)
  $r = Call GET "/danh-muc-ho-so/hieu-luc?phongId=$phong&nam=2026" $ta; Chk "nvA thay dong hieu luc cua phong A" (@($r.json.rows).Count -eq 1) ("rows=" + @($r.json.rows).Count)
  $r = Call POST "/danh-muc-ho-so/$($dm.id)/bien-ban" $tc; Chk "ban in danh muc (DMH)" ($r.json.soBienBan -match "BB-DMH") ("so=" + $r.json.soBienBan)
  $r = Call GET "/danh-muc-ho-so/$($dm.id)/doi-chieu" $tc; Chk "doi chieu co 2 dong, 3 ho so ngoai danh muc" (@($r.json.theoDanhMuc).Count -eq 2 -and @($r.json.ngoaiDanhMuc).Count -ge 0) ("theo=" + @($r.json.theoDanhMuc).Count + " ngoai=" + @($r.json.ngoaiDanhMuc).Count)
} finally {
  if ($p -and -not $p.HasExited) { $p.Kill(); $p.WaitForExit() }
  Write-Host "API da dung"
}
