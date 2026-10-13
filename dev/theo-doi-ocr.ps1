# Bảng theo dõi OCR: máy GPU (Alienware) + hàng đợi trên máy chủ này. Tự làm mới mỗi 5 giây.
#   powershell -ExecutionPolicy Bypass -File dev\theo-doi-ocr.ps1
# Cần: khoá SSH của máy này vào $GpuHost (đã có), Docker (Postgres) đang chạy. Chỉ ĐỌC, không sửa gì.
param(
  [string]$GpuHost = "do@192.168.0.205",
  [string]$OcrContainer = "digitext-ocr-fulltext-ocr-1",
  [string]$PgContainer = "digitext-dev-postgres-1",
  [int]$Seconds = 5
)
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$host.UI.RawUI.WindowTitle = "DigiText - theo dõi OCR"

$sql = @"
SELECT
  count(*) FILTER (WHERE kind='ocr' AND status='queued'),
  count(*) FILTER (WHERE kind='ocr' AND status='running'),
  count(*) FILTER (WHERE kind='ocr' AND status='done'),
  count(*) FILTER (WHERE kind='ocr' AND status IN ('failed_transient','page_failed')),
  count(*) FILTER (WHERE kind='ocr' AND status='done' AND updated_at > now() - interval '1 hour'),
  count(*) FILTER (WHERE kind='make_pdf2' AND status='queued'),
  count(*) FILTER (WHERE kind='make_pdf2' AND status='done')
FROM ocr_jobs
"@

# Ký số (13/10/2026): ký chạy ở máy trạm cắm token, máy chủ chỉ giữ phiên ký — đọc từ sign_sessions + documents.
# Cột ngăn bằng "~" (PowerShell 5.1 làm hỏng dấu nháy kép khi chuyển sang sh).
$sqlKy = @"
SELECT 'T',
  count(*) FILTER (WHERE status='open' AND lease_until > now()),
  count(*) FILTER (WHERE status='signed' AND updated_at > now() - interval '10 minutes'),
  count(*) FILTER (WHERE status='signed' AND updated_at > now() - interval '1 hour'),
  count(*) FILTER (WHERE status='signed' AND updated_at::date = (now() at time zone 'Asia/Ho_Chi_Minh')::date),
  count(*) FILTER (WHERE status='rejected' AND updated_at > now() - interval '1 hour'),
  coalesce(to_char(max(updated_at) FILTER (WHERE status='signed') at time zone 'Asia/Ho_Chi_Minh','DD/MM HH24:MI:SS'),'-'),
  coalesce((SELECT coalesce(u.display_name,u.username) FROM sign_sessions s2 JOIN users u ON u.id=s2.signer_user_id
            WHERE s2.status='signed' ORDER BY s2.updated_at DESC LIMIT 1),'-')
FROM sign_sessions;
SELECT 'P', p.code, count(*) FILTER (WHERE d.status='to_sign'), count(*) FILTER (WHERE d.status='signing'), count(*) FILTER (WHERE d.status='signed')
FROM documents d JOIN projects p ON p.id=d.project_id
WHERE d.status IN ('to_sign','signing','signed') GROUP BY p.code ORDER BY 3 DESC LIMIT 8;
"@

while ($true) {
  # 6/10/2026: Alienware đi Wi-Fi (ping 11–470 ms) — 5 s hay hụt ở bước bắt tay SSH, báo lỗi giả.
  $gpu = ssh -o BatchMode=yes -o ConnectTimeout=20 -o ServerAliveInterval=5 -o ServerAliveCountMax=3 $GpuHost "nvidia-smi --query-gpu=name,utilization.gpu,memory.used,memory.total,temperature.gpu,power.draw --format=csv,noheader,nounits; docker ps --format '{{.Names}}|{{.Status}}'; echo '@@LOG'; docker logs --tail 8 $OcrContainer 2>&1" 2>&1
  $sshOk = ($LASTEXITCODE -eq 0)
  $q = $sql | docker exec -i $PgContainer sh -c 'psql -U $POSTGRES_USER -d $POSTGRES_DB -At -F,' 2>&1

  Clear-Host
  Write-Host ("DigiText - theo dõi OCR   " + (Get-Date -Format "dd/MM/yyyy HH:mm:ss") + "   (Ctrl+C để thoát)") -ForegroundColor Cyan
  Write-Host ""

  if (-not $sshOk) {
    Write-Host "Lần này không SSH được tới $GpuHost (mạng chập chờn?) — thử lại sau $Seconds giây. OCR vẫn chạy, xem hàng đợi bên dưới." -ForegroundColor DarkYellow
    Write-Host "  $(@($gpu) | Select-Object -First 1)" -ForegroundColor DarkGray
  } else {
    $lines = @($gpu)
    $i = [Array]::IndexOf($lines, '@@LOG')
    $head = if ($i -ge 0) { $lines[0..($i - 1)] } else { $lines }
    $log = if ($i -ge 0 -and $i + 1 -lt $lines.Count) { $lines[($i + 1)..($lines.Count - 1)] } else { @() }
    $g = ($head[0] -split ',\s*')
    if ($g.Count -ge 6) {
      Write-Host "MÁY GPU  $($g[0])" -ForegroundColor Yellow
      Write-Host ("  GPU bận: {0}%   Bộ nhớ: {1}/{2} MB   Nhiệt độ: {3}°C   Công suất: {4} W" -f $g[1], $g[2], $g[3], $g[4], $g[5])
    } else { Write-Host "  $($head[0])" -ForegroundColor Red }
    Write-Host "  Dịch vụ:"
    foreach ($c in $head[1..($head.Count - 1)]) { if ($c) { $p = $c -split '\|'; Write-Host ("    {0,-32} {1}" -f $p[0], $p[1]) } }
    Write-Host ""
    Write-Host "OCR VỪA LÀM (8 dòng cuối nhật ký máy GPU)" -ForegroundColor Yellow
    foreach ($l in $log) { $t = "$l"; if ($t.Length -gt 150) { $t = $t.Substring(0, 150) }; Write-Host "  $t" }
  }

  Write-Host ""
  $v = ("$q" -split ',')
  if ($v.Count -ge 7) {
    $conLai = [int]$v[0]; $mot_gio = [int]$v[4]
    Write-Host "HÀNG ĐỢI TRÊN MÁY CHỦ" -ForegroundColor Yellow
    Write-Host ("  Trang chờ OCR: {0:N0}   Đang chạy: {1}   Xong: {2:N0}   Lỗi: {3}" -f $conLai, [int]$v[1], [int]$v[2], [int]$v[3])
    Write-Host ("  Tốc độ 1 giờ qua: {0:N0} trang/giờ" -f $mot_gio)
    if ($mot_gio -gt 0) {
      $gio = $conLai / $mot_gio
      Write-Host ("  Ước tính còn: {0:N0} giờ (~{1:N1} ngày) nếu giữ tốc độ này" -f $gio, ($gio / 24))
    }
    Write-Host ("  PDF 2 lớp: chờ {0:N0}   xong {1:N0}" -f [int]$v[5], [int]$v[6])
  } else { Write-Host "Không đọc được hàng đợi: $q" -ForegroundColor Red }

  Write-Host ""
  $k = @($sqlKy | docker exec -i $PgContainer sh -c 'psql -U $POSTGRES_USER -d $POSTGRES_DB -At -F~' 2>&1)
  $t = $k | Where-Object { "$_".StartsWith('T~') } | Select-Object -First 1
  if ($t) {
    $s = "$t" -split '~'
    $muoi = [int]$s[2]
    Write-Host "KÝ SỐ (ký ở máy trạm cắm token)" -ForegroundColor Yellow
    $dang = if ([int]$s[1] -gt 0) { "ĐANG KÝ ($($s[1]) tệp đang xử lý)" } else { "không có phiên ký nào đang mở" }
    Write-Host ("  Trạng thái: {0}   Lần ký cuối: {1} ({2})" -f $dang, $s[6], $s[7])
    Write-Host ("  Đã ký: 10 phút qua {0:N0}   1 giờ qua {1:N0}   hôm nay {2:N0}   Bị từ chối 1 giờ qua: {3}" -f $muoi, [int]$s[3], [int]$s[4], [int]$s[5])
    if ($muoi -gt 0) { Write-Host ("  Tốc độ hiện tại: ~{0:N0} tệp/giờ" -f ($muoi * 6)) }
    foreach ($r in ($k | Where-Object { "$_".StartsWith('P~') })) {
      $p = "$r" -split '~'
      $choKy = [int]$p[2]
      $eta = if ($muoi -gt 0 -and $choKy -gt 0) { "   còn ~{0:N1} giờ" -f ($choKy / ($muoi * 6)) } else { "" }
      Write-Host ("  {0,-12} chờ ký {1,9:N0}   đang ký {2,3}   đã ký {3,9:N0}{4}" -f $p[1], $choKy, [int]$p[3], [int]$p[4], $eta)
    }
  } else { Write-Host "Không đọc được tiến độ ký: $k" -ForegroundColor Red }

  Start-Sleep -Seconds $Seconds
}
