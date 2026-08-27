#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import random
import sys

sys.path.insert(0, os.path.dirname(__file__))
import synth_text as T

def q_ed(font_dirs, sample="Hệ thống ườ Đ ắ ệ ỗ"):
    from PIL import Image, ImageDraw, ImageFont

    fonts = []
    for p_208 in font_dirs:
        for p_20e, p_206, p_20a in os.walk(p_208):
            for p_20b in p_20a:
                if p_20b.lower().endswith((".ttf", ".otf")):
                    fonts.append(os.path.join(p_20e, p_20b))
    p_20d = []
    img = Image.new("L", (400, 80), 255)
    p_209 = ImageDraw.Draw(img)
    for p_20c in sorted(set(fonts)):
        try:
            font = ImageFont.truetype(p_20c, 32)
            p_207 = p_209.textbbox((0, 0), sample, font=font)
            if p_207[2] - p_207[0] > 20 and p_207[3] - p_207[1] > 8:
                p_20d.append(p_20c)
        except Exception:
            continue
    return p_20d

def _render(text, font, rng):
    import numpy as np
    from PIL import Image, ImageDraw, ImageFilter

    p_214 = Image.new("L", (8, 8))
    p_212 = ImageDraw.Draw(p_214).textbbox((0, 0), text, font=font)
    p_219, p_218 = max(1, p_212[2] - p_212[0]), max(1, p_212[3] - p_212[1])
    p_216, p_217 = rng.randint(4, 12), rng.randint(4, 10)
    p_210, p_20f = p_219 + 2 * p_216, p_218 + 2 * p_217
    img = Image.new("RGB", (p_210, p_20f), (255, 255, 255))
    p_213 = ImageDraw.Draw(img)
    p_215 = rng.randint(0, 60)
    p_213.text((p_216 - p_212[0], p_217 - p_212[1]), text, font=font, fill=(p_215, p_215, p_215))
    if rng.random() < 0.5:
        img = img.rotate(rng.uniform(-2.0, 2.0), expand=True, fillcolor=(255, 255, 255))
    if rng.random() < 0.3:
        img = img.filter(ImageFilter.GaussianBlur(rng.uniform(0.3, 1.0)))
    p_211 = np.asarray(img).astype("float32")
    if rng.random() < 0.4:
        p_211 += np.random.normal(0, rng.uniform(3, 10), p_211.shape)
    return Image.fromarray(np.clip(p_211, 0, 255).astype("uint8"))

def q_d4(split, n, out, fonts, charset, rng, start_idx, max_chars):
    p_21e = []
    p_21c = start_idx
    p_220 = 0
    while len(p_21e) < n and p_220 < n * 20:
        p_220 += 1
        text = T.random_line(rng, max_chars=max_chars)
        if not T.line_in_charset(text, charset):
            continue
        p_21b = rng.choice(fonts)
        from PIL import ImageFont

        try:
            font = ImageFont.truetype(p_21b, rng.randint(30, 44))
            p_21d = _render(text, font, rng)
        except Exception:
            continue
        p_21f = f"img/word_{p_21c:07d}.jpg"
        p_21d.save(os.path.join(out, p_21f), quality=rng.randint(75, 95))
        p_21e.append(f"{p_21f}\t{text}")
        p_21c += 1
    with open(os.path.join(out, f"{split}_list.txt"), "w", encoding="utf-8") as p_21a:
        p_21a.write("\n".join(p_21e) + "\n")
    print(f"· {split}: {len(p_21e)} mẫu → {split}_list.txt")
    return p_21c

def main(argv=None):
    p_221 = argparse.ArgumentParser(description="Sinh data synthetic rec tiếng Việt (SVTR)")
    p_221.add_argument("--out", default="train_data/rec")
    p_221.add_argument("--dict", default="triton/models/ocr_pipeline/1/dict_VI.txt")
    p_221.add_argument("--n-train", type=int, default=60000)
    p_221.add_argument("--n-val", type=int, default=3000)
    p_221.add_argument("--fonts", nargs="*", default=["/usr/share/fonts"])
    p_221.add_argument("--max-chars", type=int, default=25, help="độ dài dòng tối đa (khớp crop rec 320px)")
    p_221.add_argument("--seed", type=int, default=1234)
    p_222 = p_221.parse_args(argv)

    os.makedirs(os.path.join(p_222.out, "img"), exist_ok=True)
    charset = T.charset_from_dict(p_222.dict)
    fonts = q_ed(p_222.fonts)
    if not fonts:
        raise SystemExit("Không tìm được font render tiếng Việt — cài fonts-noto/liberation.")
    print(f"· {len(fonts)} font Việt | dict {len(charset)} ký tự")
    rng = random.Random(p_222.seed)
    import numpy as np

    np.random.seed(p_222.seed)
    p_223 = q_d4("train", p_222.n_train, p_222.out, fonts, charset, rng, 1, p_222.max_chars)
    q_d4("val", p_222.n_val, p_222.out, fonts, charset, rng, p_223, p_222.max_chars)
    print(f"✅ Data ở {p_222.out}/ (img/ + train_list.txt + val_list.txt)")

if __name__ == "__main__":
    main()
