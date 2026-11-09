#!/usr/bin/env bash
# Sao lưu máy chủ DigiText Lưu trữ trên Linux (bản Lưu trữ GĐ2, 9/11/2026) — cron 01:00 do cai-dat.sh đặt.
# CSDL: pg_dump -Fc trong container postgres → $SAO_LUU_DIR/csdl, kiểm đọc lại, giữ 14 bản. Kho: rsync thêm/cập nhật
# (không xoá bên đích) → $SAO_LUU_DIR/kho. Ghi sổ backup_runs để màn Bảo quản và sao lưu thấy.
set -uo pipefail
cd "$(dirname "$0")"
set -a; . ./.env; set +a
GIU=${GIU:-14}
dc() { docker compose "$@"; }
so() { dc exec -T postgres psql -X -q -U digitext -d digitext -v loai="$1" -v nguon="$2" -v ket="$3" -v ct="$4" -v pt="$SAO_LUU_DIR" <<'SQL'
INSERT INTO backup_runs (id, loai, nguon, phuong_tien, ket_qua, chi_tiet) VALUES (gen_random_uuid(), :'loai', :'nguon', :'pt', :'ket', :'ct');
SQL
}
mkdir -p "$SAO_LUU_DIR/csdl" "$SAO_LUU_DIR/kho"
TEP="$SAO_LUU_DIR/csdl/digitext-$(date +%Y%m%d-%H%M).dump"
LOI=0
if dc exec -T postgres pg_dump -U digitext -d digitext -Fc --no-owner > "$TEP.part" \
   && dc exec -T postgres pg_restore --list < "$TEP.part" > /dev/null; then
  mv "$TEP.part" "$TEP"
  MB=$(( $(stat -c %s "$TEP") / 1048576 ))
  echo "$(date '+%F %T') CSDL OK $TEP ($MB MB)"
  so day_du "CSDL PostgreSQL (pg_dump)" dat "$TEP — $MB MB, đọc lại mục lục đạt"
  ls -1t "$SAO_LUU_DIR"/csdl/digitext-*.dump | tail -n +$((GIU + 1)) | xargs -r rm -f
else
  rm -f "$TEP.part"; LOI=1
  echo "$(date '+%F %T') CSDL LỖI"
  so day_du "CSDL PostgreSQL (pg_dump)" khong_dat "pg_dump hoặc kiểm đọc lại lỗi"
fi
if rsync -a "$KHO_DIR"/ "$SAO_LUU_DIR/kho/"; then
  echo "$(date '+%F %T') Kho OK → $SAO_LUU_DIR/kho"
  so gia_tang "Kho tài liệu ($KHO_DIR)" dat "rsync -a → $SAO_LUU_DIR/kho"
else
  LOI=1
  echo "$(date '+%F %T') Kho LỖI rsync"
  so gia_tang "Kho tài liệu ($KHO_DIR)" khong_dat "rsync lỗi"
fi
exit $LOI
