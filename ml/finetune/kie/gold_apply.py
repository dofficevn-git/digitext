#!/usr/bin/env python3
""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

VERDICTS = {"ok", "fix_span", "missing", "ocr_wrong", "vlm_wrong"}
VLM_SIDE = {"vlm_wrong", "fix_span", "missing"}
OCR_SIDE = {"ocr_wrong"}

def field_spans_from_tags(tags: list[str]) -> dict[str, list[int]]:
    p_3e8: dict[str, list[int]] = {}
    for p_3e7, p_3e9 in enumerate(tags):
        if p_3e9 == "O" or "-" not in p_3e9:
            continue
        p_3e8.setdefault(p_3e9.split("-", 1)[1], []).append(p_3e7)
    return p_3e8

def q_151(verdict: str, given: list[int], silver_span: list[int]) -> list[int]:
    if verdict == "vlm_wrong":
        return []
    if verdict == "ok":
        return list(silver_span)
    if verdict in ("fix_span", "missing", "ocr_wrong"):
        return list(given) if given else list(silver_span)
    raise ValueError(f"verdict lạ: {verdict}")

def q_f3(bio: dict, decision: dict) -> tuple[dict, dict[str, str]]:
    p_3f0 = len(bio.get("words", []))
    tags = ["O"] * p_3f0
    p_3f1 = field_spans_from_tags(bio.get("tags", []))
    p_3eb = decision.get("fields", {})

    p_3f2: dict[str, str] = {}
    p_3ef = set(p_3f1) | set(p_3eb)
    for p_3ee in p_3ef:
        p_3ea = p_3eb.get(p_3ee, {"verdict": "ok"})
        verdict = p_3ea.get("verdict", "ok")
        if verdict not in VERDICTS:
            raise ValueError(f"{bio['id']}/{p_3ee}: {verdict!r} không hợp lệ")
        span = q_151(verdict, p_3ea.get("tokens", []), p_3f1.get(p_3ee, []))
        span = sorted(p_3ec for p_3ec in span if 0 <= p_3ec < p_3f0)
        p_3f2[p_3ee] = verdict
        if not span:
            continue
        tags[span[0]] = f"B-{p_3ee}"
        for p_3ec in span[1:]:
            tags[p_3ec] = f"I-{p_3ee}"

    gold = {p_3ed: bio[p_3ed] for p_3ed in ("id", "document_id", "page_no", "width", "height",
                                "words", "bboxes") if p_3ed in bio}
    gold["tags"] = tags
    gold["source"] = "gold"
    gold["labeler"] = "claude-opus"
    return gold, p_3f2

def q_f6(all_used: list[dict[str, str]]) -> dict:
    p_3f4: dict[str, dict[str, int]] = {}
    for p_3fa in all_used:
        for p_3f3, verdict in p_3fa.items():
            p_3f5 = p_3f4.setdefault(p_3f3, {p_3fb: 0 for p_3fb in VERDICTS})
            p_3f5[verdict] += 1
    p_3f7 = {}
    for p_3f3, p_3f5 in p_3f4.items():
        total = sum(p_3f5.values())
        vlm = sum(p_3f5[p_3fb] for p_3fb in VLM_SIDE)
        ocr = sum(p_3f5[p_3fb] for p_3fb in OCR_SIDE)
        p_3f7[p_3f3] = {
            **p_3f5,
            "total": total,
            "ok_rate": round(p_3f5["ok"] / total, 4) if total else 0.0,
            "vlm_error": vlm, "ocr_error": ocr,
            "blame": ("ocr" if ocr > vlm else "vlm" if vlm > ocr else "tie") if (vlm or ocr) else "clean",
        }
    p_3f9 = sum(p_3f6["vlm_error"] for p_3f6 in p_3f7.values())
    p_3f8 = sum(p_3f6["ocr_error"] for p_3f6 in p_3f7.values())
    return {
        "by_field": dict(sorted(p_3f7.items())),
        "totals": {"vlm_error": p_3f9, "ocr_error": p_3f8,
                   "pages_reviewed": len(all_used)},
        "note": ("blame=ocr ⇒ span đúng nhưng Triton đọc sai chữ: thêm data KHÔNG cứu được, phải "
                 "sửa recognition. blame=vlm ⇒ VLM bịa/sai chỗ/sót: data sạch + luật lọc giúp được. "
                 "labeler=claude-opus, KHÔNG phải người ⇒ 6 trường định danh cần người điểm trước "
                 "khi gác cổng production (§G7)."),
    }

def load_jsonl_by_id(path: Path) -> dict[str, dict]:
    out: dict[str, dict] = {}
    for p_3fc in path.open(encoding="utf-8"):
        p_3fc = p_3fc.strip()
        if not p_3fc:
            continue
        try:
            p_3fd = json.loads(p_3fc)
            out[p_3fd["id"]] = p_3fd
        except (json.JSONDecodeError, KeyError):
            continue
    return out

def main(argv: list[str] | None = None) -> int:
    p_3fe = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p_3fe.add_argument("bio_jsonl")
    p_3fe.add_argument("decisions_jsonl")
    p_3fe.add_argument("--out", default="data/kie_gold.jsonl")
    p_3fe.add_argument("--report", default="data/gold_adjudication.json")
    p_3ff = p_3fe.parse_args(argv)

    p_400, p_402 = Path(p_3ff.bio_jsonl), Path(p_3ff.decisions_jsonl)
    for p_408 in (p_400, p_402):
        if not p_408.is_file():
            print(f"LỖI: không thấy {p_408}", file=sys.stderr)
            return 2

    bio = load_jsonl_by_id(p_400)
    p_405, all_used = [], []
    p_407 = 0
    for p_406 in p_402.open(encoding="utf-8"):
        p_406 = p_406.strip()
        if not p_406:
            continue
        p_401 = json.loads(p_406)
        if not p_401.get("reviewed"):
            p_407 += 1
            continue
        p_409 = p_401.get("id")
        if p_409 not in bio:
            print(f"CẢNH BÁO: quyết định cho id không có trong bio: {p_409}", file=sys.stderr)
            continue
        try:
            gold, p_40b = q_f3(bio[p_409], p_401)
        except ValueError as e:
            print(f"LỖI: {e}", file=sys.stderr)
            return 2
        p_405.append(gold)
        all_used.append(p_40b)

    if not p_405:
        print("Chưa trang nào soát (reviewed:true) — không ghi gold.", file=sys.stderr)
        return 1

    out = Path(p_3ff.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    with out.open("w", encoding="utf-8") as p_403:
        for p_404 in p_405:
            p_403.write(json.dumps(p_404, ensure_ascii=False) + "\n")

    report = q_f6(all_used)
    Path(p_3ff.report).write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")

    p_40a = report["totals"]
    print(f"→ {out} ({len(p_405)} trang gold; bỏ {p_407} chưa soát)", file=sys.stderr)
    print(f"→ {p_3ff.report}: lỗi phía VLM {p_40a['vlm_error']} · phía OCR {p_40a['ocr_error']}", file=sys.stderr)
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
