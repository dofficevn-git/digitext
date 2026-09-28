"""Sinh SQL nạp ví dụ plakie (examples.jsonl + ocr.jsonl) vào bảng kie_examples (migration 0010).

  python plakie_to_db.py > plakie.sql
  docker exec -i digitext-dev-postgres-1 psql -U digitext -d digitext -v ON_ERROR_STOP=1 < plakie.sql

Nạp lại nhiều lần không nhân đôi (khoá source + ext_ref = tên ảnh).
"""
import json
import sys
import uuid
from pathlib import Path

D = Path(__file__).parent / "data" / "plakie"


def q(s: str) -> str:
    return "'" + s.replace("'", "''") + "'"


def main() -> None:
    ocr = {r["image"]: r["ocr"] for r in map(json.loads, (D / "ocr.jsonl").open(encoding="utf-8"))}
    out = ["BEGIN;"]
    n = 0
    for r in map(json.loads, (D / "examples.jsonl").open(encoding="utf-8")):
        text = (ocr.get(r["image"]) or "").strip()
        fields = {k: v for k, v in r["fields"].items() if v}
        if not text or not fields:
            continue
        out.append(
            "INSERT INTO kie_examples (id, source, ext_ref, ocr_text, fields) VALUES "
            f"({q(str(uuid.uuid4()))}, {q(r['source'])}, {q(r['image'])}, {q(text[:1500])}, {q(json.dumps(fields, ensure_ascii=False))}::jsonb) "
            "ON CONFLICT (source, ext_ref) WHERE ext_ref IS NOT NULL DO UPDATE SET ocr_text = EXCLUDED.ocr_text, fields = EXCLUDED.fields, updated_at = now();"
        )
        n += 1
    out.append("COMMIT;")
    sys.stdout.reconfigure(encoding="utf-8")
    print("\n".join(out))
    print(f"-- {n} ví dụ", file=sys.stderr)


if __name__ == "__main__":
    main()
