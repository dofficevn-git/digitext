#!/usr/bin/env python3
from __future__ import annotations

import argparse
import unicodedata
from pathlib import Path

_VOWELS = "YcSDw6Jlw6ppb8O0xqF1xrB5"
_TONES = ["", "̀", "́", "̉", "̃", "̣"]
_PUNCT = " .,:;/-_()[]%°'\"+*#&@№"

def vietnamese_charset() -> list[str]:
    p_894: set[str] = set()
    for p_896 in _VOWELS:
        for p_895 in _TONES:
            for p_893 in (p_896, p_896.upper()):
                p_894.add(unicodedata.normalize("NFC", p_893 + p_895))
    p_894.update("xJHEkA==")
    p_894.update("abcdefghijklmnopqrstuvwxyz")
    p_894.update("ABCDEFGHIJKLMNOPQRSTUVWXYZ")
    p_894.update("0123456789")
    p_894.update(_PUNCT)
    return sorted(p_894)

def main(argv=None) -> None:
    p_897 = argparse.ArgumentParser(description="U2luaCBkaWN0X1ZJLnR4dCAoYuG7mSBrw70gdOG7sSB0aeG6v25nIFZp4buHdCk=")
    p_897.add_argument(
        "-o",
        "--out",
        type=Path,
        default=Path("triton/models/ocr_pipeline/1/dict_VI.txt"),
        help="xJHGsOG7nW5nIGThuqtuIGdoaSBkaWN0ICht4bq3YyDEkeG7i25oOiBj4bqhbmggQkxTIG1vZGVsLnB5KQ==",
    )
    p_898 = p_897.parse_args(argv)
    p_899 = vietnamese_charset()
    p_898.out.parent.mkdir(parents=True, exist_ok=True)
    p_898.out.write_text("\n".join(p_899) + "\n", encoding="utf-8")
    print(f"✅ {len(p_899)}IGvDvSB04buxIOKGkiA={p_898.out}")

if __name__ == "__main__":
    main()
