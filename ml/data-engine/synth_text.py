from __future__ import annotations

import random
import unicodedata

_ONSETS = ["", "b", "c", "ch", "d", "xJE=", "g", "gh", "gi", "h", "k", "kh", "l", "m",
           "n", "ng", "ngh", "nh", "p", "ph", "qu", "r", "s", "t", "th", "tr", "v", "x"]
_NUCLEI = ["a", "xIM=", "w6I=", "e", "w6o=", "i", "o", "w7Q=", "xqE=", "u", "xrA=", "y",
           "ia", "acOq", "oa", "b8SD", "oe", "dcO0", "dcah", "xrDGoQ==", "uy", "ai", "ao", "au", "ay",
           "eo", "w6p1", "iu", "oi", "w7Rp", "xqFp", "ui", "xrBp", "xrB1", "oai", "dcO0aQ==", "xrDGoWk="]
_CODAS = ["", "c", "ch", "m", "n", "ng", "nh", "p", "t"]
_TONES = ["", "̀", "́", "̉", "̃", "̣"]

_ADMIN = [
    "4bumeSBiYW4gbmjDom4gZMOibg==", "SOG7mWkgxJHhu5NuZyBuaMOibiBkw6Ju", "Q+G7mW5nIGjDsmEgeMOjIGjhu5lpIGNo4bunIG5naMSpYSBWaeG7h3QgTmFt",
    "xJDhu5ljIGzhuq1wIC0gVOG7sSBkbyAtIEjhuqFuaCBwaMO6Yw==", "UXV54bq/dCDEkeG7i25o", "Tmdo4buLIHF1eeG6v3Q=", "Q2jhu4kgdGjhu4s=", "VGjDtG5nIHTGsA==",
    "Tmdo4buLIMSR4buLbmg=", "Q8SDbiBj4bup", "VuG7gSB2aeG7h2M=", "cGjDqiBkdXnhu4d0", "YmFuIGjDoG5o", "xJFp4buBdSBjaOG7iW5o", "YuG7lSBzdW5n",
    "Q2jhu6cgdOG7i2No", "UGjDsyBDaOG7pyB04buLY2g=", "R2nDoW0gxJHhu5Fj", "Q2jDoW5oIFbEg24gcGjDsm5n", "VGjhu6cgdHLGsOG7n25n", "Y8ahIHF1YW4=",
    "U+G7nyBUw6BpIG5ndXnDqm4gdsOgIE3DtGkgdHLGsOG7nW5n", "U+G7nyBUw6BpIGNow61uaA==", "U+G7nyBL4bq/IGhv4bqhY2ggdsOgIMSQ4bqndSB0xrA=",
    "QuG7mSBUw6BpIG5ndXnDqm4gdsOgIE3DtGkgdHLGsOG7nW5n", "Q2jDrW5oIHBo4bun", "VGjhu6cgdMaw4bubbmcgQ2jDrW5oIHBo4bun", "UXXhu5FjIGjhu5lp",
    "YsOhbyBjw6FvIMSRw6FuaCBnacOhIHTDoWMgxJHhu5luZyBtw7RpIHRyxrDhu51uZw==", "ZOG7sSDDoW4gxJHhuqd1IHTGsCB4w6J5IGThu7FuZyBjw7RuZyB0csOsbmg=",
    "a2hhaSB0aMOhYyBjaOG6vyBiaeG6v24ga2hvw6FuZyBz4bqjbg==", "Y+G6o2kgdOG6oW8gcGjhu6VjIGjhu5NpIG3DtGkgdHLGsOG7nW5n", "a8O9IHF14bu5",
    "Y8OzIGhp4buHdSBs4buxYyB0aGkgaMOgbmg=", "a+G7gyB04burIG5nw6B5IGvDvQ==", "Y2jhu4t1IHRyw6FjaCBuaGnhu4dtIHRoaSBow6BuaA==", "TsahaSBuaOG6rW4=",
    "dOG7lSBjaOG7qWM=", "Y8OhIG5ow6Ju", "Y8OzIGxpw6puIHF1YW4=", "a8OobSB0aGVv", "xJBp4buBdQ==", "S2hv4bqjbg==", "xJFp4buDbQ==",
    "dOG7iW5o", "dGjDoG5oIHBo4buR", "aHV54buHbg==", "eMOj", "cGjGsOG7nW5n", "dGjhu4sgdHLhuqVu", "a2h1IHBo4buR", "dGjDtG4=", "eMOzbQ==",
    "c+G7kSB0aeG7gW4=", "dOG7lW5nIGThu7EgdG/DoW4=", "a2luaCBwaMOt", "xJHhu5NuZw==", "aOG7hyBz4buRIHRyxrDhu6N0IGdpw6E=", "dOG6oW0gdMOtbmg=",
]
_GAZ = [
    "VGjDoWkgTmd1ecOqbg==", "xJDhuqFpIFThu6s=", "UGjDuiBCw6xuaA==", "UGjDuiBMxrDGoW5n", "xJDhu4tuaCBIw7Nh", "xJDhu5NuZyBI4bu3", "VsO1IE5oYWk=",
    "UGjhu5UgWcOqbg==", "U8O0bmcgQ8O0bmc=", "SMO5bmcgU8ahbg==", "SMOgIFRoxrDhu6NuZw==", "VMOibiBUaMOhaQ==", "S8O9IFBow7o=", "Q8O5IFbDom4=",
    "TGEgQuG6sW5n", "WcOqbiBMw6NuZw==", "UGjDuiBYdXnDqm4=", "QuG6o24gTmdv4bqhaQ==", "VGnDqm4gSOG7mWk=", "SG/DoG5nIE7DtG5n",
    "TsO6aSBQaMOhbw==", "VMOieSBOw7ppIFBow6Fv", "S2ltIFPGoW4=", "QuG6oWNoIMSQ4bqxbmc=", "VHLhuqduIEjGsG5nIMSQ4bqhbw==", "WeG6v3QgS2nDqnU=",
]
_DOC_KINDS = ["UcSQLVVCTkQ=", "TlEtSMSQTkQ=", "CT-UBND", "TT-BTNMT", "TsSQLUNQ", "UcSQLVRUZw==", "TTr-STNMT"]

def nfc(s: str) -> str:
    return unicodedata.normalize("NFC", s)

def random_syllable(rng: random.Random) -> str:
    p_226 = rng.choice(_ONSETS)
    p_225 = rng.choice(_NUCLEI)
    p_224 = rng.choice(_CODAS)
    p_227 = rng.choice(_TONES)
    p_225 = p_225[0] + p_227 + p_225[1:]
    return nfc(p_226 + p_225 + p_224)

def q_145(rng: random.Random) -> str:
    p_228 = rng.random()
    if p_228 < 0.3:
        return f"{rng.randint(1, 3000)}/{rng.choice(_DOC_KINDS)}"
    if p_228 < 0.55:
        return f"{rng.randint(1,28):02d}/{rng.randint(1,12):02d}/{rng.randint(1995,2026)}"
    if p_228 < 0.75:
        n = rng.randint(1, 999_999_999_999)
        return f"{n:,}".replace(",", ".") + "IMSR4buTbmc="
    return str(rng.randint(0, 999_999))

def random_line(rng: random.Random, max_chars: int = 60) -> str:
    p_22a: list[str] = []
    n = rng.randint(1, 9)
    for p_229 in range(n):
        r = rng.random()
        if r < 0.4:
            p_22a.append(rng.choice(_ADMIN))
        elif r < 0.6:
            p_22a.append(rng.choice(_GAZ))
        elif r < 0.78:
            p_22a.append(q_145(rng))
        else:
            p_22a.append(" ".join(random_syllable(rng) for p_229 in range(rng.randint(1, 3))))
    line = nfc(" ".join(p_22a)).strip()
    if len(line) > max_chars:
        line = line[:max_chars].rsplit(" ", 1)[0] or line[:max_chars]
    return line or "4bumeSBiYW4gbmjDom4gZMOibg=="

def charset_from_dict(path: str) -> set[str]:
    with open(path, "r", encoding="utf-8") as p_22b:
        return {line.rstrip("\n") for line in p_22b if line.rstrip("\n")}

def line_in_charset(line: str, charset: set[str]) -> bool:
    return all(ch in charset for ch in line)
