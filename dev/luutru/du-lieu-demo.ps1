# Dựng bộ dữ liệu DEMO lên một máy chủ DigiText Lưu trữ ĐÃ CÀI (cai-dat.ps1) — chạy TRÊN MÁY CHỦ, không xoá gì đã có.
# Gồm: đổi tên phông nhập từ gói AIP, thêm phông thứ hai (2 phòng ban, 14 hồ sơ, văn bản, kho / giá / hộp), 5 người dùng mẫu,
# cấu hình cơ quan lưu trữ, lịch sử phông, danh mục hồ sơ (1 ban hành, 1 chờ duyệt), nộp lưu (nhận / trả lại / chờ / quá hạn),
# mượn – trả (quá hạn, chờ duyệt, đã trả), kế hoạch kiểm tra đã chạy, sao lưu, phương tiện, sự cố, yêu cầu khai thác,
# cung cấp thông tin, chuyển đổi ra giấy, phạm vi phông cho độc giả, quét trùng.
# Nên nhập gói AIP trước (nhap-goi.ps1) để có tài liệu thật cho tra cứu / khai thác; không có gói thì vẫn chạy được.
#   powershell -ExecutionPolicy Bypass -File .\du-lieu-demo.ps1 -MatKhauQuanTriTep conf\quantri-mat-khau-lan-dau.txt
# Chạy lần hai: thấy phông mẫu đã có thì dừng, không nhân đôi dữ liệu.
param(
  [string]$Goc = "",                       # thư mục cài (mặc định: thư mục chứa script này)
  [string]$DiaChi = "",                    # https://... (mặc định đọc DIA_CHI trong conf\may-chu.env)
  [string]$QuanTri = "",                   # mặc định QUAN_TRI trong conf\may-chu.env
  [string]$MatKhauQuanTri = "",
  [string]$MatKhauQuanTriTep = "",
  [string]$MatKhauDemo = "DemoLuuTru2026"  # mật khẩu chung cho 5 tài khoản mẫu
)
$ErrorActionPreference = "Continue"
if (-not $Goc) { $Goc = $PSScriptRoot }
function Doc-Env($tep) {
  $kv = @{}
  if (Test-Path $tep) { Get-Content $tep -Encoding UTF8 | ForEach-Object { if ($_ -match '^\s*([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.*)$') { $kv[$Matches[1]] = $Matches[2].Trim().Trim('"') } } }
  return $kv
}
$mc = Doc-Env (Join-Path $Goc "conf\may-chu.env"); $apiEnv = Doc-Env (Join-Path $Goc "conf\api.env")
if (-not $apiEnv["DATABASE_URL"]) { Write-Host "Thiếu conf\api.env — chạy cai-dat.ps1 trước." -ForegroundColor Red; exit 1 }
if (-not $DiaChi) { $DiaChi = "https://" + $mc["DIA_CHI"] }
if (-not $QuanTri) { $QuanTri = $mc["QUAN_TRI"] }; if (-not $QuanTri) { $QuanTri = "quantri" }
if (-not $MatKhauQuanTri -and $MatKhauQuanTriTep) { $MatKhauQuanTri = (Get-Content $MatKhauQuanTriTep -Raw -Encoding UTF8).Trim() }
if (-not $MatKhauQuanTri) { $MatKhauQuanTri = Read-Host "Mật khẩu tài khoản $QuanTri" }
$psql = Join-Path $mc["PG_BIN"] "psql.exe"
$dbUrl = $apiEnv["DATABASE_URL"]
$script:ApiBase = "$DiaChi/api/v1"   # không đặt tên $B: hàm gọi có biến $b (thân yêu cầu) sẽ che mất vì PowerShell không phân biệt hoa thường
$utf8 = New-Object Text.UTF8Encoding($false)
$script:loi = 0

function Sql($q) {
  $t = Join-Path $env:TEMP ("dt-demo-" + [guid]::NewGuid().ToString("N") + ".sql")
  [IO.File]::WriteAllText($t, $q, $utf8)
  $env:PGCLIENTENCODING = "UTF8"
  $o = & $psql -X -q -t -A -v ON_ERROR_STOP=1 -d $dbUrl -f $t 2>&1
  if ($LASTEXITCODE -ne 0) { $script:loi++; Write-Host ("  SQL LỖI: " + ($o -join " ")) -ForegroundColor Red }
  Remove-Item $t -Force
  return ($o | ForEach-Object { "$_" } | Where-Object { $_ -ne "" })
}
# HTTP qua curl.exe (Invoke-WebRequest của PowerShell 5.1 không bắt tay được TLS với Caddy chứng chỉ nội bộ).
function Call($m, $path, $tok, $body) {
  # KHÔNG đặt tên $args — đó là biến tự động của PowerShell, gán vào nó rồi splat ra mảng rỗng.
  $cu = @("-s", "-k", "-X", $m, "$($script:ApiBase)$path", "-w", "`n__CODE__%{http_code}")
  if ($tok) { $cu += @("-H", "Authorization: Bearer $tok") }
  $t = $null
  if ($null -ne $body) {
    $t = Join-Path $env:TEMP ("dt-demo-" + [guid]::NewGuid().ToString("N") + ".json")
    [IO.File]::WriteAllText($t, ($body | ConvertTo-Json -Depth 8 -Compress), $utf8)
    $cu += @("-H", "Content-Type: application/json; charset=utf-8", "--data-binary", "@$t")
  }
  $out = & curl.exe @cu 2>&1; $exit = $LASTEXITCODE
  $raw = ($out | ForEach-Object { "$_" }) -join "`n"
  if ($t) { Remove-Item $t -Force }
  $i = $raw.LastIndexOf("__CODE__")
  $code = 0; $txt = $raw
  if ($i -ge 0) { $code = [int]$raw.Substring($i + 8); $txt = $raw.Substring(0, $i).TrimEnd("`n") } else { $txt = "curl exit $exit : $raw" }
  if ($code -eq 0) { Write-Host ("    [curl exit $exit] " + ($cu -join " ") + " => " + $raw) -ForegroundColor DarkGray }
  $json = $null; if ($txt) { try { $json = $txt | ConvertFrom-Json } catch { $json = $txt } }
  return @{ code = $code; json = $json; raw = $txt }
}
function Buoc($ten, $r, $mong) {
  if ($null -eq $mong) { $mong = @(200, 201) }
  if ($r.code -in $mong) { Write-Host ("  OK   " + $ten) -ForegroundColor Green }
  else { $script:loi++; $th = [string]$r.raw; if ($th.Length -gt 200) { $th = $th.Substring(0, 200) }; Write-Host ("  LỖI  " + $ten + " (HTTP " + $r.code + ") " + $th) -ForegroundColor Red }
  return $r
}
function Dang-Nhap($u, $p) {
  $r = Call POST "/auth/login" $null @{ username = $u; password = $p }
  if ($r.code -ne 200) { Write-Host "Không đăng nhập được $u (HTTP $($r.code)) $($r.raw)" -ForegroundColor Red; exit 1 }
  return $r.json.accessToken
}
function Ma-Moi { [guid]::NewGuid().ToString() }
$homNay = Get-Date

Write-Host "===== DỮ LIỆU DEMO → $DiaChi" -ForegroundColor Cyan
$P = (Sql "SELECT id FROM projects WHERE code = '$($mc["MA_DU_AN"])'") | Select-Object -First 1
if (-not $P) { $P = (Sql "SELECT id FROM projects ORDER BY created_at LIMIT 1") | Select-Object -First 1 }
if (-not $P) { Write-Host "Chưa có kho lưu trữ (projects) nào." -ForegroundColor Red; exit 1 }
function Co($q) { return ((Sql $q) | Select-Object -First 1) -ne "0" }
function Mot($q) { return (Sql $q) | Select-Object -First 1 }
$daCo = Co "SELECT count(*) FROM cl_phong WHERE project_id = '$P' AND ma_du_an = 'PNV-HH'"
if ($daCo) { Write-Host "Phông mẫu 'PNV-HH' đã có — chỉ bổ sung phần còn thiếu." -ForegroundColor Yellow }
$qt = Dang-Nhap $QuanTri $MatKhauQuanTri
$QID = (Sql "SELECT id FROM users WHERE username = '$QuanTri'") | Select-Object -First 1

# ---------- 1. Phông nhập từ gói AIP: đặt tên đọc được, bổ sung chế độ sử dụng / THBQ còn trống
Write-Host "===== 1. phông nhập từ gói AIP"
$PH1 = (Sql "SELECT id FROM cl_phong WHERE project_id = '$P' AND deleted_at IS NULL ORDER BY created_at LIMIT 1") | Select-Object -First 1
if ($PH1) {
  Sql "UPDATE cl_phong SET ten = 'UBND xã Diễn Xá', phong_so = COALESCE(phong_so, '01') WHERE id = '$PH1' AND ten = ma_du_an" | Out-Null
  Sql "UPDATE cl_ho_so SET che_do_su_dung = COALESCE(che_do_su_dung, 'Công khai'), thbq_id = COALESCE(thbq_id, '00000000-0000-4000-8000-00000c010001') WHERE phong_id = '$PH1'" | Out-Null
  Write-Host ("  phông " + (Sql "SELECT ten FROM cl_phong WHERE id = '$PH1'") + ": " + (Sql "SELECT count(*) FROM cl_ho_so WHERE phong_id = '$PH1'") + " hồ sơ, " + (Sql "SELECT count(*) FROM cl_van_ban v JOIN cl_ho_so h ON h.id = v.ho_so_id WHERE h.phong_id = '$PH1'") + " văn bản")
} else { Write-Host "  (chưa nhập gói AIP nào — bỏ qua)" -ForegroundColor Yellow }

# ---------- 2. Phông thứ hai: phòng ban, kho / giá, hồ sơ, văn bản
Write-Host "===== 2. phông 'Phòng Nội vụ huyện Hiệp Hoà': 2 phòng ban, 14 hồ sơ, 12 văn bản, kho / giá"
$PH2 = Ma-Moi; $VP = Ma-Moi; $TC = Ma-Moi; $KHO = Ma-Moi; $GIA1 = Ma-Moi; $GIA2 = Ma-Moi
if ($daCo) {
  $PH2 = Mot "SELECT id FROM cl_phong WHERE project_id = '$P' AND ma_du_an = 'PNV-HH'"
  $VP = Mot "SELECT id FROM cl_phong_ban WHERE phong_id = '$PH2' AND ma_don_vi = 'VP'"; $TC = Mot "SELECT id FROM cl_phong_ban WHERE phong_id = '$PH2' AND ma_don_vi = 'TCBC'"
  $KHO = Mot "SELECT id FROM cl_kho WHERE project_id = '$P' AND ten = 'Kho lưu trữ số 1'"
  $GIA1 = Mot "SELECT id FROM cl_gia WHERE kho_id = '$KHO' AND ten = 'Giá 01'"; $GIA2 = Mot "SELECT id FROM cl_gia WHERE kho_id = '$KHO' AND ten = 'Giá 02'"
} else {
Sql @"
INSERT INTO cl_phong (id, project_id, ma_du_an, ten, phong_so, muc_luc_so, created_by) VALUES ('$PH2', '$P', 'PNV-HH', 'Phòng Nội vụ huyện Hiệp Hoà', '02', '01', '$QID');
INSERT INTO cl_phong_ban (id, project_id, phong_id, ma_don_vi, ten, nhiem_ky, so_tt, created_by) VALUES
  ('$VP', '$P', '$PH2', 'VP', 'Văn phòng', '2021-2026', 1, '$QID'),
  ('$TC', '$P', '$PH2', 'TCBC', 'Bộ phận Tổ chức - Biên chế', '2021-2026', 2, '$QID');
INSERT INTO cl_kho (id, project_id, ten, ghi_chu, so_tt, suc_chua_hop, created_by) VALUES ('$KHO', '$P', 'Kho lưu trữ số 1', 'Tầng 2, nhà B', 1, 500, '$QID');
INSERT INTO cl_gia (id, project_id, kho_id, phong_ban_id, ten, tu_hop, den_hop, suc_chua_hop, canh_bao_pct, so_tt, created_by) VALUES
  ('$GIA1', '$P', '$KHO', '$VP', 'Giá 01', 1, 10, 10, 80, 1, '$QID'),
  ('$GIA2', '$P', '$KHO', '$TC', 'Giá 02', 11, 20, 10, 90, 2, '$QID');
"@ | Out-Null
}
$VV = '00000000-0000-4000-8000-00000c010001'; $N5 = '00000000-0000-4000-8000-00000c010002'; $N10 = '00000000-0000-4000-8000-00000c010003'; $N20 = '00000000-0000-4000-8000-00000c010005'
# so_tam, phòng ban, tiêu đề, ngày bắt đầu, ngày kết thúc, THBQ, số tờ, chế độ sử dụng
$dsHs = @(
  @("01", $VP, "Tập lưu Quyết định của UBND huyện năm 2023", "02/01/2023", "29/12/2023", $VV, 180, "Công khai"),
  @("02", $VP, "Tập lưu Công văn đi của UBND huyện quý I năm 2024", "02/01/2024", "29/03/2024", $N10, 120, "Công khai"),
  @("03", $VP, "Hồ sơ Hội nghị tổng kết công tác năm 2023, triển khai nhiệm vụ năm 2024", "15/12/2023", "20/01/2024", $N20, 45, "Công khai"),
  @("04", $VP, "Hồ sơ xây dựng Quy chế làm việc của UBND huyện nhiệm kỳ 2021-2026", "03/2024", "06/2024", $VV, 60, "Công khai"),
  @("05", $VP, "Tập lưu Báo cáo tháng của Văn phòng năm 2025", "01/2025", "12/2025", $N5, 96, "Công khai"),
  @("06", $VP, "Hồ sơ tiếp công dân, giải quyết đơn thư năm 2025", "01/2025", "12/2025", $N10, 210, "Hạn chế"),
  @("07", $VP, "Tập lưu Thông báo kết luận của Chủ tịch UBND huyện năm 2025", "2025", "2025", $N20, 75, "Công khai"),
  @("08", $VP, "Hồ sơ Kế hoạch cải cách hành chính năm 2026", "01/2026", "2026", $N10, 30, "Công khai"),
  @("09", $TC, "Hồ sơ tuyển dụng viên chức sự nghiệp giáo dục năm 2023", "2023", "2023", $VV, 320, "Hạn chế"),
  @("10", $TC, "Hồ sơ nâng bậc lương thường xuyên năm 2024", "2024", "2024", $N20, 150, "Công khai"),
  @("11", $TC, "Hồ sơ thi đua, khen thưởng năm 2025", "2025", "2025", $N20, 88, "Công khai"),
  @("12", $TC, "Hồ sơ bổ nhiệm, bổ nhiệm lại cán bộ quản lý năm 2025", "2025", "2025", $VV, 140, "Hạn chế"),
  @("13", $TC, "Tập lưu Quyết định về công tác cán bộ năm 2024", "2024", "2024", $VV, 200, "Công khai"),
  @("14", $TC, "Hồ sơ đào tạo, bồi dưỡng cán bộ, công chức năm 2026", "2026", "2026", $N10, 25, "Công khai")
)
$HS = @{}; $sqlHs = ""
foreach ($h in $dsHs) {
  if ($daCo) { $HS[$h[0]] = Mot "SELECT id FROM cl_ho_so WHERE phong_id = '$PH2' AND so_tam = '$($h[0])'"; continue }
  $id = Ma-Moi; $HS[$h[0]] = $id
  $tieuDeSql = $h[2] -replace "'", "''"
  $sqlHs += "INSERT INTO cl_ho_so (id, project_id, phong_id, phong_ban_id, so_tam, tieu_de, ngay_bat_dau, ngay_ket_thuc, thbq_id, so_to, che_do_su_dung, ngon_ngu, tinh_trang_vat_ly, muc_do_tin_cay, created_by) VALUES ('$id', '$P', '$PH2', '$($h[1])', '$($h[0])', '$tokDocGia', '$($h[3])', '$($h[4])', '$($h[5])', '$($h[6])', '$($h[7])', 'Tiếng Việt', 'Bình thường', 'Bản chính', '$QID');`n"
}
if ($sqlHs) { Sql $sqlHs | Out-Null }
# văn bản mẫu cho 4 hồ sơ (loai_vb '02' = Quyết định theo mã TT05)
$dsVb = @(
  @("01", 1, "02", "01/QĐ-UBND", "05/01/2023", "UBND huyện Hiệp Hoà", "Nguyễn Văn Bình", "Quyết định ban hành Chương trình công tác năm 2023", 4),
  @("01", 2, "02", "15/QĐ-UBND", "20/02/2023", "UBND huyện Hiệp Hoà", "Nguyễn Văn Bình", "Quyết định phê duyệt kế hoạch sử dụng đất năm 2023", 6),
  @("01", 3, "02", "128/QĐ-UBND", "29/12/2023", "UBND huyện Hiệp Hoà", "Trần Văn Cường", "Quyết định khen thưởng tập thể, cá nhân có thành tích năm 2023", 3),
  @("02", 1, $null, "12/UBND-VP", "08/01/2024", "UBND huyện Hiệp Hoà", "Lê Thị Hoa", "V/v triển khai nhiệm vụ trọng tâm quý I năm 2024", 2),
  @("02", 2, $null, "45/UBND-NV", "14/02/2024", "UBND huyện Hiệp Hoà", "Lê Thị Hoa", "V/v rà soát biên chế công chức cấp xã", 2),
  @("02", 3, $null, "97/UBND-VP", "29/03/2024", "UBND huyện Hiệp Hoà", "Trần Văn Cường", "V/v báo cáo kết quả công tác quý I năm 2024", 3),
  @("09", 1, "02", "210/QĐ-UBND", "10/06/2023", "UBND huyện Hiệp Hoà", "Nguyễn Văn Bình", "Quyết định phê duyệt kế hoạch tuyển dụng viên chức sự nghiệp giáo dục", 5),
  @("09", 2, $null, "01/TB-HĐTD", "15/06/2023", "Hội đồng tuyển dụng", "Phạm Văn Dũng", "Thông báo tuyển dụng viên chức sự nghiệp giáo dục năm 2023", 3),
  @("09", 3, "02", "398/QĐ-UBND", "30/09/2023", "UBND huyện Hiệp Hoà", "Nguyễn Văn Bình", "Quyết định công nhận kết quả tuyển dụng viên chức", 8),
  @("13", 1, "02", "22/QĐ-UBND", "16/01/2024", "UBND huyện Hiệp Hoà", "Nguyễn Văn Bình", "Quyết định điều động công chức", 2),
  @("13", 2, "02", "156/QĐ-UBND", "02/05/2024", "UBND huyện Hiệp Hoà", "Nguyễn Văn Bình", "Quyết định bổ nhiệm Phó Trưởng phòng", 2),
  @("13", 3, "02", "301/QĐ-UBND", "18/11/2024", "UBND huyện Hiệp Hoà", "Trần Văn Cường", "Quyết định nghỉ hưu hưởng chế độ bảo hiểm xã hội", 2)
)
$sqlVb = ""
foreach ($v in $dsVb) {
  if ($daCo) { break }
  $lv = $(if ($v[2]) { "'$($v[2])'" } else { "NULL" }); $ty = $v[7] -replace "'", "''"
  $sqlVb += "INSERT INTO cl_van_ban (id, project_id, ho_so_id, so_thu_tu, loai_vb, so_ky_hieu, ngay_thang, co_quan_ban_hanh, nguoi_ky, trich_yeu, so_trang, ngon_ngu, do_tin_cay, created_by) VALUES ('$(Ma-Moi)', '$P', '$($HS[$v[0]])', $($v[1]), $lv, '$($v[3])', '$($v[4])', '$($v[5])', '$($v[6])', '$ty', $($v[8]), 'Tiếng Việt', 'Bản chính', '$QID');`n"
}
if ($sqlVb) { Sql $sqlVb | Out-Null }
Sql "UPDATE cl_ho_so h SET tong_so_vb = (SELECT count(*) FROM cl_van_ban v WHERE v.ho_so_id = h.id) WHERE h.phong_id = '$PH2'" | Out-Null
Write-Host ("  hồ sơ: " + (Sql "SELECT count(*) FROM cl_ho_so WHERE phong_id = '$PH2'") + " · văn bản: " + (Sql "SELECT count(*) FROM cl_van_ban v JOIN cl_ho_so h ON h.id = v.ho_so_id WHERE h.phong_id = '$PH2'"))

# ---------- 3. Người dùng mẫu
Write-Host "===== 3. người dùng mẫu (mật khẩu chung: $MatKhauDemo)"
function Tao-User($u, $ten, $role, $roleCode, $pb) {
  if ((Sql "SELECT count(*) FROM users WHERE username = '$u'") -ne "0") { Write-Host "  đã có $u"; return }
  $b = @{ username = $u; password = $MatKhauDemo; displayName = $ten; role = $role; roleCode = $roleCode; projectId = $P }
  if ($pb) { $b.phongBanId = $pb }
  Buoc "tạo $u ($ten, $roleCode)" (Call POST "/users" $qt $b) | Out-Null
}
Tao-User "canbo"   "Trần Thị Lan"    "admin"    "can_bo_luu_tru"      $null
Tao-User "lanhdao" "Nguyễn Văn Bình" "manager"  "lanh_dao"            $null
Tao-User "nvvp"    "Lê Thị Hoa"      "user"     "nhan_vien_phong_ban" $VP
Tao-User "nvtc"    "Phạm Văn Dũng"   "user"     "nhan_vien_phong_ban" $TC
Tao-User "docgia"  "Hoàng Minh Tuấn" "customer" "customer"            $null
# Tên token dài, không trùng (hoa / thường) với biến dữ liệu: PowerShell coi $tokCanBo và $TC là một.
$tokCanBo = Dang-Nhap "canbo" $MatKhauDemo; $tokLanhDao = Dang-Nhap "lanhdao" $MatKhauDemo
$tokVp = Dang-Nhap "nvvp" $MatKhauDemo; $tokTc = Dang-Nhap "nvtc" $MatKhauDemo; $tokDocGia = Dang-Nhap "docgia" $MatKhauDemo
$idDocGia = (Sql "SELECT id FROM users WHERE username = 'docgia'") | Select-Object -First 1
Sql "UPDATE cl_ho_so SET created_by = (SELECT id FROM users WHERE username = 'nvvp') WHERE phong_ban_id = '$VP'; UPDATE cl_ho_so SET created_by = (SELECT id FROM users WHERE username = 'nvtc') WHERE phong_ban_id = '$TC'" | Out-Null

# ---------- 4. Cơ quan lưu trữ + lịch sử phông
Write-Host "===== 4. cơ quan lưu trữ, lịch sử phông"
Buoc "cấu hình cơ quan lưu trữ" (Call PUT "/projects/$P/co-quan" $tokCanBo @{ maCoQuan = "H49.132"; ten = "Lưu trữ cơ quan UBND huyện Hiệp Hoà"; tenCapTren = "UBND tỉnh Bắc Giang"; diaChi = "Thị trấn Thắng, huyện Hiệp Hoà, tỉnh Bắc Giang"; dienThoai = "0204 3 872 xxx"; email = "luutru.hiephoa@bacgiang.gov.vn"; loaiHinh = "luu_tru_co_quan"; nguoiKy = "Nguyễn Văn Bình"; chucVuNguoiKy = "Chánh Văn phòng"; hanNopNgay = 60; nhacTruocNgay = @(15, 7, 1); quyenSuDungNgay = 360 }) | Out-Null
if ($PH1 -and -not (Co "SELECT count(*) FROM phong_lich_su WHERE phong_id = '$PH1'")) {
  Buoc "lịch sử phông Diễn Xá v1" (Call POST "/cl/phong/$PH1/lich-su" $tokCanBo @{ lichSuDonVi = "UBND xã Diễn Xá được thành lập năm 1955 trên cơ sở xã Diễn Xá cũ; là cơ quan hành chính nhà nước cấp xã thuộc huyện Hiệp Hoà."; lichSuPhong = "Tài liệu từ 2012 đến 2017 được chỉnh lý, số hoá năm 2026 và nộp vào lưu trữ."; thoiGianTaiLieu = "2012-2017"; tongSoTaiLieu = "3 hồ sơ, 150 văn bản"; phuongAnPhanLoai = "Thời gian - Mặt hoạt động"; ngonNgu = "Tiếng Việt"; trangThaiPhong = "mo"; nguon = "Thu nộp đợt 1 năm 2026" }) | Out-Null
  Buoc "lịch sử phông Diễn Xá v2" (Call POST "/cl/phong/$PH1/lich-su" $tokCanBo @{ lichSuDonVi = "UBND xã Diễn Xá được thành lập năm 1955; từ 01/7/2025 sáp nhập vào xã mới theo Nghị quyết sắp xếp đơn vị hành chính."; lichSuPhong = "Phông đóng: đơn vị hình thành phông chấm dứt hoạt động; tài liệu 2012-2017 đã nộp lưu đợt 1, phần còn lại nộp bổ sung quý IV/2026."; thoiGianTaiLieu = "2012-2025"; tongSoTaiLieu = "3 hồ sơ, 150 văn bản (đợt 1)"; phuongAnPhanLoai = "Thời gian - Mặt hoạt động"; ngonNgu = "Tiếng Việt"; trangThaiPhong = "dong"; nguon = "Nộp bổ sung đợt 2 năm 2026" }) | Out-Null
}
if (-not (Co "SELECT count(*) FROM phong_lich_su WHERE phong_id = '$PH2'")) { Buoc "lịch sử phông Nội vụ v1" (Call POST "/cl/phong/$PH2/lich-su" $tokCanBo @{ lichSuDonVi = "Phòng Nội vụ huyện Hiệp Hoà là cơ quan chuyên môn thuộc UBND huyện, tham mưu về tổ chức bộ máy, biên chế, cán bộ công chức, thi đua khen thưởng, văn thư lưu trữ."; lichSuPhong = "Tài liệu hình thành trong hoạt động từ 2023; lập hồ sơ điện tử và nộp lưu định kỳ theo Điều 19 TT 05/2025/TT-BNV."; thoiGianTaiLieu = "2023-2026"; phuongAnPhanLoai = "Thời gian - Cơ cấu tổ chức"; ngonNgu = "Tiếng Việt"; trangThaiPhong = "mo" }) | Out-Null }

# ---------- 5. Danh mục hồ sơ
Write-Host "===== 5. danh mục hồ sơ 2026 (ban hành) + 2025 (chờ duyệt)"
$DM26 = $null
Sql "DELETE FROM lt_danh_muc d WHERE d.phong_id = '$PH2' AND NOT EXISTS (SELECT 1 FROM lt_danh_muc_dong x WHERE x.danh_muc_id = d.id)" | Out-Null
if (-not (Co "SELECT count(*) FROM lt_danh_muc WHERE phong_id = '$PH2' AND nam = 2026")) { $r = Buoc "lập danh mục 2026" (Call POST "/danh-muc-ho-so" $tokCanBo @{ projectId = $P; phongId = $PH2; nam = 2026 }); $DM26 = $r.json.id }
if ($DM26) {
  Buoc "nhập 5 dòng danh mục" (Call POST "/danh-muc-ho-so/$DM26/dong/nhap" $tokCanBo @{ rows = @(
        @{ phongBanId = $VP; maDong = "VP.01"; tieuDe = "Tập lưu Quyết định của UBND huyện"; thbqId = $VV },
        @{ phongBanId = $VP; maDong = "VP.02"; tieuDe = "Tập lưu Công văn đi, đến"; thbqId = $N10 },
        @{ phongBanId = $VP; maDong = "VP.03"; tieuDe = "Hồ sơ Kế hoạch cải cách hành chính"; thbqId = $N10 },
        @{ phongBanId = $TC; maDong = "TC.01"; tieuDe = "Hồ sơ tuyển dụng, bổ nhiệm cán bộ, công chức, viên chức"; thbqId = $VV },
        @{ phongBanId = $TC; maDong = "TC.02"; tieuDe = "Hồ sơ đào tạo, bồi dưỡng; thi đua khen thưởng"; thbqId = $N20 }) }) | Out-Null
  Buoc "trình danh mục 2026" (Call POST "/danh-muc-ho-so/$DM26/trinh" $tokCanBo) | Out-Null
  Buoc "lãnh đạo duyệt" (Call POST "/danh-muc-ho-so/$DM26/duyet" $tokLanhDao) | Out-Null
  Buoc "ban hành 05/QĐ-PNV ngày 10/01/2026" (Call POST "/danh-muc-ho-so/$DM26/ban-hanh" $tokLanhDao @{ soQuyetDinh = "05/QĐ-PNV"; ngayBanHanh = "2026-01-10" }) | Out-Null
  $dongHl = (Call GET "/danh-muc-ho-so/hieu-luc?phongId=$PH2&nam=2026" $tokCanBo).json.rows
  $d08 = @($dongHl | Where-Object { $_.ma_dong -eq "VP.03" })[0]; $d14 = @($dongHl | Where-Object { $_.ma_dong -eq "TC.02" })[0]
  if ($d08) { Sql "UPDATE cl_ho_so SET dm_dong_id = '$($d08.id)' WHERE id = '$($HS['08'])'" | Out-Null }
  if ($d14) { Sql "UPDATE cl_ho_so SET dm_dong_id = '$($d14.id)' WHERE id = '$($HS['14'])'" | Out-Null }
}
$r = @{ json = $null }
if (-not (Co "SELECT count(*) FROM lt_danh_muc WHERE phong_id = '$PH2' AND nam = 2025")) { $r = Buoc "lập danh mục 2025 (để demo duyệt)" (Call POST "/danh-muc-ho-so" $tokCanBo @{ projectId = $P; phongId = $PH2; nam = 2025 }) }
if ($r.json.id) {
  Call POST "/danh-muc-ho-so/$($r.json.id)/dong/nhap" $tokCanBo @{ rows = @(@{ phongBanId = $VP; maDong = "VP.01"; tieuDe = "Tập lưu Báo cáo tháng"; thbqId = $N5 }, @{ phongBanId = $TC; maDong = "TC.01"; tieuDe = "Hồ sơ thi đua, khen thưởng"; thbqId = $N20 }) } | Out-Null
  Buoc "trình danh mục 2025 → chờ duyệt" (Call POST "/danh-muc-ho-so/$($r.json.id)/trinh" $tokCanBo) | Out-Null
}

# ---------- 6. Nộp lưu
Write-Host "===== 6. nộp lưu: 1 phiếu đã tiếp nhận (1 nhận, 1 trả lại), 1 phiếu chờ, hồ sơ 07 / 12 quá hạn chưa nộp"
$ph = $null
if (-not (Co "SELECT count(*) FROM lt_nop_luu WHERE project_id = '$P'")) {
$r = Buoc "nvvp nộp lưu hồ sơ 05, 06" (Call POST "/nop-luu" $tokVp @{ projectId = $P; hoSoIds = @($HS["05"], $HS["06"]) })
$ph = $r.json
if ($ph.id) {
  Buoc "cán bộ chấm trả lại hồ sơ 06" (Call POST "/nop-luu/$($ph.id)/ho-so/$($HS['06'])/ket-qua" $tokCanBo @{ ketQua = "tra_lai"; lyDo = "Thiếu mục lục văn bản và chứng từ kết thúc" }) | Out-Null
  Buoc "tiếp nhận phiếu (lập biên bản giao nhận)" (Call POST "/nop-luu/$($ph.id)/tiep-nhan" $tokCanBo @{ ghiChu = "Nhận 1 hồ sơ, trả lại 1 hồ sơ để hoàn thiện" }) | Out-Null
}
Buoc "nvtc nộp lưu hồ sơ 11 (chờ tiếp nhận)" (Call POST "/nop-luu" $tokTc @{ projectId = $P; hoSoIds = @($HS["11"]) }) | Out-Null
}

# ---------- 7. Kho / giá / hộp
Write-Host "===== 7. gán hộp: hồ sơ 01-04 vào hộp 1-2 (Giá 01), 09-10, 13 vào hộp 11 (Giá 02)"
if (-not (Co "SELECT count(*) FROM cl_ho_so WHERE id = '$($HS['01'])' AND hop_id IS NOT NULL")) {
Buoc "gán hộp Văn phòng" (Call POST "/cl/ho-so/gan-hop" $tokCanBo @{ projectId = $P; phongBanId = $VP; rows = @(@{ hoSoId = $HS["01"]; hopSo = "1" }, @{ hoSoId = $HS["02"]; hopSo = "1" }, @{ hoSoId = $HS["03"]; hopSo = "2" }, @{ hoSoId = $HS["04"]; hopSo = "2" }) }) | Out-Null
}
if (-not (Co "SELECT count(*) FROM cl_ho_so WHERE id = '$($HS['09'])' AND hop_id IS NOT NULL")) {
Buoc "gán hộp Tổ chức" (Call POST "/cl/ho-so/gan-hop" $tokCanBo @{ projectId = $P; phongBanId = $TC; rows = @(@{ hoSoId = $HS["09"]; hopSo = "11" }, @{ hoSoId = $HS["10"]; hopSo = "11" }, @{ hoSoId = $HS["13"]; hopSo = "12" }) }) | Out-Null
}

# ---------- 8. Mượn – trả
Write-Host "===== 8. mượn – trả: phiếu quá hạn, phiếu chờ duyệt, phiếu đã trả"
if (-not (Co "SELECT count(*) FROM cl_muon_tra WHERE ho_so_id = '$($HS['01'])'")) {
$r = Buoc "phiếu 1: mượn hồ sơ 01, 02" (Call POST "/cl/muon-tra" $tokCanBo @{ projectId = $P; hoSoIds = @($HS["01"], $HS["02"]); nguoiMuon = "Nguyễn Thị Mai"; donVi = "Phòng Tài chính - Kế hoạch"; noiDung = "Tra cứu quyết định phân bổ dự toán năm 2023"; ngayDuKienTra = $homNay.AddDays(10).ToString("yyyy-MM-dd") })
$p1 = $r.json
if ($p1.nhomId) {
  Buoc "lãnh đạo duyệt cả phiếu 1" (Call POST "/cl/muon-tra/nhom/$($p1.nhomId)/duyet" $tokLanhDao) | Out-Null
  foreach ($d in @($p1.rows)) { Buoc "giao hồ sơ $($d.ho_so_id.Substring(0,8))" (Call POST "/cl/muon-tra/$($d.id)/giao" $tokCanBo @{ ngay = $homNay.AddDays(-20).ToString("yyyy-MM-dd") }) | Out-Null }
  Sql "UPDATE cl_muon_tra SET ngay_muon = CURRENT_DATE - 20, ngay_du_kien_tra = CURRENT_DATE - 5 WHERE nhom_id = '$($p1.nhomId)'" | Out-Null
}
if (-not (Co "SELECT count(*) FROM cl_muon_tra WHERE ho_so_id = '$($HS['09'])'")) { Buoc "phiếu 2: chờ duyệt (hồ sơ 09)" (Call POST "/cl/muon-tra" $tokCanBo @{ projectId = $P; hoSoIds = @($HS["09"]); nguoiMuon = "Vũ Đức Long"; donVi = "Thanh tra huyện"; noiDung = "Xác minh hồ sơ tuyển dụng theo đơn phản ánh"; ngayDuKienTra = $homNay.AddDays(10).ToString("yyyy-MM-dd") }) | Out-Null }
$r = @{ json = $null }
if (-not (Co "SELECT count(*) FROM cl_muon_tra WHERE ho_so_id = '$($HS['03'])'")) { $r = Buoc "phiếu 3: hồ sơ 03" (Call POST "/cl/muon-tra" $tokCanBo @{ projectId = $P; hoSoIds = @($HS["03"]); nguoiMuon = "Đặng Thu Hà"; donVi = "Văn phòng HĐND-UBND"; noiDung = "Chuẩn bị hội nghị tổng kết"; ngayDuKienTra = $homNay.AddDays(3).ToString("yyyy-MM-dd") }) }
$p3 = $r.json
if ($p3.rows) {
  $d3 = @($p3.rows)[0]
  Buoc "duyệt phiếu 3" (Call POST "/cl/muon-tra/$($d3.id)/duyet" $tokLanhDao) | Out-Null
  Buoc "giao phiếu 3" (Call POST "/cl/muon-tra/$($d3.id)/giao" $tokCanBo @{ ngay = $homNay.AddDays(-4).ToString("yyyy-MM-dd") }) | Out-Null
  Buoc "trả phiếu 3 (trả sớm)" (Call POST "/cl/muon-tra/$($d3.id)/tra" $tokCanBo @{ ngay = $homNay.AddDays(-1).ToString("yyyy-MM-dd") }) | Out-Null
}
}

# ---------- 9. Bảo quản
Write-Host "===== 9. bảo quản: kế hoạch kiểm tra đã chạy, sao lưu, phương tiện, sự cố"
$kh = $null
if (-not (Co "SELECT count(*) FROM bk_ke_hoach WHERE project_id = '$P'")) { $r = Buoc "lập kế hoạch kiểm tra 2026" (Call POST "/bao-quan/ke-hoach" $tokCanBo @{ projectId = $P; nam = 2026; ten = "Kiểm tra định kỳ tài liệu lưu trữ số năm 2026"; phamVi = @{ lauChuaKiem = $true; goi = $true } })
$kh = $r.json }
if ($kh.id) {
  Buoc "trình kế hoạch" (Call POST "/bao-quan/ke-hoach/$($kh.id)/trinh" $tokCanBo) | Out-Null
  Buoc "lãnh đạo duyệt kế hoạch" (Call POST "/bao-quan/ke-hoach/$($kh.id)/duyet" $tokLanhDao) | Out-Null
  $r = Buoc "máy chủ chạy kiểm tra" (Call POST "/bao-quan/ke-hoach/$($kh.id)/chay?limit=500" $tokCanBo)
  if ($r.json) { Write-Host ("       đã kiểm " + $r.json.daKiem + " · lỗi " + $r.json.loi + " · hoàn thành " + $r.json.hoanThanh) }
}
$runDay = $null
if (-not (Co "SELECT count(*) FROM backup_runs")) {
$r = Buoc "ghi nhận sao lưu đầy đủ CSDL đêm qua" (Call POST "/bao-quan/sao-luu" $tokCanBo @{ loai = "day_du"; nguon = "CSDL PostgreSQL (pg_dump)"; ketQua = "dat"; duongDan = "E:\DigiText-SaoLuu\csdl\digitext-$($homNay.AddDays(-1).ToString('yyyyMMdd'))-0100.dump"; dungLuongBytes = 73400320; maBam = ("3f" * 32) })
$runDay = $r.json
$lichGt = ((Call GET "/bao-quan/lich" $tokCanBo).json.rows | Where-Object { $_.loai -eq "gia_tang" } | Select-Object -First 1)
if ($lichGt) {
  Sql "INSERT INTO backup_runs (id, loai, nguon, ket_qua, lich_id, duong_dan, dung_luong_bytes, created_at) SELECT gen_random_uuid(), 'gia_tang', 'Kho tài liệu', 'dat', '$($lichGt.id)', 'E:\DigiText-SaoLuu\kho\' || to_char(now() - (g || ' day')::interval, 'YYYYMMDD'), 120000000 + g * 1000000, now() - (g || ' day')::interval + interval '1 hour' FROM generate_series(1, 7) g" | Out-Null
  Write-Host "  OK   7 bản sao lưu gia tăng 7 ngày qua"
}
}
if (-not (Co "SELECT count(*) FROM bk_phuong_tien")) {
Buoc "phương tiện: ổ cứng sao lưu (hạn 3 năm)" (Call POST "/bao-quan/phuong-tien" $tokCanBo @{ ma = "HDD-SL-01"; loai = "o_cung"; moTa = "WD Purple 4 TB - ổ sao lưu đêm"; dungLuongBytes = 4000000000000; noiBaoQuan = "Tủ sắt phòng máy chủ"; hanDoBen = $homNay.AddYears(3).ToString("yyyy-MM-dd") }) | Out-Null
Buoc "phương tiện: băng từ LTO (còn 200 ngày → sắp hết hạn)" (Call POST "/bao-quan/phuong-tien" $tokCanBo @{ ma = "LTO-2023-01"; loai = "bang_tu"; moTa = "Băng LTO-8 sao lưu 3 năm (ghi 11/2023)"; dungLuongBytes = 12000000000000; noiBaoQuan = "Kho lưu trữ số 1, két chống cháy"; hanDoBen = $homNay.AddDays(200).ToString("yyyy-MM-dd") }) | Out-Null
}
if (-not (Co "SELECT count(*) FROM bk_su_co")) {
$scBody = @{ moTa = "Ổ kho tài liệu báo lỗi sector, 3 tệp PDF không mở được"; nguyenNhan = "Lỗi phần cứng ổ cứng"; mucDo = "cao"; phamVi = "Kho tài liệu - phông UBND xã Diễn Xá"; bienPhap = "Cách ly ổ lỗi`nChép lại 3 tệp từ bản sao lưu đêm trước`nKiểm toàn vẹn lại toàn phông"; phuongPhapPhucHoi = "Khôi phục từ bản sao lưu gia tăng gần nhất" }
if ($runDay.id) { $scBody.backupRunId = $runDay.id }
$r = Buoc "sự cố 1 (đã phục hồi)" (Call POST "/bao-quan/su-co" $tokCanBo $scBody)
if ($r.json.id) { $scBody.trangThai = "da_phuc_hoi"; $scBody.ketQuaKiemTra = "Kiểm lại 150 tệp của phông: đạt 150/150"; Call PUT "/bao-quan/su-co/$($r.json.id)" $tokCanBo $scBody | Out-Null }
Buoc "sự cố 2 (đang xử lý)" (Call POST "/bao-quan/su-co" $tokCanBo @{ moTa = "Mất điện lưới 40 phút, UPS tự tắt máy chủ"; nguyenNhan = "Mất điện khu vực"; mucDo = "trung_binh"; phamVi = "Toàn máy chủ"; bienPhap = "Khởi động lại dịch vụ, kiểm tra nhật ký CSDL" }) | Out-Null
}

# ---------- 10. Khai thác, cung cấp thông tin, chuyển đổi, phạm vi phông
Write-Host "===== 10. khai thác (1 đã duyệt, 1 chờ), cung cấp thông tin, chuyển đổi ra giấy, phạm vi phông độc giả"
$docs = @(Sql "SELECT d.id FROM documents d JOIN cl_van_ban_file f ON f.document_id = d.id WHERE d.project_id = '$P' AND d.status = 'archived' ORDER BY d.uploaded_at, d.filename LIMIT 5")
if ($docs.Count -ge 2 -and -not (Co "SELECT count(*) FROM access_requests WHERE project_id = '$P'")) {
  $r = Buoc "độc giả yêu cầu đọc 2 tài liệu" (Call POST "/access" $tokDocGia @{ projectId = $P; kind = "doc"; mucDich = "Nghiên cứu lịch sử địa phương xã Diễn Xá"; phuongThuc = "truc_tiep"; documentIds = @($docs[0], $docs[1]) })
  $yc = $r.json
  if ($yc.id) {
    $ct = (Call GET "/access/$($yc.id)" $tokCanBo).json
    foreach ($it in @($ct.items)) { Call POST "/access/$($yc.id)/items/$($it.id)/review" $tokCanBo @{ verdict = "duyet" } | Out-Null }
    Buoc "cán bộ duyệt yêu cầu đọc" (Call POST "/access/$($yc.id)/approve" $tokCanBo @{ note = "Đọc tại phòng đọc, không sao chụp" }) | Out-Null
  }
  Buoc "độc giả yêu cầu cấp bản sao (chờ duyệt)" (Call POST "/access" $tokDocGia @{ projectId = $P; kind = "ban_sao"; mucDich = "Làm căn cứ giải quyết tranh chấp đất đai"; phuongThuc = "buu_dien"; nguoiNhan = "Hoàng Minh Tuấn, thôn Trung, xã Diễn Xá"; soDinhDanh = "0240xxxxxxxx"; documentIds = @($docs[2 % $docs.Count]) }) | Out-Null
  $r = Buoc "cán bộ chuyển đổi 1 tài liệu ra giấy (PL VI)" (Call POST "/chuyen-doi" $tokCanBo @{ projectId = $P; documentId = $docs[0]; mucDich = "Cung cấp bản giấy theo yêu cầu khai thác"; nguoiNhan = "Hoàng Minh Tuấn"; canCu = "Phiếu yêu cầu khai thác số 01/2026" })
  Buoc "chuyển đổi tài liệu thứ hai" (Call POST "/chuyen-doi" $tokCanBo @{ projectId = $P; documentId = $docs[1]; mucDich = "Phục vụ thanh tra"; nguoiNhan = "Đoàn thanh tra huyện" }) | Out-Null
} elseif ($docs.Count -lt 2) { Write-Host "  (chưa có tài liệu đã nối mục lục — bỏ qua khai thác / chuyển đổi)" -ForegroundColor Yellow } else { Write-Host "  đã có yêu cầu khai thác / chuyển đổi" }
if (-not (Co "SELECT count(*) FROM info_requests WHERE project_id = '$P'")) {
$r = Buoc "độc giả đề nghị cung cấp danh mục tài liệu" (Call POST "/info" $tokDocGia @{ projectId = $P; hinhThuc = "danh_muc"; chuDe = "quyết định đất đai"; mucDich = "Tìm quyết định giao đất của gia đình"; phuongThuc = "truc_tiep" })
if ($r.json.id) { Buoc "cán bộ duyệt đề nghị cung cấp thông tin" (Call POST "/info/$($r.json.id)/approve" $tokCanBo @{ note = "Trả kết quả trong 3 ngày làm việc" }) | Out-Null }
Buoc "đề nghị trích xuất (chờ duyệt)" (Call POST "/info" $tokDocGia @{ projectId = $P; hinhThuc = "trich_xuat"; chuDe = "tuyển dụng viên chức giáo dục 2023"; mucDich = "Xác nhận quá trình công tác"; phuongThuc = "email" }) | Out-Null
}
if ($PH1) { Buoc "độc giả chỉ được xem phông UBND xã Diễn Xá" (Call PUT "/users/$idDocGia/phong" $qt @{ projectId = $P; phongIds = @($PH1) }) | Out-Null }

# ---------- 11. Quét trùng + tổng kết
Write-Host "===== 11. quét hồ sơ / tài liệu trùng (Điều 48)"
$r = Buoc "quét trùng" (Call POST "/disposal/sweep" $qt)
if ($r.json) { Write-Host ("       trùng tên tệp " + $r.json.trungLap + " · trùng mục lục " + $r.json.trungMucLuc + " · trùng BLAKE3 " + $r.json.trungBlake3) }

Write-Host ""
Write-Host ("===== XONG" + $(if ($script:loi -eq 0) { " — không lỗi" } else { " — $($script:loi) bước lỗi (xem dòng đỏ)" })) -ForegroundColor $(if ($script:loi -eq 0) { "Green" } else { "Yellow" })
Write-Host "Tài khoản demo (mật khẩu $MatKhauDemo): canbo (cán bộ lưu trữ) · lanhdao (lãnh đạo) · nvvp (Văn phòng) · nvtc (Tổ chức) · docgia (độc giả)"
