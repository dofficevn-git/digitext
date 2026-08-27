; Copyright (c) 2026 DigiText. All rights reserved.
; Hook NSIS cho installer Windows (bundle.windows.nsis.installerHooks — Tauri trộn vào
; installer.nsi, nơi ${MAINBINARYNAME} đã được define).
;
; VÌ SAO: đồng bộ LAN (chia sẻ user→admin, phát cây thư mục) cần inbound TCP tới cổng
; listener + UDP 5353 (mDNS). Không có rule, Windows hiện hộp thoại "Allow access" lần
; đăng nhập đầu — người dùng bấm Cancel là bị CHẶN VĨNH VIỄN, hoặc Allow nhưng chỉ tick
; Private trong khi wifi cơ quan bị gắn profile Public → vẫn chặn. Thêm rule theo
; CHƯƠNG TRÌNH (mọi cổng — listener dùng cổng ngẫu nhiên trừ khi ghim) trên MỌI profile
; để xoá hẳn hai bẫy đó.
;
; Best-effort: netsh cần quyền admin. Installer chạy không elevate (installMode
; currentUser mặc định) thì lệnh fail — chỉ ghi log, KHÔNG chặn cài đặt; khi đó Windows
; quay về hộp thoại hỏi như cũ. Cài per-machine (elevated) thì rule được thêm tự động.

!macro NSIS_HOOK_POSTINSTALL
  ; Xoá rule cũ (nếu có) trước khi thêm — idempotent qua cài lại/cập nhật, và dọn được
  ; rule trỏ tới đường dẫn cài đặt cũ.
  nsExec::ExecToLog 'netsh advfirewall firewall delete rule name="DigiText LAN"'
  Pop $0
  nsExec::ExecToLog 'netsh advfirewall firewall add rule name="DigiText LAN" dir=in action=allow program="$INSTDIR\${MAINBINARYNAME}.exe" enable=yes profile=any'
  Pop $0
  ${If} $0 != 0
    DetailPrint "Không thêm được rule tường lửa (cần quyền admin) — Windows sẽ hỏi khi app chạy lần đầu."
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ; Gỡ app thì gỡ luôn rule — không để rule mồ côi trỏ tới file đã xoá.
  nsExec::ExecToLog 'netsh advfirewall firewall delete rule name="DigiText LAN"'
  Pop $0
!macroend
