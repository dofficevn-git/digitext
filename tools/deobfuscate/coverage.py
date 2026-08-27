#!/usr/bin/env python3
"""Do do phu: bao nhieu dinh danh bi obfuscate co the anh xa ve ten goc."""
import re, json, os, glob
from collections import defaultdict

ROOT = "D:/DigiText"
SYMS = json.load(open(os.path.join(os.path.dirname(os.path.abspath(__file__)),
                                   'original_symbols.json'), encoding='utf-8'))

# dinh danh da bi doi ten: f_xxx (ham), T<xx> (kieu), C_XX (hang)
RE_DECL = re.compile(
    r'^\s*(?:pub(?:\([^)]*\))?\s+)?'
    r'(?:async\s+)?(?:unsafe\s+)?(?:const\s+)?(?:extern\s+"[^"]*"\s+)?'
    r'(fn|struct|enum|trait|type|union)\s+'
    r'([A-Za-z_]\w*)'
)
RE_OBF_FN = re.compile(r'^f_[0-9a-z]+$')
RE_OBF_TY = re.compile(r'^T[0-9a-z][0-9a-z]?$|^T[a-z]\w$')
RE_OBF_CONST = re.compile(r'^C_[0-9A-Z]+$')


def is_obf(kind, name):
    if kind == 'fn':
        return bool(RE_OBF_FN.match(name))
    return bool(RE_OBF_TY.match(name))


rows = []
tot = defaultdict(int)

for path in sorted(glob.glob(os.path.join(ROOT, 'crates/*/src/**/*.rs'), recursive=True)):
    rel = os.path.relpath(path, ROOT).replace('\\', '/')
    decls = []
    with open(path, encoding='utf-8', errors='replace') as fh:
        for i, line in enumerate(fh, 1):
            m = RE_DECL.match(line)
            if m:
                decls.append((i, m.group(1), m.group(2)))

    obf_fn = [d for d in decls if d[1] == 'fn' and is_obf('fn', d[2])]
    obf_ty = [d for d in decls if d[1] in ('struct', 'enum', 'trait', 'type', 'union')
              and is_obf('ty', d[2])]
    keep_fn = [d for d in decls if d[1] == 'fn' and not is_obf('fn', d[2])]
    keep_ty = [d for d in decls if d[1] in ('struct', 'enum', 'trait', 'type', 'union')
               and not is_obf('ty', d[2])]

    s = SYMS.get(rel, {'functions': [], 'types': []})
    rows.append((rel, len(obf_fn), len(obf_ty), len(keep_fn), len(keep_ty),
                 len(s['functions']), len(s['types'])))
    tot['obf_fn'] += len(obf_fn); tot['obf_ty'] += len(obf_ty)
    tot['keep_fn'] += len(keep_fn); tot['keep_ty'] += len(keep_ty)
    tot['sym_fn'] += len(s['functions']); tot['sym_ty'] += len(s['types'])
    tot['files'] += 1
    if s['functions'] or s['types']:
        tot['files_covered'] += 1

print(f"{'file':52s} {'obfFn':>6s} {'obfTy':>6s} {'symFn':>6s} {'symTy':>6s}")
for r in sorted(rows, key=lambda x: -(x[1] + x[2]))[:22]:
    print(f"{r[0]:52s} {r[1]:6d} {r[2]:6d} {r[5]:6d} {r[6]:6d}")

print(f"""
=========== TONG ===========
file .rs trong crates/       : {tot['files']}
  co du lieu symbol          : {tot['files_covered']}

DINH DANH BI DOI TEN
  ham   f_*                  : {tot['obf_fn']}
  kieu  T*                   : {tot['obf_ty']}
  cong                       : {tot['obf_fn'] + tot['obf_ty']}

DINH DANH GIU NGUYEN (khong can anh xa)
  ham                        : {tot['keep_fn']}
  kieu                       : {tot['keep_ty']}

TEN GOC LAY DUOC TU BINARY
  ham                        : {tot['sym_fn']}
  kieu                       : {tot['sym_ty']}
  cong                       : {tot['sym_fn'] + tot['sym_ty']}
""")
