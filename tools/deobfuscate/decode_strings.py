#!/usr/bin/env python3
"""Giai ma cac chuoi base64 do dev/obfuscate/encode_strings.mjs sinh ra.

Cong cu ma hoa chi dong vao literal CO DAU TIENG VIET, phep bien doi la base64
cua UTF-8 bytes, va KHONG co decoder. Vi vay phep nghich co tieu chi tuyet doi:

    giai base64 -> UTF-8 hop le VA co dau tieng Viet  => khoi phuc
    nguoc lai                                          => khong dung

Do tren toan cay: moi chuoi giai duoc deu co dau tieng Viet; phan con lai khong
ra UTF-8 (base64 that: khoa, hash). Khong co vung xam.

QUAN TRONG: doc nguon tu `git show <REF>:<path>` chu khong tu thu muc lam viec,
de chay lai duoc nhieu lan ma van xuat phat tu ban goc.

Bo quet literal duoc viet tay (khong dung regex) vi regex khong phan biet noi
raw string `r#"..."#` voi chuoi thuong — nham lan o day dan toi bo qua buoc
escape va sinh ra ma khong bien dich duoc.

    python decode_strings.py                     # thu, doc tu commit goc
    python decode_strings.py --apply             # ghi de
    python decode_strings.py --from-worktree     # doc tu cay lam viec (luy dang)

GIOI HAN DA BIET: scan_python khong theo doi do sau `{}` cua f-string, nen
f-string co chuoi long CUNG LOAI dau nhay (Python 3.12+) se bi dut sai cho:

    print(f"  {"<base64>":12} {n}")

Chi gap DUNG MOT lan trong cay nguon (ml/bench/harness/golden.py) va da sua
tay. Neu ve sau gap them, hoac sua tay, hoac bo sung dem do sau nhu da lam cho
template long trong scan_clike.
"""
import io
import re, sys, glob, base64, os, subprocess, unicodedata
from collections import Counter

ROOT = "D:/DigiText"
REF = "8b5e0e2"          # commit anh chup nguon obfuscate ban dau
APPLY = '--apply' in sys.argv
# Mac dinh doc nguon tu commit goc => chay lai bao nhieu lan cung ra cung ket qua.
# NHUNG neu cay lam viec da co thay doi khac chua commit, doc tu commit se GHI DE
# mat chung. Luc do dung --from-worktree: giai ma la phep luy dang (chuoi da giai
# khong con doan base64 nao) nen chay tren cay lam viec van an toan.
FROM_WORKTREE = '--from-worktree' in sys.argv

TARGETS = [
    'crates/*/src/**/*.rs',
    'desktop/src-tauri/src/**/*.rs',
    'desktop/src/**/*.ts',
    'desktop/src/**/*.tsx',
    'ml/**/*.py',
    'triton/**/*.py',
]

# Tap chu tieng Viet CHINH XAC. Dai Latin Extended rong sinh duong tinh gia:
# "User" giai base64 ra 'Rǫ', ma ǫ khong phai chu Viet. Tieu chi dung:
# co it nhat MOT chu Viet co dau, VA moi ky tu phi-ASCII deu thuoc tap nay.
_VN = ('àáảãạăằắẳẵặ'
       'âầấẩẫậèéẻẽẹ'
       'êềếểễệìíỉĩị'
       'òóỏõọôồốổỗộ'
       'ơờớởỡợùúủũụ'
       'ưừứửữựỳýỷỹỵ'
       'đ')
VIET = re.compile('[' + re.escape(_VN + _VN.upper()) + ']')
_VN_OK = set(_VN + _VN.upper()) | {chr(c) for c in range(0x300, 0x324)}
# Toi thieu 6: manh ngan nhu "IHbDoCA=" (' va ') nam giua hai placeholder
# cua format string. Nguong 8 truoc day bo sot 72 cho nhu vay.
B64_RUN = re.compile(r'[A-Za-z0-9+/]{6,}={0,2}')
IDENT = re.compile(r'[A-Za-z0-9_]')

# ---------------------------------------------------------------- bo quet


def scan_rust(s):
    """yield (lit_start, lit_end, body_start, body_end, is_raw)"""
    i, n = 0, len(s)
    while i < n:
        c = s[i]
        if c == '/' and i + 1 < n and s[i + 1] == '/':
            i = s.find('\n', i);  i = n if i < 0 else i + 1;  continue
        if c == '/' and i + 1 < n and s[i + 1] == '*':
            j = s.find('*/', i + 2);  i = n if j < 0 else j + 2;  continue
        # raw string: [b]r#*"..."#*  — 'r' phai khong nam trong dinh danh dai hon
        if c in 'rb':
            j = i
            if c == 'b' and j + 1 < n and s[j + 1] == 'r':
                j += 1
            if s[j] == 'r' and (i == 0 or not IDENT.match(s[i - 1])):
                k = j + 1
                while k < n and s[k] == '#':
                    k += 1
                if k < n and s[k] == '"':
                    hashes = k - j - 1
                    close = '"' + '#' * hashes
                    e = s.find(close, k + 1)
                    if e >= 0:
                        yield (i, e + len(close), k + 1, e, True)
                        i = e + len(close);  continue
        if c == '"':
            j = i + 1
            while j < n:
                if s[j] == '\\':
                    j += 2;  continue
                if s[j] == '"':
                    break
                j += 1
            if j < n:
                yield (i, j + 1, i + 1, j, False)
                i = j + 1;  continue
        if c == "'":
            # char literal hoac lifetime — bo qua, khong chua chuoi can giai
            j = i + 1
            if j < n and s[j] == '\\':
                j += 2
            elif j < n:
                j += 1
            i = j + 1 if j < n and s[j] == "'" else i + 1
            continue
        i += 1


def scan_clike(s):
    """TS/JS: " ' ` + comment"""
    i, n = 0, len(s)
    while i < n:
        c = s[i]
        if c == '/' and i + 1 < n and s[i + 1] == '/':
            i = s.find('\n', i);  i = n if i < 0 else i + 1;  continue
        if c == '/' and i + 1 < n and s[i + 1] == '*':
            j = s.find('*/', i + 2);  i = n if j < 0 else j + 2;  continue
        if c == '`':
            # Template literal: backtick chi la dau DONG khi dang o ngoai moi
            # ${...}. Bo qua dieu nay se dut o backtick MO cua template long
            # ben trong, vd:
            #   `${a} x${b > 0 ? ` · ${c}<base64>` : ""}`
            j, depth = i + 1, 0
            while j < n:
                ch = s[j]
                if ch == '\\':
                    j += 2;  continue
                if ch == '$' and j + 1 < n and s[j + 1] == '{':
                    depth += 1;  j += 2;  continue
                if depth > 0 and ch == '}':
                    depth -= 1;  j += 1;  continue
                if depth == 0 and ch == '`':
                    break
                j += 1
            if j < n:
                yield (i, j + 1, i + 1, j, False)
                i = j + 1;  continue
        if c in '"\'':
            j = i + 1
            while j < n:
                if s[j] == '\\':
                    j += 2;  continue
                if s[j] == c:
                    break
                j += 1
            if j < n:
                yield (i, j + 1, i + 1, j, False)
                i = j + 1;  continue
        i += 1


def scan_python(s):
    i, n = 0, len(s)
    while i < n:
        c = s[i]
        if c == '#':
            i = s.find('\n', i);  i = n if i < 0 else i + 1;  continue
        if c in '"\'':
            triple = s[i:i + 3] in ('"""', "'''")
            q = s[i:i + 3] if triple else c
            j = i + len(q)
            while j < n:
                if s[j] == '\\':
                    j += 2;  continue
                if s.startswith(q, j):
                    break
                j += 1
            if j < n:
                yield (i, j + len(q), i + len(q), j, triple)
                i = j + len(q);  continue
        i += 1


SCANNERS = {'.rs': scan_rust, '.ts': scan_clike, '.tsx': scan_clike,
            '.js': scan_clike, '.mjs': scan_clike, '.py': scan_python}

# ---------------------------------------------------------------- giai ma


def try_decode(tok):
    if len(tok) < 6:
        return None
    try:
        raw = base64.b64decode(tok + '=' * (-len(tok) % 4), validate=True)
        s = raw.decode('utf-8')
    except Exception:
        return None
    if not s.strip() or not VIET.search(s):
        return None
    if any(ord(c) < 9 or 13 < ord(c) < 32 for c in s):
        return None
    # Chi CHU CAI phi-ASCII moi phai la chu Viet. Dau cau va ky hieu
    # (em dash, ellipsis, mui ten, emoji) deu hop le trong van ban that —
    # loai chung se bo sot hang tram chuoi dung.
    for c in s:
        if ord(c) < 128 or c in _VN_OK:
            continue
        if unicodedata.category(c).startswith('L'):
            return None
    return s


# Khoa object TS dang base64 TRAN (khong dau nhay). Bo ma hoa thay
# `"Ma Ho So":` bang `TcOjIEjhu5MgU8ah:` — base64 khong co dau '=' la mot dinh
# danh JS hop le nen no bo luon dau nhay. Khoa nay KHONG nam trong chuoi nen bo
# quet literal khong voi toi. Chi xay ra o TS/TSX.
BARE_KEY = re.compile(r"""(?<![A-Za-z0-9_$."'])([A-Za-z][A-Za-z0-9]{5,})(\s*:)""")


def decode_bare_keys(src):
    """-> (src_moi, so_cho_thay).

    Van ban giai ra co dau cach nen PHAI dat lai trong dau nhay — khong the giu
    dang khoa tran.
    """
    n = 0

    def sub(m):
        nonlocal n
        dec = try_decode(m.group(1))
        if dec is None:
            return m.group(0)
        n += 1
        quoted = dec.replace('\\', '\\\\').replace('"', '\\"')
        return '"' + quoted + '"' + m.group(2)

    return BARE_KEY.sub(sub, src), n


def esc(s, quote):
    s = s.replace('\\', '\\\\').replace(quote, '\\' + quote)
    return s.replace('\n', '\\n').replace('\r', '\\r').replace('\t', '\\t')


def raw_wrap(text):
    """boc text thanh raw string Rust, tu chon so dau # cho du"""
    h = 1
    while ('"' + '#' * h) in text:
        h += 1
    return 'r' + '#' * h + '"' + text + '"' + '#' * h


def read_source(path, rel):
    if FROM_WORKTREE:
        return io.open(path, encoding='utf-8', errors='strict').read()
    return subprocess.run(['git', 'show', f'{REF}:{rel}'], cwd=ROOT,
                          capture_output=True).stdout.decode('utf-8')


stats, changed, samples = Counter(), [], []

for pat in TARGETS:
    for path in glob.glob(os.path.join(ROOT, pat), recursive=True):
        rel = os.path.relpath(path, ROOT).replace('\\', '/')
        ext = os.path.splitext(path)[1]
        scan = SCANNERS.get(ext)
        if not scan:
            continue
        src = read_source(path, rel)
        if not src:
            continue

        out, last, hits = [], 0, 0
        for ls, le, bs, be, is_raw in scan(src):
            body = src[bs:be]
            if not B64_RUN.search(body):
                continue
            quote = src[ls:bs] and src[bs - 1] or '"'

            # ca than literal la mot doan base64 -> co the nang cap raw string
            whole = try_decode(body) if B64_RUN.fullmatch(body) else None
            if whole is not None and not is_raw and ext == '.rs' and '\n' in whole:
                new_lit = raw_wrap(whole)
                hits += 1
                if len(samples) < 5:
                    samples.append((rel, whole[:60].replace('\n', ' / ')))
            else:
                pieces, pos, k = [], 0, 0
                for m in B64_RUN.finditer(body):
                    dec = try_decode(m.group(0))
                    if dec is None:
                        continue
                    pieces.append(body[pos:m.start()])
                    pieces.append(dec if is_raw else esc(dec, quote))
                    pos = m.end();  k += 1
                    if len(samples) < 5:
                        samples.append((rel, dec[:60].replace('\n', ' / ')))
                if not k:
                    continue
                pieces.append(body[pos:])
                new_lit = src[ls:bs] + ''.join(pieces) + src[be:le]
                hits += k

            out.append(src[last:ls]);  out.append(new_lit);  last = le

        out.append(src[last:])
        new_src = ''.join(out)
        # Luot 2, chi TS/TSX: khoa object dang base64 tran.
        if ext in ('.ts', '.tsx'):
            new_src, k = decode_bare_keys(new_src)
            hits += k
        if hits:
            stats['chuoi'] += hits;  stats['file'] += 1
            changed.append((rel, hits))
            if APPLY:
                open(path, 'w', encoding='utf-8', newline='').write(new_src)

_src = 'cay lam viec' if FROM_WORKTREE else f'commit {REF}'
print(f"{'GHI DE' if APPLY else 'THU (them --apply de ghi)'}   nguon doc tu {_src}")
print(f"chuoi giai ma : {stats['chuoi']}")
print(f"file thay doi : {stats['file']}\n")
for rel, n in sorted(changed, key=lambda x: -x[1])[:10]:
    print(f"  {n:5d}  {rel}")
print()
for rel, s in samples:
    try:
        print(f"  {rel.split('/')[-1]:22s} {s}")
    except UnicodeEncodeError:
        print(f"  {rel.split('/')[-1]:22s} {s.encode('ascii', 'replace').decode()}")
