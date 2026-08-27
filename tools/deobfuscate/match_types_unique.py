#!/usr/bin/env python3
"""Vet them kieu bang LOAI TRU trong pham vi tung file.

gdb `info types` gan ten kieu goc theo file. Neu trong mot file chi con
DUNG MOT kieu T* chua anh xa va DUNG MOT ten goc chua duoc dung den,
thi cap do la duy nhat -> chot duoc ma khong can thu tu.
"""
import re, json, os, glob
from collections import defaultdict

ROOT = "D:/DigiText"
HERE = os.path.dirname(os.path.abspath(__file__))
SYMS = json.load(open(os.path.join(HERE, 'original_symbols.json'), encoding='utf-8'))
TMAP = json.load(open(os.path.join(HERE, 'type_map.json'), encoding='utf-8'))

RE_TY_DECL = re.compile(
    r'^\s*(?:pub(?:\([^)]*\))?\s+)?(struct|enum|trait|union)\s+(T[0-9a-z]{1,2})\b')
RE_IDENT = re.compile(r'[A-Za-z_]\w*')

added = 0
report = []

for path in sorted(glob.glob(os.path.join(ROOT, 'crates/*/src/**/*.rs'), recursive=True)):
    rel = os.path.relpath(path, ROOT).replace('\\', '/')
    crate = rel.split('/')[1]
    sym = SYMS.get(rel)
    if not sym or not sym['types']:
        continue

    text = open(path, encoding='utf-8', errors='replace').read()
    idents = set(RE_IDENT.findall(text))

    obf_here = [m.group(2) for line in text.splitlines()
                if (m := RE_TY_DECL.match(line))]
    done = TMAP.get(crate, {})
    pending = [t for t in obf_here if t not in done]

    # ten goc thuoc file nay, chua xuat hien nguyen van trong source
    # (xuat hien nguyen van = kieu do giu nguyen ten, khong phai dich)
    used = {v['orig'] for v in done.values()}
    cands = sorted({t.rsplit('::', 1)[-1] for t in sym['types']}
                   - idents - used)

    if len(pending) == 1 and len(cands) == 1:
        TMAP.setdefault(crate, {})[pending[0]] = {
            'orig': cands[0], 'votes': 1, 'total': 1, 'how': 'loai-tru-file'}
        report.append((rel, pending[0], cands[0]))
        added += 1

json.dump(TMAP, open(os.path.join(HERE, 'type_map.json'), 'w', encoding='utf-8'),
          ensure_ascii=False, indent=1)

print(f"chot them bang loai tru : {added}")
for rel, o, g in report[:25]:
    print(f"  {rel.split('/',2)[-1]:42s} {o:5s} -> {g}")
print(f"\nTONG KIEU DA ANH XA     : {sum(len(d) for d in TMAP.values())}")
