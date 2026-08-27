#!/usr/bin/env python3
"""Anh xa kieu T* -> ten kieu goc, bang cach doi chieu CHU KY cua cac ham da khop.

Voi moi cap ham (f_xxx -> ten_goc) da co trong name_map.json:
  - lay chu ky obfuscate tu source  -> day token kieu T*
  - lay chu ky goc tu binary (gdb)  -> day token kieu digitext_*::TenKieu
Neu hai day dai bang nhau thi zip theo thu tu -> mot phieu bau cho moi cap.
Gom phieu tren toan bo 542 ham; lay da so; loai cac truong hop mau thuan.
"""
import re, json, os, glob
from collections import defaultdict, Counter

ROOT = "D:/DigiText"
HERE = os.path.dirname(os.path.abspath(__file__))
SYMS = json.load(open(os.path.join(HERE, 'original_symbols.json'), encoding='utf-8'))
NMAP = json.load(open(os.path.join(HERE, 'name_map.json'), encoding='utf-8'))

RE_OBF_TY = re.compile(r'\bT[0-9a-z]{1,2}\b')
RE_ORIG_TY = re.compile(r'\bdigitext_\w+(?:::\w+)*::([A-Z]\w*)\b')
RE_IDENT = re.compile(r'[A-Za-z_]\w*')


def obf_signature(path, line):
    """Doc chu ky ham bat dau tai `line` (1-indexed), gom nhieu dong den khi can bang ngoac."""
    with open(path, encoding='utf-8', errors='replace') as fh:
        lines = fh.readlines()
    buf, depth, started = [], 0, False
    for raw in lines[line - 1: line + 14]:
        buf.append(raw)
        for ch in raw:
            if ch in '(<[':
                depth += 1
                started = True
            elif ch in ')>]':
                depth -= 1
        if started and depth <= 0 and ('{' in raw or ';' in raw):
            break
    s = ' '.join(x.strip() for x in buf)
    # cat phan than ham
    i = s.find('{')
    return s[:i] if i > 0 else s


votes = defaultdict(Counter)
paired = 0

# index: file -> {ten_ngan_goc: (args, ret)}
by_file = {}
for rel, d in SYMS.items():
    m = {}
    for ln, fq, args, ret in d['functions']:
        m.setdefault(fq.rsplit('::', 1)[-1], (args, ret))
    by_file[rel] = m

for obf_name, info in NMAP.items():
    rel = info['file']
    path = os.path.join(ROOT, rel)
    if not os.path.exists(path):
        continue
    sig_o = obf_signature(path, info['line_obf'])
    got = by_file.get(rel, {}).get(info['orig'])
    if not got:
        continue
    sig_g = f"{got[0]} -> {got[1]}"

    t_o = RE_OBF_TY.findall(sig_o)
    # Kieu nao con giu nguyen ten thi xuat hien NGUYEN VAN trong chu ky obfuscate
    # -> no khong sinh ra token T*, phai loai khoi day goc truoc khi can.
    idents_o = set(RE_IDENT.findall(sig_o))
    t_g = [t for t in RE_ORIG_TY.findall(sig_g) if t not in idents_o]
    if not t_o or len(t_o) != len(t_g):
        continue
    paired += 1
    for a, b in zip(t_o, t_g):
        votes[(rel.split('/')[1], a)][b] += 1

tmap, ambiguous = {}, {}
for (crate, obf), c in votes.items():
    best, n = c.most_common(1)[0]
    total = sum(c.values())
    if len(c) == 1 or n / total >= 0.75:
        tmap.setdefault(crate, {})[obf] = {'orig': best, 'votes': n, 'total': total}
    else:
        ambiguous[f"{crate}:{obf}"] = dict(c)

# kiem tra: mot ten goc co bi nhieu ten obfuscate cung tro toi khong?
rev = defaultdict(list)
for crate, d in tmap.items():
    for obf, v in d.items():
        rev[(crate, v['orig'])].append(obf)
dup = {k: v for k, v in rev.items() if len(v) > 1}

out = os.path.join(HERE, 'type_map.json')
json.dump(tmap, open(out, 'w', encoding='utf-8'), ensure_ascii=False, indent=1)

n_ty = sum(len(d) for d in tmap.values())
print(f"ham doi chieu duoc chu ky : {paired}")
print(f"kieu co phieu             : {len(votes)}")
print(f"kieu chot duoc            : {n_ty}")
print(f"kieu mau thuan (bo)       : {len(ambiguous)}")
print(f"ten goc bi trung dich     : {len(dup)}   <- phai la 0")
print(f"-> {out}\n--- MAU ---")
flat = [(c, k, v) for c, d in tmap.items() for k, v in d.items()]
for c, k, v in sorted(flat, key=lambda x: -x[2]['votes'])[:20]:
    print(f"  {c:22s} {k:6s} -> {v['orig']:28s} ({v['votes']}/{v['total']})")
for label, coll in (("MAU THUAN", ambiguous), ("TRUNG DICH", dup)):
    if coll:
        print(f"--- {label} ---")
        for k, v in list(coll.items())[:8]:
            print(f"  {k} -> {v}")
