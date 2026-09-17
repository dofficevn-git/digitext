#!/usr/bin/env python3
"""Gold KIE từ chính nút "Duyệt" của desktop, và so sánh engine bóc trường.

Không có người gõ golden set riêng — nhưng người dùng vẫn duyệt trường trong
OcrFieldsView hằng ngày, và mỗi lần Duyệt ghi `ocr_fields.verified = true`. Đó
là gold có sẵn, miễn phí. Script này:

  export-gold  : rút các trường verified → gold.jsonl (một dòng / tài liệu)
  compare      : so giá trị máy bóc của MỘT engine (theo tên trong
                 engine_versions, ví dụ qwen2.5-vl-7b-instruct-awq-kie hay
                 vnkie-kie) với gold → report.json (exact / CER theo trường)

Đường A/B (kế hoạch nâng cấp OCR, Phase 6): worker trỏ engine A → bấm
"Bóc tách 16 trường" cho các tài liệu gold → compare --engine A; đổi sang B,
lặp lại → compare --engine B. Hai report đặt cạnh nhau là quyết định.

Kết nối CSDL: --dsn (mặc định env DATABASE_URL). Dùng psycopg nếu có; không có
thì chạy qua lệnh psql (--psql, ví dụ trên máy chỉ có docker:
  --psql "docker exec -i digitext-dev-postgres-1 psql -U digitext -d digitext").
"""
from __future__ import annotations

import argparse
import csv
import io
import json
import os
import shlex
import subprocess
import sys
import unicodedata
from collections import defaultdict
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

FIELD_KEYS = [
    "thuTu", "loaiVanBan", "soKyHieu", "ngayThang", "coQuanBanHanh", "trichYeu",
    "toSo", "soTrang", "trangSo", "ngonNgu", "cheDoSuDung", "doTinCay", "butTich",
    "tinhTrangVatLy", "taiLieuSo", "ghiChu",
]
# Ba trường quyết định khi chọn engine (kế hoạch Phase 6.2).
DECISION_FIELDS = ("soKyHieu", "ngayThang", "loaiVanBan")


def norm(s: str | None, ignore_case: bool) -> str:
    s = " ".join(unicodedata.normalize("NFC", s or "").split())
    return s.casefold() if ignore_case else s


# --- truy vấn ---------------------------------------------------------------

def run_query(sql: str, dsn: str | None, psql_cmd: str | None) -> list[dict]:
    if not psql_cmd:
        try:
            import psycopg  # type: ignore

            with psycopg.connect(dsn) as conn, conn.cursor() as cur:
                cur.execute(sql)
                cols = [d.name for d in cur.description]
                return [dict(zip(cols, row)) for row in cur.fetchall()]
        except ModuleNotFoundError:
            pass
        try:
            import psycopg2  # type: ignore

            with psycopg2.connect(dsn) as conn, conn.cursor() as cur:
                cur.execute(sql)
                cols = [d[0] for d in cur.description]
                return [dict(zip(cols, row)) for row in cur.fetchall()]
        except ModuleNotFoundError:
            pass
        psql_cmd = f"psql {shlex.quote(dsn)}" if dsn else "psql"
    cmd = shlex.split(psql_cmd) + ["--csv", "-v", "ON_ERROR_STOP=1", "-c", sql]
    r = subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8")
    if r.returncode != 0:
        raise SystemExit(f"❌ psql lỗi: {r.stderr.strip()}")
    return list(csv.DictReader(io.StringIO(r.stdout)))


GOLD_SQL = """
SELECT DISTINCT ON (f.document_id, f.field_key)
       f.document_id::text AS document_id, d.filename, d.project_id::text AS project_id,
       f.field_key, f.value
FROM ocr_fields f JOIN documents d ON d.id = f.document_id
WHERE f.verified = TRUE AND d.status <> 'disposed'
ORDER BY f.document_id, f.field_key, f.updated_at DESC
"""

PRED_SQL = """
SELECT DISTINCT ON (f.document_id, f.field_key)
       f.document_id::text AS document_id, f.field_key, f.value, f.page_no
FROM ocr_fields f JOIN engine_versions e ON e.id = f.engine_version_id
WHERE e.name = {engine} AND f.text_source = {source} AND f.verified = FALSE
ORDER BY f.document_id, f.field_key, f.page_no ASC, f.updated_at DESC
"""


def sql_str(s: str) -> str:
    return "'" + s.replace("'", "''") + "'"


# --- export-gold --------------------------------------------------------------

def export_gold(args) -> int:
    rows = run_query(GOLD_SQL, args.dsn, args.psql)
    docs: dict[str, dict] = {}
    for r in rows:
        d = docs.setdefault(r["document_id"], {
            "document_id": r["document_id"], "filename": r["filename"],
            "project_id": r["project_id"], "fields": {},
        })
        d["fields"][r["field_key"]] = r["value"]
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    with out.open("w", encoding="utf-8") as f:
        for d in docs.values():
            f.write(json.dumps(d, ensure_ascii=False) + "\n")
    n_fields = sum(len(d["fields"]) for d in docs.values())
    print(f"✅ {out}: {len(docs)} tài liệu, {n_fields} trường đã duyệt")
    if len(docs) < 30:
        print("⚠  dưới 30 tài liệu — so sánh engine sẽ nhiễu; duyệt thêm trong app rồi xuất lại",
              file=sys.stderr)
    return 0


# --- compare ------------------------------------------------------------------

def load_jsonl(path: str) -> list[dict]:
    with open(path, encoding="utf-8") as f:
        return [json.loads(l) for l in f if l.strip()]


def load_pred_jsonl(path: str) -> dict[str, dict[str, str]]:
    """{document_id hoặc filename: {key: value}} — chấp nhận fields dạng dict
    hoặc list [{key, value}] (định dạng extract_vlm.py)."""
    preds: dict[str, dict[str, str]] = {}
    for r in load_jsonl(path):
        key = r.get("document_id") or r.get("filename") or r.get("id")
        if not key:
            continue
        fields = r.get("fields") or {}
        if isinstance(fields, list):
            fields = {x.get("key"): x.get("value") for x in fields if isinstance(x, dict)}
        cur = preds.setdefault(str(key), {})
        for k, v in fields.items():
            if k and v and k not in cur:  # trang đầu tiên có giá trị thắng
                cur[k] = v
    return preds


def compare(args) -> int:
    from cer import q_110 as edit_distance

    gold = load_jsonl(args.gold)
    if not gold:
        raise SystemExit("❌ gold rỗng")
    if args.pred_jsonl:
        preds = load_pred_jsonl(args.pred_jsonl)
        pred_of = lambda g: preds.get(g["document_id"]) or preds.get(g.get("filename", "")) or {}
        engine_label = args.pred_jsonl
    else:
        if not args.engine:
            raise SystemExit("❌ cần --engine <tên engine_versions> hoặc --pred-jsonl")
        rows = run_query(PRED_SQL.format(engine=sql_str(args.engine), source=sql_str(args.text_source)),
                         args.dsn, args.psql)
        preds = defaultdict(dict)
        for r in rows:
            preds[r["document_id"]][r["field_key"]] = r["value"]
        pred_of = lambda g: preds.get(g["document_id"], {})
        engine_label = args.engine

    stats: dict[str, dict] = {k: {"n": 0, "exact": 0, "edit": 0, "n_gold": 0, "missing": 0}
                              for k in FIELD_KEYS}
    n_docs_with_pred = 0
    for g in gold:
        p = pred_of(g)
        if p:
            n_docs_with_pred += 1
        for k, gv in g["fields"].items():
            if k not in stats:
                continue
            s = stats[k]
            gvn = norm(gv, args.ignore_case)
            if not gvn:
                continue
            pvn = norm(p.get(k), args.ignore_case)
            s["n"] += 1
            s["n_gold"] += len(gvn)
            if not pvn:
                s["missing"] += 1
                s["edit"] += len(gvn)
                continue
            s["exact"] += int(pvn == gvn)
            s["edit"] += edit_distance(pvn, gvn)

    def block(keys):
        n = sum(stats[k]["n"] for k in keys)
        ex = sum(stats[k]["exact"] for k in keys)
        ed = sum(stats[k]["edit"] for k in keys)
        ng = sum(stats[k]["n_gold"] for k in keys)
        miss = sum(stats[k]["missing"] for k in keys)
        return {"n": n, "exact": round(ex / n, 4) if n else None,
                "cer": round(ed / ng, 4) if ng else None,
                "missing": round(miss / n, 4) if n else None}

    report = {
        "meta": {"engine": engine_label, "gold": args.gold, "n_gold_docs": len(gold),
                 "n_docs_with_pred": n_docs_with_pred, "ignore_case": args.ignore_case},
        "overall": block([k for k in FIELD_KEYS if stats[k]["n"]]),
        "quyet_dinh": block([k for k in DECISION_FIELDS if stats[k]["n"]]),
        "by_field": {k: block([k]) for k in FIELD_KEYS if stats[k]["n"]},
    }
    Path(args.out).parent.mkdir(parents=True, exist_ok=True)
    with open(args.out, "w", encoding="utf-8") as f:
        json.dump(report, f, ensure_ascii=False, indent=2)
    q = report["quyet_dinh"]
    print(f"✅ {args.out}: {engine_label} — {n_docs_with_pred}/{len(gold)} tài liệu có dự đoán · "
          f"3 trường quyết định: exact={q['exact']} cer={q['cer']} thiếu={q['missing']}")
    return 0


def main(argv=None) -> int:
    p = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p.add_argument("--dsn", default=os.environ.get("DATABASE_URL"), help="postgres://… (env DATABASE_URL)")
    p.add_argument("--psql", default=os.environ.get("PSQL_CMD"),
                   help="lệnh psql thay cho psycopg, vd 'docker exec -i <ct> psql -U digitext -d digitext'")
    sub = p.add_subparsers(dest="cmd", required=True)
    e = sub.add_parser("export-gold", help="ocr_fields.verified=true → gold.jsonl")
    e.add_argument("--out", default="data/kie_gold_db.jsonl")
    e.set_defaults(fn=export_gold)
    c = sub.add_parser("compare", help="so một engine với gold → report.json")
    c.add_argument("--gold", default="data/kie_gold_db.jsonl")
    c.add_argument("--engine", help="engine_versions.name, vd qwen2.5-vl-7b-instruct-awq-kie / vnkie-kie")
    c.add_argument("--text-source", default="ocr_raw")
    c.add_argument("--pred-jsonl", help="thay cho --engine: dự đoán dạng jsonl {document_id|filename, fields}")
    c.add_argument("--ignore-case", action="store_true")
    c.add_argument("--out", required=True)
    c.set_defaults(fn=compare)
    args = p.parse_args(argv)
    return args.fn(args)


if __name__ == "__main__":
    sys.exit(main())
