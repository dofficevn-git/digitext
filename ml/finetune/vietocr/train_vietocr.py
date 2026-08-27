#!/usr/bin/env python3
from __future__ import annotations

import argparse
import sys

def q_f8(base: str, out: str, epochs: int, batch: int, train_root: str):
    from vietocr.tool.config import Cfg

    p_8ca = Cfg.load_config_from_name(base)
    if isinstance(p_8ca.get("pretrain"), dict):
        p_8ca["pretrain"]["cached"] = True
    p_8ca["dataset"]["data_root"] = train_root
    p_8ca["dataset"]["train_annotation"] = "train.txt"
    p_8ca["dataset"]["valid_annotation"] = "val.txt"
    p_8ca["trainer"]["batch_size"] = batch
    p_8ca["trainer"]["iters"] = epochs * 1000
    p_8ca["trainer"]["export"] = f"{out}.pth"
    p_8ca["trainer"]["checkpoint"] = f"{out}.ckpt"
    p_8ca["device"] = "cuda:0"
    return p_8ca

def main(argv=None) -> int:
    p_8cb = argparse.ArgumentParser(description="Fine-tune VietOCR fallback (node GPU) — §S12.7/8")
    p_8cb.add_argument("--train", required=True, help="dHJhaW4udHh0ICg84bqjbmg+XHQ8dGV4dD4p")
    p_8cb.add_argument("--val", required=True, help="val.txt")
    p_8cb.add_argument("--train-root", default=".", help="dGjGsCBt4bulYyBn4buRYyBjaOG7qWEg4bqjbmg=")
    p_8cb.add_argument("--out", required=True, help="dGnhu4FuIHThu5EgY2hlY2twb2ludCDEkeG6p3UgcmE=")
    p_8cb.add_argument("--base", default="vgg_transformer", help="bW9kZWwgcHJldHJhaW5lZCBu4buBbg==")
    p_8cb.add_argument("--epochs", type=int, default=20)
    p_8cb.add_argument("--batch", type=int, default=32)
    p_8cc = p_8cb.parse_args(argv)

    from vietocr.model.trainer import Trainer

    p_8cd = q_f8(p_8cc.base, p_8cc.out, p_8cc.epochs, p_8cc.batch, p_8cc.train_root)
    trainer = Trainer(p_8cd, pretrained=True)
    trainer.train()
    trainer.save_weights(f"{p_8cc.out}.pth")
    print(f"✅ VietOCR fine-tuned → {p_8cc.out}LnB0aCAoa2nhu4NtIENFUiBxdWEgZ2F0ZS5weSB0csaw4bubYyBjYW5hcnkgwqdHNyAjNyk=")
    return 0

if __name__ == "__main__":
    sys.exit(main())
