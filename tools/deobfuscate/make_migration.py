#!/usr/bin/env python3
"""Bien schema.sql (pg_dump) thanh migration sqlx dung duoc.

pg_dump chia file thanh cac khoi, moi khoi mo dau bang:
    --
    -- Name: <ten>; Type: <loai>; Schema: <schema>; Owner: -
    --
Ta giu lai khoi thuoc schema `public`, bo khoi thuoc `_sqlx_test`
(bookkeeping cua test harness, khong phai schema ung dung), bo lenh meta
cua psql va khoi SET dau file.
"""
import re, os, csv

SRC = "D:/DigiText/digitext-rescue/db/schema.sql"
CSV = "D:/DigiText/digitext-rescue/db/migrations.csv"
DEST_DIR = "D:/DigiText/migrations"

RE_HDR = re.compile(r'^-- Name: (.+?); Type: (.+?); Schema: (.+?); Owner:')
SKIP_LINE = re.compile(r'^\\(restrict|unrestrict)\b|^SET \w+|^SELECT pg_catalog\.set_config')

lines = open(SRC, encoding='utf-8').read().splitlines()

blocks, cur, meta = [], [], None
i = 0
while i < len(lines):
    ln = lines[i]
    # phat hien khoi tieu de: "--", "-- Name: ...", "--"
    if ln == '--' and i + 2 < len(lines) and (m := RE_HDR.match(lines[i + 1])) and lines[i + 2] == '--':
        if cur:
            blocks.append((meta, cur))
        meta, cur = (m.group(1), m.group(2), m.group(3)), []
        i += 3
        continue
    cur.append(ln)
    i += 1
if cur:
    blocks.append((meta, cur))

kept, dropped = [], []
for meta, body in blocks:
    if meta is None:                      # phan dau file truoc khoi tieu de dau tien
        dropped.append(('preamble', len(body)))
        continue
    name, typ, schema = meta
    # _sqlx_test: bookkeeping cua test harness, khong phai schema ung dung.
    # _sqlx_migrations: sqlx TU tao bang nay truoc khi chay migration —
    #   de lai se xung dot "relation already exists".
    if (schema.strip() == '_sqlx_test' or name.strip() == '_sqlx_test'
            or name.strip().startswith('_sqlx_migrations')):
        dropped.append((f'{schema}.{name} ({typ})', len(body)))
        continue
    text = '\n'.join(l for l in body if not SKIP_LINE.match(l)).strip()
    if text:
        kept.append((name, typ, text))

os.makedirs(DEST_DIR, exist_ok=True)

# doc lai lich su migration goc de ghi vao header
orig = []
with open(CSV, encoding='utf-8') as fh:
    for row in csv.DictReader(fh):
        orig.append((row.get('version', '?'), row.get('description', '?')))

hdr = f"""-- DigiText — baseline schema
--
-- File nay duoc sinh tu `pg_dump --schema-only` chay tren server production
-- (Postgres 16.14) ngay 2026-08-27, roi loc bo lenh meta cua psql va schema
-- `_sqlx_test`. Xem tools/deobfuscate/make_migration.py.
--
-- DAY LA BAN GOP (squashed) cua {len(orig)} migration goc — cac file .sql goc da
-- bi mat cung voi repo nguon. Danh sach migration goc (tu bang _sqlx_migrations):
--
"""
for v, d in orig:
    hdr += f"--   {v:>3}  {d}\n"
hdr += f"""--
-- CANH BAO: KHONG chay file nay len database production. O do bang
-- _sqlx_migrations da ghi nhan {len(orig)} migration; sqlx se bao loi lech
-- checksum ngay o version 1. File nay danh cho database dung moi (dev/test).
--

"""

out = [hdr]
for name, typ, text in kept:
    out.append(f"-- {typ}: {name}\n{text}\n")

dest = os.path.join(DEST_DIR, '0001_baseline.sql')
open(dest, 'w', encoding='utf-8', newline='\n').write('\n'.join(out))

body = '\n'.join(out)
print(f"khoi giu lai      : {len(kept)}")
print(f"khoi bo di        : {len(dropped)}  -> {[d[0] for d in dropped][:6]}")
print(f"migration goc     : {len(orig)}")
print()
for k in ('CREATE TABLE', 'CREATE INDEX', 'CREATE UNIQUE INDEX', 'ALTER TABLE',
          'CREATE TRIGGER', 'CREATE FUNCTION', 'CREATE SEQUENCE'):
    print(f"  {k:22s} {len(re.findall('^' + k, body, re.M))}")
print(f"\n-> {dest}  ({os.path.getsize(dest)/1024:.0f} KB)")
