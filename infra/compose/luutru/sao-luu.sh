#!/usr/bin/env bash
# Sao lưu máy chủ DigiText Lưu trữ trên Linux (bản Lưu trữ GĐ2, 9/11/2026) — cron 01:00 do cai-dat.sh đặt.
# CSDL: pg_dump -Fc trong container postgres → $SAO_LUU_DIR/csdl, kiểm đọc lại, giữ 14 bản. Kho: rsync thêm/cập nhật
# (không xoá bên đích) → $SAO_LUU_DIR/kho. Ghi sổ backup_runs để màn Bảo quản và sao lưu thấy.
set -uo pipefail
cd "$(dirname "$0")"
set -a; . ./.env; set +a
GIU=${GIU:-14}
dc() { docker compose "$@"; }
# 0032 (26/11/2026): đủ cột Mẫu 02/03 (lịch theo loại, đường dẫn, dung lượng, mã băm); máy chủ chưa áp 0032 thì câu cũ.
CO0032=$(dc exec -T postgres psql -X -t -A -U digitext -d digitext -c "SELECT 1 FROM information_schema.columns WHERE table_name='backup_runs' AND column_name='duong_dan'" 2>/dev/null | tr -d '[:space:]')
so() { # loai nguon ket chi_tiet [duong_dan] [bytes] [ma_bam]
  if [ "$CO0032" = "1" ]; then
    dc exec -T postgres psql -X -q -U digitext -d digitext -v loai="$1" -v nguon="$2" -v ket="$3" -v ct="$4" -v pt="$SAO_LUU_DIR" -v dd="${5:-}" -v by="${6:-}" -v mb="${7:-}" <<'SQL'
INSERT INTO backup_runs (id, loai, nguon, phuong_tien, ket_qua, chi_tiet, lich_id, duong_dan, dung_luong_bytes, ma_bam, ket_thuc_at, nguoi_thuc_hien)
VALUES (gen_random_uuid(), :'loai', :'nguon', :'pt', :'ket', :'ct', (SELECT id FROM bk_lich_sao_luu WHERE loai = :'loai' AND bat ORDER BY created_at LIMIT 1),
        NULLIF(:'dd', ''), NULLIF(:'by', '')::bigint, NULLIF(:'mb', ''), now(), 'sao-luu.sh');
SQL
  else
    dc exec -T postgres psql -X -q -U digitext -d digitext -v loai="$1" -v nguon="$2" -v ket="$3" -v ct="$4" -v pt="$SAO_LUU_DIR" <<'SQL'
INSERT INTO backup_runs (id, loai, nguon, phuong_tien, ket_qua, chi_tiet) VALUES (gen_random_uuid(), :'loai', :'nguon', :'pt', :'ket', :'ct');
SQL
  fi
}
da_xoa() { # đánh dấu bản cũ đã xoá theo đường dẫn
  [ "$CO0032" = "1" ] || return 0
  for f in "$@"; do dc exec -T postgres psql -X -q -U digitext -d digitext -v dd="$f" -c "UPDATE backup_runs SET da_xoa_at = now() WHERE da_xoa_at IS NULL AND duong_dan = :'dd'"; done
}
mkdir -p "$SAO_LUU_DIR/csdl" "$SAO_LUU_DIR/kho"
TEP="$SAO_LUU_DIR/csdl/digitext-$(date +%Y%m%d-%H%M).dump"
LOI=0
if dc exec -T postgres pg_dump -U digitext -d digitext -Fc --no-owner > "$TEP.part" \
   && dc exec -T postgres pg_restore --list < "$TEP.part" > /dev/null; then
  mv "$TEP.part" "$TEP"
  BY=$(stat -c %s "$TEP"); MB=$(( BY / 1048576 ))
  BAM=$(sha256sum "$TEP" | cut -d' ' -f1)
  echo "$(date '+%F %T') CSDL OK $TEP ($MB MB)"
  so day_du "CSDL PostgreSQL (pg_dump)" dat "$TEP — $MB MB, đọc lại mục lục đạt" "$TEP" "$BY" "$BAM"
  CU=$(ls -1t "$SAO_LUU_DIR"/csdl/digitext-*.dump | tail -n +$((GIU + 1)))
  if [ -n "$CU" ]; then echo "$CU" | xargs -r rm -f; da_xoa $CU; fi
else
  rm -f "$TEP.part"; LOI=1
  echo "$(date '+%F %T') CSDL LỖI"
  so day_du "CSDL PostgreSQL (pg_dump)" khong_dat "pg_dump hoặc kiểm đọc lại lỗi"
fi
if rsync -a "$KHO_DIR"/ "$SAO_LUU_DIR/kho/"; then
  echo "$(date '+%F %T') Kho OK → $SAO_LUU_DIR/kho"
  so gia_tang "Kho tài liệu ($KHO_DIR)" dat "rsync -a → $SAO_LUU_DIR/kho" "$SAO_LUU_DIR/kho"
else
  LOI=1
  echo "$(date '+%F %T') Kho LỖI rsync"
  so gia_tang "Kho tài liệu ($KHO_DIR)" khong_dat "rsync lỗi"
fi
exit $LOI
