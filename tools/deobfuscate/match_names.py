#!/usr/bin/env python3
"""Ghep ten obfuscate -> ten goc.

Y tuong: obfuscate chi DOI TEN va XOA COMMENT, khong bao gio DAO THU TU
(run_strip.py co invariant subsequence). Nen thu tu khai bao trong file
obfuscate == thu tu trong file goc. Cac ten GIU NGUYEN xuat hien o ca hai
phia -> dung lam moc neo de can hai day, roi suy ra ten bi doi nam giua.
"""
import re, json, os, glob, sys
from collections import defaultdict

ROOT = "D:/DigiText"
HERE = os.path.dirname(os.path.abspath(__file__))
SYMS = json.load(open(os.path.join(HERE, 'original_symbols.json'), encoding='utf-8'))

RE_DECL = re.compile(
    r'^\s*(?:pub(?:\([^)]*\))?\s+)?'
    r'(?:async\s+)?(?:unsafe\s+)?(?:const\s+)?(?:extern\s+"[^"]*"\s+)?'
    r'(fn|struct|enum|trait|type|union)\s+([A-Za-z_]\w*)')
RE_OBF_FN = re.compile(r'^f_[0-9a-z]+$')
RE_OBF_TY = re.compile(r'^T[0-9a-z]{1,2}$')


def obf(kind, name):
    return bool(RE_OBF_FN.match(name)) if kind == 'fn' else bool(RE_OBF_TY.match(name))


def nargs(sig: str) -> int:
    """dem tham so top-level"""
    sig = sig.strip()
    if not sig:
        return 0
    d = n = 0
    for ch in sig:
        if ch in '<([':
            d += 1
        elif ch in '>)]':
            d -= 1
        elif ch == ',' and d == 0:
            n += 1
    return n + 1


def src_decls(path):
    """[(line, kind, name)] theo thu tu xuat hien"""
    out = []
    with open(path, encoding='utf-8', errors='replace') as fh:
        for i, line in enumerate(fh, 1):
            m = RE_DECL.match(line)
            if m:
                out.append((i, m.group(1), m.group(2)))
    return out


mapping = {}          # obf_name -> {'orig':..., 'file':..., 'how':...}
conflicts = defaultdict(set)
stats = defaultdict(int)

for path in sorted(glob.glob(os.path.join(ROOT, 'crates/*/src/**/*.rs'), recursive=True)):
    rel = os.path.relpath(path, ROOT).replace('\\', '/')
    sym = SYMS.get(rel)
    if not sym:
        stats['file_khong_symbol'] += 1
        continue
    stats['file_co_symbol'] += 1

    # phia obfuscate: chi lay ham, theo thu tu dong
    o_fns = [(ln, nm) for ln, k, nm in src_decls(path) if k == 'fn']
    # phia goc: ham tu binary, theo thu tu dong, bo trung ten
    seen = set()
    g_fns = []
    for ln, fq, args, ret in sorted(sym['functions']):
        short = fq.rsplit('::', 1)[-1]
        if short in seen:
            continue
        seen.add(short)
        g_fns.append((ln, short, nargs(args)))

    if not o_fns or not g_fns:
        continue

    # --- can hai day bang cac ten giu nguyen lam moc ---
    g_index = {nm: i for i, (_, nm, _) in enumerate(g_fns)}
    anchors = []            # (i_obf, i_goc)
    for i, (_, nm) in enumerate(o_fns):
        if not obf('fn', nm) and nm in g_index:
            anchors.append((i, g_index[nm]))
    # giu day moc tang dan ca hai chieu
    mono = []
    for a in anchors:
        if not mono or (a[0] > mono[-1][0] and a[1] > mono[-1][1]):
            mono.append(a)
    stats['moc_neo'] += len(mono)

    # --- ghep phan nam giua cac moc ---
    bounds = [(-1, -1)] + mono + [(len(o_fns), len(g_fns))]
    for (oa, ga), (ob, gb) in zip(bounds, bounds[1:]):
        o_seg = [(i, o_fns[i]) for i in range(oa + 1, ob)]
        g_seg = [(j, g_fns[j]) for j in range(ga + 1, gb)]
        o_hidden = [(i, d) for i, d in o_seg if obf('fn', d[1])]
        g_free = [(j, d) for j, d in g_seg if d[1] not in
                  {nm for _, (_, nm) in o_seg if not obf('fn', nm)}]
        if len(o_hidden) == len(g_free) and o_hidden:
            how = 'doan-khop-chinh-xac'
        elif o_hidden and g_free:
            how = 'doan-uoc-luong'
        else:
            continue
        for (_, (oln, onm)), (_, (gln, gnm, gn)) in zip(o_hidden, g_free):
            if onm in mapping and mapping[onm]['orig'] != gnm:
                conflicts[onm].add(gnm)
                conflicts[onm].add(mapping[onm]['orig'])
                continue
            mapping[onm] = {'orig': gnm, 'file': rel, 'line_obf': oln,
                            'line_goc': gln, 'how': how}
            stats[how] += 1

for k in list(mapping):
    if k in conflicts:
        del mapping[k]

out = os.path.join(HERE, 'name_map.json')
json.dump(mapping, open(out, 'w', encoding='utf-8'), ensure_ascii=False, indent=1)

print(f"file co symbol        : {stats['file_co_symbol']}")
print(f"file khong symbol     : {stats['file_khong_symbol']}")
print(f"moc neo (ten giu nguyen): {stats['moc_neo']}")
print(f"khop chinh xac        : {stats['doan-khop-chinh-xac']}")
print(f"uoc luong             : {stats['doan-uoc-luong']}")
print(f"xung dot (da loai)    : {len(conflicts)}")
print(f"TONG ANH XA           : {len(mapping)}")
print(f"-> {out}")
print("\n--- MAU ---")
for k, v in list(mapping.items())[:15]:
    print(f"  {k:10s} -> {v['orig']:34s} {v['file'].split('/')[-1]:20s} [{v['how']}]")
