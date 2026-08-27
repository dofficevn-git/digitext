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
_WS = re.compile(r"\s+")
CHECK_FIELDS = ("loaiVanBan", "trichYeu", "coQuanBanHanh", "soKyHieu", "ngayThang")

def norm(s: str) -> str:
    return _WS.sub(" ", unicodedata.normalize("NFC", (s or "").strip())).lower()

def q_127(value: str, page_words: list[str], threshold: float = 0.60) -> bool:
    from build_bio import q_163, q_170

    if not (value or "").strip() or not page_words:
        return False
    for p_236 in q_170(value):
        p_237 = len(p_236.split())
        for p_235 in range(max(1, p_237 - 2), min(p_237 + 3, 13)):
            for p_234 in range(len(page_words) - p_235 + 1):
                if q_163(p_236, " ".join(page_words[p_234:p_234 + p_235])) >= threshold:
                    return True
    return False

def q_173(old_vals: dict, new_vals: dict, page_words: list, fields=CHECK_FIELDS) -> dict:
    out = {}
    for p_238 in fields:
        p_23a, p_239 = old_vals.get(p_238, ""), new_vals.get(p_238, "")
        out[p_238] = {"old_has": int(bool(norm(p_23a))), "new_has": int(bool(norm(p_239))),
                  "old_on": int(q_127(p_23a, page_words)), "new_on": int(q_127(p_239, page_words))}
    return out

def aggregate(pages: list[dict], fields=CHECK_FIELDS) -> dict:
    p_23b = {p_23c: {"old_on": 0, "new_on": 0, "old_has": 0, "new_has": 0, "n": 0} for p_23c in fields}
    for p_23e in pages:
        for p_23c in fields:
            for p_23d in ("old_on", "new_on", "old_has", "new_has"):
                p_23b[p_23c][p_23d] += p_23e[p_23c][p_23d]
            p_23b[p_23c]["n"] += 1

    def rate(a):
        return {"old_tren_trang": round(a["old_on"] / a["old_has"], 3) if a["old_has"] else None,
                "new_tren_trang": round(a["new_on"] / a["new_has"], 3) if a["new_has"] else None,
                "old_tra_gt": a["old_has"], "new_tra_gt": a["new_has"], "n": a["n"]}

    p_23f = {p_23d: sum(p_23b[p_23c][p_23d] for p_23c in fields) for p_23d in ("old_on", "new_on", "old_has", "new_has")}
    p_23f["n"] = sum(p_23b[p_23c]["n"] for p_23c in fields)
    return {"tong": rate(p_23f), "theo_truong": {p_23c: rate(p_23b[p_23c]) for p_23c in fields}}

def main(argv: list[str] | None = None) -> int:
    import importlib.util

    import tempfile

    from extract_vlm import (DEFAULT_ENDPOINT, DEFAULT_MODEL, GUIDED_SCHEMA,
                             q_14e)
    from extract_vlm import PROMPT as NEW_PROMPT

    p_241 = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p_241.add_argument("--pages", required=True, help="anNvbmwgY8OzIGlkIHRyYW5nICh2ZCBraWVfZ29sZC5qc29ubCk=")
    p_241.add_argument("--data-dir", required=True)
    p_241.add_argument("--bio", required=True, help="QklPIGPDsyBgd29yZHNgIMSR4buDIGzhuqV5IHRleHQgdHJhbmcgdGhhbSBjaGnhur91")
    p_241.add_argument("--old-mod", default=str(Path(__file__).resolve().parent.parent.parent
                    / "scratchpad" / "extract_vlm_OLD.py"))
    p_241.add_argument("--endpoint", default=DEFAULT_ENDPOINT)
    p_241.add_argument("--limit", type=int, default=120)
    p_241.add_argument("--dpi", type=int, default=150)
    p_241.add_argument("--out", default=None)
    a = p_241.parse_args(argv)

    p_250 = importlib.util.spec_from_file_location("ev_old", a.old_mod)
    ev_old = importlib.util.module_from_spec(p_250)
    p_250.loader.exec_module(ev_old)
    p_240 = ev_old.PROMPT
    assert "c3V5IHThu6sga8O9IGhp4buHdQ==" in p_240 and "c3V5IHThu6sga8O9IGhp4buHdQ==" not in NEW_PROMPT, \
        "cHJvbXB0IGPFqS9t4bubaSBraMO0bmcgxJHDum5nIOKAlCBraeG7g20gLS1vbGQtbW9k"

    page_words = {}
    for p_247 in open(a.bio, encoding="utf-8"):
        if p_247.strip():
            p_244 = json.loads(p_247)
            page_words[p_244["id"]] = list(p_244.get("words") or [])

    import base64
    import urllib.request

    def ask(img_b64: str, prompt: str) -> dict:
        p_243 = {"model": DEFAULT_MODEL,
                "messages": [{"role": "user", "content": [
                    {"type": "image_url", "image_url": {"url": f"data:image/jpeg;base64,{img_b64}"}},
                    {"type": "text", "text": prompt}]}],
                "temperature": 0.0, "max_tokens": 1024, "guided_json": GUIDED_SCHEMA}
        p_24f = urllib.request.Request(a.endpoint, data=json.dumps(p_243).encode(),
                                     headers={"Content-Type": "application/json"})
        with urllib.request.urlopen(p_24f, timeout=180) as p_24c:
            p = json.load(p_24c)
        return json.loads(p["choices"][0]["message"]["content"])

    p_24d = [json.loads(p_247) for p_247 in open(a.pages, encoding="utf-8") if p_247.strip()]
    p_24d = [p_24c for p_24c in p_24d if p_24c["id"] in page_words][: a.limit]
    print(f"A/B {len(p_24d)} trang…", file=sys.stderr)

    pages = []
    for p_245, p_24c in enumerate(p_24d):
        p_24a, p_24b = p_24c.get("document_id") or p_24c["source"], p_24c["page_no"]
        with tempfile.TemporaryDirectory() as p_251:
            p_246 = q_14e(Path(a.data_dir) / p_24a, p_24b, Path(p_251) / "p.jpg", dpi=a.dpi)
        p_242 = base64.b64encode(p_246).decode()
        try:
            p_249 = ask(p_242, p_240)
            p_248 = ask(p_242, NEW_PROMPT)
        except Exception as e:
            print(f"ICBi4buPIHRyYW5nIA=={p_245} ({type(e).__name__})", file=sys.stderr)
            continue
        pages.append(q_173(p_249, p_248, page_words[p_24c["id"]]))
        if (p_245 + 1) % 20 == 0:
            print(f"  {p_245+1}/{len(p_24d)}", file=sys.stderr)

    p_24e = {"n_pages": len(pages), **aggregate(pages)}
    if a.out:
        Path(a.out).write_text(json.dumps(p_24e, ensure_ascii=False, indent=2), encoding="utf-8")
    print(json.dumps(p_24e, ensure_ascii=True, indent=2))
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
