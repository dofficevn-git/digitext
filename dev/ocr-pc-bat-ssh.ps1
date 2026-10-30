# Chạy MỘT LẦN trên OCR PC (PowerShell admin, qua RDP) — bật OpenSSH Server chỉ mở từ Tailscale, nhận khoá máy .127,
# tắt ngủ. Sau đó máy .127 vào được bằng: ssh <user>@100.122.226.63   (30/10/2026, docs/CHUYEN-MAY-OCR-PC.md §3.1)
$ErrorActionPreference = "Stop"
$khoa = "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIO9q7pVzSDjwc0tE0I5+uxHngrG+/jjU+eudqcHMcebO hieupc@DESKTOP-2DMGE1I"

$cap = Get-WindowsCapability -Online -Name "OpenSSH.Server*"
if ($cap.State -ne "Installed") { Add-WindowsCapability -Online -Name $cap.Name | Out-Null }
Set-Service sshd -StartupType Automatic
Start-Service sshd

# Khoá cho tài khoản thuộc nhóm Administrators: OpenSSH Windows đọc tệp này, KHÔNG đọc ~\.ssh\authorized_keys.
$f = "C:\ProgramData\ssh\administrators_authorized_keys"
$cu = if (Test-Path $f) { Get-Content $f } else { @() }
if ($cu -notcontains $khoa) { Add-Content -Path $f -Value $khoa -Encoding ascii }
icacls $f /inheritance:r /grant "Administrators:F" /grant "SYSTEM:F" | Out-Null

# Shell mặc định PowerShell (để lệnh từ xa chạy được script .ps1 trực tiếp).
New-Item -Path "HKLM:\SOFTWARE\OpenSSH" -Force | Out-Null
New-ItemProperty -Path "HKLM:\SOFTWARE\OpenSSH" -Name DefaultShell -Value "C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe" -PropertyType String -Force | Out-Null

# Firewall: chỉ Tailscale (100.64.0.0/10). Bỏ rule mặc định mở mọi nơi nếu Windows tự tạo.
Get-NetFirewallRule -Name "OpenSSH-Server-In-TCP" -ErrorAction SilentlyContinue | Remove-NetFirewallRule
if (-not (Get-NetFirewallRule -Name "sshd-tailscale" -ErrorAction SilentlyContinue)) {
  New-NetFirewallRule -Name "sshd-tailscale" -DisplayName "OpenSSH (Tailscale)" -Direction Inbound -Protocol TCP -LocalPort 22 -RemoteAddress 100.64.0.0/10 -Action Allow | Out-Null
}
Restart-Service sshd

# Máy chủ không được ngủ.
powercfg /change standby-timeout-ac 0
powercfg /change hibernate-timeout-ac 0

"OK: sshd $((Get-Service sshd).Status), tài khoản: $env:USERNAME, Tailscale: $((Get-NetIPAddress -AddressFamily IPv4 | Where-Object { $_.IPAddress -like '100.*' }).IPAddress)"
