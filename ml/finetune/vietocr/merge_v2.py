"""Gộp dữ liệu fine-tune v2 (chạy trên Alienware, chỉ cần PIL):
  - data/rec_target/  : dòng thân văn bản của tài liệu đích, crop ĐÃ nới khung
  - data/rec_vi_ft/   : 4.119 dòng tiêu đề (bộ cũ) — crop CẮT SÁT → nới bằng viền
    lặp mép (12% cao trên/dưới, 15% cao trái/phải) cho cùng phân phối đầu vào.
→ data/rec_vi_v2/{train,val}.txt + crops_old_pad/. val CHỈ lấy từ tài liệu đích.
"""
import os, sys
from PIL import Image

R = "data"
OUT = f"{R}/rec_vi_v2"
os.makedirs(f"{OUT}/crops_old_pad", exist_ok=True)


def pad_replicate(im, py, px):
    w, h = im.size
    dy, dx = max(1, int(round(py * h))), max(1, int(round(px * h)))
    out = Image.new("RGB", (w + 2 * dx, h + 2 * dy))
    out.paste(im.resize((w + 2 * dx, h + 2 * dy)), (0, 0))  # nền tạm
    # viền lặp mép: kéo dãn hàng/cột ngoài cùng
    top = im.crop((0, 0, w, 1)).resize((w, dy)); bot = im.crop((0, h - 1, w, h)).resize((w, dy))
    out.paste(top, (dx, 0)); out.paste(bot, (dx, dy + h)); out.paste(im, (dx, dy))
    mid = out.crop((dx, 0, dx + w, h + 2 * dy))
    left = mid.crop((0, 0, 1, h + 2 * dy)).resize((dx, h + 2 * dy)); right = mid.crop((w - 1, 0, w, h + 2 * dy)).resize((dx, h + 2 * dy))
    out.paste(left, (0, 0)); out.paste(right, (dx + w, 0))
    return out


train = []
n_old = 0
for l in open(f"{R}/rec_vi_ft/train.txt", encoding="utf-8"):
    if "\t" not in l:
        continue
    rel, text = l.rstrip("\n").split("\t", 1)
    src = os.path.normpath(os.path.join(R, "rec_vi_ft", rel))
    name = os.path.basename(src)
    dst = f"{OUT}/crops_old_pad/{name}"
    if not os.path.exists(dst):
        pad_replicate(Image.open(src).convert("RGB"), 0.12, 0.15).save(dst)
    train.append(f"crops_old_pad/{name}\t{text}"); n_old += 1
n_new = 0
for l in open(f"{R}/rec_target/train.txt", encoding="utf-8"):
    if "\t" in l:
        rel, text = l.rstrip("\n").split("\t", 1)
        # lặp 2 lần: miền đích là thứ cần tối ưu, và bộ này nhỏ hơn bộ cũ
        train += [f"../rec_target/{rel}\t{text}"] * 2; n_new += 1
val = [f"../rec_target/{l.split(chr(9),1)[0]}\t{l.rstrip(chr(10)).split(chr(9),1)[1]}"
       for l in open(f"{R}/rec_target/val.txt", encoding="utf-8") if "\t" in l]
open(f"{OUT}/train.txt", "w", encoding="utf-8", newline="\n").write("\n".join(train) + "\n")
open(f"{OUT}/val.txt", "w", encoding="utf-8", newline="\n").write("\n".join(val) + "\n")
print(f"train: {len(train)} dòng ({n_old} tiêu đề cũ đã nới viền + {n_new}×2 thân văn bản đích) · val: {len(val)} (chỉ miền đích)")
