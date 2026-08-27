#!/usr/bin/env python3
""
from __future__ import annotations

import argparse
import json
import re
import sys
import unicodedata
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from extract_vlm import PAGE_IDENTIFIER_FIELDS

_WS = re.compile(r"\s+")

def q_171(word_ids: list, pred_labels: list[str], n_words: int) -> list[str]:
    tags = ["O"] * n_words
    p_32b = set()
    for p_32a, p_32c in enumerate(word_ids):
        if p_32c is None or p_32c in p_32b or p_32c >= n_words or p_32a >= len(pred_labels):
            continue
        p_32b.add(p_32c)
        tags[p_32c] = pred_labels[p_32a]
    return tags

def q_12c(s: str) -> str:
    return _WS.sub(" ", unicodedata.normalize("NFC", s or "")).strip()

def field_of(tag: str) -> str | None:
    if not tag or tag == "O" or "-" not in tag:
        return None
    p_32f, p_32d, p_32e = tag.partition("-")
    return p_32e if p_32f in ("B", "I") and p_32e else None

def q_17e(words: list[str], tags: list[str]) -> dict[str, str]:
    out: dict[str, list[str]] = {}
    p_331: str | None = None
    p_330: list[str] = []
    for p_337, p_335 in zip(words, list(tags) + ["O"] * max(0, len(words) - len(tags))):
        p_332 = field_of(p_335)
        p_334 = p_335.startswith("B-") or (p_332 is not None and p_332 != p_331)
        if p_332 is None or p_334:
            if p_331 and p_330:
                out.setdefault(p_331, []).append(" ".join(p_330))
            p_331, p_330 = (p_332, [p_337] if p_332 else [])
        else:
            p_330.append(p_337)
    if p_331 and p_330:
        out.setdefault(p_331, []).append(" ".join(p_330))
    return {p_333: q_12c(" ".join(p_336)) for p_333, p_336 in out.items() if q_12c(" ".join(p_336))}

def q_15f(gold: dict[str, str], pred: dict[str, str]) -> dict[str, dict]:
    from cer import q_110

    p_33b: dict[str, dict] = {}
    for p_338, p_339 in gold.items():
        p_33a = pred.get(p_338, "")
        p_33b[p_338] = {"edit": q_110(p_33a, p_339), "n_gold": len(p_339), "exact": int(p_33a == p_339)}
    return p_33b

def aggregate(pages: list[dict[str, dict]]) -> dict:
    p_343: dict[str, dict] = {}
    for p_342 in pages:
        for p_33e, s in p_342.items():
            p_33c = p_343.setdefault(p_33e, {"edit": 0, "n_gold": 0, "exact": 0, "n": 0})
            p_33c["edit"] += s["edit"]
            p_33c["n_gold"] += s["n_gold"]
            p_33c["exact"] += s["exact"]
            p_33c["n"] += 1

    def blk(names):
        p_33d = sum(p_343[p_33e]["edit"] for p_33e in names)
        p_33f = sum(p_343[p_33e]["n_gold"] for p_33e in names)
        p_345 = sum(p_343[p_33e]["exact"] for p_33e in names)
        n = sum(p_343[p_33e]["n"] for p_33e in names)
        return {"cer": round(p_33d / p_33f, 4) if p_33f else None,
                "exact": round(p_345 / n, 4) if n else None, "n": n}

    p_340 = [p_33e for p_33e in p_343 if p_33e in PAGE_IDENTIFIER_FIELDS]
    p_341 = [p_33e for p_33e in p_343 if p_33e not in PAGE_IDENTIFIER_FIELDS]
    return {
        "overall": blk(list(p_343)),
        "dinh_danh": blk(p_340),
        "nhom_thuong": blk(p_341),
        "by_field": {p_33e: {"cer": round(p_344["edit"] / p_344["n_gold"], 4) if p_344["n_gold"] else None,
                         "exact": round(p_344["exact"] / p_344["n"], 4) if p_344["n"] else None,
                         "n": p_344["n"]}
                     for p_33e, p_344 in sorted(p_343.items())},
    }

def q_13f(records: list[dict], model_dir: str, max_len: int) -> list[list[str]]:
    import torch
    from transformers import AutoTokenizer, LayoutLMv3ForTokenClassification

    from eval_kie import LABEL2ID, q_140

    p_350 = AutoTokenizer.from_pretrained(model_dir, add_prefix_space=True)
    model = LayoutLMv3ForTokenClassification.from_pretrained(model_dir).eval()
    p_346 = "cuda" if torch.cuda.is_available() else "cpu"
    model.to(p_346)
    p_34a = {p_349: p_34c for p_34c, p_349 in LABEL2ID.items()}

    out: list[list[str]] = []
    with torch.no_grad():
        for p_34f in records:
            p_348 = q_140(p_34f)
            p_347 = p_350(p_348["words"], boxes=p_348["bboxes"], word_labels=p_348["labels"],
                      truncation=True, padding="max_length", max_length=max_len,
                      return_tensors="pt")
            word_ids = p_347.word_ids(0)
            p_347.pop("labels")
            p_34e = model(**{p_34b: p_351.to(p_346) for p_34b, p_351 in p_347.items()}).logits.argmax(-1)[0].cpu()
            out.append(q_171(word_ids, [p_34a[p_34d] for p_34d in p_34e.tolist()],
                                          len(p_348["words"])))
    return out

def main(argv: list[str] | None = None) -> int:
    from eval_kie import q_121

    p_353 = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p_353.add_argument("bio_jsonl", help="QklPIGPhu6dhIE3hu5hUIGzhuqduIE9DUiAobmjDoW5oIGPhuqduIGNo4bqlbSk=")
    p_353.add_argument("--gold", required=True, help="a2llX2dvbGQuanNvbmwg4oCUIG5ndeG7k24gR0nDgSBUUuG7iiB0aOG6rXQ=")
    p_353.add_argument("--model-dir", required=True)
    p_353.add_argument("--tag", required=True, help="bmjDo24gbmjDoW5oLCB2ZCBvY3JfY3UgLyBvY3JfbW9p")
    p_353.add_argument("--out", required=True)
    p_353.add_argument("--max-len", type=int, default=512)
    p_352 = p_353.parse_args(argv)

    p_356 = {p_359["id"]: p_359 for p_359 in q_121(Path(p_352.gold))}
    p_354 = [p_359 for p_359 in q_121(Path(p_352.bio_jsonl)) if p_359["id"] in p_356]
    if not p_354:
        print("LOI: khong trang nao cua BIO trung id voi gold", file=sys.stderr)
        return 2

    p_358 = q_13f(p_354, p_352.model_dir, p_352.max_len)
    pages = []
    for p_35a, tags in zip(p_354, p_358):
        p_355 = q_17e(p_356[p_35a["id"]]["words"], p_356[p_35a["id"]]["tags"])
        p_357 = q_17e(p_35a["words"], tags)
        if p_355:
            pages.append(q_15f(p_355, p_357))

    p_35b = {"meta": {"tag": p_352.tag, "bio": Path(p_352.bio_jsonl).name, "model_dir": p_352.model_dir,
                    "n_pages": len(pages), "n_gold_pages": len(p_356)},
           **aggregate(pages)}
    Path(p_352.out).write_text(json.dumps(p_35b, ensure_ascii=False, indent=2), encoding="utf-8")
    print(json.dumps(p_35b, ensure_ascii=True, indent=2))
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
