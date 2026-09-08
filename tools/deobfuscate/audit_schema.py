#!/usr/bin/env python3
"""Doi chieu cot ma CODE dung voi cot CO THAT trong CSDL.

Vi sao can: schema duoc khoi phuc bang pg_dump tu server production, nhung ma
nguon trong repo la mot anh chup KHAC — hai ben da lech. Trieu chung dau tien
la 'column "pdfa_key" of relation "documents" does not exist', chi lo ra khi
job make_pdf2 chay. Do tung cai mot la mu; script nay tim HET trong mot luot.

Khong co macro query! nen sqlx khong bat duoc luc bien dich — moi sai lam chi
no luc chay. Day chinh la cai gia cua no.

    python audit_schema.py            # doc schema tu CSDL dang chay
    python audit_schema.py --csv <f>  # hoac tu file CSV da xuat san
"""
from __future__ import annotations

import argparse
import csv
import glob
import os
import re
import subprocess
import sys
from collections import defaultdict

ROOT = "D:/DigiText"
DB_SRC = os.path.join(ROOT, "crates", "digitext-db", "src")

# UPDATE <bang> SET a = ..., b = ...
RE_UPDATE = re.compile(r"UPDATE\s+(\w+)\s+SET\s+(.*?)(?:\s+WHERE|\s+RETURNING|\"|$)",
                       re.S | re.I)
RE_SET_COL = re.compile(r"(\w+)\s*=")
# INSERT INTO <bang> (a, b, c)
RE_INSERT = re.compile(r"INSERT\s+INTO\s+(\w+)\s*\(([^)]*)\)", re.S | re.I)

SQL_WORDS = {
    "now", "select", "from", "where", "and", "or", "not", "null", "true", "false",
    "excluded", "set", "values", "on", "conflict", "do", "update", "insert", "into",
    "returning", "interval", "coalesce", "case", "when", "then", "else", "end",
}


def load_schema_from_db() -> dict[str, set[str]]:
    env = dict(os.environ)
    url = ""
    p = os.path.join(ROOT, "crates", "digitext-api", ".env")
    for line in open(p, encoding="utf-8"):
        if line.startswith("DATABASE_URL="):
            url = line.split("=", 1)[1].strip()
    m = re.match(r"postgres://([^:]+):([^@]+)@([^:]+):(\d+)/(.+)", url)
    if not m:
        raise SystemExit(f"khong doc duoc DATABASE_URL tu {p}")
    env["PGPASSWORD"] = m.group(2)
    env["PGCLIENTENCODING"] = "UTF8"
    psql = r"C:\Program Files\PostgreSQL\18\bin\psql.exe"
    out = subprocess.run(
        [psql, "-h", m.group(3), "-p", m.group(4), "-U", m.group(1), "-d", m.group(5),
         "-t", "-A", "-F", ",", "-c",
         "SELECT table_name, column_name FROM information_schema.columns "
         "WHERE table_schema='public'"],
        capture_output=True, env=env, text=True,
    )
    if out.returncode != 0:
        raise SystemExit(f"psql loi: {out.stderr[:300]}")
    schema = defaultdict(set)
    for line in out.stdout.splitlines():
        if "," in line:
            t, c = line.split(",", 1)
            schema[t.strip()].add(c.strip())
    return schema


def load_schema_from_csv(path: str) -> dict[str, set[str]]:
    schema = defaultdict(set)
    with open(path, encoding="utf-8") as fh:
        for row in csv.reader(fh):
            if len(row) >= 2:
                schema[row[0].strip()].add(row[1].strip())
    return schema


def collect_used() -> dict[str, dict[str, set[str]]]:
    """-> {bang: {cot: {file:dong, ...}}}"""
    used: dict[str, dict[str, set[str]]] = defaultdict(lambda: defaultdict(set))
    for path in sorted(glob.glob(os.path.join(DB_SRC, "**", "*.rs"), recursive=True)):
        rel = os.path.relpath(path, ROOT).replace("\\", "/")
        src = open(path, encoding="utf-8", errors="replace").read()
        # Noi cac manh chuoi bi cat dong bang '\' cho de bat cau lenh tron ven.
        flat = src.replace("\\\n", " ")

        for m in RE_UPDATE.finditer(flat):
            table = m.group(1).lower()
            line = flat[: m.start()].count("\n") + 1
            for c in RE_SET_COL.findall(m.group(2)):
                if c.lower() not in SQL_WORDS:
                    used[table][c.lower()].add(f"{rel}:{line}")

        for m in RE_INSERT.finditer(flat):
            table = m.group(1).lower()
            line = flat[: m.start()].count("\n") + 1
            for c in m.group(2).split(","):
                c = c.strip().strip('"').split()[0] if c.strip() else ""
                if c and re.fullmatch(r"\w+", c) and c.lower() not in SQL_WORDS:
                    used[table][c.lower()].add(f"{rel}:{line}")
    return used


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--csv")
    args = ap.parse_args()

    schema = load_schema_from_csv(args.csv) if args.csv else load_schema_from_db()
    used = collect_used()

    print(f"bang trong CSDL       : {len(schema)}")
    print(f"bang code co ghi vao  : {len(used)}\n")

    missing_tables, missing_cols = [], []
    for table in sorted(used):
        if table not in schema:
            missing_tables.append(table)
            continue
        for col in sorted(used[table]):
            if col not in schema[table]:
                missing_cols.append((table, col, sorted(used[table][col])))

    if missing_tables:
        print("BANG KHONG TON TAI (co the la bi danh bang bien, bo qua duoc):")
        for t in missing_tables:
            print(f"  {t}")
        print()

    if missing_cols:
        print(f"COT CODE DUNG MA CSDL KHONG CO — {len(missing_cols)} cho:")
        for t, c, where in missing_cols:
            print(f"  {t}.{c:24s} {where[0]}")
    else:
        print("Khong thieu cot nao.")
    return 1 if missing_cols else 0


if __name__ == "__main__":
    sys.exit(main())
