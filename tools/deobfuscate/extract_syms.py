#!/usr/bin/env python3
"""Trich xuat ten goc + file:dong tu output gdb, gom lai thanh bang sach."""
import re, json, sys, glob, os
from collections import defaultdict

SYM = r"D:/DigiText/digitext-rescue/symbols"

# "File crates/digitext-db/src/access.rs:"  -> bat dau khoi
RE_FILE = re.compile(r'^File (crates/[^:]+\.rs):$')
RE_FILE_ANY = re.compile(r'^File (.+):$')
# "78:\tfn digitext_dispatcher::project_stats_with_pending(...) -> ...;"
RE_FN = re.compile(r'^(\d+):\s+(?:static\s+)?fn\s+([A-Za-z_][\w:]*)\s*\((.*)\)\s*->\s*(.+);$')
RE_FN_NORET = re.compile(r'^(\d+):\s+(?:static\s+)?fn\s+([A-Za-z_][\w:]*)\s*\((.*)\);$')
# dong type: "\tdigitext_db::access::AccessRequestRow"
RE_TYPE = re.compile(r'^\s+([A-Za-z_][\w:]*(?:::[\w]+)*)$')

# bo cac phan sinh boi compiler
NOISE = ('{closure', '{async_fn', '{async_block', '{constant', '{impl', '{opaque')


def clean(name: str) -> str | None:
    if any(n in name for n in NOISE):
        return None
    return name


def parse(path):
    """-> (funcs, types) ; funcs: file -> [(line, name, args, ret)]"""
    funcs = defaultdict(list)
    types = defaultdict(set)
    cur = None
    mode = 'fn'
    with open(path, encoding='utf-8', errors='replace') as fh:
        for raw in fh:
            line = raw.rstrip('\n')
            if line.startswith('All types matching'):
                mode = 'type'
                cur = None
                continue
            if line.startswith('All functions matching'):
                mode = 'fn'
                cur = None
                continue
            m = RE_FILE_ANY.match(line)
            if m:
                p = m.group(1)
                # pseudo-file do macro sinh: "lib.rs/@/<hash>" -> quy ve file that
                if '/@/' in p:
                    p = p.split('/@/', 1)[0]
                cur = p if p.startswith('crates/') else None
                continue
            if cur is None:
                continue
            if mode == 'fn':
                m = RE_FN.match(line) or RE_FN_NORET.match(line)
                if m:
                    ln, name = int(m.group(1)), m.group(2)
                    if clean(name):
                        args = m.group(3)
                        ret = m.group(4) if m.lastindex >= 4 else ''
                        funcs[cur].append((ln, name, args, ret))
            else:
                m = RE_TYPE.match(line)
                if not m:
                    continue
                name = m.group(1)
                if not clean(name):
                    continue
                # chi giu ten KIEU that: doan cuoi viet hoa, va thuoc crate digitext
                last = name.rsplit('::', 1)[-1]
                if last[:1].isupper() and name.startswith('digitext_'):
                    types[cur].add(name)
    return funcs, types


all_funcs = defaultdict(dict)   # file -> {(line,name): (args,ret)}
all_types = defaultdict(set)

for p in sorted(glob.glob(os.path.join(SYM, '*-gdb.txt'))):
    f, t = parse(p)
    for k, v in f.items():
        for ln, name, args, ret in v:
            all_funcs[k][(ln, name)] = (args, ret)
    for k, v in t.items():
        all_types[k] |= v
    print(f"  {os.path.basename(p):34s} files={len(f):4d} fns={sum(len(x) for x in f.values()):6d}",
          file=sys.stderr)

out = {}
for f in sorted(set(all_funcs) | set(all_types)):
    out[f] = {
        'functions': sorted([[ln, n, a, r] for (ln, n), (a, r) in all_funcs[f].items()]),
        'types': sorted(all_types[f]),
    }

dest = os.path.join(os.path.dirname(os.path.abspath(__file__)), 'original_symbols.json')
with open(dest, 'w', encoding='utf-8') as fh:
    json.dump(out, fh, ensure_ascii=False, indent=1)

nf = sum(len(v['functions']) for v in out.values())
nt = sum(len(v['types']) for v in out.values())
print(f"\nfile nguon : {len(out)}")
print(f"ham        : {nf}")
print(f"kieu       : {nt}")
print(f"-> {dest}")
