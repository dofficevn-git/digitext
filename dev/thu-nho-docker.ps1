# Thu nhỏ ổ ảo Docker (E:\Docker\wsl\disk\docker_data.vhdx) sau khi đã xoá dữ liệu bên trong.
# vhdx chỉ phình, không tự co. DỪNG CẢ DOCKER (DigiText, KPI335, KPI-DO, DOP-KHCN, do360) trong lúc chạy;
# dữ liệu các hệ thống KHÔNG bị đụng tới (không prune, chỉ fstrim báo khối trống).
# Lần 9/10/2026 bị Windows Update tắt máy giữa chừng (63%) → bản này: tạm hoãn Windows Update 1 ngày,
# không giới hạn thời gian, và LUÔN bật lại Docker kể cả khi compact lỗi/ngắt.
# Chạy: Start-Process powershell.exe -Verb RunAs -ArgumentList "-NoProfile -ExecutionPolicy Bypass -File D:\DigiText\dev\thu-nho-docker.ps1"
# Mặc định là đường dẫn trên máy .127; máy khác truyền -Vhdx / -DockerExe (30/10/2026).
param(
    [string]$Vhdx = "E:\Docker\wsl\disk\docker_data.vhdx",
    [string]$DockerExe = "C:\Users\HieuPC\AppData\Local\Programs\DockerDesktop\Docker Desktop.exe",
    [string]$Log = ""
)
$ErrorActionPreference = "Stop"
if (-not $Log) { $Log = Join-Path (Split-Path -Parent $PSScriptRoot) "logs\thu-nho-docker.log" }
$vhdx = $Vhdx
$dockerExe = $DockerExe
$log = $Log
if (-not (Test-Path $vhdx)) { throw "Không thấy ổ ảo Docker $vhdx — truyền -Vhdx" }
if (-not (Test-Path $dockerExe)) { throw "Không thấy Docker Desktop $dockerExe — truyền -DockerExe" }

$admin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if (-not $admin) {
    Start-Process powershell.exe -Verb RunAs -ArgumentList "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`" -Vhdx `"$vhdx`" -DockerExe `"$dockerExe`" -Log `"$log`""
    exit
}

function Ghi($m) {
    $l = "$(Get-Date -Format 'yyyy-MM-dd HH:mm:ss') $m"
    Write-Host $l
    Add-Content -Path $log -Value $l -Encoding utf8
}
function Co() { [math]::Round((Get-Item $vhdx).Length / 1GB, 1) }

$truoc = Co
Ghi "=== Bắt đầu (chạy trọn). vhdx $truoc GB"
$truocDs = @(docker ps --format "{{.Names}}")
Ghi "Đang chạy $($truocDs.Count) container: $($truocDs -join ', ')"

# 0. Tạm hoãn Windows Update 1 ngày để máy không tự khởi động lại giữa chừng.
try {
    $k = "HKLM:\SOFTWARE\Microsoft\WindowsUpdate\UX\Settings"
    $den = (Get-Date).ToUniversalTime().AddDays(1).ToString("yyyy-MM-ddTHH:mm:ssZ")
    Set-ItemProperty -Path $k -Name PauseUpdatesStartTime -Value (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    Set-ItemProperty -Path $k -Name PauseUpdatesExpiryTime -Value $den
    Ghi "Tạm hoãn Windows Update đến $den (UTC)"
} catch { Ghi "Không hoãn được Windows Update: $_" }

# 1. Báo cho ext4 biết khối nào trống (không đổi dữ liệu; không trim thì compact gần như không co).
try {
    docker run --rm --privileged --pid=host alpine nsenter -t 1 -m -- fstrim -av 2>&1 | ForEach-Object { Ghi "fstrim: $_" }
} catch { Ghi "Bỏ qua fstrim: $_" }

try {
    # 2. Tắt Docker theo cách bình thường (container dừng êm, lên lại theo restart policy).
    Ghi "Tắt Docker Desktop"
    docker desktop stop --timeout 180 2>&1 | ForEach-Object { Ghi "  $_" }
    Start-Sleep -Seconds 5
    Get-Process "Docker Desktop", "com.docker.backend", "com.docker.build" -ErrorAction SilentlyContinue | Stop-Process -Force
    wsl --shutdown
    Start-Sleep -Seconds 10

    # 3. Compact bằng diskpart (máy không có Hyper-V nên không có Optimize-VHD).
    $dp = Join-Path $env:TEMP "thu-nho-docker.txt"
    "select vdisk file=`"$vhdx`"`r`nattach vdisk readonly`r`ncompact vdisk`r`ndetach vdisk" | Set-Content -Path $dp -Encoding ascii
    Ghi "diskpart compact (lâu, ổ HDD)…"
    $lastPct = ""
    diskpart /s $dp | ForEach-Object {
        $s = "$_".Trim()
        if ($s -match "percent completed") {
            if ($s -ne $lastPct) { Add-Content -Path $log -Value "  $s" -Encoding utf8; $lastPct = $s }
        } elseif ($s) { Add-Content -Path $log -Value "  $s" -Encoding utf8 }
    }
    Ghi "Compact xong: $truoc GB -> $(Co) GB"
} catch {
    Ghi "LỖI trong lúc compact: $_"
} finally {
    # Luôn tháo đĩa ảo và bật lại Docker.
    $dp2 = Join-Path $env:TEMP "thu-nho-docker-detach.txt"
    "select vdisk file=`"$vhdx`"`r`ndetach vdisk" | Set-Content -Path $dp2 -Encoding ascii
    diskpart /s $dp2 | Out-Null
    Ghi "Bật lại Docker Desktop"
    Start-Process $dockerExe
    $han = (Get-Date).AddMinutes(15)
    do {
        Start-Sleep -Seconds 10
        docker info *> $null
    } until ($LASTEXITCODE -eq 0 -or (Get-Date) -gt $han)
    Start-Sleep -Seconds 90
    $sauDs = @(docker ps --format "{{.Names}}")
    Ghi "Docker: $($sauDs.Count)/$($truocDs.Count) container đang chạy"
    $thieu = $truocDs | Where-Object { $sauDs -notcontains $_ }
    if ($thieu) { Ghi "CHƯA LÊN: $($thieu -join ', ')" }
    try {
        $r = Invoke-WebRequest -UseBasicParsing -TimeoutSec 15 http://127.0.0.1:8080/healthz
        Ghi "API DigiText: $($r.StatusCode)"
    } catch { Ghi "API DigiText chưa trả lời: $_" }
    Ghi "=== XONG. vhdx $(Co) GB"
}
