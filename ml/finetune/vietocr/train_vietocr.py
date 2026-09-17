#!/usr/bin/env python3
"""Fine-tune VietOCR (vgg_transformer) trên cặp <ảnh dòng>\t<text>.

Hai điều cố ý khác bản cũ:
- Cấu hình đọc từ FILE ocr/vietocr_config/vgg_transformer.yml — đúng bản đang
  phục vụ ở :5002, nên vocab TRÙNG vocab lúc suy luận. Bản cũ gọi
  Cfg.load_config_from_name → tải từ vocr.vn (chứng chỉ TLS hết hạn) và có thể
  lệch vocab với bản phục vụ: huấn luyện ra ký tự mà model triển khai không phát
  được.
- Trọng số nền lấy từ FILE (--weights, mặc định ocr/models/vietocr/
  vgg_transformer.pth). Bản cũ để Trainer(pretrained=True) tự tải, bỏ qua trọng
  số cục bộ.

Chạy trong venv riêng (requirements-train.txt) — image :5002 cài vietocr
--no-deps nên thiếu albumentations/lmdb cần cho huấn luyện.

    python train_vietocr.py --train data/rec_vi/train.txt --val data/rec_vi/val.txt \
        --train-root data/rec_vi --out out/vgg_transformer_ft_v1 --iters 20000

Sau khi xong: chép out/vgg_transformer_ft_v1.pth vào ocr/models/vietocr/ trên máy
GPU, chạy container thứ hai với OCR_REC_WEIGHTS trỏ vào đó, rồi run_bench.py +
gate.py trước khi đổi OCR_REC_WEIGHTS của dịch vụ chính.
"""
from __future__ import annotations

import argparse
import os
import sys

REPO_ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
DEFAULT_CONFIG = os.path.join(REPO_ROOT, "ocr", "vietocr_config", "vgg_transformer.yml")
DEFAULT_WEIGHTS = os.path.join(REPO_ROOT, "ocr", "models", "vietocr", "vgg_transformer.pth")


def build_config(args) -> dict:
    from vietocr.tool.config import Cfg

    if not os.path.exists(args.config):
        raise SystemExit(f"❌ thiếu cấu hình VietOCR: {args.config}")
    if not os.path.exists(args.weights):
        raise SystemExit(f"❌ thiếu trọng số nền: {args.weights}")
    cfg = Cfg.load_config_from_file(args.config)
    # Trọng số nạp tường minh bằng trainer.load_weights() sau khi dựng (xem main),
    # không qua cơ chế pretrain/download của VietOCR — tránh mọi đường ra mạng.
    cfg["pretrain"] = args.weights
    cfg["weights"] = args.weights
    cfg["quiet"] = False
    cfg["dataset"]["data_root"] = args.train_root
    cfg["dataset"]["train_annotation"] = os.path.relpath(args.train, args.train_root)
    cfg["dataset"]["valid_annotation"] = os.path.relpath(args.val, args.train_root)
    cfg["trainer"]["batch_size"] = args.batch
    cfg["trainer"]["iters"] = args.iters
    cfg["trainer"]["valid_every"] = args.val_every
    cfg["trainer"]["print_every"] = min(200, args.val_every)
    cfg["trainer"]["export"] = f"{args.out}.pth"
    cfg["trainer"]["checkpoint"] = f"{args.out}.ckpt"
    cfg["trainer"]["log"] = f"{args.out}.log"
    cfg.setdefault("optimizer", {})["max_lr"] = args.lr
    cfg["device"] = args.device
    return cfg


def check_vocab(cfg: dict, train_path: str) -> None:
    """Ký tự trong nhãn mà vocab phục vụ không có → model không bao giờ phát
    được, huấn luyện chỉ tốn công. Báo trước, không âm thầm bỏ."""
    vocab = set(cfg.get("vocab", ""))
    missing: dict[str, int] = {}
    with open(train_path, encoding="utf-8") as f:
        for line in f:
            if "\t" not in line:
                continue
            text = line.rstrip("\n").split("\t", 1)[1]
            for ch in text:
                if ch not in vocab:
                    missing[ch] = missing.get(ch, 0) + 1
    if missing:
        top = ", ".join(f"{c!r}×{n}" for c, n in sorted(missing.items(), key=lambda x: -x[1])[:15])
        print(f"⚠  {sum(missing.values())} ký tự trong nhãn KHÔNG có trong vocab phục vụ: {top}",
              file=sys.stderr)
        print("   Lọc nhãn (ml/finetune/kie/filter_rules.py) hoặc chấp nhận model bỏ qua chúng.",
              file=sys.stderr)


def main(argv=None) -> int:
    p = argparse.ArgumentParser(description="Fine-tune VietOCR từ config + trọng số đang phục vụ")
    p.add_argument("--train", required=True, help="train.txt (<ảnh>\\t<text>)")
    p.add_argument("--val", required=True, help="val.txt")
    p.add_argument("--train-root", default=".", help="thư mục gốc chứa ảnh")
    p.add_argument("--out", required=True, help="tiền tố file đầu ra (.pth/.ckpt/.log)")
    p.add_argument("--config", default=DEFAULT_CONFIG, help="yml VietOCR đang phục vụ")
    p.add_argument("--weights", default=DEFAULT_WEIGHTS, help="trọng số nền (.pth cục bộ)")
    p.add_argument("--iters", type=int, default=20000)
    p.add_argument("--batch", type=int, default=32)
    p.add_argument("--lr", type=float, default=1e-4, help="max_lr; fine-tune thấp hơn 3e-4 mặc định")
    p.add_argument("--val-every", type=int, default=1000)
    p.add_argument("--device", default="cuda:0")
    args = p.parse_args(argv)

    cfg = build_config(args)
    check_vocab(cfg, args.train)

    from vietocr.model.trainer import Trainer

    trainer = Trainer(cfg, pretrained=False)
    trainer.load_weights(args.weights)
    print(f"→ nạp trọng số nền {args.weights}; {args.iters} iters, batch {args.batch}, lr {args.lr}",
          flush=True)
    trainer.train()
    trainer.save_weights(f"{args.out}.pth")
    print(f"✅ VietOCR fine-tuned → {args.out}.pth")
    print("   Tiếp: đặt OCR_REC_WEIGHTS cho container thử (:5003), run_bench.py --endpoint, gate.py")
    return 0


if __name__ == "__main__":
    sys.exit(main())
