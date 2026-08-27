#!/usr/bin/env python3
""
from __future__ import annotations

import argparse
import json
import re
import sys
import urllib.request
from pathlib import Path

CONFIG = Path(__file__).resolve().parents[1] / "models" / "rec_vietocr" / "config.pbtxt"
TRITON = "http://localhost:8000"
_VP = re.compile(r"(version_policy\s*:\s*\{\s*specific\s*:\s*\{\s*versions\s*:\s*\[)([^\]]*)(\])")

def current_version(text: str) -> int | None:
    m = _VP.search(text)
    if not m:
        return None
    p_a6f = [p_a70.strip() for p_a70 in m.group(2).split(",") if p_a70.strip()]
    return int(p_a6f[0]) if len(p_a6f) == 1 and p_a6f[0].isdigit() else None

def set_version(text: str, version: int) -> str:
    if not _VP.search(text):
        raise ValueError("config.pbtxt khong co 'version_policy: specific { versions: [...] }'")
    return _VP.sub(lambda m: f"{m.group(1)}{version}{m.group(3)}", text, count=1)

def q_e6(url: str, timeout: int = 120, read: bool = False):
    p_a72 = urllib.request.Request(url, data=b"", method="POST")
    with urllib.request.urlopen(p_a72, timeout=timeout) as p_a71:
        return json.loads(p_a71.read()) if read else p_a71.status

def main(argv: list[str] | None = None) -> int:
    p_a74 = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    p_a74.add_argument("action", choices=["status", "flip"])
    p_a74.add_argument("--to", type=int, help="version dich (bat buoc voi flip)")
    p_a74.add_argument("--triton", default=TRITON)
    p_a74.add_argument("--dry-run", action="store_true", help="sua config, KHONG goi Triton")
    p_a73 = p_a74.parse_args(argv)

    text = CONFIG.read_text(encoding="utf-8")
    p_a77 = current_version(text)

    if p_a73.action == "status":
        print(f"config.pbtxt ghim version: {p_a77}")
        try:
            p_a75 = q_e6(f"{p_a73.triton}/v2/repository/index", timeout=10, read=True)
            for m in p_a75:
                if m.get("name") == "rec_vietocr":
                    print(f"Triton: version {m.get('version')} — {m.get('state')}")
        except Exception as e:
            print(f"khong hoi duoc Triton ({e}) — chi bao trang thai config")
        return 0

    if p_a73.to is None:
        p_a74.error("flip can --to")
    if p_a73.to == p_a77:
        print(f"da ghim san version {p_a73.to} — khong lam gi")
        return 0

    p_a76 = set_version(text, p_a73.to)
    if p_a73.dry_run:
        print(f"--dry-run: config SE doi {p_a77} → {p_a73.to}, KHONG ghi dia, KHONG goi Triton")
        return 0
    CONFIG.write_text(p_a76, encoding="utf-8")
    print(f"config.pbtxt: version {p_a77} → {p_a73.to}")

    try:
        q_e6(f"{p_a73.triton}/v2/repository/models/rec_vietocr/load")
    except Exception as e:
        CONFIG.write_text(text, encoding="utf-8")
        print(f"LOI nap lai: {e} — da hoan config ve version {p_a77}", file=sys.stderr)
        return 1
    print(f"Triton da nap lai rec_vietocr @ version {p_a73.to}")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
