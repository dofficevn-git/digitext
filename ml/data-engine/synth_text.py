from __future__ import annotations

import random
import unicodedata

_ONSETS = ["", "b", "c", "ch", "d", "xJE=", "g", "gh", "gi", "h", "k", "kh", "l", "m",
           "n", "ng", "ngh", "nh", "p", "ph", "qu", "r", "s", "t", "th", "tr", "v", "x"]
_NUCLEI = ["a", "xIM=", "w6I=", "e", "w6o=", "i", "o", "w7Q=", "xqE=", "u", "xrA=", "y",
           "ia", "acOq", "oa", "b8SD", "oe", "dcO0", "dcah", "ươ", "uy", "ai", "ao", "au", "ay",
           "eo", "w6p1", "iu", "oi", "w7Rp", "xqFp", "ui", "xrBp", "xrB1", "oai", "uôi", "ươi"]
_CODAS = ["", "c", "ch", "m", "n", "ng", "nh", "p", "t"]
_TONES = ["", "̀", "́", "̉", "̃", "̣"]

_ADMIN = [
    "Ủy ban nhân dân", "Hội đồng nhân dân", "Cộng hòa xã hội chủ nghĩa Việt Nam",
    "Độc lập - Tự do - Hạnh phúc", "Quyết định", "Nghị quyết", "Chỉ thị", "Thông tư",
    "Nghị định", "Căn cứ", "Về việc", "phê duyệt", "ban hành", "điều chỉnh", "bổ sung",
    "Chủ tịch", "Phó Chủ tịch", "Giám đốc", "Chánh Văn phòng", "Thủ trưởng", "cơ quan",
    "Sở Tài nguyên và Môi trường", "Sở Tài chính", "Sở Kế hoạch và Đầu tư",
    "Bộ Tài nguyên và Môi trường", "Chính phủ", "Thủ tướng Chính phủ", "Quốc hội",
    "báo cáo đánh giá tác động môi trường", "dự án đầu tư xây dựng công trình",
    "khai thác chế biến khoáng sản", "cải tạo phục hồi môi trường", "ký quỹ",
    "có hiệu lực thi hành", "kể từ ngày ký", "chịu trách nhiệm thi hành", "Nơi nhận",
    "tổ chức", "cá nhân", "có liên quan", "kèm theo", "Điều", "Khoản", "điểm",
    "tỉnh", "thành phố", "huyện", "eMOj", "phường", "thị trấn", "khu phố", "thôn", "xóm",
    "số tiền", "tổng dự toán", "kinh phí", "đồng", "hệ số trượt giá", "tạm tính",
]
_GAZ = [
    "Thái Nguyên", "Đại Từ", "Phú Bình", "Phú Lương", "Định Hóa", "Đồng Hỷ", "Võ Nhai",
    "Phổ Yên", "Sông Công", "Hùng Sơn", "Hà Thượng", "Tân Thái", "Ký Phú", "Cù Vân",
    "La Bằng", "Yên Lãng", "Phú Xuyên", "Bản Ngoại", "Tiên Hội", "Hoàng Nông",
    "Núi Pháo", "Tây Núi Pháo", "Kim Sơn", "Bạch Đằng", "Trần Hưng Đạo", "Yết Kiêu",
]
_DOC_KINDS = ["QĐ-UBND", "NQ-HĐND", "CT-UBND", "TT-BTNMT", "NĐ-CP", "QĐ-TTg", "TTr-STNMT"]

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
        return f"{n:,}".replace(",", ".") + " đồng"
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
    return line or "Ủy ban nhân dân"

def charset_from_dict(path: str) -> set[str]:
    with open(path, "r", encoding="utf-8") as p_22b:
        return {line.rstrip("\n") for line in p_22b if line.rstrip("\n")}

def line_in_charset(line: str, charset: set[str]) -> bool:
    return all(ch in charset for ch in line)
